---
title: Errors and Exit Codes
status: approved
last-reviewed: 2026-09-18
related: [cli-contract.md, output-formats.md, security.md, global-flags.md, server-contract.md]
---

# Errors and Exit Codes

## Overview

The exit code is the most reliable channel a calling agent has for knowing what
happened. Every condition that ends an invocation in failure therefore carries
one of these codes, two conditions share one only where the caller's next step
is the same, and the code alone must be enough to choose the next step. This
file defines the codes, the order in which conditions are evaluated, the shape
of an error message, and the rules that keep a suggested command safe to run.

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
  | `64` | `EX_USAGE` | Unknown command or flag, missing required argument, mutually exclusive flags, malformed flag value, unknown configuration key, per `FR-ERR-035` | Fix the invocation; consult `--help` |
  | `65` | `EX_DATAERR` | Template syntax error, render failure, malformed `--context` document, render deadline exceeded, template path escaping the root | Fix the template or the context |
  | `66` | `EX_NOINPUT` | A named object does not exist: table, view, routine, template, database entry, configuration key, per `FR-ERR-035` | List what exists and choose another name |
  | `69` | `EX_UNAVAILABLE` | Server unreachable: DNS, connection refused, network deadline exceeded, TLS failure | Check host and network; the operation is read-only and therefore repeatable |
  | `70` | `EX_SOFTWARE` | Internal error — a defect in `tpl` | Report it; not fixable by the caller |
  | `73` | `EX_CANTCREAT` | `tpl init` cannot create `.tpl`, or `.tpl` already exists at the destination | Check permissions, or choose another destination |
  | `74` | `EX_IOERR` | I/O failure reading `.tpl`, or writing to stdout, including a pipe closed mid-document in JSON | Check permissions and free space |
  | `77` | `EX_NOPERM` | Authentication refused, or insufficient privileges on the catalogue | Fix the credentials, or request read access |
  | `78` | `EX_CONFIG` | No `.tpl` found; unsafe `.cfg` ownership or mode; malformed `.cfg`; a key outside the enumerated space, per `FR-ERR-035`; `password_command` not an array; invalid entry; a DSN query parameter; undefined `${VAR}`; `password_command` deadline exceeded, output cap exceeded, or non-zero exit; read-only session could not be enforced; no database entry selected; the server is not MariaDB; the server series is not supported | Fix `.tpl/.cfg`, or run `tpl init` |

  The codes are a closed set and the *Condition* column is not. *And no
  others* above fixes the list of codes; each *Condition* cell characterises
  the class of fault its code reports, so that the table alone tells a caller
  what a code means, and it does not enumerate the conditions that produce
  that code. A condition is therefore neither absent, nor misfiled, nor
  ungoverned because no cell names it: every condition is stated by the
  requirement that owns the behaviour, in the module that owns it, and
  `FR-ERR-002` obliges that requirement to carry a code of this table. The
  producing conditions of a code are read in the modules the file index of the
  [README](README.md#file-index) lists, and are never assembled from this
  column.

  *Amended in the fifth edition.* The `78` row gains five conditions, all of
  them from decisions written into
  [configuration-model.md](configuration-model.md): an unrecognised key
  (`FR-CONF-034`), a `password_command` that is not an array (`FR-CONF-035`), a
  DSN query parameter (`FR-CONF-011`), the `password_command` output cap
  (`FR-CONF-031`), and a non-zero exit from that child (`FR-CONF-033`). The
  code is unchanged for all five: each is the configuration failing to describe
  a usable connection, which is what `78` means.

  *Amended in the nineteenth edition: the table says which of its two columns
  is exhaustive.* The paragraph above is new, and nothing about `tpl` changes —
  the ten codes, their names, their conditions and the caller's next step are
  as the fifth edition left them. What changed is that the table now says how
  it is to be read. Deriving an error type from this file forced a reading the
  file never states: whether a condition no cell names is a condition this
  specification does not have. The two answers build different programs — one
  models the cells and treats anything else as a defect in this corpus, the
  other models the requirements and treats the cells as headings — and the
  second is right. Two requirements elsewhere had already read the table that
  way without the table saying so: `FR-CFG-043`'s rationale declines a fourth
  outcome because "`FR-ERR-001` fixes the code set", which closes the codes and
  not the conditions, and `FR-OUT-033`'s rationale reads the `66` cell as what
  that code is *reserved for* rather than as a list of what reaches it.

  *Observed, and bounded.* Six conditions in force carry a code of this table
  and appear in no cell of it: a single-value flag given twice (`FR-CLI-014`),
  a flag value beginning with `-` supplied as a separate token (`FR-CLI-018`),
  a bare routine name matching both a procedure and a function (`FR-SCH-010`),
  `tpl cfg database add` naming an entry that already exists (`FR-CFG-017`),
  and an unresolved segment of a help path (`FR-HELP-028`), all `64`; and an
  unclosed `${VAR` (`FR-CONF-021`), which is `78` where the cell names only an
  undefined `${VAR}`. They were found by one reading of this file, made to
  derive an error type from it, and not by a sweep, so they are the examples
  that reading produced and not the remainder. The scale of the remainder was
  measured instead, on 2026-09-12, by a textual sweep of the twenty module
  files of the file index: one hundred and twelve requirements outside this
  file name a code of this table. That figure bounds nothing exactly — a
  requirement may state two conditions, as `FR-SCH-010` does, and may name a
  code in a rationale without producing it — and it does not need to. It is
  far past what a table cell can carry, which is the whole of what it is
  needed for.

  *Rejected.* Closing the *Condition* column, by enumerating in each cell every
  condition that produces its code. It would put each of those requirements in
  two places, and the second copy is the one nobody edits — the fifth
  validation rule of the [README](README.md#maintenance-debt) was written for
  exactly that decay in a register, and a register at least has a target it can
  be re-read against, where a column of this table would have a hundred and
  twelve. Also rejected: leaving the column unstated and correcting the six
  cells found to be short. That answers this reading and not the next one: the
  six are the six one reader happened to need, and a column complete only
  against the last implementer to check it is the state this amendment exists
  to end.

- **FR-ERR-002**: Every condition that ends an invocation in failure SHALL
  carry exactly one code of `FR-ERR-001`, named by the requirement that owns
  the condition. The system SHALL NOT collapse two conditions onto one code
  where the caller's next step would differ.

  *Amended in the twentieth edition: the first sentence states an obligation
  ten codes can satisfy.* It read "Each distinct condition SHALL have its own
  code", which a closed set of ten codes cannot meet and which this corpus has
  never met. `FR-ERR-006` routes whole classes of condition onto one code by
  design — every parsing fault to `64`, every name that does not resolve to
  `66` — and the nineteenth edition measured one hundred and twelve
  requirements outside this file naming a code of the table. Read literally,
  the sentence made this corpus self-contradictory at every one of them, and
  made `FR-ERR-035` — which routes one configuration key to three codes
  because three next steps differ — a violation rather than an application of
  it. The obligation it was reaching for is the second sentence, which is
  unchanged and undiminished: a code is shared only where the caller's next
  step is the same. The first sentence now carries the other obligation this
  corpus rests on, and which `FR-ERR-001` cites this requirement for — that no
  condition is ungoverned, and that its code is stated where its behaviour is.

  *Rejected.* Deleting the first sentence and leaving the prohibition alone.
  `FR-ERR-001` cites this requirement for the rule that every condition carries
  a code of its table, which is what makes its *Condition* column safe to read
  as a characterisation; with the sentence gone, that citation resolves to
  nothing and the nineteenth edition's amendment loses the support it names.
  Also rejected: reading "distinct" as "distinct in the caller's next step",
  which makes the first sentence true only by making it the second one, and
  leaves every reader to discover for themselves that the two sentences are
  one.

- **FR-ERR-003**: `73` SHALL be produced only by `tpl init`.

  *Rationale.* With no output flags, `tpl` creates no destination other than
  `.tpl`.

- **FR-ERR-030**: `70` (`EX_SOFTWARE`) SHALL be produced by exactly two
  conditions, and by no other: a panic in the process, and a violated internal
  invariant the system detects and declines to continue past. WHEN a panic
  occurs, the system SHALL write the message `FR-ERR-032` requires and SHALL
  terminate the process with exit `70`.

  *Rationale.* It was the only code in the table of `FR-ERR-001` for which no
  requirement stated a producing condition, so nothing could confirm that the
  binary is able to return it at all.

  *Amended in the ninth edition: the first condition states an outcome, and no
  longer a mechanism.* It read "a panic **caught** at the top level of the
  process", which obliged the process to resume execution at a frame above the
  panic site. That is one way to reach the outcome and not the only one, and it
  is not one the release profile recorded outside this corpus admits at all:
  under a profile that terminates on a panic, no frame above the panic site
  runs and nothing is caught — the defect `DIV-045` recorded. What this corpus
  makes contract is the outcome a caller observes — the four labelled lines of
  `FR-ERR-008` and the code — and a process can produce both from the panic
  site itself, before the panic ends it. The condition therefore survives with
  its wording changed rather than being dropped: `70` keeps the two producing
  conditions it had, both exist in the distributed binary, and the code table
  of `FR-ERR-001` is unchanged. Which mechanism produces the outcome is an
  architecture decision, and this corpus names none, per the boundary the
  [README](README.md#still-out-of-scope) draws.

  *Rejected.* Dropping the first condition and stating the resulting limit in
  its own text — that a defect depriving the process of control is not reported
  as `70` and carries no message — which is the form `DIV-045` anticipated. The
  limit is not real: the outcome is obtainable under the profile as it stands,
  and a corpus that records a limit it does not have would send a caller
  branching away from a code the binary does return. Also rejected: leaving the
  requirement as written and obliging the profile to yield. The profile is not
  this corpus's to set, and what the requirement exists to guarantee — the
  caller receives the code and the message — is met without changing it.

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

## Which code a configuration key produces

- **FR-ERR-035**: Three codes of `FR-ERR-001` name a configuration key, and
  which of the three applies follows from where the key was met and what was
  asked of it, not from the key itself. The system SHALL produce:

  | Where the key was met | Code | Stated by |
  |---|---|---|
  | Named on the invocation of `tpl cfg set`, and outside the enumerated key space of `FR-CONF-002` | `64` | `FR-CFG-009`, and `FR-CFG-010` where the key is in that space and the value does not conform to the type declared for it |
  | Named on the invocation of `tpl cfg get`, or of `tpl cfg unset` as a key or as a block, and absent from `.tpl/.cfg` | `66` | `FR-CFG-007`, `FR-CFG-012` |
  | Carried by `.tpl/.cfg`, anywhere in the file, and outside the enumerated key space of `FR-CONF-002` | `78` | `FR-CONF-034` |

  The three cannot collide, and the order of `FR-ERR-006` is why. The file is
  read and validated at step 3, and no `cfg` subcommand is among the commands
  `FR-PROJ-025` excuses from it, so a `.tpl/.cfg` carrying a key outside the
  key space is refused with `78` before any command resolves a key of its own
  at step 4 or later. A key that reaches a `cfg` subcommand is therefore met in
  a file that carries only keys the space admits, and the remaining question is
  the one its own command asks: whether the key is in the space, for `set`, or
  whether it is in this file, for `get` and `unset`. One consequence is worth
  stating because it is easy to implement backwards: `tpl cfg set` given a key
  outside the space, against a file that already carries one, exits `78` and
  not `64`, per `FR-ERR-007`.

  *Rationale.* The three are three faults with three next steps, which is what
  `FR-ERR-002` requires three codes for. A key no space admits is a token the
  caller wrote and can rewrite, so `64` sends them to `--help`. A key the file
  does not carry is a named object that does not exist, exactly as a missing
  table is; `FR-ERR-005` reaches the same answer for a database entry on the
  same ground, and `FR-ERR-034` already names the key space of `FR-CONF-002` as
  one of the populations a `66` `cause` reports a name missing from. A key the
  file carries and the space refuses is the configuration failing to describe a
  usable connection, which is what `78` means and what the fifth edition's note
  under `FR-ERR-001` says of the five conditions it added there.

  *This requirement routes; it does not restate.* Each of the three conditions
  belongs to the module that owns the command or the file, and the three
  requirements cited keep their own wording, their suggestions and their hints.
  What this file owes a reader is the answer to the question its own table
  raises — three cells name a configuration key and no cell says which — and
  that answer is a routing table, in the terms `FR-ERR-001` now states for the
  whole column.

  *Rejected.* Carrying the discriminator in the three cells of `FR-ERR-001`.
  Those cells would become the only ones in the table that enumerate, against
  what that requirement now says of the column, and the routing would live in
  three places that are edited apart. Also rejected: leaving the separation to
  be recovered from [cfg-commands.md](cfg-commands.md) and
  [configuration-model.md](configuration-model.md). It is recoverable there,
  and only there — separating the three from this file alone was attempted and
  could not be done, which is the reading that produced this edition. A caller
  branches on an exit code and reads this file to know what one means; a file
  that names a configuration key in three cells and separates them in none
  of the three sends that reader to two other files to find out which it got.

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

  *Checked in the twenty-fourth edition, and unchanged.* This requirement
  orders **conditions**, and it is the condition order that governs. The order
  of the statements that produce the evidence for the three named above is
  `FR-SRV-042`, in [server-contract.md](server-contract.md), which derives it
  from this requirement and states it once: a condition cannot be evaluated
  before the statement that produces its evidence, so the order above places
  the read-only session statement and its read-back before the version probe.
  The two were in conflict until that requirement was written, because
  `FR-SRV-012` read an order out of a table of `FR-SRV-006` that states none,
  and that reading put the probe first. Nothing here changes: the eight steps,
  the two that are skipped, and the ordering among the three conditions of step
  5 are as the fourth edition left them.

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
  the `error` line. A `cause` restates the `error` line when it adds nothing to
  it. WHERE a row of `FR-ERR-034` obliges the `cause` to name a fact the
  `error` line also carries, the row governs and the `cause` SHALL name it.

  *Amended in the twentieth edition: what "restate" prohibits is stated,
  because two requirements in force pulled against each other on `65`.* The
  `65` row of `FR-ERR-034` obliges the `cause` of a template failure to name
  the template, the line, the column and the chain of engine errors;
  `FR-ERR-011` obliges a template error to carry all four and does not say on
  which line, so a message that names the template and its position on the
  `error` line — which is where `FR-ERR-008` puts what failed — repeats them on
  the `cause` line to satisfy the row. Read as forbidding overlap, this
  requirement subtracts from that row; read as forbidding a `cause` that adds
  nothing, the two compose and the `65` cause carries all four. The second
  reading is the one this corpus already uses: the example of `FR-ERR-008`
  names `'ordrs'` and `'shop'` on the `error` line and names both again on the
  `cause` line, and that example is this file's own model of a correct message.
  What its `cause` adds is the population searched, which is what the `66` row
  obliges; the overlap is how the reader knows the two lines are about the same
  thing.

  *Rejected.* Resolving it the other way, by excepting from a row of
  `FR-ERR-034` whatever the `error` line already carries. It makes a testable
  obligation untestable — the test would have to read both lines and know which
  of the four facts the wording of the first had used — and the wording of an
  individual message is out of this file's scope, so the content of a `cause`
  would come to depend on a thing this corpus does not fix. It also removes
  from the line that carries it the detail a caller needs most, on the path
  where `FR-ERR-033` has left the text as the only channel of detail there is.

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
  | `69` | The phase that failed — DNS resolution, TCP connect, TLS handshake, the version probe of `FR-SRV-002`, or a catalogue query — the host and port attempted, and what that phase returned |
  | `70` | The invariant that was violated, or that a panic occurred, and in either case where |
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

  *Amended in the ninth edition.* The `70` row said that a panic was "caught at
  the top level". The word goes with the same word in `FR-ERR-030`, and what
  the row obliges is otherwise unchanged: the fact — an invariant or a panic —
  and where it happened. "Where" is the location the condition arose at, not
  the text a panic carried.

  *Amended in the twenty-fifth edition: the `69` row names a fifth phase.* It
  named four, and one statement this system issues belonged to none of them.
  The version probe of `FR-SRV-002` is issued on an open session, after the
  three network phases have completed and before any catalogue statement, so a
  probe that fails because the session did not hold had no phase its `cause`
  could name — and naming one of the other four would be a `cause` that is
  false of the failure it reports, which the paragraph above bans. `FR-ERR-036`
  states the condition; this row states what its `cause` must carry. No code
  changes, and the other four phases are as the first edition left them.

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

- **FR-ERR-039**: The edit distance of `FR-ERR-019` SHALL be the **restricted**
  Damerau-Levenshtein distance — optimal string alignment — in which the
  insertion, the deletion and the substitution of one character, and the
  transposition of two **adjacent** characters, each cost one, and no substring
  is edited more than once.

  *The variant decides which candidates are offered, not merely how they are
  ranked.* `FR-ERR-019` admits a candidate by its distance, and the two forms
  of the Damerau-Levenshtein distance disagree inside the threshold that
  requirement fixes. They part company only where a further edit falls between
  the two transposed characters, and the canonical pair of that shape is `ca`
  against `abc`: **three** steps under the restricted form and **two** under
  the unrestricted one. Naming the family and not the member therefore admitted
  two conforming implementations that offer different candidates for the same
  invocation.

  *The property the transposition exists for holds under both forms*, and is
  why a plain Levenshtein distance is not the measure: `ordres` is one step
  from `orders`, where plain Levenshtein reports two, so the commonest typing
  slip of all stays inside the threshold.

  *Rejected: the unrestricted form, which admits the pair above at two.* What
  it buys is the candidates in which a caller transposed two characters **and**
  edited between them — two slips in one name, which a threshold of two is
  already at the edge of admitting. What it costs is that the distance can no
  longer be computed from a bounded window of the comparison: the unrestricted
  form reaches back to an arbitrary earlier position and holds the whole
  comparison, plus an index over the alphabet of both names. `BR-PERF-004`
  makes this a budgeted path — a `66` over `WL-001` compares against 200 names
  — and a wrong invocation is the invocation a calling agent makes most often
  while it is finding its way.

- **FR-ERR-038**: The system SHALL compare a supplied name against a candidate
  over the characters as written, and SHALL NOT fold case, of ASCII or of any
  other range, before measuring the distance of `FR-ERR-039`.

  **Where this corpus folds ASCII case it says so, and it does not say so
  here.** `FR-SCH-014` folds it for the `--pattern` filter and `FR-ENV-031`
  for the word-list tokeniser, each to widen what a comparison accepts, and
  `FR-SCH-008` folds it to **detect** a qualified prefix spelled in the wrong
  case and then refuses the token. None of the three reaches this path, and it
  is stated here so that a reader arriving from any of them is told once.

  *Accepted cost.* A name differing from the one that exists in more than two
  letters' case alone — `ORDER_ITEMS` against `order_items` — falls outside the
  threshold and is not offered. The caller receives the generic hint, which is
  the listing command, so the name is still recoverable in one further
  invocation — the same cost `FR-ERR-023` already accepts for a candidate its
  character set refuses.
  A single-letter slip — `Orders` against `orders` — is at distance one and is
  offered.

  *Rejected: folding ASCII case before measuring, as `FR-SCH-014` folds it for
  `--pattern`.* The two rules answer different questions. `--pattern` selects
  the set the caller asked for and shows everything it admits, so folding
  widens a listing the caller then reads; this is a ranking under a threshold,
  so folding changes which candidates are offered **at all** and in which order
  `FR-ERR-019` presents them. And it would place a name differing only in case
  at distance **zero** — the measure calling the candidate the supplied name,
  beneath an `error` line stating that the supplied name does not exist.
  `FR-ERR-010` reads an overlap between those two lines as the reader's signal
  that both are about the same thing; a distance of zero makes them disagree
  instead.

- **FR-ERR-020**: IF no candidate is within that distance, THEN the system SHALL
  omit the suggestion entirely rather than offer a poor one.

- **FR-ERR-021**: Suggestions SHALL apply to tables, views, routines, templates,
  database entries, commands, flags, and configuration keys.

- **FR-ERR-037**: WHERE a suggestion names more than one candidate, the system
  SHALL write all of them inside the one `did you mean` question of
  `FR-ERR-008`, each between single quotation marks, separating every pair but
  the last with `, ` and the last pair with ` or `. They SHALL appear in the
  order `FR-ERR-019` fixes.

  ```
  hint:  did you mean 'orders'? list the available tables with: tpl -d shop schema tables
  hint:  did you mean 'aorders' or 'orderz'? list the available tables with: tpl -d shop schema tables
  hint:  did you mean 'aorders', 'orderz' or 'border'? list the available tables with: tpl -d shop schema tables
  ```

  `FR-ERR-008` shows one candidate and `FR-ERR-019` admits three, and between
  them nothing said how two or three are written. The three lines above are one
  sentence at the three cardinalities that requirement admits.

  *Rejected: separating every pair with `, `, the last included.* It reads as
  an enumeration where what is meant is a choice, and the caller must take
  exactly one of the three; ` or ` is the word that says so, on the line
  `FR-ERR-009` makes the one they act on.

  *Rejected: one line per candidate.* `FR-ERR-008` fixes the message at four
  labelled lines, and `FR-ERR-024` escapes the newline in every interpolated
  value precisely so that no value can forge a fifth. A system that emits one
  itself spends that guarantee on formatting.

## Safe hints

- **FR-ERR-022**: A `hint` that contains a runnable command SHALL be built only
  from literals and from names matching `[A-Za-z0-9_]{1,64}`. A spelling this
  specification enumerates is a literal of this requirement: a command or alias
  of the command tree of [cli-contract.md](cli-contract.md), a flag a node
  declares, and a key of the enumerated space of `FR-CONF-002`. Every other
  value is subject to the character set, whatever its source; the values this
  specification names are a table, a view, a routine, a template, a database
  entry, the `<name>` segment of a `database.<name>` key, and the name of an
  environment variable. Each such value SHALL be tested on its own, and the
  separators that join names into a command or into a key are literals — the
  space between command-path segments, the `-` or `--` that introduces a flag,
  and the `.` between key segments.

  *Amended in the twentieth edition: the requirement says what the character
  set governs, because as written it governed everything and admitted two of
  the eight populations of `FR-ERR-021` nowhere.* No key of `FR-CONF-002`
  matches `[A-Za-z0-9_]{1,64}`, because all fifteen key forms contain a dot;
  five flags of this corpus carry a hyphen inside the name — `--tpl-dir`,
  `--no-cache`, `--ca-file`, `--ca-path` and `--password-command`; and
  `FR-ERR-023` drops a candidate outside the set in every form, prose included.
  Those two populations could therefore never be suggested at all, and a third
  — commands — survived only because no command or alias happens to carry a
  character outside the set. Those three are exactly the populations this
  specification enumerates itself, which is what makes them literals: a server,
  a file and a caller can each influence a table name, a template name and an
  entry name, and none of them can influence the spelling of
  `cfg database add`, of `--ca-file`, or of `core.render_timeout`. The
  character set was never a bound on what a hint may say; it is the test
  applied to a value this corpus does not fix, which is the whole of what the
  rationale of `FR-ERR-023` argues.

  *Rejected.* Widening the character set, to admit the dot, or the dot and the
  hyphen. It answers this reading and not the next one — the next enumerated
  spelling carrying a character outside the set reopens it — and every widening
  is paid for by every untrusted name, which is the population the set exists
  to bound. Also rejected: removing configuration keys and flags from
  `FR-ERR-021`, which withdraws the suggestion where it is safest and most
  useful, over two closed spaces fixed in this corpus, and leaves the caller
  who mistypes `core.databse` with a generic hint for want of a dot.

  *Consequence.* A `database.<name>` key is admissible exactly as far as its
  entry name is: `FR-CONF-008` fixes no character set for that name, so
  `database.reporting.host` may be built into a runnable command and a key
  naming an entry outside the set is dropped by `FR-ERR-023`, exactly as that
  entry name would be dropped as a candidate in its own right.

- **FR-ERR-023**: IF a nearest-match candidate is subject to that character set
  and falls outside it, THEN the system SHALL NOT present that candidate at all
  — neither as an executable suggestion nor as prose — and SHALL emit the
  generic hint alone.

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

  *Amended in the twentieth edition.* The condition read "falls outside that
  character set", of every candidate alike, which dropped every configuration
  key and every hyphenated flag this corpus enumerates. `FR-ERR-022` now says
  which candidates the set governs, and this rule drops those and no others;
  what it does to a candidate it governs is unchanged, and so is the ground
  for it.

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
  escaping in the output of a read command, where tab is excepted in `text`.

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

  *Read against `NFR-DET-004` in the twentieth edition, and unchanged.* The C0
  range covers `ESC` and therefore the two-character form of an ANSI escape
  sequence; it does not cover `U+009B`, the single-character CSI that some
  terminals honour, so an interpolated catalogue name carrying that byte
  reaches a reader as this rule leaves it. `NFR-DET-004` forbids an ANSI escape
  sequence "under any circumstances", and whether it reached a byte of the
  catalogue was the question: it does not, and the twentieth edition states so
  in that requirement's own text, because a reading that reached content would
  forbid `tpl render` the byte-for-byte output `FR-OUT-019` guarantees it.
  Nothing here widens on that account. Whether this rule should widen on its
  own ground — the ground being that a composed, line-oriented message is not
  pass-through — is a question about this requirement and `FR-OUT-018`
  together, and it is recorded as an item in the
  [README](README.md#maintenance-debt) rather than settled from one side of it.

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

## A session that opens and does not hold

- **FR-ERR-036**: IF a statement fails on a session that has opened, and the
  failure is neither a deadline under `FR-ERR-027` nor a condition another
  requirement routes elsewhere, THEN the system SHALL exit `69`
  (`EX_UNAVAILABLE`), and the `cause` line SHALL name the phase the statement
  belongs to — the version probe of `FR-SRV-002`, or the catalogue query — per
  the `69` row of `FR-ERR-034`. It SHALL NOT name DNS resolution, TCP connect,
  or the TLS handshake, each of which completed before the session opened.

  The conditions other requirements route elsewhere are the three of step 5 of
  `FR-ERR-006`, and all three are `78`: the read-only session statement and its
  read-back, under `FR-SRV-010`, which is `78` whether the server refused the
  statement or the session did not survive it, because a setting that cannot be
  applied is a setting that cannot be applied; the product check of
  `FR-SRV-003`; and the version-window check of `FR-SRV-020`. What is left for
  this requirement is therefore the other two statements of `FR-SRV-006` — the
  version probe, whose own verdicts `FR-SRV-003` and `FR-SRV-020` reach only
  when the probe answered, and the catalogue read.

  *Rationale.* `69` is right and was never in question — the session did not
  hold, the server is unreachable for this invocation, and the caller's next
  step is the one that row of `FR-ERR-001` states, which the read-only promise
  of `FR-SRV-006` makes safe to take: the operation is read-only and therefore
  repeatable. What was missing is the phase. A `cause` that named the TCP
  connect for a session that had already connected, authenticated, been set
  read only and been probed sends a caller to check whether the server is
  listening, and it is; and it is a wording equally true of a different
  failure, which `FR-ERR-034` bans in terms.

  *Added in the twenty-fifth edition.* The four conditions of `69` this corpus
  stated were a name that did not resolve, a connection that was refused, a
  TLS handshake that failed, and a deadline. A session that opens and then
  stops answering — a connection dropped mid-statement, a protocol fault — is
  none of the four, and it was reported as the second of them, which is the
  right code under a `cause` line that is false.

  *Rejected: a tenth code for a session that did not hold.* `FR-ERR-001`
  closes the code set in terms, and the caller's next step here is the one
  every other `69` carries. `FR-ERR-002` admits two conditions on one code
  exactly where that holds, and obliges the `cause` to separate them for a
  reader — which is what this requirement does.

  *Rejected: leaving the condition unstated and the phase to the
  implementation.* It is how the defect arose: with no requirement naming the
  phase, the nearest condition in force was the refused connection, and its
  `cause` was emitted for a failure it does not describe. A phase a caller
  reads is contract under `FR-ERR-034`, and this corpus states it rather than
  letting the choice of a nearest neighbour decide it.

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
