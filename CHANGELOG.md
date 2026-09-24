# Changelog

All notable changes to `tpl` are recorded in this file.

The format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
versioning is [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Neither
is chosen here: both were settled before this file existed, in `OD-03` of
[`docs/spec-technical/open-decisions.md`](docs/spec-technical/open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog),
which also fixes the pre-1.0 rule — **while `tpl` is below 1.0, a breaking change
is a minor bump.** The binary version has one source, the `version` field of
`Cargo.toml`; a release moves it there and nowhere else.

## Maintaining this file

A task writes its own entry during the documentation step of the project
workflow, before it closes.

- **Put the entry under `Unreleased`**, in the section that matches the change:
  `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed` or `Security`. Create the
  section if it is absent, and keep the six in that order.
- **One line per change**, naming the observable thing it affects — a command, a
  flag, an exit code, a document key, a configuration key or a registered
  template name. State what a caller can now do, or can no longer do, not how it
  was implemented.
- **Add nothing when nothing a caller can observe changed.** Work confined to
  `/specification`, `docs/`, `tests/`, `scripts/`, the knowledge graph or the
  roadmap leaves this file untouched; `git log` and the roadmap already carry it.
- **Renaming or removing a name from the registered template surface MUST be
  recorded here.** `FR-ENV-029` makes it the one entry a release may never omit,
  and it is breaking: under the pre-1.0 rule above, it moves the minor number.
  Adding a name is not breaking and is an ordinary `Added` line.
- **Nothing has been released, so nothing is yet a fix.** Until the first
  release, a defect corrected in work that never shipped is not a `Fixed` entry:
  amend the `Added` line it belongs to instead.

## Unreleased

`tpl` has never been released and no version has ever been tagged, so this
section covers **the whole of the project's history to date** — everything built
from the first commit to the present. It carries only `Added` for that reason:
with no previous release, there is nothing yet to change, deprecate, remove, fix
or secure. The first release renames this heading to `0.1.0` and dates it.

### Added

- **The command surface.** Eight subcommands — `schema`, `template`, `render`,
  `cache`, `cfg`, `init`, `help` and `version` — over a closed tree of 34 nodes,
  29 of them invocable. Short aliases on the `schema` leaves (`tbls`, `tbl`,
  `vws`, `vw`, `rtns`, `rtn`) and on `cfg database` (`db`).
- **Seven global flags**, accepted in any position: `-d`/`--database`,
  `--tpl-dir`, `--timeout`, `-v`/`--verbose`, `-q`/`--quiet`, `-h`/`--help` and
  `-V`/`--version`.
- **Help and version.** Help renders seven sections at a fixed width of eighty
  columns, for every node of the tree. `tpl help --format json` publishes the
  command tree, the global flags and the template surface as one JSON document.
  `tpl version` prints `tpl 0.1.0`.
- **Projects.** `tpl init` creates a `.tpl` project. Commands find one by walking
  up from the working directory as far as the mount point; `--tpl-dir` names the
  `.tpl` folder itself and suppresses the walk, and has no effect on `init`,
  `help` and `version`. `.tpl/.cfg` is checked for ownership and
  permissions, at the target of a symbolic link, before it is read.
- **Configuration.** `cfg get`, `cfg set`, `cfg unset` and `cfg list` read and
  write `.tpl/.cfg`, preserving its comments and its ordering.
  `cfg database` adds, lists, shows, updates, removes and tests the connection
  entries, through `--dsn`, `--host`, `--port`, `--user`, `--schema`, `--tls`,
  `--ca-file`, `--ca-path` and `--password-command`.
  `password_command` runs as the leader of a process group of its own, and its
  phase ends only when it has exited and its standard output has ended: at
  `core.password_timeout`, or when it writes more than 4096 bytes, the whole
  group is ended, descendants included, and the invocation exits `78` — a helper
  that exits `0` while a descendant holds its output open no longer hangs the
  invocation.
- **Testing an entry.** `cfg database test` contacts the server an entry
  describes and reports four steps in one order: the connection and its
  authentication, the read-only session and its confirmation, the server's
  series against the supported window, and a probe of the reader's catalogue
  privileges. Exit `0` says the four steps ran, not that the entry is usable:
  `can_read_catalogue` in the document is what says whether a read through it
  would be complete. It is the one `cfg` subcommand that opens a connection, and
  the last leaf of the command tree to be written — every node of the tree now
  has an implementation.
- **Reading the catalogue.** One connection, opened as late as possible and
  closed when the read ends, with the read-only session enforced before any
  statement. The catalogue is read from `INFORMATION_SCHEMA` in a fixed
  repertoire of eleven statements. `tpl` never writes to the database.
- **The model.** Databases, tables, columns with their decomposed type and their
  classified default, generated columns, indexes, foreign keys in both
  directions, check constraints, views, routines with their parameters, triggers
  and the server. What the reader's privileges did not reach is marked in the
  model rather than silently omitted.
- **Schema commands.** `schema info`, `tables`, `table`, `views`, `view`,
  `routines`, `routine` and `dump`, with `--pattern` on the listing commands.
- **The catalogue cache.** `cache load`, `cache clean` and `cache status`. A
  cached document carries `schema_version` and `cache_format`, both at `1`; a
  document that cannot be decoded is a miss, and the read goes to the server.
  `--no-cache` and `--direct` bypass the cache on the commands that consult it.
  A cache write does not force its files to disk, so a file a crash leaves torn
  is that same miss; `cache status` counts the object files held without
  reading them, so a file that cannot be read is still counted. A cached
  `schema info` reads `database.json` and counts the object files, and the
  text listing of `schema tables` decodes four members of each table file; an
  object file that fails only in what they no longer decode is not a miss for
  them, and remains one for every other read. An object file that is a symbolic
  link is a miss for every read and is never read through, and a read of one
  named table, view or routine whose file holds another object — another kind,
  or a name that differs in any byte, as on a filesystem that folds case — is a
  miss, so the read goes to the server rather than answering that the object
  does not exist.
- **Templates.** `template list`, `template show`, `template check` and
  `template path`.
- **The render environment.** MiniJinja, with templates loaded and compiled at
  run time and never embedded at compile time. The registered surface is a
  contract of twenty-three names:
  - filters — `pascal`, `camel`, `snake`, `upper_snake`, `kebab`, `quote`,
    `sql_type`, `json`, `indent`, `comment`, `escape`;
  - tests — `nullable`, `primary_key`, `auto_increment`, `unique`, `numeric`,
    `temporal`, `textual`;
  - functions — `table`, `view`, `routine`, `column`, `fail`.

  Fourteen further filters inherited from the engine are pinned to its version
  rather than guaranteed by `tpl`.
- **Rendering.** `tpl render <TEMPLATE>` renders one template, once, to stdout.
  The context comes from the selected database or from `--context`, never from
  both; `--table`, `--view` or `--routine` binds one object, and a repeatable
  `--set KEY=VALUE` passes template variables. A `--context` document in which
  a foreign key names a table its `tables` does not carry exits `65` naming the
  table, the key and the table it names. Every render is bounded, besides its
  deadline, by render fuel (`core.render_fuel`, default 100 000 000 evaluation
  steps), by the render output limit (`core.render_output_limit`, default
  64 MiB) and by the render memory limit (`core.render_memory_limit`, default
  128 MiB, the heap the process holds while the render runs, observed every
  10 ms); the first crossed exits `65` naming the bound, its value and the key,
  and a render stopped by either limit writes nothing to stdout. When a render
  reads the server, the connection is closed and the driver's runtime shut down
  before the template starts.
- **Output.** Every JSON document shares one envelope, carrying `schema_version`,
  `source` and `data`. `--format json` and `--pretty` select it on the commands
  that emit a document.
- **Diagnostics.** Exit codes drawn from the `sysexits` set, one per condition;
  nearest-match suggestions on an unknown name; verbosity gated by `-v` and `-q`;
  and a panic hook that reports a defect in `tpl` as exit `70`.
- **Help and error texts a caller can act on alone.** Every help text and
  every error message states what failed, why, and a command that can succeed.
  - Help: every leaf's `DESCRIPTION`, and its JSON `description`, ends with
    four statements — whether it contacts the server, which database entry it
    needs, which files it writes, and what it prints. `tpl help render` lists
    the context variables and every guaranteed filter, test and function with
    its signature and purpose.
  - `tpl help --format json`: each `template_surface` name is an object with
    `name`, `signature`, `operand`, `arguments` and `purpose`; `data` gains
    `context_variables`; a flag `default` is `null` or one JSON string of the
    command-line text, so `--port` shows `"3306"`.
  - `cfg get`, `cfg list` and `cfg database show` with `--format json` keep
    TOML types: an integer is a number, `password_command` an array.
  - New refusals: `--tpl-dir` naming a directory that is not a `.tpl` folder
    exits `78`, and the `hint` names the `.tpl` folder it holds; a `.tpl`
    folder without `.cfg` not owned by the caller exits `78`; `tpl init` into
    a `.tpl` folder exits `64`; `tpl render --context --direct` exits `64`;
    `tpl cfg database update` with no field flag exits `64`; `tpl cfg get` of
    a block exits `64`; `tpl cache clean` naming an object that is not cached
    exits `66`.
  - New refusals of values, each `64` on the command line with nothing
    written, and `78` in `.tpl/.cfg` where the value can stand there: a `password_command` string that yields no word,
    leaves a quote unclosed, ends in a backslash or begins with `[`; `${` in
    `ca_file` or `ca_path`; an entry name, including `core.database`, outside
    `[A-Za-z0-9_]{1,64}`; a `${NAME}` reference whose name is not
    `[A-Za-z_][A-Za-z0-9_]*`; an empty or whitespace-only host or database.
    `${VAR}` in `password_command` is stored and passed to the program as
    written.
  - `tpl template check` reports every failing template, one message each,
    before exiting `65`.
  - `tpl -d <name> cache clean` removes `.tpl/.cache/<name>/` of a name no
    entry declares.
  - New stderr warnings, exit code unchanged, suppressed by `-q`: `--tpl-dir`
    given to `tpl init`; `-d/--database` given to `cfg database add` or
    `update`, naming `--schema` or `--dsn` where one would act; `tpl cfg unset` of a `dsn`, naming
    what it carried; and, naming `tpl -d <name> cache clean`, each entry
    removed or repointed by `cfg database remove`, `cfg database update`,
    `cfg set` or `cfg unset`, since any data cached for it is kept.
  - Hints: a `tpl` command in a `hint` carries the invocation's `--tpl-dir`
    and `-d`; no `hint` deletes or overwrites a value or cached object the
    invocation did not name, so an entry defined by `dsn` is changed through
    `--dsn`; a rewritten command keeps the whole invocation; a prefix of a
    command (`tpl sch`) and an undefined value in a render get a
    nearest-match suggestion; a suggestion must share a character with the
    name; `-d` or `--tpl-dir` that took a command name reports that the flag
    needs a value.
  - Library API: `tpl::Error` gained eighteen variants, and thirty-two
    existing variants gained fields; `LoadWithoutStoring` became a struct
    variant. `Error` is `#[non_exhaustive]`, so a `match` with a wildcard arm
    still compiles, but code that constructs or destructures a changed
    variant without `..` does not. In `tpl::error`, `ContextFault::Structure`
    replaced `rule` with `at` and `expected`, `ContextFault` gained `Empty`,
    and `EntryRepair::Rewrite` was replaced by `InsideDsn`,
    `DsnWithoutPassword` and `Discrete`; neither enum is `#[non_exhaustive]`.
- **Worked examples.** `examples/` holds four complete demonstrations that build
  an application's data layer — in Go, Rust, Python and Node.js — from three
  known schemas through the command line alone, each ending in a compile gate
  that submits every file it rendered to that language's own toolchain.
- **The shared example driver.** `examples/_driver/` runs the five-command
  workflow once for all four examples, checking every exit code, keeping stdout
  and stderr apart, and redirecting each render's bytes into a file unchanged.
- **The published datasets.** `sakila` and `world` are vendored verbatim under
  `scripts/mariadb/datasets/`, each with the source URL, date and checksums it
  was taken at, and `scripts/mariadb/seed-datasets.sh` loads them into a named
  fixture server, re-grants the reader on them, and verifies what arrived.
- **The install script.** `install.sh` installs and updates `tpl` in one
  command,
  `curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh`,
  into `/usr/local/bin` or the directory `TPL_INSTALL_DIR` names, after
  verifying the archive against the release's `SHA256SUMS`.
- **Release archives.** Pushing a `v*` tag publishes a GitHub Release carrying
  one `tpl-<tag>-<triple>.tar.gz` per supported target and a `SHA256SUMS` file
  covering them, only if the tag is annotated, on `main`, equal to the
  `Cargo.toml` version, and has its release notes, and the tagged commit passes
  validation. A pre-release tag publishes a GitHub pre-release, which
  `install.sh` never installs. The archives are not signed.

## The record before this file

This file begins at the present and does not reconstruct what preceded it.
Reconstructing it would mean writing version headings for releases that never
happened, and the version number has one source, which no document may
duplicate. The detailed record of the work already exists, and is authoritative
where this file is silent:

- **`git log`** — every change as it was made, with each sprint merge naming the
  work it closes.
- **The `tpl` roadmap in `rmp`** — the sprints and their tasks, each closed task
  naming the commit that records it, and each carrying the decisions, findings
  and tests logged while the work happened.
- **[`specification/README.md`](specification/README.md)** — the functional
  specification's per-edition ledger, each edition stating what it added or
  amended and on what evidence.
- **[`docs/adr/`](docs/adr/)** — the architecture decision records, each with the
  alternatives it rejected.
