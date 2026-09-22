---
title: Technology Stack
status: draft
last-reviewed: 2026-09-17
related: [README.md, traceability.md, open-decisions.md, overview.md, architecture.md, interfaces.md, data-model.md, quality-attributes.md]
---

# Technology Stack

## What this document is

Every technology `tpl` is built from: its exact version, what it does here, why
it and not the alternative, and the source consulted with the date. How a
component *uses* a crate is [architecture.md](architecture.md); the signatures
that cross a boundary are [interfaces.md](interfaces.md); every measured figure
is `BENCHMARKS.md`.

## How to read an entry

Three rules govern every row below, and they are stated once here rather than
repeated in each.

| Rule | Consequence |
|---|---|
| **A fact a decision record holds is cited, never restated** (rule R3, [`docs/adr/`](../adr/README.md)) | Four crates carry their version in a record. Their version column is a citation, and the record is the one place the number lives. The same holds for the rationale and the rejected candidate wherever a record carries them |
| **Every claim about a technology names its source and the date consulted, or is marked unverified** | The rule is the register's, recorded in [`docs/adr/README.md`](../adr/README.md), and it governs this folder unchanged |
| **No measured figure appears here** | Startup, binary size and resident memory decided nothing in this document and are recorded in `BENCHMARKS.md` alone, per `BR-PERF-006` and the extension [`ADR-003`](../adr/adr-003-database-driver.md) makes of it |

A version inside a source locator — *docs.rs, `minijinja` 2.24.0, module …* —
names the documentation that was read, not a pin. It is the citation discipline
the first rule requires, and it is how this folder already cites
([interfaces.md](interfaces.md#context-access-from-a-filter-or-a-test)); the pin
itself remains the record's.

**A bare version below is a release read from the source beside it on the date
beside it, never the requirement the package declares.** Where the column cites
a record instead, the pin is that record's
([`ADR-001`](../adr/adr-001-template-engine-pin.md),
[`ADR-003`](../adr/adr-003-database-driver.md)). In both forms the requirement
the package resolves against is the manifest's, and it is read there and not
here.

## The language, the toolchain, and the package

| Subject | What is fixed | Where it lives |
|---|---|---|
| Language and edition | Rust, edition 2024 | `CLAUDE.md`, *Stack* |
| Minimum supported toolchain | A rule — the higher of the edition floor and the highest floor declared in the shipped graph — and the figure it yields | [`ADR-007`](../adr/adr-007-msrv.md), registered as [`OD-02`](open-decisions.md#od-02--the-msrv) |
| Release profile | Five settings, and the panic path that reports `70` under them | [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md), registered as [`OD-28`](open-decisions.md#od-28--the-release-profile-against-the-caught-panic-condition-of-70) |
| Package layout | One package carrying a library and a binary; `benches/` gets no package of its own | [`ADR-006`](../adr/adr-006-package-layout.md), registered as [`OD-04`](open-decisions.md#od-04--one-package-or-a-workspace) |
| `unsafe` | `#![forbid(unsafe_code)]` at the top of the crate | `CLAUDE.md`, *Regras Inegociáveis*; consequence [below](#the-consequence-of-forbidding-unsafe) |

The toolchain used to build the four targets of `NFR-PERF-018`, and the
development toolchain that is above the floor, are
[operations.md](operations.md#msrv-the-development-toolchain-and-the-cross-build-path)'s
with [`ADR-008`](../adr/adr-008-packaging-and-build-path.md); that section is
also the one home of the toolchain and tool versions installed on the
development host.

## The shipped graph: versions and sources

The **shipped graph** is what `cargo tree -e normal,build` prints, which
[`ADR-006`](../adr/adr-006-package-layout.md) names as the listing that
excludes dev-dependencies. Every crate below is a direct dependency of it.

| Crate | Exact version | Source | Consulted |
|---|---|---|---|
| `clap` | 4.6.6 | crates.io crate index, `clap`, `max_stable_version` | 2026-09-11 |
| `minijinja` | The pin of [`ADR-001`](../adr/adr-001-template-engine-pin.md) | That record | 2026-09-11 |
| `minijinja-contrib` | The pin of [`ADR-001`](../adr/adr-001-template-engine-pin.md) | That record | 2026-09-11 |
| `sqlx` | The pin of [`ADR-003`](../adr/adr-003-database-driver.md) | That record | 2026-09-11 |
| `tokio` | The pin of [`ADR-003`](../adr/adr-003-database-driver.md) | That record | 2026-09-11 |
| `serde` | 1.0.229 | crates.io crate index, `serde`, `max_stable_version` | 2026-09-11 |
| `serde_json` | 1.0.151 | crates.io crate index, `serde_json`, `max_stable_version` | 2026-09-11 |
| `toml` | 1.1.6+spec-1.1.0 | crates.io crate index, `toml`, `max_stable_version` | 2026-09-11 |
| `toml_edit` | 0.25.15+spec-1.1.0 | crates.io crate index, `toml_edit`, `max_stable_version` | 2026-09-11 |
| `thiserror` | 2.0.20 | crates.io crate index, `thiserror`, `max_stable_version` | 2026-09-11 |
| `rustix` | 1.1.4 | crates.io crate index, `rustix`, `max_stable_version` | 2026-09-11 |
| The TLS crates | **No version is fixed by decision.** They enter transitively through the driver's TLS feature; [`ADR-002`](../adr/adr-002-tls-mode-mapping.md) states in terms that it pins no TLS crate version, and the versions `BENCHMARKS.md` records are a property of what was measured | `sqlx-core/Cargo.toml` at tag `v0.9.0` declares its optional `webpki-roots` dependency at `1`, so the bundled root set moves with any release of that major line | 2026-09-11 |

## The shipped graph: purpose, choice, and what was rejected

Where a record or a settled entry carries the argument, this table names the
rejected option and cites the argument rather than reproducing it.

| Crate | What it does here | Chosen because | Rejected |
|---|---|---|---|
| `clap` | Parses the invocation and supplies the runtime tree introspection `FR-HELP-021` requires | It is the parser `CLAUDE.md` *Stack* fixes, and its typed error context is the only source of the token `FR-ERR-034` row `64` obliges | Recovering the token by a second parse of `argv`; letting the parser render its own diagnostics and its own help ([`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own), [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)) |
| `minijinja` | Compiles and renders a template at run time, per `FR-TMPL-004` and `CLAUDE.md` *Invariantes de Implementação* | [`ADR-001`](../adr/adr-001-template-engine-pin.md) | [`ADR-001`](../adr/adr-001-template-engine-pin.md); every compile-time engine is excluded by `CLAUDE.md` *Invariantes de Implementação* |
| `minijinja-contrib` | Adds utility filters and globals, all of them group 3 of `FR-ENV-019` | [`ADR-001`](../adr/adr-001-template-engine-pin.md) | [`ADR-001`](../adr/adr-001-template-engine-pin.md) |
| `sqlx` | Connects to MariaDB and issues the closed statement list of `FR-SRV-006` | [`ADR-003`](../adr/adr-003-database-driver.md) — settled by `FR-CONF-036`, not by the measurement | [`ADR-003`](../adr/adr-003-database-driver.md) |
| `tokio` | The current-thread runtime the asynchronous driver requires, and the timers four of the six deadlines use | [`ADR-005`](../adr/adr-005-async-runtime-scope.md) | [`ADR-005`](../adr/adr-005-async-runtime-scope.md) |
| `serde` | The derive that fixes the key order of `FR-OUT-013` in the type. It is **not** how `.tpl/.cfg` is read: [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path) records why a derive cannot answer `FR-CONF-034` or `FR-CONF-035` | [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions) | A bespoke writer, and a distinct type per absence rule ([`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)) |
| `serde_json` | Encodes the seventeen payload shapes and the envelope of `FR-OUT-024`; its pretty printer answers `FR-OUT-008` | [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions) | `preserve_order`, and with it `indexmap`, which would contradict `NFR-DET-002` ([`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)) |
| `toml` | The read path over `.tpl/.cfg`, through its document tree — spanned keys and spanned values — rather than through a `serde` derive | [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path) | `toml_edit` for both paths, which would put an editing document on the path that reads untrusted input; and a `serde` derive, which cannot name the offending key or its position |
| `toml_edit` | The write path over `.tpl/.cfg`, preserving comments, spacing and the relative order of items | [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path) | `toml` alone, which would delete the commented example `FR-PROJ-018` requires on the first write |
| `thiserror` | Derives the one public error enum and its `Display` | [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) | `anyhow` in the library; per-module enums composed by `From`; an exit code stored as a field ([`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation)) |
| `rustix` | Supplies the process's own user identifier, the one value `std` does not give, for the ownership check of `FR-PROJ-010` | [`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid) | `libc` with a local `unsafe` block; `nix`; a crate that resolves the user account; inferring ownership by attempting a write ([`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid)) |

**One crate left the graph, and both tables above lost its row.** `anyhow` was
removed by [`OD-32`](open-decisions.md#od-32--anyhow-in-the-shipped-graph),
which holds the ground, the alternative it refused and the correction it
prepared for `CLAUDE.md`; none of that is restated here. The removal landed at
commit `455e48d`: `grep -n anyhow Cargo.toml` and `cargo tree -i anyhow` both
return nothing, and the direct dependencies of the shipped graph are the eleven
the tables name (`cargo tree -e normal,build --depth 1`, read 2026-09-22). The
observation this section used to carry — that a binary reduced to calling the
library, reading the exit code and returning carries no dynamic error — is that
entry's ground and no longer this document's open question. Where the removal
left the MSRV is [`ADR-007`](../adr/adr-007-msrv.md)'s, cited here and not
restated.

## The template engine

The pin, the line it names, and what a move of it obliges are
[`ADR-001`](../adr/adr-001-template-engine-pin.md), registered as
[`OD-13`](open-decisions.md#od-13--the-engine-pin-and-minijinja-contrib). The
engine's construction — the loader, the undefined behaviour, auto-escaping — is
[architecture.md](architecture.md#the-render-component) with
[`OD-14`](open-decisions.md#od-14--which-undefined-behaviour-the-engine-is-configured-with)
and [`OD-15`](open-decisions.md#od-15--the-template-loader); the registered
surface is [interfaces.md](interfaces.md#the-template-surface).

### The three classes of engine built-in

`minijinja::filters` documents **forty-six** filters at the pinned line
(docs.rs, `minijinja` 2.24.0, module `minijinja::filters`, consulted
2026-09-11). The names below are that inventory; the class each falls into is
decided by `FR-ENV-018` and `FR-ENV-019`, and by the two registrations of
`FR-ENV-007`.

| Class | Filters | Why |
|---|---|---|
| **Guaranteed** — group 2 | `default`, `join`, `length`, `map`, `select`, `reject`, `first`, `last`, `reverse`, `sort`, `trim`, `upper`, `lower`, `replace` | `FR-ENV-018` closes the guaranteed inherited list at these fourteen and `FR-ENV-003` guarantees them against the pin of [`ADR-001`](../adr/adr-001-template-engine-pin.md) |
| **Shadowed** — group 1, registered over an engine built-in of the same name and a different arity | `indent`, `escape` | `FR-ENV-007` registers both, `FR-ENV-037` and `FR-ENV-044` give each a required argument the engine's version does not take. Neither name is in the closed list of `FR-ENV-018`, so neither shadowing weakens a group 2 guarantee; the deliberateness and its consequence are [`ADR-001`](../adr/adr-001-template-engine-pin.md)'s |
| **No guarantee** — group 3 | `abs`, `attr`, `batch`, `bool`, `capitalize`, `chain`, `dictsort`, `float`, `format`, `groupby`, `int`, `items`, `lines`, `list`, `max`, `min`, `pprint`, `rejectattr`, `round`, `safe`, `selectattr`, `slice`, `split`, `string`, `sum`, `title`, `tojson`, `unique`, `urlencode`, `zip` | `FR-ENV-019` makes the list of `FR-ENV-018` closed, so every remaining filter the engine offers is group 3 |

Three properties of the table bind anyone who reads it.

- **It is the complement at the pinned line and nowhere else.** A move of the
  pin can add a name to the third class or remove one; `FR-ENV-003` already
  obliges a re-check of the fourteen at that moment, and this table is
  re-derived in the same pass.
- **The third class extends past filters.** `FR-ENV-001` puts *everything else
  the engine offers* in group 3, which covers the engine's own global functions
  — `debug`, `dict`, `namespace`, `range` — and the thirty-one test functions of
  `minijinja::tests`, reached in a template by their item name without the `is_`
  prefix, as the engine's own example `{% if foo is odd %}` for `is_odd` shows
  (docs.rs, `minijinja` 2.24.0, modules `minijinja::functions` and
  `minijinja::tests`, consulted 2026-09-11). **Whether `Environment::new` installs
  those globals and tests without a registration call is not confirmed in the
  engine's official documentation**, and no statement here rests on it.
- **`DIV-033` is the reason the table stops where it does.** The whole
  `Environment` surface is not contract; only the three groups are.

### `minijinja-contrib`: what it offers, and under which features

The crate declares **twelve feature flags and enables none of them by default**
(docs.rs, `minijinja-contrib` 2.24.0, feature list, consulted 2026-09-11). What
it offers therefore divides in two, and the division is load-bearing.

| Gate | Items | Bearing on a requirement |
|---|---|---|
| None | Filters `filesizeformat`, `pluralize`, `striptags`, `truncate`; globals `cycler`, `joiner` | Group 3 under `FR-ENV-019`, guaranteed by nobody |
| `datetime` | Filters `dateformat`, `datetimeformat`, `timeformat`; global `now`, documented as returning the current time in UTC | `FR-ENV-025` forbids the system to provide a function that reads a clock, so this feature stays off |
| `rand` | Filter `random`; globals `lipsum`, `randrange` | `NFR-DET-001` requires byte-identical stdout for one invocation against one state, so this feature stays off |
| `wordwrap`, `wordcount` | Filters `wordwrap`, `wordcount` | Nothing requires them; they remain off with the crate's default |

Source for the item list and every gate: docs.rs, `minijinja-contrib` 2.24.0,
modules `minijinja_contrib::filters` and `minijinja_contrib::globals`,
consulted 2026-09-11.

**Recorded observation — nothing states that the crate's items are registered
at all.** Its own documentation says that to add them to an environment one
uses `add_to_environment`, which "Registers all features of this crate with an
`Environment`" (docs.rs, `minijinja-contrib` 2.24.0, crate documentation,
consulted 2026-09-11). [`ADR-001`](../adr/adr-001-template-engine-pin.md)
describes the crate's filters as available to a template and guaranteed by
nobody, which is true once that call is made and not before. Both readings are
recorded. Whether the call is made is *how a component uses a crate*, so it is
[architecture.md](architecture.md#the-render-component)'s and is not stated
there today; nothing in this folder may assume either answer, and the table
above is what makes the feature set a constraint either way.

## The database driver, and the five TLS modes

The driver, the rule that settled it and the candidate it disqualified are
[`ADR-003`](../adr/adr-003-database-driver.md); the mapping of the five modes of
`FR-CONF-013` onto it, and the bundled trust anchors, are
[`ADR-002`](../adr/adr-002-tls-mode-mapping.md), registered as
[`OD-16`](open-decisions.md#od-16--the-tls-backend-and-the-root-store). Neither
is restated. What belongs here is the criterion as a property of this stack, and
the feature selection that realises the two records.

**One crate of the driver is read from inside this repository rather than from
the index.** A `[patch.crates-io]` entry redirects `sqlx-core` to a vendored
copy, on the terms [`ADR-010`](../adr/adr-010-driver-tls-connect-stall.md) sets:
that record holds the defect, the patch, the obligations the vendored tree
carries and the condition that retires it, and it states that the copy is not a
new dependency and does not enter the budget below. The pin is unchanged and
remains [`ADR-003`](../adr/adr-003-database-driver.md)'s.

**`FR-CONF-036` is an admission criterion, not a preference.** A driver that
cannot express all five modes of `FR-CONF-013` distinctly is disqualified, and
the choice may not be settled by reducing the mode set to fit a candidate. A
requirement that disqualifies a dependency is an admission test over this
document's whole table, and this is the one the corpus states; `FR-SEC-021` is
what it protects.

The driver declares **forty-three feature flags, five of them enabled by
default** (docs.rs, `sqlx` 0.9.0, feature list, consulted 2026-09-11). Three
features are required by decisions already settled:

| Feature | Required by | Source |
|---|---|---|
| `mysql` | The product `FR-SRV-001` fixes | `Cargo.toml` at tag `v0.9.0` |
| `runtime-tokio` | The runtime of [`ADR-005`](../adr/adr-005-async-runtime-scope.md) | `Cargo.toml` at tag `v0.9.0` |
| `tls-rustls-ring-webpki` | The bundled root store of [`ADR-002`](../adr/adr-002-tls-mode-mapping.md). It resolves to `sqlx-core/_tls-rustls-ring-webpki`, which is defined as `["_tls-rustls", "rustls/ring", "webpki-roots"]`; its sibling `tls-rustls-ring-native-roots` resolves to `rustls-native-certs` instead and is the platform trust store that record rejects | `Cargo.toml` and `sqlx-core/Cargo.toml` at tag `v0.9.0` |

All three sourced from the driver's own repository at tag `v0.9.0`, consulted
2026-09-11. `tls-rustls` and `tls-rustls-ring` are documented in that manifest
as backwards-compatibility aliases that resolve to the same webpki feature.

`tokio` enables no feature by default (docs.rs, `tokio` 1.53.1, *Feature
flags*, consulted 2026-09-11), so three are added rather than subtracted: `rt`,
which "Enables `tokio::spawn`, the current-thread scheduler, and non-scheduler
utilities" ([`ADR-005`](../adr/adr-005-async-runtime-scope.md)); `time`, which
enables the types the four runtime-enforced deadlines use
([`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced)); and
`net`, which
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) already
records as the gate on the name resolution `tpl` performs itself.

**Observation — the driver's five default features are decided by nothing.**
`any`, `json`, `macros`, `migrate` and `derive` are on unless
`default-features = false` is set. `tpl` issues a fixed statement list against
one product, runs no migration and uses no compile-time checked query, so none
of the five answers a requirement; but turning them off is a decision under the
dependency budget, and this document records the question rather than settling
it.

## The argument parser

`clap` declares six default features — `std`, `color`, `help`, `usage`,
`error-context`, `suggestions` — and lists `derive`, `wrap_help`, `env`,
`cargo`, `unicode`, `string` and `deprecated` as optional (docs.rs, `clap`
4.6.6, *Feature Flags*, consulted 2026-09-11).

**Two default features must be off, and each is forced by a different
requirement.**

| Feature | Forced off by | Why the requirement forces it |
|---|---|---|
| `color` | `NFR-DET-004`, with `NFR-DET-003` | No ANSI escape sequence may reach stdout or stderr under any circumstances, and the requirement names removing colour as what lets the parser be built without its colour support ([`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own), [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)) |
| `suggestions` | `FR-ERR-019`, `FR-ERR-020`, `FR-ERR-023`, `FR-ERR-039` | The corpus fixes the candidate count, the ordering, a character-class filter and the measure itself; [`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms) fixes only how that measure is computed. A second candidate generator would compute a set that is then discarded ([`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)) |

**`error-context` stays on**, because it is the only source of the token
`FR-ERR-034` row `64` obliges the `cause` line to name
([`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)).

Three optional features complete the picture:

| Feature | State | Traced to |
|---|---|---|
| `derive` | On | `CLAUDE.md` *Stack* names the derive tree |
| `wrap_help` | Left off | `FR-HELP-009` and `FR-HELP-010` put the line breaks in the text and forbid reading `COLUMNS` ([`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own)) |
| `env` | Never enabled | `FR-CONF-030` denies the resolver an environment layer and `FR-CLI-021` forbids the environment deciding which project or database an invocation reaches. Nothing in the register enables it; it is recorded here because enabling it would contradict those two requirements silently |

Neither the help text nor a parser diagnostic is rendered by `clap`: seven
sections in a fixed order, byte-identical at every depth, are `tpl`'s to produce
([`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own)), and
`clap::Error` is intercepted and re-rendered
([`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)). The surface
that results is [interfaces.md](interfaces.md#the-help-surface)'s.

## The two TOML paths

One file, two crates, because the two paths carry opposite obligations: reading
must name the key and the byte that are wrong, and writing must not disturb a
byte it was not asked to change.
[`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path) holds
the six requirements that force the split, the three the read path's shape
answers, and the options it rejected — including the `serde` derive this
document named until 2026-09-17. The file's shape, mode and rewrite discipline
are [data-model.md](data-model.md#tplcfg); the reader's and the writer's
obligations are
[interfaces.md](interfaces.md#the-configuration-reader-and-the-writer).

**Recorded discrepancy — closed.** This document and
[`ADR-007`](../adr/adr-007-msrv.md) read `toml` and `toml_edit` on the same day
and saw different patches, one having been published between the two readings.
The only question that turned on the difference — whether the newer patches
raise the toolchain floor — was that record's, and it settled the question by
re-running its rule over the resolved graph: neither crate moves the floor. The
floor itself is cited from that record and never repeated here, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). The versions in the table above remain
this document's own reading, with the source and the date beside each.

## Serialisation

Derived `Serialize` through `serde_json`, into a writer `tpl` owns; the field
declaration order **is** the key order, `preserve_order` is off, `serde_json`
does not depend on `indexmap`, and `skip_serializing_if` governs **one field of
the model**, written once for each kind that can carry the marking. All five
answers, the four options rejected, and the readings behind the last two are
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions).
Those last two were re-checked at commit `243c4d6` on 2026-09-21 — with
`cargo tree --all-features --invert indexmap` and with
`grep -rn skip_serializing_if src/` — and both hold as that entry states them.
Each names a property a command can be run against, where the two claims they
replace — that `indexmap` enters no graph, and that the attribute appears once
in the crate — named none, and were false besides.

`serde` appears in the library's public signature, and it costs nothing: `DIV-032`
fixes the contract at the JSON document and the command line, so no consumer's
build can be broken by a major version of it
([overview.md](overview.md#the-library-api-is-not-a-public-surface)).

## Diagnostics

**No logging facade and no subscriber are dependencies.** `tracing` and
`tracing-subscriber` were removed by
[`OD-17`](open-decisions.md#od-17--observability), which records why installing
no subscriber is what makes `FR-GLOB-018` structural rather than a review item
over every crate in the graph, and what it rejected to get there. Diagnostics
are written by `diagnostics/` through a closed set of typed emission functions;
the verbosity levels of `FR-GLOB-014` and `FR-GLOB-015` select among them.

Two consequences belong to this document rather than to that entry. A facade a
dependency emits into stays a **transitive** crate in the graph and is never a
facility `tpl` uses. And the dependency budget is charged nothing at all for
observability.

## The one call `std` does not supply

`rustix`, with `default-features = false` and the `process` feature alone,
supplies the process's own user identifier as a safe function; `std` supplies
the file's identifier and its mode on the same metadata, so the ownership check
of `FR-PROJ-010` needs exactly one call and the mode check of `FR-PROJ-011`
needs none. The decision, the sourcing and the four rejected options are
[`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid).

## The dependency budget

The rule is the root coordination document's: prefer `std`, justify each crate,
check what it drags in, and refuse one used for a trivial function (`CLAUDE.md`,
*Desempenho e Eficiência*). Four conditions apply it here.

| Condition | Source |
|---|---|
| The budget is a budget on the **shipped** graph, which an unfiltered `cargo tree` overstates | [`ADR-006`](../adr/adr-006-package-layout.md) |
| A crate must compile and pass tests on all four targets of `NFR-PERF-018`; one that does not support `aarch64` does not enter | `CLAUDE.md`, *Plataformas Suportadas* |
| A crate that declares a toolchain floor above the current one raises the MSRV, so admission is also an MSRV question | [`ADR-007`](../adr/adr-007-msrv.md) |
| A closed grammar written into the corpus is implemented, not depended on | Below |

**Six facilities are implemented in the crate rather than taken from a
dependency**, and one argument decides five of them: each has a closed grammar
fixed by a requirement, so a library would have to be constrained back to that
grammar rather than consulted for it. Those five — the suggestion distance, the
`LIKE` matcher, POSIX word splitting, the word-list tokeniser and ASCII-only
case folding — are
[`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms).
The sixth is the clock read with the single date grammar of `FR-CTX-028`, and it
is [`OD-25`](open-decisions.md#od-25--the-clock-source-for-now). Each entry
carries the crate it refused and why.

Dev-dependencies are outside the budget and outside the shipped graph
([`ADR-006`](../adr/adr-006-package-layout.md)); the measurement and validation
toolchain is `operations.md`'s.

## The consequence of forbidding `unsafe`

`#![forbid(unsafe_code)]` is a lint attribute, and two properties of it decide
what the stack may contain.

- The `unsafe_code` lint "catches usage of unsafe code and other potentially
  unsound constructs like `no_mangle`, `export_name`, and `link_section`"
  (Rust compiler documentation, *Lints that are allowed by default*, consulted
  2026-09-11).
- `#[forbid(C)]` "is the same as `deny(C)`, but also forbids changing the lint
  level afterwards", and a lint attribute changes the level "for the entity to
  which the attribute applies" (The Rust Reference, *Lint check attributes*,
  consulted 2026-09-11).

So the prohibition reaches the code of this crate and cannot be relaxed locally
— the `#[allow(...)]` escape `CLAUDE.md` permits for a `clippy` lint is not
available for this one. It does not reach a dependency's internals, which is
precisely what makes
[`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid)'s
answer possible: the system call is made inside a crate that wraps it in a safe
function, rather than in an `unsafe` block here. A crate is therefore admitted
for the safe surface it presents, never rejected for using `unsafe` inside
itself — and a facility that has no safe wrapper anywhere is one `tpl` does
without.

## What this document defers, and to what

| Subject | Where |
|---|---|
| How a component uses any crate above | [architecture.md](architecture.md) |
| Every signature and every contract crossing a boundary | [interfaces.md](interfaces.md) |
| Every measured figure, and the artefacts a measurement was taken on | `BENCHMARKS.md` |
| The build, the four targets, the release gates, the validation pipeline, the measurement toolchain, and what the development host has installed | [operations.md](operations.md), with [`ADR-008`](../adr/adr-008-packaging-and-build-path.md) |
| The measurement points these choices are read against | [quality-attributes.md](quality-attributes.md) |
| Trust boundaries, credentials, and transport as subjects | `security.md`, with [`ADR-002`](../adr/adr-002-tls-mode-mapping.md) |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
