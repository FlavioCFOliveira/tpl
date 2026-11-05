---
id: ADR-003
title: The database driver
status: accepted
decided: 2026-09-10
last-reviewed: 2026-11-04
requirements: [FR-CONF-013, FR-CONF-036, FR-CONF-038, FR-SEC-021, NFR-PERF-004]
supersedes: []
superseded-by: null
---

# ADR-003 — The database driver

## Status

Accepted, 2026-09-10.

**Amended in place on 2026-11-03**, for rmp `#315`: the package now consumes the
driver as two of its crates rather than through its facade, so that the SQLite
driver of `ADR-013` can resolve beside it. The driver was not chosen again and
its pin did not move, so this is an edit in place and not a supersession, and
`decided` is unchanged; `last-reviewed` carries the amendment.

**Amended in place on 2026-11-04**, for rmp `#317`: the Decision named `tokio`
1.53.1, the release the selection was measured on, where the manifest has
required the 1.53 line, `~1.53.1`, since it was created; rmp `#320` then moved
`Cargo.lock` to 1.53.3, within that line. The Decision now states the line, by
the session coordinator's decision of that day, which states what the manifest
already required and is not a new choice. No version moved, so `decided` is
unchanged. The same pass added `hex` to the crates a *Sources* row names as
leaving the graph. The claims re-read that day are those of the *Sources* rows
dated 2026-11-04.

**Amended in place on 2026-11-04**, for rmp `#327`: the record names `ADR-014`
among what it does not decide and among the obligations of a move of its pin.
The driver, its pin and its form did not move, so `decided` is unchanged, and no
*Sources* row was re-read for this amendment.

## Context

`/specification` names no driver, and says so in the boundary it draws: "the
database driver is part of this: `FR-CONF-036` states what one must be able to
express, and no requirement names one". The choice is therefore architectural by
the corpus's own statement, and it is admitted to this register under rule R4's
second limb — the alternatives have to survive the choice, because the candidate
that was refused was refused for a reason that will recur the next time a driver
is weighed.

**`FR-CONF-036` is the rule the choice had to satisfy.** It makes the five TLS
modes of `FR-CONF-013` normative over the driver: one that cannot express all
five distinctly is **disqualified**, and the choice "SHALL NOT be settled by
reducing the mode set to what a candidate driver offers". Its own *Provenance*
records why the requirement exists at all — the criterion had to be a
requirement rather than a preference held by whoever ran the measurement — and
names the three quantities the measurement was to weigh: startup, peak resident
memory, and stripped binary size.

`FR-SEC-021` is what `FR-CONF-036` protects. A driver that collapses one mode
onto another makes `tpl` report a guarantee it is not giving, and the failure
arrives through the dependency rather than through the configuration, where
nothing in the configuration can show it.

The measurement was run on 2026-09-10 and its entire surviving record is
`BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection"; the spike itself was
deleted when the task closed. It was run against a suspicion the root
coordination document had written down: that an asynchronous runtime penalises
startup, binary size and resident memory in an ephemeral process.

## Decision

**`sqlx` 0.9.0 is the driver, on the `tokio` 1.53 line.** `mysql` 28.0.2 is
rejected. The manifest requires `tokio` as `~1.53.1`: 1.53.1, the release the
selection was measured on, or any later 1.53 release. Which release of the line
is built is `Cargo.lock`'s, and a move within the line moves no pin of this
record.

**What settled it was `FR-CONF-036`, not the measurement.** The rejected
candidate is disqualified by that requirement on two independent counts, both
established empirically against running servers rather than read off an API, and
both recorded in `BENCHMARKS.md`: `preferred` is not expressible through its
`Option<SslOpts>` surface, which affords only "never negotiate" or "require and
fail"; and `verify-ca` collapses onto `verify-identity` behind an arm that never
matches, returning a byte-identical error. `FR-CONF-036` disqualifies on either
count alone, and forbids closing the gap by narrowing the mode set, which is the
only other way either could have been lived with.

**The measurement agreed with the rule rather than opposing it**, on all three
of the quantities `FR-CONF-036`'s *Provenance* names, and it refuted the
suspicion that prompted it. The figures, the protocol, the noise floor and the
confounders are in `BENCHMARKS.md` and are **not restated here**. `BR-PERF-006`
states that discipline for a **recorded** figure — it lives in that file and
nowhere else, because a value in two places is two sources for one truth — and
this record extends it to a figure recorded against no measurement point:
`BENCHMARKS.md` marks the driver-selection entry a selection record rather than
a reading of a point of `NFR-PERF-014` recorded under `NFR-PERF-020`, and a
selection figure copied into a second file stops being true just as quietly.

**The package consumes the driver as two of its crates, not through its
facade.** It depends on `sqlx-core` and `sqlx-mysql`, each at `=0.9.0`, and not
on the crate `sqlx`, which re-exports them; where this record says `sqlx` 0.9.0
it names the driver and its release, not that crate. The exact requirement is
the one the facade itself places on each of its crates. The features are the
ones the facade enabled on those two crates under `mysql`, `runtime-tokio`,
`tls-rustls-ring-webpki` and its own defaults: on `sqlx-core`, `_rt-tokio`,
`_tls-rustls-ring-webpki`, `any`, `json`, `migrate` and `offline`; on
`sqlx-mysql`, `any`, `json`, `migrate` and `offline`. Why the facade was left,
the evidence that the crates linked into the binary did not change, and the
alternatives refused are `ADR-013`.

**The arrangement ends at the first `sqlx` release whose `sqlx-sqlite` admits
the `libsqlite3-sys` release carrying the SQLite release `FR-LITE-039`
requires**, which `ADR-013` names. At that release the package returns to the
`sqlx` facade, with facade features that give the two crates the effective set
above, and this paragraph is amended in place; the pin moves to that release
under the obligations below. The return is owed because both crates are marked
by upstream as not for direct use — "Not intended to be used directly" and "Not
for direct use; see the `sqlx` crate for details" — and the `_`-prefixed
features are internal to `sqlx`. The user decided the return on 2026-11-03.

**What this record does not decide.** The runtime's flavour, its scope, and
where it is built are `ADR-005`. The mapping of the five modes onto this
driver's TLS surface is `ADR-002`. The toolchain floor this pin implies is
`ADR-007`. What the project does about the TLS connect stall carried by the
pinned version is `ADR-010`. Why the driver is consumed as two crates is
`ADR-013`. That the driver also serves the `mysql` provider, and the patch it
carries for that provider, are `ADR-014`.

## Alternatives rejected

- **`mysql` 28.0.2, the synchronous candidate.** Disqualified by `FR-CONF-036`,
  on the two counts above. The manner of the second failure is the part worth
  preserving past the choice: it fails **silently**. The code compiles, the
  option exists, and the caller is handed a different guarantee from the one
  asked for — which is exactly the shape of failure `FR-CONF-036` and
  `FR-SEC-021` are written to prevent, and it is not visible from an API
  listing. A later candidate must be tested against running servers, mode by
  mode, and not read.

- **A binding to the C client library** — `mysqlclient-sys`, or `diesel` over
  that same library. `BENCHMARKS.md` records that there was no third candidate
  to weigh: these two are an FFI binding and an ORM over it, and the crates
  `mariadb`, `libmariadb-sys` and `mariadb-connector-c` do not exist on
  crates.io. Two further costs are **reasoned here and not measured**: a C
  library has to be built and statically linked for the two `musl` targets of
  `NFR-PERF-018`, which `ADR-008` would have to carry; and an ORM is a
  dependency whose purpose is the query surface `tpl` does not have, against the
  dependency budget the root coordination document sets.

- **Choosing on the numbers.** Refused as a method, not as an outcome. Had the
  measurement favoured the rejected candidate, `FR-CONF-036` would still have
  disqualified it, and the requirement exists precisely so that the measurement
  cannot be the thing that decides. The numbers are recorded in full because the
  suspicion they refuted was written into the project's own stack notes and
  deserves an evidence trail — not because they carried the decision.

## Consequences

**`ADR-002`'s forward reference is discharged.** That record cited
`FR-CONF-036` and `BENCHMARKS.md` directly because no record for the driver
choice existed. It now cites this one, and its Context and Consequences are
amended in the commit that creates this record.

**The suspicion in the root coordination document was refuted and is now
wrong.** Its stack table carries a note saying the async runtime did not
penalise startup, binary size or resident memory. Under R3 that rationale lives
here and the table cites `ADR-003`.

**The pin lives here; `BENCHMARKS.md` names artefacts, not pins.** The versions
in that file are a property of what was measured, in the same way `ADR-002`
treats the TLS crates it records. Moving this pin is an **edit to this record**
with a new `decided` date, and it carries two obligations that nothing else will
raise: re-checking the five-variant mapping of `ADR-002`, and re-checking the
toolchain floor of `ADR-007`.

**Under the consumption above, a move changes two requirements and re-derives
two feature lists.** Both crates move together, at the release moved to. Their
crates.io descriptions mark both as not for direct use, and the `_`-prefixed
features are what the facade forwards rather than a public interface
(`ADR-013`), so nothing promises that the lists recorded above carry across a
release: they are re-derived from the facade of the release moved to, as the
features it enables on the two crates under the same three features and its
defaults.

**A version move is also a measurement move.** A figure is comparable only with
another taken from the same build. A driver version other than the one pinned
here builds a different artefact from the one every recorded figure was taken
on, so the figures either side of the move are not readings of the same thing.
This is a design ground, not a requirement: `NFR-PERF-012` requires the target
and the server series, and names no artefact. Nothing follows from that by rule —
`BR-PERF-008` gives no figure the power to refuse a change — but a reader who
compares them without knowing is reading two things as one.

**The driver is used without a pool.** `NFR-PERF-004` allows at most one
connection per invocation, and the process is ephemeral, so nothing a pool
offers is reachable within an invocation.

**One defect is carried by the pinned version, and it is not a property of the
choice.** The blocking cost this record first carried as unexplained was
diagnosed by task `#8` on 2026-09-11: it is a TLS connect stall caused by a
regression in `sqlx-core` 0.9.0, reproducible on every supported server series
and over the bridge path as well as loopback, and it is neither specific to
`musl` nor a property of the raw loopback path as this record first described
it. `BENCHMARKS.md` supersedes that account in its 2026-09-11 entry, and
**`ADR-010` holds what the project does about it.** The decision recorded here
does not rest on the defect: the plaintext pair decides the same target the same
way, and the defect is in the version, not in the driver.

**Moving this pin onto the release that carries the upstream fix retires
`ADR-010`.** That is a third obligation on a version move, beside the two named
above, and that record states what the move must remove with it.

**Every move of this pin carries the patch of `ADR-014` with it.** That is a
fourth obligation: until the condition that record names is met, `sqlx-mysql` is
vendored again at the release moved to, and that record's change is re-applied
and checked against the release's published source.

**Under R3, the pin and its rationale live here alone.** A document that needs
to name the driver may name it, as the root coordination document's stack
table does; the version, the form in which it is consumed, the rule that
settled the choice, and the candidate rejected are cited as `ADR-003` rather
than restated —
`docs/spec-technical/technology-stack.md`, the root coordination document, and
`docs/spec-technical/open-decisions.md` entry `OD-11`, which cites this record
for the composition it records.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `sqlx` 0.9.0 with `tokio` 1.53.1 chosen and `mysql` 28.0.2 rejected on 2026-09-10; the spike was deleted and this entry is its whole surviving record | `BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection" | 2026-09-10 |
| `preferred` is not expressible through the rejected candidate's `Option<SslOpts>` surface; `verify-ca` collapses onto `verify-identity` behind an arm that never matches, returning a byte-identical error; both established against running servers | `BENCHMARKS.md`, same entry, "What decided it was a rule, not the numbers" | 2026-09-10 |
| There is no third candidate: `mysqlclient-sys` is an FFI binding, `diesel` an ORM over the same C library, and `mariadb`, `libmariadb-sys` and `mariadb-connector-c` do not exist on crates.io | `BENCHMARKS.md`, same entry | 2026-09-10 |
| The async candidate led on startup, stripped binary size and peak resident memory in every TLS-equalised comparison | `BENCHMARKS.md`, same entry, "Like for like, TLS equalised, paired per round" | 2026-09-10 |
| A blocking cost inside connection establishment, recorded on 2026-09-10 as unexplained and specific to `musl` over raw loopback. **That account is superseded**: task `#8` established a TLS connect stall in `sqlx-core` 0.9.0, on every supported series and over both network paths measured, and `ADR-010` holds the response | `BENCHMARKS.md`, "The musl anomaly — resolved on 2026-09-11" and "2026-09-11 — The TLS connect stall on Linux loopback" | 2026-09-11 |
| The manifest has required `tokio = { version = "~1.53.1", … }` since it was created, and `Cargo.lock` resolves 1.53.3; the move from 1.53.1 is rmp `#320`'s | `git log -S` over `Cargo.toml`, which finds the requirement only in `d8e7e8a`, the commit that created the manifest; `Cargo.lock` at `229edc8`; the message of commit `2fd98f8` | 2026-11-04 |
| `~1.53.1` admits `>=1.53.1, <1.54.0`: a tilde requirement with major, minor and patch allows patch-level changes only | The Cargo Book, *Specifying Dependencies*, *Tilde requirements* | 2026-11-04 |
| The session coordinator decided that the Decision states the 1.53 line, as what the manifest already required and not a new choice | The coordinator's message of 2026-11-04 for rmp `#317` | 2026-11-04 |
| `sqlx` 0.9.0 is the maximum stable release of the crate, published 2026-05-21 | crates.io crate index, `sqlx`, version 0.9.0 | 2026-09-11, re-checked 2026-11-03 |
| `sqlx` 0.9.0 requires each of its crates at `=0.9.0`; its defaults are `any`, `macros`, `migrate` and `json`; under those and `mysql`, `runtime-tokio` and `tls-rustls-ring-webpki` it enables on `sqlx-core` the features `_rt-tokio`, `_tls-rustls-ring-webpki`, `any`, `json`, `migrate` and `offline`, and on `sqlx-mysql` the features `any`, `json`, `migrate` and `offline` | crates.io API, `sqlx` 0.9.0, features and dependencies | 2026-11-03 |
| Consumed directly, `sqlx-core` and `sqlx-mysql` are linked at the same versions and with the same features as through the facade; only the facade, its macro crates, `hex`, which only they used, and what was compiled for them alone leave the graph | `cargo tree -e normal,build -f '{p} [{f}]'`, once per target of `ADR-008`; recorded in `ADR-013`, *Sources* | 2026-11-03, `hex` added 2026-11-04 |
| `sqlx-core` is "Not intended to be used directly" and `sqlx-mysql` "Not for direct use"; the `_`-prefixed features are listed upstream "for conditional compilation" | crates.io, crate descriptions; `transact-rs/sqlx` at tag `v0.9.0`, `Cargo.toml` and `sqlx-core/Cargo.toml` | 2026-11-03 |
| The five modes are normative over the driver; a driver that cannot express all five distinctly is disqualified; the choice may not be settled by reducing the mode set | `specification/configuration-model.md`, `FR-CONF-036` | 2026-09-11 |
| A recorded measurement names its target and, where it reaches a server, its series, and names no artefact | `specification/performance-requirements.md`, `NFR-PERF-012` | 2026-09-24 |
| The corpus names no driver and states that which one is chosen is an architecture decision | `specification/README.md`, *Still out of scope* | 2026-09-11 |
| The two costs of a C-library binding — static linkage for the `musl` targets, and an ORM's query surface against the dependency budget | **Reasoned, not measured.** No figure in this repository separates a C-library candidate from either measured one | — |
