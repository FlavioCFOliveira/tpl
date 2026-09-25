---
title: tpl Technical Specification
status: draft
last-reviewed: 2026-09-25
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

Every document listed below is **written**. A document not yet written would be
listed as *waiting*, where it waits on a named entry of `open-decisions.md`, or
as *unblocked*, where no entry stands against it; none is either today.

As of 2026-09-25 one entry of `open-decisions.md` is open — `OD-33`, whether
`UC-013` joins the acceptance skeleton — and **no document waits**: what it
blocks is a statement in a written document, not a document. Every other entry
is settled. `OD-30`, the one entry that was settled and **interim**, was
discharged on 2026-09-22, when the last leaf of the tree was implemented.

**The folder was audited on 2026-09-25 against the sixty-fourth edition of
`/specification` and the code at `c360c80`**, for rmp `#308`. Every internal
link and anchor resolves. Corrected:

- the status column of [The documents](#the-documents), which listed seven
  written documents as unwritten, and the count of auxiliary command groups,
  four here against five in `overview.md` and `FR-CLI-010`;
- `OD-30`, discharged on 2026-09-22, and the four documents that still cited
  its arrangement as standing;
- the reading of `.tpl/.cfg` in `architecture.md`, `interfaces.md` and
  `security.md`, which still put the trust checks before the open and passed
  an absent file without the folder check of `FR-PROJ-028`;
- the count of the suite in `verification.md`, re-read at `c360c80`;
- the module map in `architecture.md` and `OD-05`, which lacked `at.rs`, and
  the `rustix` calls in `technology-stack.md`, which lacked `open`, `fstat` and
  `statat`;
- in `traceability.md`, the count of writers inside `.tpl`, and what the
  mapping covers and how far behind it is;
- the entry of `DIV-005` in `data-model.md`, discharged since it was cited. `verification.md`'s passage on
what `NFR-PERF-007` reaches was already settled, so a report that it still
raised the question was stale.

**Changes of 2026-09-24**, by task:

- **`#292`, `#293`, `#294`, `#267`, `#299`, `#303` — distribution.**
  `operations.md` replaces its no-pipeline section with
  [Continuous integration and release](operations.md#continuous-integration-and-release)
  and cites [`ADR-012`](../adr/adr-012-ci-and-release-distribution.md) for the
  two workflows started by `workflow_dispatch` only, the CI toolchain, the
  pinned and hash-checked tools, the Darwin `tar` flags, the provenance gates
  and the pre-release path, the ignored measurement tests, the skill archive,
  `install.sh` and `install-skill.sh`; it cites `ADR-008` for the build path
  alone. `open-decisions.md` amends `OD-23`, and `OD-03` names Semantic
  Versioning 2.0.0 and sets the binary version to `0.0.1`, as does
  `data-model.md`. `technology-stack.md` cites `ADR-012`.
- **`#288`, `#302`, `#305` — the cache and a link.** `data-model.md` records the
  1 MiB bound `FR-CDOC-017` leaves to this folder, the `78` refusal of a link
  on the path to the cache, and a write row scoped to object files;
  `security.md` adds both to `cache/`'s disposition of a symbolic link and
  completes its recorded gap on the cache. Against the fifty-eighth edition's
  amendment of `FR-PROJ-023`, `data-model.md`, `traceability.md` and this file
  count five writers inside `.tpl`.
- **`#257` — the library entry point.** `interfaces.md` gains
  [The library entry points](interfaces.md#the-library-entry-points), and
  `open-decisions.md` gains `OD-34`, which holds the rejected options of
  `run_from`.
- **`#230` — what the instruments reach.** `verification.md` records as settled
  by the sixty-second edition that `NFR-PERF-007` reaches exactly the six
  requirements of form, and that `NFR-PERF-008` is verified against the
  server's statement record; `quality-attributes.md`, `operations.md` and
  `traceability.md` state that verification, and `verification.md`'s register
  names the test that makes it.
- **`#306`, `#307` — directory-relative file operations.** `open-decisions.md`
  amends `OD-24` with `rustix`'s `fs` feature, reverses `OD-10`'s rejection of
  `O_NOFOLLOW`, and records the correction to `CLAUDE.md`, applied the same
  day; `OD-15` amends its last step. `technology-stack.md` lists the four `fs`
  calls; `data-model.md` describes the cache's path operations and the
  `.tpl/.cfg` open as implemented in `src/at.rs`, with the one listing
  residual and the swap evidence; `security.md` states how a template is read.

**The whole folder was swept on 2026-09-23 against the forty-second edition of
`/specification`, `ADR-011`, and the code of sprint 20** in the working tree
over HEAD `d89ffc4`. Eleven documents changed; `overview.md` did not.
`architecture.md` gains [The render bounds](architecture.md#the-render-bounds),
which **fixes the render memory limit's poll interval at 10 ms** under
`ADR-011`'s delegation, and states that the connection and the driver's runtime
end before any render. `interfaces.md` gains the three contracts behind the
bounds, the process group of the `password_command` child, and the dangling
reference check of `FR-CTX-042`; `data-model.md` the eighteen key forms and
two cache read guards; `security.md` the render bounds with their residuals;
`technology-stack.md` `cap`, the engine's `fuel` feature, and two further
`rustix` calls. `open-decisions.md` amends `OD-05`, `OD-10`, `OD-11` and
`OD-12`; `OD-10`'s collision outcome is superseded by the user's decision for
rmp `#254`, under `FR-CACHE-033`. `traceability.md` harvests the
edition out of order; `verification.md`, `quality-attributes.md` and
`operations.md` follow.

**The whole folder was swept on 2026-09-22 against the thirty-sixth edition of
`specification/performance-requirements.md`**, which withdrew the performance
gates on the product owner's decision and retired four identifiers with them.
Six documents carry substantive change and four carry a repaired cross-reference
or a renamed term. `quality-attributes.md` is restated whole: the nine are a
**measurement set** of measurement points rather than a table of limits, the
requirements of form are set out as the correctness invariants that survived,
and the first campaign — all nine points and the `WL-002` scalar, at full
protocol, on one of the four targets — is recorded as a standing.
`operations.md` loses the no-regression release gate and the benchmark step of
the validation pipeline, and gains the measurement harness. `verification.md`,
`traceability.md`, `open-decisions.md` (`OD-20`, `OD-27`) and this file follow.
**No retired identifier and no word of the withdrawn vocabulary survives in this
folder.** Two contradictions this folder had recorded are discharged: the one
against `BR-PERF-007`, by commit `455e48d` on 2026-09-21, and the one between
`quality-attributes.md` and `traceability.md` §23, by restating that section
here.

**One obligation was discharged on 2026-09-21**, when the render work was built:
`OD-14`'s owed observation, about a **defined** `null` under the engine's strict
undefined-behaviour setting. The answer was the opposite of the expected one, so
the entry records what carries the two requirements the setting turned out not
to decide, and `architecture.md` states it rather than declining to.

**One entry was added and two were amended on 2026-09-21**, when the backlog was
cleared. `OD-32` removes `anyhow` from the dependency graph and prepared the
correction `CLAUDE.md` was owed for it, which the user applied on 2026-09-22;
`technology-stack.md` lost the crate's two rows and cites the entry for the
removal. `OD-12` is discharged of the phase attribution it owed `verification`,
with three corrections against what the connection work observed. `OD-20` is
narrowed to how the edit distance is computed, the measure itself now being
`FR-ERR-039`'s.

**Four documents were re-audited against the tree on 2026-09-21**, in the same
clearance. `verification.md` was re-measured whole against the suite at commit
`243c4d6` — the count, the per-file breakdown, which tests reach a server, which
register rows are written, and which flows the tree can reach. `traceability.md`
harvested editions ten to twenty of `/specification` and records the gap that
remains, which is editions twenty-three to thirty-two. `open-decisions.md` was
re-read whole against `CLAUDE.md` at commit `8f936d4`, as its own convention
requires, and now records every such re-read with its commit and date.
`architecture.md` and `data-model.md` lost the statements earlier sprints had
made false. `OD-27` is discharged: `scripts/mariadb/seed-bench.sql` exists, so
`quality-attributes.md`, `operations.md` and `verification.md` no longer wait on
it.

**One entry was added on 2026-09-18, against what the model work built.**
`OD-31` answers four of the five library-shape questions `DIV-032` hands to
architecture, over the model's types and no others. Three settled entries were
re-read against the same build the same day: `OD-05` gains the placement of the
document submodule under `model/`, `OD-18` gains two amendments — the count of
the omission attribute, and the four emitted shapes that are not the model's own
types — and `OD-19` gains an observation now owed to `adr-guardian`, against the
word its record uses for what an embedding site holds.

**Three settled entries were amended on 2026-09-17, against what the project and
configuration work built.** `OD-05` records that the read path and the write
path over `.tpl/.cfg` are two modules rather than two paths of one; `OD-09`
records that the read path is the TOML document tree and not a `serde` derive,
with the three requirements a derive cannot answer; `OD-12` records that the
`password_command` child is bounded by a reader thread and a polling loop rather
than by a timer thread, and why the asynchronous runtime is unavailable on that
path. Each is an amendment to a settled entry, not a reopening: the decision
each entry made stands, and what changed is the mechanism recorded under it.

The one residual that was work rather than a decision — `OD-22`'s fixture
certificate and its container harness — was discharged on 2026-09-11 by tasks
#15 and #25, and `operations.md` and `verification.md` were written against what
those tasks produced.

## The four sources of truth

The project has four, with scopes that do not overlap.

| Source | Answers | Owner |
|---|---|---|
| `/specification` | **What** `tpl` does — requirements, rules, use cases | `specification-manager` |
| `rmp` | **When and by whom** — sprints, tasks, state | `roadmap-manager` |
| Knowledge Graph | **Where** — what code exists, how it articulates, and which requirement each component satisfies | `knowledge-authority` |
| `docs/spec-technical/` | **How** the system is built — architecture, interfaces, data, security, operation, quality | `technical-writer` |

The coordination file `CLAUDE.md` carries the same four rows, applied on
2026-09-11 with the user's authorisation and recorded in
[open-decisions.md](open-decisions.md#od-26--the-boundary-against-the-knowledge-graph):
the fourth row was added, and the graph's row narrowed from *"Onde e
**como**"*, which overlapped this folder's scope. `CLAUDE.md` also governs
[`docs/adr/`](../adr/README.md) — who writes a record and when one is required
— which `OD-26` does not cover.

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
| `open-decisions.md` | The decision register: thirty-three settled entries, each carrying its rejected options or citing the record that holds them, and one open — `OD-33` — carrying its options and its owner; the obligations that survive settlement with their owners; and any conflict owed to the functional owner — there is none today | written |
| `overview.md` | What the built system is, its boundaries, what it is not, and the limits it does not overcome | written |
| `architecture.md` | Components, responsibilities, interactions, the invocation pipeline, the module map | written |
| `technology-stack.md` | Each technology: version, purpose, why chosen, what was rejected, source consulted; the dependency budget | written |
| `interfaces.md` | The contracts crossing a component boundary, and how each external contract is realised | written |
| `data-model.md` | The model in memory and everything persisted: `.tpl/.cfg`, `.tpl/.cache/`, `meta.json`, versions, migration | written |
| `security.md` | Trust boundaries as implemented, secrets, transport, containment, the structural prohibitions | written |
| `operations.md` | Build, target matrix, packaging, release gates, observability | written |
| `quality-attributes.md` | Performance and reliability targets, and how each is measured | written |
| `verification.md` | Test strategy, harness, fixture, the mandated-test register, the test seams | written |
| `decisions.md` | Retired. The register and its index live at [`docs/adr/`](../adr/README.md) | n/a |

`glossary.md` is **not** proposed. `specification/glossary.md` fixes the
vocabulary and states that a term used without being defined there is a defect;
a second glossary earns its place only if this folder introduces terms of its
own, and it would then hold those alone.

### `overview.md`

**Answers.** What `tpl` is as a built artefact: three arms and five auxiliary
command groups; read-only over the database; one render per invocation; no
file-writing surface. The primary consumer and its three channels. The five
catalogue features excluded by decision. The three limits the system states
rather than overcomes — an impostor server, hidden triggers, non-exclusive
trust material. That the library API is not a public surface.

**Must not contain.** Component detail, technology, versions, or any
requirement text.

### `architecture.md`

**Answers.** The eight ordered steps of the invocation pipeline and the four
commands that skip two of them. Step 1 in its three parts, and the division of
`cli/` by subject rather than by command. Project discovery, its mount-point
boundary, and the trust checks. Configuration resolution across two layers and
a built-in default. The connection lifecycle: at most one, opened late, five
ordered stages in the order `FR-SRV-042` fixes, closed when the read ends. The
catalogue reader and the query-count invariants. The cache as a read-through layer. The model as
the single junction of three sources and three consumers. The render component:
engine construction, loader, undefined behaviour, the output formatter,
auto-escape, context assembly. Deadlines on six named blocking phases. The four
render bounds, their composition, and the poll interval of the memory limit.
Lazy initialisation. The
synchronous process and the runtime boundary inside `mariadb/`. The module map
and the layout conventions. The division inside `project/` and the four
decisions that produce it: reading apart from resolving, reading apart from
writing, the environment as a parameter, and a credential as a type. The
division inside `model/`, the four things that block does not do, the four
construction choices it makes, the document in both directions, and the
requirements of the two files it is built from that it does not answer.

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
its six exceptions. The help surface as built: the two sources the renderer
reads, the five facts of a flag it introspects and the sixth it does not, the
six forms that reach one renderer, and the runtime-introspected command tree
with the two shapes of its document. The template surface as registered.
Context access from a filter or a test. The configuration reader and writer, the
one predicate that decides an entry's coherence for both of them, and the two
codes its two callers produce. The `password_command` child, its process
group, and why its parent polls. The phase clock's four obligations. The three
contracts behind the render bounds. The library entry points, and what an
in-process caller of `run_from` must know. The pattern matcher and the
qualified-routine-name parser. The two directions over the document: which
emitted types are the model's own and which four are not, the two projections,
the one flattening, what the read-back checks, and the four things it does not. The five
library-shape questions the functional specification hands to architecture, and
where each is now answered.

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
does not promise. The five writers inside `.tpl`.

**Must not contain.** The JSON document contract, which
`specification/context-document.md` owns and this file cites.

### `security.md`

**Answers.** The six untrusted inputs and the component that first sees each.
Credential handling, the type that denies a credential any way of being printed,
and the three read paths over `.cfg`. The six categories
never written to a diagnostic stream. The sentinel property. `${VAR}` expansion
and its ordering inside a DSN. The child process. The discovery boundary,
canonicalisation, ownership and mode checks. Template containment. The
injection surfaces. Transport, including the mode set explicitly on every
connection, the trust material that widens rather than narrows, and how a
directory of trust material is resolved and when it refuses. The
read-only promise in its two parts. The four template capability prohibitions.

**Must not contain.** A second statement of any rule. Each entry cites its
owning requirement, as `BR-SEC-001` requires of the functional file it mirrors.

### `operations.md`

**Answers.** The four targets, the static `musl` linkage and its observable
consequence. The mandatory validation pipeline. The release profile. MSRV, the
toolchain and tools as installed on the development host, and the cross-build
path for the musl targets. The measurement harness, and why it can enforce
nothing. The release gates. The four version numbers and what a breaking change
is. Observability. The content `tpl init` ships. The fixture as an operational
asset. The continuous-integration and release workflows, their toolchain and
tool pins, and the installer, by citation of `ADR-012`.

**Must not contain.** Measured figures, which are `BENCHMARKS.md` — including
the toolchain a measurement was taken under, which is part of that record's
environment and is not the development toolchain; the measurement points and
the protocol, which are `quality-attributes.md`.

### `quality-attributes.md`

**Answers.** The six requirements of form, the design decisions each forces, and
why they are the part of this subject that can fail anything. Determinism over
stdout, and the single source of non-reproducibility. The nine measurement
points, what each is measured over and with what cache posture, the distinction
between an adopted and a recorded figure, and what has been measured so far and
on which target. The measurement protocol. The three reference workloads. The
memory consequence of materialising the two foreign-key directions. The failure
path as a measurement point. The reliability properties. Cross-series
equivalence and its exceptions.

**Must not contain.** Measured figures, which are `BENCHMARKS.md`; the container
harness, which is `verification.md`; the measurement harness, which is
`operations.md`. No gate on a figure: `BR-PERF-008` leaves none to describe.

### `verification.md`

**Answers.** The register of mandated tests, each traced to the requirement
that mandates it. The published test vectors that are specification and test at
once. The properties verified from outside the process, the four instruments
that verify them and the targets each is used on. Container orchestration
across four series and the fifth container that is not a fifth series. The reduced-privilege reader and the three
shapes of absence it produces. The fixture's stated gaps and what they bound.
The two test seams that must not reach the published surface. Help snapshots at
every depth, and which tests of them exist. The parser mapping, one test per
kind. The invocation surface observed on the process. The dump round-trip. The
twelve end-to-end flows. Test naming by requirement identifier, and the suite's
divergence from it.

**Must not contain.** Targets or measurement points, which are
`quality-attributes.md`; the build pipeline and the measurement harness, which
are `operations.md`.

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
- **A statement this folder makes about a file it does not own is re-read
  against that file whenever that file changes**, in each of the three forms
  such a statement takes: a correction recorded as **owed**, a **quotation**,
  and a **summary of a register entry** about that file. An entry found
  discharged records the date and the commit that discharged it. Adopted
  2026-09-11 from the fifth validation rule of
  [`specification/README.md`](../../specification/README.md), unchanged in
  substance. The reason is that
  [open-decisions.md](open-decisions.md#corrections-owed-to-claudemd) is the
  same kind of register and decayed the same way: three entries still asked for
  a correction to `CLAUDE.md` that commit `ee7363d` had applied earlier the
  same day. The trigger is the **edit to the target file**, never an edition of
  this folder — what decays is outside this folder and decays when somebody
  else edits it, so no check this folder can run on itself would find it. The
  commit is the evidence: an entry that says *when* it stopped being owed can
  be audited, where one that merely stops being listed leaves the next reader
  to re-derive it.
- **Widened on 2026-09-11 from corrections to statements, with a form rule that
  makes most of the re-reading unnecessary.** A third sweep, recorded in
  [open-decisions.md](open-decisions.md#corrections-owed-to-claudemd), found
  eleven stale passages: three restate what an entry of `/specification`'s
  divergence register still owes, which the first wording reached, and eight
  summarise such an entry faithfully, claim no correction at all, and still
  assert in the present indicative what `CLAUDE.md` contains. It is one
  obligation seen from each end — a correction says what this folder will do, a
  summary says what the other file is like, and both go false on the same edit
  — so the rule is widened rather than doubled. The form rule follows: **state
  the technical concern or the register entry, never the content of the file
  that entry corrects.** A concern drawn from an entry does not move when the
  target file is edited, so it never needs re-reading; the file's content does,
  and a present-tense summary of it decays out of sight. Where an entry's state
  must be named, name the state and not the file — a discharge is terminal, and
  a divergence raised again takes a new identifier.
