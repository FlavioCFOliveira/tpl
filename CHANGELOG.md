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
  folder and suppresses the walk. `.tpl/.cfg` is checked for ownership and
  permissions, at the target of a symbolic link, before it is read.
- **Configuration.** `cfg get`, `cfg set`, `cfg unset` and `cfg list` read and
  write `.tpl/.cfg`, preserving its comments and its ordering.
  `cfg database` adds, lists, shows, updates, removes and tests the connection
  entries, through `--dsn`, `--host`, `--port`, `--user`, `--schema`, `--tls`,
  `--ca-file`, `--ca-path` and `--password-command`.
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
  `--set KEY=VALUE` passes template variables.
- **Output.** Every JSON document shares one envelope, carrying `schema_version`,
  `source` and `data`. `--format json` and `--pretty` select it on the commands
  that emit a document.
- **Diagnostics.** Exit codes drawn from the `sysexits` set, one per condition;
  nearest-match suggestions on an unknown name; verbosity gated by `-v` and `-q`;
  and a panic hook that reports a defect in `tpl` as exit `70`.
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
