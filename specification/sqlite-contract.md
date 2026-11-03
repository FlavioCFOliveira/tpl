---
title: SQLite Contract
status: approved
last-reviewed: 2026-11-03
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
applied, the closed statement list and the order of the connection start, the
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
  before it on the same connections succeeded. `FR-LITE-017` places it for
  that reason.

  *Why the object keeps its name.* `server` is a structural key of the
  document, present in every document, per `FR-CTX-031` and `BR-SRV-008`. A
  `sqlite` document without it would fail the contract `FR-RND-020` checks,
  and a renamed key would be a breaking change under `FR-OUT-014` for every
  MariaDB document. The object describes what read the catalogue, and for a
  `sqlite` entry that is the library.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-003**: The supported version window of `FR-SRV-001`, the refusal of
  `FR-SRV-020`, the marked read of `FR-SRV-031` and the product check of
  `FR-SRV-003` SHALL NOT apply to a `sqlite` entry. The library the binary
  carries is the one supported release, and `standing` SHALL be `supported`
  on every `sqlite` read. The carried library SHALL provide every pragma
  table-valued function `FR-LITE-015` names; `pragma_table_list`, the newest of
  them, first appeared in SQLite 3.37.0, per the official documentation of
  `PRAGMA table_list` at `https://sqlite.org/pragma.html`, checked 2026-11-03.

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
  The library parses every `CREATE` statement when it loads the schema, and
  compiles a view whenever its columns are read, per `FR-LITE-033`. A fault
  that needs an attacker's SQL is therefore reachable wherever parsing or
  compiling that SQL reaches it, although `tpl` itself issues only the
  statements of `FR-LITE-015`.

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
  `unable to open database file` (`SQLITE_CANTOPEN`) and created nothing. On a
  connection opened read-only, `CREATE TABLE` and `INSERT` failed with
  `attempt to write a readonly database` (`SQLITE_READONLY`).

  *Added in the sixty-fifth edition,* for rmp `#314`.

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
  | The database is in write-ahead-log mode and its directory refuses the files that mode needs, per `FR-LITE-008` | `SQLITE_READONLY_DIRECTORY` | `74` |
  | A read of the file fails part-way | `SQLITE_IOERR` | `74` |
  | The database stays locked by a writer until the catalogue-query deadline expires, per `FR-LITE-009` | `SQLITE_BUSY` | `69` (`EX_UNAVAILABLE`) |

  The `cause` SHALL name the path as resolved, the statement, the open or the
  check of the file's kind that failed, and what the library returned,
  including the object and the reason where the library names them — or, for
  the check of the kind, what the filesystem returned. For `74` and `78` the
  `hint` SHALL carry `tpl cfg database update <entry> --path <path>`.

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

- **FR-LITE-008**: The system SHALL NOT write to the database file. WHERE the
  database is in write-ahead-log mode, the library MAY create the two files
  that mode keeps beside the database — `<file>-wal` and `<file>-shm` — when
  they do not exist, and they MAY remain after the invocation. The system SHALL
  NOT create, change or remove any other file outside `.tpl/` on account of a
  `sqlite` read.

  *Observed.* A read-only connection to a database in write-ahead-log mode
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
  which keeps the log index in memory.* Observed: a read-only connection
  refused it with `disk I/O error` (`SQLITE_IOERR_LOCK`). *Rejected: refusing
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

- **FR-LITE-009**: For a `sqlite` entry, every statement the system issues on
  the connection SHALL run under the catalogue-query deadline of
  `FR-CONF-005`, resolved from `core.query_timeout`, and a statement that finds
  the database locked SHALL wait for it within that deadline and no longer.
  The phases of DNS resolution, TCP connect, TLS handshake and
  `password_command` SHALL NOT run, and `core.connect_timeout` and
  `core.password_timeout` SHALL have no effect on the entry.

  *Observed.* With a writer holding an exclusive transaction on a database in
  rollback-journal mode, a reader's first statement failed with
  `database is locked` (`SQLITE_BUSY`) once its wait expired. In
  write-ahead-log mode the same reader was not blocked and saw the state last
  committed.

  *Why `69`.* `FR-ERR-027` gives a catalogue-query deadline `69`, and the
  caller's next step is the one that row of `FR-ERR-001` states: the
  operation is read-only and therefore repeatable.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-040**: For a `sqlite` entry, every
  [result code](glossary.md#result-code) by which the library reports a
  failure SHALL be classified, and the failure SHALL end the invocation with
  exactly one code of `FR-ERR-001`, save where `FR-LITE-035` marks a view
  instead. The classification SHALL be the first of the following that names
  the result code:

  1. `FR-LITE-012`, at the stage that applies or confirms a control or a
     limit, whatever the library returned there;
  2. `FR-LITE-035`, for `SQLITE_ERROR` on a statement that reads the columns
     of views;
  3. the rows of `FR-LITE-007`, each by its result code, at the open or at any
     statement. The *Observed as* column records where a code was observed and
     does not bound where its row applies, so the row of `SQLITE_CANTOPEN`
     also takes a file beside the database that the library cannot open, and
     the row of `SQLITE_BUSY` a lock the library returns without waiting out
     the catalogue-query deadline;
  4. the table below, by the primary result code, under any extended code
     unless a row names one.

  | Result code | What the library reports | Code |
  |---|---|---|
  | `SQLITE_MISUSE`, `SQLITE_RANGE`, `SQLITE_NOTFOUND` | A use of the library by `tpl` that the library refuses: an interface called in a way it does not support, a parameter or column number out of range, or an operation code it does not recognise | `70` (`EX_SOFTWARE`) |
  | `SQLITE_ERROR`, anywhere but on a statement of `FR-LITE-035` | An error in a statement of `FR-LITE-015`, which `tpl` wrote and issued over a file whose schema has loaded, or in how `tpl` opened the connection | `70` |
  | `SQLITE_INTERNAL` | A malfunction inside the library the binary carries, per `FR-LITE-001` | `70` |
  | `SQLITE_ABORT`, `SQLITE_AUTH`, `SQLITE_CONSTRAINT`, `SQLITE_LOCKED`, `SQLITE_MISMATCH`, and `SQLITE_READONLY` with no extended code of its own | An outcome that only an operation `tpl` does not perform produces: a write or a constraint on one, an abort or a rollback it requests, an authorisation callback, a plan a virtual-table module refuses, or a conflict inside its one connection or with a second connection sharing its cache | `70` |
  | `SQLITE_EMPTY`, `SQLITE_FORMAT`, `SQLITE_NOTICE`, `SQLITE_WARNING` | A code the library documents as not used, or as returned by no interface | `70` |
  | `SQLITE_NOMEM` | An allocation the library needed failed | `74` (`EX_IOERR`) |
  | `SQLITE_FULL`, `SQLITE_NOLFS`, `SQLITE_PERM` | The disk is full; the filesystem cannot hold a file of the database's size; or it did not grant the access mode the library asked for a file it creates beside the database | `74` |
  | `SQLITE_READONLY` under an extended code other than `SQLITE_READONLY_DIRECTORY` | A file the read needs beside the database — the shared memory of the write-ahead log, or a journal to roll back — cannot be written, a recovery needs write access the connection does not have, or the database file moved after it was opened | `74` |
  | `SQLITE_PROTOCOL` | In write-ahead-log mode, other connections kept winning the race for the database's locks, and the library gave up after retrying | `69` (`EX_UNAVAILABLE`) |
  | `SQLITE_SCHEMA` | Another connection kept changing the schema while a statement ran, past the library's own re-preparation of it | `69` |
  | `SQLITE_INTERRUPT` | The statement was ended at the catalogue-query deadline, per `FR-LITE-009` and `FR-ERR-027` | `69` |
  | A primary result code that no row of this table or of `FR-LITE-007` names | A code SQLite had not published on the date below | `70` |

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
  which is the step `74` states. Where another connection holds or changes the
  database, the next step is the one `FR-LITE-009` gives a lock: the operation
  is read-only and therefore repeatable, which is `69`. A primary result code
  SQLite publishes after the date above is an outcome the system was not built
  to classify, and `FR-ERR-030` declines to continue past one.

  *Observed, on 2026-11-03, through the Python library 3.54.1, on a read-only
  connection with `trusted_schema` off.* A database whose schema calls a
  function and names a collation that the reading library does not hold — in
  a generated column, a `CHECK`, a `DEFAULT`, a collated index, a partial
  index and an expression index — loaded, and statements of each kind the
  last row of `FR-LITE-015` names, written for the observation, read its
  tables, columns, indexes, index members, foreign keys and schema rows
  without error. On a file whose schema has loaded, only a statement that
  reads the columns of views was observed to fail with `SQLITE_ERROR`, which
  is `FR-LITE-035`'s case; that is the ground for reading `SQLITE_ERROR`
  anywhere else as an error in the statement and not in the file.

  *Rationale for the two departures from the split the technical
  specification recommended,* `70` for a misuse of the library by `tpl` and
  `74` for every other code. The split is followed except in two places, each
  decided by `FR-ERR-002`: `SQLITE_PROTOCOL` and
  `SQLITE_SCHEMA` are `69`, because what the caller has to look at is another
  connection and not the filesystem, and the read is repeatable; and
  `SQLITE_LOCKED` is `70`, because with one connection and no write the only
  conflict it can report is one `tpl` made.

  *Rejected: `70` for every result code the rows above do not classify.* A
  host out of memory, a full disk or a database another process holds would be
  reported as a defect of `tpl`. *Rejected: `74` for every one.* A use of the
  library that `tpl` got wrong would send the caller to its filesystem.
  *Rejected: `69` for every one, by reading `FR-ERR-036` onto a `sqlite`
  entry.* That requirement's condition is a session that stops answering,
  which a library inside the process cannot do, and its `69` would tell a
  caller to repeat a read that fails on a defect every time. *Rejected: `78`
  for `SQLITE_ERROR` outside `FR-LITE-035`, as a fault of the file.* Per the
  observation above, no file whose schema loads makes those statements fail.

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

## Hardening against an untrusted file

- **FR-LITE-010**: The database file SHALL be treated as untrusted input, per
  the trust boundaries of [security.md](security.md). On every connection it
  opens, before the version probe, the system SHALL apply each of the
  following controls and SHALL confirm, by reading the setting back, that each
  took effect. Extension loading is the one control that is not a setting of
  the connection: it is a property of the library the binary carries, applied
  when the library is built, and its read-back is `PRAGMA compile_options`,
  which the system SHALL issue before any statement that reads the file. The
  control is in effect only WHERE that statement returns the row
  `OMIT_LOAD_EXTENSION`; IF the row is absent, the control has not taken
  effect, under `FR-LITE-012`. Temporary storage is a setting of the
  connection whose effect also depends on how the library is built, so it is
  confirmed twice: it is in effect only WHERE `PRAGMA temp_store` reads back
  `2` and `PRAGMA compile_options` returns the row `TEMP_STORE=1`,
  `TEMP_STORE=2` or `TEMP_STORE=3`; IF that statement returns `TEMP_STORE=0`,
  or no `TEMP_STORE` row, the control has not taken effect, under
  `FR-LITE-012`.

  | Control | Setting | Closes |
  |---|---|---|
  | Read-only open | read-only, never creating the file, per `FR-LITE-006` | any write through the connection |
  | `query_only` | on | any write the open itself did not refuse, as defence in depth |
  | Defensive mode | on | the features by which ordinary SQL can corrupt a database file |
  | `trusted_schema` | off | an SQL function or a virtual table that is not marked innocuous, run from a view, a trigger or a schema expression |
  | Extension loading | absent: the carried library is built with SQLite's compile-time option `SQLITE_OMIT_LOAD_EXTENSION`, which omits the mechanism, its library interface and the `load_extension()` function | code loaded from a path a schema or a statement names |
  | `cell_size_check` | on | a malformed page read without its cell sizes checked |
  | `mmap_size` | `0` | a page read through a memory map of a file another process can change |
  | `temp_store` | `MEMORY` (`2`) | a temporary file, written outside `.tpl` with content of the database, when a statement sorts, materialises a subquery or builds a transient index |

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

  *Observed.* Each setting read back as set, on a read-only connection: the
  pragmas `query_only` `1`, `trusted_schema` `0`, `cell_size_check` `1` and
  `mmap_size` `0`, and the configuration flags `defensive on`,
  `load_extension off` and `trusted_schema off` through the shell's
  configuration report. A `query_only` connection opened read-write refused
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
  bundles it starts every connection with the switch on: compile the mechanism
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
  with `unsafe use of bm25()` and `unsafe use of rtreedepth()`, which is
  `FR-LITE-035`'s case. A `CHECK` constraint, a `DEFAULT` and a trigger body
  calling such a function did not affect the read: the schema loaded and the
  table's columns were read. Creating such a `CHECK` was refused with
  `unsafe use of rtreedepth()` on a connection with `trusted_schema` off and
  succeeded on one with it on, so a database holding one was written under
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

  *Rejected: disabling views, as the guidance offers for an application that
  uses none.* The columns of a view are read by compiling it, per
  `FR-LITE-033`. *Rejected: disabling triggers.* No statement this system
  issues can fire one, per `FR-LITE-016`, and reading a trigger's definition
  compiles nothing; the control would close nothing that is open.

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
  SQLite's default `TEMP_STORE=1`, on a read-only connection under
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

- **FR-LITE-011**: On every connection, before the version probe, the system
  SHALL set the following run-time limits of the library and SHALL confirm
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
  | `PRAGMA query_only`, `PRAGMA trusted_schema`, `PRAGMA cell_size_check`, `PRAGMA mmap_size` and `PRAGMA temp_store`, each in the form that sets the value `FR-LITE-010` fixes and in the form that reads it back | The controls of `FR-LITE-010` that a statement applies | Once each, at connection start |
  | `PRAGMA compile_options`, in the form that reads the options | The confirmation of the extension-loading control of `FR-LITE-010`, and of the build on which its temporary-storage control depends | Once, at connection start |
  | `SELECT sqlite_version()`, reading nothing else | The version probe of `FR-LITE-002` | Once, at connection start |
  | `SELECT` against the schema table `sqlite_schema` and the pragma table-valued functions `pragma_table_list`, `pragma_table_xinfo`, `pragma_index_list`, `pragma_index_xinfo`, `pragma_foreign_key_list` and `pragma_encoding`, restricted to the schema `main` | Reading the catalogue, and the probe of `FR-CFG-056` | As the command requires |

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

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`: the second row is
  new, for the confirmation `FR-LITE-010` now requires, and the catalogue row,
  formerly the third, is named the last.

  *Amended in the sixty-seventh edition,* for rmp `#314`: the first row names
  `PRAGMA temp_store`, and the second the confirmation of the build it
  depends on, for the control `FR-LITE-010` adds. Both read nothing from the
  file, per the observation under that requirement.

- **FR-LITE-016**: The system SHALL NOT issue any other statement for a
  `sqlite` entry. It SHALL issue no DDL, no DML, no `ATTACH` or `DETACH`, no
  `VACUUM`, `ANALYZE` or `REINDEX`, no `PRAGMA` outside the list of
  `FR-LITE-015`, no statement against a schema other than `main`, and no
  transaction statement, and SHALL NOT invoke an external process to read the
  structure.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-017**: WHEN the system opens a connection for a `sqlite` entry, it
  SHALL establish the kind of the file under `FR-LITE-005`, open it under
  `FR-LITE-006`, apply and confirm every control of `FR-LITE-010` — the
  extension-loading control by `PRAGMA compile_options`, and the
  temporary-storage control by its read-back and by that statement — and
  every limit of `FR-LITE-011`, and only then issue the version probe of
  `FR-LITE-002`. Every catalogue statement SHALL follow the probe.

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

- **BR-LITE-001**: For a `sqlite` entry the read-only promise of `BR-SEC-002`
  has three parts, and two of them prevent. The read-only open of
  `FR-LITE-006` prevents at the library: a connection opened that way cannot
  write the file. The closed list of `FR-LITE-015` prevents in `tpl`: it is
  what `tpl` is built to send. `query_only` makes a write fail on a connection
  that could otherwise make one, and is defence in depth beside the two. No
  flag disables any of them, per `FR-LITE-014`. What the promise does not
  cover, in write-ahead-log mode, is stated by `FR-LITE-008`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-018**: The closed list of `FR-LITE-015` and the order of
  `FR-LITE-017` SHALL be verified by an integration test that records, through
  the library's statement trace — the event SQLite names `SQLITE_TRACE_STMT`
  — every statement the trace reports on the connection from the first
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

  *Why the statement record is taken in process.* `BR-SRV-003` observes the
  statements a server receives, from the server's side. A SQLite connection
  has no other side: the library that receives the statements runs inside the
  process, and its trace interface is the boundary at which they are received.
  What the process does to the file — the promise a caller relies on — is
  observed from outside it.

  *Observed.* On the builds 3.51.3 and 3.54.1, over the observation database:
  the trace reported a statement when it began running. Two statements whose
  preparation failed — `SELECT * FROM no_such_table`, and a `SELECT` from a
  view that names a missing table — and a statement prepared and never run
  were not reported; the statement of `FR-LITE-035` that reads the columns of
  every view, which began running and then failed, was. The library's own
  reading of the schema table, when it loaded the schema, was not reported.
  Every pragma table-valued function of `FR-LITE-015` ran a statement of its
  own for each invocation, reported with the prefix: `-- PRAGMA encoding`,
  `-- PRAGMA table_list`, and, once for each table or index the join reached,
  `-- PRAGMA 'main'.table_xinfo='<table>'`,
  `-- PRAGMA 'main'.index_list='<table>'`,
  `-- PRAGMA 'main'.index_xinfo='<index>'` and
  `-- PRAGMA 'main'.foreign_key_list='<table>'`. Reading the columns of a view
  over an `fts5` table ran, in addition,
  `-- SELECT k, v FROM 'main'.'<table>_config'`: the module's own code,
  reading a table of the file. A view whose definition did not compile
  produced no prefixed statement of its own. The documentation of the event,
  at `https://sqlite.org/c3ref/c_trace.html`, checked 2026-11-03, states that
  it fires "when a prepared statement first begins running" and that a text
  beginning with `--` marks the invocation of a trigger; it does not state the
  prefix on a nested statement, which is observed here. No statement of the
  system's fires a trigger, per `FR-LITE-016`, so every prefixed text in the
  record is a nested statement.

  *Why nested statements are neither checked nor counted.* The system does not
  issue them, and their text is not `tpl`'s: a pragma function writes its own
  `PRAGMA`, and a module writes statements named after the file's own tables,
  so no list written in advance closes over what an untrusted file can make a
  module run. Counting them would break `FR-LITE-037`, because the pragma
  functions run one for each table or index, and a count that grows with the
  objects is the dependence `NFR-PERF-001` forbids. What a nested statement
  can do to the file is bounded by the controls that bound the connection —
  the read-only open and `query_only` — and which module may run from a view
  is bounded by `trusted_schema` off, under which the view calling `bm25()`
  did not compile; the read-only promise is verified from outside the process,
  whatever ran inside it.

  *Rejected: admitting nested statements to the list of `FR-LITE-015`.* The
  list would be closed over the fixture and open over every other file.
  *Rejected: asserting the nested statements the fixture produces, as
  recorded.* It binds the test to the library's internals: a library release
  that changes how a pragma function or a module reads would fail a test of
  `tpl`'s statements with `tpl` unchanged, and `FR-LITE-039` makes a new
  release a routine event.

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

## The part of the database a read covers

- **FR-LITE-019**: A `sqlite` read SHALL cover the schema `main` and no other.
  The `name` of the `database` object of `FR-CTX-036` SHALL be `main`. The
  `temp` schema and every attached schema are outside the read.

  *Rationale.* `main` is the database file the entry names, which is the
  database `FR-CONF-041` makes a read cover for a server. `temp` holds what one
  connection created, and the read creates nothing; an attached schema is a
  second file the entry does not name, the ground `FR-CAT-057` gives for not
  reading a second schema.

  *Observed.* `pragma_table_list` returns rows for `temp` beside `main` — its
  schema table, `sqlite_temp_schema` — which is why the restriction is stated
  as a filter.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-020**: The model SHALL cover, for a `sqlite` read, the objects
  `pragma_table_list` reports in `main` with type `table`, as tables, and with
  type `view`, as views, excluding every object `FR-LITE-021` names. A covered
  table SHALL carry `table_type` `BASE TABLE`, which is the value
  `FR-CAT-031` records for an ordinary table.

  The covered set SHALL be the one the carried library reports. A table SHALL
  be excluded as a shadow table only WHERE the carried library reports it as
  `shadow`, and a shadow table of a module the carried library does not hold
  SHALL be covered as a table.

  *Observed.* `pragma_table_list` reports four types — `table`, `view`,
  `virtual` and `shadow` — and the schema table `sqlite_schema` reports a
  virtual table and its shadow tables as `table`, with nothing to tell them
  from an ordinary one. The type of `pragma_table_list` is therefore the
  coverage predicate, and the schema table is not.

  *Why `BASE TABLE` and not SQLite's `table`.* `table_type` is the coverage
  predicate of `FR-CAT-001`, and a covered SQLite table is the kind
  `BASE TABLE` names. Carrying SQLite's spelling would give one covered kind
  two values, and every template branching on it would test both.

  *Observed, where the sixty-fifth edition stated a bounded claim.* A table is
  reported as `shadow` by the module that owns it, and therefore only by a
  library that holds that module. On a build of 3.54.1 without FTS5, for which
  `sqlite_compileoption_used('ENABLE_FTS5')` returned `0`,
  `pragma_table_list` reported the `fts5` table itself as `virtual` and its
  five shadow tables — `<name>_config`, `_content`, `_data`, `_docsize` and
  `_idx` — as `table`; the builds holding FTS5 reported the same five as
  `shadow`. On the build without FTS5, the columns, indexes and keys of the
  five were read like any other table's, and a view over the `fts5` table
  failed to compile with `no such module: fts5`.

  *Consequence, stated plainly.* Which tables a read covers depends on the
  modules the carried library holds, and those are fixed by the options it is
  built with. A shadow table of a module the carried library does not hold is
  presented as a `BASE TABLE`, with its columns, indexes and keys, at exit
  `0`, and nothing in the document says that a virtual table keeps its data
  there.

  *Rejected: recognising a shadow table by its name, as a virtual table's name
  followed by a suffix.* The suffixes are the module's own, a module the
  library does not hold publishes none to the library, and an ordinary table
  may carry such a name; the mistake would be a table missing from the model
  at exit `0`, which is worse than a table present that a template can skip by
  name.

  *What would change this.* A carried library built with the module.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Amended in the sixty-sixth edition,* for rmp `#319`. The bounded claim is
  replaced by the observation that settles it, and the requirement now states
  that the carried library's report decides the covered set.

- **FR-LITE-021**: The model SHALL NOT cover, for a `sqlite` read, any of the
  following, and the system SHALL NOT read a further object to cover one:

  | Excluded | How it is recognised |
  |---|---|
  | An internal table | Its name begins with `sqlite_`, ignoring ASCII case — `sqlite_schema`, `sqlite_sequence` and the `sqlite_stat` tables among them |
  | A virtual table | `pragma_table_list` type `virtual` |
  | A shadow table | `pragma_table_list` type `shadow` |
  | An object of the `temp` schema or of an attached schema | Its schema is not `main`, per `FR-LITE-019` |
  | A trigger on a view | The object it is on is a view; SQLite admits only `INSTEAD OF` triggers there, and the model carries triggers on tables, per `FR-CAT-014` |
  | An index with a member that is an expression | `pragma_index_xinfo` reports a member at column rank `-2`, with no name |
  | A foreign key whose parent key is not columns of a covered table | Its parent table is not a covered table, or a parent column it names is not a column of that table, or it names no parent column and the parent table has no primary key |

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

  *Why `WITHOUT ROWID` and `STRICT` are read and not carried.*
  `pragma_table_list` reports both, and `FR-LITE-026` and `FR-LITE-028` use
  them. The model has no property for either, and adding one adds a key to
  every MariaDB document, which `FR-CTX-043` makes the change of
  `provider` alone. A template sees their consequences: a `WITHOUT ROWID` table
  has no rowid alias, and a `STRICT` table's declared types are its six.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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
  | `name` | `pragma_table_list` |
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
  | `data_type` | the column's **type affinity**, in lower case — `integer`, `text`, `blob`, `real` or `numeric` — derived from `column_type` by the five rules of section 3.1 of `https://sqlite.org/datatype3.html`, in their order; for a column of a `STRICT` table declared `ANY`, `blob` |
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
  reports a column's `COLLATE` clause, per `BR-LITE-002`. *Why `blob` for a
  `STRICT` `ANY`.* `blob` affinity is SQLite's name for storing a value as it
  is received, which is what `https://sqlite.org/stricttables.html` states of
  an `ANY` column in a `STRICT` table; the first four rules do not match `ANY`
  and the fifth would give `numeric`, the coercing behaviour that column does
  not have.

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
  `WITHOUT ROWID` (the `wr` field of `pragma_table_list` is `0`), the column
  is its table's only primary-key column, and `pragma_index_list` reports no
  index of origin `pk` for the table. The `AUTOINCREMENT` keyword SHALL NOT be
  read.

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

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-031**: For a `sqlite` read, a table's outgoing foreign keys SHALL
  be read from `pragma_foreign_key_list`, one key per value of its `id` field,
  less those `FR-LITE-021` excludes, each carrying:

  | Property | Source |
  |---|---|
  | `name` | `null` |
  | its columns | the `from` field, in ascending order of `seq` |
  | `referenced_table` | the `table` field |
  | `referenced_columns` | the `to` field, paired by `seq`; WHERE `to` is SQL `NULL`, the referenced table's primary-key columns, in the order of `FR-LITE-029` |
  | `on_update`, `on_delete` | the `on_update` and `on_delete` fields, carried unchanged |
  | `match_option` | the `match` field, carried unchanged |
  | `referenced_key` | `null` |

  Incoming keys, under `referenced_by`, SHALL be the same keys read from the
  other end, per `FR-CAT-013`. Because no key has a name, the members of
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

- **FR-LITE-032**: For a `sqlite` read, a table's triggers SHALL be the rows of
  `sqlite_schema` of type `trigger` whose `tbl_name` is a covered table, each
  carrying:

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
  quotings, and the header under its comment syntax. IF the header does not
  yield one timing and one event, THEN the system SHALL exit `70`: the library
  accepted the text, and a derivation that cannot read it is a defect of
  `tpl`.

  *Source.* `https://sqlite.org/lang_createtrigger.html`, checked 2026-11-03,
  for the grammar, for "BEFORE is the default when neither keyword is
  present", for "SQLite supports only FOR EACH ROW triggers", for the fixed
  names `OLD` and `NEW`, and for "INSTEAD OF triggers work only on views";
  `https://sqlite.org/schematab.html` for the normalisation of the stored
  text — the keywords upper-cased, `TEMP` and the schema qualifier removed.

  *Observed.* `CREATE TRIGGER IF NOT EXISTS "t q" AFTER UPDATE OF x ON a1 …`
  was stored as `CREATE TRIGGER "t q" AFTER UPDATE OF x ON a1 …`, with
  `IF NOT EXISTS` removed, which that page does not state; a trigger declared
  with no timing was stored as `CREATE TRIGGER t_default_timing INSERT ON a1
  BEGIN SELECT 1; END`.

  *Why the whole statement and not the body.* MariaDB's action statement is the
  body alone. Cutting the body out of the stored text means finding the
  `BEGIN` that opens it after a `WHEN` clause that may itself hold one as an
  identifier — a parse of the expression grammar. The whole statement carries
  the `WHEN` clause and the `UPDATE OF` list that no property carries, and a
  template emitting DDL emits it as it stands. `database.provider` tells a
  template which of the two it holds.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-033**: For a `sqlite` read, a view SHALL carry `name`; `definition`,
  the `sql` field of its `sqlite_schema` row — the whole stored
  `CREATE VIEW` statement, unchanged; its columns, read from
  `pragma_table_xinfo` and decomposed as `FR-LITE-025` through `FR-LITE-027`
  decompose a table's; and `null` for `check_option`, `is_updatable`,
  `definer`, `security_type`, `character_set_client`, `collation_connection`
  and `algorithm`.

  *Observed.* A view's columns are known only by compiling it. A column
  selected directly from a table reported that column's declared type —
  `VARCHAR(80)` for `country_name` — and an aggregate reported the empty
  string; every view column read `notnull` `0`.

  *Why the whole statement.* The `SELECT` follows an optional column list and
  the keyword `AS`, and the stored text is the only text SQLite keeps. MariaDB
  rewrites its own definition, per `FR-CAT-047`, so neither provider gives a
  template the author's body.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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

- **FR-LITE-035**: A view whose columns cannot be read because its definition
  does not compile — it names a table or a function the database or the
  library does not hold, or calls a function `trusted_schema` off refuses —
  SHALL be **incomplete** under `FR-PRIV-002`, with `columns` the property
  that could not be read. A listing or a dump SHALL present it marked
  `"restricted":["columns"]` and SHALL succeed, per `FR-PRIV-005`; a request
  that names it SHALL exit `77` (`EX_NOPERM`), per `FR-PRIV-003`, and the
  `cause` SHALL name the view, the property, and the reason the library gave.

  The system SHALL read the columns of every view in one statement. IF that
  statement fails because a view does not compile, THEN the system SHALL read
  each view's columns in a statement of its own, so that exactly the views
  that fail are marked, per `FR-PRIV-006` and `FR-PRIV-007`.

  *Observed.* A view selecting from a missing table failed with
  `no such table: main.no_such_table`, a view calling a function the library
  lacks with `no such function: realpath`, and a view calling a function
  `trusted_schema` off refuses with `unsafe use of realpath()`. The first and
  the third, each beside other views, made the single statement reading every
  view's columns fail whole, while the other views' columns read correctly in a
  statement that left the failing view out.

  *Rationale.* `FR-PRIV-002` makes an object incomplete whatever the reason,
  and `BR-PRIV-001` is the asymmetry that applies: a listing that refused to
  answer because one view is broken would be less useful than one that answers
  and marks it, and a named view returned without its columns is the partial
  object `FR-PRIV-004` forbids. `77` is the code `FR-PRIV-003` gives every
  named object that comes back incomplete, so a caller branches the same way
  on both providers, and the `cause` says the view does not compile rather
  than that a privilege is missing.

  *Rejected: `78` for the named view.* The entry and the file are correct, and
  that row sends the caller to `.tpl/.cfg`. *Rejected: marking every view when
  the single statement fails.* It marks complete views, which `FR-PRIV-007`
  forbids. *Rejected: failing the read.* One broken view would make every table
  unreadable.

  *Consequence, stated plainly.* The statement count is constant except on a
  database holding a view that does not compile, where it grows with the
  number of views, per the note on `NFR-PERF-001`. A dump marking a view is
  refused as a `--context`, per `FR-PRIV-008`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-036**: Apart from `FR-LITE-035`, a `sqlite` read SHALL be complete
  or SHALL fail. A fault that prevents the schema from loading SHALL fail the
  whole read under `FR-LITE-007`, and SHALL NOT be presented as an object with
  a `restricted` marking. SQLite has no privileges, so no property is withheld
  from the reader, and the cross-checks of
  [privileges-and-completeness.md](privileges-and-completeness.md) have no
  case to detect.

  *Observed.* A schema the library refuses fails every statement that touches
  the file, including the probe, so no object can be read apart from it.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## Requirements of form

- **FR-LITE-037**: For a `sqlite` entry, the requirements of form of
  [performance-requirements.md](performance-requirements.md) SHALL hold as
  they hold for a server: the number of catalogue statements a full read and a
  named read issue SHALL NOT depend on the number of objects, save on the path
  `FR-LITE-035` takes after a failed statement; an invocation SHALL open at
  most one connection to the file, per `NFR-PERF-004`; and a cache hit SHALL
  open none and issue none, per `NFR-PERF-003`. A **connection** here is one
  connection of the library to the database file.

  The equality of `NFR-PERF-001` SHALL be verified, for a `sqlite` entry, over
  the fixture database of `FR-LITE-038` and over an empty database file, with
  the statements counted under `FR-LITE-018`.

  *Observed.* Eight statements — the probe and seven catalogue statements —
  read the observation database of 32 schema objects and an empty file of zero
  bytes alike, per `FR-LITE-015`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

  *Note added in the sixty-seventh edition,* for rmp `#314`. How the clause of
  `NFR-PERF-004` is verified for a `sqlite` entry, and on which targets, is
  stated in that requirement; how the count of `NFR-PERF-008` is verified for
  one is stated in that requirement.

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
     `MATCH FULL`, a key naming a table the database does not hold, and a
     self-reference;
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
      name, and an `INSTEAD OF` trigger on a view;
  12. a virtual table with shadow tables, `sqlite_sequence` and `sqlite_stat1`;
  13. identifiers carrying a space, a backtick and a double quote.

  It SHALL also produce, beside the main database: a copy in
  write-ahead-log mode; an empty file; a text file; a truncated database; a
  directory, a FIFO, a symbolic link to the main database and a dangling one;
  a database whose schema holds a statement longer than the length limit of
  `FR-LITE-011`, written by a connection that does not set that limit; and a
  database whose only objects are the internal tables.

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

## Dependencies

- [configuration-model.md](configuration-model.md) — `FR-CONF-051` through
  `FR-CONF-054`: the `provider` and `path` keys, the keys each provider
  refuses, and how a relative `path` resolves.
- [server-contract.md](server-contract.md) — the contract this file stands
  beside, and `FR-SRV-042`, `FR-SRV-011` and `BR-SRV-003`, whose grounds it
  adopts, and `FR-SRV-019`, whose form `FR-LITE-039` adopts.
- [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) — the model and the document
  this file fills, and `FR-CTX-043`, the `provider` field.
- [privileges-and-completeness.md](privileges-and-completeness.md) —
  `FR-PRIV-002` through `FR-PRIV-008`, which `FR-LITE-035` applies.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `70`, `74`,
  `77` and `78`, and `FR-ERR-030`, which `FR-LITE-012` and `FR-LITE-040`
  reach; and `FR-ERR-046`, which bounds a value of the file in a diagnostic
  line.
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
files it names, and the index of [open-questions.md](open-questions.md) stays
empty.
