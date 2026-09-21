---
title: Performance Requirements
status: approved
last-reviewed: 2026-09-21
related: [server-contract.md, cache-documents.md, global-flags.md, catalogue-coverage.md, context-document.md, project-and-discovery.md]
---

# Performance Requirements

## Overview

`tpl` is invoked repeatedly, often inside a loop, and its speed is a first-order
requirement. This file states the part of that requirement a specification can
own: the properties of the program that are observable from outside it,
permanent, and falsifiable without a stopwatch.

What constrains from the first commit is a small set of requirements of form —
the number of catalogue queries does not depend on the number of objects, a
cache hit opens no connection, one invocation opens at most one connection.
Those catch exactly the defects a figure was meant to catch, and they can be
checked today.

Beside them, this file fixes a **protocol**: what is measured, over which
reference workload, on which target, and what has to happen before a number
becomes a limit. The protocol is normative from now. The numbers it carries are
not yet: there is no code, so `NFR-PERF-019` marks every figure in
`NFR-PERF-014` **provisional**, and the first real measurement of a budget
supersedes its provisional figure under the gate of `NFR-PERF-020`. A ratified
figure lives in `BENCHMARKS.md`; the provisional figure lives here until the
measurement that replaces it exists, so that a budget is never a blank.

No figure is invented. Where the project's root documents supplied one it is
adopted as provisional and its origin is stated; where they supplied none, the
budget is named and left unvalued, per `BR-PERF-006`.

## Scope

In scope: the requirements of form, the three reference workloads, the target
set, the measurement protocol, the budget set with its normative member, the
provisional figures it carries, and the gate by which a provisional figure
becomes a ratified one.

Out of scope: every ratified figure and every recorded baseline, which live in
`BENCHMARKS.md`; the tools used to measure; and the implementation choices that
would satisfy a budget.

## Actors

- **Calling agent**, which pays the startup cost once per object.
- **Continuous integration**, which can enforce only the budgets that need no
  server.
- **Database administrator**, from whose side the connection and statement
  invariants are observed.

## Requirements of form

- **NFR-PERF-001**: The number of catalogue queries the system issues for a full
  read SHALL NOT depend on the number of objects in the database. The count over
  `WL-001` SHALL equal the count over `WL-003`.

  *Rationale.* This is the N+1 prohibition, stated so that it can be checked
  rather than reviewed. A reader that issues one query per table passes every
  correctness test and fails this one.

  *Observed, 2026-09-21: the comparison this requirement asks for has been
  made.* Both workloads were loaded into each of the four series of
  `FR-SRV-015` from `scripts/mariadb/seed-bench.sql`, by the loader that
  verifies every count those workloads state, and a full read of each was taken
  with `tpl` as the client, from a fresh project whose cache was empty so that
  the read reached the server. The statements each server received were counted
  from the server side, per `NFR-PERF-007`, with the counting window bracketed
  per read:

  | Series | Statements over `WL-001` | Statements over `WL-003` |
  |---|---|---|
  | `10.11` | 11 | 11 |
  | `11.4` | 11 | 11 |
  | `11.8` | 11 | 11 |
  | `12.3` | 11 | 11 |

  **Eleven against eleven, on all four series.** A database of 200 tables and a
  database of one cost the reader the same eleven catalogue statements, which
  is this requirement satisfied by measurement rather than by review. The count
  is the same one the integration suite asserts for a full read of the
  correctness fixture, whose 23 objects sit between the two, so it is now
  observed over three databases spanning two orders of magnitude of object
  count.

  *What this record is not.* It ratifies nothing. No baseline is set, no figure
  becomes a limit, and no budget of `NFR-PERF-014` is measured by it: a
  statement count is a requirement of form, and ratification is
  `NFR-PERF-009` through `NFR-PERF-012` under the gate of `NFR-PERF-020`.

  *The condition the reading was taken under, stated because the fixture can
  change it.* Eleven is what the reader issues over the catalogue this corpus
  covers, at the coverage of `catalogue-coverage.md` in force on the date
  above. What this requirement fixes is the **equality** of the two counts, not
  the number eleven; a later edition that widens coverage moves both counts
  together and leaves this requirement satisfied.

- **NFR-PERF-002**: The number of catalogue queries the system issues to read
  one named object SHALL NOT depend on the number of objects in the database.
  The **rows** such a read returns MAY be the whole catalogue, and a read that
  returns them SHALL NOT be taken to violate this requirement.

  *Rationale.* This is the mirror image of the N+1, stated so that it can be
  checked rather than reviewed, and what it counts is statements. A reader that
  issues one query per named object passes every correctness test and fails
  this one.

  *Amended in the twenty-seventh edition, because the rationale argued against
  what three requirements in force oblige.* It read: *Reading the whole
  catalogue to answer `tpl schema table orders` is the mirror-image defect of
  the N+1*. That names the row volume as the defect, and the row volume is not
  what this requirement fixes — a whole-catalogue read of a fixed number of
  statements satisfies its words exactly. The three requirements are
  `FR-CTX-006` and `FR-CTX-010`, which embed **in full** the table at each end
  of every foreign key, and `FR-CTX-023`, which requires every object a
  document references to be present in it. A read that returned one table's
  rows would return that table's keys without the tables they name, and no
  document could be built from it. The sentence is replaced rather than
  softened, because a rationale that condemns the only satisfiable
  implementation is worse than none.

  *What a narrow read would have to return, stated so that the option stays
  open.* A read that presents one named table must return, in addition to that
  table's own rows, the columns, indexes and primary key of **every table at
  either end of one of its foreign keys** — the tables `FR-CTX-007` and
  `FR-CTX-010` require to be embedded in full. It need go no further: the
  embedded tables' own keys are cut to names by `FR-CTX-008`, at the first hop,
  in both directions, per `FR-CTX-009`. This corpus states no statement
  repertoire, so nothing here says how those rows are obtained; what it fixes
  is that a plan returning less than them cannot produce a document, and that a
  plan returning the whole catalogue is admissible under this requirement.

  *Rejected: obliging a narrow read.* The neighbours of a named table are not
  known until that table's key rows have been read, so a narrow plan either
  issues a second, dependent round of statements or joins the catalogue to
  itself. The first is admissible here — its count still does not depend on the
  number of objects — and the second has a cost this reader does not control.
  Neither is forbidden by this requirement and neither is required by it, and
  choosing between them is an architecture decision rather than a functional
  one. What was not admissible was leaving a rationale in force that reads as
  forbidding the only plan the embedding allows today.

  *Accepted cost, stated plainly.* `tpl schema table orders` against a database
  of two hundred tables reads two hundred tables' rows and presents one. The
  statement count is unchanged, the wall-clock and memory cost is not, and it
  is carried by the budgets of `NFR-PERF-014` and by the `WL-002` scalar rather
  than by this requirement. `BR-CTX-001` and `FR-CTX-010` record the same cost
  from the document's side, where it was accepted twice over.

- **NFR-PERF-003**: A cache hit SHALL open no connection and SHALL issue no
  catalogue query.

- **NFR-PERF-004**: One invocation SHALL open at most one connection.

- **NFR-PERF-005**: Every command named by `FR-PROJ-025` SHALL perform no
  project discovery, SHALL read no configuration file, and SHALL open no
  connection. Those commands are `tpl init`, every form of `help`, and every
  form of `version`. The discovery clause is what `FR-PROJ-025` states it
  forbids — no project above the invocation decides its outcome — and the one
  upward look that requirement licenses, the ancestor `FR-PROJ-016` obliges
  `tpl init` to warn about, satisfies it rather than excepting from it.

  The connection clause SHALL be verified on every target of `NFR-PERF-018`,
  from the server side. The discovery clause and the configuration clause SHALL
  be verified on every target of `NFR-PERF-018` by the differential run of
  `NFR-PERF-007`, and SHALL additionally be verified by the file-open
  observation of `NFR-PERF-007` **on the two Linux targets only**. On the two
  macOS targets the file-open observation SHALL NOT be made, and SHALL NOT be
  inferred from a Linux build observed in a container.

  *Amended in the third edition.* The first edition covered the two version
  forms only and left the help forms and `tpl init` unsettled, which was
  recorded as an open question. `FR-PROJ-025` now names the set functionally,
  and this requirement states the observable consequence, verified from outside
  the process per `NFR-PERF-007`: no `stat` of an ancestor directory, no open
  of `.tpl/.cfg`, no socket.

  *Amended in the thirty-first edition: the first of the third edition's three
  observables holds for three of the four commands and not for the fourth.*
  `tpl init` walks its destination's ancestors, because `FR-PROJ-016` requires
  it to warn that the project it creates shadows one above, so a syscall trace
  of it records `stat` calls above the destination. The third edition's
  sentence read over the whole set and was false of one member of it from the
  first edition onwards; the code was observed doing exactly what
  `FR-PROJ-016` obliges on 2026-09-18, recorded in that requirement and in
  `FR-PROJ-025`. The observables now read, per command:

  | Command | `stat` above the working directory or the destination | Open of `.tpl/.cfg` | Socket |
  |---|---|---|---|
  | every form of `help` | none | none | none |
  | every form of `version` | none | none | none |
  | `tpl init` | the walk of `FR-PROJ-016`, and nothing else | none | none |

  The configuration clause and the connection clause are unchanged and hold for
  all four alike. Nothing about what `tpl` does changes here: the walk has been
  required since the first edition, and what changes is that this requirement
  stops stating an observable that contradicts it.

  *Rejected: keeping the sentence and obliging `tpl init` to satisfy it.* That
  is withdrawing `FR-PROJ-016`, which `FR-PROJ-025` rejects in its own text
  because `FR-PROJ-005`'s accepted cost names the warning as one of the three
  things bounding the removal of the home boundary.

  *Rejected: reading the sentence as a summary that the requirement's own
  clauses override.* It is the sentence a verification suite is written from —
  it names three syscalls and the instrument that sees them — and a summary
  that a test asserts is not a summary.

  *Amended in the eleventh edition: the verification names its platforms.* The
  third edition's note said the requirement is verified from outside the
  process and named what would be seen there, and it named no platform. One of
  the three things it names — the open of `.tpl/.cfg` — cannot be seen on
  either macOS target, and saying so is the whole of this amendment. Nothing
  about what `tpl` does changes: the three clauses are the same three clauses,
  and each still holds on all four targets. What changes is that the evidence
  for two of them is now stated per target instead of being read as uniform.

  *Observed, 2026-09-11, on Darwin 25.6.0, while the fixture work of the tenth
  edition was in hand.* `strace` does not exist on macOS. `dtruss` refuses,
  reporting that DTrace requires additional privileges, while `csrutil status`
  reports System Integrity Protection `enabled`. `sudo -n fs_usage` refuses,
  reporting that a password is required. No instrument that records the files a
  process opens is therefore reachable on a macOS host under the protection
  that host ships with: of the three that exist, one is absent, one is withheld
  by `csrutil`, and one is interactive. This is an observation of an **absence**
  and it is not repeated: it was made once, on the platform, and what would
  overturn it is named below rather than another run of the same three
  commands.

  *Consequence, stated plainly, because the evidence is weaker on two targets
  than the requirement's wording suggests.* On `x86_64-apple-darwin` and
  `aarch64-apple-darwin` nothing records the syscalls these commands make. A
  build that opened `.tpl/.cfg`, read it, and discarded what it read would
  satisfy every observation available there, and would be caught only on a
  Linux target. What the two macOS targets do establish is the property a
  caller depends on — that no configuration file and no ancestor directory
  reaches the outcome, and that a project these commands would have tripped
  over does not fail them — and that is the differential run, not the trace.
  The startup budgets of `NFR-PERF-014` would also register a read that took
  measurable time, but they are budgets and not observations of this
  requirement, and a discarded read need not be measurable.

  *Rejected.* Crediting the Linux observation to the two macOS targets, which
  is what happens today: the observation is made against the Linux build inside
  a container, and that build is not the binary a macOS user runs. The corpus
  has already refused this exact step once — `FR-ERR-031` rejects a candidate
  trigger for `70` on the ground that the artefact verified would not be the
  artefact distributed, and cites `NFR-PERF-018` for it.

  *Rejected.* Re-expressing the property so that it is verified from inside the
  process. `NFR-PERF-007` requires an observation made outside the process and
  forbids verification by reading the source. The eighth edition did let one
  verification move in-process, in `FR-ERR-031`, but the licence it recorded
  opened only because **every** mechanism outside the process collided with a
  requirement in force, candidate by candidate. That test is not met here: two
  outside instruments remain, and one of them — the differential run — is
  available on all four targets.

  *Rejected.* A privileged path on macOS. Both instruments the platform still
  has cost a contributor something the specification cannot ask for: `dtruss`
  needs System Integrity Protection disabled, which is a configuration of the
  contributor's own machine, and `fs_usage` needs a password answered at each
  run, which no automated run can supply. Neither is this specification's to
  impose — it states what `tpl` does, not how a contributor's machine is set up,
  which is the boundary the [README](README.md#still-out-of-scope) draws. It is
  also not an alternative to what is written here: adopting it would add an
  instrument on some macOS hosts and would not remove the need for one that
  every target has.

  *What would change this.* Either an instrument on macOS that records the
  files a process opens without a privilege the platform withholds, which is a
  fact about the platform that this project does not set; or a project decision
  to accept the privileged path above and to state in this requirement what it
  costs a contributor. The first would let the file-open observation be
  required on all four targets. The second is a decision this specification
  does not make on its own, and taking it is an amendment to this requirement
  and to `NFR-PERF-007` together.

- **NFR-PERF-006**: A command that requires no catalogue data SHALL open no
  connection. This covers every `template` subcommand, every `cfg` subcommand
  except `database test`, `help`, `version`, `init`, any `tpl render` invoked
  with `--context`, and any `tpl render` whose template name does not resolve.

  *Amended in the thirty-first edition: the coverage clause names the
  invocation the thirtieth edition created.* That edition moved template
  resolution to the fourth step of `FR-ERR-006`, so a `tpl render` whose
  template does not exist is refused before an entry is resolved, a connection
  is opened or a catalogue statement is issued — and it cited the obligation
  above as one of the two requirements the move satisfies. It checked this
  clause and left it, on the ground that the clause enumerates and the
  obligation governs. The ground is sound and the omission is corrected
  anyway: this clause already reasons per invocation rather than per command —
  *any `tpl render` invoked with `--context`* is an invocation — so a reader
  takes it for the set, and the one invocation it was short of is the one the
  edition before it acted on. An enumeration short by exactly the case the
  previous edition decided from it is the enumeration worth lengthening.

  Nothing about `tpl` changes: the obligation is the same obligation, the
  invocation already satisfies it, and `FR-ERR-006` is untouched.

  *Rejected: leaving the clause as the thirtieth edition left it.* Its ground
  — the obligation governs and the clause only illustrates — is true and is
  not what a reader does with a list of five. Also rejected: replacing the
  clause with the obligation alone, dropping the list. The list is what tells
  an implementer which invocations were considered, and dropping it to avoid
  having to extend it would lose the only record of that.

- **NFR-PERF-007**: Each requirement of this section SHALL be verified by an
  observation made outside the process, and SHALL NOT be verified by reading
  the source. The instruments SHALL be exactly the following four, and each
  SHALL be used only on the targets of `NFR-PERF-018` its row names:

  | Instrument | What it observes | Targets |
  |---|---|---|
  | The server's statement record | The statements the server receives | all four |
  | The server's connection record | The connections the server accepts | all four |
  | A syscall trace of the process | The files the process opens | the two Linux targets |
  | A differential run | The observable outcome of the invocation — its exit code, the bytes on stdout, and the artefacts it leaves on disk — under an arrangement in which the operation, had it been performed, would have changed that outcome | all four |

  A **differential run** is defined in
  [glossary.md](glossary.md#differential-run). For the discovery clause of
  `NFR-PERF-005` the arrangement is `tpl init` invoked inside a subdirectory of
  an existing project, which per `FR-PROJ-012` and `FR-PROJ-013` creates a
  project in that subdirectory, asserting exit `0`, stdout empty per
  `FR-PROJ-022`, and a `.tpl` carrying the five artefacts of `FR-PROJ-017` at
  the subdirectory and not at the ancestor. For the configuration clause it is
  any command of `FR-PROJ-025` invoked inside a project whose `.tpl/.cfg` would
  fail the validation of `FR-CONF-034`, asserting exit `0` and stdout
  byte-identical to the same command invoked outside any project.

  *Amended in the thirty-first edition: the first arrangement asserted
  something this instrument does not read, and something a requirement in force
  forbids.* It read that the arrangement *creates a project in that
  subdirectory and does not report the ancestor*. The second half is wrong
  twice over. `FR-PROJ-016` **requires** the ancestor to be reported, so the
  arrangement asserted the absence of a warning this corpus obliges; and the
  report goes to stderr, which is not among the three observables the
  differential run reads — its row above names the exit code, the bytes on
  stdout, and the artefacts left on disk. The clause is replaced by what the
  arrangement does assert, in those three terms, and the differential run is
  unweakened by the change: a `tpl init` that let an ancestor decide where the
  project goes would create nothing at the subdirectory, or would refuse with
  `73` under `FR-PROJ-014`, and either outcome fails the assertion.

  *Amended in the thirty-second edition: the definition moves to the glossary
  and this requirement cites it.* The eleventh edition wrote it here, and the
  thirty-first followed the shape for *in flight*, which left the corpus with
  two conventions for where a defined term lives. The glossary governs; the
  decision, and the alternative rejected with it, are at the head of
  [glossary.md](glossary.md). **Nothing about the instrument changes.** The
  three observables stay in the table row above, which is what this instrument
  observes rather than what the term means, and the two arrangements stay
  here, because an arrangement is an application of the term and not the term.

  On a target where both the third instrument and the fourth are available, the
  third SHALL be what establishes a clause stated as a syscall, and the fourth
  SHALL corroborate it. On a target where only the fourth is available, it
  SHALL be the whole of the evidence, and the requirement it serves SHALL say
  so in its own text.

  *Amended in the eleventh edition: the instruments are named, and each is
  bound to the targets it exists on.* The rule named three instruments in a
  parenthesis and bound none of them to a platform, which read as a promise
  that all three are available everywhere. One is not: no instrument that
  records the files a process opens is reachable on a macOS host under its
  default protection, which `NFR-PERF-005` records with the date and the three
  commands that established it.

  *Why a fourth instrument rather than a relaxed third.* Dropping the file-open
  clause would leave the two macOS targets with nothing at all for two of the
  three clauses of `NFR-PERF-005`, and `NFR-PERF-018` forbids a target being
  second class. The differential run is outside the process as this rule
  requires — it reads an exit code and a stream of bytes, and reads no source —
  it needs no privilege on any platform, and it establishes the property a
  caller depends on. It establishes **less** than a trace, which is why the
  order of the two is fixed above and why `NFR-PERF-005` states the difference
  rather than leaving it to be inferred.

  *Rejected.* Allowing an in-process observation where no outside instrument
  exists. That is the licence the eighth edition gave `FR-ERR-031`, and it
  opened only because every mechanism outside the process collided with a
  requirement in force. Here two remain, one of them on every target, so the
  condition that would open it is not met.

- **NFR-PERF-008**: The catalogue-query count SHALL be observable from the
  diagnostic stream, per `FR-GLOB-017`, which requires one line per catalogue
  query issued, distinguishable from every other diagnostic line.

- **BR-PERF-001**: A requirement of form is preferred to a figure wherever both
  would catch the same defect, because a figure has to be re-measured on every
  target and every machine while a form holds everywhere and forever. The
  figures are not abandoned; they are placed where a measurement can be recorded
  beside them.

## Reference workloads

- **WL-001** — the large workload. A database of 200 tables, 2 400 columns, 600
  indexes, 180 foreign keys, 40 generated columns, 25 triggers, 30 views, 40
  routines, and comments on 60% of the tables. It SHALL be realised by a
  dedicated fixture, `scripts/mariadb/seed-bench.sql`.

  **The index count counts the primary key, and counts a composite index
  once.** The 600 are 200 primary keys, 200 secondary indexes, 20 composite
  indexes and the 180 that carry the foreign keys. `WL-003` states the same
  rule over its own count, so that the two workloads are counted alike, which
  is what makes the comparison `NFR-PERF-001` asks for a comparison of two
  databases and not of two counting conventions.

  *Stated in the thirty-third edition, because the fixture had to choose and
  this file had not.* The number stood alone, and a count of indexes admits
  two readings that differ by a third of it. The reading written above is the
  one the model already obliges: `FR-CAT-043` states that the primary key **is
  also an index**, that the catalogue reports it as one, and that the index
  named `PRIMARY` appears in the table's index collection. A workload's index
  count is a count of what a reader of that database presents, so excluding
  `PRIMARY` would count something this corpus does not carry. The second
  clause follows from the same requirement: `FR-CAT-042` folds the catalogue's
  one row per index column into one index carrying a column list, so a
  composite index is one index and never one per column.

  *Rejected: `PRIMARY` not counted.* It was the live alternative, because a
  reader may take an index count to mean the keys a DDL declares beside the
  primary key. It is rejected on three grounds. It contradicts `FR-CAT-043`,
  which puts `PRIMARY` in the collection being counted. It moves both numbers
  rather than settling one — under it the 600 would name a database of 800
  indexes and `WL-003`'s three a database of four — so adopting it is a change
  to the shape of two workloads that `NFR-PERF-014` measures nine budgets
  against; `BR-PERF-005` records what that costs, having found that a figure
  cannot be carried across a change to the quantity it measured. And it buys
  nothing: the quantity `NFR-PERF-001` compares is a count of **statements**,
  which no index convention moves, so the only thing the alternative would
  change is whether the fixture can verify that it realises what this file
  names.

- **WL-002** — the verification scalar. The compact `tpl schema dump` of
  `WL-001` SHALL be N bytes, within ±2%. N SHALL be unvalued until it is
  measured, and SHALL then be recorded in `BENCHMARKS.md` under the gate of
  `NFR-PERF-020`.

  *Rationale.* A single scalar detects, in one comparison and with no server,
  that the fixture or the document shape has changed under a benchmark. Without
  it, a regression in the document shape reads as a regression in speed.

  *Why N carries no provisional value.* It is a size derived from a document
  shape and a fixture, and neither exists: `FR-CTX-006` roughly doubles the
  column volume and `FR-CTX-010` doubles it again, so any figure written today
  would be an arithmetic guess over two multipliers and a fixture nobody has
  built. `BR-PERF-006` forbids inventing it, and unlike the four figures of
  `NFR-PERF-014` no root document supplies one.

  *Amended in the fifth edition.* `OQ-060` held this value open and is closed;
  what remains is not a question but a measurement this requirement now
  schedules.

- **WL-003** — the small workload. A database of one table, 12 columns, and 3
  indexes. **The three count the primary key**, on the rule `WL-001` states and
  for the grounds stated there: they are `PRIMARY`, one unique key and one
  composite key, and the composite key is one index and not one per column.

  *Rationale.* This is the common path. Iteration over objects is the caller's
  job under `BR-RND-002`, so the invocation a caller repeats is a read of one
  object, and the workload that measures it must be the small one.

- **BR-PERF-002**: `scripts/mariadb/seed.sql` keeps its own job and is not
  merged into the benchmark fixture. Its purpose is exhaustive variety at
  minimal volume, for correctness; `seed-bench.sql`'s purpose is volume at
  minimal variety, for measurement. One fixture serving both would hide an N+1,
  which is invisible at ten tables, or would make the correctness suite pay for
  200 tables on every run.

## The target set

- **NFR-PERF-018**: A **target** SHALL be one of exactly the following four,
  and a measurement, a baseline, and a budget SHALL each be stated against one
  of them:

  | System | Architecture | Target |
  |---|---|---|
  | Linux | amd64 | `x86_64-unknown-linux-musl`, statically linked |
  | Linux | arm64 | `aarch64-unknown-linux-musl`, statically linked |
  | macOS | amd64 | `x86_64-apple-darwin` |
  | macOS | arm64 | `aarch64-apple-darwin` |

  No target of this set SHALL be second class: a result that fails on one of
  the four fails, whichever it is.

  *What the parity clause reaches, and what it does not, stated in the eleventh
  edition.* It reaches **results**: a result that fails on one of the four
  fails. It is not a claim that every **instrument** exists on every target, and
  the eleventh edition found one that does not — the syscall trace of
  `NFR-PERF-007`, which no macOS host affords under its default protection. The
  parity clause is unchanged by that, and it is the reason the gap had to be
  written down rather than papered over: precisely because no target is second
  class, the Linux observation may not be credited to the two macOS targets,
  which is what `NFR-PERF-005` now forbids in its own text. Every requirement
  of this file is still verified on all four targets. Two clauses of one
  requirement are verified on two of them by a weaker instrument, and that
  requirement says which two and why.

  *Rationale.* `NFR-PERF-012` already forbade comparing measurements across
  targets and `BR-PERF-003` already counted them, but nothing in this corpus
  said what a target is or how many there are — so neither rule had a subject
  it could name. The set is stated here because this file is the only one that
  depends on it.

  *Provenance.* The two macOS targets and the two architectures come from the
  root `CLAUDE.md` and are contradicted by no decision. The two Linux targets
  are the settled answer to the open question that file recorded beside them:
  Linux is built against `musl` and linked statically, so one artefact per
  architecture runs wherever that architecture does, with no dependency on the
  host's C library version. The correction owed to `CLAUDE.md` is recorded as
  `DIV-041`.

  *Amended in the fifteenth edition.* This note said that `CLAUDE.md`'s matrix
  **still names** the `gnu` triples. It did name them, from `a8c5390` until
  `0ea5624` removed the matrix; what that file now carries is the deferral
  `DIV-041` quotes, which is why that entry is still due while this clause was
  not. The re-check of [upstream-divergences.md](upstream-divergences.md) found
  it, and the rule that obliges the re-check is the fifth in
  [README.md](README.md#maintenance-debt).

  *Accepted cost, and it is the one that reaches behaviour.* A statically
  linked `musl` binary resolves names through `musl`'s own `getaddrinfo`,
  which does not load the platform's name-service modules. A name that resolves
  only through such a module — an LDAP or mDNS source configured in
  `nsswitch.conf` — does not resolve for `tpl` on a Linux target, while the
  same name resolves for a dynamically linked client on the same host. The
  outcome is the ordinary one: the DNS phase of `FR-CONF-005` fails and
  `FR-ERR-027` routes it to `69` (`EX_UNAVAILABLE`), which `FR-ERR-001` already
  tells the caller is repeatable and worth checking the host over. What the
  cost obliges is precision in the message, per `FR-ERR-034`: the `cause` line
  says that the name did not resolve, and does not say that a host refused a
  connection.

## Measurement protocol

- **NFR-PERF-009**: A measurement SHALL be the median of at least 200 runs,
  taken after at least 20 warmup runs.

- **NFR-PERF-010**: A measurement SHALL be taken with no intervening shell, with
  a warm page cache, on an idle host running on mains power, and the first
  execution of a freshly built binary SHALL be discarded.

- **NFR-PERF-011**: A run whose relative standard deviation exceeds 5% SHALL be
  invalid, and its result SHALL NOT be recorded as a baseline.

- **NFR-PERF-012**: A recorded measurement SHALL name the target it was taken
  on, drawn from the set of `NFR-PERF-018`. Measurements taken on different
  targets SHALL NOT be compared with each other.

  *Amended in the fourth edition.* A measurement that reaches a server SHALL
  additionally name the server series it was taken against, and measurements
  taken against different series SHALL NOT be compared with each other. The
  reason is the reason the rule already gives for the target: the four series of
  `FR-SRV-015` are four different programs answering the catalogue queries, and
  a number that does not say which one answered cannot be a baseline. This binds
  only the budgets that touch a server; `NFR-PERF-013` keeps every normative
  budget away from one.

- **NFR-PERF-013**: Every normative budget SHALL run over `--context` or over
  the cache, and SHALL therefore require no server.

  *Rationale.* A budget that needs a server cannot gate continuous integration,
  and one that needs a network is flaky when it does.

## The budget set

- **NFR-PERF-014**: The budget set SHALL be exactly the following nine, and no
  others. Every figure in the last column is **provisional**, under the terms
  of `NFR-PERF-019`:

  | Budget | Workload | Server | Normative | Provisional figure |
  |---|---|---|---|---|
  | `tpl --version` | none | no | no | < 5 ms wall time |
  | `tpl --help` | none | no | no | < 5 ms wall time |
  | Startup to the first byte of useful work | none | no | no | < 10 ms |
  | `tpl schema dump` | `WL-001` | yes | no | < 500 ms, server time included |
  | A cache-served read of one object | `WL-003` | no | **yes** | none |
  | The failure path: a `64`, and a `66` with nearest-match over every existing name | `WL-001` | no | no | none |
  | `tpl help --format json` | none | no | no | none |
  | The canonical loop of 200 invocations | `WL-001` | yes | no | none |
  | Peak resident memory | `WL-001` | yes | no | < 32 MiB |

  *Provenance of the five figures.* They are the four figures the root
  `CLAUDE.md` states, adopted unchanged and marked provisional, filling five
  cells. `< 5 ms` is one line there covering both `tpl --version` and
  `tpl --help`; `NFR-PERF-014` splits that budget in two, per `OQ-052`, and
  each half inherits the same figure until a measurement separates them. The
  `< 500 ms` line is stated there as dominated by server time, which is why
  this table records the server column beside it. The `< 32 MiB` figure is
  stated there for a database of 200 tables, which is `WL-001`. The correction
  owed to that file was `DIV-035`, and it is discharged.

  *Amended in the fifteenth edition.* The paragraph above is a record of where
  five provisional figures came from, and it is true of `CLAUDE.md` as it stood
  when they were adopted; `0ea5624` has since removed the budget table it
  quotes, which is what discharged `DIV-035`. Nothing here changes: the figures
  keep the provenance they were adopted with, `NFR-PERF-019` still marks them
  provisional, and `NFR-PERF-020` still removes each on the first real
  measurement. Only the sentence claiming a correction is owed is corrected.

  *Why the other four are blank.* `CLAUDE.md` supplies no figure for them, and
  `BR-PERF-006` forbids inventing one. Three of the four did not exist as
  budgets before this specification created them, and the fourth — the
  200-invocation loop — replaced a `render --all-tables` line whose figure
  measured a quantity `FR-RND-007` removed, per `BR-PERF-005`, so that figure
  cannot be carried across. A blank is a budget that is named, measured when
  there is code, and ratified under `NFR-PERF-020`; it is not a budget that is
  unconstrained, because `NFR-PERF-017` still holds it to its own first
  recorded baseline.

  *Checked in the thirty-first edition against a failure on a live database,
  and no row is added.* The failure-path row runs over `WL-001` with
  `Server: no`, so nothing in this set measures a `64` or a `66` taken against
  a server, and the question put was whether the set gains a tenth budget that
  does. It does not. `BR-PERF-004` states what this row exists to measure — the
  edit distance computed against every existing name, which over `WL-001` is
  200 names for a table and more for a template — and that quantity needs no
  server. What a live-database row would add is the catalogue read the
  invocation performs before it fails, and that is two quantities this set
  already carries: the statement count is a requirement of form in
  `NFR-PERF-001` and `NFR-PERF-002`, and the wall time is the
  `tpl schema dump` row over the same workload. A tenth budget would measure
  their sum and attribute nothing.

  The one failure this set could once have been asked for, and can no longer,
  is a `tpl render` whose template does not exist: the thirtieth edition moved
  template resolution to the fourth step of `FR-ERR-006`, so that invocation
  opens no connection at all and is measured by the row already here.

  *Rejected: a tenth row, `The failure path against a live server`, over
  `WL-001` with `Server: yes`.* `NFR-PERF-014` fixes the set at nine "and no
  others", so adding one is a change to the set and not an addition to a list,
  and `BR-PERF-003` counts the cost of the set against `NFR-PERF-018`'s four
  targets. It would also be the sixth budget needing the fixture, where
  `BR-PERF-007` counts five, and the fourth needing a server as well.
  `BR-PERF-001` prefers a requirement of form where both would catch
  the same defect, and here the form exists and is stronger than the figure
  would be.

  *Corrected in the thirty-third edition, on both of its counts.* The clause
  read *the sixth budget needing a server, where `BR-PERF-007` already names
  five as unmeasurable until the fixture is complete*. Three rows of the table
  above carry `Server: yes`, so a tenth would be the fourth needing a server
  and not the sixth; the count of five is `BR-PERF-007`'s count of the budgets
  needing the **fixture**, which is what a `WL-001` row joins. And the second
  half was made false by the fixture being completed, which is the same edition
  restating `BR-PERF-007`. The rejection is unchanged and so is its arithmetic:
  a tenth row is still a change to a closed set, and the ground `BR-PERF-001`
  supplies is still the one that decides it.

  *Rejected: moving the failure-path row to `Server: yes`.* It would remove the
  one measurement of the suggestion path that can run with no server, which is
  the half of it `BR-PERF-007` says is measurable as soon as the fixture has
  been loaded once.

  *Amended in the fifth edition.* The last column previously cited an open
  question per row, `OQ-051` through `OQ-059`. Those ten entries are closed:
  what they held open was a **number**, and this requirement now carries a
  provisional number or a stated blank for every one of them, under a protocol
  that says what happens to it. The `Server` column is new, and exists so that
  `BR-PERF-007` can be true.

- **NFR-PERF-019**: A figure in `NFR-PERF-014` marked provisional SHALL be a
  working target and SHALL NOT be a limit. It SHALL NOT fail a change, SHALL
  NOT be recorded in `BENCHMARKS.md` as a baseline, and SHALL be superseded by
  the first measurement of that budget taken under `NFR-PERF-009` through
  `NFR-PERF-012` and ratified under `NFR-PERF-020`.

  *Rationale.* The alternative to a marked provisional figure is one of two
  worse things: a blank, which leaves an implementer with no target at all and
  no way to know whether a result is surprising; or an unmarked figure, which
  is a limit nobody measured and which a reader cannot distinguish from one
  somebody did. Marking is what makes the difference readable, and it is the
  same discipline `FR-SRV-019` applies to a table of version numbers.

  *Accepted cost.* Until the first measurement, a reader must consult the mark
  as well as the number. That is deliberate: a number in this file that is not
  marked provisional and is not in `BENCHMARKS.md` is a defect.

- **NFR-PERF-020**: A provisional figure SHALL become a ratified one WHEN, and
  only when, all of the following hold:

  1. the budget has been measured under `NFR-PERF-009` through `NFR-PERF-012`,
     on a target of `NFR-PERF-018`, over the workload `NFR-PERF-014` names for
     it;
  2. the measurement is valid under `NFR-PERF-011`;
  3. the measured value is recorded in `BENCHMARKS.md` against that budget and
     that target; and
  4. `NFR-PERF-014` is amended to remove the provisional figure, which the
     recorded baseline replaces.

  A ratified figure SHALL live in `BENCHMARKS.md` and SHALL NOT be restated
  here, per `BR-PERF-006`. From ratification the budget is governed by
  `NFR-PERF-017` on that target.

  *Rationale.* Without a stated gate, "provisional" has no end and the figures
  stay in this file for ever, which is the outcome `BR-PERF-006` exists to
  prevent. The gate is per budget and per target rather than for the set,
  because the four targets are measured at different times and a budget
  ratified on one of them is not ratified on the others.

- **NFR-PERF-015**: The cache-served read of one object over `WL-003` SHALL be
  the one normative budget. It carries no provisional figure, and its ratified
  target SHALL be named in the text of this requirement — not only in
  `BENCHMARKS.md` — once `NFR-PERF-020` is satisfied for it.

  *Rationale.* It is the invocation a caller now repeats once per object, since
  `FR-RND-002` gives one render per invocation and `BR-RND-002` moves iteration
  to the caller. It is also the only budget that can be enforced with no server,
  and therefore the only one that could ever gate a pipeline.

  *Why this one is named here.* A normative budget is the one figure that can
  fail a change on its own merits rather than by regression, so a reader of the
  specification must be able to see it without opening `BENCHMARKS.md`. It is
  the single exception to `BR-PERF-006`, and it is stated as an exception.

- **NFR-PERF-016**: Every budget other than the normative one SHALL carry the
  no-regression rule of `NFR-PERF-017` only, and SHALL NOT carry a ratified
  target. A provisional figure is not a target, per `NFR-PERF-019`.

- **NFR-PERF-017**: A measurement worse than the baseline recorded in
  `BENCHMARKS.md` for the same budget on the same target of `NFR-PERF-018`
  SHALL fail the change that produced it. A measurement with no baseline for
  that budget on that target SHALL NOT fail a change, and SHALL be recorded as
  the baseline.

- **BR-PERF-003**: Making every budget normative on every target was rejected.
  `NFR-PERF-014` lists nine budgets and `NFR-PERF-018` four targets, so it is
  thirty-six figures to maintain, and a pipeline would have to run all four
  targets for any one of them to mean anything.

  *Corrected in the fifth edition.* The rule previously read "four budgets over
  four targets is thirty-two figures". Both halves were wrong: the budget set
  has been nine since the third edition, and four times four is sixteen. The
  argument is unchanged and the arithmetic now follows from the two
  requirements it counts, rather than from a number written beside them.

- **BR-PERF-004**: The failure path is a budget because a `66` with a
  nearest-match suggestion computes an edit distance against every existing
  name, per `FR-ERR-019`, and over `WL-001` that is 200 names for a table, and
  more for a template. A wrong invocation is the invocation a calling agent
  makes most often while it is finding its way, and it must cost what
  `tpl --version` costs.

- **BR-PERF-005**: The 200-invocation loop replaces the `render --all-tables`
  line of the original budget table, which measured a flag that `FR-RND-007`
  removed. What it measured — the marginal cost of the two-hundredth object — is
  now 200 process startups rather than 200 iterations inside one process, and it
  is a different quantity that has to be measured separately.

## Business rules

- **BR-PERF-006**: A **ratified** figure lives in `BENCHMARKS.md` and nowhere
  else. This specification names each budget and says where its ratified value
  lives; it does not restate that value, and a value that appeared in both
  places would be two sources for one truth. A budget for which no figure was
  supplied and none has been measured is named and left unvalued rather than
  given an invented one.

  *Amended in the fifth edition.* The rule now distinguishes a ratified figure
  from a provisional one. A provisional figure is carried here, marked, under
  `NFR-PERF-019`, and it is not two sources for one truth because it is not in
  `BENCHMARKS.md` at all: `NFR-PERF-020` makes recording it there the very step
  that removes it from here. The one exception in the other direction is the
  normative budget of `NFR-PERF-015`, whose ratified target is stated in both
  places deliberately.

- **BR-PERF-007**: Five of the nine budgets need the fixture and four do not,
  and the fixture is complete. `scripts/mariadb/` carries its container
  definition, `setup.sql`, `seed.sql` and `seed-bench.sql`, buildable and
  loadable at each of the four series of `FR-SRV-015`, and `seed-bench.sql`
  realises both `WL-001` and `WL-003`. **No budget of `NFR-PERF-014` is
  blocked by the fixture.** What now stands between a budget and a figure is a
  measurement and the gate of `NFR-PERF-020`, and for three of the nine a
  running server as well, per the `Server` column of that requirement.

  *Restated in the thirty-third edition, because the condition it named has
  been met.* The rule read *the `seed-bench.sql` this file requires is still
  absent, and `WL-001` is what needs it*, and that file exists: it loads on
  each of the four series and produces both workloads at every count this file
  states for them. The rule keeps its identifier and its arithmetic — five
  budgets needed the fixture, four did not, and which are which is what its
  citations read it for — and states the condition that now holds instead of
  the one that has passed.

  *Rejected: withdrawing the rule.* It was the live alternative, because a rule
  whose blocker is gone can look spent. It is rejected because the distinction
  the rule draws is not the blocker: which budgets need a fixture, which need a
  server as well, and which need neither is a permanent property of the budget
  set, read by `NFR-PERF-014` twice in its own rejections and by two entries of
  [upstream-divergences.md](upstream-divergences.md). Withdrawing it would
  retire an identifier under the scheme of the
  [README](README.md#identifier-scheme) and leave every one of those citations
  resolving to a note about a rule instead of to a rule.

  The four that need no fixture and no server are the four whose workload is
  `none` in `NFR-PERF-014`: `tpl --version`, `tpl --help`, startup to the first
  byte of useful work, and `tpl help --format json`. Each of the four is
  measurable as soon as there is a binary, and each already carries or will
  carry a figure without anything being stood up. Two further budgets need the
  fixture but no server — the cache-served read over `WL-003`, which
  `NFR-PERF-013` requires to run over the cache, and the failure path — so they
  are measurable now that the fixture has been loaded once.

  *Amended in the sixth edition.* The whole directory was absent when this rule
  was written. Three of its four files then existed, so the blocker was
  narrowed and named precisely rather than as the absence of everything.

  *Corrected in the fifth edition.* The rule previously said none of the
  budgets can be measured. That over-claimed: it made the whole set look
  blocked when nearly half of it is not, and it was the reason the four
  server-free budgets went unmeasured longer than they had to. The `Server`
  column of `NFR-PERF-014` now carries the distinction per budget.

## Dependencies

- [global-flags.md](global-flags.md) — `FR-GLOB-017`, which makes the query
  count observable.
- [server-contract.md](server-contract.md) — the statement and connection
  invariants observed from the server side.
- [cache-documents.md](cache-documents.md) — what a cache hit serves, and the
  completeness record that decides whether it may.
- [catalogue-coverage.md](catalogue-coverage.md) — what a full read must return,
  and therefore what a query count is a count of.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `FR-ERR-027` and
  `FR-ERR-034`, which govern the DNS outcome the `musl` linkage of
  `NFR-PERF-018` makes reachable; and `FR-ERR-031`, which rejected a
  verification mechanism on the ground `NFR-PERF-005` rejects one on.
- [project-and-discovery.md](project-and-discovery.md) — `FR-PROJ-025`, the
  commands `NFR-PERF-005` constrains and the clause that says what its
  discovery clause forbids; `FR-PROJ-012` and `FR-PROJ-013`, which make the
  differential run of `NFR-PERF-007` possible for `tpl init`; and
  `FR-PROJ-016`, the one upward look any command of that set makes.

## Open questions

None specific to this module. `OQ-051` through `OQ-060` are closed and listed
under [Closed](open-questions.md#closed): each held a number open, and
`NFR-PERF-014`, `NFR-PERF-019` and `NFR-PERF-020` replace the ten questions
with one protocol that carries a provisional figure or a stated blank for every
budget and says what turns either into a limit. What remains is measurement
work, not an open question — `BR-PERF-007` says what the fixture no longer
blocks and what a budget still needs.
