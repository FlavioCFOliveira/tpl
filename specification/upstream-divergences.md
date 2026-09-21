---
title: Upstream Divergences
status: approved
last-reviewed: 2026-09-18
related: [README.md, global-flags.md, cfg-commands.md, template-environment.md, server-contract.md, performance-requirements.md]
---

# Upstream Divergences

## Overview

This specification is the single source of functional truth. The root
`README.md` and the root `CLAUDE.md` have stated functional content that this
specification either contradicts or now owns, and they have stated as present
things the repository does not yet contain. Each divergence is recorded here
with what the file said, what is in force against it, the correction owed, and
whether it is still owed.

**This file does not authorise a change to either document, and the
specification never edits them.** It is a work list for whoever holds the pen on
those two files.

**Fifty-two entries are recorded. As of 2026-09-11, seventeen are due in
full, fourteen are discharged, and twenty-one are partly discharged.**
Thirty-eight entries still owe something: thirty-two owe it to `README.md`,
seven owe it to `CLAUDE.md`, and `DIV-037` is in both counts because it owes a
different thing to each. The asymmetry has one cause. `CLAUDE.md` was reduced
to agent coordination in a single commit, `0ea5624`, which discharged twelve
entries outright and one half of nineteen more; `README.md` has been edited
four times since this file was opened, each time in one line or one paragraph.

Three kinds of entry appear:

- **Contradiction** — the file states something this specification contradicts.
  Leaving it is a defect.
- **Migration** — the file states something this specification now owns.
  Leaving it creates two sources for one truth.
- **Overstatement** — the file states as present something the repository does
  not contain. Nothing is owed to this folder and no requirement is
  contradicted; what is wrong is that a reader who acts on the passage fails.
  Leaving it spends the credit of everything else the file says.

*The third kind is added in the sixteenth edition*, and four entries carry it:
`DIV-046` against `README.md`, and `DIV-050`, `DIV-051` and `DIV-052` against
`CLAUDE.md`. Each is raised against a passage that states as present something
the repository does not hold, and no one of them discharges another, because
each is corrected in the passage it was raised against. The first three record
one fact — the repository holds no crate — and `DIV-046` and `DIV-051` are the
closest pair among them: they record the same eight `cargo` commands, under
headings that are each other's translation, once in each root document.
`DIV-052` needs a second fact beside the first: there are no benchmarks to live
in the directory its passage names, and the fixture they are said to run
against is itself incomplete, which is what `DIV-036` still owes.

*The seventeenth edition closes that kind in `CLAUDE.md`* and adds no entry to
it. The sixteenth edition's sweep left one candidate in that file unjudged —
the *verificados* of *Plataformas Suportadas* — and the reading of that section
finds that it declares a policy rather than asserting a state, so nothing is
owed for it. That reading, and the sweep establishing that no other passage of
the kind is left in the file, are at *[A candidate read, and not
recorded](#a-candidate-read-and-not-recorded)*.

The two kinds above were written when every entry described the relation
between a root document and this corpus. An entry that describes the relation
between a root document and the repository has no home among them, and
labelling it a contradiction would have named a requirement that does not
exist.

## How an entry is read

Four things carry an entry, and each answers a different question.

- **Target** names the files the entry was raised against, and is the authority
  on where to look when the entry is re-checked.
- **Kind** is one of the three above.
- **Status** says what is still owed, and where, as of the date it names. It is
  the only part of an entry that speaks about the files as they now stand.
- The *Says* clause is a **quotation of the target file as it stood when the
  entry was written**. It is not a claim about the file today, and a discharged
  entry commonly carries one whose words are no longer in the file. **Status**
  is what settles whether the clause is still there; the quotation is kept
  because it is the evidence the entry was raised on, and removing it would
  leave the entry unfalsifiable.

A **Status** takes one of three values.

- **Due** — the target file still states what the entry records, and the
  correction is owed in full.
- **Discharged** — the target file no longer states it. The status names the
  commit that removed it, so that the entry says *when* it stopped being owed
  and not merely that it is not owed. Nothing remains.
- **Partly discharged** — the correction has separable parts, some made and
  some not. This is the ordinary outcome for an entry whose **Target** is
  `both`: the two files are edited under separate authorisations and almost
  never in the same commit, so an entry against both is discharged in halves.
  Such a status names, part by part, what was discharged and by which commit,
  and what remains owed. **An entry is never recorded as discharged while any
  part of it stands**, because an entry reduced to its last clause is still a
  correction somebody has to make.

## A discharged entry is kept, not removed

A discharged entry keeps its heading and its identifier, in place, and carries
a status saying what discharged it. It is not deleted, and it is not moved.
This is the rule [README.md](README.md#identifier-scheme) already states for a
withdrawn requirement, applied for the same reason: a reference written before
the discharge must resolve to an explanation rather than to nothing. `DIV-045`
has been written this way since the ninth edition, so the rule was already in
force in this file before it was stated here.

**Rejected: the *Closed* table of [open-questions.md](open-questions.md).** A
closed open question leaves the index and becomes a row in a table at the foot
of its file, and that was the obvious alternative, since a divergence closes
the way an open question closes rather than standing in force the way a
requirement stands. It was rejected on two grounds. First, entries here cite
each other — `DIV-029` cites `DIV-020`, `DIV-038` cites `DIV-014`, `DIV-040`
cites `DIV-015`, `DIV-043` cites `DIV-013` — and a citation whose target has
been lifted out of its section resolves to a row instead of to the reasoning
the citing entry depends on. Second, `DIV-045` already carries the in-place
form, so adopting the table would put two treatments of one event in one file,
which is the defect this specification records against the root documents.

## When this file is re-read

The classification below was made against `CLAUDE.md` and `README.md` as they
stood on 2026-09-11, at `87dd6e3` — the last commit to touch either. **It stops
being true the moment either file is edited**, and until this edition nothing
in the file said so. That is how `DIV-031` came to ask, through four editions
and one amendment of its own, for the removal of a clause that had been gone
since `0ea5624`.

The obligation that replaces the presumption is recorded in
[README.md](README.md#maintenance-debt) as the fifth validation rule of this
corpus: **a register of corrections owed to a file this specification does not
own is re-read against that file whenever the file changes**, and an entry
found discharged records the commit that discharged it. The trigger is the
edit, not the edition. A register whose entries are true only of a state
nobody has checked since is worse than no register, because it sends a reader
to correct what is already correct and spends the standing of the entries that
are still owed.

## The commits named below

Seven commits are named in the statuses, and each is named by its short hash
alone after this table.

| Commit | Date | Subject |
|---|---|---|
| `3f65b5f` | 2026-09-09 | docs: describe the tpl CLI and its project conventions |
| `1352a2d` | 2026-09-09 | docs(specification): specify the complete CLI surface |
| `011c059` | 2026-09-10 | docs(specification): support a window of MariaDB series, not a floor |
| `0ea5624` | 2026-09-10 | docs: restructure CLAUDE.md around the specification |
| `50153d6` | 2026-09-11 | docs(claude): reduce four passages to citations of the records |
| `26e1739` | 2026-09-11 | docs(readme): name the five TLS modes and the default in force |
| `87dd6e3` | 2026-09-11 | docs: describe the repository the coordination documents actually have |

`3f65b5f` wrote both root documents and is the state every entry of the first
edition was raised against. `0ea5624` reduced `CLAUDE.md` to agent
coordination; it is the commit behind twelve discharges and nineteen half
discharges, and it is the reason this file's remaining work is almost entirely
`README.md`'s.

## Index

| Id | Target | Kind | Status | Subject |
|---|---|---|---|---|
| [DIV-001](#div-001) | both | Migration | Partly, `README.md` | All CLI-surface content moves here |
| [DIV-002](#div-002) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | "No command accepts a password in argv" |
| [DIV-003](#div-003) | `README.md` | Contradiction | Due | The `--password` / `-p` flag |
| [DIV-004](#div-004) | both | Contradiction | Partly, `README.md` | `--output`, `--output-dir` and the file-writing surface |
| [DIV-005](#div-005) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Only `init` and `cfg` write inside `.tpl/` |
| [DIV-006](#div-006) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Configuration printing always redacts |
| [DIV-007](#div-007) | both | Contradiction | Partly, `README.md` | `EPIPE` always exits `0` |
| [DIV-008](#div-008) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `${VAR}` expands in any string value |
| [DIV-009](#div-009) | both | Contradiction | Partly, `README.md` | `schema dump` equals the render context |
| [DIV-010](#div-010) | `README.md` | Contradiction | Due | TTY-dependent `--format` default |
| [DIV-011](#div-011) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Positional render target |
| [DIV-012](#div-012) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `tpl init` takes no argument |
| [DIV-013](#div-013) | both | Contradiction | Partly, `README.md` | `password_command` is a shell command |
| [DIV-014](#div-014) | `README.md` | Contradiction | Due | Aliases `procs` and `proc` |
| [DIV-015](#div-015) | both | Contradiction | Partly, `README.md` | The `database` and `config` command groups |
| [DIV-016](#div-016) | both | Contradiction | Partly, `README.md` | `--no-color` and `NO_COLOR` |
| [DIV-017](#div-017) | both | Contradiction | Partly, `README.md` | `TPL_DIR` and `TPL_DATABASE` |
| [DIV-018](#div-018) | `README.md` | Contradiction | Partly, `README.md` | Three TLS modes, defaulting to `preferred` |
| [DIV-019](#div-019) | both | Contradiction | Partly, `README.md` | `--all-tables` and `--pattern` on `render` |
| [DIV-020](#div-020) | both | Contradiction | Partly, `README.md` | The `.tpl/` layout omits `.cache/` |
| [DIV-021](#div-021) | `README.md` | Contradiction | Due | `template check` lints |
| [DIV-022](#div-022) | `README.md` | Contradiction | Due | `--pattern` follows the server collation |
| [DIV-023](#div-023) | both | Migration | Partly, `README.md` | Global flag tables are wrong in three ways |
| [DIV-024](#div-024) | `CLAUDE.md` | Migration | Discharged, `0ea5624` | Project discovery lacks its boundary and checks |
| [DIV-025](#div-025) | `CLAUDE.md` | Migration | Discharged, never owed | The "specification does not exist yet" note |
| [DIV-026](#div-026) | both | Contradiction | Partly, `README.md` | The `rust_type` and `go_type` filters |
| [DIV-027](#div-027) | both | Contradiction | Partly, `README.md` | The `plural` and `singular` filters |
| [DIV-028](#div-028) | both | Contradiction | Partly, `README.md` | Auto-escaping keyed on the file extension |
| [DIV-029](#div-029) | both | Contradiction | Partly, `README.md` | `tpl init` creates four artefacts |
| [DIV-030](#div-030) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | The read-only session presented as prevention |
| [DIV-031](#div-031) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `SHOW` as a permitted way to read the catalogue |
| [DIV-032](#div-032) | `CLAUDE.md` | Contradiction | Due | `model/` as the documented public surface |
| [DIV-033](#div-033) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | The whole `Environment` surface as contract |
| [DIV-034](#div-034) | both | Contradiction | Partly, `README.md` | The `table` and `column` field lists |
| [DIV-035](#div-035) | `CLAUDE.md` | Migration | Discharged, `0ea5624` | The performance budget table |
| [DIV-036](#div-036) | `CLAUDE.md` | Migration | Partly, `CLAUDE.md` | `scripts/mariadb/` lacks the benchmark fixture `seed-bench.sql` |
| [DIV-037](#div-037) | both | Contradiction | Partly, both | A withdrawn MariaDB floor, no ceiling, and no refusal of MySQL |
| [DIV-038](#div-038) | both | Migration | Partly, `README.md` | Routine naming has no disambiguator |
| [DIV-039](#div-039) | both | Contradiction | Partly, `README.md` | Determinism stated over all output |
| [DIV-040](#div-040) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `tpl cache` is absent, and the auxiliary set is closed |
| [DIV-041](#div-041) | `CLAUDE.md` | Migration | Due | The target matrix is deferred; this specification now fixes it, and Linux is `musl` |
| [DIV-042](#div-042) | `README.md` | Contradiction | Due | The JSON error envelope, the `kind` field, and `did_you_mean` |
| [DIV-043](#div-043) | `README.md` | Contradiction | Due | The `.cfg` is now read strictly; an unrecognised key is fatal |
| [DIV-044](#div-044) | `README.md` | Contradiction | Due | The four fields of `tpl schema info` |
| [DIV-045](#div-045) | `CLAUDE.md` | Contradiction | Discharged, ninth edition | The release profile aborts on panic |
| [DIV-046](#div-046) | `README.md` | Overstatement | Due | The build sequences, in a repository with no crate |
| [DIV-047](#div-047) | `README.md` | Contradiction | Due | The `render` flag table omits `--direct` and `--no-cache` |
| [DIV-048](#div-048) | `README.md` | Contradiction | Due | The entry flag is `--database`, which is the global flag's name |
| [DIV-049](#div-049) | `README.md` | Contradiction | Due | The entry flag table omits `--ca-file` and `--ca-path` |
| [DIV-050](#div-050) | `CLAUDE.md` | Overstatement | Due | The project tree, in a repository with no crate |
| [DIV-051](#div-051) | `CLAUDE.md` | Overstatement | Due | The eight `cargo` commands, in a repository with no crate |
| [DIV-052](#div-052) | `CLAUDE.md` | Overstatement | Due | The benchmark directory, in a repository with no benchmarks |

One passage of `CLAUDE.md` was read and found not to be a divergence, so it
has no entry and no row above. The reading and its grounds are at
*[A candidate read, and not recorded](#a-candidate-read-and-not-recorded)*,
at the foot of this file, with the sweep that closes the **Overstatement**
class in that file.

## DIV-001

**Target**: both. **Kind**: migration.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`: that file no longer describes the three arms, the CLI
surface, the `.tpl` project, the exit codes or the error-message format, and
defers each to a named file of this folder. The `README.md` half is due in
full — its command reference, configuration reference, exit code table and
error-message section all stand.



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

**Status**: discharged by `0ea5624`, which removed the sentence quoted below
with the CLI design rules that carried it. Nothing is owed.



*Says*: "Nenhum comando aceita password em `argv`."
*Specification*: `FR-CFG-030` removes the flag, but `FR-CFG-031` and
`FR-CFG-032` keep two documented paths open — `--dsn` and `tpl cfg set` — with a
warning in the help rather than a refusal.
*Correction*: rewrite the sentence to say that no flag named `password` exists,
and that two documented paths remain open.

## DIV-003

**Target**: `README.md`, the flag tables of `database add` and `update` and of
`render`. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. `README.md` still carries the
`--password` row and all four short forms.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the output flags and the exit code
table that attached `73` and `74` to a destination. The `README.md` half is
due: all five flags, the atomic-write paragraph and the "console by default"
framing stand.



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

**Status**: discharged by `0ea5624`, which removed the `.tpl` project section.
Nothing is owed.



*Says*: `tpl init`, `tpl database …` and `tpl config …` are the only commands
that write in `.tpl/`; all others treat it as read-only.
*Specification*: `FR-PROJ-023` — `tpl cache load` and every cached read command
also write to `.tpl/.cache/` on a miss.
*Correction*: rewrite the sentence to list the four writers.

## DIV-006

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the secrets and versioning
section. Nothing is owed.



*Says*: any command that prints configuration always redacts passwords.
*Specification*: `FR-CFG-021` redacts in `cfg list` and `cfg database show`;
`FR-CFG-006` and `BR-CFG-002` make `tpl cfg get` a deliberate exception.
*Correction*: state the exception rather than leave it to be discovered.

## DIV-007

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the exit code section that stated the
flat rule. The `README.md` half is due: the sentence stands under its exit
code table.



*Says*: `EPIPE` on stdout exits `0` silently, in every case.
*Specification*: `FR-ERR-025` keeps `0` for the ordinary case, and `FR-ERR-026`
returns `74` when the pipe closes part-way through a JSON document.
*Correction*: refine the flat rule in both files.

## DIV-008

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the secrets and versioning
section. Nothing is owed.



*Says*: `${VAR}` is substituted "em qualquer valor string".
*Specification*: `FR-CONF-015` limits expansion to six fields, and
`FR-CONF-016` and `FR-CONF-017` forbid it in `tls` and `password_command`.
*Correction*: narrow the statement to the enumerated field set.

## DIV-009

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due, and stands twice: under
`tpl schema dump` and again in *Rendering without a database*.



*Says*: `tpl schema dump` produces exactly the JSON the render receives as
context.
*Specification*: `FR-SCH-018` — the dump carries only the server-derived part;
`FR-RND-024` shows that `vars`, `tpl`, and `now` are always injected by the
render.
*Correction*: restate the round-trip as "the dump supplies the server-derived
part of the context", in both files.

## DIV-010

**Target**: `README.md`, global flags. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The TTY clause stands in the global
flag table.



*Says*: `--format` "Defaults to `text` on a TTY, `json` otherwise".
*Specification*: `FR-OUT-001` and `FR-OUT-002` — the default is `text`, fixed,
with no terminal detection anywhere.
*Correction*: change the default and remove the TTY clause.

## DIV-011

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the porcelain command list.
Nothing is owed.



*Says*: `tpl render <template> [alvo] [flags]` — a positional render target.
*Specification*: `FR-RND-003` — the target is always a flag, and the template
name is the only positional.
*Correction*: change the usage line.

## DIV-012

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the porcelain command list.
Nothing is owed.



*Says*: `tpl init` with no argument.
*Specification*: `FR-PROJ-012` — an optional positional path, defaulting to the
current directory.
*Correction*: add the optional argument.

## DIV-013

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half,
recorded as discharged by the fifth edition, was discharged by `0ea5624`,
which removed the `.cfg` example; the commit is named here because the edition
that closed the half did not name it. The `README.md` half is due: the word
"shell" and the string form of `password_command` both stand.



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

**Status**: due, checked on 2026-09-11. Both alias rows stand.



*Says*: `routines` aliases to `procs`, `routine` to `proc`.
*Specification*: `FR-CLI-011` — `rtns` and `rtn`, because a routine is a
procedure or a function.
*Correction*: change both rows.

## DIV-015

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: the `database` and
`config` groups and `tpl database test` in the quick start all stand.



*Says*: a top-level `tpl database …` group with alias `db`, a separate
`tpl config …` group, and `tpl database test` in the quick start.
*Specification*: `FR-CFG-001` through `FR-CFG-003` — one `tpl cfg` group, with a
`database` subgroup aliased `db`, and `tpl cfg database test`.
*Correction*: rewrite both command lists and the quick start.

## DIV-016

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: the flag row and the
variable both stand.



*Says*: a `--no-color` global flag, colour implied off when stdout is not a TTY
or `NO_COLOR` is set.
*Specification*: `NFR-DET-004` — no colour anywhere, so the flag does not exist
and the variable is not read.
*Correction*: remove the flag row and the variable from both files.

## DIV-017

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: both variables and the
three-layer precedence sentence stand.



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

**Status**: partly discharged, checked on 2026-09-11. The flag row is
discharged by `26e1739`, which replaced the three-mode enumeration and
`preferred`. **The `.cfg`-example half is due**: the example carries neither
`ca_file` nor `ca_path`. The `CLAUDE.md` illustration this entry also names is
moot — it went with the help-text rules in `0ea5624` — but `CLAUDE.md` is not
this entry's **Target** and nothing is owed there.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the performance budget that measured
`render --all-tables`. The `README.md` half is due: the two flags and the
multi-render examples stand.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the `.tpl` tree. The `README.md` half
is due: the tree omits `.cache/` and the `.gitignore` is still shown with one
line.



*Says*: the `.tpl/` tree contains `.cfg`, `.gitignore`, and `templates/`, and
the generated `.gitignore` holds one line.
*Specification*: `FR-PROJ-002` and `FR-PROJ-017` — the tree also contains
`.cache/`, and the generated `.gitignore` holds two lines.
*Correction*: update both trees and the `.gitignore` content in both files.

## DIV-021

**Target**: `README.md`, second arm. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The word "lint" and the required
argument both stand, and `tpl template path` is still shown without one.



*Says*: `tpl template check <name>` "Parse and lint a template", with a required
name.
*Specification*: `FR-TMPL-017` through `FR-TMPL-019` — `check` parses only,
there is no lint, and the name is optional and repeatable.
*Correction*: remove the word "lint" and make the argument optional. The same
applies to `tpl template path`, which also takes an optional name per
`FR-TMPL-021`.

## DIV-022

**Target**: `README.md`, `--pattern`. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The sentence stands under the alias
table.



*Says*: "Case sensitivity follows the server collation."
*Specification*: `FR-SCH-013` and `FR-SCH-014` — evaluated locally, ASCII
case-insensitive, independent of the server.
*Correction*: replace the sentence.

## DIV-023

**Target**: both. **Kind**: migration.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed its global flag table. The `README.md`
half is due: its table is wrong in the same three ways it was.



Both global flag tables are wrong in three ways at once: they list `--output`,
`--format`, and `--no-color` as global; they omit `--timeout`; and they do not
mention that `--direct` and `--no-cache` exist at all.
*Specification*: `FR-GLOB-001` — exactly seven global flags — and `FR-GLOB-021`,
which lists the four local flags and the commands that declare each.
*Correction*: replace both tables with a pointer to
[global-flags.md](global-flags.md).

## DIV-024

**Target**: `CLAUDE.md`, project discovery. **Kind**: migration.

**Status**: discharged by `0ea5624`, which replaced the description of the
walk with a pointer to [project-and-discovery.md](project-and-discovery.md) —
the correction this entry asked for, made as it asked for it. Nothing is owed.



*Says*: the walk climbs until a `.tpl/` folder is found, and `TPL_DIR` skips
discovery.
*Specification*: `FR-PROJ-005` adds the boundary at the mount point;
`FR-PROJ-009` through `FR-PROJ-011` add canonicalisation and the ownership and
mode checks; `FR-GLOB-009` replaces `TPL_DIR` with `--tpl-dir`.
*Correction*: the description is incomplete rather than wrong; replace it with a
pointer to [project-and-discovery.md](project-and-discovery.md).

*Amended in the eighth edition.* This entry named a boundary at the home
directory beside the mount point. `FR-PROJ-005` drops it, because locating it
required reading `HOME` and `FR-CLI-021` admits no such read. The correction
owed to `CLAUDE.md` is unchanged in kind and shorter by one clause.

## DIV-025

**Target**: `CLAUDE.md`, functional specification section. **Kind**: migration.

**Status**: discharged, and no commit discharged it. The clause quoted below
is not in `CLAUDE.md` today and is in no committed state of it: searching the
history for the quoted words finds one occurrence in the repository, in this
file, at `1352a2d`, where this entry was written. The subsection it asks to
remove therefore never existed in the file, and nothing is owed. Recorded
rather than deleted, because an entry raised against an unverified reading of
a target is the same defect as an entry left standing after the reading went
stale, and the identifier must resolve to that explanation.



*Says*: "A pasta `/specification` ainda não existe neste repositório", with an
instruction to remove the subsection once the bootstrap is done.
*Specification*: the folder now exists, and this file is part of it.
*Correction*: remove the subsection, as that text itself instructs.

## DIV-026

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: the filter table names
both filters and two examples call `rust_type`.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: the row stands in the
filter table.



*Says*: both files list `plural` and `singular` as naming filters.
*Specification*: `FR-ENV-012` and `FR-ENV-013` — neither exists. Correct English
inflection is a project in itself, and a wrong plural on a table name that is
not English is guaranteed noise in generated code.
*Correction*: remove the row from both tables.

## DIV-028

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`. The `README.md` half is due: the extension rule and
the sentence that frames it as one of two differences from stock Jinja2 both
stand.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the artefact table and the tree. The
`README.md` half is due: four artefacts are listed and the fifth is absent
from both the table and the tree.



*Says*: both files present `tpl init` as creating four artefacts, and list them.
*Specification*: `FR-PROJ-017`, as amended — five artefacts. The fifth is
`.tpl/templates/rust/_types.jinja`.
*Correction*: add the row to both tables, and add the file to the `.tpl` tree
shown in each. `README.md` additionally states that `.tpl/.gitignore` holds one
line, which `DIV-020` already corrects to two.

## DIV-030

**Target**: `CLAUDE.md`, project invariant 1. **Kind**: contradiction.

**Status**: discharged by `0ea5624`. The promise survives in `CLAUDE.md`, but
the session statement no longer follows it: the detail is deferred to
[server-contract.md](server-contract.md), which is the correction this entry
asked for. Nothing is owed.



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

**Status**: discharged by `0ea5624`. **This is the entry that showed the
register was unsafe to presume current.** The `SHOW` clause was removed
several sprints before the eleventh edition amended this entry for a miscount,
and the amendment revisited the count without checking whether the clause it
corrected was still in the file. Nothing is owed, and nothing was owed then
either.



*Says*: the catalogue is read exclusively through `INFORMATION_SCHEMA` "e,
quando estritamente necessário, `SHOW`".
*Specification*: `FR-SRV-006` and `FR-SRV-007` — the closed list has four
entries and `SHOW` is not one of them. A statement outside the list is not
permitted however necessary it seems.
*Correction*: remove the `SHOW` clause. The prohibition on `mysqldump` and on
any external process is unchanged and is restated by `FR-SRV-007`.

*Amended in the eleventh edition.* The clause above said **three** entries. It
was written in the second edition, when the list had three; the fifth edition
added a fourth — the read-back of the session read-only variable, which the
tenth edition named `@@session.tx_read_only` — and this entry was not revisited
with it. `FR-SRV-006` carries four rows, counted in its own table on
2026-09-11, and `FR-SRV-012` requires a test that expects four kinds of
statement and no fifth. **Only the count changes.** `SHOW` is no more one of
the four than it was one of the three, and the correction owed to `CLAUDE.md`
is the one stated above and nothing more.

## DIV-032

**Target**: `CLAUDE.md`, project structure. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The sentence stands, unchanged since
`3f65b5f`, below the project tree.



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

**Status**: discharged by `0ea5624`, which removed the render context section
and replaced it with a pointer to
[template-environment.md](template-environment.md) — one of the two
corrections this entry offered. Nothing is owed.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the render context table. The
`README.md` half is due in full, including the three further corrections the
seventh edition added and the exclusion count the eighth corrected.



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
`table_name` per `FR-CTX-019`. Sixteen volatile catalogue fields are excluded
outright by `FR-CAT-024`.
*Correction*: replace both descriptions with a pointer to
[catalogue-coverage.md](catalogue-coverage.md) and
[context-document.md](context-document.md).

*Amended in the seventh edition, and the correction is now cheap to make.*
When this entry was written the specification could not offer a complete
replacement, because the field lists themselves were unobserved. They are
observed and written: `FR-CAT-039` through `FR-CAT-051` fix the catalogue
field list of every object kind and what the model takes from each, and
`BR-CAT-005` states the rule by which one becomes the other. Three further
corrections join the ones above and each is a fact the root document states
wrongly rather than incompletely: a table carries **no character set**, only
a collation, per `FR-SCH-009` as amended; a column's `default` has **three**
`kind` values and not four, per `FR-CTX-013`; and a table's comment for a
**view** is the literal string `VIEW`, which `FR-CAT-040` keeps out of the
model. The exclusion list of `FR-CAT-024` grew from thirteen fields to
sixteen in the same edition.

*Amended in the eighth edition.* The *Specification* clause above still said
thirteen while the note beside it said sixteen. `FR-CAT-024` carries sixteen
rows, counted in its own table on 2026-09-10; the clause is corrected and the
two now agree.

## DIV-035

**Target**: `CLAUDE.md`, non-functional requirements. **Kind**: migration.

**Status**: discharged by `0ea5624`, which replaced the budget table with the
statement that the numeric targets do not live in that file and a pointer to
[performance-requirements.md](performance-requirements.md). The fifth
edition's requirement that `CLAUDE.md` keep no figure at all is met. Nothing
is owed.



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

**Status**: partly discharged, checked on 2026-09-11. The project-tree half is
discharged by `87dd6e3`: the tree's fixture line now describes what
`scripts/mariadb/` holds instead of enumerating its files, so there is no list
for `seed-bench.sql` to be missing from. **Two parts are due**, and neither is
a correction to prose alone: the coverage paragraph of the testing section
still names `setup.sql` and `seed.sql` and no third script, and
`scripts/mariadb/` holds no `seed-bench.sql`. The five budgets of `WL-001`
wait on the file, not on the sentence.



*Says*: `scripts/mariadb/` holds a `Dockerfile`, `setup.sql`, and `seed.sql`,
and the two SQL scripts must cover the read surface exhaustively.
*Specification*: `WL-001` requires a fourth file, `seed-bench.sql`, and
`BR-PERF-002` keeps it separate from `seed.sql` on purpose: `seed.sql` is
exhaustive variety at minimal volume, for correctness, and `seed-bench.sql` is
volume at minimal variety, for measurement. One fixture serving both would hide
an N+1, which is invisible at ten tables.
*Correction*: add `seed-bench.sql` to the tree and to the testing section.
*Amended in the sixth edition, and largely discharged.* Three of the four
files now exist and the container is buildable at each of the four series of
`FR-SRV-015`, so the two larger parts of this entry are settled: `CLAUDE.md`
no longer describes the container in the singular, and the DDL of `setup.sql`
and `seed.sql` is DDL all four accept. What is still owed is one line of the
project-structure tree and one of the testing section, both naming
`seed-bench.sql`, and the file itself.

This entry previously recorded that none of the four files existed and that
**all twenty-four remaining open questions were blocked by that absence**.
Neither half survives. The fixture exists; a first pass against it closed five
entries; a second pass recorded the field lists themselves and closed twenty
more; and the last entry, `OQ-042`, closed on a stated limit in `FR-SRV-041`
rather than on evidence, because the server it wanted is not one a fixture of
MariaDB servers can hold. **No open question is blocked by this entry, and
none is open at all.** The mandated tests of `BR-SCH-004`, `FR-SRV-029` and
`BR-SEC-003` are not blocked by this entry either, and of the five budgets that
needed a fixture, those over `WL-001` still need `seed-bench.sql`.

*Corrected in the twenty-sixth edition.* The sentence above said those three
tests were "blocked only by `tpl` not existing". The binary exists, so the
clause named a condition that does not hold, and what this entry is entitled to
say about the three is only that its own absence does not block them: what each
of them waits on is the command it drives. `BR-SCH-004` records its own block,
in [schema-commands.md](schema-commands.md), and the same claim in the
*Maintenance debt* section of [README.md](README.md) is corrected with it. The
debt of this entry is unchanged — `seed-bench.sql`, and the two lines that name
it.

*Amended in the seventh edition.* What this entry still owes is unchanged and
is now the whole of it: one line of the project-structure tree, one line of
the testing section, and the `seed-bench.sql` file itself.

*Related, and not covering it: `DIV-052`.* The *Disciplina de medição*
subsection of the same file states that the benchmarks run against the
containers' dataset, which is in part the fixture this entry still owes: the
budgets over `WL-001` wait on `seed-bench.sql`, per `BR-PERF-007`. The two
entries are not one. This one is a **migration** whose correction is an
addition — the file, and the two lines that name it — and applied exactly as
written it leaves that subsection saying benchmarks live in `benches/`, a
directory the repository has not got. `DIV-052` is an **overstatement** whose
correction is a qualifier on a tense, and it produces no fixture. Neither
discharges the other, and the link is written in both directions so that
whoever pays one is told the other still stands.

## DIV-037

**Target**: both. **Kind**: contradiction.

**Status**: partly discharged, checked on 2026-09-11, and both targets owe
something. The `README.md` version row is discharged by `011c059`, which
replaced `MariaDB 10.6 or later` with the three families and a citation of
`FR-SRV-001` — the form `BR-SRV-005` requires, and not a second copy of the
table. `CLAUDE.md`'s silence on the window is discharged by `0ea5624`, which
defers the supported series to [server-contract.md](server-contract.md). **Two
parts are due.** `README.md` still says the generated example template renders
without error against any table of any MariaDB database, which claims a server
this specification refuses. And `CLAUDE.md` still observes that MariaDB and
MySQL diverge in the catalogue without saying that a server which is not
MariaDB is refused with `78`, which is the half of this entry that was never
about a version number.



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
refused with `78`, carrying the message of `FR-SRV-030`. `FR-SRV-003` — a
server that is not MariaDB is refused with `78`, its `cause` naming the product
the server reported, because three verified divergences would make the model
silently wrong rather than empty.

*Amended in the sixth edition*: this entry cited `kind: server_version_unsupported`
and `kind: server_not_mariadb` as the specification's position. `FR-ERR-015`
withdrew the `kind` field in the fifth edition and the two owning requirements
were amended with it; this entry was missed.
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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the command list. The `README.md` half
is due: `tpl schema routine`, `tpl render --routine` and the exit-code
discussion are all as written.



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

**Status**: partly discharged, checked on 2026-09-11. The `CLAUDE.md` half is
discharged by `0ea5624`, which removed the determinism section. The
`README.md` half is due: the unqualified sentence stands under *Built for
coding agents*.



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

**Status**: discharged by `0ea5624`, which removed the three arms and the
porcelain command list together. `tpl cache` is absent from `CLAUDE.md` still,
but so is every other command, and the sentence that closed the auxiliary set
with "apenas" is gone. Nothing is owed.



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

**Status**: due, checked on 2026-09-11. The deferral stands verbatim under
*Plataformas Suportadas*, added by `0ea5624` itself — the commit that
discharged twelve other entries outright wrote this one's subject in. The two
rules beside it also still restate what `NFR-PERF-018` and `NFR-PERF-012`
carry.



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

**Status**: due, checked on 2026-09-11. The sentence, the JSON block and the
sentence about `kind` all stand. The *Note on scope* below is confirmed: the
envelope is in `README.md` alone.



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

**Status**: due, checked on 2026-09-11. The `.cfg` example is unchanged since
`3f65b5f`, and neither of the two statements the correction asks for has been
added.



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

## DIV-044

**Target**: `README.md`, the command-surface listing. **Kind**:
contradiction.

**Status**: due, checked on 2026-09-11. The line stands in the command-surface
listing.



*Says*: `tpl schema info` reports "Database metadata: name, version, charset,
collation".

*Specification*: three of the four are confirmed and the fourth does not
exist. `FR-CTX-036` fixes the metadata fields of the `database` object as
`name`, `charset` and `collation`, observed against the schema catalogue on
all four series of `FR-SRV-015`. There is **no `version` field**: the server
version reaches the document as the `server` **object** of `FR-CTX-031`,
carrying three keys — `version`, `series`, and `standing` — of which the last
two have no counterpart in the root document at all. A caller that reads
`data.database.version` finds nothing there, and `FR-SEM-012` fails a render
that reads it.

*Correction*: change the line to name `name`, `charset` and `collation`, and
either drop `version` or replace it with a pointer to the `server` object of
[context-document.md](context-document.md). The listing is the first place a
caller looks for the shape of that document.

## DIV-045

**Target**: `CLAUDE.md`, the release profile. **Kind**: contradiction.

**Status**: discharged in the ninth edition, by amendment rather than by an
edit to `CLAUDE.md`: nothing was ever owed, so no commit could discharge it.
Recorded here for completeness, and with one observation the re-check turned
up: the profile table this entry quotes is itself gone, reduced to a citation
of `ADR-004` by `50153d6`. The *Says* clause below therefore no longer matches
the file, which changes nothing about an entry that owes nothing.



*Says*: the release profile is tuned in `Cargo.toml` with `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`, and `opt-level = 3`.

*Specification*: `FR-ERR-030` — `70` (`EX_SOFTWARE`) is produced by exactly
two conditions, one of which is **a panic in the process**; `FR-ERR-032`
requires that `70` to carry the four labelled lines of `FR-ERR-008`; and
`FR-ERR-001` makes the code table contract. What `panic = "abort"` settles is
how a panic ends the process, not whether the process may report it first. By
**default** it ends it abnormally: the caller receives no code of
`FR-ERR-001` and no message, which removes one of the two producing conditions
of a contractual code in the only artefact a caller ever runs. That default is
not the whole of what the setting admits. A process that installs its own
handler for a panic writes the message and exits with the status it chooses,
before the aborting runtime is reached, and the requirement is then met under
the profile exactly as written.

*Correction*: none is owed to the profile table. The five settings stand as
written, and `FR-ERR-030` as amended in the ninth edition obliges the process
to report a panic and exit `70` without obliging the panic path to be
catchable.

*Why this was recorded rather than resolved, and how it closed.* The eighth
edition reconciled three contradictions **within** this corpus, and this one
was not within it: nothing in `/specification` states a release profile, and
the choice between an unwinding profile and a narrower promise for `70` was an
architecture decision with a cost this corpus cannot weigh. The decision came
back narrower than either option — the profile stands and the requirement
states an outcome rather than a mechanism — and the ninth edition writes the
amendment in.

*Amended in the ninth edition, and this entry is discharged.* The
*Specification* clause rested on a premise that was too strong. It said without
qualification that under `panic = "abort"` "a panic terminates the process
abnormally, the caller receives no code of `FR-ERR-001`, and no message is
written"; that is true of the **default** behaviour of such a profile and not
of a process that handles the panic itself. The clause is corrected and now
separates the two. The *Correction* clause is corrected with it: it required
the profile to leave the panic path catchable, and offered as its alternative
that `FR-ERR-030` drop the caught-panic condition and state the resulting
limit. Neither is what happened. `FR-ERR-030` keeps both producing conditions
and states the first as the outcome a caller observes — the message and the
code — which the profile as written can produce, so the two statements this
entry refused to leave standing together no longer conflict. Nothing is owed to
`CLAUDE.md` under this entry. The identifier is retained rather than removed,
so that a reference written before the ninth edition resolves to this
explanation.

## DIV-046

**Target**: `README.md`, *Installation* and *Development*. **Kind**:
overstatement.

**Status**: due, checked on 2026-09-11. Both sequences stand, and the
repository holds no crate: a search of the working tree finds no `Cargo.toml`
and no `Cargo.lock`, and none of `src/`, `tests/` or `benches/`.



*Says*: *Installation* gives `git clone`, `cd tpl`, and `cargo build
--release`, and states that "the binary is produced at `target/release/tpl`".
*Development* gives `cargo build`, `cargo run -- --help` and `cargo test`,
followed by the five-command validation pipeline that opens with `cargo fmt`.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder: `DIV-001`
leaves *Installation* to `README.md` as part of its role as the entry door, so
the section belongs where it is and only its content is false. It is false of
the repository rather than of a requirement — `cargo build --release` fails
for want of a manifest, and every command of *Development* fails with it.
*Correction*: say that the crate does not exist yet, at the head of both
sequences or in place of them. The document-scope banner at the top of the file
does not do it. That banner says no command described below is implemented yet,
which a reader takes to mean the binary builds and the commands are not
finished; it does not say there is nothing to build. *Installation* is the
first instruction in the file a reader acts on, so it is the first place the
file spends its credit, and *Development* is where a contributor goes next.

*Why this entry names two sections.* The survey that raised it named
*Installation* alone. *Development* states the same thing about the same file
under a second heading, and an entry that corrected one would leave a reader
running `cargo fmt --all -- --check` against a directory with no manifest. The
two are one correction and are recorded as one.

*One observation that this entry does not own, and where it now lives.* The
project-structure tree of `CLAUDE.md` also names `Cargo.toml`, `src/`,
`tests/`, `benches/`, `templates/` and `examples/`, none of which the
repository has. `CLAUDE.md` is not this entry's **Target** and nothing is owed
there under it, so the divergence was named here only to keep it from being
lost between the two files. It has since been raised against that file as
`DIV-050`, in the same edition. The two are one divergence written twice, and
neither discharges the other: each is corrected by an edit to its own file,
under its own authorisation, so a reader who corrects both sequences of
`README.md` leaves the tree of `CLAUDE.md` still saying the crate is there.

*The same commands stand in `CLAUDE.md`, under `DIV-051`.* That file's
*Desenvolvimento* section and the validation pipeline under it give the same
three commands and the same five, and they are recorded against that file as an
entry of their own, in the same edition and for the reason written there. The
relation is the one above: neither entry discharges the other, and correcting
*Development* here leaves the other file's copy standing.

## DIV-047

**Target**: `README.md`, the flag table of `render`. **Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The table stands with twelve rows, and
neither flag is among them.



*Says*: the flags of `tpl render` are `--table`, `--view`, `--routine`,
`--all-tables`, `--pattern`, `--set`, `--context`, `--output`, `--output-dir`,
`--output-name`, `--no-clobber` and `--dry-run`.
*Specification*: `FR-GLOB-021` — `--direct` and `--no-cache` are declared by
`render`, by the eight `schema` subcommands and by `cache load` — and
`FR-GLOB-022`, which requires each declaring command to list both in its own
help and in its `options` array in the JSON command tree. A table that
presents itself as the flag list of one command and omits two flags that
command declares tells a reader those two would be rejected there under
`FR-CLI-019`. They are declared, so they are accepted; what the omission costs
is a caller who never reaches for them.
*Correction*: add a row for `--direct` and one for `--no-cache`, or replace the
table with a pointer to [global-flags.md](global-flags.md) and
[render-command.md](render-command.md). Seven of the twelve rows are removed
outright by `DIV-004` and `DIV-019`, so the table is being rewritten in any
case.

*Not covered by `DIV-023`.* That entry records the same two flags as missing,
but from the **global** flag tables, and its correction replaces those tables
with a pointer to [global-flags.md](global-flags.md). Applied exactly as
written it leaves this table untouched and still silent: a pointer put where
the global table stood says nothing about a local table further down the file,
and the reader who consults a command's own flag list is the one this entry is
about.

## DIV-048

**Target**: `README.md`, the flag table of `database add` and `update`.
**Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The row stands, spelled `--database`.



*Says*: `--database`, "Database name on the server", among the flags of
`database add` and `database update`. The group is spelled `tpl database …`
there rather than `tpl cfg database …`, which is `DIV-015`'s subject and not
this entry's.
*Specification*: `FR-CFG-027` names that flag `--schema`, and `FR-CFG-028`
records it as the only flag of the tool whose name differs from the key it
writes — `database.<name>.database` — and gives the reason. The name the table
uses is already taken and means something else: `FR-GLOB-001` declares
`-d/--database` as one of the seven global flags, and `FR-GLOB-004` gives it
the `.cfg` entry label, not the database on the server. `FR-GLOB-002` makes
every global flag acceptable at every node, so the two definitions meet on the
very two commands this table describes, and `FR-CFG-028` states in as many
words that they cannot coexist at one node.
*Correction*: rename the row to `--schema` and keep its description, which is
right. This is not a synonym a reader may pick either way. `tpl cfg database
add shop --database shop` written from this table is accepted, because the
global flag is declared at that node, and `FR-GLOB-004` reads the value as an
entry label — so the invocation silently fails to write the key the table
promised, and writes nothing wrong enough to be refused.

## DIV-049

**Target**: `README.md`, the flag table of `database add` and `update`.
**Kind**: contradiction.

**Status**: due, checked on 2026-09-11. The table stands with eight rows, and
neither flag is among them.



*Says*: the flags of `add` and `update` are `--dsn`, `--host`, `--port`,
`--user`, `--password`, `--password-command`, `--database` and `--tls`.
*Specification*: `FR-CFG-027` declares nine flags for those two commands, and
the two the table has no row for are `--ca-file`, writing
`database.<name>.ca_file`, and `--ca-path`, writing `database.<name>.ca_path`.
The fifth edition added the last three rows of that mapping to close `OQ-017`,
on the ground that registering an entry against a private certificate
authority otherwise took three invocations where the mapping promised one; the
table carries the first of the three and neither of the others. `FR-CFG-047`
adds that none of the three carries a short form.
*Correction*: add a row for `--ca-file` and one for `--ca-path`. The table's
own `--tls` row makes the omission cost more than it looks: that row already
names `verify-ca` and `verify-identity`, and `FR-CONF-014` makes those the two
modes the trust material serves, so the table sends a reader to a mode and
withholds the flags that configure it.

*Not covered by `DIV-003` or `DIV-018`.* `DIV-003`'s correction does name
`--ca-file` and `--ca-path`, among "the other flags of those two commands",
but it names them only to say that none of them carries a short form and asks
for nothing to be added. `DIV-018` asks for `ca_file` and `ca_path` to be
added to the `.cfg` example, which is the configuration file and not this
table; that half of `DIV-018` is still due, and discharging it in full would
leave this entry standing.

## DIV-050

**Target**: `CLAUDE.md`, *Estrutura do Projecto*. **Kind**: overstatement.

**Status**: due, checked on 2026-09-11 at `87dd6e3` — the last commit to touch
either root document, and the commit that last rewrote this very tree. Six of
the tree's lines name artefacts the working tree does not hold: `Cargo.toml`,
`src/` with all seven of its children, `templates/`, `tests/`, `benches/` and
`examples/`. The five that remain — `BENCHMARKS.md`, `knowledge-model.md`,
`scripts/mariadb/`, `specification/` and `docs/` — all exist, and nothing is
owed for them.



*Says*: a fenced tree headed `tpl/`, listing `Cargo.toml`; `src/` with
`main.rs`, `cli/`, `project/`, `mariadb/`, `model/`, `render/` and `error.rs`;
`templates/`, `tests/`, `benches/` and `examples/`, each with a comment naming
what it holds — starter templates, end-to-end CLI integration tests,
benchmarks, complete pipelines; and `BENCHMARKS.md`, `knowledge-model.md`,
`scripts/mariadb/`, `specification/` and `docs/`. Nothing in the block or
around it carries a tense or a marker: it reads as the contents of the
directory, and the paragraph directly under it speaks of `model/` in the
present indicative.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. `DIV-001`
leaves this section where it is — it reduces `CLAUDE.md` to agent coordination
and names implementation conventions among the content it does not touch — so
the section belongs to that file exactly as *Installation* belongs to
`README.md` under `DIV-046`, and only its tense is false. It is false of the
repository rather than of a requirement, and false to the reader that file
declares for itself: an agent that takes the tree for the map of what it may
open finds nothing at any of the six paths. `87dd6e3` makes that reading the
natural one, because it shows the tree being maintained against the
repository — three artefacts were added to it because they exist — which tells
a reader the list is kept current.
*Correction*: say at the head of the tree that no code artefact exists yet and
that what follows is the structure the project will have, or mark the lines
that are already there and leave the rest unmarked. **Neither the tree nor the
decomposition under `src/` is to be removed**, and that distinction is the
whole of this entry.

*Planned decomposition and assertion of present state.* The six lines do two
different jobs. `Cargo.toml`, `templates/`, `tests/`, `benches/` and
`examples/` say the project will be an ordinary Rust project, and cost a reader
nothing beyond the paths that are not there. The seven lines under `src/` are a
decision: they are the module decomposition, they tell an implementer where
each responsibility goes before any of it is written, and a document whose
stated job is to say how work is executed here is the right place for them.
That decomposition is an architecture decision, recorded as one outside this
corpus, and this entry takes no position on its content — only on the tense it
is stated in. Removing the six lines would therefore make the document worse in
the name of making it true: the project would lose, from the file every agent
reads first, its only statement of where code belongs, and the next
implementer would invent a structure rather than follow one. What the section
may not do is state a plan in the indicative of the present. A direction and a
claim about the working tree are different things, and the tree is read as the
second because nothing in the section says it is the first. That is why the
correction above is a qualifier and not a deletion.

*There is no banner to argue about here.* `DIV-046` had to explain why
`README.md`'s document-scope banner does not discharge the same defect in that
file: it says no command described below is implemented yet, which a reader
takes to mean the binary builds and the commands are unfinished. `CLAUDE.md`
carries no equivalent. It opens by saying that it is agent coordination and
that it holds no functional requirements, and it says nothing anywhere about
how much of the repository exists. Every statement it makes about the working
tree is therefore taken at face value, and the qualifier this entry asks for
has nothing standing in for it.

*The same divergence as `DIV-046`, and not a widening of it.* That entry
records the build sequences of `README.md` in a repository with no crate; this
one records the tree of `CLAUDE.md` in the same repository. It is one disease
in two files, and the two are recorded apart because **Target** is the
authority on where to look when an entry is re-checked: `DIV-046` was raised by
a reading of `README.md` alone, and widening its target to `both` would have
made it assert of `CLAUDE.md` what nobody had read there. The consequence is
that neither entry discharges the other. Each is corrected by an edit to its
own file under its own authorisation, and the link is written in both
directions so that whoever holds the pen on one file is told the other is still
saying it.

*Not covered by `DIV-032` or `DIV-036`.* Both name the project structure of
`CLAUDE.md` in their **Target**, and both are the nearest thing to an entry
that already reads this section. Neither reaches the tree's six false lines.
`DIV-032` is about the sentence below the tree, which calls the structs of
`model/` the documented public surface; it contradicts a requirement, its
correction rewrites that sentence, and applied exactly as written it leaves the
tree as it stands. `DIV-036` did target the tree, and asked for a line to be
**added** to it — `seed-bench.sql`, which the fixture line then enumerated
around — and `87dd6e3` discharged that half by making the line describe the
directory instead of listing it. So the tree has been read against this corpus
under an entry and corrected under one, and both times the subject was a line
that ought to be there rather than the six that ought not to be stated in the
present. An addition and a tense are different corrections, and neither entry
carries the other.

*One observation that this entry does not own.* *Desenvolvimento* gives `cargo
build`, `cargo run -- --help` and `cargo test`, and the mandatory validation
pipeline under it adds `cargo fmt --all -- --check`, `cargo clippy`, `cargo
build --release`, `cargo test --all-features` and `cargo audit` — eight
commands, every one of which fails in this repository for want of a manifest,
which is what `DIV-046` records of *Development* in the other file. It is not
recorded under this entry: the reading that raised this one was authorised over
the project tree, and an entry that reached past its own reading is the defect
the paragraph above keeps `DIV-046` clear of. `DIV-001` leaves the validation
pipeline with `CLAUDE.md`, so that section belongs where it is too, and as here
only its tense would be in question. It needed an entry of its own, and has
since been raised as `DIV-051`, in the same edition; the reason the two are
recorded apart is written there.

## DIV-051

**Target**: `CLAUDE.md`, *Desenvolvimento* and the *Pipeline de validação
obrigatório* under it. **Kind**: overstatement.

**Status**: due, checked on 2026-09-11 at `87dd6e3` — still the last commit to
touch either root document. Both fenced blocks stand, and that commit left them
untouched while it rewrote the project tree two sections above. The
repository holds no crate: a search of the working tree finds no `Cargo.toml`
at any depth and no `Cargo.lock`, and none of `src/`, `tests/`, `benches/`,
`templates/`, `examples/` or `target/`. Every one of the eight commands fails
for want of a manifest.



*Says*: *Desenvolvimento* opens with a fenced block giving `cargo build`,
`cargo run -- --help` and `cargo test`. The subsection under it, *Pipeline de
validação obrigatório*, states that no work is complete until every command
below it passes, in the order given, and gives five:
`cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --release`, `cargo test --all-features` and `cargo audit`.
Neither block carries a tense or a marker, and the sentence between them states
an obligation in the present indicative.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. `DIV-001`
leaves the validation pipeline with `CLAUDE.md` — it names the pipeline among
the content that file keeps — so the section belongs where it is, exactly as
*Installation* belongs to `README.md` under `DIV-046`, and only its tense is
false. It is false of the repository rather than of a requirement, and false to
the reader that file declares for itself: an agent that runs the pipeline
before reporting a task complete gets five failures that say nothing about the
work it did, and one that follows the section above it gets the other three. The cost is larger here than in a passage a reader merely
consults, because the sentence between the blocks is a gate on work in the
present. Four sprints of work have been closed in this repository, and the gate
as written could not have been passed in any of them.
*Correction*: say at the head of the section that the crate does not exist yet
and that the commands become runnable with it, or mark the two blocks. **The
eight commands are not to be removed, and neither is the obligation stated
between them.** They are the right commands, they are this file's to state
under `DIV-001`, and they run unchanged the moment the manifest exists. What is
wrong is the tense and not the substance — which is the whole of this entry, as
the same distinction is the whole of `DIV-050`.

*Why this is an entry of its own and not a widening of `DIV-050`.* Both entries
are raised against `CLAUDE.md`, both record the same fact about the repository,
and both ask for a qualifier rather than a deletion, so the question is whether
they are one correction the way `DIV-046`'s two sections are. They are not, on
three grounds.

First, the criterion `DIV-046` used is the correction and not the
neighbourhood. Its two sections are about four hundred lines apart in
`README.md` — *Installation* near the head of the file and *Development* near
its foot — so proximity is not what joined them. What joined them is that both
are sequences of build commands, and one sentence written at the head of each
discharges both.

Second, the correction asked for here is not the one `DIV-050` asks for. That
entry qualifies a tree of paths and says in as many words that neither the tree
nor the decomposition under `src/` may be removed, because the decomposition is
an architecture decision the document is entitled to state. This entry
qualifies two blocks of commands and the obligation stated between them. An
editor who applies `DIV-050` exactly as written — a line at the head of the
tree, or a mark on the lines that are already there — leaves the eight commands
where they are, under a heading and a subheading that entry does not name, and
leaves a rule that gates the completion of work on commands that cannot run.

Third, the two passages are edited apart and must therefore be re-checked
apart. **Target** is the authority on where to look when an entry is
re-checked, and `87dd6e3` is the demonstration: it rewrote the tree, adding
three artefacts because they exist, and did not touch *Desenvolvimento*. An
entry whose target named both sections would have been re-read against that
commit and recorded as partly discharged on the strength of an edit that did
nothing for the commands.

What a merge would buy is that whoever holds the pen on `CLAUDE.md` sees one
item instead of two, and both corrections are likely to be made in one commit.
That is not enough to merge them. The index groups by **Target**, so a reader
working through what `CLAUDE.md` owes sees both entries together and loses
nothing; while an entry carrying two corrections of different shapes under one
status is what the *partly discharged* machinery exists to describe, and
describing as one thing what is two makes the register a worse work list rather
than a shorter one.

The reading that raised `DIV-050` was authorised over the project tree and
named this passage without recording it, which left the placement open rather
than settled. It is settled here, by a reading of the section itself and on the
three grounds above.

*The same eight commands as `DIV-046` records of `README.md`.* *Development* in
that file gives `cargo build`, `cargo run -- --help` and `cargo test`, and then
the same five-command pipeline under the sentence *Before any change is
considered complete*. The two passages are one list written twice, once in each
root document, under headings that are each other's translation. This is the
relation `DIV-046` and `DIV-050` already carry, and it holds the same way here:
neither entry discharges the other, because each is corrected by an edit to its
own file under its own authorisation, so a reader who qualifies *Development*
leaves `CLAUDE.md` still telling an agent that no work is complete until eight
commands pass that cannot run. The link is written in both directions.

*Three passages elsewhere in the file cite the pipeline and need nothing.* The
rules table at the head of `CLAUDE.md` makes it a rule that the mandatory
pipeline passes before work is complete; the platforms section requires what
passes it to pass on every supported target; and the code-conventions section
names `rustfmt` and `clippy` as the arbiters that run in it. Each states an
obligation and names the section that holds the commands, and none of them says
anything about the working tree, so the qualifier this entry asks for at the
head of that section reaches them where they stand. It is the reading `DIV-050`
gives the lines of the tree it leaves alone: a direction and a claim about the
working tree are different things, and only the second is false.

*One observation that this entry does not own.* The *Disciplina de medição*
subsection states that the benchmarks live in `benches/` and run against the
dataset of the MariaDB containers, so as to be reproducible. `benches/` is one
of the six paths `DIV-050` records as absent, and there are no benchmarks to
live in it; the sentence is the same class of claim as the two blocks above,
in the present indicative with nothing around it to carry a tense. It is not
recorded under this entry: the reading that raised this one was authorised over
the eight commands, and reaching past its own reading is the defect `DIV-046`
and `DIV-050` each keep themselves clear of. No entry covers it. `DIV-035` was
the performance budget table and is discharged; `DIV-036` targets the project
tree and the MariaDB testing section, and what it still owes is
`seed-bench.sql` and the sentence that ought to name it. It needs an entry of
its own, and has since been raised as `DIV-052`, in the same edition; the three
grounds on which it is an entry apart from this one, rather than a widening of
it, are written there.

## DIV-052

**Target**: `CLAUDE.md`, the *Disciplina de medição* subsection of *Desempenho
e Eficiência*. **Kind**: overstatement.

**Status**: due, checked on 2026-09-11 at `87dd6e3` — still the last commit to
touch either root document, and one that rewrote the project tree three
sections above while leaving this subsection alone. The sentence stands, and
there is no benchmark to live where it says benchmarks live: a search of the
working tree
finds no `benches/`, no `Cargo.toml` at any depth and no `Cargo.lock`, and none
of the seventy-one files the repository tracks is a benchmark of any kind.
`BENCHMARKS.md` does not stand in for them. It holds two measurement campaigns,
neither run from `benches/`: both measured probe binaries this repository does
not hold — driver candidates, and a defect in one of them — and that file says
of itself that no figure in it is a baseline for `tpl`.



*Says*: the subsection opens with the rule that there is no performance claim
without numbers and no optimisation without a measurement before and after,
gives a table assigning a tool to each kind of measurement, and then states:
"Os benchmarks vivem em `benches/` e correm contra o dataset dos containers
MariaDB, para serem reproduzíveis. As baselines são registadas em
`BENCHMARKS.md`, identificando o alvo em que foram medidas; **uma regressão
face à baseline reprova a alteração** e tem de ser justificada ou corrigida
antes de o trabalho ser dado por concluído." Nothing in the subsection carries
a tense or a marker.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. What the
subsection states is in force, and it states it correctly: `NFR-PERF-009`
through `NFR-PERF-013` fix the measurement protocol, `NFR-PERF-012` obliges a
measurement to name the target it was taken on, and `NFR-PERF-017` fails the
change that produces a measurement worse than the baseline recorded for the
same budget on the same target. Where a benchmark lives is no requirement of
this corpus — it names no directory — and `DIV-035` left the discipline with
`CLAUDE.md` when it took the figures out of that file, so the subsection
belongs where it is, exactly as *Installation* belongs to `README.md` under
`DIV-046`. Only the tense of one clause is false, and it is false of the
repository rather than of a requirement: an agent that reads the sentence for
what to re-run before claiming a speed-up is sent to a directory that is not
there, and one that reads it for what produced the baselines in
`BENCHMARKS.md` is told something that file denies of itself.
*Correction*: say at the head of the subsection that no benchmark exists yet
and that the sentence says where they will live, or mark that one sentence.
**Neither the discipline nor the location is to be removed.** No claim without
numbers, the baseline recorded in `BENCHMARKS.md` against a named target, and a
regression failing the change are obligations in force today — they governed
both campaigns `BENCHMARKS.md` already holds, and each of those entries says
where it departs from the normative protocol — and `benches/` is where a
benchmark should go the day one is written. What is wrong is the assertion that
benchmarks are there, and nothing else in the subsection.

*Why this is an entry of its own and not a widening of `DIV-051`.* The
criterion is the one `DIV-051` settled: **the shape of the correction decides
where an entry goes, and not the proximity of the passages.** It reached that
by reading `DIV-046`, whose two sections lie about four hundred lines apart in
`README.md`, and finding that what joined them was not neighbourhood but that
one sentence written at the head of each discharges both. This entry is the
same criterion applied against the grain. *Disciplina de medição* is close to
the two blocks `DIV-051` records — a subsection of the very next top-level
section, some fifty lines below the head of *Desenvolvimento* — so proximity
argues for a merge here as loudly as it argued against one there, and it is not
what decides.

What decides is that no one correction serves both. A qualifier at the head of
*Desenvolvimento*, which is what `DIV-051` asks for, does not reach a
subsection of *Desempenho e Eficiência*, and an editor who writes it leaves a
reader in the next section still told that benchmarks live in a directory
nothing has said is absent. The two corrections also discharge on different
conditions. `DIV-051`'s eight commands run unchanged the moment the manifest
exists, and that entry says so in as many words; this sentence does not become
true with a manifest. A benchmark has to be written, and what it is said to run
against is not complete: `WL-001` is realised by
`scripts/mariadb/seed-bench.sql`, which does not exist, and `BR-PERF-007` in
[performance-requirements.md](performance-requirements.md) records that the
budgets over it cannot be measured until it does. One entry carrying both would
be settled in one half by an event that leaves the other waiting on two more.

Third, the two passages are edited apart and must be re-checked apart, which is
the ground `DIV-051` gives against `DIV-050` and which holds here on the same
demonstration: `87dd6e3` rewrote the project tree, adding three artefacts to it
because they exist, and touched neither *Desenvolvimento* nor this subsection.
**Target** is the authority on where to look when an entry is re-checked, and
an entry naming two sections that move independently records a discharge it has
not had.

*Where the entry sits in this file.* It takes the next identifier and stands at
the foot, in identifier order, rather than beside the entries it resembles. An
identifier is stable once assigned and is cited from other entries, so the
order of this file is the order the entries were raised in; the Index is what
groups them, and a reader working through what `CLAUDE.md` owes reads it by
**Target**. That is the reason `DIV-051` gave for declining to merge into
`DIV-050`, and it applies to position as it applies to merging: the index
already puts the four entries against that file in one place, so nothing is
bought by moving this one next to them.

*The entry `DIV-051` said this passage needed.* That entry records the eight
`cargo` commands of this same file and closes by naming this sentence as an
observation it does not own, because the reading that raised it was authorised
over the commands, and reaching past its own reading is the defect `DIV-046`
and `DIV-050` each keep clear of. It also did the checking this entry rests
on — that no entry covers the sentence, that `DIV-035` is discharged, and that
`DIV-036` targets other sections — and left the entry to be raised. This is
that entry, and the link is written in both directions. Neither discharges the
other, for the three grounds above.

*The relation to `DIV-036`, which owes the fixture this sentence invokes.*
`DIV-036` is the nearest entry by subject. It is partly discharged, and what it
still owes is `scripts/mariadb/seed-bench.sql`, one line of the project tree
and one of the MariaDB testing section naming it. That file is what `WL-001` is
realised by, and `BR-PERF-002` keeps it separate from `seed.sql` on purpose, so
the dataset this sentence promises reproducibility against is in part the one
`DIV-036` still owes. The two do not cover each other in either direction.
`DIV-036` is a **migration** and its correction is an **addition** — a file,
and the two lines that name it — and applied exactly as written it leaves this
sentence saying that benchmarks live in `benches/`; this entry's correction is
a qualifier and produces no fixture. An addition, a tense and a missing file
are three corrections, and only the second is this entry's. What the relation
does establish is that this entry is not discharged by a manifest alone: the
sentence becomes true when benchmarks exist, and a benchmark over `WL-001`
becomes possible when `DIV-036` is paid.

*Not covered by `DIV-035` or `DIV-050` either.* `DIV-035` is the entry that
last read this part of the file: it moved the four-row budget table out of
`CLAUDE.md` and is discharged by `0ea5624`, which replaced the table with a
pointer to [performance-requirements.md](performance-requirements.md). Its
subject was the figures, its correction was made, and a discharged entry owes
nothing; the discipline it deliberately left behind is what this entry finds a
false tense in front of. `DIV-050` records `benches/` as one of the six lines
of the project tree the repository does not have, which is the same absence
seen from the other end, but its correction is a qualifier on the tree or a
mark on the lines already there, and applied exactly as written it leaves a
subsection a hundred lines below asserting that the directory holds something.
An absent path and a claim about what it holds are two statements, and marking
the first does not correct the second.

*What the sweep of this file found, and the one thing it does not settle.*
This entry's passage was given to it; the rest of `CLAUDE.md` was read to
answer whether any passage of this kind is left, because a claim that a class
is exhausted is worth only the sweep behind it. Every other path, file and
artefact that file names exists: the twenty-four files of this folder its
subject table points to and the `README.md` here it starts from,
`BENCHMARKS.md`, `knowledge-model.md`, `docs/adr/` with the decisions its Stack
table cites, and `scripts/mariadb/` with the fixture, the TLS material and the
harness its testing section describes. The
passages that state `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, the
release profile, the module conventions and the separation of library from
binary assert nothing about the working tree: each directs how code is to be
written, and a direction and a claim about the working tree are different
things, which is the reading `DIV-050` gives the lines of the tree it leaves
alone. The tool table directly above this entry's sentence is of the same sort,
and so is the overview, which says what `tpl` is rather than what the
repository holds.

**One candidate stands, and this entry does not record it.** *Plataformas
Suportadas* says "Os sistemas suportados e verificados são o **Linux** e o
**macOS**", and says of the other Unixes that they "não entram na matriz, não
correm em validação", which presupposes that the four targets do. Nothing has
been verified on any target, because the validation that would do it is the
pipeline `DIV-051` records as unable to run at all. It is one adjective and one
clause inside a declaration of policy, not a block a reader executes, and
whether *suportados e verificados* asserts a state or names a class is a
judgement for a reading of that section — which `DIV-041` targets on other
grounds, for its deferral of the target matrix, and does not reach. It is named
here and not recorded, on the rule the three entries before this one each kept.
**So the class is not closed in this file by this entry**: four of its passages
are recorded, and one candidate is left for a reading of its own.

*That reading was made in the seventeenth edition* and is recorded at
*[A candidate read, and not recorded](#a-candidate-read-and-not-recorded)*.
It finds the candidate declares a policy rather than asserting a state, so no
entry is owed for it, and it closes the class in this file by a sweep of its
own rather than by this entry.

## A candidate read, and not recorded

The sixteenth edition's sweep of `CLAUDE.md` named one passage and did not
record it, on the ground that the judgement it needed was a reading of the
section it sits in. This is that reading, made in the seventeenth edition.
**It finds that the passage declares an intention rather than asserting a
state, so there is no entry to open.** The finding is written here, beside the
entries, because a conclusion that nothing is owed leaves no trace otherwise
and the question is re-opened from scratch by the next reader — which is the
failure the fifth validation rule of [README.md](README.md#maintenance-debt)
was written against, seen from its other end.

*The passage.* *Plataformas Suportadas* states that "Os sistemas suportados e
**verificados** são o **Linux** e o **macOS**, nas arquitecturas **arm64** e
**amd64**", and says of the other Unixes that they "devem funcionar ... mas
**não são testados nem garantidos**: não entram na matriz, não correm em
validação, e um problema que só neles se manifeste não reprova uma alteração."
The second clause presupposes that the four targets do enter the matrix and do
run in validation. **Nothing has been run on any target.** The validation that
would do it is the five-command pipeline `DIV-051` records as unable to run at
all for want of a manifest, and `docs/adr/adr-008-packaging-and-build-path.md`
records separately that no `musl` artefact has been run outside a container and
marks that expectation **unverified**.

*The judgement rests on four grounds.*

**First, the section says two paragraphs later that the matrix is not
decided.** "A matriz concreta de alvos — target triples, escolha de libc,
linkagem e forma de empacotar o binário — é decisão de arquitectura em curso e
**não se fixa aqui**." A verification is of an artefact on a target, and a
target is a triple with a libc and a linkage. A section that declines to name
the triples cannot, earlier in itself, be reporting that artefacts were
verified on them. The state reading makes the section contradict itself; the
policy reading makes it coherent.

**Second, the presupposition sits inside a rule, and the rule's payload is its
third limb.** "Não entram na matriz, não correm em validação, e um problema que
só neles se manifeste não reprova uma alteração" is one thought in three parts,
and the third states what fails a change. A statement of what gates a change is
deontic throughout; the two limbs before it state the policy that yields the
third, not a history of runs.

**Third, the section says in its first sentence what it is for**, and it is not
reporting: "É essa a família de sistemas para que se escreve, e é a fronteira
que delimita o que o código pode assumir." Every other sentence does that job —
Windows is out of scope, the BSDs are not guaranteed, the triples are
deferred — and the subsection under it is six rules in the deontic. So does the
adjective: it marks the four as the systems that carry the project's
undertaking of verification, against the Unixes that carry none, which is the
contrast the sentence is built on. Read as a report it would be a report naming
no target, no date, no artefact and no result, which is not a report.

**Fourth, this corpus states verification in the same mode, in the requirement
the passage would be measured against.** `NFR-PERF-005` in
[performance-requirements.md](performance-requirements.md) says of a clause
of itself that it "SHALL be verified on every target of `NFR-PERF-018`", and
the eleventh-edition note
under `NFR-PERF-018` states that "every requirement of this file is still
verified on all four targets" — written when nothing had been run, and true
today for the same reason, because what it states is the reach of the
verification obligation and not a history of runs. This corpus cannot record as
an overstatement in a document it does not own a mode of statement it uses
itself, in force, in the requirement that document's sentence would be checked
against.

*Why the decision record does not carry the other way.* `ADR-008` marks the
expectation unverified, and if `CLAUDE.md` asserted a verification the two
would contradict each other on a matter of fact. That record does not read the
file that way. It calls the five-command sequence one that the root
coordination document **mandates**, and lists the no-second-class rule of
`NFR-PERF-018` among four obligations **carried by hand** until a pipeline
exists — the deontic reading, taken by the record best placed to take it. What
it marks unverified is also narrower than the passage: that no `musl` artefact
has run outside a container, and that the two `x86_64` targets have never been
measured. Both are claims about baselines under `NFR-PERF-012`, and the
sentence of `CLAUDE.md` that carries that rule is in another section, where
`DIV-041` has already read it and found it correct.

*What this kind is, and why the passage is not of it.* The **Overstatement**
kind states its own test in the Overview: nothing is owed to this folder, no
requirement is contradicted, and what is wrong is that **a reader who acts on
the passage fails**. The four entries that carry the kind each name something
to act on — a build sequence that fails, a tree of paths that open onto
nothing, eight commands that cannot run, a directory that is not there. This
passage names no path, no command and no artefact. There is nothing in it to
act on, and what a state reading of it would produce is a false belief rather
than a failed action. Manufacturing an entry for it would put a **Status** of
*due* on a correction nobody can make and weaken the four that stand.

*The relation to `DIV-041`, which targets the same section.* That entry is a
**migration** and its subject is the deferral of the target matrix: its *Says*
quotes the deferral sentence and the two rules beside it, its correction
replaces the deferral with a pointer to `NFR-PERF-018`, and nothing in it turns
on whether verification has happened. It does not reach the adjective, and the
adjective does not reach it. **What this reading owes `DIV-041` is one
caution.** The correction that entry asks for fixes the matrix in the section,
which removes the first of the four grounds above; whoever makes it re-reads
the adjective in the section it leaves behind, because a sentence calling four
named triples *verificados* is a stronger claim than the one read here, and it
would be read against a repository in which the pipeline may by then run on
some targets and not others.

*One observation this reading does not own.* The sentence naming Linux and
macOS on two architectures states, coarsely, what `NFR-PERF-018` now fixes
exactly, and `DIV-041`'s correction names the deferral and the two rules beside
it rather than that sentence. Whether a coarse statement of the supported
systems left standing beside a pointer to `NFR-PERF-018` is two sources for one
truth is a question about that entry's **Target** and **Kind**, not about this
one: this reading was authorised over the adjective and over the sweep below,
and reaching past its own reading is the defect the four entries of the kind
each keep clear of. It is named here so that it is not lost.

*The class is exhausted in `CLAUDE.md`, and this is the sweep behind the
claim.* The whole file was read again against the question rather than the
sixteenth edition's answer taken on trust, for the reason that edition gave:
a claim that a class is exhausted is worth only the sweep behind it, and the
sixteenth edition's own sweep is what found the candidate the fifteenth had
missed. Every path, file and artefact `CLAUDE.md` names was tested against the
working tree as it stands at `87dd6e3`, still the last commit to touch either
root document. All exist but the ones already recorded: the twenty-four files
of this folder its subject table points to, the `README.md` of this folder it
starts from, `BENCHMARKS.md`, `knowledge-model.md`, `docs/adr/`, the `OD-09`,
`OD-17` and `OD-24` of `docs/spec-technical/open-decisions.md` that its Stack
table cites, and `scripts/mariadb/` with its Dockerfile, `setup.sql`,
`seed.sql` and TLS material. What is absent is `Cargo.toml`, `src/` and its
seven children, `templates/`, `tests/`, `benches/` and `examples/` —
`DIV-050` for the tree and for the paragraph under it that speaks of `model/`
in the present indicative, which that entry's *Says* already names, `DIV-051`
for the eight commands, and `DIV-052` for the benchmark directory. `DIV-032`
reads the same paragraph on other grounds, where it calls the structs of
`model/` the documented public surface. The passages stating
`#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, the release profile, the
module conventions and the separation of library from binary direct how code is
to be written and assert nothing about the working tree, which is the reading
`DIV-050` gives the lines of the tree it leaves alone.

**Two further passages were read on their own and are not of the kind.**

- **The changelog.** Step 4 of *Fluxo de Trabalho* says to update "README, doc
  comments e CHANGELOG", and *Documentação* names a `CHANGELOG` among the
  documents written in English. The repository holds no `CHANGELOG.md`. Both
  passages are directions rather than claims: step 4 is reached only after step
  2 has written code, and the other is a rule about the language all project
  documentation is written in. Neither sends a reader to a path. The record
  outside this corpus reads them the same way — `OD-03` of
  `docs/spec-technical/open-decisions.md` says `CLAUDE.md` "names one in its
  workflow" and settles only its format, and
  `docs/spec-technical/data-model.md` records that neither `Cargo.toml` nor
  `CHANGELOG.md` exists at this commit and that the rows resting on them are
  prescribed and not yet observable.
- **The routing sentence of *Desempenho e Eficiência*.** It states that the
  required properties live in
  [performance-requirements.md](performance-requirements.md) and that "as
  baselines efectivamente medidas vivem em `BENCHMARKS.md`". It lies outside
  the subsection `DIV-052` targets and was therefore read on its own. It names
  a file that exists and describes it as that file describes itself — the
  register of measured baselines for `tpl`, carrying only what was actually
  measured — so it sends a reader somewhere real and tells them truly what is
  there. That no figure in it is yet a baseline for `tpl` is that file's own
  statement, made entry by entry; the sentence of `CLAUDE.md` that does assert
  benchmarks exist is in the *Disciplina de medição* subsection below it, and
  is `DIV-052`'s.

**So the class is closed in `CLAUDE.md`**: four passages are recorded there,
`DIV-046` holds the one in the root `README.md`, and the candidate the
sixteenth edition left is read here and is not one. The claim is true of the
file at `87dd6e3` and of no later state of it: by the fifth validation rule it
stops being true the moment the file is edited, and the sweep is owed again
then.
