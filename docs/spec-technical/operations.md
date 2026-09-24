---
title: Operations
status: draft
last-reviewed: 2026-09-24
related: [README.md, traceability.md, open-decisions.md, overview.md, architecture.md, technology-stack.md, data-model.md, quality-attributes.md]
---

# Operations

## What this document is

How the artefact is produced, validated, released and observed: the four
targets and what their linkage obliges, the pipeline every change passes, the
gates a release passes, the harness that takes a performance reading, where each
of the four version numbers is bumped, what a new project receives, what the
development host has installed, and what the container fixture supplies today.

**No measured figure appears here.** Every figure is `BENCHMARKS.md`'s, which
`BR-PERF-006` requires; the nine measurement points, what each is measured over
and the protocol a reading is taken under are
[quality-attributes.md](quality-attributes.md#the-measurement-set). Versions,
crate choices and the dependency budget are
[technology-stack.md](technology-stack.md); the tests themselves, and the
harness that drives the containers, are `verification.md`.

**No performance figure gates anything in this document.** `BR-PERF-008` denies
every figure named in the functional corpus, and every figure recorded against
it, the power to fail, block, reject or gate a change, a release or a piece of
work; the pipeline below and the gates below carry none.

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

**The form of the release artefact is
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md)'s** — the archive per
target, its name and contents, the checksum file, and the absence of a
signature. This document does not restate it.

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
| 4 | The test suite, with every feature active, correctness tests only. Measurement tests are ignored and are not part of the run ([`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), Decision 10) | See the two observations below |
| 5 | The dependency graph against the advisory database | `cargo audit` is described by its publisher as auditing "`Cargo.lock` for crates with security vulnerabilities" (crates.io crate index and rustsec.org, `cargo-audit` 0.22.2, consulted 2026-09-11), so it needs a resolved lockfile |

**Nothing is added to the five for a change to a hot path.** The root
coordination document states that a hot-path change — the catalogue read, the
context build, the render, process startup — is an **occasion to take a reading
and record it**, and never a condition for the work being complete
(`CLAUDE.md`, *Desenvolvimento*, with *Disciplina de medição*). The protocol a
reading is taken under is
[quality-attributes.md](quality-attributes.md#the-measurement-protocol)'s, and
the harness that takes it is [below](#the-measurement-harness). The measurement
tests the pipeline ignores run on demand with
`cargo test --all-features -- --ignored`, outside both workflows, under
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), Decision 10.

**Observation — the pipeline names no target, so it exercises the host's.**
`NFR-PERF-018` makes none of the four second class, so passing on the
development host is not passing. The `ci.yml` workflow that
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) prescribes runs the
five commands on all four targets; see
[Continuous integration and release](#continuous-integration-and-release). On
every target, command 4 leaves out the measurement tests, which carry the
`ignore` attribute that [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), Decision 10, prescribes.

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
  it. **Since 2026-09-23 the constraint is tighter**: the shipped binary already
  installs a global allocator, the counting one of
  [`ADR-011`](../adr/adr-011-render-memory-accounting.md), and a process has one.
  An in-tree profiling allocator would displace it, and with it the count the
  render memory limit reads; `ADR-011` leaves the choice between composing the
  two, gating one out, or profiling out of tree untaken, and so does this
  document.

**The pipeline has a package to run against.** The manifest and the resolved
`Cargo.lock` are both in the repository: command 5 has the lockfile it audits,
and the other four have a crate to compile.

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
  figure**, on the same ground as changing the build path
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
[`OD-02`](open-decisions.md#od-02--the-msrv) and cited rather than restated. One
consequence is operational: the floor is set by a dependency, so a dependency
bump is also an MSRV question. The obligation that record carried to re-run its
rule over the shipped graph once the graph resolved is discharged inside the
record, and the figure the rule yields is the figure `rust-version` carries.

**The development toolchain is not the pin, and it is not the measured one
either.** Three toolchain figures are distinct, all three are true at once, and
each has exactly one home.

| Figure | Where it lives | What it is |
|---|---|---|
| The floor | [`ADR-007`](../adr/adr-007-msrv.md), cited above and not restated | The figure the MSRV rule yields, written into `rust-version`. Below it the package does not build |
| The current development toolchain | **Here**, in the inventory below | What the development host has installed and active today. Above the floor, and it moves whenever the host is updated |
| The toolchain a measurement was taken under | `BENCHMARKS.md`, cited and not restated | Part of the environment of one recorded result. It does not follow the host forward |

**The floor is also the CI toolchain.** Both workflows
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) prescribes install
the `rust-version` the manifest declares, not the development toolchain. A lint
that clippy raises on the floor and not on the host's toolchain therefore fails
CI; that record's *Consequences* gives the case already observed.

**The second and the third have already parted**: the host is one patch release
ahead of the toolchain `BENCHMARKS.md` records for the driver selection. That is
not a defect and it invalidates nothing —
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) makes the build path part
of the evidence, so a recorded figure stays attached to the environment that
record names for it, and a re-measurement under a newer toolchain is a new
record rather than a correction of the old. It does mean the driver measurement
cannot be reproduced exactly on this host as it stands: the toolchain it was
taken under is not among those installed.

**What the development host has installed.** The `aarch64-apple-darwin` machine
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) records as the host of
record, observed on 2026-09-11 with `rustup show`, `rustc --version --verbose`
and `cargo --version --verbose`. It is a snapshot of one machine on one date —
not a pin, not a requirement, and re-taken rather than assumed.

| Component | Observed on 2026-09-11 |
|---|---|
| `rustc` | 1.98.1 (`48a229cea`, 2026-09-01), LLVM 22.1.8 |
| `cargo` | 1.98.1 (`797e8a9bc`, 2026-08-05) |
| `rustfmt`, which command 1 runs | 1.9.0-stable |
| `clippy`, which command 2 runs | 0.1.98 |
| Toolchains installed | `stable`, which is both the default and the active one; `nightly-aarch64-apple-darwin`; `nightly-x86_64-unknown-linux-gnu`; `1.87.0-aarch64-apple-darwin` |
| Targets installed | The four of `NFR-PERF-018`, all four present, and no other |
| Components beyond the default set | `llvm-tools` |

**All four targets being installed lets the pipeline run locally against each,
and nothing else.** Covering all four targets is no longer carried by a person:
the `ci.yml` workflow that
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) prescribes runs the
pipeline on each, by the build path of
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md).

**The `1.87.0` toolchain is installed and is below the floor.** It is not the
active one, no path in this document selects it, and the floor
[`ADR-007`](../adr/adr-007-msrv.md) yields forbids building the package with
it. It is recorded so that its presence is not read as support for it.

**Neither nightly toolchain has a role here.** No requirement, no record and no
command of the five names a nightly toolchain or a nightly-only feature.

**The cross-build path is
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md)'s**: the two `musl`
targets are built through a zig-based linker driver at the versions that record
names, and the two Darwin targets are built natively. The versions are the
record's and are not repeated, because they are part of the evidence — they are
the path that produced the artefacts `BENCHMARKS.md` measured, and a different
path makes a later figure incomparable with the recorded one. The host carries
both tools at exactly the two versions that record names, so the recorded path
is the path that is installed.

| Tool | Role | Installed on 2026-09-11 | Role fixed by |
|---|---|---|---|
| The Rust toolchain | Compiles the four targets; above the floor | The inventory above | [`ADR-007`](../adr/adr-007-msrv.md) |
| The zig-based linker driver | The `musl` cross-link from a Darwin host, which the Apple linker cannot perform | `cargo-zigbuild` and `zig`, at the two versions that record names | [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) |
| `cargo fmt`, `cargo clippy`, `cargo audit` | The validation pipeline above | The first two ship with the toolchain above; `cargo-audit`, on the terms below | `CLAUDE.md`, *Desenvolvimento*; in CI, [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) |
| `hyperfine`, `samply` / `cargo flamegraph`, `cargo bloat` | Wall time, CPU attribution and binary size, one tool per question | `hyperfine` 1.20.0; `samply` 0.13.1; `flamegraph` 0.6.14, which provides `cargo flamegraph`; `cargo-bloat` 0.12.1 | `CLAUDE.md`, *Disciplina de medição* |
| `criterion`, `dhat-rs` | Micro-benchmarks and heap profile | Neither is an installed binary; see below | `CLAUDE.md`, *Disciplina de medição* |
| Docker | Runs the four fixture containers | 29.7.2 | `scripts/mariadb/README.md` |

The third column is `cargo install --list` for the cargo-installed binaries and
the host's own report for `hyperfine` and Docker, all on 2026-09-11.

**The table describes the development host, not CI.** In the workflows of
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), `cargo-audit` is the
prebuilt release at the version that record pins, and it is checked against a
hard-coded SHA-256 before extraction, as are `cargo-zigbuild` and zig.

The measurement toolchain is listed, not specified: what question each tool
answers is `CLAUDE.md`'s table, the protocol a reading is taken under is
[quality-attributes.md](quality-attributes.md#the-measurement-protocol), and
every result is `BENCHMARKS.md`'s. That table's own preamble says what these
tools are: instruments that answer a question on demand, and refuse nothing.

**Three tools this document names were absent from the host until 2026-09-11**,
when they were installed: `cargo-audit`, which is command 5 of the mandatory
pipeline, and `cargo-bloat` and `cargo flamegraph`, which answer two of the five
measurement questions. Nothing recorded rests on their having been present
earlier — the package the pipeline compiles was not created until 2026-09-12,
and the one recorded measurement used `hyperfine`, which was installed. The
installed `cargo-audit` is at the same version as the documentation command 5's
row was consulted against, so that citation is now reproducible on this host.

**Seven things installed on the host belong to no path this document
prescribes** (`cargo install --list` and `rustup show`, 2026-09-11):
`cargo-vet` 0.10.2, `cargo-llvm-cov` 0.8.7, `cargo-fuzz` 0.13.1, `cargo-pgo`
0.3.0, `cargo-show-asm` 0.2.59, `cross` 0.2.5, and the `llvm-tools` component.
None is named by a requirement, by a record, by the validation pipeline or by
the measurement table, and this document invents no role for a tool the corpus
does not use. They are listed because an unlisted tool on the host is one a
later reader mistakes for part of the path.

`cross` is the one of the seven with a standing entry:
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md) rejected it on evidence
rather than on merit, and having it installed does not readmit it — that
rejection expires only when all four targets are re-measured under a new path.

**Two of the measurement instruments are crates, not installed binaries.**
`criterion` and `dhat-rs` enter as dev-dependencies, outside the shipped graph
and outside the dependency budget
([`ADR-006`](../adr/adr-006-package-layout.md),
[technology-stack.md](technology-stack.md#the-dependency-budget)). The manifest
declares no dev-dependency, so neither crate is resolved and no version is
recorded for either. A version for either is the manifest's and never the
host's: neither is installed as a binary, so `cargo install --list` says nothing
about them.

**No MariaDB or MySQL client is installed, and the fixture requires none**: its
readiness gate is the published port and its query path is `docker exec` into
the container, both `scripts/mariadb/README.md`'s. Nothing else this document,
`CLAUDE.md`'s two tables or any record names is missing from the host.

## The measurement harness

`benches/` holds the instrument that takes a reading of a measurement point of
`NFR-PERF-014` and of the `WL-002` scalar. Four operational facts belong here;
what it measures and under what protocol are
[quality-attributes.md](quality-attributes.md#the-measurement-protocol)'s, and
every figure it produced is `BENCHMARKS.md`'s.

| Fact | Consequence |
|---|---|
| It is a **shell runner**: no `.rs` file under `benches/`, and no `[[bench]]` table or any other change in `Cargo.toml` | Cargo determines targets from the file layout and from those tables (The Cargo Book, *Cargo Targets*, **Target auto-discovery**, consulted 2026-09-22), so it infers **no bench target** here — `cargo metadata --no-deps` lists a lib, a bin and nine test targets and nothing else, read on 2026-09-22. The **mandatory validation pipeline is untouched**: nothing under `benches/` is compiled by any of its five commands, including the one that passes `--all-targets`, which is "equivalent to specifying `--lib --bins --tests --benches --examples`" (The Cargo Book, `cargo test`, consulted 2026-09-22) |
| It **enforces nothing** | It reads no earlier figure, never opens `BENCHMARKS.md`, computes no delta and no regression, withholds no figure, and emits no verdict. A completed campaign exits `0` whatever it measured; its only non-zero exit means the instrument could not run at all |
| The dispersion rule of `NFR-PERF-011` is **reported, not applied** | Every record carries the relative standard deviation of its samples and a plain boolean saying whether it exceeds five per cent. Both are data. Applying the rule to a field is a reader's act, not the harness's |
| Its records carry the schema `tpl-bench/2` | One machine-readable record per reading, naming the point, the target, the workload, the cache posture and the conditions. It is the schema an entry of `BENCHMARKS.md` is written from |

**The first fact is what keeps `BR-PERF-008` true of the build.** A bench target
Cargo discovered would be built by `--all-targets` and would put a measurement
instrument inside the one sequence that does stop work, which is exactly the
shape the withdrawal removed. The invocation, the flags, the fixture sequencing
and the record fields are `benches/README.md`'s and are not restated here.

## The release gates

A gate is a check whose failure stops the release. Four are standing; the
fifth fires only when a pin moves.

| Gate | What is checked | Trigger | Forced by |
|---|---|---|---|
| The validation pipeline | All five commands pass, in order | Every change, not only a release | `CLAUDE.md`, *Desenvolvimento* |
| Every target | The pipeline passes on all four of `NFR-PERF-018`; a failure on one is a failure | Every release | `NFR-PERF-018`; enforced by the workflows of [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) |
| Provenance | The four gates of [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), Decision 3, pass before any validation or publishing. Gate 3 checks that the tag is `v` and a valid Semantic Versioning 2.0.0 version equal to the manifest's; gate 4, that exactly one release-notes file matches the anchored pattern carrying the full tag. The expression and the pattern are that record's. A tag with a pre-release identifier publishes a GitHub pre-release, which `install.sh` never installs | Every release | [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) |
| The supported-series table | The table of `FR-SRV-015` is re-verified against its source, and its verification date moved | **Every release**, without exception | `FR-SRV-019`, `BR-SRV-004` |
| The engine pin | Every name of `FR-ENV-018` still exists and still behaves as before | Only when the pin of [`ADR-001`](../adr/adr-001-template-engine-pin.md) moves | `FR-ENV-003` |

**There is no performance gate, and the table above is short by one row that
used to be there.** A fourth standing gate failed the release whose measurement
was worse than the figure recorded for that point on that target. It is
withdrawn with the requirement that imposed it: under `BR-PERF-008` no figure
named in the functional corpus, and no figure recorded against it, fails, blocks,
rejects or gates anything. A regression is therefore an **observation** — worth
recording, worth investigating if somebody chooses to, and never a reason a
release stops. For the same reason, none of the obligations
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) leaves to a person is
owed to a figure.

**The series gate is the one that decays on a calendar.** `BR-SRV-004` makes a
wrong table stale rather than the criterion wrong, so the correction is always
to re-derive the table from the criterion and the source — never to amend the
criterion so that the old table stays true. A re-derivation that changes the
set changes what `FR-SRV-029` must be run against, and therefore what the
fixture must contain.

**Three gates are enforced by a workflow, and two are carried by a person.**
Under [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), `release.yml`
publishes nothing unless the tag passes the provenance gates and the tagged
commit passes the validation pipeline on all four targets, which enforces the
first three rows. The supported-series table is
re-verified by hand before the `v*` tag is pushed, as that record requires,
because the push is what publishes. The engine pin is checked by hand when the
pin moves; no workflow checks it.

## The four version numbers: where a bump is enacted

The four numbers, their starting values and what moves each are
[data-model.md](data-model.md#the-four-version-numbers)'s, settled in
[`OD-03`](open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog).
This section adds only where the bump is written and what a gate checks.

| Number | Where the bump is enacted | What a release gate checks |
|---|---|---|
| Binary version | The `version` field of the one manifest, and nowhere else | That it moved, and that the changelog carries the matching entry; `release.yml` checks that it is a valid Semantic Versioning 2.0.0 version and equals the tag without its `v`, under gate 3 of [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), Decision 3 |
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

**Which of the four sites exist in the repository** is
[data-model.md](data-model.md#the-four-version-numbers)'s note. This section
prescribes where a bump is written, whichever of them exists.

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
Projecto*) names a root `templates/` directory for them; the repository has no
such directory, and no command that would ship their content.

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

**This section describes the fixture as it stands on 2026-09-11**, after tasks
#15 and #25. The arrangement — one image definition parameterised by series,
the schema and seed the entrypoint runs, the credentials, and the commands that
build, start and stop it — is recorded in `scripts/mariadb/README.md` and is not
repeated here. What belongs here is the count of servers an operator must
account for, and the obligation each discharges.

| Servers | What they are | Why the count matters operationally |
|---|---|---|
| Four | One per series of `FR-SRV-015`, each with its own image tag, container and published port, all offering TLS with the fixture's own certificate | `FR-SRV-029` and `FR-SRV-026` compare across the series in **one** run, so the four stand side by side rather than in sequence |
| One | `tpl-mariadb-notls`, a fifth container on a fifth port running the `10.11` image with `--skip-ssl`. It is **not** a fifth series | It is the right-hand column of `FR-CONF-038`'s mode table — a server offering no TLS — and a cross-series comparison must not count it as a series |

**`FR-CONF-038`'s certificate obligation is satisfied.** The fixture carries its
own trust material, committed so that a fresh clone needs no preparatory step,
and the certificate names the three spellings of the loopback a test may write.
All four series report `have_ssl=YES` and accept a connection under the
`verify-identity` default of `FR-CONF-013`, `10.11` included. Three operational
consequences follow, and each is a limit on how a run is set up rather than a
plan.

- **The default TLS mode is now exercisable.** It was the one cell of
  `FR-CONF-038`'s ten-cell table the fixture could not reach, and a
  server-reaching test no longer has to set `tls` away from its default to
  reach the fixture at all.
- **`require_secure_transport` is deliberately unset**, because two cells of
  `FR-CONF-038`'s table expect a plaintext connection to a TLS-offering server
  to be accepted.
- **Regenerating the material is a rebuild and a fresh container**, since the
  image carries it; and it is not byte-reproducible, only meaning-reproducible.

**The last file the fixture owed was written on 2026-09-21.** `seed-bench.sql`
carries `WL-001` and `WL-003`, loaded on demand by `seed-bench.sh`;
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001) settled when it was
written and records the discharge. `DIV-036` is the entry of
`specification/upstream-divergences.md` that tracks what naming it still owes
the root documents, and what that is remains that register's to state.

## Continuous integration and release

[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) prescribes two GitHub
Actions workflows, both limited to correctness validations. Their triggers,
target matrix, release gate and artefacts are that record's and are not
restated here; the build path each uses per target is
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md)'s.

| Workflow | What it enforces in this document |
|---|---|
| `ci.yml` | The [validation pipeline](#the-mandatory-validation-pipeline), on all four targets of `NFR-PERF-018`, on every push and pull request |
| `release.yml` | The first three [release gates](#the-release-gates) — the provenance gates first, then the validation on all four targets — before it publishes. A pre-release tag publishes a GitHub pre-release |

**What each workflow installs is
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md)'s, not the
development host's.** The toolchain is the
[floor](#msrv-the-development-toolchain-and-the-cross-build-path).
`cargo-zigbuild`, zig and `cargo-audit` are pinned, and each download is checked
against a hard-coded SHA-256 before extraction; a mismatch fails the job. The
Darwin archives are created with the `tar` flags that record fixes, so that they
carry no extended attributes. That the runners' `tar` behaves as the development
host's did is **unverified** in that record.

**`install.sh`, at the repository root, is the installer
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) prescribes.** It
installs or updates `tpl` from the latest release for the host's target, after
checking the archive against `SHA256SUMS`. Its one-line invocation, its target
detection, its install directory and its privilege rule are that record's.
`TPL_INSTALL_DIR` is a variable of the script only: `tpl` never reads it. What
the one-line invocation and the hash pins trust is that record's *Consequences*.

**Neither workflow runs the fixture or the
[measurement harness](#the-measurement-harness).** A green run therefore does
not verify the server-dependent assertions; they stay with a person, and
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) (*Consequences*)
lists them. No figure is produced or consumed, consistent with `BR-PERF-008`.

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every measured figure, every dispersion, and the conditions a reading was taken under | `BENCHMARKS.md` |
| The nine measurement points, what each is measured over, and the protocol a reading is taken under | [quality-attributes.md](quality-attributes.md) |
| The harness that drives the five containers, the mandated tests, and the two in-process seams | `verification.md` |
| Crate versions, features, and the dependency budget | [technology-stack.md](technology-stack.md) |
| The module map, the invocation pipeline, and lazy initialisation | [architecture.md](architecture.md) |
| The diagnostic renderer, the emitter, and the help surface | [interfaces.md](interfaces.md) |
| The four numbers themselves, and everything `.tpl` holds | [data-model.md](data-model.md) |
| The six untrusted inputs, credentials, and transport | [security.md](security.md) |
| The fixture's contents, its deliberate omissions, and the nine differences its own passes observed between the series | `scripts/mariadb/README.md` |
| The record of every difference observed between the series — fourteen | `FR-SRV-038` |
| The release artefact: the archive per target, its name and contents, the checksum file, and the absence of a signature; and `install.sh` | [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
