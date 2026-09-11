---
id: ADR-009
title: The in-memory representation of the two foreign-key embeddings
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-11
requirements: [FR-CTX-006, FR-CTX-007, FR-CTX-008, FR-CTX-009, FR-CTX-010, FR-CTX-023, FR-SCH-016, FR-SCH-022, BR-CTX-001, BR-SCH-004, NFR-PERF-014, NFR-PERF-019]
supersedes: []
superseded-by: null
---

# ADR-009 — The in-memory representation of the two foreign-key embeddings

## Status

Accepted, 2026-09-10.

## Context

The corpus fixes the **document** and says nothing about the model that produces
it. `FR-CTX-006` embeds the referenced table one level deep, `FR-CTX-007`
requires the embedded table to carry its columns, its indexes and its primary
key in full, and `FR-CTX-010` embeds the **referencing** table to the same depth
under the same rule. `FR-CTX-008` reduces the embedded table's own
`foreign_keys` to names, and `FR-CTX-009` makes that cut one rule applied at the
first hop in both directions, so that no traversal can fail to terminate.

That leaves one question the corpus does not answer and cannot: whether the
model holds the embedded tables as **owned copies** or holds one copy per table
and reproduces the embedding when the bytes are emitted. Both produce the same
document, which is why the corpus is silent, and the choice is architectural —
rule R4's second limb — because the alternatives differ in what can go wrong
rather than in what is promised.

Two obligations bear on it. `FR-SCH-016` makes the dump a single JSON document
and `FR-CTX-023` requires every object referenced from another object in it to
be present in it. `FR-SCH-022` and `BR-SCH-004` make the round-trip contract:
dump the reference database, feed the dump back through
`tpl render --context`, and assert that the result is byte-identical to the same
render against a live read.

## Decision

**Materialise both embeddings. The object graph *is* the document.** Each
embedding site holds its own owned copy of the embedded table, cut at the first
hop per `FR-CTX-009`, and serialisation is a walk over that structure rather
than a reconstruction of it.

**There is one representation to get right.** The round-trip of `FR-SCH-022`
becomes a property of that single representation rather than of an emitter that
has to agree with it, which is what makes `BR-SCH-004`'s byte-identity
assertion a test of one thing.

**This record decides the representation, not the document.** Every shape,
depth and cut remains the corpus's, and nothing here may be read as widening or
narrowing one.

## Alternatives rejected

- **Emit by reference at serialisation time**, keeping one owned copy of each
  table and reproducing the embedding as the bytes are written. The bytes would
  be identical and the memory lower — this is the alternative with a real
  benefit, and it is the one to revisit if the memory cost is ever found
  intolerable. Refused because the emitter would have to reproduce the one-hop
  cut of `FR-CTX-009` correctly **in both directions**, including the
  self-reference and cycle shapes that requirement enumerates, and a defect
  there is a wrong document emitted at exit `0` — a failure with no exit code,
  no message, and nothing for a caller to branch on.

- **Stream the dump**, so that peak memory tracks the largest table rather than
  the whole document. Refused because it cannot honour what the dump promises:
  `FR-SCH-016` makes it one document and `FR-CTX-023` promises referential
  integrity over it, so the whole model has to be in hand before the first byte
  can be trusted. Streaming would mean emitting bytes that a later read might
  contradict.

## Consequences

**Peak resident memory is the cost, and it is the largest single cost in the
document.** `BR-CTX-001` records what the outgoing embedding costs over the
reference database and `FR-CTX-010` records what the incoming one adds; neither
figure is restated here. This record changes neither cost — the document already
carries them — but it declines the one option that would have paid less than the
document's own weight.

**The provisional peak-memory budget is the figure most likely to move upward
first.** `NFR-PERF-014` carries it as provisional under `NFR-PERF-019`, which
exists precisely so that a figure can be superseded by the first real
measurement without ever having been a limit in the meantime. Nothing in this
record is invalidated by that move, and nothing in it should be read as a
commitment to the current figure.

**Streaming stays available everywhere the dump is not.** The root coordination
document requires memory to scale with the largest individual object rather than
the whole database "em tudo o que não exija o documento completo". This record
names the dump as one thing that does require it, per `FR-SCH-016` and
`FR-CTX-023`, and reaches no further: a command that reads one object is
untouched by it.

**A by-reference emitter remains a valid successor, and what would make it one
is a measurement.** The rejection above is on risk, not on impossibility. If the
memory cost is ever measured intolerable, the successor record would have to
carry the cut of `FR-CTX-009` in both directions as its own obligation, and a
test that compares its bytes against this representation's.

**Under R3, the representation lives here alone.**
`docs/spec-technical/architecture.md` and
`docs/spec-technical/data-model.md` cite `ADR-009` rather than restating it, and
`docs/spec-technical/open-decisions.md` entry `OD-19` reduces to a citation of
this record.

## Sources

Every claim in this record is a claim about the corpus or about this decision;
none is a claim about a technology, so none carries a vendor source.

| Claim | Source | Consulted |
|---|---|---|
| Both directions embed one level deep, carry columns, indexes and primary key in full, and are cut to names at the first hop by one rule applied in both directions | `specification/context-document.md`, `FR-CTX-006` … `FR-CTX-010` | 2026-09-11 |
| The dump is a single JSON document, and every object referenced from another object in it is present in it | `specification/schema-commands.md`, `FR-SCH-016`; `specification/context-document.md`, `FR-CTX-023` | 2026-09-11 |
| The dump round-trips through `--context` and carries a mandated byte-identity test | `specification/schema-commands.md`, `FR-SCH-022`, `BR-SCH-004` | 2026-09-11 |
| The peak-memory budget over the reference workload is provisional and may be superseded upward by the first real measurement | `specification/performance-requirements.md`, `NFR-PERF-014`, `NFR-PERF-019` | 2026-09-11 |
| Memory scales with the largest individual object in everything that does not require the complete document | `CLAUDE.md`, *Desempenho e Eficiência* | 2026-09-11 |
