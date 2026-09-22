---
id: ADR-001
title: The template engine and its version pin
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-22
requirements: [FR-ENV-001, FR-ENV-003, FR-ENV-018, FR-ENV-019, FR-ENV-037, FR-ENV-044]
supersedes: []
superseded-by: null
---

# ADR-001 — The template engine and its version pin

## Status

Accepted, 2026-09-10.

## Context

`FR-ENV-001` divides the template surface into three groups. Group 2 is an
enumerated list of filters inherited from the engine, and it is guaranteed
"against a pinned engine minor version". `FR-ENV-018` closes that list at
fourteen names — `default`, `join`, `length`, `map`, `select`, `reject`,
`first`, `last`, `reverse`, `sort`, `trim`, `upper`, `lower`, `replace` — and
`FR-ENV-019` makes the list closed, so every other name the engine offers falls
into group 3 and carries no guarantee.

**`FR-ENV-003` delegates the pin to this register.** It requires three things:
that the pin exist, that it be recorded in the project's architecture decision
records, and that the requirement cite it from there rather than restate it.
The version number deliberately never enters `/specification`, for the reason
`BR-SRV-005` gives about the supported-series table and `BR-PERF-006` about a
measured figure — a number copied into a second file is the copy that stops
being true without saying so, and a dependency version moves on a schedule the
specification does not set. **This record is that home**, and it is the only
place in the repository where the number appears.

The engine itself is not open. The root coordination document fixes MiniJinja
(`minijinja` with `minijinja-contrib`), loaded and compiled at run time, and
excludes every compile-time engine by name. Two questions remained: which line
to pin, and whether `minijinja-contrib` stays.

## Decision

**Pin the 2.24 stable line of `minijinja`.** `2.24.0` is the maximum stable
release of the crate, published 2026-08-12 (crates.io, verified 2026-09-10).
"Stable line" means the `2.24.x` patch series: a patch release is adopted
without a new decision, a minor or major move is not.

**`minijinja-contrib` stays**, pinned to the same line; `2.24.0` is likewise its
maximum stable release, published the same day. Its filters are therefore
**group 3** of `FR-ENV-019` — available to a template, and guaranteed by
nobody.

All fourteen names of `FR-ENV-018` are core built-ins of `minijinja::filters`,
gated by the `builtins` cargo feature, which is one of the crate's eight
default features (docs.rs, verified 2026-09-10). No `minijinja-contrib` filter
appears in that list, so `FR-ENV-019` already classifies every one of them as
group 3 and retaining the crate changes no guarantee.

**Changing this pin is an edit to this record, not a successor to it.** The
decision — pin to a stable line, and re-check the guaranteed names — does not
change when the line moves; only its parameter does, and `FR-ENV-003` requires
the change to be "recorded in the same place". `FR-ENV-003` also makes any such
change a deliberate decision that **SHALL be accompanied by a check that every
name of `FR-ENV-018` still exists and still behaves as before**. That check is
an obligation on whoever moves the pin, and it is stated here because this
record is the document they will be reading when they do.

## Alternatives rejected

- **Dropping `minijinja-contrib`.** Nothing in the specified surface uses it:
  `FR-ENV-018` closes the guaranteed inherited list at fourteen core built-ins
  and no contrib filter is among them. But removing it would alter the stack
  table of the root coordination document for no functional gain, and the crate
  costs nothing in guarantee terms because `FR-ENV-019` already withholds a
  guarantee from everything it offers. A change with a cost and no benefit was
  refused.

- **Pinning the 3.0 pre-release.** `minijinja` `3.0.0-alpha.0` and
  `minijinja-contrib` `3.0.0-alpha.0` are published, both on 2026-08-12
  (crates.io, verified 2026-09-10) — the same day as the 2.24 releases, so this
  is a live choice rather than a stale one. It was refused: a pre-release under
  a release profile, behind a determinism contract, is a dependency whose
  behaviour may change without a version signal to announce it. `FR-ENV-003`
  makes moving the pin a deliberate act requiring a re-check of all fourteen
  names, and a pre-release moves under the project rather than being moved by
  it.

## Consequences

**Group 2's guarantee now names a fact somebody is obliged to keep true.**
Before this record, `FR-ENV-001` guaranteed fourteen filters against a pin that
nothing obliged to exist, to be written down, or to be findable. The guarantee
named a fact nobody was required to establish. It now resolves.

**Two `tpl` registrations deliberately shadow engine built-ins of a different
signature.** This is intended, and it is recorded so that it is stated rather
than discovered:

| Name | The engine's built-in | What `tpl` registers instead |
|---|---|---|
| `indent` | "Indents a value with spaces", gated by `builtins` | `indent(n)`, with `n` a required argument and no default; indents neither the first line nor an empty one, and adds no trailing whitespace (`FR-ENV-037`) |
| `escape` | "Escapes a string. By default to HTML" | `escape(target)`, with `target` required and restricted to `html` or `xml`; a five-row character table in which `'` differs between the two targets (`FR-ENV-044`) |

Both differ from the engine's version in **arity**: each `tpl` filter takes a
required argument where the engine's takes none. A template written against the
engine's signature therefore fails loudly under `tpl` rather than producing
different bytes silently, which is the outcome to prefer. Neither name is in
the closed list of `FR-ENV-018`, so neither shadowing weakens a group 2
guarantee: `escape` is registered by `tpl` and belongs to group 1 under
`FR-ENV-007`.

**A reader of `FR-ENV-003` cannot see the version without opening this file.**
That cost is accepted, in the requirement's own words, and it is the same cost
`BR-PERF-006` accepts for a recorded figure. It is paid for the same reason.

**Under R3, the number lives here alone.**
`docs/spec-technical/technology-stack.md` cites `ADR-001` for the pin rather
than restating it, and `docs/spec-technical/open-decisions.md` entry `OD-13`
reduces to a citation of this record.

**A patch release is adopted without a decision; a minor is not.** Pinning the
line rather than an exact patch keeps security and correctness fixes reachable
without a register entry each time, while holding the `FR-ENV-018` re-check
obligation at the boundary where the guaranteed names could actually move.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `minijinja` maximum stable version is `2.24.0`, published 2026-08-12; newest version is `3.0.0-alpha.0`, published 2026-08-12 | crates.io crate index, `minijinja` | 2026-09-10 |
| `minijinja-contrib` maximum stable version is `2.24.0`, published 2026-08-12; `3.0.0-alpha.0` published 2026-08-12 | crates.io crate index, `minijinja-contrib` | 2026-09-10 |
| The fourteen names of `FR-ENV-018` are documented filters of `minijinja::filters`, marked `builtins` | docs.rs, `minijinja` 2.24.0, module `minijinja::filters` | 2026-09-10 |
| `builtins` is one of eight default cargo features of `minijinja` 2.24.0, alongside `adjacent_loop_items`, `debug`, `deserialization`, `macros`, `multi_template`, `serde`, `std_collections` | docs.rs, `minijinja` 2.24.0, feature list | 2026-09-10 |
| The engine documents its own `indent` ("Indents a value with spaces", gated by `builtins`) and its own `escape` ("Escapes a string. By default to HTML") | docs.rs, `minijinja` 2.24.0, module `minijinja::filters` | 2026-09-10 |
| `minijinja-contrib` supplies extra utilities behind optional features (datetime, HTML entities, random, timezone, word wrapping, word count) and none of them is a name of `FR-ENV-018` | crates.io crate index, `minijinja-contrib` | 2026-09-10 |
</content>
