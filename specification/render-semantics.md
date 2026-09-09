---
title: Render Semantics
status: draft
last-reviewed: 2026-09-09
related: [template-environment.md, render-command.md, errors-and-exit-codes.md, context-document.md]
---

# Render Semantics

## Overview

[template-environment.md](template-environment.md) says what a template may
call. This file says what happens when it calls it: how whitespace is treated,
what a filter or a test does when it is handed the wrong thing, what an
interpolated `null` produces, how that differs from a field that does not exist,
and how a template author ends a render deliberately.

One rule governs the whole file. A render that cannot do the right thing fails;
it does not produce something plausible. The failure mode this tool must avoid
above all others is emitting wrong code that compiles.

## Scope

In scope: whitespace control, operand type checking, coercion policy, the
treatment of `null` and of absence, author-signalled failure, and the context
dependency of the two derived tests.

Out of scope: the names available to a template, which belong to
[template-environment.md](template-environment.md); the flags and arguments of
`tpl render`, which belong to [render-command.md](render-command.md); and the
format of an error message, which belongs to
[errors-and-exit-codes.md](errors-and-exit-codes.md).

## Actors

- **Template author**, whose mistakes this file classifies.
- **Calling agent**, which sees the outcome as an exit code.

## Whitespace

- **FR-SEM-001**: The system SHALL NOT strip the newline that follows a block
  tag.

- **FR-SEM-002**: The system SHALL NOT strip the whitespace that precedes a
  block tag on its line.

- **FR-SEM-003**: The system SHALL preserve a trailing newline present at the
  end of a template's source.

- **FR-SEM-004**: Whitespace trimming SHALL be requested explicitly in the
  template, with the `{%-` and `-%}` markers.

- **BR-SEM-001**: The first two rules are the stock defaults of the template
  language, and keeping them means a correct template produces here exactly what
  it produces anywhere else. Turning them on would make templates more readable
  to write, at the cost of a template copied from another project of the same
  language silently losing lines. The third rule departs from the stock default
  deliberately: most formatters and most linters require a final newline, and a
  generator that omits it produces a diff on every file it writes.

  *Accepted cost.* `.tpl/templates/example.jinja`, required by `FR-PROJ-021` to
  read well, needs explicit `{%-` and `-%}` markers to do so — and that appears
  in the very first template a user of `tpl` ever sees.

## Operands and coercion

- **FR-SEM-005**: IF a test is applied to an operand of a type it does not
  accept, THEN the system SHALL fail the render with `65` (`EX_DATAERR`).

- **FR-SEM-006**: The failure of `FR-SEM-005` SHALL name the test, the type of
  the operand received, and the location in the template.

- **FR-SEM-007**: A test SHALL NOT answer `false` for an operand of a type it
  does not accept.

- **FR-SEM-008**: IF a naming filter is applied to a value that is not a string,
  THEN the system SHALL fail the render with `65`, naming the filter, the type
  received, and the location.

- **FR-SEM-009**: The system SHALL NOT coerce a value to a string before
  applying a filter that requires one. `{{ 42 | snake }}` SHALL fail.

- **BR-SEM-002**: This is the only answer coherent with the strict treatment of
  undefined variables that `FR-RND-031` already fixes. A silent `false` is what
  nearly every template engine does by default, and it is the wrong default
  here: `{% if column is nullable %}` answering `false` because `column` is a
  table emits code that compiles and is wrong, and nothing in the exit code says
  so. Coercing `42` to `"42"` has the same shape — it converts an author's
  mistake into output.

## Null and absence

- **FR-SEM-010**: WHEN a `null` is interpolated into the output, the system
  SHALL emit the empty string.

- **FR-SEM-011**: The system SHALL NOT emit the words `none` or `null` as the
  rendering of a `null`.

- **FR-SEM-012**: IF a template reads a field that does not exist on the value
  in hand, THEN the system SHALL fail the render with `65`.

- **FR-SEM-013**: The distinction of `FR-SEM-010` and `FR-SEM-012` SHALL be
  preserved. "There is no comment" and "you misspelled the field name" are
  different failures and produce different outcomes.

- **BR-SEM-003**: Treating both as the empty string would undo the strict
  undefined behaviour for fields, which is the single most valuable guard a
  template author has. Treating both as an error would be worse in practice:
  almost every field of the model is nullable, so a realistic template would
  need `| default("")` on nearly every interpolation, and the guard would be
  turned off wholesale by an author trying to make progress.

## Author-signalled failure

- **FR-SEM-014**: WHEN a template calls `fail(message)`, the system SHALL end
  the render with `65` and SHALL carry the author's message.

- **FR-SEM-015**: The message of `FR-SEM-014` SHALL be reported in the message
  format of `FR-ERR-008`, with the template, the line, and the column, per
  `FR-ERR-011`.

- **FR-SEM-016**: The system SHALL NOT provide a way for a template to emit a
  warning and continue, per `FR-ENV-020`.

  *Rationale.* A warning leaves the exit code at `0`, so a caller that checks
  only the code receives an incomplete generated file and has no way to know.

## Context dependency of the derived tests

- **FR-SEM-017**: IF a test must resolve a column's `table_name` against the
  render context and that table is not present, THEN the system SHALL fail the
  render with `65`, per `FR-ENV-017`.

- **FR-SEM-018**: `FR-SEM-017` SHALL apply to a column arriving from a
  `--context` document exactly as it applies to a column read from a server.

  *Rationale.* A hand-assembled or partial context document is the case that
  makes the rule reachable, and it is precisely the case in which answering
  `false` would be indistinguishable from a correct answer.

## Reporting

- **FR-SEM-019**: Every failure defined in this file SHALL be a render failure
  under `FR-RND-031`, and SHALL carry the template name, the line, and the
  column.

- **FR-SEM-020**: WHEN a failure defined in this file occurs, stdout SHALL carry
  at most one incomplete result, per `FR-RND-034`.

## Business rules

- **BR-SEM-004**: Every rule in this file resolves the same trade-off the same
  way: a render that cannot be certain of the right output fails loudly rather
  than producing plausible output quietly. The cost is that a template which
  worked by accident under a permissive engine stops working under `tpl`, and
  that cost is accepted.

## Dependencies

- [template-environment.md](template-environment.md) — the filters, tests, and
  functions whose behaviour is defined here.
- [render-command.md](render-command.md) — `FR-RND-030`, `FR-RND-031`, and
  `FR-RND-034`, the render failure rules this file specialises.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `65`, and the message
  format every failure here uses.
- [context-document.md](context-document.md) — `table_name`, and the reason the
  derived tests need it.

## Open questions

None specific to this module.
