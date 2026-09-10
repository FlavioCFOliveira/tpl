---
title: Catalogue Coverage
status: draft
last-reviewed: 2026-09-10
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

In scope: the covered object kinds, the properties of each, the closed exclusion
of volatile catalogue fields, the enumerated exclusions of whole features, and
the position of coverage filtering relative to `--pattern`.

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
  catalogue reports them.

- **FR-CAT-007**: The model SHALL cover views, each with its SQL definition, per
  `FR-SCH-006`.

- **FR-CAT-008**: The model SHALL cover routines — stored procedures and stored
  functions together — each stating its kind, per `FR-SCH-007`.

## What a table carries

- **FR-CAT-009**: A table SHALL carry its columns, including generated columns
  and invisible columns, in ordinal position order per `NFR-DET-002`.

- **FR-CAT-010**: An index SHALL be one object carrying its columns in the order
  the catalogue states, folded from the one-row-per-column form in which the
  catalogue reports it.

  *Rationale.* An index is one thing with an ordered column list. Presenting it
  as several rows would make every template perform the same grouping, and each
  one would order the columns differently.

- **FR-CAT-011**: A table SHALL carry its primary key.

- **FR-CAT-012**: A table SHALL carry its outgoing foreign keys, each with its
  `ON UPDATE` and `ON DELETE` rule.

- **FR-CAT-013**: A table SHALL carry its incoming foreign keys, under
  `referenced_by`.

  *Rationale.* Without the incoming direction, the has-many side of a relation
  is invisible to a template, and a generator can emit the belongs-to accessor
  but not its counterpart. The catalogue holds both directions; presenting only
  one would be a choice, not an economy.

- **FR-CAT-014**: A table SHALL carry its triggers.

- **FR-CAT-015**: A table SHALL carry its `CHECK` constraints, each with its
  name, its level, and its clause.

## What a routine carries

- **FR-CAT-016**: A routine SHALL state its kind: procedure or function.

- **FR-CAT-017**: A routine SHALL carry its body.

- **FR-CAT-018**: A routine SHALL carry its parameters in declaration order.

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

- **BR-CAT-001**: The covered set was chosen over a narrower classic core —
  tables, columns, indexes, keys, views, routines — because `CHECK` constraints,
  routine parameters, and the incoming side of a foreign key are all information
  a code generator needs and none of them can be recovered from what the
  narrower set holds. The five exclusions above are features whose absence a
  generator can notice and work around; the three additions are not.

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
  length that moves as data is written, a timestamp that moves when the table is
  touched. Carrying one would put `NFR-DET-001` in permanent conflict with the
  server, and would make a committed dump produce a diff on every regeneration,
  which is the argument `FR-SCH-018` already used to keep `now` out of the dump.

  *Rejected.* Excluding these fields from the dump while keeping them in `text`
  listings, which would make a live read and a `--context` read produce
  different output for the same template and the same state. Also rejected:
  keeping them everywhere and narrowing the determinism requirement to
  accommodate them.

  *Accepted cost.* `tpl` cannot answer "how big is this table". It is a
  structure reader, and a size estimate is not structure.

## Coverage and filtering

- **FR-CAT-028**: The system SHALL apply coverage before `--pattern`. An object
  excluded by this file is not a candidate for the pattern, and a pattern never
  reintroduces one.

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
  provide them, and `FR-SRV-025`, which writes a field of ambiguous meaning into
  the closed exclusion of `FR-CAT-025`.
- [privileges-and-completeness.md](privileges-and-completeness.md) — what
  happens when the covered material cannot be read in full.

## Open questions

- [OQ-025](open-questions.md#oq-025) — the table types a supported server
  actually reports, and whether temporary tables appear at all.
- [OQ-026](open-questions.md#oq-026) — the column attribute strings that carry
  auto-increment, generated, and invisible status.
- [OQ-032](open-questions.md#oq-032) — the index fields available, and the exact
  ordering column the folding of `FR-CAT-010` uses.
- [OQ-033](open-questions.md#oq-033) — how the primary key is reported, and
  whether it is distinguishable from any other unique index.
- [OQ-034](open-questions.md#oq-034) — the foreign-key fields and rule
  spellings, in both directions.
- [OQ-035](open-questions.md#oq-035) — the `CHECK` constraint fields, including
  the values the level takes.
- [OQ-036](open-questions.md#oq-036) — the view fields beyond the definition.
- [OQ-037](open-questions.md#oq-037) — the routine fields, and whether the body
  is readable without additional privilege.
- [OQ-038](open-questions.md#oq-038) — the routine parameter fields and their
  ordering, including how a function's return type is reported.
- [OQ-039](open-questions.md#oq-039) — the trigger fields.
- [OQ-040](open-questions.md#oq-040) — how a generated column's expression is
  reported, and how a virtual column is distinguished from a stored one.
