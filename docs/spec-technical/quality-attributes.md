---
title: Quality Attributes
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md, open-decisions.md, overview.md]
---

# Quality Attributes

## What this document is

The properties of form the design is held to from the first commit, the nine
budgets and what each is measured against, the protocol that decides when a
figure becomes a limit, and the reliability and equivalence properties a caller
may rely on. Every statement cites the requirement that forces it; no
requirement text is reproduced here.

**No figure appears in this document.** A budget's provisional figure lives in
`NFR-PERF-014` and a ratified one in `BENCHMARKS.md`, and `NFR-PERF-020` is the
step that moves it from the first place to the second. A third copy would be the
copy nobody updates, which is the point `DIV-035` carries against the root
coordination document. A **budget** is a requirement's target; a **baseline**
is a measurement. The two are never written in one place.

The tests and the harness that take a measurement are `verification.md`; the
build, the target matrix and the release gates are `operations.md`.

## The six requirements of form

`BR-PERF-001` prefers a form to a figure wherever both catch the same defect,
so these six — not the budgets — are what constrains the design. Each is
observable, permanent, and checkable with no stopwatch.

| Requirement | The defect it forbids | The design decision it forces | Described in |
|---|---|---|---|
| `NFR-PERF-001` | A query per object on a full read | The catalogue reader is organised per object **kind**: a fixed set of queries, each returning every object of its kind, so the count over `WL-001` equals the count over `WL-003` | `architecture.md`, `interfaces.md` |
| `NFR-PERF-002` | Reading the whole catalogue to answer for one object | The reader carries a named-object path distinct from the full-read path, with the name restricting the query the server receives | `architecture.md`, `interfaces.md` |
| `NFR-PERF-003` | A connection opened before the cache is consulted | The cache is a read-through layer **in front of** the reader: a hit is served from disk with no connection and no query (`FR-CACHE-006`, `FR-CACHE-007`) | `architecture.md` |
| `NFR-PERF-004` | A pool, or a second connection for any purpose | One connection at most, opened late and closed when the read ends; it is also what rejected a pre-flight TCP connect for phase attribution in [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), and it is verifiable from the server side (`FR-SRV-014`) | `architecture.md` |
| `NFR-PERF-005` | Startup work a command does not need | The four commands of `FR-PROJ-025` are classified **before** any filesystem access: no `stat` of an ancestor, no open of `.tpl/.cfg`, no socket — and, per [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), no timer thread | `architecture.md` |
| `NFR-PERF-006` | A connection opened by dispatch rather than by need | Connection establishment is lazy: the reader opens it when it is about to read, so every command that needs no catalogue opens none | `architecture.md` |

Two obligations attach to the six and belong elsewhere. Each is verified by an
observation made **outside** the process and never by reading the source
(`NFR-PERF-007`), which is `verification.md`; and the catalogue-query count is
observable on the diagnostic stream (`NFR-PERF-008`, `FR-GLOB-017`), which makes
one diagnostic line structurally load-bearing although stderr is not contract —
that is `operations.md`, settled in
[`OD-17`](open-decisions.md#od-17--observability).

## Determinism

`NFR-DET-001` fixes the extent of the contract: **stdout**, byte-identical for
one invocation against one project state and one database state. stderr is
neither deterministic nor contract, and cannot be, because `FR-GLOB-017`
requires phase timings that differ on every run. `DIV-039` is the register
entry that carries the unqualified form of this rule against the root
documents; this folder cites it and does not restate it.

| Consequence for the built system | Forced by | Described in |
|---|---|---|
| Every ordering is applied by the emitter and none is inherited from the server or the filesystem: one default and six named exceptions, three of which the default would silently corrupt | `NFR-DET-002` | `interfaces.md` |
| No terminal detection on any path, and no colour or ANSI byte on either stream — which also lets the argument parser be built with its colour support off | `NFR-DET-003`, `NFR-DET-004` | `technology-stack.md` |
| The environment is not an input; `${VAR}` inside the configuration file is the single exception | `FR-CLI-021`, `FR-CLI-023` | `security.md` |
| No load time in the output of a read: `loaded_at` appears in `meta.json` and in `tpl cache status` and nowhere else, because a load time in a read would make two identical invocations against an unchanged project differ | `FR-CDOC-012`, `FR-CDOC-013`, `BR-CDOC-003` | `data-model.md` |

**The single source of non-reproducibility is `now`** (`NFR-DET-005`,
`FR-CTX-030`): a template that does not reference it produces byte-identical
output between runs against unchanged inputs. The clock is read once per
invocation (`FR-CTX-029`) and emitted in the one fixed form of `FR-CTX-028`; the
clock source and the one grammar used in both directions are settled in
[`OD-25`](open-decisions.md#od-25--the-clock-source-for-now) and described in
`technology-stack.md`.

The cache is part of the project state that `NFR-DET-001` quantifies over. What
a cache-served document additionally does not promise is `FR-CDOC-015`, and
`FR-CDOC-016` is the signal by which a consumer recognises that neither promise
applies.

## The three reference workloads

| Workload | What it is | Realised by |
|---|---|---|
| `WL-001` | The large workload, against which every budget that needs volume is measured | `scripts/mariadb/seed-bench.sql`, as schema `freight_wl001` |
| `WL-002` | The verification scalar: the size of the compact dump of `WL-001`, which detects a change in the fixture or in the document shape with no server. **Unvalued**, and `BR-PERF-006` forbids inventing it | The same fixture; the fixture now exists, so the value can be computed and has not been |
| `WL-003` | The small workload — the common path, because `FR-RND-002` gives one render per invocation and `BR-RND-002` moves iteration to the caller | The same file, as schema `freight_wl003` |

`BR-PERF-002` keeps `seed.sql` and `seed-bench.sql` apart deliberately: one
fixture serving both would hide an N+1 or make the correctness suite pay for the
benchmark volume. **`seed-bench.sql` was written on 2026-09-21**, in the sprint
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001) named, and it is
DDL alone, loaded on demand by `seed-bench.sh` rather than baked into the image;
that script verifies every count each workload states. The counts, the schema
names and the load procedure are `scripts/mariadb/README.md`'s and are not
restated here. The fixture itself is `verification.md`.

## The nine budgets

`NFR-PERF-014` fixes the set as exactly these nine and no others. The last
column is the **standing of the figure**, in the vocabulary `NFR-PERF-019` and
`BR-PERF-006` provide: *provisional* for a figure carried in `NFR-PERF-014` and
marked, *unvalued* for a budget named with no figure supplied by any root
document and none invented.

| # | Budget | Workload | Needs a server | Normative | Standing |
|---|---|---|---|---|---|
| 1 | `tpl --version` | none | no | no | Provisional, not ratified |
| 2 | `tpl --help` | none | no | no | Provisional, not ratified |
| 3 | Startup to the first byte of useful work | none | no | no | Provisional, not ratified |
| 4 | `tpl schema dump` | `WL-001` | yes | no | Provisional, not ratified |
| 5 | A cache-served read of one object | `WL-003` | no | **yes** | Unvalued, not ratified |
| 6 | The failure path: a `64`, and a `66` with nearest match | `WL-001` | no | no | Unvalued, not ratified |
| 7 | `tpl help --format json` | none | no | no | Unvalued, not ratified |
| 8 | The canonical loop of 200 invocations | `WL-001` | yes | no | Unvalued, not ratified |
| 9 | Peak resident memory | `WL-001` | yes | no | Provisional, not ratified |

**None of the nine is ratified.** `BENCHMARKS.md` carries no entry recorded
under `NFR-PERF-020`; its one entry states its own standing as a
driver-selection record and departs from the protocol in three ways it names.
Until a budget is ratified, `NFR-PERF-019` makes its provisional figure a
working target and not a limit: it fails no change and is not a baseline.

**Budget 5 is the one normative budget** (`NFR-PERF-015`). Two properties make
it so, and both are structural rather than editorial: it is the invocation a
caller repeats once per object, since `FR-RND-002` gives one render per
invocation and `BR-RND-002` moves iteration to the caller; and it runs over the
cache, so it needs no server (`NFR-PERF-013`) and is therefore the only budget
that could ever gate a pipeline. It carries no provisional figure, and once it
is ratified its target is to be stated in the text of `NFR-PERF-015` as well as
in `BENCHMARKS.md` — the single deliberate exception to `BR-PERF-006`. Every other
budget carries the no-regression rule of `NFR-PERF-017` only and no ratified
target (`NFR-PERF-016`).

**Which of the nine the fixture gates.** Budgets 4, 6, 8 and 9 are measured over
`WL-001` and budget 5 over `WL-003`. Budgets 1, 2, 3 and 7 need neither fixture
nor server; each waits only on the command it measures. Of the five that need
the fixture, three also need a server — 4, 8 and 9 — and those three can never
be normative, which is exactly what `NFR-PERF-013` requires.

**Recorded contradiction — `BR-PERF-007` against the tree.** That rule states
that *"the `seed-bench.sql` this file requires is still absent, and `WL-001` is
what needs it"*, and both workloads were realised on 2026-09-21:
`scripts/mariadb/seed-bench.sql` carries them, `seed-bench.sh` loads and
verifies them, and a measurement over both has been taken and recorded in
`scripts/mariadb/README.md`. **Both readings are recorded and neither is chosen
here.** `/specification` governs, so nothing of this folder is written around
the rule; what the rule says of the repository is simply older than the
repository, which is the fifth validation rule's own case. The defect is owed to
the functional owner and names `BR-PERF-007`. What follows for the five budgets
— whether each is now measurable, and which move from provisional under the gate
of `NFR-PERF-019` — is the functional owner's to restate and is not decided
here.

**Recorded discrepancy.** [traceability.md](traceability.md) §23 reads *"Nine
budgets, one normative, five needing a server"*. `NFR-PERF-014`'s `Server`
column marks three, and `BR-PERF-007` counts five as needing the **fixture**.
The table above follows `NFR-PERF-014`, which is the requirement. The
distinction is load-bearing: it decides which budgets could gate a pipeline
under `NFR-PERF-013`, so the row is reported rather than worked around.

## The measurement protocol

The protocol is normative from now, and it is stated in `NFR-PERF-009` through
`NFR-PERF-012`; its values are not restated here. What it obliges of this
folder and of anything that records a result:

| Obligation | Requirement |
|---|---|
| A recorded measurement names the **target** it was taken on, drawn from the four of `NFR-PERF-018`; measurements on different targets are never compared | `NFR-PERF-012` |
| A measurement that reaches a server additionally names the **series** it was taken against, because the four series of `FR-SRV-015` are four different programs answering the catalogue queries | `NFR-PERF-012`, as amended |
| A run that fails the stated dispersion limit is invalid and is not recorded as a baseline | `NFR-PERF-011` |
| Every normative budget runs over `--context` or over the cache, and therefore needs no server | `NFR-PERF-013` |

**Ratification** is per budget and per target, under the four conditions of
`NFR-PERF-020`; the four targets are measured at different times, so a budget
ratified on one is not ratified on the others. The step that records a measured
value in `BENCHMARKS.md` is the same step that removes the provisional figure
from `NFR-PERF-014`. From ratification the budget is governed by `NFR-PERF-017`:
a measurement worse than the recorded baseline for that budget on that target
fails the change. A budget with **no** baseline fails nothing and its first
valid measurement becomes the baseline — so an unvalued budget is constrained
from its first measurement onward, not unconstrained. Where that gate sits in
the release process is `operations.md`.

## The failure path is a budget

Budget 6 exists because a wrong invocation is the invocation a calling agent
makes most often while it is finding its way, and it must cost what a `64`
costs (`BR-PERF-004`). The cost is real work: a `66` with a suggestion computes
an edit distance against every existing name of the relevant population
(`FR-ERR-019`), over the eight populations `FR-ERR-021` names, and over `WL-001`
that population is large. `FR-ERR-020` omits the suggestion rather than offering
a poor one, which bounds the output but not the computation.

The measure is `FR-ERR-039`'s, because it decides which candidates are offered
and not only their order. How it is computed is
[`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms):
hand-rolled, no dependency, a three-row window — chosen partly *because* this is
a budgeted path and the implementation must be ours to measure. Its surface is
`interfaces.md`.

## Peak memory, and the two embeddings

Budget 9 is the budget the model's shape presses on hardest. `FR-CTX-006`
embeds the referenced table and `FR-CTX-010` embeds the referencing table, and
`BR-CTX-001` and `FR-CTX-010`'s accepted cost each record what that does to the
volume of the document; neither figure is restated here.

Whether the model holds those embeddings as owned copies or reproduces them at
serialisation time is the architectural question the corpus leaves open, and it
is settled in [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md)
— materialise both — whose rationale, alternatives and consequences are **not**
restated here, per rule R3 of [`docs/adr/README.md`](../adr/README.md). Two
consequences bear on this document and are recorded in that record: the
provisional peak-memory figure is the one most likely to be superseded upward by
the first real measurement, which `NFR-PERF-019` permits without its ever having
been a limit; and streaming remains available everywhere the dump is not,
because `FR-SCH-016` and `FR-CTX-023` are what require the whole model to be in
hand before the first byte of a dump can be trusted.

## Reliability properties

| Property | How the built system holds it | Forced by |
|---|---|---|
| No invocation blocks without bound | A deadline on every blocking phase, composed with the overall budget measured from process start, exiting with the code of the phase in progress. The three mechanisms are settled in [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) and described in `architecture.md` | `FR-SEC-022`, `FR-CONF-005`, `FR-GLOB-011`, `FR-GLOB-012`, `FR-GLOB-013` |
| A failed render leaves at most one incomplete result on stdout | The render deadline exits `65`, and one invocation renders once, so there is at most one result to leave incomplete | `FR-RND-033`, `FR-RND-034`, `FR-RND-002` |
| A cache file is never half-written, and a killed process leaves nothing locked | Each object is written through a temporary file in the same directory and renamed over the target; no lock is taken, so two writers yield one whole result or the other | `FR-CACHE-030`, `FR-CACHE-031` |
| A cache failure is never an event in the contract | An unreadable file or an unknown format version is treated as a miss and reported neither as error nor as warning; an unwritable cache still answers from what was read, exits `0`, and changes no byte of stdout | `FR-CACHE-033`, `FR-CACHE-036` |
| An interrupted load loses nothing already stored | An unreachable server during `tpl cache load` exits `69` and leaves stored objects unchanged | `FR-CACHE-032` |
| Each distinct condition has its own exit code, and the code is sufficient on its own to choose the next step | The classification is stated with the condition: a `69` is repeatable precisely because the operation is read-only | `FR-ERR-001`, `FR-ERR-002` |
| Referential integrity is promised where it can be, and a snapshot is never promised | A server-read document contains every object referenced from another object in it; it does not promise to be point-in-time, and a cache-served document promises neither | `FR-CTX-023`, `FR-CTX-024`, `FR-CDOC-015` |

The stance behind these rows — fail loudly rather than produce plausible output
quietly — is stated once, in [overview.md](overview.md#the-stance-the-artefact-is-built-with).

## Cross-series equivalence and its exceptions

`FR-SRV-026` is the strongest testable form of "supported": for a database
created from the same accepted DDL on each series of `FR-SRV-015`, the dump
document is byte-identical across the four. It is a quality attribute of the
built system rather than a property of any one component, because every stage
from the reader to the emitter can break it.

| Exception | Requirement |
|---|---|
| Fields marked `null` where the series does not have them | `FR-SRV-004` |
| The `server` object, which reports the server that answered | `FR-SRV-028` |
| Values passed through verbatim, where the raw value differs between two supported servers | `FR-SRV-039` |

Three bounds keep the exceptions from swallowing the promise:

- The equivalence is **bounded to the window** of `FR-SRV-015` and does not
  extend to a server above it (`FR-SRV-033`).
- A passed-through field is excepted **only** if it carries a row in the
  register of `FR-SRV-027` and `FR-SRV-036`. A field that differs without a row
  there is a failure of `FR-SRV-026`, not an instance of its exception.
- The treatment applied to any difference is one of exactly four (`BR-SRV-006`,
  with `FR-SRV-024`, `FR-SRV-004`, `FR-SRV-025`, `FR-SRV-039`); which treatment
  each field receives is `data-model.md`.

Verification is an integration test executed against every series
(`FR-SRV-029`), which cannot be written until `tpl` can emit a document and is
therefore `verification.md`'s. The consequence for measurement is
`NFR-PERF-012` as amended: the three server-bound budgets carry a series as well
as a target, and results from two series are not compared.

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every measured figure and every recorded baseline | `BENCHMARKS.md` |
| The harness, the fixture, and the tests that verify a property of form | `verification.md` |
| The four targets, the validation pipeline, and where the regression gate sits in a release | `operations.md` |
| Components, the invocation pipeline, lazy initialisation, and the deadline machinery | `architecture.md` |
| Orderings as implemented, the error type, and the emitter | `interfaces.md` |
| The model's shape, the per-kind field lists, and the cache on disk | `data-model.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
