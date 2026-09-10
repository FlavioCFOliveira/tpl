---
title: Cache Documents
status: approved
last-reviewed: 2026-09-10
related: [cache-commands.md, context-document.md, output-formats.md, catalogue-coverage.md]
---

# Cache Documents

## Overview

[cache-commands.md](cache-commands.md) defines when the cache is read and
written. This file defines what is written: the two versions the cache carries,
the record of how much of each collection it holds, the field by which a read
declares where it came from, and the field that is deliberately kept out of a
read.

Everything here exists to close one gap. A cache that ages silently — which is
what `FR-CACHE-008` and `BR-CACHE-004` deliberately make it — must at least be
honest about what it holds, or a caller cannot tell a complete answer from a
partial one.

## Scope

In scope: `meta.json` and its two version fields, per-collection completeness,
the `source` field of a read, the exclusion of the load time from a read, the
path of a cached routine, and the consistency a cache-served document does not
promise.

Out of scope: when the cache is consulted or written, which is
[cache-commands.md](cache-commands.md); the content of a cached object, which is
[catalogue-coverage.md](catalogue-coverage.md); and the on-disk encoding, which
`BR-CACHE-001` places outside the plumbing contract.

## Actors

- **Calling agent**, reading the `source` field.
- **Operator**, reading `tpl cache status`.
- **`tpl` itself**, deciding on each read whether what it holds may be served.

## Two versions

- **FR-CDOC-001**: The system SHALL write `.tpl/.cache/<entry>/meta.json`, and
  that file SHALL carry `cache_format` and `schema_version`.

- **FR-CDOC-002**: `cache_format` SHALL version the on-disk arrangement of the
  cache: which files exist, where they sit, and how they are named.

- **FR-CDOC-003**: `schema_version` SHALL version the model content, and SHALL
  be the same version the documents in the cache carry under `FR-OUT-011`.

- **FR-CDOC-004**: IF either version is unknown to the running binary, THEN the
  affected data SHALL be treated as a miss, per `FR-CACHE-033`.

- **FR-CDOC-005**: The two versions SHALL be independent. Neither SHALL be
  incremented on account of a change that affects only the other.

- **BR-CDOC-001**: One version cannot express both facts. A change to the model
  invalidates the content of every cached object without changing the
  arrangement of a single file, and a change to the arrangement leaves the
  content valid. With one version, one of the two changes would either
  needlessly discard a warm cache or silently serve documents in a shape the
  binary no longer understands.

  *Rejected.* A single version, paired with a boolean `cached` field on reads.

## Per-collection completeness

- **FR-CDOC-006**: `meta.json` SHALL record, for each collection, whether that
  collection was loaded whole.

- **FR-CDOC-007**: A listing SHALL be served from the cache only WHERE its
  collection is recorded as whole. Otherwise the listing SHALL be a miss.

- **FR-CDOC-008**: An individual object SHALL be served from the cache whenever
  it is present, regardless of the completeness of its collection.

  *Note added in the fifth edition.* "Present" is now narrower than it was. An
  object marked `restricted` is never written, per `FR-CACHE-037`, so it is
  never present and a read of it is always a miss. This requirement is
  unchanged; what changed is the population it ranges over, and the change
  closes the case `OQ-048` raised — a stub served under this requirement to a
  reader who could have seen the whole object.

- **BR-CDOC-002**: Without the record, `tpl schema tables` run after
  `tpl -d shop cache load --table orders` would return exactly one table and
  exit `0`. That is a wrong answer wearing the appearance of a right one, and it
  is the kind of failure the caller has no way to detect.

  *Rejected.* Never serving a listing from the cache, which needs no marker at
  all but makes the ordinary exploration flow — list, then inspect, then inspect
  again — pay the network on every listing.

## Where a read came from

- **FR-CDOC-009**: Every read SHALL carry `source`, and a read that reaches a
  catalogue SHALL set it to `cache` or `server` according to which served the
  read. `source` is a field of the envelope of `FR-OUT-024`, whose position and
  full value set are fixed by `FR-OUT-026`; this file owns what the value
  `cache` withdraws, per `FR-CDOC-015` and `FR-CDOC-016`.

  *Amended in the third edition.* The first edition fixed the value set here as
  `cache` or `server`, at a time when no JSON document had a specified shape.
  `FR-OUT-026` now owns the set and has grown it to four: `project` for a read
  served from `.tpl/` alone and `binary` for a document derived from the binary
  itself. Both were foreseen by the rationale of `FR-CDOC-010` and neither is a
  breaking change, per `FR-OUT-014`.

- **FR-CDOC-010**: `source` SHALL be an enumerated string and SHALL NOT be a
  boolean.

  *Rationale.* `FR-OUT-014` lets an enumerated field gain a value without
  breaking the contract, and a boolean cannot gain one. A read that is partly
  served from the cache, or served from some third source, is foreseeable, and
  a boolean would force a breaking change to say so. Two such values arrived
  with the third edition, which is the argument holding.

- **FR-CDOC-011**: `source` SHALL satisfy `FR-CACHE-012`, which requires a
  cached read to state that it was cached.

- **FR-CDOC-012**: `loaded_at` SHALL NOT appear in the output of any read.

- **FR-CDOC-013**: `loaded_at` SHALL appear in `meta.json` and in the output of
  `tpl cache status`, and nowhere else.

- **BR-CDOC-003**: A load time in a read would make two identical invocations
  against an unchanged project produce different bytes, which `NFR-DET-001`
  forbids. `tpl cache status` is the place where the age of the cache is the
  answer rather than an incidental detail, and `BR-CACHE-003` already names it
  as the only signal that a cache is stale.

## Cached routines

- **FR-CDOC-014**: A cached routine SHALL be stored under a path that carries
  its kind as well as its name, in the form `routines/<kind>.<name>.json`.

  *Rationale.* Procedures and functions occupy distinct namespaces on the
  server, so one name can denote two objects and a path keyed on the name alone
  would collide. The qualified command-line form of `FR-SCH-008` and this path
  form are the same rule applied at two layers.

## What a cache-served document does not promise

- **FR-CDOC-015**: A document served wholly or partly from the cache SHALL NOT
  promise the referential integrity of `FR-CTX-023`, and SHALL NOT promise to
  be a point-in-time snapshot.

- **FR-CDOC-016**: `"source":"cache"` SHALL be the signal by which a consumer
  recognises that neither promise applies.

- **BR-CDOC-004**: The cache is written on a miss, per `FR-CACHE-007`, one file
  per object renamed into place, per `FR-CACHE-030`, and takes no lock, per
  `FR-CACHE-031`. It can
  therefore legitimately hold one table read on Monday beside another read on
  Friday, and a dump assembled from it is a document that never existed on any
  server at any instant. That is not a defect to be fixed; it is the direct
  consequence of a cache that changes only when it is told to, which
  `BR-CACHE-004` establishes deliberately. What was missing was the declaration,
  and `FR-CDOC-015` is it.

  *Rejected.* Caching the dump as a single document alongside the per-object
  files, which would give a cached dump the same guarantee as a live read, at
  the cost of storing everything twice and of two copies that drift apart.

## Business rules

- **BR-CDOC-005**: `meta.json` is not plumbing contract. `tpl cache status`
  remains the supported way to learn the state of the cache, per `BR-CACHE-001`.
  The two version fields are required so that the binary can decide what it may
  serve, not so that a caller can read them.

- **BR-CDOC-006**: `source` and `restricted` answer different questions.
  `source` says where the bytes came from; `restricted`, defined in
  [privileges-and-completeness.md](privileges-and-completeness.md), says that
  the reader could not see all of them. They differ in shape as well as in
  subject: `source` is on the envelope, is enumerated, and is present on every
  document, per `FR-OUT-026`; `restricted` is on the object, is an array of
  property names, and is present only where there is something to report, per
  `FR-PRIV-016`.

  *Amended in the fifth edition.* This rule previously said both can appear on
  the same object, which was two errors in one sentence. `source` is not a
  field of an object at all, per `FR-OUT-029`; and since `FR-CACHE-037` a
  restricted object is never cached, so no object in a document with
  `"source":"cache"` carries a `restricted` array.

## Dependencies

- [cache-commands.md](cache-commands.md) — when the cache is read and written,
  and `FR-CACHE-005`, `FR-CACHE-012`, and `FR-CACHE-033`, which this file
  details.
- [context-document.md](context-document.md) — the consistency a server read
  does promise.
- [output-formats.md](output-formats.md) — `FR-OUT-011` and `FR-OUT-014`.
- [privileges-and-completeness.md](privileges-and-completeness.md) — the other
  field that qualifies a read.

## Open questions

None specific to this module. `OQ-048` is answered by `FR-CACHE-037` and is
listed under [Closed](open-questions.md#closed).
