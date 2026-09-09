---
title: Output Formats
status: draft
last-reviewed: 2026-09-09
related: [cli-contract.md, schema-commands.md, errors-and-exit-codes.md, help-and-version.md]
---

# Output Formats

## Overview

`tpl` produces two kinds of output. `text` is laid out for a person to read and
is explicitly not a contract. `json` is the plumbing contract: versioned, stable,
and safe to parse. This file defines both, the `--pretty` rule, and how
catalogue bytes that are not valid text are handled on the way out.

## Scope

In scope: the `--format` and `--pretty` flags, the properties of `text` output,
the JSON contract and its compatibility rule, character encoding, and the
separation of stdout from stderr.

Out of scope: the content of any particular document, which belongs to the
module that owns the command producing it.

## `--format`

- **FR-OUT-001**: `--format <text|json>` SHALL default to `text`.

- **FR-OUT-002**: The default SHALL be fixed. The system SHALL NOT consult
  `isatty()`, `TERM`, or any other terminal property to choose a format.

  *Rationale.* `tpl schema tables` and `tpl schema tables | cat` produce
  identical bytes; the help can state `Default: text.` and have it be true in
  every context; and there is one set of escaping rules for catalogue-derived
  text instead of one per terminal state.

- **FR-OUT-003**: `--format` SHALL be declared only by the commands listed in
  `FR-GLOB-021`. Every other command SHALL reject it as an unknown flag.

## `text`

- **FR-OUT-004**: `text` output SHALL NOT be a contract. Its shape may change
  without a version bump.

- **FR-OUT-005**: The specification and the help SHALL state that anything
  parsing output must use `--format json`.

- **FR-OUT-006**: `text` output SHALL be laid out for a person: aligned columns
  under a header row, carrying the useful information rather than only the name.

- **BR-OUT-001**: Making `text` a contract would freeze every listing forever —
  no column, header, or total could ever be added. Making it a bare list of
  names would compose better into `xargs`, at the cost of the information a
  reader wants. The choice is deliberate, and the consequence is that iterating
  over objects goes through `--format json`.

## `json`

- **FR-OUT-007**: JSON output SHALL be compact by default: one line, no
  superfluous whitespace, terminated by a single newline.

- **FR-OUT-008**: `--pretty` SHALL produce a two-space indent with one key per
  line.

- **FR-OUT-009**: On a command that declares `--format`, `--pretty` SHALL
  require `--format json`. IF `--pretty` is supplied without it, THEN the system
  SHALL exit `64` (`EX_USAGE`).

- **FR-OUT-010**: On a command whose only output is JSON, `--pretty` SHALL stand
  alone, without any accompanying `--format`. `tpl schema dump --pretty` is
  valid; `tpl schema dump --format json` is `64`, per `FR-SCH-019`.

- **FR-OUT-011**: Every JSON document `tpl` emits SHALL begin with
  `schema_version`, which versions the document contract independently of the
  binary version.

- **FR-OUT-012**: An absent value SHALL be emitted as `null` and SHALL NOT be
  omitted, so that the shape of a document is constant.

- **FR-OUT-013**: Keys SHALL be emitted in a fixed order per structure. The
  emitting path SHALL NOT use an unordered map.

- **FR-OUT-014**: The JSON contract SHALL follow this compatibility rule:

  | Change | Breaking |
  |---|---|
  | Adding a field | No |
  | Adding a value to an enumerated field such as `kind` | No |
  | Removing a field | Yes |
  | Renaming a field | Yes |
  | Changing the type of a field | Yes |

- **FR-OUT-015**: WHEN an error is emitted in JSON, the document SHALL go to
  stderr and stdout SHALL remain empty. See `FR-ERR-011`.

- **FR-OUT-016**: `tpl render --context` SHALL accept a document in either
  compact or indented form.

## Character encoding

- **FR-OUT-017**: WHEN a catalogue value contains a byte sequence that is not
  valid UTF-8, the system SHALL replace it with U+FFFD and SHALL continue.

  *Rationale.* One odd byte in a legacy comment must not prevent the other 199
  tables from being read. Failing the whole command would block `schema dump`
  entirely.

  *Accepted cost.* A printed value is then no longer byte-identical to what the
  server holds, and a template generating code from a comment receives U+FFFD
  rather than the original byte — which was not representable anyway.

- **FR-OUT-018**: The system SHALL escape C0 control characters, tab excepted,
  in both `text` and `json` output.

  *Rationale.* No catalogue byte may reach the terminal uninterpreted. An escape
  character in a column comment must not be able to rewrite what the user sees.

- **FR-OUT-019**: The escaping of `FR-OUT-018` SHALL apply to every value
  interpolated into output or into a diagnostic message, whatever its source:
  the catalogue, a `--context` document, or the argument vector.

## Streams

- **FR-OUT-020**: Results SHALL go to stdout. Warnings, diagnostics, and log
  output SHALL go to stderr.

- **FR-OUT-021**: A command whose output is meant to be piped SHALL write
  nothing else to stdout.

- **FR-OUT-022**: Help output is a result and goes to stdout, per `BR-CLI-005`.

- **FR-OUT-023**: WHEN a command writes no result — `tpl init`,
  `tpl cfg database add` — stdout SHALL remain empty.

## Dependencies

- [cli-contract.md](cli-contract.md) — determinism, and the stream separation
  rule this file details.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — the JSON error
  document, and the `EPIPE` rule that depends on whether a JSON document is
  mid-flight.
- [help-and-version.md](help-and-version.md) — the JSON command tree, which
  obeys every rule here.

## Open questions

None specific to this module.
