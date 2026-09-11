---
title: Architecture Decision Records
status: approved
last-reviewed: 2026-09-11
related: [../spec-technical/README.md, ../spec-technical/open-decisions.md]
---

# Architecture Decision Records

This folder is the project's **architecture decision register** — the place two
requirements in force name as the home of a fact the functional corpus refuses
to hold:

- `FR-ENV-003` (`specification/template-environment.md`): the template-engine
  pin "SHALL be recorded in the project's architecture decision records, and
  SHALL be cited from there by this requirement rather than restated in this
  specification".
- `FR-CONF-038` (`specification/configuration-model.md`): the mapping of the
  five TLS modes onto the driver "SHALL be recorded in the project's
  architecture decision records and cited from there … and SHALL NOT be
  restated in this corpus".

Both cite this register **by role and not by address**. Neither names a path, a
filename, or a record number, and neither ever will. That is deliberate, and
rule R1 below is what preserves it.

A third requirement delegates in a weaker form. `FR-ERR-030`
(`specification/errors-and-exit-codes.md`) states that which mechanism produces
the `70` outcome "is an architecture decision, and this corpus names none". It
hands the fact out of the corpus without naming this register as its home, so it
is not a delegation R1 constrains — but it is the delegation `ADR-004` answers,
and a reader arriving from that requirement resolves it through the index
below.

## This is not a fifth source of truth

The project has four sources of truth, and this folder does not add one. It is
the **rationale layer of the fourth**. `docs/spec-technical/` prescribes *how*
the system is built; this register records *why that and not the alternatives*,
for the subset of decisions admitted by rule R4.

Stated plainly so that nobody has to infer it: a record here is subordinate to
`/specification` in exactly the way `docs/spec-technical/` is subordinate to it,
and a record here never becomes the authority for a requirement.

## Location and file naming

- Records live in this folder, one file per record.
- File name: `adr-NNN-<kebab-slug>.md`, with a three-digit zero-padded
  sequence — `adr-001-template-engine-pin.md`.
- Three digits, matching the width of the corpus's own citation unit
  (`FR-ENV-003`, `FR-CONF-038`). A third identifier width in one repository
  would buy nothing.
- The slug names the decision's **subject**, never its outcome:
  `adr-001-template-engine-pin`, not `adr-001-pin-minijinja-2-24`. An outcome
  can change while the subject stays, and a slug naming the outcome becomes a
  lie the filename tells after the first amendment.

## Numbering

- `ADR-NNN` is the **citation unit**, and the only one. The slug is a human
  affordance and is not part of the identifier, so renaming a slug cannot break
  a citation.
- Numbers are allocated from the folder listing, strictly monotonic.
- A number is **never reused and never renumbered**, including for a superseded
  or withdrawn record.

## Record format

YAML front matter, then six sections.

```
---
id: ADR-NNN
title: <the decision's subject>
status: proposed | accepted | superseded | withdrawn
decided: YYYY-MM-DD
last-reviewed: YYYY-MM-DD
requirements: [<identifiers this record serves>]
supersedes: []
superseded-by: null
---

# ADR-NNN — <Title>

## Status
## Context
## Decision
## Alternatives rejected
## Consequences
## Sources
```

- **Status** — one lifecycle value with its date; for a superseded record, the
  successor's identifier.
- **Context** — the forces, and the requirements that motivate the decision,
  cited by identifier. Where a requirement in force *delegates* the fact out of
  the corpus, the record says so and names it. That sentence is what makes
  `FR-ENV-003` and `FR-CONF-038` satisfiable.
- **Decision** — the decision in force, present tense, unambiguous. Version
  pins and named driver variants appear here and nowhere else in the
  repository.
- **Alternatives rejected** — a first-class section. The rejection is the part
  a later reader needs, because a decision without its alternatives reads as
  the only thing anybody thought of.
- **Consequences** — trade-offs, the impact on the project's first-order
  principles, and what the code must or must not do as a result.
- **Sources** — one row per source: the claim, the exact locator, and the date
  consulted, or an explicit `unverified` mark. Claims are cited inline in the
  body; the locator and the date are carried here once, so that no date is
  duplicated and the audit is cheap.

The `requirements:` field makes the reverse index machine-derivable, which
matters for a project whose primary consumer is an agent reading the repository
programmatically.

## Status lifecycle

| Status | Meaning |
|---|---|
| `proposed` | Written, not yet approved. **Not authority**: nothing may cite it as settled and no code may be written against it |
| `accepted` | Approved and in force. The only status that can serve as the home of a fact a requirement delegates |
| `superseded` | No longer in force. A named successor holds the decision |
| `withdrawn` | Never in force, or made moot with no successor. The number is retired, not reused |

### What supersession does to the old record

- The successor is a **new record with a new number**. The old record's
  Decision is never edited into the new one.
- The old record is **reduced to a stub**: front matter `status: superseded`
  and `superseded-by: ADR-NNN`, plus two or three lines saying what it decided
  and pointing at the successor. Its Context, Alternatives, Consequences and
  Sources are deleted — the successor restates whatever still applies, and the
  repository history holds the rest.
- **Every citation of the old record is repointed to the successor in the same
  commit.** A superseded record with a live citation is a defect.
- After supersession the old record's content is authority for nothing. A
  reader who lands on it is bounced, not informed.

### Supersession against editing in place

Supersession is for a decision that **changed**. Editing in place is for a
decision that was recorded imprecisely, or whose **parameter** moved.

Moving the engine pin from one stable line to the next is an **edit in place**
with a new `decided` date: the decision — pin to a stable line, and re-check
the fourteen names of `FR-ENV-018` — is unchanged, and only its parameter
moved. `FR-ENV-003` requires exactly this, in the words "recorded in the same
place". Abandoning the bundled root store for the platform trust store would be
a genuine supersession, because the decision itself would have changed.

Getting this backwards produces either a register of near-identical records or
a register that hides its own changes.

## The four rules

These four are the whole anti-drift mechanism. `specification/upstream-divergences.md`
exists because two documents held one fact with neither declared subordinate;
R2 declares subordination and R3 removes the second copy.

**R1 — The corpus cites by role; the register cites by identifier.** A
requirement says "the project's architecture decision records" and never names
a path, a filename, or a record number. A record names the requirements it
serves by identifier. Identifiers flow **up**, from record to requirement;
roles flow **down**, from requirement to register. Two consequences: this
register can be renumbered, relocated or reorganised without touching
`/specification`, and no requirement can ever be made to depend on a record's
address — so the register can never become the authority for a requirement.

A reader resolves a role citation as: requirement → this index → the record →
back to the requirements by identifier. The index is what makes a role citation
resolvable, and it is the one job it has.

**R2 — The register is subordinate.** Where a record and `/specification`
disagree, the specification governs and the record is the defect. It is never
closed by revising the specification. A record never creates, weakens,
reinterprets or supplies a requirement. Where a record and the code disagree,
both readings are reported rather than silently reconciled.

**R3 — Exclusivity per fact.** A fact a record holds appears in that record and
nowhere else in the repository. `docs/spec-technical/technology-stack.md`,
`docs/spec-technical/security.md`, `docs/spec-technical/open-decisions.md`, the
root `README.md` and the root coordination document cite `ADR-NNN` instead of
restating it. This is the rule the corpus already applies to itself in
`BR-SRV-005` for the supported-series table and `BR-PERF-006` for a measured
figure.

**R4 — Admission is narrow.** A record is admissible only when a requirement in
force delegates the fact outside the corpus, or when the decision is
architectural and its alternatives must survive the choice. Everything else
stays in `docs/spec-technical/open-decisions.md` or in the technical
specification. Without R4 the register accretes into a second technical
specification, which is how a fourth place starts drifting.

## Index

| Record | Subject | Status | Serves | Superseded |
|---|---|---|---|---|
| [ADR-001](adr-001-template-engine-pin.md) | The template engine and its version pin | Accepted 2026-09-10 | `FR-ENV-003` | — |
| [ADR-002](adr-002-tls-mode-mapping.md) | The TLS mode mapping onto the database driver | Accepted 2026-09-10 | `FR-CONF-038` | — |
| [ADR-003](adr-003-database-driver.md) | The database driver | Accepted 2026-09-10 | `FR-CONF-036`, `FR-CONF-038` | — |
| [ADR-004](adr-004-release-profile-and-panic-path.md) | The release profile and the panic path | Accepted 2026-09-11 | `FR-ERR-030`, `FR-ERR-032`, `FR-ERR-034` | — |
| [ADR-005](adr-005-async-runtime-scope.md) | The scope of the async runtime | Accepted 2026-09-10 | `NFR-PERF-005`, `NFR-PERF-007` | — |
| [ADR-006](adr-006-package-layout.md) | The package layout | Accepted 2026-09-10 | `FR-ERR-031`, `NFR-PERF-018` | — |
| [ADR-007](adr-007-msrv.md) | The minimum supported Rust version | Accepted 2026-09-11 | — | — |
| [ADR-008](adr-008-packaging-and-build-path.md) | Packaging and the build path for the four targets | Accepted 2026-09-10 | `NFR-PERF-018`, `NFR-PERF-012` | — |
| [ADR-009](adr-009-foreign-key-embedding-representation.md) | The in-memory representation of the two foreign-key embeddings | Accepted 2026-09-10 | `FR-CTX-006` … `FR-CTX-010`, `FR-SCH-022` | — |

The `Serves` column names the requirements a reader is most likely to arrive
from. The complete list per record is its `requirements:` field, which is the
machine-derivable one; `ADR-007` serves none, and says why in its own Context.

## Conventions

- English. Requirement identifiers are the citation unit, as
  `specification/README.md` fixes them.
- Every factual claim about a technology names the source consulted and the
  date, or is marked as unverified. Recollection is not a source.
- A record is complete when written. A half-written record is not `proposed`;
  it is not in the register.
</content>
</invoke>
