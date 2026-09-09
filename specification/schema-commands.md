---
title: Schema Commands (First Arm)
status: draft
last-reviewed: 2026-09-09
related: [cli-contract.md, cache-commands.md, output-formats.md, render-command.md]
---

# Schema Commands (First Arm)

## Overview

The first arm reads the structure of a database and presents it. It is the only
arm that talks to a server on its own account, and every one of its subcommands
reads through the catalogue cache.

## Scope

In scope: the eight subcommands, their arguments and flags, the `--pattern`
filter, the shape of the dump document, and the ordering and format of the
result.

Out of scope: the content of the catalogue itself — which fields a table, view,
or routine carries, how they are read, and how they map onto the render context.
Where a field list is named below it is named as a scope statement, not as a
data model.

## Actors

- **Calling agent** or **operator**, invoking the command.
- **Database entry**, selected by `-d/--database`, naming the server and the
  database to read.

## Command surface

```
tpl schema info                        Database metadata
tpl schema tables    [--pattern <p>]   List tables
tpl schema table     <name>            Full table metadata
tpl schema views     [--pattern <p>]   List views
tpl schema view      <name>            Full view metadata, including its SQL definition
tpl schema routines  [--pattern <p>]   List procedures and functions
tpl schema routine   <name>            Full routine metadata
tpl schema dump                        The whole database as one JSON document
```

- **FR-SCH-001**: `tpl schema` SHALL be a group node, per `FR-CLI-007`.

- **FR-SCH-002**: The system SHALL provide exactly the eight subcommands listed
  above, with the aliases declared in `FR-CLI-011`.

- **FR-SCH-003**: `tpl schema info` SHALL report the metadata of the selected
  database.

- **FR-SCH-004**: `tpl schema tables`, `tpl schema views`, and
  `tpl schema routines` SHALL list the objects of their kind in the selected
  database.

- **FR-SCH-005**: `tpl schema table <name>`, `tpl schema view <name>`, and
  `tpl schema routine <name>` SHALL each take exactly one positional argument
  naming the object.

  *Rationale.* The object is named positionally here because the subcommand
  already carries its type. This is a deliberate asymmetry with `render` and
  `cache`, where the type must come from the flag.

- **FR-SCH-006**: `tpl schema view <name>` SHALL include the SQL definition of
  the view in its result.

  *Provenance.* Root `README.md`; not contradicted by any decision.

- **FR-SCH-007**: `tpl schema routines` SHALL return stored procedures and
  stored functions together, and SHALL state the kind of each object — as a
  column in `text` output and as a field in `json` output.

- **FR-SCH-008**: The system SHALL NOT provide a `--type` flag on
  `tpl schema routines`. Selecting one kind from a listing is done downstream,
  from `--format json`. Wherever a command names one routine — `tpl schema
  routine`, `tpl render --routine`, `tpl cache load --routine`, and
  `tpl cache clean --routine` — the system SHALL accept the qualified forms
  `procedure:<name>` and `function:<name>` as well as the bare name.

  *Amended in the second edition.* The prohibition was written for the listing
  and is unchanged by the amendment. The addition is the disambiguator the
  singular forms lacked: procedures and functions occupy distinct namespaces on
  the server, so `calc_vat` can legally name two objects, and every command that
  names one routine was ambiguous. The cached form of the same rule is
  `FR-CDOC-014`.

  *Rejected.* A `--kind` flag to be supplied only when a name is ambiguous,
  which is easy to forget until the day an ambiguity appears; and resolving a
  bare ambiguous name in favour of the function with a warning on stderr, which
  exits `0` — so a caller checking the code never sees it — and leaves the
  procedure unreachable.

- **FR-SCH-009**: `tpl schema table <name>` SHALL be exhaustive over what the
  catalogue holds for that table: its columns with position, type, nullability,
  default, comment, and generated-column status; its primary key, indexes, and
  foreign keys with their `ON UPDATE` and `ON DELETE` rules; its triggers; and
  its engine, character set, collation, and comment.

- **FR-SCH-010**: IF a named table, view, or routine does not exist in the
  selected database, THEN the system SHALL exit `66` (`EX_NOINPUT`) with a
  nearest-match suggestion over the objects of that kind that do exist. IF a
  bare routine name matches both a procedure and a function, THEN the system
  SHALL exit `64` (`EX_USAGE`), naming both candidates in the qualified form of
  `FR-SCH-008`.

  *Amended in the second edition.* The ambiguity case is new. The system SHALL
  NOT resolve it in favour of either kind under any circumstance: a first-wins
  rule would make one of the two objects permanently unreachable through a bare
  name, and which one it was would depend on the order the catalogue returned
  them.

## The `--pattern` filter

- **FR-SCH-011**: `tpl schema tables`, `tpl schema views`, and
  `tpl schema routines` SHALL each declare an optional `--pattern <p>` flag that
  filters the listing by object name.

- **FR-SCH-012**: `--pattern` SHALL use MariaDB `LIKE` syntax: `%` matches any
  sequence of characters including the empty sequence, `_` matches exactly one
  character, `\%` matches a literal `%`, and `\_` matches a literal `_`.

- **FR-SCH-013**: The system SHALL evaluate `--pattern` locally, in memory,
  against the names it has read. It SHALL NOT send the pattern to the server.

  *Rationale.* Local evaluation is the only way the same pattern selects the
  same set with a database, without one, and against any server. Server-side
  `LIKE` would make the result depend on the collation of the catalogue columns,
  which is invisible on the command line and therefore breaks determinism.
  Catalogue reads are already whole-list reads, so filtering in memory costs
  nothing.

- **FR-SCH-014**: `--pattern` SHALL fold case over ASCII `A-Z` and `a-z` only,
  independently of the server, of the database collation, and of the locale.

  *Accepted cost.* `order%` also matches `Orders`, which a binary server
  collation would not match.

- **FR-SCH-015**: `--pattern` SHALL NOT be declared by any command other than
  `tpl schema tables`, `tpl schema views`, and `tpl schema routines`.

- **BR-SCH-001**: A pattern selects the same set whatever the source of the
  catalogue: a live read, a cached read, or a context file.

## `tpl schema dump`

- **FR-SCH-016**: `tpl schema dump` SHALL emit the whole selected database as a
  single JSON document.

- **FR-SCH-017**: The document SHALL have the shape
  `{"schema_version":1,"database":{…}}`.

- **FR-SCH-018**: The document SHALL contain only the server-derived part of the
  render context. It SHALL NOT contain `vars`, `tpl`, or `now`.

  *Rationale.* Three of the five top-level context variables do not come from
  the database. Including `now` would make two dumps of an unchanged database
  differ, and a committed snapshot would produce a diff on every regeneration.

- **FR-SCH-019**: `tpl schema dump` SHALL NOT declare `--format`. Supplying
  `--format` to it is an unknown-flag error under `FR-CLI-019`, exit `64`.

  *Rationale.* A flag with a single permitted value is not a choice, and
  declaring it would suggest an alternative exists. The `DESCRIPTION` section of
  its help states that the output is JSON.

- **FR-SCH-020**: `tpl schema dump` SHALL declare `--pretty`, which SHALL apply
  without any accompanying `--format`, per `FR-OUT-009`.

- **FR-SCH-021**: IF `--pattern` is supplied to `tpl schema dump`, THEN the
  system SHALL exit `64`.

  *Rationale.* A partial context would fail at render time on a missing object,
  and the round-trip contract depends on a dump being complete. A context file
  must be able to say for itself that it is whole.

- **FR-SCH-022**: The document emitted by `tpl schema dump` SHALL be accepted by
  `tpl render --context`, per `FR-RND-009`.

## Flags and output

- **FR-SCH-023**: Every `schema` subcommand except `dump` SHALL declare
  `--format <text|json>`, defaulting to `text`, and `--pretty`.

- **FR-SCH-024**: Every `schema` subcommand, `dump` included, SHALL declare
  `--direct` and `--no-cache`, with the meanings defined in
  [cache-commands.md](cache-commands.md).

- **FR-SCH-025**: Every `schema` subcommand SHALL read through the catalogue
  cache, per `FR-CACHE-002`.

- **FR-SCH-026**: In `text` output, a listing SHALL be presented as aligned
  columns under a header row:

  ```
  tpl -d shop schema tables

  NAME          ENGINE  COLUMNS  COMMENT
  customers     InnoDB        14  Registered buyers
  order_items   InnoDB         7
  orders        InnoDB        21  One row per order
  ```

  *Amended in the second edition.* The listing previously carried a `ROWS`
  column, which is the server's row estimate. The storage engine revises that
  estimate without any change to the structure, so two reads of an unchanged
  database differ — which contradicts `NFR-DET-001` and the argument
  `FR-SCH-018` used to keep `now` out of the dump. `COLUMNS` is a structural
  count and is stable. The general rule is `FR-CAT-024`.

- **FR-SCH-027**: The `text` output of any `schema` subcommand is not a
  contract, per `FR-OUT-004`. Anything parsing a listing must use
  `--format json`.

- **FR-SCH-028**: The system SHALL order tables by name, columns by ordinal
  position, and indexes by name, per `NFR-DET-002`.

- **FR-SCH-029**: The `EXAMPLES` section of `tpl schema tables` SHALL show the
  canonical loop over every table, which uses `--format json`.

  *Rationale.* `--all-tables` no longer exists, so iterating over objects is the
  caller's job and the help must show how.

## Business rules

- **BR-SCH-002**: The first arm is read-only with respect to the database under
  every circumstance. It issues no DDL, no DML, and no write statement, and it
  enforces a read-only session at the engine level on every connection it opens.
  How that session is established, and the exact statement used, is outside the
  scope of this edition; the failure to establish it is `78`, per `FR-ERR-001`.

- **BR-SCH-003**: The first arm is not read-only with respect to the filesystem.
  A cache miss writes to `.tpl/.cache/`, per `FR-CACHE-004`. Only
  `--direct --no-cache` guarantees that no file is touched.

## Dependencies

- [cache-commands.md](cache-commands.md) — read-through behaviour, `--direct`,
  `--no-cache`.
- [output-formats.md](output-formats.md) — `text` and `json` rules, `--pretty`.
- [render-command.md](render-command.md) — the `--context` half of the
  dump round-trip.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `64`, `66`, `69`, `77`,
  `78`.

## Open questions

- [OQ-009](open-questions.md#oq-009) — catalogue return types and collation, to
  be verified against the container.
- [OQ-010](open-questions.md#oq-010) — the exact field lists for routines,
  triggers, generated columns, and foreign-key rules.
- [OQ-024](open-questions.md#oq-024) — the field list of `tpl schema info`.
