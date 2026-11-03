---
title: SQLite Contract
status: approved
last-reviewed: 2026-11-03
related: [configuration-model.md, server-contract.md, catalogue-coverage.md, context-document.md, privileges-and-completeness.md, security.md, cfg-commands.md, performance-requirements.md]
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
decisions recorded in the project's decision records, per `FR-LITE-001` and
`FR-LITE-004`; and the text of
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
  The `hint` SHALL carry `tpl cfg database update <entry> --path <path>`.

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

- **FR-LITE-006**: The system SHALL open the database file read-only, and SHALL
  NOT create it. A path that names no file SHALL stay naming no file after any
  invocation.

  *Observed.* A read-only open of a path that names nothing failed with
  `unable to open database file` (`SQLITE_CANTOPEN`) and created nothing. On a
  connection opened read-only, `CREATE TABLE` and `INSERT` failed with
  `attempt to write a readonly database` (`SQLITE_READONLY`).

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-007**: Once the kind of the file is established under
  `FR-LITE-005`, a failure to read it SHALL end the invocation with the code
  below, and SHALL NOT be reported as an empty database:

  | Condition | Observed as | Code |
  |---|---|---|
  | The path names nothing, the filesystem refuses the open, or a link dangles | `SQLITE_CANTOPEN` | `74` (`EX_IOERR`) |
  | The file is not a SQLite database | `SQLITE_NOTADB`, at the version probe | `78` (`EX_CONFIG`) |
  | The file is a SQLite database that is damaged, or whose schema the library cannot load — a corrupt page, a schema object the library cannot parse, or one the hardening of `FR-LITE-010` and `FR-LITE-011` refuses | `SQLITE_CORRUPT`, including `malformed database schema (<object>) - <reason>`, at the version probe or any later statement | `78` |
  | The database is in write-ahead-log mode and its directory refuses the files that mode needs, per `FR-LITE-008` | `SQLITE_READONLY_DIRECTORY` | `74` |
  | A read of the file fails part-way | `SQLITE_IOERR` | `74` |
  | The database stays locked by a writer until the catalogue-query deadline expires, per `FR-LITE-009` | `SQLITE_BUSY` | `69` (`EX_UNAVAILABLE`) |

  The `cause` SHALL name the path as resolved, the statement or the open that
  failed, and what the library returned, including the object and the reason
  where the library names them. For `74` and `78` the `hint` SHALL carry
  `tpl cfg database update <entry> --path <path>`.

  *Observed.* A text file failed with `file is not a database`; a file whose
  header bytes 16 to 19 were overwritten failed the same way; a file cut to
  100 bytes failed with `database disk image is malformed`; a table whose
  `CHECK` calls a function the library does not admit under
  `trusted_schema` off made **every** statement fail with
  `malformed database schema (t) - unsafe use of realpath() (11)`, and an
  expression depth above the limit set did the same with
  `Expression tree is too large`. A file that cannot be read is never an empty
  database: an **empty file** of zero bytes, on the other hand, opened and
  returned no schema row, which is SQLite's own reading of it, and it is read
  as a database with no object.

  *Why `78` for a file that is not a database.* The entry points at something
  `tpl` cannot read as a database, which is the reading `FR-SRV-003` gives `78`
  for a server that is not MariaDB, and the next step is the same: point the
  entry elsewhere. A damaged file and a schema the library cannot load share
  that next step, so `FR-ERR-002` admits them to the same code, and the `cause`
  separates them. *Why `74` for a path that cannot be opened.* It is the
  reading `FR-RND-035` gives a `--context` path that cannot be read: the caller
  looks at the path and at what the filesystem said, and a filesystem path is
  no population a `66` could suggest from.

  *Rejected: `65` for a damaged file.* Its row of `FR-ERR-001` sends the caller
  to the template or the context, and neither is at fault. *Rejected: reading
  what can still be read from a damaged file.* A model built from a schema the
  library refused would be a model whose completeness nothing establishes,
  presented at exit `0`.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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

## Hardening against an untrusted file

- **FR-LITE-010**: The database file SHALL be treated as untrusted input, per
  the trust boundaries of [security.md](security.md). On every connection it
  opens, before the version probe, the system SHALL apply each of the
  following controls and SHALL confirm, by reading the setting back, that each
  took effect:

  | Control | Setting | Closes |
  |---|---|---|
  | Read-only open | read-only, never creating the file, per `FR-LITE-006` | any write through the connection |
  | `query_only` | on | any write the open itself did not refuse, as defence in depth |
  | Defensive mode | on | the features by which ordinary SQL can corrupt a database file |
  | `trusted_schema` | off | an SQL function or a virtual table that is not marked innocuous, run from a view, a trigger or a schema expression |
  | Extension loading | off, for both the library interface and the `load_extension()` function | code loaded from a path a schema or a statement names |
  | `cell_size_check` | on | a malformed page read without its cell sizes checked |
  | `mmap_size` | `0` | a page read through a memory map of a file another process can change |

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
  re-check is a change to that guidance.

  *Observed.* Each setting read back as set, on a read-only connection: the
  pragmas `query_only` `1`, `trusted_schema` `0`, `cell_size_check` `1` and
  `mmap_size` `0`, and the configuration flags `defensive on`,
  `load_extension off` and `trusted_schema off` through the shell's
  configuration report. A `query_only` connection opened read-write refused
  `CREATE TABLE`, `INSERT` and `PRAGMA user_version=5` with
  `attempt to write a readonly database`.

  *What `trusted_schema` off costs, and where.* Observed: a view calling a
  function that is not innocuous failed to compile with
  `unsafe use of realpath()`, which is `FR-LITE-035`'s case; a `CHECK`
  constraint calling one made the whole schema fail to load, which is
  `FR-LITE-007`'s; a `DEFAULT` and a trigger body calling one did not affect
  the read. A function the library does not hold at all failed only the view
  that called it, with `no such function: realpath`, and did not stop the
  schema loading. Among the functions the shell held, those not marked
  innocuous were its own additions and a handful of aggregate, full-text and
  R-tree functions.

  *Rejected: disabling views, as the guidance offers for an application that
  uses none.* The columns of a view are read by compiling it, per
  `FR-LITE-033`. *Rejected: disabling triggers.* No statement this system
  issues can fire one, per `FR-LITE-016`, and reading a trigger's definition
  compiles nothing; the control would close nothing that is open.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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
  | `PRAGMA query_only`, `PRAGMA trusted_schema`, `PRAGMA cell_size_check` and `PRAGMA mmap_size`, each in the form that sets the value `FR-LITE-010` fixes and in the form that reads it back | The controls of `FR-LITE-010` that a statement applies | Once each, at connection start |
  | `SELECT sqlite_version()`, reading nothing else | The version probe of `FR-LITE-002` | Once, at connection start |
  | `SELECT` against the schema table `sqlite_schema` and the pragma table-valued functions `pragma_table_list`, `pragma_table_xinfo`, `pragma_index_list`, `pragma_index_xinfo`, `pragma_foreign_key_list` and `pragma_encoding`, restricted to the schema `main` | Reading the catalogue, and the probe of `FR-CFG-056` | As the command requires |

  A control the system applies through the library's configuration interface
  rather than by a statement issues no statement, and the list admits the four
  pragmas without obliging their use. The table fixes membership and not
  order; the order is `FR-LITE-017`.

  *Observed.* Over the observation database, the whole catalogue was read by
  seven statements of the third row after the probe — the encoding, the
  covered objects, the columns of every table, the indexes and their columns,
  the foreign keys, the stored definitions of the views and triggers, and the
  columns of every view — and the same eight statements read an empty database
  of zero bytes. Each joins `pragma_table_list` to the function it reads
  through, so its count does not depend on the number of objects.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-016**: The system SHALL NOT issue any other statement for a
  `sqlite` entry. It SHALL issue no DDL, no DML, no `ATTACH` or `DETACH`, no
  `VACUUM`, `ANALYZE` or `REINDEX`, no `PRAGMA` outside the list of
  `FR-LITE-015`, no statement against a schema other than `main`, and no
  transaction statement, and SHALL NOT invoke an external process to read the
  structure.

  *Added in the sixty-fifth edition,* for rmp `#314`.

- **FR-LITE-017**: WHEN the system opens a connection for a `sqlite` entry, it
  SHALL establish the kind of the file under `FR-LITE-005`, open it under
  `FR-LITE-006`, apply and confirm every control of `FR-LITE-010` and every
  limit of `FR-LITE-011`, and only then issue the version probe of
  `FR-LITE-002`. Every catalogue statement SHALL follow the probe.

  *Rationale.* The probe is the first statement that reads the file and loads
  its schema, per the observation under `FR-LITE-002`, so every control is in
  force before any byte of the untrusted file is parsed. It is the order
  `FR-SRV-042` gives a server for the same reason: the guarantee is confirmed
  before anything is read.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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
  `FR-LITE-017` SHALL be verified by an integration test that records every
  statement the library prepares on the connection, through the library's own
  trace interface, reached through the seam of `FR-ERR-031` and from no
  invocation of the distributed binary. The read-only promise SHALL be
  verified from outside the process: over a read of the fixture database of
  `FR-LITE-038` in rollback-journal mode, the file's bytes, its size and its
  modification time SHALL be unchanged and no file SHALL appear in its
  directory.

  *Why the statement record is taken in process.* `BR-SRV-003` observes the
  statements a server receives, from the server's side. A SQLite connection
  has no other side: the library that receives the statements runs inside the
  process, and its trace interface is the boundary at which they are received.
  What the process does to the file — the promise a caller relies on — is
  observed from outside it.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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

  *Observed.* `pragma_table_list` reports four types — `table`, `view`,
  `virtual` and `shadow` — and the schema table `sqlite_schema` reports a
  virtual table and its shadow tables as `table`, with nothing to tell them
  from an ordinary one. The type of `pragma_table_list` is therefore the
  coverage predicate, and the schema table is not.

  *Why `BASE TABLE` and not SQLite's `table`.* `table_type` is the coverage
  predicate of `FR-CAT-001`, and a covered SQLite table is the kind
  `BASE TABLE` names. Carrying SQLite's spelling would give one covered kind
  two values, and every template branching on it would test both.

  *Bounded claim.* A table is reported as `shadow` by the module that owns it.
  Every shadow table observed belonged to a module the reading library held —
  `fts5` and `rtree` — and how a shadow table of a module the carried library
  does not hold is reported was not observed.

  *Added in the sixty-fifth edition,* for rmp `#314`.

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

  *Added in the sixty-fifth edition,* for rmp `#314`.

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
  9. `CHECK` constraints at column and table level, one named;
  10. views, one with a column list, one selecting from a missing table, and
      one calling a function the carried library does not hold;
  11. triggers of each timing on each event, one with no timing written, one
      `UPDATE OF` a column list, one with a `WHEN` clause, one with a quoted
      name, and an `INSTEAD OF` trigger on a view;
  12. a virtual table with shadow tables, `sqlite_sequence` and `sqlite_stat1`;
  13. identifiers carrying a space, a backtick and a double quote.

  It SHALL also produce, beside the main database: a copy in
  write-ahead-log mode; an empty file; a text file; a truncated database; a
  directory, a FIFO, a symbolic link to the main database and a dangling one;
  a database whose `CHECK` makes the schema fail to load under `FR-LITE-010`
  or `FR-LITE-011`; and a database whose only objects are the internal tables.

  *Rationale.* This is the obligation `FR-CONF-038` and the root `CLAUDE.md`
  place on `scripts/mariadb/` for the catalogue a server reads, written for
  the catalogue a file holds. A database built from text is reviewable in a
  diff and buildable by the library the binary carries; a committed binary
  file is neither. The modelled domain is realistic for the reason the root
  `CLAUDE.md` gives, and the observation of this file was taken in the
  freight domain the MariaDB fixture models.

  *Added in the sixty-fifth edition,* for rmp `#314`.

## Dependencies

- [configuration-model.md](configuration-model.md) — `FR-CONF-051` through
  `FR-CONF-054`: the `provider` and `path` keys, the keys each provider
  refuses, and how a relative `path` resolves.
- [server-contract.md](server-contract.md) — the contract this file stands
  beside, and `FR-SRV-042`, `FR-SRV-011` and `BR-SRV-003`, whose grounds it
  adopts.
- [catalogue-coverage.md](catalogue-coverage.md) and
  [context-document.md](context-document.md) — the model and the document
  this file fills, and `FR-CTX-043`, the `provider` field.
- [privileges-and-completeness.md](privileges-and-completeness.md) —
  `FR-PRIV-002` through `FR-PRIV-008`, which `FR-LITE-035` applies.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `69`, `70`, `74`,
  `77` and `78`, and `FR-ERR-030`, which `FR-LITE-012` reaches.
- [security.md](security.md) — `FR-SEC-028`, which points here.
- [cfg-commands.md](cfg-commands.md) — `FR-CFG-055` and `FR-CFG-056`.
- [template-environment.md](template-environment.md) — `FR-ENV-048`.
- [performance-requirements.md](performance-requirements.md) — `NFR-PERF-001`
  through `NFR-PERF-004`, which `FR-LITE-037` applies.

## Open questions

**None.** Every point the provider raised is settled in this file or in the
files it names, and the index of [open-questions.md](open-questions.md) stays
empty.
