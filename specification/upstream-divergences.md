---
title: Upstream Divergences
status: draft
last-reviewed: 2026-09-09
related: [README.md, cli-contract.md, render-command.md, cfg-commands.md]
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
