---
title: Upstream Divergences
status: draft
last-reviewed: 2026-09-09
related: [README.md, template-environment.md, server-contract.md, performance-requirements.md]
---

# Upstream Divergences

## Overview

This specification is the single source of functional truth. The root
`README.md` and the root `CLAUDE.md` currently state functional content that
this edition either contradicts or now owns. Each divergence is recorded here
with what the file says, what this specification says, and the correction owed.

**This file does not authorise a change to either document, and the
specification never edits them.** It is a work list for whoever holds the pen on
those two files.

Two kinds of entry appear:

- **Contradiction** — the file states something this specification contradicts.
  Leaving it is a defect.
- **Migration** — the file states something this specification now owns.
  Leaving it creates two sources for one truth.

## Index

| Id | Target | Kind | Subject |
|---|---|---|---|
| [DIV-001](#div-001) | both | Migration | All CLI-surface content moves here |
| [DIV-002](#div-002) | `CLAUDE.md` | Contradiction | "No command accepts a password in argv" |
| [DIV-003](#div-003) | `README.md` | Contradiction | The `--password` / `-p` flag |
| [DIV-004](#div-004) | both | Contradiction | `--output`, `--output-dir` and the file-writing surface |
| [DIV-005](#div-005) | `CLAUDE.md` | Contradiction | Only `init` and `cfg` write inside `.tpl/` |
| [DIV-006](#div-006) | `CLAUDE.md` | Contradiction | Configuration printing always redacts |
| [DIV-007](#div-007) | both | Contradiction | `EPIPE` always exits `0` |
| [DIV-008](#div-008) | `CLAUDE.md` | Contradiction | `${VAR}` expands in any string value |
| [DIV-009](#div-009) | both | Contradiction | `schema dump` equals the render context |
| [DIV-010](#div-010) | `README.md` | Contradiction | TTY-dependent `--format` default |
| [DIV-011](#div-011) | `CLAUDE.md` | Contradiction | Positional render target |
| [DIV-012](#div-012) | `CLAUDE.md` | Contradiction | `tpl init` takes no argument |
| [DIV-013](#div-013) | `README.md` | Contradiction | `password_command` is a shell command |
| [DIV-014](#div-014) | `README.md` | Contradiction | Aliases `procs` and `proc` |
| [DIV-015](#div-015) | both | Contradiction | The `database` and `config` command groups |
| [DIV-016](#div-016) | both | Contradiction | `--no-color` and `NO_COLOR` |
| [DIV-017](#div-017) | both | Contradiction | `TPL_DIR` and `TPL_DATABASE` |
| [DIV-018](#div-018) | `README.md` | Contradiction | Three TLS modes, defaulting to `preferred` |
| [DIV-019](#div-019) | both | Contradiction | `--all-tables` and `--pattern` on `render` |
| [DIV-020](#div-020) | both | Contradiction | The `.tpl/` layout omits `.cache/` |
| [DIV-021](#div-021) | `README.md` | Contradiction | `template check` lints |
| [DIV-022](#div-022) | `README.md` | Contradiction | `--pattern` follows the server collation |
| [DIV-023](#div-023) | both | Migration | Global flag tables are wrong in three ways |
| [DIV-024](#div-024) | `CLAUDE.md` | Migration | Project discovery lacks its boundary and checks |
| [DIV-025](#div-025) | `CLAUDE.md` | Migration | The "specification does not exist yet" note |
| [DIV-026](#div-026) | both | Contradiction | The `rust_type` and `go_type` filters |
| [DIV-027](#div-027) | both | Contradiction | The `plural` and `singular` filters |
| [DIV-028](#div-028) | both | Contradiction | Auto-escaping keyed on the file extension |
| [DIV-029](#div-029) | both | Contradiction | `tpl init` creates four artefacts |
| [DIV-030](#div-030) | `CLAUDE.md` | Contradiction | The read-only session presented as prevention |
| [DIV-031](#div-031) | `CLAUDE.md` | Contradiction | `SHOW` as a permitted way to read the catalogue |
| [DIV-032](#div-032) | `CLAUDE.md` | Contradiction | `model/` as the documented public surface |
| [DIV-033](#div-033) | `CLAUDE.md` | Contradiction | The whole `Environment` surface as contract |
| [DIV-034](#div-034) | both | Contradiction | The `table` and `column` field lists |
| [DIV-035](#div-035) | `CLAUDE.md` | Migration | The performance budget table |
| [DIV-036](#div-036) | `CLAUDE.md` | Migration | `scripts/mariadb/` lacks the benchmark fixture |
| [DIV-037](#div-037) | both | Contradiction | No minimum server version, and no refusal of MySQL |
| [DIV-038](#div-038) | both | Migration | Routine naming has no disambiguator |
| [DIV-039](#div-039) | both | Contradiction | Determinism stated over all output |

## DIV-001

**Target**: both. **Kind**: migration.

The root documents are the two places the CLI surface was described before this
specification existed. Both must now be reduced.

- `README.md` keeps its role as the entry door: what `tpl` is, how to install
  it, a short quick start, and a pointer to `/specification`. Its command
  reference, configuration reference, exit code table, and error-message section
  are owned here.
- `CLAUDE.md` keeps agent coordination: which agent does what, the mandatory
  skills, the workflow, and the validation pipeline. Its description of the
  three arms, the CLI surface, the `.tpl` project, exit codes, and error message
  format is owned here.

Content of those files that this edition does not cover — implementation
language and conventions, performance and resource budgets, supported platforms,
the MariaDB container discipline — is untouched by this entry.

## DIV-002

**Target**: `CLAUDE.md`, CLI design rules. **Kind**: contradiction.

*Says*: "Nenhum comando aceita password em `argv`."
*Specification*: `FR-CFG-030` removes the flag, but `FR-CFG-031` and
`FR-CFG-032` keep two documented paths open — `--dsn` and `tpl cfg set` — with a
warning in the help rather than a refusal.
*Correction*: rewrite the sentence to say that no flag named `password` exists,
and that two documented paths remain open.

## DIV-003

**Target**: `README.md`, flags of `database add` and `update`. **Kind**:
contradiction.

*Says*: `--password`, short `-p`, "Password, written to `.cfg`".
*Specification*: `FR-CFG-030` — the flag does not exist. The short form would
also collide with `-P` for `--port` by case alone.
*Correction*: remove the row.

## DIV-004

**Target**: both. **Kind**: contradiction.

*Says*: `README.md` presents `--output`, `--output-dir`, `--output-name`,
`--no-clobber`, and `--dry-run`, with a paragraph on atomic writes;
`CLAUDE.md` presents `--output` and `--output-dir` as the explicit way to save a
result, and its exit code table attaches `73` and `74` to those destinations.
*Specification*: `FR-RND-028` — none of the five flags exists, stdout is the
only destination, and `FR-ERR-003` restricts `73` to `tpl init`.
*Correction*: remove the flags, the atomic-write paragraph, the "console by
default, file on request" framing, and the destination clauses from both exit
code tables.

## DIV-005

**Target**: `CLAUDE.md`, the `.tpl` project section. **Kind**: contradiction.

*Says*: `tpl init`, `tpl database …` and `tpl config …` are the only commands
that write in `.tpl/`; all others treat it as read-only.
*Specification*: `FR-PROJ-023` — `tpl cache load` and every cached read command
also write to `.tpl/.cache/` on a miss.
*Correction*: rewrite the sentence to list the four writers.

## DIV-006

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

*Says*: any command that prints configuration always redacts passwords.
*Specification*: `FR-CFG-021` redacts in `cfg list` and `cfg database show`;
`FR-CFG-006` and `BR-CFG-002` make `tpl cfg get` a deliberate exception.
*Correction*: state the exception rather than leave it to be discovered.

## DIV-007

**Target**: both. **Kind**: contradiction.

*Says*: `EPIPE` on stdout exits `0` silently, in every case.
*Specification*: `FR-ERR-025` keeps `0` for the ordinary case, and `FR-ERR-026`
returns `74` when the pipe closes part-way through a JSON document.
*Correction*: refine the flat rule in both files.

## DIV-008

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

*Says*: `${VAR}` is substituted "em qualquer valor string".
*Specification*: `FR-CONF-015` limits expansion to six fields, and
`FR-CONF-016` and `FR-CONF-017` forbid it in `tls` and `password_command`.
*Correction*: narrow the statement to the enumerated field set.

## DIV-009

**Target**: both. **Kind**: contradiction.

*Says*: `tpl schema dump` produces exactly the JSON the render receives as
context.
*Specification*: `FR-SCH-018` — the dump carries only the server-derived part;
`FR-RND-024` shows that `vars`, `tpl`, and `now` are always injected by the
render.
*Correction*: restate the round-trip as "the dump supplies the server-derived
part of the context", in both files.

## DIV-010

**Target**: `README.md`, global flags. **Kind**: contradiction.

*Says*: `--format` "Defaults to `text` on a TTY, `json` otherwise".
*Specification*: `FR-OUT-001` and `FR-OUT-002` — the default is `text`, fixed,
with no terminal detection anywhere.
*Correction*: change the default and remove the TTY clause.

## DIV-011

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

*Says*: `tpl render <template> [alvo] [flags]` — a positional render target.
*Specification*: `FR-RND-003` — the target is always a flag, and the template
name is the only positional.
*Correction*: change the usage line.

## DIV-012

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

*Says*: `tpl init` with no argument.
*Specification*: `FR-PROJ-012` — an optional positional path, defaulting to the
current directory.
*Correction*: add the optional argument.

## DIV-013

**Target**: `README.md`, `password_command`. **Kind**: contradiction.

*Says*: "Shell command whose trimmed stdout is used as the password."
*Specification*: `FR-CONF-024` — executed directly, without a shell, from an
array; shell metacharacters are literal arguments.
*Correction*: remove the word "shell" and state the array form. The example
given in the file still works exactly as written.

## DIV-014

**Target**: `README.md`, alias table. **Kind**: contradiction.

*Says*: `routines` aliases to `procs`, `routine` to `proc`.
*Specification*: `FR-CLI-011` — `rtns` and `rtn`, because a routine is a
procedure or a function.
*Correction*: change both rows.

## DIV-015

**Target**: both. **Kind**: contradiction.

*Says*: a top-level `tpl database …` group with alias `db`, a separate
`tpl config …` group, and `tpl database test` in the quick start.
*Specification*: `FR-CFG-001` through `FR-CFG-003` — one `tpl cfg` group, with a
`database` subgroup aliased `db`, and `tpl cfg database test`.
*Correction*: rewrite both command lists and the quick start.

## DIV-016

**Target**: both. **Kind**: contradiction.

*Says*: a `--no-color` global flag, colour implied off when stdout is not a TTY
or `NO_COLOR` is set.
*Specification*: `NFR-DET-004` — no colour anywhere, so the flag does not exist
and the variable is not read.
*Correction*: remove the flag row and the variable from both files.

## DIV-017

**Target**: both. **Kind**: contradiction.

*Says*: environment variables `TPL_DIR` and `TPL_DATABASE`, with `-d` overriding
`TPL_DATABASE`, which overrides `core.database`.
*Specification*: `FR-CLI-021` and `FR-CONF-030` — no environment variable is
read for behaviour, and precedence has two layers. `${VAR}` inside `.cfg`
remains the only environment read.
*Correction*: remove both variables and correct the precedence sentence in both
files.

## DIV-018

**Target**: `README.md`, TLS flag and the `.cfg` example. **Kind**:
contradiction.

*Says*: `--tls <mode>` is one of `disabled`, `preferred`, `required`, defaulting
to `preferred`.
*Specification*: `FR-CONF-013` — five modes, defaulting to `verify-identity`,
with `ca_file` and `ca_path` as new keys.
*Correction*: replace the enumeration and the default, and add the two keys to
the example.

## DIV-019

**Target**: both. **Kind**: contradiction.

*Says*: `README.md` documents `--all-tables`, `--pattern` on `render`, and
repeatable object flags, with examples that concatenate many renders;
`CLAUDE.md` refers to `render --all-tables` in its performance budget.
*Specification*: `FR-RND-002`, `FR-RND-004`, and `FR-RND-007` — one render per
invocation, each object flag at most once, and no `--all-*` or `--pattern` on
`render`.
*Correction*: remove the flags and the multi-render examples, and replace them
with the caller-side loop of `UC-008`. The performance budget in `CLAUDE.md`
needs a different unit of measure.

## DIV-020

**Target**: both. **Kind**: contradiction.

*Says*: the `.tpl/` tree contains `.cfg`, `.gitignore`, and `templates/`, and
the generated `.gitignore` holds one line.
*Specification*: `FR-PROJ-002` and `FR-PROJ-017` — the tree also contains
`.cache/`, and the generated `.gitignore` holds two lines.
*Correction*: update both trees and the `.gitignore` content in both files.

## DIV-021

**Target**: `README.md`, second arm. **Kind**: contradiction.

*Says*: `tpl template check <name>` "Parse and lint a template", with a required
name.
*Specification*: `FR-TMPL-017` through `FR-TMPL-019` — `check` parses only,
there is no lint, and the name is optional and repeatable.
*Correction*: remove the word "lint" and make the argument optional. The same
applies to `tpl template path`, which also takes an optional name per
`FR-TMPL-021`.

## DIV-022

**Target**: `README.md`, `--pattern`. **Kind**: contradiction.

*Says*: "Case sensitivity follows the server collation."
*Specification*: `FR-SCH-013` and `FR-SCH-014` — evaluated locally, ASCII
case-insensitive, independent of the server.
*Correction*: replace the sentence.

## DIV-023

**Target**: both. **Kind**: migration.

Both global flag tables are wrong in three ways at once: they list `--output`,
`--format`, and `--no-color` as global; they omit `--timeout`; and they do not
mention that `--direct` and `--no-cache` exist at all.
*Specification*: `FR-GLOB-001` — exactly seven global flags — and `FR-GLOB-021`,
which lists the four local flags and the commands that declare each.
*Correction*: replace both tables with a pointer to
[global-flags.md](global-flags.md).

## DIV-024

**Target**: `CLAUDE.md`, project discovery. **Kind**: migration.

*Says*: the walk climbs until a `.tpl/` folder is found, and `TPL_DIR` skips
discovery.
*Specification*: `FR-PROJ-005` adds the boundary at the home directory and the
mount point; `FR-PROJ-009` through `FR-PROJ-011` add canonicalisation and the
ownership and mode checks; `FR-GLOB-009` replaces `TPL_DIR` with `--tpl-dir`.
*Correction*: the description is incomplete rather than wrong; replace it with a
pointer to [project-and-discovery.md](project-and-discovery.md).

## DIV-025

**Target**: `CLAUDE.md`, functional specification section. **Kind**: migration.

*Says*: "A pasta `/specification` ainda não existe neste repositório", with an
instruction to remove the subsection once the bootstrap is done.
*Specification*: the folder now exists, and this file is part of it.
*Correction*: remove the subsection, as that text itself instructs.

## DIV-026

**Target**: both. **Kind**: contradiction.

*Says*: `CLAUDE.md` lists `rust_type` among the SQL and code filters;
`README.md` lists `rust_type` and `go_type` in its filter table and uses
`rust_type` in two template examples.
*Specification*: `FR-ENV-009` and `FR-ENV-010` — neither filter exists. The
mapping is delivered as a template macro at `.tpl/templates/rust/_types.jinja`,
written by `tpl init` per the amended `FR-PROJ-017`, so that the opinion it
encodes belongs to the project.
*Correction*: remove both filters from both tables, and rewrite the two
`README.md` examples to call the macro instead of the filter.

## DIV-027

**Target**: both. **Kind**: contradiction.

*Says*: both files list `plural` and `singular` as naming filters.
*Specification*: `FR-ENV-012` and `FR-ENV-013` — neither exists. Correct English
inflection is a project in itself, and a wrong plural on a table name that is
not English is guaranteed noise in generated code.
*Correction*: remove the row from both tables.

## DIV-028

**Target**: both. **Kind**: contradiction.

*Says*: `CLAUDE.md` requires auto-escaping to be enabled by extension for
`.html`, `.xml`, and `.htm`; `README.md` states that auto-escaping is off "for
every extension except `.html`, `.htm`, and `.xml`".
*Specification*: `FR-ENV-026` through `FR-ENV-028` — auto-escaping is off
always, no rule is keyed on any extension, and `escape` is an explicit filter.
*Correction*: remove the extension rule from both files. In `README.md` the
surrounding sentence, "Two behaviours differ from stock Jinja2 defaults", must
also be corrected: after this change the two are strict undefined variables and
the preserved trailing newline of `FR-SEM-003`, not auto-escaping.

## DIV-029

**Target**: both. **Kind**: contradiction.

*Says*: both files present `tpl init` as creating four artefacts, and list them.
*Specification*: `FR-PROJ-017`, as amended — five artefacts. The fifth is
`.tpl/templates/rust/_types.jinja`.
*Correction*: add the row to both tables, and add the file to the `.tpl` tree
shown in each. `README.md` additionally states that `.tpl/.gitignore` holds one
line, which `DIV-020` already corrects to two.

## DIV-030

**Target**: `CLAUDE.md`, project invariant 1. **Kind**: contradiction.

*Says*: "O `tpl` **nunca** emite DDL, DML ou qualquer statement de escrita",
followed immediately by the read-only session statement, presenting the session
setting as the mechanism that delivers the promise.
*Specification*: `FR-SRV-006` through `FR-SRV-011` and `BR-SRV-002` — the
promise has two parts and only one of them prevents. The guarantee is the closed
list of statements `tpl` will issue. The session setting is defence in depth: it
makes a write fail, it does not stop the connection attempting one, and it
constrains the transaction rather than the session. The specification adds a
read-back of the setting, and a limit of at most one connection per invocation.
*Correction*: state the closed list as the guarantee, keep the session setting as
defence in depth, and add the read-back and the connection limit. The `78` on
failure, and the absence of a flag to disable any of it, are unchanged.

## DIV-031

**Target**: `CLAUDE.md`, project invariant 1. **Kind**: contradiction.

*Says*: the catalogue is read exclusively through `INFORMATION_SCHEMA` "e,
quando estritamente necessário, `SHOW`".
*Specification*: `FR-SRV-006` and `FR-SRV-007` — the closed list has three
entries and `SHOW` is not one of them. A statement outside the list is not
permitted however necessary it seems.
*Correction*: remove the `SHOW` clause. The prohibition on `mysqldump` and on
any external process is unchanged and is restated by `FR-SRV-007`.

## DIV-032

**Target**: `CLAUDE.md`, project structure. **Kind**: contradiction.

*Says*: "as suas structs são a superfície pública documentada", of the `model/`
module.
*Specification*: only the JSON document and the command line are contract. The
library carries no compatibility guarantee, and the document contract lives in
[context-document.md](context-document.md) rather than in a set of Rust types.
*Correction*: state that the contract is the document, not the types that
produce it. Five questions this edition would otherwise have had to settle —
owned versus borrowed types, public fields versus accessors, newtypes for names,
whether the serialisation crate is a public dependency, and `#[non_exhaustive]`
— are architecture decisions rather than functional requirements, and belong in
an architecture decision record.

## DIV-033

**Target**: `CLAUDE.md`, render context. **Kind**: contradiction.

*Says*: "Filtros, testes e funções registados no `Environment` são superfície
pública: acrescentar é permitido, renomear ou remover é uma quebra de
compatibilidade."
*Specification*: `FR-ENV-001` through `FR-ENV-004` — the surface has three
groups and only the first carries that guarantee unqualified. The named
inherited filters of `FR-ENV-018` are guaranteed only against a pinned engine
minor version, and everything else the engine offers works but is guaranteed by
nobody.
*Correction*: replace the sentence with the three groups, or with a pointer to
[template-environment.md](template-environment.md).

## DIV-034

**Target**: both. **Kind**: contradiction.

*Says*: `README.md` states that a `table` carries `name`, `comment`, `engine`,
`charset`, `collation`, `columns`, `primary_key`, `indexes`, and
`foreign_keys`, and that a `column` carries `name`, `position`, `data_type`,
`nullable`, `default`, `comment`, "and the raw catalogue attributes MariaDB
reports for it". `CLAUDE.md` names the same set of model types.
*Specification*: the list is incomplete and, in two places, wrong in shape. A
table also carries `table_type` per `FR-CAT-002`, `referenced_by` per
`FR-CAT-013`, its triggers per `FR-CAT-014`, and its `CHECK` constraints per
`FR-CAT-015`. A column's `default` is a discriminated structure per
`FR-CTX-012`, not a value; its type is `column_type` plus eight decomposed parts
per `FR-CTX-015`, not "the raw catalogue attributes"; and it carries
`table_name` per `FR-CTX-019`. Thirteen volatile catalogue fields are excluded
outright by `FR-CAT-024`.
*Correction*: replace both descriptions with a pointer to
[catalogue-coverage.md](catalogue-coverage.md) and
[context-document.md](context-document.md).

## DIV-035

**Target**: `CLAUDE.md`, non-functional requirements. **Kind**: migration.

*Says*: a table of four performance budgets with figures in milliseconds, a
peak-memory figure in MiB, and a set of rules derived from them.
*Specification*: [performance-requirements.md](performance-requirements.md) —
the corpus keeps six requirements of form, the three reference workloads, and
the measurement protocol; the figures stay outside it. The budget set itself
changed: `--version` and `--help` are separate lines, the
`render --all-tables` line is replaced by the 200-invocation loop because
`FR-RND-007` removed the flag it measured, and four budgets are added — the
cache-served read per object, which is the one normative target, the failure
path, `tpl help --format json`, and the loop.
*Correction*: replace the four-row table with the set of `NFR-PERF-014`, keep
the figures in `CLAUDE.md` or move them to `BENCHMARKS.md` as that file's
authors prefer, and point the derived rules at
[performance-requirements.md](performance-requirements.md). Note that every
figure in the current table is unmeasured; `OQ-051` through `OQ-060` record
them.

## DIV-036

**Target**: `CLAUDE.md`, project structure and the MariaDB testing section.
**Kind**: migration.

*Says*: `scripts/mariadb/` holds a `Dockerfile`, `setup.sql`, and `seed.sql`,
and the two SQL scripts must cover the read surface exhaustively.
*Specification*: `WL-001` requires a fourth file, `seed-bench.sql`, and
`BR-PERF-002` keeps it separate from `seed.sql` on purpose: `seed.sql` is
exhaustive variety at minimal volume, for correctness, and `seed-bench.sql` is
volume at minimal variety, for measurement. One fixture serving both would hide
an N+1, which is invisible at ten tables.
*Correction*: add `seed-bench.sql` to the tree and to the testing section.
Separately, and more urgently than any correction listed in this file, none of
the four exists in the repository today. `OQ-009`, `OQ-010`, `OQ-024`, and
`OQ-025` through `OQ-042` are all blocked by that absence.

## DIV-037

**Target**: both. **Kind**: contradiction.

*Says*: neither file states a minimum server version. `README.md` states that
the generated example template "renders without error against any table of any
MariaDB database", and `CLAUDE.md` observes that MariaDB and MySQL diverge in
the catalogue without saying what `tpl` does about it.
*Specification*: `FR-SRV-001` — the floor is MariaDB 10.6. `FR-SRV-003` — a
server that is not MariaDB is refused with `78` and `kind: server_not_mariadb`,
because three verified divergences would make the model silently wrong rather
than empty.
*Correction*: state the floor wherever requirements are stated, qualify the
`README.md` sentence to a supported server, and state that MySQL is refused
rather than attempted.

## DIV-038

**Target**: both. **Kind**: migration.

*Says*: `README.md` documents `tpl schema routine <name>` and the repeatable
`--routine <name>` flag with a bare name; `CLAUDE.md` does the same in its
command list.
*Specification*: `FR-SCH-008`, as amended — wherever a command names one
routine, the qualified forms `procedure:<name>` and `function:<name>` are
accepted; and `FR-SCH-010`, as amended — a bare name matching both a procedure
and a function is `64`, never resolved in favour of either.
*Correction*: document the qualified form on `tpl schema routine`,
`tpl render --routine`, and both `tpl cache` subcommands that name an object,
and add the ambiguity case to the exit-code discussion. `DIV-014`, which
corrects the aliases of the same commands, applies to the same rows.

## DIV-039

**Target**: both. **Kind**: contradiction.

*Says*: "The same invocation against the same database produces byte-identical
output", without qualification, in `README.md`; `CLAUDE.md` states the same rule
in its determinism section.
*Specification*: `NFR-DET-001`, as amended — byte-identical **stdout**. The
diagnostic output written to stderr is neither deterministic nor contract, and
cannot be: `FR-GLOB-017` requires phase timings at `INFO`, which differ on every
run by construction.
*Correction*: add the word stdout, and state that stderr is excluded. The `now`
exception stated in both files is unaffected and remains correct.
