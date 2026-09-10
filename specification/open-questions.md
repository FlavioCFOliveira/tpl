---
title: Open Questions
status: approved
last-reviewed: 2026-09-10
related: [README.md, catalogue-coverage.md, context-document.md, server-contract.md]
---

# Open Questions

## Overview

Every entry here is a point this specification cannot yet fix. None is resolved
by inference: an open question stays open until the user settles it or a
measurement answers it. An entry states the question, where it came from, why
it cannot be answered now, and what it blocks. When one is closed, the
requirement it produces is written into the owning module and the entry moves
to the *Closed* table below; its identifier is retired and never reused, per
the identifier scheme of the [README](README.md#identifier-scheme).

**Twenty-five entries remain, and they are of only two kinds.** No design
question is open. Every point on which this specification had to choose has
been chosen, and what is left is what a specification cannot choose for itself.

**Twenty-four are catalogue facts.** `OQ-009`, `OQ-010`, the residue of
`OQ-024`, `OQ-025` through `OQ-042`, `OQ-045`, `OQ-070`, and `OQ-072`. They are
not design questions at all: each is a fact about what a MariaDB server
actually returns, and this project forbids documenting catalogue behaviour that
has not been observed. None can be closed by reasoning, by analogy with MySQL,
or by reading documentation alone, and all twenty-four are blocked by the same
absence — **`scripts/mariadb/` does not exist in this repository**, so there is
no container to observe against. `OQ-045` is the most demanding of them: it
must be observed against each of the four series of `FR-SRV-015` rather than
one, per `FR-SRV-029`, and whoever stands the container up must plan for four.

**One is a mapping that waits on a measurement.** `OQ-003` asks what each of
the five TLS modes maps to in the database driver. The driver is chosen by
measuring startup, peak resident memory, and stripped binary size, and the
mapping must be verified against the driver that measurement selects rather
than assumed. `FR-CONF-036` fixes what any candidate must satisfy — all five
modes, expressible distinctly — so the criterion is settled and only the
mapping is not.

Nothing else is open. If a point of this specification appears unsettled and is
not one of these twenty-five, it is a defect to be reported and corrected, not
an entry to be added here.

## Index

| Id | Question | Owner module | Blocks |
|---|---|---|---|
| [OQ-003](#oq-003) | TLS mode to driver mapping | `configuration-model.md` | Freezing TLS semantics |
| [OQ-009](#oq-009) | Catalogue return types and collation | `schema-commands.md` | Freezing catalogue value types |
| [OQ-010](#oq-010) | Field lists for routines, triggers, generated columns, FK rules | `schema-commands.md` | `schema table` and `schema routine` |
| [OQ-024](#oq-024) | Metadata fields of the `database` object | `schema-commands.md` | `schema info` |
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
| [OQ-045](#oq-045) | Which fields differ among the four supported series | `server-contract.md` | `FR-SRV-004`, `FR-SRV-024`, `FR-SRV-027`, `FR-CAT-029` |
| [OQ-070](#oq-070) | The escaping of a backtick inside an identifier | `template-environment.md` | `FR-ENV-035` |
| [OQ-072](#oq-072) | The membership of the three type families | `template-environment.md` | `FR-ENV-041`, `FR-ENV-042` |

## Closed

An entry is closed when the requirement it produced is written into the owning
module. A few are closed differently: an entry is **dissolved** when the
mechanism it governed ceases to exist, so there is no answer to record because
there is no longer a question. Either way the identifier is retired and never
reused, so a gap in the sequence above is expected rather than a defect.

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
| OQ-002 | Whether `tpl cfg database test` reports effective privileges | `FR-CFG-044`, `FR-CFG-045`, the `can_read_catalogue` field of `FR-CFG-039`, and the fourth step of `FR-CFG-024` | Fifth edition |
| OQ-004 | Which query parameters a DSN may carry | `FR-CONF-011` as amended — none — and `FR-CONF-012`, now a special case of it | Fifth edition |
| OQ-005 | The cap on `password_command` output | `FR-CONF-031` — 4096 bytes; exceeding it is `78` | Fifth edition |
| OQ-006 | What happens to the child's stderr | `FR-CONF-032` — the null device | Fifth edition |
| OQ-007 | What happens when `password_command` exits non-zero | `FR-CONF-033` — `78` | Fifth edition |
| OQ-011 | Pre-scan on ambiguous `--format` forms | **Dissolved.** `FR-ERR-033` withdraws the JSON error document and with it the pre-scan of `FR-ERR-017`; no mechanism is left to specify | Fifth edition |
| OQ-012 | Whether a mandatory password sentinel test is contract | `BR-SEC-003` | Fifth edition |
| OQ-013 | The order of `tpl template list` | `FR-TMPL-013` | Fifth edition |
| OQ-014 | Whether `tpl help` accepts a nested command path | `FR-HELP-026` through `FR-HELP-029` | Fifth edition |
| OQ-015 | Where a global flag may appear on the command line | `FR-CLI-024`, with `FR-CLI-001` as amended | Fifth edition |
| OQ-016 | Short forms for local flags | `FR-GLOB-024` — there are none | Fifth edition |
| OQ-017 | Flags for `password_command`, `ca_file`, and `ca_path` | `FR-CFG-027` as amended, `FR-CFG-046`, `FR-CFG-047`, and `FR-CFG-033` as amended | Fifth edition |
| OQ-018 | An unknown key in a hand-written `.cfg` | `FR-CONF-034`, `BR-CONF-004` | Fifth edition |
| OQ-019 | `password_command` written as a TOML string in the file | `FR-CONF-035`, `BR-CONF-004` | Fifth edition |
| OQ-021 | The outcome of a failed cache write | `FR-CACHE-036` — silent success | Fifth edition |
| OQ-023 | Pre-scan on a command that declares no `--format` | **Dissolved.** The same mechanism, withdrawn by the same requirement; `FR-RND-027` records the ordinary outcome that remains | Fifth edition |
| OQ-043 | Whether `referenced_by` embeds | `FR-CTX-010`, with `FR-CTX-009` as amended | Fifth edition |
| OQ-046 | The read-back within the closed statement list | The fourth entry of `FR-SRV-006`, with `FR-SRV-012` as amended | Fifth edition |
| OQ-047 | The shape of the `restricted` field | `FR-PRIV-016` | Fifth edition |
| OQ-048 | Whether an incomplete object may be cached | `FR-CACHE-037` — it may not | Fifth edition |
| OQ-049 | The pinned engine minor version | `FR-ENV-003` as amended: the pin must exist, be recorded outside this corpus, and be cited; the number never enters the corpus | Fifth edition |
| OQ-050 | The contract group and behaviour of `escape` | `FR-ENV-007` as amended, and `FR-ENV-044` | Fifth edition |
| OQ-051 | Budget: `tpl --version` | `NFR-PERF-014`, carrying a provisional figure under `NFR-PERF-019` and the gate of `NFR-PERF-020` | Fifth edition |
| OQ-052 | Budget: `tpl --help` | The same | Fifth edition |
| OQ-053 | Budget: startup to the first useful byte | The same | Fifth edition |
| OQ-054 | Budget: `tpl schema dump` over `WL-001` | The same | Fifth edition |
| OQ-055 | Budget: cache-served read over `WL-003` | `NFR-PERF-014` and `NFR-PERF-015`; named and unvalued, per `BR-PERF-006` | Fifth edition |
| OQ-056 | Budget: the failure path | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-057 | Budget: `tpl help --format json` | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-058 | Budget: the 200-invocation loop | `NFR-PERF-014`; named and unvalued | Fifth edition |
| OQ-059 | Budget: peak resident memory over `WL-001` | `NFR-PERF-014`, carrying a provisional figure; `FR-CTX-010` records why it is the likeliest of them to move | Fifth edition |
| OQ-060 | The byte scalar N of `WL-002` | `WL-002` as amended: unvalued, and scheduled under `NFR-PERF-020` | Fifth edition |
| OQ-071 | What `sql_type` does beyond `column_type` | `FR-ENV-039` — the normalised type name, which is the `data_type` of `FR-CTX-015` | Fifth edition |

## OQ-003

**What does each of the five TLS modes map to in the chosen driver?**

*Origin*: flagged for verification when the five modes were adopted.
*Why it is open*: the database driver has not been chosen, and the mapping must
be verified against it rather than assumed.
*Narrowed by the fifth edition*: the criterion is settled even though the
mapping is not. `FR-CONF-036` makes the five modes of `FR-CONF-013` normative
over the driver: a driver that cannot express all five distinctly is
disqualified, and the mode set is never reduced to what a candidate offers.
What remains open is the mapping itself, and it is open for one reason — the
driver is chosen by measuring startup, peak resident memory, and stripped
binary size, and no such measurement has been made. The specification names no
driver and will not: which one is chosen is an architecture decision.
*Blocks*: freezing the semantics of `FR-CONF-013`.

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

## OQ-024

**Which fields does `tpl schema info` report?**

*Origin*: a gap found while writing this edition. The root `README.md` names
"name, version, charset, collation"; no decision confirms the list.
*Why it is open*: the field list is contract surface in `json`, and it must be
verified against the container like every other catalogue field list.
*Blocks*: `FR-SCH-003` and the metadata fields of the `database` object of
`FR-SCH-031`.
*Narrowed by the third edition*: `FR-SCH-030` and `FR-SCH-031` fix the
envelope of the document and its single `data` key, `database`. What that
object carries remains open, and unlike `OQ-001` and `OQ-022` — which the
third edition closed against the envelope — it cannot be closed by a decision:
it is a catalogue field list, and `scripts/mariadb/` does not exist, so there
is no container to observe against. It is blocked by the same absence as
`OQ-009`, `OQ-010`, and `OQ-025` through `OQ-042`.
*Narrowed again by the fourth edition*: one field of the `database` object is
now fixed and is outside this question — `server`, carrying the probed version,
the series, and the standing, per `FR-CTX-031` and `FR-CTX-034`. It is not a
catalogue field, so `BR-CTX-006` could fix it without observing anything.

*Narrowed again by the fifth edition*: three more are fixed and are outside
this question — the collections `tables`, `views`, and `routines`, per
`FR-CTX-035`. They are structural rules of
[context-document.md](context-document.md) rather than catalogue fields, and
fixing their names, their array shape, and their order asserts nothing about
what a catalogue table returns.

**What remains open is the metadata fields of the `database` object** — the
fields describing the database itself rather than the objects in it. The root
`README.md` names "name, version, charset, collation"; `version` is superseded
by `server`, and `name`, the charset, and the collation are catalogue fields
that no decision confirms and that must be observed. `name` in particular was
**not** fixed by the fifth edition and stays inside this question.

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
*Widened by the fifth edition*: two of the three obligations changed shape and
one gained a home, so this question now feeds three places rather than two.
`FR-SRV-024` normalises a fact present on **more than one** series rather than
on every series, so a fact present on two of the four and reported differently
between them is now an observation this question owes. `FR-SRV-036` names the
register the observations are written into — the *divergence register* section
of [server-contract.md](server-contract.md), empty until this question is
answered. And `FR-SRV-025` writes its exclusions into the ambiguous-meaning
list of `FR-CAT-029`, also empty for the same reason.
*Blocks*: `FR-SRV-004`, `FR-SRV-024`, `FR-SRV-025`, `FR-SRV-027`, the register
of `FR-SRV-036`, the list of `FR-CAT-029`, and the verification of
`FR-SRV-026`.

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
