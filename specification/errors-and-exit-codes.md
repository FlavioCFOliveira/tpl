---
title: Errors and Exit Codes
status: approved
last-reviewed: 2026-09-24
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
  | `65` | `EX_DATAERR` | Template syntax error, render failure, malformed `--context` document, render deadline or render bound exceeded, template path escaping the root | Fix the template or the context |
  | `66` | `EX_NOINPUT` | A named object does not exist: table, view, routine, template, database entry, configuration key, per `FR-ERR-035` | List what exists and choose another name |
  | `69` | `EX_UNAVAILABLE` | Server unreachable: DNS, connection refused, network deadline exceeded, TLS failure | Check host and network; the operation is read-only and therefore repeatable |
  | `70` | `EX_SOFTWARE` | Internal error — a defect in `tpl` | Report it; not fixable by the caller |
  | `73` | `EX_CANTCREAT` | `tpl init` cannot create `.tpl`, or `.tpl` already exists at the destination | Check permissions, or choose another destination |
  | `74` | `EX_IOERR` | I/O failure on a file or stream the invocation reads or writes: `.tpl` and what it holds, a `--context` document, trust material named by `ca_file` or `ca_path`, or stdout, including a pipe closed part-way through a JSON document | Check permissions and free space |
  | `77` | `EX_NOPERM` | Authentication refused, or insufficient privileges on the catalogue | Fix the credentials, or request read access |
  | `78` | `EX_CONFIG` | No `.tpl` found, or a `--tpl-dir` that names no `.tpl` folder; unsafe `.cfg` ownership or mode, or a `.tpl` folder without `.cfg` owned by another user; malformed `.cfg`, including a `${VAR}` in `ca_file` or `ca_path`; a key outside the enumerated space, per `FR-ERR-035`; `password_command` not an array; invalid entry; a DSN query parameter; undefined `${VAR}`; `password_command` deadline exceeded, output cap exceeded, or non-zero exit; read-only session could not be enforced; no database entry selected; the server is not MariaDB; the server series is not supported | Fix `.tpl/.cfg`, or run `tpl init` |

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

  *Amended in the thirty-first edition: the `74` cell characterises the class
  its code reports, where it named one direction of it.* It read *I/O failure
  reading `.tpl`, or writing to stdout*, which excludes two conditions this
  corpus states under that code: `FR-CFG-041` rewrites `.tpl/.cfg` and exits
  `74` when the write fails, and `FR-RND-035` exits `74` when a `--context`
  file or stream cannot be read. Under the paragraph above a condition is
  neither absent nor misfiled because no cell names it, so neither condition
  was ungoverned; what was wrong is that the cell **characterised** the class
  as reading `.tpl` and writing to stdout, and the class is I/O on what the
  invocation reads or writes. The code set is unchanged, no other cell moves,
  and the column is no more of an enumeration than it was: the kinds of file
  and stream this system touches are what the cell names, not a list of the
  conditions that can fail on them.

  *Rejected: leaving the cell and relying on the paragraph above.* It is
  correct and it is not what the cell is for. A cell that characterises a class
  narrower than the class sends a reader looking for another code, which is the
  reading that put `74` in five commands' help before any requirement said so.

  *Amended in the thirty-third edition: the cell names a fourth kind, because
  the class reaches four.* The cell and the note above named three — `.tpl` and
  what it holds, a `--context` document, and stdout — and called them the kinds
  of file and stream this system touches. **Trust material is a fourth.**
  `FR-CONF-014` routes an entry of a `ca_path` the system cannot resolve, or
  cannot read at its target, to this code, and nothing in this corpus requires
  either key's path to lie inside `.tpl` — a conventional `CApath` is a system
  trust store, which is the arrangement that requirement exists to serve.
  Nothing was ungoverned while the count stood at three, because the paragraph
  above says a condition is neither absent nor misfiled because no cell names
  it, and no requirement rests on the count. What was wrong is the same thing the thirty-first edition corrected in
  this cell: a characterisation narrower than the class it characterises. The
  observation was recorded and deliberately not acted on by the edition that
  made it, on the ground that it had no reading to choose between; this edition
  has one.

  *Rejected: turning the cell back into an enumeration of the four kinds.* It
  is the reading the thirty-first edition refused and the nineteenth closed the
  column against, and a fourth member is no argument for reopening either. The
  cell characterises; four is not a list any more than three was, and the four
  are named so that a reader of the cell can tell whether a failure of theirs
  falls inside the class — which is what a characterisation is for. The count
  is not load-bearing and no requirement cites it, so a fifth kind moves this
  cell and nothing else.

  *Rejected: a code of its own for trust material.* `FR-ERR-002` forbids
  collapsing two conditions onto one code only where the caller's next step
  would differ, and it does not differ here: a certificate that cannot be read
  is answered by checking permissions on the path the configuration names,
  which is the step this row already prints. `78` was the near alternative,
  since the path comes from `.tpl/.cfg`, and the two codes divide on what
  failed: `FR-CONF-014` takes `74` where the reading of a file failed, and
  `FR-CONF-044` takes `78` where the directory named nothing to read, which is
  a fault in the configuration and not in the filesystem.

  *Amended in the forty-second edition: the `65` cell names the render bounds
  beside the deadline.* `FR-RND-036`, `FR-RND-037` and `FR-RND-039` end a
  render that exhausts its render fuel, reaches its render output limit or
  crosses its render memory limit, all with `65`.
  The cell named the deadline and would otherwise have characterised the class
  as bounded in time alone. The code set is unchanged and the column still
  characterises rather than enumerates.

  *Amended in the forty-eighth edition: the `78` cell names three conditions
  the class already held.* `FR-PROJ-027` refuses a `--tpl-dir` that names a
  directory which is not a `.tpl` folder, `FR-PROJ-028` refuses a `.tpl`
  folder without `.cfg` that another user owns, and `FR-CONF-047` refuses a
  `${VAR}` written into `ca_file` or `ca_path`. Each is the project or its
  configuration failing to be usable, which is what `78` means. The code set
  is unchanged and the column still characterises rather than enumerates.

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
  SHALL exit `66` with a nearest-match suggestion. The condition arises only on
  a command that requires an entry, per `FR-GLOB-025`; `FR-GLOB-007` owns the
  rule and states the outcome where the command requires none.

  *Rationale.* Nothing selected is a configuration problem, pointing at the
  file. A name that does not resolve is a named object that does not exist, like
  a missing table. Folding both into `78` would make one code cover six distinct
  conditions and leave the real distinction only in the `cause` line, which
  `BR-ERR-002` says a program does not branch on.

  *Amended in the fifth edition.* The last clause said "only in the `kind`
  field". `FR-ERR-015` withdraws that field; the argument is unchanged and now
  names the line that actually carries the distinction.

  *Amended in the thirty-first edition.* The second sentence of each of the two
  requirements above is a cross-reference and not a second rule.
  `FR-GLOB-006` has qualified its `78` with *and the command requires one*
  since the first edition and `FR-GLOB-007` carried no such qualifier, so read
  literally this pair said that `tpl -d nope cfg list` is `66` while
  `tpl cfg list` in a project with no `core.database` is `0`. `FR-GLOB-007` as
  amended qualifies the `66` in the same terms and `FR-GLOB-025` names the
  commands both qualifiers mean. Nothing here changes what a code means.

## Which code a configuration key produces

- **FR-ERR-035**: Three codes of `FR-ERR-001` name a configuration key, and
  which of the three applies follows from where the key was met and what was
  asked of it, not from the key itself. The system SHALL produce:

  | Where the key was met | Code | Stated by |
  |---|---|---|
  | Named on the invocation of `tpl cfg set`, and outside the enumerated key space of `FR-CONF-002` | `64` | `FR-CFG-009`, and `FR-CFG-010` where the key is in that space and the value does not conform to the type declared for it |
  | Named on the invocation of `tpl cfg get` in the form of a block — `core`, `database`, or `database.<name>` — rather than a key of `FR-CONF-002` | `64` | `FR-CFG-007` |
  | Named on the invocation of `tpl cfg get`, or of `tpl cfg unset` as a key or as a block, and absent from `.tpl/.cfg` | `66` | `FR-CFG-007`, `FR-CFG-012` |
  | Carried by `.tpl/.cfg`, anywhere in the file, and outside the enumerated key space of `FR-CONF-002` | `78` | `FR-CONF-034` |

  The three cannot collide, and the order of `FR-ERR-006` is why. The file is
  read and validated at step 3, and no `cfg` subcommand is among the commands
  `FR-PROJ-025` excuses from it, so a `.tpl/.cfg` carrying a key outside the
  key space is refused with `78` before any command resolves a key of its own
  at any later step. A key that reaches a `cfg` subcommand is therefore met in
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

  *Amended in the forty-third edition.* The row for a block given to
  `tpl cfg get` is new, and is decided by `FR-CFG-007`. It is evaluated before
  the question of presence, so a block form is `64` whether or not the file
  carries the block. It does not collide with the other rows: `tpl cfg unset`
  accepts a block, per `FR-CFG-011`, and `tpl cfg get` does not.

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
  4. template resolution                       66
  5. database entry resolution                 78 or 66
  6. cache or connection                       69, 77, or 78
  7. catalogue object resolution               66
  8. render                                    65
  ```

  Steps 2 and 3 SHALL be skipped for the commands `FR-PROJ-025` names, which
  require no project. Step 1 runs for every command without exception, so an
  unknown flag on `tpl --help` is still `64`.

  **A step whose condition an invocation does not raise is not a step the
  invocation skips.** Steps 2 and 3 are named above because they are skipped
  for four commands that would otherwise have a project to discover, which is
  an exemption. The other steps need none: step 4 is reached only by an
  invocation that names a template, step 5 only by a command `FR-GLOB-025`
  says requires a database entry, steps 6 and 7 only by one that reads the
  catalogue, and step 8 only by a render. This order fixes the sequence in
  which conditions are evaluated and does not assert that every invocation
  raises every condition.

  *Added in the thirty-first edition.* The paragraph states what was always
  meant and was readable the other way: naming two steps as skipped invited the
  contrast that the other six always run, and an implementer took it — the help
  of roughly thirty commands was written on a reading of step 5 that this
  requirement never stated. The reading was right, and it is now `FR-GLOB-007`
  and `FR-GLOB-025` that say so.

  *Amended in the third edition.* The sentence about steps 2 and 3 is new. The
  order alone did not say whether `--help` reached discovery; `FR-PROJ-025` now
  names the commands that skip them.

  *Amended in the fourth edition.* The cache-or-connection step gains `78`.
  Three conditions are decided once a connection is open and before any
  catalogue read, and all three are configuration faults rather than
  availability ones: the read-only session of `FR-SRV-010`, which the first
  edition already routed to `78` without the order saying where; the product
  check of `FR-SRV-003`; and the version-window check of `FR-SRV-020`. They are
  evaluated in that order among themselves, so the strongest guarantee is
  confirmed before the server is characterised.

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
  the two that are skipped, and the ordering among the three conditions of the
  cache-or-connection step are as the fourth edition left them.

  *Amended in the thirtieth edition: template resolution moves from the seventh
  position to the fourth, and nothing else about this requirement changes.* The
  steps are still eight, none is added or withdrawn, and no step carries a
  different code. What changes is where one of them is evaluated: a template
  name is resolved immediately after `.tpl/.cfg`, and before the entry, the
  cache, the connection and the catalogue. The two notes above name the
  cache-or-connection step instead of numbering it, because its number has
  moved beneath them.

  *Why it moves: the condition needed nothing the old position gave it.*
  `FR-TMPL-023` makes the template root a property of the resolved project, and
  `FR-TMPL-003` establishes that resolving a template name requires no entry,
  no cache and no connection — so the condition was decidable as soon as step 2
  had run. `FR-SCH-008` had already placed a condition on that ground: the
  shape of a qualified routine name is decidable without a server, so it is
  evaluated at step 1 and precedes every catalogue read. This is the same
  ground one step later, because a template name needs the project where that
  token needed only itself.

  *What the old position cost.* An invocation whose template cannot resolve
  cannot render, and it paid first for everything the steps in between perform:
  the entry resolved, with the `${VAR}` expansion of `FR-CONF-015` and the
  child of `FR-CONF-024`; the one connection of `NFR-PERF-004`; the whole
  catalogue read that `NFR-PERF-001` governs; and the store written one file
  per object under `FR-CACHE-030`. None of it could be curtailed, because
  `FR-CACHE-007` obliges the write to happen before the answer, so the store is
  full by the time the refusal is reached. `BR-PERF-004` states what the
  performance family requires of this exact invocation: a wrong invocation is
  the one a calling agent makes most often while it is finding its way, and it
  must cost what `tpl --version` costs. Over `WL-001` it could not, while a
  name answerable from a directory listing was answered after the catalogue.
  From the fourth position it can — every step that resolves an entry, opens a
  connection, issues a catalogue statement or writes a cache file now follows
  the refusal.

  *`NFR-PERF-006` reaches the same invocation, and the order stood against it.*
  That requirement obliges a command requiring no catalogue data to open no
  connection, and the clause naming what it covers already reasons about
  invocations rather than command forms — *any `tpl render` invoked with
  `--context`* is an invocation and not a command. A `tpl render` whose
  template does not exist requires no catalogue data either. A requirement of
  form and an order pulled against each other, and `BR-PERF-001` prefers the
  form, which holds everywhere and forever.

  *It is one order still.* Moving a step does not make a second order, and no
  command gains one of its own. `tpl render` is the only command that reaches
  both sides of the move: `FR-TMPL-003` and `FR-CACHE-011` keep every
  `template` subcommand away from an entry, the cache, a connection and the
  catalogue, and no other command names a template. For every command but one
  the move is unobservable.

  *Accepted cost, and it is `FR-ERR-007` working rather than a defect.* The
  first unsatisfied condition is the one reported, so an invocation carrying
  two faults now reports the other of them: `tpl render nosuch` in a project
  that selects no entry exits `66` where it exited `78`, and
  `tpl render nosuch --context bad.json` exits `66` where `FR-RND-020` gave it
  `65`. Neither requirement changes and neither is weakened — each states its
  own condition and is reported whenever it is the first to fail — and a caller
  still corrects one fault per invocation and reaches a correct invocation in
  the same number of them. The second cost is small and falls the other way: a
  render that fails at a later step resolves its template name first, which is
  one lookup under a root already canonicalised.

  *Accepted cost, stated plainly, because it is visible outside the process.*
  Under `--context -` the document is no longer read before the template name
  is judged, so a producer at the other end of the pipe is cut off. It exits
  `74` under `FR-ERR-026` only where it had **already written bytes of its
  document into the pipe** when the consumer went away; where it had not, its
  first write finds the reader gone with nothing delivered, which is the silent
  `0` of `FR-ERR-025`. What a caller branches on is unchanged either way: the
  refusal is downstream of the producer, so `66` is the status a shell reports
  for the pair of `FR-RND-017` with `pipefail` and without it, and the
  producer's own code is reachable only by inspecting each stage.

  *Corrected in the thirty-first edition: the stated condition was necessary
  and not sufficient.* The thirtieth edition gave it as *where its document
  does not fit the pipe buffer*, which is the condition under which the
  producer is still writing when the consumer leaves — a necessary condition,
  and short of the one that decides the code, which is whether a byte reached
  the pipe. Measured with the fixture up, eight runs of eight of
  `tpl schema dump --direct --no-cache | tpl render nosuch --context -` gave
  `producer=0 consumer=66 pair=66` on every run: the consumer refuses in about
  **3 ms**, on the fourth step of this order, while the producer is still
  connecting and reading the catalogue, so no byte of the dump had been
  written. The test
  `fr_err_006_under_context_dash_the_producer_is_cut_off_and_the_pair_is_still_66`
  asserts it. `FR-ERR-025` and `FR-ERR-026` are amended in the same edition to
  say what puts a document in flight, so that the condition this note states is
  the condition those two requirements state.

  *Rejected: leaving the order as it was and recording why it stands.* The
  ground would have had to be that one order for every command is worth what
  the old position cost — and moving a step keeps one order, so there was
  nothing for the cost to buy.

  *Rejected: a clause exempting `tpl render`, evaluating template resolution
  before entry resolution for that command alone.* It puts a second order in
  the corpus, leaves `FR-ERR-007` two orders to choose between, and buys
  nothing the move does not, since no other command observes the difference.

  *Rejected: keeping the numbering and stating the new position in prose.* It
  preserves every citation of a step by number, at the price of a numbered list
  whose numbers are not the order — and being the order is the list's only job.
  The six citations of a moved number elsewhere in this corpus are corrected
  instead, each by naming the step it means or by dropping a number it never
  needed.

  *Rejected: placing template resolution below entry resolution, so that only
  the cache, the connection and the catalogue follow it.* Entry resolution
  selects the entry, expands `${VAR}` and obtains the password, none of which
  an invocation that cannot render has a use for, and the `78` it can raise is
  a fault the caller meets on the next invocation anyway.

  *Note added in the fortieth edition.* A miss discovered during the render
  step, under `FR-CACHE-039`, returns the invocation to the cache-or-connection
  step. A condition of that step or of catalogue object resolution raised then
  is reported with that step's code. A render condition raised before the miss
  is reached is reported as the first failure, and no connection is opened.
  This is one order still: the render that was abandoned produced no result,
  and the steps run again in their order for the render that does.

  *Amended in the forty-second edition.* A render bound the abandoned render
  crosses, before the miss or after it, is a render condition of step 8 and is
  reported as the first failure with `65`: the invocation does not return to
  the cache-or-connection step, and no connection is opened. The steps run
  again only for an abandoned render that returned within every bound, per
  `FR-CACHE-039` and `FR-RND-038`.

  *Note added in the forty-sixth edition.* "The first that fails" names a
  condition, not an object. One condition met by several objects of one
  invocation is reported once per object only where a requirement says so,
  and one does: `FR-TMPL-032` reports every template `tpl template check`
  finds with a syntax error, one message each, all with `65`. The order of the
  steps, and which code wins between two conditions, are unchanged.

  *Note added in the forty-eighth edition.* Steps 2 and 3 run for the four
  `template` subcommands, which require a project and are not named by
  `FR-PROJ-025`; `FR-TMPL-003` states it. `FR-PROJ-027` and `FR-PROJ-028` are
  conditions of step 2, and a project without `.cfg` passes step 3 with an
  empty configuration, per `FR-PROJ-028`.

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

- **FR-ERR-043**: WHERE a `hint` carries a runnable `tpl` command, and the
  invocation was given `--tpl-dir` or `-d/--database`, the command SHALL carry
  the same flag with the same value, written immediately after `tpl`, with
  `--tpl-dir <path>` before `-d <entry>`. It SHALL NOT carry either flag:

  - WHERE the flag has no effect on the node the command names, per the table
    of `BR-GLOB-001`;
  - WHERE the `hint` exists to change or to remove that flag, as a `hint`
    answering `FR-RND-018` removes `-d`, and one answering `FR-GLOB-007` puts a
    candidate entry in place of the value given; the command then carries what
    the `hint` proposes.

  The value SHALL be written as the caller wrote it: the path under the set of
  `FR-ERR-041`, and the entry name under the set of `FR-ERR-022`. IF the set
  refuses the value, THEN the command SHALL carry the flag with a placeholder
  in the value's position, and the `hint` SHALL state in words that the
  placeholder stands for the value this invocation was given.

  ```
  tpl --tpl-dir /srv/shop/.tpl -d shop schema table ordrs
  hint:  did you mean 'orders'? list the available tables with: tpl --tpl-dir /srv/shop/.tpl -d shop schema tables

  tpl --tpl-dir '/srv/my shop/.tpl' -d shop schema table ordrs
  hint:  did you mean 'orders'? list the available tables with: tpl --tpl-dir <path> -d shop schema tables, where <path> is the --tpl-dir this invocation was given
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The examples fix which flags the command carries,
  where, and the placeholder.

  A value taken from `TPL_DIR`, `TPL_DATABASE` or `core.database` SHALL NOT be
  written: the same command, run in the same environment and project,
  resolves it again.

  *Rationale.* A `hint` is copied and run. A `tpl` command copied without the
  `--tpl-dir` the invocation was given discovers a project from the current
  directory, and one copied without its `-d` selects the default entry, so
  either can succeed against another project or another entry and report
  nothing wrong. `BR-ERR-004` obliges a `hint` to carry a value the invocation
  knows; this requirement names the two values that decide what a command acts
  on and fixes where they are written.

  *Why only these two flags.* They are the global flags that decide which
  project and which entry a command acts on. `--timeout`, `-v` and `-q` change
  how a command runs and not what it acts on, and `-h` and `-V` are not carried
  into a command at all.

  *Rejected: writing the resolved absolute path of `--tpl-dir`.* A caller runs
  the `hint` from the directory it ran the invocation from, where the value as
  written resolves identically, and the value as written is the one the caller
  recognises.

  *Added in the forty-seventh edition,* for rmp `#276`.

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
  | `65` | For a template, the template name, the line, the column, and the chain of underlying engine errors, per `FR-ERR-011`. For a `--context` document, the path and either the position of the malformed JSON or, for a document that does not match the contract, the key path of the first member in document order that fails it and what the contract expects there: the type expected, that the key is required, or, for a reference `FR-CTX-042` governs, the table named and that `tables` does not carry it. It SHALL NOT cite a file of this specification. For a deadline, which deadline expired and its resolved value, per `FR-GLOB-012`. For a render bound, which bound was exceeded, its resolved value, and the key of `FR-CONF-002` that raises it, per `FR-RND-036`, `FR-RND-037` and `FR-RND-039` |
  | `66` | The identifier that was not found, the kind of object it was sought as, and the population it was sought in — the database entry and the server-side database, the template root, the key space of `FR-CONF-002`, or the `--context` document and the collection of it the name was sought in |
  | `69` | The phase that failed — DNS resolution, TCP connect, TLS handshake, the version probe of `FR-SRV-002`, or a catalogue query — the host and port attempted, and what that phase returned |
  | `70` | The invariant that was violated, or that a panic occurred, and in either case where |
  | `73` | The path `tpl init` could not create, and whether the obstacle was an existing `.tpl` or a failure the filesystem reported |
  | `74` | The path or stream that failed, the operation attempted on it, and what the filesystem or the stream returned |
  | `77` | For authentication, the user and the host the server refused, and that the refusal came from the server. For privileges, which property of which object could not be read, per `FR-PRIV-013` |
  | `78` | The key and the file, with the value found and the value expected; or, where the fault is not a key, the specific condition — the directory the walk ended at without finding `.tpl`; or, WHERE `--tpl-dir` suppressed the walk, the path it named, why that path is not usable, and that no upward search was made, per `FR-PROJ-008`; the name of the undefined variable, or the series found and why it is not supported, per `FR-SRV-030` |

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

  *Amended in the thirty-first edition: the `66` row names a fourth
  population.* It named three, and `FR-RND-032` produces a `66` over a fourth:
  an object flag naming a table, a view or a routine that is not in the
  document supplied to `--context`. That population is neither the server-side
  database — no server was reached, per `FR-RND-022` — nor the template root
  nor the key space, so the row obliged a `cause` to name a population it did
  not admit, and a `cause` naming the server-side database for a read that
  touched no server is a wording false of the failure it reports, which the
  paragraph above bans. No code changes and the other three populations are as
  the first edition left them.

  *Amended in the forty-second edition: the `65` row names the render bounds.*
  A render that ends on render fuel, the render output limit or the render
  memory limit is not a
  template error at a line and not a deadline, so no clause of the row could be
  met by its `cause`. The clause added obliges the bound, its resolved value and
  the key that raises it, so the caller learns both what stopped the render and
  where to change it. The other clauses are unchanged.

  *Amended in the forty-third edition: the `65` row names the fault in a
  `--context` document, and the `78` row the path `--tpl-dir` named.* The `65`
  row asked for "the structural rule of context-document.md it failed", and
  the implementation obeyed it: the `cause` restated the rule and cited a file
  the caller does not have, and named no key, per finding E-08 of the audit of
  rmp `#259`. The row now asks for the key path and what was expected there,
  which is the instance this requirement prefers to the category. The `78` row
  named only the directory a walk ended at, and `--tpl-dir` suppresses the
  walk, so a wrong `--tpl-dir` was reported as a walk that never happened,
  per finding E-03. The row now names the path the flag named. The other
  clauses of both rows are unchanged.

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

  *Note added in the forty-sixth edition.* For a command token, `FR-ERR-042`
  also admits a child of which the token is a proper prefix, at any distance.
  The cap of three and this order govern the candidates of both rules
  together.

  *Note added in the forty-ninth edition.* `FR-ERR-044` bounds the distance
  further by the length of the names compared.

- **FR-ERR-044**: A candidate SHALL be admitted by the edit distance of
  `FR-ERR-019` only WHERE that distance is also strictly less than the length
  of the longer of the two names, the supplied name and the candidate,
  counted in characters as `FR-ERR-039` counts them. The bound applies to
  every population of `FR-ERR-021` and to no candidate `FR-ERR-042` admits by
  prefix.

  | Supplied | Candidate | Distance | Longer length | Admitted |
  |---|---|---|---|---|
  | `zz` | `n1` | 2 | 2 | no |
  | `q` | `n1` | 2 | 2 | no |
  | `a` | `b` | 1 | 1 | no |
  | `t1` | `t2` | 1 | 2 | yes |
  | `shp` | `shop` | 1 | 4 | yes |
  | `ordres` | `orders` | 1 | 6 | yes |

  *What the bound means.* No two names are further apart than the length of
  the longer, which is the cost of substituting or inserting every one of its
  characters. A distance below that length is reached only by an alignment
  that keeps at least one character, unchanged or transposed with its
  neighbour. A candidate that keeps no character of the supplied name is not
  offered.

  *Rationale.* A distance of two covers every pair of names of two
  characters or fewer, so `FR-ERR-019` alone offered any short name for any
  other: with entries `n1` and `shop`, `tpl cfg database remove zz`, `ab` and
  `q` each answered `did you mean 'n1'?`, and a caller that follows the hint
  deletes an entry it never named. This is finding W-02 of the sixth
  re-audit, recorded for rmp `#281`. The bound removes every suggestion of
  that shape and none of the slips the threshold exists for, which keep
  nearly every character in place.

  *Why no rule of its own for a command that deletes.* The suggestion is a
  question on every command alike, and nothing runs until the caller runs
  it. The harm observed came from a candidate with nothing in common with the
  name supplied, and the bound removes that candidate from every command at
  once. Where two candidates remain, `FR-ERR-037` writes them as a choice.

  *Rejected: a threshold that scales with the length, such as
  `min(2, ⌊len/3⌋)`.* It withdraws distance-one suggestions from every name of
  three to five characters, `shp` for `shop` among them, which are the slips
  the threshold exists for. Also rejected: withholding every suggestion on
  `tpl cfg database remove` and `tpl cfg unset`. It removes the useful
  suggestions there along with the harmful ones.

  *Accepted cost.* A one-character name is never offered for another
  one-character name, and a two-character name is never offered for one that
  keeps no character of it. The generic `hint`, the listing
  command, still leads to the name in one further invocation.

  *Added in the forty-ninth edition,* for rmp `#281`, from finding W-02 of the
  sixth re-audit.

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
  makes this a measured path — a `66` over `WL-001` compares against 200 names
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

  *Note added in the forty-sixth edition.* For a command, a candidate is also
  one `FR-ERR-042` admits by prefix. A suggestion is omitted only where neither
  rule admits a candidate.

  *Note added in the forty-ninth edition.* A candidate within the distance of
  `FR-ERR-019` and outside the bound of `FR-ERR-044` is not within that
  distance for this requirement.

- **FR-ERR-042**: WHERE the supplied name is a command token — a first
  non-flag token that is not a command, per `FR-CLI-003`; a token after a group
  node that names none of its children; or a segment of the path of
  `tpl help` that names no child, per `FR-HELP-028` — the system SHALL also
  admit as a candidate every canonical name and every alias among the children
  of the node reached of which the supplied token is a proper prefix, compared
  over the characters as written, per `FR-ERR-038`, whatever its distance
  under `FR-ERR-039`. The candidates admitted by `FR-ERR-019` and by this
  requirement SHALL form one set, ordered by distance and then by name and
  capped at three, per `FR-ERR-019`, and written per `FR-ERR-037`. The
  suggestion SHALL NOT be acted on: the invocation still exits `64`
  (`EX_USAGE`), per `FR-CLI-004`.

  ```
  tpl sch tables
  error: unknown command 'sch'
  cause: 'sch' is not a command of tpl; commands are matched in full, never by prefix
  hint:  did you mean 'schema'? list the commands with: tpl help
  exit:  64 (EX_USAGE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the candidate and the code.

  *Rationale.* A caller that shortens a command writes a prefix of it, and a
  prefix is far from the whole name by edit distance: `sch` is three steps
  from `schema`, outside the threshold of `FR-ERR-019`, so the caller received
  no suggestion for the slip it was most likely to make. The command space is
  closed and small, so every prefix candidate is a real command, and the
  suggestion costs nothing on the success path.

  *Why this does not reopen `FR-CLI-004`.* That requirement bars **inferring**
  a command from a prefix, because a prefix unique today stops being unique
  when a command is added. A suggestion is not an inference: nothing runs, the
  caller chooses, and a prefix that matches two commands produces a suggestion
  naming both.

  *Rejected: extending the rule to every population of `FR-ERR-021`.* Tables,
  templates and entries are open populations of up to hundreds of names, where
  a short prefix admits many candidates and the cap of three would choose
  among them by name alone. The finding was about commands, and commands are
  where the population is closed and known.

  *Added in the forty-sixth edition,* for rmp `#274`, from finding S-14 of the
  second re-audit.

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
  value is subject to a character set, whatever its source — the set above,
  except for the populations `FR-ERR-040` and `FR-ERR-041` govern by sets of
  their own; the values this specification names are a table, a view, a
  routine, a template, a database entry, the `<name>` segment of a
  `database.<name>` key, and the name of an environment variable. Each such
  value SHALL be tested on its own, and the
  separators that join names into a command or into a key are literals — the
  space between command-path segments, the `-` or `--` that introduces a flag,
  and the `.` between key segments.

  *Amended in the twentieth edition: the requirement says what the character
  set governs, because as written it governed everything and admitted two of
  the eight populations of `FR-ERR-021` nowhere.* No key of `FR-CONF-002`
  matches `[A-Za-z0-9_]{1,64}`, because every key form contains a dot — fifteen
  when this note was written, eighteen since the forty-second edition;
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

  *Amended in the forty-ninth edition: the consequence above no longer
  drops anything.* `FR-CONF-048` restricts an entry name to the set of this
  requirement, so every entry name, and every `database.<name>` key, is
  admissible in a runnable command.

  *Amended in the thirty-first edition: one population is named that this
  requirement governs and could not admit.* *Every other value is subject to
  the character set, whatever its source* reaches a **flag value the caller
  supplied in a separate token**, which `FR-CLI-018` obliges a `hint` to write
  back in the corrected `--flag=value` form. It is not one of the values this
  requirement enumerates and it is not a literal — a caller chooses it — so it
  falls to the closing clause, and the closing clause refuses every one of
  them: the condition of `FR-CLI-018` is that the value **begins with `-`**,
  and no value beginning with `-` matches `[A-Za-z0-9_]{1,64}`. `FR-ERR-040`
  states the set that governs it, and this requirement's own set is unchanged
  for every other value.

  *Amended in the forty-third edition: two populations are governed by a third
  set.* A template name is a path relative to `.tpl/templates/`, per
  `FR-TMPL-006`, so every nested template carries a `/` and the set above
  refused all of them: `tpl template show rust/_type` offered no suggestion,
  although `rust/_types` is one edit away, while the help promised the nearest
  matches, per finding H-07 of the audit of rmp `#259`. A filesystem path
  written into a hint was refused for the same reason, so a hint could name
  `.tpl/.cfg` only in the relative form, which fails when run from a
  subdirectory of the project, per finding E-22. `FR-ERR-041` states the set
  that governs both, and the set above is unchanged for every other value.

- **FR-ERR-023**: IF a nearest-match candidate is subject to the character set
  that governs it — that of `FR-ERR-022`, or that of `FR-ERR-041` for a
  template name — and falls outside it, THEN the system SHALL NOT present that
  candidate at all
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

  *Amended in the forty-third edition.* A template name is governed by the set
  of `FR-ERR-041`, and this rule drops a template candidate that falls outside
  that set. What it does to a dropped candidate is unchanged.

  *Accepted cost.* A caller that mistypes the name of an object whose real name
  contains a character outside `[A-Za-z0-9_]` receives no suggestion, only the
  generic hint. That hint is the listing command, so the name is still
  recoverable in one further invocation, where `FR-ERR-024` escapes it on the
  way out. The alternative — printing the candidate as prose beside a generic
  hint — was rejected because a caller that copies a whole `hint` line does not
  reliably distinguish its prose half from its command half, which is the
  assumption `FR-ERR-022` exists to avoid relying on.

- **FR-ERR-040**: A flag value the caller supplied in a separate token, written
  into a `hint` under `FR-CLI-018`, SHALL be governed by the character set
  `[A-Za-z0-9_-]{1,64}`, measured over the **whole value** including every
  leading `-`. IF the value falls outside that set, THEN the system SHALL write
  the corrected form with a placeholder in the value's position rather than the
  value, and SHALL present the value nowhere else — neither elsewhere in the
  `hint` nor as prose, on the ground `FR-ERR-023` states for a candidate its
  own set refuses.

  ```
  hint:  write the value in one token: --pattern=-x
  hint:  write the value in one token: --pattern=<value>
  ```

  *It is governed, and the question was whether it is.* `FR-ERR-022` enumerates
  seven values it names and then closes with *every other value is subject to
  the character set, whatever its source*, so a flag value is governed by that
  closing clause and always was. What it could not be is governed by that
  **set**: every value reaching `FR-CLI-018` begins with `-`, so the set
  refuses all of them, and `FR-CLI-018`'s obligation to show the corrected form
  would never once show a value. The set is the thing that yields, for this
  population and no other.

  *Why the hyphen is admitted and nothing else is.* `FR-ERR-022`'s ground is
  that a `hint` is copied and run, so a value built into one must not be able
  to end the command and start another. `[A-Za-z0-9_-]` holds no shell
  metacharacter, no quotation mark, no whitespace and no newline, and the value
  is written after an `=` that joins it to its flag in one token, so it cannot
  be read as a flag of its own by the shell or by `tpl`. The twentieth
  edition's refusal to widen the set of `FR-ERR-022` itself stands and is the
  reason this is a second set: widening there is paid for by every untrusted
  name, and the population that needs the hyphen is one.

  *Why the bound is over the whole value.* It is the half the implementation
  could not enforce. The admission test derived from `FR-ERR-022` splits a
  value on `-` and bounds each segment at 64 characters, which bounds no value
  at all: `tpl -d -a-a-a-a-a-a-a-a version` reaches the `hint` with every
  segment one character long, and a value of any length composed the same way
  reaches it too. `{1,64}` measured over the whole value gives that test
  something to enforce, and 64 is `FR-ERR-022`'s own number, taken rather than
  chosen so that the two sets differ in their alphabet and in nothing else.

  *`FR-CLI-018` is satisfiable, and is satisfied both ways.* That requirement
  obliges the `hint` to **show the corrected form**, and the corrected form is
  `--flag=value` — the joining, not the value. A value the set admits is
  written into it and a value the set refuses leaves a placeholder in its
  position, and in both cases the caller is shown the shape their invocation
  should have had. `FR-ERR-023` governs a nearest-match **candidate** and not
  this value, so it is not extended here; what is borrowed from it is its
  ground, that a value a set refuses is not shown at all rather than shown
  beside a warning.

  *Not exploitable as things stand, and stated so that the bound is not read as
  a fix for a breach.* Nothing outside `[A-Za-z0-9_-]` ever reached a `hint`
  through this path, because the admission test bounds the alphabet correctly
  and bounds only the length wrongly. What was unbounded was how much of the
  caller's own token could be written back to the caller's own terminal.

  *Rejected: declaring a flag value ungoverned.* It is the reading that leaves
  an untrusted value in a runnable `hint` with no test at all, and the closing
  clause of `FR-ERR-022` says the opposite in terms. A later reader finding a
  value in a `hint` and no rule governing it would reopen this, which is what
  stating the answer here prevents.

  *Rejected: widening `FR-ERR-022`'s set to `[A-Za-z0-9_-]{1,64}` for every
  population.* The twentieth edition rejected exactly this and its ground is
  undiminished: every widening is paid for by every untrusted name, and a
  hyphen admitted for a caller's flag value would be admitted for a table name
  the server chooses.

  *Rejected: writing the placeholder form always, never the value.* It is safe
  and it throws away the line's whole value to a calling agent, which is the
  correction it copies. `FR-ERR-009` makes the `hint` the line that gives most
  in return, and a hint that never names the value the caller wrote tells them
  only that a rule exists.

- **FR-ERR-041**: A template name, and a filesystem path written into a
  `hint`, SHALL be governed by the character set `[A-Za-z0-9_./-]`, measured
  over the whole value, which SHALL be at most 1024 characters long and SHALL
  NOT begin with `-`. The paths this requirement governs are the path of the
  project's `.tpl` folder or of a file inside it, as resolved under
  `FR-PROJ-009`; the path a caller named with `--tpl-dir`, its parent
  directory, or that path followed by `/.tpl`, per `FR-PROJ-027`; and the path a caller gave to `--context`, other than `-`, which
  names standard input and is not a path.

  IF a template name that is a nearest-match candidate falls outside the set,
  THEN the system SHALL drop it, per `FR-ERR-023`. IF a path falls outside the
  set, THEN the system SHALL write a placeholder in the path's position, and
  SHALL present the path nowhere else in the `hint`, on the ground
  `FR-ERR-040` states for a flag value.

  ```
  hint:  did you mean 'rust/_types'? list the templates with: tpl template list
  hint:  restrict the file's mode with: chmod 600 /home/ana/shop/.tpl/.cfg
  hint:  restrict the file's mode with: chmod 600 <project>/.tpl/.cfg
  ```

  *Why these characters.* `/` and `.` are the characters a path is made of, and
  `-` is common in a file name. None of the three is a shell metacharacter, and
  the set holds no quotation mark, no whitespace, and no newline, so a value it
  admits cannot end the command it is written into and start another. A value
  that began with `-` could be read as an option by the command it is passed
  to, so none may.

  *Why 1024.* It is `PATH_MAX` on macOS, the smaller of the two limits of the
  supported systems; Linux fixes 4096. A longer path cannot be passed to a
  command on every supported system, so a hint carrying one could not succeed
  on all of them. The number is taken rather than chosen, as `FR-ERR-040`
  takes 64.

  *Rejected: quoting the value instead of testing it.* Quoting admits every
  byte, and the value is escaped by `FR-ERR-024` after it is quoted, so a path
  holding a newline would reach the caller as a different path. A set the
  value either meets or does not keeps one rule for every population:
  what is written is what is run.

  *Rejected: widening the set of `FR-ERR-022` for every value.* The ground the
  twentieth edition gave stands: every widening is paid for by every untrusted
  name, and a `/` admitted for a template name would be admitted for a table
  name the server chooses.

  *Accepted cost.* A project under a directory whose name holds a space, or
  any other character outside the set, receives a placeholder where the path
  would stand, and the caller substitutes it.

  *Added in the forty-third edition,* for rmp `#260`, from findings H-07 and
  E-22 of the audit of rmp `#259`.

  *Amended in the forty-seventh edition: the `--context` path is named,* for
  rmp `#276`. A `hint` that lists the members of a `--context` document names
  the document's path, and the implementation tested it against this set,
  while the list above named only the project's paths and that of
  `--tpl-dir`. A path written into a `hint` and named by no set falls to the
  closing clause of `FR-ERR-022`, whose set admits no `/`, so the list now
  names what the implementation already does. The set is unchanged.

  *Amended in the forty-eighth edition: the corrected `--tpl-dir` value is
  named,* for rmp `#265`. `FR-PROJ-027` writes the path given to `--tpl-dir`
  followed by `/.tpl` into its `hint`. The set is unchanged, and the suffix
  holds only characters of it.

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

- **FR-ERR-045**: WHERE a `hint` carries a command that changes the host, the
  port, the user, the password or the database of an entry defined by `dsn`,
  the command SHALL be `tpl cfg database update <entry> --dsn <url>`, with
  the entry name filled in. The `hint` SHALL state in words that the field is
  changed inside the dsn, and that `<url>` stands for the whole connection URL
  with that field changed. It SHALL NOT carry the stored dsn or any part of
  it, per `BR-ERR-003`, and SHALL NOT carry `--host`, `--port`, `--user`,
  `--schema`, or `tpl cfg set` of one of those fields for that entry.

  This governs every `hint` that repoints or completes an entry: the
  connection `hint` of a `69` for DNS resolution, a refused connection or a
  connect deadline; the `hint` of `FR-SRV-030`; the `hint` of `FR-CONF-041`
  for a dsn with no `/database` segment; and the `hint` of `FR-CFG-048`. An
  entry defined by the discrete fields keeps the flags those requirements
  name.

  ```
  PW=x tpl -d ds schema info
  error: cannot connect to 127.0.0.1:1 for database entry 'ds'
  cause: TCP connect to 127.0.0.1:1 was refused
  hint:  check that the server is running and listening on port 1, or change the address inside the dsn: tpl cfg database update ds --dsn <url>, where <url> is the whole connection URL with the new host or port
  exit:  69 (EX_UNAVAILABLE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the command carried and the code.

  *Rationale.* `FR-CONF-006` makes `dsn` and the discrete fields exclusive in
  one entry, and `FR-CFG-048` refuses a write that would join them. A `hint`
  naming `--host` for a dsn entry is therefore a command that cannot succeed,
  which `BR-ERR-004` forbids, and its own conflict `hint` then led to the loss
  `BR-ERR-005` describes. `tpl cfg database update <entry> --dsn <url>`
  exits `0` and keeps every other field. This is finding X-01 of the seventh
  re-audit of rmp `#263`, recorded for rmp `#282`.

  *Why a placeholder.* The new URL is a value only the caller knows, and the
  stored one may carry a password, which `BR-ERR-003` bars from every message.

  *Added in the fiftieth edition,* for rmp `#282`.

## `EPIPE`

**In flight** is defined in [glossary.md](glossary.md#in-flight). The two
requirements below divide the outcomes of a closed stdout on that line and on
no other.

*Amended in the thirty-second edition: the definition moves to the glossary
and this passage cites it.* The thirty-first edition wrote the definition
here, following `NFR-PERF-007`'s treatment of *differential run*, which left
the corpus with two conventions for where a defined term lives and broke the
one [glossary.md](glossary.md) states of itself. The glossary governs; the
decision, and the alternative rejected with it, are recorded at the head of
that file. **Nothing about the line changes** — the definition there is this
one, word for word on every clause `FR-ERR-025` and `FR-ERR-026` turn on, and
no requirement of this file is amended.

- **FR-ERR-025**: WHEN stdout is closed by the consumer and no JSON document is
  in flight, the system SHALL terminate silently with exit `0`.

  *Rationale.* `tpl … | head -1` is not an error. In `text`, a cut listing is
  exactly what `head` asked for.

  *Amended in the thirty-first edition: "mid-flight" is defined above and the
  wording follows it.* The word was read two ways and the readings disagree on
  a real invocation. A producer that has begun composing a document but has
  written no byte of it can be said to have one mid-flight; on the reading this
  corpus needs it has not, because the rationale of `FR-ERR-026` turns on what
  the **consumer received**, and a consumer that received nothing did not
  receive truncated JSON. Nothing about `tpl` changes: this is the only
  behaviour either requirement ever admitted, and it is what
  `src/output/writer.rs` already does.

- **FR-ERR-026**: IF stdout is closed while a JSON document is in flight, THEN
  the system SHALL exit `74`.

  *Rationale.* The consumer received truncated JSON and cannot tell that it is
  incomplete.

  *Amended in the thirty-first edition, with `FR-ERR-025` and for the same
  reason.* "Part-way through a JSON document" is now "while a JSON document is
  in flight", which the paragraph above defines, so the necessary and
  sufficient condition for this code is stated rather than left to the word
  *through*. A producer whose very first write fails is not part-way through
  anything a consumer can see, and it is `FR-ERR-025`.

  *Observed, 2026-09-21.* Eight runs of eight of
  `tpl schema dump --direct --no-cache | tpl render nosuch --context -`, with
  the fixture up, gave `producer=0 consumer=66 pair=66` on every run. The
  consumer refuses in about 3 ms at step 4 of `FR-ERR-006`, before the producer
  has connected, so the producer's first write finds the reader gone with no
  byte emitted and this requirement is not reached. The note of `FR-ERR-006`
  that records the cost of the move states the same condition, and the test is
  `fr_err_006_under_context_dash_the_producer_is_cut_off_and_the_pair_is_still_66`.

  *Rejected: qualifying the note of `FR-ERR-006` alone and leaving these two
  as written.* The note is an account of a cost and these two are the
  requirements a caller and an implementer read; a condition stated exactly in
  a note and loosely in the requirement it cites is the second copy nobody
  edits, pointing the wrong way round.

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

  The conditions other requirements route elsewhere are the three of the
  cache-or-connection step of `FR-ERR-006`, and all three are `78`: the
  read-only session statement and its read-back, under `FR-SRV-010`, which is
  `78` whether the server refused the statement or the session did not survive
  it, because a setting that cannot be applied is a setting that cannot be
  applied; the product check of `FR-SRV-003`; and the version-window check of
  `FR-SRV-020`. What is left for this requirement is therefore the other two
  statements of `FR-SRV-006` — the version probe, whose own verdicts
  `FR-SRV-003` and `FR-SRV-020` reach only when the probe answered, and the
  catalogue read.

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
  the resolved DSN, or the contents of `.tpl/.cfg`, with exactly one exception:
  the `password_command` array **as stored**, where a requirement of this
  specification obliges a `cause` line to name it. See `FR-GLOB-018`, which
  owns the rule and the exception, `FR-CONF-033`, which carries the residual it
  leaves, and [security.md](security.md).

  *Amended in the thirty-first edition.* The rule barred the contents of
  `.tpl/.cfg` from every error message while `FR-CONF-033` obliged the `cause`
  of a non-zero `password_command` to name the command as stored, and
  `FR-CONF-031` obliged the same of the output cap. The two specific
  requirements govern; this rule states the exception rather than being read
  past. Nothing else on this list yields: a credential is barred without
  exception, and so is the resolved DSN and every other key of the file.

- **BR-ERR-004**: A `hint` SHALL NOT name a command that cannot succeed in the
  state the invocation found. A `tpl cfg` command cannot succeed while
  `.tpl/.cfg` fails step 3 of `FR-ERR-006`, because no `cfg` subcommand is
  excused from that step, per `FR-ERR-035`; for such a fault the `hint` SHALL
  name the file, the position, and the edit to make. WHERE the invocation knows
  a value the `hint` needs — the command path, the template, the database
  entry, the key, or the path of the project — the `hint` SHALL carry that
  value and not a placeholder. A placeholder is admissible only WHERE
  `FR-ERR-022`, `FR-ERR-040` or `FR-ERR-041` refuses the value, or WHERE the
  value is one only the caller knows, such as a new host or a new value for a
  key.

  *What this adds to `FR-ERR-009`.* That requirement asks for a concrete,
  runnable command wherever one exists. A command can be concrete and runnable
  and still fail: `tpl cfg set database.s6.port 3306` for a `.tpl/.cfg` whose
  port is `0` fails with the same `78`, per finding E-01 of the audit of rmp
  `#259`. A placeholder where the value is known is not concrete: the audit
  found `<entry>` printed for an entry the invocation had named, per finding
  E-07. This rule states both outcomes, which a caller copying the line
  depends on.

  *Added in the forty-third edition,* for rmp `#260`.

  *Note added in the forty-seventh edition.* The values this rule lists do not
  include the global flags a copied command needs to act on the same project
  and entry. `FR-ERR-043` states that a `hint` command carries `--tpl-dir` and
  `-d/--database` as the invocation was given them.

  *Note added in the fiftieth edition.* A command that repoints an entry
  defined by `dsn` with `--host`, `--port`, `--user` or `--schema` cannot
  succeed: `FR-CFG-048` refuses it with `64`. `FR-ERR-045` states the command
  such a `hint` carries instead.

- **BR-ERR-005**: A `hint` SHALL NOT name a command that deletes or overwrites
  stored state the invocation did not name: a value of `.tpl/.cfg`, or an
  object of `.tpl/.cache/`. The invocation names a value it writes or deletes,
  and a value whose information its own write supplies anew: the password of
  an entry, where the invocation writes `password`, `password_command`, or a
  `dsn` that carries a password. WHERE the only command that would make the
  invocation succeed deletes or overwrites such state, the `hint` SHALL name
  instead a command that makes the change the invocation asked for and deletes
  nothing else. `FR-ERR-045` and `FR-CFG-048` state that command for a
  database entry, and `FR-CACHE-040` states the `hint` of a clean that names
  nothing cached.

  A nearest-match candidate stands for the name the invocation gave. A `hint`
  MAY carry a command that writes the invocation's own value under the
  candidate, and SHALL NOT carry a command that deletes the candidate: the
  caller may have meant another name, and the next command does not undo a
  deletion.

  *What this adds to `BR-ERR-004`.* That rule asks whether the command can
  succeed. A command can succeed and still cost the caller configuration it
  never named. For an entry added with
  `--dsn 'mysql://reader:${PW}@127.0.0.1:1/shop'`, the conflict `hint` of
  `tpl cfg database update ds --host 127.0.0.1` read
  `tpl cfg unset database.ds.dsn, then run the command again`. Both commands
  exited `0`. The user `reader`, the database `shop` and the `${PW}` reference
  were gone, no message said so, and the next read logged in with no user.
  Every step was a `hint` copied as written. This is finding X-01 of the
  seventh re-audit of rmp `#263`, recorded for rmp `#282`.

  *Rejected: naming the deleting command beside a statement of what it
  deletes.* A caller that copies the command runs it, and a weak reader acts
  on the command and not on the sentence around it. Switching an entry from
  one form of connection to the other is the caller's decision; the `cause`
  of `FR-CFG-048` states what that switch removes, and no `hint` performs it.

  *Added in the fiftieth edition,* for rmp `#282`.

## Dependencies

- [cli-contract.md](cli-contract.md) — the parsing rules that produce `64`.
- [output-formats.md](output-formats.md) — `FR-OUT-015` and `FR-OUT-032`,
  which record on the output side that no error is JSON, and the escaping rules
  `FR-ERR-024` restates for messages.
- [security.md](security.md) — hint construction and redaction as a
  cross-cutting concern.
- [configuration-model.md](configuration-model.md) — `FR-CONF-014`, which
  routes an unreadable entry of a `ca_path` to the `74` of `FR-ERR-001`, and
  `FR-CONF-044`, which answers a `ca_path` that yields nothing with `78`
  instead; together they are why the `74` cell names trust material.

## Open questions

None specific to this module. `OQ-011` and `OQ-023` are dissolved with the
pre-scan of `FR-ERR-017`, and `OQ-021` is answered by `FR-CACHE-036`; all three
are listed under [Closed](open-questions.md#closed).
