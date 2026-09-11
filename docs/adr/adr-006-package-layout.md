---
id: ADR-006
title: The package layout
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-11
requirements: [FR-ERR-031, BR-ERR-001, BR-SCH-004, NFR-PERF-018]
supersedes: []
superseded-by: null
---

# ADR-006 — The package layout

## Status

Accepted, 2026-09-10. The question the decision left inside it — whether
`benches/` needs a package of its own — was settled on 2026-09-11 and is stated
below as part of the decision in force.

## Context

The root coordination document describes one manifest at the root with `src/`
beneath it, and requires the logic to live in a library that is testable without
launching a process, with `main.rs` limited to parsing, dispatching, and mapping
an error to an exit code. What it does not settle is whether that is one Cargo
package or the root of a workspace, and the answer is architectural: the
alternatives have to survive the choice, which admits this record under rule
R4's second limb.

**`DIV-032` removes the usual reason to split.** Only the JSON document and the
command line are contract; the library carries no compatibility guarantee, and
the document contract lives in `specification/context-document.md` rather than
in a set of Rust types. There is therefore no published crate boundary to
protect, which is what a workspace normally exists to draw.

Three obligations bear on the shape from the other side. `FR-ERR-031` requires a
trigger for `70` that is reachable only from within the system's own test
configuration and from **no** invocation of the distributed binary;
`BR-ERR-001` makes that trigger stand in for the integration test `70` cannot
have; and `BR-SCH-004` mandates a round-trip test that dumps, feeds back through
`--context`, and asserts byte-identity. All three need the logic to be
addressable from a test without launching a process, which is the library half
of the split rather than the number of packages.

## Decision

**One Cargo package**, carrying a library and a binary. The logic lives in the
library; the binary parses, dispatches, and maps the error to an exit code.

**`benches/` gets no package of its own.** `criterion` and `dhat` are
dev-dependencies, and nothing a dev-dependency drags in is compiled into the
release artefact — so a second package would move nothing out of a graph it was
never in. The question a second manifest was reached for is answered by naming
the command that answers it: `cargo tree`'s `--edges` defaults to
`normal,build,dev`, which is why an unfiltered listing shows a dev-dependency,
and the shipped graph is what `cargo tree -e normal,build` prints — "a mostly
equivalent overview of what `cargo build` does" (Cargo Book, verified
2026-09-11).

**One manifest holds the settings that must not disagree**: the release profile
of `ADR-004` and the toolchain floor of `ADR-007`. Neither can drift against a
second copy, because there is no second copy.

## Alternatives rejected

- **A workspace of several crates.** Refused because it buys a boundary nothing
  consumes — `DIV-032` having removed the published surface a boundary would
  protect — and multiplies the manifests that must agree on the release profile
  and the toolchain floor. The cost is paid on every change to either, and the
  benefit is available to no consumer.

- **A `benches` package, or a workspace member for it.** Refused for the same
  reason at smaller scale: it adds a manifest that must agree with the first on
  the release profile and the floor, and changes nothing about what
  `cargo build --release` compiles.

- **A binary with no library.** Refused because it puts every test behind a
  process launch. `FR-ERR-031`'s trigger would then have to be reachable from an
  invocation of the binary, which that requirement forbids outright, and
  `BR-ERR-001`'s exception for `70` would have nothing to rest on.

## Consequences

**An unfiltered `cargo tree` overstates the shipped graph, and that is expected
rather than a defect.** Anyone auditing what the distributed artefact links
reads `cargo tree -e normal,build`. This matters because the dependency budget
the root coordination document sets is a budget on the shipped graph, and the
default listing is not it.

**Heap profiling of the binary is the one case that leaves the dev-dependency
graph, and it belongs to operations.** `dhat` profiles the process it is linked
into, and enabling it means installing a global allocator in the program itself
(docs.rs, verified 2026-09-11) — an optional **normal** dependency behind a
feature, which does enter the graph of the build that carries it. That build is
not distributed, and `NFR-PERF-018` reaches distributed artefacts. Whether the
project takes that path is a build question for
`docs/spec-technical/operations.md`; nothing in this record depends on the
answer, and a feature-gated allocator does not reopen the package count.

**The four targets share one manifest.** `NFR-PERF-018` makes no target second
class, and one manifest is what keeps the profile and the floor identical across
all four without a rule to enforce it.

**Splitting later is cheap; splitting now is not free.** Nothing in this record
forecloses a workspace if a consumer for a crate boundary ever appears. What it
refuses is paying for the boundary before one does.

**Under R3, the package count lives here alone.**
`docs/spec-technical/architecture.md` and
`docs/spec-technical/operations.md` cite `ADR-006` rather than restating it, and
`docs/spec-technical/open-decisions.md` entry `OD-04` reduces to a citation of
this record.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `cargo tree`'s `--edges` defaults to `normal,build,dev`; "for a mostly equivalent overview of what `cargo build` does, `cargo tree -e normal,build` is pretty close" | The Cargo Book, *cargo-tree*, `--edges` | 2026-09-11 |
| `dhat` heap profiling requires installing `dhat::Alloc` as the program's `#[global_allocator]`, and the custom allocator is slower than the normal one, so it is activated only while profiling | docs.rs, `dhat` 0.3.3, crate documentation | 2026-09-11 |
| The library carries no compatibility guarantee; only the JSON document and the command line are contract | `specification/upstream-divergences.md`, `DIV-032` | 2026-09-11 |
| The trigger for `70` is reachable only from the system's own test configuration and from no invocation of the distributed binary | `specification/errors-and-exit-codes.md`, `FR-ERR-031` | 2026-09-11 |
| One manifest at the root with `src/` beneath it; the logic in the library, testable without launching a process | `CLAUDE.md`, *Estrutura do Projecto* and *Organização* | 2026-09-11 |
