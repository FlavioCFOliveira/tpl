# Configuration, credentials and security

All configuration lives in `.tpl/.cfg` (TOML) of the project. There is no global configuration and no environment layer. Precedence is: command-line flag, then `.tpl/.cfg`, then the built-in default. The full key list, with types and defaults, is `tpl help cfg set`, and it is the authority.

## The key space (closed; an unknown key is refused)

- `[core]`: `database` (the default entry), `connect_timeout` (10), `query_timeout` (30), `password_timeout` (5), `render_timeout` (30), `render_fuel` (100000000), `render_output_limit` (67108864), `render_memory_limit` (134217728). Times are whole seconds, 1 or more. No key admits 0.
- `[database.NAME]`: `dsn`, **or** the discrete fields `host`, `port` (3306), `user`, `password`, `database`. Either shape may add `password_command`, `tls` (default `verify-identity`), `ca_file` and `ca_path`. `password` and `password_command` exclude each other.
- Entry names are 1 to 64 letters, digits or underscores.

A key outside the space, or a value of the wrong type, anywhere in the file, makes every command that reads it exit 78, naming the line. The file is read strictly and never repaired.

## Which command for which change

| Goal | Command |
|---|---|
| Create an entry | `tpl cfg database add NAME --host … --user … --schema …` or `--dsn URL` |
| Change some fields | `tpl cfg database update NAME --host …` (other fields keep their values) |
| Set one key | `tpl cfg set database.NAME.tls verify-ca`, `tpl cfg set core.query_timeout 60` |
| Remove a field or an entry | `tpl cfg unset database.NAME.port`, `tpl cfg database remove NAME` |
| Choose the default entry | `tpl cfg set core.database NAME`. `NAME` must already be declared, byte for byte, so run `tpl cfg database add NAME …` first. An undeclared name is exit 66 and nothing is written. It is a literal name, never `${VAR}` |
| Inspect safely | `tpl cfg list --format json`, `tpl cfg database show NAME --format json` (redacted) |
| Read one raw value | `tpl cfg get KEY` (**not redacted**: never use it on a password key in a transcript) |

`cfg get` on a key the file does not set exits 66, and its message states the default. `add` never overwrites an existing entry (64), and `update` never creates one (66). The `cfg` commands write atomically, keep comments and key order, and never touch the cache.

Edit the file by hand only for what no command can write: a `${VAR}` for `port` (`port = "${SHOP_PORT}"`). Afterwards, keep the mode at 0600 and run `tpl cfg list` to confirm the file still parses.

## Credentials

Order of preference:

1. **`password_command`**: `--password-command "security find-generic-password -s tpl-shop -w"`. It is given as one string, split like a shell would split it, stored as an array, and run **without** a shell. Its trimmed stdout is the password (at most 4096 bytes). A non-zero exit, a timeout (`core.password_timeout`) or too much output is exit 78. `${VAR}` is not expanded inside it.
2. **`${VAR}` reference**: `tpl cfg set database.NAME.password '${SHOP_PASSWORD}'`, or inside a DSN: `'mysql://user:${SHOP_PASSWORD}@host:3306/db'`. Use single quotes so your shell passes it literally. It is expanded only in `dsn`, `host`, `port`, `user`, `password` and `database`. `$$` is a literal `$`. Never expanded: `tls`, `ca_file`, `ca_path` (a `${` there is refused), `password_command`, and every `[core]` key.
3. **Literal password**: possible through `cfg set` or a DSN, but visible in the process list while the command runs, and stored in plain text. Don't use it unless the user explicitly asks.

Rules for the operator:

- There is no `--password` flag and no `-p`. Don't look for one.
- When running a command that needs the variable, supply it in the environment (`SHOP_PASSWORD=… tpl …`, or `export` it once). Never print it. If you don't have the secret, ask the user to export it. Don't ask them to paste it into the chat.
- The variable must be defined for **every** command that uses the entry, cache hits included. An undefined variable exits 78, and the hint names the variable.
- `tpl` never prints a credential in an error, at any verbosity. `cfg list` and `cfg database show` print a literal password as `***` and a `${VAR}` as written.
- A DSN takes no query parameters: a `?` is refused. Strip parameters from strings copied from other tools. `mysql://` and `mariadb://` are equivalent.

## TLS

| `tls` | Meaning |
|---|---|
| `disabled` | No TLS |
| `preferred` | TLS when the server offers it |
| `required` | TLS, certificate not checked |
| `verify-ca` | Certificate chain checked |
| `verify-identity` | Chain and host name checked (the default) |

`ca_file` and `ca_path` add trusted CAs for the two verifying modes. Their paths are read literally. An unreadable CA file is exit 74, and a `ca_path` holding no certificate is exit 78. For a local server with a self-signed certificate, prefer `--ca-file` with `verify-identity` over lowering the mode. Lowering TLS on a remote server is the user's decision, not yours.

## Project safety

- `.tpl/.cfg` must be a regular file, owned by the invoking user, with no group or other access (0600). Otherwise exit 78, before the file is read. A symbolic link, directory, FIFO, socket or device at `.cfg` is refused, naming the kind; to keep a project elsewhere, name its `.tpl` folder with `--tpl-dir` instead of linking `.cfg`. If you created the file by copying, fix it with `chmod 600 .tpl/.cfg`.
- A `.tpl` folder with no `.cfg` is read as an empty configuration, provided you own the folder.
- Never commit `.tpl/.cfg` or `.tpl/.cache/`. `.tpl/.gitignore` excludes both. Commit `.tpl/templates/`.

## Read-only guarantee

`tpl` runs only `SELECT` statements, most of them against `INFORMATION_SCHEMA`. It makes each session read-only first, and exits 78 if that cannot be done. No flag or key disables this. The database user needs only read access to the catalogue of the target schema. `tpl cfg database test` reports `read_only_session` and `can_read_catalogue`. A user who can log in but can't read the catalogue gets 77 on schema reads, or objects marked `restricted` in listings.

Supported servers are MariaDB only, in a window of series listed in `specification/server-contract.md` (for v0.0.1: 10.11, 11.4, 11.8 and 12.3). MySQL and servers below the window exit 78. A series newer than the window is read, and reported with `standing: newer_than_supported`.

## Cache interplay

- Changing `host`, `port`, `user`, `database`, `tls` or `dsn`, or removing or re-adding an entry, keeps its cache under `.tpl/.cache/NAME/`, and that cache keeps being served. The command warns on stderr. Run `tpl -d NAME cache clean` afterwards.
- `cache clean` also removes the cache of an entry that no longer exists (`tpl -d OLD cache clean`).
