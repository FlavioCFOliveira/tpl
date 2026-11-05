---
title: MySQL Contract
status: approved
last-reviewed: 2026-11-05
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

## Sources, the observation, and the pending-observation marker

Every statement this file makes about MySQL's behaviour rests on one of three
grounds:

1. **An observation of the fixture of `FR-MYSQL-044`**, the fourth provenance
   of the [README](README.md#provenance). rmp `#326` made it on 2026-11-04,
   through `scripts/mysql/` at `1b3ff53`, operated by its harness alone,
   against five servers: `8.4.13` and `9.7.4`, one of each series of
   `FR-MYSQL-003`, each offering TLS; a server of each of the same two
   releases offering none; and `8.0.46`, of a series outside the set. A
   statement it grounds carries an *Observed* note. The note names the series
   it was seen on; where it names none, the fact was seen on `8.4` and on
   `9.7` and was the same on both. Every difference between the two series is
   recorded under `FR-MYSQL-014`.
2. **MySQL's own documentation**, a published external authority in the sense
   of the third provenance of the [README](README.md#provenance), checked on
   2026-11-04: the reference manuals of the two supported series, at
   `https://dev.mysql.com/doc/refman/8.4/en/` and
   `https://dev.mysql.com/doc/refman/9.7/en/`, and the reference manual of
   the `26.10` Innovation release, at
   `https://dev.mysql.com/doc/refman/26.10/en/`, which is cited for MySQL's
   release model and, beside a page of a supported series, as a later manual
   stating the same fact — `26.10` is not a supported series, per
   `FR-MYSQL-002`; the release notes, at
   `https://dev.mysql.com/doc/relnotes/mysql/<series>/en/`; the download page,
   at `https://dev.mysql.com/downloads/mysql/`; the end-of-life notices, at
   `https://www.mysql.com/support/eol-notice.html`; and the support tiers, at
   `https://www.mysql.com/support/`. A page is cited below by its path under
   the manual of the series named, as `refman/8.4/en/<page>.html`. It grounds
   the statements still marked pending observation, below, and is cited
   beside an observation where it states the same fact or a different one.
   **Where the servers contradict their manual, both are written down**: the
   manual's words, cited by page, and what the servers did, in a note headed
   *Observed, and the manual contradicted.* The observation governs.
3. **Prior art**, which informs how and never what: the MySQL inspector of
   `ariga/atlas`, `sql/mysql/inspect.go` and `sql/mysql/driver.go` at the
   public source of its default branch, and
   `sql/mysql/internal/mysqlversion/mysqlversion.go` at tag `v0.21.1`, read on
   2026-11-04. It is cited where it bears on a choice this file makes, and it
   establishes no fact here.

**The conditions of observation**, recorded once for every *Observed* note
of this file, per the fourth validation rule of the
[README](README.md#maintenance-debt). Every server ran with
`lower_case_table_names` `0`,
`show_gipk_in_create_table_and_information_schema` `ON` and
`sql_generate_invisible_primary_key` `OFF`, the servers' own defaults, which
the fixture does not change. The three servers offering TLS presented the
fixture's own certificate, valid from 2026-11-04 to 2036-11-01; the two others
offered none, with `tls_version` set to the empty string. On both series
`information_schema_stats_expiry` read `86400`,
`restrict_fk_on_non_standard_key` `ON` and `authentication_policy` `*,,`;
`gtid_mode` and `enforce_gtid_consistency` read `OFF` on `8.4` and `ON` on
`9.7`, which is difference 6 of `FR-MYSQL-014`. Every statement was issued by
the stock `mysql` client of the server's own image, in the character set
`utf8mb4`, as `root` and as the reduced-grant reader of item 16 of
`FR-MYSQL-044`. Every statement that read the catalogue ran in a session set
read only, exited `0` and drew no warning. Every write the observation needed
was made in a scratch schema and with scratch accounts, all dropped before
the fixture was stopped, and no object of the fixture was changed. One build
of each release was read, the image the fixture pins, and nothing was
observed of another build or another release of either series.

**What a substitute client cannot show.** The stock client stood in for the
driver `tpl` will read MySQL with. What a server does is observed through it;
an exit code or a message of `tpl` is not, and is verified by the tests
`FR-MYSQL-043` requires once the provider is implemented. Where a requirement
rests on the driver meeting a server as the stock client met it, its note
says so.

**Three statements remain marked pending observation**, each with a note
headed ***Pending observation.***, the marker the
[README](README.md#writing-conventions) defines and
[glossary.md](glossary.md#pending-observation) names. Each note says what the
documentation states, why this fixture cannot observe it, and what would. The
marker qualifies the ground of a fact and not the force of a requirement:
every requirement here is in force.

| Note on | The statement | Why this fixture cannot observe it |
|---|---|---|
| `FR-MYSQL-021` | That a server generates TLS material at startup, and whether the certificate names a host | Every server of the fixture is given its TLS material, so none generated a certificate |
| `FR-MYSQL-030` | The `MASKING POLICY` phrase of the attribute field | It needs a component the Community images do not carry |
| `FR-MYSQL-040` | The MySQL cell of row 24 | It restates the first statement above |

What this file states about MariaDB is cited from the requirement of this
corpus that records it, every one of them an observation of the fixture of
`scripts/mariadb/`, and carries no marker.

*Added in the seventy-third edition,* for rmp `#324`.

*Amended in the seventy-fourth edition,* for rmp `#331`: the first source
named the manuals of three supported series. `26.10` is no longer one, by the
user's decision recorded under `FR-MYSQL-002`, and its manual stays a source
for the release model and as corroboration only. A *Pending observation* note
owes an observation on the series of `FR-MYSQL-003` alone, so no note of this
file owes one of `26.10`.

*Amended in the seventy-fifth edition,* for rmp `#326`. The section read that
no statement of MySQL behaviour in this file had been observed, because the
fixture of `FR-MYSQL-044` did not exist, and that every such statement carried
a *Pending observation* note. The fixture exists, and rmp `#326` observed the
thirty statements so marked — twenty-nine in this file and one on
`FR-ERR-046` in [errors-and-exit-codes.md](errors-and-exit-codes.md). Sixteen
were confirmed, ten refined and four contradicted. A confirmed statement now
carries an *Observed* note, a refined one is rewritten as observed, and a
contradicted requirement is amended where it stands, each with a note saying
what it read. The section now names the observation as the first ground, so
the documentation, which the note above calls the first source, is the
second; it records the conditions of the observation once, and lists the six
statements that remain pending. The observation's evidence is kept in a directory the repository
does not version, so every *Observed* note carries what was seen rather than
citing a file.

*Amended in the seventy-seventh edition,* for two follow-ups of rmp `#329`.
The section counted six statements pending. Two were codes of `tpl` — the
`69` of three cells of `FR-MYSQL-021` and the `78` and the message of
`FR-MYSQL-045` — that the stock client could not show; the provider exists,
at `2d652bd`, its tests observed both, and each note is now an *Observed*
note, through `tpl`. A third, whether trust material stays additional to the
public roots, waited for the driver that reads MySQL to be chosen; the one
chosen is the driver `FR-CONF-039` observed, and its note now says so. The
table lists the three that remain, each a server the fixture does not
stage, and its last row restates the first statement, where it restated the
second.

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

  *Note added in the seventy-seventh edition,* for rmp `#332`, `#333` and
  `#334`. The requirement that nothing about MariaDB support changes is the
  user's for the `mysql` provider. On 2026-11-04 the user decided, apart
  from it, to fix in sprint 28 three defects of a `mariadb` entry found while
  the provider was built, accepting that one of them changes a message a
  `mariadb` entry prints while its exit code stays the same. They are
  `FR-ERR-047`, `FR-SRV-030` and `FR-CONF-055`, and each records the
  decision.

## The supported version window

- **FR-MYSQL-002**: For a `mysql` entry, the system SHALL support MySQL and
  no other product, and SHALL support exactly those MySQL release series that
  satisfy **both** of the following conditions:

  1. Oracle designates the series an
     [LTS release](glossary.md#lts-release--innovation-release); and
  2. the series is under Oracle **Premier Support** or **Extended Support**.

  An [Innovation release](glossary.md#lts-release--innovation-release) SHALL
  NOT be a supported series, whatever its number and for as long as Oracle
  supports it. A series, an LTS release and an Innovation release have the
  meanings [glossary.md](glossary.md) gives them. Series SHALL be compared by
  version number — major number first, then minor number, each compared as an
  integer — so that `8.4` is older than `9.7`, `9.7` older than `26.7`, and
  `26.7` older than `26.10`. A point release does not enter the comparison:
  `8.4.13` and `8.4.2` are one series, as `FR-SRV-021` states for MariaDB.

  *Provenance.* The user's decision on `OQ-076`, relayed for rmp `#331` on
  2026-11-04: the supported MySQL series are the latest release of each MySQL
  LTS line still under Oracle Premier or Extended Support, which on that date
  are `8.4` and `9.7`; Innovation releases are not supported, `26.10` and every
  later Innovation release numbered by the calendar among them; and a future
  LTS line joins the set when it is released, and a line leaves when it moves
  past Extended Support. The first condition is *LTS line*; the second is
  *still under Oracle Premier or Extended Support*. The user's statement of
  the set names series, so *the latest release of each LTS line* admits an LTS
  series through every release of it, as the seventy-third edition read *the
  latest release of each major series*.

  *Why no condition names the newest series.* An LTS line is one series for
  its whole support: "When a release is designated as an LTS release, its
  YY.M release base remains fixed throughout the support lifecycle, and
  subsequent maintenance releases increment the P component", per
  `refman/26.10/en/mysql-releases.html`, checked 2026-11-04. Its latest
  release is therefore a point release of the series, which the comparison
  above does not read.

  *Why Premier and Extended Support, and not Sustaining Support.* Oracle's
  support page gives Premier Support "MySQL maintenance releases, updates,
  fixes", Extended Support "error correction", and Sustaining Support "No new
  releases, no new fixes", at `https://www.mysql.com/support/`, checked
  2026-11-04. Sustaining Support is indefinite: read as support, it would
  admit `5.7` and every series before it, which the user's examples for rmp
  `#324`, "8.x.x, 9.x.x, …", exclude. The two tiers admitted are the two
  under which a series still changes, which is the ground `FR-SRV-001` gives
  for following MariaDB's community maintenance.

  *What excluding Innovation releases changes.* Oracle supports an Innovation
  release while it is current: "All LTS and Innovation releases include bug
  and security fixes, and are considered production-grade quality", and
  "Innovation releases are supported until the next Innovation release", per
  `refman/26.10/en/mysql-releases.html`. The exclusion is therefore the user's
  choice and not a reading of Oracle's policy, where `FR-SRV-016` excludes a
  MariaDB rolling release by reading MariaDB's: a rolling release receives no
  maintenance after GA. The set no longer moves when Oracle publishes an
  Innovation release, which it did on 2026-07-28 and on 2026-10-20, per the
  release notes of the `26.10` series; it moves when an LTS series is first
  released and when one leaves Extended Support, per `FR-MYSQL-004`. What a
  server of an Innovation release meets is `FR-MYSQL-009` below the newest
  supported series and `FR-MYSQL-010` above it.

  *Rejected, by the user: the compatibility lineage.* It was the first reading
  `OQ-076` named, recommended there, and the one the seventy-third edition
  applied: under calendar versioning a major family was the Innovation
  releases before an LTS release and the LTS release that closes them, and
  the newest series of each family was supported. It admitted the newest
  Innovation release while it was current, which was `26.10` on 2026-11-04.
  *Rejected, by the user: the calendar year.* It was the second reading: the
  leading number of a calendar version, which is the year, taken as the major
  family. It would have dropped an LTS series from the set as soon as an
  Innovation release of the same year followed it, while Oracle still
  supported the LTS series. *Not the rule either: every series under Premier
  or Extended Support, with no per-family restriction*, the third reading
  `OQ-076` named. It admits every Innovation release while it is current,
  which the decision excludes.

  *Rejected: the window of `FR-SRV-001`, three most recent families under
  maintenance.* It is MariaDB's criterion, not the user's, and it would admit
  `8.0` beside `8.4` while both were maintained.

  *Closes* `OQ-076`, now listed under [Closed](open-questions.md#closed).

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, by the user's
  decision on `OQ-076`. The first condition read "the series is the newest
  series Oracle has released in its major family", and the second added that
  Premier or Extended Support, "for an Innovation release, lasts until the
  next Innovation release is published", so that the newest Innovation
  release was supported while it was current. The criterion now admits LTS
  series alone, and a major family no longer enters it. The comparison of
  series, which `FR-MYSQL-009` and `FR-MYSQL-010` apply, is unchanged.

- **FR-MYSQL-003**: The series that satisfy `FR-MYSQL-002` SHALL be the
  following, as verified on the date recorded below:

  | Series | Track | First release of the series | Latest release, as of 2026-11-04 | Support |
  |---|---|---|---|---|
  | `9.7` | LTS | `9.7.0`, 2026-04-21 | `9.7.4`, 2026-10-20 | five years of Premier and three of Extended Support from general availability |
  | `8.4` | LTS | `8.4.0`, 2024-04-30 | `8.4.13`, 2026-10-20 | five years of Premier and three of Extended Support from general availability |

  *Source.* The release notes of each series, at
  `https://dev.mysql.com/doc/relnotes/mysql/9.7/en/` and
  `https://dev.mysql.com/doc/relnotes/mysql/8.4/en/`, for the releases and
  their dates; the download page, which offers `26.10.0`, `26.7.0`,
  `9.7.4 LTS`, `8.4.13 LTS` and `8.0.46`; the preface of the 9.7 manual, which
  "covers the 9.7 LTS series"; and `refman/8.4/en/mysql-releases.html`, for
  "An LTS series follows the Oracle Lifetime Support Policy, which includes 5
  years of premier support and 3 years of extended support". **Verified
  2026-11-04.**

  *Derived, and not read.* The support column of an LTS series states the
  durations the manual gives and no date. Counted from the first release of
  each series, they end Premier Support in 2029 for `8.4` and in 2031 for
  `9.7`, and Extended Support in 2032 and 2034. The Oracle Lifetime Support
  Policy document that would state the days refused retrieval, with HTTP 403,
  on 2026-11-04, so the years are a count and the days are not recorded.

  *Worked examples of the two conditions.* Each of these is unsupported:

  | Series | Excluded because |
  |---|---|
  | `26.10` | it is an Innovation release: the preface of its manual names "the 26.10.0 Innovation release" |
  | `26.7` | it is an Innovation release, whose support also ended when the next, `26.10.0`, was published on 2026-10-20 |
  | `9.0` to `9.6` | each is an Innovation release, whose support ended with the next |
  | `8.0` | "as of April 21, 2026, MySQL 8.0 is covered under Oracle Sustaining Support", per the end-of-life notices |
  | `5.7` | "as of October 25, 2023, MySQL 5.7 is covered under Oracle Sustaining Support", per the same page |

  Where each of these servers is refused and where it is read is stated by
  `FR-MYSQL-009` and `FR-MYSQL-010`, by comparison with the newest series of
  the table above.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended. The row of `26.10`, an Innovation release, is
  removed from the table, and the major-family column with it, because the
  criterion names no family. The two remaining rows are those verified on
  2026-11-04, unchanged, and the sources of the removed row are no longer
  cited for it. `26.10` joins the worked examples, and each example now gives
  the condition it fails under the amended criterion.

- **FR-MYSQL-004**: The set of `FR-MYSQL-003` SHALL change by the two
  conditions of `FR-MYSQL-002` and by nothing else, and its size SHALL NOT be
  fixed by this specification. Two series is what the criterion admits on the
  verification date, not a quantity to preserve.

  | Event | Effect |
  |---|---|
  | Oracle publishes the first release of a series it designates LTS | The series enters; no series leaves |
  | Oracle publishes a later release of an LTS series | Nothing enters and nothing leaves: the release belongs to a series of the set, per the comparison of `FR-MYSQL-002` |
  | Oracle publishes an Innovation release | Nothing enters and nothing leaves |
  | An LTS series leaves Extended Support | It leaves |

  *Rationale.* It is `FR-SRV-017` and `FR-SRV-018` for MySQL's release model,
  and the user's words for it: a future LTS line joins the set when it is
  released, and a line leaves when it moves past Extended Support.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended. The table had an Innovation release enter the set
  and push out the one it succeeded, an LTS release enter and close its
  family, and a series leave Extended Support together with its family; the
  rationale added that Innovation releases make the set move faster than
  MariaDB's. An Innovation release now moves nothing, and no family enters the
  criterion. The count on the verification date moves from three series to
  two.

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

  *The next scheduled change.* The set of the table is known to change when
  Oracle publishes the first release of a new LTS series, which no source read
  on 2026-11-04 dates, and, by the count `FR-MYSQL-003` derives and does not
  read, when `8.4` leaves Extended Support in 2032. Its column of latest
  releases goes stale at every maintenance release of either series, which
  changes no member of the set.

  *Rationale.* It is `FR-SRV-019` for MySQL: a release of `tpl` built on a
  stale table reads a new LTS series as newer than the supported set, under
  `FR-MYSQL-010`, and reads a series that has left Extended Support as
  supported.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended. The note named the next Innovation release as the
  event that would make the table wrong, and the rationale said that this
  requirement binds more often than `FR-SRV-019` and that a stale table
  refuses a series that has entered the set. An Innovation release no longer
  changes the set, and a series that enters it is newer than every series a
  stale table holds, so such a table marks it rather than refusing it.

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

  *Observed.* The probe answered `8.4.13` on `8.4`, `9.7.4` on `9.7` and
  `8.0.46` on `8.0`, verbatim: three integers and no suffix, which is the
  string the example above shows for `8.4`. `@@version` returned the same
  string as the probe on all three servers, and `@@version_comment` returned
  `MySQL Community Server - GPL`. The manual's description of the value — "The
  version number for the server. The value might also include a suffix
  indicating server build or configuration information. `-debug` indicates
  that the server was built with debugging support enabled"
  (`refman/8.4/en/server-system-variables.html`) — admits the suffix, and no
  server of the fixture carried one.

  *Consequence, stated plainly: `version` carries three parts.* Both manuals
  document a second variable, `version_with_update`, "The complete server
  version in MAJOR.MINOR.PATCH.UPDATE form. The update component is always
  present, including when its value is 0"; it answered `8.4.13.0` on `8.4`
  and `9.7.4.0` on `9.7`, and reading it on `8.0` failed with `ERROR 1193`.
  The 9.7 manual calls `version` "The three-part compatibility version for
  the server" and adds that "The update component is not included; use
  version_with_update to obtain the exact four-part release identity"
  (`refman/9.7/en/server-system-variables.html`). The closed list of
  `FR-MYSQL-015` reads the probe and no other variable, so an update
  component never reaches `server.version`. `series` is derived from the
  first two parts and is unaffected.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`: the note no longer
  cites the release notes of `26.10.0` on the version identity that release
  introduced. They applied to `26.10` alone, which is no longer a supported
  series, per `FR-MYSQL-002`, and is owed no observation by rmp `#326`. The
  requirement is unchanged.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note, which quoted the manual and said that the example was the
  form the manual describes rather than a string any server returned, is
  replaced by the *Observed* note above, and the consequence note on
  `version_with_update` is new. The requirement is unchanged.

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

  *Observed, on MySQL.* No answer of the probe on `8.4`, `9.7` or `8.0`
  contains the marker, and each begins with three integers separated by dots,
  per `FR-MYSQL-006`. The condition therefore passes every MySQL server of the
  fixture. The reference inspector separates the two products on the same
  marker: its `Maria()` reports whether the version string contains
  `MariaDB`, in `sql/mysql/internal/mysqlversion/mysqlversion.go` at tag
  `v0.21.1`.

  *A stated limit, adopted.* **A server determined to pass as MySQL will
  pass.** It is the limit `FR-SRV-041` states for MariaDB, applied to MySQL in
  its terms and for its reason: every reading the condition can rest on is a
  response the server composes. A server that is not MySQL and reports a
  version string of MySQL's form without the marker is read as MySQL, under the
  standing its series earns, whoever built it. This requirement applies the
  first member of the family the [README](README.md#writing-conventions) names,
  and is not a fourth member of it.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note on MySQL is replaced by the *Observed* note above. The
  requirement is unchanged.

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
  reached it. Its `cause` SHALL state that the connection and the
  authentication succeeded; SHALL name the version string the probe returned;
  SHALL state why the series found is not supported, as the criterion of
  `FR-MYSQL-002` it fails — that it is not an LTS series under Oracle Premier
  or Extended Support, and that it is older than the newest series that is —
  a reason that is the same for every series this requirement refuses; and
  SHALL list the series of `FR-MYSQL-003`. Its `hint` SHALL carry the command
  of `FR-MYSQL-008`.

  ```
  error: server series '8.0' is not supported, for database entry 'shop'
  cause: connected and authenticated; the server reports 8.0.46, and 8.0 is not a MySQL LTS series under Oracle Premier or Extended Support, and is older than the newest that is. A mysql entry supports 9.7 and 8.4
  hint:  repoint the entry at a supported server: tpl cfg database update shop --host <host>
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code.

  *Why the reason is the criterion, and not the series' own standing.* The
  binary knows the series of `FR-MYSQL-003` and the comparison of
  `FR-MYSQL-002`, and nothing else about a series it refuses. The criterion
  is therefore the one reason it can state that is true of every series the
  rule reaches, and it stays true for as long as the table the binary carries
  is current, per `FR-MYSQL-005`. Where a reader needs a series' support tier
  and its dates, `FR-MYSQL-003` names the sources.

  *Rejected: a reason particular to the series found*, its support tier and
  the date it entered it, as the example read before the seventy-sixth
  edition: "8.0 is in Oracle Sustaining Support since 2026-04-21". The binary
  could print it only from a second table, of every series it may refuse,
  with a tier and a date for each, which `FR-MYSQL-003` does not hold and
  which would go stale without a sign at every change Oracle makes to a
  series' support. `BR-MYSQL-001` keeps the facts that change so to one
  table, verified on one date.

  *Rationale.* It is `FR-SRV-020` and `FR-SRV-030` for MySQL, on their ground:
  a series below the window is one the project will never observe, because
  observation follows support. Under the comparison of `FR-MYSQL-002`, one
  rule reaches a series that left support (`8.0`), an Innovation release
  older than the newest supported series (`9.0` to `9.6`), and an LTS series
  that has left Extended Support. An Innovation release newer than the newest
  supported series is not reached by this requirement and is read under
  `FR-MYSQL-010`; on 2026-11-04 that is every Innovation release numbered by
  the calendar.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended. The example's `cause` listed `26.10` among the
  supported series, and the rationale named `26.7`, a superseded Innovation
  release, as refused here and a series of a family that has left the set as
  the third case. `26.7` is now newer than the newest supported series, and
  no family enters the criterion. The requirement is unchanged.

  *Amended in the seventy-sixth edition,* for rmp `#329`. The requirement
  asked the `cause` to state why the series found is not supported and left
  the reason open, and the example, which fixes the facts named, gave a reason
  particular to `8.0`, its support tier and a date. The implementation of the
  provider found that the binary holds no such fact for any series it
  refuses. The requirement now names the version string and fixes the reason
  as the criterion of `FR-MYSQL-002` that the series fails, the same for
  every series refused, and the example states it. The condition, the code,
  the series listed and the `hint` are unchanged.

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

  *Consequence, stated plainly: an Innovation release is read and marked.*
  Every Innovation release numbered by the calendar is newer than the newest
  series of `FR-MYSQL-003`. A server of `26.7` or `26.10`, or of any
  Innovation release Oracle publishes before it first releases a newer LTS
  series, is therefore read under this requirement, marked
  `newer_than_supported`, at exit `0`. None of them is a supported series, per
  `FR-MYSQL-002`, so `standing` is never `supported` for one, and none will be
  observed against the fixture, because observation follows support. The
  system cannot tell such a server from one of a new LTS series: "A calendar
  version does not determine whether a release is an Innovation or LTS
  release. Release maturity remains an explicit property of each release",
  per `refman/26.10/en/mysql-releases.html`, checked 2026-11-04, and
  `FR-MYSQL-006` derives the series from the version string and from nothing
  else. Refusing every series newer than the table would refuse a new LTS
  series until a binary built on a later table is released, which is the
  outcome `FR-SRV-031` rejects. Once a newer LTS series enters the table, the
  Innovation releases older than it are refused under `FR-MYSQL-009`, as
  `9.0` to `9.6` are today.

  *Consequence, stated plainly: a binary carries its table.* A server whose
  series the binary's table admits is read as `supported` by that binary after
  the series has left Extended Support; a later binary, built on a later
  table, refuses it under `FR-MYSQL-009`. A new LTS series is read as
  `newer_than_supported` by every binary built before its table admitted it.
  The table a binary carries is the one it was released with, per
  `FR-MYSQL-005`.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended. The one consequence note said that a binary meets
  a series newer than its table within months of its release, because Oracle
  published Innovation releases on 2026-07-28 and 2026-10-20, and that a
  series the table admits is read as `supported` after Oracle has superseded
  it. Neither describes the set as amended: an Innovation release numbered by
  the calendar is newer than the table on the day it is published, and a
  series of the table leaves it only when its Extended Support ends. The first
  note now states what a server of an Innovation release meets, and the second
  the stale-table case for an LTS series. The requirement is unchanged: a
  MySQL series newer than the newest supported series is read and marked,
  whatever its track.

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

  *Observed, on MySQL.* A read of `@@session.tx_read_only` fails with
  `ERROR 1193`, `Unknown system variable 'tx_read_only'`, on `8.4`, on `9.7`,
  on both servers offering no TLS and on `8.0`, so a `mariadb` entry that
  reaches any of them is refused at its read-back, before its product check.
  The variable lists of the 8.4 and 9.7 manuals carry `transaction_read_only`
  and no `tx_read_only`, and the 8.0 manual records the removal: "The
  `tx_isolation` and `tx_read_only` system variables have been removed. Use
  `transaction_isolation` and `transaction_read_only` instead"
  (`refman/8.0/en/mysql-nutshell.html`). Because the read-back refuses first,
  the `cause` names the read-only session and not the product, which is the
  cost `FR-SRV-042` states and accepts.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above. The table and
  the requirement are unchanged.

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

  **The record holds ten differences, and the register holds no row.** rmp
  `#326` compared `8.4.13` and `9.7.4` on 2026-11-04: the twelve tables of
  `FR-MYSQL-015`, their field lists and the declared type of every field, and
  every row they return for the fixture's three schemas, as the privileged and
  as the reduced-grant reader; and, in a scratch schema, each of the three
  candidates the seventy-third edition named from the documentation. Their
  rows differed in wall-clock times alone — the creation and update times of
  the table catalogue, the creation and alteration times of the routine
  catalogue and the creation time of the trigger catalogue, all excluded by
  `FR-CAT-024` — and in the one value of difference 5. Every field the model
  carries returned the same value on both series.

  | # | Observed | `8.4` | `9.7` | What it obliges |
  |---|---|---|---|---|
  | 1 | A `VECTOR` column at DDL time | refused, `ERROR 1064 (42000)`, a syntax error at `VECTOR(3)` | accepted with no warning; `DATA_TYPE` `vector`, `COLUMN_TYPE` `vector(3)`, `CHARACTER_MAXIMUM_LENGTH` and `CHARACTER_OCTET_LENGTH` `12`, every other part SQL `NULL` | The shared DDL of `FR-MYSQL-044` declares no `VECTOR` column. A `VECTOR` column of a `9.7` database is a column of a covered table, carried under `FR-MYSQL-028`. No register row: no database both series hold carries one |
  | 2 | JSON duality views, and six `INFORMATION_SCHEMA` tables: `JSON_DUALITY_VIEWS`, `JSON_DUALITY_VIEW_COLUMNS`, `JSON_DUALITY_VIEW_LINKS`, `JSON_DUALITY_VIEW_TABLES`, `LIBRARIES` and `ROUTINE_LIBRARIES` | `CREATE JSON RELATIONAL DUALITY VIEW` refused, `ERROR 1064`; none of the six tables | accepted; the view is a row of the table catalogue with `TABLE_TYPE` `VIEW`, `ENGINE` `NULL` and `TABLE_COMMENT` `VIEW`, a row of the view catalogue with `CHECK_OPTION` `NONE`, `IS_UPDATABLE` `YES` and its `select` as `VIEW_DEFINITION`, and one row of the column catalogue; the six tables exist | A duality view is a covered view, per `FR-MYSQL-023`. None of the six tables is among the twelve of `FR-MYSQL-015`, so none is read. No register row |
  | 3 | The declared type of `ROUTINE_TYPE`, in the routine and the parameter catalogues | `enum('FUNCTION','PROCEDURE')` | `enum('FUNCTION','PROCEDURE','LIBRARY')`; the 9.7 manual still describes the field as `PROCEDURE` or `FUNCTION` | `FR-MYSQL-023` does not cover a row of type `LIBRARY`, because a library is not a routine in the sense of `FR-CAT-008`. No row of the fixture carries it, and no register row |
  | 4 | The declared type of `SQL_MODE`, in the routine and the trigger catalogues | a set of 33 members | the same set with a 34th member, `INTERPRET_UTF8_AS_UTF8MB4` | `sql_mode` is carried as the server returns it, per `FR-MYSQL-036` and `FR-MYSQL-038`, so the member reaches a document WHERE a routine or a trigger of a `9.7` database was created under it. No row of the fixture carries it, and no register row |
  | 5 | The text of a functional key part, `STATISTICS.EXPRESSION` of `uq_customer_vat_normalised`, on an `ascii` column | ``upper(replace(`vat_number`,_ascii\' \',_utf8mb4\'\'))`` | ``upper(replace(`vat_number`,_ascii\' \',_ascii\'\'))`` | Nothing: an index with a functional key part is excluded, per `FR-MYSQL-024`, and the field is not carried, per `FR-MYSQL-032`. The same expression in a generated column reads `_utf8mb4` for both literals on both series. It is the one row value, of the twelve tables, that differs and is not a wall-clock time |
  | 6 | GTID consistency, by default | `gtid_mode` and `enforce_gtid_consistency` `OFF` | both `ON`; an insert into an InnoDB table whose `AFTER INSERT` trigger writes a MyISAM table is refused with `ERROR 1785`, "Statement violates GTID consistency", and `Warning 6414`, "Combining the storage engines MyISAM and InnoDB is deprecated" | What the fixture may hold: its MyISAM table is written on its own, per `FR-MYSQL-044`. It reaches no read, because `tpl` writes nothing |
  | 7 | `FLUSH PRIVILEGES` | no warning | `Warning 1681`, deprecated | Nothing for `tpl`, which never issues it. It empties the authentication cache on both, which is how a test of `FR-MYSQL-045` sets that cache |
  | 8 | `CREATE MASKING POLICY` | refused, `ERROR 1064` | parsed, and refused with `ERROR 6561 (HY000)`, "MASKING POLICY component error: component is unavailable" | Nothing the fixture can show of the read: the phrase of `FR-MYSQL-030` stays pending observation there |
  | 9 | The version, as each manual describes it | "The version number for the server" | "The three-part compatibility version for the server" | Nothing: both servers answer the probe in three parts, and both carry `version_with_update`, which the closed list does not read, per `FR-MYSQL-006`. The difference is the manual's, not the servers' |
  | 10 | The size of the server-wide catalogue | 176 base tables, 105 views, 78 system views | 179 base tables, 105 views, 84 system views | Nothing: a read covers one database, per `FR-MYSQL-023` |

  *Why the register holds no row.* `FR-SRV-036` fixes a row as a field of the
  model, its treatment, and what each series returned, and no difference above
  supplies all three. Differences 1 to 4 are in what each series can hold or
  declares: no database both series hold yields a different value, because
  the fixture holds none of the four, and each is carried, where a `9.7`
  database holds one, under a requirement that already reaches it. Difference
  5 is in a field the model does not carry, so it is the case difference 12 of
  `FR-SRV-038` records for MariaDB, and the equivalence `FR-SRV-026` requires
  of identical DDL across the series is not reached. Differences 6 to 10
  reach the fixture, the harness or the manual, and not the read.

  *How the three candidates of the seventy-third edition were settled.* The
  attribute phrase `MASKING POLICY` is difference 8, and is pending
  observation under `FR-MYSQL-030`; the `VECTOR` type is difference 1; and the
  duality views are difference 2, which places a duality view among the
  covered views. The "not checked" cell of the `VECTOR` candidate is filled:
  the 8.4 manual's page for the type, `refman/8.4/en/vector.html`, redirects
  to the 9.7 page.

  *Bounded claim.* One build of each series was compared, under the
  conditions recorded in the section on sources. The comparison covers the
  objects of the fixture and the scratch objects named above; a difference in
  what a series reports for an object neither holds was not observable.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`: the column of
  `26.10` is removed with the series, per `FR-MYSQL-002`. The candidate of the
  duality views rested on that column alone, and is now placed by the index of
  each remaining manual, read on 2026-11-04: the 9.7 manual lists the four
  tables and the 8.4 manual lists none of them. The note names the series of
  `FR-MYSQL-003` where it counted three.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The paragraph read
  that the record held no observed difference, because no MySQL server had
  been observed, and the table held three candidates the documentation named,
  under a *Pending observation* note. The record now holds the ten differences
  rmp `#326` observed, and the three candidates are settled among them. The
  register still holds no row, now because no observed difference reaches a
  field of the model for a database both series hold. The requirement is
  unchanged.

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

  *Observed.* Each of the twelve tables was read with `SELECT *`, restricted
  to the fixture's schemas, as the privileged and as the reduced-grant
  reader, on `8.4` and on `9.7`. Every table and every field this file names
  exists by that name on both series, and the two field lists of each table
  are identical in name and in order: `SCHEMATA` 6 fields, `TABLES` 21,
  `COLUMNS` 22, `STATISTICS` 18, `KEY_COLUMN_USAGE` 12,
  `REFERENTIAL_CONSTRAINTS` 11, `TABLE_CONSTRAINTS` 7, `CHECK_CONSTRAINTS` 4,
  `VIEWS` 10, `ROUTINES` 31, `PARAMETERS` 16 and `TRIGGERS` 22. Every
  character field is declared `utf8mb3`. The declared types differ between
  the series in two fields alone, differences 3 and 4 of `FR-MYSQL-014`.
  `FR-SRV-037` therefore has nothing to govern for a `mysql` read today. The
  version probe, the read-only session statement and the read-back were each
  accepted on all five servers of the fixture, with no warning.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note, which rested the existence of every table and field on
  the manual pages cited in the mapping requirements, is replaced by the
  *Observed* note above. The requirement is unchanged.

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

  *Observed.* The manual states that "Columns in TABLES that represent table
  statistics hold cached values" under the `information_schema_stats_expiry`
  variable (`refman/8.4/en/information-schema-tables-table.html`). On a
  scratch table holding 20 rows inserted after it was created, with no
  `ANALYZE TABLE`, the cached read returned `TABLE_ROWS` `0`,
  `AUTO_INCREMENT` and `UPDATE_TIME` SQL `NULL` and `CARDINALITY` `0`, and a
  read with the variable at `0` returned `20`, `21`, a timestamp, and `15` on
  `8.4` and `16` on `9.7`. No field the model carries moved: the table's name,
  type, engine, collation and comment, and every field of the index catalogue
  but `CARDINALITY`, read the same under both settings. Every field that moved
  is one `FR-CAT-024` excludes as volatile. The server-variable page of the
  same manual adds that the cache is not updated "When the server is in …
  transaction_read_only … mode" (`refman/8.4/en/server-system-variables.html`),
  which is the session `FR-MYSQL-018` opens.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it. The requirement is unchanged.

- **FR-MYSQL-048**: For a `mysql` entry, `FR-SRV-043` SHALL hold, with the
  closed list of `FR-MYSQL-015` in place of `FR-SRV-006`: a name the system
  did not write itself — the database the entry names, or the name of a
  table, a view or a routine a command names — changes no part of a
  statement but the value its rows are compared with, whatever characters it
  holds.

  *Rationale.* rmp `#352` asks how a name a caller supplies reaches a MySQL
  catalogue statement, which the technical specification records as a gap.
  The means is the technical specification's to record; the property is the
  one `FR-SRV-043` states for a `mariadb` entry, and a `mysql` entry's closed
  list, which `BR-MYSQL-002` makes the part of its read-only promise that
  prevents, rests on it as that list does. The fixture's identifiers holding
  a space, a backtick and a double quote, item 14 of `FR-MYSQL-044`, are
  names a test of it reads.

  *Added in the seventy-ninth edition,* for rmp `#352`.

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
  took effect. It SHALL read that variable and no other spelling of it. The
  setting SHALL be confirmed WHERE the read-back returns the integer `1`, and
  SHALL NOT be confirmed by any other value.

  *Observed.* `SET SESSION TRANSACTION READ ONLY` was accepted with no
  warning on all five servers of the fixture, and so was
  `SET SESSION transaction_read_only = ON`. The read-back returned the
  integer `0` before any setting, `1` after either statement, and `0` again
  after `= OFF`, on all five; its wire type is an integer, `LONGLONG`, and not
  text. A write issued in the read-only session was refused with
  `ERROR 1792 (25006)`, "Cannot execute statement in a READ ONLY
  transaction.", and created nothing. These are the values `FR-SRV-006`
  records of MariaDB's read-back, `0` and then `1`.

  *Why the requirement names the value.* The manual states that "The value
  can be OFF (read/write; the default) or ON (read only)"
  (`refman/8.4/en/server-system-variables.html`), which are the values an
  assignment accepts and not what a read returns. A read-back compared with
  the text `ON` would confirm nothing on any server of the fixture, and every
  `mysql` read would end with `78` under `FR-MYSQL-019`.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The last sentence
  of the requirement is new: the requirement said the read-back confirms the
  setting and named no value. The *Pending observation* note, which reported
  the manual's `OFF` and `ON`, is replaced by the *Observed* note above.

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
  observed there, save the one statement `FR-MYSQL-021` keeps pending.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the rationale said
  that the server side was pending observation in those two requirements. The
  requirement is unchanged.

  *Amended in the seventy-seventh edition,* for a follow-up of rmp `#329`:
  the rationale counted three statements pending in `FR-MYSQL-021`. The
  provider's tests observed one of them, and the observation of
  `FR-CONF-039` discharges another. The requirement is unchanged.

- **FR-MYSQL-047**: `FR-CONF-055` SHALL hold for a `mysql` entry: WHERE the
  password the configuration supplies for it is the empty string, or the
  configuration supplies none, the system SHALL present the entry to the
  server as the server's own clients present an empty password, and SHALL
  NOT send a scramble computed from the empty string. `FR-CONF-055` names the
  three sources of the empty string and the prior art that fixes what is
  sent.

  *Provenance.* The implementation of the provider, for rmp `#329`, found the
  corpus silent on an empty password; rmp `#334` found that the presentation
  the seventy-sixth edition then fixed is not the one the server's own
  clients use. The login it fixes is observed on `8.4` and `9.7` under
  `FR-CONF-055`.

  *Added in the seventy-sixth edition,* for rmp `#329`.

  *Amended in the seventy-seventh edition,* for rmp `#334`. The requirement
  read: the system "SHALL present the empty string to the server as the
  entry's password, through the exchange of the authentication method the
  server asks for, exactly as it presents a password of any other value, and
  SHALL NOT present the entry as one that supplies no password". That
  exchange computes a scramble from the empty string, which no client of the
  servers' own sends, and a MySQL server met it, for an account with no
  password over a session TLS does not protect, by asking for the full
  exchange, which ended at `78` under `FR-MYSQL-045`, as `FR-CONF-055`
  records. The rule is now `FR-CONF-055`, which states it
  for a `mariadb` and a `mysql` entry alike, and this requirement holds it for
  a `mysql` entry; the table of the three sources, the rationale and what the
  rule does not settle move there, beside the requirements that read each
  source, and both readings this requirement held are recorded there as
  rejected. The note that no requirement stated the rule for a `mariadb`
  entry is removed, because `FR-CONF-055` states it.

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
  `FR-MYSQL-022` and `FR-MYSQL-045`. *Amended in the seventy-fourth edition,*
  for rmp `#331`: the sentence named `OQ-077` as its one unsettled case, which
  `FR-MYSQL-045` closes.

  WHERE the TLS handshake of a `mysql` entry is refused before any
  certificate is judged — the server offers no TLS, or the negotiation
  failed — the `hint` of the `69` SHALL state that the server must offer TLS
  for the entry's mode to connect, and SHALL NOT carry a command that sets
  `tls` to `preferred` or to `disabled`.

  ```
  error: the TLS handshake with db.example.com:3306 failed, for database entry 'shop'
  cause: the TLS handshake with db.example.com:3306 was refused before any certificate was checked: the server offers no TLS, or the negotiation failed
  hint:  check that the server offers TLS, and enable TLS on the server where it does not
  exit:  69 (EX_UNAVAILABLE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, that the `hint`
  carries no command, and the code.

  *Why no mode that does not encrypt.* Under such a mode the login of an
  account on the default method of every supported series needs the
  server's RSA public key whenever the server holds no cached login for it,
  per the observations of `FR-MYSQL-045`, and that requirement then ends the
  read with `78`, whose `hint` sends the caller back to a mode that encrypts.
  A `hint` naming `preferred` would send the caller round the two. Whether
  the server holds a cached login is not a state the invocation found, and
  `BR-ERR-004` asks for a command that can succeed in the state it found.
  *Why no command.* `FR-ERR-009` asks for a runnable command wherever one
  exists. What makes the read succeed here is TLS on the server, which no
  command of `tpl` gives it.

  *Rejected: keeping `preferred` in the `hint`, with a statement of when it
  succeeds.* The condition it would state, a cached login on the server, is
  one the caller cannot see before running the command, and a weak reader
  acts on the command and not on the sentence around it, which is the ground
  `BR-ERR-005` gives.

  *A `mariadb` entry.* This paragraph does not reach one. `FR-ERR-050` fixes
  the `hint` of this condition for a `mariadb` entry, and keeps `preferred`
  in it, because a `mariadb` entry meets no `78` of `FR-MYSQL-045`.

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
     *Observed*: the two servers of the fixture started so, one of each
     series, report their TLS channel as not enabled, and every client mode
     that requires TLS fails against them.

  *Observed, through the stock client.* The reduced-grant reader logged in
  under each mode, with the server's authentication cache filled, and the
  encryption of each session was read from the session's own status:

  | Client mode | `8.4` and `9.7`, offering TLS | `8.4` and `9.7`, offering none |
  |---|---|---|
  | `DISABLED` | plaintext | plaintext |
  | `PREFERRED` | encrypted, `TLS_AES_128_GCM_SHA256` over `TLSv1.3` | plaintext |
  | `REQUIRED` | encrypted | refused: `ERROR 2026`, "SSL is required but the server doesn't support it" |
  | `VERIFY_CA`, with the fixture's root | encrypted | refused, `ERROR 2026` |
  | `VERIFY_IDENTITY`, with the fixture's root | encrypted | refused, `ERROR 2026` |

  Each cell is the server side of the cell of the same mode in the table
  above. The three controls `FR-CONF-038` records held: `VERIFY_CA` without
  trust material failed before any handshake, "CA certificate is required if
  ssl-mode is VERIFY_CA or VERIFY_IDENTITY"; `PREFERRED` without trust
  material connected encrypted; and `REQUIRED` with trust material that does
  not match connected encrypted, warning that no verification of the server
  certificate would be done. Two further controls held: `VERIFY_CA` with
  trust material that does not match failed with "certificate verify
  failed", and `VERIFY_IDENTITY` to a host name the certificate does not
  carry failed the same way while `VERIFY_CA` to that name connected.

  *Observed, through `tpl`, on 2026-11-04, at `2d652bd`.* The test
  `fr_mysql_021_the_five_modes_against_a_server_offering_tls_and_one_offering_none`,
  in `tests/mysql_provider.rs`, ran `tpl schema info --direct --no-cache` as
  `root` under each mode against the four servers of the fixture that are
  supported — one offering TLS and one offering none, at `8.4.13` and at
  `9.7.4` — and read each session's encryption from the server's own record
  of the connection. Against the servers offering TLS, `verify-identity`,
  `verify-ca`, `required` and `preferred` exited `0` encrypted, and then
  `disabled`, with the server's authentication cache filled by the logins
  before it, exited `0` in plaintext. Against the servers offering none,
  `disabled` and `preferred` exited `0` in plaintext, with the cache filled by
  the fixture's harness first, as `FR-MYSQL-045` requires of a plaintext
  login; and `required`, `verify-ca` and `verify-identity` exited `69`, with a
  `cause` naming the TLS handshake. The message, which names no mode, as rmp
  `#329` recorded it against the server of `8.4` offering none:

  ```
  error: the TLS handshake with 127.0.0.1:13322 failed, for database entry 'shop'
  cause: the TLS handshake with 127.0.0.1:13322 was refused before any certificate was checked: the server offers no TLS, or the negotiation failed
  hint:  check that the server offers TLS; for a server without it, on a network you trust, allow a plain connection with: tpl cfg set database.shop.tls preferred
  exit:  69 (EX_UNAVAILABLE)
  ```

  The `hint` is the one the implementation printed at `2d652bd`, before the
  paragraph above on the `hint` was written, and is not what that paragraph
  requires.

  *Pending observation.* A server's own TLS material. The manual states that
  a MySQL server built with OpenSSL "can generate missing certificate and key
  files automatically at startup"
  (`refman/8.4/en/using-encrypted-connections.html`), the first item above,
  and says nothing of whether that certificate names a host. No server of the
  fixture generated a certificate: each server offering TLS is given the
  fixture's own, and the data directory of every server held the RSA key pair
  of `FR-MYSQL-045`, `private_key.pem` and `public_key.pem`, and no
  certificate. A server started with TLS enabled and no material configured
  would observe both, and the fixture of `FR-MYSQL-044` stages none.

  *Observed, under `FR-CONF-039`.* Trust material beside the public roots.
  Whether trust material supplied under `FR-CONF-014` stays additional to the
  public root bundle is a property of the driver and not of the server: the
  driver builds the set of authorities it trusts from its bundled public
  roots and the material the entry names, and from nothing a server sends,
  as the source of the driver the repository carries shows. `FR-CONF-039`
  observed that the driver that reads MariaDB adds the supplied material to
  the public roots rather than substituting it. The project's decision
  records name that driver, at the same pin, for a `mysql` entry, and
  `FR-MYSQL-020` connects a `mysql` entry with the trust material and the
  rules of a `mariadb` entry. That observation is therefore of the driver a
  `mysql` entry connects through, and the consequence `FR-CONF-039` states —
  pinning an authority widens the certificates that pass `verify-ca` rather
  than narrowing them — holds for a `mysql` entry. *Bounded claim.* No MySQL
  server presenting a certificate issued by a publicly trusted authority was
  observed, and the property does not need one, because no server takes part
  in it.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. One *Pending
  observation* note covered every cell, the three controls, a generated
  certificate and the public roots. The server side of every cell and every
  control is now observed, and the second item above records what was
  observed of the servers that offer no TLS. Three notes remain, one for each
  statement this fixture cannot observe. The requirement and its table are
  unchanged.

  *Amended in the seventy-seventh edition,* for three follow-ups of rmp
  `#329`. The paragraph on the `hint` of a refused TLS handshake is new: the
  implementation's `hint`, recorded above, named `preferred` for a server
  offering no TLS, which for a `mysql` entry ends in the `78` of
  `FR-MYSQL-045` whenever the server holds no cached login. The *Pending
  observation* note on the code of three cells is replaced by the *Observed*
  note above, which the provider's tests made. The note on trust material is
  discharged by the observation of `FR-CONF-039`: it waited for the driver
  that reads MySQL to be chosen, and the one chosen is the driver that
  requirement observed. One note remains pending, on a server's own TLS
  material. The table is unchanged.

- **FR-MYSQL-022**: The system SHALL authenticate a `mysql` entry through the
  default authentication method of every series of `FR-MYSQL-003`, which is
  `caching_sha2_password`, under every TLS mode of `FR-CONF-013` that yields
  an encrypted session. A database driver that cannot SHALL be disqualified
  from the choice, as `FR-CONF-036` disqualifies one that cannot express the
  five modes. WHERE the session is not encrypted — `disabled`, and
  `preferred` against a server offering no TLS — `FR-MYSQL-045` governs an
  authentication that needs the server's RSA public key.

  *Source.* "The default plugin is determined by the value of the
  `authentication_policy` system variable; the default is to use
  `caching_sha2_password`"; on a secure connection "an RSA key pair is
  unnecessary and is not used"; on a connection that is not secure "an RSA key
  pair is used", and without the server's public key the login fails with
  "Authentication requires secure connection"
  (`refman/9.7/en/caching-sha2-pluggable-authentication.html`).

  *Observed.* `authentication_policy` read `*,,` on `8.4` and on `9.7`, and
  every account of the fixture — `root` and the reduced-grant reader, on both
  servers of each series — reports `caching_sha2_password`, which the 8.4
  manual names as the default in the same words as the 9.7 manual
  (`refman/8.4/en/caching-sha2-pluggable-authentication.html`). With the
  server's cache emptied, the reduced-grant reader authenticated under every
  client mode that yields an encrypted session; the logins that do not are
  `FR-MYSQL-045`'s.

  *Rationale.* `FR-CONF-036` makes the modes normative over the driver; a
  driver that connects in every mode and cannot authenticate the account a
  supported server creates by default connects nowhere that matters.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`. The last sentence
  of the requirement read that the authentication of an account the server
  has not cached, over a session that is not encrypted, was `OQ-077` and that
  no requirement of this file settled it; the user's decision is
  `FR-MYSQL-045`. The *Pending observation* note named `26.10` beside `8.4`,
  and `26.10` is no longer a supported series, per `FR-MYSQL-002`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note, which owed the default method on `8.4` and a login under
  each mode, is replaced by the *Observed* note above. The requirement is
  unchanged.

- **FR-MYSQL-045**: The system SHALL NOT request a MySQL server's RSA public
  key over a session that is not encrypted — under `disabled`, and under
  `preferred` against a server offering no TLS, per `FR-MYSQL-021` — whatever
  the authentication method. IF the authentication of a `mysql` entry over
  such a session needs that key, THEN the system SHALL end the connection
  without completing the authentication, SHALL exit `78` (`EX_CONFIG`), and
  SHALL NOT read the catalogue. No flag, configuration key or environment
  condition SHALL change this.

  The `cause` SHALL state that the connection succeeded and that the
  authentication was not completed; SHALL name the user, the host and the
  port, and the `tls` mode in force, and, WHERE that mode is `preferred`, that
  the server offered no TLS; SHALL name the authentication method the server
  asked for, escaped under `FR-ERR-024`; and SHALL state that `tpl` does not
  request the server's public key over a session TLS does not protect. The
  `hint` SHALL carry `tpl cfg database update <entry> --tls verify-identity`,
  with the entry name filled in, and, WHERE the server offered no TLS, SHALL
  also state that the server must offer TLS before that mode can connect.

  ```
  error: cannot authenticate user 'reader' over an unencrypted session, for database entry 'shop'
  cause: connected to db.example.com:3306 as user 'reader' under tls 'disabled', so the session is not encrypted; the authentication was not completed: the server asks for the full caching_sha2_password exchange, which needs its RSA public key, and tpl does not request that key over a session TLS does not protect
  hint:  encrypt the session: tpl cfg database update shop --tls verify-identity
  exit:  78 (EX_CONFIG)
  ```

  ```
  error: cannot authenticate user 'reader' over an unencrypted session, for database entry 'shop'
  cause: connected to db.example.com:3306 as user 'reader' under tls 'preferred' and the server offered no TLS, so the session is not encrypted; the authentication was not completed: the server asks for the full caching_sha2_password exchange, which needs its RSA public key, and tpl does not request that key over a session TLS does not protect
  hint:  encrypt the session: tpl cfg database update shop --tls verify-identity; the server offered no TLS, and must offer it before that mode can connect
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The examples fix the facts named, the command
  carried and the code. They are the messages the implementation prints at
  `2d652bd`, the first as the *Observed* note below records it with the
  fixture's values.

  The condition is one of the cache-or-connection step of `FR-ERR-006`, and
  it is raised at the authentication, before the read-only session of
  `FR-MYSQL-018` is set. It holds for every command that opens a connection
  for a `mysql` entry, `tpl cfg database test` among them, which ends at the
  first step of `FR-CFG-024` with this message.

  *Provenance.* The user's decision on `OQ-077`, relayed for rmp `#331` on
  2026-11-04: never request the server's RSA public key over a session that
  TLS does not protect; when the authentication would need it, the read fails
  with a clear error, whose code and message are stated here consistently
  with the error requirements in force. It is the first option `OQ-077` named
  and recommended.

  *Why `78`.* The next step is the configuration's: a `tls` mode that
  encrypts the session, which is the remedy the `78` row of `FR-ERR-001`
  gives — fix `.tpl/.cfg`. *Rejected: `77`.* The server refused nothing:
  `tpl` ended the authentication, and the credentials may be right. The `77`
  row of `FR-ERR-034` names a refusal that came from the server, and
  `FR-CONF-033` rejects `77` on the same ground for an authentication that was
  never attempted. *Rejected: `69`.* Nothing was
  unavailable: the host answered and the session opened, and the `69` row of
  `FR-ERR-001` tells the caller that the operation is repeatable, where a
  repetition under the same configuration ends the same way for as long as
  the server holds no cached login for the account.

  *Why the `hint` names `verify-identity`.* `BR-ERR-004` asks for the value
  where it is known, and admits a placeholder only for a value the caller
  alone knows. The decision exists to keep the password out of an
  intermediary's reach, and that is the reason `FR-CONF-013` gives for
  defaulting to `verify-identity`, so that mode is the one the `hint` carries.
  A weaker mode that encrypts is the caller's to choose, and `FR-CONF-013`
  states what it costs. The command names the value it writes, so
  `BR-ERR-005` admits it, and `tls` is not a field `FR-ERR-045` reroutes for an
  entry defined by `dsn`.

  *Rejected, by the user: an opt-in configuration key that allows the
  request.* Beside the user's decision, the corpus has two grounds against
  it: a protection that a key of the entry can turn off is advisory, the
  ground `FR-SRV-011` gives for the read-only promise; and the key would be
  one a `mariadb` entry does not take, which `FR-MYSQL-001` excludes.
  *Rejected: requesting the key on every login*, the second option `OQ-077`
  named; it is what the decision forbids. *Not adopted: a configuration key
  naming a file that holds the server's public key*, the third option
  `OQ-077` named. The decision fails the read whenever the authentication
  needs the key, and `FR-MYSQL-001` gives a `mysql` entry no key a `mariadb`
  entry lacks.

  *Accepted cost, stated plainly.* Whether a login needs the key depends on
  the server's cache and not on the entry. The same entry, under `disabled`,
  authenticates while the server holds a cached login for the account, and
  exits `78` once the server has dropped it — at a restart, at
  `FLUSH PRIVILEGES`, or when the account changes — with nothing in
  `.tpl/.cfg` changed.

  *Observed, through the stock client.* With the server's cache emptied, a
  login of the reduced-grant reader under `DISABLED`, against every server of
  both series, and under `PREFERRED` against both servers offering no TLS,
  failed with `ERROR 2061 (HY000)`, "Authentication plugin
  'caching_sha2_password' reported error: Authentication requires secure
  connection.", and the client exited `1`. With the cache filled, the same
  logins succeeded, in plaintext. The server sent its RSA public key only when
  the client asked for it, which is how the fixture's harness fills the cache
  over a session TLS does not protect. Four events emptied the cache: a
  server start, `FLUSH PRIVILEGES`, `ALTER USER … IDENTIFIED BY` with the same
  password, and `RENAME USER`. These are the events the manual names — "the
  server does not send the RSA public key to clients by default", and the
  cache is cleared for "any account that is renamed or dropped, or any account
  for which the credentials or authentication plugin are changed", by
  `FLUSH PRIVILEGES` and at "server shutdown"
  (`refman/9.7/en/caching-sha2-pluggable-authentication.html`, and the same
  on the 8.4 page) — and they are the moments the accepted cost above names.

  *Observed, through `tpl`, on 2026-11-04, at `2d652bd`.* The test
  `fr_mysql_045_an_uncached_login_over_an_unencrypted_session_ends_with_78`,
  in `tests/mysql_provider.rs`, emptied the server's authentication cache
  through the fixture's harness before each run, and ran
  `tpl schema info --direct --no-cache` as the reduced-grant reader
  `tpl_reader` under `disabled` against the four servers of the fixture that
  are supported, at `8.4.13` and at `9.7.4`, and under `preferred` against
  the two of them that offer no TLS. Every run exited `78`. The `error` line
  named the user. The `cause` named the host and the port, the user, the
  mode, `caching_sha2_password`, that the authentication was not completed
  and that `tpl` does not request the key, and stated that the server offered
  no TLS under `preferred` and under no other mode. The `hint` carried
  `tpl cfg database update shop --tls verify-identity`, and stated that the
  server must offer TLS under `preferred` alone. The server's record of each
  session held no statement, so the condition was raised at the
  authentication, and `tpl cfg database test shop`, with the cache emptied
  again, exited `78`. Against the two servers offering TLS, once a login
  under `verify-identity` had filled the cache, the same entry under
  `disabled` exited `0`, which is the accepted cost above. One message, as
  rmp `#329` recorded it against the server of `8.4` offering TLS, under
  `disabled`:

  ```
  error: cannot authenticate user 'tpl_reader' over an unencrypted session, for database entry 'shop'
  cause: connected to 127.0.0.1:13320 as user 'tpl_reader' under tls 'disabled', so the session is not encrypted; the authentication was not completed: the server asks for the full caching_sha2_password exchange, which needs its RSA public key, and tpl does not request that key over a session TLS does not protect
  hint:  encrypt the session: tpl cfg database update shop --tls verify-identity
  exit:  78 (EX_CONFIG)
  ```

  That the request for the key is never written is not in that record, which
  holds the statements of a session and not the exchange of its
  authentication. It rests on where the refusal is raised, before the request
  is written, which the project's decision records describe.

  *Closes* `OQ-077`, now listed under [Closed](open-questions.md#closed).

  *Added in the seventy-fourth edition,* for rmp `#331`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note quoted the manual and owed a login under `disabled`, and
  under `preferred` against a server offering no TLS, before and after the
  cache was filled, with the exit code and the message of each. The server's
  side is observed above; the code and the message, which are `tpl`'s, remain
  pending in the note above. The requirement is unchanged.

  *Amended in the seventy-seventh edition,* for a follow-up of rmp `#329`:
  the *Pending observation* note on the code and the message of `tpl` is
  replaced by the *Observed* note above, which the provider's tests made. The
  example's `cause` omitted the user and the statement that the
  authentication was not completed, both of which the requirement obliges
  and the implementation prints; it now carries both, and a second example
  shows the `preferred` case, whose `cause` and `hint` say that the server
  offered no TLS. The requirement is unchanged.

## The part of the database a read covers

- **FR-MYSQL-023**: A `mysql` read SHALL cover the database the entry names,
  per `FR-CONF-041`, and no other. The model SHALL cover, from the table
  catalogue, the rows whose table type is `BASE TABLE`, as tables carrying
  `table_type` `BASE TABLE`, per `FR-CAT-001` and `FR-CAT-002`, and the rows
  whose table type is `VIEW`, as views, per `FR-CAT-007`; and SHALL cover
  every routine of the database, per `FR-CAT-008`. A row whose table type is
  any other value SHALL NOT be covered. The coverage SHALL be a filter on the
  table type, per `FR-CAT-032`, and SHALL be applied to the column read as
  well, per `FR-CAT-052`. A row of the routine catalogue whose routine type is
  `LIBRARY` SHALL NOT be covered, and nor SHALL the rows of the parameter
  catalogue that belong to it.

  *Observed.* The table-type field is declared
  `enum('BASE TABLE','VIEW','SYSTEM VIEW')` on both series. Server-wide, `8.4`
  holds 176 rows of `BASE TABLE`, 105 of `VIEW` and 78 of `SYSTEM VIEW`, and
  `9.7` 179, 105 and 84, and no row of any other type. A table created with
  `CREATE TEMPORARY TABLE` in the reading session appears in none of the
  table, column and index catalogues, as the manual states: "The TABLES table
  does not list TEMPORARY tables"
  (`refman/8.4/en/information-schema-tables-table.html`). `table_type`
  therefore takes `BASE TABLE` alone on a `mysql` read. `SYSTEM VERSIONED` and
  `SEQUENCE`, which `FR-CAT-031` records for MariaDB, are outside the declared
  set, per `FR-MYSQL-039`.

  *A JSON duality view is a covered view.* On `9.7` a duality view is a row of
  the table catalogue whose table type is `VIEW`, and a row of the view
  catalogue, per difference 2 of `FR-MYSQL-014`, so this requirement covers it
  as it covers every view, and `FR-MYSQL-035` carries it. Nothing that marks
  it as a duality view is read: the tables that describe it are outside the
  closed list of `FR-MYSQL-015`. *Consequence, stated plainly.* A duality view
  is presented as an ordinary view, with its `select` as its definition, and a
  template that writes `CREATE VIEW` from that definition does not recreate a
  duality view. Excluding it instead would need a thirteenth table, which
  `FR-MYSQL-015` closes against.

  *Why a `LIBRARY` row is not covered.* `FR-CAT-008` covers routines as
  stored procedures and stored functions, and `FR-CAT-016` requires a routine
  read from a `supported` server to carry one of those two kinds. `9.7`
  declares a third routine type, `LIBRARY`, per difference 3 of
  `FR-MYSQL-014`, and creates a library by a statement of its own, apart from
  the statements that create a procedure and a function, per the statement
  index of the 9.7 manual, `refman/9.7/en/sql-data-definition-statements.html`.
  A library is therefore not a routine in the sense of `FR-CAT-008`, and
  covering it would break `FR-CAT-016` on a supported server. The last
  sentence of the requirement names the one routine type this corpus knows
  not to be a routine, and no other: a routine type outside all three, which
  a series newer than the set could declare, is still carried unchanged,
  under `FR-CAT-055`, as it is for a `mariadb` read.

  *Bounded claim.* No row of the fixture carries the routine type `LIBRARY`,
  and the fixture creates no library, so what the routine and parameter
  catalogues return for one was not observed; the 9.7 manual's description of
  the field still names `PROCEDURE` and `FUNCTION` alone
  (`refman/9.7/en/information-schema-routines-table.html`).

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The last sentence
  of the requirement is new: `FR-MYSQL-014` records that `9.7` declares a
  routine type that is not a procedure or a function, and the requirement read
  "every routine" without saying whether a row of that type is one. The
  *Pending observation* note, which quoted the manual and expected
  `table_type` to take `BASE TABLE` alone, is replaced by the *Observed* note
  above, which confirms it, and the notes on a duality view and a library are
  new.

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

  *Observed.* Every functional key part of the fixture reports `COLUMN_NAME`
  SQL `NULL` and its expression in `EXPRESSION`, with `INDEX_TYPE` `BTREE`,
  `COLLATION` `A` and `NULLABLE` `YES`, as the manual states: "For a
  functional key part, `COLUMN_NAME` column is `NULL` and `EXPRESSION`
  indicates the expression for the key part"
  (`refman/8.4/en/information-schema-statistics-table.html`). The fixture's
  multi-valued index is reported as a functional key part,
  ``cast(json_extract(`preferences`,_utf8mb4\'$.milestone_alerts\') as char(20) array)``,
  so this requirement excludes it. An index mixing a column and an
  expression, `(id, (LOWER(u_utf8)))`, reports `id` at sequence 1 and a
  functional key part at sequence 2, so the whole index is excluded. The
  hidden column that implements a functional key part, "implemented as hidden
  virtual generated columns" (`refman/8.4/en/create-index.html`), does not
  appear in the column catalogue: `customer` reports 24 columns, the largest
  position is 24, and no column carries the server's name for such a column,
  so `FR-CAT-052` has nothing to filter. The expression's text differs
  between the series for one index, per difference 5 of `FR-MYSQL-014`, and
  reaches no document.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it. The requirement is unchanged.

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

  *Observed.* The six fields, by the names above, on both series. The
  fixture's schemas returned `CATALOG_NAME` `def`, `SQL_PATH` SQL `NULL` and
  `DEFAULT_ENCRYPTION` `NO`. `freight`, which declares its character set and
  collation, returned `utf8mb4` and `utf8mb4_unicode_520_ci`; `masterdata`,
  which declares no collation, returned `utf8mb4_0900_ai_ci` on both series.
  The column headed *The manual says* is what
  `refman/8.4/en/information-schema-schemata-table.html` states, and no field
  returned anything else. MariaDB's schema catalogue also has six fields, the
  sixth a schema comment, per `FR-CTX-036`; MySQL's sixth is the encryption
  default. Neither reaches the model.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it. The requirement is unchanged.

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

  *Observed.* The 21 fields, by the names above, on both series. A view's row
  carries `TABLE_COMMENT` `VIEW`, and SQL `NULL` in `ENGINE`, `VERSION`,
  `ROW_FORMAT`, `TABLE_ROWS`, `TABLE_COLLATION` and `CREATE_OPTIONS`, as the
  manual states — "For views, most `TABLES` columns are 0 or `NULL` except
  that `TABLE_NAME` indicates the view name, `CREATE_TIME` indicates the
  creation time, and `TABLE_COMMENT` says `VIEW`"
  (`refman/8.4/en/information-schema-tables-table.html`) — so `FR-CAT-040`
  reaches a MySQL view as it reaches a MariaDB one. **An object with no
  comment carries the empty string, never SQL `NULL`**: a table's comment, a
  column's, an index's and a routine's each read as zero-length text, which
  is what `FR-CAT-039` records for MariaDB. The fixture's partitioned table
  reports `CREATE_OPTIONS` `partitioned`, which is not carried.

  *A comment loses every character outside the basic multilingual plane, as
  on MariaDB.* The catalogue declares every comment field `utf8mb3`. A table,
  column, index, function and procedure comment written with U+20BB7 was
  accepted with `Warning 1300`, "Cannot convert string … from utf8mb4 to
  utf8mb3", and returned `?`, byte `3F`, in the character's place, on both
  series; `SHOW CREATE TABLE` showed the same `?`, so the character is lost
  when the definition is stored and not when it is read. A character inside
  the plane, such as `港`, survives. `FR-CAT-036` therefore holds for a `mysql`
  read in its terms. The one difference from MariaDB is the warning, which
  MariaDB does not raise, per `FR-CAT-036`, and which reaches no read.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note, which left the comment of a table with none to be
  observed, is replaced by the *Observed* note above, and the note on
  characters outside the basic multilingual plane is new. The requirement is
  unchanged.

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

  The order SHALL be taken from the values of `ORDINAL_POSITION`, and the
  system SHALL NOT assume that the first column of a table is at position `1`
  or that the positions are contiguous.

  *Observed.* The 22 fields, by the names above, on both series, and no
  is-generated field. MariaDB's column catalogue carries an is-generated
  field MySQL's does not, per `FR-CAT-051`, and two system-versioning fields
  on three of its series, per difference 5 of `FR-SRV-038`; every field
  `FR-CTX-040` and `FR-CTX-041` read on MariaDB is in MySQL's list.

  *Observed: a generated invisible primary key.* A table created with no
  primary key while `sql_generate_invisible_primary_key` is `ON` reports, at
  position 1, a column `my_row_id` of type `bigint unsigned` whose `EXTRA` is
  `auto_increment INVISIBLE`, and the index catalogue reports it as the index
  `PRIMARY`. The read carries both as the catalogue reports them: an
  invisible auto-incremental column, and the table's primary key, under
  `FR-MYSQL-030` and `FR-MYSQL-032`. With
  `show_gipk_in_create_table_and_information_schema` set `OFF`, the column,
  its row of the index catalogue and its row of the table-constraint catalogue
  all disappear, **and the next column keeps position 2**, so the table's
  positions start at 2. The read then carries the table with no such column
  and no primary key. The manual describes both settings
  (`refman/8.4/en/create-table-gipks.html`). The fixture's servers run with
  the first setting `OFF` and the second `ON`, per the conditions recorded in
  the section on sources, so no table of the fixture has a generated key, and
  this was observed on a scratch table.

  *Consequence, stated plainly.* What a `mysql` read presents for a table with
  a generated invisible primary key depends on a setting of the reading
  session that no statement of `FR-MYSQL-015` reads or sets, per
  `FR-MYSQL-016`: the same table is read with an invisible column and a
  primary key, or with neither, as the server's default for the setting
  decides. Nothing in the document says which.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The paragraph
  that fixes the order by the values of the position, and not by a count
  from `1`, is new: a hidden generated invisible primary key leaves the
  positions starting at 2. The *Pending observation* note, which listed the
  fields from the manual and quoted it on a generated invisible primary key,
  is replaced by the two *Observed* notes above, and the consequence note is
  new.

- **FR-MYSQL-028**: For a `mysql` read, a column's type SHALL be decomposed
  as `FR-CTX-014` through `FR-CTX-018` and `FR-CTX-038` through `FR-CTX-041`
  decompose a MariaDB column's: `column_type` exactly as the catalogue returns
  it; each part from the field `FR-CTX-040` names; `unsigned` derived from
  `column_type`; `values` read from `column_type` under the rules of
  `FR-CTX-039`; and every part `null` for a type the system does not
  recognise, per `FR-CTX-018`. The grammar of `column_type` SHALL be taken to
  differ from MariaDB's as follows:

  | MariaDB, observed under `FR-CTX-038` | MySQL, observed |
  |---|---|
  | Every integer type carries a display width, `int(10)` | An integer type carries none, `int`, save `tinyint(1)` and a type with `ZEROFILL`, `smallint(4) unsigned zerofill` |
  | A column declared `JSON` reports `longtext` | A column declared `JSON` reports `json`, a native type |
  | A column declared `GEOMETRYCOLLECTION` reports `geometrycollection`, per `FR-ENV-046` | It reports `geomcollection` |
  | `inet4`, `inet6` and `uuid` are types | MySQL has none of the three |
  | `VECTOR` on `11.8` and `12.3` | `vector(N)` on `9.7` alone, per difference 1 of `FR-MYSQL-014` |

  The three `data_type` values a `mysql` read produces and no series of
  `FR-SRV-015` was observed to produce — `json`, `geomcollection` and
  `vector` — SHALL belong to the families `FR-ENV-049` gives them, and every
  other value a `mysql` read produces to the family `FR-ENV-046` gives it.

  *Observed.* The fixture's columns yield 37 distinct `DATA_TYPE` values on
  both series, none of them `inet4`, `inet6` or `uuid`, and the three rows of
  the table above that MySQL adds to MariaDB's are the whole difference
  between the two sets: `json` and `geomcollection` are the two values no
  MariaDB read produced, and every other value is one of MariaDB's. The
  display width is absent from every integer type but `tinyint(1)`, which
  every column declared `BOOLEAN` reports, and the one type declared with
  `ZEROFILL`, as the release notes state: statements "no longer show the
  display width for integer types", save "`TINYINT(1)`" and a type that
  "includes the `ZEROFILL` attribute"
  (`relnotes/mysql/8.0/en/news-8-0-19.html`). A column declared `NUMERIC`
  reports `decimal`. A `JSON` column reports `json` in both fields, with SQL
  `NULL` for its character set and its collation. An `ENUM` member holding an
  apostrophe reports it doubled, `'Lloyd''s Register'`, and one holding a
  comma reports it as written inside its quotes,
  `'Class 3, Flammable liquids'`, which are the rules of `FR-CTX-039`. The
  parts each family populates are those `FR-CTX-040` records for MariaDB, in
  every row the two providers share:

  | Type family, as `data_type` | `length` | `precision` | `scale` |
  |---|---|---|---|
  | `bigint` `int` `mediumint` `smallint` `tinyint` | `null` | `3`, `5`, `7`, `10` or `19`, and `20` for an unsigned `bigint` | `0` |
  | `decimal` | `null` | set | set |
  | `float` | `null` | `12` | `null` |
  | `double` | `null` | `22` | `null` |
  | `bit` | `null` | the bit width | `null` |
  | `datetime` `time` `timestamp` | `null` | `0`, `3` or `6` | `null` |
  | `date` `year`, the geometry types, `json` | `null` | `null` | `null` |
  | the character and text types, `enum`, `set` | set | `null` | `null` |
  | the binary and blob types | set | `null` | `null` |

  *Observed, on `9.7`: a `VECTOR` column.* A scratch column declared
  `VECTOR(3)` reports `DATA_TYPE` `vector`, `COLUMN_TYPE` `vector(3)`, `12` in
  `CHARACTER_MAXIMUM_LENGTH` and in `CHARACTER_OCTET_LENGTH`, and SQL `NULL`
  in every other part; the 9.7 manual states that "Each entry is a 4-byte
  (single-precision) floating-point value" (`refman/9.7/en/vector.html`).
  *Consequence, stated plainly.* A `vector` column's `length` is its size in
  bytes, four times its dimension count, and not the dimension count, which
  only `column_type` carries.

  *Consequence, stated plainly.* A column declared `JSON` is **textual** on
  both providers, per `FR-ENV-049`, and its parts differ: a MariaDB read
  carries `longtext`, with a length, a character set and a collation, and a
  MySQL read carries `json`, with all three `null`. A template that needs the
  declaration reads `data_type`, which says `json` on MySQL; on MariaDB the
  declaration survives only as the implicit constraint `FR-CAT-038` records.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The table's MySQL
  column read *as documented*, and is now observed; its row for
  `GEOMETRYCOLLECTION` is new. The sentence on the values MariaDB does not
  produce named `json` and `vector`, which took the family `FR-ENV-046` gives
  an unplaced value, none of the three, "until an observation places it".
  The observation added `geomcollection`, and the user decided on 2026-11-04
  that the three be placed by analogy with what MariaDB reports for the same
  declaration, which `FR-ENV-049` does; the sentence now cites it, and the
  consequence note on `JSON`, which read that the `textual` test answered
  differently on the two providers, follows. The sentence also read that no
  MariaDB read produces `vector`, which no observation supports: two series
  of `FR-SRV-015` accept the type, and what their catalogue reports for it
  is not recorded. The *Pending observation* note is replaced by the two
  *Observed* notes above.

- **FR-MYSQL-029**: For a `mysql` read, the system SHALL classify a column's
  default from `COLUMN_DEFAULT`, `EXTRA` and `IS_NULLABLE`, by the first row
  of the following table that matches, and SHALL derive the forms of
  `FR-CTX-012` from that classification alone:

  | Shape | Emitted |
  |---|---|
  | `COLUMN_DEFAULT` is SQL `NULL`, and `IS_NULLABLE` is `NO` | `null` |
  | `COLUMN_DEFAULT` is SQL `NULL`, and `IS_NULLABLE` is `YES` | `{"kind":"null"}` |
  | `EXTRA` carries the phrase `DEFAULT_GENERATED`, and the first character of the text is `(` and its last character is `)` | `expression`, carrying the text without that first and that last character, unescaped per `FR-MYSQL-046` |
  | `EXTRA` carries the phrase `DEFAULT_GENERATED` | `expression`, carrying the text unescaped per `FR-MYSQL-046` |
  | Anything else | `literal`, carrying the text unchanged |

  The classification of `FR-CTX-037` SHALL NOT be applied to a `mysql` read.
  The rows SHALL be matched against the text as the catalogue returns it. The
  third row SHALL remove exactly two characters, the first and the last, and
  SHALL remove them once; it SHALL NOT test whether the `(` it removes is
  closed by the `)` it removes, and SHALL NOT remove a further pair from what
  remains.

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
  the catalogue returns SQL `NULL` for both; a `NOT NULL` column with no
  default returns SQL `NULL` too. `IS_NULLABLE` separates the two forms of
  `FR-CTX-012` that MariaDB's field separates in one value, which is the
  reading `FR-LITE-027` makes for SQLite and not the reconstruction
  `FR-CTX-012` rejects.

  *Observed.* Every default of the fixture, and of a scratch table declaring
  the shapes the fixture does not, was read on both series, and every one
  falls to the row of the classification its shape gives it. In the second
  column, text is shown as the field holds it, each backslash a byte of the
  value:

  | Declared | `COLUMN_DEFAULT` | `EXTRA` carries | Emitted |
  |---|---|---|---|
  | No `DEFAULT` clause, or `DEFAULT NULL` | SQL `NULL` | no phrase | `null` or `{"kind":"null"}`, by `IS_NULLABLE` |
  | `'EUR'` | `EUR` | no phrase | `literal`, `EUR` |
  | `''` | the empty string, of length 0 | no phrase | `literal`, the empty string |
  | `'NULL'` | `NULL`, the four characters | no phrase | `literal`, `NULL` |
  | A string holding an apostrophe, a backslash or a double quote | `it's`, `back\slash`, `8'6"`, as stored, with no escape | no phrase | `literal`, the text unchanged |
  | `1.5e3`, `18.5`, `-1`, `TRUE` | `1500`, `18.5`, `-1`, `1` | no phrase | `literal` |
  | `b'00000101'`, and `5` on a `BIT(4)` column | `b'101'` | no phrase | `literal`, `b'101'` |
  | `x'27'`; `0x0A0B` and `X'0a0b'`; `'ab'` on a `VARBINARY` column | `0x27`; `0x0A0B`; `0x6162` | no phrase | `literal`, the hexadecimal text |
  | `CURRENT_TIMESTAMP`, `CURRENT_TIMESTAMP(3)`, `CURRENT_TIMESTAMP(6)`, `NOW()` | the first three as declared; `CURRENT_TIMESTAMP` for the fourth | `DEFAULT_GENERATED` | `expression`, the text unchanged |
  | `(CURRENT_DATE + INTERVAL 30 DAY)`, `((1 + 2) * (3 + 4))` | `(curdate() + interval 30 day)`, `((1 + 2) * (3 + 4))` | `DEFAULT_GENERATED` | `expression`, the text between the outer parentheses |
  | `(NOW())`, `(UUID())`, `(PI())`, `(-1)`, `(JSON_OBJECT(…))` | `now()`, `uuid()`, `pi()`, `-(1)`, `json_object(…)` | `DEFAULT_GENERATED` | `expression`, the text, unescaped per `FR-MYSQL-046` where it holds a string literal |
  | `(x'554E413A2B2E3F2027')` on a `BLOB` column | `0x554e413a2b2e3f2027`, in lower case | `DEFAULT_GENERATED` | `expression`, the text unchanged |
  | `('it''s')` | `_utf8mb4\'it\\\'s\'`, hex `5F757466386D62345C2769745C5C5C27735C27` | `DEFAULT_GENERATED` | `expression`, `_utf8mb4'it\'s'`, unescaped per `FR-MYSQL-046` |

  `CURRENT_TIMESTAMP` is marked `DEFAULT_GENERATED` in every form. The server
  writes an expression in its own rendering and keeps outer parentheses only
  around an operator expression, and every text observed that begins with `(`
  and ends with `)` has matching outer parentheses, so on every text observed
  the two characters the third row of the classification removes are a pair
  that encloses the whole. The manual states
  that `COLUMN_DEFAULT` "is `NULL` if the column has an explicit default of
  `NULL`, or if the column definition includes no `DEFAULT` clause", and that
  `EXTRA` holds "`DEFAULT_GENERATED` for columns that have an expression
  default value" (`refman/8.4/en/information-schema-columns-table.html`); it
  does not state the text a default is reported as, on that page, on
  `refman/8.4/en/data-type-defaults.html` or on
  `refman/8.4/en/string-literals.html`.

  *The text of an expression is the server's, with two marks no MariaDB
  expression this corpus records carries.* Each string literal in it carries
  an [introducer](glossary.md#introducer),
  `_utf8mb4` or the character set of the column, and the whole text carries
  one layer of backslash escaping beyond MySQL's own SQL escaping: a quote is
  written `\'`, a backslash `\\`, and a newline inside a literal `\\n`.
  `('back\\slash')` is reported `_utf8mb4\'back\\\\slash\'`. The same two
  marks are in a generation expression, per `FR-MYSQL-031`, a check clause,
  per `FR-MYSQL-034`, and the expression of a functional key part, per
  `FR-MYSQL-024`; they are not in a routine body or a trigger body, which the
  catalogue returns as written. A literal default carries neither.

  *What the model receives.* An expression default is carried with the extra
  layer removed and the introducers kept, per `FR-MYSQL-046`: `('it''s')`
  yields an `expression` whose value is `_utf8mb4'it\'s'`, and not the
  nineteen characters the server writes. An observed expression that holds no
  string literal holds no backslash either, and is carried as the server
  writes it, unchanged or between its outer parentheses, which is the
  principle `FR-CTX-037` applies to a MariaDB expression.

  *Why the first and the last character, and not a pair that matches.* The
  third row is the sixth row of `FR-CTX-037`, which classifies a MariaDB
  default that "begins with `(` and ends with `)`" and carries the text
  between them, so a `mariadb` read and a `mysql` read remove the outer
  parentheses of an expression default by one rule. The two readings agree on
  every text observed, because each such text has matching outer
  parentheses. They differ on a text such as `(a) + (b)`, whose first `(`
  closes before its last character: this row carries it as `a) + (b`, and a
  reading by matching pair would carry it unchanged. No server of the fixture
  wrote such a text, because both series write an operator expression inside
  one pair that encloses it, as the *Observed* note records.

  *Rejected: removing the outer parentheses only where they are a matching
  pair.* Telling whether the first `(` is closed by the last `)` needs a
  reading of the text that skips every parenthesis inside a string literal or
  a quoted identifier, under the escaping `FR-MYSQL-046` describes: a parser
  that no observed text calls for, and a rule that a `mariadb` read does not
  apply. *Accepted cost, stated because it is silent.* A server that writes
  an expression default in that shape is carried with parentheses that do not
  balance, at exit `0`. Such a text is a defect to report with the DDL that
  produced it, as `FR-CTX-037` states of its own rule, and is the case this
  row is most likely to be amended for.

  *Consequence, stated plainly: a binary literal default is hexadecimal
  text.* A string literal default on a binary column is reported in
  hexadecimal, with no phrase in `EXTRA`, so `'ab'` on a `VARBINARY` column
  yields the `literal` `0x6162`, and a template that quotes a literal writes
  `'0x6162'`, which is another default. A bit default yields `b'101'`, with
  any leading zeros dropped. Both are the server's literal text carried
  unchanged, which is how `FR-CTX-037` carries MariaDB's `b'101'`.

  *Consequence, stated plainly.* A string default the DDL wrote `'EUR'`
  yields `{"kind":"literal","value":"EUR"}` from either provider. An
  expression's text is the server's own and differs between them —
  `current_timestamp()` from MariaDB, per `FR-CTX-037`, where MySQL writes
  `CURRENT_TIMESTAMP` — and is passed through as each server writes it, save
  the layer `FR-MYSQL-046` removes from MySQL's.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note listed six questions — whether `CURRENT_TIMESTAMP` is
  marked, whether parentheses are kept, whether a binary literal is
  hexadecimal, how a bit literal reads, whether the empty string is empty
  text, and whether a quote is escaped — and said that the amendment after the
  observation would fix a rule removing any escapes. The *Observed* note
  above answers all six, and the classification holds for every shape
  observed. The observation found the extra layer and raised `OQ-078`, and
  the user's decision of 2026-11-04 closed it in the same edition: the two
  rows that emit an `expression` carry the text unescaped per
  `FR-MYSQL-046`, the sentence after the table says which text the rows are
  matched against, and the note on what the model receives says what is
  carried. The consequence note on a binary literal default is new, and the
  last consequence note names the layer removed.

  *Amended in the seventy-sixth edition,* for rmp `#329`. The third row read
  "the text begins with `(` and ends with `)`" and carried "the text between
  the outer parentheses", which left open whether the outer parentheses are
  the first and the last character or a pair that matches; the two differ on
  a text whose first `(` closes before its end. The row now names the first
  and the last character, the sentence after the table says that they are
  removed once and not tested for a match, the *Observed* note says that on
  every text observed they are a pair, and the note on why, with the reading
  rejected, is new. Every text observed is carried as before.

- **FR-MYSQL-046**: For a `mysql` read, the system SHALL remove exactly one
  layer of backslash escaping from three texts, and from no other: the text
  an `expression` default carries, per `FR-MYSQL-029`; a generated column's
  expression, per `FR-MYSQL-031`; and a `CHECK` constraint's clause, per
  `FR-MYSQL-034`. The system SHALL read each text from its first character to
  its last and write:

  | Where the text holds | The system SHALL write |
  |---|---|
  | A backslash followed by a backslash, `\\` | one backslash, `\`, and continue after the pair |
  | A backslash followed by a single quote, `\'` | one single quote, `'`, and continue after the pair |
  | Any other character, a backslash followed by any other character or ending the text included | that character, unchanged |

  Every [introducer](glossary.md#introducer) SHALL be kept, and no other
  character of the text SHALL change.

  *Observed*, as bytes, on both series, the same on each:

  | Field, and the DDL that wrote it | As the server writes it | As the model carries it |
  |---|---|---|
  | `COLUMN_DEFAULT`, `DEFAULT ('it''s')` | `_utf8mb4\'it\\\'s\'` | `_utf8mb4'it\'s'` |
  | `COLUMN_DEFAULT`, `DEFAULT ('back\\slash')` | `_utf8mb4\'back\\\\slash\'` | `_utf8mb4'back\\slash'` |
  | `COLUMN_DEFAULT`, `DEFAULT ('line1\nline2')` | `_utf8mb4\'line1\\nline2\'` | `_utf8mb4'line1\nline2'` |
  | `GENERATION_EXPRESSION`, `AS (CONCAT('it''s ', l_quote))` | ``concat(_utf8mb4\'it\\\'s \',`l_quote`)`` | ``concat(_utf8mb4'it\'s ',`l_quote`)`` |
  | `CHECK_CLAUSE`, `CHECK (l_quote <> 'o''clock')` | ``(`l_quote` <> _utf8mb4\'o\\\'clock\')`` | ``(`l_quote` <> _utf8mb4'o\'clock')`` |

  The fixture's own texts read the same way: the generation expression
  ``(`imdg_class` <> _utf8mb4\'Not regulated\')`` is carried
  ``(`imdg_class` <> _utf8mb4'Not regulated')``. The observation read as
  bytes fifteen distinct values holding a backslash — defaults, generation
  expressions, check clauses and functional key parts, of the fixture and of
  a scratch table. Fourteen are the text of an expression, and each, read as
  this requirement reads it, resolves into the two pairs above and characters
  that are not a backslash: no backslash of an expression's text was seen
  outside a pair. The fifteenth is the literal default `back\slash`, which
  carries no layer and which this requirement does not reach.

  *What the result is.* Read under the grammar MySQL's manual gives a string
  literal — `[_charset_name]'string'`, in which `\'` stands for a single
  quote, `\\` for a backslash and `\n` for a newline
  (`refman/8.4/en/string-literals.html`) — each text the model carries is the
  expression the DDL declared, its literals marked with the character set the
  server chose. A template that writes it into DDL writes MySQL's own SQL, as
  a template that writes a MariaDB expression writes MariaDB's. No statement
  of the observation submitted a carried text to a server. *Consequence,
  stated plainly.* The same page states that the escape sequences have their
  meaning "unless the `NO_BACKSLASH_ESCAPES` SQL mode is enabled", so the
  text a read carries is SQL for a session without that mode.

  *What the rule does not reach.* A literal default carries no layer, per
  `FR-MYSQL-029`, and is carried unchanged; a routine body and a trigger body
  carry none, per `FR-MYSQL-036` and `FR-MYSQL-038`; the expression of a
  functional key part carries the layer and is not carried into the model,
  per `FR-MYSQL-032`; and no view of the fixture held a string literal, so
  whether a view's definition carries the layer was not observed, per
  `FR-MYSQL-035`, and this requirement does not reach it.

  *Rationale.* The user decided on 2026-11-04 that a `mysql` read carry SQL
  text the server accepts, as a `mariadb` read already does, and that the
  layer be removed from these three texts with the introducers kept. The
  layer is regular, so one rule removes it. Removing a layer of quoting the
  catalogue adds is the step `FR-CTX-037` and `FR-LITE-027` already take for
  a literal, whose doubled apostrophes they collapse; this requirement takes
  it for an expression, and only for the layer MySQL adds beyond its own SQL
  escaping, so the escaping SQL needs stays in the text.

  *Prior art.* The MySQL inspector of `ariga/atlas` passes the same three
  texts, and a functional key part's expression, through its `unescape`,
  defined in `sql/mysql/driver.go` and called in `sql/mysql/inspect.go`, and
  passes no MariaDB text through it. That function replaces `\'` and `\\` by
  the character after the backslash, as this requirement does, and drops a
  backslash followed by any other character, where this requirement keeps it:
  the user's decision names the sequences the observation shows, and the
  observation shows those two alone. No text observed holds such a backslash,
  so the two rules agree on every text observed.

  *Rejected, by the user:* carrying each text as the server writes it, which
  keeps the principle of `FR-CTX-037` and leaves every template that writes
  DDL from a `mysql` read to remove the layer itself, the heuristic
  `BR-CTX-002` keeps out of templates, and a template that forgets it writes
  text that MySQL's grammar does not read as the expression declared.
  *Rejected, by the user:* removing the introducers
  too. The text would resemble MariaDB's, but an introducer names a literal's
  character set, which the server chooses by column on one series, per
  difference 5 of `FR-MYSQL-014`, so dropping it changes what the text means,
  and it is a second rewrite with no ground in the observation. *Rejected, by
  the user:* carrying both texts, as written and with the layer removed. It
  adds a property to the model, which adds a key to every MariaDB document,
  and `BR-CTX-002` rejects a raw field carried beside the structure.

  *Accepted cost, stated because it is silent.* The system rewrites three
  fields the server returned, and the rule rests on one observation, which
  saw a quote, a backslash and a newline escaped. A text holding another
  sequence is carried with that sequence unchanged, at exit `0`; a text
  carried wrongly is a defect to report with the DDL that produced it, and is
  the case this requirement is most likely to be amended for.

  *Closes* `OQ-078`, now listed under [Closed](open-questions.md#closed).

  *Added in the seventy-fifth edition,* for rmp `#326`, by the user's
  decision of 2026-11-04.

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
  | `on update` followed by one word | the column carries an `ON UPDATE` default, carried as the sentence after this table states | `FR-CTX-020` |
  | `DEFAULT_GENERATED` | the column's default is an expression | `FR-MYSQL-029` |
  | `MASKING POLICY` | the column carries a masking policy | not carried — the model has no property for it |

  WHERE `EXTRA` carries the phrase `on update`, the value the model carries
  for the column's `ON UPDATE` default SHALL be the phrase itself: the
  characters of `EXTRA` from the start of `on update` to the end of the word
  that follows it, exactly as `EXTRA` writes them, letter case included. A
  word is the characters after the single space that follows `update`, up to
  the next space or the end of the field. The value is thus `on update`, one
  space and the server's spelling of the expression —
  `on update CURRENT_TIMESTAMP`, `on update CURRENT_TIMESTAMP(3)` — which is
  the shape of the value a `mariadb` read carries, the whole of the column
  attribute field, `on update current_timestamp()`, under `FR-CAT-041` and
  `FR-CTX-020`. A column whose `EXTRA` carries no `on update` phrase carries
  no `ON UPDATE` default.

  IF `EXTRA` holds text that no phrase matches, THEN the system SHALL take
  the facts of the phrases it recognises, SHALL NOT fail the read, and SHALL
  NOT change the exit code on account of it, on the ground `FR-CAT-055` gives
  for a value outside a recorded set.

  *Observed.* Over the fixture's 329 columns, on both series, `EXTRA` took
  eleven values, and no other:

  | `EXTRA` | Columns |
  |---|---|
  | the empty string | 290 |
  | `auto_increment` | 11 |
  | `auto_increment INVISIBLE` | 1 |
  | `DEFAULT_GENERATED` | 13 |
  | `DEFAULT_GENERATED INVISIBLE` | 1 |
  | `DEFAULT_GENERATED on update CURRENT_TIMESTAMP` | 1 |
  | `DEFAULT_GENERATED on update CURRENT_TIMESTAMP(3)` | 1 |
  | `INVISIBLE` | 1 |
  | `STORED GENERATED` | 7 |
  | `VIRTUAL GENERATED` | 2 |
  | `VIRTUAL GENERATED INVISIBLE` | 1 |

  **A column carries more than one phrase, separated by single spaces, and
  in one order**: the storage phrase or the default phrase first, then
  `on update` and its word, and `INVISIBLE` last. The word after `on update`
  is `CURRENT_TIMESTAMP` or `CURRENT_TIMESTAMP(3)`, one word in each case.
  The reading above takes the phrases in any order, so it reads the observed
  order and does not depend on it. Each phrase is one the manual lists:
  `auto_increment`; "`on update CURRENT_TIMESTAMP` for `TIMESTAMP` or
  `DATETIME` columns that have the `ON UPDATE CURRENT_TIMESTAMP` attribute";
  "`STORED GENERATED` or `VIRTUAL GENERATED` for generated columns"; and
  "`DEFAULT_GENERATED` for columns that have an expression default value"
  (`refman/8.4/en/information-schema-columns-table.html`); and "For invisible
  columns, `EXTRA` displays `INVISIBLE`" (`refman/8.4/en/invisible-columns.html`).
  A generated invisible primary key reads `auto_increment INVISIBLE`, per
  `FR-MYSQL-027`.

  *Pending observation.* The phrase `MASKING POLICY`. The 9.7 manual lists
  "`MASKING POLICY` for columns with a masking policy"
  (`refman/9.7/en/information-schema-columns-table.html`), and the 8.4
  manual does not. No column of the fixture can carry it: `8.4` refuses
  `CREATE MASKING POLICY` as a syntax error, and `9.7` parses it and refuses
  it because the masking-policy component is not installed, per difference 8
  of `FR-MYSQL-014`; the 9.7 manual states that "If the component is not
  installed, this statement returns an error"
  (`refman/9.7/en/create-masking-policy.html`), the server's component table
  is empty, and neither Community image carries the component's library.
  Where the phrase falls among the others, and whether it is spelt as the
  manual spells it, is therefore unobserved. A `9.7` server with that
  component installed, which needs an image of another edition, would observe
  it. The reading above does not depend on either answer: the phrase carries
  no fact the model has a property for, and text the reading does not
  recognise leaves the read and its exit code unchanged.

  *Consequence, stated plainly.* MariaDB writes the `ON UPDATE` attribute
  `on update current_timestamp()`, per `FR-CAT-041`, and MySQL writes it
  `on update CURRENT_TIMESTAMP`, or `on update CURRENT_TIMESTAMP(3)` for a
  fractional-second column. A column declared `ON UPDATE CURRENT_TIMESTAMP`
  therefore carries `on update current_timestamp()` from a `mariadb` read and
  `on update CURRENT_TIMESTAMP` from a `mysql` read: one shape from both, the
  phrase and then the expression, and in each the expression as its server
  spells it, which is the principle the last consequence note of
  `FR-MYSQL-029` states for a default. The words observed after `on update`
  are `CURRENT_TIMESTAMP` and `CURRENT_TIMESTAMP(3)`; a word the fixture does
  not produce, such as another fractional-second precision, is carried as
  `EXTRA` writes it, and was not observed.

  *Rejected: carrying the word alone*, `CURRENT_TIMESTAMP`, which the table
  read until the seventy-sixth edition as "an `ON UPDATE` default, which is
  that word". It would give one property two shapes, the phrase from a
  `mariadb` read and the expression alone from a `mysql` read, so a template
  writing the value into DDL would have to tell the providers apart to know
  whether to write `ON UPDATE` before it, where the overview of this file has
  a template render from a MySQL read on the principles it renders from a
  MariaDB read.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note covered every phrase and every combination, and said that
  the amendment after the observation would fix the order and the separator.
  The *Observed* note above records both. The requirement keeps reading the
  phrases in any order, because a reading that tolerates every order reads the
  observed one, and requiring that order would turn a server's change of order
  into a lost fact. The *Pending observation* note now covers `MASKING POLICY`
  alone. The requirement and its table are unchanged.

  *Amended in the seventy-sixth edition,* for rmp `#329`. The table said that
  the column carries an `ON UPDATE` default "which is that word", the word
  after `on update`, and the consequence note said that the expression
  carried is each server's own spelling, so the value carried could be read
  as the word or as the phrase. The sentence after the table now fixes it as
  the phrase, in the shape a `mariadb` read carries, and the table's cell
  points at it; the consequence note says what each provider yields, and the
  reading rejected is recorded. No fact the reading takes from `EXTRA`
  changes.

- **FR-MYSQL-031**: For a `mysql` read, a generated column SHALL carry its
  storage kind from the phrase of `FR-MYSQL-030`, `VIRTUAL GENERATED` or
  `STORED GENERATED`, and its expression from `GENERATION_EXPRESSION`,
  unescaped per `FR-MYSQL-046`, as `FR-CAT-051` requires. A column whose
  `EXTRA` carries neither phrase SHALL NOT be generated, whatever
  `GENERATION_EXPRESSION` holds.

  *Why `EXTRA` and not another field.* MySQL's column catalogue has no
  is-generated field, which `FR-CAT-051` reads beside the attribute field on
  MariaDB and which `FR-SRV-003` names as the third reason a MariaDB reader is
  silently wrong on MySQL. `FR-CAT-051` already takes the storage kind from
  the attribute field, so the fact is read from the field both servers carry.

  *Observed.* `GENERATION_EXPRESSION` is the empty string, and never SQL
  `NULL`, on every one of the 319 columns of the fixture that are not
  generated, as the manual states: "Empty for nongenerated columns"
  (`refman/8.4/en/information-schema-columns-table.html`). The empty string
  makes no column generated, because the field does not decide it: the
  phrase of `FR-MYSQL-030` does. On a generated column the field holds the
  expression in the server's own rendering, with the two marks
  `FR-MYSQL-029` records — an introducer before each string literal and one
  extra layer of backslash escaping — and with outer parentheses around an
  operator expression: ``(`imdg_class` <> _utf8mb4\'Not regulated\')``,
  ``((`base_rate` + `bunker_surcharge`) + `congestion_surcharge`)``, and, on a
  scratch column, ``concat(_utf8mb4\'it\\\'s \',`l_quote`)``. A generated
  column's `COLUMN_DEFAULT` is SQL `NULL`. The expression is carried with the
  extra layer removed and the introducers kept, per `FR-MYSQL-046`, so the
  first of these is carried ``(`imdg_class` <> _utf8mb4'Not regulated')``.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note said that the amendment after the observation would fix a
  rule removing any escaping observed. The *Observed* note above records the
  escaping, which raised `OQ-078`, and the user's decision of 2026-11-04
  closed it in the same edition: the expression is now carried unescaped per
  `FR-MYSQL-046`, which also records the reference inspector's reading.

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

  *Observed, and the manual contradicted.* The manual documents `INDEX_TYPE`
  as "The index method used (`BTREE`, `FULLTEXT`, `HASH`, `RTREE`)"
  (`refman/8.4/en/information-schema-statistics-table.html`). The fixture's
  spatial index reports **`SPATIAL`**, which is not in that list, with
  `SUB_PART` `32` and `COLLATION` `A`, on both series. It is what MariaDB
  reports, value for value, per `FR-CAT-042`, so a spatial index is no
  divergence between the two products, and its `SUB_PART` of `32` reads as
  MariaDB's does: what the catalogue states about the index, not a prefix the
  DDL asked for.

  *Observed.* The 18 fields, by the names above, on both series. A `FULLTEXT`
  index reports SQL `NULL` in `COLLATION` and in `SUB_PART`; a descending
  member reports `COLLATION` `D`; a prefix reports its length in `SUB_PART`,
  `20` or `24` over the fixture; and the fixture's invisible index reports
  `IS_VISIBLE` `NO`, the others `YES`. `NULLABLE` reads `YES` or the empty
  string, and `PACKED` SQL `NULL` on every row. `COMMENT` is the empty string
  on all 91 rows of the fixture, and reads `disabled` on a MyISAM table after
  `ALTER TABLE … DISABLE KEYS`, as the manual's "information about the index
  not described in its own column, such as `disabled`" states. A table with no
  primary key and a `NOT NULL` unique index, `currency`, reports that index
  under its own name, `uq_currency_iso_code`, and no row named `PRIMARY`, so
  the model carries no primary key for it.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note left what a spatial index reports to be observed, beside
  a manual whose list omits `SPATIAL`. It reports `SPATIAL`, as MariaDB does,
  and row 16 of `FR-MYSQL-040`, which recorded a divergence there, is removed.
  The two *Observed* notes above replace the note. The requirement is
  unchanged: `INDEX_TYPE` was always carried unchanged.

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

  `UPDATE_RULE` and `DELETE_RULE` SHALL be carried as the catalogue reports
  them, `SET DEFAULT` included. `FR-CAT-033`, which bars `SET DEFAULT` from
  the model, governs a `mariadb` read, as its note of the sixty-fifth edition
  states, and does not reach a `mysql` read.

  *Observed, and the manual contradicted.* The manual states that `SET
  DEFAULT` "is recognized by the MySQL parser, but both InnoDB and NDB reject
  table definitions containing ON DELETE SET DEFAULT or ON UPDATE SET DEFAULT
  clauses" (`refman/8.4/en/create-table-foreign-keys.html`, and the same on
  the 9.7 page). On both series, InnoDB accepted
  `FOREIGN KEY … ON DELETE SET DEFAULT ON UPDATE SET DEFAULT` with no error
  and no warning; `SHOW CREATE TABLE` kept both clauses; and
  `REFERENTIAL_CONSTRAINTS` reported `SET DEFAULT` for both rules. A key
  added by `ALTER TABLE … ADD CONSTRAINT … ON DELETE SET DEFAULT` was accepted
  too, and reported `NO ACTION` and `SET DEFAULT`. **The action is not
  performed**: a `DELETE` or an `UPDATE` of a parent row the key references
  failed with `ERROR 1451 (23000)`, "Cannot delete or update a parent row: a
  foreign key constraint fails", and left the child row unchanged. MariaDB
  accepts the same clause and reports `RESTRICT`, per `FR-CAT-033`; MySQL
  reports what was declared and refuses at run time.

  *Observed.* The 11 fields of `REFERENTIAL_CONSTRAINTS` and the 12 of
  `KEY_COLUMN_USAGE`, by the names above, on both series. `CASCADE`,
  `SET NULL`, `RESTRICT` and `NO ACTION` are reported as declared, and a key
  declared with neither clause reports `NO ACTION` for both, as the manual
  states: "For an `ON DELETE` or `ON UPDATE` that is not specified, the
  default action is always `NO ACTION`". The two rule fields are declared as
  an enumeration of the five actions, so no sixth value can arrive.
  `MATCH_OPTION` reads `NONE` on every key. A key that crosses into another
  schema names that schema in `UNIQUE_CONSTRAINT_SCHEMA`, and `FR-CAT-057`
  excludes it. `UNIQUE_CONSTRAINT_NAME` reads `PRIMARY` for every key of the
  fixture, and the unique key's name for a key that references a `UNIQUE`
  key; for a key that references an index that is not unique it reads SQL
  `NULL`, and `FR-CAT-056` carries `referenced_key` as `null`. Such a key is
  refused by default, with `ERROR 6125`, and is accepted only with
  `restrict_fk_on_non_standard_key` set `OFF`, which draws the deprecation
  warnings `4166` and `6124`; the fixture's servers run with it `ON`.

  *Accepted cost, stated because it is silent.* A key declared
  `SET DEFAULT` is presented with `SET DEFAULT`, at exit `0`, and nothing in
  the document says that the server does not perform the action. A template
  that emits DDL from it emits the clause the server accepted and kept, so
  the round trip is faithful; a template that documents behaviour from it
  states an action no write will ever see. It is the cost `FR-MYSQL-034`
  accepts for a `CHECK` constraint declared `NOT ENFORCED`, on the same
  ground: the model carries what the catalogue states, and the catalogue
  states the declaration.

  *Rejected: carrying `RESTRICT` in its place*, which is what a MariaDB read
  carries for the same declaration. MySQL reports `SET DEFAULT`, and
  substituting another action is a translation the server does not make, and
  one no template could undo. *Rejected:
  `null`.* The rule is reported, so it is not a fact the server fails to
  provide, which is the only ground `BR-MYSQL-003` gives for `null`.
  *Rejected: refusing the read or excluding the key.* The key exists and is
  enforced against deletes and updates of its parent, and excluding it would
  drop a relation the database holds. *What would change this.* A property
  stating whether a referential action is performed, which adds a key to
  every foreign key of every document and is a change to
  [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) the user has not asked for.

  *Why this needs no change to the model.* A `sqlite` read already carries
  `SET DEFAULT`, per `FR-LITE-031`, and `FR-CAT-055` already records a value
  set of five spellings for that provider; `on_update` and `on_delete` are
  carried unchanged on every provider. A `mysql` read adds no value the model
  does not already admit.

  *Why byte for byte, and what bounds it.* No statement of `FR-MYSQL-015` reads
  `lower_case_table_names`, which decides how the server compares table names;
  the reference inspector reads it at connection time, in
  `sql/mysql/driver.go`. *Observed*, under `lower_case_table_names` `0`: a
  key referencing a table declared `Shipping_Line` reports
  `REFERENCED_TABLE_NAME` `Shipping_Line`, byte for byte equal to the table's
  own name in the table catalogue, and a key written to reference
  `shipping_line` is refused at DDL time with `ERROR 1824`, "Failed to open
  the referenced table 'shipping_line'", so no key reaches the catalogue with
  a referenced name in another case. On a server that stores or compares
  names folded, a key whose referenced name differed in case from the table's
  own would be excluded, at exit `0`, under `FR-CTX-042`'s ground. Only the
  setting `0` was observed, which is every server of the fixture, and the
  observation is bounded to it.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The paragraph on
  the two rule fields is new. The *Pending observation* note read, from the
  manual, that InnoDB refuses `SET DEFAULT` at DDL time, so that `FR-CAT-033`
  held for a `mysql` read by that refusal. The servers accept the clause and
  report it, so the requirement now states how the model carries it, and the
  note on `FR-CAT-033` in [catalogue-coverage.md](catalogue-coverage.md) is
  corrected with it. The rest of the note is replaced by the *Observed* notes
  above, and the bounding note now records what was observed of letter case.
  Row 18 of `FR-MYSQL-040` follows.

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
  | `CHECK_CONSTRAINTS` | `CHECK_CLAUSE` | `clause`, unescaped per `FR-MYSQL-046` |

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

  *Observed.* `CHECK_CONSTRAINTS` has four fields and `TABLE_CONSTRAINTS`
  seven, by the names above, on both series, and no field of the first names
  the owning table, as the manual states
  (`refman/8.4/en/information-schema-check-constraints-table.html`,
  `refman/8.4/en/information-schema-table-constraints-table.html`).
  `ENFORCED` reads `NO` on the fixture's two constraints declared
  `NOT ENFORCED` and `YES` on the others. An unnamed constraint is named from
  its table, `_chk_` and an ordinal, whichever level it was written at:
  `cargo_item_chk_1` was written on a column and `container_type_chk_1` on
  the table, as the manual's "from the table name, a literal `_chk_`, and an
  ordinal number" states (`refman/8.4/en/create-table-check-constraints.html`).
  No constraint names a `JSON` column, so MySQL attaches no implicit
  constraint where MariaDB attaches one, per `FR-CAT-038`. As the
  reduced-grant reader, every row of `CHECK_CONSTRAINTS` has its row in
  `TABLE_CONSTRAINTS`, so the join drops none and no cross-check is called
  for; a scratch reader holding no privilege on a table saw that table's
  constraint in neither catalogue, which is the table's own absence and not
  a short read of it.

  *The clause is the server's rendering, with the marks of `FR-MYSQL-029`.*
  `CHECK_CLAUSE` holds the clause wrapped in one pair of outer parentheses,
  with an introducer before each string literal and one extra layer of
  backslash escaping: `CHECK (l_quote <> 'o''clock')` reads
  ``(`l_quote` <> _utf8mb4\'o\\\'clock\')``, and the fixture's
  `chk_voyage_direction` reads
  ``(right(`voyage_number`,1) in (_utf8mb4\'N\',_utf8mb4\'S\',_utf8mb4\'E\',_utf8mb4\'W\'))``.
  The clause `FR-CAT-046` records of MariaDB carries no outer parentheses,
  and no clause this corpus records of MariaDB carries either mark.
  `clause` carries the field with the extra layer removed and the
  introducers kept, per `FR-MYSQL-046`, so the first is carried
  ``(`l_quote` <> _utf8mb4'o\'clock')``, with its outer parentheses; a
  template that writes `CHECK (` and `)` around it writes a second pair of
  parentheses, which the server accepts.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it, and the note on the clause, which the observation refined, is new. The
  escaping it records raised `OQ-078`, and the user's decision of 2026-11-04
  closed it in the same edition: the row of `CHECK_CLAUSE` now carries the
  clause unescaped per `FR-MYSQL-046`.

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
  `FR-CAT-052`. A view whose `VIEW_DEFINITION` is the empty string SHALL be
  treated as a missing privilege, per `FR-PRIV-011`.

  *Observed, and the manual contradicted, twice.* The manual states that the
  value of `CHECK_OPTION` "is one of NONE, CASCADE, or LOCAL", and that "You
  must have the SHOW VIEW privilege to access this table"
  (`refman/8.4/en/information-schema-views-table.html`, and the same on the
  9.7 page). On both series, the fixture's view declared
  `WITH CASCADED CHECK OPTION` reports **`CASCADED`**, and the field is
  declared `enum('NONE','LOCAL','CASCADED')`, so `CASCADE` cannot be
  returned. It is what MariaDB returns, per `FR-CAT-047`, so the value is no
  divergence between the two products. And the reduced-grant reader, which
  holds no `SHOW VIEW`, **reads every row of the table**, all five of the
  fixture, with every field populated but one: `VIEW_DEFINITION` is the
  **empty string**, of length 0, and not SQL `NULL`. That is the shape
  `FR-PRIV-018` records of MariaDB, and the one `FR-PRIV-011` detects, so the
  last sentence of the requirement applies that check to a `mysql` read in
  its own terms.

  *Observed.* Ten fields, by the names above, on both series, and no
  `ALGORITHM`. `IS_UPDATABLE` reads `YES`, and `NO` for the view declared
  `ALGORITHM = TEMPTABLE`; `SECURITY_TYPE` reads `DEFINER` or `INVOKER`; and
  `VIEW_DEFINITION`, to a privileged reader, is the server's rewritten form,
  qualified and back-quoted, as on MariaDB:
  ``select `p`.`port_id` AS `port_id`,… from `freight`.`port` `p` ``. No view
  of the fixture holds a string literal, so whether a view definition carries
  the marks `FR-MYSQL-029` records of an expression was not observed.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The last sentence of
  the requirement is new: the requirement mapped `VIEW_DEFINITION` to
  `definition` and said nothing of the empty string, which `FR-MYSQL-041` left
  to MariaDB's shapes. The *Pending observation* note quoted the manual's
  `CASCADE` and its `SHOW VIEW` requirement, and both are contradicted; the
  two *Observed* notes above replace it, and row 13 of `FR-MYSQL-040`, which
  recorded `CASCADE` as a divergence, is removed.

- **FR-MYSQL-036**: For a `mysql` read, a routine SHALL carry the properties
  `FR-CAT-048` gives it, from the fields of the routine catalogue, `ROUTINES`,
  of the same names, with the same exclusions: the specific name as a
  restatement; the external name, the external language and the SQL path,
  which the model does not carry; and the creation and alteration times as
  volatile, per `FR-CAT-024`. A procedure's return type SHALL be `null` in
  full, per `FR-CTX-005`, whatever its type fields hold. The routine body SHALL
  be `null` WHERE `ROUTINE_DEFINITION` is SQL `NULL`, and the routine is then
  incomplete, per `FR-PRIV-017`.

  *Observed, and the manual contradicted.* The manual states of
  `DTD_IDENTIFIER`: "If the routine is a stored procedure, this value is
  empty" (`refman/8.4/en/information-schema-routines-table.html`). On both
  series a procedure's `DTD_IDENTIFIER` is **SQL `NULL`**, and its
  `DATA_TYPE` the empty string — the two shapes `FR-CAT-048` records side by
  side for MariaDB. The procedure's return type is `null` in full either way,
  per the requirement, so the field was never a divergence that reached the
  model, and row 20 of `FR-MYSQL-040`, which recorded it, is removed.

  *Observed.* The 31 fields, by the names `FR-CAT-048` reads, on both series.
  `ROUTINE_BODY` and `PARAMETER_STYLE` read `SQL`, and `EXTERNAL_NAME` and
  `SQL_PATH` SQL `NULL`, on all seven routines of the fixture.
  `EXTERNAL_LANGUAGE` reads `SQL` on all seven, where MariaDB returned SQL
  `NULL` on every routine; neither is carried. `SQL_DATA_ACCESS` returns
  `CONTAINS SQL` beside the three values `FR-CAT-048` records, because the
  fixture declares a routine of each of the four characteristics; it is
  carried as returned. `ROUTINE_DEFINITION` is the body as written, with no
  introducer and no extra escaping, unlike the expressions of `FR-MYSQL-029`;
  to the reduced-grant reader, which holds `EXECUTE`, it is SQL `NULL` on all
  seven, as the manual states: it "is `NULL` if you have only
  `CREATE ROUTINE`, `ALTER ROUTINE`, or `EXECUTE`". `9.7` declares a third
  routine type, `LIBRARY`, which `FR-MYSQL-023` does not cover.

  *Consequence, stated plainly.* A routine whose language is not SQL, which
  `EXTERNAL_LANGUAGE` admits, is carried with `body_kind` `SQL` and its body,
  and nothing in the model names its language. No routine of the fixture is
  of another language.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The *Pending
  observation* note read, from the manual, that a procedure's
  `DTD_IDENTIFIER` is empty, where MariaDB returns SQL `NULL`. It is SQL
  `NULL`, as on MariaDB, and the two *Observed* notes above replace the note.
  The requirement is unchanged.

- **FR-MYSQL-037**: For a `mysql` read, a routine's parameters SHALL be the
  rows of the parameter catalogue, `PARAMETERS`, carrying the properties
  `FR-CAT-049` gives them from the fields of the same names, in declaration
  order. A function's return value, the row at ordinal position `0` with mode
  and name SQL `NULL`, SHALL NOT be presented as a parameter.

  *Observed.* Sixteen fields, the sixteen `FR-CAT-049` records on MariaDB's
  three oldest series, on both series. Every function has a row at position
  `0` whose `PARAMETER_MODE` and `PARAMETER_NAME` are SQL `NULL` and whose
  type fields carry the return type, such as `varchar` and `varchar(16)`; no
  procedure has one, and a declared parameter starts at `1`. `IN`, `OUT` and
  `INOUT` are reported as declared. The manual states it in the same terms:
  "The ORDINAL_POSITION value is 0. The PARAMETER_NAME and PARAMETER_MODE
  values are NULL" (`refman/8.4/en/information-schema-parameters-table.html`).

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it. The requirement is unchanged.

- **FR-MYSQL-038**: For a `mysql` read, a table's triggers SHALL be the rows
  of the trigger catalogue, `TRIGGERS`, whose event object table is that
  table, carrying the properties `FR-CAT-050` gives them from the fields of
  the same names, with the same exclusions.

  *Observed.* Twenty-two fields, the twenty-two of `FR-CAT-050`, on both
  series. `ACTION_ORDER` reads `1`, and `2` for the trigger the fixture
  declares `FOLLOWS` another of the same event and timing on the same table,
  as the manual's "The ordinal position of the trigger's action within the
  list of triggers on the same table with the same `EVENT_MANIPULATION` and
  `ACTION_TIMING` values" states
  (`refman/8.4/en/information-schema-triggers-table.html`).
  `ACTION_CONDITION` and the two table-reference fields are SQL `NULL`,
  `ACTION_ORIENTATION` is `ROW`, and the row references are `OLD` and `NEW`,
  on every trigger. `ACTION_STATEMENT` is the body as written, with no
  introducer and no extra escaping. The reduced-grant reader, which holds no
  `TRIGGER` privilege, receives **no row** of the table, as the manual's "To
  see information about a table's triggers, you must have the `TRIGGER`
  privilege for the table" states; `FR-MYSQL-041` reaches it.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note is replaced by the *Observed* note above, which confirms
  it. The requirement is unchanged.

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

  *Observed, by enumeration and by refusal*, the two forms the sixth
  validation rule of the [README](README.md#maintenance-debt) admits for a
  record of an absence. By enumeration: no table type exists server-wide
  other than the three of `FR-MYSQL-023`, and the field is declared with
  those three alone; none of the fixture's 37 `data_type` values is `inet4`,
  `inet6` or `uuid`; no check constraint names a `JSON` column; and the field
  lists of `FR-MYSQL-015` hold no algorithm field in `VIEWS` and no level
  field in `CHECK_CONSTRAINTS`. By refusal, each failing on both series with
  `ERROR 1064`, a syntax error and not a silence: a column of type `INET4`,
  `INET6` or `UUID`, `CREATE SEQUENCE`, `WITH SYSTEM VERSIONING`, and
  `PERIOD FOR`, each of which MariaDB accepts.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`: the *Pending
  observation* note, which rested each "never" on the manuals' silence, is
  replaced by the *Observed* note above. The requirement and its table are
  unchanged.

## Divergences between MySQL and MariaDB

- **FR-MYSQL-040**: Every divergence between MySQL and MariaDB that reaches a
  read SHALL be recorded in the following table, naming what this corpus
  records of MariaDB, what was observed of MySQL, and what the divergence
  obliges. A divergence that an observation of MySQL finds and this table
  does not hold is a missing row.

  | # | Subject | MariaDB, as this corpus records it | MySQL, as observed | What it obliges |
  |---|---|---|---|---|
  | 1 | The version string | `<x.y.z>-MariaDB`, with a build suffix, `FR-SRV-040` | `<x.y.z>`, with no suffix and no product marker: `8.4.13`, `9.7.4` | `FR-MYSQL-006`, `FR-MYSQL-007` |
  | 2 | The read-only read-back | `tx_read_only`, present on all four series, `FR-SRV-009` | `tx_read_only` absent, `ERROR 1193`; `transaction_read_only` reads the integer `0`, and `1` once set | `FR-MYSQL-018`, `FR-MYSQL-012` |
  | 3 | A string default | quoted, `'EUR'`, `FR-CTX-037` | unquoted and unescaped: `EUR`, `it's` | `FR-MYSQL-029` |
  | 4 | A default of `NULL` | the four characters `NULL`, `FR-CTX-012` | SQL `NULL`; the string `'NULL'` is the four characters | `FR-MYSQL-029` |
  | 5 | An expression default | told apart by its shape, `FR-CTX-037` | marked `DEFAULT_GENERATED` in the attribute field, `CURRENT_TIMESTAMP` included | `FR-MYSQL-029`, `FR-MYSQL-030` |
  | 6 | A generated column | an is-generated field beside the attribute field, `FR-CAT-051` | the attribute field alone; the expression field is the empty string on every other column | `FR-MYSQL-031` |
  | 7 | The check-constraint table | six fields, carrying the table and the level, `FR-CAT-046` | four fields, carrying neither; the table, and enforcement, in the table-constraint table | `FR-MYSQL-034` |
  | 8 | `JSON` | an alias of `longtext`, with an implicit `json_valid` constraint, `FR-CAT-038` | a native type, `json`, with no character set and no constraint | `FR-MYSQL-028`, `FR-MYSQL-034` |
  | 9 | An invisible column | the attribute field reads `INVISIBLE`, `FR-CAT-035` | the attribute field carries `INVISIBLE`, last, after any other phrase | `FR-MYSQL-030` |
  | 10 | An index on an expression | none; an expression is indexed through a generated column, `FR-CAT-042` | a functional key part, with no column name and an expression | `FR-MYSQL-024` |
  | 11 | An index the optimizer does not use | an ignored field, `FR-CAT-042` | a visibility field of the opposite polarity | `FR-MYSQL-032` |
  | 12 | An integer type's display width | always present, `FR-CTX-038` | absent, save `tinyint(1)` and `ZEROFILL` | `FR-MYSQL-028` |
  | 14 | A view's algorithm | a field of the view catalogue, `FR-CAT-047` | no such field | `null`, `FR-MYSQL-035` |
  | 15 | The schema catalogue's sixth field | a schema comment, `FR-CTX-036` | the default encryption | neither carried, `FR-MYSQL-025` |
  | 17 | The table types | five, and `TEMPORARY` on three series, `FR-CAT-031` | `BASE TABLE`, `VIEW`, `SYSTEM VIEW`; no temporary table listed | `FR-MYSQL-023` |
  | 18 | `ON ... SET DEFAULT` | accepted, reported `RESTRICT`, `FR-CAT-033` | accepted and reported `SET DEFAULT`; the action is refused at run time, `ERROR 1451` | carried unchanged, `FR-MYSQL-033`; `FR-CAT-033` governs a `mariadb` read |
  | 19 | A referential action not written | not recorded by this corpus | `NO ACTION` | `FR-MYSQL-033` |
  | 21 | A routine's external language | SQL `NULL` on every routine, `FR-CAT-048` | `SQL` on every routine | not carried, `FR-MYSQL-036` |
  | 22 | Identifier letter case | not recorded by this corpus | under `lower_case_table_names` `0`, a referenced table's name as declared; a reference in another letter case refused, `ERROR 1824` | names matched byte for byte, `FR-MYSQL-033` |
  | 23 | An expression's text | backtick-quoted and rewritten, no backslash observed, `FR-CAT-051`, `FR-CTX-037` | rewritten as MariaDB's is, with an introducer before each string literal and one extra layer of backslash escaping, in a default, a generation expression, a check clause and a functional key part, and not in a routine or trigger body; a check clause in outer parentheses | the layer removed and the introducers kept in a default, a generation expression and a check clause, `FR-MYSQL-046`, with `FR-MYSQL-029`, `FR-MYSQL-031`, `FR-MYSQL-034`; a functional key part's expression not carried, `FR-MYSQL-032` |
  | 24 | A server with no TLS material | `10.11` offers no TLS until configured, difference 3 of `FR-SRV-038` | generates material at startup where built with OpenSSL, per its manual | `FR-MYSQL-021` |
  | 25 | The default authentication | not recorded by this corpus | `caching_sha2_password` on every account; the server's RSA public key sent only on request | `FR-MYSQL-022`, `FR-MYSQL-045` |
  | 26 | A string literal default on a binary column | not recorded by this corpus | its hexadecimal text, `0x6162`, with no phrase in the attribute field | a `literal` carrying that text, `FR-MYSQL-029` |
  | 27 | The `VECTOR` type | accepted on `11.8` and `12.3`, and no catalogue reading recorded, difference 14 of `FR-SRV-038` | on `9.7` alone: `data_type` `vector`, `column_type` `vector(N)`, and a length of four bytes for each dimension | `FR-MYSQL-028`, difference 1 of `FR-MYSQL-014` |
  | 28 | A JSON duality view | none | on `9.7` alone, a row of the table catalogue of type `VIEW` and a row of the view catalogue | a covered view, `FR-MYSQL-023` |
  | 29 | A generated invisible primary key | none | a column `my_row_id`, invisible and auto-incremental, and the key `PRIMARY` on it, or neither, by a session setting | carried as reported, `FR-MYSQL-027` |
  | 30 | What a reduced-grant reader loses of the constraint tables | every row of `TABLE_CONSTRAINTS` and `REFERENTIAL_CONSTRAINTS`, `FR-PRIV-018` | nothing: every row of the database | `FR-MYSQL-041`: the cross-check of `FR-PRIV-019` has nothing to detect over the fixture |
  | 31 | The spelling of the `ON UPDATE` attribute | `on update current_timestamp()` and `on update current_timestamp(3)`, lower case, the whole of the column attribute field, `FR-CAT-041` | `on update CURRENT_TIMESTAMP` and `on update CURRENT_TIMESTAMP(3)`, the expression upper case, after `DEFAULT_GENERATED` in the attribute field | the phrase carried as each server spells it, in one shape for both, `FR-MYSQL-030` |

  **The rows are numbered as they were first written, and a number is not
  reused.** Rows 13, 16 and 20 recorded divergences the documentation
  described and the servers did not show, and are removed: a view's cascaded
  check option reads `CASCADED`, as on MariaDB, per `FR-MYSQL-035`; a spatial
  index reports `SPATIAL`, as on MariaDB, per `FR-MYSQL-032`; and a
  procedure's DTD identifier is SQL `NULL`, as on MariaDB, per `FR-MYSQL-036`.
  Rows 26 to 30 are divergences the observation found and the table did not
  hold. Row 31 is one `FR-MYSQL-030` recorded and the table did not hold,
  found when the provider was implemented.

  *Observed.* Every cell of the MySQL column is an observation of the fixture,
  recorded in the requirement the last column names, save the cell of row 24.
  Every cell of the MariaDB column is an observation this corpus records, or
  says that it records none.

  *Pending observation.* The MySQL cell of row 24 is the manual's statement
  that a server built with OpenSSL "can generate missing certificate and key
  files automatically at startup"
  (`refman/8.4/en/using-encrypted-connections.html`), and is pending
  observation in `FR-MYSQL-021`, for the reason that note gives: every server
  of the fixture is given its TLS material.

  *Four candidates read, and not recorded.* The observation named four more
  differences as rows this table lacked, and none is one. A view's definition
  read by a reader without `SHOW VIEW` is the empty string on MySQL, which is
  what `FR-PRIV-018` records of MariaDB. A comment stores a character outside
  the basic multilingual plane as `?` on MySQL, as on MariaDB per
  `FR-CAT-036`; MySQL warns at DDL time where MariaDB does not, and the
  warning reaches no read, per `FR-MYSQL-026`. A check clause's escaping is
  row 23. And the empty generation expression of a column that is not
  generated reaches no property, per `FR-MYSQL-031`, which row 6 now states.

  *Rationale.* `FR-SRV-038` keeps a record of every difference between the
  MariaDB series because a difference without a home is rediscovered or
  contradicted. Two products are further apart than two series, and the
  coordinator asked for the divergences to be stated: default quoting, the
  check-constraint table, `JSON`, invisible columns, functional indexes, the
  version string and identifier case are rows 3 to 5, 7 to 10, 1 and 22.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`: the last cell of
  row 25 named `OQ-077`, which `FR-MYSQL-045` closes.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The requirement
  named what MySQL's documentation states, and now names what was observed;
  the column of the table is renamed with it. Every cell of that column is
  rewritten as observed, and rows 13, 16 and 20 are removed, because the
  servers contradicted the documentation each rested on and agreed with
  MariaDB. Row 18 is rewritten, because the servers contradicted the
  documentation and do not agree with MariaDB: `SET DEFAULT` is accepted and
  reported. Rows 26 to 30 are added. The *Pending observation* note, which
  covered every MySQL cell, now covers the cell of row 24 alone. The last
  cell of row 23 names `FR-MYSQL-046`, by the user's decision on `OQ-078`,
  which the observation raised and the same edition closed.

  *Amended in the seventy-seventh edition,* for a follow-up of rmp `#329`:
  row 31 is added. `FR-MYSQL-030` has recorded since the seventy-fifth
  edition that MySQL spells the `ON UPDATE` attribute in upper case among the
  other phrases of the attribute field, where MariaDB spells it in lower case
  as the whole field, and the seventy-sixth fixed the value carried as each
  server's phrase. A divergence the table does not hold is a missing row, by
  the requirement's own text. No other row changes.

## Completeness

- **FR-MYSQL-041**: The rules of
  [privileges-and-completeness.md](privileges-and-completeness.md) SHALL hold
  for a `mysql` read as for a `mariadb` read: a read is complete or
  incomplete under `FR-PRIV-001` and `FR-PRIV-002`, an incomplete named object
  is `77` under `FR-PRIV-003`, an incomplete object in a listing or a dump is
  marked under `FR-PRIV-005` through `FR-PRIV-007`, and the limit of
  `FR-PRIV-020` holds for triggers. The cross-checks of `FR-PRIV-011`,
  `FR-PRIV-017` and `FR-PRIV-019` SHALL be applied to a `mysql` read in the
  shapes of the following table, which are the shapes `FR-PRIV-018` records
  for MariaDB wherever MySQL gives an absence at all:

  | Property | Read as a privileged reader | Read as the reduced-grant reader | Detectable |
  |---|---|---|---|
  | A view's definition | the full text | the **empty string**, length 0, on a row that is present | yes, `FR-PRIV-011`, per `FR-MYSQL-035` |
  | A routine's body | the full body | **SQL `NULL`**, on a row that is present | yes, `FR-PRIV-017`, per `FR-MYSQL-036` |
  | A table's referential rules, and the constraint table beside them | every row of the database | every row of the database: nothing is lost | nothing to detect; `FR-PRIV-019` is still applied |
  | A table's `CHECK` constraints | every row | every row, each with its row in the constraint table | nothing to detect |
  | A table's triggers | 7 rows for the fixture | **zero rows** | **no**, `FR-PRIV-020` |

  *Observed.* The fixture's reduced-grant reader holds `SELECT` and
  `EXECUTE` on `freight` and nothing more, on both series. It sees every
  table and view of `freight`, and all five rows of the view catalogue with
  every field populated but `VIEW_DEFINITION`, which is the empty string; all
  seven rows of the routine catalogue, with `ROUTINE_DEFINITION` SQL `NULL`;
  no row of the trigger catalogue, which is indistinguishable from a table
  with no trigger; and every other catalogue complete for `freight`,
  `REFERENTIAL_CONSTRAINTS` and `TABLE_CONSTRAINTS` among them, with nothing
  of the second schema, `masterdata`, on which it holds no grant. The manual
  states the routine shape and the trigger privilege, as `FR-MYSQL-036` and
  `FR-MYSQL-038` quote it, and states of the view catalogue that "You must
  have the SHOW VIEW privilege to access this table", which the reader
  contradicts, per `FR-MYSQL-035`.

  *Where MySQL differs from MariaDB.* A reduced-grant reader of a MariaDB
  server loses every row of `REFERENTIAL_CONSTRAINTS` and `TABLE_CONSTRAINTS`,
  per `FR-PRIV-018`, which is the loss `FR-PRIV-019` exists to detect; the
  same reader of a MySQL server loses neither, so the cross-check finds
  nothing over the fixture and costs nothing to keep. It stays applied,
  because a reader with another set of grants may lose them, and
  `FR-PRIV-012` requires every cross-check on every read that presents the
  property it guards. It is row 30 of `FR-MYSQL-040`.

  *Bounded claim.* One reduced grant was observed, the one item 16 of
  `FR-MYSQL-044` fixes, and a scratch reader holding no privilege on one
  table, which saw that table in no catalogue at all: an absent table, which
  `FR-PRIV-015` gives no cross-check, and not a short read of one. What a
  reader with any other set of grants receives was not observed.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fifth edition,* for rmp `#326`. The requirement
  applied the cross-checks in MariaDB's shapes until an observation recorded
  MySQL's, and said this corpus would not claim they detect every such
  absence of a `mysql` read. The shapes are recorded in the table above, so
  the requirement now applies the cross-checks in them. They are MariaDB's
  shapes for a view and a routine, which is why the detection was already
  correct for both, and MySQL loses neither constraint table, which MariaDB
  does. The *Pending observation* note is replaced by the notes above.

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
  | `FR-SRV-029` | The equivalence of `FR-SRV-026` across the series of `FR-MYSQL-003`, and the refusal of `FR-MYSQL-009` against at least one series outside the set and older than its newest series |
  | `FR-SRV-035` | The marked read of `FR-MYSQL-010`, through the seam it authorises |
  | `BR-SRV-003` | That the first three are observations made outside the process, with the one exception it names |

  The product refusal of `FR-MYSQL-008` SHALL be verified by an integration
  test against a MariaDB server of the fixture of `scripts/mariadb/`, of a
  series whose read-back a `mysql` entry passes, per `FR-MYSQL-012`.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended: the `FR-SRV-029` row counted three series, and
  named any series outside the set for the refusal. Every series outside the
  set was older than its newest while `26.10` was in it; an Innovation release
  numbered by the calendar is now newer, and is read under `FR-MYSQL-010`
  rather than refused.

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
  and SHALL exercise the following cases:

  1. tables of the InnoDB engine and of at least one other engine;
  2. every native data type every series accepts: the integer types signed
     and unsigned, one with `ZEROFILL` and one `tinyint(1)`; `decimal`,
     `float`, `double` and `bit`; `date`, `year`, and `datetime`, `timestamp`
     and `time` with and without fractional seconds; the character, text,
     binary and blob types; `enum` with members holding a comma and an
     apostrophe, and `set`; `json`; and the geometry types, one with an
     `SRID`. `VECTOR` is not among them, because `8.4` refuses it, per
     difference 1 of `FR-MYSQL-014`;
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
  13. comments on tables, columns, indexes and routines, one written with a
      character outside the basic multilingual plane, which every series
      stores and returns as `?`, per `FR-MYSQL-026`;
  14. identifiers carrying a space, a backtick and a double quote;
  15. a scheduled event and a partitioned table, which the read excludes; and
  16. a reduced-grant reader holding `SELECT` and `EXECUTE` on the main
      schema and nothing more, and a schema declaring no collation of its own.

  A structure or a statement of `setup.sql` or `seed.sql` that a series of
  `FR-MYSQL-003` refuses SHALL be omitted from them rather than made
  conditional, and the refusal SHALL be recorded under `FR-MYSQL-014`, as
  `FR-SRV-029` records difference 14 for MariaDB. Differences 1 and 6 of
  `FR-MYSQL-014` are the two such refusals recorded.

  The server of each series of `FR-MYSQL-003` SHALL present a certificate
  that names the host by which the project's tests reach it. Beside those
  servers, the fixture SHALL present:

  - at each series of `FR-MYSQL-003`, **a server that offers no TLS**, staged
    by configuration as the second item of `FR-MYSQL-021` describes, so that
    the right-hand column of that requirement and the logins of
    `FR-MYSQL-045` are observable on every series; and
  - **one server of a MySQL series outside `FR-MYSQL-003` and older than its
    newest series**, for the refusal of `FR-MYSQL-009`, presenting a
    certificate that names the host by which the tests reach it, so that a
    `mysql` entry reaches the version check under the default `tls` mode of
    `FR-CONF-013` instead of ending at the TLS handshake.

  The condition each server runs under that a reading depends on — its TLS
  material, its `lower_case_table_names`, its
  `show_gipk_in_create_table_and_information_schema`, and its
  `sql_generate_invisible_primary_key` — SHALL be recorded with every reading
  taken against it, per the fourth validation rule of the
  [README](README.md#maintenance-debt).

  The release each server runs within its series, which series stands
  outside the set, the ports the servers publish, and where their data
  directories are kept are the fixture's choices, recorded in
  `scripts/mysql/README.md`, and are not requirements of this corpus. *On
  2026-11-04* the fixture runs the release the table of `FR-MYSQL-003` names
  as the latest of each series, with and without TLS; its series outside the
  set is `8.0`, at `8.0.46`, which is the series the example of
  `FR-MYSQL-009` refuses; it publishes the five servers on the ports `13320`
  to `13324`; and it keeps each data directory in memory, rebuilt from
  `setup.sql` and `seed.sql` at
  every start.

  *Why those four choices are not requirements.* No requirement of this file
  depends on any of them: a test reaches a server at the address the harness
  reports, a data directory is rebuilt from the same versioned files wherever
  it is kept, and any series outside the set and older than its newest
  exercises the same refusal. Writing them here would make a second copy of
  values `scripts/mysql/` keeps in one place that its scripts read, and a
  copy that drifts without a sign. The date names the reading, so that the
  sentence above is a record of the fixture on that day and not a statement
  that ages.

  *What the list does not exercise.* Two cases this file states were observed
  on scratch objects and are not among the sixteen: a key declared
  `SET DEFAULT`, which `FR-MYSQL-033` carries, and a generated invisible
  primary key, which `FR-MYSQL-027` carries. A test of either needs DDL the
  shared files do not hold. Neither is refused by a series, so either could
  join the list.

  *Rationale.* It is what `FR-CONF-038` and the root `CLAUDE.md` require of
  `scripts/mariadb/`, and `FR-LITE-038` of `scripts/sqlite/`, written for the
  catalogue a MySQL server reads. rmp `#326` took every observation of this
  file through it, and each statement still marked *Pending observation*
  names what it cannot stage.

  *Added in the seventy-third edition,* for rmp `#324`.

  *Amended in the seventy-fourth edition,* for rmp `#331`, under
  `FR-MYSQL-002` as amended: the server outside the set must be older than its
  newest series, for the reason the amendment to `FR-MYSQL-043` gives. The
  servers of the set are those of `FR-MYSQL-003`, which on 2026-11-04 holds
  two series where it held three.

  *Amended in the seventy-fifth edition,* for rmp `#326`, to state what the
  fixture built for rmp `#325` is. The list read "every case this file
  states", which now includes two cases the fixture does not hold, and names
  its cases instead. Item 2 says that `VECTOR` is not among the types, and
  item 13 that the character is returned as `?`. The rule on refusals reaches
  a statement of `seed.sql`, which difference 6 of `FR-MYSQL-014` is. The
  servers beside those of the set are listed: the requirement read that the
  fixture retain *a* server that offers no TLS, and it now presents one at
  each series, which `FR-MYSQL-021` and `FR-MYSQL-045` need on every series;
  and the server outside the set, which read *at least one*, is one, and
  presents a certificate naming the host. The conditions recorded gain
  `sql_generate_invisible_primary_key`, which decides whether a table created
  without a primary key gains a generated one. The paragraph on the fixture's
  own choices and the notes after it are new, and the rationale no longer
  says that the observations are still to be taken.

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
  of an expression index `FR-MYSQL-024` shares; `FR-LITE-027`, whose
  reading of two fields `FR-MYSQL-029` shares; and `FR-LITE-031`, under which
  a `sqlite` read already carries the `SET DEFAULT` that `FR-MYSQL-033`
  carries.
- [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) — the model and the document
  this file fills; `FR-CTX-043`, the `provider` field; `FR-CAT-033`, which
  governs a `mariadb` read and does not reach a `mysql` read, per
  `FR-MYSQL-033`; and `FR-CAT-036`, which holds for a `mysql` read, per
  `FR-MYSQL-026`.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the
  rules `FR-MYSQL-041` applies, and the cross-check of `FR-PRIV-011`, which
  `FR-MYSQL-035` applies to a view.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `77` and
  `78`; the condition order of `FR-ERR-006`; and the message rules
  `FR-MYSQL-008`, `FR-MYSQL-009` and `FR-MYSQL-045` follow, with the `hint`
  rules `BR-ERR-004` and `BR-ERR-005` that `FR-MYSQL-045` applies.
- [performance-requirements.md](performance-requirements.md) —
  `NFR-PERF-001` to `NFR-PERF-008`, which `FR-MYSQL-042` applies.
- [template-environment.md](template-environment.md) — `FR-ENV-046`, which
  places every `data_type` value a `mysql` read shares with a `mariadb` read,
  and `FR-ENV-049`, which places the three only a `mysql` read was observed
  to produce, per `FR-MYSQL-028`.
- [security.md](security.md) — `BR-SEC-002`, the promise `BR-MYSQL-002`
  states for a `mysql` entry.
- [open-questions.md](open-questions.md) — `OQ-076`, `OQ-077` and `OQ-078`,
  which this file raised and which `FR-MYSQL-002`, `FR-MYSQL-045` and
  `FR-MYSQL-046` close.

## Open questions

**None.** The three entries this file raised are closed by the user's
decisions: `OQ-076` by `FR-MYSQL-002` as amended, `OQ-077` by
`FR-MYSQL-045`, and `OQ-078` by `FR-MYSQL-046`. All three are listed under
[Closed](open-questions.md#closed), and the index of
[open-questions.md](open-questions.md) holds no entry for this file.

*Amended in the seventy-fourth edition,* for rmp `#331`. This section held a
table of the two entries, open in the index, with what each decided and what
waited on it.

*Amended in the seventy-fifth edition,* for rmp `#326`. The section named two
entries. `OQ-078`, which the observation of rmp `#326` raised, was closed in
the same edition by the user's decision, and is the third.

The statements marked *Pending observation* are not open questions. Each
names what this fixture cannot stage and what would observe it, and the
[README](README.md#maintenance-debt) records them as the one item of
maintenance debt the corpus carries.
