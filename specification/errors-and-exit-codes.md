---
title: Errors and Exit Codes
status: approved
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

The exit code is also the *only* machine-comparable channel. No error is ever
emitted as JSON, per `FR-ERR-033`, so the four-line text message is an error's
only channel of detail — which is why `FR-ERR-034` states, per code, what the
`cause` line is obliged to name.

## Scope

In scope: the exit code table, validation order, the message format and what
each code's `cause` line must name, nearest-match suggestions, hint
construction, and `EPIPE`.

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
  | `78` | `EX_CONFIG` | No `.tpl` found; unsafe `.cfg` ownership or mode; malformed `.cfg`; a key outside the enumerated space; `password_command` not an array; invalid entry; a DSN query parameter; undefined `${VAR}`; `password_command` deadline exceeded, output cap exceeded, or non-zero exit; read-only session could not be enforced; no database entry selected; the server is not MariaDB; the server series is not supported | Fix `.tpl/.cfg`, or run `tpl init` |

  *Amended in the fifth edition.* The `78` row gains five conditions, all of
  them from decisions written into
  [configuration-model.md](configuration-model.md): an unrecognised key
  (`FR-CONF-034`), a `password_command` that is not an array (`FR-CONF-035`), a
  DSN query parameter (`FR-CONF-011`), the `password_command` output cap
  (`FR-CONF-031`), and a non-zero exit from that child (`FR-CONF-033`). The
  code is unchanged for all five: each is the configuration failing to describe
  a usable connection, which is what `78` means.

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

  *A consequence that reaches outside this corpus, recorded in the eighth
  edition.* The first condition requires the process to retain control after a
  panic and to report it with the message `FR-ERR-032` fixes. The release
  profile the root `CLAUDE.md` states aborts on panic, and under an aborting
  profile a panic terminates the process abnormally: the caller receives no
  code of `FR-ERR-001` and no message, so one of the two producing conditions
  of a code this corpus makes contract does not exist in the distributed
  binary. This requirement is unchanged, because the specification precedes
  the implementation; the correction owed to that file is `DIV-045`.

- **FR-ERR-031**: The system SHALL provide a deliberate trigger for `70`, so
  that the test `BR-ERR-001` mandates for it can exist. The trigger SHALL be
  reachable only from within the system's own test configuration, SHALL NOT be
  reachable from any invocation of the binary the project distributes, and
  SHALL NOT appear in any help text, in the JSON command tree of
  `FR-HELP-016`, or in the command tree of `FR-CLI-002`.

  *Rationale.* A code with no test is a code nobody has confirmed the binary
  can return, and `70` cannot be reached from a correct invocation by
  definition. Keeping the trigger out of the published surface keeps the tree
  closed, per `FR-CLI-002`, and keeps it out of the caller's context window.

  *Amended in the eighth edition: the mechanism is named, because every
  mechanism reachable from outside the process collides with a requirement in
  force.* The first form of this requirement said where the trigger must not
  appear and left open what it is. Each of the three candidates fails against
  a requirement this corpus holds:

  | Candidate | The requirement it collides with |
  |---|---|
  | A command or a flag, hidden or declared | `FR-CLI-002` closes the tree, and `FR-HELP-021` derives the JSON tree by introspecting the very tree the parser accepts — so a node the parser accepts is in the document this requirement bars it from |
  | An environment variable | `FR-CLI-021` reads none, `FR-CLI-023` names the one exception, and `NFR-DET-001` promises byte-identical stdout for one invocation against one state |
  | A build selected by a feature | The artefact verified would not be the artefact distributed, and `NFR-PERF-018` makes every distributed artefact first class |

  The trigger is therefore inside the process and reachable from nothing a
  caller can write. Nothing on the published surface changes, and none of the
  four requirements above yields.

  *The exception, stated against the requirement it excepts from.*
  `BR-ERR-001` requires an **integration** test per exit code. `70` alone is
  excepted, and `BR-ERR-001` says so in its own text. What replaces it is a
  composition rather than a lesser test: the guard that detects a violated
  internal invariant is exercised in process and observed to produce the
  condition of `FR-ERR-030` carrying the message `FR-ERR-032` requires, and
  the step from that condition to the process exit status is the same step
  every other code of `FR-ERR-001` travels — and each of those nine does have
  the integration test `BR-ERR-001` mandates.

  *Consequence, stated plainly.* No invocation of the distributed binary is
  observed returning `70`. What is observed is the guard, and separately the
  step from an error condition to an exit status; the composition of the two
  is reasoned rather than executed. That is weaker than the nine other rows of
  `FR-ERR-001`, and it is the price of the trigger being unreachable — which
  is the property `FR-ERR-030` needs, because a `70` a caller could provoke
  would not be a defect in `tpl`.

  *Rejected.* Dropping the trigger and leaving `70` with no test of any kind,
  which is the state the third edition closed in `OQ-067` and would restore.
  Also rejected: provoking the invariant from an input a caller controls — a
  hand-written cache document, a `--context` document, a `.cfg` value — which
  would make `70` reachable from bad input and would then tell the caller that
  a file they can delete is a defect they cannot fix. That contradicts
  `FR-ERR-032` and diagnoses the wrong fault, which is the error
  `FR-CONF-033` rejects in the same shape.

- **FR-ERR-032**: A `70` SHALL follow the message format of `FR-ERR-008`, and
  its `hint` SHALL say that the condition is a defect in `tpl` and is not
  correctable by the caller.

- **BR-ERR-001**: Exit codes are contract. Each code SHALL have at least one
  integration test that exercises it, and that test is part of the definition of
  done for the feature that can produce it. `70` is the one exception, stated
  here rather than left to be inferred: neither of its producing conditions in
  `FR-ERR-030` can be reached deliberately from an invocation of the
  distributed binary, so it is exercised in process through the trigger of
  `FR-ERR-031`, and not by an integration test.

  *Amended in the eighth edition.* The rule required an integration test for
  every code while also saying that `70` "is exercised through the trigger of
  `FR-ERR-031`", and no trigger satisfying both existed: every mechanism
  reachable from outside the process collides with `FR-CLI-002`,
  `FR-CLI-021`, `FR-HELP-021` or `NFR-PERF-018`, which `FR-ERR-031` now
  records candidate by candidate. This rule yields, for `70` alone. The other
  nine codes are unchanged, and the composition that stands in for the missing
  test — together with what it does not establish — is stated in
  `FR-ERR-031`.

## No database entry versus a missing entry

- **FR-ERR-004**: IF no database entry is selected and the command requires one,
  THEN the system SHALL exit `78`.

- **FR-ERR-005**: IF a named database entry does not exist, THEN the system
  SHALL exit `66` with a nearest-match suggestion.

  *Rationale.* Nothing selected is a configuration problem, pointing at the
  file. A name that does not resolve is a named object that does not exist, like
  a missing table. Folding both into `78` would make one code cover six distinct
  conditions and leave the real distinction only in the `cause` line, which
  `BR-ERR-002` says a program does not branch on.

  *Amended in the fifth edition.* The last clause said "only in the `kind`
  field". `FR-ERR-015` withdraws that field; the argument is unchanged and now
  names the line that actually carries the distinction.

## Validation order

- **FR-ERR-006**: The system SHALL evaluate conditions in the following fixed
  order, and SHALL report the first that fails:

  ```
  1. argument parsing                          64
  2. .tpl discovery and trust checks           78
  3. .cfg read and validation                  78
  4. database entry resolution                 78 or 66
  5. cache or connection                       69, 77, or 78
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

  *Amended in the fourth edition.* Step 5 gains `78`. Three conditions are
  decided once a connection is open and before any catalogue read, and all three
  are configuration faults rather than availability ones: the read-only session
  of `FR-SRV-010`, which the first edition already routed to `78` without the
  order saying where; the product check of `FR-SRV-003`; and the version-window
  check of `FR-SRV-020`. They are evaluated in that order among themselves, so
  the strongest guarantee is confirmed before the server is characterised.

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

- **FR-ERR-034**: The `cause` line SHALL state the specific fact that failed,
  and SHALL name an instance wherever one is available rather than the category
  the instance belongs to. The system SHALL NOT emit a `cause` whose wording
  would be equally true of a different failure. Each code of `FR-ERR-001` that
  can carry a message obliges its `cause` to name at least the following:

  | Code | The `cause` line SHALL name |
  |---|---|
  | `64` | The token rejected as written, and why it was rejected: the unknown command or flag, the value that did not conform together with the type expected, or both members of the mutually exclusive pair |
  | `65` | For a template, the template name, the line, the column, and the chain of underlying engine errors, per `FR-ERR-011`. For a `--context` document, the path and either the position of the malformed JSON or the structural rule of [context-document.md](context-document.md) it failed. For a deadline, which deadline expired and its resolved value, per `FR-GLOB-012` |
  | `66` | The identifier that was not found, the kind of object it was sought as, and the population it was sought in — the database entry and the server-side database, the template root, or the key space of `FR-CONF-002` |
  | `69` | The phase that failed — DNS resolution, TCP connect, TLS handshake, or catalogue query — the host and port attempted, and what that phase returned |
  | `70` | The invariant that was violated, or that a panic was caught at the top level, and where |
  | `73` | The path `tpl init` could not create, and whether the obstacle was an existing `.tpl` or a failure the filesystem reported |
  | `74` | The path or stream that failed, the operation attempted on it, and what the filesystem or the stream returned |
  | `77` | For authentication, the user and the host the server refused, and that the refusal came from the server. For privileges, which property of which object could not be read, per `FR-PRIV-013` |
  | `78` | The key and the file, with the value found and the value expected; or, where the fault is not a key, the specific condition — the directory the walk ended at without finding `.tpl`, the name of the undefined variable, or the series found and why it is not supported, per `FR-SRV-030` |

  `0` is the tenth code of `FR-ERR-001` and produces no message.

  *Rationale.* The exit code says which class of thing went wrong; the `cause`
  line is now the only place the instance can be named, because `FR-ERR-033`
  withdraws the JSON error document. Stating the obligation per code is what
  makes it testable: a `cause` can be checked against the row for the code the
  invocation returned, and the check fails on any wording that omits what the
  row names.

  *Banned.* A `cause` that would read identically for a different failure —
  "the configuration is invalid", "the server rejected the statement", "the
  template could not be rendered". Each names a category where an instance was
  available, and each is what `FR-ERR-012` already forbids in the `hint` line,
  applied to the line that carries the fact.

  *Composition.* This requirement raises what a `cause` must contain and
  weakens nothing. `FR-ERR-022` and `FR-ERR-023` still govern what may be built
  into an executable suggestion, `FR-ERR-024` still escapes every interpolated
  value, and `FR-ERR-013` and `FR-SEC-005` still bar every credential.
  Precision never licenses echoing a secret, and never licenses reproducing a
  name `FR-ERR-023` refuses.

- **FR-ERR-011**: A template error SHALL carry the template name, the line, the
  column, and the chain of underlying template-engine errors.

- **FR-ERR-012**: The system SHALL NOT emit vague advice. It SHALL name the key,
  the file, and the expected value rather than suggesting that the caller check
  their configuration.

- **FR-ERR-013**: The system SHALL NOT include credentials in any error message,
  at any verbosity level.

## Errors are never JSON

- **FR-ERR-033**: The system SHALL NOT emit an error as a JSON document, from
  any command, at any verbosity level, under any value of `--format`. WHEN the
  outcome of an invocation is an error, the system SHALL ignore `--format`,
  SHALL write the four labelled lines of `FR-ERR-008` to stderr, and SHALL
  leave stdout empty. Only a successful result may be JSON-formatted.

  *Rationale.* Two channels carried the same information and only one of them
  was guaranteed to arrive. `--format json` is the plumbing contract for a
  *result*, and a failure has no result: the caller's machine-readable answer
  is the exit code, which is present on every path, needs no parsing, and
  cannot be truncated. A second machine-readable channel that appeared only on
  stderr, only when the parse had got far enough for the flag to be seen, and
  only for the commands that declare that flag, is a contract with three
  preconditions — and each of the three had already been recorded in this
  corpus as an open question.

  *Rejected.* Emitting the error envelope only where `--format json` was
  successfully parsed, which is the first edition's rule and makes the shape of
  a diagnostic depend on how far the parse got. Also rejected: emitting every
  error as JSON unconditionally, which would answer a person in JSON when
  `tpl init` cannot create a directory.

  *Accepted cost.* A calling agent that wants more than the exit code must read
  the `cause` and `hint` lines as text. `FR-ERR-034` raises what those lines
  are obliged to contain, precisely because the text is now an error's only
  channel of detail.

- **FR-ERR-014**: *Withdrawn in the fifth edition.* This requirement fixed the
  JSON error document. `FR-ERR-033` withdraws it: no error is emitted as JSON.
  The identifier is retired and SHALL NOT be reused.

- **FR-ERR-015**: *Withdrawn in the fifth edition.* This requirement made
  `kind` a stable enumerated identifier meant to be compared programmatically.
  Its only carrier was the document of `FR-ERR-014`, and with that document
  withdrawn the field has nowhere to appear. The exit code is the sole
  machine-comparable signal, per `BR-ERR-002`; where two conditions share a
  code, `FR-ERR-034` obliges the `cause` line to separate them for a reader.
  The identifier is retired and SHALL NOT be reused.

  *Rejected.* Retaining `kind` as an internal taxonomy with no external
  carrier. A classification nothing outside the process can observe cannot be
  tested or demonstrated, which the [README](README.md#requirement-style) makes
  the test of whether something is a requirement at all. It would also leave
  every `kind: …` citation in this corpus reading as though a caller sees the
  value — the class of quietly-untrue statement this corpus names as its own
  characteristic defect.

- **FR-ERR-016**: *Withdrawn in the fifth edition.* The compatibility rule of
  `kind`, withdrawn with the field it governed. The identifier is retired and
  SHALL NOT be reused.

- **FR-ERR-017**: *Withdrawn in the fifth edition.* The argument pre-scan
  existed only to decide whether an error would be emitted as JSON.
  `FR-ERR-033` settles that question for every invocation before any argument
  is read, so the mechanism has nothing left to decide. The identifier is
  retired and SHALL NOT be reused.

  *Consequence.* `OQ-011` asked how the pre-scan behaves on an ambiguous
  `--format` form, and `OQ-023` how it behaves on a command that declares no
  `--format`. Both are **dissolved** rather than answered: the mechanism they
  governed ceases to exist, and neither question can be put again.

- **FR-ERR-018**: *Withdrawn in the fifth edition.* The non-extension rule of
  the pre-scan, withdrawn with the pre-scan. The identifier is retired and
  SHALL NOT be reused.

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

- **FR-ERR-023**: IF a nearest-match candidate falls outside that character
  set, THEN the system SHALL NOT present that candidate at all — neither as an
  executable suggestion nor as prose — and SHALL emit the generic hint alone.

  *Rationale.* The declared purpose of `hint` is that the caller copies it and
  runs it. A table name is free text in MariaDB and can contain semicolons,
  quotes, and newlines, so formatting one straight into a suggested command is
  command injection with the caller as the interpreter.

  *Amended in the fifth edition.* The first edition routed such a name into the
  `did_you_mean` field of the JSON error document, "where it is data rather
  than a command". `FR-ERR-033` withdraws that document, so the escape hatch is
  gone and the name now goes nowhere. Only the machine-readable
  `did_you_mean` array is withdrawn; nearest-match suggestion itself survives
  in the text `hint` line, for every candidate the character set admits.

  *Accepted cost.* A caller that mistypes the name of an object whose real name
  contains a character outside `[A-Za-z0-9_]` receives no suggestion, only the
  generic hint. That hint is the listing command, so the name is still
  recoverable in one further invocation, where `FR-ERR-024` escapes it on the
  way out. The alternative — printing the candidate as prose beside a generic
  hint — was rejected because a caller that copies a whole `hint` line does not
  reliably distinguish its prose half from its command half, which is the
  assumption `FR-ERR-022` exists to avoid relying on.

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
- [output-formats.md](output-formats.md) — `FR-OUT-015` and `FR-OUT-032`,
  which record on the output side that no error is JSON, and the escaping rules
  `FR-ERR-024` restates for messages.
- [security.md](security.md) — hint construction and redaction as a
  cross-cutting concern.

## Open questions

None specific to this module. `OQ-011` and `OQ-023` are dissolved with the
pre-scan of `FR-ERR-017`, and `OQ-021` is answered by `FR-CACHE-036`; all three
are listed under [Closed](open-questions.md#closed).
