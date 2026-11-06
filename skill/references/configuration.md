# Configuration, credentials and security

All configuration lives in `.tpl/.cfg` (TOML) of the project. There is no global configuration and no environment layer. Precedence is: command-line flag, then `.tpl/.cfg`, then the built-in default. The full key list, with types and defaults, is `tpl help cfg set`, and it is the authority.

## The key space (closed; an unknown key is refused)

- `[core]`: `database` (the default entry), `connect_timeout` (10), `query_timeout` (30), `password_timeout` (5), `render_timeout` (30), `render_fuel` (100000000), `render_output_limit` (67108864), `render_memory_limit` (134217728). Times are whole seconds, 1 or more. No key admits 0.
- `[database.NAME]`: `provider`, `mariadb` (the default, also when the key is absent), `mysql` or `sqlite`. Each provider takes only its own keys:
  - `mariadb` and `mysql` alike: `dsn`, **or** the discrete fields `host`, `port` (3306), `user`, `password`, `database`. Either shape may add `password_command`, `tls` (default `verify-identity`), `ca_file` and `ca_path`. `password` and `password_command` exclude each other.
  - `sqlite`: `path`, the database file, and nothing else.
- Entry names are 1 to 64 letters, digits or underscores.

A key outside the space, or a value of the wrong type, anywhere in the file, makes every command that reads it exit 78, naming the line. The file is read strictly and never repaired. An entry carrying a key its provider does not take (for example `path` with no `provider`) exits 78 when a command selects it; `tpl cfg database show NAME` shows what it carries.

## Which command for which change

| Goal | Command |
|---|---|
| Create an entry | `tpl cfg database add NAME --host … --user … --schema …` or `--dsn URL` |
| Register a MySQL server | The same, with `--provider mysql` |
| Register a SQLite file | `tpl cfg database add NAME --provider sqlite --path FILE` |
| Change some fields | `tpl cfg database update NAME --host …` (other fields keep their values) |
| Set one key | `tpl cfg set database.NAME.tls verify-ca`, `tpl cfg set core.query_timeout 60` |
| Remove a field or an entry | `tpl cfg unset database.NAME.port`, `tpl cfg database remove NAME` |
| Choose the default entry | `tpl cfg set core.database NAME`. `NAME` must already be declared, byte for byte, so run `tpl cfg database add NAME …` first. An undeclared name is exit 66 and nothing is written. It is a literal name, never `${VAR}` |
| Inspect safely | `tpl cfg list --format json`, `tpl cfg database show NAME --format json` (redacted) |
| Read one raw value | `tpl cfg get KEY` (**not redacted**: never use it on a password key in a transcript) |

`cfg get` on a key the file does not set exits 66, and its message states the default. `add` never overwrites an existing entry (64), and `update` never creates one (66). A write that would leave an entry carrying a key its provider does not take is refused (64): to move between a server provider and `sqlite`, add a new entry. `mariadb` and `mysql` take the same keys, so `tpl cfg set database.NAME.provider mysql` switches a server entry, and the cache warning that follows applies. The `cfg` commands write atomically, keep comments and key order, and never touch the cache.

Edit the file by hand only for what no command can write: a `${VAR}` for `port` (`port = "${SHOP_PORT}"`). Afterwards, keep the mode at 0600 and run `tpl cfg list` to confirm the file still parses.

## MySQL entries

```sh
tpl cfg database add orders --provider mysql --host mysql.example.com --user reader --schema orders
tpl cfg database test orders --format json
tpl -d orders schema tables --format json
```

- A `mysql` entry is described, resolved and connected exactly as a `mariadb` entry: the same keys, `${VAR}` rules, password sources, TLS modes and deadlines. Only `provider` tells them apart; `mysql://` or `mariadb://` in a DSN selects nothing.
- Supported: MySQL's LTS series under Oracle Premier or Extended Support, `8.4` and `9.7` in this build. A server that is not MySQL, MariaDB included, and an older series outside that set, such as `8.0`, exit 78. A newer series is read, with `standing: newer_than_supported`.
- **Keep TLS on.** Over an unencrypted session — `tls = "disabled"`, or `preferred` against a server that offers no TLS — `tpl` never requests the server's RSA public key. MySQL's default authentication method needs that key when the server holds no cached login for the account, and then the login exits 78 with `encrypt the session: tpl cfg database update NAME --tls verify-identity` in the hint. A server that offers no TLS must be given TLS first; don't lower the mode to get around it.
- The catalogue is read from twelve `INFORMATION_SCHEMA` tables, in a session made read-only and confirmed by reading `@@session.transaction_read_only` back. A reader without the privileges to see view definitions, routine bodies or triggers gets marked views and routines and no triggers, as on MariaDB.
- `cfg database test` reports the same four answers as for a `mariadb` entry.

## SQLite entries

```sh
tpl cfg database add ledger --provider sqlite --path data/ledger.sqlite
tpl cfg database test ledger --format json
tpl -d ledger schema tables --format json
```

- `path` is read literally: no `${VAR}` and no `~` is expanded, and a value holding `${` is refused (64 on the command line, 78 in the file). A relative path is resolved against the directory that holds `.tpl/`, never the current directory. Use an absolute path for a file kept elsewhere.
- The path must name a regular file; a symbolic link to one is followed. The file is opened read-only and never created, so a missing file is exit 74, never an empty database.
- The file is untrusted input. `tpl` reads it with the SQLite library built into the binary, which cannot load extensions, after setting and reading back safeguards and limits (`trusted_schema` off, defensive mode, views disabled, a length limit, no attached databases, among others). No flag or key relaxes them. A file that is not a SQLite database, is damaged, or holds a schema `tpl` refuses (for example a table or view named `pragma_…`, or a value over the length limit) is exit 78.
- `core.query_timeout` bounds every statement, including a wait for a writer's lock (69 when it expires). `connect_timeout` and `password_timeout` have no effect.
- A database in write-ahead-log mode keeps `FILE-wal` and `FILE-shm` beside it, and SQLite may create both on a read and leave them. If they are absent and the directory refuses them, the read is exit 74.
- No password, no TLS, no privileges: a SQLite read is complete or fails, and is never exit 77.
- `cfg database test` on a SQLite entry: `connected` is the read-only open, `read_only_session` the safeguards, `server` the SQLite library (`standing` is always `supported`), and `can_read_catalogue` the check of the schema, always `true` at exit 0.

## Credentials

These apply to a `mariadb` and a `mysql` entry alike. Order of preference:

1. **`password_command`**: `--password-command "security find-generic-password -s tpl-shop -w"`. It is given as one string, split like a shell would split it, stored as an array, and run **without** a shell. Its trimmed stdout is the password (at most 4096 bytes). A non-zero exit, a timeout (`core.password_timeout`) or too much output is exit 78. `${VAR}` is not expanded inside it.
2. **`${VAR}` reference**: `tpl cfg set database.NAME.password '${SHOP_PASSWORD}'`, or inside a DSN: `'mysql://user:${SHOP_PASSWORD}@host:3306/db'`. Use single quotes so your shell passes it literally. It is expanded only in `dsn`, `host`, `port`, `user`, `password` and `database`. `$$` is a literal `$`. Never expanded: `tls`, `ca_file`, `ca_path` (a `${` there is refused), `password_command`, and every `[core]` key.
3. **Literal password**: possible through `cfg set` or a DSN, but visible in the process list while the command runs, and stored in plain text. Don't use it unless the user explicitly asks.

Rules for the operator:

- There is no `--password` flag and no `-p`. Don't look for one.
- When running a command that needs the variable, supply it in the environment (`SHOP_PASSWORD=… tpl …`, or `export` it once). Never print it. If you don't have the secret, ask the user to export it. Don't ask them to paste it into the chat.
- The variable must be defined for every command that **connects** with the entry: a read that misses the cache or carries `--direct`, `tpl cache load`, `tpl cfg database test`, and a render the cache cannot serve. An undefined variable then exits 78, and the hint names the variable. A read served wholly from the cache expands no `${VAR}` and runs no `password_command`, so it succeeds with the variable unset; `--context` renders never read the entry's credentials at all.
- `tpl` never prints a credential in an error, at any verbosity. `cfg list` and `cfg database show` print a literal password as `***` and a `${VAR}` as written.
- A DSN takes no query parameters: a `?` is refused. Strip parameters from strings copied from other tools. `mysql://` and `mariadb://` are equivalent, and neither selects the provider.
- An account with no password: `password = ""`, a DSN with an empty password part, a `password_command` that prints nothing, or no password source at all. Each is sent as the server's own clients send an empty password.

## TLS

| `tls` | Meaning |
|---|---|
| `disabled` | No TLS |
| `preferred` | TLS when the server offers it |
| `required` | TLS, certificate not checked |
| `verify-ca` | Certificate chain checked |
| `verify-identity` | Chain and host name checked (the default) |

`ca_file` and `ca_path` add trusted CAs for the two verifying modes. Their paths are read literally. An unreadable CA file is exit 74, and a `ca_path` holding no certificate is exit 78. For a local server with a self-signed certificate, prefer `--ca-file` with `verify-identity` over lowering the mode. Lowering TLS on a remote server is the user's decision, not yours. Over a mode that does not encrypt, `tpl` never requests the server's RSA public key, so a login that needs it fails: 78 for a `mysql` entry (see *MySQL entries*), 69 for a `mariadb` entry, each with a `--tls verify-identity` hint. A server that accepts only encrypted sessions refuses a login under a mode that does not encrypt, and `tpl` exits 78 with a `--tls verify-identity` hint.

## Project safety

- `.tpl/.cfg` must be a regular file, owned by the invoking user, with no group or other access (0600). Otherwise exit 78, before the file is read. A symbolic link, directory, FIFO, socket or device at `.cfg` is refused, naming the kind; to keep a project elsewhere, name its `.tpl` folder with `--tpl-dir` instead of linking `.cfg`. If you created the file by copying, fix it with `chmod 600 .tpl/.cfg`.
- A `.tpl` folder with no `.cfg` is read as an empty configuration, provided you own the folder.
- Never commit `.tpl/.cfg` or `.tpl/.cache/`. `.tpl/.gitignore` excludes both. Commit `.tpl/templates/`.

## Read-only guarantee

For a `mariadb` or a `mysql` entry, `tpl` issues only a closed list of statements: the version probe, the statement that makes the session read-only and one read-back of it, then `SELECT`s against `INFORMATION_SCHEMA` (for `mysql`, twelve of its tables and no other). It exits 78 if the session cannot be made read-only. For a `sqlite` entry, it opens the file read-only, sets and reads back its safeguards, and reads the catalogue only through the schema table `sqlite_schema` and SQLite's pragma functions, inside one read transaction. No flag or key disables this. The database user needs only read access to the catalogue of the target schema; a SQLite file needs only read access to the file. `tpl cfg database test` reports `read_only_session` and `can_read_catalogue`. A user who can log in but can't read the catalogue gets 77 on schema reads, or objects marked `restricted` in listings.

A `mariadb` entry supports MariaDB only, in the window of series the binary carries, listed in `specification/server-contract.md` (in this build: 10.11, 11.4, 11.8 and 12.3); a MySQL server and a series below the window exit 78. A `mysql` entry supports MySQL only, in the series listed in `specification/mysql-contract.md` (in this build: 8.4 and 9.7); a MariaDB server and an older series outside the set exit 78. For either, a series newer than every supported one is read, and reported with `standing: newer_than_supported`. No window applies to SQLite: the library is the one built into `tpl`, and `standing` is always `supported`.

## Cache interplay

- Changing `host`, `port`, `user`, `database`, `tls`, `dsn`, `provider` or `path`, or removing or re-adding an entry, keeps its cache under `.tpl/.cache/NAME/`, and that cache keeps being served. The command warns on stderr. Run `tpl -d NAME cache clean` afterwards.
- `cache clean` also removes the cache of an entry that no longer exists (`tpl -d OLD cache clean`).
