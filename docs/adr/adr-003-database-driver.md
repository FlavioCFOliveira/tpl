---
id: ADR-003
title: The database driver
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-11
requirements: [FR-CONF-013, FR-CONF-036, FR-CONF-038, FR-SEC-021, NFR-PERF-004, NFR-PERF-012]
supersedes: []
superseded-by: null
---

# ADR-003 — The database driver

## Status

Accepted, 2026-09-10.

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

**`sqlx` 0.9.0 is the driver, on `tokio` 1.53.1.** `mysql` 28.0.2 is rejected.

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
states that discipline for a **ratified** figure — it lives in that file and
nowhere else, because a value in two places is two sources for one truth — and
this record extends it to a figure that is not ratified: `BENCHMARKS.md` marks
the driver-selection entry a selection record rather than a budget under
`NFR-PERF-020`, and a selection figure copied into a second file stops being
true just as quietly.

**What this record does not decide.** The runtime's flavour, its scope, and
where it is built are `ADR-005`. The mapping of the five modes onto this
driver's TLS surface is `ADR-002`. The toolchain floor this pin implies is
`ADR-007`.

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

**A version move is also a measurement move.** `NFR-PERF-012` makes a recorded
baseline meaningful only against the artefact and target it was taken on, and
`NFR-PERF-017` fails a change that regresses against one. A driver version other
than the one pinned here is a different artefact from the one every recorded
baseline was taken on.

**The driver is used without a pool.** `NFR-PERF-004` allows at most one
connection per invocation, and the process is ephemeral, so nothing a pool
offers is reachable within an invocation.

**One defect is open against this driver and is not a property of the choice.**
`BENCHMARKS.md` records an unexplained blocking cost inside connection
establishment on `musl` over raw loopback — reproducible, absent on the macOS
host, and tracked as roadmap task `#8`. The decision does not rest on that row:
the plaintext pair decides the same target the same way. It is a defect to be
understood before TLS over loopback is relied upon.

**Under R3, the pin and its rationale live here alone.** A document that needs
to name the driver may name the crate, as the root coordination document's stack
table does; the version, the rule that settled the choice, and the candidate
rejected are cited as `ADR-003` rather than restated —
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
| An unexplained blocking cost inside connection establishment on `musl` over raw loopback, reproducible, absent on macOS, tracked as roadmap task `#8` | `BENCHMARKS.md`, same entry, "The musl anomaly" | 2026-09-10 |
| `sqlx` 0.9.0 is the maximum stable release of the crate, published 2026-05-21 | crates.io crate index, `sqlx`, version 0.9.0 | 2026-09-11 |
| The five modes are normative over the driver; a driver that cannot express all five distinctly is disqualified; the choice may not be settled by reducing the mode set | `specification/configuration-model.md`, `FR-CONF-036` | 2026-09-11 |
| The corpus names no driver and states that which one is chosen is an architecture decision | `specification/README.md`, *Still out of scope* | 2026-09-11 |
| The two costs of a C-library binding — static linkage for the `musl` targets, and an ORM's query surface against the dependency budget | **Reasoned, not measured.** No figure in this repository separates a C-library candidate from either measured one | — |
