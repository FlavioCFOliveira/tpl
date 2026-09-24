---
id: ADR-007
title: The minimum supported Rust version
status: accepted
decided: 2026-09-11
last-reviewed: 2026-09-24
requirements: []
supersedes: []
superseded-by: null
---

# ADR-007 — The minimum supported Rust version

## Status

Accepted, 2026-09-11. The **rule** below was settled on 2026-09-10; the floor it
yields was established by verifying every direct dependency on 2026-09-11, and
is higher than the figure the project carried before that verification. The
rule was re-run over the resolved graph on 2026-09-12, and again on 2026-09-22
after a dependency left it, and the floor did not move on either occasion —
which is why the `decided` date above is unchanged.

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
verified 2026-09-11). When this record was opened, the same document deferred
the number rather than stating it, and no file of the repository stated one.
That is the gap this record was opened to close, and it is closed: commit
`50153d6` of 2026-09-11 reduced the deferring row to a citation of this record,
and `Cargo.toml` now carries `rust-version = "1.94.0"`.

**The paragraph above is in the past tense deliberately.** It used to quote the
deferral and to assert, in the present, that nothing in the repository stated an
MSRV; both halves were falsified by that commit and by the pin. Leaving them on
the ground that a Sources row dates them was weighed and refused: a date says
when a claim was checked, not that the body may go on asserting it. This is the
decay `OD-02` recorded for this same pair of documents, and the convention this
folder keeps against it is to re-read a quotation against the file that owns it,
not to date it and leave it standing.

## Decision

**The MSRV is the highest of two floors, and it is stated as a rule because the
number moves under it:**

1. the **edition floor** — 1.85.0, the release that stabilised edition 2024;
2. the highest floor **declared by any dependency in the shipped graph**, which
   is `cargo tree -e normal,build` under `ADR-006`.

**Applying the rule on 2026-09-22, over the resolved graph, the MSRV is
`1.94.0`**, and `rust-version = "1.94.0"` goes in `Cargo.toml`. It is set by the
driver crate `ADR-003` pins, which declares that floor; every other direct
dependency declares a floor at or below the edition floor:

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
| `rustix` | 1.1.4 | 1.63 |
| `serde` | 1.0.229 | 1.56 |

These are the **direct** dependencies the root coordination document's stack
table names, and no others: no logging facade appears, because none is a
dependency of this project. The second column carries the maximum stable release
at the date consulted except where it cites a record; **this record pins no
crate version**, only the toolchain floor the versions imply.

**The development toolchain is not the pin, and neither is the toolchain a
measurement was taken under.** They are two figures and not one, and neither is
this record's to hold: what the development host has installed is recorded in
`docs/spec-technical/operations.md`, and the toolchain the driver selection was
measured under is recorded in `BENCHMARKS.md`. This record cites both and
restates neither. Both stand above this floor, and the floor follows neither of
them.

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
  version builds a different artefact, and a figure is comparable only with
  another taken from the same build, so no recorded figure could be read
  against a later one. Buying a lower floor with a re-measurement of four
  targets, and a re-verification of the five-mode mapping of `ADR-002`, is a
  trade nothing in the corpus asks for: no requirement names a toolchain
  version at all.

- **Pinning to the development toolchain, or to the toolchain a measurement was
  taken under.** Refused: either forbids every toolchain between the floor and
  today, for no stated gain.

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

**The rule has been run over the whole shipped graph, and the floor did not
move.** `cargo tree -e normal,build` prints 136 crates, `tpl` among them; of the
135 dependencies, **115 declare a `rust-version` and 20 declare none** — and a
crate that declares none declares no floor, so it constrains nothing. The
highest declared floor is **1.94.0**, declared by the driver's own five crates —
`sqlx`, `sqlx-core`, `sqlx-mysql`, `sqlx-macros`, `sqlx-macros-core` — and by no
other crate in the graph. No transitive crate raises the floor, so the record's
one unverified point is discharged and `Cargo.toml` needs no edit. The reading
was taken for each of the four targets `ADR-008` names, which differ by exactly
one crate: `linux-raw-sys` 0.12.1 on the two Linux targets, in place of `errno`
on the two macOS ones, declaring 1.63. Two direct dependencies have moved since
the table above was read, without moving their floor: `toml` resolves at 1.1.6
and `toml_edit` at 0.25.15, and both still declare 1.85.

**A dependency has left the graph, and the floor did not move.** `anyhow`
carried a declared floor of 1.68 and is no longer a dependency of this project;
the decision that removed it, its ground and the alternative it rejected are
`docs/spec-technical/open-decisions.md` entry `OD-32`, and are not restated
here. It was the only crate to leave, so the counts above fall by one crate and
one declaration; the figure it declared was below the edition floor and bound
nothing, and the highest declared floor is the driver's either way. A
dependency **leaving** the graph is therefore the one dependency change that
cannot raise this floor — but it can lower it, when the crate that leaves is
the crate that set it, so the rule is re-run on a removal exactly as on a bump.

**The vendored copy of `ADR-010` does not reach this floor.** `sqlx-core` is
read from `vendor/sqlx-core-0.9.0/` rather than from the crate index, and the
vendored manifest carries the published `rust-version = "1.94.0"` unchanged.
Editing that manifest would be an MSRV question exactly as a dependency bump is,
and deleting the patch returns the reading to the index copy at the same figure.

**Only the driver's subtree declares above the edition floor.** After 1.94.0 the
next floors in the graph are 1.88, declared by the seven `icu` crates, and 1.86,
declared by `idna_adapter`; all eight reach the graph through `url`, which
nothing but the driver pulls in. Every remaining crate declares 1.85 or less.

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
| Declared floors: `clap` 4.6.6 → 1.85; `toml` 1.1.5 → 1.85; `toml_edit` 0.25.13 → 1.85; `tokio` 1.53.1 → 1.71; `serde_json` 1.0.151 → 1.71; `thiserror` 2.0.20 → 1.71; `rustix` 1.1.4 → 1.63; `serde` 1.0.229 → 1.56 | crates.io crate index, one request per crate | 2026-09-11 |
| The composition of the shipped graph, and the floor declared by every crate in it | `cargo tree -e normal,build` and `cargo metadata`, resolved against this repository's `Cargo.lock` at commit `455e48d` and run once per target `ADR-008` names; the `sqlx-core` figure is read from `vendor/sqlx-core-0.9.0/Cargo.toml`, which the `ADR-010` patch substitutes for the index copy | 2026-09-22 |
| `anyhow` is no longer declared and no longer appears in the shipped graph | `Cargo.toml` and `cargo tree -e normal,build`, at commit `455e48d` | 2026-09-22 |
| The decision that removed `anyhow`, its ground and the alternative it rejected | `docs/spec-technical/open-decisions.md`, `OD-32` | 2026-09-22 |
| The engine `ADR-001` pins declares 1.70 on the crate index; neither it nor the 1.63 read from its own documentation on 2026-09-10 — **not re-verified here** — reaches the edition floor | crates.io crate index, `minijinja` and `minijinja-contrib` | 2026-09-11 |
| Edition 2024 | `CLAUDE.md`, *Stack*, the language row | 2026-09-22 |
| That the same row deferred the number before this record existed, and that commit `50153d6` reduced it to a citation of this record | `git show 50153d6 -- CLAUDE.md` | 2026-09-22 |
| The development toolchain and the toolchain the driver selection was measured under are two distinct figures, and both stand above the floor | `docs/spec-technical/operations.md`, *MSRV, the development toolchain, and the cross-build path*, whose inventory is `rustup show`, `rustc --version --verbose` and `cargo --version --verbose` taken on the development host; and `BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection", *Environment* | 2026-09-11 |
