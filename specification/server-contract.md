---
title: Server Contract
status: draft
last-reviewed: 2026-09-10
related: [catalogue-coverage.md, context-document.md, cfg-commands.md, cache-commands.md, errors-and-exit-codes.md, privileges-and-completeness.md, security.md, performance-requirements.md]
---

# Server Contract

## Overview

This file states what `tpl` requires of a server and what it promises in return.
Three things are settled here. The first is which servers are supported:
MariaDB, within a window of versions that moves as MariaDB's own maintenance
moves, and nothing else — with a refusal below that window and a marked read
above it. The second is what `tpl` owes the differences between the catalogues
of the supported versions, which the fourth edition raised from tolerating them
to supporting them. The third is the read-only promise, which
the first edition stated as a session setting and which is restated here in the
two parts it actually has — a closed list of statements `tpl` will issue, and a
session setting that catches what the list already prevents.

## Scope

In scope: the criterion that fixes the supported version window, the series that
criterion admits today, how the window is maintained, the refusal of a server
below it and of a server that is not MariaDB, the marked read of a server above
it, which commands the two checks reach, what the model owes a difference
between two supported series, the visibility of the probed version and of the
server's standing to a template, the closed statement list, the read-only
session and its read-back, and the connection count.

Out of scope: which fields are read, which belongs to
[catalogue-coverage.md](catalogue-coverage.md); the handling of a read that
returns less than it should, which belongs to
[privileges-and-completeness.md](privileges-and-completeness.md); connection
settings and TLS, which belong to
[configuration-model.md](configuration-model.md); and the text of any statement.

Also out of scope, and for a reason worth stating rather than assuming: **what
any particular series' catalogue actually returns.** This file fixes the window,
the outcomes at its edges, the detection mechanism, and the contract the model
owes a difference. Which differences exist among the four supported series is
[OQ-045](open-questions.md#oq-045), and no such difference may be written down
until it has been observed, because `scripts/mariadb/` does not exist in this
repository.

## Actors

- **Server**, which must satisfy the version requirement.
- **`tpl`**, which must satisfy the statement restriction.
- **Database administrator**, who can verify both from the server side.
- **MariaDB's maintenance policy**, the external authority the window follows
  and which no party to this specification controls.

## The supported version window

- **FR-SRV-001**: The system SHALL support MariaDB and no other product, and
  SHALL support exactly those MariaDB release series that satisfy **both** of
  the following conditions:

  1. the series belongs to one of the **three most recent MariaDB major
     families**; and
  2. the series is **under MariaDB community maintenance**.

  *Amended in the fourth edition.* The second and third editions fixed a
  minimum of MariaDB 10.6 and stated no newest supported version. That floor is
  withdrawn: 10.6 left community maintenance on 2026-07-06 and is no longer a
  version `tpl` can claim to serve. The window above replaces it and, for the
  first time, states a ceiling as well as a floor.

  *Rationale.* Both conditions are load-bearing and neither is sufficient alone.
  The family condition alone would admit `10.5` and `10.6`, which no longer
  receive fixes; it is also what caps the window's breadth, so that the set does
  not grow every time MariaDB lengthens a series' support. The maintenance
  condition alone would admit a series whose family the window has left, and
  would let the set widen as maintenance windows lengthen. Read together they
  answer the question a reader actually asks — why is `10.11` supported — with
  *because the `10` family is still one of the three most recent and `10.11` is
  the series of it that is still maintained*, and not with an accident of
  position in a list.

  *Rejected.* A bare count of series — "the current release and its three
  predecessors". It selects the same four servers today and explains none of
  them: it gives no reason why `10.11` is in the set rather than `10.6`, it
  cannot express that `12.0` through `12.2` are excluded while `12.3` is not,
  and it would have to be padded to keep its own arithmetic true as maintenance
  dates pass.

- **FR-SRV-015**: The series that satisfy `FR-SRV-001` SHALL be the following,
  as verified on the date recorded below:

  | Major family | Series | GA | Community maintenance ends |
  |---|---|---|---|
  | `12` | `12.3` | 2026-05-28 | 2029-06-12 |
  | `11` | `11.8` | 2025-06-04 | 2028-06-04 |
  | `11` | `11.4` | 2024-05-29 | 2029-05-29 |
  | `10` | `10.11` | 2023-02-16 | 2028-02-16 |

  *Source.* MariaDB's own maintenance policy, at
  `https://mariadb.org/about/maintenance-policy/`, corroborated by MariaDB's
  documentation of its release model. **Verified 2026-09-10.**

- **FR-SRV-016**: A MariaDB **rolling release** SHALL NOT be a supported series.
  `12.0`, `12.1` and `12.2` are therefore unsupported, while `12.3` is
  supported.

  *Rationale.* A rolling release receives no maintenance release after GA,
  reaches end of life as soon as the next rolling release of its family ships,
  and MariaDB advises against it in production. It is excluded by the second
  condition of `FR-SRV-001` and needs no rule of its own; this requirement
  exists because the exclusion is the one an implementer is most likely to get
  wrong. `12.3` is the fourth and final release of the `12` family and the one
  that became long-term-support, so a reader who takes "the last four releases"
  literally arrives at `12.3, 12.2, 12.1, 12.0` — a set of which three members
  are unmaintained and one is the whole supported `12` family. Wherever this
  specification, a help text, or a message says *supported version*, it means a
  series admitted by `FR-SRV-001`, which in practice means a long-term-support
  series.

  *Worked examples of the second condition.* Each of these belongs to a family
  inside the window and is nonetheless unsupported:

  | Series | Community maintenance | Excluded because |
  |---|---|---|
  | `10.6` | ended 2026-07-06 | it outlived its own maintenance while its family stayed in the window |
  | `10.5` | ended 2025-06-24 | the same, one series earlier |
  | `12.2`, `12.1`, `12.0` | never maintained after GA | rolling releases, per `FR-SRV-016` |

- **FR-SRV-017**: The size of the set of `FR-SRV-015` SHALL NOT be fixed by this
  specification. Four series across three families is what `FR-SRV-001` admits
  on the verification date, not a quantity to be preserved.

  *Rationale.* The two conditions of `FR-SRV-001` intersect, and the
  intersection is free to change size. A family can be emptied by the second
  condition while remaining inside the window — on 2028-02-16, `10.11` leaves
  community maintenance and, with `10.6` and `10.5` already out, the `10` family
  leaves the window with it, without MariaDB having shipped anything. Nothing
  may then be added to restore a count of four: the criterion is the
  requirement, and the set is whatever it selects.

- **FR-SRV-018**: The window SHALL roll forward by the two mechanisms of
  `FR-SRV-001`, and by no other:

  | Mechanism | Effect |
  |---|---|
  | MariaDB declares a new major family GA | The oldest family in the window leaves, and every series of it leaves with it. When a `13` family arrives, `10` leaves and `10.11` leaves with it. |
  | A series enters or leaves community maintenance | A newly maintained series of a family already inside the window enters the set; a series whose maintenance ends leaves it, whatever its family. |

- **BR-SRV-004**: The criterion of `FR-SRV-001` is the requirement. The table of
  `FR-SRV-015` is its instance on one date, and the date is part of the table
  rather than a note beside it. A maintainer who finds the table wrong has found
  the table stale, not the requirement wrong, and the correction is to re-derive
  the table from the criterion and the source — never to amend the criterion so
  that the old table remains true.

- **FR-SRV-019**: The table of `FR-SRV-015` SHALL carry its source and its
  verification date, and SHALL be re-verified against that source before every
  release of `tpl`.

  *The next scheduled change.* The earliest date on which the table of
  `FR-SRV-015` is known to become wrong is **2028-02-16**, when `10.11` leaves
  community maintenance. MariaDB ships a long-term-support series roughly
  yearly, so a change is likely well before then and will not announce itself
  here.

  *Rationale.* This corpus's characteristic defect is the statement that quietly
  stopped being true, and a table of version numbers with no verification date
  is the purest form of it: nothing in it decays visibly, and a reader cannot
  tell a checked table from a remembered one. The date and the source together
  make staleness readable; the release gate makes it someone's job.

- **BR-SRV-005**: The set of `FR-SRV-015` is stated **once**, here. Every other
  passage in this corpus that depends on which servers are supported SHALL cite
  `FR-SRV-001` or `FR-SRV-015` rather than repeat a version number. A second
  copy of the table is a second thing to keep true, and it is the copy that will
  be missed.

## Refusal of an unsupported server

- **FR-SRV-002**: WHEN the system opens a connection, it SHALL determine the
  server product and version before issuing any statement other than the
  read-only session statement of `FR-SRV-008` and its read-back under
  `FR-SRV-009`.

  *Amended in the fourth edition.* The requirement previously read "before
  reading any catalogue data". That tied the gate to *reading* rather than to
  *connecting*, and so it did not reach `tpl cfg database test` — the one
  command that opens a connection and reads no catalogue, per `FR-CACHE-010`.
  Attached to the connection, it reaches every command that opens one. The two
  statements it defers to are the ones `FR-SRV-006` already places at connection
  start, so the strongest guarantee this tool makes is still confirmed before
  the server is characterised, as `FR-ERR-006` records.

- **FR-SRV-003**: IF the server is not MariaDB, THEN the system SHALL exit `78`
  (`EX_CONFIG`) with `kind: server_not_mariadb`, and SHALL NOT read the
  catalogue.

  *Rationale.* MySQL is refused rather than attempted because three verified
  divergences make the model **silently wrong** rather than empty. The catalogue
  quotes a string default in MariaDB and does not in MySQL, so the
  literal-versus-expression discriminant of `FR-CTX-012` inverts. MySQL's check
  constraint table carries no table name, so `FR-CAT-015` cannot attribute a
  constraint to its table. MySQL's column table carries no generated-column
  flag, so `FR-CAT-009` cannot distinguish a generated column from an ordinary
  one. Each of the three produces plausible output that is wrong.

  *Rejected.* Attempting a non-MariaDB server with a warning on stderr, which
  leaves the exit code at `0` and hands the caller a wrong model.

- **FR-SRV-020**: IF the server is MariaDB, its series is not one of
  `FR-SRV-015`, and its series is older than the newest series of `FR-SRV-015`,
  THEN the system SHALL exit `78` with `kind: server_version_unsupported`, and
  SHALL NOT read the catalogue.

  *Rationale.* This is the condition of `FR-SRV-001` failing downward, and it
  covers three cases with one rule: a series whose maintenance has ended
  (`10.6`), a rolling release (`12.1`), and a series of a family the window has
  left. Every one of them is a series the project will never observe against the
  container of `scripts/mariadb/`, because observation follows support; so `tpl`
  reading one would be presenting a model whose correctness nothing has
  established. That is the same argument `FR-SRV-003` makes about MySQL, applied
  to a MariaDB that is out of the window.

  `78` rather than `69` (`EX_UNAVAILABLE`) because the server is reachable and
  the credentials work: what is wrong is that the entry points at a server `tpl`
  does not serve, which is a fact about the configuration. It is the code
  `FR-SRV-003` already uses for the same shape of problem.

  *Rejected.* Reading an out-of-window server and emitting every unverified
  field as `null`, which keeps an old setup working at the price of a model
  `tpl` cannot vouch for, indistinguishable from a model it can. Also rejected:
  a distinct code per case — maintenance ended, rolling release, family out of
  window — which the caller cannot act on differently; the `kind` field carries
  the distinction, per `FR-ERR-015`. Also rejected: a flag or configuration key
  that overrides the window. The command tree is closed by `FR-CLI-002`, so no
  such flag exists unless this specification declares one, and declaring one
  would make the support commitment advisory — the same reason `FR-SRV-011`
  gives for the read-only promise.

  *Closes* `OQ-044`, now listed under [Closed](open-questions.md#closed).

  *The other edge.* This requirement fixes the outcome **below** the window. The
  outcome **above** it is deliberately different and is fixed by `FR-SRV-031`:
  such a server is read, not refused.

- **FR-SRV-034**: The product check of `FR-SRV-003`, the version check of
  `FR-SRV-020`, and the marked read of `FR-SRV-031` SHALL apply to every command
  that opens a connection, without exception. Those commands are the eight `schema` subcommands and `tpl render`
  without `--context` when the read reaches the server, per `FR-CACHE-009`;
  `tpl cache load`, per `FR-CACHE-022`; and `tpl cfg database test`, per
  `FR-CACHE-010`.

  *Rationale.* The list is closed and can be checked, because four requirements
  bar everything else from opening a connection: `FR-CACHE-011` bars every
  `template` subcommand and every `cfg` subcommand other than `database test`,
  `FR-RND-022` bars the `--context` path, `NFR-PERF-003` bars a cache hit, and
  `NFR-PERF-005` bars `init`, every form of `help`, and every form of `version`.
  Of the four that remain, `tpl cfg database test` is the only one that reads no
  catalogue, and it is therefore the only one the previous wording of
  `FR-SRV-002` let through. No other command is in that position.

  *Closes* `OQ-074`, now listed under [Closed](open-questions.md#closed). The
  consequence for what that command reports is `FR-CFG-039`.

- **FR-SRV-021**: The comparison of `FR-SRV-020` SHALL be made against the
  series, not the point release, and series SHALL be ordered by version number —
  major family first, then series number. `10.11.14` and `10.11.2` are therefore
  the same series and are treated identically, and `12.1` is newer than `11.8`
  and older than `12.3`.

  *Rationale.* Ordering by version number rather than by GA date is what makes
  the three refused cases of `FR-SRV-020` fall out of one comparison: `12.1` is
  in a supported family and is nonetheless below the newest supported series, so
  the rule reaches it without a clause of its own.

- **FR-SRV-030**: The message of `FR-SRV-020` SHALL name the series found and
  the database entry that reached it. Its `cause` SHALL state that the
  connection and the authentication succeeded, SHALL state why the series found
  is not supported, and SHALL list the series that are. Its `hint` SHALL carry
  the `tpl cfg database update` command that repoints the entry, with the entry
  name filled in:

  ```
  error: server series '10.6' is not supported, for database entry 'shop'
  cause: connected and authenticated; the server reports 10.6.21-MariaDB, and 10.6 left community maintenance on 2026-07-06. tpl supports 12.3, 11.8, 11.4 and 10.11
  hint:  repoint the entry at a supported server: tpl cfg database update shop --host <host>
  exit:  78 (EX_CONFIG)
  ```

  *Rationale for the `cause`.* This is the line that separates `78` here from
  `69` and `77`. All three mean "the read did not happen", and a caller's next
  step differs completely between them: an unreachable host is retried, refused
  credentials are corrected, and an unsupported series is neither — the entry
  points somewhere else, or the server is upgraded. `FR-ERR-002` requires each
  distinct condition to carry its own code, and `FR-ERR-010` requires the
  `cause` to be factual and specific; saying **connected and authenticated**
  first is what makes the code readable without a second invocation. It matters
  most for `tpl cfg database test`, whose whole purpose is to answer which of
  the three has gone wrong.

  *Rationale for the `hint`.* `FR-ERR-009` asks for a runnable command wherever
  one exists, and exactly one does: `tpl cfg database update` repoints the entry
  at another server. The other remedy — upgrading the server — is outside
  anything `tpl` can express, so it is not offered as a command. The command is
  given with the entry name resolved and only `<host>`, the one value the caller
  alone knows, left as a placeholder; `FR-ERR-012` forbids replacing it with
  advice to check the configuration.

## A server newer than the window

- **FR-SRV-031**: IF the server is MariaDB and its series is newer than the
  newest series of `FR-SRV-015`, THEN the system SHALL read the catalogue, SHALL
  apply the treatment of the newest series of `FR-SRV-015` to every difference
  it knows, and SHALL set the `standing` field of `FR-CTX-034` to
  `newer_than_supported`.

  *Rationale.* Refusing would make a read-only structure reader break on a
  release cadence it does not control: MariaDB ships a long-term-support series
  roughly yearly, so every binary of `tpl` will in time meet a server released
  after it, and a caller whose server merely got newer would be told to
  downgrade. Reading is also cheaper than it looks, because `tpl` reads an
  **enumerated** field set rather than whatever the catalogue offers — the
  closed coverage of [catalogue-coverage.md](catalogue-coverage.md) — so a
  catalogue that has only grown cannot change the model at all. What reading
  cannot rule out is a field `tpl` already reads whose meaning changed, and that
  residual risk is what the marking of `FR-SRV-032` exists to declare.

  *Rejected.* Refusing with `78`, symmetric with `FR-SRV-020`. The symmetry is
  false: below the window sits a series the project will never observe, because
  observation follows support; above it sits a series the project has not
  observed **yet**, and whose fixtures arrive with the support commitment. Also
  rejected: reading silently, which is `FR-SRV-003`'s rejected option in a new
  place — it hands the caller a model whose standing nothing states.

  *Closes* `OQ-073`, now listed under [Closed](open-questions.md#closed).

- **FR-SRV-032**: The system SHALL carry the marking of `FR-SRV-031` in the
  document. It SHALL NOT signal it on stderr alone, and it SHALL NOT change the
  exit code on account of it.

  *Rationale.* stderr is not contract — `NFR-DET-001` covers stdout only — so a
  warning there is a signal a calling agent may never see and can never depend
  on. That is precisely the option `FR-SRV-003` rejected for a non-MariaDB
  server: a warning with the exit code left at `0`. A field in the document is
  neither silent nor outside the contract, and it travels with the bytes through
  `tpl schema dump`, the cache, and `tpl render --context`.

- **FR-SRV-033**: WHERE `standing` is `newer_than_supported`, the document SHALL
  promise the shape of `FR-SRV-005` and no more, and the specification SHALL
  state that it promises no more.

  *Rationale.* `FR-SRV-026` promises byte-identical documents from identical
  structures across four series that have been observed. Nothing has been
  observed above the window, so the same promise there would be a promise about
  a program nobody has run. What survives is the part that does not depend on
  observation: the document has the shape `FR-SRV-005` fixes, every field it
  defines is present, and the treatment applied was the newest supported
  series'. What does not survive is the content: a field may be `null` that a
  newer catalogue could have filled, and a field `tpl` reads may have changed
  meaning. `standing` is the field that says so, and it is the only honest form
  the promise can take.

- **BR-SRV-008**: The marker is a **permanent part of the shape** of
  `database.server`, present in every document with an enumerated value, and not
  a field that appears when there is something to report. Three requirements
  make that the only coherent form. `FR-SRV-005` fixes the document's shape as
  constant across supported servers and `FR-OUT-012` forbids omitting a field
  because a value is uninteresting, so a conditional marker would break both.
  And `FR-SEM-012` fails the render when a template reads a field that does not
  exist — so the guard itself, `{% if database.server.standing != "supported" %}`,
  would fail on every supported server, which is every server a template is
  normally run against. The precedent is `source`: `FR-CDOC-009` requires it on
  every read with an enumerated value, rather than only when the read came from
  the cache.

- **BR-SRV-009**: What a template does with `standing` is the template author's
  decision, and all three answers are supported. A generator that must not emit
  code from an unverified read tests the field and calls `fail(message)`, ending
  the render with `65` and its own message, per `FR-ENV-021`. A generator that
  tolerates the risk emits a caveat into the file header it is already writing.
  A generator that does not test the field behaves exactly as it did before this
  edition, which is the reason the ordinary value `supported` is present rather
  than the field being absent when all is well. `standing` is an ordinary string
  compared with `==`. No filter and no test is registered for it, and none is
  needed: `FR-ENV-029` keeps the door open to adding one later without breaking
  a template that compares the string today.

## Difference between supported series

The requirement this section serves is stronger than tolerance. Where the four
series of `FR-SRV-015` report the catalogue differently, the difference must be
**supported** — normalised away where the fact exists on all four, marked where
it does not — however small the variation. Emitting `null` and moving on
satisfies only the last of the three cases below.

- **FR-SRV-022**: The system SHALL resolve the connected server to a series of
  `FR-SRV-015`, from the probe of `FR-SRV-002`, before issuing any catalogue
  read, and SHALL select its treatment of every known difference from that
  series alone. A server newer than the window resolves to the newest series of
  `FR-SRV-015`, per `FR-SRV-031`; a server below the window does not reach this
  requirement, per `FR-SRV-020`.

- **FR-SRV-023**: The system SHALL NOT discover a difference by attempting a
  read and handling its failure.

  *Rationale.* Two of this specification's guarantees forbid it. A statement
  that fails is still a statement issued, so an attempt-and-fall-back reader
  sends statements the closed list of `FR-SRV-006` does not contain. And the
  number of catalogue queries would then depend on the server rather than on the
  command, which `NFR-PERF-001` and `NFR-PERF-002` fix. The probe is already in
  the closed list and already mandatory; one comparison against its result costs
  nothing and is deterministic.

- **FR-SRV-024**: WHERE a fact of the model is present on every series of
  `FR-SRV-015` but is reported in a different catalogue location, under a
  different name, or in a different spelling, the system SHALL normalise it to
  the single representation [context-document.md](context-document.md) fixes.

- **FR-SRV-004**: WHERE the model defines a field that the connected series does
  not provide at all, the system SHALL emit that field as `null`.

  *Amended in the fourth edition.* The field was previously described as one
  "introduced after the floor of `FR-SRV-001`". There is no floor now; the
  condition is that the connected series does not carry the fact, whatever the
  reason.

- **FR-SRV-025**: IF a catalogue field that the system reads carries a different
  **meaning** on two series of `FR-SRV-015`, and the two meanings cannot be
  normalised to one, THEN the model SHALL NOT carry that field at all, and the
  exclusion SHALL be recorded as a requirement in
  [catalogue-coverage.md](catalogue-coverage.md), per `FR-CAT-025`.

  *Rationale.* This is the `FR-SRV-003` argument at field granularity. A field
  that means one thing on `11.4` and another on `12.3` produces plausible output
  that is wrong on one of them, and a template cannot detect the difference. An
  absent field is a gap a generator notices; a redefined field is not.

- **FR-SRV-005**: The shape of the document SHALL be constant across every
  series of `FR-SRV-015`. A field SHALL NOT be omitted because the server does
  not provide it, per `FR-OUT-012`.

  *Amended in the fourth edition.* "Every supported server version" now means
  the set of `FR-SRV-015` rather than everything above a floor, and the promise
  is bounded above as well as below.

- **FR-SRV-026**: For a database created from the same accepted DDL on each
  series of `FR-SRV-015`, the document of `FR-SCH-017` SHALL be byte-identical
  across the four, except for the fields `null` under `FR-SRV-004` and the
  `server` object of `FR-SRV-028`. This equivalence SHALL be bounded to the
  series of `FR-SRV-015` and SHALL NOT extend to a server above the window, per
  `FR-SRV-033`.

  *Rationale.* This is what "properly supported" means, stated so that it can be
  tested rather than reviewed. `FR-SRV-005` fixes the shape; this fixes the
  content, and it is the only form of the requirement that a fixture can
  falsify. A normalisation that is merely intended is indistinguishable from one
  that is absent until four documents are diffed.

- **FR-SRV-027**: Every difference the model accommodates under `FR-SRV-024`,
  `FR-SRV-004`, or `FR-SRV-025` SHALL be recorded in this specification, naming
  the field, the series affected, and what each of the four series was observed
  to return.

  *Known gap.* The register that `FR-SRV-027` requires is empty, and must stay
  empty until the differences are observed. Which fields differ among `12.3`,
  `11.8`, `11.4` and `10.11` is [OQ-045](open-questions.md#oq-045), blocked by
  the absence of `scripts/mariadb/`. No entry may be written from a changelog,
  from a release note, or from knowledge of MySQL.

- **BR-SRV-006**: Three cases, three different obligations, and the distinction
  is the substance of this section. A fact reported differently is
  **normalised**, so no template ever sees the difference. A fact absent from a
  series is **marked**, as `null`, so a template can see that it is absent. A
  field whose meaning differs is **excluded**, because neither normalising nor
  marking it can make it safe. Collapsing all three onto `null` — which is how
  the second edition read — would leave the first case delivering different
  documents from identical databases, and the third delivering wrong ones.

## The probed version in the model

- **FR-SRV-028**: The system SHALL carry the product and version determined by
  `FR-SRV-002` into the model, and it SHALL therefore reach the render context
  and the document of `FR-SCH-017`. Its shape in the document is fixed by
  `FR-CTX-031`, `FR-CTX-033` fixes what is validated about it on each of the two
  paths a document can arrive by, and `FR-CTX-034` fixes the `standing` field
  that `FR-SRV-031` sets.

- **BR-SRV-007**: A template that must accommodate a difference cannot do so
  blind, and before the fourth edition it was blind. `FR-SEM-013` lets a
  template distinguish a `null` from a field that does not exist, but nothing
  let it distinguish the two reasons a field is `null`: *this server has no such
  fact*, under `FR-SRV-004`, and *this object has no such value*, which is
  ordinary. The series is the only thing that separates them. It travels in the
  model rather than beside it so that it survives `tpl schema dump` and returns
  through `tpl render --context`, per `FR-SCH-022` — a version that reached only
  a live render would leave the `--context` path unable to make the same
  decision from the same document.

## The closed statement list

- **FR-SRV-006**: The system SHALL issue only statements drawn from the
  following closed list:

  | Statement | Purpose |
  |---|---|
  | `SELECT` against `INFORMATION_SCHEMA.*` | Reading the catalogue |
  | The server version probe | `FR-SRV-002` |
  | The read-only session statement issued at connection start | `FR-SRV-008` |

- **FR-SRV-007**: The system SHALL NOT issue any other statement. It SHALL issue
  no DDL, no DML, no `SHOW`, no statement against any schema other than
  `INFORMATION_SCHEMA`, and SHALL NOT invoke an external process such as a dump
  utility to read structure.

- **BR-SRV-001**: The closed list is the read-only guarantee. It is a property of
  what `tpl` is built to send, it holds whatever the server permits, and it is
  the only part of the promise that prevents rather than detects.

## The read-only session

- **FR-SRV-008**: The system SHALL additionally set the session read-only at the
  engine level on every connection it opens.

- **FR-SRV-009**: The system SHALL read the session state back and SHALL confirm
  that the setting took effect.

- **FR-SRV-010**: IF the setting cannot be applied, or the read-back does not
  confirm it, THEN the system SHALL exit `78` and SHALL NOT read the catalogue.

- **FR-SRV-011**: There SHALL be no flag, configuration key, or environment
  condition that disables `FR-SRV-006` through `FR-SRV-010`.

- **BR-SRV-002**: The session setting is defence in depth and is stated as such.
  It makes a write **fail**; it does not stop the connection from attempting
  one, and it constrains the transaction rather than the session. Describing it
  as prevention, which the first edition and the root documents do, overstates
  it: a connection under that setting can still send a write and receive an
  error, and nothing about the setting alone tells a reader that `tpl` never
  sends one.

  *Rejected.* Keeping the promise as the first edition stated it — a session
  setting presented as prevention — which reads as a guarantee and is a
  detection.

## Verification

- **FR-SRV-012**: The closed list of `FR-SRV-006` SHALL be verified by an
  integration test that observes the statements the server actually receives.

- **FR-SRV-013**: The read-back of `FR-SRV-009` SHALL be verified by an
  integration test that exercises both outcomes: the setting taking effect, and
  the setting failing to take effect.

- **FR-SRV-014**: The connection count of an invocation SHALL be as fixed by
  `NFR-PERF-004`, and SHALL be verifiable from the server side.

- **FR-SRV-029**: The equivalence of `FR-SRV-026` SHALL be verified by an
  integration test executed against every series of `FR-SRV-015`, and the
  refusal of `FR-SRV-020` by an integration test against at least one series
  outside it.

  *Consequence for the fixture.* The container of `scripts/mariadb/` must
  therefore be buildable at four server versions rather than one, and the DDL of
  its fixture must be DDL that all four accept. Where a structure cannot be
  created on all four, the difference is not a fixture problem but an entry the
  register of `FR-SRV-027` owes. This is recorded in
  [OQ-045](open-questions.md#oq-045) so that it reaches whoever stands the
  container up.

- **FR-SRV-035**: The marked read of `FR-SRV-031` SHALL be verified by an
  integration test that presents the reader with a series above its own window,
  and that asserts the exit code is `0` and that `standing` is
  `newer_than_supported`. The seam by which the test does so SHALL NOT appear in
  any help text, in the JSON command tree of `FR-HELP-016`, or in the command
  tree of `FR-CLI-002`.

  *Rationale.* No such server exists to point the test at — by construction, the
  window contains the newest one there is — so the test must narrow the reader's
  window rather than widen the server. Keeping the seam off the published
  surface is the discipline `FR-ERR-031` already applies to the deliberate
  trigger for `70`, and for the same reason: a test hook that is reachable from
  the command line is a flag, and `FR-SRV-020` rejected a flag that overrides
  the window.

- **BR-SRV-003**: All three of `FR-SRV-012` through `FR-SRV-014` are
  observations made from outside the process, on the server. A promise about what
  a process sends that can only be checked by reading that process's own source
  is not a promise a caller can rely on.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the fields whose absence
  `FR-SRV-004` turns into `null`, and `FR-CAT-025`, the closed exclusion that
  `FR-SRV-025` writes into.
- [context-document.md](context-document.md) — `FR-CTX-031` and `FR-CTX-034`,
  the shape of the `server` object and of its `standing` field; and
  `FR-CTX-012`, the discriminant that a non-MariaDB server would invert.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the other
  reason a read can return less than the model defines.
- [performance-requirements.md](performance-requirements.md) — `NFR-PERF-001`,
  `NFR-PERF-002` and `NFR-PERF-004`, the query-count and connection invariants
  that `FR-SRV-023` relies on.
- [security.md](security.md) — `BR-SEC-002`, the cross-cutting statement of the
  read-only promise.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `77`, and `78`;
  the validation order of `FR-ERR-006` in which the refusals of `FR-SRV-003` and
  `FR-SRV-020` fall; and `FR-ERR-002`, `FR-ERR-009`, `FR-ERR-010` and
  `FR-ERR-012`, which govern the message of `FR-SRV-030`.
- [cache-commands.md](cache-commands.md) — `FR-CACHE-009`, `FR-CACHE-010`,
  `FR-CACHE-011` and `FR-CACHE-022`, which together close the list of commands
  `FR-SRV-034` covers.
- [cfg-commands.md](cfg-commands.md) — `FR-CFG-024` and `FR-CFG-039`, the
  command `FR-SRV-034` newly reaches and what it reports.
- [template-environment.md](template-environment.md) — `FR-ENV-021`, the
  `fail(message)` a template calls on the strength of `standing`.
- [render-semantics.md](render-semantics.md) — `FR-SEM-012`, which is why the
  marker of `BR-SRV-008` cannot be a conditional field.

## Open questions

- [OQ-042](open-questions.md#oq-042) — what the version probe returns, and how
  MariaDB is distinguished from a server reporting a MariaDB-compatible version
  string. It also fixes the exact string `FR-SRV-028` carries.
- [OQ-045](open-questions.md#oq-045) — which fields of the model differ among
  the four series of `FR-SRV-015`, and what each returns; the subject of the
  register `FR-SRV-027` requires.
- [OQ-046](open-questions.md#oq-046) — how the session read-back of `FR-SRV-009`
  is performed within the closed statement list of `FR-SRV-006`.
