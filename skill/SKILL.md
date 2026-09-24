---
name: tpl
description: Sole operator of the tpl CLI, a read-only tool that reads a MariaDB database's structure through INFORMATION_SCHEMA and renders MiniJinja templates from it to generate code, documentation or any other text. Use this skill for every tpl invocation and every tpl task. That covers creating a project (tpl init, a .tpl folder), adding, testing, updating or removing database entries (tpl cfg, tpl cfg database), exploring tables, views and routines (tpl schema), loading or clearing the catalogue cache (tpl cache), listing, showing or checking templates (tpl template), and rendering (tpl render, --context documents, schema dump). Trigger on any mention of tpl, a .tpl project or .tpl/.cfg, a tpl command or exit code, or on a request to generate code or docs from a MariaDB schema with Jinja or MiniJinja templates, even when tpl is not named.
---

# tpl operator

`tpl` reads the structure of a MariaDB database (tables, columns, indexes, keys, views, routines) and renders MiniJinja templates with it. It works like `git`: one binary, nested subcommands, and a project folder (`.tpl/`) found by walking up from the current directory. It never writes to the database, and it writes to disk only inside `.tpl/` (plus the directory `tpl init` creates). A render prints to stdout and nowhere else.

This skill is the only way to operate `tpl`. Its main purpose is to carry what the help does not: workflows, decision rules, recovery steps and pitfalls. The binary's own help is the authority for exact flags, values and exit codes.

## Operator rules

1. **Always invoke `tpl`. Never reimplement what it does.** Don't query `INFORMATION_SCHEMA` with `mysql`, parse `.tpl/.cache/` files, or render Jinja with another engine. `tpl` applies read-only guarantees, credential handling, cache semantics and render limits that a reimplementation would silently lose.
2. **Read the exit code first, then stdout, then stderr.** The exit code is the machine signal, and each failure class has its own code (see *Exit codes* below). An error is always four text lines on stderr (`error:`, `cause:`, `hint:`, `exit:`), never JSON, even with `--format json`. The `hint:` line often holds a runnable fix.
3. **Use `--format json` for any output you parse.** Text output is laid out for people and its layout may change. JSON is a stable contract: `{"schema_version":1,"source":…,"data":{…}}`. `source` is `server`, `cache`, `project` or `binary`, and tells you whether the data is live or possibly stale. Add `--pretty` only when a person will read it (`--pretty` without `--format json` is exit 64).
4. **Don't hand-edit `.tpl/.cfg` when a `cfg` command can make the change.** The `cfg` commands validate every value, keep comments and key order, write atomically at mode 0600, and refuse a change that would break the file. The one exception: a `${VAR}` reference for `port` can only be written by editing the file.
5. **Never write to the database, and never try to.** `tpl` issues only `SELECT` statements and makes the session read-only first. No flag changes this. Don't pass DDL or DML through any channel, and don't suggest giving the database user write privileges. It only needs to read the catalogue.
6. **Keep secrets off command lines and out of output.** `tpl` has no `--password` flag. Use a `${VAR}` reference or `--password-command` (see *Credentials*). Never echo a password, and never run `tpl cfg get database.<name>.password` in a transcript: `cfg get` is the one command that does not redact.
7. **When in doubt about a flag, value or exit code, ask the binary:** `tpl help <path>` (for example `tpl help cfg database add`), or `tpl help <path> --format json` for machine reading. Aliases work in help paths. Write large JSON to a file instead of piping it into `head`: if stdout closes part-way through a JSON document, tpl exits 74 by design.

## Finding and installing the binary

```sh
command -v tpl && tpl version        # prints e.g. "tpl 0.0.1"
```

If `tpl` is missing or out of date, **ask the user before installing**. The README one-liner both installs and updates (by default to `/usr/local/bin`, and it may use `sudo`; set `TPL_INSTALL_DIR` to install somewhere else):

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | TPL_INSTALL_DIR="$HOME/.local/bin" sh
```

In a checkout of the tpl source repository, `cargo build --release` produces `target/release/tpl`.

The first call worth making in an unfamiliar setting is `tpl help --format json > /tmp/tpl-help.json`. It needs no project, opens no connection, and returns every command, flag, exit code, context variable and template filter in one document.

## Inside a project, or with `--tpl-dir`

- **Inside a project** (the working directory or a parent holds `.tpl/`), run commands directly. `tpl` walks upward to the first `.tpl` folder it finds, stopping at the filesystem's mount point.
- **From anywhere else**, add `--tpl-dir <path>/.tpl`. The path must be the `.tpl` folder itself, not the directory that holds it (that is exit 78, and the hint names the right path). It turns off the upward search.
- `tpl init [PATH]` creates a project in `PATH` (or in the current directory) and ignores `--tpl-dir`. `tpl help` and `tpl version` need no project.
- No environment variable points `tpl` at a project or changes its behaviour. The only environment it reads is `${VAR}` references inside `.tpl/.cfg`.
- `.tpl/.cfg` must be owned by you and grant no access to group or other (mode 0600). Otherwise every command that reads it exits 78. Version `.tpl/templates/`, but never `.tpl/.cfg` or `.tpl/.cache/`. The `.tpl/.gitignore` that `tpl init` writes already excludes both.

## The standard workflow

```sh
tpl init                                              # 1. create .tpl/ (exit 73 if one already exists)
tpl cfg database add shop --host db.example.com --user reader --schema shop
tpl cfg set database.shop.password '${SHOP_PASSWORD}' # 2. register the entry; the password stays in the environment
tpl cfg set core.database shop                        # 3. optional: the entry to use when -d is absent
SHOP_PASSWORD=… tpl cfg database test shop --format json   # 4. check: connected, read_only_session, server, can_read_catalogue
tpl -d shop cache load                                # 5. optional: read the whole catalogue once
tpl -d shop schema tables --format json               # 6. explore
tpl template list; tpl template check                 # 7. find templates and check their syntax
tpl -d shop render rust/struct --table orders > out.rs.tmp && mv out.rs.tmp out.rs   # 8. render
```

For the full workflows, including one file per table, offline rendering, repointing an entry and authoring templates, read `references/workflows.md`.

## Decision rules

| Question | Rule |
|---|---|
| Live server, cache or `--context`? | By default, reads go through the cache: the first read connects and stores what it read, and later reads open no connection. Use `--direct` when the schema may have changed, or run `tpl -d NAME cache load` once. Use `--direct --no-cache` for a pure read that touches no file. Use `tpl render --context FILE` (from `tpl schema dump`) for a render that needs no server, no entry and no password: CI, sharing a snapshot, or a down server. |
| Is the cache stale? | Nothing in the cache expires, and no output says it is stale. Only `tpl -d NAME cache status` shows `loaded_at`. After a schema change, run `cache load` or add `--direct`. After repointing an entry, run `tpl -d NAME cache clean`. |
| Text or JSON? | JSON for anything a program or you will parse. Text only to show a person. `schema dump` is always JSON. `render`, `template show` and `template check` have no `--format`. |
| Which global flag? | `-d NAME` picks the entry for one invocation, and overrides `core.database`. `--tpl-dir` works outside a project. `--timeout SECONDS` sets an overall deadline (a positive integer; the per-phase limits in `.cfg` still apply). `-q` shows errors only; `-v`/`-vv`/`-vvv` adds diagnostics such as phase timings. Never combine `-v` with `-q` (exit 64). Global flags work in any position. |
| One object or many? | `tpl render` renders one template once, and binds at most one object with `--table`, `--view` or `--routine`. To produce one file per table, loop over `tpl schema tables --format json` (see workflows). There is no `--all-tables` and no `--output`. |
| Writing a render to a file? | Redirect to a temporary file, then `mv`: `tpl render … > f.tmp && mv f.tmp f`. A plain `> f` empties the file before tpl runs, and a failed render prints nothing. |

## Exit codes

| Code | Meaning | First action |
|---|---|---|
| 0 | Success. An empty listing is still success. | Continue. For `cfg database test`, also check `can_read_catalogue`. |
| 64 | Usage: bad command, flag, value, conflicting flags, or unknown key for `cfg set`. | Fix the invocation. Run `tpl help <path>`. |
| 65 | Template syntax error, render failure, `fail()`, malformed `--context`, a render limit, or a template path outside `.tpl/templates/`. | Fix the template, or its input. Run `tpl template check NAME`. |
| 66 | A named thing does not exist: a table, view, routine, template, entry or key. | List what exists, then retry with a real name. |
| 69 | The server is unreachable, or a connection step timed out. | Check host, port and network. Retrying is safe. |
| 70 | A bug in tpl. | Report it with the command and `tpl version`. Don't work around it. |
| 73 | `tpl init`: `.tpl` already exists, or the destination can't be created. | Use the existing project, or pick another path. |
| 74 | I/O error on `.tpl` files, the `--context` file, CA files, or stdout. | Check paths, permissions and disk space. |
| 77 | Login refused, or the user can't read the catalogue (including a named object that could only be read in part). | Fix the credentials, or ask for catalogue read access. |
| 78 | Configuration: no project, an unsafe or invalid `.cfg`, no entry selected, an undefined `${VAR}`, `password_command` failed, the session could not be made read-only, or an unsupported server. | Fix `.tpl/.cfg` or the environment, or run `tpl init`. |

For the per-command playbook, message format, validation order and traps, read `references/errors.md`.

## Credentials and security

- Keep a password out of the file with a `${VAR}` reference: `tpl cfg set database.NAME.password '${VAR}'`, in single quotes so your shell does not expand it. Or put it inside a DSN: `--dsn 'mysql://user:${VAR}@host:3306/db'`. `tpl` reads the variable when it connects. An undefined variable is exit 78.
- Or use `--password-command "pass db/shop"`. It runs without a shell, and its trimmed stdout is the password.
- A literal value on a command line is visible in the process list. `tpl` warns about this in its help but does not prevent it. Don't do it.
- `tpl cfg list` and `tpl cfg database show` redact passwords and print `${VAR}` unexpanded, so they are safe to show. `tpl cfg get` does not redact.
- TLS defaults to `verify-identity`. A local server without a trusted certificate needs `--tls disabled` or `--tls preferred`. Say so to the user instead of weakening TLS for a remote server without asking.
- Templates can arrive through a clone. `tpl template check` only parses, so it is safe on an unread template. Renders are bounded by time, fuel, output and memory limits.

For keys, entry shapes, TLS, `${VAR}` rules and cache interplay, read `references/configuration.md`.

## Reference files

| File | Load it when |
|---|---|
| `references/commands.md` | You need to know which command exists, its aliases, what it needs (project, entry, server), or the `data` key of its JSON. Every node of the command tree is listed. |
| `references/workflows.md` | You are carrying out a multi-step task: setup, exploring a schema, one file per object, offline or CI rendering, repointing or removing an entry, or iterating on a template. |
| `references/configuration.md` | You are adding or changing entries or keys, handling credentials or TLS, or asking why `.cfg` is refused. |
| `references/templates.md` | You are writing or debugging a template: context variables, the model's field names, filters, tests, functions, includes and imports, and template traps. |
| `references/errors.md` | Any command exited non-zero, or you need the recovery action for a specific code. |

The coverage check `scripts/check-coverage.sh` confirms that `references/commands.md` names every command path the installed binary publishes. Run it after updating `tpl`.
