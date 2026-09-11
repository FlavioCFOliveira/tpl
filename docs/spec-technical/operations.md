---
title: Operations
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md, open-decisions.md, overview.md, architecture.md, technology-stack.md, data-model.md, quality-attributes.md]
---

# Operations

## What this document is

How the artefact is produced, validated, released and observed: the four
targets and what their linkage obliges, the pipeline every change passes, the
gates a release passes, where each of the four version numbers is bumped, what
a new project receives, and what the container fixture supplies today.

**No measured figure and no budget appears here.** Every figure is
`BENCHMARKS.md`'s, which `BR-PERF-006` requires; every budget, the standing of
its figure and the protocol that ratifies one are
[quality-attributes.md](quality-attributes.md#the-nine-budgets). Versions, crate
choices and the dependency budget are
[technology-stack.md](technology-stack.md); the tests themselves, and the
harness that drives the containers, are `verification.md`.

**One section is provisional by construction.**
[The fixture as an operational asset](#the-fixture-as-an-operational-asset)
describes `scripts/mariadb/` as it stands on 2026-09-11, with one requirement
on it unsatisfied. It is the one part of this document that changes when the
fixture half of [`OD-22`](open-decisions.md#od-22--the-test-harness-and-the-fixture-certificate)
lands.

## The four targets, and what the linkage obliges

`NFR-PERF-018` fixes the set at exactly four and makes none of them second
class. `DIV-041` records that the `gnu` triples are not targets and that the
libc question the root coordination document deferred is answered for Linux.

| System | Architecture | Target triple | C library | Linkage | Fixed by |
|---|---|---|---|---|---|
| Linux | amd64 | `x86_64-unknown-linux-musl` | `musl` | Static | `NFR-PERF-018`, `DIV-041` |
| Linux | arm64 | `aarch64-unknown-linux-musl` | `musl` | Static | `NFR-PERF-018`, `DIV-041` |
| macOS | amd64 | `x86_64-apple-darwin` | The platform's own | **Not fixed** | `NFR-PERF-018` fixes the triple and no more |
| macOS | arm64 | `aarch64-apple-darwin` | The platform's own | **Not fixed** | `NFR-PERF-018` fixes the triple and no more |

**The two Darwin rows carry no linkage decision because none was ever
needed.** `DIV-041` answers the libc question as the *Linux* question; neither
the corpus nor the register states a linkage for a Darwin target, and this
document does not invent one. Whether macOS admits a statically linked C
library at all is **not confirmed in the official documentation** consulted
here, and nothing below rests on it.

**The static linkage is not packaging, and `DIV-041` forbids presenting it as
such.** `FR-CONF-005`'s note records the one behavioural consequence: the
static `getaddrinfo` of the Linux targets does not load the platform's
name-service modules, so a name resolvable only through such a module does not
resolve. The deadline and the exit status are unchanged — the phase fails and
`FR-ERR-027` routes it to `69` — but the `cause` line owes the caller the
distinction between a name that did not resolve and a host that refused a
connection, per `FR-ERR-034`. This folder cites that observation and does not
re-derive it; the diagnostic that carries the distinction is
[interfaces.md](interfaces.md#the-diagnostic-renderer)'s.

**What the static linkage is expected to buy has not been observed.**
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) records that no `musl`
artefact has been run outside a container, and marks the expectation
unverified. It is repeated here as a limit on what may be claimed of the Linux
artefacts, not as a second statement of the fact.

**The form of the release artefact is not fixed** — bare binary, archive,
checksums, signature. [`ADR-008`](../adr/adr-008-packaging-and-build-path.md)
declines to invent a constraint the corpus does not state, and this document
adds none.

## The mandatory validation pipeline

`CLAUDE.md` (*Desenvolvimento*) fixes five commands and their order, and makes
no work complete until all five pass. They are reproduced here in order because
the order is the contract; their authority is that file's.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --all-features
cargo audit
```

| # | What it decides | Bearing on this folder |
|---|---|---|
| 1 | `rustfmt` with its default configuration is the formatting authority | No formatting question is settled by this folder |
| 2 | `clippy` with `-D warnings` is the idiom authority. A justified suppression is a local `#[allow(...)]` with its reason, never a crate-level one | `#![forbid(unsafe_code)]` is the one prohibition this escape does not reach ([technology-stack.md](technology-stack.md#the-consequence-of-forbidding-unsafe)) |
| 3 | The only command of the five that compiles under the release profile | The profile is [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md)'s |
| 4 | The test suite, with every feature active | See the two observations below |
| 5 | The dependency graph against the advisory database | `cargo audit` is described by its publisher as auditing "`Cargo.lock` for crates with security vulnerabilities" (crates.io crate index and rustsec.org, `cargo-audit` 0.22.2, consulted 2026-09-11), so it needs a resolved lockfile |

Benchmarks and a comparison against `BENCHMARKS.md` are added to the five when
a change touches a hot path (`CLAUDE.md`, *Desenvolvimento*, *Disciplina de
medição*). Which measurement is valid, and when a figure becomes a limit, are
[quality-attributes.md](quality-attributes.md#the-measurement-protocol)'s.

**Observation — the pipeline names no target, so it exercises the host's.**
`NFR-PERF-018` makes none of the four second class, so passing on the
development host is not passing. Covering the other three is the second of the
four obligations [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) records
as carried by hand while no pipeline exists.

**Observation — `--all-features` decides what a cargo feature costs here.**
The flag is documented as "Activate all available features of all selected
packages" (The Cargo Book, *cargo-test*, consulted 2026-09-11), and it appears
in two of the five commands. Two consequences follow, and the second is a
question this document is handed and does not answer.

- The two test seams are **not** features and are unaffected: both are
  `#[cfg(test)]` constructs inside the library, settled in
  [`OD-21`](open-decisions.md#od-21--two-test-seams-that-must-not-be-on-the-published-surface),
  and `FR-ERR-031` rejects a cargo feature by name.
- [`ADR-006`](../adr/adr-006-package-layout.md) hands this document the one
  build question that would introduce a feature: heap profiling requires the
  profiling allocator to be installed as the program's own global allocator,
  and that record notes the allocator is slower than the normal one. A feature
  carrying it would be activated by commands 2 and 4, so the whole test suite
  would run under it. **No decision is taken here**: an arrangement nobody has
  run is not described in this folder, and the choice — a feature the pipeline
  would enable, a separate profiling build, or no in-tree heap profiling at all
  — belongs to the register. What is recorded is the constraint that decides
  it.

**The pipeline cannot be run at this commit.** No manifest exists, which
[technology-stack.md](technology-stack.md#how-to-read-an-entry) and
[data-model.md](data-model.md#the-four-version-numbers) both record, so there
is no lockfile for command 5 and nothing for the other four to compile.

## The release profile

Five settings, and the panic path that reports `70` under them, are
[`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md)'s and are not
restated. Three operational facts belong here.

- **All five are written explicitly** into `[profile.release]`. The profile's
  documented defaults do not coincide with them throughout (The Cargo Book,
  *Profiles*, `release`, consulted 2026-09-11), and a setting left implicit
  would move with a future default.
- **There is one manifest to write them in.**
  [`ADR-006`](../adr/adr-006-package-layout.md) settles one package carrying a
  library and a binary, so the profile and the toolchain floor have no second
  copy to drift against, and the four targets share both.
- **Changing the profile invalidates the comparability of every recorded
  baseline**, on the same ground as changing the build path
  ([`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md),
  [`ADR-008`](../adr/adr-008-packaging-and-build-path.md), with `NFR-PERF-012`).
  Whoever changes it re-measures all four targets or states that the figures
  before and after are not comparable.

`DIV-045` is discharged: the aborting profile stands as written and
`FR-ERR-030` as amended obliges the message and the code without obliging the
panic path to be catchable.

## MSRV, the development toolchain, and the cross-build path

**The MSRV is a rule and the figure it yields**, both
[`ADR-007`](../adr/adr-007-msrv.md)'s, registered as
[`OD-02`](open-decisions.md#od-02--the-msrv) and cited rather than restated. Two
consequences are operational: the floor is set by a dependency, so a dependency
bump is also an MSRV question; and that record carries an obligation to re-run
its rule over the shipped graph the first time the graph resolves, because only
the direct dependencies have been read.

**The development toolchain is not the pin.** It is above the floor, and it is
the toolchain the driver selection was measured under, recorded in
`BENCHMARKS.md` ([`ADR-007`](../adr/adr-007-msrv.md)).

**The cross-build path is
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md)'s**: the two `musl`
targets are built through a zig-based linker driver at the versions that record
names, and the two Darwin targets are built natively. The versions are the
record's and are not repeated, because they are part of the evidence — they are
the path that produced the artefacts `BENCHMARKS.md` measured, and a different
path makes a later figure incomparable with the recorded one.

| Tool | Role | Source |
|---|---|---|
| The Rust toolchain | Compiles the four targets; above the floor | [`ADR-007`](../adr/adr-007-msrv.md) |
| The zig-based linker driver | The `musl` cross-link from a Darwin host, which the Apple linker cannot perform | [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) |
| `cargo fmt`, `cargo clippy`, `cargo audit` | The validation pipeline above | `CLAUDE.md`, *Desenvolvimento* |
| `hyperfine`, `criterion`, `dhat-rs`, `samply` / `cargo flamegraph`, `cargo bloat` | The measurement toolchain, one tool per question | `CLAUDE.md`, *Disciplina de medição* |
| Docker | Runs the four fixture containers | `scripts/mariadb/README.md` |

The measurement toolchain is listed, not specified: what each tool is used to
decide is `CLAUDE.md`'s table, the protocol that makes a result valid is
[quality-attributes.md](quality-attributes.md#the-measurement-protocol), and
every result is `BENCHMARKS.md`'s.

## The release gates

A gate is a check whose failure stops the release. Four are standing; the
fifth fires only when a pin moves.

| Gate | What is checked | Trigger | Forced by |
|---|---|---|---|
| The validation pipeline | All five commands pass, in order | Every change, not only a release | `CLAUDE.md`, *Desenvolvimento* |
| Every target | The pipeline passes on all four of `NFR-PERF-018`; a failure on one is a failure | Every release | `NFR-PERF-018`; carried by hand per [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) |
| No regression | No measurement is worse than the recorded baseline for that budget on that target | Every release, and every change to a hot path | `NFR-PERF-017`, `NFR-PERF-012` |
| The supported-series table | The table of `FR-SRV-015` is re-verified against its source, and its verification date moved | **Every release**, without exception | `FR-SRV-019`, `BR-SRV-004` |
| The engine pin | Every name of `FR-ENV-018` still exists and still behaves as before | Only when the pin of [`ADR-001`](../adr/adr-001-template-engine-pin.md) moves | `FR-ENV-003` |

**The no-regression gate binds nothing today.** No budget is ratified.
[quality-attributes.md](quality-attributes.md#the-nine-budgets) records why, and
what a budget carrying no baseline does and does not fail.

**The series gate is the one that decays on a calendar.** `BR-SRV-004` makes a
wrong table stale rather than the criterion wrong, so the correction is always
to re-derive the table from the criterion and the source — never to amend the
criterion so that the old table stays true. A re-derivation that changes the
set changes what `FR-SRV-029` must be run against, and therefore what the
fixture must contain.

**Every gate above is carried by a person.** No pipeline enforces one, which is
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md)'s decision and its
consequence together.

## The four version numbers: where a bump is enacted

The four numbers, their starting values and what moves each are
[data-model.md](data-model.md#the-four-version-numbers)'s, settled in
[`OD-03`](open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog).
This section adds only where the bump is written and what a gate checks.

| Number | Where the bump is enacted | What a release gate checks |
|---|---|---|
| Binary version | The `version` field of the one manifest, and nowhere else | That it moved, and that the changelog carries the matching entry |
| `schema_version` | One constant, read by `output/` when it writes the envelope | That the release contains no change on a breaking row of `FR-OUT-014` without a bump, and no bump without one |
| `cache_format` | One constant, read by `cache/` | The same test against the on-disk arrangement, independently of `schema_version` (`FR-CDOC-005`) |
| Changelog | An entry in `CHANGELOG.md` | That every change `FR-ENV-029` requires to be recorded there is |

**The binary version has one source and the bump must not acquire a second.**
`tpl version` and the `tpl.version` context variable both read
`CARGO_PKG_VERSION`, documented as "The full version of your package" (The
Cargo Book, *Environment variables Cargo sets for crates*, consulted
2026-09-11), so editing the manifest is the whole of the bump and `FR-HELP-005`
and `FR-CTX-027` cannot disagree. The same property is prescribed for the other
two: each is one constant, never a literal at a second site.

**Nothing here is observable yet.** Neither the manifest nor the changelog
exists at this commit, which
[data-model.md](data-model.md#the-four-version-numbers) already records.

## What a breaking change is

The corpus fixes the answer for two of the three published surfaces and states
it in each surface's own file. Each is cited, none is reproduced.

| Surface | What is breaking | Where the rule lives |
|---|---|---|
| The JSON document | Three of five changes; adding a field or an enumerated value is not among them | `FR-OUT-014`, with `FR-CDOC-010` on why an enumerated field and not a boolean |
| The template surface, group 1 | Renaming or removing a registered name. Adding one is not breaking, and a rename or a removal must be recorded in the changelog | `FR-ENV-002`, `FR-ENV-029` |
| The template surface, behaviour | A change to any cell of the behaviour tables, which are simultaneously specification and test vector | `BR-ENV-007` |

A breaking change moves the number that versions the surface it broke: the
document's is `schema_version` (`FR-OUT-011`), and the template surface's is
the binary's own, under the rule
[`OD-03`](open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog)
fixes and [data-model.md](data-model.md#the-four-version-numbers) carries.
`cache_format` versions an arrangement no caller reads, and an unknown value is
a miss rather than an error (`FR-CDOC-004`), so moving it breaks nothing.

**Recorded gap — the command line has no compatibility rule.** `DIV-032` names
two contracts, the JSON document and the command line, and the corpus supplies a
rule for the first and none for the second, although the command tree
(`FR-CLI-002`), the short-flag set (`FR-GLOB-024`) and the exit-code table
(`FR-ERR-001`) are all contract. What makes a command-line change breaking is
therefore not stated anywhere, and this document does not supply it: a
compatibility rule over a functional contract is the functional owner's. The
gap is reported rather than filled.

## Observability

Settled in [`OD-17`](open-decisions.md#od-17--observability), whose rationale
and rejected options are not restated. What this document owns is the operating
picture.

| Property | What it means in operation | Forced by |
|---|---|---|
| Two streams, one contract | stdout alone is contract; stderr is neither deterministic nor contract, and cannot be, because the timings `FR-GLOB-017` requires differ on every run. The extent of the contract is [quality-attributes.md](quality-attributes.md#determinism)'s | `NFR-DET-001`, `FR-GLOB-017`, `DIV-039` |
| Four levels, selected by two flags | `-v` raises and saturates, `-q` lowers to errors only, and neither alters stdout | `FR-GLOB-014`, `FR-GLOB-015`, `FR-GLOB-016` |
| What each level reports | Phases and their durations, and one line per catalogue query, at `INFO`; cache hits and misses at `DEBUG`; internal detail permitted at `TRACE` | `FR-GLOB-017` |
| One line is structurally load-bearing | The per-query line carries a fixed leading token, which is what makes the query count observable from outside the process and `NFR-PERF-001` and `NFR-PERF-002` checkable at all | `NFR-PERF-008`, `NFR-PERF-007` |
| Six categories reach no stream, at any level | Denied a home rather than denied by a rule: the sink is a closed set of typed emission functions, and no subscriber is installed for a dependency's events to reach | `FR-GLOB-018`, `FR-SEC-005` |
| No colour, no terminal detection | Neither stream carries an ANSI byte, on any path | `NFR-DET-003`, `NFR-DET-004` |

**There is no logging facade to configure, and no sink to point anywhere.**
`tracing-subscriber` is not a dependency and `tracing` has no role of its own,
which [technology-stack.md](technology-stack.md#diagnostics) records; the
diagnostic writer is [interfaces.md](interfaces.md#the-diagnostic-renderer)'s.
An operator has exactly two controls, `-v` and `-q`, and one destination,
stderr.

**A test may depend on the token and not on the wording.** That is the property
`FR-GLOB-017` states and the only one; everything after the token is free to
change and no test reads it
([`OD-17`](open-decisions.md#od-17--observability)).

**Consequence prepared for the user and not made here.** `CLAUDE.md`'s stack
table names `tracing` and `tracing-subscriber` for logging. The register
prepared that correction rather than applying it, because that file is
coordination and is never edited unilaterally.

## What `tpl init` ships

`FR-PROJ-017` fixes exactly five artefacts and `FR-PROJ-013` the one write
outside `.tpl` — the destination directory and its missing parents. The five
are enumerated in [data-model.md](data-model.md#tpl-on-disk-and-its-four-writers)
and in the requirement; what belongs here is that **two of the five are shipped
content** and therefore travel with the binary.

| Artefact | Why it is content rather than structure | Standing obligation |
|---|---|---|
| `.tpl/templates/example.jinja` | A working example a caller reads before documentation | Four properties fixed by `FR-PROJ-021`, one of which reaches a server: it renders without error against any table of any supported series, which makes the fixture what checks it |
| `.tpl/templates/rust/_types.jinja` | The column-to-Rust-type mapping, delivered as a macro so the opinion belongs to the project and can be edited there | It replaces a filter the binary no longer provides (`FR-ENV-011`) |

The generated `.cfg` is structure, not content, with one exception that behaves
like content: the commented-out entry showing the exact shape a real entry
takes, which `FR-PROJ-018` requires and which the format-preserving write path
exists to keep alive past the first `tpl cfg set`
([data-model.md](data-model.md#tplcfg)).

**`FR-PROJ-021` is a standing obligation, not a one-off.** It is checked against
the fixture, so it is re-checked whenever the series set changes under
`FR-SRV-019` and whenever the example changes. The test is `verification.md`'s.

**Neither shipped template exists at this commit.** `CLAUDE.md` (*Estrutura do
Projecto*) names a root `templates/` directory for them; there is no such
directory, and no source tree beside it.

**Recorded reading — the runtime-loading invariant and the init payload.**
`CLAUDE.md` (*Invariantes de Implementação*) and `FR-TMPL-004` put templates on
disk, loaded and compiled at render time, never embedded at compile time; and
`tpl init` must nevertheless produce these two files from a single binary with
nothing named beside it. Both readings are recorded. This document takes the
invariant as governing the **render path**: its stated ground is that a template
change must not oblige a recompilation of `tpl`, and an artefact `tpl init`
copies into a project is thereafter edited there and loaded from disk like any
other. **The mechanism that carries the bytes is settled by nothing** — no
requirement and no record names one — and this document does not settle it
either; it belongs to the register. The wording of the invariant, if it is to
exclude the init payload in its own text, is the coordination document owner's.

## The fixture as an operational asset

`scripts/mariadb/` is the only database any validation may use: `CLAUDE.md`
(*Testes contra MariaDB*) admits no mock, no stub and no external instance, and
the fixture's own documentation adds that no container is left running after a
validation run. It is an operational asset because three obligations rest on
it — `FR-SRV-029`'s cross-series test, `FR-SRV-026`'s byte-identity across the
four series, and `FR-PROJ-021`'s example template.

**This section describes the fixture as it is on 2026-09-11.** The
arrangement — one image definition parameterised by series, one image, one
container and one published port per series of `FR-SRV-015`, and the schema and
seed the entrypoint runs — is recorded in `scripts/mariadb/README.md` and is
not repeated here.

**`FR-CONF-038`'s certificate obligation is not satisfied.** The requirement
obliges the fixture to be able to present, at each series of `FR-SRV-015`, a
server whose certificate names the host by which the project's tests reach it,
and to retain a server that offers no TLS. What is missing, named exactly:

| Missing | What the fixture has today |
|---|---|
| A certificate naming the reachable host, on the three TLS-capable series | The self-signed certificate MariaDB generates automatically, which `FR-CONF-038` observed to carry no `subjectAltName` |
| Any TLS at all on `10.11` | `have_ssl=DISABLED`; a MariaDB client of a later series reaches it over TCP only with `--skip-ssl` |
| A server offering no TLS, retained beside the TLS-capable ones | `10.11` alone offers none, and it is the one series that cannot supply the row above |

Three consequences follow, and each is a limit on what may be claimed today
rather than a plan.

- **The default TLS mode has no acceptance test.** The default of
  `FR-CONF-013` is the one cell of `FR-CONF-038`'s ten-cell table the fixture
  cannot exercise, and it is the mode every caller meets without asking for
  it.
- **Every server-reaching test must set `tls` away from its default**, which is
  precisely the cost the eighth edition refused to leave standing as a
  consequence and made a requirement instead.
- **Nothing here describes how the gap is closed.** `FR-CONF-038` hands the
  generation of the certificate, where the fixture keeps it, and how the no-TLS
  server is retained beside it to the fixture's own work, and
  [`OD-22`](open-decisions.md#od-22--the-test-harness-and-the-fixture-certificate)
  records why no arrangement is written down before it has been run: a
  description of something imaginary is what that residual exists to prevent,
  and it is the same ground
  [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) gives for prescribing
  no continuous-integration pipeline.

**A second file is owed and is not this document's.** `seed-bench.sql`, which
`WL-001` needs, does not exist; `BR-PERF-007` counts what cannot be realised
without it, and
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001) settles when it is
written. `DIV-036` is the correction owed to the root documents for it.

**This section is superseded when the fixture work lands**, and nothing else in
this document is affected by it.

## No continuous integration is prescribed

There is no pipeline, and
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) declines to describe one
rather than record an aspiration as though it were the state of the system.
Every gate in this document is consequently carried by a person, and
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) lists the four hand-carried
obligations for that reason — an unlisted manual obligation is one nobody is
accountable for.

Two properties make the absence tolerable rather than merely recorded.
`BR-PROJ-001` makes behaviour fully determined by the contents of the project,
so a result is reproducible between machines; and `NFR-DET-001` makes a
difference between two runs over unchanged inputs a defect rather than noise.

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every measured figure and every recorded baseline | `BENCHMARKS.md` |
| Every budget, its standing, and the protocol that ratifies one | [quality-attributes.md](quality-attributes.md) |
| The harness that drives the four containers, the mandated tests, and the two in-process seams | `verification.md` |
| Crate versions, features, and the dependency budget | [technology-stack.md](technology-stack.md) |
| The module map, the invocation pipeline, and lazy initialisation | [architecture.md](architecture.md) |
| The diagnostic renderer, the emitter, and the help surface | [interfaces.md](interfaces.md) |
| The four numbers themselves, and everything `.tpl` holds | [data-model.md](data-model.md) |
| The six untrusted inputs, credentials, and transport | [security.md](security.md) |
| The fixture's contents, its deliberate omissions, and the seven differences observed between the series | `scripts/mariadb/README.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
