---
title: The Context Document
status: draft
last-reviewed: 2026-09-10
related: [catalogue-coverage.md, output-formats.md, schema-commands.md, render-command.md]
---

# The Context Document

## Overview

One document carries the model. `tpl schema dump` emits it, `tpl render
--context` consumes it, and a template sees the same material as the `database`
context variable. This file fixes its structure: how collections are shaped, how
deep a reference is followed, how a column default and a column type are
expressed, and what the document promises about its own consistency.

The document is plumbing contract. Its compatibility rule is `FR-OUT-014`, and
every rule of [output-formats.md](output-formats.md) applies to it.

## Scope

In scope: the structural rules of the document — array shape, reference depth,
the default discriminant, the decomposition of a column type, the treatment of
absence, and the consistency the document promises; and the content of the
three context variables that do not come from a server.

Out of scope: which objects and fields the document carries, which is
[catalogue-coverage.md](catalogue-coverage.md); the transport rules of JSON,
which are [output-formats.md](output-formats.md); the cache-specific fields,
which are [cache-documents.md](cache-documents.md); and how each context
variable is bound to a source, which is
[render-command.md](render-command.md).

## Actors

- **Calling agent**, parsing the document.
- **Template**, reading the same material through the render context.
- **`tpl` itself**, on the `--context` path, where the document is untrusted
  input.

## Identity of the document

- **FR-CTX-001**: The document defined here SHALL be the server-derived part of
  the render context, per `FR-SCH-018`, and SHALL be the document `tpl schema
  dump` emits under the shape of `FR-SCH-017`.

- **FR-CTX-002**: The document SHALL obey `FR-OUT-011` for its version key,
  `FR-OUT-012` for absent values, `FR-OUT-013` for key order, and `FR-OUT-014`
  for compatibility.

## Collections

- **FR-CTX-003**: Every collection SHALL be a JSON array, ordered by the rules
  of `NFR-DET-002`.

- **FR-CTX-004**: An empty collection SHALL be emitted as `[]`. It SHALL NOT be
  emitted as `null` and SHALL NOT be omitted.

- **FR-CTX-005**: `null` SHALL be reserved for an absent scalar. A consumer may
  therefore test a collection for emptiness without first testing it for
  nullity.

  *Rejected.* Maps keyed by object name, which would give a template direct
  lookup without a helper. They were rejected because ordering would then exist
  only in the emitter and could not be read back from the document, and
  `NFR-DET-002` would have no observable subject.

## Reference depth

- **FR-CTX-006**: A foreign key SHALL embed the table it references, one level
  deep.

- **FR-CTX-007**: The embedded table SHALL carry its columns, its indexes, and
  its primary key in full.

- **FR-CTX-008**: The `foreign_keys` of an embedded table SHALL hold the names
  of the referenced tables as strings, not as objects.

- **FR-CTX-009**: The cut of `FR-CTX-008` SHALL be one rule applied at the first
  hop, whatever the shape of the reference graph. A cycle `A → B → A` therefore
  terminates at the first hop with a string, and no traversal can fail to
  terminate.

- **BR-CTX-001**: One constant rule was chosen over a rule that adapts to the
  graph because a template author must be able to know, without inspecting the
  database, exactly how deep the object in hand goes.

  *Rejected.* Names only everywhere, with a `table()` lookup function to resolve
  them — the smallest document and a trivial round-trip, but every template that
  wants a referenced column has to perform the lookup itself. Also rejected: a
  summary embedding of name, primary key, and column names only, which grows the
  document by roughly a sixth instead of roughly doubling it, but which stops
  short of exactly the field — the referenced column's type — that a foreign-key
  accessor needs.

  *Accepted cost.* Over a database with 180 foreign keys across 200 tables, the
  embedding roughly doubles the column volume of the document. That cost is
  carried by the memory budget and by the `WL-002` scalar of
  [performance-requirements.md](performance-requirements.md).

- **FR-CTX-010**: `referenced_by` SHALL be a collection, per `FR-CAT-013` and
  `FR-CTX-003`.

## Column defaults

- **FR-CTX-011**: A column default SHALL be a structure carrying a `kind`
  discriminant, or `null`.

- **FR-CTX-012**: The system SHALL emit exactly the following four forms, and no
  others:

  | Case | Emitted |
  |---|---|
  | The column has no default | `null` |
  | The default is a literal | `{"kind":"literal","value":"0"}` |
  | The default is an expression | `{"kind":"expression","value":"current_timestamp()"}` |
  | The default is an explicit `DEFAULT NULL` | `{"kind":"null"}` |

- **FR-CTX-013**: `kind` SHALL be an enumerated field. Adding a value to it is
  not a breaking change, per `FR-OUT-014`.

- **BR-CTX-002**: The server distinguishes a string literal from an expression
  by quoting the literal, and that distinction is the whole content of the
  field. Emitting the raw string would make every template reimplement the same
  unquoting heuristic, and each one would get a different case wrong — an empty
  string, a quote inside a literal, a function call whose name looks like a
  word.

  *Rejected.* The raw server string alone; and the structure with a `default_raw`
  field carried alongside it, which would offer two answers to one question and
  guarantee that some templates read the one that was not intended.

## Column types

- **FR-CTX-014**: A column SHALL carry `column_type`, the type exactly as the
  server writes it.

- **FR-CTX-015**: A column SHALL additionally carry the decomposed parts of its
  type: `data_type`, `precision`, `scale`, `length`, `unsigned`, `charset`,
  `collation`, and `values`.

- **FR-CTX-016**: `values` SHALL carry the member list of an `ENUM` or `SET`
  column, and SHALL be `null` for every other type.

- **FR-CTX-017**: A part that does not apply to a type SHALL be `null`, per
  `FR-CTX-005`.

- **FR-CTX-018**: IF the system does not recognise a type, THEN it SHALL emit
  `column_type` unchanged and SHALL emit every decomposed part as `null`.

  *Rationale.* The raw string is the safety net. A type introduced by a later
  server reaches the template intact rather than being lost or guessed at, and a
  template can always fall back to matching on `column_type`.

  *Rejected.* The raw string alone, which would force the tests of
  [template-environment.md](template-environment.md) to classify a column by
  parsing a string; and the decomposition alone, which would erase an
  unrecognised type entirely.

## Column identity and derived facts

- **FR-CTX-019**: Every column SHALL carry `table_name`, naming the table it
  belongs to.

  *Rationale.* A column reached through a loop, through an index, or through a
  foreign key must be able to find its own table without the template having
  carried a reference to it.

- **FR-CTX-020**: A column SHALL carry the static attributes the catalogue
  states for it, including whether it is auto-incremental, whether it is
  generated, and whether it is invisible, per `FR-CAT-009` and `FR-CAT-027`.

- **FR-CTX-021**: The document SHALL NOT materialise on a column any fact the
  table already states. `is_primary_key` and `is_unique` SHALL NOT be fields.

- **FR-CTX-022**: The facts of `FR-CTX-021` SHALL be reachable through the tests
  `primary_key` and `unique`, which resolve `table_name` against the render
  context, per `FR-ENV-015`.

- **BR-CTX-003**: Calculating rather than materialising keeps one statement of
  each fact. A column that says it is not part of the primary key while the
  table says it is would be a document that contradicts itself, and nothing in
  the emitter could prevent it once both are written.

  *Rejected.* Passing the table to the test as an argument,
  `col is primary_key(table)`, which is pure and needs no context lookup, but
  which would leave `tpl` with two grammatical classes of test and a template
  author guessing which class a given test belongs to.

## The non-server variables

`FR-RND-023` names five top-level context variables and `FR-RND-024` requires
three of them always to be injected. The two that come from a server are
everything above; these are the other three. They are in this file because a
template reads all five through one context and needs one place that says what
each holds, and not because they appear in the document `tpl schema dump`
emits — `FR-SCH-018` keeps them out of it.

- **FR-CTX-026**: `vars` SHALL be a JSON object whose keys are the `--set` keys
  of the invocation, per `FR-RND-008`, and whose values are strings, per
  `FR-RND-015`. WHEN no `--set` is supplied, `vars` SHALL be `{}`.

- **FR-CTX-027**: `tpl` SHALL be a JSON object carrying exactly one key,
  `version`, whose value is the version string `FR-HELP-005` prints:

  ```json
  {"version":"0.1.0"}
  ```

  *Rationale.* An object rather than the bare string, so that the variable can
  gain a field without breaking a template, which `FR-OUT-014` makes the only
  non-breaking way to grow. `{{ tpl.version }}` in a generated file header is
  the case it exists for.

- **FR-CTX-028**: `now` SHALL be a string carrying the render time as an
  RFC 3339 timestamp in UTC, with a `Z` offset and second precision:

  ```
  2026-09-10T08:14:22Z
  ```

  *Rationale.* A string in a fixed format is the only form a template can use
  today. `FR-ENV-018` guarantees no date filter, and `FR-ENV-025` forbids a
  function that reads a clock, so a structured `now` would give a template
  parts it has no way to format and no way to reassemble. UTC with a fixed
  precision also means the one documented source of non-reproducibility varies
  in exactly one way — the instant — rather than also with the machine's zone
  and locale.

  *Rejected.* A structured value carrying named parts, which needs a formatting
  filter this specification does not provide; and the engine's own datetime
  type with a date filter added to `FR-ENV-018`, which would put a filter into
  group 2 for the sake of one variable and tie its behaviour to the pinned
  engine version of `OQ-049`.

- **FR-CTX-029**: `now` SHALL be evaluated once per invocation, at render time,
  and every reference to it in one render SHALL yield the same value.

  *Rationale.* A template that interpolates `now` twice must not produce two
  timestamps, and `FR-RND-002` gives one render per invocation, so there is one
  instant to record.

- **FR-CTX-030**: `now` SHALL be the single documented source of
  non-reproducibility, per `NFR-DET-005`. A template that does not reference it
  SHALL produce byte-identical output between runs against unchanged inputs.

- **BR-CTX-005**: These three are not part of the document this file otherwise
  fixes, and a `--context` document that carries any of them has its value
  ignored, per `FR-RND-024`. They are specified here because a template author
  reading the context needs one file that answers "what is in each of the five
  variables", and splitting three of the five into `render-command.md` would
  have answered where each *comes from* without ever saying what it *holds*.

## Consistency

- **FR-CTX-023**: WHEN the document is produced by a server read, every object
  referenced from another object in it SHALL be present in it.

- **FR-CTX-024**: The document SHALL NOT promise to be a point-in-time snapshot,
  and the specification SHALL state that it is not one.

  *Rationale.* A snapshot of the catalogue is not attainable through the
  catalogue tables under any transaction mode the server offers. Promising one
  would be a promise `tpl` cannot keep, and a caller that believed it would
  build on a guarantee that does not exist.

- **FR-CTX-025**: WHEN the document is served wholly or partly from the cache,
  it SHALL promise neither the referential integrity of `FR-CTX-023` nor a
  snapshot. The `source` field of `FR-CDOC-009` is the signal, and
  [cache-documents.md](cache-documents.md) governs.

- **BR-CTX-004**: The three levels are distinct and are stated separately
  because a caller's next step differs between them: referential integrity means
  a traversal will not dangle, a snapshot would mean two objects were read at
  one instant, and a cache-served document means neither is claimed.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the objects and fields this
  document carries.
- [output-formats.md](output-formats.md) — the JSON rules the document obeys.
- [schema-commands.md](schema-commands.md) — `tpl schema dump`, which emits it.
- [render-command.md](render-command.md) — `--context`, which consumes it, and
  `FR-RND-020`, the code for a document that does not match this contract.
- [cache-documents.md](cache-documents.md) — the fields a cached read adds.

## Open questions

- [OQ-027](open-questions.md#oq-027) — how the catalogue distinguishes a literal
  default from an expression, and how an explicit `DEFAULT NULL` is reported.
- [OQ-028](open-questions.md#oq-028) — the textual form of `column_type` per
  type family, and how the unsigned attribute appears in it.
- [OQ-029](open-questions.md#oq-029) — how `ENUM` and `SET` members are
  delimited and escaped.
- [OQ-030](open-questions.md#oq-030) — which of `precision`, `scale`, and
  `length` the catalogue populates for each type.
- [OQ-031](open-questions.md#oq-031) — how the character set and the collation
  of a column are reported, and what they hold for a non-textual type.
- [OQ-043](open-questions.md#oq-043) — whether `referenced_by` embeds the
  referencing table under the rule of `FR-CTX-006`, or holds names only.
