---
title: SQLite Contract
status: approved
last-reviewed: 2026-11-06
related: [configuration-model.md, server-contract.md, catalogue-coverage.md, context-document.md, privileges-and-completeness.md, security.md, cfg-commands.md, performance-requirements.md, errors-and-exit-codes.md, global-flags.md, project-and-discovery.md]
---

# SQLite Contract

## Overview

`tpl` reads a database through a **provider**, named by the `provider` key of
the entry, per `FR-CONF-051`. [server-contract.md](server-contract.md) states
what `tpl` requires of a MariaDB server and promises in return. This file states
the same for the `sqlite` provider: the library that reads the file, how the
driver that reaches it is chosen, how the file is opened and refused, the
hardening applied to a file `tpl` treats as untrusted, the closed list of
statements, the part of the database a read covers, and how the SQLite
catalogue fills the model of [catalogue-coverage.md](catalogue-coverage.md) and
the document of [context-document.md](context-document.md).

The model is the same model. A SQLite read fills the objects and properties
those two files define and no others, and where SQLite reports nothing for a
property, this file says so. A template written against the model renders from
a SQLite read on the same principles as from a MariaDB read, and
`database.provider`, per `FR-CTX-043`, is how a template tells the two apart.

## Scope

In scope: the SQLite library and the `server` object it yields, the selection
of the driver, the opening of the database file and the outcomes when it
cannot be read, the hardening controls and what happens when one cannot be
applied, the closed statement list and the order of the steps that open the
file, the
schema a read covers and what it excludes, the source of every model property
for a SQLite read, the completeness of a SQLite read, its requirements of form,
and the fixture its tests use.

Out of scope: the `provider` and `path` keys and their validation, which belong
to [configuration-model.md](configuration-model.md); the commands that write
them, which belong to [cfg-commands.md](cfg-commands.md); the shape of the
document, which belongs to [context-document.md](context-document.md); the
choice of the driver and of the library release, which are architecture
decisions recorded in the project's decision records, under the rules of
`FR-LITE-004` and `FR-LITE-039` and per `FR-LITE-001`; and the text of
any statement beyond the names this file must give to make its list closed.

Everything [server-contract.md](server-contract.md) states about a server —
the supported version window, the refusal of a server that is not MariaDB, the
differences between series, the read-only session statement and its read-back,
TLS and the privilege model — governs a `mariadb` entry and does not reach a
`sqlite` entry. Where this file adopts one of those rules for SQLite, it says
so and cites it.

## Actors

- **SQLite library**, linked into the binary, which reads the database file.
- **Database file**, named by the entry's `path`, and untrusted input.
- **`tpl`**, which must satisfy the statement restriction and the hardening.
- **Calling agent**, reading the model through a command or a template.

## Observations

Every statement this file makes about SQLite's behaviour rests on one of two
grounds: SQLite's official documentation, cited by page, and the observation
recorded here.

*Observed on 2026-11-03*, against two releases of the library on
`aarch64-apple-darwin`: **3.54.0**, the `sqlite3` shell at `/usr/bin/sqlite3`
(source identifier `2026-04-09 12:25:13 8fa8248e…`), and **3.54.1**, the
library the host's Python carries. Each reading was taken over a throwaway
database built from DDL written for the observation in a freight domain: eight
ordinary tables (one `WITHOUT ROWID`, one `STRICT`), five foreign keys of every
referential action, a composite primary key, unique, partial, descending,
collated and expression indexes, two generated columns, three views (one
referencing a missing table), four triggers (one on a view), an `fts5` virtual
table with its shadow tables, `sqlite_sequence` and `sqlite_stat1`, and
identifiers carrying a space, a backtick and a double quote. Further databases
were built for single questions: default-value forms, rowid aliases, unsafe
functions, a module the reading library lacks, a 234 MB file, write-ahead-log
mode, and files that are not databases.

*Conditions of the observation.* The DDL is not kept in this repository: the
fixture of `FR-LITE-038` is what makes each reading repeatable, and a reading
is re-taken against it when it exists. Neither release observed is the release
the binary carries, which `FR-LITE-001` leaves to the project's decision
records. A reading here is therefore a record of what two releases of the
library returned, falsifiable by the fixture, in the sense of the fourth
provenance of the [README](README.md#provenance).

*Observed again on 2026-11-03, for the sixty-sixth edition,* against two
builds of the library as a Rust driver bundles it: **3.51.3** and **3.54.1**,
each compiled from the library's source and linked into a test program written
without `unsafe` code, on `aarch64-apple-darwin`. Both releases were built
with the options the driver sets and with the extension-loading mechanism
omitted (`SQLITE_OMIT_LOAD_EXTENSION`), and 3.54.1 also without FTS5. The
databases read were one built from DDL written for the driver evaluation of
rmp `#315`, in the same freight domain and holding the kinds of object listed
above, and further databases built for single questions. Wherever both
releases were read the same way, the readings were identical save the version
string. Which of the two the binary carries is decided in the project's
decision records, under `FR-LITE-039`; these readings are records of what the
two builds returned, in the same sense as those above.

*Observed again on 2026-11-03, for the sixty-ninth edition,* for the security
review of rmp `#317`, on `aarch64-apple-darwin`, in two ways: through the
Python library **3.54.1**, the release of the library the binary carries, with
the file opened read-only under every control of `FR-LITE-010` and every limit of
`FR-LITE-011` that the Python library can set; and through the release build
of `tpl` at `cb99236`. The databases read were built for single questions,
outside this repository, and the main fixture database of `FR-LITE-038` was
built from `scripts/sqlite/freight.sql` for the comparisons that needed it.
Where a reading states what the library does inside a statement, it rests on
the library's source as well: the single-file source of 3.54.1 that the
published package of the driver selected under `FR-LITE-004` bundles, read at
that public source on the same date.

*Observed again on 2026-11-04, for the seventieth edition,* for rmp `#317`,
on `aarch64-apple-darwin`, through the Python library **3.54.1**, with
the file opened read-only under every control of `FR-LITE-010`, views disabled
included, and every limit of `FR-LITE-011` that the Python library can set,
with the statements of `FR-LITE-015` as the sixty-ninth edition writes them.
The database read was the main fixture database of `FR-LITE-038`, built from
`scripts/sqlite/freight.sql` outside this repository, once in
rollback-journal mode and once in write-ahead-log mode, and an empty file of
zero bytes was read beside it; a second, ordinary read-write open of
the same file was the writer. The database of one
table and 320 000 views that `FR-LITE-009` records served the reading of the
deadline.

*Observed again on 2026-11-04, for the seventy-first edition,* for rmp
`#317`, on `aarch64-apple-darwin`, through the Python library **3.54.1**, with
the file opened read-only under every control of `FR-LITE-010`, views disabled
included, and every limit of `FR-LITE-011` that the Python library can set,
with the statements of `FR-LITE-015` as the seventieth edition writes them.
The databases read were built for single questions, outside this repository:
each held a table, an index, a view and a trigger in the freight domain, and
had the `type` of one row of the schema table rewritten under
`writable_schema` through an open of the file whose defensive mode was off. Whether the
object of that row still worked in the library was read on a copy, through an
ordinary read-write open of the file with views enabled. The main fixture database
of `FR-LITE-038`, and the databases `internal.sql` and `fts5_view.sql` build,
were built from `scripts/sqlite/` outside this repository for the comparisons
that needed them. Further databases were built for two questions: which rows
SQLite writes with no text, over a database holding a text primary key,
single and composite `UNIQUE` constraints, a `WITHOUT ROWID` table with a
`UNIQUE` column, `AUTOINCREMENT`, a `STRICT` table with a generated column,
`fts5`, `fts4` and `rtree` tables, and the statistics `ANALYZE` writes; and
how the catalogue reports a name one object writes for another in a
different letter case, where how SQLite resolved such a name was read by
inserting rows with foreign keys enforced, through an ordinary read-write
open of the file. A final sweep took each column of the schema table — `type`,
`name`, `tbl_name`, `rootpage` and `sql` — and rewrote it, one shape to a
database, under `writable_schema` as above; and it read, over databases
SQLite wrote, the values the catalogue statements take from the pragma
functions — a column named `rowid`, a table and a column with an empty
name, a declared type holding `HIDDEN`, an index repeating a column,
foreign keys to a view, a virtual table, an internal table, a `rowid` and a
missing column, foreign keys naming no parent column of either arity, and
trigger headers SQLite stored from statements written in every letter case,
with comments, quoted names, keyword names and a schema qualifier. For each
shape that loaded, it asked what the statements of the seventieth edition
would present.

## The library

- **FR-LITE-001**: The binary SHALL carry its own SQLite library, linked at
  build time, and SHALL read every `sqlite` entry through it. The system SHALL
  NOT load a SQLite library from the host, and SHALL NOT load a library
  extension.

  *Rationale.* The version a read reports is then a property of the binary and
  not of the machine that runs it, which is what `FR-LITE-003` rests on, and
  the statically linked Linux targets of `NFR-PERF-018` carry no host library
  to load in any case. Which release is carried is an architecture decision,
  recorded in the project's decision records and cited from there, as
  `FR-ENV-003` does for the template engine's pin; the number never enters
  this corpus.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the sixty-sixth edition.* Which release is carried is chosen
  under `FR-LITE-039`.

- **FR-LITE-002**: For a `sqlite` entry, the `server` object of `FR-CTX-031`
  SHALL describe the library: `version` SHALL be the string the version probe
  `SELECT sqlite_version()` returns, unaltered; `series` SHALL be derived from
  it as `<major>.<minor>` and from nothing else; and `standing` SHALL be
  `supported`.

  ```json
  {"server":{"version":"3.54.1","series":"3.54","standing":"supported"}}
  ```

  *Observed.* The probe returned `3.54.0` from the shell and `3.54.1` from the
  Python library: three dot-separated integers, no prefix and no suffix. It
  reads no table, and it is nonetheless the first statement that reads the
  file, because preparing it loads the schema: over a text file it failed with
  `file is not a database` and over a truncated file with
  `database disk image is malformed`, while the two pragma statements issued
  before it on the same opens of the file succeeded. `FR-LITE-017` places it for
  that reason.

  *Why the object keeps its name.* `server` is a structural key of the
  document, present in every document, per `FR-CTX-031` and `BR-SRV-008`. A
  `sqlite` document without it would fail the contract `FR-RND-020` checks,
  and a renamed key would be a breaking change under `FR-OUT-014` for every
  MariaDB document. The object describes what read the catalogue, and for a
  `sqlite` entry that is the library.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The observation says that
  the two pragma statements succeeded on the same opens of the file, where it
  said connections. The requirement is unchanged.

- **FR-LITE-003**: The supported version window of `FR-SRV-001`, the refusal of
  `FR-SRV-020`, the marked read of `FR-SRV-031` and the product check of
  `FR-SRV-003` SHALL NOT apply to a `sqlite` entry. The library the binary
  carries is the one supported release, and `standing` SHALL be `supported`
  on every `sqlite` read. The carried library SHALL provide every pragma
  table-valued function `FR-LITE-015` names, and SHALL answer
  `pragma_index_info` for the name of a `WITHOUT ROWID` table with that
  table's primary-key columns, which `FR-LITE-020` reads. Per the official
  documentation at `https://sqlite.org/pragma.html`, checked 2026-11-03, the
  pragma table-valued functions first appeared in SQLite 3.16.0, and that
  answer of `PRAGMA index_info` in 3.30.0.

  *Rationale.* The window exists because the server is a separate program
  whose version moves without `tpl`: a series below it is one the project has
  never observed, and one above it one the project has not observed yet. A
  library linked into the binary moves only with the binary, and the binary is
  verified against the fixture of `FR-LITE-038` with the library it carries.
  There is nothing to refuse at run time and nothing to mark.

  *Rejected: a window of SQLite releases.* No invocation can meet a release
  outside it, so the refusal would be unreachable and its test unwritable.
  *Rejected: `standing` as `null`, or a third value such as `bundled`.*
  `BR-SRV-008` makes the field permanent and never `null`, and a third value
  would make the guard `standing != "supported"` of `BR-SRV-009` fire on every
  `sqlite` read, where nothing is unverified.

  *Consequence, stated plainly.* A database file written by a newer SQLite
  can use a schema feature the carried library does not parse. Such a file
  fails to load and is refused with `78` under `FR-LITE-007`; no newer library
  was available to observe it, and this file claims nothing about which
  features those are.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`: the floor names
  `pragma_index_info` on a `WITHOUT ROWID` table, which `FR-LITE-020` now
  reads, in place of `pragma_table_list`, which no statement invokes, per the
  user's decision recorded under `FR-LITE-020`.

- **FR-LITE-039**: When each release of `tpl` is built, the SQLite library the
  binary carries, per `FR-LITE-001`, SHALL be the most recent SQLite release
  obtainable through the driver selected under `FR-LITE-004`, and SHALL be
  secure. The terms are fixed as follows:

  1. *Most recent* SHALL be judged against the release history SQLite
     publishes at `https://sqlite.org/changes.html`.
  2. A release is *obtainable* WHEN a release of the selected driver carries
     it and that driver release satisfies every step of `FR-LITE-004`.
  3. The carried release SHALL NOT be affected by any vulnerability that
     `https://sqlite.org/cves.html` lists as fixed in a later release and that
     is reachable under the controls of `FR-LITE-010`, the limits of
     `FR-LITE-011` and the statements of `FR-LITE-015`.
  4. A release older than the most recent one SHALL be carried only WHERE the
     newer release is not obtainable, or WHERE it carries such a reachable
     vulnerability and the older release does not.

  The project's decision records SHALL record, beside the release carried,
  the judgement of clause 3 for every vulnerability listed with a fix later
  than that release, and, WHERE clause 4 applies, the reason the newer release
  is not carried. The choice SHALL be re-verified against both pages before
  every release of `tpl`.

  This requirement SHALL NOT override `FR-LITE-004`: it chooses among the
  releases of the driver that rule selects and never among drivers, a driver
  that a step of that rule disqualifies is not made eligible by the library it
  carries, and the order of the rule stands. It governs the `sqlite` provider
  alone, and nothing a `mariadb` entry does changes.

  *Provenance.* The user's decision of 2026-11-03, relayed for rmp `#319`:
  the library must be the most recent release possible, and secure. The two
  pages are a published authority in the sense of the third provenance of the
  [README](README.md#provenance), both checked 2026-11-03: the release history
  states no date of its own, and the vulnerability page was last updated
  2026-08-01. What they say changes with every SQLite release, so this
  requirement names no release, and re-verifying the choice against them is
  the obligation, in the form `FR-SRV-019` gives the supported-series table.
  The release chosen is recorded in the decision records, per `FR-LITE-001`,
  and the number never enters this corpus.

  *What reachable includes.* The schema of the file is SQL its author wrote.
  The library parses every `CREATE` statement when it loads the schema. No
  statement of `FR-LITE-015` compiles a view or connects a virtual table, per
  that requirement, so no module's code runs during a read, per the
  observation under `FR-LITE-018`. A fault that needs an attacker's SQL is
  therefore reachable wherever parsing that SQL reaches it, although `tpl`
  itself issues only the statements of `FR-LITE-015`.

  *Why reachability and not presence.* The vulnerability page states, for
  most of its entries, the conditions a fault needs — SQL the attacker
  supplies, defensive mode off, an optional feature built in, or the
  command-line shell alone — and a `sqlite` read meets only some of them:
  defensive mode is on, per `FR-LITE-010`, and `tpl` runs no shell. A rule on
  presence would refuse a release for a fault no `sqlite` read can reach; a
  rule on reachability needs a judgement, so the judgement is recorded where
  it can be checked.

  *Rejected: naming a release in this corpus.* The number would decay with the
  next SQLite release and nothing here would show it. *Rejected: the most
  recent SQLite release, whatever the selected driver carries.* It can only be
  met by carrying a library the driver was not released with, which is a
  modified driver.

  *Consequence, stated plainly.* WHERE no release obtainable through the
  selected driver is free of a vulnerability that clause 3 names, no carried
  library satisfies this requirement, and a release of `tpl` built in that
  state does not conform to it. The remedy lies outside this requirement: a
  driver release that carries the fix, or a new evaluation under
  `FR-LITE-004`.

  *Added in the sixty-sixth edition,* for rmp `#319`.

  *Amended in the sixty-eighth edition,* for rmp `#317`: the note on what
  reachable includes said that the library compiles a view whenever its
  columns are read. No statement reads them now, per `FR-LITE-033`, so the
  note names what the read still reaches: the parse of the schema, and the
  module of a virtual table the read connects. The requirement is unchanged.

  *Amended in the sixty-ninth edition,* for rmp `#317`: the note on what
  reachable includes no longer names the module of a virtual table the read
  connects. No statement invokes `pragma_table_list` any more, whose listing
  of the objects compiled views and connected virtual tables inside the
  library, per `FR-LITE-015` and the user's decision recorded under
  `FR-LITE-020`, so what a read reaches narrows to the parse of the schema.
  The requirement is unchanged.

## Selecting the driver

- **FR-LITE-004**: The driver through which `tpl` reaches the SQLite library
  SHALL be selected by the following rule, applied in order, and by no other.
  A candidate that fails a step is disqualified, and the rule SHALL NOT be
  satisfied by removing a hardening control from `FR-LITE-010` or
  `FR-LITE-011`:

  | Step | Selected or disqualified |
  |---|---|
  | 0. Official driver | WHERE a Rust driver published by the SQLite project itself — the SQLite developers, at sqlite.org — exists, AND it can apply every hardening control of `FR-LITE-010` and `FR-LITE-011` through an interface that requires no `unsafe` code in `tpl`, it SHALL be selected. An official driver that cannot apply one of them is disqualified like any other candidate |
  | 1. Security | Otherwise, a candidate SHALL be able to apply every hardening control of `FR-LITE-010` and `FR-LITE-011`, and confirm each by reading it back, through an interface that requires no `unsafe` code in `tpl`. A candidate that cannot is disqualified |
  | 2. Catalogue visibility | Among the candidates that remain, the one selected SHALL return every catalogue fact this file reads, through the statements of `FR-LITE-015`, with SQLite's own dynamic value types preserved — no pragma value coerced to another type, and a SQL `NULL` kept distinct from every value — and SHALL fail no read this specification requires |

  WHERE more than one candidate remains after step 2, the choice among them is
  an architecture decision, recorded in the project's decision records, and it
  changes no requirement of this corpus. The specification names no driver.

  *Provenance.* The rule is the user's, decided on 2026-11-03 and relayed for
  rmp `#314`, in three decisions: that the driver must be the most secure and
  offer the best visibility over the catalogue; that an official driver, if
  one exists, must be chosen; and that security disqualifies even an official
  driver. *Official* is the user's definition: a driver published by the
  SQLite developers. A community driver is not official, whatever its name,
  and whether or not it carries the official SQLite C source. Whether an
  official Rust driver exists is a fact the evaluation that applies this rule
  establishes on the date it is made, and this corpus does not record it. The
  no-`unsafe` clause is the rule the root `CLAUDE.md` states for the whole
  crate, cited here because step 0 and step 1 turn on it. It is recorded as a
  requirement, in the form `FR-CONF-036` uses for the MariaDB driver, so that
  the criterion binds whoever runs the evaluation rather than being a
  preference held by them.

  *Why step 2 names value types.* `FR-LITE-027` classifies a default by the
  shape of a text value and treats SQL `NULL` as a different case, and
  `FR-LITE-025` reads integers from three pragma columns. A driver that
  returned every value as a string, or a `NULL` as an empty string, would
  produce a model `FR-CAT-056` forbids, at exit `0`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## The database file

- **FR-LITE-005**: The path the entry names, resolved per `FR-CONF-054`, SHALL
  name a regular file. The system SHALL establish the kind of the file by
  following symbolic links and without waiting for a writer, before the
  library opens it. IF it is anything else — a directory, a FIFO, a socket, or
  a character or block device — THEN the system SHALL exit `78` (`EX_CONFIG`)
  and SHALL NOT open it. The `cause` SHALL name the path as resolved, the kind
  of file found, and that a SQLite database is read only from a regular file.
  The `hint` SHALL carry `tpl cfg database update <entry> --path <path>`. IF
  the kind cannot be established because the lookup of the path fails — the
  path names nothing, a link on it dangles, or the filesystem refuses the
  lookup — THEN the system SHALL NOT open the path, and SHALL end the
  invocation under the first row of `FR-LITE-007`, which decides the code and
  the `cause`.

  *Observed.* Opened read-only, a FIFO **blocked**: the read was still waiting
  when a three-second bound ended it. A directory failed with
  `disk I/O error` (`SQLITE_IOERR_READ`), which names neither the path nor the
  fault.

  *Why links are followed.* The path is written in `.tpl/.cfg`, which
  `FR-PROJ-010` and `FR-PROJ-011` make the caller's alone, so a link is one the
  caller put there or pointed at — the ground `FR-CONF-014` gives for following
  a link in `ca_path`. Nothing the file holds is a secret the model would
  disclose beyond what the caller can already read.

  *Accepted cost, stated plainly.* The library opens the path by name after the
  kind is established, so a file replaced by a FIFO between the two still
  blocks the open, and no deadline of `FR-CONF-005` interrupts a blocked open.
  Arranging it needs write access to the directory that holds the database.
  `FR-PROJ-030` closes the same race for `.tpl/.cfg` by testing the file it
  then reads; the library takes a path and not an open file, so that form is
  not available here.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-seventh edition,* for rmp `#314`: the last sentence is
  new. The kind is established before the library opens the path, so a path
  that names nothing is found by the check of its kind and never reaches the
  open that returns `SQLITE_CANTOPEN`. This requirement refused only the kinds
  it lists, and `FR-LITE-007` began "once the kind of the file is
  established", so neither decided that case. It now has one rule, the first
  row of `FR-LITE-007`, whichever of the two operations meets it.

  *Rejected: `78`, as a kind this requirement refuses.* A file removed between
  the check of its kind and the open would then be `78` or `74` by which
  operation met its absence first, one condition with two codes, which
  `FR-ERR-002` forbids; and a path naming nothing is the condition
  `FR-LITE-007` already gives `74`, for the reason it states. *Rejected:
  letting the open find it.* The open would follow a check that failed, and
  `FR-LITE-017` puts the check first so that nothing is opened before the kind
  is known.

- **FR-LITE-006**: The system SHALL open the database file read-only, and SHALL
  NOT create it. A path that names no file SHALL stay naming no file after any
  invocation.

  *Observed.* A read-only open of a path that names nothing failed with
  `unable to open database file` (`SQLITE_CANTOPEN`) and created nothing. On the
  file opened read-only, `CREATE TABLE` and `INSERT` failed with
  `attempt to write a readonly database` (`SQLITE_READONLY`).

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The observation speaks of
  the file opened read-only, where it said a connection. The requirement is
  unchanged.

- **FR-LITE-007**: A failure to read the file — from the check of its kind
  under `FR-LITE-005` onwards, save the kinds that requirement refuses — SHALL
  end the invocation with the code below, and SHALL NOT be reported as an
  empty database:

  | Condition | Observed as | Code |
  |---|---|---|
  | The path names nothing, the filesystem refuses the lookup or the open, or a link dangles | The check of the file's kind, finding nothing, or `SQLITE_CANTOPEN` at the open | `74` (`EX_IOERR`) |
  | The file is not a SQLite database | `SQLITE_NOTADB`, at the version probe | `78` (`EX_CONFIG`) |
  | The file is a SQLite database that is damaged, or whose schema the library cannot load — a corrupt page, or a schema object the library cannot parse within the limits it is built with | `SQLITE_CORRUPT`, including `malformed database schema (<object>) - <reason>`, at the version probe or any later statement | `78` |
  | The file's schema holds a value longer than the length limit of `FR-LITE-011` | `SQLITE_TOOBIG`, `string or blob too big`, at the version probe | `78` |
  | The file's schema holds a table or a view whose name begins with `pragma_`, ignoring ASCII case, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a value of a [storage class](glossary.md#storage-class) its contract excludes, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a table whose root page contradicts its text, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a row whose `type` is not one of the four texts SQLite writes, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a row with no text that is not an index's, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a NUL character in a name or a text, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file's schema table holds a trigger whose header SQLite does not write, per `FR-LITE-041` | The check of `FR-LITE-041` returns its row | `78` |
  | The file holds two names of one set of `FR-LITE-043` that differ in their bytes and convert to one text | The rows the catalogue statements returned, compared before the model is presented | `78` |
  | The database is in write-ahead-log mode and its directory refuses the files that mode needs, per `FR-LITE-008` | `SQLITE_READONLY_DIRECTORY` | `74` |
  | A read of the file fails part-way | `SQLITE_IOERR` | `74` |
  | The database stays locked by a writer until the catalogue-query deadline expires, per `FR-LITE-009` | `SQLITE_BUSY` | `69` (`EX_UNAVAILABLE`) |

  The `cause` SHALL name the path as resolved, the statement, the open or the
  check of the file's kind that failed, and what the library returned,
  including the object and the reason where the library names them — or, for
  the check of the kind, what the filesystem returned. For the seven rows of
  `FR-LITE-041`, and for the row of `FR-LITE-043`, the `cause` SHALL name what
  that requirement fixes. For `74`
  and `78` the `hint` SHALL carry
  `tpl cfg database update <entry> --path <path>`.

  *Observed.* A text file failed with `file is not a database`; a file whose
  header bytes 16 to 19 were overwritten failed the same way; a file cut to
  100 bytes failed with `database disk image is malformed`; and a table whose
  `CHECK` nests deeper than the expression-depth limit made **every**
  statement fail with
  `malformed database schema (t) - Expression tree is too large (maximum depth 100)`,
  with that limit lowered to `100` for the reading, as `FR-LITE-011` records.
  On the builds 3.51.3 and 3.54.1, with every control of `FR-LITE-010` and
  every limit of `FR-LITE-011` in force, a file whose schema holds one
  `CREATE TABLE` statement of 1 100 076 bytes failed at the version probe with
  `string or blob too big` (`SQLITE_TOOBIG`, extended code 18). A file that
  cannot be read is never an empty database: an **empty file** of zero bytes,
  on the other hand, opened and returned no schema row, which is SQLite's own
  reading of it, and it is read as a database with no object.

  *Why `78` for a file that is not a database.* The entry points at something
  `tpl` cannot read as a database, which is the reading `FR-SRV-003` gives `78`
  for a server that is not MariaDB, and the next step is the same: point the
  entry elsewhere. A damaged file and a schema the library cannot load share
  that next step, so `FR-ERR-002` admits them to the same code, and the `cause`
  separates them. A schema longer than the length limit shares it too: no
  flag or key relaxes the limit, per `FR-LITE-014`, so the file is one this
  entry cannot read. *Why `74` for a path that cannot be opened.* It is the
  reading `FR-RND-035` gives a `--context` path that cannot be read: the caller
  looks at the path and at what the filesystem said, and a filesystem path is
  no population a `66` could suggest from.

  *Rejected: `65` for a damaged file.* Its row of `FR-ERR-001` sends the caller
  to the template or the context, and neither is at fault. *Rejected: reading
  what can still be read from a damaged file.* A model built from a schema the
  library refused would be a model whose completeness nothing establishes,
  presented at exit `0`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. The length limit of
  `FR-LITE-011` is a row of its own, because the library refuses an oversized
  schema with `SQLITE_TOOBIG` and not with `SQLITE_CORRUPT`; the code is
  unchanged. The damaged-schema row no longer names a schema the hardening of
  `FR-LITE-010` refuses, and the *Observed* note no longer records that a
  `CHECK` calling a function not marked innocuous stops the schema loading:
  that reading is withdrawn, for the reason stated under `FR-LITE-010`. The
  expression-depth reading was re-taken on the builds 3.51.3 and 3.54.1 and
  holds.

  *Amended in the sixty-seventh edition,* for rmp `#314`. The opening sentence
  and the first row take a path whose kind cannot be established, because its
  lookup fails, which `FR-LITE-005` now routes here: the condition was this
  row's, and the requirement began only once the kind was established. The
  `cause` names the check of the kind where that is what failed, with what
  the filesystem returned; the code and the `hint` are unchanged.
  `FR-LITE-040` reads each row of this table by its result code, wherever the
  library returns it, and gives every result code the table does not name a
  code of its own. What the library returned is stated in the `cause` in the
  form `FR-GLOB-018` fixes, bounded under `FR-ERR-046`.

  *Amended in the sixty-ninth edition,* for rmp `#317`. Three rows are new,
  one for each kind of row the check of `FR-LITE-041` refuses, and the
  `cause` of each is that requirement's. None is a result code: the library
  loads all three kinds of schema without error, so no other row could take
  them. A value of a storage class the schema table's contract excludes, and a
  root page that contradicts the object a row's text creates, are damaged
  schemas, the condition of the third row, found by the check rather than by
  the library. No other row moves.

  *Amended in the seventy-first edition,* for rmp `#317`. Four rows are new,
  for the fourth to the seventh kinds of row the check of `FR-LITE-041`
  refuses, and the `cause` sentence names seven rows. Like the three before
  them, none is a result code: the library loads such a row without error.
  Each is a damaged schema, the condition of the third row, found by the
  check rather than by the library. No other row moves.

  *Amended in the seventy-ninth edition,* for rmp `#349`. One row is new,
  for the two names `FR-LITE-043` refuses, and the `cause` sentence names it.
  It is not a result code, nor a row of the check: SQLite writes and loads
  such names without error, and the condition is met once the rows the
  catalogue statements returned are compared. Unlike the rows before it, it
  is not a damaged schema; its ground is the first row's, a file this entry
  cannot read into the model. No other row moves.

- **FR-LITE-008**: The system SHALL NOT write to the database file. WHERE the
  database is in write-ahead-log mode, the library MAY create the two files
  that mode keeps beside the database — `<file>-wal` and `<file>-shm` — when
  they do not exist, and they MAY remain after the invocation. The system SHALL
  NOT create, change or remove any other file outside `.tpl/` on account of a
  `sqlite` read.

  *Observed.* A read-only open of a database in write-ahead-log mode
  created `-wal` and `-shm` beside it and left both after it closed. The same
  database in a directory with no write permission, and without the two files,
  could not be read at all: `attempt to write a readonly database`
  (`SQLITE_READONLY_DIRECTORY`). A database in rollback-journal mode, read the
  same way, left its directory as it found it.

  *Consequence, stated plainly.* For a database in write-ahead-log mode, a read
  is not free of filesystem writes beside the database. The two files belong
  to the database's own locking protocol, every SQLite client that reads such a
  database creates them, and neither changes the database's content.

  *Rejected: the `immutable` open, which creates nothing.* Observed: with a
  writer holding the database open, a table committed to the write-ahead log
  and not yet checkpointed was **absent** from an `immutable` read and present
  in an ordinary read-only one. A read that misses committed structure is the
  silent corruption this corpus refuses. *Rejected: the exclusive locking mode,
  which keeps the log index in memory.* Observed: a read-only open of the
  file refused it with `disk I/O error` (`SQLITE_IOERR_LOCK`). *Rejected: refusing
  a database in write-ahead-log mode.* It refuses the mode an application
  holding the database open is most likely to use.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the sixty-seventh edition,* for rmp `#314`. The requirement
  is unchanged, by the user's decision of 2026-11-03. `FR-PROJ-024` now admits
  the two files as an exception to "no file outside `.tpl`", and
  `BR-PROJ-002`, `FR-CACHE-016` and `BR-SCH-003` admit them beside the pure
  read of `--direct --no-cache`. The temporary files the library would
  otherwise write outside `.tpl` are kept in memory by the `temp_store`
  control of `FR-LITE-010`, which is what keeps the last sentence of this
  requirement true.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The observation and the
  rejected exclusive locking mode speak of a read-only open of the file, where
  they said a connection. The requirement is unchanged.

- **FR-LITE-009**: For a `sqlite` entry, every statement the system issues on
  the open database file SHALL run under the catalogue-query deadline of
  `FR-CONF-005`, resolved from `core.query_timeout`, and a statement that finds
  the database locked SHALL wait for it within that deadline and no longer.
  The phases of DNS resolution, TCP connect, TLS handshake and
  `password_command` SHALL NOT run, and `core.connect_timeout` and
  `core.password_timeout` SHALL have no effect on the entry.

  *Observed,* as *Observations* above states, with the statements of
  `FR-LITE-015` as this edition writes them and an interrupt of the
  statements on the open file — the means by which the deadline ends a statement — sent from a
  second thread:

  - Over one table and 40 000, 80 000, 160 000 or 320 000 views of the form
    `SELECT 1` (files of 2.8, 5.4, 10.6 and 21.1 MB), the seven catalogue
    statements took 0.06, 0.11, 0.24 and 0.61 s together, and the version
    probe, which loads the schema, took 0.14, 0.66, 2.82 and 11.49 s. An
    interrupt sent at a quarter of the probe's time ended the probe within
    0.07 s of it, with `SQLITE_INTERRUPT`, at every size.
  - Over the review's file of 16 KB, whose views each select twice from the
    one before, the catalogue statements took 0.0022 s, with views enabled
    and with views disabled; over 20 000 tables beside one view selecting from
    a missing table, 0.16 s; over a view whose definition fails to compile
    after 0.31 s beside 40 small tables, 0.0025 s; over a chain of 20 000
    views each selecting from the one before, 0.03 s. No figure differed by
    more than its noise between views enabled and disabled.
  - Over the same databases, the statements as `tpl` at `cb99236` issued them
    took the times `FR-LITE-015` records, for the reason it states.

  *Consequence, stated plainly.* The time a read takes grows with the number
  of objects the file declares, in the library's loading of the schema more
  than in proportion to it, and a crafted schema can therefore make a read
  end with `69` at the catalogue-query deadline. It cannot make a read outlast
  the deadline: the observation found no work of the read that the deadline
  did not end.

  *Observed.* With a writer holding an exclusive transaction on a database in
  rollback-journal mode, a reader's first statement failed with
  `database is locked` (`SQLITE_BUSY`) once its wait expired. In
  write-ahead-log mode the same reader was not blocked and saw the state last
  committed.

  *Why `69`.* `FR-ERR-027` gives a catalogue-query deadline `69`, and the
  caller's next step is the one that row of `FR-ERR-001` states: the
  operation is read-only and therefore repeatable.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`. The *Observed* and
  *Consequence* notes above are new, and the requirement is unchanged. The
  security
  review found a read of a crafted file running past the deadline, through
  the library's listing of the objects; by the user's decision recorded under
  `FR-LITE-020`, no statement invokes `pragma_table_list` any more, and the
  read is linear in the objects apart from the schema load, which the
  deadline ends. Nothing a `mariadb` entry does changes.

  *Note added in the seventieth edition,* for rmp `#317`. The requirement is
  unchanged. The [read transaction](glossary.md#read-transaction) of
  `FR-LITE-042` moves where a read meets a writer's lock, and nothing else:
  `BEGIN DEFERRED` reads nothing and waits for no lock, and the check of
  `FR-LITE-041`, the first statement of the transaction that reads the file,
  is where the read waits for a writer within its deadline, as the probe
  already may. Once the check has read, no later statement of the read waits
  for a writer. Observed as *Observations* above states: in rollback-journal
  mode, with a writer holding `BEGIN EXCLUSIVE` after the probe,
  `BEGIN DEFERRED` returned at once and the check failed with
  `database is locked` (`SQLITE_BUSY`) when the reader's wait of 0.5 s
  expired; and while a writer that had changed the file waited to commit,
  each catalogue statement after the check took under a millisecond. In
  write-ahead-log mode the check ran at once beside a writer holding
  `BEGIN EXCLUSIVE`. Over the database of 320 000 views in rollback-journal
  mode, an interrupt sent 5 ms into a catalogue statement inside the
  transaction ended it with `SQLITE_INTERRUPT` and left the transaction
  open, and the connection's close ended it, as `FR-LITE-042` requires. What
  the read costs a writer of the database is stated there.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The statements the
  deadline bounds are those the system issues on the open database file, where
  the text said the connection, and the observation speaks of an interrupt of
  the statements on the open file. The behaviour is unchanged.

- **FR-LITE-040**: For a `sqlite` entry, every
  [result code](glossary.md#result-code) by which the library reports a
  failure SHALL be classified, and the failure SHALL end the invocation with
  exactly one code of `FR-ERR-001`. The classification SHALL be the first of
  the following that names the result code:

  1. `FR-LITE-012`, at the stage that applies or confirms a control or a
     limit, whatever the library returned there;
  2. the rows of `FR-LITE-007`, each by its result code, at the open or at any
     statement. The *Observed as* column records where a code was observed and
     does not bound where its row applies, so the row of `SQLITE_CANTOPEN`
     also takes a file beside the database that the library cannot open, and
     the row of `SQLITE_BUSY` a lock the library returns without waiting out
     the catalogue-query deadline;
  3. the table below, by the primary result code, under any extended code
     unless a row names one.

  | Result code | What the library reports | Code |
  |---|---|---|
  | `SQLITE_MISUSE`, `SQLITE_RANGE`, `SQLITE_NOTFOUND` | A use of the library by `tpl` that the library refuses: an interface called in a way it does not support, a parameter or column number out of range, or an operation code it does not recognise | `70` (`EX_SOFTWARE`) |
  | `SQLITE_ERROR` | An error in a statement of `FR-LITE-015`, which `tpl` wrote and issued over a file whose schema has loaded, or in how `tpl` opened the database file | `70` |
  | `SQLITE_INTERNAL` | A malfunction inside the library the binary carries, per `FR-LITE-001` | `70` |
  | `SQLITE_ABORT`, `SQLITE_AUTH`, `SQLITE_CONSTRAINT`, `SQLITE_LOCKED`, `SQLITE_MISMATCH`, and `SQLITE_READONLY` with no extended code of its own | An outcome that only an operation `tpl` does not perform produces: a write or a constraint on one, an abort or a rollback it requests, an authorisation callback, a plan a virtual-table module refuses, or a conflict inside its one open of the file or with a second open of it sharing its cache | `70` |
  | `SQLITE_EMPTY`, `SQLITE_FORMAT`, `SQLITE_NOTICE`, `SQLITE_WARNING` | A code the library documents as not used, or as returned by no interface | `70` |
  | `SQLITE_NOMEM` | An allocation the library needed failed | `74` (`EX_IOERR`) |
  | `SQLITE_FULL`, `SQLITE_NOLFS`, `SQLITE_PERM` | The disk is full; the filesystem cannot hold a file of the database's size; or it did not grant the access mode the library asked for a file it creates beside the database | `74` |
  | `SQLITE_READONLY` under an extended code other than `SQLITE_READONLY_DIRECTORY` | A file the read needs beside the database — the shared memory of the write-ahead log, or a journal to roll back — cannot be written, a recovery needs write access the file was not opened with, or the database file moved after it was opened | `74` |
  | `SQLITE_PROTOCOL` | In write-ahead-log mode, other opens of the file kept winning the race for the database's locks, and the library gave up after retrying | `69` (`EX_UNAVAILABLE`) |
  | `SQLITE_SCHEMA` | Another open of the file kept changing the schema while a statement ran, past the library's own re-preparation of it | `69` |
  | `SQLITE_INTERRUPT` | The statement was ended at the catalogue-query deadline, per `FR-LITE-009` and `FR-ERR-027` | `69` |
  | A primary result code that no row of this table or of `FR-LITE-007` names | A code SQLite had not published on the date below | `70` |

  A value a statement returns is not a result code, and this classification
  does not reach it. A value of the schema table whose
  [storage class](glossary.md#storage-class) the table's contract excludes is
  a condition of the file: it SHALL end the invocation with `78`, under
  `FR-LITE-041` and the row of `FR-LITE-007` that names it, and SHALL NOT be
  reported as a defect of `tpl`. A value a pragma table-valued function of
  `FR-LITE-015` returns is written by the carried library from the schema it
  has parsed, not read from the file, so a storage class other than the one
  the library writes for that field is a defect of the binary, under
  `FR-ERR-030`.

  The `cause` SHALL name what `FR-LITE-007` obliges it to name — the path as
  resolved, the statement or the open that failed, and what the library
  returned — in the form `FR-GLOB-018` fixes. For `70` these are the invariant
  and the place the `70` row of `FR-ERR-034` obliges the `cause` to name, and
  the `hint` SHALL be the one `FR-ERR-032` fixes. What a `69` `cause` names is
  stated in the note on `FR-ERR-034` for a `sqlite` entry.

  *Source.* SQLite's result-code page, `https://sqlite.org/rescode.html`, last
  updated 2026-02-18 and checked 2026-11-03, lists twenty-eight primary result
  codes that report a failure, from `SQLITE_ERROR` to `SQLITE_WARNING`, beside
  `SQLITE_OK`, `SQLITE_ROW` and `SQLITE_DONE`, which report none. `FR-LITE-007`
  names six of them — `SQLITE_CANTOPEN`, `SQLITE_NOTADB`, `SQLITE_CORRUPT`,
  `SQLITE_TOOBIG`, `SQLITE_IOERR` and `SQLITE_BUSY` — and the extended code
  `SQLITE_READONLY_DIRECTORY`; the table names the other twenty-two. The
  second column paraphrases that page, which states that `SQLITE_MISUSE`
  indicates a bug in the application, that `SQLITE_EMPTY` and `SQLITE_FORMAT`
  are not currently used, that no interface returns `SQLITE_NOTICE` or
  `SQLITE_WARNING`, and that `SQLITE_SCHEMA` reaches the application only after
  the library has re-prepared the statement up to its retry limit.

  *Why three codes, divided by whose the fault is.* `FR-ERR-002` gives two
  failures one code only where the caller's next step is the same, and a
  result code says whose the fault is, which decides the next step. Where the
  library reports that `tpl` used it wrongly, that it failed itself, or an
  outcome only an operation `tpl` never performs can produce, the binary is at
  fault and nothing the caller does repairs it: the result code is how the
  system detects the violated invariant `FR-ERR-030` gives `70`, the ground
  `FR-LITE-012` and `FR-LITE-032` already use. Where the host or the
  filesystem did not supply what the read needed — memory, space, an access
  mode, a writable file beside the database — the next step is the host's,
  which is the step `74` states. Where another open of the file holds or changes the
  database, the next step is the one `FR-LITE-009` gives a lock: the operation
  is read-only and therefore repeatable, which is `69`. A primary result code
  SQLite publishes after the date above is an outcome the system was not built
  to classify, and `FR-ERR-030` declines to continue past one.

  *Observed, on 2026-11-03, through the Python library 3.54.1, with the file
  opened read-only and `trusted_schema` off.* A database whose schema calls a
  function and names a collation that the reading library does not hold — in
  a generated column, a `CHECK`, a `DEFAULT`, a collated index, a partial
  index and an expression index — loaded, and statements of each kind the
  last row of `FR-LITE-015` names, written for the observation, read its
  tables, columns, indexes, index members, foreign keys and schema rows
  without error. On a file whose schema has loaded, only a statement that
  reads the columns of views was observed to fail with `SQLITE_ERROR`, and no
  statement of `FR-LITE-015` reads them since the sixty-eighth edition, per
  `FR-LITE-033`, under whose observation the statements that remain read a
  database holding views that do not compile without error. That is the
  ground for reading `SQLITE_ERROR` as an error in the statement and not in
  the file, for every file the check of `FR-LITE-041` admits: a table or a
  view named like a pragma table-valued function made those statements fail
  with `SQLITE_ERROR`, per the observation under that requirement, and the
  check refuses such a file before they run.

  *Rationale for the two departures from the split the technical
  specification recommended,* `70` for a misuse of the library by `tpl` and
  `74` for every other code. The split is followed except in two places, each
  decided by `FR-ERR-002`: `SQLITE_PROTOCOL` and
  `SQLITE_SCHEMA` are `69`, because what the caller has to look at is another
  open of the file and not the filesystem, and the read is repeatable; and
  `SQLITE_LOCKED` is `70`, because with one open of the file and no write the only
  conflict it can report is one `tpl` made.

  *Rejected: `70` for every result code the rows above do not classify.* A
  host out of memory, a full disk or a database another process holds would be
  reported as a defect of `tpl`. *Rejected: `74` for every one.* A use of the
  library that `tpl` got wrong would send the caller to its filesystem.
  *Rejected: `69` for every one, by reading `FR-ERR-036` onto a `sqlite`
  entry.* That requirement's condition is a session that stops answering,
  which a library inside the process cannot do, and its `69` would tell a
  caller to repeat a read that fails on a defect every time. *Rejected: `78`
  for `SQLITE_ERROR`, as a fault of the file.* Per the observation above, no
  file whose schema loads and that the check of `FR-LITE-041` admits makes
  those statements fail.

  *Consequence, stated plainly.* A malfunction of the library that a crafted
  file provokes is reported with `70`, as a defect of the binary, and it is
  one: the library is the binary's, per `FR-LITE-001`, and a release that
  carries a fix is what `FR-LITE-039` obliges. `SQLITE_NOMEM` reaches `74`,
  whose class is I/O: the code set of `FR-ERR-001` has no code for the host's
  memory, and the host's resources are the next step `74` names, so its cell
  in `FR-ERR-001` names memory for a SQLite read.

  *Added in the sixty-seventh edition,* for rmp `#314`. The technical
  specification recorded that no requirement gave these result codes an exit
  code, and left that one arm of the reader's fault mapping open on it.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The second step of
  the classification, which gave `SQLITE_ERROR` on a statement reading the
  columns of views to `FR-LITE-035`, is removed with that requirement, and the
  steps after it are renumbered. No statement that remains reads a view's
  columns or compiles a view, per `FR-LITE-015` and `FR-LITE-033`, so none can
  produce the error that step took, and the `SQLITE_ERROR` row takes every
  `SQLITE_ERROR` with `70`: a statement that compiled a view would be a defect
  of `tpl`. No other row moves.

  *Amended in the sixty-ninth edition,* for rmp `#317`. The paragraph on
  values is new. Observed as *Observations* above states: a database whose
  schema table held the `sql` of a view as a BLOB loaded, and `tpl` at
  `cb99236` ended the read with `70`, reporting a defect of its own; one
  holding the `type` of a trigger as a BLOB loaded too, and `tpl` presented
  the trigger's table with no trigger, at exit `0`, because the statement
  reading stored text selects the rows whose type is the text `trigger`.
  Neither is a fault of `tpl`. The schema table's contract makes those
  columns text, per `FR-LITE-041`, and a file holding another class was
  written outside it. The rows of the classification are unchanged. The
  observation grounding the `SQLITE_ERROR` row, and the rejection of `78` for
  it, now name the files the check admits, because a file the check refuses
  was observed to make the statements fail with that code.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The rows of
  `SQLITE_ERROR`, of `SQLITE_LOCKED`, of `SQLITE_READONLY` under an extended
  code, of `SQLITE_PROTOCOL` and of `SQLITE_SCHEMA` speak of the database file `tpl`
  opened and of another open of the file, where they said connections, and so
  do the note on the three codes, the observation and the rationale below the
  table. No code changes.

## Hardening against an untrusted file

- **FR-LITE-010**: The database file SHALL be treated as untrusted input, per
  the trust boundaries of [security.md](security.md). Each time it opens
  the file, before the version probe, the system SHALL apply each of the
  following controls and SHALL confirm, by reading the setting back, that each
  took effect. Extension loading is the one control that is not a setting of
  the open database file: it is a property of the library the binary carries, applied
  when the library is built, and its read-back is `PRAGMA compile_options`,
  which the system SHALL issue before any statement that reads the file. The
  control is in effect only WHERE that statement returns the row
  `OMIT_LOAD_EXTENSION`; IF the row is absent, the control has not taken
  effect, under `FR-LITE-012`. Temporary storage is a setting of the
  open database file whose effect also depends on how the library is built, so it is
  confirmed twice: it is in effect only WHERE `PRAGMA temp_store` reads back
  `2` and `PRAGMA compile_options` returns the row `TEMP_STORE=1`,
  `TEMP_STORE=2` or `TEMP_STORE=3`; IF that statement returns `TEMP_STORE=0`,
  or no `TEMP_STORE` row, the control has not taken effect, under
  `FR-LITE-012`.

  | Control | Setting | Closes |
  |---|---|---|
  | Read-only open | read-only, never creating the file, per `FR-LITE-006` | any write through the open file |
  | `query_only` | on | any write the open itself did not refuse, as defence in depth |
  | Defensive mode | on | the features by which ordinary SQL can corrupt a database file |
  | `trusted_schema` | off | an SQL function or a virtual table that is not marked innocuous, run from a view, a trigger or a schema expression |
  | Extension loading | absent: the carried library is built with SQLite's compile-time option `SQLITE_OMIT_LOAD_EXTENSION`, which omits the mechanism, its library interface and the `load_extension()` function | code loaded from a path a schema or a statement names |
  | `cell_size_check` | on | a malformed page read without its cell sizes checked |
  | `mmap_size` | `0` | a page read through a memory map of a file another process can change |
  | `temp_store` | `MEMORY` (`2`) | a temporary file, written outside `.tpl` with content of the database, when a statement sorts, materialises a subquery or builds a transient index |
  | Views | disabled: `SQLITE_DBCONFIG_ENABLE_VIEW` set to `0`, and read back through the same option | the expansion of a view through the views it names, and a view that takes the place of a pragma table-valued function, should a statement name a view, as defence in depth beside `FR-LITE-015` and `FR-LITE-041` |

  *Source.* SQLite's own guidance for untrusted database files, *Defense
  Against The Dark Arts*, at `https://sqlite.org/security.html`, last updated
  2026-07-10; the configuration options at
  `https://sqlite.org/c3ref/c_dbconfig_defensive.html`; and the pragmas at
  `https://sqlite.org/pragma.html`. All three checked 2026-11-03. The guidance
  lists `trusted_schema` off for an untrusted file, and `cell_size_check` on
  and `mmap_size` `0` among the layers for security-sensitive applications;
  the defensive flag, extension loading and the limits of `FR-LITE-011` are
  the options those pages give. A published authority, in the sense of the
  third provenance of the [README](README.md#provenance): what would oblige a
  re-check is a change to that guidance. *Added in the sixty-sixth edition:*
  the compile-time options at `https://sqlite.org/compile.html`, last updated
  2026-10-01 and checked 2026-11-03, state that `SQLITE_OMIT_LOAD_EXTENSION`
  "omits the entire extension loading mechanism from SQLite", including its
  two library interfaces; the pragma page states that `PRAGMA compile_options`
  "returns the names of compile-time options used when building SQLite, one
  option per row", without the `SQLITE_` prefix.

  *Observed.* Each setting read back as set, on the file opened read-only: the
  pragmas `query_only` `1`, `trusted_schema` `0`, `cell_size_check` `1` and
  `mmap_size` `0`, and the configuration flags `defensive on`,
  `load_extension off` and `trusted_schema off` through the shell's
  configuration report. With `query_only` on, the file opened read-write refused
  `CREATE TABLE`, `INSERT` and `PRAGMA user_version=5` with
  `attempt to write a readonly database`.

  *Extension loading, observed.* On the builds 3.51.3 and 3.54.1 compiled
  with `SQLITE_OMIT_LOAD_EXTENSION`, `PRAGMA compile_options` returned 51
  rows, among them `OMIT_LOAD_EXTENSION`, and `load_extension()` failed with
  `no such function: load_extension`. Compiled with the options the driver
  sets, the same builds returned no such row, and `load_extension()` failed
  with `not authorized`. `PRAGMA compile_options` answered alike over the
  observation database, a text file, a truncated file and a schema longer than
  the length limit, so it reads nothing from the file, and the statement trace
  of `FR-LITE-018` reported it.

  *Why the omission is read back at every open, although the build fixes it.*
  SQLite's own page on compile-time options warns that the `SQLITE_OMIT_*`
  options "may not work with the amalgamation", the single-file form in which
  a driver bundles the library, and calls them "mostly unsupported". The
  omission was observed to take effect on both builds, and that observation
  is of two builds, not of every build the project will make. A build that
  misses the option is a defect of the binary, and the read-back at every open
  is what makes it end in `70` under `FR-LITE-012` rather than read a file with
  the mechanism present.

  *Provenance.* The user's decision of 2026-11-03, relayed for rmp `#319`,
  taken after rmp `#315` found that the closest candidate driver could switch
  extension loading off through the library interface without `unsafe` code
  and could not read the switch back, and that the library as that driver
  bundles it starts every open of a database file with the switch on: compile the mechanism
  out, confirm the omission at every open, and end the invocation with `70`,
  like any other control not in effect. The control is not removed, which
  `FR-LITE-004` forbids: the mechanism absent is a stronger setting than the
  mechanism off, and a candidate confirms it with a statement, through no
  `unsafe` code.

  *Rejected: confirming the omission with
  `SELECT sqlite_compileoption_used('OMIT_LOAD_EXTENSION')`.* Observed on both
  builds: preparing it loads the schema, as preparing the probe does — over a
  text file, a truncated file and an oversized schema it failed with
  `file is not a database`, `database disk image is malformed` and
  `string or blob too big`. It would parse the untrusted schema before the
  confirmation answered, and over a file that is not a database a build that
  missed the option would end with `78` instead of `70`, hiding the defect of
  the binary that `FR-LITE-012` exists to report. *Rejected: a driver patched
  to read the switch back.* It is a maintained fork, and moves the `unsafe`
  code into it rather than removing it. *Rejected: switching extension
  loading off without reading it back.* It leaves the one control nothing
  confirms, and the user judged it the weakest option.

  *What `trusted_schema` off costs, and where.* Observed on the builds 3.51.3
  and 3.54.1, with defensive mode on and with it off: a view calling a
  function the library holds and does not mark innocuous failed to compile,
  with `unsafe use of bm25()` and `unsafe use of rtreedepth()`; no statement
  of `FR-LITE-015` compiles a view, per that requirement, so a read meets
  neither failure. A `CHECK` constraint, a `DEFAULT` and a trigger body
  calling such a function did not affect the read: the schema loaded and the
  table's columns were read. Creating such a `CHECK` was refused with
  `unsafe use of rtreedepth()` with `trusted_schema` off and
  succeeded with it on, so a database holding one was written under
  `trusted_schema` on. A function the library does not hold at all failed only
  the view that called it, with `no such function: realpath`, and did not stop
  the schema loading. Among the functions the shell held, those not marked
  innocuous were its own additions and a handful of aggregate, full-text and
  R-tree functions.

  *Corrected in the sixty-sixth edition,* for rmp `#319`. The sixty-fifth
  edition recorded, from the `sqlite3` shell 3.54.0, that a `CHECK` calling a
  function not marked innocuous made the whole schema fail to load, and
  `FR-LITE-007` and `FR-LITE-038` relied on it. The reading does not hold on
  the two builds above, and it could not be reproduced on 2026-11-03 in that
  shell either, nor in the Python library 3.54.1: over a database re-created
  for the purpose, a `CHECK` calling `realpath()` written under
  `trusted_schema` on, the schema loaded under `trusted_schema` off — in the
  shell with defensive mode on and with it off, and in the Python library with
  it off. The database the first reading was taken over was not kept, so that
  reading can be neither re-taken nor explained. It is withdrawn, and this
  note rests on the readings of the sixty-sixth edition.

  *Rejected: disabling triggers.* No statement this system issues can fire
  one, per `FR-LITE-016`, and reading a trigger's definition compiles
  nothing; the control would close nothing that is open.

  *Views: why disabling them is a control, and what it closes.* The guidance
  offers it for an application that uses no view, and no statement this
  system issues names a view, compiles one or invokes `pragma_table_list`,
  per `FR-LITE-015`, so the control closes nothing a read of that list opens.
  It is kept as defence in depth, like `query_only` beside the read-only open,
  because it closes two things a defect in a statement would open. Observed as
  *Observations* above states:

  - A statement that passed a view's name to a pragma table-valued function
    would compile that view. Over the review's database, whose thirteen views
    each select twice from the one before over a table of 2 000 columns,
    `pragma_table_xinfo('a', 'main')` failed in 0.002 s with
    `access to view "v0" prohibited` with views disabled, and was still
    running when a bound of 25 s ended it with views enabled. With views
    disabled, the library still compiles the view it is handed, and refuses
    to expand it through the views it names.
  - A view named like a pragma table-valued function the read calls without
    an argument takes its place: a view named `pragma_encoding` made the
    statement reading the encoding return the view's value with views
    enabled, and fail with `access to view "pragma_encoding" prohibited` with
    views disabled. `FR-LITE-041` refuses such a file first.

  With the statements of `FR-LITE-015`, every catalogue statement returned
  the same rows, in the same time to within its noise, with views enabled and
  with views disabled, over the main fixture database, the database of
  internal tables, an empty file and every database `FR-LITE-009` records.
  The configuration option, at
  `https://sqlite.org/c3ref/c_dbconfig_defensive.html`, checked 2026-11-03,
  disables views in the main schema and in attached ones, leaves `TEMP` views
  allowed, and reads the setting back, without changing it, when passed a
  negative value; `temp` holds no view on a file just opened, per the observation
  under `FR-LITE-041`. The driver the decision records select sets and reads
  back the option through its safe configuration interface, as its published
  package shows, read 2026-11-03, so the control disqualifies no driver under
  step 1 of `FR-LITE-004`.

  *Temporary storage: why it is a control.* It is not in SQLite's guidance for
  an untrusted file; it is what keeps `FR-LITE-008` and `FR-PROJ-024` true.
  The library keeps the temporary data of a sort, of a materialised subquery
  and of a transient index in a file of the host's temporary directory, from
  the moment that data no longer fits in its page cache, unless
  `temp_store` keeps it in memory, per `https://sqlite.org/tempfiles.html`,
  last updated 2025-06-12, and the `temp_store` pragma at
  `https://sqlite.org/pragma.html`, both checked 2026-11-03. The compile-time
  option `SQLITE_TEMP_STORE` decides whether the pragma is obeyed: `0` always
  uses files and `3` always memory, while `1`, the default, and `2` let the
  pragma choose, per `https://sqlite.org/compile.html`, checked 2026-11-03.
  Such a file lies outside `.tpl`, in a directory `tpl` does not choose, and
  holds content of an untrusted file in an amount the file decides.

  *Observed, on 2026-11-03, through the Python library 3.54.1, built with
  SQLite's default `TEMP_STORE=1`, with the file opened read-only under
  `query_only`, `trusted_schema` off, `cell_size_check` on and `mmap_size`
  `0`.* The database held 40 tables of 250 columns each, with column names
  3 000 characters long, each schema statement within the length limits of
  `FR-LITE-011`. A statement reading every column of every table, sorted on a
  column of its result, opened a temporary file named `etilqs_` and a suffix
  in the directory `SQLITE_TMPDIR` named, removed it before it ended, and
  moved that directory's modification time, with `temp_store` at its default
  and at `1`. With `temp_store` at `2` the same statement opened no file and
  left the directory's modification time as it was, and the setting read back
  as `2`. The statement without its sort opened no file under any setting.
  Setting `temp_store` and reading it back succeeded over a text file and a
  truncated file, before the probe failed on each, and the statement trace
  reported both, so neither reads anything from the file. The library the
  binary carries is the same release, built — per the build script of the
  driver the project's decision records select, read at its public source on
  2026-11-03 — with no `SQLITE_TEMP_STORE` of its own, so with the default
  `1`; the read-back of `PRAGMA compile_options` confirms the build at every
  open whatever it is.

  *Rejected: amending `FR-PROJ-024` and `FR-LITE-008` to admit the library's
  temporary files.* They land in a directory the host's environment chooses,
  hold content of an untrusted file in an amount that file decides — the
  observation's schema made one — and would leave the pure read of
  `FR-CACHE-016` writing where the caller cannot see. `FR-PROJ-002`, which
  fixes what `.tpl` holds, is not the requirement such a file contradicts: it
  lies outside `.tpl`. *Rejected: directing the temporary files into `.tpl`.*
  `.tpl` need not be writable, which is the case the pure read exists for, and
  `FR-PROJ-002` fixes what it holds. *Rejected: statements that never sort or
  materialise.* The text of a statement is outside this corpus, per the
  [README](README.md#still-out-of-scope), and whether a transient index is
  built is the library's planning, not the statement's wording; the control
  closes every case whatever the text. *Rejected: relying on the page cache.*
  A temporary file is created only once the cache spills, and the observation
  above is a schema that made it spill.

  *Accepted cost.* A statement's temporary data is held in memory, so the
  memory a read uses grows with the largest sort or transient index a
  statement makes over the schema, which is of the order of the schema the
  model holds in any case. A host that cannot supply it ends the read with
  `74`, per `SQLITE_NOMEM` in `FR-LITE-040`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. Extension loading was
  a switch set off and read back; no candidate could read it back without
  `unsafe` code. It is now absent from the carried library and confirmed by
  `PRAGMA compile_options` before the file is read. The observation of
  `trusted_schema` off is corrected, in the note above.

  *Amended in the sixty-seventh edition,* for rmp `#314`: `temp_store` is a
  new control, at `MEMORY`, confirmed by its read-back and by
  `PRAGMA compile_options`, for the reasons in the notes on temporary storage
  above. The other controls are unchanged.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The rejection of
  disabling views rested on reading a view's columns, which compiles it, and
  no statement reads them now, per `FR-LITE-033`; the rejection rests instead
  on the ground it already gave for triggers. The note on what
  `trusted_schema` off costs states that a read no longer meets the failure of
  a view it refuses. The controls are unchanged.

  *Amended in the sixty-ninth edition,* for rmp `#317`: a ninth control,
  views disabled, confirmed by its read-back, kept as defence in depth. The
  rejection of disabling views rested on the control closing nothing that is
  open, and that remains true of the statements of `FR-LITE-015`, which since
  this edition compile no view; it is reversed because the control closes,
  observed, what a statement that named a view would open, and expanding one
  view through others is what the security review found running past every
  deadline through the listing this edition removes. The other eight controls
  are unchanged.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The controls apply each
  time the system opens the file, where the text said on every connection;
  extension loading is not, and temporary storage is, a setting of the open
  database file, the first row of the table closes any write through the open
  file, and the observations and the provenance speak of the file opened
  read-only or of an open of a database file. No control changes.

- **FR-LITE-011**: Each time it opens the database file, before the version
  probe, the system SHALL set the following run-time limits of the library and SHALL confirm
  each by reading it back:

  | Limit | Value |
  |---|---|
  | The length of a string, a BLOB or a row (`SQLITE_LIMIT_LENGTH`) | `1000000` bytes |
  | The length of one SQL statement (`SQLITE_LIMIT_SQL_LENGTH`) | `1000000` bytes |
  | The number of attached databases (`SQLITE_LIMIT_ATTACHED`) | `0` |

  Every other limit SHALL be left at the value the carried library is built
  with.

  *Rationale.* The two lengths bound what one value of a hostile file can make
  `tpl` hold: a schema statement is a string the library reads and a statement
  it parses, and the default of both is a thousand times larger. A million
  bytes is the guidance's figure for a string, and is far above any statement a
  legitimate schema stores — a `CREATE TABLE` at SQLite's default ceiling of
  2000 columns, at forty bytes a column, is 80 000 bytes. Attaching is closed
  entirely, because no statement of `FR-LITE-015` attaches a database.

  *Rejected: lowering the depth and count limits to the guidance's values* —
  expression depth `10`, parser depth `100`, compound select `3`, function
  arguments `8`, columns `100`. Observed: with the expression depth at `100`, a
  table whose `CHECK` held 150 terms joined by `OR` made the whole schema fail
  to load, per `FR-LITE-007`, and at `200` it loaded. A limit below what a
  legitimate schema uses does not bound an attack; it makes a legitimate
  database unreadable. The guidance gives those values for SQL from an
  untrusted source and for a processor with little stack, and `tpl` issues
  only the statements of `FR-LITE-015`.

  *Accepted cost.* A database holding a schema statement longer than a million
  bytes cannot be read, and is refused with `78` under `FR-LITE-007`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Checked in the sixty-sixth edition, and unchanged.* The depth reading was
  re-taken on the builds 3.51.3 and 3.54.1: at `100` the version probe failed
  with
  `malformed database schema (t) - Expression tree is too large (maximum depth 100)`,
  and at `200` the schema loaded. The refusal the accepted cost names was
  observed as `SQLITE_TOOBIG`, which `FR-LITE-007` now carries in a row of its
  own.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The limits apply each
  time the system opens the database file, where the text said on every
  connection. No limit changes.

- **FR-LITE-012**: IF a control of `FR-LITE-010` or a limit of `FR-LITE-011`
  cannot be applied, or its read-back does not confirm the value set, THEN the
  system SHALL exit `70` (`EX_SOFTWARE`), SHALL read nothing from the file, and
  the `cause` SHALL name the control or the limit, the value set and the value
  read back.

  *Rationale.* No control depends on the file: each is a setting of the
  library the binary carries, and each was confirmed on every file observed,
  including files that are not databases. A control that does not take effect
  is therefore a defect of the binary, which is the second producing condition
  `FR-ERR-030` gives `70`. `78`, which `FR-SRV-010` gives a server that refuses
  the read-only session, would send the caller to `.tpl/.cfg`, which is not at
  fault.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-013**: The system SHALL NOT run an integrity check of the database
  — `PRAGMA quick_check` or `PRAGMA integrity_check`, whole or restricted to a
  table — before or during a read.

  *Rationale.* The cost of the check grows with the data, and the catalogue
  read's does not. Observed over a 234 MB file holding 400 000 rows, warm in
  the page cache: `PRAGMA quick_check` took 92.7 ms, and a read of every
  column of every table took 0.1 ms. The check would make every invocation pay
  for the size of the data in order to read the structure, and on a large
  database it would end the read at the catalogue-query deadline with `69` on
  a healthy file. What the read does touch, the pages of the schema, is read
  under `cell_size_check`, and a damaged page among them is refused under
  `FR-LITE-007`. The figures are informative, per `BR-PERF-008`; the decision
  rests on how the two costs grow, not on the figures.

  *Rejected: the check restricted to the schema table.* It reads again the
  pages the read reads, and finds structure the read never depends on. *Also
  rejected: running the check only under a flag.* `FR-GLOB-001` closes the
  global flags, and a check that runs only when asked for is not a control.

  *Accepted cost.* Damage confined to pages the catalogue read never touches
  is not detected, and the model is read correctly from the pages it does
  touch.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-014**: There SHALL be no flag, configuration key, or environment
  condition that disables `FR-LITE-006`, `FR-LITE-010` or `FR-LITE-011`, or
  that relaxes any value they fix.

  *Rationale.* The ground `FR-SRV-011` gives for the read-only promise on a
  server: a control a caller can switch off makes the promise advisory.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## The closed statement list

- **FR-LITE-015**: For a `sqlite` entry, the system SHALL issue only statements
  drawn from the following closed list:

  | Statement | Purpose | When |
  |---|---|---|
  | `PRAGMA query_only`, `PRAGMA trusted_schema`, `PRAGMA cell_size_check`, `PRAGMA mmap_size` and `PRAGMA temp_store`, each in the form that sets the value `FR-LITE-010` fixes and in the form that reads it back | The controls of `FR-LITE-010` that a statement applies | Once each, when the file is opened |
  | `PRAGMA compile_options`, in the form that reads the options | The confirmation of the extension-loading control of `FR-LITE-010`, and of the build on which its temporary-storage control depends | Once, when the file is opened |
  | `SELECT sqlite_version()`, reading nothing else | The version probe of `FR-LITE-002` | Once, when the file is opened |
  | `BEGIN DEFERRED` and `COMMIT` | The [read transaction](glossary.md#read-transaction) of `FR-LITE-042`, in which the check of `FR-LITE-041` and the catalogue statements read one snapshot of the file | Once each, around the check and the catalogue statements, by a read that issues a catalogue statement after the check |
  | `SELECT` against the schema table `sqlite_schema` and the pragma table-valued functions `pragma_table_xinfo`, `pragma_index_list`, `pragma_index_info`, `pragma_index_xinfo`, `pragma_foreign_key_list` and `pragma_encoding`, restricted to the schema `main` | Reading the catalogue, and the check of `FR-LITE-041`, which is also the catalogue probe of `FR-CFG-056` | As the command requires |

  **No statement SHALL invoke `pragma_table_list`.** No statement of the last
  row SHALL read from a view, or pass the name of a view or of a virtual table
  to a pragma table-valued function, so that no statement makes the library
  compile a view's definition or connect a virtual table. The schema table
  names the objects, per `FR-LITE-020`, and holds the stored text of the views,
  which is all `FR-LITE-033` reads of a view; every statement that reads a
  table through a pragma function takes the table's name from a row of the
  schema table that `FR-LITE-020` reads as a table.

  *Why not `pragma_table_list`.* Its implementation, in the library's source
  read as *Observations* above states, fills its `ncol` field by preparing
  `SELECT*FROM"<name>"` for each view and virtual table whose column count the
  library does not yet know, never running that statement and discarding its
  outcome; after each such compile it restarts its scan of the schema from
  the first object, for as many passes as the schema has objects, and it does
  so before it consults its argument. Every invocation therefore compiles
  views and connects virtual tables, whatever the statement names, and takes
  time that grows with the square of the number of objects. Observed: a view
  selecting from a table alone reported a column count of `3` once the
  objects were listed, with views enabled and with views disabled, so the
  listing compiled it; beside a view selecting from a missing table both
  reported `0`, because each pass stopped at the view it could not compile
  and compiled it again. Listing the objects took 0.32, 1.48 and 6.40 s over
  10 000, 20 000 and 40 000 views; each of four statements took 1.65 s over
  20 000 tables beside one view that does not compile, and 12.9 s over 40
  small tables beside one view whose definition fails to compile after
  0.31 s; an interrupt sent at 1 s ended the listing at 8.88 s over 80 000
  views; over the review's file of 16 KB, whose views each select twice from
  the one before, the listing was still running at 3.4 GB when a bound of
  45 s ended it with views enabled; and the review recorded `tpl` at
  `cb99236` reading 80 000 views for 71.6 s against a deadline of 30 s.
  Passing one table's name, `pragma_table_list('t')`, took the same 1.48 s
  over 20 000 views.

  A control the system applies through the library's configuration interface
  rather than by a statement issues no statement, and the list admits the five
  pragmas of the first row without obliging their use; the pragma of the
  second row is the one `FR-LITE-010` obliges. The table fixes membership and
  not order; the order is `FR-LITE-017`.

  *Observed.* Over the observation database, the whole catalogue was read by
  seven statements of the last row after the probe — the encoding, the
  covered objects, the columns of every table, the indexes and their columns,
  the foreign keys, the stored definitions of the views and triggers, and the
  columns of every view — and the same eight statements read an empty database
  of zero bytes. Each joins `pragma_table_list` to the function it reads
  through, so its count does not depend on the number of objects.

  *Observed again, for the sixty-eighth edition,* under `FR-LITE-033`: the six
  catalogue statements that remain once the columns of views are no longer
  read — the encoding, the covered objects, the columns of every table, the
  indexes and their columns, the foreign keys, and the stored definitions of
  the views and triggers — read a database holding views that do not compile
  without error, and the same seven statements, the probe with them, read an
  empty database of zero bytes.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`: the second row is
  new, for the confirmation `FR-LITE-010` now requires, and the catalogue row,
  formerly the third, is named the last.

  *Amended in the sixty-seventh edition,* for rmp `#314`: the first row names
  `PRAGMA temp_store`, and the second the confirmation of the build it
  depends on, for the control `FR-LITE-010` adds. Both read nothing from the
  file, per the observation under that requirement.

  *Amended in the sixty-eighth edition,* for rmp `#317`: the paragraph after
  the table is new. The statement reading the columns of every view, and the
  statement reading one view's columns, which `FR-LITE-035` issued, go with
  that requirement, by the user's decision recorded under `FR-LITE-033`. The
  rows of the table are unchanged: the last row still admits
  `pragma_table_xinfo`, for the columns of a table.

  *Observed again, for the sixty-ninth edition,* as *Observations* above
  states: the probe and seven catalogue statements of the last row — the
  check of `FR-LITE-041`, the encoding, the covered objects, the columns of
  every table, the indexes and their columns, the foreign keys, and the stored
  definitions of the views and triggers — read the main fixture database, the
  database of internal tables and an empty file, each joining the schema table
  to the function it reads through, and the statement record held no nested
  statement but a pragma function's own, per `FR-LITE-018`.

  *Amended in the sixty-ninth edition,* for rmp `#317`. The last row no
  longer admits `pragma_table_list`, and admits `pragma_index_info`, which
  `FR-LITE-020` reads for `WITHOUT ROWID`; the paragraph after the table
  forbids invoking the first, by the user's decision recorded under
  `FR-LITE-020`. The sixty-eighth edition forbade any statement that makes
  the library compile a view, and every statement that invoked
  `pragma_table_list` did, as the note above records; the reading that
  edition took under `FR-LITE-033`, a column count of `0` for every view, was
  the listing stopping at the first view it could not compile. With the
  function gone, the prohibition holds as written and reaches the connection
  of a virtual table as well. The last row also names the check of
  `FR-LITE-041`, a statement against the schema table that the row already
  admitted.

  *Amended in the seventieth edition,* for rmp `#317`: the third row is new,
  for the read transaction `FR-LITE-042` requires, and the catalogue row stays
  the last, so every passage that names the last row keeps its meaning.
  Neither statement of the new row reads a catalogue fact, so neither is a
  catalogue statement: neither is counted under `FR-LITE-037`, compared under
  `NFR-PERF-008`, or reported by the line `FR-GLOB-017` gives a catalogue
  query.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The first three rows of
  the table run when the file is opened, where they said at connection start.
  No statement changes. The virtual table a statement must not connect keeps
  the library's own verb, per [glossary.md](glossary.md#connection).

- **FR-LITE-016**: The system SHALL NOT issue any other statement for a
  `sqlite` entry. It SHALL issue no DDL, no DML, no `ATTACH` or `DETACH`, no
  `VACUUM`, `ANALYZE` or `REINDEX`, no `PRAGMA` outside the list of
  `FR-LITE-015`, no statement against a schema other than `main`, and no
  transaction statement other than the `BEGIN DEFERRED` and the `COMMIT` that
  `FR-LITE-042` requires — no transaction that takes a write lock, no
  rollback, and no savepoint — and SHALL NOT invoke an external process to
  read the structure.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the seventieth edition,* for rmp `#317`. The requirement
  forbade every transaction statement, and the sixty-fifth edition gave no
  ground for it. Read beside the statements it is listed with, it keeps out
  what a transaction statement can do to the file or to the connections that
  share it: start a write transaction, which `BEGIN IMMEDIATE` and
  `BEGIN EXCLUSIVE` do at once and a write inside a deferred transaction
  would do; stop other connections reading, which `BEGIN EXCLUSIVE` does; or
  end a transaction with a rollback, which may abort the statements in
  progress with `SQLITE_ABORT` — per
  `https://sqlite.org/lang_transaction.html`, last updated 2026-02-18 and
  checked 2026-11-04. A deferred transaction in which only the check and the
  catalogue statements run does none of these: it takes the shared lock each
  statement already takes, and holds it for the read instead of for one
  statement, per `FR-LITE-042`. The prohibition
  stands for every other transaction statement. Observed as *Observations*
  above states: `BEGIN IMMEDIATE` and `BEGIN EXCLUSIVE` failed on the
  read-only connection with `attempt to write a readonly database`
  (`SQLITE_READONLY`), in both journal modes, and `BEGIN DEFERRED` followed
  by `COMMIT` succeeded under `query_only`.

- **FR-LITE-017**: WHEN the system reads the database of a `sqlite` entry, it
  SHALL establish the kind of the file under `FR-LITE-005`, open it under
  `FR-LITE-006`, apply and confirm every control of `FR-LITE-010` — the
  extension-loading control by `PRAGMA compile_options`, and the
  temporary-storage control by its read-back and by that statement — and
  every limit of `FR-LITE-011`, and only then issue the version probe of
  `FR-LITE-002`, followed, where `FR-LITE-042` requires it, by
  `BEGIN DEFERRED`, and then by the check of `FR-LITE-041`. Every other
  catalogue statement SHALL follow the check, and the `COMMIT` of
  `FR-LITE-042` SHALL follow the last of them.

  *Rationale.* The probe is the first statement that reads the file and loads
  its schema, per the observation under `FR-LITE-002`, so every control is in
  force before any byte of the untrusted file is parsed. It is the order
  `FR-SRV-042` gives a server for the same reason: the guarantee is confirmed
  before anything is read.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`: the confirmation of
  extension loading is named in the order. `PRAGMA compile_options` reads
  nothing from the file, per the observation under `FR-LITE-010`, so the
  probe stays the first statement that does, and a control fixed when the
  library was built is confirmed, like the others, before the schema is
  parsed.

  *Amended in the sixty-seventh edition,* for rmp `#314`: the
  temporary-storage control is named in the order, for the same reason.

  *Amended in the sixty-ninth edition,* for rmp `#317`: the check of
  `FR-LITE-041` follows the probe, which loads the schema, and precedes every
  statement that reads through a pragma table-valued function, because those
  are the statements an object of the file could otherwise replace. The check
  reads only `main.sqlite_schema`, which no object of the file can replace.
  Observed as *Observations* above states: databases holding an object named
  `sqlite_master` or `SQLITE_SCHEMA` failed at the probe with
  `SQLITE_CORRUPT`, the first with
  `malformed database schema (sqlite_master) - table sqlite_master already exists`,
  and SQLite refuses to create a name beginning with `sqlite_`, per the
  observation under `FR-LITE-021`. The views control of `FR-LITE-010` is
  applied and confirmed with the others, before the probe.

  *Amended in the seventieth edition,* for rmp `#317`: the two statements of
  the read transaction of `FR-LITE-042` are placed. `BEGIN DEFERRED` reads
  nothing; it follows the probe, so that the order up to the probe is
  unchanged and the probe stays the first statement that reads the file, and
  it precedes the check, so that the snapshot the read takes is the one the
  check reads. `COMMIT` follows the last catalogue statement. Observed as
  *Observations* above states: the statement trace reported `BEGIN DEFERRED`,
  the check, the six catalogue statements and `COMMIT`, in that order, each
  without the prefix `-- `.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The order applies WHEN
  the system reads the database of a `sqlite` entry, where the text said WHEN
  it opens a connection for one. The order is unchanged.

- **FR-LITE-041**: For a `sqlite` entry, after the version probe and before
  every other catalogue statement, the system SHALL issue one statement
  against the schema table `sqlite_schema` of `main` — the **check** — and IF
  the schema table holds a row of any kind below, THEN the system SHALL exit
  `78` (`EX_CONFIG`) and SHALL issue no further statement that reads the
  file:

  | Refused | How it is recognised |
  |---|---|
  | A table or a view named in the namespace of the pragma table-valued functions | A row whose `type` is `table` or `view` and whose `name` begins with `pragma_`, the comparison ignoring ASCII case and made on the characters of the name, whatever its [storage class](glossary.md#storage-class) |
  | A value of a storage class the schema table's contract excludes | A row whose `type`, `name` or `tbl_name` is not text, whose `sql` is neither text nor `NULL`, or whose `rootpage` is neither an integer nor `NULL` |
  | A table whose root page contradicts its text | A row of type `table` whose `rootpage` is `0` and whose text does not create a virtual table, or whose `rootpage` is not `0` and whose text creates one. A text creates a virtual table WHERE its first word after `CREATE`, read under SQLite's comment syntax, is `VIRTUAL`, ignoring ASCII case |
  | A type SQLite does not write | A row whose `type` is text and is not exactly one of the four texts `table`, `index`, `view` and `trigger`: the comparison takes the whole value, an embedded NUL character included, and folds no letter case |
  | A row with no text that is not an index's | A row whose `sql` is `NULL` or the empty text and whose `type` is not exactly `index` |
  | A NUL character in a name or a text | A row whose `name`, `tbl_name` or `sql` holds a NUL character anywhere in its value |
  | A trigger's header SQLite does not write | A row of type `trigger` whose `sql`, read under SQLite's comment syntax and ignoring ASCII letter case, does not begin with the words `CREATE` and `TRIGGER`, or whose next word is an unquoted `IF` |

  Every comparison of a row's `type` that this file makes — in the first and
  third kinds above, and in `FR-LITE-020`, `FR-LITE-021` and `FR-LITE-032` —
  SHALL be exact in the sense of the fourth kind. The fourth kind makes that
  complete: every row of a file the check admits carries one of the four
  texts, so an exact comparison misses none.

  The `cause` SHALL name the path as resolved, the check, the row's `type`
  and `name`, and the kind of row: for the first, that SQLite reads a table
  or a view so named in place of the pragma table-valued function of that
  name; for the second, the column and the storage class it holds; for the
  third, the root page and whether the text creates a virtual table; for the
  fourth, that SQLite writes no other type; for the fifth, that the library
  reads a row with no text as an index's; for the sixth, the column that
  holds the NUL character; for the seventh, that SQLite writes a trigger's
  text beginning `CREATE TRIGGER` and the trigger's name. The
  `hint`
  SHALL carry `tpl cfg database update <entry> --path <path>`, per
  `FR-LITE-007`.

  *Rationale for the first kind.* SQLite resolves a name in a `FROM` clause
  among the tables and views of the schema first, and as a pragma
  table-valued function only when none bears it, comparing ASCII case alone
  (`sqlite3LocateTable` and `sqlite3FindTable`, in the source read as
  *Observations* above states). A table or a view of the file named like a
  function of `FR-LITE-015` therefore replaces it in every statement that
  reads through that function, and the catalogue the read presents becomes
  whatever the file's author wrote into the object. Qualifying the function
  with `main.` does not escape, because the object is in `main`.

  *Observed,* as *Observations* above states. With views enabled, a view
  named `pragma_table_list` holding one forged row made `tpl` at `cb99236`
  list one table, named `forged`, with no column, at exit `0`. With views
  disabled under `FR-LITE-010`, the statements reading through the function
  failed with `access to view "pragma_table_list" prohibited`, which
  `FR-LITE-040` reports as a defect of `tpl` with `70`. A table named
  `PRAGMA_TABLE_XINFO` made the statement reading columns fail with
  `'pragma_table_xinfo' is not a function`, and `tpl` ended with `70`. With
  the statements of `FR-LITE-015` as this edition writes them, which call
  `pragma_table_list` nowhere, a view or a table named `pragma_table_xinfo`
  made the statement reading columns fail the same way, views enabled or
  not, and a view named `pragma_encoding`, a function the read calls without
  an argument, replaced the encoding with its own value at exit `0` with
  views enabled. A view
  whose `name` and `tbl_name` were stored as BLOBs replaced the function as
  the view named in text did. `main.pragma_table_list` read the view, and
  `temp.pragma_table_list` read the function. An index named
  `pragma_table_list`, a trigger named `pragma_index_list`, and a view whose
  name differed from `PRAGMA_TABLE_LIST` only by U+0130 in place of the `I`
  replaced nothing. A row whose `name` read `innocent` over a text creating
  `pragma_table_list` failed at the probe with
  `malformed database schema (innocent)`: the library checks a row's name
  against its `CREATE` text when it loads the schema, so the name the check
  reads is the name the object is resolved by. On a file just opened,
  `temp.sqlite_schema` held no row, and `pragma_database_list` named `main`
  and `temp` alone.

  *Why the prefix, and not the six names `FR-LITE-015` uses.* SQLite tries
  every name beginning with `pragma_` that matches no object as a pragma
  function, and its documentation forms the name of every pragma function
  that way, at `https://sqlite.org/pragma.html`, checked 2026-11-03. The
  prefix is SQLite's own namespace for them, as `sqlite_` is its namespace
  for internal tables, per `FR-LITE-021`. A list of the six would age with
  `FR-LITE-015`: a function added there without the list following would
  reopen the forgery at exit `0`, and nothing would show it. *Why tables and
  views alone.* An index and a trigger replaced nothing, per the observation:
  the names a `FROM` clause resolves are those of tables and views.

  *Rationale for the second kind.* The schema table's contract, at
  `https://sqlite.org/schematab.html`, last updated 2023-06-16 and checked
  2026-11-03, declares the table as
  `CREATE TABLE sqlite_schema(type text, name text, tbl_name text, rootpage integer, sql text)`,
  states that `type` is one of the text strings `table`, `index`, `view` and
  `trigger`, gives `rootpage` as the page of a table's or an index's root
  b-tree and as `0` or `NULL` for a view, a trigger or a virtual table, and
  gives `sql` as the text of a `CREATE` statement, `NULL` for an index a
  constraint creates. SQLite writes nothing else there, so a file holding
  another class was written outside the library's own statements, which is
  the damaged schema the third row of `FR-LITE-007` refuses with `78`.
  Observed under `FR-LITE-040`: such values load without error, and one of
  them removed a trigger from the read at exit `0`; a `rootpage` stored as
  the BLOB `'2'` loaded too, read as page 2.

  *Rationale for the third kind.* `FR-LITE-020` recognises a virtual table
  by its root page, which the schema table's contract makes `0` or `NULL` for
  a virtual table and the page of the root b-tree for any other table. The
  library decides what a row creates from its text, and it loaded both
  contradictions. Observed: a `CREATE TABLE` row with root page `0` loaded,
  and the read would have excluded the table as virtual, at exit `0`; a
  `CREATE VIRTUAL TABLE` row with root page `2` loaded, and the statement
  reading columns connected its module, which failed with
  `vtable constructor failed`, so `tpl` would have ended with `70` after
  running the module's code. A view with a `NULL` root page failed at the
  probe with `malformed database schema`. The first word after `CREATE` is
  what the library's grammar decides on: `VIRTUAL` is the second word of a
  `CREATE VIRTUAL TABLE` and of no other `CREATE` statement. Observed over 26
  table rows, among them the shadow tables of four `fts5` tables, a table
  named `"virtual"`, and virtual tables written as
  `CREATE /* note */ VIRTUAL TABLE` and with a line comment between the two
  words, which loaded: the word agreed with the library's own classification
  on every row, and so did the root page. Reading that word is the reading of
  a header's tokens under SQLite's comment syntax that `FR-LITE-032` already
  makes, not the parser `BR-LITE-002` declines.

  *Rationale for the fourth kind.* The schema table's contract, at the page
  the second kind cites, states that `type` is one of the four text strings,
  and SQLite writes nothing else there. The library is more lenient when it
  loads the schema: it accepts a row whose `type` names the kind of object
  the row's text creates in any ASCII letter case, and reads it only up to
  its first NUL character. Such a row loads and its object works, while
  every comparison that selects rows by type misses it. A file holding one
  was written outside the library's own statements, which is the damaged
  schema the third row of `FR-LITE-007` refuses with `78`.

  *Observed,* as *Observations* above states. A database whose statements
  wrote their keywords in mixed case, created a table with `AUTOINCREMENT`,
  a `WITHOUT ROWID` table, an `fts5` table, an index, a view and a trigger,
  renamed a table and ran `ANALYZE` held the four texts alone, in lower
  case, on all fifteen rows of its schema table. For each of the four types,
  the type written in upper case, in title case and in mixed case, the type
  followed by a NUL character, and the type followed by a NUL character and
  a letter loaded, and the object worked: the table was read, the view
  selected from, the trigger fired on an insert, and the index was listed
  and used by the query planner. With the statements of the seventieth
  edition, a table `pragma_encoding` holding the row `forged`, typed
  `TABLE`, `Table`, `tAbLe`, or `table` followed by a NUL character, passed
  the check, and the statement reading the encoding returned `forged`; a
  table, a view or a trigger so typed was missing from what the statements
  read. The type preceded or followed by a space, followed by a tab or a
  line feed, prefixed or suffixed by a letter, cut short by its last letter,
  empty, written with a non-ASCII letter — a dotless `ı` (U+0131), a dotted
  `İ` (U+0130) or an `á` — in place of an ASCII one, or replaced by another
  of the four failed at the probe with `malformed database schema (<name>)`
  (`SQLITE_CORRUPT`), under the third row of `FR-LITE-007`. A row whose `sql`
  is `NULL`, which the contract reserves for an index a constraint creates,
  loaded with the `type` `INDEX`, `Index`, `garbage` or the empty string.
  `length()` reported five characters for `table` followed by a NUL
  character, a value of six bytes: it stops at the NUL, which is why the
  comparison takes the whole value. The fourth kind returned the row of every
  spelling above that loaded, and no row over the main fixture database in
  either journal mode, an empty file of zero bytes, the databases
  `internal.sql` and `fts5_view.sql` build, or the database of mixed-case
  keywords above.

  *Rationale for the fifth kind.* The schema table's contract, at the page
  the second kind cites, gives `sql` as `NULL` for an index a constraint
  creates and for nothing else. The library reads every row with no text —
  `NULL`, or the empty text — as an index's, whatever its `type`: it looks
  for the index the row names and refuses the schema when it finds none. A row with no text that names an
  existing index under another type therefore loads, describes no object the
  library holds, and is read as a table, a view or a trigger by every
  statement that selects rows by type.

  *Observed,* as *Observations* above states. Over the main fixture database,
  the databases `internal.sql` and `fts5_view.sql` build, and the database
  of constraints and modules that *Observations* describes, every row with
  no text was of type `index`, named `sqlite_autoindex_<table>_<n>` — the
  index of a `UNIQUE` constraint, of a primary key that is not a rowid
  alias, or of a module's shadow table — and `sqlite_sequence`,
  `sqlite_stat1` and `sqlite_stat4` each carried its text; the fifth kind
  returned no row over any of them. A row with no text naming no index, and
  a table's row whose text was set to `NULL`, each failed at the probe with
  `malformed database schema (<name>) - orphan index` (`SQLITE_CORRUPT`).
  The row of an automatic index retyped `table`, `view` or `trigger`, and a
  second row with no text for an index created by `CREATE INDEX`, typed the
  same way, each loaded, and the statements of the seventieth edition read a
  table with no column, reported as a `WITHOUT ROWID` table because
  `pragma_index_info` answers for the index of that name; a view whose
  definition is `NULL`; or a trigger whose stored text is `NULL`, which
  `FR-LITE-032` would end with `70` as a defect of `tpl`. The check of the
  seventieth edition returned no row for any of them, and the fifth kind
  returned each. The same second row with the empty text in place of
  `NULL`, typed `table`, `view` or `trigger`, loaded and was read the same
  way, and the fifth kind returned it. A second row with no text typed
  `index` loaded, and no statement of `FR-LITE-015` reads it: the indexes
  are read through `pragma_index_list`, per `FR-LITE-030`.

  *Rationale for the sixth kind.* The library reads each of these three
  columns only up to its first NUL character, while a statement of
  `FR-LITE-015` reads the whole value. It compares a row's `name` and
  `tbl_name` with the names the row's text creates only up to the NUL, so
  the row loads with a name the library does not hold; and it parses `sql`
  only up to the NUL, so the text a read presents differs from the text the
  library parsed. SQLite wrote no such value in any database observed; a
  file holding one was written outside the library's own statements, which
  is the damaged schema the third row of `FR-LITE-007` refuses with `78`.

  *Observed,* as *Observations* above states. A table's or a view's `name`,
  an index's `name`, a table's `tbl_name` and a trigger's `tbl_name`, each
  followed by a NUL character, loaded. A trigger so attached matched no
  table under `BR-LITE-003`, and would have been dropped at exit `0`. A row
  whose `sql` began with a NUL character was read by the library as an
  index's, as the fifth kind states, and a second row so written for an
  existing index, typed `table`, loaded and was read as a table with no
  column. A view whose `sql` held its definition, a NUL character and
  `; DROP TABLE vessel` loaded, and the statement reading stored text
  returned the whole value as the view's definition. The sixth kind returned
  each of these rows, and no row over any database SQLite wrote.

  *Rationale for the seventh kind.* `FR-LITE-032` derives a trigger's timing
  and event from the words that follow its name, which it reads after
  `CREATE TRIGGER`. SQLite's normalisation of the stored text, at the page
  the second kind cites, upper-cases the keywords `CREATE` and `TRIGGER`
  that begin it, removes a `TEMP` or `TEMPORARY` keyword after `CREATE`, and
  removes a schema qualifier before the name; it also removes
  `IF NOT EXISTS`, as observed under `FR-LITE-032`. A text that begins any
  other way was not written by SQLite, and a derivation that read it would
  start at the wrong word and end the read with a false `70`, or worse.
  `IF` cannot name a trigger unless it is quoted: observed,
  `CREATE TRIGGER if AFTER INSERT ON x …` failed with a syntax error, so an
  unquoted `IF` after `TRIGGER` opens `IF NOT EXISTS` and nothing else.

  *Observed,* as *Observations* above states. A trigger's text beginning
  `CREATE TEMP TRIGGER`, `CREATE TEMPORARY TRIGGER` or
  `CREATE TRIGGER IF NOT EXISTS` loaded, and the seventh kind returned each.
  A text beginning `create trigger`, or with a comment between `CREATE` and
  `TRIGGER`, loaded too and is admitted: `FR-LITE-032` reads the keywords
  ignoring letter case and the header under comment syntax. Over triggers
  SQLite wrote — headers in lower and mixed case, a comment inside the
  header, `ON main.x`, names quoted with brackets and double quotes, and
  triggers named `after`, `before`, `temp`, `"on"` and `"IF"` — the seventh
  kind returned no row.

  *Why `78`.* The next step is the one `78` states for a file that is not a
  database, per `FR-LITE-007`: the entry names a file this entry cannot read,
  and the caller points it elsewhere, so `FR-ERR-002` admits both kinds to
  that code, the third to the seventh kinds on the ground the second
  gives. `tpl` is not at
  fault, and `70` would send the caller to a defect that does not exist.

  *Rejected: comparing names under Unicode case folding.* SQLite folds ASCII
  case alone, per the observation, so a wider comparison would refuse names
  that replace nothing. *Rejected: qualifying every pragma function with
  `temp.`, which read the function on the release observed.* The
  documentation of virtual tables, at `https://sqlite.org/vtab.html`, last
  updated 2025-12-22 and checked 2026-11-03, states that such a table
  "will not work if prefixed with a different schema name", and
  `FR-LITE-016` forbids a statement against a schema other than `main`; the
  read would rest on behaviour SQLite documents as absent. *Rejected:
  reading a value of an excluded class as text.* A trigger whose type is not
  the text `trigger` is missed by any statement that selects by type, so
  leniency would have to reach every statement, and a schema SQLite never
  writes would be read as if it had written it.

  *Accepted cost.* A file holding a table or a view whose name begins with
  `pragma_`, which SQLite accepts, cannot be read. Observed: a table named
  `pragma_settings` was read correctly by `tpl` at `cb99236`, and it is
  refused now. The `cause` names it.

  *Rejected, for the third kind: trusting the root page alone.* Either
  contradiction is then read at exit `0` as something it is not, and the
  second runs the code of a module the read was meant never to connect.
  *Rejected: the library's own classification of the row.* `pragma_table_list`
  is the one statement that reports it, and `FR-LITE-015` forbids it.

  *Rejected, for the fourth kind: comparing `type` as the library does,
  folding ASCII letter case, in the check and in every catalogue statement.*
  Folding case alone leaves the forgery open, because the library also stops
  at a NUL character: observed, a comparison under SQLite's `NOCASE`
  collation found the table `pragma_encoding` typed `TABLE` and did not find
  it typed `table` followed by a NUL character, over which the encoding read
  `forged`. Matching the library in full means imitating, in every statement
  that compares `type`, two behaviours of its loader that its documentation
  does not state and that observation alone found; a release that changed
  either, which `FR-LITE-039` makes a routine event, would reopen the forgery
  or drop an object at exit `0`, with nothing to show it. It would also read
  a schema SQLite never writes as if SQLite had written it, the ground on
  which reading a value of an excluded class as text is rejected above.

  *Accepted cost, for the fourth kind.* A file whose schema table was edited
  to another spelling of a type, which the library loads and uses, cannot be
  read. The `cause` names the row.

  *Rejected, for the fifth kind: reading such a row as its `type` says.* A
  table, a view or a trigger that the file does not hold would be presented
  at exit `0`, or a trigger with no text would end the read with a false
  `70`. Decided under the criterion the user set for the provider, that it be
  the most secure, recorded under `FR-LITE-020`.

  *Rejected, for the sixth and seventh kinds: reading the value as the
  library does,* up to the first NUL character, or skipping the words
  SQLite removes from a header. Each imitates the library's loader, the
  ground on which the fourth kind's rejected option fails.

  *Added in the sixty-ninth edition,* for rmp `#317`.

  *Note added in the seventieth edition,* for rmp `#317`. The check governs
  the statements after it only if they read the schema it read. Issued as a
  statement of its own, while every statement ran as its own transaction, it
  left a writer free to commit between the check and a later statement, and
  a table so added took the place of a function at exit `0`, as observed under
  `FR-LITE-042`; a view so added is stopped by the views control of
  `FR-LITE-010`. `FR-LITE-042` holds the check and the catalogue statements
  in one read transaction. The requirement is unchanged: a file the check
  refuses ends the read before any statement after the check, `COMMIT`
  included.

  *Amended in the seventy-first edition,* for rmp `#317`. The fourth kind is
  new, with the paragraph that makes every comparison of `type` exact, its
  `cause`, its rationale, its observation, its rejected option and its
  accepted cost. Found by the implementation at `82eb1d4`: the library loads
  a row whose `type` differs from the four texts in letter case, so the
  first kind, which compared `type` exactly, let a table typed `TABLE` and
  named `pragma_encoding` forge the encoding at exit `0`, and `FR-LITE-020`
  left a table typed `Table` out of the model at exit `0`. The observation
  found a spelling the report did not name, the type followed by a NUL
  character. Chosen on the criterion the user set for the provider, that it
  be the most secure, recorded under `FR-LITE-020`. The fifth kind is new
  too, with its `cause`, its rationale, its observation and its rejected
  option: the observation of the fourth kind found that the library compares
  no type on a row with no text, and the coordinator of rmp `#317` decided,
  under the same criterion, that such a row is refused unless it is an
  index's; the sweep below added the empty text to it. The sixth kind is
  new: the coordinator accepted, under the same criterion, the refusal of a
  NUL character in `name` and `tbl_name`, and the sweep added `sql`. The
  seventh kind is new, found by the sweep. The sweep took every column of
  the schema table and every value the catalogue statements read from the
  pragma functions, and asked of each shape the library loads whether it
  makes a read drop, invent or mis-attach an object, or end with a false
  `70`; *Observations* above states its conditions, and the shapes it found
  already handled are recorded in the README's account of this edition. The
  first three kinds are unchanged: the first and the third compared `type`
  exactly already, and the new paragraph states it.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The observation speaks of
  a file just opened, where it said a new connection. The requirement is
  unchanged.

- **FR-LITE-042**: For a `sqlite` entry, WHEN a read issues a catalogue
  statement after the check of `FR-LITE-041`, the system SHALL issue
  `BEGIN DEFERRED` immediately before the check, and `COMMIT` immediately
  after the last catalogue statement of the read has completed, so that the
  check and every catalogue statement of the read run in one
  [read transaction](glossary.md#read-transaction) and read one snapshot of
  the file. No statement other than the check and the catalogue statements
  SHALL run between the two. IF the read ends before its last catalogue
  statement completes — the check refuses the file, a statement fails, or a
  deadline ends one — THEN the system SHALL NOT issue `COMMIT`, and the
  transaction SHALL end when the database file is closed: the library
  rolls it back when it closes the file. A command whose only catalogue statement is the check —
  `tpl cfg database test`, under `FR-CFG-056` — SHALL issue neither
  statement: one statement reads one snapshot of its own. The placement of
  the two statements SHALL be verified by the statement record of
  `FR-LITE-018`, under its first assertion.

  *Rationale.* The check exists so that no table or view of the file takes
  the place of a function the read reads through, and it can answer only for
  the schema it reads. With each statement its own transaction, the check
  read the file as last committed when the check ran, and each later
  statement read it as last committed when that statement began, so a writer
  that committed between the two changed what the later statement read
  without the check seeing it. In one read transaction every statement reads
  the snapshot the check read. SQLite's documentation states that a deferred
  transaction starts when the database is first accessed, and that a
  `SELECT` then starts a read transaction
  (`https://sqlite.org/lang_transaction.html`, last updated 2026-02-18); that
  in rollback-journal mode isolation is by locking, a writer waiting until
  every shared lock clears before it can take the exclusive lock it writes
  under (`https://sqlite.org/isolation.html`, last updated 2022-04-18, and
  `https://sqlite.org/lockingv3.html`, last updated 2025-05-31); and that in
  write-ahead-log mode a reader sees a snapshot, a write committed while its
  read transaction is active staying invisible to it
  (`https://sqlite.org/isolation.html`), its end mark "unchanged for the
  duration of the transaction" (`https://sqlite.org/wal.html`, last updated
  2026-08-25). All four pages were checked 2026-11-04.

  *Observed,* as *Observations* above states, in rollback-journal mode and in
  write-ahead-log mode alike unless a mode is named:

  - Without the transaction, the check returned no row; the writer then
    created a table `pragma_encoding` holding one row, `forged`, and
    committed; and the statement reading the encoding returned `forged`,
    while the statement listing the objects listed `pragma_encoding`. That is
    the forgery at exit `0` the check exists to refuse, reached past it.
  - With the transaction, the statement reading the encoding returned
    `UTF-8`, and no statement listed `pragma_encoding`. In rollback-journal
    mode the writer, given no busy timeout, failed with `database is locked`
    (`SQLITE_BUSY`), and committed when tried again after `COMMIT`; given a
    busy timeout of 5 s, it waited and committed after `COMMIT`. In
    write-ahead-log mode the writer committed during the read, and the read
    did not see it. After `COMMIT`, a new statement on the same open of the file
    returned `forged`, in both modes.
  - A writer that committed the same table after `BEGIN DEFERRED` and before
    the check was seen by the check, which returned its row: the snapshot is
    taken by the check, not by `BEGIN DEFERRED`.
  - In rollback-journal mode, a reader that closed the file inside its
    transaction without `COMMIT` held off a writer until the close and not
    after it, and so did one whose statement an interrupt had ended inside the
    transaction, per the note on `FR-LITE-009`.

  *Why a transaction, and why these two statements.* It closes the race by
  prevention: no statement of the read reads a schema the check did not
  read, so nothing a writer adds during the read is ever read, compiled or
  connected. It adds two statements and no condition: each can fail only with
  a result code `FR-LITE-040` already classifies, and a read has the outcomes
  it had. `DEFERRED` is the one form that takes no write lock: `IMMEDIATE`
  starts a write at once, and `EXCLUSIVE` also stops other opens of the file
  reading, per `https://sqlite.org/lang_transaction.html`; both failed on the
  file opened read-only, per the observation under `FR-LITE-016`. `COMMIT`,
  and not `ROLLBACK`, ends it: the transaction wrote nothing, so either ends
  it alike, but `COMMIT` "runs immediately, even if there are pending SELECT
  statements", where a `ROLLBACK` may abort them with `SQLITE_ABORT`, per the
  same page. `SQLITE_ABORT` therefore stays in the row of `FR-LITE-040` that
  takes the outcomes only an operation `tpl` does not perform produces. Both
  ended the transaction under `query_only`, as observed.

  *Why no `COMMIT` on a read that ends early.* The read has failed, and the
  close ends the transaction as `COMMIT` would: "If an sqlite3 object is
  destroyed while a transaction is open, the transaction is automatically
  rolled back" (`https://sqlite.org/c3ref/close.html`, checked 2026-11-04).
  Issuing nothing after the failure keeps the rule of `FR-LITE-041`, that a
  refused file is read by no further statement, without an exception.

  *Rejected: bracketing the read with the schema cookie* — reading
  `PRAGMA schema_version` before the check and after the last catalogue
  statement, and ending with `69`, the code `FR-LITE-040` gives another
  open of the file changing the schema, when the two readings differ. The
  statement form is not resolved among the file's objects: observed, over a
  file holding a table `pragma_schema_version` whose one row held `999999`,
  `PRAGMA schema_version` returned the cookie, `42`, where
  `SELECT schema_version FROM pragma_schema_version` returned `999999`. It is
  rejected on three grounds. It detects after the fact: every statement
  between the two readings has already read through what the writer added —
  observed, the statement reading the encoding returned `forged` before the
  second reading — and a row of the third kind `FR-LITE-041` refuses, added
  after the check, would have had its module run before the second reading,
  per the observation under that requirement. It rests on a value the writer
  sets: `PRAGMA schema_version` given a value sets the cookie, and defensive
  mode makes that a no-op only on the open of the file that has it on — `tpl`'s,
  not the writer's — per `https://sqlite.org/pragma.html`, last updated
  2026-10-04 and checked 2026-11-04. Observed, in both modes: a writer that
  created the table, let the statement reading the encoding return `forged`,
  and then set the cookie back to `41` left both readings at `41`, and the
  read would have ended at exit `0` with the forged encoding. And it adds
  more than the transaction: a pragma the closed list does not hold, read
  twice, and a new condition of `69`.

  *Rejected: bracketing with `PRAGMA data_version`,* which a writer cannot
  set — observed, a writer's `PRAGMA data_version = 1` left the reader's
  value unchanged. It detects after the fact too, and it moves on every
  commit by another open of the file, data included — observed, in both modes, a
  one-row update moved it by one with the schema cookie unchanged — so a
  read of a database an application writes to would end with `69` on any
  write during it. *Rejected: repeating the check after the last catalogue
  statement.* It detects after the fact, and a writer that adds the table
  and drops it again between the two checks passes both.

  *Why the record, and not a test with a writer.* The record shows whether
  the check and the catalogue statements run between `BEGIN DEFERRED` and
  `COMMIT`, which is what `tpl` decides. That one read transaction reads one
  snapshot is the library's documented isolation, observed above; a test
  that paused the read between two statements for a writer to commit would
  test the library, through a pause no invocation has.

  *Accepted cost, stated plainly.* In rollback-journal mode, a writer of the
  database cannot commit while a read runs: it waits under its own busy
  handler, or fails with `SQLITE_BUSY` without one, for as long as the check
  and the catalogue statements take — under a millisecond each over the
  fixture database, observed, and at most what their deadlines under
  `FR-LITE-009` allow over a crafted file. Without the transaction it met the
  same lock for as long as any one statement ran. In write-ahead-log mode no
  writer is held off, and a checkpoint stops at the reader's snapshot until
  `COMMIT`, per `https://sqlite.org/wal.html`, so the log may grow while the
  read runs.

  *What the transaction does not reach.* A process that writes the file's
  bytes without the library's locks. SQLite's documentation states that "any
  process can open the file and overwrite it with garbage", and that "there
  is nothing that the SQLite library can do to defend against this"
  (`https://sqlite.org/howtocorrupt.html`, last updated 2026-04-13 and checked
  2026-11-04). Such a process changes what a statement reads inside a
  transaction or outside one; what it writes is a damaged file, which
  `FR-LITE-007` refuses where the library detects it, and this requirement
  claims nothing against it.

  *What it does not promise.* `FR-CTX-024` is unchanged: the document still
  promises no snapshot, for either provider. The transaction exists so that
  the check governs what the read reads, and a document served from the
  cache mixes reads under `FR-CTX-025` whatever a read did.

  *Provenance.* Found by the technical specification at `bc7885a`, against
  the sixty-ninth edition. Chosen for rmp `#317` on the criterion the user
  set for the provider, that it be the most secure, recorded under
  `FR-LITE-020`, and on the observations above.

  *Added in the seventieth edition,* for rmp `#317`.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. A transaction a read
  leaves without `COMMIT` ends when the database file is closed, where the
  text said with the connection; the observations and the rejected
  alternatives speak of an open of the file. The behaviour is unchanged.

- **BR-LITE-001**: For a `sqlite` entry the read-only promise of `BR-SEC-002`
  has three parts, and two of them prevent. The read-only open of
  `FR-LITE-006` prevents at the library: the file, opened that way, cannot be
  written through that open. The closed list of `FR-LITE-015` prevents in `tpl`: it is
  what `tpl` is built to send. `query_only` makes a write fail on an open of the
  file that could otherwise make one, and is defence in depth beside the two. No
  flag disables any of them, per `FR-LITE-014`. What the promise does not
  cover, in write-ahead-log mode, is stated by `FR-LITE-008`. The
  [read transaction](glossary.md#read-transaction) of `FR-LITE-042` is part of
  the read and not a breach of the promise: deferred, and running only the
  check and the catalogue statements, it takes the shared lock every
  statement takes and writes nothing, and a transaction that could write is
  refused by the read-only open and excluded by `FR-LITE-016`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the seventieth edition,* for rmp `#317`: the last sentence is
  new. Observed as *Observations* above states: a read of the fixture
  database in rollback-journal mode, inside the transaction, left the file's
  bytes, its size and its modification time unchanged, and no file appeared
  in its directory; in write-ahead-log mode the file was likewise unchanged,
  and the two files `FR-LITE-008` admits appeared beside it.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The read-only open and
  `query_only` are stated of the file opened read-only and of an open of the
  file, where the rule said connections. The promise is unchanged.

- **FR-LITE-018**: The closed list of `FR-LITE-015` and the order of
  `FR-LITE-017` SHALL be verified by an integration test that records, through
  the library's statement trace — the event SQLite names `SQLITE_TRACE_STMT`
  — every statement the trace reports on the open database file from the first
  statement the system issues after the open, reached through the seam of
  `FR-ERR-031` and from no invocation of the distributed binary. Over the
  fixture database of `FR-LITE-038` and over an empty database file, the test
  SHALL assert that:

  1. every statement the trace reports without the prefix `-- ` is a
     statement of `FR-LITE-015`, and the statements so reported come in the
     order of `FR-LITE-017`; and
  2. every statement the trace reports with that prefix is a
     [nested statement](glossary.md#nested-statement), which is not checked
     against the list of `FR-LITE-015` and is not counted among the statements
     the system issues, under `FR-LITE-037` or anywhere else.

  The read-only promise SHALL be verified from outside the process: over a
  read of the fixture database of `FR-LITE-038` in rollback-journal mode, the
  file's bytes, its size and its modification time SHALL be unchanged and no
  file SHALL appear in its directory.

  That no statement names a view, per `FR-LITE-015`, SHALL be verified by the
  read of the fixture database of `FR-LITE-038`, which holds views that do
  not compile: the read SHALL succeed. A statement that named one of them
  would fail, whether it failed when prepared or when run, and would end the
  read with `70` under `FR-LITE-040`. The outcome of the read is the
  instrument here, not the record, so the limit of the record stated below
  does not reach this assertion.

  That no statement compiles a view or connects a virtual table, per
  `FR-LITE-015`, SHALL be verified over the database of `FR-LITE-038` that
  holds an `fts5` table and a view over it that compiles: the record of its
  read SHALL hold no nested statement other than one whose text begins
  `-- PRAGMA `. Connecting the table, whether a statement names it or
  compiles the view, makes the module run a statement of its own against the
  file, which the record reports. The database holds no view that does not
  compile, so that nothing stops a statement before it reaches the table.

  *Why the statement record is taken in process.* `BR-SRV-003` observes the
  statements a server receives, from the server's side. An open SQLite
  database file has no other side: the library that receives the statements runs inside the
  process, and its trace interface is the boundary at which they are received.
  What the process does to the file — the promise a caller relies on — is
  observed from outside it.

  *Observed.* On the builds 3.51.3 and 3.54.1, over the observation database:
  the trace reported a statement when it began running. Two statements whose
  preparation failed — `SELECT * FROM no_such_table`, and a `SELECT` from a
  view that names a missing table — and a statement prepared and never run
  were not reported; a statement reading the columns of every view, which
  `FR-LITE-035` issued until the sixty-eighth edition, began running, then
  failed, and was. The library's own reading of the schema table, when it
  loaded the schema, was not reported. Every pragma table-valued function of
  `FR-LITE-015` ran a statement of its own for each invocation, reported with
  the prefix: `-- PRAGMA encoding`, `-- PRAGMA table_list`, and, once for each
  table or index the join reached,
  `-- PRAGMA 'main'.table_xinfo='<table>'`,
  `-- PRAGMA 'main'.index_list='<table>'`,
  `-- PRAGMA 'main'.index_xinfo='<index>'` and
  `-- PRAGMA 'main'.foreign_key_list='<table>'`. Reading the columns of a
  view over an `fts5` table ran, in addition,
  `-- SELECT k, v FROM 'main'.'<table>_config'`: the module's own code,
  reading a table of the file. A view whose definition did not compile produced no prefixed statement
  of its own. *Observed again, for the sixty-eighth edition,* under
  `FR-LITE-033`: the first statement invoking `pragma_table_list` ran the same
  nested statement, with no view read, once the library connected the `fts5`
  table to list it. *Observed again, for the sixty-ninth edition,* as
  *Observations* above states: over a database holding an `fts5` table and a
  view over it that compiles, the statements as `tpl` at `cb99236` issued them
  ran `-- SELECT k, v FROM 'main'.'cargo_note_config'`, and the statements of
  `FR-LITE-015` as this edition writes them ran no nested statement but the
  pragma functions' own, with views enabled and with views disabled. Over the
  main fixture database the earlier statements ran none either: their listing
  stopped at a view that does not compile before it reached the `fts5` table,
  which is why the assertion above names a database of its own. The
  documentation of the event, at `https://sqlite.org/c3ref/c_trace.html`,
  checked 2026-11-03, states that it fires "when a prepared statement first
  begins running" and that a text beginning with `--` marks the invocation of
  a trigger; it does not state the prefix on a nested statement, which is
  observed here. No statement of the system's fires a trigger, per
  `FR-LITE-016`, so every prefixed text in the record is a nested statement.

  *Why nested statements are neither checked nor counted.* The system does not
  issue them, and their text is not `tpl`'s: a pragma function writes its own
  `PRAGMA`, and a module writes statements named after the file's own tables,
  so no list written in advance closes over what an untrusted file can make a
  module run. Counting them would break `FR-LITE-037`, because the pragma
  functions run one for each table or index, and a count that grows with the
  objects is the dependence `NFR-PERF-001` forbids. What a nested statement
  can do to the file is bounded by the controls that bound the open of the file —
  the read-only open and `query_only` — and no view is compiled and no
  virtual table connected, per `FR-LITE-015`, so nothing a view names and no
  module runs during the read; the read-only promise is verified from outside
  the process, whatever ran inside it.

  *Rejected: admitting nested statements to the list of `FR-LITE-015`.* The
  list would be closed over the fixture and open over every other file.
  *Rejected: asserting the nested statements the fixture produces, as
  recorded.* It binds the test to the library's internals: a library release
  that changes how a pragma function or a module reads would fail a test of
  `tpl`'s statements with `tpl` unchanged, and `FR-LITE-039` makes a new
  release a routine event. The assertion over the database holding the
  `fts5` table states a class, that no nested statement is a module's, and
  not a list. *Accepted cost.* It rests on the library reporting a pragma
  function's own statement with the prefix `-- PRAGMA `, observed on the
  builds 3.51.3 and 3.54.1, and a release that wrote it otherwise would fail
  the assertion with `tpl` unchanged; it is the one instrument observed to
  see a module run.

  *Consequence, stated plainly.* A statement that fails when it is prepared,
  or that is prepared and never run, does not appear in the record.
  `FR-LITE-015` and `FR-LITE-016` bind it all the same, and this test cannot
  find one. The read-only promise does not rest on the record: it is verified
  from outside the process, above.

  *What would change this.* An instrument that reports a statement when it is
  prepared rather than when it runs, reachable on the terms of this
  requirement.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. The requirement said
  the test records every statement the library prepares. The trace reports a
  statement when it begins running, and it reports nested statements
  `FR-LITE-015` does not name, so the requirement now states what the record
  holds, what the test asserts of each kind of statement in it, and what the
  record does not show.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The record no longer
  holds a statement reading the columns of views, which the user's decision
  under `FR-LITE-033` removes. The paragraph verifying that no statement
  compiles a view is new, the note on what bounds a nested statement says
  that no view is compiled, and the observation records where the `fts5`
  module's nested statement runs now that no view's columns are read. The two
  assertions on the record are unchanged.

  *Amended in the sixty-ninth edition,* for rmp `#317`. The paragraph
  verifying the views verified that no statement compiles a view, on the
  premise that a statement compiling one that does not compile would fail;
  `pragma_table_list` compiled views and discarded the failure, per
  `FR-LITE-015`, so the read's success showed only that no statement named a
  view, and the paragraph now says so. The paragraph verifying that no
  statement compiles a view or connects a virtual table is new, now that no
  statement invokes that function, and the observation records why it names
  a database of its own. The two assertions on the record are unchanged.

  *Note added in the seventieth edition,* for rmp `#317`. The requirement is
  unchanged. The record of a read now holds `BEGIN DEFERRED` before the check
  and `COMMIT` after the last catalogue statement, both statements of
  `FR-LITE-015` reported without the prefix `-- `, and the first assertion
  places them in the order of `FR-LITE-017`; that is how `FR-LITE-042` is
  verified. Neither fires a trigger, so the reading of every prefixed text as
  a nested statement holds. Observed as *Observations* above states, over the
  main fixture database in both journal modes and over an empty file of zero
  bytes: the trace reported the two statements, in their places, and no
  prefixed text but the pragma functions' own.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The record holds every
  statement the trace reports on the open database file, where the text said
  on the connection, and the note on the statement record speaks of an open
  SQLite database file. The assertions are unchanged.

## The part of the database a read covers

- **FR-LITE-019**: A `sqlite` read SHALL cover the schema `main` and no other.
  The `name` of the `database` object of `FR-CTX-036` SHALL be `main`. The
  `temp` schema and every attached schema are outside the read.

  *Rationale.* `main` is the database file the entry names, which is the
  database `FR-CONF-041` makes a read cover for a server. `temp` holds what one
  open of the file created, and the read creates nothing; an attached schema is a
  second file the entry does not name, the ground `FR-CAT-057` gives for not
  reading a second schema.

  *Observed.* `pragma_table_list` returns rows for `temp` beside `main` — its
  schema table, `sqlite_temp_schema` — which is why the restriction is stated
  as a filter.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the sixty-ninth edition,* for rmp `#317`. No statement reads
  `pragma_table_list` any more, per `FR-LITE-015`: the objects come from
  `main.sqlite_schema`, and every pragma function is passed the schema
  `main`, so no statement reaches `temp`. The requirement is unchanged.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The rationale says that
  `temp` holds what one open of the file created, where it said one
  connection. The requirement is unchanged.

- **FR-LITE-020**: The model SHALL cover, for a `sqlite` read, the rows of
  the schema table `main.sqlite_schema` of type `table`, as tables, and of
  type `view`, as views, excluding every object `FR-LITE-021` names. A covered
  table SHALL carry `table_type` `BASE TABLE`, which is the value `FR-CAT-031`
  records for an ordinary table. The objects SHALL be recognised as follows:

  | Object | How it is recognised |
  |---|---|
  | A view | A row of type `view` |
  | A virtual table, excluded by `FR-LITE-021` | A row of type `table` whose `rootpage` is `0` |
  | A table | A row of type `table` whose `rootpage` is not `0` — every other table, a shadow table among them |
  | A `WITHOUT ROWID` table | A table for whose name `pragma_index_info` returns at least one row |

  A shadow table — a table in which a virtual table's module keeps its data —
  SHALL be covered as a table, like any other, whether or not the carried
  library holds its module.

  A row's `type` SHALL be compared exactly, per `FR-LITE-041`, whose check
  refuses, before any other catalogue statement, a file whose schema table
  holds any other spelling of a type; no row a read reaches is therefore
  missed by the comparison.

  *Provenance.* The user's decision of 2026-11-03, relayed for rmp `#317`, on
  the criterion that the `sqlite` provider be the most secure: the objects
  are read from the schema table alone, and no statement invokes
  `pragma_table_list`, so no statement makes the library compile a view or
  connect a virtual table, and the time a read takes is linear in the
  objects, per `FR-LITE-015` and `FR-LITE-009`. Two accepted costs come with
  it, both stated by the user: shadow tables are read as `BASE TABLE`, and a
  `STRICT` table's `ANY` column loses its `blob` mapping, per `FR-LITE-026`,
  because no other statement of the library reports either fact. *Rejected
  by the user: keeping `pragma_table_list`, with its cost stated as a limit.*
  The limit was a read that a crafted file could hold past the
  catalogue-query deadline for a time growing with the square of its
  objects, and a compile of every view, inside the library, on every read;
  the figures are under `FR-LITE-015`.

  *Source.* The schema table's contract, at
  `https://sqlite.org/schematab.html`, last updated 2023-06-16 and checked
  2026-11-03: "The 'table' string is used for both ordinary and virtual
  tables", and the `rootpage` of a virtual table is `0` or `NULL`. The
  documentation of `PRAGMA index_info`, at `https://sqlite.org/pragma.html`,
  checked 2026-11-03: for a name that is no index's but a `WITHOUT ROWID`
  table's, it returns that table's primary-key columns. A table and an index
  cannot share a name: observed, creating an index named like a table failed
  with `there is already a table named rid_none`. A `NULL` root page cannot
  reach the read: observed, the library refused such a row when it loaded the
  schema, per `FR-LITE-041`, which also refuses a root page that contradicts
  the row's text.

  *Observed,* as *Observations* above states. Over the main fixture database,
  these rules read the same tables, views and virtual table as
  `pragma_table_list` had, and the same `WITHOUT ROWID` flag for every table,
  except the five shadow tables of its `fts5` table — `<name>_config`,
  `_content`, `_data`, `_docsize` and `_idx` — which are now covered, with
  their columns and their primary keys. Over nine tables built for the
  purpose — four `WITHOUT ROWID`, with an integer, a composite, a text and a
  `STRICT` primary key, and five rowid tables, with a text key, no key, a
  rowid alias, an `INTEGER PRIMARY KEY DESC` and a `STRICT` key —
  `pragma_index_info` answered as `pragma_table_list` did on all nine. A
  virtual table whose module the reading library lacks was excluded by its
  root page and never touched.

  *Consequence, stated plainly.* A virtual table's data is presented as the
  tables its module keeps it in: an `fts5` table appears as five `BASE TABLE`
  objects, with their columns and keys, at exit `0`, and nothing in the
  document says a virtual table owns them. A template that must skip them does
  so by name. Which tables a read covers no longer depends on the modules the
  carried library holds.

  *Why `BASE TABLE` and not SQLite's `table`.* `table_type` is the coverage
  predicate of `FR-CAT-001`, and a covered SQLite table is the kind
  `BASE TABLE` names. Carrying SQLite's spelling would give one covered kind
  two values, and every template branching on it would test both.

  *Rejected: recognising a shadow table by its name, as a virtual table's name
  followed by a suffix.* The suffixes are the module's own, a module the
  library does not hold publishes none to the library, and an ordinary table
  may carry such a name; the mistake would be a table missing from the model
  at exit `0`, which is worse than a table present that a template can skip by
  name.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. The bounded claim is
  replaced by the observation that settles it, and the requirement now states
  that the carried library's report decides the covered set.

  *Amended in the sixty-ninth edition,* for rmp `#317`, by the user's decision
  above. The covered objects were those `pragma_table_list` reported with
  type `table` or `view`, and a table it reported as `shadow` was excluded;
  they are now the rows of the schema table, recognised by the table above,
  and a shadow table is covered. Two observations ground nothing now and are
  removed, with the *Consequence* and *What would change this* notes they
  supported: that the type `pragma_table_list` reports was the coverage
  predicate and the schema table was not, because the schema table cannot
  tell a shadow table, which the model no longer needs; and that a build
  without FTS5 reported the shadow tables of an `fts5` table as ordinary
  tables, because a shadow table is now covered whichever modules the
  library holds.

  *Amended in the seventy-first edition,* for rmp `#317`. The paragraph on
  how `type` is compared is new. The library loads a row whose `type` is
  `Table`, or `table` followed by a NUL character, and uses its table, so the
  exact comparison left such a table out of the model at exit `0`, as
  observed under `FR-LITE-041`. The check now refuses such a file with `78`,
  by its fourth kind, and the comparison stays exact. The objects covered
  for every file the check admits are unchanged.

- **FR-LITE-021**: The model SHALL NOT cover, for a `sqlite` read, any of the
  following, and the system SHALL NOT read a further object to cover one:

  | Excluded | How it is recognised |
  |---|---|
  | An internal table | Its name begins with `sqlite_`, ignoring ASCII case — `sqlite_schema`, `sqlite_sequence` and the `sqlite_stat` tables among them |
  | A virtual table | Its row of the schema table is of type `table` with a `rootpage` of `0`, per `FR-LITE-020` |
  | An object of the `temp` schema or of an attached schema | Its schema is not `main`, per `FR-LITE-019` |
  | A trigger on a view | The `tbl_name` of its row matches a view under `BR-LITE-003`; SQLite admits only `INSTEAD OF` triggers there, and the model carries triggers on tables, per `FR-CAT-014` |
  | An index with a member that is an expression | `pragma_index_xinfo` reports a member at column rank `-2`, with no name |
  | A foreign key whose parent key is not columns of a covered table | Its parent table matches no covered table, or a parent column it names matches no column of that table, each under `BR-LITE-003`, or it names no parent column and the parent table has no primary key, or a primary key of another number of columns than the key has |

  *Observed.* A user table named `sqlite_custom`, `SQLITE_upper` or
  `Sqlite_mixed` was refused at creation with
  `object name reserved for internal use`, so the prefix, in any letter case,
  is SQLite's own reservation and not a guess. A
  virtual table whose module the reading library lacks did not stop the read,
  because the filter excludes it before its columns are asked for; asking for
  them failed with `no such module: zipfile`.

  *Why an expression index is excluded and not carried.* The model's index
  carries its members as columns of its table, and `FR-CAT-044` forbids a
  member naming a column the table lacks. An expression names none, and
  MariaDB has no expression index, per `FR-CAT-042`, so the model has no shape
  for one.

  *Why a dangling foreign key is excluded.* SQLite accepts a key naming a
  table it does not hold. `FR-CTX-006` embeds the referenced table and
  `FR-CTX-042` refuses, on the `--context` path, a key naming a table the
  document does not carry, so a carried dangling key would make a dump that
  its own round trip refuses. It is the ground `FR-CAT-057` gives for a key
  that crosses a schema.

  *Accepted cost, stated because it is silent.* An excluded index or key is
  presented as absent, at exit `0`, and nothing in the document says so. The
  exclusions are decisions, in the terms
  [catalogue-coverage.md](catalogue-coverage.md#enumerated-exclusions) opens
  its own with.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`, by the user's decision
  recorded under `FR-LITE-020`: a virtual table is recognised by its root
  page, and a shadow table is no longer excluded; it is covered as a table.
  Observed again: a virtual table whose module the reading library lacks was
  excluded by its root page before any statement asked for its columns.

  *Amended in the seventy-first edition,* for rmp `#317`: the rows of a
  trigger on a view and of a dangling foreign key name the match of
  `BR-LITE-003`. SQLite writes a trigger's `tbl_name`, and reports a key's
  parent table and columns, as a statement wrote them, so an exact match
  would have excluded as dangling a key SQLite enforces. A key that names no
  parent column is also excluded when the parent table's primary key has
  another number of columns than the key. SQLite accepts the declaration,
  and refuses every write the key governs: observed,
  `FOREIGN KEY (x, y) REFERENCES p1`, where `p1` has a one-column primary
  key, and `FOREIGN KEY (z) REFERENCES p2`, where `p2` has a two-column one,
  were created, each in a table of its own, and an insert into either table
  failed with `foreign key mismatch`, with foreign keys enforced, while a
  one-column key on `p1` was enforced. Carried, such a key would pair two
  columns with one under `FR-LITE-031`. A key that names its parent columns cannot
  differ in number: SQLite refused one at creation with
  `number of columns in foreign key does not match the number of columns in the referenced table`.

- **FR-LITE-022**: SQLite has no stored routines. For a `sqlite` read,
  `routines` SHALL be `[]`, per `FR-CTX-004`, and the read SHALL be complete
  under `FR-PRIV-001`: nothing is withheld, because nothing exists. A routine
  named on the command line SHALL be answered with `66` (`EX_NOINPUT`), as a
  routine a server's database does not hold is answered, per `FR-ERR-001`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## How the catalogue fills the model

- **BR-LITE-002**: For a `sqlite` read, a property is filled from what the
  library reports through the statements of `FR-LITE-015`, or from a fact
  SQLite's documentation fixes for every object of its kind; a property that
  neither supplies SHALL be `null`, and a comment SHALL be the empty string,
  which is the value the model carries for an object with no comment, per
  `FR-CAT-039`. **One fact is derived from stored text**, the event and the
  timing of a trigger, per `FR-LITE-032`; no other property is. SQLite has no
  comments on any object.

  The facts SQLite keeps only in the stored `CREATE` text, and what the model
  does with each:

  | Fact | In the model |
  |---|---|
  | A trigger's event and timing | Derived from the trigger's header, per `FR-LITE-032` |
  | `AUTOINCREMENT` | Not read: auto-increment is the rowid alias, read from the pragmas, per `FR-LITE-028` |
  | A generated column's expression | `null`, per `FR-LITE-025` |
  | A `CHECK` constraint | Not read, per `FR-LITE-034` |
  | A foreign key's constraint name, and `DEFERRABLE` | The name is `null`, per `FR-LITE-031`; deferral has no property |
  | A column's `COLLATE` clause | The column's `collation` is `null`, per `FR-LITE-026` |
  | A partial index's `WHERE` clause | No property; the index is carried, per `FR-LITE-030` |
  | A table's `STRICT` option | Not read: a `STRICT` table's columns take their affinity by the rules every column takes, per `FR-LITE-026` |

  *Rationale.* A trigger without its event and timing cannot be used at all,
  and both sit in a header whose grammar SQLite fixes in a few tokens. Every
  other fact sits inside a column definition or a table-constraint list, and
  reaching it means splitting a `CREATE TABLE` into its parts, in each of four
  identifier quotings, around comments and nested parentheses — a parser in
  all but name, whose mistake is a wrong value at exit `0`.

  *Rejected: deriving every fact from the stored text.* It builds the parser
  this file declines. *Rejected: deriving none, and leaving a trigger's event
  and timing `null`.* It carries every trigger as an object a template cannot
  act on.

  *Why `WITHOUT ROWID` is read and not carried.* `pragma_index_info`
  reports it, per `FR-LITE-020`, and `FR-LITE-028` uses it. The model has no
  property for it, and adding one adds a key to every MariaDB document, which
  `FR-CTX-043` makes the change of `provider` alone. A template sees its
  consequence: a `WITHOUT ROWID` table has no rowid alias. *Why `STRICT` is
  not read.* No statement of `FR-LITE-015` reports it, by the user's decision
  recorded under `FR-LITE-020`; a `STRICT` table's declared types are still its
  six, as `column_type` shows.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`: the table gains the
  `STRICT` option, which no statement reads any more, and `WITHOUT ROWID` is
  read from `pragma_index_info`, both by the user's decision recorded under
  `FR-LITE-020`.

- **BR-LITE-003**: For a `sqlite` read, WHERE this file finds a table, a view
  or a column by a name that another catalogue row or field carries, two
  names SHALL match WHERE they are equal once each of the 26 ASCII upper-case
  letters in either is folded to lower case, over the whole of each name; no
  other character is folded. The object found SHALL be carried under its own
  name, as the requirement that reads it gives — never under the spelling of
  the name that found it. The file makes three such matches:

  | Name that finds | Against | Where |
  |---|---|---|
  | A trigger's table: the `tbl_name` field of its row of the schema table | The covered tables, and the views for the exclusion of a trigger on a view | `FR-LITE-032`, `FR-LITE-021` |
  | A foreign key's parent table: the `table` field of `pragma_foreign_key_list` | The covered tables | `FR-LITE-031`, `FR-LITE-021` |
  | A foreign key's parent columns: the `to` field of `pragma_foreign_key_list` | The columns of the parent table the key matched | `FR-LITE-031`, `FR-LITE-021` |

  An index's members and a foreign key's own columns are not matched: the
  library reports each under the name its table declares, so each is the
  name of a column of that table as `FR-LITE-025` carries it, byte for byte.

  *Rationale.* SQLite resolves a name this way, and records some names as a
  statement wrote them rather than as the object declares them. A rule that
  compared names exactly would drop a trigger, or exclude a foreign key as
  dangling, that SQLite attaches and enforces — at exit `0`, with nothing in
  the document to show it.

  *Observed,* as *Observations* above states, over a database whose tables
  `Vessel`, `Voyage` and `cargo`, view `Vessel_Names`, index and triggers
  named one another in other letter cases:

  - SQLite wrote each object's own row with the name its `CREATE` statement
    declared, and an index's `tbl_name` as its table declares it — `Vessel`
    for an index created `ON VESSEL`. It wrote a trigger's `tbl_name` as the
    trigger's `ON` clause wrote it: `VESSEL` for a trigger on `Vessel`, and
    `VESSEL_NAMES` for an `INSTEAD OF` trigger on `Vessel_Names`. Neither
    matched a table or a view exactly; folded, each matched one.
  - `pragma_foreign_key_list` reported the `table` and `to` fields as the key
    wrote them — `VESSEL` and `imo` for a key on `Vessel` and its column
    `IMO`, `vessel` for a key naming no column, and `ÉMBARQUE` and `ID` for a
    key on `Émbarque` and its column `id` — and the `from` field as the child
    table declares the column: `Vessel_Name` for a key written
    `FOREIGN KEY (VESSEL_NAME)`.
  - `pragma_index_xinfo` reported a member as the table declares it: `Name`
    for an index created on `NAME`, and `Flag` for a `UNIQUE (FLAG)`
    constraint.
  - With foreign keys enforced, inserting a child row whose parent did not
    exist failed with `FOREIGN KEY constraint failed` for each key above, so
    SQLite resolved `VESSEL` and `ÉMBARQUE` to a table. A parent row inserted
    into `Émbarque` satisfied the key written `ÉMBARQUE`, and one inserted
    into `émbarque` did not: SQLite folded the ASCII letters and not `É`.

  *Why the match finds one object.* SQLite refused, observed, a table, a
  view or an index named like an existing table in another ASCII letter
  case, an index or a trigger named like an existing one, and a second
  column named like an existing one, each with an error naming the clash;
  and a file whose schema table held tables named `a` and `A` failed at the
  probe with `malformed database schema (A) - table A already exists`
  (`SQLITE_CORRUPT`). Among the tables and views of a file whose schema
  loads, and among the columns of one table, at most one name therefore
  matches. Names that differ in a letter outside ASCII are distinct:
  tables `Émbarque` and `émbarque` were both created.

  *Why the object's own name.* The model carries each object under its own
  `name`, and `FR-CTX-023` promises that every object a document refers to
  is in it. A key carrying `VESSEL` as its referenced table would name a
  table the document does not hold, and the document would be refused as a
  `--context` under `FR-CTX-042`.

  *Rejected: comparing names exactly.* Observed above, it drops the trigger
  on `Vessel` and excludes the key on `Vessel` as dangling. *Rejected:
  folding case under Unicode.* It would match `ÉMBARQUE` to `émbarque`, a
  table SQLite does not resolve it to, and attach the key to the wrong table.

  *Added in the seventy-first edition,* for rmp `#317`, at the coordinator's
  request that every match between catalogue names be stated, after the
  observation under `FR-LITE-041` found the trigger's `tbl_name` written in
  another letter case.

  *Note added in the seventy-ninth edition,* for rmp `#349`. The names this
  rule compares are the bytes the file holds for them, with the 26 ASCII
  letters folded as above, and not the text they convert to under
  `FR-OUT-017`, per `FR-LITE-043`.

- **FR-LITE-043**: For a `sqlite` read, the system SHALL decide which object
  a row belongs to, and every match of `BR-LITE-003`, on the bytes the file
  holds for each name, and SHALL NOT decide either on the text a name
  converts to under `FR-OUT-017`. IF two names of one of the following sets
  differ in those bytes and convert to one text, THEN the system SHALL exit
  `78` (`EX_CONFIG`), SHALL present no model, and SHALL write nothing to the
  cache:

  1. the tables and the views the read covers, taken together;
  2. the columns of one covered table;
  3. the indexes of one covered table;
  4. the triggers of one covered table.

  The `cause` SHALL name the path as resolved, the kind of the two objects,
  for a column, an index or a trigger the table that holds them, and the text
  both names convert to, escaped under `FR-ERR-024` and bounded under
  `FR-ERR-046`; and SHALL state that the file holds them as two names that
  differ only in bytes that are not valid UTF-8, which the model, naming each
  object by its text, cannot carry apart. The `hint` SHALL carry
  `tpl cfg database update <entry> --path <path>`, per `FR-LITE-007`.

  ```
  error: the database file holds two tables tpl cannot tell apart, for database entry 'ledger'
  cause: /home/ana/shop/data/ledger.db holds two tables whose names both read 'r�f': they differ only in bytes that are not valid UTF-8, and tpl names each object by its text
  hint:  point the entry at another file: tpl cfg database update ledger --path <path>
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code. The text in the `cause` is the names as
  `FR-OUT-017` converts them, with U+FFFD, the replacement character, where
  the file holds the bytes.

  *Rationale.* SQLite holds a name as the bytes a statement wrote, valid
  UTF-8 or not, and compares names on those bytes, folding the ASCII letters
  alone, as `BR-LITE-003` records: "As far as SQLite is concerned, those
  names are just byte sequences," and "SQLite does not care whether they are
  valid UTF or not", at `https://sqlite.org/invalidutf.html`, last updated
  2025-05-31 and checked 2026-11-05. Two names that differ only in bytes that
  are not valid UTF-8 are therefore two objects to SQLite, and one text to
  the model once `FR-OUT-017` replaces those bytes. The implementation
  grouped the rows of each statement by that text, so the rows of two such
  tables were folded together and each table carried the other's columns, at
  exit `0`; this is rmp `#349`. Deciding on the bytes keeps every row with
  its object, and every reference with the object SQLite resolves it to: a
  foreign key whose parent table converts to a covered table's name from
  other bytes names a table SQLite does not hold, and `FR-LITE-021` excludes
  it as dangling.

  *Why the read is refused where two names of one set convert to one text.*
  The bytes keep the two objects apart inside the read, and nothing keeps
  them apart after it. The document names every member of these collections
  by its text, and `NFR-DET-002` orders them by it: a foreign key names its
  referenced table and its columns, an index its members, a cached object is
  a file named after it, per `FR-CACHE-030`, and a command names an object by
  its name, per `FR-SCH-010`. Two objects under one name would make each of
  those name both, tie in the order, and present at exit `0` a document in
  which a reference may resolve to the wrong object with nothing to show it,
  which is the defect moved out of the read. A refusal says why, and the
  remedy is the file's: rename one of the two objects.

  *Why `78`.* The file is one this entry cannot read into the model, the
  ground `FR-LITE-007` gives for a file that is not a database, and
  `FR-LITE-041` for a table named like a pragma function, which SQLite also
  writes legitimately. The next step is the same: correct the file, or point
  the entry at one `tpl` can read.

  *Two names in different sets are not refused.* A table and a column, or
  the columns of two tables, are never compared with one another by name, so
  their texts may coincide without any name in the document naming two
  objects. *Consequence, stated plainly.* A file whose names hold bytes that
  are not valid UTF-8 is read, each such name carrying U+FFFD, wherever no
  two names of one set convert to one text.

  *Rejected: carrying both objects, each with its own rows, under one name.*
  It is what deciding on the bytes alone yields, and it moves the merge out
  of the read into every consumer of the document: the cache would write
  both tables to one file, so that a read served from it would present one
  where the file holds two, and a command naming the object, or a template
  resolving a reference by name, would find either. Making it safe needs a
  rule in each of those places, for a file whose two names no output can tell
  apart. *Rejected: excluding both objects under `FR-LITE-021`.* Every
  exclusion there is of a shape the model has no form for; these are tables
  and columns the read covers under `FR-LITE-020`, and an exclusion would drop
  them at exit `0` with nothing to say so.

  *Added in the seventy-ninth edition,* for rmp `#349`.

- **FR-LITE-023**: For a `sqlite` read, the `database` object SHALL carry
  `name` `main`, per `FR-LITE-019`; `charset` read from `pragma_encoding`;
  `collation` `null`; `provider` `sqlite`, per `FR-CTX-043`; and the `server`
  object of `FR-LITE-002`.

  *Observed.* `pragma_encoding` read `UTF-8`. SQLite has no database-level
  default collation: a column with no `COLLATE` clause uses the built-in
  `BINARY`, per `https://sqlite.org/datatype3.html`, and nothing stores it.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-024**: For a `sqlite` read, a table SHALL carry the properties of
  `FR-CAT-053` from the following sources:

  | Property | Source |
  |---|---|
  | `name` | the `name` field of its row of the schema table, per `FR-LITE-020` |
  | `table_type` | `BASE TABLE`, per `FR-LITE-020` |
  | its columns | `FR-LITE-025` |
  | its primary key | `FR-LITE-029` |
  | its indexes | `FR-LITE-030` |
  | `foreign_keys` and `referenced_by` | `FR-LITE-031` |
  | its triggers | `FR-LITE-032` |
  | its `CHECK` constraints | `[]`, per `FR-LITE-034` |
  | `engine`, `collation` | `null`: SQLite has no storage engine per table and no table collation |
  | its comment | the empty string, per `BR-LITE-002` |

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`: the `name` comes from
  the schema table, which `FR-LITE-020` now reads in place of
  `pragma_table_list`.

- **FR-LITE-025**: For a `sqlite` read, a column SHALL be read from
  `pragma_table_xinfo`, in the order of its `cid` field, which is the
  declaration order and the ordinal position of `NFR-DET-002`, and SHALL carry:

  | Property | Source |
  |---|---|
  | `name` | the `name` field |
  | `table_name` | the table it belongs to, per `FR-CTX-019` |
  | its type | `FR-LITE-026` |
  | its nullability | nullable WHERE the `notnull` field is `0`, not nullable WHERE it is `1` |
  | its default | `FR-LITE-027` |
  | auto-increment | `FR-LITE-028` |
  | generated | WHERE the `hidden` field is `2`, the column is generated with the storage kind `FR-CAT-051` carries for a `VIRTUAL GENERATED` column; WHERE it is `3`, the storage kind it carries for a `STORED GENERATED` one; its expression is `null` |
  | invisible | false: the `hidden` value `1` marks a hidden column of a virtual table, which `FR-LITE-021` excludes |
  | its comment | the empty string |

  *Observed.* `pragma_table_xinfo` returned seven fields — `cid`, `name`,
  `type`, `notnull`, `dflt_value`, `pk` and `hidden` — with `notnull`, `pk`
  and `hidden` integers and `dflt_value` text or SQL `NULL`. The two generated
  columns read `hidden` `2` and `3`; every other column read `0`.

  *Consequence, stated plainly, and it reaches the `nullable` test.* The
  nullability is what SQLite states. An `INTEGER PRIMARY KEY` column of a rowid
  table read `notnull` `0` although inserting `NULL` into it assigns a rowid,
  and `https://sqlite.org/lang_createtable.html` documents that any other
  primary-key column of a rowid table that is not `STRICT` admits `NULL`
  unless it is declared `NOT NULL`; a primary-key column of a `WITHOUT ROWID`
  table read `1`. A template
  that needs "never `NULL` once stored" for a rowid alias tests
  `auto_increment`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-026**: For a `sqlite` read, a column's type SHALL be decomposed as
  follows:

  | Part | Source |
  |---|---|
  | `column_type` | the `type` field, exactly as it is stored — the declared type as written, and the empty string for a column declared with none |
  | `data_type` | the column's **type affinity**, in lower case — `integer`, `text`, `blob`, `real` or `numeric` — derived from `column_type` by the five rules of section 3.1 of `https://sqlite.org/datatype3.html`, in their order, for every column, a column of a `STRICT` table included |
  | `precision`, `scale`, `length` | `null` |
  | `unsigned` | `null` |
  | `charset`, `collation` | `null` |
  | `values` | `null` |

  The five rules, in their order: a declared type containing `INT` has
  `integer` affinity; one containing `CHAR`, `CLOB` or `TEXT`, `text`; one
  containing `BLOB`, or no declared type, `blob`; one containing `REAL`,
  `FLOA` or `DOUB`, `real`; any other, `numeric`. The comparison ignores ASCII
  case. *Source*, checked 2026-11-03.

  *Rationale for affinity.* SQLite's declared type is free text and
  `VARCHAR(80)`, `DOUBLE PRECISION` and `STRING` are all legal; affinity is the
  closed set of five by which SQLite itself decides how a value is stored, and
  it is SQLite's rule rather than a classification `tpl` invents. The declared
  text stays whole in `column_type`, which is the safety net `FR-CTX-018`
  keeps for every type.

  *Why `null` for the size parts.* SQLite ignores the numbers in parentheses
  after a type name, per the same page, and enforces no length, precision or
  scale. A `length` of `80` for `VARCHAR(80)` would let a generator emit a
  check the database does not make. The numbers stay readable in
  `column_type`. *Why `null` for `collation`.* No statement of `FR-LITE-015`
  reports a column's `COLLATE` clause, per `BR-LITE-002`.

  *Accepted cost, stated because it is silent: a `STRICT` table's `ANY`
  column.* No statement of `FR-LITE-015` reports that a table is `STRICT`, by
  the user's decision recorded under `FR-LITE-020`, so a column declared
  `ANY` takes the affinity the five rules give it: none of the first four
  matches `ANY`, and the fifth gives `numeric`. In a `STRICT` table such a
  column stores a value as it is received, which
  `https://sqlite.org/stricttables.html`, checked 2026-11-03, states and which
  SQLite names `blob` affinity elsewhere; the model carries `numeric`, at exit
  `0`, and a template that needs the difference reads `column_type`, which
  holds `ANY` either way.

  *Consequence, stated plainly.* SQLite has no date-and-time type, per
  `https://sqlite.org/datatype3.html`. A column declared `DATETIME` has
  `numeric` affinity and satisfies the `numeric` test of `FR-ENV-041`, never
  `temporal`; a template that needs the declared name reads `column_type`.
  `FR-ENV-048` fixes the families of the five values.

  *Observed.* The declared types `VARCHAR(80)`, `DECIMAL(12,3)`,
  `NUMERIC(5,2)`, `DOUBLE PRECISION`, `BOOLEAN` and `DATETIME` were each
  returned exactly as written, and a column declared with no type returned the
  empty string.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`, by the user's decision
  recorded under `FR-LITE-020`: a column of a `STRICT` table declared `ANY`
  had `data_type` `blob`, read through the `strict` field of
  `pragma_table_list`, which no statement invokes now; it has the affinity the
  five rules give, `numeric`. Every other column is unchanged.

- **FR-LITE-027**: For a `sqlite` read, the system SHALL classify a column's
  default from the `dflt_value` field by the first row of the following table
  that matches, and SHALL derive the forms of `FR-CTX-012` from that
  classification and the column's nullability alone:

  | Shape of `dflt_value` | Example | Emitted |
  |---|---|---|
  | SQL `NULL`, and the column is not nullable | — | `null` |
  | SQL `NULL`, and the column is nullable | — | `{"kind":"null"}` |
  | `NULL`, ignoring ASCII case | `NULL` | `{"kind":"null"}` |
  | A single quoted string: matches `'([^']|'')*'` | `'unnamed'`, `''`, `'Lloyd''s "A1"'` | `literal`, carrying the text between the outer quotes with each doubled apostrophe collapsed to one |
  | A number: matches `[+-]?([0-9]+(\.[0-9]*)?|\.[0-9]+)([eE][+-]?[0-9]+)?` or `[+-]?0[xX][0-9A-Fa-f]+` | `0.0`, `-1.5`, `+5`, `1e3`, `0x1F` | `literal`, carrying the value unchanged |
  | A BLOB: matches `[xX]'[0-9A-Fa-f]*'` | `x'00ff'`, `X'AB'` | `literal`, carrying the value unchanged |
  | `TRUE` or `FALSE`, ignoring ASCII case | `TRUE`, `false` | `literal`, carrying the value unchanged |
  | Anything else | `CURRENT_TIMESTAMP`, `current_time`, `date('now')`, `1+2`, `"dq"` | `expression`, carrying the value unchanged |

  *Observed.* `dflt_value` is SQL `NULL` exactly where the column declares no
  `DEFAULT` clause, nullable or not, and text everywhere else. The text is the
  default **as written**, case preserved, except that a parenthesised default
  is stored **without its outer parentheses**: `DEFAULT ('a')` returned `'a'`,
  `DEFAULT (1+2)` returned `1+2`, `DEFAULT (NULL)` returned `NULL`,
  `DEFAULT (-1)` returned `-1`, and `DEFAULT (  lower( 'X' )  )` returned
  `lower( 'X' )`. `DEFAULT NULL` returned the four characters `NULL`.

  *Rationale.* The emitted forms keep the meaning `FR-CTX-012` gives them on a
  server: `null` is a column that has no default, and `{"kind":"null"}` is one
  whose default is `NULL`, including a nullable column with no `DEFAULT`
  clause, which SQLite, like MariaDB, fills with `NULL`. SQLite draws, through
  `dflt_value` and `notnull`, exactly the distinction MariaDB's catalogue
  draws in one field, so the combination is a reading of two reported fields
  and not the reconstruction `FR-CTX-012` rejects. A parenthesised constant is
  classified with the literal it encloses because SQLite stores the two
  identically and they are the same value. The final row is the direction
  `FR-CTX-037` argues for: an expression mistaken for a literal can produce SQL
  that parses and means something else.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-028**: For a `sqlite` read, a column SHALL be auto-incremental
  exactly WHEN it is the **rowid alias** of its table: the table is not
  `WITHOUT ROWID` (`pragma_index_info` returns no row for its name, per
  `FR-LITE-020`), the column is its table's only primary-key column, and
  `pragma_index_list` reports no index of origin `pk` for the table. The
  `AUTOINCREMENT` keyword SHALL NOT be read.

  *Observed.* `INTEGER PRIMARY KEY`, `integer primary key autoincrement`, and
  `INTEGER` with `PRIMARY KEY(id)` as a table constraint each produced no
  index of origin `pk`; `INT PRIMARY KEY`, `BIGINT PRIMARY KEY`,
  `INTEGER PRIMARY KEY DESC` and the `INTEGER PRIMARY KEY` of a
  `WITHOUT ROWID` table each produced one. That matches SQLite's documented
  rule, in `https://sqlite.org/lang_createtable.html`, that only a column
  declared exactly `INTEGER`, as the sole primary key of a rowid table and not
  `DESC`, becomes an alias for the rowid; the test above reads the outcome of
  the rule rather than restating it.

  *Rationale.* A rowid alias is assigned by the database when an insert
  supplies none, which is the fact a generator branches on.
  `AUTOINCREMENT` changes only whether a value is ever reused, which no
  property of the model carries.

  *Rejected: reading `AUTOINCREMENT` from the stored text.* It marks only the
  columns that carry the keyword, and leaves an `INTEGER PRIMARY KEY` that the
  database assigns marked as one it does not. *Rejected: the presence of the
  table in `sqlite_sequence`.* That row appears on the first insert, so it
  answers by the data and not by the structure.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-ninth edition,* for rmp `#317`: whether a table is
  `WITHOUT ROWID` is read from `pragma_index_info`, per `FR-LITE-020`, in
  place of the `wr` field of `pragma_table_list`, by the user's decision
  recorded there. Observed: the two agreed on every table of the main fixture
  database and on nine tables built for the purpose. The rule is unchanged.

- **FR-LITE-029**: For a `sqlite` read, a table's primary key SHALL be the
  columns for which `pragma_table_xinfo` reports a `pk` greater than `0`, in
  ascending order of `pk`, and the table SHALL carry none WHERE no column
  reports one. WHERE `pragma_index_list` reports an index of origin `pk`, the
  primary key SHALL carry that index's name and the sort direction of its key
  members; WHERE it reports none, which is the rowid alias of `FR-LITE-028`,
  the primary key's name SHALL be `null` and its member SHALL be ascending.

  *Observed.* The composite key `(vessel_imo, voyage_no)` read `pk` `1` and
  `2`. A rowid alias has no index of its own: the rowid is the table's B-tree
  key, in ascending order.

  *Why `pk` and not the index.* `FR-CAT-043` reads a server's primary key from
  the index catalogue. In SQLite the commonest primary key, the rowid alias,
  has no index, so the index cannot be the source, and the `pk` field is
  populated in every case.

  WHERE the primary key is the rowid alias, its remaining properties of
  `FR-CAT-042` SHALL be: `unique` true; the prefix length of its member `null`;
  `type` `null`; `comment` the empty string; and `ignored` false.

  *Rationale.* `unique` is a fact SQLite's documentation fixes for every rowid
  alias, which `BR-LITE-002` admits as a source: the rowid is "a 64-bit signed
  integer key that uniquely identifies the row within its table", per
  `https://sqlite.org/lang_createtable.html`, last updated 2025-04-30 and
  checked 2026-11-03. The other four are the values `FR-LITE-030` gives every
  index of a `sqlite` read: no statement of `FR-LITE-015` reports a prefix
  length or an index type, SQLite keeps no comment, per `BR-LITE-002`, and
  `ignored` is false on every SQLite index. A rowid-alias key therefore carries
  what a primary key read from an index of origin `pk` carries, so a template
  reads the key's properties the same way whichever kind of primary key a
  table has.

  *Rejected: `unique` `null`, because no index reports it.* It presents as
  unknown a uniqueness SQLite guarantees, and a template testing the key's
  `unique` would branch differently on the commonest primary key of a SQLite
  table than on any other. *Rejected: `type` `BTREE`, because the rowid is the
  key of the table's B-tree.* `FR-LITE-030` gives `null` to every SQLite
  index, which is a B-tree too, and one `sqlite` read would then carry two
  answers to one question.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-seventh edition,* for rmp `#314`: the paragraph on
  the remaining properties of a rowid-alias key is new. The requirement fixed
  its name and the direction of its member, and the key has no index from
  which `FR-LITE-030` could supply the rest; the technical specification
  recorded a reading of them, which this paragraph adopts. The `unique` test
  of `FR-ENV-041` consults the primary key as well as the index collection, so
  a rowid-alias column satisfies it, as a MariaDB primary-key column does.

- **FR-LITE-030**: For a `sqlite` read, a table's indexes SHALL be those
  `pragma_index_list` reports for it, less those `FR-LITE-021` excludes, each
  carrying:

  | Property | Source |
  |---|---|
  | `name` | the `name` field, including the names `sqlite_autoindex_<table>_<n>` SQLite gives the index of a `UNIQUE` or `PRIMARY KEY` constraint |
  | `unique` | true WHERE the `unique` field is `1` |
  | its members | the rows of `pragma_index_xinfo` whose `key` field is `1`, in ascending order of `seqno`, each named by its `name` field and descending WHERE `desc` is `1` |
  | the prefix length of a member | `null` |
  | `type` | `null` |
  | `comment` | the empty string |
  | `ignored` | false |

  *Observed.* `pragma_index_list` reports `seq`, `name`, `unique`, `origin`
  (`c`, `u` or `pk`) and `partial`; `pragma_index_xinfo` reports `seqno`,
  `cid`, `name`, `desc`, `coll` and `key`. Rows with `key` `0` are the rowid
  and, for a `WITHOUT ROWID` table, the columns that follow the key; they are
  not members. The index of a `WITHOUT ROWID` table's primary key is reported
  by `pragma_index_list` and has no row in `sqlite_schema`.

  *A partial index is carried as an index, and it says less than its flag.* Its
  predicate has no property, per `BR-LITE-002`. A partial `UNIQUE` index is
  `unique`, as SQLite reports it, and the `unique` test of `FR-ENV-041` is true
  for its members although only the rows its predicate selects are held
  unique.

  A member's name is the `name` field as `pragma_index_xinfo` reports it,
  which is the column's name as its table declares it, whatever case the
  index wrote it in, per `BR-LITE-003`; it is therefore the `name` of a
  column the table carries, as `FR-CAT-044` requires, with no match made.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the seventy-first edition,* for rmp `#317`: the paragraph on a
  member's name is new. It states what was observed under `BR-LITE-003`, and
  the source of the members is unchanged.

- **FR-LITE-031**: For a `sqlite` read, a table's outgoing foreign keys SHALL
  be read from `pragma_foreign_key_list`, one key per value of its `id` field,
  less those `FR-LITE-021` excludes, each carrying:

  | Property | Source |
  |---|---|
  | `name` | `null` |
  | its columns | the `from` field, in ascending order of `seq`, which is the column's name as the referencing table declares it, per `BR-LITE-003` |
  | `referenced_table` | the `name` of the covered table the `table` field matches under `BR-LITE-003` |
  | `referenced_columns` | for each `to` field, paired by `seq`, the `name` of the column of the referenced table it matches under `BR-LITE-003`; WHERE `to` is SQL `NULL`, the referenced table's primary-key columns, in the order of `FR-LITE-029` |
  | `on_update`, `on_delete` | the `on_update` and `on_delete` fields, carried unchanged |
  | `match_option` | the `match` field, carried unchanged |
  | `referenced_key` | `null` |

  Incoming keys, under `referenced_by`, SHALL be the same keys read from the
  other end, per `FR-CAT-013`, each listed under the table its parent table
  matched. Because no key has a name, the members of
  `foreign_keys` and of `referenced_by` SHALL be ordered by the referencing
  table's name, then the referencing columns, then the referenced table's
  name, then the referenced columns, each compared byte by byte, per
  `NFR-DET-002`.

  *Observed.* The rule spellings returned were `CASCADE`, `NO ACTION`,
  `RESTRICT`, `SET NULL` and **`SET DEFAULT`** — the fifth, which `FR-CAT-033`
  records MariaDB discarding, is kept and enforced by SQLite. `match` read
  `NONE` on every key, including one declared `MATCH FULL`. `to` was SQL
  `NULL` on the keys declared `REFERENCES port` and `REFERENCES country`,
  which name no column; SQLite then uses the parent's primary key, per
  `https://sqlite.org/foreignkeys.html`, checked 2026-11-03, which also states
  that SQLite parses a `MATCH` clause without enforcing it and handles every
  key as `MATCH SIMPLE` — the field reads `NONE` regardless, and is carried as
  read. The `CONSTRAINT` name a key was
  declared with is reported nowhere, and `id` numbers a table's keys in the
  reverse of their declaration order.

  *Why the order is by content.* `NFR-DET-002` orders a collection by name, and
  every name here is `null`. The `id` field is a position the caller cannot
  see and that reverses the declaration, which is the reading that requirement
  rejects.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the seventy-first edition,* for rmp `#317`.
  `referenced_table` and `referenced_columns` carried the `table` and `to`
  fields, which report the parent table and columns as the key wrote them:
  a key written `REFERENCES VESSEL (imo)` on a table `Vessel` with a column
  `IMO` would have named a table and a column the document does not hold,
  and `FR-LITE-021` would have excluded it as dangling at exit `0`. They now
  carry the names of the table and the columns the fields match under
  `BR-LITE-003`, and an incoming key is listed under the table its parent
  matched. The `from` field already reported the declared name, as
  observed; the row states it.

- **FR-LITE-032**: For a `sqlite` read, a table's triggers SHALL be the rows of
  `sqlite_schema` of type `trigger` whose `tbl_name` matches that covered
  table under `BR-LITE-003`, each carrying:

  | Property | Source |
  |---|---|
  | `name` | the `name` field |
  | `event`, `timing` | derived from the header of the stored text, below |
  | `statement` | the `sql` field, the whole stored `CREATE TRIGGER` statement, unchanged |
  | `orientation` | `ROW` |
  | `old_row_alias`, `new_row_alias` | `OLD`, `NEW` |
  | `action_order`, `definer`, `sql_mode`, and the three character-set and collation fields of `FR-CAT-050` | `null` |

  **The derivation reads the tokens between the trigger's name and the keyword
  `ON`, and nothing else.** SQLite stores the header as
  `CREATE TRIGGER <name> [BEFORE | AFTER | INSTEAD OF] (DELETE | INSERT |
  UPDATE [OF <columns>]) ON <table>`. `timing` SHALL be `BEFORE` or `AFTER` as
  written, and `BEFORE` WHERE neither is written; `event` SHALL be `DELETE`,
  `INSERT` or `UPDATE`, an `UPDATE OF` column list being read as `UPDATE`. The
  name and the column list SHALL be read under SQLite's four identifier
  quotings, and the header under its comment syntax. Every keyword of the
  header SHALL be recognised ignoring ASCII letter case, as SQLite reads it,
  and `timing` and `event` SHALL carry the keyword in upper case: SQLite
  upper-cases the leading `CREATE TRIGGER` alone and keeps the rest of the
  header as written. A text that does not begin `CREATE TRIGGER` followed by
  the name is refused before the read, per `FR-LITE-041`. IF the header does not
  yield one timing and one event, THEN the system SHALL exit `70`: the library
  accepted the text, and a derivation that cannot read it is a defect of
  `tpl`.

  *Source.* `https://sqlite.org/lang_createtrigger.html`, checked 2026-11-03,
  for the grammar, for "BEFORE is the default when neither keyword is
  present", for "SQLite supports only FOR EACH ROW triggers", for the fixed
  names `OLD` and `NEW`, and for "INSTEAD OF triggers work only on views";
  `https://sqlite.org/schematab.html`, last updated 2023-06-16 and checked
  2026-11-04, for the normalisation of the stored text: "The CREATE, TABLE,
  VIEW, TRIGGER, and INDEX keywords at the beginning of the statement are
  converted to all upper case letters", "The TEMP or TEMPORARY keyword is
  removed if it occurs after the initial CREATE keyword", and "Any database
  name qualifier that occurs prior to the name of the object being created
  is removed". *Corrected in the seventy-first edition:* this line read "the
  keywords upper-cased", which the page states of the leading keywords
  alone.

  *Observed.* `CREATE TRIGGER IF NOT EXISTS "t q" AFTER UPDATE OF x ON a1 …`
  was stored as `CREATE TRIGGER "t q" AFTER UPDATE OF x ON a1 …`, with
  `IF NOT EXISTS` removed, which that page does not state; a trigger declared
  with no timing was stored as `CREATE TRIGGER t_default_timing INSERT ON a1
  BEGIN SELECT 1; END`. *Observed again, for the seventy-first edition,* as
  *Observations* above states: `create trigger t1 after insert on x …` was
  stored as `CREATE TRIGGER t1 after insert on x …`, and
  `Create Trigger If Not Exists t2 Before Update Of a, "b" On x …` as
  `CREATE TRIGGER t2 Before Update Of a, "b" On x …`; `CREATE TRIGGER main.t3`
  lost its qualifier while `ON main.x` kept it; and `CREATE TEMP TRIGGER t5`
  was stored in the `temp` schema as `CREATE TRIGGER t5`.

  *Why the whole statement and not the body.* MariaDB's action statement is the
  body alone. Cutting the body out of the stored text means finding the
  `BEGIN` that opens it after a `WHEN` clause that may itself hold one as an
  identifier — a parse of the expression grammar. The whole statement carries
  the `WHEN` clause and the `UPDATE OF` list that no property carries, and a
  template emitting DDL emits it as it stands. `database.provider` tells a
  template which of the two it holds.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the seventy-first edition,* for rmp `#317`: a trigger's
  `tbl_name` matches its table under `BR-LITE-003`. SQLite writes it as the
  trigger's `ON` clause wrote the table's name: observed, `VESSEL` for a
  trigger on `Vessel`, which an exact comparison would have dropped at exit
  `0`. The trigger is carried under the table it matches; its own properties
  are unchanged. The keywords of the header are recognised ignoring ASCII
  letter case, and the *Source* is corrected: SQLite upper-cases only the
  leading `CREATE TRIGGER`, so a trigger written `after insert` keeps those
  words as written, and a derivation that recognised upper case alone would
  have ended the read of a file SQLite wrote with a false `70`. A header
  SQLite does not write is refused before the read by the seventh kind of
  `FR-LITE-041`.

- **FR-LITE-033**: For a `sqlite` read, a view SHALL carry exactly the nine
  properties `FR-CAT-047` gives a view, and no other: `name`; `definition`, the
  `sql` field of its `sqlite_schema` row — the whole stored `CREATE VIEW`
  statement, unchanged; and `null` for `check_option`, `is_updatable`,
  `definer`, `security_type`, `character_set_client`, `collation_connection`
  and `algorithm`. A view SHALL NOT carry its columns, and the system SHALL
  NOT read them: no statement reads from a view or compiles one, per
  `FR-LITE-015`. A view whose definition does not compile SHALL be read as
  every other view is, and SHALL NOT be marked, per `FR-LITE-036`.

  **The view object is the same on both providers.** A `mariadb` read carries
  no column of a view either: `FR-CAT-047` gives a view nine properties and no
  columns, and the rows the column catalogue returns for a view are presented
  nowhere, per the note on `FR-CAT-052`. A template reads the same keys from a
  view whichever provider the document names, and only their values differ.

  *Why the whole statement.* The `SELECT` follows an optional column list and
  the keyword `AS`, and the stored text is the only text SQLite keeps. MariaDB
  rewrites its own definition, per `FR-CAT-047`, so neither provider gives a
  template the author's body.

  *Provenance.* The user's decision of 2026-11-03, relayed for rmp `#317`, to
  read no view's columns and to mark no view, on four grounds:

  1. **Parity with MariaDB.** No requirement asks a `mariadb` read whether a
     view compiles. The one marking a view takes there is `FR-PRIV-011`'s, for
     a definition the reader cannot see.
  2. **Nothing is missing from the document.** Every property this requirement
     gives a view is read, whether the view compiles or not.
  3. **No statement of the read compiles a view.** A view's definition is SQL
     the file's author wrote, and the library resolves none of the tables,
     functions and modules it names during a read, per `FR-LITE-015`. What
     `FR-LITE-039` counts as reachable narrows to the parse of the schema;
     since the sixty-ninth edition no virtual table is connected either, per
     the observation below.
  4. **The cache and `--context` treat the read as any complete read.** No
     object is marked, so the read holds nothing `FR-CACHE-037` keeps out of
     the cache, and `FR-PRIV-008` accepts its dump as a `--context`.

  *Observed, on 2026-11-03, through the Python library 3.54.1, with the file
  opened read-only and `trusted_schema` off.* Over a database holding a view that
  selects from a missing table, one that calls a function the library lacks,
  one that calls `bm25()`, which `trusted_schema` off refuses, and one over an
  `fts5` table, the probe and the six catalogue statements the read issues
  each succeeded. `pragma_table_list` reported a column count of `0` for every
  view, the one that compiles included, a reading the correction below
  withdraws; a statement reading the columns of the views failed with
  `unsafe use of bm25()`. The first statement invoking `pragma_table_list`
  ran the nested statement `-- SELECT k, v FROM 'main'.'docs_config'`: the
  library
  connected the `fts5` table to list it, and its module read a table of the
  file, with no view read. A statement reading the columns of the view over
  that table, on an open of the file that had not yet listed the objects, ran the
  same nested statement. With the statements of `FR-LITE-015` as the
  sixty-ninth edition writes them, which invoke no `pragma_table_list`, no
  nested statement but a pragma function's own ran, per the observation under
  `FR-LITE-018`: ground 3 now removes both the compile of a view's definition
  and a virtual table's module code.

  *Rejected: keeping the marking of the sixty-fifth edition, with the column
  read it rested on.* A document that lacks nothing would be refused as a
  `--context`, per `FR-PRIV-008`, and its read would never be recorded whole
  in the cache, per `FR-CACHE-037`, for a view every property of which was
  read; and the read would compile SQL from an untrusted file to learn a fact
  the model does not carry.

  *Rejected: a `columns` key on the view object of both providers.* It adds a
  key to every view of every MariaDB document, which `FR-CTX-043` makes the
  change of `provider` alone — the ground `BR-LITE-002` gives for not carrying
  `WITHOUT ROWID` and `STRICT`. *Rejected: a `columns` key on a view of a
  `sqlite` read alone.* One object would take two shapes by provider, against
  the Overview of this file: a template reading `view.columns` would fail the
  render under `FR-SEM-012` on every MariaDB document, and reading a view
  would first need a test of `provider`, which `FR-CTX-043` provides for a
  template to branch on, not as a condition of reading an object.

  *Accepted cost, stated because it is silent.* A view that does not compile
  is no longer flagged. It is presented as every other view is, at exit `0`,
  in a listing, in a dump and in answer to a request that names it, and
  nothing in the document tells it from a view that compiles. A template that
  emits code querying it produces code that fails at the database, not at
  `tpl`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The requirement gave a
  view its columns, read from `pragma_table_xinfo` and decomposed as
  `FR-LITE-025` through `FR-LITE-027` decompose a table's, and the view object
  of `FR-CAT-047` has no key to carry them, for either provider. By the user's
  decision above, a view carries the nine properties alone, which is what the
  Overview of this file already stated, and the system no longer reads a
  view's columns at all; `FR-LITE-035`, whose marking that read served, is
  withdrawn. The observations of what a view's column reports, and that a
  view's columns are known only by compiling it, ground nothing now and are
  removed.

  *Corrected in the sixty-ninth edition,* for rmp `#317`. The observation
  above read a column count of `0` for every view as showing that listing a
  view does not compile it, and ground 3 rested on that reading. The count was
  the listing stopping at the first view it could not compile, which the
  database observed held, and `pragma_table_list` did compile views, per
  `FR-LITE-015`. Ground 3 holds again as written because, by the user's
  decision recorded under `FR-LITE-020`, no statement invokes that function;
  it rests on the closed list now, not on that reading, which is withdrawn.
  The decision of this requirement, the view object, the absence of a
  marking, and what the cache and `--context` do with the read are
  unchanged.

  *Note added in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. The observation speaks of
  the file opened read-only and of an open of the file, where it said
  connections. The requirement is unchanged.

- **FR-LITE-034**: For a `sqlite` read, every table's `CHECK` constraints SHALL
  be `[]`. The system SHALL NOT read a `CHECK` constraint from the stored
  text.

  *Rationale.* No statement of `FR-LITE-015` reports one: SQLite keeps a
  `CHECK` constraint only inside the `CREATE TABLE` text, at column level or
  among the table constraints, with an optional name, and reading it is the
  parse `BR-LITE-002` declines.

  *Accepted cost, stated because it is silent.* A table declaring `CHECK`
  constraints is presented with none, at exit `0`, and the document cannot
  tell it from a table that declares none. `FR-CAT-015` holds for a `mariadb`
  read and is not met by a `sqlite` one.

  *What would change this.* An amendment admitting a reader of the constraint
  list of the stored `CREATE TABLE` text, with its fixture cases, which would
  also make the constraint names of `FR-LITE-031` and the generation
  expressions of `FR-LITE-025` reachable.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## Completeness

- **FR-LITE-035**: *Withdrawn in the sixty-eighth edition,* for rmp `#317`.
  This requirement read the columns of every view to learn whether its
  definition compiled, marked a view that did not with
  `"restricted":["columns"]` in a listing or a dump, answered a request naming
  it with `77`, and read each view in a statement of its own when the
  statement reading them all failed. By the user's decision of 2026-11-03,
  recorded under `FR-LITE-033`, the system reads no view's columns and
  compiles no view, per `FR-LITE-015`, so a view that does not compile is read
  whole and marked nowhere, per `FR-LITE-036`. The identifier is retired and
  SHALL NOT be reused.

- **FR-LITE-036**: A `sqlite` read SHALL be complete or SHALL fail, and no
  object it presents SHALL carry a `restricted` marking. A fault that prevents
  the schema from loading SHALL fail the whole read under `FR-LITE-007`, and
  SHALL NOT be presented as an object with a `restricted` marking. A view
  whose definition does not compile SHALL be presented whole and unmarked:
  every property `FR-LITE-033` gives it is read. SQLite has no privileges, so
  no property is withheld from the reader, and the cross-checks of
  [privileges-and-completeness.md](privileges-and-completeness.md) have no
  case to detect.

  *Observed.* A schema the library refuses fails every statement that touches
  the file, including the probe, so no object can be read apart from it.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The requirement
  excepted the view `FR-LITE-035` marked, and that requirement is withdrawn by
  the user's decision recorded under `FR-LITE-033`. A `sqlite` read now
  presents no incomplete object, and the sentence on a view that does not
  compile states the case the exception used to take.

## Requirements of form

- **FR-LITE-037**: For a `sqlite` entry, the requirements of form of
  [performance-requirements.md](performance-requirements.md) SHALL hold as
  they hold for a server: the number of catalogue statements a full read and a
  named read issue SHALL NOT depend on the number of objects; an invocation
  SHALL open the database file at most once, per `NFR-PERF-004`; and a
  cache hit SHALL NOT open it and SHALL issue none, per `NFR-PERF-003`.

  The equality of `NFR-PERF-001` SHALL be verified, for a `sqlite` entry, over
  the fixture database of `FR-LITE-038` and over an empty database file, with
  the statements counted under `FR-LITE-018`.

  *Observed.* Eight statements — the probe and seven catalogue statements —
  read the observation database of 32 schema objects and an empty file of zero
  bytes alike, per `FR-LITE-015`. *Observed again, for the sixty-eighth
  edition,* under `FR-LITE-033`: seven statements — the probe and the six
  catalogue statements that remain — read a database holding views that do
  not compile and an empty file of zero bytes alike.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the sixty-seventh edition,* for rmp `#314`. How the clause of
  `NFR-PERF-004` is verified for a `sqlite` entry, and on which targets, is
  stated in that requirement; how the count of `NFR-PERF-008` is verified for
  one is stated in that requirement.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The count is constant
  without exception: the path on which `FR-LITE-035` read each view in a
  statement of its own, the one path on which the count grew with the
  objects, is gone with that requirement. A full read issues the probe and
  six catalogue statements, one fewer than before, because no statement reads
  the columns of views.

  *Amended in the sixty-ninth edition,* for rmp `#317`. A full read issues
  the probe and seven catalogue statements — the check of `FR-LITE-041` and
  the six that read the catalogue — one more than before, and the count stays
  independent of the objects. Each statement now joins the schema table to
  the function it reads through, and none invokes `pragma_table_list`, so the
  time each takes is linear in the objects, per the figures under
  `FR-LITE-009`; this requirement fixes how many statements a read issues,
  not how long one takes.

  *Note added in the seventieth edition,* for rmp `#317`. A full read also
  issues the `BEGIN DEFERRED` and the `COMMIT` of `FR-LITE-042`, once each.
  Neither is a catalogue statement, per `FR-LITE-015`, so neither enters the
  count this requirement fixes, and both are issued once whatever the
  objects.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. An invocation SHALL open
  the database file at most once, and a cache hit SHALL NOT open it, where the
  text said at most one connection to the file and none, and defined a
  connection of the library to the file; the definition is withdrawn, because
  [glossary.md](glossary.md#opening-the-database-file) holds the term. The
  requirement holds as before.

## The fixture

- **FR-LITE-038**: The fixture of `scripts/sqlite/` SHALL build, from versioned
  SQL text and at test time, the SQLite databases the tests of this file read,
  and SHALL NOT be committed as a binary database file. Its main database SHALL
  model a realistic domain and SHALL exercise every case this file states:

  1. ordinary rowid tables, a `WITHOUT ROWID` table and a `STRICT` table;
  2. declared types yielding each of the five affinities, a column with no
     declared type, and a `STRICT` column declared `ANY`;
  3. every row of the default classification of `FR-LITE-027`, including a
     parenthesised default and `DEFAULT NULL`, on nullable and on `NOT NULL`
     columns;
  4. a rowid alias with and without `AUTOINCREMENT`, and each of the four
     integer primary keys `FR-LITE-028` observes as not an alias;
  5. a composite primary key;
  6. `UNIQUE` constraints at column and table level, `CREATE INDEX` unique and
     not, a descending member, a collated member, a partial index, an
     expression index and an index on a generated column;
  7. a virtual and a stored generated column;
  8. foreign keys carrying each of the five referential actions on update and
     on delete, a composite key, a key naming no parent column, a key declared
     `MATCH FULL`, a key naming a table the database does not hold, a key
     naming no parent column of a table whose primary key has another number
     of columns, and a self-reference;
  9. `CHECK` constraints at column and table level, one named, and one
     calling a function the carried library holds and does not mark
     innocuous, written under `trusted_schema` on;
  10. views, one with a column list, one selecting from a missing table, one
      calling a function the carried library does not hold, and one calling a
      function the carried library holds and does not mark innocuous, which
      `trusted_schema` off refuses — on the builds observed, `bm25()` over an
      `fts5` table and `rtreedepth()` are two;
  11. triggers of each timing on each event, one with no timing written, one
      `UPDATE OF` a column list, one with a `WHEN` clause, one with a quoted
      name, one whose timing and event are written in lower case, one named
      `temp`, and an `INSTEAD OF` trigger on a view;
  12. a virtual table with shadow tables, `sqlite_sequence` and `sqlite_stat1`;
  13. identifiers carrying a space, a backtick and a double quote;
  14. a name written in another ASCII letter case than its object declares
      it, in each place `BR-LITE-003` names and in an index: the table in a
      trigger's `ON` clause, the view in an `INSTEAD OF` trigger's, the
      parent table and a parent column of a foreign key, and a member of an
      index; and two tables whose names differ only in a letter outside
      ASCII, with a foreign key naming one of them in another case.

  It SHALL also produce, beside the main database: a copy in
  write-ahead-log mode; an empty file; a text file; a truncated database; a
  directory, a FIFO, a symbolic link to the main database and a dangling one;
  a database whose schema holds a statement longer than the length limit of
  `FR-LITE-011`, written through an open of the file that does not set that limit; a
  database whose only objects are the internal tables; a database whose
  schema holds a view named `pragma_table_list`, and one whose schema holds a
  table named `PRAGMA_TABLE_XINFO`; a database whose schema table holds the
  `sql` of a view as a BLOB, one whose schema table holds the `type` of a
  trigger as a BLOB, one whose schema table holds a `CREATE TABLE` row with a
  `rootpage` of `0`, one whose schema table holds a `CREATE VIRTUAL TABLE`
  row with a `rootpage` other than `0`, one whose schema table holds a table
  named `pragma_encoding`, holding one row, whose `type` is `TABLE`, one
  whose schema table holds an ordinary table whose `type` is `Table`, one
  whose schema table holds a trigger whose `type` is `trigger` followed by a
  NUL character, one whose schema table holds, for an index the database
  holds, a second row with no text whose `type` is `table`, one whose schema
  table holds such a row whose `sql` is the empty text and whose `type` is
  `view`, one whose schema table holds a trigger whose `tbl_name` is its
  table's name followed by a NUL character, one whose schema table holds a
  view whose `sql` is its definition followed by a NUL character and further
  text, and one whose schema table holds a trigger whose text begins
  `CREATE TEMP TRIGGER`, the last twelve written under `writable_schema` through an
  open of the file that does not set the controls of `FR-LITE-010`; and a
  database holding an `fts5` table, a view over it that compiles, and no
  view that does not. A read of each of the fourteen databases named like a
  pragma function or written under `writable_schema` SHALL end with `78`
  under `FR-LITE-041`; the last database is the one `FR-LITE-018` records
  statements over.

  It SHALL also produce three databases whose names hold bytes that are not
  valid UTF-8: one holding two tables whose names differ only in such a
  byte; one holding a table with two columns whose names differ only in such
  a byte; and one holding a table whose name holds such a byte, and a second
  table with a foreign key whose parent table's name differs from the first
  table's only in another such byte. A read of each of the first two SHALL
  end with `78` under `FR-LITE-043`, and a read of the third SHALL succeed,
  carrying the first table under its name with U+FFFD and excluding the key
  as dangling under `FR-LITE-021`.

  The three views of item 10 that do not compile are the cases of
  `FR-LITE-033` and `FR-LITE-036`: a read of the main database SHALL succeed
  over them, mark none of them, and present each with its nine properties.
  A statement that named one of them would fail, per `FR-LITE-018`, so that
  success also shows that no statement names a view; and `FR-LITE-015`
  admits no statement that compiles one, so the function `trusted_schema` off
  refuses is never run.

  *Rationale.* This is the obligation `FR-CONF-038` and the root `CLAUDE.md`
  place on `scripts/mariadb/` for the catalogue a server reads, written for
  the catalogue a file holds. A database built from text is reviewable in a
  diff and buildable by the library the binary carries; a committed binary
  file is neither. The modelled domain is realistic for the reason the root
  `CLAUDE.md` gives, and the observation of this file was taken in the
  freight domain the MariaDB fixture models.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. The database "whose
  `CHECK` makes the schema fail to load" is replaced by the length-limit
  database. A `CHECK` calling a function not marked innocuous does not stop
  the schema loading on the library observed, per `FR-LITE-010`, so that
  database could not have exercised the refusal it was listed for; the length
  limit does stop it, per `FR-LITE-007`, and a connection that does not set
  the limit can write it. The `CHECK` itself moves into the main database, as
  a case of the corrected statement: the schema loads, and the table that
  holds the constraint is read. Item 10 gains the view that `trusted_schema`
  off refuses, the third cause `FR-LITE-035` names, which no case exercised.

  *Amended in the sixty-eighth edition,* for rmp `#317`. The paragraph on the
  views of item 10 that do not compile is new. They stay in the fixture, and
  what they exercise changes with the user's decision recorded under
  `FR-LITE-033`: they no longer prove a marking, which `FR-LITE-035` made and
  which is withdrawn, but that the read succeeds over them, marks nothing,
  and compiles no view.

  *Amended in the sixty-ninth edition,* for rmp `#317`. Seven databases are
  new. Six are the kinds of row `FR-LITE-041` refuses: a view and a table
  named like a pragma table-valued function, the second in another letter
  case; a view's `sql` and a trigger's `type` stored as BLOBs; and a root page
  that contradicts its row's text, in each direction — the last four written
  the only way SQLite admits them. The seventh, an `fts5` table with a view
  over it and no view that does not compile, is where `FR-LITE-018` sees
  whether a statement connects a virtual table: over the main database, a
  statement invoking `pragma_table_list` stopped at a view that does not
  compile before it reached the `fts5` table, so the main database could not
  show it. Items 1, 2 and 12 are unchanged and now exercise what the user's
  decision recorded under `FR-LITE-020` makes of them: the shadow tables of
  item 12 are covered as tables, and the `ANY` column of item 2 takes
  `numeric`. The paragraph on the views of item 10 says that the read's
  success shows that no statement names a view; that no statement compiles
  one is `FR-LITE-015`'s, verified under `FR-LITE-018`.

  *Amended in the seventy-first edition,* for rmp `#317`. Item 14 is new: the
  main database read under `BR-LITE-003` shows that a trigger, an index and a
  foreign key naming their objects in another letter case are attached to
  them, and that a key naming one of two tables that differ outside ASCII is
  attached to that one. Item 8 gains the key `FR-LITE-021` now excludes for
  the number of its parent columns, and item 11 a trigger whose header
  SQLite keeps in lower case, read under `FR-LITE-032`, and one named
  `temp`, which the seventh kind of `FR-LITE-041` admits. Eight databases are
  new, each written the only way SQLite admits it. The two second rows with
  no text — `NULL` typed `table`, which the statements of the seventieth
  edition read as a table with no column, and the empty text typed `view` —
  are the fifth kind of row `FR-LITE-041` refuses; the trigger attached by a
  name ending in a NUL character, which matched no table, and the view whose
  definition carried text after a NUL character are the sixth; the trigger
  whose text begins `CREATE TEMP TRIGGER` is the seventh. The other three
  are the fourth kind. The table `pragma_encoding` typed `TABLE`
  forged the encoding at exit `0` under the seventieth edition; the table
  typed `Table` was missing from the model at exit `0`; and the trigger whose
  `type` ends in a NUL character, missing from the read at exit `0` too,
  shows that the comparison takes the whole value, which a comparison
  folding letter case alone does not, per the rejected option under
  `FR-LITE-041`.

  *Amended in the seventy-ninth edition,* for rmp `#349`. The paragraph on
  the three databases whose names hold bytes that are not valid UTF-8 is new.
  The first two are the refusals of `FR-LITE-043`, and the first is the case
  of rmp `#349`, in which each of the two tables carried the other's columns
  at exit `0`. The third shows that a name holding such a byte is read, and
  that a reference is matched on the bytes and not on the text they convert
  to. They are built apart from the main database so that no name of it
  carries U+FFFD.

  *Amended in the ninetieth edition,* for rmp `#373`: one meaning of
  connection, per the user's decision of 2026-11-06. Two databases of the list
  are written through an open of the file that does not set a limit or the
  controls, where the text said a connection. The fixture is unchanged.

## Dependencies

- [configuration-model.md](configuration-model.md) — `FR-CONF-051` through
  `FR-CONF-054`: the `provider` and `path` keys, the keys each provider
  refuses, and how a relative `path` resolves.
- [server-contract.md](server-contract.md) — the contract this file stands
  beside, and `FR-SRV-042`, `FR-SRV-011` and `BR-SRV-003`, whose grounds it
  adopts, and `FR-SRV-019`, whose form `FR-LITE-039` adopts.
- [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) — the model and the document
  this file fills, `FR-CAT-047`, the view object `FR-LITE-033` fills, and
  `FR-CTX-043`, the `provider` field.
- [privileges-and-completeness.md](privileges-and-completeness.md) —
  `FR-PRIV-001`, under which `FR-LITE-022` and `FR-LITE-036` hold a `sqlite`
  read complete, and `FR-PRIV-008`, which accepts its dump as a `--context`
  because nothing in it is marked.
- [cache-commands.md](cache-commands.md) — `FR-CACHE-037`, which keeps no
  object of a `sqlite` read out of the cache, because none is marked.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `70`, `74` and
  `78`, and `FR-ERR-030`, which `FR-LITE-012` and `FR-LITE-040` reach; and
  `FR-ERR-046`, which bounds a value of the file in a diagnostic line.
- [global-flags.md](global-flags.md) — `FR-GLOB-018`, which fixes what a
  `cause` carries of what the library returned.
- [project-and-discovery.md](project-and-discovery.md) — `FR-PROJ-024`, which
  admits the two files of `FR-LITE-008`.
- [security.md](security.md) — `FR-SEC-028`, which points here.
- [cfg-commands.md](cfg-commands.md) — `FR-CFG-055` and `FR-CFG-056`.
- [template-environment.md](template-environment.md) — `FR-ENV-048`, and
  `FR-ENV-041`, whose `unique` test consults the primary key of
  `FR-LITE-029`.
- [performance-requirements.md](performance-requirements.md) — `NFR-PERF-001`
  through `NFR-PERF-004`, which `FR-LITE-037` applies.

## Open questions

**None.** Every point the provider raised is settled in this file or in the
files it names, and the index of [open-questions.md](open-questions.md) holds
no entry for it.

*Corrected in the seventy-third edition.* The sentence read that the index
"stays empty", which stopped being true when that edition opened two entries
for the `mysql` provider; neither concerns a `sqlite` entry.
