---
title: Catalogue Coverage
status: approved
last-reviewed: 2026-09-20
related: [context-document.md, schema-commands.md, server-contract.md, privileges-and-completeness.md]
---

# Catalogue Coverage

## Overview

The first edition defined the commands that read a database. This file defines
the material those commands present: which object kinds enter the model, which
of their properties enter it, and which properties are excluded and why. It is
the answer to "what does `tpl` know about a database", stated once, so that
every command, every output format, and every template sees the same set.

The model is the same whatever the source. A live read, a cached read, and a
`--context` document present the objects and fields defined here and no others.

## Scope

In scope: the covered object kinds, the properties of each, the catalogue field
list behind each and the rule by which a field list becomes a property list,
the rule for a catalogue value outside a set this file records as closed, what
the model carries where a field it carries returns SQL `NULL`, the closed
exclusion of volatile catalogue fields, the closed exclusion of fields whose
meaning differs between supported series, the enumerated exclusions of whole
features and of the foreign key that crosses a schema boundary, the observed
catalogue behaviour that constrains what the model can carry, and the position
of coverage filtering relative to `--pattern`.

Out of scope: how the model is encoded as JSON, which belongs to
[context-document.md](context-document.md); which statements read it, which
belongs to [server-contract.md](server-contract.md); what happens when a read
comes back incomplete, which belongs to
[privileges-and-completeness.md](privileges-and-completeness.md); and the text
of any catalogue query, which this specification does not state.

## Actors

- **Calling agent** or **operator**, reading the model through a command.
- **Server catalogue**, the origin of every field in the model.
- **Template**, which sees the model as the `database` context variable.

## Covered object kinds

- **FR-CAT-001**: The model SHALL cover tables whose catalogue table type is
  `BASE TABLE` or `SYSTEM VERSIONED`.

- **FR-CAT-002**: Every table in the model SHALL carry `table_type`, stating
  which of the two covered types it is.

  *Rationale.* The two are covered together because both are ordinary tables to
  a generator, and both carry columns, keys, and indexes. A template that must
  treat a system-versioned table differently needs the distinction stated rather
  than inferred from a column name.

- **FR-CAT-003**: The model SHALL NOT present a view among tables. Views are a
  collection of their own, per `FR-CAT-007`.

- **FR-CAT-004**: The model SHALL NOT cover tables whose catalogue table type is
  `SEQUENCE`.

- **FR-CAT-005**: The model SHALL NOT cover tables whose catalogue table type is
  `SYSTEM VIEW`.

- **FR-CAT-006**: The model SHALL NOT cover temporary tables, whether or not the
  catalogue reports them. Whether it reports them differs between series, and
  `FR-CAT-032` requires the exclusion to be a filter rather than a reliance.

- **FR-CAT-007**: The model SHALL cover views, each with its SQL definition, per
  `FR-SCH-006`.

- **FR-CAT-008**: The model SHALL cover routines — stored procedures and stored
  functions together — each stating its kind, per `FR-SCH-007`.

- **FR-CAT-031**: The catalogue table types a series of `FR-SRV-015` can emit
  SHALL be taken to be exactly the following, as observed on 2026-09-10 against
  all four:

  | Table type | `12.3` | `11.8` | `11.4` | `10.11` | Covered |
  |---|---|---|---|---|---|
  | `BASE TABLE` | yes | yes | yes | yes | `FR-CAT-001` |
  | `SYSTEM VERSIONED` | yes | yes | yes | yes | `FR-CAT-001` |
  | `VIEW` | yes | yes | yes | yes | `FR-CAT-003`, `FR-CAT-007` |
  | `SEQUENCE` | yes | yes | yes | yes | excluded, `FR-CAT-004` |
  | `SYSTEM VIEW` | yes | yes | yes | yes | excluded, `FR-CAT-005` |
  | `TEMPORARY` | yes | yes | yes | **never emitted** | excluded, `FR-CAT-006` |

  *Closes the first half of* `OQ-025`, now listed under
  [Closed](open-questions.md#closed). The strings are the values the coverage
  predicate of `FR-CAT-001` through `FR-CAT-006` compares against, and they
  are contract surface once `table_type` reaches the document under
  `FR-CAT-002`.

- **FR-CAT-032**: The exclusion of temporary tables required by `FR-CAT-006`
  SHALL be implemented as a filter on the table type, and SHALL NOT rely on the
  catalogue omitting them.

  *Observed.* After a `CREATE TEMPORARY TABLE`, `10.11` returns no row for it
  in `INFORMATION_SCHEMA.TABLES` at all, while `11.4`, `11.8` and `12.3` each
  return one with `TABLE_TYPE='TEMPORARY'`. On no series does such a table
  appear in `INFORMATION_SCHEMA.COLUMNS` or `INFORMATION_SCHEMA.STATISTICS`.
  Recorded as difference 4 of `FR-SRV-038`.

  *Rationale.* `FR-CAT-006` was written to hold "whether or not the catalogue
  reports them", and the observation is why that clause earns its place: on one
  supported series the exclusion is automatic and on the other three it is not.
  A reader developed against `10.11` alone would pass every test and then
  present temporary tables — without columns or indexes, because those tables
  reach neither of the other two catalogue tables — against `11.4` and later.
  That is the shape of failure `FR-SRV-029` exists to catch, and it is
  cheaper to forbid the reliance than to detect it.

  *Closes the second half of* `OQ-025`, now listed under
  [Closed](open-questions.md#closed).

## What a table carries

- **FR-CAT-009**: A table SHALL carry its columns, including generated columns
  and invisible columns, in ordinal position order per `NFR-DET-002`.

- **FR-CAT-010**: An index SHALL be one object carrying its columns in the order
  the catalogue states, folded from the one-row-per-column form in which the
  catalogue reports it.

  *Rationale.* An index is one thing with an ordered column list. Presenting it
  as several rows would make every template perform the same grouping, and each
  one would order the columns differently.

- **FR-CAT-011**: A table SHALL carry its primary key, read from the catalogue
  source `FR-CAT-043` names and from no other.

  *Amended in the seventh edition.* The requirement named no source. The
  observation of 2026-09-10 found that the three places a primary key is
  reachable from **disagree** on a system-versioned table, and that one of
  them names a column the model does not carry, so the source is now part of
  the requirement. `FR-CAT-043` names it and `FR-CAT-044` states the invariant
  the choice guarantees.

- **FR-CAT-012**: A table SHALL carry its outgoing foreign keys, each with its
  `ON UPDATE` and `ON DELETE` rule, read from the two catalogue tables
  `FR-CAT-045` names.

- **FR-CAT-013**: A table SHALL carry its incoming foreign keys, under
  `referenced_by`.

  *Rationale.* Without the incoming direction, the has-many side of a relation
  is invisible to a template, and a generator can emit the belongs-to accessor
  but not its counterpart. The catalogue holds both directions; presenting only
  one would be a choice, not an economy.

- **FR-CAT-014**: A table SHALL carry its triggers.

- **FR-CAT-015**: A table SHALL carry its `CHECK` constraints, each with its
  name, its level, and its clause.

  *Two observed consequences, both stated under `FR-CAT-037` and
  `FR-CAT-038`.* A column-level constraint is named after the column it
  qualifies, so a constraint name is not guaranteed distinct from a column
  name of the same table; and MariaDB attaches an implicit `json_valid`
  constraint to every `JSON` column and reports it identically, so the model
  carries constraints nobody wrote. Excluding those would be a change to this
  requirement.

## What a routine carries

- **FR-CAT-016**: A routine SHALL carry `kind`, stating whether it is a
  procedure or a function. The value SHALL be the catalogue's own routine-type
  string, carried unchanged, and WHERE the server's `standing` is `supported`,
  per `FR-CTX-034`, it SHALL be exactly one of the two strings `PROCEDURE` and
  `FUNCTION`. A value outside those two is carried unchanged under
  `FR-CAT-055`.

  *Amended in the twenty-third edition: the emitted value is stated.* The
  requirement read "A routine SHALL state its kind: procedure or function",
  which names the two kinds and fixes no string. A template or a `jq` filter
  branching on `kind` therefore had no casing to branch on, and a generator
  that guessed wrong matched nothing and exited `0`, which is silence rather
  than a failure. The two strings are the values `FR-CAT-048` records the
  catalogue returning for the routine-type field, and this requirement carries
  them through rather than composing a value of its own.

  **The value is not the prefix of the qualified form of `FR-SCH-008`, and the
  two are not the same string.** That requirement admits `procedure:<name>`
  and `function:<name>` on the command line, in lower case; this field carries
  `PROCEDURE` and `FUNCTION`, in upper case. A caller composing a qualified
  name from `kind` folds the case, and that is the whole of the difference:
  the two kinds are the same two, their spelling is otherwise identical, and
  neither requirement admits a third value. `FR-SCH-008` states what a caller
  that does not fold receives — it is `64`, decided from the token alone —
  and states it there rather than here, because the prefix is a token of the
  command line and this field is a value of the document.

  *Amended in the twenty-fifth edition.* The last sentence is new. The
  obligation to fold was stated in the edition above and its consequence was
  stated nowhere, so a caller that concatenated `kind` unfolded met a token
  neither this requirement nor `FR-SCH-008` governed. Nothing about the field
  changes: it carries the catalogue's own string, in upper case, as the
  twenty-third edition fixed it.

  *Amended in the twenty-seventh edition: the word "therefore" is given the
  server it depends on.* The first paragraph read that the value carries the
  catalogue's string unchanged and **SHALL therefore be exactly one of the two
  strings**, which is true of every series of `FR-SRV-015` and is not a
  property of the field. The routine-type column is declared `varchar(13)` and
  not an `ENUM`, on all four series, so nothing but the observation of
  `FR-CAT-048` closes the set. The sentence now reads: the value SHALL be
  exactly one of the two strings **on a server whose `standing` is
  `supported`**, per `FR-CTX-034`; a third string is carried unchanged under
  `FR-CAT-055`, which also states the one place a third kind is not reachable —
  the qualified name of `FR-SCH-008`. Nothing about a supported server changes,
  and the two strings are the same two.

  *Rejected.* Carrying `kind` in lower case so that it equals the prefix
  `FR-SCH-008` admits. It would make `kind` the only enumerated catalogue
  value the model rewrites: `table_type` carries `BASE TABLE` and
  `SYSTEM VERSIONED` exactly as `FR-CAT-031` observed them, and a view's
  `check_option`, `is_updatable` and `security_type`, a routine's `body_kind`,
  `parameter_style`, `is_deterministic`, `sql_data_access` and
  `security_type`, a trigger's `event`, `timing` and `orientation`, and a
  parameter's `mode` are each the catalogue's own string, per `FR-CAT-047`,
  `FR-CAT-048`, `FR-CAT-049` and `FR-CAT-050`. One field folded and its
  neighbours not is a rule every reader of the document has to memorise, and
  it buys one case fold at the one place a qualified name is composed. Also
  rejected: leaving the casing to the implementation and stating only the two
  kinds, which is the defect being corrected — an unstated discriminant is a
  contract surface fixed by whoever writes the first generator, and fixed
  where no reader can find it.

- **FR-CAT-017**: A routine SHALL carry its body.

- **FR-CAT-018**: A routine SHALL carry its parameters in declaration order.

## Observed catalogue behaviour

The facts in this section were observed on 2026-09-10 against all four series
of `FR-SRV-015`, using the fixture of `scripts/mariadb/`, and are recorded here
because each constrains what the model can carry or what a template can be
told. Every one of them holds identically on all four series unless the
requirement says otherwise; a difference between series is recorded under
`FR-SRV-038` instead.

- **FR-CAT-033**: The model SHALL NOT report `SET DEFAULT` as the value of a
  foreign key's `ON UPDATE` or `ON DELETE` rule.

  *Observed.* `ON DELETE SET DEFAULT ON UPDATE SET DEFAULT` is accepted by all
  four series **without error and without warning**, and the catalogue then
  reports both rules as `RESTRICT`; `SHOW CREATE TABLE` omits the clause
  entirely. The rule is unrepresentable rather than unsupported: the server
  discards it silently on the way in, so no read can ever return it.

  *Rationale.* MariaDB names five referential actions and the catalogue can
  return only four of them. A model that enumerated five would carry a value
  no read can produce, and a generator written against that enumeration would
  carry a branch that never executes. Stating the absence is also the only way
  a reader learns that a `SET DEFAULT` in someone's DDL has silently become a
  `RESTRICT` in the database, which is a fact about their schema and not about
  `tpl`.

  *Partially closes* `OQ-034`; `FR-CAT-045` closes
  the rest, and the entry is now listed under
  [Closed](open-questions.md#closed).

- **FR-CAT-034**: The model SHALL treat a comma as a safe delimiter for the
  members of a `SET` and SHALL NOT treat it as one for the members of an
  `ENUM`.

  *Observed.* A `SET` member containing a comma is rejected by all four series
  at DDL time with `ERROR 1367 (22007)`, because the comma is the storage
  delimiter of the type; such a member cannot exist and cannot be read. An
  `ENUM` member containing a comma is accepted by all four, and so is one
  containing an apostrophe; the fixture carries three of the first and one of
  the second.

  *Rationale.* The two types look alike and are not. This is the fact that
  decides whether the naive split of `OQ-029`
  corrupts data: for `SET` it cannot, for `ENUM` it can, and a reader that
  learns the rule from one type and applies it to the other silently truncates
  a member list.

  *Partially closes* `OQ-029`; `FR-CTX-039` closes
  the rest, and the entry is now listed under
  [Closed](open-questions.md#closed).

- **FR-CAT-035**: The catalogue field that reports a column's invisibility
  SHALL be taken to be the column attribute field, whose value is `INVISIBLE`,
  on every series of `FR-SRV-015`.

  *Observed.* `INFORMATION_SCHEMA.COLUMNS.EXTRA` reads `INVISIBLE` on all
  four. The keyword's position in `SHOW CREATE TABLE` differs between series —
  difference 2 of `FR-SRV-038` — and does not reach `tpl`, which `FR-SRV-007`
  bars from issuing `SHOW` at all.

  *Partially closes* `OQ-026`; `FR-CAT-041` closes
  the rest, and the entry is now listed under
  [Closed](open-questions.md#closed).

  *Amended in the seventh edition, and the amendment is a bound rather than a
  change.* The wording *whose value is `INVISIBLE`* was written from an
  observation of columns that carry **one** attribute each, which is every
  column of the fixture. `FR-CAT-041` records that the same field carries five
  further values and that no column carrying two at once was observed, so this
  requirement may not be read as establishing that the field holds exactly one
  attribute. What it fixes is the token, not the shape of the field.

- **FR-CAT-036**: The model SHALL NOT promise that a comment it carries is the
  comment the author wrote.

  *Observed.* The `INFORMATION_SCHEMA` comment columns are `utf8mb3`. A
  supplementary-plane character in a table or column comment is **silently
  replaced by `?` when the DDL is parsed**, with `warning_count` left at 0, so
  the substitution happens in the server before any read and cannot be
  detected by one.

  *Rationale.* The loss is the server's and is invisible to `tpl`, which is
  exactly why it must be stated: without it, a reader who finds a `?` in a
  generated file looks for the bug in `tpl`. It also bounds what `FR-OUT-017`
  can promise about the encoding of a comment — the value is always valid
  `utf8mb3`, whatever the author supplied.

  *Partially closes* `OQ-009`; `FR-CAT-039` and
  `FR-CTX-037` close the rest, and the entry is now listed under
  [Closed](open-questions.md#closed).

  *Extended in the seventh edition.* The column-default field is declared
  `utf8mb3` in the catalogue for the same reason the comment fields are, so
  the loss this requirement describes reaches a **string default** as well as
  a comment. `FR-CAT-039` records the observation.

- **FR-CAT-037**: A `CHECK` constraint declared at column level SHALL be
  carried under the name the catalogue gives it, which is the name of the
  column it qualifies, with its level reported as `Column`.

  *Observed.* A column-level `CHECK` appears in
  `INFORMATION_SCHEMA.CHECK_CONSTRAINTS` with `CONSTRAINT_NAME` equal to the
  column name and `LEVEL` equal to `Column`, on all four series.

  *Consequence, stated rather than left to be discovered.* A check constraint
  name is therefore **not** guaranteed to be distinct from a column name of the
  same table, and a template that keys a map on the one can collide with the
  other.

  *Partially closes* `OQ-035`; `FR-CAT-046` closes
  the rest — the field list is six fields and the level takes `Table` beside
  `Column` and nothing else — and the entry is now listed under
  [Closed](open-questions.md#closed).

- **FR-CAT-038**: A column declared `JSON` SHALL be carried as the type the
  catalogue reports for it, which is `longtext`, and the model SHALL NOT
  present it as a distinct type.

  *Observed.* MariaDB implements `JSON` as an alias, and the catalogue reports
  such a column as `longtext` on all four series. The server additionally
  attaches an **implicit `json_valid` check constraint** to the column and
  reports it exactly as `FR-CAT-037` describes an authored column-level check:
  named after the column, at level `Column`.

  *Consequence.* Under `FR-CAT-015` a table carries its `CHECK` constraints
  with no exception, so the implicit constraint enters the model, and it is
  distinguishable from an authored one only by reading its clause. Excluding
  it would be a change to `FR-CAT-015`, not an inference from this
  requirement. The only trace of the `JSON` declaration that survives into the
  catalogue is that constraint, so a template cannot otherwise tell a `JSON`
  column from a `LONGTEXT` one.

  *Partially closes* `OQ-028`, which `FR-CTX-038`
  closes, and bears on `OQ-072`, which `FR-ENV-046`
  closes: `json` is not a value `data_type` takes, so it is a member of no
  family, and a column declared `JSON` falls in the **textual** family with
  every other `longtext` column.

## The catalogue field lists

The requirements in this section fix, for each object kind the model covers,
which catalogue fields the model reads and which it does not. They were
written from an observation pass run on 2026-09-10 against all four series of
`FR-SRV-015`, using the fixture of `scripts/mariadb/` — 23 catalogue objects,
301 columns, 77 index rows, 68 constraints, 54 key columns, 15 referential
rules, 24 check constraints, 5 views, 7 routines, 22 parameters, 6 triggers.
Every value quoted below was returned by all four series unless the
requirement says otherwise.

- **BR-CAT-005**: A catalogue field list is not the same thing as a model
  property list, and the difference between them is stated once here rather
  than argued per object. The model carries **every field the catalogue
  provides for an object**, except a field excluded on one of four grounds:

  | Ground | What it excludes | Fixed by |
  |---|---|---|
  | Row identity | The catalogue name and the schema name, which locate the row in `INFORMATION_SCHEMA` rather than describe the object, and the field that names the object the row hangs from where the model already carries it there | This rule |
  | Volatility | A field on the closed list of `FR-CAT-024` | `FR-CAT-024`, `FR-CAT-025` |
  | Ambiguous meaning | A field on the closed list of `FR-CAT-029` | `FR-CAT-029`, `FR-CAT-030` |
  | Restatement | A field that repeats a fact the model already carries in one place, which `FR-CTX-021` forbids materialising twice | `FR-CTX-021` |

  A fifth case is not an exclusion and is stated so it is not mistaken for
  one. **Nothing observed**: a field that held the same *absent* value for
  every object of the fixture on every series — SQL `NULL` throughout, or the
  empty string throughout — is named in the requirement that covers its
  object kind and is **not carried**, because carrying it would carry
  nothing. Admitting such a field later requires an observation of it holding
  a value, not a decision.

  A field that held the same *populated* value throughout is a different case
  and **is carried**. `NONE` for a foreign key's match option and `NO` for an
  index's ignored flag are answers, not absences, and the fixture exercising
  one value of a field is a bound on the claim rather than a ground for
  dropping it — recorded as a bounded claim beside the requirement. The
  asymmetry is deliberate: carrying a field a template ignores costs bytes,
  and omitting one it needs cannot be recovered from without amending this
  specification.

  *Rationale.* Carrying by default and excluding by stated ground is the only
  rule under which a reader can check the specification against the catalogue.
  The alternative — enumerating what to carry — makes every omission
  indistinguishable from an oversight, which is exactly the state
  `OQ-036` through
  `OQ-040` left the corpus in for five editions.

  *Rejected.* Carrying only what a code generator was judged to need. It is
  the narrower model `BR-CAT-001` already rejected once, at object-kind
  granularity, and at field granularity it is worse: the judgement is made
  once, by whoever writes the requirement, and a template author who needs the
  field has no way to recover it.

- **FR-CAT-055**: IF a catalogue field this file records as taking a closed
  set of values returns a value outside that set, THEN the system SHALL carry
  the catalogue's own string unchanged. It SHALL NOT substitute a recorded
  value, SHALL NOT drop the object the field belongs to, SHALL NOT refuse the
  read, and SHALL NOT change the exit code on account of it.

  The rule reaches **six model properties over five value sets**, and no
  others:

  | Model property | Value set | Recorded by |
  |---|---|---|
  | `on_update`, `on_delete` | the four referential actions | `FR-CAT-045`, with `FR-CAT-033` |
  | `level` | `Table` and `Column` | `FR-CAT-046`, with `FR-CAT-037` |
  | `event` | `INSERT`, `UPDATE`, `DELETE` | `FR-CAT-050` |
  | `timing` | `BEFORE`, `AFTER` | `FR-CAT-050` |
  | `kind` | `PROCEDURE` and `FUNCTION` | `FR-CAT-016` |

  **`table_type` is not one of them**, and the difference is the field's job
  rather than its shape. `FR-CAT-001` through `FR-CAT-006` make that value a
  coverage predicate, so a value outside the set `FR-CAT-031` records selects
  no covered kind and the object is not presented — which is the same outcome
  `FR-CAT-004` and `FR-CAT-005` produce for a value the set does hold. Every
  other enumerated-looking field the model carries — a foreign key's
  `match_option`, a trigger's `orientation`, a view's `check_option`,
  `is_updatable` and `security_type`, a routine's `body_kind`,
  `parameter_style`, `is_deterministic`, `sql_data_access` and
  `security_type`, and a parameter's `mode` — is already carried as the
  catalogue's own string under a bounded claim, so this requirement changes
  nothing for it.

  *Observed, 2026-09-20, against all four series of `FR-SRV-015` through the
  harness of `scripts/mariadb/`.* **The catalogue does not close any of the
  five sets.** Every field behind them is declared a plain `varchar` and
  `NOT NULL`, never an `ENUM`: the referential-constraint table's update rule
  and delete rule at `varchar(64)`, the check-constraint table's level at
  `varchar(6)`, the trigger table's timing at `varchar(6)`, and the routine
  table's routine type at `varchar(13)` — all identically on all four series.
  The trigger table's event column is the one that is **not** identical:
  `varchar(6)` on `10.11`, `11.4` and `11.8` and `varchar(20)` on `12.3`,
  which is difference 9 of `FR-SRV-038`, re-confirmed by this reading. The
  table catalogue's table type, which the clause above excepts, is
  `varchar(64)` and `NOT NULL` on all four. The five sets are therefore closed
  by the observations that recorded them and by nothing the server declares,
  and the declared width behind one of them has already grown inside the
  window.

  *Rationale, and it is `FR-SRV-031` read in its own terms.* The one
  foreseeable trigger for a value outside a set is a server newer than the
  window, and that server is one this corpus promises to **read and mark**, not
  to refuse: `FR-SRV-031` reads it, `FR-SRV-032` carries the marking in the
  document, and `FR-SRV-033` states that such a document promises the shape of
  `FR-SRV-005` and no more — naming, in its own text, a field whose meaning
  may have changed. A value outside a recorded set is that case arriving, so
  the treatment `FR-SRV-033` already describes is the treatment it gets.
  `standing` is the field that says the read is unverified, and there is no
  second thing for an exit code to add.

  *Rejected: refusing with `70` (`EX_SOFTWARE`).* It is what the first
  implementation of the reader does, and it is wrong on two counts.
  `FR-ERR-030` closes `70` to a panic and to an invariant the system detects
  **in itself**, and nothing about `tpl` is defective when a server returns a
  value `tpl` has not seen — the same correction `FR-PRIV-021` made in the
  twenty-fifth edition for a schema catalogue that returns no row. And the
  `70` row of `FR-ERR-001` tells the caller the condition is not fixable by
  them, which makes a whole database unreadable on account of one rule of one
  key on a server whose only fault is being newer than the binary. That is the
  outcome `FR-SRV-031` rejected when it rejected refusing with `78`.

  *Rejected: substituting the nearest recorded value.* It puts a value in the
  document the server did not state, at exit `0`. *Also rejected: dropping the
  object that carries the field.* It presents a table without a foreign key it
  has, also at exit `0`. Both are the silent corruption the amendment to
  `NFR-DET-002` names as the class of failure this corpus works hardest to
  prevent, and neither is distinguishable by a caller from a correct read.

  *Consequence, stated plainly.* The values each requirement records remain
  what a **supported** series returns, and a template that branches on them
  branches correctly on every server of `FR-SRV-015`. A value outside a set on
  a server whose `standing` is `supported` falsifies the requirement that
  recorded the set, in the way the fourth provenance of the
  [README](README.md#provenance) makes every observation falsifiable; it is
  answered by a second observation and an amendment, not by an exit code.

  *One consequence reaches beyond the document, and it reaches `kind` alone.*
  `FR-SCH-008` admits the two qualified prefixes `procedure:` and `function:`
  and no third, and `FR-CDOC-014` builds a cache path from the same two. A
  routine whose kind is outside the two is therefore carried in the document
  and is **not reachable by a qualified name**; its bare name still reaches it
  unless it is ambiguous, per `FR-SCH-010`. Widening either of those two
  requirements is a change to them and not an inference from this one.

- **FR-CAT-056**: Six catalogue fields the model carries are declared to admit
  SQL `NULL`. The model SHALL carry each as this table states, and SHALL NOT
  substitute the empty string for any of them:

  | Catalogue field | Model property | Declared | Observed over the fixture | What the model carries for SQL `NULL` |
  |---|---|---|---|---|
  | trigger definer | `definer`, `FR-CAT-050` | nullable, `varchar(384)` | populated on all 6 triggers | `null` |
  | trigger action statement | `statement`, `FR-CAT-050` | nullable, `longtext` | populated on all 6 triggers | `null` |
  | routine definition | `body`, `FR-CAT-048` and `FR-CAT-017` | nullable, `longtext` | populated on all 7 routines | `null`, and the routine is incomplete per `FR-PRIV-017` |
  | referential-constraint unique-constraint name | `referenced_key`, `FR-CAT-045` | nullable, `varchar(64)` | populated on all 15 rules | `null` |
  | referential-constraint referenced table name | `referenced_table`, `FR-CAT-045` | nullable, `varchar(64)` | populated on all 15 rules | `null`, and see the clause below |
  | key-column referenced column name | `referenced_column`, `FR-CAT-045` | nullable, `varchar(64)` | populated on all 17 rows the read presents, SQL `NULL` on the other 37 | no row carrying one reaches the model — see the clause below |

  `null` is the absent scalar of `FR-CTX-005`, emitted rather than omitted per
  `FR-OUT-012`. None of the six is the *nothing observed* case of
  `BR-CAT-005`: each was populated on every row the read presents, so each is
  carried, and this requirement fixes only what happens when one is not.

  **The last row is settled by the read's population rather than by a value.**
  The key-column table carries one row per column of **every** key, and its
  referenced-column field is SQL `NULL` on every row that is not a foreign
  key's. `FR-CAT-045` restricts the read to the rows that name a referenced
  table, and those are exactly the rows on which the field is populated, so
  the substitution has no reachable case. The model carries a string there,
  not an absent scalar.

  **The fifth row has a structural consequence and the others do not.** A key
  whose referenced table is `null` names no table, so `FR-CTX-006` has nothing
  to embed and the key contributes to no table's `referenced_by` under
  `FR-CAT-013`; it is carried on the referencing table with its name, its
  columns and its rules. `FR-CTX-006` states the same limit from the
  document's side.

  *Observed, 2026-09-20, against all four series of `FR-SRV-015` through the
  harness of `scripts/mariadb/`.* All six fields are declared nullable, with
  the declared types above, identically on all four series. Over the `freight`
  schema, identically on all four series: the trigger definer and action
  statement are populated on all 6 trigger rows; the routine definition is
  populated on all 7 routine rows for a privileged reader; the
  unique-constraint name and the referenced table name are populated on all 15
  referential-constraint rows; and the referenced column name is SQL `NULL` on
  37 of the 54 key-column rows and populated on all 17 of the rows a foreign
  key contributes.

  *Bounded claim.* **No row of the fixture returns SQL `NULL` in any of the
  first five fields**, so what this requirement fixes for them is a shape the
  model must have and not a value that has been seen. The declaration is the
  evidence: a field the catalogue declares nullable is one a server may return
  `NULL` in, and a model that cannot represent that has a gap whether or not
  the fixture reaches it. Admitting the absence costs a `null` a caller already
  has to handle under `FR-OUT-012`; refusing it costs a wrong value with no
  way back.

  *Rejected: substituting the empty string, which is what the first
  implementation of the reader does for five of the six.* It makes an absent
  value indistinguishable from a present empty one, which is precisely the
  distinction `FR-PRIV-011` depends on for a view definition and `FR-CAT-048`
  records twice over for a routine's two absent-value shapes — the data-type
  field empty and the DTD identifier `NULL`, side by side and different. It is
  worst for the routine body, where the empty string is the substitution and
  `FR-PRIV-017` makes SQL `NULL` there a **missing privilege**: the two cases
  a reader most needs apart would arrive identical.

  *Also rejected: declaring the five fields `NOT NULL` in the model on the
  strength of the fixture.* The fixture populating a field on every row it has
  is not the catalogue promising to, and the catalogue's own declaration says
  the opposite. Writing the stronger claim down would be the inference the
  fourth provenance of the [README](README.md#provenance) forbids, pointing
  the other way.

### Comments and defaults

- **FR-CAT-039**: The catalogue value types for the three fields of
  `OQ-009` SHALL be taken to be the following, on
  every series of `FR-SRV-015`:

  | Field | Absent value arrives as | Never |
  |---|---|---|
  | A table's comment | the **empty string** | SQL `NULL` |
  | A column's comment | the **empty string** | SQL `NULL` |
  | A column's default | see `FR-CTX-037` — SQL `NULL`, or the four-character string `NULL` | the empty string |

  *Observed.* Over the fixture's 23 objects and 301 columns, identically on
  all four series: the table comment is the empty string for the one object
  that carries none and is never SQL `NULL`; the column comment is the empty
  string on 159 columns and non-empty on 142, and is never SQL `NULL`; the
  column default is SQL `NULL` on 132 columns and a non-empty string on 169,
  and is never the empty string. The empty string **is** a legal default
  value, and it arrives as the two characters `''` rather than as a
  zero-length value — `consignment.handling_flags` is the fixture's case.

  *Consequence for `FR-OUT-017`.* The three fields are declared `utf8mb3` in
  the catalogue itself, so the value a read returns is always valid `utf8mb3`
  whatever the author supplied — which is what `FR-CAT-036` already states for
  a comment and which holds for a default for the same reason. The
  invalid-UTF-8 replacement of `FR-OUT-017` therefore has no known trigger in
  these three fields, and remains in force for the fields it does reach.

  *Closes* `OQ-009`, with `FR-CAT-036` and
  `FR-CTX-037`, now listed under [Closed](open-questions.md#closed).

- **FR-CAT-040**: The model SHALL NOT present the table comment of a view as a
  comment of that view.

  *Observed.* `INFORMATION_SCHEMA.TABLES` carries a row for every view, and
  its comment field on that row holds the four-character literal `VIEW` — not
  the empty string and not SQL `NULL` — on all five views of the fixture and
  on all four series. For the fixture's sequence the same field is the empty
  string.

  *Rationale.* The value is a constant the server writes, not text an author
  supplied, and `CREATE VIEW` offers no clause that could put anything else
  there. Carrying it would put the string `VIEW` into the comment slot of
  every view in every generated file, and a template could not tell it from an
  authored comment. This is the same failure `FR-CAT-036` guards against — a
  value the reader would take for the author's — arriving from the other
  direction, and it is stated rather than inferred because the field is
  populated rather than absent, so nothing in a naive read would flag it.

### Columns

- **FR-CAT-041**: The catalogue field that carries a column's static
  attributes SHALL be taken to be the column attribute field, and the values
  it was observed to take SHALL be taken to be the following, no other value
  having been observed:

  | Value | Fact it reports | Carried under |
  |---|---|---|
  | the empty string | the column carries no such attribute | — |
  | `auto_increment` | the column is auto-incremental | `FR-CAT-027`, `FR-CTX-020` |
  | `INVISIBLE` | the column is invisible | `FR-CAT-035`, `FR-CTX-020` |
  | `VIRTUAL GENERATED` | the column is generated and not stored | `FR-CAT-051` |
  | `STORED GENERATED` | the column is generated and stored | `FR-CAT-051` |
  | `on update current_timestamp()`, `on update current_timestamp(3)` | the column carries an `ON UPDATE` default | `FR-CTX-020` |

  The spelling of each value is contract surface once the attribute it reports
  reaches the document.

  *Observed.* Over the fixture's 301 columns: the empty string on 277,
  `auto_increment` on 12, `INVISIBLE` on 1, `STORED GENERATED` on 6,
  `VIRTUAL GENERATED` on 3, and the two `on update` forms on one column each.
  The strings are byte-identical on all four series. Note the case: three of
  the values are upper case and three are lower case, and `auto_increment` is
  lower case with an underscore.

  *Closes* `OQ-026`, with `FR-CAT-035`, now listed
  under [Closed](open-questions.md#closed).

  *Bounded claim, and it is the one place this field is not settled.* **No
  column of the fixture carries two attributes at once**, so how the field
  renders a column that is both generated and invisible, or both
  auto-incremental and invisible, was not observed. A reader that compares the
  whole field against one of the six values above is therefore correct for
  every case that has been observed and may be wrong for a case that has not.
  `FR-CAT-035` and this requirement are both written as *the field reads
  `X`*, and neither may be read as licence to assume the field can hold only
  one attribute. Settling it needs DDL the fixture does not carry.

  *The `ON UPDATE` value is a sixth thing this field carries*, and it reaches
  the model because `FR-CTX-020` requires a column to carry the static
  attributes the catalogue states for it and names three of them with an
  *including*. An `ON UPDATE` default is such an attribute: it is part of the
  column's declaration, it does not move without a change to the structure,
  and it is not reported anywhere else — the column-default field of
  `FR-CTX-037` carries the `DEFAULT` clause and says nothing about `ON
  UPDATE`.

- **FR-CAT-051**: A generated column SHALL carry its expression and its
  storage kind, and the model SHALL take the storage kind from the column
  attribute field of `FR-CAT-041` and not from the is-generated field.

  *Observed.* Over the fixture's nine generated columns, identically on all
  four series:

  | Catalogue field | What it holds |
  |---|---|
  | the column attribute field | `VIRTUAL GENERATED` (3 columns) or `STORED GENERATED` (6) |
  | the is-generated field | `ALWAYS` on all nine, `NEVER` on the other 292 |
  | the generation-expression field | the expression, identifiers **backtick-quoted** and function names lower-cased, carrying no `AS`, no enclosing parentheses, and neither the `VIRTUAL` nor the `STORED` keyword |

  A DDL that wrote `ROUND(declared_value * COALESCE(insurance_rate, 0), 2)` is
  returned as ``round(`declared_value` * coalesce(`insurance_rate`,0),2)``.
  The is-generated field says only **whether**, never **how**, which is why
  the storage kind must come from the attribute field.

  *Two further facts about a generated column, both observed and neither
  obvious.* Its nullability reads nullable on all nine, whatever the
  expression; and its default is the four-character string `NULL`, exactly as
  for any other nullable column with no default, per `FR-CTX-037`. It is
  otherwise unmarked: it appears at its declared ordinal position like any
  other column and can be indexed — three of the fixture's indexes are.

  *Closes* `OQ-040`, now listed under
  [Closed](open-questions.md#closed).

### Indexes and the primary key

- **FR-CAT-042**: The catalogue field list for an index SHALL be taken to be
  seventeen fields, and the model SHALL carry the properties named in the
  third column:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema, index schema | `def` and the database name | not carried — row identity, `BR-CAT-005` |
  | table name | the table the index belongs to | the index is carried on that table, per `FR-CAT-010` |
  | index name | `PRIMARY`, and the names the DDL gave | `name` |
  | non-unique | `0` for the primary key and for a unique index, `1` otherwise — an integer, and there is **no** is-unique field | `unique`, true WHEN the field is `0` |
  | sequence in index | one-based, and **this is the column `FR-CAT-010` folds on** | the order of the column list |
  | column name | one row per column | the column list |
  | collation | `A` ascending (70 rows), `D` descending (6), SQL `NULL` for the one full-text row | the sort direction of that column |
  | cardinality | an estimate | not carried — `FR-CAT-024` |
  | sub-part | the prefix length in characters, SQL `NULL` for a whole-column index | the prefix length of that column |
  | packed | SQL `NULL` on all 77 rows | not carried — nothing observed, `BR-CAT-005` |
  | nullable | the **empty string** for a `NOT NULL` column and `YES` for a nullable one — not `NO`/`YES` | not carried — restates the column's own nullability, `FR-CTX-021` |
  | index type | `BTREE` (75), `FULLTEXT` (1), `SPATIAL` (1) | `type` |
  | comment | the **empty string** on all 77 rows; **not** the index comment | not carried — nothing observed, and its declared shape differs between series, difference 8 of `FR-SRV-038` |
  | index comment | the text written with `COMMENT` on the key, the empty string where none was given | `comment` |
  | ignored | `NO` on all 77 rows | `ignored`, true WHEN the field is not `NO` |

  *Closes* `OQ-032`, now listed under
  [Closed](open-questions.md#closed).

  *The two fields that are easy to swap.* The comment field and the
  index-comment field are **different fields**, and the text a DDL writes as
  `KEY … COMMENT '…'` lands in the second. A reader that takes the first gets
  the empty string on every index and reports every index as uncommented.

  *No expression index exists to report.* MariaDB has no functional-index
  syntax; an expression is indexed by declaring a generated column and
  indexing that, and such an index appears as an ordinary row naming the
  generated column. Three of the fixture's indexes are of that shape. The
  question `OQ-032` asked — whether an expression
  index is reported at all — therefore has no case to report.

  *The spatial row is the one that contradicts a plain reading of sub-part.*
  The fixture's spatial index declares no prefix and its row nonetheless reads
  `32`. A model that presents a prefix length as *the author asked for a
  prefix* is wrong for that row; the field is what the catalogue states about
  the index, not what the DDL said.

  *Bounded claim.* Two fields were observed with one value each. The ignored
  flag read `NO` on all 77 rows, because the fixture declares no ignored
  index; and the index type took three values and **no `HASH` row appeared**.
  Both are carried as the catalogue returns them, and no claim is made here
  about the rest of either population.

  *Identifiers are returned unescaped.* The fixture's hostile names —
  an index called `uq segment tag`, an index called ``idx`backtick``, columns
  called ``back`tick`` and `space in name` — appear in these fields unquoted
  and unescaped. Quoting them for a target dialect is the template's job, per
  `FR-ENV-045`.

- **FR-CAT-043**: The index catalogue table SHALL be the authoritative source
  of a table's primary key, and the primary key SHALL be the index it reports
  under the name `PRIMARY`. The system SHALL NOT read the primary key from the
  key-column-usage table and SHALL NOT read it from the constraint table.

  *Observed, and the three sources disagree.* On the fixture's
  system-versioned table `tariff`, identically on all four series:

  | Source | Reports the primary key of `tariff` as | Schema-wide count of primary-key columns |
  |---|---|---|
  | key column usage | `(tariff_id, row_end)` | 21 |
  | the index table | `(tariff_id)` | 20 |
  | the column table's key field | `tariff_id` | 20 |

  `row_end` is the implicit period column system versioning adds. The column
  table **carries no row for it, on any table**: the count of columns named
  `row_end` or `row_start` in the whole fixture is zero. So key column usage
  can name a column the model does not carry, and the index table cannot.

  *Rationale, and it is the property the decision turned on.* A primary key
  that names a column absent from the same table's column list is an
  internally inconsistent document, and `FR-CAT-044` makes the absence of that
  state an invariant rather than a hope. Two further properties confirm the
  choice rather than decide it: the index table reports the key **as the DDL
  wrote it**, and it survives a reduced-privilege reader that loses the
  constraint table entirely — 77 index rows against zero constraint rows, per
  `FR-PRIV-018` — so the primary key is readable by a reader for whom the
  constraint table returns nothing.

  *How the primary key is told from any other unique index*, which is the
  second half of `OQ-033`: by the index name
  `PRIMARY`. Over the fixture, the indexes so named are exactly the 17 the
  constraint table reports as primary keys, they carry the non-unique field
  `0` on every row, and their 20 columns are exactly the 20 the column table
  marks with its primary-key value. The three agree everywhere except on the
  implicit period column above.

  *Closes* `OQ-033`, now listed under
  [Closed](open-questions.md#closed).

  *The primary key is also an index.* The catalogue reports it as one, and the
  model does not drop a member of a population it presents: the index named
  `PRIMARY` appears in the table's index collection, and `FR-CAT-011` presents
  the same index additionally as the table's primary key so that a template
  need not find it by matching a name. One consequence reaches
  [template-environment.md](template-environment.md) and is stated so it is
  not discovered: a primary-key column satisfies the `unique` test of
  `FR-ENV-041`, because the index it belongs to reports the non-unique field
  as `0`. `primary_key` and `unique` are not disjoint and nothing requires
  them to be.

  *Rejected.* Reading key column usage and admitting the implicit period
  column into the model. It is faithful to the engine and it obliges the model
  to carry columns the catalogue deliberately hides, which changes what
  `tpl schema table` presents for every system-versioned table. Also rejected:
  reading key column usage and filtering out what the column table lacks —
  the same result by a longer route, and it makes the primary key depend on
  two reads where one suffices.

- **FR-CAT-044**: A key the model carries SHALL NOT name a column that is
  absent from the column list of the same table. This applies to the primary
  key, to a unique key, to an index, and to both directions of a foreign key.

  *Rationale.* It is the invariant `FR-CAT-043` exists to guarantee, stated
  once over every key so that a future source change cannot reintroduce the
  state through another door. `FR-CTX-023` requires every object referenced
  from another to be present in the document; this is the same property one
  level down, over column names within one table, and the observation shows
  the catalogue does not supply it for free.

### Foreign keys

- **FR-CAT-045**: The catalogue field lists for a foreign key SHALL be taken
  to be the following, across the two tables that carry them, and the model
  SHALL carry the properties named:

  The referential-constraint table, one row per foreign key:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema, unique-constraint catalogue and schema | `def` and the database name | not carried — row identity |
  | constraint name | the name the DDL gave | `name` |
  | unique-constraint name | `PRIMARY` on all fifteen | `referenced_key` — the key on the referenced table that the foreign key points at |
  | match option | `NONE` on all fifteen | `match_option` |
  | update rule, delete rule | one of the four spellings below | `on_update`, `on_delete` |
  | table name | the referencing table | the key is carried on that table, per `FR-CAT-012` |
  | referenced table name | the referenced table, **the table only** | `referenced_table` |

  The key-column-usage table, one row per column of every key:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema, table catalogue, table schema | `def` and the database name | not carried — row identity |
  | constraint name | joins the row to the referential rule above | not carried — restatement |
  | table name, column name | the referencing side | the ordered column list |
  | ordinal position | one-based, **the order of the referencing columns** | that order |
  | position in unique constraint | one-based on a foreign-key row, SQL `NULL` on a primary-key or unique-key row | the pairing of referencing to referenced column |
  | referenced table schema, referenced table name, referenced column name | populated on a foreign-key row, SQL `NULL` otherwise | `referenced_columns`, in the order above |

  *The four reachable rule spellings, and they are contract surface*:

  ```text
  CASCADE
  NO ACTION
  RESTRICT
  SET NULL
  ```

  Upper case, with a single space in `NO ACTION`. `SET DEFAULT` is never
  returned, per `FR-CAT-033`. The fixture declares fifteen foreign keys with
  fifteen distinct rule pairs as written and the catalogue returns fourteen,
  because the one declared `SET DEFAULT` on both rules comes back
  `RESTRICT` / `RESTRICT`.

  *How the incoming direction is obtained*, which is the third question
  `OQ-034` asked: by filtering the **same**
  key-column-usage table on the referenced table name. There is no second
  catalogue table for it and no field that states it; the incoming direction
  of `FR-CAT-013` is the outgoing rows of every other table, read from the
  other end.

  *The two tables must be joined, because neither is sufficient.* The
  referential-constraint table carries the rules and names no column; the
  key-column-usage table carries the columns and no rule. This is also why
  `FR-PRIV-019` can detect the privilege shortfall it detects: the two tables
  are lost independently.

  *Closes* `OQ-034`, with `FR-CAT-033`, now listed
  under [Closed](open-questions.md#closed), and with it
  `OQ-010`'s foreign-key fragment.

  *Bounded claim.* The match option read `NONE` on all fifteen rows, and it is
  carried as the catalogue returns it; no second value was observed. Every
  foreign key of the fixture also references a table in the
  same database, so the referenced schema field was observed to hold the
  database's own name on all fifteen rows and never anything else. What a
  cross-schema foreign key returns has not been observed.

  *Amended in the twenty-seventh edition: the bounded claim's last sentence
  said what the model does and named no requirement that said it.* The
  sentence read *What the model does with a foreign key that crosses schemas
  has not been observed*, which left a reader to conclude that the model does
  something and that nobody had looked. `FR-CAT-057` settles it: the model
  does not cover such a key, in either direction, and the referenced-schema
  field of the key-column table above is the field that detects the outgoing
  one. The claim that remains bounded is the catalogue's, not the model's, and
  the sentence now says which.

  *Two further bounds, added by `FR-CAT-056` and `FR-CAT-055`.* The
  unique-constraint name and the referenced table name of the rules table, and
  the referenced column name of the key-column table, are all **declared
  nullable** by the catalogue, and `FR-CAT-056` fixes what the model carries
  when one arrives absent. The four rule spellings above are closed by this
  observation and by nothing the server declares: the update-rule and
  delete-rule fields are `varchar(64)`, not an `ENUM`, and `FR-CAT-055` fixes
  what the model does with a fifth spelling.

- **FR-CAT-046**: The catalogue field list for a `CHECK` constraint SHALL be
  taken to be the following, and the level SHALL be taken to admit exactly two
  values:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema | `def` and the database name | not carried — row identity |
  | table name | the table the constraint belongs to | the constraint is carried on that table, per `FR-CAT-015` |
  | constraint name | the name the DDL gave, or the column name for a column-level constraint per `FR-CAT-037` | `name` |
  | level | `Table` (22 of 24) or `Column` (2) — **mixed case, and no third value** | `level` |
  | check clause | the clause with identifiers **backtick-quoted** and operators lower-cased: `is null`, `and` | `clause` |

  Six fields, on all four series. A DDL that wrote
  `un_number IS NULL OR (un_number >= 1 AND un_number <= 3550)` is returned as
  `` `un_number` is null or `un_number` >= 1 and `un_number` <= 3550 ``.

  *Closes* `OQ-035`, with `FR-CAT-037` and
  `FR-CAT-038`, now listed under [Closed](open-questions.md#closed).

  *Unlike every other constraint table, this one is readable by a reduced
  reader.* The fixture's reduced-grant reader sees all 24 rows here and zero
  rows in the constraint table beside it, so it knows every check constraint
  and cannot know which table-level constraints exist. That asymmetry is
  recorded in `FR-PRIV-018` and is why no cross-check is available here.

  *The two values are closed by this observation and by nothing the server
  declares.* The level field is `varchar(6)` and `NOT NULL`, not an `ENUM`, on
  all four series, observed 2026-09-20. `FR-CAT-055` fixes what the model does
  with a third value.

### Tables

- **FR-CAT-053**: Every property a table object carries SHALL be named in the
  index below, beside the requirement that fixes it. A property a later
  requirement gives a table SHALL be added to this index by that requirement,
  and a property not named here SHALL NOT be carried.

  | Property | Fixed by |
  |---|---|
  | `name` | this requirement. Every other named object of this section carries its name under `name` — `FR-CAT-042`, `FR-CAT-046`, `FR-CAT-047`, `FR-CAT-048`, `FR-CAT-050` — and `DIV-034` corrects the root `README.md`'s list of a table's fields in several places without ever contesting the `name` in it |
  | `table_type` | `FR-CAT-002`, whose two values are those `FR-CAT-031` marks covered |
  | its columns | `FR-CAT-009`, in the order `NFR-DET-002` fixes, with the coverage of `FR-CAT-052` applied to the column read |
  | its primary key | `FR-CAT-011`, read from the source `FR-CAT-043` names, under the invariant of `FR-CAT-044` |
  | its indexes | this requirement, with `FR-CAT-010` folding an index into one object and `FR-CAT-042` for its catalogue field list. No requirement of this file had said a table carries them; `FR-SCH-009` and `FR-CTX-007` said it from outside, and the index is where the model's own statement now sits |
  | `foreign_keys` | `FR-CAT-012`, whose catalogue field lists are `FR-CAT-045`, and which `FR-CAT-033` bars from reporting `SET DEFAULT` |
  | `referenced_by` | `FR-CAT-013`, shaped by `FR-CTX-010` |
  | its triggers | `FR-CAT-014`, whose catalogue field list is `FR-CAT-050` |
  | its `CHECK` constraints | `FR-CAT-015`, whose catalogue field list is `FR-CAT-046`, with `FR-CAT-037` and `FR-CAT-038` |
  | `engine` and `collation` | `FR-CAT-054`, which records the catalogue field, the declared type and the observed value of each, and which records that a table has a collation and **no** character set |
  | its comment | `FR-SCH-009`, which names it; the comment's absent value is fixed by `FR-CAT-039`, and `FR-CAT-040` bars the table comment of a view from becoming a view's comment |

  **An embedded table is a reduction of this object and not a second shape.**
  `FR-CTX-006` through `FR-CTX-010` fix what survives one hop: an embedded
  table carries its columns, its indexes and its primary key in full, per
  `FR-CTX-007`, and its `foreign_keys` hold names rather than objects, per
  `FR-CTX-008`. Nothing in that reduction adds a property, so this index
  covers the embedded table as well.

  **A table has no catalogue field list in this section, and that is an
  absence of evidence rather than a decision.** `FR-CAT-039` through
  `FR-CAT-051` were written from the observation pass of 2026-09-10, which
  recorded the field list of every object kind an entry of
  [open-questions.md](open-questions.md) had asked for. No entry asked for the
  row the table catalogue returns, so that pass did not record it and this
  corpus holds no reading of it. Fragments of it are recorded: `FR-CAT-024`
  names twelve of its fields as volatile and excludes them, `FR-CAT-031`
  records the table-type field on all four series, `FR-CAT-039` records the
  comment field's absent value, and `FR-CAT-054` records the engine and the
  collation fields. The list itself is not recorded, and writing one from
  MariaDB's documentation is what the fourth provenance of the
  [README](README.md#provenance) forbids.

  *Amended in the twenty-sixth edition: a fourth fragment joins the three.*
  Two properties this index names — the engine and the collation — were fixed
  by `FR-SCH-009` alone, which requires `tpl schema table` to print them and
  names neither the catalogue field either is read from nor what that field
  holds. That is the gap this index was written to make visible, and it blocked
  the command outright. `FR-CAT-054` closes it from the evidence, on all four
  series of `FR-SRV-015`. It records two fields and not the list, so the pass
  this note names is still untaken and nothing in this corpus waits on it.

  *What would change this.* An observation pass of the kind that produced
  `FR-CAT-047` and `FR-CAT-048`, recording the table catalogue's field list
  verbatim on all four series of `FR-SRV-015` against the fixture of
  `scripts/mariadb/`. It would add a field list beside this index and would
  settle from evidence whether the catalogue offers a table a field this index
  does not name — which is the one question the index cannot answer about
  itself. **Nothing in this corpus waits on it.** Every property a requirement
  in force gives a table is named above, so no requirement is ambiguous while
  it stands; it is not an open question and no entry is opened for it, and the
  index of [open-questions.md](open-questions.md) stays empty.

  *Rejected.* Writing the catalogue field list from the fields this corpus
  already names — the twelve of `FR-CAT-024`, the table type of `FR-CAT-031`,
  the comment of `FR-CAT-039` — filled out from what MariaDB documents beside
  them. It is exactly the shape the seventh edition's validation rule was
  written from: four requirements were internally coherent, cross-referenced
  correctly, and describing a catalogue that does not exist, and only reading
  them against the evidence found it. A list assembled from exclusions also
  runs `BR-CAT-005` backwards, which carries by default and excludes on a
  stated ground. Also rejected: leaving the table with no entry here and
  letting a reader assemble its properties from the requirements the index
  names. That is what this corpus did through twenty-two editions, and the
  first reader to write a worked example against the model found the gap — a
  reader who must collect a property list from scattered requirements has no
  way to know when the collection is complete.

- **FR-CAT-054**: A table SHALL carry `engine` and `collation`, read from the
  two catalogue fields below and carried exactly as the server returns them,
  per `FR-SRV-039`:

  | Catalogue field | Declared | Observed | Model |
  |---|---|---|---|
  | engine | `varchar(64)`, nullable | `InnoDB` on every covered table | `engine` |
  | table collation | `varchar(64)`, nullable | `utf8mb4_unicode_520_ci` on every covered table | `collation` |

  *Observed on 2026-09-18 against all four series of `FR-SRV-015`.* The
  `freight` schema returns 23 rows from the table catalogue — 16 `BASE TABLE`,
  one `SYSTEM VERSIONED`, one `SEQUENCE` and five `VIEW`, exactly the types
  `FR-CAT-031` records. The two fields hold the values above on all eighteen
  non-view rows, are SQL `NULL` together on all five view rows, and are never
  the empty string; the declared type of each is the same on every series, and
  no reading differs between the four. The seventeen rows the model covers are
  the `BASE TABLE` and `SYSTEM VERSIONED` ones, per `FR-CAT-001`.

  *Conditions of the observation.* Taken through the harness of
  `scripts/mariadb/`, raised by `up.sh` and taken down by `down.sh`, against
  the four servers it raised: server versions `12.3.3`, `11.8.9`, `11.4.13`
  and `10.11.19`. The fixture declares its schema `CHARACTER SET utf8mb4 COLLATE
  utf8mb4_unicode_520_ci` and every table `ENGINE=InnoDB` with no table-level
  `COLLATE`. A reading of these two fields is re-taken whenever that DDL
  changes either declaration.

  **A view takes neither field.** Both are SQL `NULL` on the row a view has in
  the table catalogue, on all four series. Nothing follows for the model —
  `FR-CAT-003` keeps a view out of the tables collection and `FR-CAT-047`
  gives it a field list of its own — and it is recorded because the same row
  is where `FR-CAT-040` reads the literal `VIEW` a view's table comment
  carries, so a reader of that requirement meets these two fields beside it.

  *A table has a collation and no character set*, which `FR-SCH-009` has
  recorded since the seventh edition. This observation does not disturb it:
  the table catalogue offers the collation field above and no character-set
  field. A character set is reachable on the database, per `FR-CTX-036`, and on
  each individual column, per `FR-CTX-041`, and not on the table between them.

  *An inherited collation is reported explicitly*, exactly as `FR-CTX-041`
  records at column level. No table of the fixture declares a `COLLATE` of its
  own and every one reports the schema's default explicitly, rather than SQL
  `NULL` or the empty string, so the model cannot say whether a table's
  collation was written on the table or inherited from the schema, and does
  not claim to.

  *Bounded claim.* The fixture declares one engine and one table collation, so
  no second value of either field was observed, and no claim is made here
  about the population either can take. Both are carried under `BR-CAT-005`,
  which carries a field holding the same **populated** value throughout and
  records the bound beside it rather than dropping the field — the treatment
  that rule states for a foreign key's match option and an index's ignored
  flag.

  *This is not the table catalogue's field list.* `FR-CAT-053` names the
  observation pass that would record that list verbatim, and the pass is still
  untaken. This requirement records two of its fields; it establishes nothing
  about whether the catalogue offers a table a field this corpus does not
  carry.

  *Rejected: carrying neither, and having `FR-SCH-009` obtain what it prints
  from somewhere else.* There is nowhere else. No statement this system issues
  reports a table's engine or its collation other than the table catalogue,
  and deriving the collation from the schema's default — which is what every
  table of the fixture happens to report — is an inference the catalogue does
  not state, and is the ground on which the seventh edition removed the
  character set rather than reconstructing it. Carrying is also what
  `BR-CAT-005` requires by default: neither field is on the closed exclusion
  list of `FR-CAT-024`, neither restates a fact the model holds elsewhere
  under `FR-CTX-021`, and neither is the *nothing observed* case, because both
  hold a populated value on every covered table of every series.

### Views

- **FR-CAT-047**: The catalogue field list for a view SHALL be taken to be
  eleven fields, and the model SHALL carry nine of them:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema | `def` and the database name | not carried — row identity |
  | table name | the view's name | `name` |
  | view definition | the server's **rewritten** form: fully qualified, backtick-quoted, and not the text of the `CREATE VIEW` | `definition`, per `FR-CAT-007` and `FR-SCH-006` |
  | check option | `NONE`, `LOCAL`, `CASCADED` — all three observed | `check_option` |
  | is updatable | `YES`, `NO` — both observed | `is_updatable` |
  | definer | `root@localhost` on all five | `definer` |
  | security type | `DEFINER`, `INVOKER` — both observed | `security_type` |
  | character set client | `utf8mb4` on all five and all four series | `character_set_client`, passed through per `FR-SRV-039` |
  | collation connection | **differs between series** — difference 1 of `FR-SRV-038` | `collation_connection`, passed through per `FR-SRV-039` |
  | algorithm | `UNDEFINED` on all five | `algorithm` |

  *There is no creation or alteration timestamp on a view*, and there is no
  comment field either — the row a view has in the table catalogue carries the
  literal `VIEW` in its comment, per `FR-CAT-040`.

  *Closes* `OQ-036`, now listed under
  [Closed](open-questions.md#closed). The sixth edition barred the
  collation-connection field from the model while
  `OQ-075` was open; that entry is settled and the
  field is admitted, its value passed through exactly as the server returns
  it, per `FR-SRV-039`.

  *Bounded claim.* The fixture declares no view with an explicit algorithm, so
  only `UNDEFINED` was observed in that field; and every view is owned by the
  same definer, so no second value of the definer field was observed either.
  Both fields are carried as the catalogue returns them, and neither claim
  about the population they can take is made here.

### Routines and their parameters

- **FR-CAT-048**: The catalogue field list for a routine SHALL be taken to be
  31 fields, and the model SHALL carry the properties named below:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema | `def` and the database name | not carried — row identity |
  | specific name | equal to the routine name on all seven | not carried — restatement, `FR-CTX-021` |
  | routine name | the routine's name | `name` |
  | routine type | `FUNCTION` (4) or `PROCEDURE` (3) | `kind`, carried unchanged, per `FR-CAT-016` |
  | data type, the five size fields, character set, collation, DTD identifier | for a **function**, the return type decomposed exactly as a column's type is; for a **procedure**, the data type is the **empty string**, the DTD identifier is SQL `NULL`, and the rest are SQL `NULL` | the return type, decomposed as `FR-CTX-015` decomposes a column's, and `null` in full for a procedure |
  | routine body | `SQL` on all seven | `body_kind` |
  | routine definition | the body **as written**, newlines and identifier case preserved, and **not** rewritten as a view definition is | `body`, per `FR-CAT-017` |
  | external name, external language, SQL path | SQL `NULL` on all seven | not carried — nothing observed, `BR-CAT-005` |
  | parameter style | `SQL` on all seven | `parameter_style` |
  | is deterministic | `YES`, `NO` — both observed | `is_deterministic` |
  | SQL data access | `READS SQL DATA` (3 routines), `NO SQL` (2), `MODIFIES SQL DATA` (2) | `sql_data_access` |
  | security type | `DEFINER` on all seven | `security_type` |
  | created, last altered | wall-clock times, differing between two containers of the **same** series | not carried — `FR-CAT-024` |
  | SQL mode | the session mode in force at creation | `sql_mode` |
  | routine comment | the comment, the empty string where none was given | `comment` |
  | definer | `root@localhost` on all seven | `definer` |
  | character set client, collation connection, database collation | `utf8mb4`; **collation connection differs between series**; database collation follows the schema and does **not** differ | carried, and passed through per `FR-SRV-039` |

  *The two absent-value shapes sit side by side and are different.* For a
  procedure the data-type field is the **empty string** while the DTD
  identifier beside it is SQL `NULL`. A reader that tests one shape finds the
  other absent value populated. The model emits the whole return type as
  `null` for a procedure, per `FR-CTX-005`.

  *A third absent-value shape reaches the body, and it is a privilege rather
  than a kind of routine.* The routine-definition field is **declared
  nullable**, `longtext`, on all four series, observed 2026-09-20. SQL `NULL`
  there is the missing privilege of `FR-PRIV-017`, and `FR-CAT-056` fixes what
  `body` carries when it arrives: `null`, and not the empty string, which is
  what a genuinely empty body would carry. The routine-type field beside it is
  `varchar(13)` and `NOT NULL`, not an `ENUM`, so the two kinds recorded above
  are closed by this observation alone and `FR-CAT-055` fixes what the model
  does with a third.

  *Closes* `OQ-037`, with `FR-PRIV-017`, now listed
  under [Closed](open-questions.md#closed), and with it
  `OQ-010`'s routine fragment.

  *Bounded claim, and one item of it is a discrepancy in the evidence rather
  than a gap in the fixture.* Four fields were observed with one value each
  over the seven routines — the body kind and the parameter style both read
  `SQL`, the security type reads `DEFINER`, and the definer reads one user —
  and three were never populated, so the external-name, external-language and
  SQL-path fields are excluded under `BR-CAT-005` rather than recorded. The
  discrepancy: the observation record's own summary names a **fourth** value
  of the SQL-data-access field, `CONTAINS SQL`, which appears on **no row** of
  the seven it also records. Three values are written above because three
  were returned; the fourth is not recorded as observed, and a later
  observation is what would add it.

- **FR-CAT-049**: The catalogue field list for a routine parameter SHALL be
  taken to be 16 fields on `10.11`, `11.4` and `11.8` and 17 on `12.3`, and
  the model SHALL carry the properties named below:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema | `def` and the database name | not carried — row identity |
  | specific name | the routine the parameter belongs to | the parameter is carried on that routine, per `FR-CAT-018` |
  | ordinal position | **one-based for a declared parameter, and `0` for a function's return** | the declaration order of `FR-CAT-018` |
  | parameter mode | `IN`, `OUT`, `INOUT` for a declared parameter; SQL `NULL` on the return row | `mode` |
  | parameter name | the name; SQL `NULL` on the return row | `name` |
  | data type, the five size fields, character set, collation, DTD identifier | the type, decomposed exactly as a column's is | the type, decomposed as `FR-CTX-015` decomposes a column's |
  | routine type | repeated on every parameter row | not carried — the routine already states its kind, `FR-CTX-021` |
  | parameter default (`12.3` only) | SQL `NULL` on all 22 rows | not carried — nothing observed, `BR-CAT-005` |

  **A function's return value is a row in this table and SHALL NOT be
  presented as a parameter.** It is the row at ordinal position `0`, with both
  the mode and the name SQL `NULL`; declared parameters start at `1`; a
  procedure has no row at `0`. The return type the model carries is the one
  `FR-CAT-048` reads from the routine row, and it is the same type this row
  reports.

  *Observed.* 22 rows for seven routines — 18 declared parameters and four
  function returns — identically on all four series but for the extra column
  on `12.3`. A function's parameters are declared with no mode keyword,
  because MariaDB accepts none on a function, and are reported as `IN`.

  *Closes* `OQ-038`, now listed under
  [Closed](open-questions.md#closed).

  *Why the extra column of `12.3` produces no row in the register of
  `FR-SRV-036`.* The narrowing recorded against this entry warned that a
  parameter default entering the model would be `null` on three of the four
  series under `FR-SRV-004` and would owe a row there. It does not enter: it
  was observed to be SQL `NULL` on every parameter of the fixture on the one
  series that has it, so nothing has been observed for the model to carry, per
  `BR-CAT-005`. `FR-SRV-037` still governs any statement that names a column
  list against this table.

### Triggers

- **FR-CAT-050**: The catalogue field list for a trigger SHALL be taken to be
  22 fields, and the model SHALL carry the properties named below:

  | Catalogue field | Observed | Model |
  |---|---|---|
  | catalogue, schema, event object catalogue, event object schema | `def` and the database name | not carried — row identity |
  | trigger name | the trigger's name | `name` |
  | event manipulation | `INSERT`, `UPDATE`, `DELETE` — all three observed | `event` |
  | event object table | the table the trigger is on | the trigger is carried on that table, per `FR-CAT-014` |
  | action order | `1` on all six | `action_order` |
  | action condition | SQL `NULL` on all six | not carried — nothing observed, `BR-CAT-005` |
  | action statement | the body **as written**, newlines and case preserved | `statement` |
  | action orientation | `ROW` on all six | `orientation` |
  | action timing | `BEFORE`, `AFTER` — both observed | `timing` |
  | action reference old table, action reference new table | SQL `NULL` on all six | not carried — nothing observed, `BR-CAT-005` |
  | action reference old row, action reference new row | the literals `OLD` and `NEW` on **all six rows, whatever the event** | `old_row_alias`, `new_row_alias` |
  | created | a wall-clock time with two fractional digits, differing between two containers of the **same** series | not carried — `FR-CAT-024` |
  | SQL mode | the session mode in force at creation | `sql_mode` |
  | definer | `root@localhost` on all six | `definer` |
  | character set client, collation connection, database collation | as for a routine, per `FR-CAT-048` | carried, and passed through per `FR-SRV-039` |

  All six combinations of the three events and the two timings appear in the
  fixture, and the field pair that names them is the only place the model
  learns either.

  **The two row-alias fields say nothing about availability**, and the model
  SHALL NOT be read as claiming they do. They hold `OLD` and `NEW` on every
  row, including the `INSERT` triggers where `OLD` does not exist and the
  `DELETE` triggers where `NEW` does not. What is available follows from the
  event, and a template that branches on these fields branches on a constant.
  They are carried because they are what the catalogue states, and the limit
  is recorded here for the reason `FR-CAT-036` records its own: a value that
  is present and misleading is worse than one that is absent.

  *There is no trigger comment field.*

  *Bounded claim.* Over the six triggers the action order read `1`, the
  orientation read `ROW`, and the definer read one user; no second value of
  any of the three was observed, and each is carried as the catalogue returns
  it. All six event-and-timing combinations **were** exercised, so that field
  pair is not bounded.

  *Amended in the twenty-seventh edition: three further bounds, and none
  changes what a supported server produces.* The event and the timing are
  closed by the observation above and by nothing the server declares — the
  event field is `varchar(6)` on `10.11`, `11.4` and `11.8` and `varchar(20)`
  on `12.3`, which is difference 9 of `FR-SRV-038`, and the timing field is
  `varchar(6)` on all four; neither is an `ENUM`. `FR-CAT-055` fixes what the
  model does with a value outside either set. The definer and the action
  statement are both **declared nullable**, and `FR-CAT-056` fixes what the
  model carries when one arrives absent; no row of the fixture returned one.
  Observed 2026-09-20, against all four series.

  *Closes* `OQ-039`, now listed under
  [Closed](open-questions.md#closed), and with it
  `OQ-010`'s trigger fragment. With
  `FR-CAT-045`, `FR-CAT-048` and `FR-CAT-051` it closes
  `OQ-010` itself, which the second edition split
  into exactly those four and which has no query of its own.

## Enumerated exclusions

The following features are outside the model. Each exclusion is a requirement in
its own right, and that requirement's identifier is the exclusion identifier by
which it is cited. An exclusion is a decision, not an omission: a request to add
one of these is a change to this file, not a defect report.

- **FR-CAT-019**: The model SHALL NOT cover scheduled events.

- **FR-CAT-020**: The model SHALL NOT cover sequences.

- **FR-CAT-021**: The model SHALL NOT cover table partitions.

- **FR-CAT-022**: The model SHALL NOT cover application-time periods.

- **FR-CAT-023**: The model SHALL NOT cover spatial reference identifiers.

- **FR-CAT-057**: The model SHALL NOT cover a foreign key that crosses a
  schema boundary, in either direction, and the system SHALL NOT read a second
  schema in order to cover one.

  | The key | What the catalogue shows the read | What the model does |
  |---|---|---|
  | Declared in the database the read covers, referencing a table in another schema | the key-column rows and the referential rule are both returned, with the referenced-schema field naming the other schema | the key is **not carried** on the referencing table |
  | Declared in another schema, referencing a table in the database the read covers | nothing: the read is filtered on the schema the key is declared in, per `FR-CAT-052` | the key is **not carried** under `referenced_by`, and the read never sees it |

  The referenced-schema field the first row turns on is the one `FR-CAT-045`
  records on the key-column table. The database a read covers is the one the
  selected entry names, per `FR-CONF-041`.

  *Rationale.* `FR-CTX-006` and `FR-CTX-010` embed the table at **each** end of
  every foreign key, in full, and `FR-CTX-023` requires every object a document
  references to be present in it. A table in another schema is in no model this
  read builds, so a carried cross-schema key is a reference with nothing at the
  other end — the one condition those three requirements between them forbid.
  The exclusion is therefore what keeps the embedding materialisable, and it is
  stated here rather than discovered at the moment a document is built.

  *Rejected: reading the referenced schema so that the key can be embedded.*
  One invocation would present objects of a database the selected entry does
  not name, against `FR-CONF-041`; the completeness of `FR-PRIV-001` would be
  claimed over a population nothing in the configuration bounds; and the read
  would follow the reference graph wherever it led, which is the traversal of
  unbounded depth `BR-CTX-001` refused at one hop.

  *Rejected: carrying the key with the referenced table as a bare name and no
  embedding.* It makes the document's depth depend on which side of a schema
  boundary a table happens to sit, so a template author could no longer know,
  without inspecting the database, how deep the object in hand goes — which is
  the depth that adapts to the graph that `BR-CTX-001` chose one constant depth
  over.

  *Rejected: refusing the read.* A database would become unreadable on account
  of a key naming something `tpl` was never asked for, which is a larger loss
  than the one relation the exclusion drops, and no exit code of `FR-ERR-001`
  describes it.

  *Accepted cost, stated because it is silent.* A table that references, or is
  referenced from, another schema is presented as having one relation fewer
  than the server holds, at exit `0`, and nothing in the document says so.
  `FR-PRIV-002` is not engaged: the property was read and excluded, not
  withheld. This is an exclusion and therefore a decision, in the terms this
  section opens with, and admitting the feature is a change to this
  requirement.

  **The behaviour is excluded rather than recorded, because it could not be
  observed.** The fixture of `scripts/mariadb/` declares one user schema,
  `freight`, and carries no cross-schema foreign key. Observed on 2026-09-20
  against all four series of `FR-SRV-015`, through the harness of that
  directory: the schema catalogue returns `freight` beside the four the server
  ships; the key-column table returns **17** foreign-key rows across the whole
  server, every one of them with the referencing and the referenced schema both
  `freight`; and the referential-constraint table returns **15** rows across
  the whole server, none of them with the constraint schema differing from the
  unique-constraint schema. Identically on all four series. What a cross-schema
  key returns is therefore unknown to this corpus, and this requirement makes
  no claim about it: it names a feature the model does not cover, which
  `BR-CAT-001` and the five exclusions above it establish needs no observation.

  *What would change this.* A second schema in the fixture's DDL — one table in
  it referenced by a table of `freight`, and one table in it referencing a
  table of `freight` — which would show, on each series, what the
  referenced-schema field holds on an outgoing cross-schema key, whether the
  referential rule for it is returned at all, and whether the unique-constraint
  name of `FR-CAT-056` is populated when the referenced table is not the
  reader's own schema. Until that exists, admitting the feature would mean
  writing down behaviour nobody has seen.

- **BR-CAT-001**: The covered set was chosen over a narrower classic core —
  tables, columns, indexes, keys, views, routines — because `CHECK` constraints,
  routine parameters, and the incoming side of a foreign key are all information
  a code generator needs and none of them can be recovered from what the
  narrower set holds. The five whole-feature exclusions above — `FR-CAT-019`
  through `FR-CAT-023` — are features whose absence a generator can notice and
  work around; the three additions are not.

  *Amended in the twenty-seventh edition: the five are named, and the sixth
  exclusion beside them is placed.* The sentence counted the exclusions above
  it, and `FR-CAT-057` joined them, so the count had to be either corrected or
  pinned to the requirements it was written over. It is pinned, because the two
  kinds are not alike: those five remove a whole feature, and `FR-CAT-057`
  removes one case of a feature this rule counts among the **additions** — the
  incoming side of a foreign key. That narrowing is real and is stated in its
  own *Accepted cost*: the incoming side remains reachable for every key of the
  database a read covers, which is every key the model has ever been observed
  to carry, and it is not reachable across a schema boundary.

## Volatile fields

- **FR-CAT-024**: The model SHALL NOT carry any of the following catalogue
  fields, anywhere, under any name:

  | Field | Where the catalogue reports it |
  |---|---|
  | `TABLE_ROWS` | table |
  | `AVG_ROW_LENGTH` | table |
  | `DATA_LENGTH` | table |
  | `MAX_DATA_LENGTH` | table |
  | `INDEX_LENGTH` | table |
  | `DATA_FREE` | table |
  | `AUTO_INCREMENT` | table — the counter, not the column attribute |
  | `CREATE_TIME` | table |
  | `UPDATE_TIME` | table |
  | `CHECK_TIME` | table |
  | `CHECKSUM` | table |
  | `VERSION` | table |
  | `CARDINALITY` | index |
  | `CREATED` | routine |
  | `LAST_ALTERED` | routine |
  | `CREATED` | trigger |

  *Amended in the seventh edition: three rows added, and the third is the one
  that had to be added.* The observation of 2026-09-10 found that a routine's
  creation and alteration timestamps and a trigger's creation timestamp are
  **wall-clock times recorded when the object was installed**, and that they
  differ between two containers built from the same DDL — including two of
  the *same* series. Carrying any of the three would make `FR-SRV-026`
  unsatisfiable outright: the document could never be byte-identical across
  four servers, because no two servers ran their DDL at the same instant. The
  three are the routine and trigger analogues of the table timestamps already
  on the list, and they are excluded on the same ground rather than on a new
  one.

  *Observed, and it is why `CARDINALITY` stays on this list rather than
  becoming a passed-through field under `FR-SRV-039`.* The four series return
  **different cardinalities for the same index over the same rows** — the
  fixture's `audit_event.idx_audit_observed_desc` reads `12` on `10.11` and
  `6` on the other three — while the true row counts and the table row
  estimates agree on all four, so the data is identical and the estimate is
  not. Repeating the query against one container returned byte-identical
  output, so the value is stable within a server and varies between them.
  That is superficially the shape `FR-SRV-039` covers, and it is not the
  same thing: `FR-SRV-039` passes a value through because the value is a
  **fact about the object** that the server records faithfully, and a
  cardinality is an **estimate the server computes** and revises without the
  structure changing. Passing an estimate through would make two reads of one
  unchanged database differ, which is the whole ground of `BR-CAT-002`. It is
  excluded, it stays excluded, and the cross-series evidence strengthens the
  exclusion rather than reopening it. Recorded as difference 10 of
  `FR-SRV-038`.

- **FR-CAT-025**: The list of `FR-CAT-024` SHALL be closed. A field is excluded
  by appearing in it and by nothing else, and adding a field to the model
  requires establishing that it is not volatile.

- **FR-CAT-026**: The exclusion SHALL apply to every consumer without exception:
  the model, `text` output, `json` output, the dump document, the cache, and the
  render context.

- **FR-CAT-027**: The fact that a column is auto-incremental SHALL remain in the
  model. It is a static column attribute, and only the table-level counter is
  excluded by `FR-CAT-024`.

- **BR-CAT-002**: A volatile field is one the server changes without any change
  to the structure — a row estimate the storage engine revises on its own, a
  length that moves as data is written, a timestamp that moves when the object is
  touched. Carrying one would put `NFR-DET-001` in permanent conflict with the
  server, and would make a committed dump produce a diff on every regeneration,
  which is the argument `FR-SCH-018` already used to keep `now` out of the dump.

  *Amended in the seventh edition.* The third example read "when the **table**
  is touched", which was true of the thirteen fields the list then held and
  false of the three the seventh edition adds: a routine and a trigger carry
  the same kind of timestamp and are not tables. The ground is unchanged and
  the word is now the general one.

  *Rejected.* Excluding these fields from the dump while keeping them in `text`
  listings, which would make a live read and a `--context` read produce
  different output for the same template and the same state. Also rejected:
  keeping them everywhere and narrowing the determinism requirement to
  accommodate them.

  *Accepted cost.* `tpl` cannot answer "how big is this table". It is a
  structure reader, and a size estimate is not structure.

## Fields excluded because their meaning differs between series

`FR-SRV-025` excludes a catalogue field whose **meaning** differs between two
series of `FR-SRV-015` and cannot be normalised to one. That is a second
ground of exclusion, distinct from the volatile-field ground above, and it
needs a list of its own for the same reason the first one does: an exclusion
that is not written down is an omission.

- **FR-CAT-029**: The model SHALL NOT carry any of the following catalogue
  fields, anywhere, under any name:

  | Field | Where the catalogue reports it | Series that disagree | Registered as |
  |---|---|---|---|
  | *(none excluded)* | | | |

  **The list is empty, and this is an observed result.** The four series were
  observed against the fixture of `scripts/mariadb/` on 2026-09-10 and again
  on 2026-09-11, and no field of ambiguous meaning was found; the
  **fourteen** differences that were found are recorded under `FR-SRV-038` and
  not one of them is a difference of meaning. The claim is bounded by what the
  fixture exercises, which `FR-SRV-038` states. No entry may be written from a
  changelog, from a release note, or from knowledge of MySQL.

  *Amended in the seventh edition: the count only.* The second observation
  pass took the recorded differences from seven to eleven. Three of the four
  new ones are differences of declared shape or of an estimate and one is a
  difference of value, so the list is still empty for the reason it was
  before — none of the eleven is a field whose **meaning** two series
  disagree about. A difference of value is now treated by `FR-SRV-039`
  instead.

  *Amended in the tenth edition: the count only.* A third observation pass
  added a twelfth difference, recorded under `FR-SRV-038`. It is the presence
  of a session variable and not a catalogue field at all, so it cannot be a
  field of ambiguous meaning and the list stays empty for the reason it has
  always been empty.

  *Amended in the twelfth edition: the count only.* A fourth observation pass
  added a thirteenth difference, recorded under `FR-SRV-038`. It is the version
  a server announces when the connection opens — not a catalogue field, and not
  a reading any requirement of this corpus takes — so it cannot be a field
  whose meaning two series disagree about, and the list stays empty for the
  reason it has always been empty.

  *Amended in the thirteenth edition: the count only.* A fourteenth difference
  was brought into the record of `FR-SRV-038` from the note in which it had
  been written down. It is the refusal of a `VECTOR` column or index by two of
  the four series at DDL time — a difference in what a server accepts, not in
  what its catalogue reports, so there is no field for two series to disagree
  about, and the list stays empty for the reason it has always been empty.

  *Checked in the eighteenth edition, and the count does not move.* That
  edition records three readings of the **build** that differ across the window
  — the distribution each image was built on, the build's source revision and
  the SSL library string — below the table of `FR-SRV-038` rather than in it,
  because none of them is established as a difference between the series. No
  row enters the table, so the count above stands at fourteen; and none of the
  three is a catalogue field at all, so none could be a field whose meaning two
  series disagree about however it were classified.

  The last column names the row of the divergence register of `FR-SRV-036` that
  records the observation. An entry here without a row there is a defect: the
  exclusion is normative here and the evidence for it lives there.

- **FR-CAT-030**: The list of `FR-CAT-029` SHALL be closed. A field is excluded
  on this ground by appearing in it and by nothing else, and adding such a
  field to the model requires establishing that the series of `FR-SRV-015`
  agree about what it means.

  A field SHALL leave this list WHEN the disagreement ends — because the series
  that disagreed has left the window of `FR-SRV-001`, or because the
  disagreement was resolved upstream — and the removal SHALL be a change to
  this requirement's list rather than an inference from the window having
  moved.

- **BR-CAT-004**: The two exclusion lists are separate because their grounds,
  their tests, and their futures are different. A field is on the list of
  `FR-CAT-024` because the server changes it without the structure changing;
  the test is whether two reads of an unchanged database differ, it can be
  applied to one server, and the answer never becomes yes. A field is on the
  list of `FR-CAT-029` because two supported series disagree about what it
  means; the test needs all four series at once, and the answer changes as the
  window of `FR-SRV-001` moves. Merging them would put an exclusion that is
  permanent beside one that expires, under one closing rule that could only be
  right for one of them.

  *Rejected.* Recording an ambiguous-meaning exclusion in the volatile list of
  `FR-CAT-024`, which is what `FR-SRV-025` did before the fifth edition — and
  which pointed at `FR-CAT-025`, the rule that closes that list rather than the
  list itself, so the reference did not even resolve to a place a field could
  be written.

## Coverage and filtering

- **FR-CAT-028**: The system SHALL apply coverage before `--pattern`. An object
  excluded by this file is not a candidate for the pattern, and a pattern never
  reintroduces one.

- **FR-CAT-052**: The coverage of `FR-CAT-001` through `FR-CAT-006` SHALL be
  applied to the column read as well as to the object read. The system SHALL
  NOT present a column whose owning object is not covered.

  *Observed.* `INFORMATION_SCHEMA.COLUMNS` is not restricted to the objects
  the model covers. In the fixture it carries **eight rows for the sequence**
  — `next_not_cached_value`, `minimum_value`, `maximum_value`, `start_value`,
  `increment`, `cache_size`, `cycle_option`, `cycle_count` — and a full set of
  rows for every view, and its 301 rows include both. Three of the fixture's
  105 distinct column-type combinations exist only because of the sequence.

  *Rationale.* `FR-CAT-004` excludes sequences and `FR-CAT-020` excludes them
  again as a feature, but both are written over the object. A reader that
  takes the object list from one catalogue table and the columns from another
  without applying the same filter to the second presents eight columns
  belonging to an object the model says it does not cover — attached to
  nothing, since the sequence itself is filtered out, or worse attached to a
  table by a careless join. The same holds for a temporary table on the three
  series that report one, per `FR-CAT-032`.

  *A view's columns are a different case and are not excluded.* A view is
  covered by `FR-CAT-007`, and its rows in the column table carry their own
  types, character sets and collations, derived by the server: the fixture's
  `v_consignment_manifest.charged_total` reads `decimal(36,2)`, a precision
  that appears nowhere in the DDL.

- **BR-CAT-003**: The order matters because it is what makes a count
  explainable. A caller comparing the number of tables it sees against the
  number the server holds must be able to attribute every difference to either
  coverage or the pattern, and never to the interaction between them.

## Dependencies

- [context-document.md](context-document.md) — how the covered material is
  shaped as a document.
- [schema-commands.md](schema-commands.md) — the commands that present it, and
  the `--pattern` filter of `FR-SCH-011` through `FR-SCH-015`.
- [server-contract.md](server-contract.md) — the supported version window of
  `FR-SRV-001`, the fields emitted as `null` where the connected series does not
  provide them, `FR-SRV-025`, which writes a field of ambiguous meaning into
  the closed exclusion of `FR-CAT-029`, and `FR-SRV-036`, the register in which
  the observation behind each such exclusion is recorded.
- [privileges-and-completeness.md](privileges-and-completeness.md) — what
  happens when the covered material cannot be read in full, and `FR-PRIV-018`,
  the three shapes that shortfall takes in the catalogue.

## Open questions

**None.** The ten entries this file carried were catalogue field lists, and
the observation pass of 2026-09-10 recorded every one of them. Each is now
closed by a requirement in this file and is listed under
[Closed](open-questions.md#closed):

| Entry | Closed by |
|---|---|
| [OQ-009](open-questions.md#closed) | `FR-CAT-039`, with `FR-CAT-036` and `FR-CTX-037` |
| [OQ-010](open-questions.md#closed) | `FR-CAT-045`, `FR-CAT-048`, `FR-CAT-050`, `FR-CAT-051` — the four entries it was the umbrella over |
| [OQ-026](open-questions.md#closed) | `FR-CAT-041`, with `FR-CAT-035` |
| [OQ-032](open-questions.md#closed) | `FR-CAT-042` |
| [OQ-033](open-questions.md#closed) | `FR-CAT-043`, with `FR-CAT-011` as amended and `FR-CAT-044` |
| [OQ-034](open-questions.md#closed) | `FR-CAT-045`, with `FR-CAT-033` |
| [OQ-035](open-questions.md#closed) | `FR-CAT-046`, with `FR-CAT-037` and `FR-CAT-038` |
| [OQ-036](open-questions.md#closed) | `FR-CAT-047` |
| [OQ-037](open-questions.md#closed) | `FR-CAT-048`, with `FR-PRIV-017` |
| [OQ-038](open-questions.md#closed) | `FR-CAT-049` |
| [OQ-039](open-questions.md#closed) | `FR-CAT-050` |
| [OQ-040](open-questions.md#closed) | `FR-CAT-051` |

`OQ-025` is answered by `FR-CAT-031` and `FR-CAT-032`, and `OQ-045` by
`FR-SRV-038`; both were closed in the sixth edition.
