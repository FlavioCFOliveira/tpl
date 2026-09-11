---
id: ADR-007
title: The minimum supported Rust version
status: accepted
decided: 2026-09-11
last-reviewed: 2026-09-11
requirements: []
supersedes: []
superseded-by: null
---

# ADR-007 — The minimum supported Rust version

## Status

Accepted, 2026-09-11. The **rule** below was settled on 2026-09-10; the floor it
yields was established by verifying every direct dependency on 2026-09-11, and
is higher than the figure the project carried before that verification.

## Context

**No requirement in force names a toolchain version**, which is why the
`requirements:` field above is empty. The record is admitted under rule R4's
second limb: the choice is architectural, and its alternatives — in particular
the one the verification below rejected — must survive it.

The floor is nevertheless not free. It is determined by `FR-CONF-036`, at one
remove: that requirement disqualified the synchronous driver candidate, `ADR-003`
pins the one that survived, and that crate declares the highest toolchain floor
in the graph. A record that stated a floor without saying where it came from
would leave the next reader to rediscover this.

Two facts fix the lower bound. The root coordination document fixes edition
2024, and Rust 1.85.0 is the release that stabilised it (Rust Edition Guide,
verified 2026-09-11). The same document deferred the number explicitly — "MSRV a
fixar no `Cargo.toml`" — so nothing in the repository has ever stated one.

## Decision

**The MSRV is the highest of two floors, and it is stated as a rule because the
number moves under it:**

1. the **edition floor** — 1.85.0, the release that stabilised edition 2024;
2. the highest floor **declared by any dependency in the shipped graph**, which
   is `cargo tree -e normal,build` under `ADR-006`.

**Applying the rule on 2026-09-11, the MSRV is `1.94.0`**, and
`rust-version = "1.94.0"` goes in `Cargo.toml`. It is set by the driver crate
`ADR-003` pins, which declares that floor; every other direct dependency
declares a floor at or below the edition floor:

| Crate | Version consulted | Declared floor |
|---|---|---|
| `sqlx` | the version `ADR-003` pins | **1.94.0** |
| `clap` | 4.6.6 | 1.85 |
| `toml` | 1.1.5 | 1.85 |
| `toml_edit` | 0.25.13 | 1.85 |
| `tokio` | the version `ADR-003` pins | 1.71 |
| `serde_json` | 1.0.151 | 1.71 |
| `thiserror` | 2.0.20 | 1.71 |
| `minijinja`, `minijinja-contrib` | the version `ADR-001` pins | 1.70 |
| `anyhow` | 1.0.104 | 1.68 |
| `rustix` | 1.1.4 | 1.63 |
| `serde` | 1.0.229 | 1.56 |

These are the **direct** dependencies the root coordination document's stack
table names, and no others: no logging facade appears, because none is a
dependency of this project. The second column carries the maximum stable release
at the date consulted except where it cites a record; **this record pins no
crate version**, only the toolchain floor the versions imply.

**The development toolchain is not the pin.** The toolchain the driver selection
was measured under, recorded in `BENCHMARKS.md`, remains the development
toolchain and is above this floor.

**Moving the floor is an edit to this record** with a new `decided` date, in the
sense `docs/adr/README.md` distinguishes: the decision — take the highest of the
two floors — does not change when the number does, and only its parameter moves.

## Alternatives rejected

- **Pinning to the edition floor and leaving it there.** This is the figure the
  project carried before the verification: 1.85.0, on the ground that pinning to
  the floor is the widest compatibility the edition permits. It is refused now
  that the dependency floors are known, because 1.85.0 is a toolchain the
  package **cannot be built with**: the driver crate declares 1.94.0. A declared
  MSRV that does not build is worse than none — it is a claim that fails for the
  first user who believes it, and it fails at compile time in a dependency
  rather than in this crate, where the message names neither the pin nor the
  reason.

- **Downgrading the driver to lower the floor.** Refused. The driver was
  disqualified-and-chosen against `FR-CONF-036` at the version `ADR-003` pins,
  and every figure in `BENCHMARKS.md` was taken on that artefact; a different
  version is a different artefact, which `NFR-PERF-012` makes incomparable with
  every recorded baseline. Buying a lower floor with a re-measurement of four
  targets, and a re-verification of the five-mode mapping of `ADR-002`, is a
  trade nothing in the corpus asks for: no requirement names a toolchain
  version at all.

- **Pinning to the measured development toolchain.** Refused: it forbids every
  toolchain between the floor and today, for no stated gain.

- **Following stable.** Refused: it is not a pin at all, and it makes the floor
  a fact nobody records — the precise gap this record closes.

## Consequences

**The floor is set by a dependency, so a dependency bump is now also an MSRV
question.** Nothing else in the repository will raise it. `ADR-003` carries the
matching obligation on its own side: moving that pin obliges a re-check here.

**The floor is high, and it is high for a reason nothing in the corpus asks
for.** No requirement names a toolchain version, so the cost of 1.94.0 is borne
entirely to keep the driver `FR-CONF-036` left standing. That is stated rather
than hidden, because a reader meeting `rust-version = "1.94.0"` in `Cargo.toml`
will otherwise assume the project chose it for itself.

**The caveat that preceded this record is discharged.** Before it, five
dependency floors stood explicitly unchecked and the number rested on the
assumption that none exceeded the edition floor. All five are now read from the
crate index, together with every other direct dependency, and the assumption was
wrong. Nothing in the table rests on
recollection. One incidental discrepancy is reported rather than reconciled: the
crate index declares 1.70 for the engine `ADR-001` pins, where a reading taken
from its own documentation on 2026-09-10 gave 1.63. Both are below the edition
floor, so neither binds and the difference changes no outcome.

**Only the direct dependencies have been read, and the rule reaches further
than that.** A transitive crate declaring a higher floor raises this one just as
a direct one does, and the graph cannot be enumerated before a manifest and a
lockfile exist. This is the record's one **unverified** point, and it carries an
obligation: re-run the rule over `cargo tree -e normal,build` the first time the
graph resolves, and amend this record if the floor moves. It can only move
upward.

**Dev-dependencies are outside the rule.** The floor above is the floor of the
shipped graph, per `ADR-006`. A dev-dependency that demanded a higher toolchain
would constrain whoever runs `cargo test`, not whoever builds the distributed
artefact, and it is not covered by this record.

**Under R3, the floor lives here alone.**
`docs/spec-technical/technology-stack.md`,
`docs/spec-technical/operations.md` and the root coordination document cite
`ADR-007` rather than restating it, and
`docs/spec-technical/open-decisions.md` entry `OD-02` reduces to a citation of
this record.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| Rust 1.85.0 is the release that stabilised the 2024 edition | The Rust Edition Guide, *Rust 2024* | 2026-09-11 |
| The driver crate `ADR-003` pins declares `rust-version = "1.94.0"`, and is the maximum stable release of that crate | crates.io crate index, `sqlx`, version 0.9.0 | 2026-09-11 |
| Declared floors: `clap` 4.6.6 → 1.85; `toml` 1.1.5 → 1.85; `toml_edit` 0.25.13 → 1.85; `tokio` 1.53.1 → 1.71; `serde_json` 1.0.151 → 1.71; `thiserror` 2.0.20 → 1.71; `anyhow` 1.0.104 → 1.68; `rustix` 1.1.4 → 1.63; `serde` 1.0.229 → 1.56 | crates.io crate index, one request per crate | 2026-09-11 |
| The floors of the **transitive** graph | **Unverified.** No manifest and no lockfile exist yet, so the graph cannot be resolved | — |
| The engine `ADR-001` pins declares 1.70 on the crate index; neither it nor the 1.63 read from its own documentation on 2026-09-10 — **not re-verified here** — reaches the edition floor | crates.io crate index, `minijinja` and `minijinja-contrib` | 2026-09-11 |
| Edition 2024, with the MSRV deferred to `Cargo.toml` | `CLAUDE.md`, *Stack* | 2026-09-11 |
