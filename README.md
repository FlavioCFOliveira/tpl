# tpl

`tpl` is a command-line tool that reads the structure — the DDL, or schema — of a **MariaDB** database, a **MySQL** database or a **SQLite** database file and renders it through **[MiniJinja](https://docs.rs/minijinja) templates** to produce text or source code. Which of the three a database entry reads is its provider.

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh
```

The same command installs and updates `tpl`, into `/usr/local/bin` by default. It installs the latest published release; the first is v0.0.1. See [Installation](#installation).

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install-skill.sh | sh
```

This command installs or updates the Claude Code skill in your personal skills folder, `~/.claude/skills/tpl` by default. Releases ship the skill archive from v0.0.2; v0.0.1 does not. See [Claude Code skill](#claude-code-skill).

Its interaction model is modelled on `git`: a single executable, commands with subcommands, short aliases, and read commands whose output is stable enough to pipe into something else. Templates are **plain files on disk, loaded and compiled at render time**, so changing a template never requires rebuilding `tpl`.

The intended caller is an AI coding agent rather than a person at a prompt. Such a caller has three channels for understanding a command-line tool — its help text, its exit code, and what it prints — so all three are treated as contract.

---

> ## Status: implemented, released (latest v0.1.2)
>
> The table below is the whole command tree, and every command in it is written: nothing this file describes is unbuilt. Every node parses, every node has help, and `tpl help --format json` publishes the whole surface in one call. v0.0.1 was the first release of `tpl` and v0.1.2 is the latest; [`CHANGELOG.md`](CHANGELOG.md) carries the record of what it holds. The SQLite and MySQL providers ship from v0.1.0.
>
> | Command | State |
> |---|---|
> | `tpl init` | **works** |
> | `tpl help`, `tpl help <path>`, `tpl help <path> --format json`, `-h/--help` at any node, `tpl version`, `-V/--version` | **works** |
> | `tpl cfg get`, `set`, `unset`, `list` | **works** |
> | `tpl cfg database add`, `list`, `show`, `update`, `remove` | **works** |
> | `tpl cfg database test` | **works** — the one `cfg` subcommand that opens a connection |
> | `tpl schema info`, `tables`, `table`, `views`, `view`, `routines`, `routine`, `dump` | **works** |
> | `tpl cache load`, `clean`, `status` | **works** |
> | `tpl template list`, `show`, `check`, `path` | **works** |
> | `tpl render` | **works** |
>
> **The three arms are joined.** The eight `schema` subcommands and `tpl cache load` open one connection to the MariaDB or MySQL server or the SQLite database file the selected entry names, read the catalogue, and store what they read under `.tpl/.cache/`; a later read of the same entry is served from there and opens no connection at all. The four `tpl template` subcommands read `.tpl/templates/` and reach no database at all. `tpl render` composes the two: it assembles the render context from the selected database or from a `--context` document, binds at most one object, renders one template, and writes the result to stdout. When it reads the database, the connection is closed — and, for a MariaDB or MySQL server, the driver's runtime shut down — before the template starts: nothing of the read is alive while a template runs.
>
> **And they compose.** [`examples/`](examples/README.md) holds four worked examples that build an application's data layer — in Go, Rust, Python and Node.js — out of three known schemas, through the command line alone, and each one ends by submitting what it rendered to that language's own compiler.

---

## Contents

- [What `tpl` is for](#what-tpl-is-for)
- [Requirements](#requirements)
- [Installation](#installation)
- [Claude Code skill](#claude-code-skill)
- [Quick start](#quick-start)
- [Worked examples](#worked-examples)
- [The `.tpl` project](#the-tpl-project)
- [Configuration](#configuration)
- [Global flags](#global-flags)
- [Deadlines](#deadlines)
- [Failure](#failure)
- [Where the truth lives](#where-the-truth-lives)
- [Development](#development)

---

## What `tpl` is for

`tpl` is three read-only capabilities and nothing else.

| # | Arm | Command | What it does |
|---|---|---|---|
| 1 | Explore the database | `tpl schema …` | Reads and presents the structure: database characteristics, tables, columns, indexes, keys, views, routines |
| 2 | Explore the templates | `tpl template …` | Reads and presents the templates under `.tpl/templates/` |
| 3 | Render | `tpl render …` | Reads the database, reads the template, renders it, and prints the result |

**All three are written.** Two groups stand beside them: `tpl cfg …`, which maintains the project's configuration, and `tpl cache …`, which loads, cleans and reports on the catalogue cache the first and third arms read through.

Two properties bound the tool, and both are permanent.

- **Read-only over the database.** `tpl` issues no DDL, no DML and no other write statement, and no flag turns that off. The guarantee is the closed list of statements it will issue. Beside it, a MariaDB or MySQL session is set read-only on every connection, as defence in depth, and a SQLite database file is opened read-only and never created. See [`server-contract.md`](specification/server-contract.md), [`mysql-contract.md`](specification/mysql-contract.md) and [`sqlite-contract.md`](specification/sqlite-contract.md).
- **Almost nothing is written to disk.** `tpl` writes inside its own `.tpl` folder and nowhere else, with two exceptions: the destination directory `tpl init` creates, and the `<file>-wal` and `<file>-shm` files the SQLite library may create beside a SQLite database in write-ahead-log mode when it is read, and which may remain afterwards. Neither changes the database's content. See [`FR-PROJ-024`](specification/project-and-discovery.md) and [`FR-LITE-008`](specification/sqlite-contract.md). A render prints to stdout and has no output flag; where its result lands is the caller's business, through a redirection the caller writes.

---

## Requirements

| | |
|---|---|
| MariaDB | The supported series — see [`FR-SRV-001`](specification/server-contract.md) for the window and [`FR-SRV-015`](specification/server-contract.md) for the current members |
| MariaDB privileges | `SELECT` on `INFORMATION_SCHEMA` for the target database |
| MySQL | The supported series — see [`FR-MYSQL-002`](specification/mysql-contract.md) for the rule, MySQL's LTS series under Oracle Premier or Extended Support and no Innovation release, and [`FR-MYSQL-003`](specification/mysql-contract.md) for the current members |
| MySQL privileges | A reader that sees the target database's objects. One that lacks the privileges to read view definitions, routine bodies or triggers gets a short read: in a listing or a dump such views and routines are marked `restricted`, one requested by name — `tpl schema view`, `tpl schema routine`, or a `tpl render` object flag — exits `77`, and triggers it may not see are absent, as [`FR-MYSQL-041`](specification/mysql-contract.md) states |
| SQLite | Read access to the database file, and no privilege. The SQLite library is built into `tpl`, so nothing is installed on the host. A database in write-ahead-log mode also needs a directory that admits its `-wal` and `-shm` files when they are absent. See [`sqlite-contract.md`](specification/sqlite-contract.md) |
| Build | Rust, edition 2024 |
| Platform | Linux and macOS, on `aarch64` and `x86_64`. Windows is out of scope |

MySQL is read through a `mysql` entry, and through no other. A `mariadb` entry that reaches a server that is not MariaDB — a MySQL server included — is still refused rather than read, because the two diverge in `INFORMATION_SCHEMA` in ways that would make the model silently wrong; a `mysql` entry that reaches a MariaDB server is refused the same way ([`FR-MYSQL-008`](specification/mysql-contract.md), [`FR-MYSQL-012`](specification/mysql-contract.md)).

---

## Installation

Releases are published on [GitHub Releases](https://github.com/FlavioCFOliveira/tpl/releases), each by a release workflow started by hand against an already pushed `v*` tag. The first is v0.0.1. The installer below installs the latest release, and building from source remains an alternative. How releases are built and published is [`ADR-012`](docs/adr/adr-012-ci-and-release-distribution.md).

### With the install script

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh
```

The same command installs and updates. It resolves the latest release and compares its tag with the output of `tpl --version` for the `tpl` in the install directory; when they match it downloads nothing and exits `0`. It checks only that one file: a `tpl` elsewhere on `PATH` is neither consulted nor replaced. The script never installs a pre-release: "latest" is the newest release not marked as one.

The install directory is `/usr/local/bin` unless `TPL_INSTALL_DIR` names another. That variable is read by the script, not by `tpl`, which never reads it:

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | TPL_INSTALL_DIR="$HOME/.local/bin" sh
```

A missing directory is created. `sudo` is used only when the directory, or its parent when the directory must be created, is not writable.

The script supports four targets, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin` and `aarch64-apple-darwin` — Linux and macOS on `x86_64` and `arm64` — and exits non-zero on any other. Before installing, it verifies the downloaded archive against the release's `SHA256SUMS` and stops on a mismatch. **The archives are not signed**: the checksum proves an archive matches the release's `SHA256SUMS`, not who produced either.

### By hand

Each release carries one archive per target, `tpl-<tag>-<triple>.tar.gz`, holding the `tpl` binary, `README.md`, `LICENSE` and `CHANGELOG.md`, and one `SHA256SUMS` file. From v0.0.2, each release also carries the skill archive, `tpl-skill-<tag>.tar.gz`, and its `SHA256SUMS` covers all five archives; in v0.0.1, it covers the four target archives. Download the archive for your target and `SHA256SUMS`, verify, and extract:

```sh
tag=vX.Y.Z                    # the release's tag
triple=aarch64-apple-darwin   # one of the four targets
base=https://github.com/FlavioCFOliveira/tpl/releases/download/$tag
curl -fsSLO "$base/tpl-$tag-$triple.tar.gz"
curl -fsSLO "$base/SHA256SUMS"
grep " tpl-$tag-$triple.tar.gz\$" SHA256SUMS | shasum -a 256 -c -   # sha256sum -c - on Linux
tar -xzf "tpl-$tag-$triple.tar.gz" tpl
```

Then move `tpl` into a directory on your `PATH`.

### From source

```bash
git clone https://github.com/FlavioCFOliveira/tpl.git
cd tpl
cargo build --release
```

The binary is produced at `target/release/tpl`.

---

## Claude Code skill

[`skill/`](skill/README.md) is a Claude Code skill that makes Claude the sole operator of `tpl` for Claude Code agents: every `tpl` task goes through the `tpl` binary, never through a reimplementation of it. The skill needs `tpl` on `PATH`.

Install or update it with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install-skill.sh | sh
```

The script downloads `tpl-skill-<tag>.tar.gz` from the latest release, verifies it against the release's `SHA256SUMS`, and installs it into `$CLAUDE_CONFIG_DIR/skills/tpl`, where `CLAUDE_CONFIG_DIR` defaults to `~/.claude`. `TPL_SKILL_DIR` names another destination; both variables are read by the script, not by `tpl`. An existing skill at the destination is replaced; if it is a symbolic link, only the link is removed, never its target. The script never uses `sudo`. Releases ship the skill archive from v0.0.2; v0.0.1 does not.

For development, install from a clone instead. From its root, link the folder, so that `git pull` updates the skill:

```sh
mkdir -p ~/.claude/skills
ln -s "$PWD/skill" ~/.claude/skills/tpl
```

Or copy it, which gives a snapshot you must replace to update: `rm -rf ~/.claude/skills/tpl && cp -R skill ~/.claude/skills/tpl`.

Claude Code picks up changes to its skills directory in the current session. Only a skills directory created after the session started needs Claude Code to be restarted.

[`skill/README.md`](skill/README.md) describes what the skill holds, and how to run `skill/scripts/check-coverage.sh`, which checks that the skill's command map names every command and alias the installed `tpl` publishes. How the skill is released and installed is [`ADR-012`](docs/adr/adr-012-ci-and-release-distribution.md).

---

## Quick start

Every command below is implemented and behaves as shown.

```bash
# 1. Create a project in the current directory.
cd my-project
tpl init

# 2. Register a database: a MariaDB server, from discrete fields or from one
#    URL, a MySQL server, or a SQLite database file, by a path relative to
#    this directory.
tpl cfg database add shop --host db.example.com --user reader --schema shop
tpl cfg database add reporting \
    --dsn 'mysql://reader:${REPORTING_PASSWORD}@db.example.com:3306/reporting'
tpl cfg database add orders --provider mysql --host mysql.example.com --user reader --schema orders
tpl cfg database add ledger --provider sqlite --path data/ledger.sqlite

# 3. Make one of them the default for invocations that omit -d.
tpl cfg set core.database shop

# 4. Read the configuration back, with passwords redacted.
tpl cfg list
tpl cfg database list
tpl cfg database show shop

# 5. Learn the whole command surface in one call.
tpl help --format json

# 6. Read the structure of the default database. The first of these opens one
#    connection and stores what it read under .tpl/.cache/; the rest are served
#    from there and open none.
tpl schema tables
tpl schema tables --pattern 'order%' --format json
tpl schema table orders
tpl schema dump --pretty > context.json

# 7. Manage that store directly.
tpl cache status
tpl cache load
tpl cache clean

# 8. Read the templates the project carries.
tpl template list
tpl template show example
tpl template check

# 9. Render a template. The result goes to stdout and nowhere else, so where
#    it lands is a redirection you write. rust/struct and docs/table.md stand
#    for templates of your own: tpl init writes only example and rust/_types.
tpl render rust/struct --table orders > src/models/orders.rs
tpl render docs/table.md --table orders --set title=Orders

# 10. Render without a database, from the document step 6 wrote.
tpl render rust/struct --context context.json --table orders
tpl schema dump | tpl render rust/struct --context - --table orders
```

Steps 4, 5, 6 and 8, and `tpl cache status` in step 7, are read commands and accept `--format json` and `--pretty`, except `tpl schema dump`, which emits JSON and nothing else, and `tpl template show` and `tpl template check`, neither of which has a second representation. Steps 1 to 3, `tpl cache load` and `tpl cache clean` in step 7, and `tpl template check` print nothing on success: the exit code is the message. Steps 9 and 10 write the rendered text and nothing else — `tpl render` has no `--output`, no `--format` and no `--pretty`.

---

## Worked examples

The commands above, composed into something whole. [`examples/`](examples/README.md) holds four complete demonstrations that build an application's **data layer** out of a database schema, using the command line and nothing else — no library interface of `tpl` is reached at any point. They are the subject of [`specification/examples.md`](specification/examples.md) and of its `UC-013`.

| Example | Language | What decides it is correct |
|---|---|---|
| [`examples/go-data-layer/`](examples/go-data-layer/README.md) | Go | `go build ./...` and `go vet ./...` |
| [`examples/rust-data-layer/`](examples/rust-data-layer/README.md) | Rust | `cargo build` and `cargo clippy -- -D warnings` |
| [`examples/python-data-layer/`](examples/python-data-layer/README.md) | Python | `python -m compileall` and `mypy --strict` |
| [`examples/node-data-layer/`](examples/node-data-layer/README.md) | Node.js | `node --check` and `tsc --checkJs --noEmit` |

All four make the same five reads, through all three providers: the three MariaDB schemas `sakila`, `world` and `freight`, from one server of the most recent supported MariaDB series; the `freight` schema of the MySQL fixture, through a `mysql` entry, from one server of the most recent supported MySQL series; and the main database the SQLite fixture builds from `scripts/sqlite/freight.sql`, through a `sqlite` entry — **82 tables and 755 columns** in all. Each example renders 247 to 252 files from as many invocations of `tpl render`, because a rendered file is a redirection the caller performs. The rendered trees are committed, so an example can be read without being run.

**An example is correct when the code it renders compiles**, and not when `tpl` exited `0`: a render succeeds precisely when the template evaluated, which is a fact about the template and not about the file. Each example's compile gate is its acceptance signal, and a missing toolchain fails that gate rather than skipping it.

The type mapping is the exercise. `tpl` ships no per-language type filter and never will — a type mapping is an opinion, and an opinion belongs to the project holding it — so each example carries its own macro, covering all **44 `data_type` values** the five reads report: the 39 the three MariaDB schemas declare, of which 25 are carried by `freight` alone; MySQL's `json` and `geomcollection`; and the SQLite affinities `integer`, `real` and `numeric`, which no server read produces.

The databases are the project's own fixtures rather than databases of yours: a server of the MariaDB fixture, a server of the MySQL fixture, and the database file the SQLite fixture builds on every run. [`examples/README.md`](examples/README.md) is how to bring it up, what each example needs installed, and what exercising these templates against a real catalogue turned up.

---

## The `.tpl` project

`tpl` works on **projects**. A project is any directory containing a `.tpl` folder — the same way `git` uses `.git`.

```
my-project/
└── .tpl/
    ├── .cfg                          # TOML: this project's database entries — not versioned
    ├── .gitignore                    # two lines: .cfg and .cache/
    ├── .cache/                       # catalogue cache, created by a read — not versioned
    └── templates/                    # the project's templates — versioned
        ├── example.jinja             # a worked example
        └── rust/
            └── _types.jinja          # a column-to-Rust-type mapping, as a macro
```

`tpl init` writes five of those artefacts: `.cfg`, `.gitignore`, `templates/`, `templates/example.jinja`, and `templates/rust/_types.jinja`. It does **not** create `.cache/`, which is per-machine state a read populates.

`tpl init` takes an optional path, defaulting to the current directory, and creates any missing parent with it. It refuses a destination that already holds a `.tpl` folder, changing nothing, and exits `73`; it refuses a destination that is itself a `.tpl` folder, such as `projects/reports/.tpl`, with `64`, naming the parent directory to use instead. `--tpl-dir` has no effect on `tpl init` and draws a warning: the destination is the path operand. A project created inside another succeeds, exits `0`, and warns on stderr that it shadows the one above.

```bash
tpl init
tpl init projects/reports
```

### Discovery

`tpl` finds the project by walking **up** from the current directory to the first `.tpl` folder it meets. The walk stops at the mount point of the filesystem it started on; there is no fallback anywhere else, and no environment variable takes part. A command that needs a project and finds none exits `78` and tells you to run `tpl init` or to name one with `--tpl-dir`.

`--tpl-dir <path>` names the `.tpl` folder outright and suppresses the walk. The path must be the `.tpl` folder itself, its last segment `.tpl`: naming the directory that holds it, or any other directory, exits `78`, and the `hint` names the `.tpl` folder where there is one. It exempts nothing: the folder it names is subject to every check below.

Four entries of the tree need no project, and no project above the invocation decides what any of them does: `tpl init`; `tpl help` in its three forms; `-h/--help` at any node; and `tpl version` with `-V/--version`. `tpl init` alone looks upward, only to warn on stderr that the project it creates shadows one above it; see [`FR-PROJ-025`](specification/project-and-discovery.md). Each runs where no project exists, reads no file under `.tpl` and opens no socket — which is what makes `tpl help --format json` safe as an agent's first invocation, before it knows `tpl init` exists.

### `.tpl/.cfg` must be yours alone

The file decides which host is contacted, which credential is used and which child process is run, so `tpl` refuses to read it unless three things hold, and it checks all three, in this order, **before** reading it:

- it is a regular file — not a symbolic link, a directory, a FIFO, a socket or a device;
- it is owned by the invoking user; and
- it grants no access to group and none to other — mode `0600`, as `tpl init` creates it.

Any failure exits `78`, naming what was found — for the first, the kind of file — and nothing is read from the file. To keep the configuration outside the project, name that project's `.tpl` folder with `--tpl-dir` rather than linking `.cfg`. An **absent** `.cfg` is not a failure: the `.tpl` folder must then be owned by the invoking user, or the invocation exits `78`, and the project reads as one with an empty configuration, which `tpl cfg set` can write again.

### What to version, and what not to

| Path | Fate | Why |
|---|---|---|
| `.tpl/templates/` | **versioned** | Shared team work; it belongs in the repository and should be reviewed like any other code |
| `.tpl/.cfg` | **not versioned** | Per-machine access configuration, possibly holding credentials |
| `.tpl/.cache/` | **not versioned** | Derived per-machine state |

A leading dot does not hide a file from `git`, so `tpl init` writes `.tpl/.gitignore` with two lines:

```gitignore
.cfg
.cache/
```

Keeping the rule inside `.tpl` makes the folder self-contained: copy it into another project and the exclusion travels with it.

---

## Configuration

All configuration lives in `.tpl/.cfg`, in the project's `.tpl` folder. It is TOML. There is no global configuration: nothing in `~`, nothing under XDG, nothing in `/etc`. What `tpl` does is therefore determined by the project alone.

Precedence, strongest first: **command-line flag → `.tpl/.cfg` → built-in default**. There is no environment layer; `${VAR}` inside the file, described below, supplies the value of a key the file already carries and is not a fourth.

### The key space is closed

`.tpl/.cfg` admits a closed set of keys, under `[core]` and per `[database.<name>]` block, each with a declared type and a declared default. The table is [`FR-CONF-002`](specification/configuration-model.md) and is not copied here.

**The file is read strictly.** A key outside that space, **anywhere** in the file, exits `78` naming the key and suggesting the nearest one that exists. A value of the wrong type exits `78` naming the line and column, what was found, and what that key takes. Nothing is ignored and nothing is repaired, because a file that decides where to connect and what to run is not a file to guess at — so a `.cfg` a newer `tpl` wrote and an older one does not understand is refused whole rather than read in part.

### A database entry and its provider

Every entry has a provider, named by its `provider` key: `mariadb` — the provider of an entry without the key — `mysql`, or `sqlite`. Each provider takes only its own keys — `path` for `sqlite`, the connection keys below for `mariadb` and `mysql` alike — and the entry an invocation selects is refused with `78` when it carries a key its provider does not take ([`FR-CONF-051`](specification/configuration-model.md), [`FR-CONF-052`](specification/configuration-model.md)).

A `mariadb` or a `mysql` entry is described **either** by `dsn` **or** by the discrete fields. Declaring both in one entry is refused, and so is an invocation that would produce one.

```toml
[core]
database = "shop"          # the entry used when -d/--database is omitted

# One URL.
[database.shop]
dsn = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop"
tls = "verify-identity"

# Or discrete fields.
[database.reporting]
host             = "10.0.1.5"
port             = 3306
user             = "reader"
database         = "reporting"
tls              = "verify-ca"
ca_file          = "/etc/ssl/certs/db-ca.pem"
password_command = ["security", "find-generic-password", "-s", "tpl-reporting", "-w"]

# A MySQL server: the same keys, and the provider.
[database.orders]
provider = "mysql"
host     = "mysql.example.com"
user     = "reader"
database = "orders"

# A SQLite database file.
[database.ledger]
provider = "sqlite"
path     = "data/ledger.sqlite"
```

A DSN takes the form `scheme://[user[:password]@]host[:port]/database`, with `mysql://` and `mariadb://` accepted as equivalent; the scheme does not select the provider, which only the `provider` key does ([`FR-CONF-010`](specification/configuration-model.md)). It carries **no query parameters**: a `?` is refused whatever follows it, so a connection string copied from another tool commonly needs its parameters removed. Encryption is the `tls` key's to decide and nothing else's.

`tls` takes one of five modes — `disabled`, `preferred`, `required`, `verify-ca`, `verify-identity` — defaulting to `verify-identity`, which validates the certificate chain and the hostname. `ca_file` and `ca_path` supply trust material to the two verifying modes. See [`FR-CONF-013`](specification/configuration-model.md).

### A MySQL server

A `mysql` entry is described, resolved and connected exactly as a `mariadb` entry, and every command reads it as it reads a MariaDB server:

```bash
tpl cfg database add orders --provider mysql --host mysql.example.com --user reader --schema orders
tpl cfg database test orders
tpl -d orders schema tables
```

The catalogue is read from twelve `INFORMATION_SCHEMA` tables and nothing else, in a session set read-only and confirmed by reading it back. A server that is not MySQL, or whose series is older than the newest supported one and not supported, is refused with `78`; a series newer than every supported one is read and marked `newer_than_supported`. See [`mysql-contract.md`](specification/mysql-contract.md).

**`tpl` never requests a MySQL server's RSA public key over a session TLS does not protect** — under `tls = "disabled"`, or `preferred` against a server that offers no TLS. A login that would need that key, which MySQL's default authentication method asks for when the server holds no cached login for the account, ends with `78` and a `hint` that sets `--tls verify-identity`. A refused TLS handshake for a `mysql` entry therefore points at TLS on the server rather than at a mode that does not encrypt ([`FR-MYSQL-045`](specification/mysql-contract.md), [`FR-MYSQL-021`](specification/mysql-contract.md)).

The model a MySQL read fills is the same model, with `database.provider` `mysql`. MySQL records no level for a `CHECK` constraint and no algorithm for a view, so both are `null`; how every other property is filled is [`mysql-contract.md`](specification/mysql-contract.md).

### A SQLite database file

A `sqlite` entry names its database file in `path`, and every command reads it as it reads a server:

```bash
tpl cfg database add ledger --provider sqlite --path data/ledger.sqlite
tpl cfg database test ledger
tpl -d ledger schema tables
tpl -d ledger render example --table carrier
```

`path` is read literally: neither `${VAR}` nor a leading `~` is expanded, and a value containing `${` is refused. A relative path is resolved against the directory that holds the `.tpl` folder, never against the working directory, so every invocation inside the project reads the same file. The path must name a regular file, a symbolic link to one included; the file is opened read-only and never created.

The file is treated as untrusted input. `tpl` reads it through the SQLite library built into the binary, which cannot load an extension, and applies and reads back a fixed set of safeguards and limits before the first statement that reads the file; no flag or key relaxes any of them, and one that does not take effect exits `70`. The safeguards are [`FR-LITE-010`](specification/sqlite-contract.md) and the limits [`FR-LITE-011`](specification/sqlite-contract.md). A path that names nothing or cannot be opened exits `74`; a path that names something other than a regular file, or a file that is not a SQLite database, is damaged, or holds a schema `tpl` refuses, exits `78`; a database a writer keeps locked past `core.query_timeout` exits `69`. See [`FR-LITE-007`](specification/sqlite-contract.md).

A database in write-ahead-log mode keeps `<file>-wal` and `<file>-shm` beside it, and the library may create both on a read: the only write outside `.tpl` a read can cause. Where they are absent and the directory refuses them, the read exits `74` ([`FR-LITE-008`](specification/sqlite-contract.md)).

The model a SQLite read fills is the same model: `database.provider` is `sqlite`, the database is named `main`, its `collation` is `null`, `server` describes the SQLite library, and `routines` is always empty, because SQLite has none. SQLite has no privileges either, so a read is complete or fails. A template tells the two providers apart by `database.provider`, per [`FR-CTX-043`](specification/context-document.md); how each property is filled is [`sqlite-contract.md`](specification/sqlite-contract.md).

### Keeping a password off disk

Two mechanisms, and each is a key of a `mariadb` or a `mysql` entry; a `sqlite` entry has no password. Both take effect when the configuration is **resolved for a connection** — which is what every invocation that opens one does: `tpl cache load`, `tpl cfg database test`, and a `schema` subcommand or a `tpl render` whose context comes from the database, on a read that misses the cache or carries `--direct`. A read served from the cache expands no variable and runs no `password_command` ([`FR-CONF-056`](specification/configuration-model.md)). The commands that only read or write `.tpl/.cfg` expand no variable and run no `password_command`.

- **`${VAR}`** expands from the environment in six fields: `dsn`, `host`, `port`, `user`, `password`, and `database`. It is a single pass — an expanded value is never re-expanded — `$$` is a literal `$`, and in the file an undefined variable, an unclosed `${` or a name that is not `[A-Za-z_][A-Za-z0-9_]*` exits `78` rather than substituting nothing; on the command line the last two are refused with `64`. It is deliberately **not** expanded anywhere else, so no environment variable can weaken transport or choose the program that runs: `ca_file`, `ca_path` and `core.database` refuse a `${`, and in `password_command` it reaches the program as written.
- **`password_command`** is an argument **array**, executed directly, with no shell. Shell metacharacters are literal arguments. Its trimmed standard output is the password, read to a bounded length; its standard error goes to the null device; a non-zero exit fails the invocation. The bound and the exit codes are [`FR-CONF-031`](specification/configuration-model.md) and [`FR-CONF-033`](specification/configuration-model.md). On the command line you write it as one string and `tpl` stores the array it splits into; a string that yields no word, leaves a quote unclosed, ends in a backslash or begins with `[` is refused with `64`:

  ```bash
  tpl cfg database update reporting \
      --password-command 'security find-generic-password -s tpl-reporting -w'
  ```

There is no `--password` flag, and no `-p`. A literal password still reaches the file through `tpl cfg set` or inside a `--dsn`, and both say so in their own help: a value on a command line is visible in the process table for the life of the invocation. `tpl` warns; it does not prevent.

An empty password — `password = ""`, a DSN whose password part is empty, or a `password_command` that prints nothing — and an entry that supplies no password are presented to the server as its own clients present an empty password, so an account that has no password logs in ([`FR-CONF-055`](specification/configuration-model.md)).

### Reading the configuration back

```bash
tpl cfg get core.database                       # one key, as written
tpl cfg get database.shop.dsn --format json
tpl cfg list                                    # the whole file, literally
tpl cfg database list                           # entry names
tpl cfg database show shop                      # one entry
```

`tpl cfg list` and `tpl cfg database show` **redact**: a literal password prints as `***`, the password inside a DSN prints as `***` with the user, host, port and database left visible, and a `${VAR}` prints exactly as written — so what the variable holds never reaches stdout. Neither expands a variable, runs `password_command`, or applies a default; both show the file, not the resolved settings.

`tpl cfg get` is the one deliberate exception. It is a directed read of a key you named, and it does not redact, so a password can be fed to another command.

### Changing it

```bash
tpl cfg set core.database shop                  # one key
tpl cfg unset database.shop.port                # one field
tpl cfg unset database.shop                     # a whole block
tpl cfg database update shop --host db-staging.example.com
tpl cfg database remove staging
```

Four properties hold across every write.

- **`add` creates and `update` changes.** Neither does the other's job: `add` against a name that exists is refused and points at `update`, and `update` against a name that does not exist is refused. There is no `--force`. The exit code of each is in `tpl help cfg database add` and `tpl help cfg database update`.
- **An update touches only the fields its flags name.** The rest of the entry is left exactly as it was.
- **Comments and key order survive.** The file is rewritten through a format-preserving parser, so the commented example `tpl init` writes is still there after the first `tpl cfg set`, and a comment you wrote beside a key stays beside it.
- **A write that would break the file is refused before the file is touched.** Writing `dsn` into an entry that carries `host`, or a `password` into one that carries `password_command`, exits `64` with the file unchanged, names both keys, and hands you a command that makes the change without deleting anything you did not name — for an entry defined by `dsn`, `tpl cfg database update <name> --dsn <url>`. A write that would leave an entry carrying a key its provider does not take — `host` into a `sqlite` entry, or `provider = "mariadb"` into one that carries `path` — is refused the same way, with `64` and the file unchanged.

Removing the entry `core.database` names also clears `core.database`, silently and in the same rewrite, so the file stays coherent. Both `tpl cfg database remove shop` and `tpl cfg unset database.shop` do it; `tpl cfg unset database.shop.host` does not, because the entry survives.

No `cfg` command touches `.tpl/.cache/`. Removing an entry, or changing where it points, leaves any data cached for it in place and says so on stderr, naming `tpl -d <name> cache clean`, which removes that cache even after the entry is gone.

The rewrite is atomic: a temporary file inside `.tpl/`, created at mode `0600`, renamed over the target. An interrupted write leaves the previous file exactly as it was. No lock is taken, so two simultaneous writers leave one whole file or the other, never a mixture.

---

## Global flags

Seven flags, accepted at **every** node of the tree and in any position — before the command, between a command and its subcommand, or after the positional arguments.

| Flag | Short | What it does |
|---|---|---|
| `--database <name>` | `-d` | Which `[database.<name>]` entry this invocation uses. Absent, `core.database` applies |
| `--tpl-dir <path>` | | Use this `.tpl` folder and do not walk. The path must end in `.tpl`. No effect on `init`, `help` and `version` |
| `--timeout <seconds>` | | An overall wall-clock budget for the invocation, measured from process start. No default |
| `--verbose` | `-v` | Raise the diagnostic level. Repeatable: once, twice, three times |
| `--quiet` | `-q` | Lower it to errors only |
| `--help` | `-h` | Print the help of the node it was given at |
| `--version` | `-V` | Print the version |

Those five short forms are the whole short-flag space of the tool. **No other flag has one**, so a single letter means one thing wherever it appears.

Local flags belong to the commands that declare them. `--format <text|json>` and `--pretty` are declared by the commands that answer with a document; `--format` defaults to `text`, fixed, with no terminal detection anywhere. `--direct` and `--no-cache` are declared by the commands that read the database. The full list is [`global-flags.md`](specification/global-flags.md).

**No environment variable configures behaviour.** `${VAR}` inside `.tpl/.cfg` is the only environment read `tpl` performs. There is no colour anywhere, so there is no flag and no variable to turn it off.

Results go to stdout; diagnostics go to stderr. **stdout is byte-identical for the same invocation against the same state**; stderr is not, and is not contract — at raised verbosity it carries phase timings, which differ on every run.

---

## Deadlines

Every blocking phase has a deadline, so an invocation cannot hang with no diagnosis. Four `[core]` keys supply them: `connect_timeout`, shared by DNS resolution, the TCP connect, the TLS handshake and the login; `query_timeout`, per catalogue query and per session statement — the read-only statement, its read-back and the version probe, each with a deadline of its own; `password_timeout`, for `password_command`; and `render_timeout`, for one render. Each falls back to the built-in default declared for it in [`FR-CONF-002`](specification/configuration-model.md). For a `sqlite` entry, `query_timeout` bounds every statement on the file, a wait for a writer's lock included, and `connect_timeout` and `password_timeout` have no effect ([`FR-LITE-009`](specification/sqlite-contract.md)).

`--timeout` is separate. It is an overall budget measured from process start, it has no default, and it does not replace a phase deadline: a phase ends at the first of the two to expire. The diagnostic names which one it was and its resolved value.

**Every deadline is now reachable from a command.** The four connection phases, the three session statements and the catalogue query bound a read that misses the cache ([`FR-CONF-005`](specification/configuration-model.md)), `password_command` runs when the configuration is resolved for such a read, and `render_timeout` bounds the one render of `tpl render` — which exits `65` naming which of the two bounds expired and its resolved value.

### Render bounds

A render is bounded by three more limits besides its deadline, each set by a `[core]` key:

- **`render_fuel`** — the evaluation steps one render may execute, counted by the template engine.
- **`render_output_limit`** — the bytes one render may produce, counted as they are produced. A render stopped here writes nothing to stdout.
- **`render_memory_limit`** — the heap the process may hold while the render runs, as its allocator counts it, observed periodically. A render stopped here writes nothing further to stdout.

Each key's default and admitted range are printed by `tpl help cfg set` and fixed by [`FR-CONF-045`](specification/configuration-model.md); they are not repeated here.

Fuel and output are counts, so a template that loops or writes without end stops at the same point on every run. The memory limit is observed periodically: the process may hold more than the limit between two observations, and a single allocation the operating system refuses outright still ends the process by a signal rather than with `65`. The output is held in memory until the render ends, which is why its default sits below the memory default — endless output is reported as the output limit, not as memory.

Whichever of the four is crossed first ends the render and exits `65`, naming the bound, its resolved value, and the key that raises it. No key admits `0` or any value meaning "no bound": a value outside the range in `.tpl/.cfg` exits `78`, and `tpl cfg set` refuses it with `64`. No flag and no environment variable sets any of them, and `--timeout` does not affect them. See [`FR-RND-036` … `FR-RND-039`](specification/render-command.md) and [`FR-CONF-045`](specification/configuration-model.md).

---

## Failure

An error answers three questions, in this order and no others — what failed, why, and what to do next — followed by the exit code. All four lines go to stderr, and stdout stays empty.

```
error: no .tpl project found
cause: no .tpl folder in the working directory or in any parent of it, up to /
hint:  create a project here with: tpl init, or name an existing one with: tpl --tpl-dir <path>/.tpl <command>
exit:  78 (EX_CONFIG)
```

`hint` carries a runnable command wherever one exists, with the `--tpl-dir` and `-d` the invocation was given, and never one that deletes or overwrites what the invocation did not name. A misspelled key, entry, or command name gets a nearest-match suggestion inside it. Credentials never appear, at any verbosity.

**`--format` applies to a result and never to a failure.** No error is ever emitted as JSON, under any value of that flag: when the outcome is an error the flag is ignored and the four lines above are what a caller receives. The **exit code** is the machine-comparable signal.

Ten exit codes are used, following `sysexits.h`, and each distinct failure has its own so that the code alone decides what to do next. The table, and what the `cause` line names for each code, is [`errors-and-exit-codes.md`](specification/errors-and-exit-codes.md).

`EPIPE` on stdout — the `tpl … | head` case — exits `0` silently in the ordinary case. A pipe that closes part-way through a JSON document is an error, because the document written is not the document promised; its exit code is [`FR-ERR-025`](specification/errors-and-exit-codes.md)'s.

---

## Where the truth lives

Nothing in this file is the contract. It is the entry door.

| Question | Where it is answered |
|---|---|
| **What** `tpl` does — every command, flag, format, exit code and rule | [`specification/`](specification/README.md), starting at its `README.md` |
| **How** `tpl` is built — architecture, interfaces, data, security, operation, quality | [`docs/spec-technical/`](docs/spec-technical/README.md) |
| **Why** a binding architecture decision went the way it did | [`docs/adr/`](docs/adr/README.md) |
| How agents coordinate work in this repository | [`CLAUDE.md`](CLAUDE.md) |

Where this file and `specification/` disagree, `specification/` governs.

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

Both workflows run only when started by hand, through `workflow_dispatch`: `ci.yml` runs these five commands on the four targets against a chosen ref, and `release.yml` publishes a GitHub Release from an annotated `v*` tag, only after its gates and the same validation pass. The triggers, the gates and the release procedure are [`ADR-012`](docs/adr/adr-012-ci-and-release-distribution.md)'s.

`unsafe` is forbidden; `#![forbid(unsafe_code)]` stays at the top of the crate.

Tests that read a MariaDB server use the containers defined in `scripts/mariadb/`: one server per supported MariaDB series, seeded from `setup.sql` and `seed.sql` and presenting the fixture's own TLS certificate, and one further server offering no TLS at all. The integration tests that reach a server use them, and skip with a notice when the fixture is not up.

The fixture is driven by its own scripts, and they are the supported way to operate it: they bring the servers up and verify each one, take them down and prove nothing was left behind, answer whether the fixture is up before a server-dependent test runs, and instrument the observations that have to be made from outside the process. Use them rather than Docker commands of your own. What the directory holds, which scripts drive it, how to invoke them and how to verify the result are described in [`scripts/mariadb/README.md`](scripts/mariadb/README.md). External database instances are not used, and neither are mocks standing in for a real engine.

Tests that read a MySQL server use the containers defined in `scripts/mysql/`: one server per supported MySQL series presenting the fixture's own TLS certificate, one server per supported series offering no TLS, and one server of a series outside the supported set, which a `mysql` entry must refuse. That fixture is driven by its own scripts, on the same terms as the MariaDB one, and the tests that reach it skip with a notice when it is not up. What the directory holds, which scripts drive it and how to invoke them are described in [`scripts/mysql/README.md`](scripts/mysql/README.md).

The tests of the SQLite provider build their databases at test time from the SQL text in `scripts/sqlite/`, through the SQLite library `tpl` carries; nothing is started, and no binary database file is committed. What each file builds is described in [`scripts/sqlite/README.md`](scripts/sqlite/README.md).
