---
id: ADR-008
title: Packaging and the build path for the four targets
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-22
requirements: [FR-SRV-019, NFR-PERF-012, NFR-PERF-018]
supersedes: []
superseded-by: null
---

# ADR-008 — Packaging and the build path for the four targets

## Status

Accepted, 2026-09-10.

## Context

`NFR-PERF-018` fixes the target set at exactly four and makes no target second
class: `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, both
statically linked, and `x86_64-apple-darwin` and `aarch64-apple-darwin`.
`DIV-041` records that the `gnu` triples are not targets and that the root
coordination document's sentence deferring the matrix is superseded by that
requirement.

**What the corpus does not settle is how the four artefacts are produced.** How
a target is reached is a build question, not a functional one, and the corpus
names no toolchain — which admits this record under rule R4's second limb.

Two forces constrain the answer. The first is mechanical and `BENCHMARKS.md`
names it: the Apple linker cannot emit ELF, so a Darwin development host cannot
produce a `musl` artefact without a cross-linking toolchain. The second is
evidential: `NFR-PERF-012` makes a recorded measurement meaningful only against
the target it was taken on, and `BR-PERF-008` makes every figure a reading kept
for a reader rather than a verdict on a change. A build path that differs from
the one that produced the recorded artefacts puts every later comparison in
question before it is made — not because a comparison can refuse anything, but
because two figures taken under two build paths are not readings of the same
thing.

## Decision

**`cargo-zigbuild` for the two `musl` targets; native builds for the two Darwin
targets.** This reproduces the path `BENCHMARKS.md` records for the measured
artefacts — `cargo-zigbuild` 0.23.4 with zig 0.16.0 — so that a later
measurement can be read against the recorded one, which is the attribution
`NFR-PERF-012` requires of a recorded figure.
`cargo-zigbuild` is described by its publisher as compiling "Cargo project with
zig as linker" (crates.io, verified 2026-09-11).

**No continuous integration is prescribed.** The targets and the toolchain are
described; no pipeline is. There is none today, and its absence is recorded here
as future work rather than described as though it existed.

**The form of the release artefact is not fixed by this record** — bare binary,
archive, checksums, signature. Nothing in the corpus constrains it yet, and a
record is not the place to invent a constraint.

## Alternatives rejected

- **`cross`, or a container-based build**, for the `musl` targets. Either may be
  right later, and neither produced the recorded figures. Adopting one now would
  make the first post-decision measurement unreadable against the recorded
  figures it would stand beside, which is the cost `NFR-PERF-012` is written to
  prevent. This is a rejection on evidence, not on the merits of the tools, and
  it expires the moment a re-measurement of all four targets is done under a new
  path.

- **Prescribing a CI pipeline now.** Refused because it would be aspiration
  rather than specification: a description of something imaginary in a corpus
  whose whole discipline is to describe what is true today. The obligations a
  pipeline would carry are real and are listed below; naming them as manual is
  accurate, and naming them as automated would not be.

- **Building the `musl` targets on a Linux host instead.** It removes the
  cross-linking problem rather than solving it, and it removes it only for
  whoever has such a host. The development host of record is a Darwin machine,
  and a build path that only some contributors can run makes two of the four
  targets second class in practice, which `NFR-PERF-018` forbids in principle.

## Consequences

**Three obligations are carried by hand, and each is a hand that can forget.**
Until a pipeline exists, whoever runs the work carries: the five-command
validation sequence the root coordination document mandates; `NFR-PERF-018`'s
no-second-class rule across all four targets; and `FR-SRV-019`'s re-verification
of the supported-series table before every release. They are listed because an
unlisted manual obligation is one nobody is accountable for.
No performance figure joins that list: `BR-PERF-008` gives no figure the power
to refuse a change, so nothing here is owed to a number before work can be
called done.

**The build path is part of the evidence, not only of the process.** A change to
it invalidates the comparability of every recorded figure in the same way a
profile change would — see `ADR-004`, where the same argument refuses a
different change. Whoever changes it re-measures all four targets, or states
that the figures before and after are not comparable.

**Two targets have never been measured.** `BENCHMARKS.md` records figures for
the two `arm64` targets only. The `x86_64` halves of `NFR-PERF-018` are reached
by the same build path by construction, but no figure stands against them, and
none should be inferred from their `arm64` twins — `NFR-PERF-012` forbids
exactly that comparison.

**No `musl` artefact has been run outside a container.** `BENCHMARKS.md` records
that every `musl` figure was taken inside Docker. The static linkage
`NFR-PERF-018` requires is what makes running outside one expected to work; that
expectation is **unverified** and is recorded here rather than assumed.

**Under R3, the build path lives here alone.**
`docs/spec-technical/operations.md` cites `ADR-008` rather than restating it,
and `docs/spec-technical/open-decisions.md` entry `OD-23` reduces to a citation
of this record.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `cargo-zigbuild` compiles a "Cargo project with zig as linker"; 0.23.4 is its maximum stable release | crates.io crate index, `cargo-zigbuild` | 2026-09-11 |
| The Apple linker cannot emit ELF; the measured artefacts were produced with `cargo-zigbuild` 0.23.4 and zig 0.16.0; every `musl` figure was taken inside a container; only the two `arm64` targets were measured | `BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection", Environment and Results | 2026-09-11 |
| The target set is exactly four, the two Linux targets are `musl` and statically linked, and no target is second class | `specification/performance-requirements.md`, `NFR-PERF-018` | 2026-09-11 |
| A measurement names the target it was taken on and measurements on different targets are not compared; no figure named in the corpus and no figure recorded against it fails, blocks, rejects or gates a change | `specification/performance-requirements.md`, `NFR-PERF-012`, `BR-PERF-008` | 2026-09-22 |
| The `gnu` triples are not targets | `specification/upstream-divergences.md`, `DIV-041` | 2026-09-11 |
| The supported-series table is re-verified against its source before every release | `specification/server-contract.md`, `FR-SRV-019` | 2026-09-11 |
