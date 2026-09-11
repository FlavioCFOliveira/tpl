---
title: Overview
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md, open-decisions.md]
---

# Overview

## What this document is

The shape of the built artefact: what `tpl` is, who invokes it, what it refuses
to do, and the three limits it states rather than overcomes. Each statement
cites the requirement that forces it and says only how the built system
realises it; no requirement is reproduced here. Components are
`architecture.md`; technologies and versions are `technology-stack.md`.

## What `tpl` is

One executable, and the whole of its surface is the closed tree rooted at `tpl`
(`FR-CLI-001`, `FR-CLI-002`, `FR-CLI-010`). A token that is not a node of that
tree is a usage error and never a lookup on `PATH` (`FR-CLI-006`), so the
surface the built system exposes is exactly the surface this corpus can
enumerate. Group nodes carry no action of their own (`FR-CLI-008`,
`FR-CLI-009`).

`specification/glossary.md`, term *arm*, divides the eight top-level commands
of `FR-CLI-010` into three arms and the auxiliary set:

| Kind | Command | Subject |
|---|---|---|
| Arm | `tpl schema …` | the database structure |
| Arm | `tpl template …` | the project's templates |
| Arm | `tpl render` | one template, rendered once |
| Auxiliary | `tpl cache …` | the catalogue cache |
| Auxiliary | `tpl cfg …` | the configuration file and its database entries |
| Auxiliary | `tpl init` | creating a project |
| Auxiliary | `tpl help`, `tpl version` | the tool's description of itself |

**Recorded discrepancy — the size of the auxiliary set.** This folder's
[README.md](README.md#overviewmd) fixes the scope of this document as *"three
arms and four auxiliary command groups"*. `specification/glossary.md` names
five auxiliary command groups — `tpl cache`, `tpl cfg`, `tpl init`, `tpl help`
and `tpl version` — which is what `FR-CLI-010` leaves once the three arms are
removed. `DIV-040` reaches four by counting inside the sentence of `CLAUDE.md`
it corrects, not inside `FR-CLI-010`. Both readings are recorded; the table
above follows `FR-CLI-010`, which is the requirement. The count is load-bearing
for nothing in this folder, and the wording is the functional owner's to settle.

## The three properties that bound the artefact

| Property | How the built system holds it | Forced by |
|---|---|---|
| Read-only over the database | Only the statements of the closed list are ever sent, whatever the server would permit; the session setting is defence in depth, confirmed before any catalogue read by a read of `@@session.tx_read_only`, and its failure stops the read; no flag, key or environment condition disables either half | `FR-SRV-006`, `FR-SRV-007`, `BR-SRV-001`, `FR-SRV-008`, `FR-SRV-009`, `FR-SRV-010`, `FR-SRV-011`, `BR-SRV-002` |
| One render per invocation | An invocation yields one result, so the design questions a multi-object render would force are not questions this artefact has to answer | `FR-RND-002` |
| No file-writing surface | The rendered result goes to stdout and nowhere else, and no destination flag exists; every write the process makes is inside `.tpl`, with one enumerated exception; a read command's only write is the cache | `FR-RND-028`, `BR-RND-003`, `FR-PROJ-024`, `BR-PROJ-002` |

`BR-RND-003` is what makes the third row a property of the artefact rather than
a feature that happens to be absent: if a destination is ever brought back into
scope, the two findings it preserves are re-applied and not re-derived.

## What `tpl` is not

| Not | Because |
|---|---|
| A tool that writes to a database | `FR-SRV-007`, `BR-SRV-001` |
| A reader of table data | The closed list reaches `INFORMATION_SCHEMA` and nothing else (`FR-SRV-006`) |
| A reader of any product but MariaDB | `FR-SRV-001`, `FR-SRV-003` |
| Interactive | No prompt, no pager, no stdin but an explicit `--context -` (`BR-CLI-003`) |
| Configurable from the environment | The command line and the project decide everything; `${VAR}` inside the configuration file is the single exception (`FR-CLI-021`, `FR-CLI-023`, `BR-CLI-002`) |
| Terminal-aware | No terminal detection, no colour, no ANSI byte on either stream (`NFR-DET-003`, `NFR-DET-004`) |
| Extensible at the command level | `FR-CLI-002`, `FR-CLI-006` |
| Talkative on success | A successful command writes its result and nothing else (`BR-CLI-004`) |

## The primary consumer and its three channels

The primary consumer is a calling agent invoking `tpl` programmatically, not a
person at a prompt; the operator is served by the same surface, and where the
two would pull apart the agent decides the outcome (`specification/cli-contract.md`
*Actors*, `specification/errors-and-exit-codes.md` *Actors*,
`specification/help-and-version.md` *Overview*). That consumer has three
channels and no others, so all three are built as contracts.

| Channel | What the built system owes it | Forced by |
|---|---|---|
| Help text | Six forms, byte-identical within each equivalence, complete at every depth, and the whole tree obtainable as one machine-readable document in one invocation | `FR-HELP-001`, `FR-HELP-002`, `FR-HELP-016` |
| Exit code | One code per condition, sufficient on its own to choose the next step; it is also the only machine-comparable channel, because no error is ever emitted as JSON, which is why each code's `cause` line carries a stated obligation | `FR-ERR-001`, `FR-ERR-033`, `FR-ERR-034` |
| stdout and stderr | Results on stdout and everything else on stderr; byte-identical stdout for one invocation against one state; one envelope shared by every JSON document | `BR-CLI-006`, `NFR-DET-001`, `FR-OUT-024` |

Nothing about the artefact may be learned by asking, and nothing by looking: a
consumer that cannot answer a prompt, cannot read a terminal and cannot set an
environment must still reach every fact about the surface through those three
channels alone. That is why they are contracts rather than conveniences, and it
is why `interfaces.md` exists as a document.

## What is excluded from the model by decision

Five whole features are outside the model. Each exclusion is its own
requirement, and that identifier is how it is cited; a request to add one is a
change to `specification/catalogue-coverage.md`, not a defect report.

| Feature | Excluded by |
|---|---|
| Scheduled events | `FR-CAT-019` |
| Sequences | `FR-CAT-020` |
| Table partitions | `FR-CAT-021` |
| Application-time periods | `FR-CAT-022` |
| Spatial reference identifiers | `FR-CAT-023` |

`BR-CAT-001` states why the covered set was drawn wider than the classic core
and why these five are the ones a generator can notice and work around. The
consequence for the built system is that no part of it reads them, no context
variable carries them, and no template can ask for them: the exclusion is
enforced at the model, which every output format and every render shares
(`specification/catalogue-coverage.md` *Overview*). The covered set itself is
`data-model.md`.

## The three limits the system states rather than overcomes

`specification/README.md` *Writing conventions* fixes one shape for all three:
the limiting clause lives in the requirement, and the requirement says what a
reader would otherwise wrongly assume. This folder inherits that restraint —
where a guarantee stops, the document that would otherwise be read as promising
more says so.

| Limit | Stated in | What the built system therefore does not contain |
|---|---|---|
| An impostor server | `FR-SRV-041`, with `FR-SRV-003` | Any impostor detection. The product marker is stated there as a condition that is necessary and not sufficient, so the artefact refuses exactly the servers that condition rejects and claims nothing about the rest; no passage of this folder may claim more |
| Hidden triggers | `FR-PRIV-020` | Any inference of a reader's privileges from its grants. The unreadable case and the empty case have one representation in the model, so the completeness guarantee of `FR-PRIV-002` cannot hold for such a table and the artefact does not report that it does |
| Non-exclusive trust material | `FR-CONF-039` | Any claim of exclusive trust. Pinned trust material is additional to the root store rather than a replacement for it, on every path that uses it |

The mechanisms behind the second and third rows are `security.md`; the
supported-product criterion behind the first is below.

## The product the artefact supports

`FR-SRV-001` is a two-part criterion, `FR-SRV-015` is that criterion's instance
on a stated date, and `BR-SRV-004` fixes which of the two is the requirement: a
table found wrong is stale, and the correction re-derives the table rather than
amending the criterion. The built system is therefore written against the
criterion, and re-verifying the table is a release gate (`FR-SRV-019`), owned by
`operations.md`. What a difference between two supported series obliges is
`data-model.md` and `quality-attributes.md`.

## The library API is not a public surface

`DIV-032` fixes where the contract runs: through the JSON document and through
the command line, and not through the library. The document's shape is owned by
`specification/context-document.md` rather than by any set of types, so nothing
in the library is promised to a consumer and no change to it is a compatibility
event. Two consequences for the artefact:

- There is no published boundary for a package split to protect. What the
  layout is instead, and why, is settled in
  [ADR-006](../adr/adr-006-package-layout.md) and registered as
  [`OD-04`](open-decisions.md#od-04--one-package-or-a-workspace); a library
  exists there so that the logic is reachable from a test without launching a
  process, not so that a surface is published.
- The five shape questions the functional corpus declines — named in `DIV-032`
  — are architecture decisions of this folder, carried by
  [`OD-05`](open-decisions.md#od-05--the-module-decomposition) and answered in
  `interfaces.md`. None of them is a compatibility question.

## The stance the artefact is built with

Silent wrongness is the failure mode the functional corpus is written against,
and the same resolution recurs in every file of it: where the system cannot be
certain of the right output, it fails loudly rather than producing plausible
output quietly (`BR-SEM-004`, `BR-PRIV-001`, `BR-CAT-002`, `FR-SRV-003`). It is
one design stance applied repeatedly, and it is the tie-breaker for any
technical question this folder leaves under-determined.

## What this document defers, and to what

| Subject | Document |
|---|---|
| Components, their responsibilities, and the order an invocation runs in | `architecture.md` |
| Every technology, its version, and what was rejected | `technology-stack.md` |
| The contracts crossing a boundary, and how each external contract is realised | `interfaces.md` |
| The model and everything persisted | `data-model.md` |
| Trust boundaries, credentials, transport, containment | `security.md` |
| Build, targets, packaging, release gates, observability | `operations.md` |
| Targets, budgets, and how each is measured | `quality-attributes.md` |
| Tests, harness, fixture | `verification.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |

Two boundaries bound this document itself:

- **Nothing is open.** `specification/open-questions.md` carries an empty
  index, and every entry of [open-decisions.md](open-decisions.md) is settled.
  No statement in this folder may be deferred by citing an open functional
  question, and the boundary against the other three sources of truth is
  [README.md](README.md#the-four-sources-of-truth), settled as
  [`OD-26`](open-decisions.md#od-26--the-boundary-against-the-knowledge-graph).
- **No catalogue fact is re-derived here.** A requirement resting on direct
  observation is falsifiable by a later observation
  (`specification/README.md` *Provenance*, item 4), so this folder cites such a
  fact and never restates or re-derives it. The same restraint applies to the
  corrections `specification/upstream-divergences.md` owes the root documents:
  they are cited by `DIV` identifier, never reproduced.
