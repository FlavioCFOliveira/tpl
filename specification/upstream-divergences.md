---
title: Upstream Divergences
status: approved
last-reviewed: 2026-09-10
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
| [DIV-036](#div-036) | `CLAUDE.md` | Migration | `scripts/mariadb/` lacks the benchmark fixture, and is described in the singular |
| [DIV-037](#div-037) | both | Contradiction | A withdrawn MariaDB floor, no ceiling, and no refusal of MySQL |
| [DIV-038](#div-038) | both | Migration | Routine naming has no disambiguator |
| [DIV-039](#div-039) | both | Contradiction | Determinism stated over all output |
| [DIV-040](#div-040) | `CLAUDE.md` | Contradiction | `tpl cache` is absent, and the auxiliary set is closed |
| [DIV-041](#div-041) | `CLAUDE.md` | Migration | The target matrix is deferred; this specification now fixes it, and Linux is `musl` |
| [DIV-042](#div-042) | `README.md` | Contradiction | The JSON error envelope, the `kind` field, and `did_you_mean` |
| [DIV-043](#div-043) | `README.md` | Contradiction | The `.cfg` is now read strictly; an unrecognised key is fatal |

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

**Target**: `README.md`, the flag tables of `database add` and `update` and of
`render`. **Kind**: contradiction.

*Says*: `--password`, short `-p`, "Password, written to `.cfg`"; and short
forms `-H` for `--host`, `-P` for `--port`, `-u` for `--user`, and `-s` for
`--set` on `tpl render`.
*Specification*: `FR-CFG-030` — the `--password` flag does not exist. And
`FR-GLOB-024` — the five short forms of `FR-GLOB-001` are the whole short-flag
set of the tool, and no local flag carries one.
*Correction*: remove the `--password` row, and remove the short-form column
entry from the `--host`, `--port`, `--user`, and `--set` rows. `--dsn`,
`--schema`, `--tls`, `--password-command`, `--ca-file`, and `--ca-path` are the
other flags of those two commands, per `FR-CFG-027`, and none has a short form
either.

*Extended in the fifth edition.* The entry previously covered `-p` alone, on
the ground that it collides with `-P` by case. `OQ-016` is now closed the other
way from the root document: rather than deciding which local short forms to
keep, `FR-GLOB-024` declares that there are none, so all four of the root
document's remaining short forms are wrong and not only the one that collides.
The reasoning is in `FR-GLOB-024`; the `-P`/`-p` collision it describes is the
worked example of why the whole one-letter space is reserved.

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

**Target**: both. **Kind**: contradiction.

*Says*: `README.md` describes `password_command` as a "shell command whose
trimmed stdout is used as the password"; the `.cfg` example in `CLAUDE.md`
writes it as a TOML string, `password_command = "security …"`.
*Specification*: `FR-CONF-024` — executed directly, without a shell, from an
array; shell metacharacters are literal arguments. `FR-CONF-023` — stored in
`.cfg` as an array. `FR-CONF-025` splits a string supplied to a command, not
one found in the file, and `FR-CONF-035` refuses a string found in the file
with `78`.
*Correction*: in `README.md`, remove the word "shell", state the array form,
and rewrite the `.cfg` example line as
`password_command = ["security", "find-generic-password", "-s", "tpl-reporting", "-w"]`.

*Amended in the fifth edition.* Two things changed. `OQ-019` is closed by
`FR-CONF-035`: a `password_command` that is not a TOML array of strings is
`78`, so the string in the `README.md` example is no longer merely a
non-canonical spelling — it is a `.cfg` that `tpl` refuses to read, and the
correction is required rather than tidying. And the `CLAUDE.md` half of this
entry is discharged: that file no longer carries a `.cfg` example, having been
reduced to agent coordination, so only `README.md` is left to correct.

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
to `preferred`. `CLAUDE.md` carries the same three-mode enumeration, as the
illustration of an enumerated flag value in its help-text rules.
*Specification*: `FR-CONF-013` — five modes, defaulting to `verify-identity`,
with `ca_file` and `ca_path` as new keys.
*Correction*: replace the enumeration and the default, and add the two keys to
the example. Correct the `CLAUDE.md` illustration to the five modes, or choose
a flag whose value set this specification does not fix.

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
*Correction*: replace the four-row table with the set of `NFR-PERF-014` and
point the derived rules at
[performance-requirements.md](performance-requirements.md).

*Amended in the fifth edition.* The correction previously offered a choice —
keep the figures in `CLAUDE.md` or move them to `BENCHMARKS.md` — and cited
`OQ-051` through `OQ-060` for their being unmeasured. Those ten entries are
closed and the choice is settled by what the specification now does with them:
**the four figures `CLAUDE.md` states are adopted into `NFR-PERF-014` as
provisional**, marked under `NFR-PERF-019`, and each is removed from that table
by the same step that records a real measurement in `BENCHMARKS.md`, per
`NFR-PERF-020`. `CLAUDE.md` should therefore keep no figure at all: a third
copy would be the one nobody updates when the first measurement lands.
`NFR-PERF-014` records the provenance of each of the four, so removing them
from `CLAUDE.md` loses nothing.

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
*Correction*: add `seed-bench.sql` to the tree and to the testing section, and
state that the container must be buildable at **each** supported server series
rather than at one, per `FR-SRV-029`. `CLAUDE.md` describes the container in the
singular throughout; the version window of `FR-SRV-001` makes four the number,
and the DDL of `setup.sql` and `seed.sql` must be DDL that all four accept.
Separately, and more urgently than any correction listed in this file, none of
the four files exists in the repository today. **All twenty-four open
questions that remain in this specification are blocked by that absence** —
`OQ-009`, `OQ-010`, `OQ-024`, `OQ-025` through `OQ-042`, `OQ-045`, `OQ-070`,
and `OQ-072` — and so are the mandated tests of `BR-SCH-004`, `FR-SRV-029`,
and `BR-SEC-003`, and five of the nine budgets of `NFR-PERF-014`. It is the
single largest blocker this specification records.

## DIV-037

**Target**: both. **Kind**: contradiction.

*Says*: `README.md` states, in its requirements table, `MariaDB 10.6 or later`,
and states that the generated example template "renders without error against
any table of any MariaDB database". `CLAUDE.md` states no version at all, and
observes that MariaDB and MySQL diverge in the catalogue without saying what
`tpl` does about it. The `README.md` line is wrong in both directions: `10.6`
left community maintenance on 2026-07-06, and `or later` states no ceiling.
*Specification*: `FR-SRV-001` — the supported servers are the MariaDB series
that belong to one of the three most recent major families **and** are under
community maintenance, which on 2026-09-10 is `12.3`, `11.8`, `11.4` and
`10.11`, per `FR-SRV-015`. `FR-SRV-020` — a MariaDB below that window is
refused with `78` and `kind: server_version_unsupported`. `FR-SRV-003` — a
server that is not MariaDB is refused with `78` and `kind: server_not_mariadb`,
because three verified divergences would make the model silently wrong rather
than empty.
*Correction*: qualify the `README.md` sentence to a supported server, state that
MySQL is refused rather than attempted, and — wherever either file needs to name
the supported versions — cite `FR-SRV-001` rather than copy the table of
`FR-SRV-015`, per `BR-SRV-005`. A second copy of four version numbers in a file
this specification does not own is the copy nobody will re-verify.
*Amended in the fourth edition*: this entry previously recorded the correction
against a minimum of MariaDB 10.6. That floor is withdrawn — 10.6 left community
maintenance on 2026-07-06 — and the correction owed is now the window, not a
floor.

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

## DIV-040

**Target**: `CLAUDE.md`, the three arms and the porcelain command list.
**Kind**: contradiction.

*Says*: "Fora dos três braços existe apenas a gestão do projecto (`tpl init`,
`tpl database …`, `tpl config …`)" — and the porcelain command list names no
other group.
*Specification*: `FR-CLI-010` — the top-level commands are `schema`,
`template`, `render`, `cache`, `cfg`, `init`, `help`, and `version`.
`FR-CLI-008` makes `tpl cache` a group node, and
[cache-commands.md](cache-commands.md) gives it three subcommands. The
auxiliary set is therefore four groups and not three, and `tpl cache` is absent
from `CLAUDE.md` altogether — including from the sentence that closes the set
with "apenas".
*Correction*: add `tpl cache load|clean|status` to the porcelain command list,
and open the auxiliary sentence to include it. `DIV-015`, which replaces
`tpl database …` and `tpl config …` with `tpl cfg …`, applies to the same
sentence.

## DIV-041

**Target**: `CLAUDE.md`, supported platforms. **Kind**: migration.

*Says*: "A matriz concreta de alvos — target triples, escolha de libc,
linkagem e forma de empacotar o binário — é decisão de arquitectura em curso e
**não se fixa aqui**." It also states, correctly, that no target is second
class and that a performance baseline always names the target it was measured
on.
*Specification*: `NFR-PERF-018` — the target set is exactly four, and it is
fixed:

| System | Architecture | Target |
|---|---|---|
| Linux | amd64 | `x86_64-unknown-linux-musl`, statically linked |
| Linux | arm64 | `aarch64-unknown-linux-musl`, statically linked |
| macOS | amd64 | `x86_64-apple-darwin` |
| macOS | arm64 | `aarch64-apple-darwin` |

The libc question the sentence defers is answered: Linux is `musl`, linked
statically, so one artefact per architecture runs wherever that architecture
does with no dependency on the host's C library version. The `gnu` triples are
not targets.
*Correction*: replace the deferral with a pointer to `NFR-PERF-018`, and drop
the two sentences that describe the matrix as an open architecture decision.
The two rules beside it are correct and this specification now carries them
both — no target is second class, per `NFR-PERF-018`, and a baseline names its
target, per `NFR-PERF-012` — so they should point there rather than restate it.

*What the correction must not lose.* The `musl` static linkage has an
observable consequence, and it is recorded in `NFR-PERF-018` and again at
`FR-CONF-005`: a statically linked `musl` binary resolves names through
`musl`'s own `getaddrinfo`, which does not load the platform's name-service
modules, so a name resolvable only through such a module does not resolve for
`tpl` on a Linux target. The outcome is an ordinary `69`, per `FR-ERR-027`, and
what it obliges is precision in the `cause` line, per `FR-ERR-034`. Whoever
edits `CLAUDE.md` should not present the linkage as a pure packaging decision.

## DIV-042

**Target**: `README.md`, the exit codes and error messages section. **Kind**:
contradiction.

*Says*: "Under `--format json`, errors are JSON too", followed by the envelope
verbatim —
`{"error":{"exit":66,"code":"EX_NOINPUT","kind":"table_not_found", … ,"did_you_mean":["orders"]}}`
— and then "`kind` is a stable enumerated identifier meant to be compared
programmatically; `message` is meant to be read."
*Specification*: `FR-ERR-033` — **no error is ever emitted as JSON**, from any
command, at any verbosity, under any value of `--format`. When the outcome is
an error, `--format` is ignored, the four labelled lines of `FR-ERR-008` go to
stderr, and stdout is empty. Three of the four things the passage states are
withdrawn with the document that carried them: `FR-ERR-014`, the document;
`FR-ERR-015` and `FR-ERR-016`, the `kind` field and its compatibility rule;
and the `did_you_mean` array, per the amendment to `FR-ERR-023`. Nearest-match
suggestion survives, in the text `hint` line.
*Correction*: delete the sentence, the JSON block, and the sentence about
`kind`. Replace them with the statement that `--format` applies to a result and
never to a failure, that the exit code is the machine-comparable signal, and
that `FR-ERR-034` states per code what the `cause` line names. Whatever remains
of the four-line example in that section is correct and stays.

*Note on scope.* The dossier for the fifth edition recorded this envelope as
appearing in `CLAUDE.md`. It no longer does: that file has been reduced to
agent coordination and now defers the whole error contract to
`specification/errors-and-exit-codes.md`, which is `DIV-001` discharged for
that section. The envelope survives only in the root `README.md`, and this
entry is written against the files as they now stand.

## DIV-043

**Target**: `README.md`, the configuration section. **Kind**: contradiction.

*Says*: a `.tpl/.cfg` example, and prose describing the file, written at a time
when an unrecognised key had no stated outcome.
*Specification*: `FR-CONF-034` — a key outside the enumerated space of
`FR-CONF-002`, **anywhere** in `.tpl/.cfg`, is `78` (`EX_CONFIG`) with a
nearest-match suggestion. `FR-CONF-011` — a DSN carries no query parameters,
and any `?` is `78`. `BR-CONF-004` gives the reason for both: the file is
untrusted input that decides which host is contacted, which credential is
used, and which child process is executed, so a reader that accepted what it
did not understand would be guessing at those three.
*Correction*: three things follow, and the first is the one that matters.

1. **Check every `.cfg` example in the file against the key space of
   `FR-CONF-002`, key by key.** A key shown in an example and absent from that
   table is no longer undocumented — it is a file `tpl` refuses to read, and a
   reader who copies the example gets `78` on every subsequent invocation. The
   root document is the first place a user copies a `.cfg` from.
2. State that the file is read strictly, so that a user who adds a key of their
   own knows the outcome before they meet it.
3. State that a DSN carries no query parameters. No example in the root
   document shows one today, so this is a gap rather than a contradiction, but
   a DSN copied from another tool commonly carries them and the document is
   where a reader would look for permission.

`DIV-013` corrects the one line of that example this specification refuses
outright — `password_command` as a string — and applies to the same block.
