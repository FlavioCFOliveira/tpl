---
title: Quality Attributes
status: draft
last-reviewed: 2026-09-24
related: [README.md, traceability.md, open-decisions.md, overview.md]
---

# Quality Attributes

## What this document is

The requirements of form the design is held to from the first commit, the nine
measurement points and what each is measured over, the protocol a reading is
taken under, what has been measured so far, and the reliability and equivalence
properties a caller may rely on. Every statement cites the requirement that
forces it; no requirement text is reproduced here.

**No figure fails a change.** `BR-PERF-008` gives no figure named in the
functional corpus, and no figure recorded against it in `BENCHMARKS.md`, the
power to fail, block, reject or gate a change, a release or a piece of work. The
obligation toward speed and toward sparing use of the machine survives as
**design and architecture**: it decides how `tpl` is built, and it decides
nothing about whether a change is accepted. What still fails is the requirements
of form, and the `WL-002` scalar with them — deterministic counts, absences and
a size, none of them a figure — and each section below says so in its own words.
What does not fail is a **timing**, a **memory reading**, or any comparison
between two of them.

**No figure appears in this document.** A measurement point's **adopted** figure
lives in `NFR-PERF-014` and a **measured** one in `BENCHMARKS.md`, and
`NFR-PERF-020` is the step that moves it from the first place to the second
(`NFR-PERF-019`, `BR-PERF-006`). A third copy would be the copy nobody updates,
which is the concern `DIV-035` carries.

*Restated on 2026-09-22, against the thirty-sixth edition of
`specification/performance-requirements.md`.* That edition withdrew the whole
enforcement regime — the one figure that could fail a change on its own merits,
the no-regression rule, and the two requirements that existed only to divide the
set by them — and retired four identifiers with it, none of which is cited
anywhere in this folder any more. The vocabulary went with the regime: a
*budget* is a **measurement point**, a *provisional* figure is an **adopted**
one, a *ratified* figure is a **recorded** one, and *baseline* — the regime's
word for the figure a later measurement was failed against — has no referent.
Every section below is restated in the vocabulary in force.
**The requirements of form are untouched by the withdrawal**, and they are set
out first so that no reader takes them for the gates that went.

The tests and the container harness are `verification.md`; the build, the target
matrix, the release gates and the measurement harness are `operations.md`.

## The six requirements of form

**These are correctness invariants, they are the part of this subject that can
fail something, and they are what survived the withdrawal.**
Each states a deterministic count or a deterministic absence, observable from
outside the process and independent of how fast the host is, so none of them is
a figure and none is reached by `BR-PERF-008`. A cache hit that opened a
connection is a **functional defect**, failing a test for the reason a wrong
exit code fails one — not a slow run. `BR-PERF-001` records that a requirement
of form is now the only instrument the performance corpus has that can catch a
defect at all, which is why these six, and not the measurement points, are what
constrains the design. Each is observable, permanent, and checkable with no
stopwatch.

**The integration suite carries all six**, under no feature and no flag, so each
runs with the rest of the suite on `cargo test`; the assertions that need a
server are gated on the fixture standing. Which test carries which, by which
instrument, on which targets and under what gate is
[verification.md](verification.md#the-register-of-mandated-tests)'s.

| Requirement | The defect it forbids | The design decision it forces | Described in |
|---|---|---|---|
| `NFR-PERF-001` | A query per object on a full read | The catalogue reader is organised per object **kind**: a fixed set of queries, each returning every object of its kind, so the count over `WL-001` equals the count over `WL-003` | `architecture.md`, `interfaces.md` |
| `NFR-PERF-002` | Reading the whole catalogue to answer for one object | The reader carries a named-object path distinct from the full-read path, with the name restricting the query the server receives | `architecture.md`, `interfaces.md` |
| `NFR-PERF-003` | A connection opened before the cache is consulted | The cache is a read-through layer **in front of** the reader: a hit is served from disk with no connection and no query (`FR-CACHE-006`, `FR-CACHE-007`) | `architecture.md` |
| `NFR-PERF-004` | A pool, or a second connection for any purpose | One connection at most, opened late and closed when the read ends — and, with its runtime, before any render starts (`FR-RND-040`); it is also what rejected a pre-flight TCP connect for phase attribution in [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), and it is verifiable from the server side (`FR-SRV-014`) | `architecture.md` |
| `NFR-PERF-005` | Startup work a command does not need | The four commands of `FR-PROJ-025` are classified **before** any filesystem access: no `stat` of an ancestor, no open of `.tpl/.cfg`, no socket — and, per [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), no timer thread | `architecture.md` |
| `NFR-PERF-006` | A connection opened by dispatch rather than by need | Connection establishment is lazy: the reader opens it when it is about to read, so every command that needs no catalogue opens none | `architecture.md` |

Two obligations attach to the six and belong elsewhere. Each is verified by an
observation made **outside** the process and never by reading the source
(`NFR-PERF-007`), which is `verification.md`; and the catalogue-query count is
observable on the diagnostic stream (`NFR-PERF-008`, `FR-GLOB-017`), which makes
one diagnostic line structurally load-bearing although stderr is not contract —
that is `operations.md`, settled in
[`OD-17`](open-decisions.md#od-17--observability). Since the sixty-second
edition `NFR-PERF-008` is verified by counting those lines at `-v` against the
server's statement record for the same invocation, which is `verification.md`.

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
| `WL-001` | The large workload, over which every measurement point that needs volume is taken | `scripts/mariadb/seed-bench.sql`, as schema `freight_wl001` |
| `WL-002` | The verification scalar: the size of the compact dump of `WL-001`, which detects a change in the fixture or in the document shape with no server. **Valued**: it was measured on 2026-09-22 and recorded in `BENCHMARKS.md` under `NFR-PERF-020`, byte-identical across five takes, and `BR-PERF-006` keeps the value there and not here | The same fixture, read through the document shape in force on the day of the reading |
| `WL-003` | The small workload — the common path, because `FR-RND-002` gives one render per invocation and `BR-RND-002` moves iteration to the caller | The same file, as schema `freight_wl003` |

**`WL-002` is a deterministic size and not a timing, and it did not lose its
force with the gates.** The same fixture read through the same document shape
produces the same byte count on every target and every machine, so a departure
beyond the ±2% that requirement states is a statement about the fixture or about
the document and never about how fast anything ran. It belongs with the
requirements of form: a mismatch is a **functional defect** to be explained,
which is what `BR-PERF-008` leaves standing. It may never be read as a
performance result.

`BR-PERF-002` keeps `seed.sql` and `seed-bench.sql` apart deliberately: one
fixture serving both would hide an N+1 or make the correctness suite pay for the
benchmark volume. **`seed-bench.sql` was written on 2026-09-21**, in the sprint
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001) named, and it is
DDL alone, loaded on demand by `seed-bench.sh` rather than baked into the image;
that script verifies every count each workload states. The counts, the schema
names and the load procedure are `scripts/mariadb/README.md`'s and are not
restated here. The fixture itself is `verification.md`.

## The measurement set

`NFR-PERF-014` fixes the set as exactly these nine **measurement points** and no
others, and each row of it fixes the invocation, the workload it is measured
over, whether a server answers, and what the cache does while the reading is
taken. A figure a point carries is a **reference figure**: informative, a target
to build toward, and never a limit (`BR-PERF-008`).

The sixth column says whether `NFR-PERF-014` carries a figure for the point and
where that figure came from — **adopted** for one taken from a root document
rather than measured (`NFR-PERF-019`), **none** for a point named with no figure
supplied and none invented (`BR-PERF-006`). The seventh says where a reading has
actually been taken; a measured figure is `BENCHMARKS.md`'s and appears nowhere
here.

| # | Measurement point | Workload | Server | Cache | Figure in `NFR-PERF-014` | Measured on |
|---|---|---|---|---|---|---|
| 1 | `tpl --version` | none | no | not reached | Adopted | `aarch64-apple-darwin` |
| 2 | `tpl --help` | none | no | not reached | Adopted | `aarch64-apple-darwin` |
| 3 | Startup to the first byte of useful work, by `tpl template list` in a project holding no database entry | none | no | not reached | Adopted | `aarch64-apple-darwin` |
| 4 | `tpl schema dump` | `WL-001` | yes | bypassed, `--direct --no-cache` | Adopted | `aarch64-apple-darwin`, series `12.3` |
| 5 | A cache-served read of one object | `WL-003` | no | served from | None | `aarch64-apple-darwin`; **dispersion above the line** |
| 6 | The failure path: a `64`, and a `66` with nearest match | `WL-001` | no | served from | None | `aarch64-apple-darwin`; the `64` half's **dispersion above the line** |
| 7 | `tpl help --format json` | none | no | not reached | None | `aarch64-apple-darwin` |
| 8 | The canonical loop of 200 invocations | `WL-001` | yes | empty when each run begins | None | `aarch64-apple-darwin`, series `12.3` |
| 9 | Peak resident memory | `WL-001` | yes | bypassed, `--direct --no-cache` | Adopted | `aarch64-apple-darwin`, series `12.3` |

**Point 6 is one point measured by two invocations**, and its figure is the
slower of the two, with the other recorded beside it (`NFR-PERF-014`).

**All nine were measured once, on one target, on 2026-09-22.** The campaign ran
at the full protocol of `NFR-PERF-009` — the median of 200 runs after 20 warmups
— on `aarch64-apple-darwin`, against series `12.3` for the three points whose
reading reached a server, and it measured the `WL-002` scalar with them, against
the same server. Every figure,
every dispersion, the noise floor of the instrument on that host and the two
conditions the campaign departed from are recorded in `BENCHMARKS.md` under
`NFR-PERF-020`, and none of them is restated here.

**Three of the four targets carry no figure at all.**
`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and
`x86_64-apple-darwin` were not measured, and nothing in the table above may be
read as settled on them: `NFR-PERF-012` forbids carrying a figure from one
target to another, and `BR-PERF-003` makes a point with a figure on one target
and none on the other three **complete rather than short**.

**Two readings sit above the dispersion line of `NFR-PERF-011`** — point 5, and
the `64` half of point 6 — because the host was neither idle nor on mains power,
which are two departures from `NFR-PERF-010` that `BENCHMARKS.md` records. That
requirement settles what the five per cent is about: a dispersion above it is a
statement about **the host the reading was taken on** and never about `tpl`, so
neither reading is discarded, neither is a finding about the program, and both
are recorded with the fact stated beside them. What follows is only this: neither
stands as its point's reference figure until the reading is retaken on a quiet
host. **Retaking them is registered as `#228` and is not resolved here.**

**Five adopted figures are still carried in `NFR-PERF-014` beside a recorded
reading of the same point** — points 1, 2, 3, 4 and 9. `NFR-PERF-020` obliges
that requirement to be amended to remove each, which the recorded figure
replaces. The amendment is the functional owner's, it is scheduled, and this
folder states the state rather than working around it: until it is made, a
reader of `NFR-PERF-014` sees a figure nobody measured next to one somebody did.

**Which of the nine need the fixture.** Points 4, 6, 8 and 9 are measured over
`WL-001` and point 5 over `WL-003`; points 1, 2, 3 and 7 need neither fixture
nor server. Of the five needing the fixture, three need a server as well — 4, 8
and 9 — which the `Server` column names and which `BR-PERF-007` counts. The
requirement that once obliged one privileged point to run with no server is
withdrawn with the class it governed, and which points reach a server is read
from the `Server` and `Cache` columns of `NFR-PERF-014`.

**Discharged — the contradiction this section recorded against `BR-PERF-007`.**
That rule stated that `seed-bench.sql` was still absent while the file existed,
and it was restated over the fixture that now exists in commit `455e48d`, on
2026-09-21. Nothing is owed to the functional owner for it, and what the rule
counts — five points needing the fixture, four needing neither it nor a server —
is unchanged and is cited above.

**What point 5 keeps, now that no figure is privileged.** It is still the
invocation a caller repeats once per object, since `FR-RND-002` gives one render
per invocation and `BR-RND-002` moves iteration to the caller, and it is still
served from the cache with no server standing. What it no longer is, is the one
figure that could stop anybody: the requirement that made it so is withdrawn,
and `BR-PERF-008` denies that power to every figure alike.

## The measurement protocol

The protocol binds **how a reading is taken and how it is written down**, and
nothing else: it decides no outcome for the change that produced the reading. It
is stated in `NFR-PERF-009` through `NFR-PERF-012` and its values are not
restated here. What it obliges of anything that records a result:

| Obligation | Requirement |
|---|---|
| A recorded measurement names the **target** it was taken on, drawn from the four of `NFR-PERF-018`; measurements on different targets are never compared | `NFR-PERF-012` |
| A measurement that reaches a server additionally names the **series** it was taken against, because the four series of `FR-SRV-015` are four different programs answering the catalogue queries | `NFR-PERF-012`, as amended |
| Every recorded figure carries the relative standard deviation of the samples behind it. One above five per cent is a statement about the **host**, never about `tpl`: it is recorded with that fact stated, never discarded, and it merely does not stand as its point's reference figure until the reading is retaken on a quiet host | `NFR-PERF-011` |
| A measured figure is recorded in `BENCHMARKS.md` against its point, its target and, where a server answered, its series, stating the conditions it was taken under, its dispersion, and the invocation as a reader would retype it | `NFR-PERF-020` |

**Recording is per point and per target**, because the four targets are measured
at different times and a point measured on one is not measured on the others. The
step that records a measured value in `BENCHMARKS.md` is the same step that
removes the adopted figure from `NFR-PERF-014`, so that one figure has one home
(`BR-PERF-006`). `NFR-PERF-020` is a **recording rule and not a gate**: nothing
follows from a recorded figure by rule, and a reader who compares two of them —
against the same point on the same target, which is the only comparison
`NFR-PERF-012` admits — draws whatever the comparison is worth and no obligation
from it.

The instrument that takes a reading, and the reason it can enforce nothing, are
[operations.md](operations.md#the-measurement-harness)'s.

## The failure path is a measurement point

Point 6 exists because a wrong invocation is the invocation a calling agent
makes most often while it is finding its way. `BR-PERF-004` states the **design
expectation** that it should cost what `tpl --version` costs, and the point
exists so that the expectation can be checked against a reading instead of
asserted; it obliges no figure, per `BR-PERF-008`. The cost is real work: a `66`
with a suggestion computes an edit distance against every existing name of the
relevant population (`FR-ERR-019`), over the eight populations `FR-ERR-021`
names, and over `WL-001` that population is large. `FR-ERR-020` omits the
suggestion rather than offering a poor one, which bounds the output but not the
computation. The point's two halves — a refusal that computes nothing and a
refusal that computes against 200 names — are what a reader compares, and
`NFR-PERF-014` makes the slower of them the figure the point carries.

The measure is `FR-ERR-039`'s, because it decides which candidates are offered
and not only their order. How it is computed is
[`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms):
hand-rolled, no dependency, a three-row window — chosen partly *because* this is
a measured path and the implementation must be ours to measure. Its surface is
`interfaces.md`.

## Peak memory, and the two embeddings

Point 9 is the point the model's shape presses on hardest. `FR-CTX-006`
embeds the referenced table and `FR-CTX-010` embeds the referencing table, and
`BR-CTX-001` and `FR-CTX-010`'s accepted cost each record what that does to the
volume of the document; neither figure is restated here.

Whether the model holds those embeddings as owned copies or reproduces them at
serialisation time is the architectural question the corpus leaves open, and it
is settled in [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md)
— materialise both — whose rationale, alternatives and consequences are **not**
restated here, per rule R3 of [`docs/adr/README.md`](../adr/README.md). Two
consequences bear on this document and are recorded in that record: the adopted
peak-memory figure is the one that record expected to move upward first, marked
adopted under `NFR-PERF-019` and a limit at no point; and streaming remains
available everywhere the dump is not, because `FR-SCH-016` and `FR-CTX-023` are
what require the whole model to be in hand before the first byte of a dump can
be trusted.

**The first reading of point 9 did not move the figure upward.** It was taken on
one target on 2026-09-22 and came in well under the adopted figure, which
`BENCHMARKS.md` states as an observation and not as a verdict; the expectation
`ADR-009` recorded is therefore unconfirmed on that target and untested on the
other three. Nothing follows from the gap by rule (`BR-PERF-008`), and the record
is unaffected: what it decided was the representation, not the figure.

**Two readings [`ADR-011`](../adr/adr-011-render-memory-accounting.md) calls
for are not recorded in `BENCHMARKS.md`** as of 2026-09-23: the render wall time
of the four worked examples before and after the counting allocator, and the
realised overshoot of the memory limit. `SECURITY-AUDIT.md`, *Remediation*, row
H-1 carries an overshoot reading taken during the audit; it is not a record
under `NFR-PERF-020`. Neither gates anything (`BR-PERF-008`).

## Reliability properties

| Property | How the built system holds it | Forced by |
|---|---|---|
| No invocation blocks without bound | A deadline on every blocking phase, composed with the overall budget measured from process start, exiting with the code of the phase in progress; the `password_command` phase runs until the child has exited and its output has closed, and a descendant holding the pipe cannot extend it. The three mechanisms are settled in [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) and described in `architecture.md` | `FR-SEC-022`, `FR-CONF-005`, `FR-CONF-028`, `FR-GLOB-011`, `FR-GLOB-012`, `FR-GLOB-013` |
| No render runs unbounded in work, output or memory | Render fuel, the render output limit and the render memory limit beside the deadline; the first crossed ends the render with `65`. The memory limit is observed every 10 ms and can be overshot within one interval, and an allocation the OS refuses aborts without a `cause` ([architecture.md](architecture.md#the-render-bounds), [`ADR-011`](../adr/adr-011-render-memory-accounting.md)) | `FR-SEC-025`, `FR-RND-036` … `FR-RND-039` |
| A failed render leaves at most one incomplete result on stdout | The render writes nothing to stdout until it returns; a bound that ends it from the watchdog exits without flushing, and one invocation renders once | `FR-RND-033`, `FR-RND-034`, `FR-RND-037`, `FR-RND-039`, `FR-RND-002` |
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
`NFR-PERF-012` as amended: the three server-bound measurement points carry a
series as well as a target, and results from two series are not compared.

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every measured figure, every dispersion, and the conditions a reading was taken under | `BENCHMARKS.md` |
| The container harness, the fixture, and the tests that verify a requirement of form | `verification.md` |
| The four targets, the validation pipeline, the release gates, and the measurement harness | `operations.md` |
| Components, the invocation pipeline, lazy initialisation, and the deadline machinery | `architecture.md` |
| Orderings as implemented, the error type, and the emitter | `interfaces.md` |
| The model's shape, the per-kind field lists, and the cache on disk | `data-model.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
