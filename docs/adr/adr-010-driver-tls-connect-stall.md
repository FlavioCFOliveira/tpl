---
id: ADR-010
title: The TLS connect stall in the pinned driver
status: accepted
decided: 2026-09-11
last-reviewed: 2026-09-11
requirements: [FR-CONF-013, FR-CONF-036, NFR-PERF-012, NFR-PERF-014, NFR-PERF-017]
supersedes: []
superseded-by: null
---

# ADR-010 — The TLS connect stall in the pinned driver

## Status

Accepted, 2026-09-11.

**The choice was made by the user on 2026-09-11**, from four options put to them
with the measured effect of each. This record does not make that choice and does
not re-open it; it records the option taken, the three refused, and the
condition that ends it.

**The one question this record left open — where the patched source lives — was
closed by the same user on 2026-09-11**, from three options put to them. The
decision below did not change; the parameter it was missing was supplied. That
makes this an edit in place under this register's own rule rather than a
successor record, and the two halves are meant to read as one decision.

## Context

`ADR-003` pins `sqlx` 0.9.0. That version carries a defect which task `#8`
diagnosed on 2026-09-11 and `BENCHMARKS.md` records in full under
"2026-09-11 — The TLS connect stall on Linux loopback". The mechanism, and only
the mechanism, is restated here, because the decision is unintelligible without
it:

- `sqlx-core` 0.9.0 never calls `set_nodelay`, so Nagle's algorithm stays on;
- `sqlx_core::net::tls::util::StdSocket` implements `io::Write` with `write` and
  `flush` only, so it inherits `std`'s default `write_vectored`, **which writes
  the first buffer only**. `rustls` offers the whole client flight at once and
  the gather never happens.

Chained, the 6-byte `ChangeCipherSpec` record leaves alone, Nagle holds the
`Finished` record behind it, and the server — which has nothing to send until it
sees that `Finished` — answers only with Linux's delayed ACK. `TCP_DELACK_MIN`
is `HZ/25`, so the wait is of the order of 40 ms. **It is a block, not a cost:**
nothing is computing meanwhile, and removing either ingredient removes it. Every
figure, every repeat, the protocol and the instrument's resolution are in
`BENCHMARKS.md` and are **not restated here**, for the reason `BR-PERF-006`
gives and `ADR-003` extends to a figure that is not a ratified budget.

**It is a regression in a published version, already fixed upstream and not yet
released.** `set_nodelay` was present in `sqlx-core` 0.7.4 through 0.8.6, was
dropped by the runtime rewrite, and is present again on `main` through PR
`#4336`, merged 2026-08-17 after a third party re-reported the symptom as issue
`#4335`. No release carries it: 0.9.0, published 2026-05-21, is still the
maximum stable release on crates.io as of 2026-09-11. The project is therefore
choosing what to do **in the interval before a release**, not what to do about a
permanent upstream position.

**Why this is admitted to the register.** No requirement in force delegates this
fact out of the corpus, so rule R1's first limb does not apply. It is admitted
under R4's second limb: the decision changes what the package is built against,
and its alternatives have to survive the choice, because the question returns
unchanged at every dependency review until a release carries the fix, and
because the option taken has an expiry that a later reader must be able to
recognise.

**What the corpus makes of the defect.** `FR-CONF-013` defaults `tls` to
`verify-identity`, so the default mode is one of the four that pay the stall;
only `disabled` does not. Three of `NFR-PERF-014`'s nine budgets reach a server,
and two of those three are time budgets — `tpl schema dump` over `WL-001`, and
the canonical loop of 200 invocations — against provisional figures stated in
milliseconds. `NFR-PERF-012` and `NFR-PERF-017` make the identity of the built
artefact material to every baseline taken against it.

**Nothing is built yet.** There is no `Cargo.toml` and no `src/`. This record
prescribes what the sprint that creates the manifest must put in it; it
describes nothing that exists.

## Decision

**The package patches the driver rather than living with the stall, changes
version, or diverges from upstream's own fix.**

The manifest, when it is created, SHALL carry a `[patch.crates-io]` entry
redirecting `sqlx-core` to a copy of the published 0.9.0 source carrying one
added statement — `socket.set_nodelay(true)?`, where the TLS transport
establishes its socket. That statement is the change upstream made in PR
`#4336`, and the patched source SHALL differ from published 0.9.0 in that and in
nothing else.

**The pin of `ADR-003` is unchanged.** The dependency remains `sqlx` 0.9.0; the
patch applies over that version and replaces no version. This record holds the
patch; `ADR-003` holds the pin.

Two mechanical constraints follow from how Cargo reads a patch, and are stated
here so the manifest is written once: the entry SHALL be declared in the
workspace-root manifest, because Cargo reads patch settings nowhere else; and
the patched source SHALL be pinned to an immutable revision — a git `rev` or
tag, or a path inside this repository — never a branch, because `NFR-PERF-012`
requires a recorded baseline to name the artefact it was taken on and a floating
source leaves that artefact unnameable.

**The condition that retires this record, stated so that it cannot be forgotten:
the first `sqlx` release whose `sqlx-core` contains PR `#4336`.** On that
release the patch entry is deleted, the pin of `ADR-003` moves to that release
under the two obligations that record already carries, and **this record becomes
`withdrawn`** — made moot, with no successor, under the second limb of the
register's own lifecycle. The patch is an interval measure with a named end; a
patch with no stated expiry becomes permanent by forgetting, and this one must
not.

**Where the patched source lives: a vendored copy inside this repository.** The
sprint that writes the manifest SHALL place the patched source at
`vendor/sqlx-core-0.9.0/` — a tracked directory holding the published 0.9.0
source of `sqlx-core` with the one added statement and nothing else — and SHALL
redirect to it by path, `sqlx-core = { path = "vendor/sqlx-core-0.9.0" }` under
the `[patch.crates-io]` table of that manifest. It is the form `BENCHMARKS.md`
records for the measured candidates, so what this record prescribes is the
configuration the measurement was taken on. The version belongs in the directory
name: a later version is a different tree, never an overwrite of this one. The
tree is maintained by hand — nothing here prescribes `cargo vendor` — and the
single statement is the only edit it carries, so any other divergence from the
published source is a defect in it.

**Of the three forms the immutability constraint admits, this is the
strongest.** The bytes compiled are fixed by the commit of this repository the
build was taken at, so a baseline names its artefact by naming that commit and
its target, which is what `NFR-PERF-012` requires; no revision of a second
repository has to be recorded, resolved, or still reachable for that name to
keep its meaning. The vendored tree SHALL carry its provenance beside it: the
crate and version it was taken from, and the upstream change the added
statement reproduces, PR `#4336`. Both are already facts of this record; what
the vendored form adds is that they travel with the source, so the divergence
from the published crate is checkable with no network access — the property the
form was chosen for.

**The manifest SHALL exclude that directory from the workspace.** Cargo makes
every path dependency residing in the workspace directory a workspace member
automatically, and `exclude` is what prevents it. Whether a `[patch.crates-io]`
entry whose source is a path inside the workspace is itself caught by that rule
is not established here; the exclusion costs one line and makes the question
moot. Without it the mandatory validation pipeline — `cargo fmt --all`, `cargo
clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`
— would run over third-party source this project does not own and cannot fix,
and `-D warnings` is not negotiable here. The exclusion adds a `[workspace]`
table to the one manifest of `ADR-006`; it adds no second manifest, and that
record's decision is untouched.

**The choice of form was the user's, on 2026-09-11**, on the ground that a
vendored copy is self-contained and builds with no network access. The driver
and its pin are `ADR-003`; the mapping of the five TLS modes onto it is
`ADR-002`; the runtime's scope is `ADR-005`; the toolchain floor the pin implies
is `ADR-007`; the build path and the four targets are `ADR-008`.

## Alternatives rejected

The first three refusals answer *whether and how to patch*; the last two answer
*where the patched source lives*. Each was put to the user with its merits, and
each was refused by the user.

- **Implementing `write_vectored` on `StdSocket` instead.** It removes the same
  stall, and `BENCHMARKS.md` records the two as equivalent at the point they
  were measured — their repeats overlap, and the entry states in terms that the
  two together are not measurably better than either alone on this path. It is
  refused on a ground the numbers cannot reach: **it is not what upstream did.**
  `StdSocket` still lacks a `write_vectored` implementation on `main`, so this
  variant has no upstream terminus. The patched copy would not disappear when
  the release lands; it would have to be carried, re-based and re-justified at
  every subsequent release. The project would own a permanent divergence for an
  effect the option taken obtains on loan. The option taken is deleted by a
  release; this one is not.

- **Pinning `sqlx` 0.8.6, the last release that carries `set_nodelay`.** It was
  **not measured in this investigation**: no figure in this repository separates
  it from either variant that was. It reverses, without the evidence that
  settled it, the pin `ADR-003` holds and the root coordination document's stack
  row cites. The objection is not that the mode API is absent — `MySqlSslMode`
  declares the same five variants in 0.8.6 as in 0.9.0, and this record checked
  that rather than assuming it. The objection is that **the verification does
  not carry across**: `FR-CONF-038`'s cell-by-cell behaviour was established
  against running servers on 2026-09-10, against the version `ADR-003` pins and
  against no other, and `ADR-003` records that this class of failure is silent
  and is not visible from an API listing — "a later candidate must be tested
  against running servers, mode by mode, and not read". `FR-CONF-036` is the
  requirement that would have to be satisfied again, not assumed. Two further
  records would move with the pin: `ADR-002`, whose mapping names feature names
  established at tag `v0.9.0`, and `ADR-007`, whose floor of 1.94.0 is declared
  by the crate at the version `ADR-003` pins. And it would give up everything
  0.9.0 carries — 0.8.6 was published 2025-05-19 and 0.9.0 2026-05-21, a year
  apart — to recover one statement.

- **Accepting the stall and building on the published crate unchanged.** The
  cost is a floor of the order of 40 ms on every connection that negotiates TLS,
  measured on the Linux target of record, on all four supported server series
  and over both network paths tried. It is refused because of where the cost
  would have to be absorbed: the two time budgets of `NFR-PERF-014` that reach a
  server are stated in milliseconds, and each would have to be widened to
  accommodate a defect in a dependency — the canonical loop of 200 invocations
  pays the floor once per invocation, since `NFR-PERF-004` gives each invocation
  its own connection. Under R2 this register is subordinate and cannot move a
  requirement; the corpus would have to be amended through
  `specification-manager`, so that `/specification` carried an upstream defect
  as though it were a property of the system. The option taken costs one
  statement and expires; this one costs an amendment to the corpus that would
  outlive the defect.

- **A fork of `sqlx` pinned by revision, rather than a vendored copy.** Its
  merits are real and were put as such: it keeps this repository's tree free of
  third-party source, so the review surface, the licence surface and every
  repository-wide search stay exactly what they are today; it makes the
  divergence from upstream visible as a diff in the fork itself, maintained by
  the tool built for precisely that; and pinned to a `rev` it satisfies the
  immutability constraint as fully as a path does. It is refused for two costs
  the vendored form does not carry: a second repository to create, keep, and
  keep reachable for as long as the patch lives, and a build that must reach the
  network for a dependency's source. The second weighs more here than it would
  elsewhere — the two Linux targets of `ADR-008` are `musl` targets, and the
  dependency budget the root coordination document imposes already requires
  every dependency to build on all four. Against that, the fork's advantages are
  bounded by the size of what is being carried: one statement, over a source
  that already exists in published form.

- **Deferring the question to the sprint that writes the manifest.** It is the
  cheapest option today and the only one that costs something later. This
  record's own Consequences make the `[patch.crates-io]` entry part of the
  manifest's first version, so the sprint that inherits the question would
  inherit it as a known blocker rather than discover it — a pendency recorded as
  closing one sprint by opening a hole in the next. The question was answerable
  now; nothing was waiting on evidence that did not yet exist.

## Consequences

**An obligation lands on the sprint that creates the manifest**, which is the
next one, and it is now fully specified. The `[patch.crates-io]` entry, the
vendored tree it points at, and the workspace exclusion are part of the
manifest's first version, not a later addition. Nothing about this decision is
left for that sprint to settle.

**No artefact built under the patch is the published crate.** A baseline
taken against it names an artefact that differs from `sqlx-core` 0.9.0 by one
statement, and `NFR-PERF-012` requires that to be visible in what is recorded.
Removing the patch when the fix ships changes the artefact again; the change
should be behaviour-neutral by construction, since the statement removed is the
statement the release adds. Under the vendored form that identity is cheap to
state: the commit of this repository names the patched source exactly, so a
baseline names the commit and the target and has named its artefact.

**The stall is not a corner case, and the record should not be read as covering
one.** `FR-CONF-013` defaults to `verify-identity`; four of its five modes can
negotiate TLS, and a connection that does negotiate it pays the floor without
the patch. Only the plaintext path is exempt, which is why the plaintext pair of
the driver selection was unaffected and why `ADR-003`'s choice does not rest on
the defect.

**The divergence is one statement; the tree is a whole crate.** That asymmetry
is what the vendored form buys and what it costs. Auditing the patch against the
published crate stays cheap — one statement, checkable with no network access —
and removing it when the fix ships is the deletion of a directory and of the
manifest lines that point at it, never a merge. What changes is everything that
reads this repository by walking it: the tree now contains third-party source,
so a repository-wide search, a code review, a licence review, and any inventory
of "the code in this project" have to treat that directory as what it is. Three
obligations follow, and none of them existed under the fork form:

- The copy SHALL preserve the upstream licence and notice files exactly as
  published. Vendoring moves source; it does not move it onto new terms.
- The prohibition on `unsafe` is a property of this project's own crate and does
  not reach the vendored tree. That source is the same third-party code it was
  when Cargo fetched it from the registry, and a copy of it sitting in the tree
  does not make it this project's to conform.
- The vendored crate is **not a new dependency** and does not enter the
  dependency budget as one. The dependency is still `sqlx` 0.9.0 under
  `ADR-003`; only where one of its crates is read from has changed.

Whether the knowledge graph covers the vendored tree, and on what terms, is a
question for the owner of that graph. It is raised here, not answered.

**One thing the manifest must confirm and this record cannot.** `cargo audit`,
command 5 of the mandatory validation pipeline, audits `Cargo.lock`. Whether it
resolves advisories through a `[patch.crates-io]` source is not established
anywhere in this repository, and the vendored form sharpens that question rather
than settling it: what the check must establish is whether the patched
`sqlx-core` reaches the audit at all, and if it does, under which identity, the
crate being read from a path rather than from the registry. It remains
unverifiable today for the reason it was unverifiable when this record was
written — there is no `Cargo.toml`, so there is no lockfile to look at — and it
stays marked unverified below rather than asserted either way. The sprint that
writes the manifest owes the check against a real lockfile. Whatever it finds,
the exposure is bounded and knowable without the tool: the vendored tree is
`sqlx-core` 0.9.0, an advisory against that version applies to it unchanged, and
the one added statement is the whole of the divergence.

**`ADR-003`'s open defect is closed by this record.** That record carried the
anomaly as unexplained, tracked as task `#8`, and described it as a property of
`musl` over raw loopback. It is none of those things: it is explained, it is a
regression in the pinned version rather than a property of the driver choice,
and it reproduces on every supported server series and over the bridge path as
well as loopback. `ADR-003`'s Consequences and Sources are amended in the commit
that creates this record, and it cites this one.

**Under R3, this fact lives here alone.** `BENCHMARKS.md` holds the mechanism,
the figures and the upstream chronology, and this record cites that file rather
than copying it; the decision, the options refused and the retiring condition
live here and are cited from elsewhere as `ADR-010` rather than restated. The
root coordination document needs no edit: its stack row already cites `ADR-003`,
and `ADR-003` now cites this record, so the path from that table to this
decision is one hop longer and no fact is duplicated. A citation of this record
in `docs/spec-technical/technology-stack.md` or in
`docs/spec-technical/open-decisions.md` is owed by the owner of those documents,
not by this register.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The mechanism: `sqlx-core` 0.9.0 never calls `set_nodelay`; `StdSocket` implements `io::Write` with `write` and `flush` only and inherits `std`'s default `write_vectored`, which writes the first buffer only; chained, they hold the `Finished` record for one Linux delayed ACK, `TCP_DELACK_MIN` being `HZ/25` | `BENCHMARKS.md`, "2026-09-11 — The TLS connect stall on Linux loopback", *What was established* and *Wire evidence* | 2026-09-11 |
| The measured effect of each candidate fix, the repeats, the protocol and the instrument's resolution — **not restated here**, per `BR-PERF-006` and the extension `ADR-003` makes of it | same entry, *Results — candidate fixes patched into `sqlx-core` 0.9.0* and *Results — phase instrumentation* | 2026-09-11 |
| `set_nodelay` and `write_vectored` are equivalent at this point: either alone removes the stall, and the two together are not measurably better than either alone | same entry, *Results — candidate fixes patched into `sqlx-core` 0.9.0* | 2026-09-11 |
| The floor is paid on all four supported server series, over loopback and over the docker bridge path, and is absent only on the macOS host path, where a userland proxy terminates the connection | same entry, *Independent of server series and of network interface* | 2026-09-11 |
| The regression chronology: present in 0.7.4 through 0.8.6, zero occurrences in 0.9.0, present again on `main`; dropped by the runtime rewrite; re-reported as issue `#4335` on 2026-07-10; fixed on `main` by PR `#4336`, merged 2026-08-17. Repository `https://github.com/transact-rs/sqlx` | same entry, *Whose defect it is, and its status upstream* | 2026-09-11 |
| `StdSocket` still lacks a `write_vectored` implementation on `main`, so the `rustls` flight still leaves split upstream | same entry, *What is not reported upstream* | 2026-09-11 |
| `sqlx` 0.9.0, published 2026-05-21, is still the maximum stable release; 0.8.6 was published 2025-05-19. No release carries PR `#4336` | crates.io index API, crate `sqlx`, `max_stable_version` and version list | 2026-09-11 |
| `sqlx::mysql::MySqlSslMode` declares the same five variants — `Disabled`, `Preferred`, `Required`, `VerifyCa`, `VerifyIdentity` — at 0.8.6 and at 0.9.0 | docs.rs, `sqlx` 0.8.6 and `sqlx` 0.9.0, `sqlx::mysql::MySqlSslMode` | 2026-09-11 |
| The cell-by-cell behaviour of the five modes was established against running servers on 2026-09-10, against the version `ADR-003` pins and against no other; the failure this guards against is silent and is not visible from an API listing | `ADR-002`, *Context*; `ADR-003`, *Alternatives rejected* | 2026-09-11 |
| The toolchain floor of 1.94.0 is declared by the crate at the version `ADR-003` pins, and moves with it | `ADR-007`, *Decision* | 2026-09-11 |
| Cargo reads `[patch]` settings only from the workspace-root manifest and ignores them in dependencies; a patch source may be a git repository pinned to a branch, tag or rev, or a local path | The Cargo Book, *Overriding Dependencies* | 2026-09-11 |
| All `path` dependencies residing in the workspace directory automatically become workspace members, and the `exclude` key prevents a path from being included | The Cargo Book, *Workspaces*, the `members` and `exclude` fields | 2026-09-11 |
| The measured candidate fixes were produced by vendoring `sqlx-core` 0.9.0 and redirecting to it with `[patch.crates-io]` and a `path` — the form this record prescribes | `BENCHMARKS.md`, "2026-09-11 — The TLS connect stall on Linux loopback", *Reproduction* | 2026-09-11 |
| `cargo audit`, command 5 of the mandatory validation pipeline, audits `Cargo.lock` | `docs/spec-technical/operations.md`, the pipeline table, row 5, which cites the crates.io index and rustsec.org for `cargo-audit` 0.22.2 | 2026-09-11 |
| Whether `cargo audit` resolves advisories through a `[patch.crates-io]` source, and under which identity a path-sourced crate reaches it | **Unverified.** Nothing in this repository establishes it, and no claim is made either way. It cannot be settled while there is no `Cargo.toml` and therefore no lockfile; the sprint that writes the manifest owes the check | — |
| Whether a `[patch.crates-io]` entry whose source is a path inside the workspace directory is itself made a workspace member by the rule above | **Unverified.** The Cargo Book states that rule for `path` dependencies and says nothing about patch entries; the workspace exclusion this record prescribes makes the question moot | — |
