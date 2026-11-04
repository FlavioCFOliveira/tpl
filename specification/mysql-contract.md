---
title: MySQL Contract
status: approved
last-reviewed: 2026-11-04
related: [configuration-model.md, server-contract.md, sqlite-contract.md, catalogue-coverage.md, context-document.md, privileges-and-completeness.md, security.md, cfg-commands.md, performance-requirements.md, errors-and-exit-codes.md, template-environment.md, open-questions.md]
---

# MySQL Contract

## Overview

`tpl` reads a database through a **provider**, named by the `provider` key of
the entry, per `FR-CONF-051`. [server-contract.md](server-contract.md) states
what `tpl` requires of a MariaDB server and promises in return, and
[sqlite-contract.md](sqlite-contract.md) states the same for a SQLite database
file. This file states it for the `mysql` provider: the product and the
release series it supports and the rule that selects them, the refusal of a
server outside them, the read-only promise and how it is imposed, the
connection and its transport, the closed list of statements and of
`INFORMATION_SCHEMA` tables a read draws on, the part of the database a read
covers, how the MySQL catalogue fills the model of
[catalogue-coverage.md](catalogue-coverage.md) and the document of
[context-document.md](context-document.md), the divergences between MySQL and
MariaDB that reach the read, and the fixture its tests use.

The model is the same model. A MySQL read fills the objects and properties
those two files define and no others; where MySQL reports nothing for a
property, this file says so, and where MySQL offers a field the model has no
property for, this file says that it is not carried. A template written
against the model renders from a MySQL read on the same principles as from a
MariaDB read, and `database.provider`, per `FR-CTX-043`, is how a template
tells the providers apart.

## Scope

In scope: the `mysql` provider and what selects it; the supported release
series, the criterion that admits them, the series it admits today, and how
the set is maintained; the version probe, the product check and the outcomes
at the edges of the window; what the model owes a difference between two
supported MySQL series; the closed statement list, the closed list of
`INFORMATION_SCHEMA` tables, and the order of the connection start; the
read-only session and its read-back; connection, authentication and TLS where
they differ from a `mariadb` entry; the part of the database a read covers and
what it excludes; the source of every model property for a MySQL read; the
record of divergences between MySQL and MariaDB that reach the read; the
completeness of a MySQL read; its requirements of form; its verification; and
the fixture of `scripts/mysql/`.

Out of scope: the `provider` key and the keys an entry takes, which belong to
[configuration-model.md](configuration-model.md); the commands that write
them, which belong to [cfg-commands.md](cfg-commands.md); the shape of the
document, which belongs to [context-document.md](context-document.md); the
choice of the database driver, which is an architecture decision under
`FR-CONF-036` and `FR-MYSQL-022`; and the text of any statement, save the one
variable the read-back of `FR-MYSQL-018` must name.

Every requirement of [server-contract.md](server-contract.md) governs a
`mariadb` entry and does not reach a `mysql` entry. Where this file adopts one
of them for MySQL, it cites it and states what takes the place of its
MariaDB-specific terms. Nothing a `mariadb` entry or a `sqlite` entry does
changes on account of this file.

## Actors

- **MySQL server**, which must be a supported series and must accept the
  read-only session.
- **`tpl`**, which must satisfy the statement restriction.
- **Database administrator**, who can verify both from the server side.
- **Oracle's support policy for MySQL**, the external authority the supported
  set follows and which no party to this specification controls.

## Sources, and the pending-observation marker

Every statement this file makes about MySQL's behaviour rests on one of two
grounds, and **neither of them is an observation**:

1. **MySQL's own documentation**, a published external authority in the sense
   of the third provenance of the [README](README.md#provenance), checked on
   2026-11-04: the reference manuals of the three supported series, at
   `https://dev.mysql.com/doc/refman/8.4/en/`,
   `https://dev.mysql.com/doc/refman/9.7/en/` and
   `https://dev.mysql.com/doc/refman/26.10/en/`; the release notes, at
   `https://dev.mysql.com/doc/relnotes/mysql/<series>/en/`; the download page,
   at `https://dev.mysql.com/downloads/mysql/`; the end-of-life notices, at
   `https://www.mysql.com/support/eol-notice.html`; and the support tiers, at
   `https://www.mysql.com/support/`. A page is cited below by its path under
   the manual of the series named, as `refman/8.4/en/<page>.html`.
2. **Prior art**, which informs how and never what: the MySQL inspector of
   `ariga/atlas`, `sql/mysql/inspect.go` and `sql/mysql/driver.go` at the
   public source of its default branch, and
   `sql/mysql/internal/mysqlversion/mysqlversion.go` at tag `v0.21.1`, read on
   2026-11-04. It is cited where it bears on a choice this file makes, and it
   establishes no fact here.

**No statement of MySQL behaviour in this file has been observed.** The
fixture of `FR-MYSQL-044` does not exist yet, and rmp `#326` observes each
statement on every series of `FR-MYSQL-003` through it. Every such statement
carries a note headed ***Pending observation.***, the marker the
[README](README.md#writing-conventions) defines and
[glossary.md](glossary.md#pending-observation) names: the note says what the
documentation states, cites the page, and says what observation discharges it.
A table whose rows are documented facts carries one such note for all of its
rows. The marker qualifies the ground of a fact and not the force of a
requirement: every requirement here is in force, and a requirement whose
marked fact an observation contradicts is amended, as a requirement resting on
the fourth provenance is amended by a second observation.

What this file states about MariaDB is cited from the requirement of this
corpus that records it, every one of them an observation of the fixture of
`scripts/mariadb/`, and carries no marker.

*Added in the seventy-third edition,* for rmp `#324`.

## The provider

- **FR-MYSQL-001**: The system SHALL read an entry whose provider is `mysql`,
  per `FR-CONF-051`, under this file, and under no requirement of
  [server-contract.md](server-contract.md) save those this file adopts by
  citing them. A `mysql` entry SHALL take exactly the keys a `mariadb` entry
  takes and SHALL refuse the keys a `mariadb` entry refuses, per
  `FR-CONF-052`.

  *Provenance.* The user's requirements for rmp `#324`, relayed on
  2026-11-04: `tpl` reads the catalogue of a MySQL database and renders
  templates from it on the principles it applies to MariaDB; MySQL is a
  provider of its own, which the `provider` value of an entry selects, by the
  mechanism that already distinguishes `mariadb` from `sqlite`; and nothing
  about MariaDB support changes.

  *Rationale.* `FR-SRV-003` refuses MySQL through a `mariadb` entry because
  three divergences of the catalogue would make a MariaDB reader silently
  wrong on it: the quoting of a default, the shape of the check-constraint
  table, and the absence of a generated-column flag. A provider of its own
  reads MySQL with a reader written for MySQL's catalogue, and leaves that
  refusal standing for the entry it protects.

  *Rejected: recognising a MySQL server inside a `mariadb` entry and switching
  the reader.* The server would choose the provider, where the user's
  requirement is that the configuration chooses it; and a `mariadb` entry that
  reaches a MySQL server, refused today under `FR-SRV-003`, would be read
  instead, which changes what a `mariadb` entry does. *Rejected: taking the
  provider from the scheme of a DSN.* `FR-CONF-010` treats `mysql://` and
  `mariadb://` as equivalent, and an entry written `mysql://` with no
  `provider` key is a `mariadb` entry, per `FR-CONF-051`; every such entry
  would change provider, and the root `README.md` shows one.

  *Added in the seventy-third edition,* for rmp `#324`.

## The supported version window

- **FR-MYSQL-002**: For a `mysql` entry, the system SHALL support MySQL and
  no other product, and SHALL support exactly those MySQL release series that
  satisfy **both** of the following conditions:

  1. the series is the newest series Oracle has released in its
     [major family](glossary.md#series--supported-series--major-family); and
  2. the series is under Oracle **Premier Support** or **Extended Support** —
     which, for an [Innovation release](glossary.md#lts-release--innovation-release),
     lasts until the next Innovation release is published.

  A series, a major family, an LTS release and an Innovation release have the
  meanings [glossary.md](glossary.md) gives them. Series SHALL be compared by
  version number — major number first, then minor number, each compared as an
  integer — so that `8.4` is older than `9.7`, `9.7` older than `26.7`, and
  `26.7` older than `26.10`. A point release does not enter the comparison:
  `8.4.13` and `8.4.2` are one series, as `FR-SRV-021` states for MariaDB.

  *Provenance.* The user's requirement for rmp `#324`: only the latest release
  of each MySQL major series still under Oracle support, in the user's words
  "8.x.x, 9.x.x, …". The first condition is *the latest of each major series*;
  the second is *still under Oracle support*.

  *Why Premier and Extended Support, and not Sustaining Support.* Oracle's
  support page gives Premier Support "MySQL maintenance releases, updates,
  fixes", Extended Support "error correction", and Sustaining Support "No new
  releases, no new fixes", at `https://www.mysql.com/support/`, checked
  2026-11-04. Sustaining Support is indefinite: read as support, it would
  admit `5.7` and every series before it, which the user's examples exclude.
  The two tiers admitted are the two under which a series still changes, which
  is the ground `FR-SRV-001` gives for following MariaDB's community
  maintenance.

  *Why the newest series of a family.* It is the user's rule, and it removes
  what the second condition alone would admit: until 2026-04-21, when it
  entered Sustaining Support, `8.0` was supported beside `8.4`, and a
  criterion of support alone would have admitted both.

  *Why Innovation releases are admitted.* Oracle supports an Innovation
  release, until the next one, as it supports an LTS release: "All LTS and
  Innovation releases include bug and security fixes, and are considered
  production-grade quality", and "Innovation releases are supported until the
  next Innovation release", per `refman/26.10/en/mysql-releases.html`, checked
  2026-11-04. A MariaDB rolling release is excluded because it receives no
  maintenance after GA, per `FR-SRV-016`; a MySQL Innovation release receives
  it while it is current, so the second condition admits it while it is
  current and drops it when the next ships.

  *A major family under calendar versioning.* MySQL numbered its releases
  sequentially up to `9.7`, and numbers them `YY.M.P` from `26.7.0`, per
  `refman/26.10/en/mysql-releases.html`. Under sequential versioning a major
  family is the leading number. Under calendar versioning this file takes it
  to be the compatibility lineage the manual defines — the Innovation releases
  before an LTS release and the LTS release that closes them — because that is
  what a sequential major number was: `9.0` to `9.6` and the `9.7` LTS release
  that closed them. Whether that reading is the user's is
  [OQ-076](open-questions.md#index). Every reading that question names selects
  the same series on 2026-11-04, so the table of `FR-MYSQL-003` does not wait
  on it.

  *Rejected: the window of `FR-SRV-001`, three most recent families under
  maintenance.* It is MariaDB's criterion, not the user's, and it would admit
  `8.0` beside `8.4` while both were maintained.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-003**: The series that satisfy `FR-MYSQL-002` SHALL be the
  following, as verified on the date recorded below:

  | Major family | Series | Track | First release of the series | Latest release, as of 2026-11-04 | Support |
  |---|---|---|---|---|---|
  | the calendar lineage opened by `26.7.0` | `26.10` | Innovation | `26.10.0`, 2026-10-20 | `26.10.0`, 2026-10-20 | until the next Innovation release |
  | `9` | `9.7` | LTS | `9.7.0`, 2026-04-21 | `9.7.4`, 2026-10-20 | five years of Premier and three of Extended Support from general availability |
  | `8` | `8.4` | LTS | `8.4.0`, 2024-04-30 | `8.4.13`, 2026-10-20 | five years of Premier and three of Extended Support from general availability |

  *Source.* The release notes of each series, at
  `https://dev.mysql.com/doc/relnotes/mysql/26.10/en/`,
  `https://dev.mysql.com/doc/relnotes/mysql/9.7/en/` and
  `https://dev.mysql.com/doc/relnotes/mysql/8.4/en/`, for the releases and
  their dates; the download page, which offers `26.10.0`, `26.7.0`,
  `9.7.4 LTS`, `8.4.13 LTS` and `8.0.46`; the preface of the 26.10 manual, "the
  Reference Manual for the MySQL Database System, for the 26.10.0 Innovation
  release", and of the 9.7 manual, which "covers the 9.7 LTS series"; and
  `refman/8.4/en/mysql-releases.html`, for "An LTS series follows the Oracle
  Lifetime Support Policy, which includes 5 years of premier support and 3
  years of extended support". **Verified 2026-11-04.**

  *Derived, and not read.* The support column of an LTS series states the
  durations the manual gives and no date. Counted from the first release of
  each series, they end Premier Support in 2029 for `8.4` and in 2031 for
  `9.7`, and Extended Support in 2032 and 2034. The Oracle Lifetime Support
  Policy document that would state the days refused retrieval, with HTTP 403,
  on 2026-11-04, so the years are a count and the days are not recorded.

  *Worked examples of the two conditions.* Each of these is unsupported:

  | Series | Excluded because |
  |---|---|
  | `26.7` | the next Innovation release, `26.10.0`, was published on 2026-10-20, so its support ended, and it is no longer the newest series of its family |
  | `9.0` to `9.6` | none is the newest series of family `9`, and each Innovation release's support ended with the next |
  | `8.0` | it is not the newest series of family `8`, and "as of April 21, 2026, MySQL 8.0 is covered under Oracle Sustaining Support", per the end-of-life notices |
  | `5.7` | "as of October 25, 2023, MySQL 5.7 is covered under Oracle Sustaining Support", per the same page |

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-004**: The set of `FR-MYSQL-003` SHALL change by the two
  conditions of `FR-MYSQL-002` and by nothing else, and its size SHALL NOT be
  fixed by this specification. Three series across three families is what the
  criterion admits on the verification date, not a quantity to preserve.

  | Event | Effect |
  |---|---|
  | Oracle publishes an Innovation release | It is the newest series of its family and enters; the Innovation release it succeeds, where that is still in the set, leaves; no LTS release leaves |
  | Oracle publishes an LTS release | It is the newest series of the family it closes and enters; the Innovation release before it, where that is still in the set, leaves |
  | A series leaves Extended Support | It leaves, and its family with it, since no newer series of that family exists to take its place |

  *Rationale.* It is `FR-SRV-017` and `FR-SRV-018` for MySQL's release model.
  Innovation releases make the set move faster than MariaDB's: MySQL
  published Innovation releases on 2026-07-28 and 2026-10-20, per the release
  notes of the 26.10 series.

  *Added in the seventy-third edition,* for rmp `#324`.

- **BR-MYSQL-001**: The criterion of `FR-MYSQL-002` is the requirement, and
  the table of `FR-MYSQL-003` is its instance on one date, which is part of
  the table. A maintainer who finds the table wrong has found it stale, and
  re-derives it from the criterion and the sources; the criterion is never
  amended to keep an old table true. The set is stated once, there: every
  other passage of this corpus that depends on which MySQL series are
  supported SHALL cite `FR-MYSQL-002` or `FR-MYSQL-003` and SHALL NOT repeat a
  version number, which is `BR-SRV-004` and `BR-SRV-005` for MySQL. The set of
  `FR-SRV-015` is MariaDB's and is never read as this one.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-005**: The table of `FR-MYSQL-003` SHALL carry its sources and
  its verification date, and SHALL be re-verified against those sources before
  every release of `tpl`.

  *The next scheduled change.* The table is known to become wrong when Oracle
  publishes the next Innovation release, which removes `26.10` and admits the
  new series. No source read on 2026-11-04 dates it; the two Innovation
  releases before it were published 84 days apart.

  *Rationale.* It is `FR-SRV-019` for MySQL, and it binds more often: a
  release of `tpl` built on a stale table admits a series that has left
  support and refuses one that has entered it.

  *Added in the seventy-third edition,* for rmp `#324`.

## The product and the version

- **FR-MYSQL-006**: WHEN the system opens a connection for a `mysql` entry,
  it SHALL determine the server's version by the version probe, in the
  position `FR-MYSQL-017` fixes and before any statement against
  `INFORMATION_SCHEMA`. The string the probe returns SHALL be taken to have
  the form `<major>.<minor>.<patch>`, three decimal integers separated by
  dots, optionally followed by `-<suffix>`. The `server` object of
  `FR-CTX-031` SHALL carry the string unaltered as `version`; `series` SHALL
  be derived from it as `<major>.<minor>`, and from nothing else; and the
  suffix SHALL NOT be used to derive anything. `standing` SHALL be set under
  `FR-MYSQL-010`.

  ```json
  {"server":{"version":"8.4.13","series":"8.4","standing":"supported"}}
  ```

  *Pending observation.* The manual states that the `version` variable is
  "The version number for the server. The value might also include a suffix
  indicating server build or configuration information. `-debug` indicates
  that the server was built with debugging support enabled"
  (`refman/8.4/en/server-system-variables.html`). The 26.10 manual adds a
  four-part identity read through `VERSION_WITH_UPDATE()`, and states that
  "Existing interfaces, including `VERSION()`, `version`, the classic
  handshake, numeric version IDs, and existing C APIs, retain three-part
  compatibility semantics" (`relnotes/mysql/26.10/en/news-26-10-0.html`); no
  statement of `FR-MYSQL-015` reads the four-part form. The example above is
  the form the manual describes for the latest `8.4` release, not a string any
  server was seen to return. rmp `#326` records the probe's answer, verbatim,
  on every series of `FR-MYSQL-003`.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-007**: IF the version string the probe returns contains the
  product marker `MariaDB`, or does not begin with three decimal integers
  separated by dots, THEN the server SHALL NOT be taken to be MySQL, and
  `FR-MYSQL-008` SHALL apply. The condition is **necessary and not
  sufficient**: the system SHALL NOT claim to distinguish MySQL from a server
  that is not MySQL and reports a MySQL-compatible version string.

  *Observed, on MariaDB.* Every series of `FR-SRV-015` carries `-MariaDB`
  after the patch number in the probe's answer, per `FR-SRV-040`. The
  condition therefore refuses every supported MariaDB server that a `mysql`
  entry reaches with its read-only pair confirmed; `FR-MYSQL-012` states the
  case of a MariaDB server whose read-back fails first.

  *Pending observation.* That no supported MySQL series reports the marker,
  and that each begins with the three integers, rests on the manual's
  description quoted under `FR-MYSQL-006`. The reference inspector separates
  the two products on the same marker: its `Maria()` reports whether the
  version string contains `MariaDB`, in
  `sql/mysql/internal/mysqlversion/mysqlversion.go` at tag `v0.21.1`.

  *A stated limit, adopted.* **A server determined to pass as MySQL will
  pass.** It is the limit `FR-SRV-041` states for MariaDB, applied to MySQL in
  its terms and for its reason: every reading the condition can rest on is a
  response the server composes. A server that is not MySQL and reports a
  version string of MySQL's form without the marker is read as MySQL, under the
  standing its series earns, whoever built it. This requirement applies the
  first member of the family the [README](README.md#writing-conventions) names,
  and is not a fourth member of it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-008**: IF the server a `mysql` entry reaches is not MySQL under
  `FR-MYSQL-007`, THEN the system SHALL exit `78` (`EX_CONFIG`) and SHALL NOT
  read the catalogue. The `cause` SHALL state that the connection and the
  authentication succeeded, SHALL name the product the server reported —
  MariaDB, where the string carries the marker, and otherwise the string
  itself, escaped under `FR-ERR-024` — and SHALL state that a `mysql` entry
  reads MySQL only. The `hint` SHALL carry
  `tpl cfg database update <entry> --host <host>`, or
  `tpl cfg database update <entry> --dsn <url>` for an entry defined by
  `dsn`, per `FR-ERR-045`, and SHALL NOT carry a command that changes the
  provider, per `BR-ERR-005`.

  ```
  error: the server is not MySQL, for database entry 'shop'
  cause: connected and authenticated; the server reports 11.8.9-MariaDB-ubu2404, which is MariaDB, and a mysql entry reads MySQL only
  hint:  repoint the entry at a MySQL server: tpl cfg database update shop --host <host>
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code.

  *Rationale.* It is `FR-SRV-003` with the products exchanged, and `78` for
  the reason that requirement gives: the server is reachable and the
  credentials work, and the entry points at a server `tpl` does not serve
  through it.

  *Why the `hint` does not change the provider.* Changing it overwrites a
  value of `.tpl/.cfg` the invocation did not name, which `BR-ERR-005` bars a
  `hint` from carrying, and `FR-CONF-052` keeps the same bar for an entry's
  provider. The caller who meant a MariaDB server changes the provider by
  naming it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-009**: IF the server is MySQL, its series is not one of
  `FR-MYSQL-003`, and its series is older than the newest series of
  `FR-MYSQL-003`, THEN the system SHALL exit `78` and SHALL NOT read the
  catalogue. The message SHALL name the series found and the entry that
  reached it; its `cause` SHALL state that the connection and the
  authentication succeeded, why the series found is not supported, and the
  series that are; and its `hint` SHALL carry the command of `FR-MYSQL-008`.

  ```
  error: server series '8.0' is not supported, for database entry 'shop'
  cause: connected and authenticated; the server reports 8.0.46, and 8.0 is in Oracle Sustaining Support since 2026-04-21. A mysql entry supports 26.10, 9.7 and 8.4
  hint:  repoint the entry at a supported server: tpl cfg database update shop --host <host>
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code.

  *Rationale.* It is `FR-SRV-020` and `FR-SRV-030` for MySQL, on their ground:
  a series below the window is one the project will never observe, because
  observation follows support. Under the comparison of `FR-MYSQL-002`, one
  rule reaches a series that left support (`8.0`), an Innovation release that
  was superseded (`26.7`, `9.6`), and a series of a family that has left the
  set.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-010**: IF the server is MySQL and its series is newer than the
  newest series of `FR-MYSQL-003`, THEN the system SHALL read the catalogue,
  SHALL apply the treatment of the newest series of `FR-MYSQL-003` to every
  difference it knows, and SHALL set `standing` to `newer_than_supported`.
  Otherwise, WHEN the series is one of `FR-MYSQL-003`, `standing` SHALL be
  `supported`. `FR-SRV-032` and `FR-SRV-033`, and `BR-SRV-008` and
  `BR-SRV-009`, SHALL hold for a `mysql` read as for a `mariadb` read, with
  the set of `FR-MYSQL-003` in place of the set of `FR-SRV-015`.

  *Rationale.* It is `FR-SRV-031`, on its ground: a read-only reader of an
  enumerated field set does not break on a release cadence it does not
  control, and the field says the read is unverified.

  *Consequence, stated plainly.* Oracle published Innovation releases on
  2026-07-28 and 2026-10-20, per the release notes of `FR-MYSQL-003`, so a
  binary of `tpl` can meet a series newer than its table within months of its
  release, and marks the read. A server whose series the binary's table admits
  is read as `supported` by that binary after Oracle has superseded the series;
  a later binary, built on a later table, refuses it under `FR-MYSQL-009`. The
  table a binary carries is the one it was released with, per `FR-MYSQL-005`.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-011**: The product check of `FR-MYSQL-007` and `FR-MYSQL-008`,
  the version check of `FR-MYSQL-009` and the marked read of `FR-MYSQL-010`
  SHALL apply to every command that opens a connection for a `mysql` entry,
  without exception: the commands `FR-SRV-034` names, on the ground it gives.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-012**: An entry whose provider does not name the product its
  server runs SHALL be refused, and the requirements that refuse it are these:

  | Entry | Server | What refuses it |
  |---|---|---|
  | `mariadb` | MySQL | [server-contract.md](server-contract.md), unchanged: the read-back of `FR-SRV-009` where it does not confirm, under `FR-SRV-010`, and otherwise the product check of `FR-SRV-003` |
  | `mysql` | MariaDB `10.11` | The read-back of `FR-MYSQL-018`, under `FR-MYSQL-019`: that series has no `transaction_read_only` |
  | `mysql` | MariaDB `11.4`, `11.8` or `12.3` | The product check of `FR-MYSQL-007`, under `FR-MYSQL-008` |

  Every row ends with `78` and reads no catalogue. **Nothing in this file
  changes what a `mariadb` entry does on a MySQL server**: the first row is a
  reading of [server-contract.md](server-contract.md) as it stands.

  *Observed, on MariaDB.* `10.11` answers a read of
  `@@session.transaction_read_only` with `ERROR 1193 (HY000)` and the other
  three series answer it, per difference 12 of `FR-SRV-038`.

  *Pending observation.* That a supported MySQL series answers a read of
  `@@session.tx_read_only` with an error, so that a `mariadb` entry is refused
  at its read-back before its product check, rests on the manual: "The
  `tx_isolation` and `tx_read_only` system variables have been removed. Use
  `transaction_isolation` and `transaction_read_only` instead"
  (`refman/8.0/en/mysql-nutshell.html`); the 8.4 list of system variables
  carries `transaction_read_only` and no `tx_read_only`. Where the read-back
  refuses first, the `cause` names the read-only session and not the
  product, which is the cost `FR-SRV-042` states and accepts.

  *Added in the seventy-third edition,* for rmp `#324`.

## Differences between the supported series

- **FR-MYSQL-013**: Where the series of `FR-MYSQL-003` report the catalogue
  differently, the difference SHALL be supported as
  [server-contract.md](server-contract.md) requires it to be between the
  series of `FR-SRV-015`. The following requirements SHALL hold for a `mysql`
  read, each with the set of `FR-MYSQL-003` in place of the set of
  `FR-SRV-015` and the series resolved from the probe of `FR-MYSQL-006`:
  `FR-SRV-022`, `FR-SRV-023`, `FR-SRV-037`, `FR-SRV-024`, `FR-SRV-004`,
  `FR-SRV-025`, `FR-SRV-039`, `FR-SRV-005` and `FR-SRV-026`, with
  `BR-SRV-006`. A field excluded under `FR-SRV-025` for a `mysql` read SHALL
  enter the list of `FR-CAT-029` with the series that disagree named as MySQL
  series.

  *Rationale.* The four treatments of `BR-SRV-006` — normalised, marked,
  excluded, passed through — are the principles the user asked a MySQL read to
  share, and nothing in them is particular to MariaDB but the set they range
  over.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-014**: Every difference observed between the series of
  `FR-MYSQL-003` SHALL be recorded in this section, as `FR-SRV-038` requires
  of the series of `FR-SRV-015`, and every difference the model accommodates
  SHALL be registered as `FR-SRV-027` and `FR-SRV-036` require, in a table of
  this section.

  **The record holds no observed difference**, because no MySQL server has
  been observed. The register holds no row. The documentation names three
  candidates, recorded so that the observation that settles them knows where
  to look:

  | Candidate | `8.4` | `9.7` | `26.10` | What it would oblige |
  |---|---|---|---|---|
  | The column-attribute field lists `MASKING POLICY` | not in the manual's list | in the list | in the list | Nothing for the model, which has no property for it; `FR-MYSQL-030` reads past it |
  | The `VECTOR` type | not checked | documented, `refman/9.7/en/vector.html` | not checked | What the shared DDL of `FR-MYSQL-044` may declare, as difference 14 of `FR-SRV-038` did for MariaDB |
  | JSON duality views and their four `INFORMATION_SCHEMA` tables | not checked | not checked | documented in the 26.10 manual's index of `INFORMATION_SCHEMA` tables | Whether a duality view is a row of the table catalogue, and so a covered view, per `FR-MYSQL-023` |

  *Pending observation.* Every cell of the table is the documentation, cited
  in each row or in `FR-MYSQL-030`; "not checked" means that no page was read
  for that series. rmp `#326` compares the catalogue of the fixture of
  `FR-MYSQL-044` across the three series, as the passes recorded under
  `FR-SRV-038` did across MariaDB's, and moves every difference it finds into
  this record and, where it reaches the model, into the register.

  *Added in the seventy-third edition,* for rmp `#324`.

## The closed statement list

- **FR-MYSQL-015**: For a `mysql` entry, the system SHALL issue only
  statements drawn from the following closed list:

  | Statement | Purpose | When |
  |---|---|---|
  | `SELECT` against the twelve `INFORMATION_SCHEMA` tables named below, and against no other | Reading the catalogue, and the privilege probe of `FR-CFG-044` | As the command requires |
  | The server version probe | `FR-MYSQL-006` | Once, at connection start |
  | The read-only session statement | `FR-MYSQL-018` | Once, at connection start |
  | One read of the session variable `@@session.transaction_read_only`, reading nothing else | The read-back of `FR-MYSQL-018` | Once, at connection start, in the position `FR-MYSQL-017` fixes |

  The twelve tables, and the requirement that maps every field of each to the
  model:

  | Table | Read for | Field mapping |
  |---|---|---|
  | `SCHEMATA` | The database object | `FR-MYSQL-025` |
  | `TABLES` | Tables, views' rows, and the table type | `FR-MYSQL-026` |
  | `COLUMNS` | Columns | `FR-MYSQL-027` |
  | `STATISTICS` | Indexes and the primary key | `FR-MYSQL-032` |
  | `KEY_COLUMN_USAGE` | Foreign-key columns | `FR-MYSQL-033` |
  | `REFERENTIAL_CONSTRAINTS` | Foreign-key rules | `FR-MYSQL-033` |
  | `TABLE_CONSTRAINTS` | The table of a `CHECK` constraint | `FR-MYSQL-034` |
  | `CHECK_CONSTRAINTS` | `CHECK` constraints | `FR-MYSQL-034` |
  | `VIEWS` | Views | `FR-MYSQL-035` |
  | `ROUTINES` | Stored procedures and functions | `FR-MYSQL-036` |
  | `PARAMETERS` | Routine parameters | `FR-MYSQL-037` |
  | `TRIGGERS` | Triggers | `FR-MYSQL-038` |

  The first table fixes **membership**, as `FR-SRV-006` does, and not order;
  the order is `FR-MYSQL-017`. The read-back reads no schema, so it is not a
  catalogue query and does not count toward `NFR-PERF-001` and
  `NFR-PERF-002`.

  *Why the variable is named.* MySQL has one spelling of the session
  read-only state, and it is not the one `FR-SRV-006` names for MariaDB:
  `tx_read_only` was removed in MySQL 8.0, per the passage quoted under
  `FR-MYSQL-012`. A closed list whose entry admitted either spelling would
  admit one that no supported MySQL series can run.

  *Why the tables are closed.* `FR-SRV-006` closes the statements of a
  `mariadb` entry at "`SELECT` against `INFORMATION_SCHEMA.*`". This file
  closes the tables as well, because the read draws on twelve, and a reader
  that reached a thirteenth — the tables of InnoDB's internals, the global
  variables, the process list — would be reading something this file does not
  map to the model. The list is a ceiling on what a read may touch; the
  statements a read issues against it are fixed in number by `FR-MYSQL-042`.

  *Pending observation.* That every table and every field named in this file
  exists, with the name given, on every series of `FR-MYSQL-003`, rests on the
  manual pages cited in each mapping requirement. `FR-SRV-037` governs a field
  that turns out to be present on some series and not on others.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-016**: The system SHALL NOT issue any other statement for a
  `mysql` entry. It SHALL issue no DDL, no DML, no `SHOW`, no `SET` other than
  the read-only session statement, no statement against `performance_schema`,
  `mysql`, `sys` or any schema other than `INFORMATION_SCHEMA`, no read of a
  system variable outside the version probe and the read-back, and SHALL NOT
  invoke an external process such as a dump utility to read structure. No flag,
  configuration key or environment condition SHALL widen the list.

  *Consequence, stated plainly.* No statement sets a session variable that
  governs what the catalogue returns, and none needs to for the fields the
  model carries.

  *Pending observation.* The manual states that the table catalogue's
  statistics "hold cached values" under the `information_schema_stats_expiry`
  variable (`refman/8.4/en/information-schema-tables-table.html`); every
  statistics field the cache governs is excluded as volatile by
  `FR-CAT-024`, so none reaches the model. rmp `#326` confirms that no field
  the model carries moves with the variable.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-017**: WHEN the system opens a connection for a `mysql` entry, it
  SHALL issue the three connection-start statements of `FR-MYSQL-015` in the
  order `FR-SRV-042` fixes for their MariaDB counterparts — the read-only
  session statement, then the read-back, then the version probe — and every
  `SELECT` against `INFORMATION_SCHEMA` SHALL follow all three.

  *Rationale.* The order is forced by the condition order of `FR-ERR-006`, as
  `FR-SRV-042` derives it, and the strongest guarantee is confirmed before the
  server is characterised. `FR-SRV-042` states the cost: a server that is not
  the product the entry names receives the read-only pair first, and
  `FR-MYSQL-012` records the case in which that pair refuses it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **BR-MYSQL-002**: For a `mysql` entry the read-only promise of `BR-SEC-002`
  has the two parts it has for a `mariadb` entry. The closed lists of
  `FR-MYSQL-015` and `FR-MYSQL-016` are what `tpl` is built to send, hold
  whatever the server permits, and are the only part that prevents. The
  read-only session of `FR-MYSQL-018` makes a write fail and is defence in
  depth, in the sense `BR-SRV-002` states.

  *Added in the seventy-third edition,* for rmp `#324`.

## The read-only session

- **FR-MYSQL-018**: The system SHALL set the session read-only on every
  connection it opens for a `mysql` entry, SHALL read
  `@@session.transaction_read_only` back, and SHALL confirm that the setting
  took effect. It SHALL read that variable and no other spelling of it.

  *Pending observation.* The manual states that the session access mode is
  set by `SET SESSION transaction_read_only = value` or by
  `SET SESSION TRANSACTION READ ONLY`, which correspond to each other, and
  that the variable's value is `OFF` for read/write and `ON` for read only
  (`refman/8.4/en/server-system-variables.html`). rmp `#326` records, on every
  series of `FR-MYSQL-003`, the value the read-back returns before and after
  the setting, as `FR-SRV-006` records it for MariaDB.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-019**: IF the setting of `FR-MYSQL-018` cannot be applied, or the
  read-back does not confirm it, THEN the system SHALL exit `78` and SHALL NOT
  read the catalogue. There SHALL be no flag, configuration key or environment
  condition that disables `FR-MYSQL-015` through `FR-MYSQL-019`.

  *Rationale.* It is `FR-SRV-010` and `FR-SRV-011` for MySQL, on their
  ground: a support commitment or a safety guarantee that a flag can turn off
  is advisory.

  *Added in the seventy-third edition,* for rmp `#324`.

## Connection, authentication and TLS

- **FR-MYSQL-020**: A `mysql` entry SHALL be described, resolved and
  connected exactly as a `mariadb` entry is: by `dsn` or the discrete
  connection fields, per `FR-CONF-006`, `FR-CONF-007` and `FR-CONF-009`
  through `FR-CONF-012`, with both schemes of `FR-CONF-010` accepted and
  neither selecting the provider; with the host and the database it must
  name, per `FR-CONF-040`, `FR-CONF-041` and `FR-CONF-050`; with the default
  port of `FR-CONF-002`; with the password from `password` or
  `password_command`, per `FR-CONF-023` through `FR-CONF-033`; with the
  deadlines of `FR-CONF-005`; and with the TLS mode, the trust material and
  their rules, `FR-CONF-013`, `FR-CONF-014`, `FR-CONF-036`, `FR-CONF-037`,
  `FR-CONF-039`, `FR-CONF-044` and `BR-CONF-001`.

  *Rationale.* Every one of those keys and rules describes a connection to a
  server and what the caller asks of it. The user asked that the principles
  be the same, and the configuration is where a caller meets them first. That
  a MySQL server honours each of them as a MariaDB server does is what the
  cells of `FR-MYSQL-021` and the logins of `FR-MYSQL-022` hold it to, and is
  pending observation there.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-021**: The five modes of `FR-CONF-013` SHALL be distinguishable
  by observed behaviour against a MySQL server, and their behaviour SHALL be
  the behaviour `FR-CONF-038` fixes, against a MySQL server that offers TLS
  and one that offers none:

  | Mode | Server offering TLS | Server offering no TLS |
  |---|---|---|
  | `disabled` | plaintext | plaintext |
  | `preferred` | encrypted | plaintext |
  | `required` | encrypted, chain not validated | `69` (`EX_UNAVAILABLE`) |
  | `verify-ca` | encrypted, chain validated against the trust material of `FR-CONF-014` | `69` |
  | `verify-identity` | encrypted, chain and hostname validated | `69` |

  The authentication that precedes a plaintext session is governed by
  `FR-MYSQL-022`, and its one unsettled case is
  [OQ-077](open-questions.md#index).

  *Where MySQL differs.* Two facts of the server side differ from what
  `FR-CONF-038` records of MariaDB, and both reach a test rather than a cell:

  1. **A server with no TLS material of its own.** A MySQL server built with
     OpenSSL "can generate missing certificate and key files automatically at
     startup", and enables encrypted connections where it finds valid files in
     its data directory; where it does not, "it continues executing but
     without support for encrypted connections"
     (`refman/8.4/en/using-encrypted-connections.html`). The server writes a
     warning that such a certificate is self-signed, and nothing on that page
     says it names a host, so `verify-identity` reaches its success cell only
     against a certificate the fixture configures to name the host, as
     `FR-CONF-038` obliges of `scripts/mariadb/`.
  2. **A server that offers no TLS.** The manual names `tls_version=''` as
     the way to start the server "with encrypted connections disabled on the
     main interface", and names no `have_ssl` variable on that page; the
     right-hand column is therefore staged by configuration, where MariaDB
     `10.11` offers it unconfigured, per difference 3 of `FR-SRV-038`.

  *Pending observation.* Every cell of the table, against servers of every
  series of `FR-MYSQL-003`, with encryption read from the live session; the
  three controls `FR-CONF-038` records — `verify-ca` without trust material
  fails, `preferred` without trust material connects encrypted, `required`
  ignores supplied trust material; whether a certificate the server generates
  carries a `subjectAltName`; and whether trust material stays additional to
  the public roots, as `FR-CONF-039` records of the driver, against a MySQL
  server.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-022**: The system SHALL authenticate a `mysql` entry through the
  default authentication method of every series of `FR-MYSQL-003`, which is
  `caching_sha2_password`, under every TLS mode of `FR-CONF-013` that yields
  an encrypted session. A database driver that cannot SHALL be disqualified
  from the choice, as `FR-CONF-036` disqualifies one that cannot express the
  five modes. WHERE the session is not encrypted — `disabled`, and
  `preferred` against a server offering no TLS — the authentication of an
  account the server has not cached is [OQ-077](open-questions.md#index), and
  no requirement of this file settles it.

  *Source.* "The default plugin is determined by the value of the
  `authentication_policy` system variable; the default is to use
  `caching_sha2_password`"; on a secure connection "an RSA key pair is
  unnecessary and is not used"; on a connection that is not secure "an RSA key
  pair is used", and without the server's public key the login fails with
  "Authentication requires secure connection"
  (`refman/9.7/en/caching-sha2-pluggable-authentication.html`).

  *Pending observation.* The default method on `8.4` and `26.10`, which the
  page above was read for `9.7` alone; and, on every series, a login of the
  fixture's reader under each TLS mode, before and after the server has
  cached the account.

  *Rationale.* `FR-CONF-036` makes the modes normative over the driver; a
  driver that connects in every mode and cannot authenticate the account a
  supported server creates by default connects nowhere that matters.

  *Added in the seventy-third edition,* for rmp `#324`.

## The part of the database a read covers

- **FR-MYSQL-023**: A `mysql` read SHALL cover the database the entry names,
  per `FR-CONF-041`, and no other. The model SHALL cover, from the table
  catalogue, the rows whose table type is `BASE TABLE`, as tables carrying
  `table_type` `BASE TABLE`, per `FR-CAT-001` and `FR-CAT-002`, and the rows
  whose table type is `VIEW`, as views, per `FR-CAT-007`; and SHALL cover
  every routine of the database, per `FR-CAT-008`. A row whose table type is
  any other value SHALL NOT be covered. The coverage SHALL be a filter on the
  table type, per `FR-CAT-032`, and SHALL be applied to the column read as
  well, per `FR-CAT-052`.

  *Pending observation.* The manual gives the table type as "`BASE TABLE` for
  a table, `VIEW` for a view, or `SYSTEM VIEW` for an `INFORMATION_SCHEMA`
  table", and states that the table catalogue "does not list `TEMPORARY`
  tables" (`refman/8.4/en/information-schema-tables-table.html`, and the
  same on `refman/26.10/en/information-schema-tables-table.html`). MySQL has
  no system-versioned tables and no sequences, so `SYSTEM VERSIONED` and
  `SEQUENCE`, which `FR-CAT-031` records for MariaDB, are expected never to
  appear, and `table_type` to take `BASE TABLE` alone. rmp `#326` records the
  table types every series emits, after a `CREATE TEMPORARY TABLE` in the
  reading session as well, as `FR-CAT-031` and `FR-CAT-032` record them.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-024**: The enumerated exclusions of
  [catalogue-coverage.md](catalogue-coverage.md#enumerated-exclusions) —
  `FR-CAT-019` through `FR-CAT-023` and `FR-CAT-057` — SHALL hold for a
  `mysql` read. The model SHALL NOT cover, in addition, **an index with a
  [functional key part](glossary.md#functional-key-part)**: an index of which
  any row of the index catalogue carries SQL `NULL` as the column name and a
  value as the expression. The system SHALL NOT read a further object to cover
  one.

  *Why an index with a functional key part is excluded and not carried.* The
  model's index carries its members as columns of its table, and
  `FR-CAT-044` forbids a member naming a column the table lacks. A functional
  key part names none, and MariaDB has no such index, per `FR-CAT-042`, so the
  model has no shape for one. It is the ground `FR-LITE-021` gives for a
  SQLite expression index, and the same exclusion. A multi-valued index on a
  JSON array is excluded with it WHERE the catalogue reports it as a
  functional key part.

  *Accepted cost, stated because it is silent.* An excluded index is
  presented as absent, at exit `0`, and nothing in the document says so. A
  `UNIQUE` index with a functional key part enforces a uniqueness the model
  does not show.

  *Pending observation.* The manual states that "For a functional key part,
  `COLUMN_NAME` column is `NULL` and `EXPRESSION` indicates the expression for
  the key part" (`refman/8.4/en/information-schema-statistics-table.html`),
  that functional indexes "are implemented as hidden virtual generated
  columns", and that "primary keys cannot include functional key parts"
  (`refman/8.4/en/create-index.html`). rmp `#326` records the rows of every
  index of that kind in the fixture, a multi-valued index among them, and
  whether its hidden column appears in the column catalogue, which
  `FR-CAT-052` would then have to filter.

  *Added in the seventy-third edition,* for rmp `#324`.

## How the catalogue fills the model

- **BR-MYSQL-003**: For a `mysql` read, a property is filled from the field
  the requirements below name for it, and from no other. Where no field of the
  MySQL catalogue supplies a property the model defines, the property SHALL
  be `null`, on the ground `FR-SRV-004` gives for a fact a series does not
  provide, and the document SHALL keep its shape, per `FR-SRV-005`. Where the
  MySQL catalogue offers a field the model has no property for, the field
  SHALL NOT be carried: the model's properties are those
  [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) fix, and adding one adds a key to
  every MariaDB document, which `FR-CTX-043` makes the change of `provider`
  alone. Every character-set and collation value SHALL be passed through
  under `FR-SRV-039`.

  *Rationale.* The user asked that MySQL fill the model MariaDB already fills,
  and that MariaDB support not change. A property MySQL cannot supply is the
  case `FR-SRV-004` marks; a field only MySQL supplies is the case a model
  shared by three providers must decline, as `BR-LITE-002` declines
  `WITHOUT ROWID` for SQLite.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-025**: For a `mysql` read, the `database` object SHALL be filled
  from the row of the schema catalogue, `SCHEMATA`, for the database the entry
  names:

  | Field | The manual says | Model |
  |---|---|---|
  | `CATALOG_NAME` | always `def` | not carried — row identity, `BR-CAT-005` |
  | `SCHEMA_NAME` | the name of the schema | `name`, per `FR-CTX-036` |
  | `DEFAULT_CHARACTER_SET_NAME` | the schema default character set | `charset`, passed through |
  | `DEFAULT_COLLATION_NAME` | the schema default collation | `collation`, passed through |
  | `SQL_PATH` | always `NULL` | not carried — `FR-CTX-036` gives it no field |
  | `DEFAULT_ENCRYPTION` | the schema default encryption | not carried — `FR-CTX-036` fixes four metadata fields |

  `provider` SHALL be `mysql`, per `FR-CTX-043`, and the `server` object that
  of `FR-MYSQL-006`.

  *Pending observation.* Every row, from
  `refman/8.4/en/information-schema-schemata-table.html`. MariaDB's schema
  catalogue also has six fields, the sixth a schema comment, per
  `FR-CTX-036`; MySQL's sixth is the encryption default. Neither reaches the
  model.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-026**: For a `mysql` read, a table SHALL carry the properties of
  `FR-CAT-053` from the following sources, and the fields of the table
  catalogue, `TABLES`, SHALL be mapped as follows:

  | Field | Model |
  |---|---|
  | `TABLE_CATALOG`, `TABLE_SCHEMA` | not carried — row identity |
  | `TABLE_NAME` | `name` |
  | `TABLE_TYPE` | the coverage predicate of `FR-MYSQL-023`; `table_type` |
  | `ENGINE` | `engine`, per `FR-CAT-054` |
  | `TABLE_COLLATION` | `collation`, per `FR-CAT-054` |
  | `TABLE_COMMENT` | the table's comment; on a view's row, not presented as the view's comment, per `FR-CAT-040` |
  | `VERSION`, `TABLE_ROWS`, `AVG_ROW_LENGTH`, `DATA_LENGTH`, `MAX_DATA_LENGTH`, `INDEX_LENGTH`, `DATA_FREE`, `AUTO_INCREMENT`, `CREATE_TIME`, `UPDATE_TIME`, `CHECK_TIME`, `CHECKSUM` | not carried — volatile, `FR-CAT-024` |
  | `ROW_FORMAT`, `CREATE_OPTIONS` | not carried — `FR-CAT-053` names no property for either |

  Its columns are `FR-MYSQL-027`; its primary key and indexes
  `FR-MYSQL-032`; `foreign_keys` and `referenced_by` `FR-MYSQL-033`; its
  triggers `FR-MYSQL-038`; and its `CHECK` constraints `FR-MYSQL-034`.

  *Pending observation.* The 21 fields and their descriptions, from
  `refman/8.4/en/information-schema-tables-table.html` and
  `refman/26.10/en/information-schema-tables-table.html`, which list the same
  21. "For views, most `TABLES` columns are 0 or `NULL` except that
  `TABLE_NAME` indicates the view name, `CREATE_TIME` indicates the creation
  time, and `TABLE_COMMENT` says `VIEW`", so `FR-CAT-040` reaches a MySQL view
  as it reaches a MariaDB one. What `TABLE_COMMENT` holds for a table with no
  comment — the empty string, as `FR-CAT-039` records for MariaDB, or
  another value — is not stated, and rmp `#326` records it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-027**: For a `mysql` read, a table's columns SHALL be the rows of
  the column catalogue, `COLUMNS`, for that table, in the order of
  `ORDINAL_POSITION`, which is the order of `FR-CAT-009` and `NFR-DET-002`,
  and the fields SHALL be mapped as follows:

  | Field | Model |
  |---|---|
  | `TABLE_CATALOG`, `TABLE_SCHEMA` | not carried — row identity |
  | `TABLE_NAME` | `table_name`, per `FR-CTX-019`, and the table the column is carried on |
  | `COLUMN_NAME` | `name` |
  | `ORDINAL_POSITION` | the order of the columns |
  | `COLUMN_DEFAULT` | the default, per `FR-MYSQL-029` |
  | `IS_NULLABLE` | the nullability — nullable WHERE `YES`, not nullable WHERE `NO` — and `FR-MYSQL-029` |
  | `DATA_TYPE` | `data_type`, per `FR-MYSQL-028` |
  | `CHARACTER_MAXIMUM_LENGTH` | `length` |
  | `CHARACTER_OCTET_LENGTH` | not carried — derivable, per `FR-CTX-040` |
  | `NUMERIC_PRECISION`, `DATETIME_PRECISION` | `precision`, per `FR-CTX-040` |
  | `NUMERIC_SCALE` | `scale` |
  | `CHARACTER_SET_NAME`, `COLLATION_NAME` | `charset`, `collation`, per `FR-CTX-041` |
  | `COLUMN_TYPE` | `column_type`, and `unsigned` and `values` derived from it, per `FR-MYSQL-028` |
  | `COLUMN_KEY` | not carried — restates the table's keys, `FR-CTX-021` |
  | `EXTRA` | the static attributes, per `FR-MYSQL-030` |
  | `PRIVILEGES` | not carried — it describes the reader's privileges and no property of a column carries it |
  | `COLUMN_COMMENT` | the column's comment |
  | `GENERATION_EXPRESSION` | the expression of a generated column, per `FR-MYSQL-031` |
  | `SRS_ID` | not carried — spatial reference identifiers are excluded, `FR-CAT-023` |

  *Pending observation.* The 22 fields, from
  `refman/8.4/en/information-schema-columns-table.html`, with the same 22 on
  the 9.7 and 26.10 pages. MariaDB's column catalogue carries an
  is-generated field MySQL's does not, per `FR-CAT-051`, and two
  system-versioning fields on three of its series, per difference 5 of
  `FR-SRV-038`; every field `FR-CTX-040` and `FR-CTX-041` read on MariaDB is
  in MySQL's list. The manual
  states that information about a generated invisible primary key "is
  visible in this table by default" and is hidden by setting
  `show_gipk_in_create_table_and_information_schema` off; a server so
  configured presents such a table with neither the column nor the key, and
  the fixture records the setting it reads under, per the fourth validation
  rule of the [README](README.md#maintenance-debt).

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-028**: For a `mysql` read, a column's type SHALL be decomposed
  as `FR-CTX-014` through `FR-CTX-018` and `FR-CTX-038` through `FR-CTX-041`
  decompose a MariaDB column's: `column_type` exactly as the catalogue returns
  it; each part from the field `FR-CTX-040` names; `unsigned` derived from
  `column_type`; `values` read from `column_type` under the rules of
  `FR-CTX-039`; and every part `null` for a type the system does not
  recognise, per `FR-CTX-018`. The grammar of `column_type` SHALL be taken to
  differ from MariaDB's as follows:

  | MariaDB, observed under `FR-CTX-038` | MySQL, as documented |
  |---|---|
  | Every integer type carries a display width, `int(10)` | An integer type carries none, `int`, save `tinyint(1)` and a type with `ZEROFILL` |
  | A column declared `JSON` reports `longtext` | A column declared `JSON` reports `json`, a native type |
  | `inet4`, `inet6` and `uuid` are types | MySQL has none of the three |
  | `VECTOR` on `11.8` and `12.3` | `VECTOR(N)` on `9.7`, per `FR-MYSQL-014` |

  A `data_type` value no MariaDB read produces — `json` and `vector` among
  them — SHALL take the family `FR-ENV-046` gives an unplaced value, which is
  none of the three, until an observation places it.

  *Pending observation.* The display width: "now statements that include data
  type definitions in their output no longer show the display width for
  integer types", save "`TINYINT(1)`" and a type that "includes the
  `ZEROFILL` attribute", a change that "affects the output from `SHOW CREATE`
  and `DESCRIBE` statements, and from `INFORMATION_SCHEMA` tables"
  (`relnotes/mysql/8.0/en/news-8-0-19.html`). JSON: "MySQL supports a native
  `JSON` (JavaScript Object Notation) data type" (`refman/8.4/en/json.html`).
  `VECTOR`: `refman/9.7/en/vector.html`. rmp `#326` records, on every series,
  every distinct `column_type` and `data_type` the fixture produces, the
  parts populated for each type family as `FR-CTX-040` records them, the
  escaping of an apostrophe and a comma in an `ENUM` member as `FR-CTX-039`
  records it, and whether a `JSON` column reports a character set.

  *Consequence, stated plainly.* A column declared `JSON` is `longtext` and
  **textual** on a MariaDB read, and `json` and in no family on a MySQL read,
  so the `textual` test of `FR-ENV-041` answers differently for one
  declaration on the two providers. A template that needs the declaration
  reads `data_type`, which says `json` on MySQL; on MariaDB the declaration
  survives only as the implicit constraint `FR-CAT-038` records.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-029**: For a `mysql` read, the system SHALL classify a column's
  default from `COLUMN_DEFAULT`, `EXTRA` and `IS_NULLABLE`, by the first row
  of the following table that matches, and SHALL derive the forms of
  `FR-CTX-012` from that classification alone:

  | Shape | Emitted |
  |---|---|
  | `COLUMN_DEFAULT` is SQL `NULL`, and `IS_NULLABLE` is `NO` | `null` |
  | `COLUMN_DEFAULT` is SQL `NULL`, and `IS_NULLABLE` is `YES` | `{"kind":"null"}` |
  | `EXTRA` carries the phrase `DEFAULT_GENERATED`, and the text begins with `(` and ends with `)` | `expression`, carrying the text between the outer parentheses |
  | `EXTRA` carries the phrase `DEFAULT_GENERATED` | `expression`, carrying the text unchanged |
  | Anything else | `literal`, carrying the text unchanged |

  The classification of `FR-CTX-037` SHALL NOT be applied to a `mysql` read.

  *Why a separate classification.* MariaDB tells a string literal from
  everything else by quoting it, and writes a default of `NULL` as the four
  characters `NULL`, per `FR-CTX-037`. MySQL quotes no literal and marks an
  expression in another field. Read by `FR-CTX-037`, a MySQL default `EUR`
  would be an expression and a MySQL default `'NULL'` would be a null default:
  the inversion `FR-SRV-003` names as the first reason a MariaDB reader is
  silently wrong on MySQL. The reference inspector reads MySQL the same way:
  a default is an expression where `EXTRA` marks it `DEFAULT_GENERATED`, and
  is otherwise quoted as a literal by the inspector itself, in
  `sql/mysql/inspect.go`, `myDefaultExpr`.

  *Why the nullability.* A nullable column with no `DEFAULT` clause and a
  column declaring `DEFAULT NULL` are one state in MySQL as in MariaDB, and
  the manual returns SQL `NULL` for both; a `NOT NULL` column with no default
  returns SQL `NULL` too. `IS_NULLABLE` separates the two forms of
  `FR-CTX-012` that MariaDB's field separates in one value, which is the
  reading `FR-LITE-027` makes for SQLite and not the reconstruction
  `FR-CTX-012` rejects.

  *Pending observation.* `COLUMN_DEFAULT` "is `NULL` if the column has an
  explicit default of `NULL`, or if the column definition includes no `DEFAULT`
  clause", and `EXTRA` holds "`DEFAULT_GENERATED` for columns that have an
  expression default value"
  (`refman/8.4/en/information-schema-columns-table.html`); an expression
  default is written in parentheses, save `CURRENT_TIMESTAMP` on a `TIMESTAMP`
  or `DATETIME` column, and `BLOB`, `TEXT`, `GEOMETRY` and `JSON` columns take
  only expression defaults (`refman/8.4/en/data-type-defaults.html`). The
  manual does not state the text a default is reported as. rmp `#326` records,
  on every series, for each case the fixture declares: whether
  `CURRENT_TIMESTAMP` is marked `DEFAULT_GENERATED`, as the reference
  inspector's handling of it implies; whether a parenthesised expression is
  reported with its parentheses; whether a binary literal is reported in
  hexadecimal, as the inspector's note "MySQL v8 uses Hexadecimal
  representation" states; how a bit literal is reported; whether an empty
  string default is reported as the empty text; and whether a quote inside an
  expression is reported escaped with a backslash, which the inspector undoes.
  WHERE an expression is reported escaped, carrying the text unchanged carries
  the escapes, and the amendment that follows the observation fixes the rule
  that removes them.

  *Consequence, stated plainly.* A string default the DDL wrote `'EUR'`
  yields `{"kind":"literal","value":"EUR"}` from either provider. An
  expression's text is the server's own and differs between them —
  `current_timestamp()` from MariaDB, per `FR-CTX-037`, where the manual
  writes `CURRENT_TIMESTAMP` for MySQL — and is passed through as each server
  writes it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-030**: For a `mysql` read, the system SHALL read `EXTRA` as a
  sequence of the following phrases, separated by single spaces and in any
  order, each recognised ignoring ASCII letter case, and SHALL take from it the
  facts the table names:

  | Phrase | Fact | Carried under |
  |---|---|---|
  | `auto_increment` | the column is auto-incremental | `FR-CAT-027`, `FR-CTX-020` |
  | `INVISIBLE` | the column is invisible | `FR-CAT-035`, `FR-CTX-020` |
  | `VIRTUAL GENERATED` | the column is generated and not stored | `FR-MYSQL-031` |
  | `STORED GENERATED` | the column is generated and stored | `FR-MYSQL-031` |
  | `on update` followed by one word | the column carries an `ON UPDATE` default, which is that word | `FR-CTX-020` |
  | `DEFAULT_GENERATED` | the column's default is an expression | `FR-MYSQL-029` |
  | `MASKING POLICY` | the column carries a masking policy | not carried — the model has no property for it |

  IF `EXTRA` holds text that no phrase matches, THEN the system SHALL take
  the facts of the phrases it recognises, SHALL NOT fail the read, and SHALL
  NOT change the exit code on account of it, on the ground `FR-CAT-055` gives
  for a value outside a recorded set.

  *Pending observation.* The manual lists `auto_increment`,
  "`on update CURRENT_TIMESTAMP` for `TIMESTAMP` or `DATETIME` columns that
  have the `ON UPDATE CURRENT_TIMESTAMP` attribute", "`STORED GENERATED` or
  `VIRTUAL GENERATED` for generated columns", "`DEFAULT_GENERATED` for columns
  that have an expression default value", and, on 9.7 and 26.10 only,
  "`MASKING POLICY` for columns with a masking policy"
  (`refman/8.4/en/information-schema-columns-table.html`, and the 9.7 and
  26.10 pages); "For invisible columns, `EXTRA` displays `INVISIBLE`", and
  "Generated columns can be invisible" (`refman/8.4/en/invisible-columns.html`).
  The manual shows no column carrying two phrases at once, which is the case
  `FR-CAT-041` bounds for MariaDB. rmp `#326` records, on every series, the
  field for every combination the fixture declares — an auto-incremental and
  an expression default each with `INVISIBLE`, a generated invisible column,
  `DEFAULT_GENERATED` with `on update`, and a fractional-second `ON UPDATE` —
  and the amendment that follows fixes the order and the separator observed.

  *Consequence, stated plainly.* MariaDB writes the `ON UPDATE` attribute
  `on update current_timestamp()`, per `FR-CAT-041`, and the manual writes it
  `on update CURRENT_TIMESTAMP` for MySQL; the expression carried is each
  server's own spelling.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-031**: For a `mysql` read, a generated column SHALL carry its
  storage kind from the phrase of `FR-MYSQL-030`, `VIRTUAL GENERATED` or
  `STORED GENERATED`, and its expression from `GENERATION_EXPRESSION`, as
  `FR-CAT-051` requires. A column whose `EXTRA` carries neither phrase SHALL
  NOT be generated, whatever `GENERATION_EXPRESSION` holds.

  *Why `EXTRA` and not another field.* MySQL's column catalogue has no
  is-generated field, which `FR-CAT-051` reads beside the attribute field on
  MariaDB and which `FR-SRV-003` names as the third reason a MariaDB reader is
  silently wrong on MySQL. `FR-CAT-051` already takes the storage kind from
  the attribute field, so the fact is read from the field both servers carry.

  *Pending observation.* `GENERATION_EXPRESSION` "For generated columns,
  displays the expression used to compute column values. Empty for
  nongenerated columns" (`refman/8.4/en/information-schema-columns-table.html`).
  The text's form is not stated. The reference inspector passes a MySQL
  generation expression through its `unescape` before use, and a MariaDB one
  not, in `sql/mysql/inspect.go`. rmp `#326` records the expression of every
  generated column of the fixture on every series, one of them holding a
  string literal, and the amendment that follows fixes a rule for removing
  any escaping observed, as `FR-MYSQL-029` states for a default.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-032**: For a `mysql` read, a table's indexes SHALL be folded from
  the rows of the index catalogue, `STATISTICS`, for that table, per
  `FR-CAT-010`, less those `FR-MYSQL-024` excludes; its primary key SHALL be
  the index the catalogue reports under the name `PRIMARY`, read from that
  catalogue and from no other, per `FR-CAT-043`; and the fields SHALL be
  mapped as follows:

  | Field | Model |
  |---|---|
  | `TABLE_CATALOG`, `TABLE_SCHEMA`, `INDEX_SCHEMA` | not carried — row identity |
  | `TABLE_NAME` | the table the index is carried on |
  | `INDEX_NAME` | `name`; `PRIMARY` is the primary key |
  | `NON_UNIQUE` | `unique`, true WHEN the field is `0` |
  | `SEQ_IN_INDEX` | the order of the members, which `FR-CAT-010` folds on |
  | `COLUMN_NAME` | the member's column; SQL `NULL` marks a functional key part, per `FR-MYSQL-024` |
  | `COLLATION` | the member's sort direction: `A`, `D`, or SQL `NULL` |
  | `CARDINALITY` | not carried — volatile, `FR-CAT-024` |
  | `SUB_PART` | the member's prefix length |
  | `PACKED` | not carried — no property of the index object carries it |
  | `NULLABLE` | not carried — restates the column's nullability, `FR-CTX-021` |
  | `INDEX_TYPE` | `type`, carried unchanged |
  | `COMMENT` | not carried — `FR-CAT-042` gives this field no property |
  | `INDEX_COMMENT` | `comment` |
  | `IS_VISIBLE` | `ignored`, true WHEN the field is `NO` |
  | `EXPRESSION` | not carried; a value marks a functional key part, per `FR-MYSQL-024` |

  *Why `IS_VISIBLE` fills `ignored`.* The fact is the one MariaDB reports in
  its ignored field, per `FR-CAT-042`: the server keeps the index and the
  optimizer does not use it. MySQL names the same fact visibility, "Whether
  the index is visible to the optimizer", and inverts its polarity, so the
  model carries one representation of one fact, under `FR-SRV-024`'s ground,
  across providers.

  *Pending observation.* The 18 fields and their descriptions, from
  `refman/8.4/en/information-schema-statistics-table.html` and
  `refman/26.10/en/information-schema-statistics-table.html`, which list the
  same 18. `INDEX_TYPE` is documented as "(`BTREE`, `FULLTEXT`, `HASH`,
  `RTREE`)", where MariaDB was observed to report `SPATIAL` for a spatial
  index, per `FR-CAT-042`; the value is carried as the server returns it, and
  rmp `#326` records what a spatial index reports and what its `SUB_PART`
  reads. `COMMENT` holds "information about the index not described in its
  own column, such as `disabled`". A table with no primary key and a
  `NOT NULL` unique index is reported under that index's own name, and the
  model then carries no primary key; rmp `#326` records it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-033**: For a `mysql` read, a table's outgoing foreign keys SHALL
  be read from the referential-constraint table, `REFERENTIAL_CONSTRAINTS`,
  joined to the key-column table, `KEY_COLUMN_USAGE`, as `FR-CAT-045` reads
  them, and SHALL carry the same properties from the fields of the same
  names. The incoming keys under `referenced_by` SHALL be the same rows read
  from the other end, per `FR-CAT-013`. `FR-CAT-056` SHALL hold for the
  nullable fields it names, and `FR-CAT-057` for a key that crosses a schema.

  | Table | Field | Model |
  |---|---|---|
  | `REFERENTIAL_CONSTRAINTS` | `CONSTRAINT_CATALOG`, `CONSTRAINT_SCHEMA`, `UNIQUE_CONSTRAINT_CATALOG`, `UNIQUE_CONSTRAINT_SCHEMA` | not carried — row identity |
  | `REFERENTIAL_CONSTRAINTS` | `CONSTRAINT_NAME` | `name` |
  | `REFERENTIAL_CONSTRAINTS` | `UNIQUE_CONSTRAINT_NAME` | `referenced_key` |
  | `REFERENTIAL_CONSTRAINTS` | `MATCH_OPTION` | `match_option` |
  | `REFERENTIAL_CONSTRAINTS` | `UPDATE_RULE`, `DELETE_RULE` | `on_update`, `on_delete`, carried unchanged |
  | `REFERENTIAL_CONSTRAINTS` | `TABLE_NAME` | the table the key is carried on |
  | `REFERENTIAL_CONSTRAINTS` | `REFERENCED_TABLE_NAME` | `referenced_table` |
  | `KEY_COLUMN_USAGE` | `CONSTRAINT_CATALOG`, `CONSTRAINT_SCHEMA`, `TABLE_CATALOG`, `TABLE_SCHEMA` | not carried — row identity |
  | `KEY_COLUMN_USAGE` | `CONSTRAINT_NAME` | not carried — joins the row to its rule |
  | `KEY_COLUMN_USAGE` | `TABLE_NAME`, `COLUMN_NAME` | the referencing columns |
  | `KEY_COLUMN_USAGE` | `ORDINAL_POSITION` | their order |
  | `KEY_COLUMN_USAGE` | `POSITION_IN_UNIQUE_CONSTRAINT` | the pairing of referencing to referenced column |
  | `KEY_COLUMN_USAGE` | `REFERENCED_TABLE_SCHEMA`, `REFERENCED_TABLE_NAME`, `REFERENCED_COLUMN_NAME` | `referenced_columns`, and the schema test of `FR-CAT-057` |

  A referenced table and a trigger's table SHALL be matched to a covered
  table by their names compared byte for byte.

  *Pending observation.* The fields, from
  `refman/8.4/en/information-schema-referential-constraints-table.html` and
  `refman/8.4/en/information-schema-key-column-usage-table.html`. The rules
  are documented as `CASCADE`, `SET NULL`, `SET DEFAULT`, `RESTRICT` and
  `NO ACTION`, and `MATCH_OPTION` as "The only valid value at this time is
  `NONE`". `SET DEFAULT` "is recognized by the MySQL parser, but both InnoDB
  and NDB reject table definitions containing" it, so `FR-CAT-033` holds for
  a MySQL read by a refusal at DDL time, where MariaDB accepts the clause and
  reports `RESTRICT`; and "For an `ON DELETE` or `ON UPDATE` that is not
  specified, the default action is always `NO ACTION`"
  (`refman/8.4/en/create-table-foreign-keys.html`). InnoDB admits a key
  referencing a non-unique index under `restrict_fk_on_non_standard_key`,
  whose `UNIQUE_CONSTRAINT_NAME` the manual does not describe. rmp `#326`
  records the rules a key declared without the clauses reports, the
  referenced-key name of every key, and, under the fixture's setting of
  `lower_case_table_names`, whether a referenced table's name is reported as
  the table declares it.

  *Why byte for byte, and what bounds it.* No statement of `FR-MYSQL-015` reads
  `lower_case_table_names`, which decides how the server compares table names;
  the reference inspector reads it at connection time, in
  `sql/mysql/driver.go`. On a server that compares names as written, a key's
  referenced name is expected to be the table's own, which the pending
  observation above confirms or refutes. On a server that stores or compares
  them folded, a key whose referenced name differed in case from the table's
  own would be excluded, at exit `0`, under `FR-CTX-042`'s ground. The
  fixture's servers run with the setting they are started with, which the
  fixture records, and the observation is bounded to it.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-034**: For a `mysql` read, a table's `CHECK` constraints SHALL be
  the rows of the table-constraint table, `TABLE_CONSTRAINTS`, whose
  `CONSTRAINT_TYPE` is `CHECK`, for that table, each joined to the row of the
  check-constraint table, `CHECK_CONSTRAINTS`, of the same schema and the same
  constraint name. Each SHALL carry:

  | Table | Field | Model |
  |---|---|---|
  | `TABLE_CONSTRAINTS` | `CONSTRAINT_CATALOG`, `CONSTRAINT_SCHEMA` | not carried — row identity |
  | `TABLE_CONSTRAINTS` | `CONSTRAINT_NAME` | `name`, and the join |
  | `TABLE_CONSTRAINTS` | `TABLE_SCHEMA`, `TABLE_NAME` | the table the constraint is carried on |
  | `TABLE_CONSTRAINTS` | `CONSTRAINT_TYPE` | selects the `CHECK` rows; no other row is read from this table |
  | `TABLE_CONSTRAINTS` | `ENFORCED` | not carried — the model has no property for it |
  | `CHECK_CONSTRAINTS` | `CONSTRAINT_CATALOG`, `CONSTRAINT_SCHEMA` | not carried — row identity, and the join |
  | `CHECK_CONSTRAINTS` | `CONSTRAINT_NAME` | the join |
  | `CHECK_CONSTRAINTS` | `CHECK_CLAUSE` | `clause` |

  `level` SHALL be `null`: MySQL records no level for a constraint.

  *Why the join.* MySQL's check-constraint table carries no table name, which
  is the second reason `FR-SRV-003` gives for refusing MySQL through a
  `mariadb` entry; the table-constraint table carries it. MySQL makes a
  `CHECK` constraint's name unique in its schema, so the join attributes each
  constraint to one table. The reference inspector reads MySQL's constraints
  by the same join, in `sql/mysql/inspect.go`, `myChecksQuery`, and reads
  MariaDB's from the check-constraint table alone.

  *Why `level` is `null`.* A column-level and a table-level constraint are
  recorded alike: MySQL gives an unnamed one the name of its table, `_chk_`
  and an ordinal, whichever level it was written at, and reports no level.
  `null` is the fact the server does not provide, per `BR-MYSQL-003`.
  *Rejected: `Table` for every constraint.* It states a fact the server does
  not hold, and is false of every constraint written on a column.
  *Rejected: deriving the level from the clause.* A clause naming one column
  is as often a table constraint.

  *A constraint not enforced is carried.* `FR-CAT-015` requires a table to
  carry its `CHECK` constraints, and a constraint declared `NOT ENFORCED` is
  one. *Accepted cost, stated because it is silent.* It is presented as every
  other constraint is, at exit `0`, and a template that emits DDL from it
  emits a constraint that the database it came from does not enforce.
  *What would change this.* A property stating enforcement on the constraint
  object, which adds a key to every constraint of every MariaDB document and
  is a change to [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) the user has not asked for.

  *Pending observation.* `CHECK_CONSTRAINTS` has four fields, and no field
  names the owning table
  (`refman/8.4/en/information-schema-check-constraints-table.html`);
  `TABLE_CONSTRAINTS` has seven, `CONSTRAINT_TYPE` "can be `UNIQUE`,
  `PRIMARY KEY`, `FOREIGN KEY` or `CHECK`", and `ENFORCED` is "`YES` or `NO`"
  for a `CHECK` constraint
  (`refman/8.4/en/information-schema-table-constraints-table.html`); an
  unnamed constraint is named "from the table name, a literal `_chk_`, and an
  ordinal number", and "`CHECK` constraint names must be unique per schema"
  (`refman/8.4/en/create-table-check-constraints.html`). A `JSON` column
  carries no implicit constraint, where MariaDB attaches one, per
  `FR-CAT-038`. rmp `#326` records, on every series, the clause of every
  constraint of the fixture, the name of every unnamed one, the field
  `ENFORCED`, and that no constraint appears on a `JSON` column the fixture
  did not write; and, as the reduced-grant reader, whether the
  check-constraint table returns a row the table-constraint table does not.
  The join would drop such a row at exit `0`, and observing one would call
  for a cross-check of the kind `FR-PRIV-019` makes for foreign keys.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-035**: For a `mysql` read, a view SHALL carry exactly the nine
  properties `FR-CAT-047` gives a view, from the view catalogue, `VIEWS`:

  | Field | Model |
  |---|---|
  | `TABLE_CATALOG`, `TABLE_SCHEMA` | not carried — row identity |
  | `TABLE_NAME` | `name` |
  | `VIEW_DEFINITION` | `definition` |
  | `CHECK_OPTION` | `check_option`, carried unchanged |
  | `IS_UPDATABLE` | `is_updatable` |
  | `DEFINER` | `definer` |
  | `SECURITY_TYPE` | `security_type` |
  | `CHARACTER_SET_CLIENT` | `character_set_client`, passed through |
  | `COLLATION_CONNECTION` | `collation_connection`, passed through |

  `algorithm` SHALL be `null`: MySQL's view catalogue has no algorithm field.
  A view SHALL NOT carry its columns, as on every provider, per the note on
  `FR-CAT-052`.

  *Pending observation.* Ten fields, and no `ALGORITHM`, on
  `refman/8.4/en/information-schema-views-table.html` and
  `refman/26.10/en/information-schema-views-table.html`. `CHECK_OPTION` is
  documented as "one of `NONE`, `CASCADE`, or `LOCAL`", where MariaDB was
  observed to return `CASCADED`, per `FR-CAT-047`; the value is carried as the
  server returns it, and rmp `#326` records it. `VIEW_DEFINITION` "has most of
  what you see in the `Create Table` column that `SHOW CREATE VIEW` produces",
  and the table requires the `SHOW VIEW` privilege; what a reader without it
  sees is `FR-MYSQL-041`'s.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-036**: For a `mysql` read, a routine SHALL carry the properties
  `FR-CAT-048` gives it, from the fields of the routine catalogue, `ROUTINES`,
  of the same names, with the same exclusions: the specific name as a
  restatement; the external name, the external language and the SQL path,
  which the model does not carry; and the creation and alteration times as
  volatile, per `FR-CAT-024`. A procedure's return type SHALL be `null` in
  full, per `FR-CTX-005`, whatever its type fields hold. The routine body SHALL
  be `null` WHERE `ROUTINE_DEFINITION` is SQL `NULL`, and the routine is then
  incomplete, per `FR-PRIV-017`.

  *Pending observation.* The 31 fields, by the names `FR-CAT-048` reads, on
  `refman/8.4/en/information-schema-routines-table.html` and
  `refman/26.10/en/information-schema-routines-table.html`. Three documented
  values differ from MariaDB's observed ones: a procedure's `DTD_IDENTIFIER`
  "is empty", where MariaDB returns SQL `NULL`; `EXTERNAL_LANGUAGE` is "The
  language of the stored routine", where MariaDB returned SQL `NULL` on every
  routine; and `SQL_DATA_ACCESS` may be `CONTAINS SQL`, which `FR-CAT-048`
  records MariaDB not returning over its fixture. `ROUTINE_BODY` is "always
  `SQL`", and `ROUTINE_DEFINITION` "is `NULL` if you have only
  `CREATE ROUTINE`, `ALTER ROUTINE`, or `EXECUTE`".

  *Consequence, stated plainly.* A routine whose language is not SQL, which
  `EXTERNAL_LANGUAGE` admits, is carried with `body_kind` `SQL` and its body,
  and nothing in the model names its language.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-037**: For a `mysql` read, a routine's parameters SHALL be the
  rows of the parameter catalogue, `PARAMETERS`, carrying the properties
  `FR-CAT-049` gives them from the fields of the same names, in declaration
  order. A function's return value, the row at ordinal position `0` with mode
  and name SQL `NULL`, SHALL NOT be presented as a parameter.

  *Pending observation.* Sixteen fields, the sixteen `FR-CAT-049` records on
  MariaDB's three oldest series, and the return row as stated
  (`refman/8.4/en/information-schema-parameters-table.html`).

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-038**: For a `mysql` read, a table's triggers SHALL be the rows
  of the trigger catalogue, `TRIGGERS`, whose event object table is that
  table, carrying the properties `FR-CAT-050` gives them from the fields of
  the same names, with the same exclusions.

  *Pending observation.* Twenty-two fields, the twenty-two of `FR-CAT-050`
  (`refman/8.4/en/information-schema-triggers-table.html`): `ACTION_ORDER` is
  "The ordinal position of the trigger's action within the list of triggers on
  the same table with the same `EVENT_MANIPULATION` and `ACTION_TIMING`
  values"; `ACTION_CONDITION` and the two table-reference fields are "always
  `NULL`"; `ACTION_ORIENTATION` is "always `ROW`"; the row references are
  "always `OLD`" and "always `NEW`"; and "To see information about a table's
  triggers, you must have the `TRIGGER` privilege for the table", which
  `FR-MYSQL-041` reaches.

  *Added in the seventy-third edition,* for rmp `#324`.

- **FR-MYSQL-039**: Where the model defines a value or a property that MySQL
  has no source for, a `mysql` read SHALL carry what this table states:

  | The model | On a `mysql` read |
  |---|---|
  | `table_type` `SYSTEM VERSIONED`, and a system-versioned table's hidden period columns | never: MySQL has no system-versioned table |
  | A sequence, excluded by `FR-CAT-004` and `FR-CAT-020` | never present to exclude |
  | An application-time period, excluded by `FR-CAT-022` | never present to exclude |
  | A view's `algorithm` | `null`, per `FR-MYSQL-035` |
  | A `CHECK` constraint's `level` | `null`, per `FR-MYSQL-034` |
  | The implicit `json_valid` constraint of `FR-CAT-038` | never: MySQL attaches none |
  | The `inet4`, `inet6` and `uuid` types | never: MySQL has none of them |

  *Pending observation.* Each "never" is the absence of the feature from the
  manuals cited in this file, which no page states in terms; rmp `#326`
  confirms it over the catalogue the fixture produces, under the sixth
  validation rule of the [README](README.md#maintenance-debt): a record of an
  absence rests on an enumeration of what the catalogue returns.

  *Added in the seventy-third edition,* for rmp `#324`.

## Divergences between MySQL and MariaDB

- **FR-MYSQL-040**: Every divergence between MySQL and MariaDB that reaches a
  read SHALL be recorded in the following table, naming what this corpus
  records of MariaDB, what MySQL's documentation states, and what the
  divergence obliges. A divergence that an observation of rmp `#326` finds and
  this table does not hold is a missing row.

  | # | Subject | MariaDB, as this corpus records it | MySQL, as its documentation states | What it obliges |
  |---|---|---|---|---|
  | 1 | The version string | `<x.y.z>-MariaDB`, with a build suffix, `FR-SRV-040` | `<x.y.z>`, with an optional suffix and no product marker | `FR-MYSQL-006`, `FR-MYSQL-007` |
  | 2 | The read-only read-back | `tx_read_only`, present on all four series, `FR-SRV-009` | `tx_read_only` removed; `transaction_read_only` | `FR-MYSQL-018`, `FR-MYSQL-012` |
  | 3 | A string default | quoted, `'EUR'`, `FR-CTX-037` | unquoted | `FR-MYSQL-029` |
  | 4 | A default of `NULL` | the four characters `NULL`, `FR-CTX-012` | SQL `NULL` | `FR-MYSQL-029` |
  | 5 | An expression default | told apart by its shape, `FR-CTX-037` | marked `DEFAULT_GENERATED` in the attribute field | `FR-MYSQL-029`, `FR-MYSQL-030` |
  | 6 | A generated column | an is-generated field beside the attribute field, `FR-CAT-051` | the attribute field alone | `FR-MYSQL-031` |
  | 7 | The check-constraint table | six fields, carrying the table and the level, `FR-CAT-046` | four fields, carrying neither; the table, and enforcement, in the table-constraint table | `FR-MYSQL-034` |
  | 8 | `JSON` | an alias of `longtext`, with an implicit `json_valid` constraint, `FR-CAT-038` | a native type, `json`, with none | `FR-MYSQL-028`, `FR-MYSQL-034` |
  | 9 | An invisible column | the attribute field reads `INVISIBLE`, `FR-CAT-035` | the attribute field carries `INVISIBLE`, and may carry it beside other phrases | `FR-MYSQL-030` |
  | 10 | An index on an expression | none; an expression is indexed through a generated column, `FR-CAT-042` | a functional key part, with no column name and an expression | `FR-MYSQL-024` |
  | 11 | An index the optimizer does not use | an ignored field, `FR-CAT-042` | a visibility field of the opposite polarity | `FR-MYSQL-032` |
  | 12 | An integer type's display width | always present, `FR-CTX-038` | absent, save `tinyint(1)` and `ZEROFILL` | `FR-MYSQL-028` |
  | 13 | A view's cascaded check option | `CASCADED`, `FR-CAT-047` | documented as `CASCADE` | carried unchanged, `FR-MYSQL-035` |
  | 14 | A view's algorithm | a field of the view catalogue, `FR-CAT-047` | no such field | `null`, `FR-MYSQL-035` |
  | 15 | The schema catalogue's sixth field | a schema comment, `FR-CTX-036` | the default encryption | neither carried, `FR-MYSQL-025` |
  | 16 | A spatial index's type | `SPATIAL`, `FR-CAT-042` | the documented list is `BTREE`, `FULLTEXT`, `HASH`, `RTREE` | carried unchanged, `FR-MYSQL-032` |
  | 17 | The table types | five, and `TEMPORARY` on three series, `FR-CAT-031` | `BASE TABLE`, `VIEW`, `SYSTEM VIEW`; no temporary table listed | `FR-MYSQL-023` |
  | 18 | `ON ... SET DEFAULT` | accepted, reported `RESTRICT`, `FR-CAT-033` | refused at DDL time by InnoDB and NDB | `FR-CAT-033` holds; `FR-MYSQL-033` |
  | 19 | A referential action not written | not recorded by this corpus | `NO ACTION` | `FR-MYSQL-033` |
  | 20 | A procedure's DTD identifier | SQL `NULL`, `FR-CAT-048` | empty | none: the return type is `null`, `FR-MYSQL-036` |
  | 21 | A routine's external language | SQL `NULL` on every routine, `FR-CAT-048` | the routine's language | not carried, `FR-MYSQL-036` |
  | 22 | Identifier letter case | not recorded by this corpus | decided by `lower_case_table_names`, which no statement reads | names matched byte for byte, `FR-MYSQL-033` |
  | 23 | An expression's text | backtick-quoted and rewritten, no backslash observed, `FR-CAT-051`, `FR-CTX-037` | not stated; the reference inspector unescapes MySQL's | `FR-MYSQL-029`, `FR-MYSQL-031` |
  | 24 | A server with no TLS material | `10.11` offers no TLS until configured, difference 3 of `FR-SRV-038` | generates material at startup where built with OpenSSL | `FR-MYSQL-021` |
  | 25 | The default authentication | not recorded by this corpus | `caching_sha2_password` | `FR-MYSQL-022`, [OQ-077](open-questions.md#index) |

  *Pending observation.* Every cell of the MySQL column rests on the page
  cited by the requirement the last column names, and on nothing observed.
  Every cell of the MariaDB column is an observation this corpus records, or
  says that it records none.

  *Rationale.* `FR-SRV-038` keeps a record of every difference between the
  MariaDB series because a difference without a home is rediscovered or
  contradicted. Two products are further apart than two series, and the
  coordinator asked for the divergences to be stated: default quoting, the
  check-constraint table, `JSON`, invisible columns, functional indexes, the
  version string and identifier case are rows 3 to 5, 7 to 10, 1 and 22.

  *Added in the seventy-third edition,* for rmp `#324`.

## Completeness

- **FR-MYSQL-041**: The rules of
  [privileges-and-completeness.md](privileges-and-completeness.md) SHALL hold
  for a `mysql` read as for a `mariadb` read: a read is complete or
  incomplete under `FR-PRIV-001` and `FR-PRIV-002`, an incomplete named object
  is `77` under `FR-PRIV-003`, an incomplete object in a listing or a dump is
  marked under `FR-PRIV-005` through `FR-PRIV-007`, and the limit of
  `FR-PRIV-020` holds for triggers. Until an observation of rmp `#326`
  records the shapes in which MySQL gives a privilege-driven absence, the
  cross-checks of `FR-PRIV-011`, `FR-PRIV-017` and `FR-PRIV-019` SHALL be
  applied to a `mysql` read in the shapes `FR-PRIV-018` records for MariaDB,
  and this corpus SHALL NOT claim that they detect every such absence of a
  `mysql` read.

  *Pending observation.* The manual states that `ROUTINE_DEFINITION` "is
  `NULL` if you have only `CREATE ROUTINE`, `ALTER ROUTINE`, or `EXECUTE`",
  the shape `FR-PRIV-017` detects; that the trigger catalogue shows a
  trigger only to a reader holding the `TRIGGER` privilege on its table,
  which is the absence `FR-PRIV-020` cannot tell from no trigger; and that
  the view catalogue needs the `SHOW VIEW` privilege, without stating what a
  reader without it sees. rmp `#326` reads the fixture as its reduced-grant
  reader on every series and records the shape of each absence, as
  `FR-PRIV-018` records it.

  *Added in the seventy-third edition,* for rmp `#324`.

## Requirements of form

- **FR-MYSQL-042**: For a `mysql` entry, the requirements of form of
  [performance-requirements.md](performance-requirements.md) SHALL hold as
  they hold for a `mariadb` entry: the number of catalogue statements a full
  read and a named read issue SHALL NOT depend on the number of objects, per
  `NFR-PERF-001` and `NFR-PERF-002`; a cache hit SHALL open no connection and
  issue no catalogue query, per `NFR-PERF-003`; an invocation SHALL open at
  most one connection, per `NFR-PERF-004`; a command that needs no catalogue
  SHALL open none, per `NFR-PERF-006`; and the count SHALL be observable as
  `NFR-PERF-008` requires. Each SHALL be verified from the server's records,
  per `NFR-PERF-007`, against the fixture of `FR-MYSQL-044`, on every series
  of `FR-MYSQL-003`. The equality of `NFR-PERF-001` SHALL be taken over the
  fixture's database and over a database that holds no object.

  *Rationale.* A statement that reads one object makes the count grow with the
  objects, which these requirements forbid, and MySQL's catalogue needs none:
  each of the twelve tables of `FR-MYSQL-015` can be restricted to the
  database alone, and a check constraint is related to its table by its
  schema and its name, not by a statement per table. Which statements read
  them is not specified here.

  *Added in the seventy-third edition,* for rmp `#324`.

## Verification

- **FR-MYSQL-043**: The verifications [server-contract.md](server-contract.md)
  requires for a `mariadb` entry SHALL be made for a `mysql` entry, each with
  the set of `FR-MYSQL-003` in place of the set of `FR-SRV-015` and the
  fixture of `FR-MYSQL-044` in place of `scripts/mariadb/`:

  | Requirement | What it verifies for a `mysql` entry |
  |---|---|
  | `FR-SRV-012` | The closed list of `FR-MYSQL-015`, observed on the server, and the order of `FR-MYSQL-017` |
  | `FR-SRV-013` | Both outcomes of the read-back of `FR-MYSQL-018`, the confirming one on every series, the failing one in process through the seam it authorises |
  | `FR-SRV-014` | The connection count, from the server side |
  | `FR-SRV-029` | The equivalence of `FR-SRV-026` across the three series, and the refusal of `FR-MYSQL-009` against at least one series outside the set |
  | `FR-SRV-035` | The marked read of `FR-MYSQL-010`, through the seam it authorises |
  | `BR-SRV-003` | That the first three are observations made outside the process, with the one exception it names |

  The product refusal of `FR-MYSQL-008` SHALL be verified by an integration
  test against a MariaDB server of the fixture of `scripts/mariadb/`, of a
  series whose read-back a `mysql` entry passes, per `FR-MYSQL-012`.

  *Added in the seventy-third edition,* for rmp `#324`.

## The fixture

- **FR-MYSQL-044**: The fixture of `scripts/mysql/` SHALL present one MySQL
  server of each series of `FR-MYSQL-003`, built from versioned files, and
  SHALL be operated by a harness with the contract of the harness of
  `scripts/mariadb/`: scripts that start the fixture and verify it server by
  server, stop it and prove that nothing of it was left running, answer
  whether it is up before a server-dependent test runs, and instrument the
  observations the specification requires to be made outside the process —
  the statements each server receives and the connections it accepts, per
  `NFR-PERF-007` and `BR-SRV-003`. What the directory holds, how the harness
  is invoked and what each script does SHALL be described in
  `scripts/mysql/README.md`.

  Its `setup.sql` and `seed.sql` SHALL model a realistic domain, never
  `id=1, name='test'`, SHALL be accepted by every series of `FR-MYSQL-003`,
  and SHALL exercise every case this file states:

  1. tables of the InnoDB engine and of at least one other engine;
  2. every native data type every series accepts: the integer types signed
     and unsigned, one with `ZEROFILL` and one `tinyint(1)`; `decimal`,
     `float`, `double` and `bit`; `date`, `year`, and `datetime`, `timestamp`
     and `time` with and without fractional seconds; the character, text,
     binary and blob types; `enum` with members holding a comma and an
     apostrophe, and `set`; `json`; and the geometry types, one with an
     `SRID`;
  3. every row of the default classification of `FR-MYSQL-029`: no default
     on a `NOT NULL` and on a nullable column, `DEFAULT NULL`, a string
     literal, the empty string, the string `'NULL'`, a literal holding an
     apostrophe, a number, a bit literal, a binary literal,
     `CURRENT_TIMESTAMP` with and without fractional seconds, and
     parenthesised expressions on a scalar column and on a `TEXT`, `BLOB` and
     `JSON` column, one of them holding a string literal;
  4. an auto-incremental column, an invisible column, and each combination
     of phrases `FR-MYSQL-030` names for observation;
  5. a virtual and a stored generated column, one whose expression holds a
     string literal, and an index on a generated column;
  6. simple and composite primary keys, a table with no primary key, and a
     table with a `NOT NULL` unique index and no primary key;
  7. unique and composite indexes, a descending member, a prefix, a
     `FULLTEXT` and a `SPATIAL` index, an invisible index, an index with a
     comment, a functional index, a `UNIQUE` functional index and a
     multi-valued index;
  8. foreign keys with distinct `ON UPDATE` and `ON DELETE` rules among
     `CASCADE`, `SET NULL`, `RESTRICT` and `NO ACTION`, one written with
     neither clause, a composite key and a self-reference, and a second
     schema holding a table that a table of the main schema references and a
     table that references one;
  9. `CHECK` constraints at column and at table level, named and unnamed, one
     `NOT ENFORCED`, and one whose clause holds a string literal;
  10. views with each check option and each security type, updatable and
      not, and with each algorithm a view can declare;
  11. stored procedures and functions with `IN`, `OUT` and `INOUT`
      parameters, deterministic and not, with each data-access
      characteristic, with a comment, and with each security type;
  12. triggers of each timing on each event, and two on one table with the
      same event and timing;
  13. comments on tables, columns, indexes and routines, one holding a
      character outside the basic multilingual plane;
  14. identifiers carrying a space, a backtick and a double quote;
  15. a scheduled event and a partitioned table, which the read excludes; and
  16. a reduced-grant reader holding `SELECT` and `EXECUTE` on the main
      schema and nothing more, and a schema declaring no collation of its own.

  A structure that a series of `FR-MYSQL-003` refuses SHALL be omitted from
  the shared DDL rather than made conditional, and the refusal SHALL be
  recorded under `FR-MYSQL-014`, as `FR-SRV-029` records difference 14 for
  MariaDB.

  The fixture SHALL also present, at each series of `FR-MYSQL-003`, a server
  whose certificate names the host by which the project's tests reach it;
  SHALL retain a server that offers no TLS; and SHALL present at least one
  server of a MySQL series outside `FR-MYSQL-003`, for the refusal of
  `FR-MYSQL-009`. The condition each server runs under that a reading depends
  on — its TLS material, its `lower_case_table_names`, and its
  `show_gipk_in_create_table_and_information_schema` — SHALL be recorded
  with every reading taken against it, per the fourth validation rule of the
  [README](README.md#maintenance-debt).

  *Rationale.* It is what `FR-CONF-038` and the root `CLAUDE.md` require of
  `scripts/mariadb/`, and `FR-LITE-038` of `scripts/sqlite/`, written for the
  catalogue a MySQL server reads: every statement of this file that is marked
  *Pending observation* names a case of this list, and rmp `#326` takes the
  observations through it.

  *Added in the seventy-third edition,* for rmp `#324`.

## Dependencies

- [configuration-model.md](configuration-model.md) — `FR-CONF-051` and
  `FR-CONF-052`, the `provider` key and the keys a `mysql` entry takes; the
  connection, DSN, password and TLS rules `FR-MYSQL-020` adopts; and
  `FR-CONF-038`, whose fixture obligation `FR-MYSQL-044` restates.
- [server-contract.md](server-contract.md) — the contract this file stands
  beside. It adopts, each where it is cited: `FR-SRV-004`, `FR-SRV-005`,
  `FR-SRV-022` to `FR-SRV-026` and `FR-SRV-037`, with `FR-SRV-039` and
  `BR-SRV-006`, in `FR-MYSQL-013`; `FR-SRV-027`, `FR-SRV-036` and
  `FR-SRV-038` in `FR-MYSQL-014`; `FR-SRV-031` to `FR-SRV-033`,
  `BR-SRV-008` and `BR-SRV-009` in `FR-MYSQL-010`; `FR-SRV-034` in
  `FR-MYSQL-011`; `FR-SRV-041` in `FR-MYSQL-007`; `FR-SRV-042` in
  `FR-MYSQL-017`; `FR-SRV-010` and `FR-SRV-011` in `FR-MYSQL-019`;
  `BR-SRV-002` in `BR-MYSQL-002`; `BR-SRV-004` and `BR-SRV-005` in
  `BR-MYSQL-001`; and `FR-SRV-012` to `FR-SRV-014`, `FR-SRV-029`,
  `FR-SRV-035` and `BR-SRV-003` in `FR-MYSQL-043`. `FR-SRV-003`,
  `FR-SRV-009` and `FR-SRV-010` are the requirements `FR-MYSQL-012` reads,
  unchanged, for a `mariadb` entry on a MySQL server.
- [sqlite-contract.md](sqlite-contract.md) — `FR-LITE-021`, whose exclusion
  of an expression index `FR-MYSQL-024` shares, and `FR-LITE-027`, whose
  reading of two fields `FR-MYSQL-029` shares.
- [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) — the model and the document
  this file fills; `FR-CTX-043`, the `provider` field.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the
  rules `FR-MYSQL-041` applies.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `77` and
  `78`; the condition order of `FR-ERR-006`; and the message rules
  `FR-MYSQL-008` and `FR-MYSQL-009` follow.
- [performance-requirements.md](performance-requirements.md) —
  `NFR-PERF-001` to `NFR-PERF-008`, which `FR-MYSQL-042` applies.
- [template-environment.md](template-environment.md) — `FR-ENV-046`, whose
  fall-through places the `data_type` values only MySQL produces.
- [security.md](security.md) — `BR-SEC-002`, the promise `BR-MYSQL-002`
  states for a `mysql` entry.
- [open-questions.md](open-questions.md) — `OQ-076` and `OQ-077`.

## Open questions

Two entries this file raises are open in the index of
[open-questions.md](open-questions.md#index):

| Entry | What it decides | What waits on it |
|---|---|---|
| [OQ-076](open-questions.md#index) | Which releases form one major family under calendar versioning, for the first condition of `FR-MYSQL-002` | The re-verification of `FR-MYSQL-005` at the first release on which the readings select different series; not the table of 2026-11-04 |
| [OQ-077](open-questions.md#index) | How `caching_sha2_password` authenticates an account over a session TLS does not protect | The authentication outcome of `disabled`, and of `preferred` against a server offering no TLS, under `FR-MYSQL-022` |

The statements marked *Pending observation* are not open questions: each has
an owner, rmp `#326`, and a fixture, `FR-MYSQL-044`, and the
[README](README.md#maintenance-debt) records them as the one item of
maintenance debt this edition leaves.
