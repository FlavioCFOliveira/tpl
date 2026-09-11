# tpl

`tpl` is a command-line tool that reads the structure — the DDL, or schema — of a **MariaDB** database and renders it through **[MiniJinja](https://docs.rs/minijinja) templates** to produce text or source code.

Its interaction model is deliberately modelled on `git`: a single executable, porcelain commands with subcommands, short aliases, long and short flags, hierarchical configuration with well-defined precedence, and plumbing commands whose output is stable enough to pipe into something else.

Templates are **plain files on disk, loaded and compiled at render time**. Changing a template never requires rebuilding `tpl`.

`tpl` is strictly read-only, in every direction. It never issues DDL, DML, or any write statement, and it enforces a read-only session at the engine level on every connection it opens. It writes no generated files either: a render is printed to stdout, and what lands on disk is up to you.

> **Status: pre-implementation.** This document describes the intended design of the tool. No command described below is implemented yet. Sections will be marked as they land.

---

## Contents

- [The three arms](#the-three-arms)
- [Built for coding agents](#built-for-coding-agents)
- [The `.tpl` project folder](#the-tpl-project-folder)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Command reference](#command-reference)
  - [Global flags](#global-flags)
  - [First arm — `schema`](#first-arm--schema)
  - [Second arm — `template`](#second-arm--template)
  - [Third arm — `render`](#third-arm--render)
  - [Project management](#project-management)
  - [`help` — the whole command tree](#help--the-whole-command-tree)
- [Configuration](#configuration)
  - [The `.cfg` file](#the-cfg-file)
  - [What to version, and what not to](#what-to-version-and-what-not-to)
- [Writing templates](#writing-templates)
  - [Render context](#render-context)
  - [Filters](#filters)
  - [Tests](#tests)
- [Rendering without a database](#rendering-without-a-database)
- [Exit codes](#exit-codes)
- [Error messages](#error-messages)
- [Development](#development)

---

## The three arms

`tpl` is three read-only capabilities and nothing else.

| # | Arm | Command | What it does |
|---|---|---|---|
| 1 | Explore the database | `tpl schema …` | Reads and presents the structure: database characteristics, tables, columns, indexes, keys, views, routines |
| 2 | Explore the templates | `tpl template …` | Reads and presents the templates available in `.tpl/templates/` |
| 3 | Render | `tpl render …` | Reads the database, reads the template, renders it, and **prints the output** |

The third arm composes the first two. Its pipeline is fixed and has no other steps: read the database, read the template, render, print.

**Every command prints to the console by default.** stdout is the destination unless you say otherwise, and that holds for all three arms.

Saving to disk is possible, but only when you ask for it explicitly — with `--output`, or `--output-dir` for a render that produces many files. Without one of those flags, `tpl` touches no file outside `.tpl`. Where output lands is therefore always visible on the command line, never an implicit side effect.

```bash
tpl -d shop render rust/struct.jinja --table orders                          # to the console
tpl -d shop render rust/struct.jinja --table orders > src/models/orders.rs   # redirected
tpl -d shop render rust/struct.jinja --table orders -o src/models/orders.rs  # written directly
```

Everything outside these three arms is project management — creating a `.tpl` folder and maintaining its `.cfg` file — and it writes nowhere but `.tpl`.

---

## Built for coding agents

`tpl` is meant to be driven by an AI coding agent — Claude Code, Codex, and the like — rather than typed by a person at a prompt. Such a caller has three channels for understanding a command-line tool: its help text, its exit code, and what it prints. All three are built to be sufficient on their own.

**Help is self-contained and complete.** Every `--help` fully describes its own level and lists its children, in the same fixed order everywhere: `USAGE`, `DESCRIPTION`, `ARGUMENTS`, `OPTIONS`, `EXAMPLES`, `EXIT CODES`, `SEE ALSO`. Types, defaults, permitted values, repeatability, and mutual exclusions are stated rather than implied. Every command's help ends with at least one complete, copyable example. Nothing points you at a website or a manual page.

**The whole CLI fits in one call.** `tpl help --format json` prints the entire command tree — commands, subcommands, aliases, arguments, flags, types, defaults, enums, and exit codes — as a single JSON document, so the tool can be learned in one invocation instead of probed.

**`tpl` is never interactive.** No confirmation prompts, no password prompts, no pager, and stdin is read only for an explicitly requested `--context`. When something is missing it fails at once and says what is missing, rather than waiting for an answer that will never come.

**Success is silent.** A successful command delivers its result and exits `0`. There is no `OK`, no summary, no elapsed time, no progress bar. A command with nothing to report prints nothing — the `0` is the message. Anything that is not the result goes to stderr.

**Failure is specific and actionable.** Every distinct failure has its own exit code, and every error says what failed, why, and what to run next. See [Exit codes](#exit-codes) and [Error messages](#error-messages).

**Output is deterministic.** The same invocation against the same database produces byte-identical output. Tables are ordered by name, columns by ordinal position, indexes by name — never in server return order. The one exception is the `now` render variable, which by definition changes between runs; templates that need reproducible output should not use it.

---

## The `.tpl` project folder

`tpl` works on **projects**. A project is any directory containing a `.tpl` folder — the same way `git` uses `.git`.

```
my-project/
└── .tpl/
    ├── .cfg              # TOML: the databases available to this project — not versioned
    ├── .gitignore        # ignores .cfg
    └── templates/        # the templates available to this project — versioned
        └── example.jinja # a working example template, written by `tpl init`
```

`tpl` locates the project by walking up from the current directory until it finds a `.tpl` folder. The first one found is the project root.

There is no global configuration. Nothing in `~`, nothing under XDG, nothing in `/etc`. A command run outside a project fails and tells you to run `tpl init`, rather than falling back to settings you cannot see. What `tpl` does is therefore fully determined by the contents of the project — which makes it reproducible on another machine and in CI.

Commands take the parameters of the operation on the command line; everything else — credentials, project defaults, template bodies — is read from `.tpl`.

---

## Requirements

| | |
|---|---|
| Database | MariaDB 12.x, 11.x or 10.x — see [`FR-SRV-001`](specification/server-contract.md) for the supported series |
| Privileges | `SELECT` on `INFORMATION_SCHEMA` for the target database |
| Build | Rust, edition 2024 |

MySQL is not a supported target. The two engines diverge in `INFORMATION_SCHEMA`, and `tpl` follows MariaDB.

---

## Installation

**From source:**

```bash
git clone <repository-url> tpl
cd tpl
cargo build --release
```

The binary is produced at `target/release/tpl`.

---

## Quick start

```bash
# 1. Create a project in the current directory
#    Writes .tpl/ with a default .cfg, a .gitignore, and templates/example.jinja
cd my-project
tpl init

# 2. Register a database
tpl database add shop --dsn 'mysql://alice@db.example.com:3306/shop'

# 3. Verify connectivity and read-only enforcement
tpl database test shop

# 4. Inspect the schema
tpl -d shop schema tables
tpl -d shop schema table orders

# 5. Render a template from .tpl/templates against it
tpl -d shop render rust/struct.jinja --table orders
```

---

## Command reference

```
tpl [global flags] <command> [<subcommand>] [<args>] [flags]
```

### Global flags

| Flag | Short | Description |
|---|---|---|
| `--database <name>` | `-d` | Which `[database.<name>]` entry to use. Overrides `TPL_DATABASE` and `core.database`. |
| `--tpl-dir <path>` | | Use this `.tpl` folder instead of discovering one. |
| `--output <path>` | `-o` | Write the result to this file instead of the console. |
| `--format <text\|json>` | | Output format for read commands. Defaults to `text` on a TTY, `json` otherwise. |
| `--verbose` | `-v` | Increase log verbosity. Repeatable. |
| `--quiet` | `-q` | Suppress everything but errors. |
| `--no-color` | | Disable colour. Implied when stdout is not a TTY, or when `NO_COLOR` is set. |
| `--version` | `-V` | Print version and exit. |
| `--help` | `-h` | Print help. Available on every subcommand. |

Environment variables: `TPL_DIR`, `TPL_DATABASE`, `NO_COLOR`.

Results go to stdout; diagnostics go to stderr. A command whose output is meant to be piped never writes anything else to stdout.

### First arm — `schema`

Read the database structure. Read-only, always.

```
tpl schema info                        Database metadata: name, version, charset, collation
tpl schema tables    [--pattern <p>]   List tables
tpl schema table     <name>            Full table metadata
tpl schema views     [--pattern <p>]   List views
tpl schema view      <name>            Full view metadata, including its SQL definition
tpl schema routines  [--pattern <p>]   List stored procedures and functions
tpl schema routine   <name>            Full routine metadata
tpl schema dump                        The entire schema as a single JSON document
```

| Canonical | Alias |
|---|---|
| `tables` | `tbls` |
| `table` | `tbl` |
| `views` | `vws` |
| `view` | `vw` |
| `routines` | `procs` |
| `routine` | `proc` |

`--pattern` filters by name using MariaDB `LIKE` syntax: `%` matches any sequence, `_` matches a single character. Case sensitivity follows the server collation.

`tpl schema dump` emits exactly the document that `render` uses as its context. See [Rendering without a database](#rendering-without-a-database).

### Second arm — `template`

Explore the templates available to the project. Read-only, always.

```
tpl template list                      List templates under .tpl/templates/
tpl template show  <name>              Print a template's source
tpl template check <name>              Parse and lint a template without rendering it
tpl template path                      Print the resolved .tpl/templates/ path
```

Templates are ordinary files. A template name is its path relative to `.tpl/templates/`, so `rust/struct.jinja` refers to `.tpl/templates/rust/struct.jinja`. `{% include %}`, `{% import %}`, and `{% extends %}` resolve within that directory and cannot reach outside it.

### Third arm — `render`

Read the database, read the template, render it, print the result to the console — or write it to disk if you ask.

```
tpl render <template> [flags]
```

| Flag | Short | Description |
|---|---|---|
| `--table <name>` | | Scope the render to one table; binds `table` in the context. Repeatable. |
| `--view <name>` | | Scope the render to one view; binds `view` in the context. Repeatable. |
| `--routine <name>` | | Scope the render to one routine; binds `routine` in the context. Repeatable. |
| `--all-tables` | | Render once per table in the database. |
| `--pattern <p>` | | Restrict the objects selected by `--all-tables` to matching names. |
| `--set <key=value>` | `-s` | Define a variable under `vars`. Repeatable. |
| `--context <path>` | | Read the context from a JSON file instead of connecting to a database. |
| `--output <path>` | `-o` | Write the result to this file instead of the console. |
| `--output-dir <dir>` | | With `--all-tables`, write one file per object into this directory. Requires `--output-name`. Mutually exclusive with `--output`. |
| `--output-name <expr>` | | Expression producing each filename under `--output-dir`, e.g. `'{{ table.name \| snake }}.rs'`. |
| `--no-clobber` | | Fail instead of overwriting a file that already exists. |
| `--dry-run` | | With an output flag, print the paths that would be written and write nothing. |

With no scoping flag, the template is rendered once with the whole database in context.

```bash
# One table, to stdout
tpl -d shop render rust/struct.jinja --table orders

# One table, redirected to a file
tpl -d shop render rust/struct.jinja --table orders > src/models/orders.rs

# Every table, concatenated into one file
tpl -d shop render rust/struct.jinja --all-tables > src/models/all.rs

# Every table, one file each
tpl -d shop render rust/struct.jinja --all-tables \
    --output-dir src/models --output-name '{{ table.name | snake }}.rs'

# Check where those files would land, without writing them
tpl -d shop render rust/struct.jinja --all-tables --dry-run \
    --output-dir src/models --output-name '{{ table.name | snake }}.rs'

# Only tables whose name ends in _log, with an extra variable
tpl -d shop render docs/table.md.jinja --all-tables --pattern '%_log' --set author=alice
```

With `--all-tables` and no output flag, the template is rendered once per table and the results go to the console back to back, in the order the tables are listed.

Writing to disk follows a few fixed rules. Files are written atomically — to a temporary file in the same directory, then renamed — so an interrupted render never leaves a truncated file behind. `--output` requires its parent directory to exist; `--output-dir` creates the directory if it is missing. Existing files are overwritten unless `--no-clobber` is given, because regenerating should be repeatable. A filename produced by `--output-name` that would escape `--output-dir` is rejected. When the result goes to a file, stdout stays empty and the exit code is still `0`.

### Project management

These commands are not one of the arms. They exist to create and maintain the `.tpl` folder, and they write nowhere else.

#### `init`

```
tpl init [<path>]      Create a ready-to-use .tpl project
```

`<path>` defaults to the current directory. `init` creates a project you can use immediately, not an empty folder:

| Path | Contents |
|---|---|
| `.tpl/.cfg` | Default configuration: a `[core]` section and one commented-out `[database.*]` entry showing the exact shape a real entry takes |
| `.tpl/.gitignore` | One line: `.cfg` |
| `.tpl/templates/` | The project's template directory |
| `.tpl/templates/example.jinja` | A working example template |

No database entry is active in the generated `.cfg` — a fresh project knows about no database until you add one, either by uncommenting the example or with `tpl database add`. The commented entry is there so the correct shape is in front of you without a trip to the documentation.

`example.jinja` renders without error against any table of any MariaDB database, and doubles as a live demonstration of the render context: it walks the columns, uses a filter, uses a test, and carries the command that runs it in its header.

```bash
tpl init
tpl database add shop --dsn 'mysql://alice@db.example.com:3306/shop'
tpl -d shop render example.jinja --table orders
```

`init` refuses to overwrite an existing `.tpl` folder, and exits `73` if it cannot create the destination.

#### `database`

Alias: `db`.

```
tpl database add    <name> [flags]   Add an entry to .tpl/.cfg
tpl database list                    List entry names
tpl database show   <name>           Print an entry, credentials redacted
tpl database update <name> [flags]   Update fields of an existing entry
tpl database remove <name>           Remove an entry
tpl database test   <name>           Connect, enforce read-only, print the server version
```

Flags for `add` and `update`:

| Flag | Short | Description |
|---|---|---|
| `--dsn` | | Full connection URL. Mutually exclusive with the discrete flags below. |
| `--host` | `-H` | Server hostname or IP. |
| `--port` | `-P` | TCP port. Defaults to `3306`. |
| `--user` | `-u` | Database user. |
| `--password` | `-p` | Password, written to `.cfg`. See [What to version, and what not to](#what-to-version-and-what-not-to). |
| `--password-command` | | Shell command whose trimmed stdout is used as the password. |
| `--database` | | Database name on the server. |
| `--tls <mode>` | | One of `disabled`, `preferred`, `required`, `verify-ca`, or `verify-identity`. Defaults to `verify-identity`, which validates the certificate chain and the hostname. See [`FR-CONF-013`](specification/configuration-model.md). |

The entry name is a label local to the project; it need not match the database name on the server.

#### `config`

```
tpl config get   <key>            Print a value from .tpl/.cfg
tpl config set   <key> <value>    Set a value
tpl config unset <key>            Remove a value
tpl config list                   Print the effective configuration, credentials redacted
```

### `help` — the whole command tree

```
tpl help                     Print the top-level help
tpl help <command>           Print help for one command
tpl help --format json       Print the entire command tree as JSON
```

`tpl help --format json` is the intended entry point for an agent. It emits every command, subcommand, alias, argument, and flag, each with its type, default, permitted values, whether it is required, and the exit codes the command can produce. The shape of that document is part of the tool's contract.

---

## Configuration

All configuration lives in `.tpl/.cfg`, in the project's `.tpl` folder. The file is TOML; its name is `.cfg` — a dotfile, leading dot, no extension — because it is meant to stay out of version control. It is created with mode `0600`.

Precedence, strongest first: **command-line flag → environment variable → `.tpl/.cfg` → built-in default**.

### The `.cfg` file

```toml
[core]
database = "shop"          # used when -d/--database is omitted

[database.shop]
dsn = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop"

[database.reporting]
host     = "10.0.1.5"
port     = 3306
user     = "reader"
database = "reporting"
password_command = "security find-generic-password -s tpl-reporting -w"
tls      = "required"
```

A `[database.<name>]` entry is defined **either** by `dsn` **or** by the discrete fields. Mixing both in one entry is a configuration error.

### What to version, and what not to

The `.tpl` folder has two halves with opposite fates, and the split is deliberate:

| Path | Fate | Why |
|---|---|---|
| `.tpl/templates/` | **Versioned** | Shared team work; it belongs in the repository and should be reviewed like any other code |
| `.tpl/.cfg` | **Not versioned** | Per-machine access configuration, possibly holding credentials |

A leading dot does not make a file invisible to `git`. To make the intent hold in practice, `tpl init` writes a `.tpl/.gitignore` containing:

```gitignore
.cfg
```

This keeps the `.tpl` folder self-contained: copy it into another project and the exclusion rule travels with it, without depending on the repository's root `.gitignore`.

Even with `.cfg` outside the repository, two mechanisms let you avoid writing a password to disk at all:

- **`${VAR}` in any string value** is replaced with the corresponding environment variable when the configuration is read. An undefined variable is an error, never a silent empty substitution.
- **`password_command`** runs the given command and uses its trimmed stdout as the password.

Commands that print configuration — `tpl database show`, `tpl config list` — always redact passwords, including the credentials inside a DSN.

---

## Writing templates

Templates use MiniJinja, which implements a large subset of Jinja2: `{{ expression }}`, `{% statement %}`, `{# comment #}`, filters, tests, macros, inheritance, and the `loop` variable.

Two behaviours differ from stock Jinja2 defaults and are worth knowing:

- **Undefined variables are an error.** A typo in a variable name fails the render with a location, rather than silently producing an empty string.
- **Auto-escaping is off** for every extension except `.html`, `.htm`, and `.xml`. The primary target is source code, not markup.

```jinja
{% for column in table.columns -%}
pub {{ column.name | snake }}: {{ column | rust_type }},
{% endfor %}
```

### Render context

| Variable | Bound when | Contents |
|---|---|---|
| `database` | always | Database metadata and the `tables`, `views`, `routines` collections |
| `table` | `--table` or `--all-tables` | The table being rendered |
| `view` | `--view` | The view being rendered |
| `routine` | `--routine` | The routine being rendered |
| `vars` | always | Values supplied with `--set` |
| `tpl` | always | `{ version }` |
| `now` | always | UTC timestamp of the render |

A `table` carries `name`, `comment`, `engine`, `charset`, `collation`, `columns`, `primary_key`, `indexes`, and `foreign_keys`. A `column` carries `name`, `position`, `data_type`, `nullable`, `default`, `comment`, and the raw catalogue attributes MariaDB reports for it.

`tpl schema dump` prints this document in full, which is the authoritative way to see what a template can reach.

### Filters

| Filter | Purpose |
|---|---|
| `pascal`, `camel`, `snake`, `upper_snake`, `kebab` | Identifier casing |
| `plural`, `singular` | English inflection |
| `quote` | Quote an identifier for MariaDB, using backticks |
| `sql_type` | The column's MariaDB type descriptor |
| `rust_type`, `go_type` | Map a column to a target-language type |
| `json` | Serialise a value as JSON |
| `indent`, `comment` | Block formatting helpers |

### Tests

`nullable`, `primary_key`, `auto_increment`, `unique`, `numeric`, `temporal`, `textual`.

```jinja
{% if column is nullable %}Option<{{ column | rust_type }}>{% else %}{{ column | rust_type }}{% endif %}
```

---

## Rendering without a database

`tpl schema dump` and `tpl render --context` are two halves of the same contract: the JSON that `dump` writes is exactly the context that `render` reads.

```bash
# On a machine with database access
tpl -d shop schema dump > shop.schema.json

# Anywhere else, offline
tpl render rust/struct.jinja --context shop.schema.json --all-tables > src/models/all.rs
```

This makes renders reproducible in CI, and lets a schema snapshot be committed and reviewed like any other artefact.

---

## Exit codes

Every distinct failure has its own code, so that the code alone is enough to decide what to do next. The convention is `sysexits.h`.

| Code | Name | Condition | What to do |
|---|---|---|---|
| `0` | `EX_OK` | Success | Carry on |
| `64` | `EX_USAGE` | Unknown command or flag, missing required argument, mutually exclusive flags | Fix the invocation; check `--help` |
| `65` | `EX_DATAERR` | Template syntax error, render failure, malformed `--context` JSON | Fix the template or the context file |
| `66` | `EX_NOINPUT` | A named object does not exist: template, table, view, routine, database entry | List what exists and pick another name |
| `69` | `EX_UNAVAILABLE` | Server unreachable: DNS, connection refused, timeout, TLS failure | Check the host and the network; the operation is read-only, so retrying is safe |
| `70` | `EX_SOFTWARE` | Internal error — a bug in `tpl` | Report it; not fixable by the caller |
| `73` | `EX_CANTCREAT` | Cannot create a destination: `tpl init` on `.tpl`, or the target of `--output` / `--output-dir` | Check permissions and that the directory exists |
| `74` | `EX_IOERR` | I/O failure reading `.tpl` or writing the result | Check permissions and free space |
| `77` | `EX_NOPERM` | Authentication refused, or insufficient privileges on `INFORMATION_SCHEMA` | Fix the credentials, or request `SELECT` |
| `78` | `EX_CONFIG` | No `.tpl` found; malformed `.cfg`; invalid entry; undefined `${VAR}`; read-only session could not be enforced | Fix `.tpl/.cfg`, or run `tpl init` |

`EPIPE` on stdout — the `tpl … | head` case — exits `0` silently. It is not an error.

---

## Error messages

An error answers three questions, in this order and no others: what failed, why, and what to do next. All of it goes to stderr.

```
error: table 'ordrs' does not exist in database 'shop'
cause: no row in INFORMATION_SCHEMA.TABLES matches table_schema='shop' and table_name='ordrs'
hint: did you mean 'orders'? list the available tables with: tpl -d shop schema tables
exit: 66 (EX_NOINPUT)
```

`hint` carries a concrete, runnable command wherever one exists. Misspelled table, view, routine, template, and database-entry names get a nearest-match suggestion. Template errors carry the template name, line, and column. Credentials never appear in an error, at any verbosity.

Under `--format json`, errors are JSON too:

```json
{"error":{"exit":66,"code":"EX_NOINPUT","kind":"table_not_found","message":"table 'ordrs' does not exist in database 'shop'","cause":"no row in INFORMATION_SCHEMA.TABLES matches table_schema='shop' and table_name='ordrs'","hint":"tpl -d shop schema tables","did_you_mean":["orders"]}}
```

`kind` is a stable enumerated identifier meant to be compared programmatically; `message` is meant to be read.

---

## Development

```bash
cargo build
cargo run -- --help
cargo test
```

Before any change is considered complete:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --all-features
cargo audit
```

Tests that need a live database use the containers defined in `scripts/mariadb/`: one server per supported MariaDB series, seeded from `setup.sql` and `seed.sql` and presenting the fixture's own TLS certificate, and one further server offering no TLS at all. What the directory holds, how to bring it up and how to verify it are described in [`scripts/mariadb/README.md`](scripts/mariadb/README.md). External database instances are not used, and neither are mocks standing in for a real engine.
