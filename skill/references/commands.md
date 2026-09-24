# Command map

Every node of the `tpl` command tree, as `tpl help --format json` publishes it for v0.0.1. This map says what exists and what each command needs. For exact flags, value types and exit codes, the binary is the authority:

```sh
tpl help <path>                   # text, e.g. tpl help cfg database add
tpl help <path> --format json     # the same, as a stable document; aliases work in <path>
```

Legend for **Needs**: **P** is a project (`.tpl/` found, or `--tpl-dir`). **E** is a selected database entry (`-d NAME` or `core.database`). **S** means it may connect to the server (only on a cache miss, unless marked *always*). **W** means it writes files under `.tpl/`.

A group node (`tpl schema`, `tpl template`, `tpl cache`, `tpl cfg`, `tpl cfg database`) given no subcommand prints its help and exits 0.

## Global flags (every node, any position)

| Flag | Short | Purpose |
|---|---|---|
| `--database NAME` | `-d` | Selects the database entry, and overrides `core.database` |
| `--tpl-dir PATH` | | Uses this `.tpl` folder, with no upward search. Ignored by `init` |
| `--timeout SECONDS` | | Overall deadline for the invocation, a positive integer |
| `--verbose` | `-v` | More diagnostics on stderr. Repeatable up to `-vvv` |
| `--quiet` | `-q` | Errors only on stderr. Not together with `-v` |
| `--help` | `-h` | Prints help for the node it is given to |
| `--version` | `-V` | Prints `tpl <version>` |

No other flag has a short form. A flag value that begins with `-` must be written as `--flag=-value`.

## Database exploration: `tpl schema`

All `schema` subcommands read through the cache and declare `--direct` (read the server and replace the cached copy) and `--no-cache` (store nothing). Every one except `dump` also declares `--format text|json` and `--pretty`.

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl schema` | | Group: reads the structure of the selected database | | |
| `tpl schema info` | | Name, charset, collation and server of the database. Text output adds object counts | P E S | `database` (without collections) |
| `tpl schema tables` | `tbls` | Lists tables. `--pattern` takes a LIKE pattern (`%`, `_`) | P E S | `tables` (full objects) |
| `tpl schema table NAME` | `tbl` | Describes one table: columns, primary key, indexes, foreign keys, referenced by, triggers, CHECKs | P E S | `table` |
| `tpl schema views` | `vws` | Lists views. `--pattern` | P E S | `views` |
| `tpl schema view NAME` | `vw` | Describes one view, with its SQL definition | P E S | `view` |
| `tpl schema routines` | `rtns` | Lists procedures and functions, each with its `kind`. `--pattern` | P E S | `routines` |
| `tpl schema routine NAME` | `rtn` | Describes one routine. `NAME` may be `procedure:NAME` or `function:NAME` | P E S | `routine` |
| `tpl schema dump` | | The whole database as one JSON document, the input for `render --context`. Always JSON, so `--format` is exit 64 | P E S | `database` (with `tables`, `views`, `routines`) |

## Template exploration: `tpl template`

These never connect and never run a template. A template name has no `.jinja` extension (`rust/struct` is `.tpl/templates/rust/struct.jinja`).

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl template` | | Group: reads the project's templates | | |
| `tpl template list` | | Every template name, sorted | P | `templates` (`[{"name":…}]`) |
| `tpl template show NAME` | | Prints the source exactly as stored. No `--format` | P | none |
| `tpl template check [NAME]...` | | Parses only, and reports every syntax error with line and column. With no `NAME`, checks all templates. Prints nothing on success | P | none |
| `tpl template path [NAME]` | | Absolute path of a template, or of `.tpl/templates/` | P | `path` |

## Rendering: `tpl render`

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl render TEMPLATE` | | Renders one template once to stdout. `--table`, `--view` or `--routine` (at most one) binds one object. `--set key=value` (repeatable) fills `vars`. `--context FILE` or `--context -` reads a `schema dump` document instead of a database. `--direct`, `--no-cache` | P, and E S unless `--context` | none (prints the rendered text only) |

`--context` cannot be combined with `-d` or `--direct` (exit 64). There is no `--output`, `--format` or `--pretty`.

## Cache: `tpl cache`

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl cache` | | Group: manages `.tpl/.cache/` | | |
| `tpl cache load` | | Reads from the server and replaces the cached copy: the whole database, or one `--table`, `--view` or `--routine`. `--no-cache` is exit 64. `--direct` has no effect | P E S (always) W | none (prints nothing) |
| `tpl cache clean` | | Deletes the entry's cached data, or one object's. Also removes the cache left behind by a removed entry | P E W | none (prints nothing) |
| `tpl cache status` | | When the cache was loaded, and per-collection counts and completeness | P E | `entry`, `loaded_at`, `collections` |

## Configuration: `tpl cfg` and `tpl cfg database`

Only `cfg database test` connects. No `cfg` command touches the cache.

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl cfg` | | Group: reads and writes `.tpl/.cfg` | | |
| `tpl cfg get KEY` | | One value, exactly as stored. **Not redacted**, and `${VAR}` is not expanded | P | `key`, `value` |
| `tpl cfg set KEY VALUE` | | Writes one key. `tpl help cfg set` lists every key, its type and its default | P W | none |
| `tpl cfg unset KEY` | | Removes a key or a whole block (`database.NAME`, `database`) | P W | none |
| `tpl cfg list` | | The whole file, passwords redacted. Text output keeps comments | P | `core`, `database` |
| `tpl cfg database` | `db` | Group: manages database entries (`tpl cfg db list` works) | | |
| `tpl cfg database add NAME` | | Creates an entry from `--dsn`, or from `--host`/`--port`/`--user`/`--schema`, plus optional `--tls`, `--password-command`, `--ca-file`, `--ca-path` | P W | none |
| `tpl cfg database list` | | Entry names | P | `entries` (`[{"name":…}]`) |
| `tpl cfg database show NAME` | | One entry, passwords redacted | P | `entry` |
| `tpl cfg database update NAME` | | Changes only the fields named by the flags given (same flags as `add`) | P W | none |
| `tpl cfg database remove NAME` | | Deletes an entry, and clears `core.database` if it named it | P W | none |
| `tpl cfg database test NAME` | | Connects and reports four answers. Exit 0 means the checks ran, not that all passed | P S (always) | `entry`, `connected`, `read_only_session`, `server`, `can_read_catalogue` |

## Project and self-description

| Command | Alias | Purpose | Needs | JSON `data` |
|---|---|---|---|---|
| `tpl init [PATH]` | | Creates `.tpl/` with `.cfg` (0600), `.gitignore` and `templates/` (`example`, `rust/_types`). Creates missing parent directories | W (the new project) | none |
| `tpl help [COMMAND_PATH]...` | | Help for any node. With `--format json`, the whole command tree, or one subtree | nothing | `tpl_version`, `global_flags`, `commands`, `template_surface`, `context_variables` |
| `tpl version` | | Prints `tpl <version>` | nothing | none |

Running `tpl` with no arguments prints the top-level help and exits 0.

## JSON conventions

- Every document is `{"schema_version":1,"source":"server|cache|project|binary","data":{…}}`, with the keys in that order. A listing's `data` holds one plural key with an array. A single object's `data` holds one singular key.
- An absent value is `null`, never omitted. An empty listing is `[]` with exit 0.
- An object that could only be read in part (because of missing privileges) carries `restricted`, an array naming the unreadable properties. A listing or dump still exits 0. Naming that object directly exits 77.
- Useful help queries (write to a file first):

  ```sh
  tpl help --format json > /tmp/tpl-help.json
  jq -r '.data.commands[].path | join(" ")' /tmp/tpl-help.json          # every command path
  tpl help render --format json | jq '.data.commands[0].exit_codes'    # one command's exit codes
  ```
