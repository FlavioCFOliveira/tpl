---
title: Open Questions
status: draft
last-reviewed: 2026-09-10
related: [README.md, catalogue-coverage.md, context-document.md, performance-requirements.md]
---

# Open Questions

## Overview

Every entry here is a point this specification cannot yet fix. Each was left
open deliberately during the interview that settled the command surface, or is a
gap this edition found while writing. None is resolved by inference: an open
question stays open until the user settles it or a measurement answers it.

An entry states the question, where it came from, why it cannot be answered now,
and what it blocks. When one is closed, the requirement it produces is written
into the owning module and the entry is removed from this file.

Three things about this list are specific to the second edition.

First, `OQ-025` through `OQ-042` are not design questions at all. They are
eighteen facts about what a MariaDB server actually returns, and the project
forbids documenting catalogue behaviour that has not been observed. None can be
closed by reasoning, by analogy with MySQL, or by reading documentation alone,
and all eighteen are blocked by the same thing: `scripts/mariadb/` does not
exist in this repository, so there is no container to observe against. The same
absence blocks `OQ-009`, `OQ-010`, and `OQ-024`.

Second, `OQ-051` through `OQ-060` are not questions either. Each is a number
that must be measured and cannot be measured yet, because there is no code.
They are recorded here rather than guessed at in
[performance-requirements.md](performance-requirements.md), which states what is
observable now and cites these entries for what is not.

Third, sixteen entries have been closed and are listed under *Closed* below
rather than removed without trace. `OQ-020` was answered by the second edition;
twelve were answered by the third, which put the audit's findings to the user
and wrote the decisions into the owning modules; and three — `OQ-044`, `OQ-073`
and `OQ-074` — by the fourth. Their identifiers are not reused, per the
identifier scheme of the [README](README.md#identifier-scheme).

Fourth, `OQ-070` through `OQ-072` are what remained of `OQ-068` after the third
edition specified the behaviour of the registered filters and tests. Two are
facts about MariaDB that must be observed and one is a design decision the
specification declined to invent.

Fifth, the fourth edition opened three questions and closed all three within the
same edition, so none appears in the index below. `OQ-044` — the outcome below
the supported window — was answered by `FR-SRV-020`. `OQ-073` and `OQ-074` were
the two the window itself created: the outcome above the ceiling, and the reach
of the check that enforces it. Both were put to the user as escalations and both
came back decided, as `FR-SRV-031` and `FR-SRV-034`. Their entries are in the
*Closed* table, not above it; the identifiers are retired all the same.

## Index

| Id | Question | Owner module | Blocks |
|---|---|---|---|
| [OQ-002](#oq-002) | Whether `test` reports effective privileges | `cfg-commands.md` | Implementation of `test` |
| [OQ-003](#oq-003) | TLS mode to driver mapping | `configuration-model.md` | Freezing TLS semantics |
| [OQ-004](#oq-004) | Enumerated DSN query parameters | `configuration-model.md` | DSN validation |
| [OQ-005](#oq-005) | Cap on `password_command` output | `configuration-model.md` | `password_command` execution |
| [OQ-006](#oq-006) | Handling of the child's stderr | `configuration-model.md` | `password_command` execution |
| [OQ-007](#oq-007) | Handling of a non-zero child exit | `configuration-model.md` | `password_command` execution |
| [OQ-009](#oq-009) | Catalogue return types and collation | `schema-commands.md` | Freezing catalogue value types |
| [OQ-010](#oq-010) | Field lists for routines, triggers, generated columns, FK rules | `schema-commands.md` | `schema table` and `schema routine` |
| [OQ-011](#oq-011) | Pre-scan on ambiguous `--format` forms | `errors-and-exit-codes.md` | Error formatting |
| [OQ-012](#oq-012) | Mandatory password sentinel test | `security.md` | The test plan |
| [OQ-013](#oq-013) | Ordering of `tpl template list` | `template-commands.md` | Determinism of the listing |
| [OQ-014](#oq-014) | Nested command path for `tpl help` | `help-and-version.md` | `help` argument parsing |
| [OQ-015](#oq-015) | Position of a global flag | `global-flags.md` | Argument parsing |
| [OQ-016](#oq-016) | Short forms for local flags | `cfg-commands.md` | Flag declarations |
| [OQ-017](#oq-017) | Flags for `password_command`, `ca_file`, `ca_path` | `cfg-commands.md` | `cfg database add` and `update` |
| [OQ-018](#oq-018) | Unknown key in a hand-written `.cfg` | `configuration-model.md` | `.cfg` validation |
| [OQ-019](#oq-019) | `password_command` written as a string in the file | `configuration-model.md` | `.cfg` validation |
| [OQ-021](#oq-021) | Exit code for a failed cache write | `cache-commands.md` | Cache write path |
| [OQ-023](#oq-023) | Pre-scan on a command without `--format` | `errors-and-exit-codes.md` | Error formatting for `render` |
| [OQ-024](#oq-024) | Field list of `tpl schema info` | `schema-commands.md` | `schema info` |
| [OQ-025](#oq-025) | Table types reported, and temporary tables | `catalogue-coverage.md` | `FR-CAT-001`, `FR-CAT-006` |
| [OQ-026](#oq-026) | Column attribute strings for auto-increment and invisibility | `catalogue-coverage.md` | `FR-CAT-027`, `FR-CTX-020` |
| [OQ-027](#oq-027) | Literal, expression, and explicit `DEFAULT NULL` | `context-document.md` | `FR-CTX-012` |
| [OQ-028](#oq-028) | Textual form of `column_type` | `context-document.md` | `FR-CTX-014` |
| [OQ-029](#oq-029) | `ENUM` and `SET` member delimiting and escaping | `context-document.md` | `FR-CTX-016` |
| [OQ-030](#oq-030) | Which type parts the catalogue populates | `context-document.md` | `FR-CTX-015` |
| [OQ-031](#oq-031) | Per-column character set and collation | `context-document.md` | `FR-CTX-015` |
| [OQ-032](#oq-032) | Index fields and the folding order column | `catalogue-coverage.md` | `FR-CAT-010` |
| [OQ-033](#oq-033) | How the primary key is reported | `catalogue-coverage.md` | `FR-CAT-011` |
| [OQ-034](#oq-034) | Foreign-key fields and rule spellings | `catalogue-coverage.md` | `FR-CAT-012`, `FR-CAT-013` |
| [OQ-035](#oq-035) | `CHECK` constraint fields and level values | `catalogue-coverage.md` | `FR-CAT-015` |
| [OQ-036](#oq-036) | View fields beyond the definition | `catalogue-coverage.md` | `FR-CAT-007` |
| [OQ-037](#oq-037) | Routine fields, and body readability | `catalogue-coverage.md` | `FR-CAT-016`, `FR-CAT-017` |
| [OQ-038](#oq-038) | Routine parameter fields and ordering | `catalogue-coverage.md` | `FR-CAT-018` |
| [OQ-039](#oq-039) | Trigger fields | `catalogue-coverage.md` | `FR-CAT-014` |
| [OQ-040](#oq-040) | Generated-column expression and storage kind | `catalogue-coverage.md` | `FR-CAT-009` |
| [OQ-041](#oq-041) | How an unreadable view is reported | `privileges-and-completeness.md` | `FR-PRIV-011` |
| [OQ-042](#oq-042) | The version probe and MariaDB detection | `server-contract.md` | `FR-SRV-002`, `FR-SRV-003` |
| [OQ-043](#oq-043) | Whether `referenced_by` embeds | `context-document.md` | `FR-CTX-010` |
| [OQ-045](#oq-045) | Which fields differ among the four supported series | `server-contract.md` | `FR-SRV-004`, `FR-SRV-024`, `FR-SRV-027` |
| [OQ-046](#oq-046) | Read-back within the closed statement list | `server-contract.md` | `FR-SRV-009` |
| [OQ-047](#oq-047) | Shape of the `restricted` field | `privileges-and-completeness.md` | `FR-PRIV-005` |
| [OQ-048](#oq-048) | Whether an incomplete object may be cached | `cache-documents.md` | `FR-CACHE-007` |
| [OQ-049](#oq-049) | The pinned engine minor version | `template-environment.md` | `FR-ENV-003` |
| [OQ-050](#oq-050) | Contract group and behaviour of `escape` | `template-environment.md` | `FR-ENV-028` |
| [OQ-051](#oq-051) | Budget: `tpl --version` | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-052](#oq-052) | Budget: `tpl --help` | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-053](#oq-053) | Budget: startup to the first useful byte | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-054](#oq-054) | Budget: `tpl schema dump` over `WL-001` | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-055](#oq-055) | Budget: cache-served read over `WL-003` | `performance-requirements.md` | `NFR-PERF-015` |
| [OQ-056](#oq-056) | Budget: the failure path | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-057](#oq-057) | Budget: `tpl help --format json` | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-058](#oq-058) | Budget: the 200-invocation loop | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-059](#oq-059) | Budget: peak resident memory over `WL-001` | `performance-requirements.md` | `NFR-PERF-014` |
| [OQ-060](#oq-060) | The byte scalar N of `WL-002` | `performance-requirements.md` | `WL-002` |
| [OQ-070](#oq-070) | The escaping of a backtick inside an identifier | `template-environment.md` | `FR-ENV-035` |
| [OQ-071](#oq-071) | What `sql_type` does, beyond `column_type` | `template-environment.md` | `FR-ENV-039` |
| [OQ-072](#oq-072) | The membership of the three type families | `template-environment.md` | `FR-ENV-041`, `FR-ENV-042` |

## Closed

An entry is closed when the requirement it produced is written into the owning
module. The identifier is retired and never reused, so a gap in the sequence
above is expected rather than a defect.

| Id | Question | Answered by | Closed in |
|---|---|---|---|
| OQ-001 | Output of `tpl cfg database test` | `FR-CFG-035`, `FR-CFG-039` | Third edition |
| OQ-008 | `--timeout` versus the per-phase keys | `FR-GLOB-011`, `FR-GLOB-012`, `FR-GLOB-013`, `FR-CONF-004` | Third edition |
| OQ-020 | Cache format version, and the cached-versus-live field | `FR-CDOC-001` through `FR-CDOC-005`, `FR-CDOC-009` through `FR-CDOC-011` | Second edition |
| OQ-022 | Shape of `tpl cache status` output | `FR-CACHE-034`, `FR-CACHE-035` | Third edition |
| OQ-061 | Whether `--help` runs before project discovery | `FR-PROJ-025`, `NFR-PERF-005`, `FR-ERR-006` | Third edition |
| OQ-062 | The `json` document shape of every read command | `FR-OUT-024` through `FR-OUT-032`, and the `data` shapes of `BR-OUT-002` | Third edition |
| OQ-063 | The order of the collections `NFR-DET-002` did not name | `NFR-DET-002` | Third edition |
| OQ-064 | The outcome of a read whose result set is empty | `FR-OUT-033` through `FR-OUT-037` | Third edition |
| OQ-065 | The durability of a `.tpl/.cfg` rewrite | `FR-CFG-041`, `FR-CFG-042` | Third edition |
| OQ-066 | `password` beside `password_command`, and either beside `dsn` | `FR-CONF-006`, `FR-CONF-007` | Third edition |
| OQ-067 | How `70` is produced, and how it is exercised | `FR-ERR-030` through `FR-ERR-032`, `BR-ERR-001` | Third edition |
| OQ-068 | What each registered filter and test does | `FR-ENV-030` through `FR-ENV-043`; residue in `OQ-070` through `OQ-072` | Third edition |
| OQ-069 | The shape of the `vars`, `tpl`, and `now` context variables | `FR-CTX-026` through `FR-CTX-030` | Third edition |
| OQ-044 | The outcome for a server below the supported window | `FR-SRV-020`, `FR-SRV-021` | Fourth edition |
| OQ-073 | The outcome for a server newer than the supported window | `FR-SRV-031` through `FR-SRV-033`, `BR-SRV-008`, `BR-SRV-009`, `FR-CTX-034` | Fourth edition |
| OQ-074 | Whether `tpl cfg database test` applies the version gate | `FR-SRV-002` as amended, `FR-SRV-034`, `FR-CFG-024`, `FR-CFG-039` | Fourth edition |

## OQ-002

**Does `tpl cfg database test` report the effective privileges, or only that the
read-only session was established?**

*Origin*: left open in the same decision.
*Why it is open*: the two answers imply different queries and different failure
modes, and the choice was deferred.
*Blocks*: `FR-CFG-024`, and the set of exit codes that command can produce.

## OQ-003

**What does each of the five TLS modes map to in the chosen driver?**

*Origin*: flagged for verification when the five modes were adopted.
*Why it is open*: the database driver has not been chosen, and the mapping must
be verified against it rather than assumed.
*Blocks*: freezing the semantics of `FR-CONF-013`.

## OQ-004

**Which query parameters may a DSN carry?**

*Origin*: flagged as open when the DSN grammar was settled.
*Why it is open*: the list must be written out, and no decision enumerated it.
The one constraint already fixed is that it contains nothing TLS-related.
*Blocks*: `FR-CONF-011`.

## OQ-005

**What is the cap on the output of `password_command`?**

*Origin*: left to settle when `password_command` was made shell-free.
*Why it is open*: an unbounded read from a child process is a denial-of-service
surface, and no limit was chosen.
*Blocks*: `FR-CONF-027`.

## OQ-006

**What happens to the child's stderr?**

*Origin*: left to settle in the same decision.
*Why it is open*: it must not be logged, per `FR-SEC-005`, but whether it is
discarded, inherited, or captured for a diagnostic was not decided.
*Blocks*: `FR-CONF-024`.

## OQ-007

**What happens when `password_command` exits non-zero?**

*Origin*: left to settle in the same decision.
*Why it is open*: the deadline case is `78`, but the non-zero exit case has no
code assigned.
*Blocks*: `FR-CONF-027` and the exit code table.

## OQ-009

**What does the catalogue return for table comments, column comments, and column
defaults, and in which collation?**

*Origin*: flagged for verification against the container when the
invalid-UTF-8 policy was adopted.
*Why it is open*: it must be observed against a real MariaDB instance and
confirmed against the official documentation, not inferred from knowledge of
MySQL.
*Blocks*: freezing the value types behind `FR-OUT-017`.

## OQ-010

**What exactly does the catalogue return for routines, triggers, generated
columns, and foreign-key rules?**

*Origin*: flagged for verification against the container when the scope of
`schema table` and `schema routines` was settled.
*Why it is open*: the same reason as OQ-009.
*Blocks*: the field lists behind `FR-SCH-007` and `FR-SCH-009`.

## OQ-011

**How does the argument pre-scan behave on ambiguous forms?**

*Origin*: the decision states that the pre-scan is specified in detail,
including its behaviour on ambiguous forms, but the detail was not recorded.
*Why it is open*: cases such as `--format` as the last token with no value,
`--format json` appearing after `--`, or `--format=json` appearing twice, each
need a stated outcome.
*Blocks*: `FR-ERR-017`.

## OQ-012

**Does a mandatory sentinel test — asserting that a known password never appears
in any byte of output — become part of the contract?**

*Origin*: recorded as worth reconsidering when the test plan is written.
*Why it is open*: it was deliberately deferred to the test plan rather than
decided with the logging rules.
*Blocks*: nothing in the surface; it strengthens `FR-SEC-005`.

## OQ-013

**In what order does `tpl template list` print names?**

*Origin*: a gap found while writing this edition. The example listing in the
decision is not in lexicographic order.
*Why it is open*: determinism requires a fixed order, and no decision states
which.
*Blocks*: `FR-TMPL-013`.

## OQ-014

**Does `tpl help` accept a nested command path?**

*Origin*: a gap found while writing this edition.
*Why it is open*: the decision establishes `tpl help <cmd>`, but the tree now
reaches three levels, and it is not stated whether
`tpl help cfg database add` is valid.
*Blocks*: `FR-HELP-001` and `FR-HELP-016`.

## OQ-015

**Where may a global flag appear on the command line?**

*Origin*: a gap found while writing this edition.
*Why it is open*: every example shows a global flag before the command name, and
no decision states whether `tpl schema tables -d shop` is accepted.
*Blocks*: `FR-GLOB-002`.

## OQ-016

**Do local flags keep the short forms the root `README.md` declares?**

*Origin*: a gap found while writing this edition. The root document declares
`-s` for `--set`, and `-H`, `-P`, `-u` for the entry flags; the decision that
enumerated the entry flags listed no short forms.
*Why it is open*: the enumeration is the later and more specific source, so this
edition declares long forms only.
*Blocks*: `FR-RND-008` and `FR-CFG-027`.

## OQ-017

**Do `password_command`, `ca_file`, and `ca_path` have flags on
`tpl cfg database add` and `update`?**

*Origin*: a gap found while writing this edition. The three keys exist in the
key space, but the flag-to-key mapping covers only six flags.
*Why it is open*: either the three are set only through `tpl cfg set`, or three
flags are missing from the mapping.
*Blocks*: `FR-CFG-027`.

## OQ-018

**What happens when `.tpl/.cfg` contains a key outside the enumerated space?**

*Origin*: a gap found while writing this edition. `tpl cfg set` rejects an
unknown key with `64`, but the file may be hand-written.
*Why it is open*: rejecting the file with `78` and ignoring the key are both
defensible, and neither was decided.
*Blocks*: `FR-CONF-002`.

## OQ-019

**Is a `password_command` written as a TOML string, rather than an array,
accepted in a hand-edited `.cfg`?**

*Origin*: a gap found while writing this edition.
*Why it is open*: the decision fixes the stored form as an array and fixes the
splitting rule for a string supplied to a command, but says nothing about a
string found in the file.
*Blocks*: `FR-CONF-023` and `FR-CONF-025`.

## OQ-021

**What exit code does a failed cache write produce?**

*Origin*: a gap found while writing this edition.
*Why it is open*: a read command that succeeds against the server but cannot
write `.tpl/.cache/` has no stated outcome. Failing the read and succeeding with
a warning are both defensible.
*Blocks*: `FR-CACHE-007`.

## OQ-023

**How does the error pre-scan behave for a command that does not declare
`--format`?**

*Origin*: a gap found while writing this edition. `tpl render` produces rendered
text and declares no `--format`, yet `tpl render x --format json` would trigger
the pre-scan and then fail as an unknown flag.
*Why it is open*: emitting that `64` as JSON is arguably correct and arguably
confusing, and no decision covers it.
*Blocks*: `FR-ERR-017` and `FR-RND-027`.

## OQ-024

**Which fields does `tpl schema info` report?**

*Origin*: a gap found while writing this edition. The root `README.md` names
"name, version, charset, collation"; no decision confirms the list.
*Why it is open*: the field list is contract surface in `json`, and it must be
verified against the container like every other catalogue field list.
*Blocks*: `FR-SCH-003` and the `database` object of `FR-SCH-031`.
*Narrowed by the third edition*: `FR-SCH-030` and `FR-SCH-031` fix the
envelope of the document and its single `data` key, `database`. What that
object carries remains open, and unlike `OQ-001` and `OQ-022` — which the
third edition closed against the envelope — it cannot be closed by a decision:
it is a catalogue field list, and `scripts/mariadb/` does not exist, so there
is no container to observe against. It is blocked by the same absence as
`OQ-009`, `OQ-010`, and `OQ-025` through `OQ-042`.
*Narrowed again by the fourth edition*: one field of the `database` object is
now fixed and is outside this question — `server`, carrying the probed version,
the series, and the standing, per `FR-CTX-031` and `FR-CTX-034`. It is not a catalogue field, so `BR-CTX-006`
could fix it without observing anything. Every other field of the object
remains open.

## OQ-025

**Which table types does a supported MariaDB server actually report, and do
temporary tables appear in the catalogue at all?**

*Origin*: `FR-CAT-001` and `FR-CAT-006`, which name `BASE TABLE` and
`SYSTEM VERSIONED` as covered and temporary tables as excluded.
*Why it is open*: the exact strings the server writes decide the coverage
predicate, and whether the exclusion of temporary tables needs a filter at all
or is automatic. Neither may be inferred from MySQL.
*Blocks*: the coverage predicate behind `FR-CAT-001` through `FR-CAT-006`, and
the `table_type` value of `FR-CAT-002`.

## OQ-026

**Which catalogue field and which exact strings report that a column is
auto-incremental, and that it is invisible?**

*Origin*: `FR-CAT-027`, which keeps the auto-increment fact while `FR-CAT-024`
excludes the table-level counter, and `FR-CAT-009`, which covers invisible
columns.
*Why it is open*: both facts are carried in a free-text attribute field whose
contents must be observed. A wrong parse silently misclassifies a column.
*Blocks*: `FR-CTX-020`, and the `auto_increment` test of `FR-ENV-014`.

## OQ-027

**How does the catalogue distinguish a literal default from an expression, and
how does it report an explicit `DEFAULT NULL` as against no default at all?**

*Origin*: `FR-CTX-012`, which makes the distinction a four-way discriminant.
*Why it is open*: the discriminant is the whole content of the field, and the
quoting convention that carries it is exactly one of the three points on which
MariaDB and MySQL are known to diverge. It must be observed on MariaDB.
*Blocks*: `FR-CTX-011` through `FR-CTX-013`.

## OQ-028

**What is the textual form of `column_type` for each type family, and how does
the unsigned attribute appear in it?**

*Origin*: `FR-CTX-014` and `FR-CTX-015`, which carry the raw form alongside the
decomposition.
*Why it is open*: the decomposition is derived from this string, so its grammar
must be observed before the parts can be extracted.
*Blocks*: `FR-CTX-015` and `FR-CTX-018`.

## OQ-029

**How are `ENUM` and `SET` members delimited and escaped in the catalogue, and
what happens to a member containing a quote or a comma?**

*Origin*: `FR-CTX-016`, which carries the member list as `values`.
*Why it is open*: a naive split on the delimiter corrupts any member that
contains it, and the escaping convention must be observed rather than assumed.
*Blocks*: `FR-CTX-016`.

## OQ-030

**Which of `precision`, `scale`, and `length` does the catalogue populate for
each type, and which are left empty?**

*Origin*: `FR-CTX-015`, which names the decomposed parts.
*Why it is open*: `FR-CTX-017` requires a part that does not apply to be `null`,
which needs the set of parts that do apply per type to be known.
*Blocks*: `FR-CTX-015` and `FR-CTX-017`.

## OQ-031

**How are a column's character set and collation reported, and what do they hold
for a type that is not textual?**

*Origin*: `FR-CTX-015`, which carries both as decomposed parts.
*Why it is open*: whether a non-textual column carries an empty value, a null,
or an inherited one decides whether `FR-CTX-017` applies to these two parts.
*Blocks*: `FR-CTX-015`.

## OQ-032

**Which fields does the catalogue provide for an index, and which column carries
the position that `FR-CAT-010` folds on?**

*Origin*: `FR-CAT-010`, which requires the one-row-per-column form to be folded
into one object with an ordered column list.
*Why it is open*: the ordering column is the whole basis of the folding, and the
remaining fields — uniqueness, nullability, index type, prefix length, sort
direction, and whether an expression index is reported at all — are the content
of an index in the model.
*Blocks*: `FR-CAT-010` and the index part of `FR-CTX-007`.

## OQ-033

**How is the primary key reported, and is it distinguishable from any other
unique index?**

*Origin*: `FR-CAT-011`.
*Why it is open*: if the primary key is reported only as a specially named
index, the model must derive it, and the derivation must be stated. If it is
reported separately, it is read directly.
*Blocks*: `FR-CAT-011`, and the `primary_key` test of `FR-ENV-015`.

## OQ-034

**Which fields does the catalogue provide for a foreign key, in what spelling
does it report the `ON UPDATE` and `ON DELETE` rules, and how is the incoming
direction obtained?**

*Origin*: `FR-CAT-012` and `FR-CAT-013`.
*Why it is open*: the rule spellings are contract surface once they reach the
document, and the incoming direction is a separate read whose shape has not been
observed.
*Blocks*: `FR-CAT-012`, `FR-CAT-013`, and `FR-CTX-010`.

## OQ-035

**Which fields does the catalogue provide for a `CHECK` constraint, and what
values does the level take?**

*Origin*: `FR-CAT-015`, which requires name, level, and clause.
*Why it is open*: the second of the three known MariaDB-versus-MySQL divergences
is in this table, so nothing about it may be carried over from MySQL knowledge.
The values of the level are contract surface.
*Blocks*: `FR-CAT-015`.

## OQ-036

**Which fields does the catalogue provide for a view, beyond its definition?**

*Origin*: `FR-CAT-007`, which covers views with their SQL definition.
*Why it is open*: the definition is named; the remaining fields are not, and
they are contract surface in `json`.
*Blocks*: `FR-CAT-007` and `FR-SCH-006`.

## OQ-037

**Which fields does the catalogue provide for a routine, and is the body
readable without a privilege beyond the one needed to list routines?**

*Origin*: `FR-CAT-016` and `FR-CAT-017`.
*Why it is open*: the field list is contract surface, and the privilege question
decides how often `FR-PRIV-003` fires in practice — a routine whose body cannot
be read is an incomplete object.
*Blocks*: `FR-CAT-016`, `FR-CAT-017`, and the calibration of `FR-PRIV-003`.

## OQ-038

**Which fields does the catalogue provide for a routine parameter, in what order
does it report them, and how is a function's return type reported?**

*Origin*: `FR-CAT-018`, which requires parameters in declaration order.
*Why it is open*: a function's return value is commonly reported in the same
place as its parameters and must not be presented as one.
*Blocks*: `FR-CAT-018`.

## OQ-039

**Which fields does the catalogue provide for a trigger?**

*Origin*: `FR-CAT-014`.
*Why it is open*: the field list is contract surface and has not been observed.
*Blocks*: `FR-CAT-014`.

## OQ-040

**How is a generated column's expression reported, and how is a virtual column
distinguished from a stored one?**

*Origin*: `FR-CAT-009`, which covers generated columns.
*Why it is open*: the third of the three known MariaDB-versus-MySQL divergences
is the field that carries this fact, so it must be observed on MariaDB.
*Blocks*: `FR-CAT-009` and `FR-CTX-020`.

## OQ-041

**How does the catalogue report a view whose definition the reader may not see:
empty, truncated, or absent?**

*Origin*: `FR-PRIV-011`, which infers a missing privilege from the combination of
view rows among tables and an empty view collection.
*Why it is open*: the cross-check is stated as if the collection comes back
empty. If the rows come back with an empty definition instead, the check must be
written differently, and it must be observed with two readers of different
privilege.
*Blocks*: `FR-PRIV-011` and `FR-PRIV-012`.

## OQ-042

**What does the server version probe return, and how is MariaDB distinguished
from a server that reports a MariaDB-compatible version string?**

*Origin*: `FR-SRV-002` and `FR-SRV-003`.
*Why it is open*: the refusal of a non-MariaDB server is only as good as the
detection, and several servers report version strings designed to be mistaken
for another product's. It also fixes the exact string `FR-CTX-031` carries in
`version`, and the form from which its `series` is derived.
*Blocks*: `FR-SRV-002`, `FR-SRV-003`, and `FR-CTX-031`.

## OQ-043

**Does `referenced_by` embed the referencing table under the rule of
`FR-CTX-006`, or does it hold names only?**

*Origin*: a gap found while writing the second edition. The embedding rule was
settled for the outgoing direction; the incoming direction was added separately
and the two decisions were never composed.
*Why it is open*: the two answers differ by roughly the same factor the outgoing
embedding already costs, so the choice has a direct effect on the memory budget
and on the `WL-002` scalar. It is a design decision, not a measurement.
*Blocks*: `FR-CTX-010`, and the value of N in `OQ-060`.

## OQ-045

**Which fields of the model differ among MariaDB `12.3`, `11.8`, `11.4` and
`10.11`, and what does each of the four return for them?**

*Origin*: `FR-SRV-004`, `FR-SRV-024` and `FR-SRV-025`, which fix three different
obligations for three kinds of difference, and `FR-SRV-027`, which requires each
accommodated difference to be registered.
*Restated by the fourth edition*: the subject was "which fields were introduced
after MariaDB 10.6". The floor is withdrawn and the subject is now the
differences among the four series the window of `FR-SRV-001` admits — in both
directions, since a field absent from `10.11` and a field reported differently
on `12.3` are both differences and carry different obligations.
*Why it is open*: `FR-SRV-027` requires the field, the series, and the observed
behaviour of each of the four, and the project forbids documenting catalogue
behaviour that has not been observed. It cannot be answered from a changelog, a
release note, or MySQL knowledge. `scripts/mariadb/` does not exist in this
repository, so there is no container to observe against; it is blocked by the
same absence as `OQ-009`, `OQ-010`, `OQ-024`, and `OQ-025` through `OQ-042`.
*Consequence for the container*: the fixture must now be observed against **four
server versions rather than one**, per `FR-SRV-029`, and the DDL of `setup.sql`
and `seed.sql` must be DDL that all four accept. Where a structure cannot be
created on all four, that is not a fixture defect but an entry this question
owes. Whoever stands the container up must plan for four, not discover it after
building one.
*Blocks*: `FR-SRV-004`, `FR-SRV-024`, `FR-SRV-025`, `FR-SRV-027`, and the
verification of `FR-SRV-026`.

## OQ-046

**How is the read-back of `FR-SRV-009` performed within the closed statement
list of `FR-SRV-006`?**

*Origin*: a gap found while writing the second edition. The read-back confirms
that the read-only session took effect, and the natural way to perform it is a
statement that the closed list does not contain.
*Why it is open*: either the closed list gains a fourth entry, stated as
narrowly as the other three, or the read-back is performed another way. Widening
the list is a decision about the strongest guarantee this tool makes.
*Blocks*: `FR-SRV-006` and `FR-SRV-009`.

## OQ-047

**What is the shape and the content of the `restricted` field?**

*Origin*: `FR-PRIV-005`, which names the field without fixing its value.
*Why it is open*: a boolean cannot later say which property was unreadable, and
`FR-OUT-014` allows an enumerated field to gain a value while a boolean cannot.
The choice is the same one `FR-CDOC-010` already made for `source`, and it
should be made deliberately rather than by analogy.
*Blocks*: `FR-PRIV-005` and `FR-PRIV-006`.

## OQ-048

**May an incomplete object be written to the cache?**

*Origin*: a gap found while writing the second edition. `FR-CACHE-007` writes
every miss to the cache, and `FR-PRIV-005` allows a listing to contain an
incomplete object.
*Why it is open*: caching a restricted object persists a stub that a later
invocation, perhaps by a reader with sufficient privilege, would be served from
disk under `FR-CDOC-008`. Refusing to cache it, and caching it with the marking
intact, are both defensible.
*Blocks*: `FR-CACHE-007` and `FR-CDOC-008`.

## OQ-049

**Which minor version of the template engine is group 2 pinned to?**

*Origin*: `FR-ENV-003`, which guarantees the inherited filters against a pinned
engine minor version.
*Why it is open*: the guarantee has no meaning until the version is named, and
naming it is bound to the dependency decision, which lives outside this
specification.
*Blocks*: `FR-ENV-003` and `FR-ENV-018`.

## OQ-050

**Which contract group does `escape` belong to, and exactly what does it
escape?**

*Origin*: `FR-ENV-028`, which makes `escape` the explicit alternative to the
auto-escaping that `FR-ENV-026` removes.
*Why it is open*: `escape` is absent from the enumerated inherited list of
`FR-ENV-018`, so as written it falls into the unguaranteed group 3 — which
cannot be right for the one filter the escaping decision depends on. What it
escapes, and for which target, is equally unstated.
*Blocks*: `FR-ENV-018` and `FR-ENV-028`.

## OQ-051

**What is the budget for `tpl --version`?**

*Origin*: `NFR-PERF-014`.
*Why it is open*: there is no code, so no figure is defensible. It is also the
reference the failure-path budget of `OQ-056` is stated against.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-052

**What is the budget for `tpl --help`?**

*Origin*: `NFR-PERF-014`. The line was split from `tpl --version` because the
two do not measure the same thing: one prints a constant, the other assembles a
help text from a typed table, per `FR-HELP-022`.
*Why it is open*: there is no code.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.
*Narrowed by the third edition*: neither form runs project discovery, per
`FR-PROJ-025` and `NFR-PERF-005`, so the budget no longer has to absorb a
discovery walk. Only the figure remains open.

## OQ-053

**What is the budget for startup to the first byte of useful work?**

*Origin*: `NFR-PERF-014`.
*Why it is open*: there is no code, and the figure now has to absorb a discovery
walk that canonicalises a path and checks the owner and mode of `.tpl/.cfg`,
per `FR-PROJ-009` through `FR-PROJ-011`.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-054

**What is the budget for `tpl schema dump` over `WL-001`?**

*Origin*: `NFR-PERF-014`.
*Why it is open*: there is no code, and `WL-001` has no fixture.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-055

**What is the normative target for a cache-served read of one object over
`WL-003`?**

*Origin*: `NFR-PERF-015`, which requires the target to be named in the text of
the requirement once it is known.
*Why it is open*: there is no code. This is the most consequential of the ten,
because it is the only normative budget and the only one that can gate a
pipeline.
*Blocks*: `NFR-PERF-015`.

## OQ-056

**What is the budget for the failure path — a `64`, and a `66` carrying a
nearest-match suggestion computed over every existing name of `WL-001`?**

*Origin*: `NFR-PERF-014` and `BR-PERF-004`.
*Why it is open*: there is no code, and `WL-001` has no fixture. The intent
recorded with the budget is that it should cost what `tpl --version` costs, so
it is stated against `OQ-051`.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-057

**What is the budget for `tpl help --format json`?**

*Origin*: `NFR-PERF-014`.
*Why it is open*: there is no code. It is the largest help payload the tool
produces, and under `FR-HELP-021` it is derived at runtime by introspecting the
command tree rather than emitted from a constant.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-058

**What is the budget for the canonical loop of 200 invocations over `WL-001`?**

*Origin*: `NFR-PERF-014` and `BR-PERF-005`.
*Why it is open*: there is no code, and `WL-001` has no fixture. The quantity it
measures changed when `--all-tables` was removed: it is now 200 process
startups, not 200 iterations inside one process.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-059

**What is the peak-resident-memory budget over `WL-001`?**

*Origin*: `NFR-PERF-014`.
*Why it is open*: there is no code, and the figure depends on a document shape
that `FR-CTX-006` roughly doubles and that `OQ-043` may double again.
*Blocks*: the corresponding baseline in `BENCHMARKS.md`.

## OQ-060

**What is the value of N in the `WL-002` scalar — the size in bytes of the
compact `tpl schema dump` of `WL-001`?**

*Origin*: `WL-002`.
*Why it is open*: there is no code and no fixture, and the value moves with
every decision about the document shape, including `OQ-043`.
*Blocks*: `WL-002`, and the use of that scalar to tell a shape regression from a
speed regression.

## OQ-070

**How is a backtick inside a MariaDB identifier escaped, so that `quote`
returns a single identifier MariaDB parses back to the original string?**

*Origin*: `FR-ENV-035`, which fixes the product requirement — the output is a
valid quoted MariaDB identifier for every legal MariaDB identifier, including
one containing a backtick — without stating the mechanism.
*Why it is open*: the mechanism is a fact about how MariaDB parses a quoted
identifier, and the project forbids documenting engine behaviour that has not
been observed. `scripts/mariadb/` does not exist in this repository, so there
is no container to observe against. It must not be carried over from MySQL
knowledge or from a documentation reading alone; it is blocked by the same
absence as `OQ-009`.
*Blocks*: `FR-ENV-035`, and the correctness of every generated identifier. A
`quote` that gets this wrong emits invalid SQL from a legal table name, which
is the class of failure `BR-SEM-004` works hardest to prevent — except that
here the wrong output compiles as far as `tpl` is concerned and fails at the
server.

## OQ-071

**What does `sql_type` do that `column_type` does not?**

*Origin*: `FR-ENV-039`, which keeps the name registered and states no
behaviour.
*Why it is open*: `FR-CTX-014` already carries `column_type`, the type exactly
as the server writes it, on every column. A `sql_type` filter that returned the
same string would be redundant with a field the template already has. What it
would add — a full DDL type clause including nullability and default, a
normalised form, a form adjusted for a target dialect — was never decided, and
the third edition declined to invent it. This is a design decision, not a
measurement.
*Blocks*: `FR-ENV-039`, and the closure of `FR-ENV-007`, which registers the
name. Until it is answered the name is reserved and no template may rely on it.

## OQ-072

**Which `data_type` values belong to the numeric, the date-and-time, and the
character-string families?**

*Origin*: `FR-ENV-041` and `FR-ENV-042`, which fix `numeric`, `temporal`, and
`textual` as predicates over a family without stating the membership of any
family.
*Why it is open*: the membership is a set of strings MariaDB writes in the
catalogue, and the textual form of `data_type` is itself `OQ-028`. Neither may
be carried over from MySQL, and there is no container to observe against. A
wrong membership silently misclassifies a column, which is exactly the failure
`FR-CTX-018` keeps `column_type` as a safety net against.
*Blocks*: `FR-ENV-041` and `FR-ENV-042`, and it is blocked in turn by
`OQ-028`.
