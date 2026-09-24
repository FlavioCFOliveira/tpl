# Errors and exit codes

## Reading a failure

Every failure writes exactly four lines to stderr and leaves stdout empty:

```
error: template 'exampel' does not exist
cause: no template named 'exampel' exists under the template folder /…/.tpl/templates
hint:  did you mean 'example'? list the project's templates with: tpl template list
exit:  66 (EX_NOINPUT)
```

- Branch on the process exit code. The `exit:` line repeats it.
- Errors are never JSON, whatever `--format` says. Don't try to parse stderr as JSON.
- `hint:` often holds a runnable command, built from your own invocation (it keeps your `-d` and `--tpl-dir`), and often a nearest-match name. It never suggests a command that deletes or overwrites something you did not name. It is usually the right next step, but check a suggested name before using it.
- Warnings (for example "cache kept, clear it with …") also go to stderr with exit 0. Relay them to the user and act on them.
- Checks run in a fixed order, and the first failure wins: argument parsing (64), then project discovery and trust (78), `.cfg` validation (78), template name (66), entry resolution (78 or 66), cache or connection (69, 77 or 78), object lookup (66), render (65). A 64 therefore tells you nothing about the project, and a 69 means everything before the connection was valid.

## Playbook by code

### 64 EX_USAGE: the invocation is wrong
Causes: an unknown command or flag (commands must be spelled in full; prefixes are never accepted), a missing argument, a flag given twice, conflicting flags (`--table` with `--view`, `--context` with `-d` or `--direct`, `-v` with `-q`, `--dsn` with `--host`), `--pretty` without `--format json`, a malformed value (`--timeout 0`, a bad `--set`, a bad `--tls` mode), an unknown `cfg set` key or a wrong value type, `cfg database add` on an existing name, a bare routine name shared by a procedure and a function, `tpl init X/.tpl`, and `--format` on `schema dump`.
Action: run `tpl help <path>`, fix the command, and retry. A value that starts with `-` must be written `--flag=-value`.

### 65 EX_DATAERR: a template or its input is wrong
Causes: a syntax error (line and column given), an evaluation error (undefined variable or field, a filter or test given the wrong type, a missing object passed to `table()` and similar), `fail(message)`, a malformed `--context` document, a render limit reached, or a template path that resolves outside `.tpl/templates/` (symbolic links included).
Action: `tpl template check NAME` for syntax. Read the `cause`, which names the expression and often lists the valid attributes. Fix the template and re-render. For a limit, confirm the template does not loop without end before raising the `core.render_*` key the message names.

### 66 EX_NOINPUT: a named thing does not exist
Causes: a template, table, view or routine, a database entry (`-d` or `cfg database show`, `update`, `remove`, `test`), a key that `cfg get` or `cfg unset` finds unset (the message states the default), an object to `cache clean` that the cache does not hold, or an object missing from a `--context` document.
Action: list what exists (`tpl template list`, `tpl -d NAME schema tables --format json`, `tpl cfg database list`) and retry with an exact name. Names are case-sensitive.

### 69 EX_UNAVAILABLE: the server cannot be reached
Causes: DNS failure, connection refused, a network or TLS handshake failure, or a connection step past `core.connect_timeout`, `core.query_timeout` or `--timeout`.
Action: check the host and port (`tpl cfg database show NAME`) and whether the server is up. Every operation is read-only, so retrying is safe. A failed `cache load` leaves the old cache intact. If the server stays down, work from the cache or from a `--context` dump.
Trap: a cache miss connects. A **mistyped object name** is a miss, so with the server down it reports 69 instead of 66. Check the name against a cached listing first.

### 70 EX_SOFTWARE: a bug in tpl
Action: stop. Report the exact command and `tpl version` to the user. Don't retry variants to work around it, and don't edit cache or config files to "fix" it.

### 73 EX_CANTCREAT: only from `tpl init`
Causes: `.tpl` already exists at the destination (nothing is changed), or the destination cannot be created.
Action: use the existing project, or choose another path. Check permissions.

### 74 EX_IOERR: a file or stream failed
Causes: reading or writing a file under `.tpl`, the `--context` file, `ca_file` or `ca_path`, or writing stdout. That includes a pipe closed part-way through a JSON document (`tpl … --format json | head`).
Action: check the path, permissions and free disk space. Write JSON to a file instead of piping it into a consumer that stops early. A consumer that closes a text stream early exits 0.

### 77 EX_NOPERM: the server refused
Causes: login refused (wrong user or password), or the user may not read the catalogue. That includes an object named with `schema table|view|routine`, or bound for `render`, that could only be read in part.
Action: check the user and the password source (`${VAR}` value or `password_command`) without printing the secret. Ask the user or a DBA for catalogue read access. `tpl cfg database test NAME --format json` separates a failed login (77) from missing privileges (`can_read_catalogue: false`, exit 0).

### 78 EX_CONFIG: the project or its configuration is unusable
Causes and actions:

| Cause | Action |
|---|---|
| No `.tpl` found | `tpl init`, or `--tpl-dir <path>/.tpl` |
| `--tpl-dir` does not name a `.tpl` folder | Use the path the hint gives |
| `.cfg` not owned by you, or group- or world-accessible | `chmod 600 .tpl/.cfg` (and fix the owner) |
| Malformed `.cfg`, unknown key, bad value, dsn with a `?`, `${` in `ca_file` or `ca_path` | Fix it with `cfg set`/`unset`, or edit the named line |
| No entry selected | `-d NAME` or `tpl cfg set core.database NAME` |
| Entry incomplete (no host or no server database) | `tpl cfg database update NAME --host … --schema …` |
| `${VAR}` undefined | Define it in the environment of the command, cache hits included |
| `password_command` failed, timed out or printed too much | Run the command yourself (without echoing its output) and fix it |
| Session could not be made read-only; the server is not MariaDB or its series is unsupported | Report to the user. Don't try to bypass it |

## Success that still needs attention

- `cfg database test` exit 0 with `can_read_catalogue: false`: the entry can't read the schema.
- An empty listing (`[]`, exit 0) is an answer, not an error. Confirm you selected the right entry (`-d`) and that the pattern is correct.
- `"source":"cache"`: the data may be stale. Check `tpl -d NAME cache status` and reload if needed.
- Objects carrying `restricted` in a listing or dump: some properties couldn't be read because of privileges.
- A render exit 0 only means the template evaluated. Validate the generated artefact.
