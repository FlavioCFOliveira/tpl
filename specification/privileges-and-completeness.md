---
title: Privileges and Completeness
status: approved
last-reviewed: 2026-09-20
related: [catalogue-coverage.md, server-contract.md, context-document.md, errors-and-exit-codes.md]
---

# Privileges and Completeness

## Overview

[catalogue-coverage.md](catalogue-coverage.md) defines what the model contains.
A reader whose privileges do not extend to all of it gets less. This file
defines what `tpl` does then: when a shortfall is a failure, when it is a fact
to be reported, and how a document that is known to be short is prevented from
being used as though it were whole.

The rule is asymmetric on purpose. Asking for one object by name and receiving
half of it is a failure of the request. Listing what is there and finding that
some of it is out of reach is a fact about the reader's privileges, and a
listing that refuses to answer would be less useful than one that answers and
says so.

## Scope

In scope: the definition of a complete and an incomplete read, the outcome for a
named object, the outcome for a listing and a dump, the marking of an incomplete
object and the shape of that marking, the refusal of a marked dump as a context,
the three shapes a privilege-driven absence takes, the population every count
in this file is taken over, the cross-checks that distinguish a missing
privilege from an absence, the objects each cross-check marks and the property
each of them loses, the one place where the catalogue makes no such
distinction possible, and the outcome where the database a read covers has no
row in the schema catalogue at all.

Out of scope: which privileges a reader needs, which depends on the server and
is not stated here; the connection and authentication failures that precede a
read, which are `FR-ERR-001`; and the fields of the model, which are
[catalogue-coverage.md](catalogue-coverage.md).

## Actors

- **Reader**, the database user the selected entry authenticates as.
- **Calling agent**, which must be able to tell a short answer from a wrong one.

## Complete and incomplete

- **FR-PRIV-001**: A read SHALL be complete WHEN it returns, for every object it
  presents, every property [catalogue-coverage.md](catalogue-coverage.md)
  defines for that object. Otherwise it SHALL be incomplete.

- **FR-PRIV-002**: An object SHALL be incomplete WHEN any property the model
  defines for it could not be read, whatever the reason.

## An object requested by name

- **FR-PRIV-003**: IF an object requested by name — `tpl schema table`,
  `tpl schema view`, `tpl schema routine`, or a `tpl render` object flag — comes
  back incomplete, THEN the system SHALL exit `77` (`EX_NOPERM`).

- **FR-PRIV-004**: The system SHALL NOT return a partial object in answer to a
  request that named it.

  *Rationale.* The caller asked for that object and would receive half of it
  with exit `0`. A routine whose body could not be read renders as a stub, and
  the stub gets committed.

## A listing and a dump

- **FR-PRIV-005**: WHEN a listing or a dump contains an incomplete object, the
  system SHALL succeed and SHALL mark that object with a `restricted` field.

- **FR-PRIV-006**: The system SHALL mark each incomplete object individually.
  A single flag on the document SHALL NOT stand in for the per-object marking.

- **FR-PRIV-007**: A complete object in the same document SHALL NOT be marked.

- **FR-PRIV-016**: `restricted` SHALL be a JSON array of strings, each naming
  one property of the model that could not be read for that object. The array
  SHALL be ordered by name, ascending, byte-wise, per `NFR-DET-002`, SHALL
  never be empty, and SHALL be **present only on an incomplete object**, per
  `FR-PRIV-007`.

  ```json
  {"name":"sp_book_consignment","restricted":["body"]}
  ```

  *Amended in the sixth edition.* The example read
  `{"name":"orders","restricted":["triggers"]}`, and `FR-PRIV-020` establishes
  that `tpl` cannot produce it: a hidden trigger list is indistinguishable
  from an empty one, so `triggers` can never appear in this array. An
  unreadable routine body can, and is the commonest case.

  *Rationale for the array.* A boolean says that something was unreadable and
  cannot ever say what. Naming the properties is what lets a caller decide
  whether the shortfall matters: a generator that emits column definitions is
  unaffected by an unreadable trigger list and stopped by an unreadable column
  list, and a boolean makes those two the same answer. Strings rather than an
  enumerated set of identifiers, because the population is the property names
  of the model itself, which
  [catalogue-coverage.md](catalogue-coverage.md) already fixes and
  [context-document.md](context-document.md) already names in the document —
  a second vocabulary for the same things would be a second thing to keep true.

  *Rationale for presence only where there is something to report.* This
  departs deliberately from the always-present pattern `FR-CTX-034` sets for
  `standing` and `FR-CDOC-009` for `source`, and the two cases are not alike.
  Those two fields answer a question that has an answer on every document —
  where did this come from, how does this server stand — so an always-present
  enumerated value is the honest shape. `restricted` answers a question that
  has no answer on a complete object: there is no list of properties that could
  not be read, not even an empty one, because nothing was attempted and
  refused. An empty array on every object of every document would also add one
  key per object to the largest document `tpl` emits, for the benefit of the
  case that does not arise.

  *Accepted cost, and it reaches a template.* `FR-SEM-012` fails a render with
  `65` when a template reads a field that does not exist on the value in hand,
  so `{% if table.restricted %}` fails on every complete table — which is the
  argument `BR-SRV-008` used to make `standing` unconditional, arriving here
  and being answered the other way. It is answered the other way because the
  field is not for a template. `FR-PRIV-003` fails a **named** incomplete
  object with `77` before any render begins, and `FR-PRIV-008` refuses a marked
  document as a `--context`, so the only way a marked object reaches a template
  at all is inside a collection of a whole-database render against a live or
  cached source. For that case the specification offers no in-template guard
  and states so here rather than leaving it to be discovered. `restricted` is
  plumbing contract, per `BR-PRIV-002`: it is read by the caller that parses
  the JSON, which is where the decision to render at all belongs.

  *This is the second exception to `FR-OUT-012`*, which otherwise requires an
  absent value to be emitted as `null` rather than omitted. The first is the
  `data` of `tpl cfg list`, per `FR-CFG-037`. Both are recorded as exceptions
  rather than left to be inferred, and there are exactly two.

  *Rejected.* A boolean, for the first reason above. Also rejected: an
  always-present array that is empty on a complete object, which satisfies
  `FR-OUT-012` and `FR-SEM-012` and pays for it in every object of every
  document; and an enumerated string naming a single reason, which cannot
  report two unreadable properties on one object.

- **BR-PRIV-001**: The asymmetry is the whole design and neither uniform answer
  is acceptable. Failing everywhere would mean a reader without the privilege to
  read view definitions could not even list tables. Succeeding everywhere with a
  marking would mean a stub routine body reaches a generated file behind an exit
  code of `0`, which is the failure this specification works hardest to prevent.

  *Rejected.* Both uniform policies, for the reasons above.

## A marked dump is not a context

- **FR-PRIV-008**: IF a document supplied to `tpl render --context` marks any
  object as restricted, THEN the render SHALL NOT proceed, and the system SHALL
  exit `65` (`EX_DATAERR`) under `FR-RND-020`.

- **FR-PRIV-009**: `FR-PRIV-008` SHALL apply whatever the render's object flag
  selects, including when the selected object is itself complete.

  *Rationale.* The round-trip of `FR-SCH-022` depends on a dump being whole, and
  `FR-SCH-021` already refuses a filtered dump for the same reason. A dump that
  knows it is short must say so at the point of use, not only at the point of
  production, because the two are separated by a commit and often by a machine.

## Absence versus privilege

- **FR-PRIV-010**: WHERE the catalogue makes the two cases distinguishable,
  the system SHALL NOT report an absence whose true cause is a missing
  privilege. WHERE the catalogue does not, the limit SHALL be stated in this
  file rather than left to be discovered, per `FR-PRIV-020`.

  *Amended in the sixth edition.* The requirement was an unqualified
  prohibition, and the observation of 2026-09-10 established that it cannot be
  honoured for one of the properties of the model. A
  reader that lacks the privilege receives **zero rows** from
  `INFORMATION_SCHEMA.TRIGGERS`, which is byte-for-byte what a table with no
  triggers returns; nothing in the catalogue separates them. Left absolute,
  the requirement obliged a detection that does not exist, and a requirement
  that cannot be satisfied is not a requirement — it is a place where the
  specification would have been believed and wrong. `FR-PRIV-020` records
  exactly where the guarantee stops.

- **FR-PRIV-018**: A privilege-driven absence SHALL be detected by the shape
  the catalogue actually gives it. There are **three shapes** — the empty
  string, `NULL`, and zero rows — and they fall across the properties of the
  model as follows, as observed on 2026-09-10 against all four series of
  `FR-SRV-015`:

  | Property | Read as a privileged reader | Read as a reader without the privilege | Detectable |
  |---|---|---|---|
  | A view's definition | the full text | the **empty string**, length 0, on a row that is present | yes, `FR-PRIV-011` |
  | A routine's body | the full body | **`NULL`**, on a row that is present | yes, `FR-PRIV-017` |
  | A table's referential rules, and the constraint table beside them | 15 and 68 rows for the fixture | **zero rows** | yes, `FR-PRIV-019` |
  | A table's triggers | 6 rows for the fixture | **zero rows** | **no**, `FR-PRIV-020` |

  **Every count in this file names the population it was taken over**, and one
  catalogue table has two. A count over a catalogue table is taken over the
  rows it returns for the database the read covers. A count over the rows the
  foreign-key cross-check of `FR-PRIV-019` is evaluated over is taken over the
  **filtered** population `FR-CAT-045` selects: the key-column rows that name a
  referenced table, which are the only rows a foreign key contributes. For the
  fixture the two are **54** and **17**, a factor of three apart, which is why
  the distinction has to be written down rather than inferred from a number.

  Every other catalogue table this file names has one population. For the
  fixture they return 15 referential-constraint rows, 68 table-constraint rows,
  24 check-constraint rows, 77 index rows and 6 trigger rows, and no
  requirement of this corpus selects a subset of any of them.

  *Observed.* The reduced-grant reader of the fixture holds
  `SELECT, EXECUTE ON freight.*` and nothing more. It sees all 23 catalogue
  objects and all 7 routine rows; it loses the view definitions to an empty
  string, the routine bodies to `NULL`, and the whole of
  `INFORMATION_SCHEMA.TABLE_CONSTRAINTS`, `REFERENTIAL_CONSTRAINTS` and
  `TRIGGERS` to zero rows, while `KEY_COLUMN_USAGE` and `CHECK_CONSTRAINTS`
  are unaffected. The behaviour is identical on all four series.

  *Observed, 2026-09-20, against all four series of `FR-SRV-015` through the
  harness of `scripts/mariadb/`, as the privileged reader and as the
  reduced-grant reader in turn.* The 17 rows carry 15 distinct
  table-and-constraint pairs, which is the 15 referential rules exactly; they
  name 9 distinct referencing tables and 9 distinct referenced tables, whose
  union is 14 of the fixture's tables. The other 37 key-column rows belong to
  primary and unique keys, and each of them returns SQL `NULL` in the
  referenced-table, referenced-schema and referenced-column fields, per
  `FR-CAT-045`. The reduced-grant reader sees the same 54 and the same 17, and
  zero rows in `TABLE_CONSTRAINTS` and `REFERENTIAL_CONSTRAINTS`. Identically
  on all four series.

  *The table-constraint table is observed here and read nowhere.* It is the
  second table a reduced reader loses entirely, which is why its count belongs
  in this file; no requirement of
  [catalogue-coverage.md](catalogue-coverage.md) takes a property from it, and
  `FR-CAT-043` reads the primary key from the index table instead. Its absence
  is therefore detectable only through the referential rules beside it, which
  is what `FR-PRIV-019` detects.

  *Rationale.* The first edition of this file assumed one shape of absence and
  wrote one detection for it. There are three, they do not resemble each
  other, and a reader that looks for one of them finds none of the others: an
  empty string is not the same absence as a `NULL`, and neither of them is a
  row that is not there.

  *Closes* `OQ-041`, now listed under [Closed](open-questions.md#closed).

  *Amended in the seventh edition: the third row is narrowed, because it
  over-stated what such a reader loses.* Of the four kinds of key and
  constraint a table carries, the reduced reader loses exactly one. It keeps
  every **check constraint**, which it reads from a table the privilege does
  not remove — all 24 rows of the fixture. It keeps every **index**,
  including the unique ones, and it keeps the **primary key**, because
  `FR-CAT-043` reads both from the index table and that table returns all 77
  rows to it. What it loses is the **referential rule** of every foreign key,
  and only that. The row above is worded accordingly, and `FR-PRIV-019`
  detects exactly the loss that remains.

  *Amended in the twenty-seventh edition: the counts name their population.*
  The third row above and `FR-PRIV-019` both quote counts of the key-column
  table taken over the whole table, and no requirement said so. A reader
  building the cross-check from `FR-PRIV-019`'s *all 54 rows* and finding 17
  has two readings available — that the statement is wrong, or that the
  observation is — and the two build different programs. Neither is: the
  populations are different, and both numbers are correct over the population
  each was taken over. Nothing about what a reduced reader loses changes.

- **FR-PRIV-011**: IF the catalogue reports a view whose definition is the
  empty string, THEN the system SHALL treat that as a missing privilege and
  SHALL exit `77`.

  *Amended in the sixth edition, because the requirement was written against a
  shape the server does not produce.* It read: *IF the catalogue reports rows
  of table type `VIEW` while the view collection comes back empty, THEN …*.
  Observed against all four series, the view collection does **not** come back
  empty for a reader without `SHOW VIEW`: every row is present, carrying its
  name and its attributes, and only `VIEW_DEFINITION` is short — and it is
  short by being the empty string rather than `NULL`. The check as written
  could therefore never fire, on any supported server, against the exact
  reader it was written for. The corrected check is also strictly stronger:
  it fires per view rather than only when every view is unreadable, so a
  reader that can see some definitions and not others is caught too.

  *Rationale.* A view's definition is the view. The empty string is not a
  value a view can legitimately carry, so the observation admits one
  explanation: the reader can see that the view exists and cannot read it.
  Reporting "this view has no definition" would be a wrong answer wearing the
  appearance of a right one, and a generator acting on it would silently emit
  nothing.

- **FR-PRIV-017**: IF the catalogue reports a routine whose body is `NULL`,
  THEN the system SHALL treat that as a missing privilege, and the routine
  SHALL be incomplete under `FR-PRIV-002`.

  *Observed.* A reader holding `SELECT` and `EXECUTE` and no more sees every
  routine row and receives `NULL` for every `ROUTINE_DEFINITION`, on all four
  series. So the body is **not** readable with the privilege that lists
  routines, which is the second half of `OQ-037`.

  *Consequence, and it calibrates `FR-PRIV-003`.* `tpl schema routine` names
  an object, so such a reader receives `77` for every routine in the database
  rather than a stub. In a listing or a dump the routine is marked instead,
  per `FR-PRIV-005`, with `body` among the names in `restricted`. This is the
  ordinary case for a least-privilege reader, not an edge one: the privilege
  that exposes a routine body is not among those a read-only catalogue user is
  usually granted.

  *Partially closed* `OQ-037`; `FR-CAT-048` closes
  the rest, and the entry is now listed under
  [Closed](open-questions.md#closed).

- **FR-PRIV-019**: IF the catalogue reports a key column that names a
  referenced table while no referential-constraint row exists for the
  constraint that column belongs to, THEN the system SHALL treat that as a
  missing privilege and SHALL exit `77` for a named object, or mark under
  `FR-PRIV-005` in a listing or a dump.

  **Two tables lose a property, not one, and each loses a different one.** The
  key column names both of them, and `FR-CAT-045` presents one foreign key from
  both ends, so a rules row that is not there costs each end the direction it
  owns:

  | Table | Named by | Property lost | Required by |
  |---|---|---|---|
  | The **referencing** table | the key column's own table | `foreign_keys` | `FR-CAT-012` |
  | The **referenced** table | the key column's referenced-table field | `referenced_by` | `FR-CAT-013` |

  Each SHALL be marked under `FR-PRIV-005`, with that property named in its
  `restricted` array, per `FR-PRIV-016`, and per `FR-PRIV-006` each SHALL be
  marked individually. A table this requirement would mark that the read does
  not present SHALL NOT be marked, because there is no object to carry the
  marking; the coverage of `FR-CAT-052` decides which those are. For a named
  object, per `FR-PRIV-003`, the `77` is produced when **either** end is the
  object that was named.

  *Rationale.* This is the second place the catalogue offers two independent
  views of one population, and it is the one the third edition was waiting
  for. The two observations are contradictory — a column cannot reference a
  table under no constraint — and only one explanation fits. Without the
  check, `FR-CAT-012` and `FR-CAT-013` would report every table as having no
  foreign keys, with exit `0`, and a generator would emit a schema with no
  relations at all. That is a larger silent failure than the one
  `FR-PRIV-011` prevents.

  *Rationale for marking both, which is the whole of this amendment.*
  `FR-PRIV-002` is unconditional: an object is incomplete when any property the
  model defines for it could not be read, whatever the reason. Marking only the
  referencing end would leave the referenced table presenting an **empty
  `referenced_by`** as a complete answer at exit `0` — a table that is
  referenced reported as referenced by nothing — which is the silent failure
  this file exists to prevent, arriving from the end nobody was looking at. It
  is also the failure the *Rationale* above already names: without the check,
  `FR-CAT-012` **and** `FR-CAT-013` report nothing. That sentence cited
  `FR-CAT-013` beside `FR-CAT-012` from the edition that wrote it, which
  supports this reading and never stated it, while the requirement itself said
  "the table" and its antecedent named two.

  *Rejected: marking the referencing end alone.* It is the reading the shorter
  wording admitted, and it is wrong for the reason above. *Also rejected:
  marking the referenced end alone*, which reports the loss on the table that
  did not declare the key and leaves the declaring table looking as though it
  declared none. *Also rejected: a single marking on the document*, which
  `FR-PRIV-006` already forbids and which cannot say which tables are affected.

  *Accepted cost.* One missing rules row marks two objects, so a reduced reader
  produces more markings than there are lost rules. Over the fixture that is 14
  marked tables for 15 lost rules — 9 referencing tables and 9 referenced ones,
  4 of which are both. The alternative is a document in which half the loss is
  invisible, and `FR-PRIV-016` makes the marking per property precisely so that
  a caller can tell which half of a relation it lost.

  *Observed, and the count is stated over the population it was taken over,
  per `FR-PRIV-018`.* The reduced-grant reader receives **zero rows** from
  `INFORMATION_SCHEMA.TABLE_CONSTRAINTS` and from `REFERENTIAL_CONSTRAINTS`,
  and **all 54 rows** of the unfiltered `KEY_COLUMN_USAGE` for the database —
  of which **17** name a referenced table and are the rows this cross-check is
  evaluated over, per `FR-CAT-045`. It therefore sees every foreign-key column
  and not one foreign-key rule: the table appears structurally whole while its
  referential semantics are gone.

  *Amended in the twenty-seventh edition, in two places.* The requirement said
  the system SHALL "mark the table" while its antecedent names two tables, and
  it quoted a count of 54 against a population of 17. Both are settled above:
  both ends are marked, each with the property it lost, and each count names
  the population it was taken over.

- **FR-PRIV-021**: IF the schema catalogue returns no row for the database a
  read covers, THEN the system SHALL exit `77` (`EX_NOPERM`), and SHALL NOT
  present a database whose own metadata it could not read.

  The `cause` SHALL name that database and state that its metadata could not be
  read, per `FR-PRIV-013` and the `77` row of `FR-ERR-034`. The database a read
  covers is the one the selected entry names, per `FR-CONF-041`, so the `cause`
  has an instance to name in every case.

  *The requirement does not claim which of two explanations holds, and says so
  rather than implying one.* No row for that name is what a reader who may not
  see the database receives and what a reader of a database that is not there
  receives, and the catalogue offers no second view of the schema population to
  separate them — the shape is the zero-rows shape of `FR-PRIV-018`, arriving
  for the object the whole document describes. `FR-PRIV-010` requires that
  limit to be stated here, and this is where it is stated. The code is chosen
  on the caller's next step, which `FR-ERR-002` makes the test: under either
  explanation the step is to establish that this reader can read that database,
  which is the step the `77` row of `FR-ERR-001` states.

  *Added in the twenty-fifth edition.* The condition was reported as a violated
  internal invariant, exit `70`. It is not one. `FR-ERR-030` closes `70` to a
  panic and to an invariant the system detects in itself, and nothing about
  `tpl` is defective when a server declines to show a schema: the shortfall is
  in what the reader was shown, which is the subject of this file. A `70` also
  tells the caller, per its row of `FR-ERR-001`, that the condition is not
  fixable by them, when it is — by a grant, or by correcting
  `database.<name>.database`.

  *Rejected: `66`.* Its row of `FR-ERR-001` sends the caller to list what
  exists and choose another name, and `tpl` cannot support that step here: the
  nearest-match suggestion `FR-ERR-005` and `FR-ERR-019` attach to a missing
  name is drawn from a population, and the population of databases is one this
  system never reads. Obtaining it is a second catalogue statement on a path
  whose statement count `NFR-PERF-001` and `NFR-PERF-002` fix. It would also
  tell a caller whose name is correct that it is not.

  *Consequence, stated plainly, because it is a limit on the evidence rather
  than on the guarantee.* No invocation of the distributed binary is observed
  producing this condition, and this corpus does not establish that one can.
  The requirement exists because a reader can meet the absence of that row and
  must answer it with something, not because a reachable state has been
  demonstrated. What can be exercised is this system's own handling of a schema
  read that returned no row, in process, and the step from that condition to
  the exit status is the step every condition of `FR-ERR-001` travels — the
  composition `FR-ERR-031` records for `70`, arriving here for a different
  reason. `BR-ERR-001` is not weakened by it: `77` has the integration tests
  that rule mandates, through `FR-PRIV-003` with `FR-PRIV-011`, `FR-PRIV-017`
  and `FR-PRIV-019`. The other two requirements written this way are
  `FR-ERR-031` and `NFR-PERF-005`.

  *What would change this.* An observation, against a series of `FR-SRV-015`,
  of a session that is open and a schema catalogue that returns no row for the
  database that session was opened against; or a second view of the schema
  population in the catalogue, which would let the `cause` separate the two
  explanations. Neither has been made, and this note is what an amendment
  changes.

- **FR-PRIV-020**: The system SHALL NOT claim to distinguish a table with no
  triggers from a table whose triggers the reader may not see, and this file
  SHALL state that limit rather than leave it to be discovered.

  *Observed.* `INFORMATION_SCHEMA.TRIGGERS` returns zero rows to a reader
  without the privilege and zero rows for a table that has no triggers. The
  catalogue offers no second view of the trigger population — `KEY_COLUMN_USAGE`
  has no analogue here — so the two cases are identical in every byte a
  reader can obtain.

  *Consequence, stated plainly because it is a weaker guarantee than the words
  suggest.* A table whose triggers are hidden is reported as having none,
  complete, at exit `0`. `FR-PRIV-002` is not satisfied for it and cannot be:
  the system does not know that a property could not be read.

  *Rejected.* Inferring the privilege from the reader's grants by reading
  `INFORMATION_SCHEMA.USER_PRIVILEGES` or an equivalent. It is a second
  catalogue query on every read, `NFR-PERF-001` and `NFR-PERF-002` fix the
  query count, and the inference is not sound in any case — the grant tables
  do not settle what the current session can see through the catalogue's own
  filtering. Also rejected: marking every table `restricted` for triggers
  whenever the trigger collection is empty across the whole database, which
  reports a privilege problem for the common case of a database with no
  triggers at all.

  *What would change this.* A second view of the trigger population in the
  catalogue, or a reading that announces the privilege rather than being
  inferred from grants. Neither exists on any series of `FR-SRV-015`. If one
  appears, it is an amendment to this requirement and to `FR-PRIV-015`
  together, and the trigger gains the cross-check it currently admits none of.

  *A stated limit.* This requirement names where a guarantee stops, in the
  form the [README](README.md#writing-conventions) fixes for all three:
  `FR-SRV-041`, where a server determined to pass as MariaDB will pass, and
  `FR-CONF-039`, where pinned trust material is additional to the public root
  bundle rather than exclusive of it.

- **FR-PRIV-012**: The cross-checks of `FR-PRIV-011`, `FR-PRIV-017` and
  `FR-PRIV-019` SHALL be performed on every read that presents the property
  they guard, including a dump.

  *Amended in the sixth edition.* The requirement named the view cross-check
  alone, which was the only one that existed.

- **FR-PRIV-015**: The cross-check of `FR-PRIV-011` SHALL be performed for
  views, that of `FR-PRIV-019` for foreign keys, and no cross-check SHALL be
  performed for any other object kind.

  *Rationale.* A cross-check needs two observations of one population that can
  disagree, and the catalogue offers exactly two such pairs: the rows of table
  type `VIEW` against the readability of each view's definition, and the key
  columns that name a referenced table against the referential-constraint rows
  that should describe them. Tables and routines offer no second observation —
  a reader who cannot see a table does not see it counted somewhere else
  either — and triggers offer none, which is `FR-PRIV-020`.

  *Amended in the sixth edition, and the gap it recorded is now closed.* The
  third edition left open whether the check should be generalised, because the
  answer depended on what a reader without the privilege actually receives.
  `OQ-041` has been observed, and it generalises to
  exactly one further kind: the foreign key, through `KEY_COLUMN_USAGE`
  surviving a privilege that removes `REFERENTIAL_CONSTRAINTS` entirely. The
  routine body needed no cross-check at all — `NULL` is self-announcing — and
  the trigger admits none.

## Reporting

- **FR-PRIV-013**: A `77` produced under this file SHALL follow the message
  format of `FR-ERR-008`, and the `cause` line SHALL state which property could
  not be read.

- **FR-PRIV-014**: The system SHALL NOT include the reader's credentials in that
  message, per `FR-ERR-013`.

## Business rules

- **BR-PRIV-002**: `restricted` is a field of the document and therefore
  plumbing contract, subject to `FR-OUT-014`. `text` output is not a contract
  and this file does not constrain it. It is a field of the object it
  qualifies and never of the envelope, per `FR-OUT-029`.

- **BR-PRIV-003**: Completeness is a property of a read, not of a server. The
  same database read by two users can produce a complete document for one and a
  marked document for the other, and nothing in the model records which reader
  produced it.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the properties whose absence
  makes an object incomplete.
- [context-document.md](context-document.md) — the document the marking appears
  in.
- [render-command.md](render-command.md) — `FR-RND-020`, the code a refused
  context produces.
- [schema-commands.md](schema-commands.md) — `FR-SCH-021` and `FR-SCH-022`, the
  completeness the round-trip already depends on.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `77`, and the message
  format.
- [configuration-model.md](configuration-model.md) — `FR-CONF-041`, which names
  the database `FR-PRIV-021` reports on.
- [cache-commands.md](cache-commands.md) — `FR-CACHE-037`, which keeps a marked
  object out of the cache.
- [cfg-commands.md](cfg-commands.md) — `FR-CFG-044`, whose probe reports
  whether the catalogue is readable at all, and does not report completeness.

## Open questions

**None.** `OQ-041` is answered by `FR-PRIV-018` and by `FR-PRIV-011` as
amended, `OQ-047` by `FR-PRIV-016`, and `OQ-048` by `FR-CACHE-037`; all three
were closed in the sixth edition. The half of `OQ-037` that belongs to this
file — whether a routine body is readable without additional privilege — is
answered by `FR-PRIV-017`, and the seventh edition closed the routine field
list itself in `FR-CAT-048`. All four are listed under
[Closed](open-questions.md#closed).
