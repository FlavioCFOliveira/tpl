---
title: The Template Environment
status: draft
last-reviewed: 2026-09-09
related: [render-semantics.md, render-command.md, context-document.md, help-and-version.md]
---

# The Template Environment

## Overview

A template reaches the model through the context variables and acts on it
through filters, tests, and global functions. This file defines that surface:
what `tpl` registers, what it guarantees from the engine underneath, what it
deliberately does not provide, and what it forbids.

The surface is contract. What is guaranteed here can be relied on by a template
committed to a repository; what is not guaranteed may work today and stop
working when the engine is updated, and the difference is stated rather than
left to be discovered.

## Scope

In scope: the three contract groups, the filters and tests `tpl` registers, the
global functions, the removals and their reasons, auto-escaping, and how the
surface is published.

Out of scope: evaluation semantics — what a filter does when its operand is
wrong, what an interpolated `null` produces, and how whitespace is handled —
which belong to [render-semantics.md](render-semantics.md); and the content of
the context variables, which belongs to
[context-document.md](context-document.md).

## Actors

- **Template author**, writing against this surface.
- **Calling agent**, discovering it through `tpl help --format json`.
- **Template engine**, whose own surface underlies part of it.

## The three contract groups

- **FR-ENV-001**: The template surface SHALL consist of exactly three groups:

  | Group | Contents | Guarantee |
  |---|---|---|
  | 1 | The filters, tests, and functions `tpl` registers | Full contract |
  | 2 | An enumerated list of filters inherited from the engine | Guaranteed against a pinned engine minor version |
  | 3 | Everything else the engine offers | Works; no guarantee |

- **FR-ENV-002**: A name in group 1 SHALL NOT be renamed or removed without a
  breaking change. Adding a name to group 1 SHALL NOT be breaking.

- **FR-ENV-003**: A name in group 2 SHALL behave as the pinned engine minor
  version defines it. Changing the pinned minor version SHALL be a deliberate
  decision recorded outside this specification, and SHALL be accompanied by a
  check that every name of group 2 still exists and still behaves as before.

- **FR-ENV-004**: The specification and the help SHALL state that group 3
  carries no guarantee, so that a template author knows which side of the line a
  name falls on before depending on it.

- **BR-ENV-001**: Inheriting the engine's whole surface as contract was
  rejected: it would let an engine update break `tpl` without anyone having
  decided anything. Guaranteeing only `tpl`'s own registrations was also
  rejected: it would leave `{{ cols | join(", ") }}`, the most common idiom
  there is, with no guarantee at all.

- **FR-ENV-005**: `tpl help --format json` SHALL enumerate all three groups: the
  names of group 1, the names of group 2, and a statement that group 3 exists
  and is unguaranteed.

  *Rationale.* A calling agent loads the whole surface in one invocation, per
  `FR-HELP-016`. A surface it cannot enumerate is one it will discover by trial
  and error.

## Filters that `tpl` registers

- **FR-ENV-006**: The system SHALL register the naming filters `pascal`,
  `camel`, `snake`, `upper_snake`, and `kebab`.

- **FR-ENV-007**: The system SHALL register the code filters `quote`,
  `sql_type`, `json`, `indent`, and `comment`.

- **FR-ENV-008**: `quote` SHALL quote an identifier for MariaDB using backticks,
  and SHALL be correct for MariaDB rather than for SQL in general.

## Filters that do not exist

- **FR-ENV-009**: The system SHALL NOT provide a `rust_type` filter.

- **FR-ENV-010**: The system SHALL NOT provide a `go_type` filter, nor a filter
  mapping a column to the types of any other target language.

- **FR-ENV-011**: The mapping those filters performed SHALL instead be delivered
  as a template macro at `.tpl/templates/rust/_types.jinja`, written by
  `tpl init` per `FR-PROJ-017`.

- **BR-ENV-002**: A type mapping is an opinion, and it belongs to the project
  that holds it rather than to the binary. Whether `DECIMAL` becomes a
  third-party decimal type, an `f64`, or a `String` is a decision each project
  makes once and edits when it changes its mind; frozen into `tpl` it would be
  editable only by releasing a new version of `tpl`. Delivering it as a macro
  also stops the tool privileging one target language, which is what a
  `rust_type` in the binary and a `go_type` beside it would institutionalise.

  *Rejected.* Keeping `rust_type` with a standard-library-only mapping, which
  answers the easy cases and leaves `DECIMAL`, `JSON`, and the temporal types
  exactly where they were. Also rejected: keeping it with third-party crate
  types, which would oblige `tpl` to track those crates' major versions for as
  long as it exists.

- **FR-ENV-012**: The system SHALL NOT provide a `plural` filter.

- **FR-ENV-013**: The system SHALL NOT provide a `singular` filter.

- **BR-ENV-003**: Correct English inflection is a project in itself —
  person/people, status/statuses, index/indices or indexes, data/data — and a
  wrong plural applied to a table name that is not English is guaranteed noise
  in generated code. Removing the two filters before anything depends on them is
  free; removing them later would be a compatibility break.

  *Rejected.* Keeping them with a closed rule set and an irregulars list
  published as a test vector, which makes the behaviour predictable without
  making it right. Also rejected: a naive suffix-only version, which would
  document `persons` and `indexs` as correct output.

## Tests

- **FR-ENV-014**: The system SHALL register the tests `nullable`,
  `primary_key`, `auto_increment`, `unique`, `numeric`, `temporal`, and
  `textual`.

- **FR-ENV-015**: `primary_key` and `unique` SHALL resolve the operand's
  `table_name` against the render context, per `FR-CTX-019` and `FR-CTX-022`,
  and SHALL answer from what the table states.

- **FR-ENV-016**: The tests of `FR-ENV-015` therefore depend on the render
  context and not only on their operand. The specification and the help SHALL
  state this.

- **FR-ENV-017**: IF the table named by a column's `table_name` is absent from
  the render context, THEN the system SHALL fail the render with `65`
  (`EX_DATAERR`), per `FR-SEM-017`.

  *Rationale.* The case is reachable: a `--context` document assembled by hand,
  or a column carried across from one context into another, can name a table
  that is not there. Answering `false` would report "this column is not a
  primary key" when the truth is that the question could not be asked.

## Inherited filters

- **FR-ENV-018**: The system SHALL guarantee the following inherited filters,
  and no others, under the terms of `FR-ENV-003`:

  `default`, `join`, `length`, `map`, `select`, `reject`, `first`, `last`,
  `reverse`, `sort`, `trim`, `upper`, `lower`, `replace`.

- **FR-ENV-019**: The list of `FR-ENV-018` SHALL be closed. A filter the engine
  offers that is absent from it belongs to group 3.

## Global functions

- **FR-ENV-020**: The system SHALL provide exactly the following global
  functions, and no others:

  | Function | Purpose |
  |---|---|
  | `table(name)` | Resolve a table by name in the render context |
  | `view(name)` | Resolve a view by name in the render context |
  | `routine(name)` | Resolve a routine by name in the render context |
  | `column(table, name)` | Resolve a column of a named table |
  | `fail(message)` | End the render with `65`, carrying the author's message |

- **FR-ENV-021**: `fail(message)` SHALL end the render with `65` and SHALL carry
  the message the template supplied, per `FR-SEM-014`.

- **FR-ENV-022**: The system SHALL NOT provide a function that reads the
  environment.

- **FR-ENV-023**: The system SHALL NOT provide a function that reads a file.

- **FR-ENV-024**: The system SHALL NOT provide a function that performs network
  access.

- **FR-ENV-025**: The system SHALL NOT provide a function that reads a clock.
  The `now` context variable of `FR-RND-023` is the only source of time
  available to a template.

- **BR-ENV-004**: A template arrives in a clone and is read by `tpl` before
  anybody has reviewed it, so it is semi-trusted input. The four prohibitions
  above are what keep `tpl render` from being a way to run arbitrary code with
  the caller's privileges, and they are stated as prohibitions rather than left
  as an absence so that adding one is visibly a change of position.

  *Rejected.* Lookups only, without `fail`, which would leave a generator that
  meets an unmappable type unable to say why it stopped. Also rejected: adding
  a `warn` function, which would let a render finish with exit `0` and hand the
  caller an incomplete file with nothing in the exit code to say so.

## Escaping

- **FR-ENV-026**: Auto-escaping SHALL be off, always.

- **FR-ENV-027**: The system SHALL NOT key escaping on a file extension, on a
  penultimate extension, or on any other property of a name.

- **FR-ENV-028**: `escape` SHALL be available as an explicit filter. Escaping
  happens where a template asks for it and nowhere else.

- **BR-ENV-005**: Silent escaping in a code generator produces `&amp;`
  inside a SQL string and `&lt;` inside a generic type parameter, and the
  author sees it only in the generated file. A rule keyed on the penultimate
  extension would additionally make the same command emit different bytes
  depending on the filename — invisible on the command line, which is precisely
  what was rejected for the `--format` default in `FR-OUT-002`.

  *Rejected.* The penultimate-extension rule covering `.html`, `.htm`, and
  `.xml`; and an explicit `{% autoescape %}` block, which keeps a second mode
  alive for a target this tool does not have.

## Compatibility

- **FR-ENV-029**: Adding a name to group 1 SHALL NOT break the contract.
  Renaming or removing one SHALL, and SHALL be recorded in the project
  changelog.

- **BR-ENV-006**: The compatibility rule of the template surface mirrors that of
  the JSON document in `FR-OUT-014`, for the same reason: a caller may depend on
  what is present and must not depend on what is absent.

## Dependencies

- [render-semantics.md](render-semantics.md) — what happens when a filter, a
  test, or a function is given the wrong thing.
- [context-document.md](context-document.md) — the material the filters and
  tests act on.
- [help-and-version.md](help-and-version.md) — the JSON document in which the
  three groups are published.
- [project-and-discovery.md](project-and-discovery.md) — `FR-PROJ-017`, which
  writes `.tpl/templates/rust/_types.jinja`.

## Open questions

- [OQ-049](open-questions.md#oq-049) — which engine minor version group 2 is
  pinned to.
- [OQ-050](open-questions.md#oq-050) — which contract group `escape` belongs to,
  and exactly what it escapes.
