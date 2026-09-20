---
title: Server Contract
status: approved
last-reviewed: 2026-09-20
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
between two supported series, the register in which each accommodated
difference is recorded, the record of the differences actually observed between
the four series, the visibility of the probed version and of the server's
standing to a template, the closed statement list, the order in which the
three statements that open a connection are issued, the read-only session and
its read-back, and the connection count.

Out of scope: which fields are read, which belongs to
[catalogue-coverage.md](catalogue-coverage.md); the handling of a read that
returns less than it should, which belongs to
[privileges-and-completeness.md](privileges-and-completeness.md); connection
settings and TLS, which belong to
[configuration-model.md](configuration-model.md); and the text of any
statement, save for the one variable the read-back of `FR-SRV-009` must name,
which `FR-SRV-006` records and explains.

Also out of scope, and for a reason worth stating rather than assuming: **what
any particular series' catalogue actually returns.** That belongs to
[catalogue-coverage.md](catalogue-coverage.md), which records the field lists,
and to [context-document.md](context-document.md), which records the shape of
each value. This file fixes the window, the outcomes at its edges, the
detection mechanism, the form of the version string, and the contract the
model owes a difference. Which differences exist among the four supported
series was `OQ-045`, now listed under [Closed](open-questions.md#closed). The
fixture of `scripts/mariadb/` exists, the four series have been observed
against it on five occasions — four comparison passes, and the settling of the
shared DDL that made the fixture buildable at all four — and what was found is
recorded in *Differences observed between the series* below, per `FR-SRV-038`. A difference that is not in that
record has not been observed, and SHALL NOT be written down.

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
  command that opens a connection and reads nothing into the model, per
  `FR-CACHE-010`. Attached to the connection, it reaches every command that
  opens one. The two
  statements it defers to are the ones `FR-SRV-006` already places at connection
  start, so the strongest guarantee this tool makes is still confirmed before
  the server is characterised, as `FR-ERR-006` records.

  *Checked in the twenty-fourth edition, and unchanged.* This requirement
  permits both orders. It defers to the read-only pair without saying whether
  that pair is issued before this probe or after it, and the sentence above
  states the intent without making it an obligation — which is why the intent
  needed a requirement of its own. `FR-SRV-042` is it, and it places this probe
  third. The deferral itself is untouched, and the order no longer has to be
  inferred from the amendment note of a requirement that admits its opposite.

- **FR-SRV-040**: The version string the probe of `FR-SRV-002` returns SHALL
  be taken to have the form
  `<major>.<minor>.<patch>-MariaDB` optionally followed by `-<suffix>`, and
  the `series` of `FR-CTX-031` SHALL be derived from it as `<major>.<minor>`
  and from nothing else. The suffix SHALL NOT be used to derive anything.

  *Observed.* The four fixture servers returned, verbatim:

  ```text
  10.11 -> 10.11.19-MariaDB-ubu2204
  11.4  -> 11.4.13-MariaDB-ubu2404
  11.8  -> 11.8.9-MariaDB-ubu2404
  12.3  -> 12.3.3-MariaDB-ubu2404
  ```

  The suffix is the distribution the image was built on and is a property of
  the **build**, not of the series: three of the four carry `ubu2404` and one
  carries `ubu2204`. It differs across the window and is not a difference
  between the series, so it is recorded below the table of `FR-SRV-038` with
  the two readings named further down. Two further readings return the same
  string on all four —
  the session version variable and its global counterpart — and the global
  variables table of `INFORMATION_SCHEMA` carries the same value under the
  variable name `VERSION`, which matters because that reading is a `SELECT`
  against `INFORMATION_SCHEMA` and so falls inside the closed list of
  `FR-SRV-006` without needing the probe's own entry.

  *Closes the first half of* [OQ-042](open-questions.md#closed). The second
  half is closed by `FR-SRV-041`, which states the limit that closes it.

  *Two further readings were taken and neither is a version.* The build's
  source revision is a distinct 40-character hash per build and differs
  between all four; the SSL library string differs between `10.11` and the
  other three. Both are properties of the build, and the variables they were
  read from are `version_source_revision` and `version_ssl_library`. A third
  variable of the same family, `version_malloc_library`, returns a row on
  every server of the fixture and the same value on all of them, so it varies
  with nothing; the amendment below records it and states where it belongs.

  *Amended in the eighteenth edition: the two readings are classified where
  the observations live, not here.* They were recorded here as properties of
  the build and given no home in the record of `FR-SRV-038`, which that
  requirement's first sentence obliged, and the classification was asserted
  rather than grounded — no two servers of one series had been compared for
  either. Both are now recorded below the table of `FR-SRV-038`, with the
  suffix above, on the ground the fourth validation rule of the
  [README](README.md#maintenance-debt) supplies; the source revision is
  settled there by *per build* in the sentence above, and the SSL library
  string carries its bound and the observation that would settle it. The
  readings themselves are unchanged, and this requirement still derives
  `series` from `<major>.<minor>` and from nothing else.

  *Amended in the twenty-third edition: a reading recorded as absent is
  present on every server.* The paragraph above read that a malloc-library
  variable was requested and **no row came back on any of the four**, so no
  such variable exists on these servers. `version_malloc_library` returns a
  row on all four series of `FR-SRV-015` and on the fifth listener of the
  fixture, and its value is `system`. It was read three ways on each of the
  five on 2026-09-18 — a `SHOW` of the variables matching a prefix, the
  global-variables table of `INFORMATION_SCHEMA`, and a `SELECT` of the global
  variable itself — and all fifteen readings agreed. The run that produced
  them is recorded with the fixture, in `scripts/mariadb/README.md`, which
  holds the values and is not restated here. What returned no row was the
  **name**: a name a server does not have prints nothing at all under a `SHOW`
  with a `LIKE`, header included, and exits `0`, while selecting that same
  wrong name directly fails with `ERROR 1193 (HY000)`. Both forms were
  observed on `11.8`, and the earlier record read the silence of the first as
  an answer. None of these readings is a statement `tpl` issues: `FR-SRV-006`
  and `FR-SRV-007` are untouched, and the probe of `FR-SRV-002` is still the
  only version reading this requirement governs.

  *The malloc reading is neither a row of `FR-SRV-038` nor a line below its
  table.* It agrees on all four series and on the fifth listener, and both
  homes that requirement offers hold an observation that **differs** across
  the servers read — a row where the difference is established as one between
  the series, a line below the table where it is not. A reading that differs
  nowhere is neither, and it is recorded here, beside the readings it was
  taken with.

  *Rejected.* Striking the sentence and recording nothing in its place. The
  correction's whole content would go with it, and so would the reason the
  record was wrong: a `SHOW` with a `LIKE` that prints nothing and exits `0`
  is a shape the next reader will meet again, and naming it is what stops the
  same silence being read as an answer twice. Also rejected: recording the
  reading below the table of `FR-SRV-038` beside the three readings of the
  build. That home holds what differs across the servers read and is not
  established as a difference between the series; a reading that agrees
  everywhere, placed there, would turn a home into a list of readings taken,
  and would invite the counts beside difference 8 to move for a reading that
  separates nothing.

  *Checked in the twelfth edition against difference 13 of `FR-SRV-038`, and
  unchanged.* A server also announces a version when the connection opens, and
  on `10.11` that announcement is the same string with a `5.5.5-` prefix in
  front of it. This requirement is not reached by it: it fixes the form of the
  string **the probe returns**, and the probe's answer carries no prefix on any
  of the four. The clause *and from nothing else* is what keeps it that way,
  and it is load-bearing rather than decorative — `<major>.<minor>` taken from
  the announcement would read `5.5` on `10.11`, and `FR-SRV-020` would refuse a
  supported server as older than the window. The requirement stands as
  written, and the observation is recorded under `FR-SRV-038` rather than
  here.

- **FR-SRV-041**: IF the version string the probe returns does not contain the
  product marker `MariaDB`, THEN the server SHALL NOT be taken to be MariaDB,
  and `FR-SRV-003` SHALL apply. The condition is **necessary and not
  sufficient**: the system SHALL NOT claim to distinguish MariaDB from a
  server that is not MariaDB and reports a MariaDB-compatible version string,
  and this file SHALL state that limit rather than leave it to be
  discovered.

  *Observed.* All four series of `FR-SRV-015` carry the literal `-MariaDB`
  immediately after the patch number, and the version-comment variable reads
  `mariadb.org binary distribution` on all four. No server that is not
  MariaDB was observed: the fixture of `scripts/mariadb/` holds four MariaDB
  servers and no impostor.

  *Consequence, stated plainly because it is a weaker guarantee than the words
  suggest.* **A server determined to pass as MariaDB will pass.** Every
  reading the condition can rest on — the version string, the version-comment
  variable, and any other reading the probe could take — is a response the
  server itself composes, so a server that emulates MariaDB completely is
  indistinguishable from MariaDB by any observation `tpl` can make over the
  wire. `FR-SRV-003` refuses every server the condition rejects and no other:
  `tpl` does not detect an impostor, and no passage of this corpus says it
  does.

  *Closes the second half of* [OQ-042](open-questions.md#closed), and with
  `FR-SRV-040` the entry entire — on the necessary condition, with the limit
  written here rather than left as an entry. It closes rather than waits
  because the evidence it asked for is very likely unobtainable in principle
  and not merely unobserved for want of a fixture: every check available is
  made over responses the server itself controls, and a complete emulation
  defeats all of them at once. An entry that no achievable observation can
  close is not an open question but a limit, and a limit belongs beside the
  requirement it qualifies.

  *Rejected.* Building an impostor into `scripts/mariadb/`. MySQL cannot be
  made to report a MariaDB version string, so the fixture would need a proxy
  that lies; and observing one impostor yields a rule that separates that
  impostor rather than the class. Also rejected: leaving the entry open. The
  evidence will not arrive, so it would sit open permanently and drain the
  meaning of an index whose other entries were each something someone could
  go and settle.

  *What would change this.* A published external authority stating how the two
  are separated, cited by name and by date under the fourth provenance of the
  [README](README.md#provenance) and re-verified on the schedule that
  provenance requires. That is an amendment to this requirement and to
  `FR-SRV-003` together, and it would turn a necessary condition into a
  sufficient one. Nothing short of it will.

  *Checked in the twelfth edition against difference 13 of `FR-SRV-038`, and
  unchanged.* The marker this condition rests on is carried by both readings of
  the version on all four series: the announcement a server makes when the
  connection opens carries `MariaDB` where the probe's answer does, and on
  `10.11` carries it behind the `5.5.5-` prefix as well. The condition returns
  the same verdict from either reading, so no server is admitted or refused by
  that difference, and the limit below is untouched by it — an announcement is
  one more response the server composes, and a server that emulates MariaDB
  completely composes that one too.

  *A stated limit.* This requirement names where a guarantee stops, in the
  form the [README](README.md#writing-conventions) fixes for all three:
  `FR-PRIV-020`, where a table whose triggers are hidden cannot be told from
  a table that has none, and `FR-CONF-039`, where pinned trust material is
  additional to the public root bundle rather than exclusive of it.

- **FR-SRV-003**: IF the server is not MariaDB, THEN the system SHALL exit `78`
  (`EX_CONFIG`), and SHALL NOT read the catalogue. The `cause` SHALL state that
  the connection and the authentication succeeded, SHALL name the product the
  server reported, and SHALL state that `tpl` supports MariaDB only, per
  `FR-ERR-034`.

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

  *Amended in the fifth edition.* The requirement previously also named
  `kind: server_not_mariadb`, a field `FR-ERR-015` withdraws. The obligation
  moves to the `cause` line, where `FR-ERR-034` makes it testable: this
  condition and the one `FR-SRV-020` refuses share the code `78`, so the
  `cause` is what separates them for the reader who has to act.

- **FR-SRV-020**: IF the server is MariaDB, its series is not one of
  `FR-SRV-015`, and its series is older than the newest series of `FR-SRV-015`,
  THEN the system SHALL exit `78` with the message of `FR-SRV-030`, and SHALL
  NOT read the catalogue.

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
  window — which the caller cannot act on differently; the `cause` line of
  `FR-SRV-030` carries the distinction, per `FR-ERR-034`. Also rejected: a flag
  or configuration key
  that overrides the window. The command tree is closed by `FR-CLI-002`, so no
  such flag exists unless this specification declares one, and declaring one
  would make the support commitment advisory — the same reason `FR-SRV-011`
  gives for the read-only promise.

  *Closes* `OQ-044`, now listed under [Closed](open-questions.md#closed).

  *The other edge.* This requirement fixes the outcome **below** the window. The
  outcome **above** it is deliberately different and is fixed by `FR-SRV-031`:
  such a server is read, not refused.

  *Amended in the fifth edition.* The requirement previously named
  `kind: server_version_unsupported`, a field `FR-ERR-015` withdraws. The
  message of `FR-SRV-030` was already the whole of what a caller receives, and
  it already carries the three cases in its `cause`; the reference now points
  at it.

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
  Of the four that remain, `tpl cfg database test` is the only one that reads
  nothing **into the model** — its one catalogue statement is the privilege
  probe of `FR-CFG-044`, whose result is a boolean, per `FR-CACHE-010` — and it
  is therefore the only one the previous wording of `FR-SRV-002` let through.
  No other command is in that position.

  *Amended in the fifth edition.* The clause read "the only one that reads no
  catalogue". `FR-CFG-044` gives that command a catalogue statement, so the
  clause stopped being true when `OQ-002` was answered. The property this
  rationale actually rests on is that nothing it reads enters the model, and
  `FR-CACHE-010` now states that property.

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

- **FR-SRV-037**: The system SHALL NOT issue a statement that names a fixed
  list of `INFORMATION_SCHEMA` columns unless every column named is present on
  every series of `FR-SRV-015`. Where a column is present on some series and
  not on others, the system SHALL select the column list from the series
  resolved by `FR-SRV-022`, or SHALL restrict the list to the columns common to
  all four series, and SHALL NOT determine the list by issuing a statement and
  handling its failure.

  *Observed.* Three `INFORMATION_SCHEMA` tables differ in width across the four
  series, recorded as differences 5, 6 and 7 of *Differences observed between
  the series* below. Naming a column a series does not have is a hard
  `ERROR 1054 (42S22)` — not a `NULL`, not a warning — so a single fixed
  column list against `INFORMATION_SCHEMA.COLUMNS` or
  `INFORMATION_SCHEMA.PARAMETERS` cannot run unmodified on all four.

  *Rationale.* This is `FR-SRV-023` applied to the shape of the catalogue
  rather than to its content, and it is stated separately because the failure
  mode is different in kind. A difference of content produces a wrong value; a
  difference of width produces a statement the server refuses outright, which
  would make `tpl` unusable against a whole supported series rather than
  inaccurate against it. The two permitted answers are the two `FR-SRV-022`
  already licenses: the treatment of a known difference is selected from the
  resolved series, and a list that names nothing series-specific needs no
  selection at all.

  *Rejected.* Probing the shape of these tables at connection time. It is a
  `SELECT` against `INFORMATION_SCHEMA` and so is inside the closed list of
  `FR-SRV-006`, but it adds a catalogue query whose presence depends on the
  server, which `NFR-PERF-001` and `NFR-PERF-002` forbid, and it buys nothing
  the series already determines. Also rejected: naming every column of the
  widest series and tolerating the error on the others, which `FR-SRV-023`
  forbids and which would make a supported series unreadable.

- **FR-SRV-024**: WHERE a fact of the model is present on **more than one**
  series of `FR-SRV-015` but is reported in a different catalogue location,
  under a different name, or in a different spelling on those series, the
  system SHALL normalise it to the single representation
  [context-document.md](context-document.md) fixes. `FR-SRV-004` SHALL mark the
  series that do not carry the fact at all.

  *Amended in the fifth edition.* The precondition was "present on every series
  of `FR-SRV-015`". That left a case with no obligation and made
  `BR-SRV-006`'s three cases non-exhaustive: a fact present on `12.3` and
  `11.8` and reported differently between them, and absent from `11.4` and
  `10.11` altogether, met neither `FR-SRV-024`'s "every series" nor
  `FR-SRV-004`'s "the connected series does not provide it at all" — so the
  document could carry two different representations of one fact from two
  supported servers, which is precisely what this section exists to forbid.
  Lowering the precondition to more than one series closes it: the fact is
  normalised wherever it exists, and marked `null` wherever it does not. Both
  obligations can apply to one field on different series, and that combination
  is now the ordinary case rather than a gap.

  *What is preserved.* `FR-SRV-026` is unchanged and is what makes the
  amendment testable. It excepts the fields `null` under `FR-SRV-004` from the
  byte-identical comparison across the four series, so a field normalised on
  two series and `null` on the other two still satisfies it — which it could
  not have done had the amendment instead widened `FR-SRV-026`.

- **FR-SRV-004**: WHERE the model defines a field that the connected series does
  not provide at all, the system SHALL emit that field as `null`. This applies
  whether or not the fact is normalised on the series that do provide it, per
  `FR-SRV-024`.

  *Amended in the fourth edition.* The field was previously described as one
  "introduced after the floor of `FR-SRV-001`". There is no floor now; the
  condition is that the connected series does not carry the fact, whatever the
  reason.

  *Amended in the fifth edition.* The last sentence is new. It states that this
  requirement and `FR-SRV-024` compose over one field rather than dividing the
  fields between them, which is what makes the two cover every difference of
  presence between the four series.

- **FR-SRV-025**: IF a catalogue field that the system reads carries a different
  **meaning** on two series of `FR-SRV-015`, and the two meanings cannot be
  normalised to one, THEN the model SHALL NOT carry that field at all, and the
  exclusion SHALL be entered in the **ambiguous-meaning exclusion list** of
  `FR-CAT-029`.

  *Rationale.* This is the `FR-SRV-003` argument at field granularity. A field
  that means one thing on `11.4` and another on `12.3` produces plausible output
  that is wrong on one of them, and a template cannot detect the difference. An
  absent field is a gap a generator notices; a redefined field is not.

  *Amended in the fifth edition.* This requirement previously wrote its
  exclusions into `FR-CAT-025`, which is not a list at all: it is the rule that
  **closes** the volatile-field list of `FR-CAT-024`. Writing a field of
  ambiguous meaning into the volatile list would also have been wrong on the
  merits — the two exclusions have different grounds, different tests, and
  different futures. A volatile field is excluded because the server changes it
  without the structure changing, and it will never be admitted; a field of
  ambiguous meaning is excluded because two supported series disagree about it,
  and it can be admitted the day they agree or the day one of them leaves the
  window. `FR-CAT-029` is the second list, parallel to the first, with its own
  closing rule in `FR-CAT-030`.

- **FR-SRV-039**: WHERE a catalogue field the model carries is present on
  every series of `FR-SRV-015`, means the same thing on every one of them, is
  reported in the same place under the same name, and nonetheless carries a
  **different value** on one of them because the servers' own defaults
  differ, the system SHALL carry that value **exactly as the server returns
  it**, without normalisation, substitution or adjustment of any kind. Every
  character-set value and every collation value the model carries SHALL be
  passed through under this rule.

  *This is the fourth treatment of `BR-SRV-006`, and it exists because the
  first three do not reach the case.* The field is not reported differently,
  so `FR-SRV-024` cannot normalise it; it is present everywhere, so
  `FR-SRV-004` does not mark it; its meaning is identical everywhere, so
  `FR-SRV-025` does not exclude it. It is a difference of **value**, and the
  three cases were written over differences of presence and of
  representation.

  *Observed.* The session collation recorded against every view, stored
  routine and trigger reads `utf8mb4_general_ci` on `10.11` and
  `utf8mb4_uca1400_ai_ci` on the other three, from identical DDL, because the
  servers' own default collations differ — difference 1 of `FR-SRV-038`. The
  same root cause reaches the schema's default collation, which differs for a
  database created without an explicit one — difference 11.

  *Rationale, in the product owner's terms.* Each collation is a collation.
  `utf8mb4_general_ci` and `utf8mb4_uca1400_ai_ci` are two different
  collations, not two spellings of one, so normalising either onto the other
  would discard the information the field exists to carry — and every
  candidate target for such a normalisation is a choice of one server's answer
  over another's. Values are passed through exactly as received, without
  interference, and **character sets and collations are respected and
  preserved regardless of the server version**.

  *Accepted cost, stated here so it is not discovered later.* The same
  template rendered against a `10.11` and against a `12.3` holding the same
  schema **will differ** wherever it reads such a field. This ranks
  **fidelity to what the server holds above determinism across servers**, and
  it is the one place in this specification where those two are traded
  against each other. `FR-SRV-026` excepts these fields for exactly this
  reason, and `NFR-DET-001` is untouched: two reads of one server still
  produce identical output.

  *Closes* `OQ-075`, now listed under
  [Closed](open-questions.md#closed).

  *Rejected by the product owner.* Excluding such a field from the model, as
  a third ground of exclusion parallel to `FR-CAT-024` and `FR-CAT-029` —
  which was the recommendation. Also rejected: admitting the field and
  normalising the value away, in any of the three forms offered — to the
  character set alone, to the newest series' value, or to the schema's own
  collation — each of which discards a real difference between two real
  collations.

  *What this rule does not reach.* A value that differs between two servers
  because the server **computed** it rather than recorded it. An index
  cardinality is the case, and `FR-CAT-024` excludes it as volatile for the
  reason `BR-CAT-002` gives: passing an estimate through would make two reads
  of one unchanged database differ, which this rule never does.

- **FR-SRV-005**: The shape of the document SHALL be constant across every
  series of `FR-SRV-015`. A field SHALL NOT be omitted because the server does
  not provide it, per `FR-OUT-012`.

  *Amended in the fourth edition.* "Every supported server version" now means
  the set of `FR-SRV-015` rather than everything above a floor, and the promise
  is bounded above as well as below.

- **FR-SRV-026**: For a database created from the same accepted DDL on each
  series of `FR-SRV-015`, the document of `FR-SCH-017` SHALL be byte-identical
  across the four, except for the fields `null` under `FR-SRV-004`, the
  `server` object of `FR-SRV-028`, and the values passed through under
  `FR-SRV-039`. This equivalence SHALL be bounded to the series of
  `FR-SRV-015` and SHALL NOT extend to a server above the window, per
  `FR-SRV-033`.

  *Amended in the seventh edition, and the exception is a real weakening.*
  `FR-SRV-039` admits a field whose raw value differs between two supported
  servers, and a raw value that differs cannot be byte-identical, so the
  exception follows from the decision rather than being a further choice. The
  fields it reaches are enumerated in the register of `FR-SRV-036`, which is
  what keeps the exception from swallowing the comparison: a field that
  differs and is not in that register is a failure of this requirement, not
  an instance of its exception.

  *Rationale.* This is what "properly supported" means, stated so that it can be
  tested rather than reviewed. `FR-SRV-005` fixes the shape; this fixes the
  content, and it is the only form of the requirement that a fixture can
  falsify. A normalisation that is merely intended is indistinguishable from one
  that is absent until four documents are diffed.

- **FR-SRV-027**: Every difference the model accommodates under `FR-SRV-024`,
  `FR-SRV-004`, `FR-SRV-025`, or `FR-SRV-039` SHALL be recorded in this
  specification, naming the field, the series affected, and what each of the
  four series was observed to return.

  *Amended in the seventh edition.* `FR-SRV-039` is a fourth treatment and
  owes the register the same record as the other three. Adding it is also
  what turns the register from a statement about what was **not** found into
  a record of what was.

  *State of the register.* The register held nothing through the sixth
  edition and now holds two rows, both created by `FR-SRV-039`. Of the
  **fourteen** differences observed between the series, two reach a field the
  model carries; the other twelve reach the reader, the fixture, or a
  requirement, and produce no row here. No entry may be written from a
  changelog, from a release note, or from knowledge of MySQL.

- **FR-SRV-036**: The register of `FR-SRV-027` SHALL be the table in the
  section *The divergence register* below, and SHALL be nowhere else. Each row
  SHALL carry exactly three columns: the field of the model, the treatment
  applied to it, and what each series of `FR-SRV-015` was observed to return.

  *Rationale.* `FR-SRV-027` required a record and named no place for it, which
  is how a register comes to be kept in four places or in none. One table,
  named here, is also what lets `FR-SRV-029` check it: the integration test
  executed against every series has a list to compare its observations
  against, and a difference the test finds and the table does not hold is a
  missing row rather than an open question about where to put it.

## The divergence register

Every difference the model accommodates under `FR-SRV-024`, `FR-SRV-004`, or
`FR-SRV-025` is registered here, per `FR-SRV-027` and `FR-SRV-036`.

**The register held nothing through the sixth edition and now holds two
rows.** The four series were stood up from the fixture of `scripts/mariadb/`
on 2026-09-10, the `freight` catalogue was dumped from each and compared field
by field, `INFORMATION_SCHEMA` itself was compared table by table, and a
second pass recorded the field lists themselves; a third pass, on 2026-09-11,
read the session read-only state under each of its two spellings, and a fourth,
the same day, read the version each server announces when the connection opens.
A fifth observation was made before any of the four, while the shared DDL was
being settled, and was recorded elsewhere until the thirteenth edition brought
it here. Fourteen differences were found, and the two rows below are the two
that reach a field the model carries. Both arrived by the same route — the servers'
own default collations differ — and both are accommodated by `FR-SRV-039`
rather than by any of the three treatments the sixth edition had.

| Field | Treatment | Observed on `12.3` / `11.8` / `11.4` / `10.11` |
|---|---|---|
| `collation_connection`, on every view, routine and trigger — `FR-CAT-047`, `FR-CAT-048`, `FR-CAT-050` | passed through | `utf8mb4_uca1400_ai_ci` / `utf8mb4_uca1400_ai_ci` / `utf8mb4_uca1400_ai_ci` / **`utf8mb4_general_ci`** |
| `database.collation`, WHERE the database declares no collation of its own — `FR-CTX-036` | passed through | `utf8mb4_uca1400_ai_ci` / `utf8mb4_uca1400_ai_ci` / `utf8mb4_uca1400_ai_ci` / **`utf8mb4_general_ci`** |

The second row was observed on the `mysql` schema of the four servers rather
than on `freight`, which declares its collation explicitly and therefore reads
identically on all four. The row is registered on the property rather than on
the fixture object, because the difference is a property of any database
created without an explicit collation.

The **Treatment** column takes one of exactly four values, per `BR-SRV-006`:
`normalised` under `FR-SRV-024`, `marked null` under `FR-SRV-004`, `excluded`
under `FR-SRV-025`, or `passed through` under `FR-SRV-039` — and a field may
carry `normalised` on the series that have it and `marked null` on those that
do not, per the amendment to `FR-SRV-004`. A row whose treatment is `excluded`
SHALL also appear in the list of `FR-CAT-029`, which is where the exclusion is
normative; the row here records the observation that justified it. A row whose
treatment is `passed through` is also the licence for that field's exception
to `FR-SRV-026`, and a field that differs without a row here is a failure of
that requirement.

- **BR-SRV-006**: Three cases, three different obligations, and the distinction
  is the substance of this section. A fact reported differently is
  **normalised**, so no template ever sees the difference. A fact absent from a
  series is **marked**, as `null`, so a template can see that it is absent. A
  field whose meaning differs is **excluded**, because neither normalising nor
  marking it can make it safe. Collapsing all three onto `null` — which is how
  the second edition read — would leave the first case delivering different
  documents from identical databases, and the third delivering wrong ones.

  *Amended in the seventh edition: there are four cases, and the fourth was
  found by observation.* A field present on all four series, meaning the same
  thing on all four, reported in the same place under the same name, and
  carrying a **different value** on one of them because the servers' own
  defaults differ — difference 1 of `FR-SRV-038` — is reached by none of the
  three above. Such a field is **passed through**, verbatim, under
  `FR-SRV-039`: the value is carried exactly as the server returns it, and
  `FR-SRV-026` excepts it from the byte-identical comparison. That is the one
  place this specification prefers fidelity to the server over determinism
  across servers, and `FR-SRV-039` states the cost.

  The four cases are now exhaustive over differences of **presence**, of
  **representation**, and of **value**. They remain silent about a difference
  of **meaning** only in the sense that `FR-SRV-025` answers it by exclusion.

  *Closes* `OQ-075`, now listed under
  [Closed](open-questions.md#closed).

## Differences observed between the series

- **FR-SRV-038**: Every difference observed between the series of
  `FR-SRV-015` SHALL be recorded in this section, whether or not it reaches the
  model, naming what was observed on each series and what the difference
  obliges. A difference recorded here that reaches the model SHALL also
  produce a row in the register of `FR-SRV-036`; a difference that does not
  reach the model SHALL NOT produce one. A row SHALL be given only where the
  observation is **established** as a difference between the series, and SHALL
  name what each of them returned. Every other observation that differs across
  the servers read SHALL be recorded **below the table**, so that it is not
  counted as a difference between the series, and three grounds place it
  there: that it was observed to differ between two servers **of the same
  series**; that it is a property of the **build**, which this project's
  fixture selects and does not pin; or that its entailment by the series is
  **not established**, in which case the record SHALL state the bound and
  SHALL name the observation that would settle it.

  *Rationale.* `FR-SRV-027` records only what the model **accommodates**, which
  is the right scope for a normative register and the wrong scope for an
  observation. Twelve of the fourteen differences below reach the reader, the
  fixture, or a requirement rather than the document, and each of them
  constrains work that has not been done yet: without a home they would be
  rediscovered, or worse, contradicted. Keeping the two apart also keeps the
  register honest — two rows in the register beside fourteen in the observation
  record says *we looked, and this is the part the document carries*, which is
  a much stronger statement than either table alone.

  *Amended in the eighteenth edition: the second home is stated over what it
  actually holds.* The rule offered an observation two homes and named one
  test between them — a row if it is a difference between the series, a line
  below the table if it was observed to differ between two servers of the same
  series — and an observation can fail both. Three readings of the **build**
  did: the distribution each image was built on, the build's source revision,
  and the SSL library string, all three recorded under `FR-SRV-040` since the
  seventh edition and none of them in this section at all. They differ across
  the four servers, so the first sentence of this requirement obliged this
  section to hold them; no two servers of one series had been compared for any
  of them, so the sentence that places the routine and trigger timestamps below
  the table did not reach them either. The ground that decides them is the one
  the fourteenth edition wrote for difference 3 and the
  [README](README.md#maintenance-debt) states as its fourth validation rule: a
  reading this project's own fixture can move is not credited to the series
  until the condition is varied. The fixture names a series and not a patch
  release, so the build behind it is exactly such a condition, and the three
  readings are recorded below the table on that ground.

  *Rejected.* Giving the three rows, which is what an observation that differs
  across the window looks like from outside. It fails twice over. This record
  holds no reading for any series for two of the three — only that the four
  servers differ — so four columns of each row could not be filled from
  anything this corpus holds; and a row asserts an entailment by the series
  that the fourth provenance of the [README](README.md#provenance) forbids
  writing down unobserved. Also rejected: leaving all three where they were
  until the observation is taken. That would leave this requirement's own first
  sentence unsatisfied for as long as nobody takes it, and would knowingly
  repeat the defect the thirteenth edition's third validation rule was written
  from — an observation recorded in the requirement whose decision it drove
  rather than in the record that owns it, where two recounts passed over it.

**Method and date.** The four images were built from `scripts/mariadb/` and run
side by side on 2026-09-10; server versions `12.3.3`, `11.8.9`, `11.4.13` and
`10.11.19`. Five observation occasions are recorded here: four comparison
passes over running servers and, before all of them, the settling of the shared
DDL. The first pass dumped the `freight` catalogue from each and compared them
field by field, and compared `INFORMATION_SCHEMA` table by table; it found
seven differences. The second recorded the **field lists themselves**,
verbatim, for the twenty entries of
[open-questions.md](open-questions.md) that asked for them; it found four
more, taking the total to eleven, and took with the version readings of
`FR-SRV-040` three readings of the **build**, which are recorded below the
table rather than in it because none of them is a difference between the
series. The third ran the same four images on
2026-09-11 and read the session read-only state under each of its two
spellings, each spelling in its own statement; it found one more, taking the
total to twelve. The fourth, the same day and on the same four images, read
the version each server announces when the connection opens, over a plain
socket and without authenticating; it found one more, taking the total to
thirteen.

**The fifth occasion was not a pass, and its late arrival is recorded with
it.** The rejection of a `VECTOR` column or index by two of the four series was
observed on 2026-09-10, while the shared DDL of `scripts/mariadb/` was being
settled — the work that made the fixture buildable at all four — and it was
written down beside `FR-SRV-029`, whose fixture consequence it decided, rather
than here. No pass found it because no pass looked for it: a pass compares what
four running servers **report**, and this difference is in what each server
**accepts**, which is settled before any server can be compared with another.
It is difference 14 below, and it takes the total to **fourteen**.

**One condition of observation changed between the occasions, and one row
turns on it.** The fixture carried no TLS material of its own when the first
two passes ran, and carried it by the fourth, whose bounded claim records that
the four servers offer TLS. Difference 3 is the only row whose reading any
setting of the fixture can move — which is checked below the table — and it
names the condition it was taken under. No other row needs one. The three
readings of the build recorded below the table are moved by the fixture too,
and not by a setting: it selects the build and does not pin it, which is the
ground on which they are placed there.

| # | Observed | `12.3` | `11.8` | `11.4` | `10.11` | What it obliges |
|---|---|---|---|---|---|---|
| 1 | Default server collation, which propagates into the session collation recorded against every view, routine and trigger | `utf8mb4_uca1400_ai_ci` | `utf8mb4_uca1400_ai_ci` | `utf8mb4_uca1400_ai_ci` | `utf8mb4_general_ci` | Passed through verbatim, `FR-SRV-039`; excepted from `FR-SRV-026`. Registered under `FR-SRV-036`. Carried by `FR-CAT-047`, `FR-CAT-048`, `FR-CAT-050` |
| 2 | Position of `INVISIBLE` in `SHOW CREATE TABLE`; `INFORMATION_SCHEMA.COLUMNS.EXTRA` is `INVISIBLE` on all four | after the default | after the default | before the default | before the default | Nothing. `FR-SRV-007` bars `SHOW`, so the difference cannot reach `tpl` |
| 3 | `have_ssl`, on a server **left to itself**, with no TLS material configured for it | `YES` | `YES` | `YES` | `DISABLED` | A supported series may offer no TLS at all. `FR-CONF-038`, which states that condition and obliges the fixture to configure a certificate at every series — see below |
| 4 | A temporary table in `INFORMATION_SCHEMA.TABLES` after `CREATE TEMPORARY TABLE` | one row, `TABLE_TYPE='TEMPORARY'` | one row | one row | no row | Fixes the set of `table_type` values a server can emit. `FR-CAT-031`, `FR-CAT-032` |
| 5 | Width of `INFORMATION_SCHEMA.COLUMNS` | 24 | 24 | 24 | 22 | `FR-SRV-037`. The two extra columns are `IS_SYSTEM_TIME_PERIOD_START` and `IS_SYSTEM_TIME_PERIOD_END` |
| 6 | Width of `INFORMATION_SCHEMA.PARAMETERS` | 17 | 16 | 16 | 16 | `FR-SRV-037`. The extra column is `PARAMETER_DEFAULT`, and `FR-CAT-049` does not carry it: it was SQL `NULL` on every parameter of the fixture on the one series that has it |
| 7 | `INFORMATION_SCHEMA.PERIODS` | present, empty | present, empty | present, empty | absent, `ERROR 1109 (42S02)` | `FR-SRV-037`. `FR-CAT-022` already excludes application-time periods, so nothing else follows today |
| 8 | Declared nullability of the index table's comment column | `varchar(16)` `NOT NULL` | `varchar(16)` `NOT NULL` | `varchar(16)` nullable | `varchar(16)` nullable | Nothing today. `FR-CAT-042` does not carry the field. **The split falls between `11.4` and `11.8`** — see below |
| 9 | Declared width of the trigger table's event column | `varchar(20)` | `varchar(6)` | `varchar(6)` | `varchar(6)` | Nothing. The values returned are `INSERT`, `UPDATE`, `DELETE` on all four, and the column is present on all four, so `FR-SRV-037` is not engaged |
| 10 | Index cardinality **over identical data** | an estimate | the same estimate as `11.8`'s neighbours | agrees with `11.8` and `12.3` | **differs** | Nothing. `FR-CAT-024` excludes it as volatile, and its amendment states why an estimate is not passed through under `FR-SRV-039` |
| 11 | The schema catalogue's default collation, for a database that declares none | `utf8mb4_uca1400_ai_ci` | `utf8mb4_uca1400_ai_ci` | `utf8mb4_uca1400_ai_ci` | `utf8mb4_general_ci` | Passed through, `FR-SRV-039`. Registered under `FR-SRV-036`. Same root cause as difference 1 |
| 12 | The session read-only state under the spelling `transaction_read_only`; the spelling `tx_read_only` is present on all four | present, `0` | present, `0` | present, `0` | absent, `ERROR 1193 (HY000)` | Fixes the spelling of the read-back at `tx_read_only`, in the fourth entry of `FR-SRV-006` and in `FR-SRV-009`. Reaches no field of the model, so no row in the register of `FR-SRV-036`. Does not engage `FR-SRV-037` — see below |
| 13 | The version a server announces when the connection opens — the initial handshake greeting, sent before authentication and before any statement; the probe of `FR-SRV-002` answers without the prefix on all four | `12.3.3-MariaDB-ubu2404` | `11.8.9-MariaDB-ubu2404` | `11.4.13-MariaDB-ubu2404` | **`5.5.5-10.11.19-MariaDB-ubu2204`** | Nothing today. The model carries the probe's string, per `FR-CTX-031` and `FR-SRV-040`, so the greeting reaches no field of it and takes no row in the register of `FR-SRV-036`. It obliges a test, a diagnostic or a fixture gate that reads a greeting — see below |
| 14 | Acceptance of a `VECTOR` column or a vector index at DDL time | accepted | accepted | rejected, `ERROR 4161 (HY000)` | rejected, `ERROR 4161 (HY000)` | Fixes what the shared DDL of `scripts/mariadb/` may declare: the structure is omitted from it rather than made conditional, per `FR-SRV-029`. Bounds the memberships of `FR-ENV-046`, which names the type for that reason. Reaches no field of the model — no `VECTOR` column was read on any series — so no row in the register of `FR-SRV-036`. **The split falls between `11.4` and `11.8`** — see below |

**Difference 1 is the one that would have falsified `FR-SRV-026`, and it is
now the one `FR-SRV-039` accommodates.** The four servers were given identical
DDL and recorded a different session collation against every view, routine and
trigger, because their own defaults differ. It is not a difference of catalogue
shape, of catalogue meaning, or of presence, so `FR-SRV-024`, `FR-SRV-004` and
`FR-SRV-025` do not reach it. The sixth edition recorded it as a gap in the
taxonomy of `BR-SRV-006` and barred the field from the model while
`OQ-075` was open. That entry is settled: the field
is admitted, its value is carried exactly as the server returns it under
`FR-SRV-039`, and `FR-SRV-026` excepts it. Difference 11 is the same cause
reaching the schema's own collation, and it is registered beside it.

**Difference 3 records the four servers left to themselves, and the fixture
does not leave them that way.** `have_ssl` reports whether a server has TLS
material to offer, and on `10.11` there is none until an administrator
supplies it, where the other three generate a certificate of their own at
every start — both readings are recorded under `FR-CONF-038`. That is the
condition the row was read under, and the same requirement that states it
obliges the fixture of `scripts/mariadb/` to present, at every series of
`FR-SRV-015`, a server whose certificate names the host the project's tests
reach it by. The fixture carries that material and configures it at all four
series, so the row and the fixture describe the same servers in two different
states. The row is the state nothing has configured, which is the state in
which the series differ at all; what the fixture reports under the obligation
is the fixture's own record and is not restated here.

*Bounded claim.* One server of each series was read, at the four patch
releases `FR-SRV-040` records, with no TLS material supplied to any of them by
an administrator or by the fixture — the three that report `YES` do so on a
certificate they generate themselves — and the reading is of `have_ssl` alone.
Nothing was observed about a server outside the window of `FR-SRV-015`, and
nothing here records what a server reports once a certificate is configured
for it, which is the fixture's own state and is recorded with the fixture. The
fifth listener of the fixture, the same `10.11` image started without TLS, is
not a fifth series and is not counted as one.

*Amended in the fourteenth edition: the row names the condition it was read
under.* It read `have_ssl`, unqualified, against a reading taken on
2026-09-10, before the fixture carried TLS material of its own. `FR-CONF-038`
had stated the condition since the eighth edition and this record did not cite
it, so a reader consulting the table for the fixture's state — which is what
an observation record is consulted for — was told the opposite of what that
fixture does. Nothing about the difference itself changes: `10.11` still
offers no TLS until something configures it, and that is the whole of what
makes it a difference between the series. What changes is that the row can no
longer be read as a reading of the fixture as it now stands.

**The other thirteen rows were checked for the same defect and none has it.**
The check is not a re-reading of the rows against each other, which would have
found nothing, but a reading of each row against what the fixture actually
configures: `scripts/mariadb/` gives a server three TLS settings in one
configuration file, the two initialisation scripts, a root password and a
published port, and sets no other server variable on any series; one further
listener, which is not a series, is started with TLS disabled. A row is
exposed to this defect only where something in that list can move the value it
records, and only `have_ssl` is in it. Differences 2, 4, 5, 6, 7, 8, 9, 12 and
13 are properties of the server build — the text a server emits for `SHOW
CREATE TABLE`, the width of a catalogue table, the presence of one, a declared
type, the treatment of a temporary table, the spelling of a session variable,
and the greeting sent before any statement — and no setting in a configuration
file adds a column to `INFORMATION_SCHEMA` or changes what a server announces
on connect. Differences 1 and 11 are the servers' own default collation, which
is visible precisely because the fixture sets no character set and no
collation anywhere; the note above records the cause, and neither row is a
reading of anything the fixture chose. Difference 10 is a statistic over the
fixture's own data, and its row already states the condition it was read under
— identical data — while the fixture sets no variable governing statistics or
query planning. Difference 14 is not a reading at all: it is what a server
accepted at DDL time. The result is a clean negative, and it is recorded with
its method because a sweep that finds nothing is worth only as much as the
reader can see of how far it went.

**Difference 8 splits the window in the middle rather than at one of its
ends, and it is one of three differences in this record that do. It is recorded
for that reason as much as for its content.** Of the fourteen, nine separate
`10.11` from the other three and two separate `12.3` from the other three; the
remaining three — differences 2, 8 and 14 — put `10.11` and `11.4` on one side
and `11.8` and `12.3` on the other. Those three counts are over the fourteen
rows above, as is the fourth in the caution below, and all four are recomputed
whenever a row is added. Nothing structural
follows — `FR-SRV-022` already selects a treatment from the resolved series
rather than from a two-way split, and no requirement in this corpus is written
as *`10.11` against the rest*. What follows is a caution for the reader, the
fixture, and the test of `FR-SRV-029`: **a difference may fall anywhere in the
window**, and code or tests that model the four series as one old server and
three modern ones will be right for nine of the fourteen and wrong for five —
differences 2, 6, 8, 9 and 14.

*Amended in the twelfth edition: two splits fall in the middle of the window,
not one, and the closing count was wrong for a second reason.* The note read
*difference 8 is the only split in this record that does not fall after
`10.11`*. Difference 2 has the identical shape — `10.11` and `11.4` report the
position of `INVISIBLE` one way, `11.8` and `12.3` the other — and it was
recorded before the note was written, so the claim was never true. The closing
count was wrong independently of that: it reused the figure of eleven, which
counts the two splits isolating `12.3` alongside the nine isolating `10.11`,
and a model of one old server against three modern ones is defeated by the
first two as surely as by the two that fall in the middle. Both counts were
restated against the thirteen rows the record then held, and each was made to
name the differences it counts, so that either can be checked without being
recomputed from scratch — which is how the first one came to be wrong. The caution is unchanged, and the second example
strengthens it.

*Amended in the thirteenth edition: three splits fall in the middle of the
window, and all four counts are recomputed over fourteen rows.* Difference 14
has the shape difference 2 and difference 8 have — `10.11` and `11.4` on one
side, `11.8` and `12.3` on the other — and it was observed before either of
them and written down outside this table, which is why neither of the two
recounts reached it. Two counts move: the middle split, from two to three, and
the closing one, from four differences to five. The nine that isolate `10.11`
and the two that isolate `12.3` are unchanged, which is the check that the
recount was done over the rows rather than adjusted by one. The caution is
unchanged again, and a third example of the same shape makes it harder to read
as an exception.

**Differences 8 and 9 do not engage `FR-SRV-037`.** Both change a column's
declared type and neither changes a table's width, so a statement naming a
fixed column list runs unmodified on all four.

**Difference 12 does not engage `FR-SRV-037` either, and the reason is written
down rather than left to be inferred.** `FR-SRV-037` governs the column list
of a statement against `INFORMATION_SCHEMA`, and the read-back names no column
and reads no catalogue table, so the prohibition does not reach it by its
terms. The principle behind it does reach it: name only what every series has.
That is where the two stop agreeing. `FR-SRV-037` offers two answers, and only
its second is available here: the list may be selected from the resolved
series, or restricted to what all four share, and a per-series selection is
shut out because the read-back is one entry of a closed list issued
unconditionally on every connection, and an entry that takes a different form
per server is not one entry. The second answer is also sufficient, which is
why nothing is lost: `tx_read_only` is present on all four series and agrees
with the longer spelling wherever both exist. `FR-SRV-037` is therefore left
as it stands, governing catalogue column lists, and the obligation this
difference creates is discharged inside `FR-SRV-006` and `FR-SRV-009`, which
name the variable. Widening `FR-SRV-037` to cover it was considered and
rejected: it would import a licence to select per series into the one
statement that must be identical on every server, and would make one
requirement say two things.

**Difference 13 is a disagreement between two readings of the same version,
and the reading this specification takes is written down rather than left
implied.** A MariaDB server announces a version when the connection opens,
before authentication and before any statement, and on `10.11` that
announcement carries a `5.5.5-` prefix that the probe of `FR-SRV-002` does not.
Nothing in this corpus reads the announcement: `FR-CTX-031` carries *the string
the probe returns, unaltered*, and `FR-SRV-040` derives `series` from that
string *and from nothing else*, which bars every other reading in terms. The
difference therefore reaches no field of the model and takes no row in the
register of `FR-SRV-036`.

**The reason it takes no row is not difference 12's reason, and the two are
worth keeping apart.** Difference 12 is a session variable the model could not
carry whatever `tpl` did with it. This one touches a field the model does
carry — the `version` key of the `server` object of `FR-CTX-031` — by a
reading the model does not take. An implementation that took the announcement
for the probe's answer would put `5.5.5-10.11.19-MariaDB-ubu2204` in `version`
and derive `5.5` for `series`, and `FR-SRV-020` would then refuse a supported
server as older than the window. The register records what the model
**accommodates**; this is a difference the model **avoids**, by naming the
reading it takes.

**It engages neither `FR-SRV-006` nor `FR-SRV-037`.** The announcement is not a
statement — it arrives unasked when the socket opens, ahead of anything `tpl`
could issue — so the closed list is untouched. It names no catalogue column,
so the column-list rule of `FR-SRV-037` has no subject here. What the
difference obliges falls entirely on work not yet done: a test, a diagnostic
or a fixture gate that reads a greeting cannot assume the two readings agree,
because on one of the four supported series they do not.

*Observed*, on the four series of `FR-SRV-015`, on 2026-09-11:

```text
series  announced when the connection opens  returned by the probe
10.11   5.5.5-10.11.19-MariaDB-ubu2204       10.11.19-MariaDB-ubu2204
11.4    11.4.13-MariaDB-ubu2404              11.4.13-MariaDB-ubu2404
11.8    11.8.9-MariaDB-ubu2404               11.8.9-MariaDB-ubu2404
12.3    12.3.3-MariaDB-ubu2404               12.3.3-MariaDB-ubu2404
```

The left-hand column was read over a plain TCP socket by the fixture's own
gate, which opens the connection, reads the announcement and closes without
authenticating; the right-hand column is the reading `FR-SRV-040` records. The
prefix is the whole of the difference: strip it and the two agree on every
series.

*Bounded claim.* One server of each series was read, at the four patch releases
`FR-SRV-040` records, and nothing was observed about any other build of the
same series. The four servers offer TLS, and a fifth listener of the fixture —
the same `10.11` image started without it — announced the same prefixed string,
so the prefix does not depend on whether TLS is in force; that fifth listener
is not a fifth series and is not counted as one. Nothing was observed about
what a server outside the window of `FR-SRV-015` announces.

**Difference 14 is a difference in what each server will accept, not in what
it reports, and it is the only one of that kind in this record.** A `VECTOR`
column and a vector index are refused outright at DDL time by `10.11` and by
`11.4`, and created without complaint by `11.8` and `12.3`. What it obliges
falls on the fixture: `FR-SRV-029` requires a container buildable at four
server versions whose DDL all four accept, so the structure is absent from the
shared DDL rather than hidden behind a conditional, and the note beside that
requirement records the consequence.

**It takes no row in the register of `FR-SRV-036`, and its reason is neither
difference 12's nor difference 13's.** `FR-SRV-036` fixes three columns for a
row — the field of the model, the treatment applied to it, and what each series
was observed to return — and none of the three can be filled here. No field is
in question: the difference was observed in a server's answer to a `CREATE`,
and the catalogue was never asked about a `VECTOR` column on any series,
because the shared DDL declares none. There is therefore no treatment to
record, and no per-series value to record. Difference 12 is a fact the model
could not carry whatever `tpl` did with it; difference 13 touches a field the
model does carry, by a reading it does not take; this one is upstream of both,
in which databases can exist on each series at all.

**What it bounds, rather than what it obliges.** `FR-SRV-026` requires
identical DDL to produce byte-identical documents across the four series, and
this difference fixes what *identical DDL* may contain: a schema holding a
`VECTOR` column exists on two of the four series and cannot be created on the
other two, so no document can be compared across the window for it. The bound
is on the evidence and not on the requirement — `FR-SRV-026` speaks of a
database all four servers hold, and a database two of them cannot hold is
outside its subject. Where the same bound reaches a requirement it is already
stated: `FR-ENV-046` names `VECTOR` in its bounded claim as a type a supported
server can hold and the fixture's 39 `data_type` values do not cover, and a
`data_type` no observation covers falls through `FR-CTX-018` rather than being
assigned a family.

**It engages neither the closed statement list nor `FR-SRV-037`.** The
statement observed is a `CREATE` issued by the fixture's own initialisation,
not a statement `tpl` issues — `FR-SRV-006` and `FR-SRV-007` between them
allow `tpl` no DDL at all — so the closed list is untouched. It names no
`INFORMATION_SCHEMA` column, so the column-list rule of `FR-SRV-037` has no
subject here.

*Observed*, on the four series of `FR-SRV-015`, on 2026-09-10, while the shared
DDL of `scripts/mariadb/` was being settled:

```text
series  a VECTOR column or a vector index, at DDL time
10.11   ERROR 4161 (HY000): Unknown data type: 'VECTOR'
11.4    ERROR 4161 (HY000): Unknown data type: 'VECTOR'
11.8    accepted
12.3    accepted
```

*Bounded claim.* The observation is of **acceptance**, and of nothing else. One
server of each series was read, at the four patch releases `FR-SRV-040`
records. Nothing was observed about what the catalogue reports for a `VECTOR`
column or a vector index on the two series that accept one, because no such
object survives in the shared DDL to be read: no field list, no `data_type`
value, and no index shape may be written from this. Nothing was observed about
a server outside the window of `FR-SRV-015`, and nothing about which release
first accepted the type — the record states which of the four supported series
accept it, which is what was seen.

**The routine and trigger timestamps were observed to differ, and are not a
difference between the series.** A routine's creation and alteration
timestamps and a trigger's creation timestamp are wall-clock times recording
when each container ran its initialisation scripts, and they differ between
the four captures for that reason alone — two servers of the **same** series
would differ in the same way. They are listed here so that a reader comparing
the four captures does not count them as a fifteenth series difference. Their
consequence is real and is carried elsewhere: `FR-CAT-024` excludes all three
fields, because a document carrying any of them could never satisfy
`FR-SRV-026` against any pair of servers.

*Amended in the eighteenth edition: the count this paragraph guards against was
stale.* It read *a twelfth series difference*, which was right when the record
held eleven rows and has been wrong since the tenth edition added the twelfth.
The number a reader would wrongly reach is one past the rows the table holds,
so it moves with them; it is stated against fourteen now and is recomputed with
the counts beside difference 8. Nothing else in the paragraph changes, and the
observation it records is untouched.

**"A property of the build" is said of nine rows above and of three readings
below, and it does not mean the same thing twice.** The check of the thirteen
rows calls differences 2, 4, 5, 6, 7, 8, 9, 12 and 13 properties of the server
build, and it says so to establish that no setting of the fixture can move
them: the width of a catalogue table is compiled in, and a configuration file
does not add a column to `INFORMATION_SCHEMA`. Those nine are also **entailed
by the series** — every build of `10.11` carries the 22 columns difference 5
records, because that is what the series' own source declares — which is why
each of them is a row. The three readings below are properties of the build in
a second sense: the build carries them and the series does not fix them, so a
second build of the same series may carry others. The first sense makes a row
immune to the fixture; the second keeps a reading out of the table. A reading
is placed by which of the two senses it falls under, and that is settled by the
question `FR-SRV-038` asks — would two servers of the same series differ? The
answer decides where a reading goes, and the phrase decides nothing.

**Three readings of the build were observed, and none of them is a difference
between the series.** They were taken on the second pass, with the version
readings `FR-SRV-040` records, and are recorded under that requirement. They
are named here because this is where an observation that differs across the
four servers and is not a difference between them belongs, per `FR-SRV-038`:

| Reading | How it varied across the four servers |
|---|---|
| The distribution each image was built on, carried as the version string's suffix | `ubu2404` on `12.3`, `11.8` and `11.4`; `ubu2204` on `10.11` |
| The build's source revision, a 40-character hash, read from `version_source_revision` | a distinct hash on each of the four |
| The SSL library string, read from `version_ssl_library` | one reading on `12.3`, `11.8` and `11.4`; a different one on `10.11` |

The three were read again on 2026-09-18, on the same four series and the same
fifth listener, with **what each server returned recorded** — in
`scripts/mariadb/README.md`, which holds the values and is where they stay.
The suffix is carried in the version string of `FR-SRV-040`; the other two are
the variables the table above names.

**None of the three is given a row, and the ground is the classification
alone.** A row of the table above names what each series returned, which
`FR-SRV-038` requires of it, and the reading of 2026-09-18 supplies that for
the source revision and for the SSL library string. What it does not supply is
the entailment a row asserts. It read one server of each series, and the fifth
listener it added is the same `10.11` image and therefore the same build, so
the question a row answers — would two servers of one series return the same
value? — is untouched by it.

*Amended in the twenty-third edition: one of the two grounds for declining a
row is discharged, and the classification carries both readings alone.* This
passage held that four columns of each row could not be filled from anything
this corpus holds, because the observation recorded that the four servers
differ and did not record what any of them returned. The reading of 2026-09-18
records all five, so that ground is gone. The classification is untouched and
is what declines the row: the source revision is a property of the build by
what a source revision is, and the SSL library string is **not established**
as a difference between the series, because its split is still coextensive
with the distribution each image was built on and the run that supplied the
values compared no two builds of one series. Naming the variables is part of
the same correction, and the malloc-library variable read with them is
classified under `FR-SRV-040`, where it belongs: it agrees on all five
servers, so it is neither a row here nor a fourth reading below this table.

*Rejected.* Giving the two readings a row now that their four columns can be
filled. A value is not the thing a row asserts — a row asserts that the series
fixes the value, and the fourth validation rule of the
[README](README.md#maintenance-debt) refuses that credit while the fixture
selects the build without pinning it. The run that supplied the values
strengthens the refusal rather than weakening it: the fifth listener runs the
`10.11` image and returns the same source revision and the same SSL library
string byte for byte, which is what *per build* predicts and what a fifth
series would not. Also rejected: transcribing the readings into this corpus
beside the citation. A per-build value copied here decays the moment the
fixture's upstream tag moves, and it decays silently, where the file that
records the run is the file that is re-run and rewritten; difference 3 above
declines to restate the fixture's own record for the same reason.

**The build is the thing that varies, and for the first two readings that is
settled.** The suffix names a distribution and not a MariaDB fact, and
`FR-SRV-040` has classified it as a property of the build since the seventh
edition and bars it from deriving anything. The source revision is, in that
requirement's own words, *a distinct 40-character hash per build* — and a
series is not a build. The fixture names a series and not a patch release:
`scripts/mariadb/` builds each image from the upstream tag that names the
series, and the four patch releases `FR-SRV-040` records are what those tags
resolved to on the day, which is why every bounded claim in this section states
them as a bound. Two servers of one series at two patch releases are two
builds, and their source revisions differ by what a source revision is.

**The SSL library string splits the four along a line that already has an
explanation, and the explanation is not the series.** It divides `10.11` from
the other three exactly where the distribution each image was built on divides
them, and that distribution is the first reading in the table above —
classified as a property of the build by the requirement that records both.
Two explanations fit the four readings without remainder, the series and the
distribution, and nothing observed separates them, because no two builds of one
series have been compared for this reading. The fourth validation rule of the
[README](README.md#maintenance-debt) decides which way that uncertainty falls:
an observation of what a server reports must record the conditions it was taken
under wherever this project's own fixture can change them, and the fixture
selects the build without pinning it. A reading whose split is coextensive with
a condition of observation the fixture selects is **not established** as a
difference between the series, so it is not counted as one. That is the weaker
of the two claims available and the only one this evidence carries; the
stronger one — that the series fixes it — is what a row would assert.

*Bounded claim.* One server of each series was read, at the four patch releases
`FR-SRV-040` records, on the occasion that recorded those releases, and read
again on 2026-09-18 at the same four patch releases, with the values recorded.
No two servers of one series have been compared for any of the three: the
fifth listener of the fixture is the same `10.11` image and therefore the same
build, so it establishes nothing here, and it is not a fifth series. Nothing
was observed about a server outside
the window of `FR-SRV-015`, and nothing about any other build of the same four
series.

*What would change this, for the SSL library string alone.* Reading it from two
builds of one series that were built on different distributions — the `10.11`
image as the fixture stands and any `10.11` build carrying the other suffix, or
the same image once its upstream tag has moved to one. Two readings that agree
would establish the series as the thing that fixes it, earning it a row whose
four series columns that same observation would fill; two that differ would
settle it as a property of the build and leave this passage as it stands. The
variable is `version_ssl_library`, and the fixture reads it, so this is a
second run of a reading that already exists rather than an ad-hoc read whose
first step is recovering a name. **Nothing in this corpus waits on
it.** No requirement reads any of the three, so it is not an open question and
no entry is opened for it; the index of
[open-questions.md](open-questions.md) stays empty.

**This passage takes the note shapes of a stopped verification and joins
neither family the [README](README.md#writing-conventions) names.** The two
that file names — `FR-ERR-031` and `NFR-PERF-005` — are limits on an
observation that **cannot** be made on the surface available. This one is an
observation that can be made and has not been, which is a weaker thing and is
recorded as one, so that the count of two is not read later as a miscount.

**The counts beside difference 8 do not move.** Nothing enters the table: it
holds fourteen rows, nine of them isolating `10.11`, two isolating `12.3`, and
three splitting the window between `11.4` and `11.8`. Two of the three readings
above have the shape that isolates `10.11`, so promoting either would move the
first of those counts and the closing one; neither is promoted, and this is
recorded so that a later reader can see what a promotion would cost.
`FR-CAT-029` is checked with them and does not move either, because none of the
three is a catalogue field and so none can be a field whose meaning two series
disagree about.

**Everything else matched exactly**, on all four: all 301 columns and every
field of them — types, nullability, defaults, generation expressions,
per-column character sets and collations, comments and attribute strings —
all 68 constraint rows, all fifteen referential constraints, all 54 key
columns, all twenty-four check constraints, all 77 index rows **but for their
cardinality**, all five views but for difference 1, all seven routines and all
six triggers but for difference 1 and the timestamps, the `freight` row of the
schema catalogue, the sequence, and every reading taken as the reduced-grant
reader. `11.8` and `12.3` produced byte-identical catalogue dumps and are
separated only by differences 6 and 9.

**What this evidence is, and what it is not.** It is a comparison of the
**catalogue material the model draws on**, which is the strongest evidence
available before `tpl` exists. It is not the test of `FR-SRV-026`, which
compares the document `tpl` emits and which `FR-SRV-029` still requires. Three
gaps in the fixture bound the claim and are named so that they are not mistaken
for observations: no row holds a non-`NULL` `INET4`, `INET6` or `UUID` value;
no comment carries a supplementary-plane character; and the system-versioned
table uses implicit versioning, so `IS_SYSTEM_TIME_PERIOD_START` and
`IS_SYSTEM_TIME_PERIOD_END` never read `YES` for any column.

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

  | Statement | Purpose | When |
  |---|---|---|
  | `SELECT` against `INFORMATION_SCHEMA.*` | Reading the catalogue, and the privilege probe of `FR-CFG-044` | As the command requires |
  | The server version probe | `FR-SRV-002` | Once, at connection start |
  | The read-only session statement | `FR-SRV-008` | Once, at connection start |
  | One read of the session variable `@@session.tx_read_only`, reading nothing else | The read-back of `FR-SRV-009` | Once, at connection start, in the position `FR-SRV-042` fixes |

  *Amended in the fifth edition.* The fourth entry is new, and closes
  `OQ-046`. `FR-SRV-009` has required a read-back since the second edition and
  the list contained no statement that could perform one, so the strongest
  guarantee this tool makes was stated in two requirements that could not both
  be satisfied. The list gains an entry rather than the read-back being
  performed another way, because there is no other way: confirming that a
  session setting took effect means asking the session what it is.

  The entry is stated as narrowly as the other three, and the narrowness is the
  point. It reads the session read-only state and nothing else — not the whole
  session, not a set of variables, not anything a widening could later be
  argued into. It is issued once, at connection start, immediately after the
  statement whose effect it confirms. It reads no schema, per `FR-SRV-007`, so
  it does not touch the catalogue and does not count toward the query counts of
  `NFR-PERF-001` and `NFR-PERF-002`, which count catalogue queries.

  *Rejected.* Inferring that the setting took effect from the absence of an
  error when it was applied, which is what a list of three would have obliged
  and which is a detection that detects nothing: a server that accepts the
  statement and does not apply it is exactly the case `FR-SRV-009` exists to
  catch. Also rejected: performing the read-back lazily, before the first
  catalogue query rather than at connection start, which would leave a window
  in which `FR-SRV-010` had not yet decided and `FR-ERR-006` could not place
  the check.

  *Amended in the tenth edition: the fourth entry names the variable it reads.*
  The entry said "the session read-only state" and named nothing, and the
  session read-only state has two spellings on MariaDB which are not
  interchangeable across the window. `10.11` has `tx_read_only` and does not
  have `transaction_read_only`; the other three series have both. An
  implementer who chose `transaction_read_only` would fail on one of the four
  supported series, at connection start, on the statement that confirms the
  strongest guarantee this tool makes — and would not be warned by testing,
  because three of the four series accept it. A closed list whose entry admits
  two spellings, one of which cannot run on a supported server, is not closed.
  The variable is named here and in `FR-SRV-009`; the rest of the statement's
  text remains out of scope, and `FR-SRV-007` already bars the listing form of
  the read, so the read-back reads the variable itself.

  *Observed.* Each spelling was read in its own statement, so that a failure
  of one could not mask the other, as `root` over each container's Unix socket,
  on the four series of `FR-SRV-015`, on 2026-09-11:

  ```text
  series  @@session.transaction_read_only  @@session.tx_read_only
  10.11   ERROR 1193 (HY000)               0
  11.4    0                                0
  11.8    0                                0
  12.3    0                                0
  ```

  The server's own message, taken from the diagnostics area so that it
  carries no client decoration, is
  `Unknown system variable 'transaction_read_only'`, with error number `1193`
  and SQLSTATE `HY000`.
  **The two spellings agree wherever both exist, and not only at the default
  value**: read in one session before and after the session was set read
  only, `11.4`, `11.8` and `12.3` returned `0` and then `1` under both
  spellings, and `10.11` returned `0` and then `1` under `tx_read_only` and
  has no other. The session variable listing corroborates it — `10.11`
  exposes `tx_read_only` alone, and the other three expose both. Recorded as
  difference 12 of `FR-SRV-038`.

  *Bounded claim.* One patch release of each series was read — `10.11.19`,
  `11.4.13`, `11.8.9` and `12.3.3`, the versions `FR-SRV-040` records — and
  the two spellings were compared at the two values a session read-only flag
  takes. Nothing was observed about any other release of `10.11`, and
  nothing about a server outside the window of `FR-SRV-015`. The choice is a
  property of the window rather than of MariaDB: it is `10.11`, and only
  `10.11`, that makes it.

  *Rejected.* Selecting the spelling from the series resolved by `FR-SRV-022`,
  in the manner `FR-SRV-037` licenses for a catalogue column list. It is
  unnecessary — `tx_read_only` is present on all four series and, where both
  exist, the two report the same value — and it would give one entry of a
  closed list two forms on different servers, which is the opposite of what a
  closed list is for. Also rejected: issuing `transaction_read_only` and
  falling back on `1193`, which `FR-SRV-023` forbids in terms.

  *Amended in the twenty-fourth edition: the table enumerates and does not
  order, and its fourth row cites the requirement that does.* The rows fix
  **membership** — which four kinds of statement this system may issue — and
  the *When* column fixes each kind's occasion and count. The table is not a
  sequence and cannot be read as one: its first row is the catalogue read,
  which is issued last and as many times as the command requires, so a reader
  taking the rows top to bottom is given the one statement that must follow the
  other three first. `FR-SRV-012` read the rows as a sequence and required a
  test to assert them in that sequence, which puts the version probe before the
  read-only pair and contradicts the condition order of `FR-ERR-006`.
  `FR-SRV-042` closes that, and this note says which kind of table this is so
  that the next reader does not make the same reading. The only order any cell here ever
  carried is the adjacency of the third and fourth rows, decided by the fifth
  edition in the paragraph above on a ground this edition leaves untouched;
  that adjacency is now statements 1 and 2 of `FR-SRV-042`, and the fourth
  row's *When* cell cites it instead of repeating it. No statement enters or
  leaves the list, no count changes, and no row moves.

- **FR-SRV-007**: The system SHALL NOT issue any other statement. It SHALL issue
  no DDL, no DML, no `SHOW`, no statement against any schema other than
  `INFORMATION_SCHEMA`, and SHALL NOT invoke an external process such as a dump
  utility to read structure.

- **BR-SRV-001**: The closed list is the read-only guarantee. It is a property of
  what `tpl` is built to send, it holds whatever the server permits, and it is
  the only part of the promise that prevents rather than detects.

## The order of the connection start

- **FR-SRV-042**: WHEN the system opens a connection, it SHALL issue the three
  connection-start statements of `FR-SRV-006` in this order, and in no other:

  | # | Statement | What it settles |
  |---|---|---|
  | 1 | The read-only session statement of `FR-SRV-008` | That the session refuses a write |
  | 2 | The read-back of `FR-SRV-009`, issued immediately after statement 1 | That the setting took effect |
  | 3 | The version probe of `FR-SRV-002` | The product, the series, and the standing |

  Every `SELECT` against `INFORMATION_SCHEMA.*` — the first entry of
  `FR-SRV-006`, whether a catalogue read or the privilege probe of
  `FR-CFG-044` — follows all three, which `FR-SRV-002` and `FR-SRV-022` already
  require in their own words and this requirement does not restate. This order
  is stated here and nowhere else in this corpus, and every other passage that
  depends on it SHALL cite this requirement rather than repeat it, which is the
  discipline `BR-SRV-005` states for the supported set.

  *New in the twenty-fourth edition, and it closes a contradiction between two
  requirements in force.* `FR-SRV-012` required an integration test to assert
  the three "in the order that table states", and the table of `FR-SRV-006`
  states none: it enumerates the four kinds of statement the closed list
  admits, and its first row is the statement issued last. `FR-ERR-006` fixes
  the order of the three conditions these statements settle — the read-only
  session of `FR-SRV-010`, then the product check of `FR-SRV-003`, then the
  version-window check of `FR-SRV-020` — and a condition cannot be evaluated
  before the statement that produces its evidence, so the two requirements
  ordered the same three statements differently and no test could satisfy both.
  The question the defect turned on is whether that table is an ordered list at
  all. It is not, and `FR-SRV-006` now says so in its own text.

  *Why the session is guaranteed before the server is identified.* The
  read-only promise is the strongest guarantee this tool makes, and the version
  probe is a read. Under any other order the system issues a read on a session
  it has not confirmed refuses a write; and because every connection issues the
  probe, the gap would be in every invocation that opens a connection.
  `BR-SRV-001` and `BR-SRV-002` divide that promise into a part that prevents
  and a part that detects; this order is what keeps the detecting part from
  starting one statement late. It is also the order three requirements in force
  already state for themselves: `FR-ERR-006` for the conditions, `FR-CFG-024`
  for the four steps of `tpl cfg database test`, and `FR-CFG-039` for the five
  fields in which that command reports them. `FR-SRV-002`'s fourth-edition
  amendment states the intent in terms — the strongest guarantee is confirmed
  before the server is characterised — and that requirement permits both
  orders, which is why the intent needed a requirement of its own.

  *The cost, stated rather than discovered.* A server that is not MariaDB
  receives the read-only pair before anything has established what it is. Where
  such a server refuses the `SET`, the system exits `78` under `FR-SRV-010` and
  the `cause` names the read-only session, where `FR-SRV-003` would have named
  the product. Both conditions carry `78`, both name the entry that reached the
  server per `FR-ERR-034`, both leave the catalogue unread, and the caller's
  next step is the same in either case: the entry points at a server `tpl` does
  not serve. What is lost is which of two configuration faults the `cause`
  names, and not the code the caller branches on.

  *Rejected.* Identifying the server before setting its session, which is the
  order `FR-SRV-012` read out of the table of `FR-SRV-006`. The argument for it
  is real and is recorded rather than dismissed. `FR-SRV-022`
  derives the treatment of every known difference from the probe, so probing
  first is the order under which every later statement is issued against a
  characterised server; and it gives the more accurate diagnosis for the
  commonest fault this check meets, an entry pointed at the wrong server. It
  loses on the exchange. What it buys is a `cause` line separating two
  conditions that already share a code, an entry name and a remedy; what it
  spends is the read-only guarantee, on the statement every connection issues.
  Taking it would also reverse the fourth edition's decision in `FR-ERR-006`,
  whose stated ground is that the strongest guarantee is confirmed before the
  server is characterised, and a better-targeted `cause` for one class of
  misconfiguration is not a ground that edition failed to weigh.

  *Also rejected.* Probing first and deferring every verdict until all three
  answers are held. It satisfies the condition order of `FR-ERR-006` literally,
  because the three conditions are then evaluated in that order once the
  answers are in, and it keeps the diagnosis the rejected order buys. It is
  refused for what it does in between: a server the probe has already shown to
  be the wrong product, or to be below the window, still receives the read-only
  pair, so two statements are sent to a server the system has by then
  established it refuses. It buys the `cause` line at a higher price than the
  rejected order above, not a lower one.

  *Also rejected.* Reordering the rows of `FR-SRV-006` and declaring that table
  the order. It assigns no identifier and leaves `FR-SRV-012` as written. It is
  refused because it gives one table two jobs, membership and sequence, which
  is the shape that produced this defect; and because the order is forced by
  `FR-ERR-006`, in [errors-and-exit-codes.md](errors-and-exit-codes.md). A
  table cell cannot carry that derivation, so an edition changing the condition
  order would leave the rows silently wrong. A requirement can carry it, and
  this one does.

## The read-only session

- **FR-SRV-008**: The system SHALL additionally set the session read-only at the
  engine level on every connection it opens.

- **FR-SRV-009**: The system SHALL read `@@session.tx_read_only` back and SHALL
  confirm that the setting took effect. It SHALL read that variable and no
  other spelling of it.

  *Amended in the tenth edition: the variable is named.* The requirement read
  "the session state" and left the spelling to the implementer, which is not a
  free choice: `tx_read_only` is the only spelling present on every series of
  `FR-SRV-015`. The evidence, the bound on it, and the alternatives rejected
  are recorded under the fourth entry of `FR-SRV-006`, which is the statement
  this requirement commands, and the observation is difference 12 of
  `FR-SRV-038`.

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
  The test SHALL expect the four kinds of statement `FR-SRV-006` lists and no
  fifth, and SHALL assert that the three connection-start statements are issued
  exactly once each, in the order `FR-SRV-042` fixes.

  *Amended in the fifth edition.* The list had three entries and now has four,
  per `OQ-046`. A test written against three would fail on the read-back the
  specification requires, which is how a closed list and a test drift apart:
  the list is the contract, and the count in the test is what keeps the list
  from growing by accident.

  *Amended in the twenty-fourth edition: the order is cited, not read out of a
  table that states none.* The clause read "in the order that table states",
  and the table of `FR-SRV-006` enumerates the four kinds of statement the
  closed list admits without ordering them — its first row is the catalogue
  read, which is issued last. Read as a sequence it puts the version probe
  before the read-only session statement and its read-back, which is the
  reverse of the order that the condition order of `FR-ERR-006` forces, so this
  requirement and that one could not both be satisfied by one test, and the
  test mandated here could not be written. `FR-SRV-042` states the order once,
  and this requirement asserts it without stating it. What the test
  expects is otherwise unchanged: the four kinds, no fifth, and the three
  connection-start statements exactly once each, observed on the server under
  `BR-SRV-003`.

- **FR-SRV-013**: The read-back of `FR-SRV-009` SHALL be verified in both of
  its outcomes, each by the test form that can reach it.

  The **confirming** outcome — the setting taking effect — SHALL be verified by
  an integration test that observes, on the server, that the read-back is
  issued and that the value the session reports confirms the setting. That test
  SHALL be executed against every series of `FR-SRV-015`, and it is an
  observation made from outside the process, per `BR-SRV-003`.

  The **failing** outcome — a read-back that does not confirm the setting —
  SHALL be verified in process, and not by an integration test, through a seam
  on the terms of `FR-ERR-031`: reachable only from within the system's own
  test configuration, reachable from no invocation of the binary the project
  distributes, and appearing in no help text, in the JSON command tree of
  `FR-HELP-016`, or in the command tree of `FR-CLI-002`. The seam SHALL present
  the read-back with an answer that does not confirm the setting, and the test
  SHALL assert that the system produces the condition of `FR-SRV-010` in the
  half of it this read-back decides, and issues no catalogue statement. For
  that outcome alone this requirement takes an exception to `BR-SRV-003`, which
  states it in its own text.

  *Amended in the tenth edition: the test is bound to every series.* The
  requirement named no server, and on this requirement the series is the whole
  of what is at risk: the spelling `FR-SRV-009` names is discriminated by
  exactly one series of the window, and a test that runs anywhere else passes
  under either spelling. A verification that cannot fail on the defect it
  exists to catch is not a verification. The binding is the one `FR-SRV-029`
  already states for the equivalence test, in the same words.

  *Amended in the twenty-eighth edition: the failing outcome is produced in
  process, because nothing outside the process can produce it.* The requirement
  demanded one integration test exercising both outcomes, and neither route to
  the second existed. **No server produces it**, which the observation below
  records. **No admissible seam reaches it either**: a seam on `FR-ERR-031`'s
  terms is reachable only from within the system's own test configuration, and
  an integration test drives the binary the project distributes, which carries
  no such seam. The two halves of this requirement could therefore not both be
  satisfied as written. What yields is the **form** of the failing half's test,
  and nothing else: that outcome is still verified, the seam is authorised
  here, in this requirement's own text, rather than left to be inferred, and
  the confirming half is unchanged — still observed on the server, and still
  bound to every series of `FR-SRV-015`. The tenth edition's binding is
  undiminished and now attaches where it can act, on the half that reaches a
  server; the failing half reaches none, so no series can be named for it. This
  is the resolution `FR-SRV-035` took in the eighth edition for the same
  collision, in the same shape — the requirement names the seam, names the test
  form, and states what the form does not establish.

  *Observed.* Against the fixture of `scripts/mariadb/`, on 2026-09-20, by the
  pass that wrote the confirming half. Three conditions were tried, and under
  each of them the read-back still confirmed the setting:

  | Tried | What the session reported |
  |---|---|
  | An open transaction before the read-only session statement | The statement is accepted on all four series of `FR-SRV-015`, and the read-back still answers the enforced value. MariaDB does not refuse it |
  | The same as the reduced-grant reader | Accepted; the read-back still answers the enforced value |
  | A server already read only at the global level | `@@global.read_only` is `0` on the fixture, and setting it governs the global state rather than whether a **session** setting took effect, so it cannot make the read-back disagree |

  *Bounded claim.* Three conditions, on the fixture as it stood on that date,
  the first of them on each of the four series; nothing was observed about a
  server outside the window of `FR-SRV-015`, and no exhaustive search of server
  configurations was made. One further candidate was reasoned against rather
  than observed, and is recorded so that it is not tried again: a server-side
  `init_connect`, which the server runs **before** the client's own statements
  and which therefore cannot reach a setting the client makes after it.

  *Nothing is owed to the record of differences.* The four series behaved
  identically under the condition that was tried on all four, so this occasion
  observed **no** difference between them and owes no row to `FR-SRV-038` and
  no row to the register of `FR-SRV-036`. The negative is written here, beside
  the observation, so that a sweep of the corpus for observations recorded
  outside the record that owns them — the check the thirteenth edition
  added — meets an answer rather than a question.

  *The exception, stated against the rule it excepts from.* `BR-SRV-003`
  requires all three of `FR-SRV-012` through `FR-SRV-014` to be observed from
  outside the process, on the server, and this requirement is one of the three.
  The exception is the failing outcome and nothing besides. Everything this
  requirement promises about the statement the process **sends** — that the
  read-back is issued, in the spelling `FR-SRV-009` names, in the position
  `FR-SRV-042` fixes — lies in the confirming half, and is observed on the
  server on every series. What the failing half verifies is not what the
  process sends but what it **does with the answer it receives**, and no server
  can show that: a server that accepts the read-only session statement and does
  not apply it is exactly the case `FR-SRV-009` exists to catch, and no
  supported MariaDB behaves that way. `BR-SRV-003` yields for that clause
  alone, in its own text, as `BR-ERR-001` does for `70`.

  *Consequence, stated plainly.* No invocation of the distributed binary is
  observed refusing on a read-back that did not confirm, and no server is
  observed producing one. What is observed is the decision itself, in process,
  and separately the step from a condition to the exit status it carries —
  `78` (`EX_CONFIG`), which `BR-ERR-001` obliges to have an integration test
  and which other producing conditions of that code reach from an invocation.
  The composition of the two is reasoned rather than executed. That is weaker
  than the confirming half of this requirement, and it is the price of the
  condition being one no server produces — which is the same limit
  `FR-ERR-031` states for `70` and `FR-SRV-035` for the marked read.

  *What would change this.* A server inside the window that accepts the
  read-only session statement and does not apply it. Were one ever observed,
  the failing outcome would be producible from outside the process, the second
  half of this requirement would return to the form of the first, and the
  exception to `BR-SRV-003` would be withdrawn with the seam.

  *Rejected.* Withdrawing the demand for the failing outcome and leaving it
  with no test of any kind. It is the branch of `FR-SRV-010` this read-back
  decides, and it is the detecting half of the strongest guarantee this tool
  makes, per `BR-SRV-002`: the condition `FR-SRV-009` exists to catch is
  precisely the one no fixture can stage, so a path that is never exercised
  would first run on the day it matters. The ground is `BR-ERR-001`'s for `70`
  — a code no test exercises is a code nobody has confirmed the binary can
  return — and it applies with more force here, because the terms on which a
  seam is admitted are already stated, in `FR-ERR-031`, and this corpus has
  accepted them twice.

  *Also rejected.* Naming a fixture condition concretely. The three above were
  tried and none produces it, and the reason is not that the right server
  setting has yet to be found: a server that accepts the statement and does not
  apply it is **defective**, not configured, so there is no state a conforming
  MariaDB can be put into that produces the outcome. A requirement that named
  such a condition would mandate a test the fixture can never run, which is the
  defect this amendment exists to end.

  *Also rejected.* A stand-in between the reader and the server, rewriting the
  read-back's answer. It is possible in principle — `FR-SRV-041` states that a
  server determined to pass as MariaDB will pass — and it is refused on
  `FR-SRV-035`'s ground: it obliges the project to implement and maintain
  enough of the MariaDB wire protocol to rewrite one result set, for one
  assertion, and to keep it true across four series. It also buys less than it
  looks: what the reader would then be observed against is an artefact this
  project wrote, so the observation satisfies `BR-SRV-003`'s letter — outside
  the process — while abandoning its substance, which is that the observation
  is made on a server.

- **FR-SRV-014**: The connection count of an invocation SHALL be as fixed by
  `NFR-PERF-004`, and SHALL be verifiable from the server side.

- **FR-SRV-029**: The equivalence of `FR-SRV-026` SHALL be verified by an
  integration test executed against every series of `FR-SRV-015`, and the
  refusal of `FR-SRV-020` by an integration test against at least one series
  outside it.

  *Consequence for the fixture, now discharged.* The container of
  `scripts/mariadb/` is buildable at four server versions rather than one, and
  its DDL is DDL that all four accept. Two structures could not be created on
  all four and are absent from the shared DDL rather than hidden behind a
  conditional: a `VECTOR` column or index, rejected by `10.11` and `11.4` with
  `ERROR 4161`, and a `SET` member containing a comma, rejected by all four
  with `ERROR 1367` — the second is not a series difference at all but a
  property of the type, recorded in `FR-CAT-034`. The first **is** a difference
  between the series: it is difference 14 of `FR-SRV-038`, where that
  requirement obliges it to be, and it was written here alone until the
  thirteenth edition.

  *Still owed.* The test itself. The observation recorded under `FR-SRV-038`
  compares catalogue dumps, not the documents `tpl` emits, so it is evidence
  for `FR-SRV-026` and not the verification `FR-SRV-026` requires. That test
  cannot be written until `tpl` can emit a document.

- **FR-SRV-035**: The marked read of `FR-SRV-031` SHALL be verified by a test
  that presents the reader with a series above its own window, and that asserts
  that the read completes without error and that `standing` is
  `newer_than_supported`. The seam by which the test narrows the window SHALL
  be the seam of `FR-ERR-031`: reachable only from within the system's own test
  configuration, and reachable from no invocation of the binary the project
  distributes. It SHALL NOT appear in any help text, in the JSON command tree
  of `FR-HELP-016`, or in the command tree of `FR-CLI-002`.

  *Rationale.* No such server exists to point the test at — by construction, the
  window contains the newest one there is — so the test must narrow the reader's
  window rather than widen the server. Keeping the seam off the published
  surface is the discipline `FR-ERR-031` already applies to the deliberate
  trigger for `70`, and for the same reason: a test hook that is reachable from
  the command line is a flag, and `FR-SRV-020` rejected a flag that overrides
  the window.

  *Amended in the eighth edition: the seam is the one `FR-ERR-031` names, and
  the assertion on the exit code is what yields.* This requirement called for
  an integration test asserting exit `0`, which only a test that runs the
  binary can observe, while its own rationale ruled out every seam such a test
  could reach. The two halves could not both stand. The rationale is the half
  that survives, because it is the argument `FR-SRV-020` had already made: the
  seam moves inside the process, and what the test asserts is that the read
  completes without error and that the document carries
  `standing: newer_than_supported`. The step from a read that completes to
  exit `0` is `BR-CLI-004`, and it is observed by the integration test of
  every successful command; nothing is left unobserved but the composition of
  the two, which `FR-ERR-031` states plainly for `70` and which is the same
  limit here.

  *What this does not except from.* `BR-SRV-003` requires an observation made
  outside the process, on the server, and it names `FR-SRV-012` through
  `FR-SRV-014` — the three promises about what the process **sends**. This
  requirement is not among them and takes no exception to it: what is verified
  here is what the reader **emits** into the document, and narrowing the window
  changes neither the statements of `FR-SRV-006` nor their count. The one
  exception taken is `BR-ERR-001`'s, and it is taken once, in `FR-ERR-031`.

  *Rejected.* A server, or a stand-in for one, reporting a series above the
  window. `FR-SRV-041` states that a server determined to pass as MariaDB will
  pass, so such a stand-in is possible in principle and would make this an
  outside observation. It was rejected because it obliges the project to build
  and maintain an impostor of the MariaDB wire protocol for one assertion, and
  because what it would establish — that the reader believes the version the
  server reports — is `FR-SRV-040`, already verified against four real
  servers.

- **BR-SRV-003**: All three of `FR-SRV-012` through `FR-SRV-014` are
  observations made from outside the process, on the server. The failing
  outcome of `FR-SRV-013` is the one exception, stated here rather than left to
  be inferred: no arrangement outside the process can present the read-back
  with an answer that does not confirm the setting, so that outcome alone is
  verified in process, through the seam `FR-SRV-013` authorises on the terms of
  `FR-ERR-031`. A promise about what a process sends that can only be checked
  by reading that process's own source is not a promise a caller can rely on.

  *Amended in the twenty-eighth edition: the one exception is named.* The rule
  required all three to be observed on the server while `FR-SRV-013` demanded
  an outcome no server produces, so the two could not both be satisfied and the
  test `FR-SRV-013` mandated could not be written. This rule yields, for that
  one clause. It is unchanged over `FR-SRV-012`, over `FR-SRV-014`, and over
  the confirming outcome of `FR-SRV-013`, which carries the whole of what that
  requirement promises about the statement the process **sends** and is
  observed on the server at every series of `FR-SRV-015`. The ground stated
  above is untouched: what yields is not a claim about what is sent, but the
  verification of what the process does with the answer it receives, which no
  server can show. `FR-SRV-013` states the exception against this rule, records
  the three fixture conditions that failed to produce the outcome, and states
  what the in-process form does not establish.

  *What this rule reaches, stated in the eighth edition.* It reaches the three
  requirements it names, which are promises about the statements `tpl` sends
  and the connections it opens. It does not reach `FR-SRV-035`, which is a
  promise about what the reader emits into the document and which is verified
  through the in-process seam of `FR-ERR-031`, because no arrangement outside
  the process can present the reader with a series above its own window. The
  distinction is the one this rule already draws: a claim about what is sent is
  checkable on the server, and a claim about what is emitted is checkable in
  the bytes, and neither is checkable by reading the source.

  *The platform gap of the eleventh edition does not reach this rule, checked
  and recorded so that it is not re-opened.* Both instruments this rule relies
  on are server-side — the statements the server receives and the connections
  it accepts — and `NFR-PERF-007` marks both available on all four targets of
  `NFR-PERF-018`. What the eleventh edition found missing on macOS is the third
  instrument of that rule, the syscall trace that records the files a process
  opens, and none of `FR-SRV-012` through `FR-SRV-014` depends on it: the three
  are promises about statements and connections, not about files. The three
  requirements are unchanged, and so is this rule.

## Dependencies

- [catalogue-coverage.md](catalogue-coverage.md) — the fields whose absence
  `FR-SRV-004` turns into `null`, and `FR-CAT-029`, the ambiguous-meaning
  exclusion list that `FR-SRV-025` writes into.
- [context-document.md](context-document.md) — `FR-CTX-031` and `FR-CTX-034`,
  the shape of the `server` object and of its `standing` field; and
  `FR-CTX-012`, the discriminant that a non-MariaDB server would invert.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the other
  reason a read can return less than the model defines.
- [performance-requirements.md](performance-requirements.md) — `NFR-PERF-001`,
  `NFR-PERF-002` and `NFR-PERF-004`, the query-count and connection invariants
  that `FR-SRV-023` relies on.
- [security.md](security.md) — `BR-SEC-002`, the cross-cutting statement of the
  read-only promise, and `FR-SEC-021`, the transport guarantee that difference
  3 of `FR-SRV-038` bounds.
- [configuration-model.md](configuration-model.md) — `FR-CONF-038`, the
  behaviour of the five TLS modes against a server that offers TLS and one
  that does not.
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

**None.** The seven entries this file carried are closed and are listed under
[Closed](open-questions.md#closed):

| Entry | Closed by |
|---|---|
| [OQ-042](open-questions.md#closed) | `FR-SRV-040` and `FR-SRV-041` |
| [OQ-044](open-questions.md#closed) | `FR-SRV-020`, with `FR-SRV-021` |
| [OQ-045](open-questions.md#closed) | `FR-SRV-038`, and the register of `FR-SRV-036` |
| [OQ-046](open-questions.md#closed) | The fourth entry of `FR-SRV-006`, with `FR-SRV-012` as amended |
| [OQ-073](open-questions.md#closed) | `FR-SRV-031` through `FR-SRV-033`, with `BR-SRV-008`, `BR-SRV-009` and `FR-CTX-034` |
| [OQ-074](open-questions.md#closed) | `FR-SRV-002` as amended, with `FR-SRV-034` |
| [OQ-075](open-questions.md#closed) | `FR-SRV-039`, with `BR-SRV-006` and `FR-SRV-026` as amended |

`OQ-042` was the last entry the corpus held open, and it is the one entry
here that closes on a **limit** rather than on an answer. `FR-SRV-040` fixes
what the probe returns — the form of the version string, the readings that
carry it, and the derivation of `series` — and `FR-SRV-041` fixes the
necessary condition for a server to be MariaDB and states, in the requirement
itself, that the condition is not a sufficient one: a server determined to
pass as MariaDB will pass. That limit is now specification rather than an
entry, because no observation this project can make would close it.
