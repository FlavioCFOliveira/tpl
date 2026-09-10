---
title: Errors and Exit Codes
status: draft
last-reviewed: 2026-09-10
related: [cli-contract.md, output-formats.md, security.md, global-flags.md]
---

# Errors and Exit Codes

## Overview

The exit code is the most reliable channel a calling agent has for knowing what
happened. Each distinct condition therefore has its own code, and the code alone
must be enough to choose the next step. This file defines the codes, the order
in which conditions are evaluated, the shape of an error message, and the rules
that keep a suggested command safe to run.

## Scope

In scope: the exit code table, validation order, message format in text and in
JSON, nearest-match suggestions, hint construction, and `EPIPE`.

Out of scope: the wording of any individual message.

## Actors

- **Calling agent**, which branches on the exit code and self-corrects from the
  `hint` line. This is the primary consumer.
- **Operator**, reading the four-line message at a shell prompt.

## Exit codes

- **FR-ERR-001**: The system SHALL use the following codes, following the
  `sysexits.h` convention, and no others:

  | Code | Name | Condition | What the caller should do |
  |---|---|---|---|
  | `0` | `EX_OK` | Success | Continue |
  | `64` | `EX_USAGE` | Unknown command or flag, missing required argument, mutually exclusive flags, malformed flag value, unknown configuration key | Fix the invocation; consult `--help` |
  | `65` | `EX_DATAERR` | Template syntax error, render failure, malformed `--context` document, render deadline exceeded, template path escaping the root | Fix the template or the context |
  | `66` | `EX_NOINPUT` | A named object does not exist: table, view, routine, template, database entry, configuration key | List what exists and choose another name |
  | `69` | `EX_UNAVAILABLE` | Server unreachable: DNS, connection refused, network deadline exceeded, TLS failure | Check host and network; the operation is read-only and therefore repeatable |
  | `70` | `EX_SOFTWARE` | Internal error — a defect in `tpl` | Report it; not fixable by the caller |
  | `73` | `EX_CANTCREAT` | `tpl init` cannot create `.tpl`, or `.tpl` already exists at the destination | Check permissions, or choose another destination |
  | `74` | `EX_IOERR` | I/O failure reading `.tpl`, or writing to stdout, including a pipe closed mid-document in JSON | Check permissions and free space |
  | `77` | `EX_NOPERM` | Authentication refused, or insufficient privileges on the catalogue | Fix the credentials, or request read access |
  | `78` | `EX_CONFIG` | No `.tpl` found; unsafe `.cfg` ownership or mode; malformed `.cfg`; invalid entry; undefined `${VAR}`; `password_command` deadline exceeded; read-only session could not be enforced; no database entry selected | Fix `.tpl/.cfg`, or run `tpl init` |

- **FR-ERR-002**: Each distinct condition SHALL have its own code. The system
  SHALL NOT collapse two conditions onto one code where the caller's next step
  would differ.

- **FR-ERR-003**: `73` SHALL be produced only by `tpl init`.

  *Rationale.* With no output flags, `tpl` creates no destination other than
  `.tpl`.

- **FR-ERR-030**: `70` (`EX_SOFTWARE`) SHALL be produced by exactly two
  conditions, and by no other: a panic caught at the top level of the process,
  and a violated internal invariant the system detects and declines to
  continue past.

  *Rationale.* It was the only code in the table of `FR-ERR-001` for which no
  requirement stated a producing condition, so nothing could confirm that the
  binary is able to return it at all.

- **FR-ERR-031**: The system SHALL provide a deliberate trigger for `70`, so
  that the test `BR-ERR-001` mandates for it can exist. The trigger SHALL NOT
  appear in any help text, in the JSON command tree of `FR-HELP-016`, or in the
  command tree of `FR-CLI-002`.

  *Rationale.* A code with no test is a code nobody has confirmed the binary
  can return, and `70` cannot be reached from a correct invocation by
  definition. Keeping the trigger out of the published surface keeps the tree
  closed, per `FR-CLI-002`, and keeps it out of the caller's context window.

- **FR-ERR-032**: A `70` SHALL follow the message format of `FR-ERR-008`, and
  its `hint` SHALL say that the condition is a defect in `tpl` and is not
  correctable by the caller.

- **BR-ERR-001**: Exit codes are contract. Each code SHALL have at least one
  integration test that exercises it, and that test is part of the definition of
  done for the feature that can produce it. `70` is exercised through the
  trigger of `FR-ERR-031`.

## No database entry versus a missing entry

- **FR-ERR-004**: IF no database entry is selected and the command requires one,
  THEN the system SHALL exit `78`.

- **FR-ERR-005**: IF a named database entry does not exist, THEN the system
  SHALL exit `66` with a nearest-match suggestion.

  *Rationale.* Nothing selected is a configuration problem, pointing at the
  file. A name that does not resolve is a named object that does not exist, like
  a missing table. Folding both into `78` would make one code cover six distinct
  conditions and leave the real distinction only in the `kind` field.

## Validation order

- **FR-ERR-006**: The system SHALL evaluate conditions in the following fixed
  order, and SHALL report the first that fails:

  ```
  1. argument parsing                          64
  2. .tpl discovery and trust checks           78
  3. .cfg read and validation                  78
  4. database entry resolution                 78 or 66
  5. cache or connection                       69 or 77
  6. catalogue object resolution               66
  7. template resolution                       66
  8. render                                    65
  ```

  Steps 2 and 3 SHALL be skipped for the commands `FR-PROJ-025` names, which
  require no project. Step 1 runs for every command without exception, so an
  unknown flag on `tpl --help` is still `64`.

  *Amended in the third edition.* The sentence about steps 2 and 3 is new. The
  order alone did not say whether `--help` reached discovery; `FR-PROJ-025` now
  names the commands that skip them.

- **FR-ERR-007**: The order of `FR-ERR-006` SHALL decide which code wins when
  more than one condition is unsatisfied.

## Message format

- **FR-ERR-008**: An error message SHALL answer three questions, in this order
  and no others: what failed, why, and what to do next. It SHALL be written to
  stderr in four labelled lines:

  ```
  error: table 'ordrs' does not exist in database 'shop'
  cause: no row in the catalogue matches schema 'shop' and table 'ordrs'
  hint:  did you mean 'orders'? list the available tables with: tpl -d shop schema tables
  exit:  66 (EX_NOINPUT)
  ```

- **FR-ERR-009**: `hint` SHALL carry a concrete, runnable command wherever one
  exists.

  *Rationale.* It is the line a calling agent uses to correct itself, and the
  one that gives the most in return.

- **FR-ERR-010**: `cause` SHALL be factual and specific, and SHALL NOT restate
  the `error` line.

- **FR-ERR-011**: A template error SHALL carry the template name, the line, the
  column, and the chain of underlying template-engine errors.

- **FR-ERR-012**: The system SHALL NOT emit vague advice. It SHALL name the key,
  the file, and the expected value rather than suggesting that the caller check
  their configuration.

- **FR-ERR-013**: The system SHALL NOT include credentials in any error message,
  at any verbosity level.

## JSON errors

- **FR-ERR-014**: WHEN JSON error output is in force, the system SHALL emit the
  error as a single JSON document on stderr, with stdout left empty:

  ```json
  {"error":{"exit":66,"code":"EX_NOINPUT","kind":"table_not_found","message":"table 'ordrs' does not exist in database 'shop'","cause":"no row in the catalogue matches schema 'shop' and table 'ordrs'","hint":"tpl -d shop schema tables","did_you_mean":["orders"]}}
  ```

- **FR-ERR-015**: `kind` SHALL be a stable, enumerated identifier meant to be
  compared programmatically. `message` is meant to be read.

- **FR-ERR-016**: Adding a value to `kind` SHALL NOT break the contract;
  renaming one SHALL.

- **FR-ERR-017**: Before invoking the parser, the system SHALL scan the raw
  argument vector for the exact forms `--format json` and `--format=json`,
  taking the first occurrence, with no inference of any kind. IF either is
  present, THEN every subsequent error, parse errors included, SHALL be emitted
  as JSON.

  *Rationale.* The parser does not hand over the value of `--format` when the
  parse itself fails, so without a pre-scan a caller asking for JSON would
  receive a text parse error.

- **FR-ERR-018**: The pre-scan SHALL NOT be extended to any other flag. In
  particular it SHALL NOT scan for `-q/--quiet`.

## Nearest-match suggestions

- **FR-ERR-019**: WHEN a supplied name does not exist, the system SHALL offer at
  most three suggestions, drawn from names within an edit distance of two,
  ordered by distance and then by name.

- **FR-ERR-020**: IF no candidate is within that distance, THEN the system SHALL
  omit the suggestion entirely rather than offer a poor one.

- **FR-ERR-021**: Suggestions SHALL apply to tables, views, routines, templates,
  database entries, commands, flags, and configuration keys.

## Safe hints

- **FR-ERR-022**: A `hint` that contains a runnable command SHALL be built only
  from literals and from names matching `[A-Za-z0-9_]{1,64}`.

- **FR-ERR-023**: IF a name falls outside that character set, THEN the system
  SHALL fall back to the generic hint with no executable suggestion, and the
  name SHALL appear only in the `did_you_mean` field of the JSON document, where
  it is data rather than a command.

  *Rationale.* The declared purpose of `hint` is that the caller copies it and
  runs it. A table name is free text in MariaDB and can contain semicolons,
  quotes, and newlines, so formatting one straight into a suggested command is
  command injection with the caller as the interpreter.

- **FR-ERR-024**: The system SHALL escape `\n`, `\r`, `\t`, and every C0 control
  character in every value it interpolates into a message — catalogue names,
  comments, defaults, `--context` values, and the argument vector. This
  requirement owns escaping in a diagnostic message; `FR-OUT-018` owns
  escaping in the output of a read command, where tab is excepted.

  *Rationale.* Escaping newlines separately protects the line-oriented
  `error:` / `cause:` / `hint:` / `exit:` format from having a whole diagnostic
  line forged. Tab is escaped here and excepted there because a `text` listing
  is laid out in aligned columns, per `FR-OUT-006`, and this format has no
  columns: a tab inside an interpolated catalogue name can only misalign the
  labels a caller reads on.

  *Amended in the third edition.* The ownership statement is new. The first
  edition had `FR-OUT-018` extend its own rule to "every diagnostic message"
  while excepting tab, which contradicted this requirement outright. The
  contradiction is resolved in favour of escaping tab in messages; `FR-OUT-018`
  no longer reaches them.

## `EPIPE`

- **FR-ERR-025**: WHEN stdout is closed by the consumer and no JSON document is
  mid-flight, the system SHALL terminate silently with exit `0`.

  *Rationale.* `tpl … | head -1` is not an error. In `text`, a cut listing is
  exactly what `head` asked for.

- **FR-ERR-026**: IF stdout is closed part-way through a JSON document, THEN the
  system SHALL exit `74`.

  *Rationale.* The consumer received truncated JSON and cannot tell that it is
  incomplete.

## Deadlines

- **FR-ERR-027**: WHEN a deadline is exceeded, the system SHALL exit with the
  code of the phase: `69` for DNS resolution, TCP connect, TLS handshake, or a
  catalogue query; `78` for `password_command`; `65` for render.

## Template codes

- **FR-ERR-028**: A missing template SHALL be `66`. A template syntax error
  SHALL be `65`. A render failure SHALL be `65`.

- **FR-ERR-029**: A malformed `--context` document SHALL be `65`.

## Business rules

- **BR-ERR-002**: The code alone must be enough to choose the next step. A
  message improves the caller's chance of self-correcting, but the code is what
  a program branches on.

- **BR-ERR-003**: No error message, at any verbosity, may disclose a credential,
  the resolved DSN, or the contents of `.tpl/.cfg`. See `FR-GLOB-018` and
  [security.md](security.md).

## Dependencies

- [cli-contract.md](cli-contract.md) — the parsing rules that produce `64`.
- [output-formats.md](output-formats.md) — the JSON rules the error document
  obeys, and the escaping rules `FR-ERR-024` restates for messages.
- [security.md](security.md) — hint construction and redaction as a
  cross-cutting concern.

## Open questions

- [OQ-011](open-questions.md#oq-011) — the behaviour of the pre-scan on
  ambiguous `--format` forms.
- [OQ-021](open-questions.md#oq-021) — the exit code for a failed cache write.
- [OQ-023](open-questions.md#oq-023) — the pre-scan on a command that does not
  declare `--format`.
