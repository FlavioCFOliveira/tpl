---
title: Performance Requirements
status: draft
last-reviewed: 2026-09-10
related: [server-contract.md, cache-documents.md, global-flags.md, catalogue-coverage.md]
---

# Performance Requirements

## Overview

`tpl` is invoked repeatedly, often inside a loop, and its speed is a first-order
requirement. This file states the part of that requirement a specification can
own: the properties of the program that are observable from outside it,
permanent, and falsifiable without a stopwatch.

No figure in milliseconds or bytes appears here. There is no code yet, so no
figure is defensible, and a corpus of round numbers nobody measured constrains
nothing. What does constrain, from the first commit, is a small set of
requirements of form — the number of catalogue queries does not depend on the
number of objects, a cache hit opens no connection, one invocation opens at most
one connection. Those catch exactly the defects the figures were meant to catch,
and they can be checked today.

The figures live outside this specification, in the project's architecture
decision records and in `BENCHMARKS.md`. This file names each budget, says which
workload it runs over, says which one is normative, and cites the open question
that holds its unmeasured value.

## Scope

In scope: the requirements of form, the three reference workloads, the
measurement protocol, and the budget set with its normative member.

Out of scope: every numeric target and every measured baseline; the tools used
to measure; and the implementation choices that would satisfy a budget.

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

- **NFR-PERF-002**: The number of catalogue queries the system issues to read
  one named object SHALL NOT depend on the number of objects in the database.

  *Rationale.* Reading the whole catalogue to answer `tpl schema table orders`
  is the mirror-image defect of the N+1, and it is the more likely one once the
  full-read path exists.

- **NFR-PERF-003**: A cache hit SHALL open no connection and SHALL issue no
  catalogue query.

- **NFR-PERF-004**: One invocation SHALL open at most one connection.

- **NFR-PERF-005**: Every command named by `FR-PROJ-025` SHALL perform no
  project discovery, SHALL read no configuration file, and SHALL open no
  connection. Those commands are `tpl init`, every form of `help`, and every
  form of `version`.

  *Amended in the third edition.* The first edition covered the two version
  forms only and left the help forms and `tpl init` unsettled, which was
  recorded as an open question. `FR-PROJ-025` now names the set functionally,
  and this requirement states the observable consequence, verified from outside
  the process per `NFR-PERF-007`: no `stat` of an ancestor directory, no open
  of `.tpl/.cfg`, no socket.

- **NFR-PERF-006**: A command that requires no catalogue data SHALL open no
  connection. This covers every `template` subcommand, every `cfg` subcommand
  except `database test`, `help`, `version`, `init`, and any `tpl render`
  invoked with `--context`.

- **NFR-PERF-007**: Each requirement of this section SHALL be verified by an
  observation made outside the process — the statements the server receives, the
  connections it accepts, or the files the process opens — and SHALL NOT be
  verified by reading the source.

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

- **WL-002** — the verification scalar. The compact `tpl schema dump` of
  `WL-001` SHALL be N bytes, within ±2%. The value of N is unmeasured; see
  [OQ-060](open-questions.md#oq-060).

  *Rationale.* A single scalar detects, in one comparison and with no server,
  that the fixture or the document shape has changed under a benchmark. Without
  it, a regression in the document shape reads as a regression in speed.

- **WL-003** — the small workload. A database of one table, 12 columns, and 3
  indexes.

  *Rationale.* This is the common path. Iteration over objects is the caller's
  job under `BR-RND-002`, so the invocation a caller repeats is a read of one
  object, and the workload that measures it must be the small one.

- **BR-PERF-002**: `scripts/mariadb/seed.sql` keeps its own job and is not
  merged into the benchmark fixture. Its purpose is exhaustive variety at
  minimal volume, for correctness; `seed-bench.sql`'s purpose is volume at
  minimal variety, for measurement. One fixture serving both would hide an N+1,
  which is invisible at ten tables, or would make the correctness suite pay for
  200 tables on every run.

## Measurement protocol

- **NFR-PERF-009**: A measurement SHALL be the median of at least 200 runs,
  taken after at least 20 warmup runs.

- **NFR-PERF-010**: A measurement SHALL be taken with no intervening shell, with
  a warm page cache, on an idle host running on mains power, and the first
  execution of a freshly built binary SHALL be discarded.

- **NFR-PERF-011**: A run whose relative standard deviation exceeds 5% SHALL be
  invalid, and its result SHALL NOT be recorded as a baseline.

- **NFR-PERF-012**: A recorded measurement SHALL name the target it was taken
  on. Measurements taken on different targets SHALL NOT be compared with each
  other.

- **NFR-PERF-013**: Every normative budget SHALL run over `--context` or over
  the cache, and SHALL therefore require no server.

  *Rationale.* A budget that needs a server cannot gate continuous integration,
  and one that needs a network is flaky when it does.

## The budget set

- **NFR-PERF-014**: The budget set SHALL be exactly the following, and no
  others:

  | Budget | Workload | Normative | Unmeasured value |
  |---|---|---|---|
  | `tpl --version` | none | no | [OQ-051](open-questions.md#oq-051) |
  | `tpl --help` | none | no | [OQ-052](open-questions.md#oq-052) |
  | Startup to the first byte of useful work | none | no | [OQ-053](open-questions.md#oq-053) |
  | `tpl schema dump` | `WL-001` | no | [OQ-054](open-questions.md#oq-054) |
  | A cache-served read of one object | `WL-003` | **yes** | [OQ-055](open-questions.md#oq-055) |
  | The failure path: a `64`, and a `66` with nearest-match over every existing name | `WL-001` | no | [OQ-056](open-questions.md#oq-056) |
  | `tpl help --format json` | none | no | [OQ-057](open-questions.md#oq-057) |
  | The canonical loop of 200 invocations | `WL-001` | no | [OQ-058](open-questions.md#oq-058) |
  | Peak resident memory | `WL-001` | no | [OQ-059](open-questions.md#oq-059) |

- **NFR-PERF-015**: The cache-served read of one object over `WL-003` SHALL be
  the one normative budget. Its target SHALL be named in the text of this
  requirement once it has been measured.

  *Rationale.* It is the invocation a caller now repeats once per object, since
  `FR-RND-002` gives one render per invocation and `BR-RND-002` moves iteration
  to the caller. It is also the only budget that can be enforced with no server,
  and therefore the only one that could ever gate a pipeline.

- **NFR-PERF-016**: Every budget other than the normative one SHALL carry the
  no-regression rule only, and SHALL NOT carry a target.

- **NFR-PERF-017**: A measurement worse than the baseline recorded in
  `BENCHMARKS.md` for the same budget on the same target SHALL fail the change
  that produced it.

- **BR-PERF-003**: Four fully normative targets were rejected. Four budgets over
  four targets is thirty-two figures to maintain, and a pipeline would have to
  run all four for any one of them to mean anything.

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

- **BR-PERF-006**: The figures live in the project's architecture decision
  records and in `BENCHMARKS.md`. This specification names each budget and cites
  where its value lives; it does not restate the value, and a value that appears
  in both places would be two sources for one truth.

- **BR-PERF-007**: None of the budgets can be measured, and neither `WL-001` nor
  `WL-003` can be realised, until `scripts/mariadb/` exists. That directory,
  with its container definition, `setup.sql`, `seed.sql`, and the
  `seed-bench.sql` this file requires, is absent from the repository, and it
  blocks every open question this file records.

## Dependencies

- [global-flags.md](global-flags.md) — `FR-GLOB-017`, which makes the query
  count observable.
- [server-contract.md](server-contract.md) — the statement and connection
  invariants observed from the server side.
- [cache-documents.md](cache-documents.md) — what a cache hit serves, and the
  completeness record that decides whether it may.
- [catalogue-coverage.md](catalogue-coverage.md) — what a full read must return,
  and therefore what a query count is a count of.

## Open questions

- [OQ-051](open-questions.md#oq-051) through
  [OQ-059](open-questions.md#oq-059) — the value of each budget in the set of
  `NFR-PERF-014`.
- [OQ-060](open-questions.md#oq-060) — the value of N in the `WL-002` scalar.
