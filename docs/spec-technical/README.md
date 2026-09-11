---
title: tpl Technical Specification
status: draft
last-reviewed: 2026-09-11
related: [traceability.md, open-decisions.md]
---

# tpl Technical Specification

This folder is the single source of truth for **how** `tpl` is built. Every
statement in it traces to a requirement of `/specification` or to an explicit
constraint, and cites that trace. Nothing here is a requirement: where this
folder and `/specification` disagree, `/specification` governs and the
difference is a defect to be reported.

## Where to begin

1. This file, for the shape of the set and what each document owns.
2. [traceability.md](traceability.md), for the functional needs each document
   answers, cited to requirement identifiers.
3. [open-decisions.md](open-decisions.md), for what is settled, what is open,
   and who owns each open point.

A document listed below as *waiting* does not exist yet, and it waits on a
named entry of `open-decisions.md`, or on a residual inside an entry that is
otherwise settled. It waits deliberately: writing it around an unsettled point
would produce a technical statement with no trace, which this folder does not
admit. A document listed as *unblocked* has no entry against it and has simply
not been written.

As of 2026-09-11 every entry of `open-decisions.md` is settled, so seven of the
nine unwritten documents below are unblocked. Two still wait, on the two halves
of the one residual that is work rather than a decision: `OD-22`'s fixture
certificate and its container harness.

## The four sources of truth

The project has four, with scopes that do not overlap.

| Source | Answers | Owner |
|---|---|---|
| `/specification` | **What** `tpl` does — requirements, rules, use cases | `specification-manager` |
| `rmp` | **When and by whom** — sprints, tasks, state | `roadmap-manager` |
| Knowledge Graph | **Where** — what code exists, how it articulates, and which requirement each component satisfies | `knowledge-authority` |
| `docs/spec-technical/` | **How** the system is built — architecture, interfaces, data, security, operation, quality | `technical-writer` |

The fourth row is new. The coordination file `CLAUDE.md` currently names three
sources and assigns *"Onde e **como**"* to the knowledge graph, which overlaps
this folder's scope. The correction is **prepared for the user's approval and
has not been made**: `CLAUDE.md` is coordination and is never edited
unilaterally. The two rows it needs are recorded in
[open-decisions.md](open-decisions.md#od-26--the-boundary-against-the-knowledge-graph),
in the language of the file they are destined for.

## The architecture is carried twice

The knowledge graph represents the component architecture, the application
flows, and requirement-satisfaction edges, so that it can answer *which
component satisfies `FR-X`* and *what breaks if I change this*. The same
architecture is carried here as prose. **The two must not drift.**

| Question | Answered by | Why |
|---|---|---|
| What the architecture **shall** be, and why | this folder | It prescribes, and it traces every decision to a requirement or a constraint |
| What the code **is**, and which component satisfies which requirement | the knowledge graph | It describes what exists, and a fact in it does not legitimise code the specification never asked for |
| The two disagree | **Stop and report both readings** | A prescription and a description that differ are either an unimplemented decision or an undocumented implementation. Neither is corrected silently |

A graph fact is never cited here as authority for a decision, and a decision
here is never recorded as though the code already realised it.

## The documents

Identifier-level citations for every row below live in
[traceability.md](traceability.md); this table names the areas, not the
requirements.

| File | Answers | Status |
|---|---|---|
| `README.md` | The index: the four sources of truth, the two carriers of the architecture, what each document owns | written |
| `traceability.md` | The functional-to-technical mapping, and the reverse mapping from every `specification/` file | written |
| `open-decisions.md` | The decision register: twenty-eight settled entries, each carrying its rejected options or citing the record that holds them, the obligations that survive settlement with their owners, and any conflict owed to the functional owner — there is none today | written |
| `overview.md` | What the built system is, its boundaries, what it is not, and the limits it does not overcome | unblocked |
| `architecture.md` | Components, responsibilities, interactions, the invocation pipeline, the module map | unblocked. `OD-14`'s owed observation bounds one sentence: the behaviour of a **defined** `null` under `UndefinedBehavior::Strict` may not be asserted until it is verified |
| `technology-stack.md` | Each technology: version, purpose, why chosen, what was rejected, source consulted; the dependency budget | unblocked |
| `interfaces.md` | The contracts crossing a component boundary, and how each external contract is realised | unblocked |
| `data-model.md` | The model in memory and everything persisted: `.tpl/.cfg`, `.tpl/.cache/`, `meta.json`, versions, migration | unblocked |
| `security.md` | Trust boundaries as implemented, secrets, transport, containment, the structural prohibitions | unblocked |
| `operations.md` | Build, target matrix, packaging, release gates, observability | waiting on the residual of `OD-22` — the fixture certificate |
| `quality-attributes.md` | Performance and reliability targets, and how each is measured | unblocked |
| `verification.md` | Test strategy, harness, fixture, the mandated-test register, the test seams | waiting on the residual of `OD-22` — the harness |
| `decisions.md` | Retired. The register and its index live at [`docs/adr/`](../adr/README.md) | n/a |

`glossary.md` is **not** proposed. `specification/glossary.md` fixes the
vocabulary and states that a term used without being defined there is a defect;
a second glossary earns its place only if this folder introduces terms of its
own, and it would then hold those alone.

### `overview.md`

**Answers.** What `tpl` is as a built artefact: three arms and four auxiliary
command groups; read-only over the database; one render per invocation; no
file-writing surface. The primary consumer and its three channels. The five
catalogue features excluded by decision. The three limits the system states
rather than overcomes — an impostor server, hidden triggers, non-exclusive
trust material. That the library API is not a public surface.

**Must not contain.** Component detail, technology, versions, or any
requirement text.

### `architecture.md`

**Answers.** The eight ordered steps of the invocation pipeline and the four
commands that skip two of them. Project discovery, its mount-point boundary,
and the trust checks. Configuration resolution across two layers and a built-in
default. The connection lifecycle: at most one, opened late, probe then
read-only set then read-back, closed when the read ends. The catalogue reader
and the query-count invariants. The cache as a read-through layer. The model as
the single junction of three sources and three consumers. The render component:
engine construction, loader, undefined behaviour, auto-escape, context
assembly. Deadlines on six named blocking phases. Lazy initialisation. The
synchronous process and the runtime boundary inside `mariadb/`. The module map
and the layout conventions.

**Must not contain.** Versions or crate rationale, which are
`technology-stack.md`; signatures, which are `interfaces.md`; on-disk shapes,
which are `data-model.md`.

### `technology-stack.md`

**Answers.** Every technology with its exact version, its purpose here, why it
was chosen, what was rejected, and the official source consulted with the date.
The template engine and its pin. Which engine built-ins are guaranteed, which
are shadowed by a registration of a different signature, and which carry no
guarantee. The database driver, already decided on measured evidence. The five
TLS modes as a criterion that disqualifies a driver. The argument parser and
the default features that must be off. The TOML read and write paths.
Serialisation. The error crates. Diagnostics. MSRV, edition, and the release
profile. The dependency budget, and the consequence of forbidding `unsafe`.

**Must not contain.** How a component uses a crate, which is
`architecture.md`; any measured figure, which is `BENCHMARKS.md`.

### `interfaces.md`

**Answers.** The catalogue reader's surface and how a short read is reported.
The three privilege cross-checks. The error type, its exit-code derivation, and
the per-code obligation on the `cause` line. The diagnostic renderer, its
escaping, and suggestion selection. The document emitter: one envelope,
seventeen payload shapes, fixed key order, the two omissions, the compact and
indented forms, C0 escaping, lossy UTF-8, the pipe state. The ordering rule and
its six exceptions. The help surface and the runtime-introspected command tree.
The template surface as registered. Context access from a filter or a test. The
configuration reader and writer. The `password_command` child. The pattern
matcher and the qualified-routine-name parser. The five library-shape questions
the functional specification hands to architecture.

**Must not contain.** Command-line or JSON syntax, which `/specification` owns
and this file cites; persisted shapes, which are `data-model.md`.

### `data-model.md`

**Answers.** The covered object kinds as a predicate, and its application to
objects and to columns. The per-kind property lists and the rule that derives
one from a catalogue field list. The two closed exclusion lists. The type
decomposition and the raw-string safety net. The default discriminant and its
ordered classification. The `database` object. The four treatments of a
cross-series difference. `.tpl/.cfg`: format, key space, mode, rewrite
discipline. `.tpl/.cache/`: layout, keying, encoding, filenames, atomic write.
`meta.json` and its two independent versions. Migration. What a cached document
does not promise. The four writers inside `.tpl`.

**Must not contain.** The JSON document contract, which
`specification/context-document.md` owns and this file cites.

### `security.md`

**Answers.** The six untrusted inputs and the component that first sees each.
Credential handling and the three read paths over `.cfg`. The six categories
never written to a diagnostic stream. The sentinel property. `${VAR}` expansion
and its ordering inside a DSN. The child process. The discovery boundary,
canonicalisation, ownership and mode checks. Template containment. The
injection surfaces. Transport, including the mode set explicitly on every
connection and the trust material that widens rather than narrows. The
read-only promise in its two parts. The four template capability prohibitions.

**Must not contain.** A second statement of any rule. Each entry cites its
owning requirement, as `BR-SEC-001` requires of the functional file it mirrors.

### `operations.md`

**Answers.** The four targets, the static `musl` linkage and its observable
consequence. The mandatory validation pipeline. The release profile. MSRV, the
toolchain and tools as installed on the development host, and the cross-build
path for the musl targets. The release gates. The four version numbers and what
a breaking change is. Observability. The content `tpl init` ships. The fixture
as an operational asset. That no continuous-integration pipeline is prescribed.

**Must not contain.** Measured figures, which are `BENCHMARKS.md` — including
the toolchain a measurement was taken under, which is part of that record's
environment and is not the development toolchain; budgets, which are
`quality-attributes.md`.

### `quality-attributes.md`

**Answers.** The six requirements of form and the design decisions each
forces. Determinism over stdout, and the single source of
non-reproducibility. The nine budgets, the one normative among them, and the
distinction between a provisional and a ratified figure. The measurement
protocol. The three reference workloads. The memory consequence of
materialising the two foreign-key directions. The failure path as a budget. The
reliability properties. Cross-series equivalence and its exceptions.

**Must not contain.** Baselines, which are `BENCHMARKS.md`; the harness, which
is `verification.md`.

### `verification.md`

**Answers.** The register of mandated tests, each traced to the requirement
that mandates it. The published test vectors that are specification and test at
once. The properties verified from outside the process. Container
orchestration across four series. The reduced-privilege reader and the three
shapes of absence it produces. The fixture's stated gaps and what they bound.
The two test seams that must not reach the published surface. Help snapshots at
every depth. The dump round-trip. The twelve end-to-end flows. Test naming by
requirement identifier.

**Must not contain.** Targets or budgets, which are `quality-attributes.md`;
the build pipeline, which is `operations.md`.

### `decisions.md`

**Retired, and not written.** The architecture decision register lives at
[`docs/adr/`](../adr/README.md), and `docs/adr/README.md` carries both the
convention and the authoritative index. A `decisions.md` here could hold nothing
that is not already in a record or in [traceability.md](traceability.md), and a
third carrier of one mapping is the duplication rule R3 of the register exists to
prevent.

A settled decision's rationale lives in its record and this folder cites it by
`ADR-NNN`. Where no record exists yet, the rationale is carried in
[open-decisions.md](open-decisions.md), which is why that file records the
rejected options as well as the choice.

## Conventions

- English. Requirement identifiers are the citation unit, as
  `specification/README.md` fixes them, including in test names.
- The rule on sourcing a factual claim about a technology is the register's,
  recorded in [`docs/adr/README.md`](../adr/README.md). It governs this folder
  unchanged, and is cited here rather than copied.
- Each fact lives in one place and is cross-referenced elsewhere. A second copy
  is the copy that stops being true without saying so.
- A contradiction between this folder and the code, or between this folder and
  `/specification`, is reported with both readings. It is never closed by
  revising this folder to match.
