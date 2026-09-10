---
title: Privileges and Completeness
status: approved
last-reviewed: 2026-09-10
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
and the cross-check that distinguishes a missing privilege from an absence.

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
  {"name":"orders","restricted":["triggers"]}
  ```

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

- **FR-PRIV-010**: The system SHALL NOT report an absence where the true cause
  is a missing privilege.

- **FR-PRIV-011**: IF the catalogue reports rows of table type `VIEW` while the
  view collection comes back empty, THEN the system SHALL treat that as a
  missing privilege and SHALL exit `77`.

  *Rationale.* The two observations are contradictory and only one explanation
  fits: the reader can see that views exist but cannot read them. Reporting
  "this database has no views" would be a wrong answer wearing the appearance of
  a right one, and a generator acting on it would silently emit nothing.

- **FR-PRIV-012**: The cross-check of `FR-PRIV-011` SHALL be performed on every
  read that presents the view collection, including a dump.

- **FR-PRIV-015**: The cross-check of `FR-PRIV-011` SHALL be performed for
  views and for no other object kind.

  *Rationale.* Views are the only kind for which the catalogue offers two
  independent counts to compare: the rows of table type `VIEW` among the tables,
  and the members of the view collection. A cross-check needs two observations
  that can disagree, and neither tables nor routines offer a second one — a
  reader who cannot see a table does not see it counted somewhere else either.
  The general principle remains `FR-PRIV-010`; this is the one place the
  catalogue makes it enforceable.

  *Known gap.* Whether the check should instead be generalised — to any object
  kind for which the catalogue offers two views of the same population — cannot
  be settled until [OQ-041](open-questions.md#oq-041) is observed against the
  container, because it depends on what a reader without the privilege actually
  receives. The audit of 2026-09-10 left the choice open deliberately and wrote
  the reason for the present asymmetry rather than the decision.

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
- [cache-commands.md](cache-commands.md) — `FR-CACHE-037`, which keeps a marked
  object out of the cache.
- [cfg-commands.md](cfg-commands.md) — `FR-CFG-044`, whose probe reports
  whether the catalogue is readable at all, and does not report completeness.

## Open questions

- [OQ-041](open-questions.md#oq-041) — how an unreadable view is reported, and
  whether the cross-check of `FR-PRIV-011` is observable as stated.

`OQ-047` is answered by `FR-PRIV-016` and `OQ-048` by `FR-CACHE-037`; both are
listed under [Closed](open-questions.md#closed).
