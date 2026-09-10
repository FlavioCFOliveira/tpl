---
title: Decision Register
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md]
---

# Decision Register

## What this is

Twenty-eight entries, each a decision the repository could not settle on its
own. Nineteen are settled — by the product owner in the interview of
2026-09-10, by the establishment of the decision register, and by the eighth
edition of `/specification` — and nine are open under `technical-writer`.
Nineteen and nine are the whole of the twenty-eight. None is open under
`adr-guardian`.

**No entry is a conflict.** The three that were — `OD-21`, `OD-22` and
`OD-24` — were resolved in the eighth edition, which read requirement against
requirement and named the requirement that yields in each case. Each is now
settled with the resolution the corpus made, and each carries a residual that
is a technical choice rather than a defect.

Each settled entry records the decision, its rationale, and **the options
rejected**, for the reason [`docs/adr/README.md`](../adr/README.md) gives for
the same section in a record.

The architecture decision record register now exists (`OD-01`). A settled
entry's rationale belongs there where rule R4 of
[`docs/adr/README.md`](../adr/README.md) admits it, and this file cites it by
`ADR-NNN`; everything R4 does not admit is carried here.

## Status legend

| Status | Meaning |
|---|---|
| **Settled** | Decided. The rationale and the rejected options are recorded in the entry |
| **Settled, with a residual** | Decided in substance. One narrow point remains, named in the entry with its owner and the document that settles it |
| **Open** | Not decided. The entry names the options and the owner |
| **Conflict** | Two requirements, or a requirement and a mandated constraint, cannot both be honoured. Not a choice: a defect owed to `specification-manager`, and the documents it blocks wait for the correction rather than being written around it |

## Index

| Entry | Subject | Status | Owner |
|---|---|---|---|
| [OD-01](#od-01--where-the-architecture-decision-records-live) | Where the architecture decision records live | Settled | — |
| [OD-02](#od-02--the-msrv) | The MSRV | Settled | — |
| [OD-03](#od-03--versioning-the-binary-the-document-the-cache-the-changelog) | Versioning: binary, document, cache, changelog | Settled | — |
| [OD-04](#od-04--one-package-or-a-workspace) | One package, or a workspace | Settled, with a residual | `technical-writer` |
| [OD-05](#od-05--the-module-decomposition) | The module decomposition | Open | `technical-writer` |
| [OD-06](#od-06--the-error-types-shape-and-the-exit-code-derivation) | The error type's shape and the exit-code derivation | Open | `technical-writer` |
| [OD-07](#od-07--help-the-parsers-renderer-or-tpls-own) | Help: the parser's renderer, or `tpl`'s own | Settled | — |
| [OD-08](#od-08--the-parsers-own-diagnostics) | The parser's own diagnostics | Open | `technical-writer` |
| [OD-09](#od-09--toml-the-read-path-and-the-write-path) | TOML: the read path and the write path | Settled | — |
| [OD-10](#od-10--cache-filenames-and-the-case-collision) | Cache filenames, and the case collision | Settled | — |
| [OD-11](#od-11--the-scope-of-the-async-runtime) | The scope of the async runtime | Settled | — |
| [OD-12](#od-12--how-six-phase-deadlines-are-enforced) | How six phase deadlines are enforced | Open | `technical-writer` |
| [OD-13](#od-13--the-engine-pin-and-minijinja-contrib) | The engine pin, and `minijinja-contrib` | Settled | — |
| [OD-14](#od-14--which-undefined-behaviour-the-engine-is-configured-with) | Which undefined behaviour the engine is configured with | Settled, with one observation owed | — |
| [OD-15](#od-15--the-template-loader) | The template loader | Open | `technical-writer` |
| [OD-16](#od-16--the-tls-backend-and-the-root-store) | The TLS backend and the root store | Settled | — |
| [OD-17](#od-17--observability) | Observability | Open | `technical-writer` |
| [OD-18](#od-18--serialisation-key-order-and-the-two-omissions) | Serialisation, key order, and the two omissions | Open | `technical-writer` |
| [OD-19](#od-19--whether-the-two-embeddings-are-materialised) | Whether the two embeddings are materialised | Settled | — |
| [OD-20](#od-20--edit-distance-and-the-other-small-algorithms) | Edit distance, and the other small algorithms | Settled, with a residual | `technical-writer` |
| [OD-21](#od-21--two-test-seams-that-must-not-be-on-the-published-surface) | Two test seams that must not be on the published surface | Settled, with a residual | `technical-writer` |
| [OD-22](#od-22--the-test-harness-and-the-fixture-certificate) | The test harness, and the fixture certificate | Settled, with a residual | `technical-writer` |
| [OD-23](#od-23--packaging-artefacts-and-the-musl-build-path) | Packaging, artefacts, and the musl build path | Settled | — |
| [OD-24](#od-24--the-discovery-boundary-and-the-process-uid) | The discovery boundary, and the process uid | Settled, with a residual | `technical-writer` |
| [OD-25](#od-25--the-clock-source-for-now) | The clock source for `now` | Open | `technical-writer` |
| [OD-26](#od-26--the-boundary-against-the-knowledge-graph) | The boundary against the knowledge graph | Settled | — |
| [OD-27](#od-27--seed-benchsql-and-wl-001) | `seed-bench.sql` and `WL-001` | Settled | — |
| [OD-28](#od-28--the-release-profile-against-the-caught-panic-condition-of-70) | The release profile against the caught-panic condition of `70` | Open | `technical-writer` |

Two editorial defects were reported at the end as `ED-01` and `ED-02`. Both
were corrected in the eighth edition; neither is outstanding.

## Verification note

Every version number and every library behaviour cited below was verified on
**2026-09-10** against the source named beside it — vendor documentation on
`docs.rs`, the crate index, the Rust Edition Guide, or a file of this
repository. Anything not verified says so in its own text. No claim rests on
recollection.

The three entries the eighth edition settled — `OD-21`, `OD-22`, `OD-24` —
were re-read against the corpus as it stands at commit `9efa791` on
**2026-09-11**, requirement by requirement. Every identifier they cite was
confirmed to exist in that corpus.

---

## OD-01 — Where the architecture decision records live

**Status: settled.**

**Decision.** The register lives at `docs/adr/` and is cited by `ADR-NNN`. The
convention and the authoritative index are documented in
[`docs/adr/README.md`](../adr/README.md), and are **not restated here**, per
rule R3 of that document.

---

## OD-02 — The MSRV

**Status: settled.**

**Decision.** **MSRV = 1.85.0.** The measured toolchain, rustc/cargo 1.98.0,
remains the development toolchain and is not the pin.

**Rationale.** 1.85.0 is the release that stabilised edition 2024 (Rust Edition
Guide, *Rust 2024*, verified 2026-09-10), which `CLAUDE.md` fixes as the
edition. It is therefore the hard floor, and pinning to the floor is the widest
compatibility the edition permits. `CLAUDE.md` deferred the number explicitly —
"MSRV a fixar no `Cargo.toml`" — and this settles it.

**Rejected.** Pinning to the measured 1.98.0, which would forbid every
toolchain between the floor and today for no stated gain; and following stable,
which is not a pin at all and would make the floor a fact nobody records.

**Not verified.** The MSRV each dependency declares. minijinja states `1.63+`
(docs.rs, minijinja 2.24.0), which is below the floor and therefore not
binding; the floors of `clap` 4.6.6, `sqlx` 0.9.0, `tokio` 1.53.1, `toml`
1.1.5 and `toml_edit` 0.25.13 were not checked. If one of them exceeds 1.85.0
the pin rises to it, and the entry is amended with the source.

---

## OD-03 — Versioning: the binary, the document, the cache, the changelog

**Status: settled.**

**Decision.** Four numbers, four rules.

| Number | Value now | Rule |
|---|---|---|
| Binary version | `0.1.0` | Full semantic versioning. While below 1.0, a breaking change is a **minor** bump |
| `schema_version` | `1` | Versions the document contract, independently of the binary, per `FR-OUT-011`; what breaks it is `FR-OUT-014` |
| `cache_format` | `1` | Versions the on-disk arrangement only, independently of `schema_version`, per `FR-CDOC-002` and `FR-CDOC-005` |
| Changelog | `CHANGELOG.md` | Keep a Changelog format |

`tpl.version` and the output of `tpl version` both come from
`CARGO_PKG_VERSION`, so the number has **one source**.

**Rationale.** `FR-HELP-005` gives `tpl 0.1.0\n` as its worked case and
`FR-CTX-027` carries the same string, so `0.1.0` is the version the corpus
already assumes. Reading it from `CARGO_PKG_VERSION` is what makes
`FR-HELP-005` and `FR-CTX-027` incapable of disagreeing: a second literal
would be the copy that stops being true. Pre-1.0 breaking changes as minor
bumps is the semver convention for a version below 1.0 and needs no local
rule. `FR-ENV-029` requires a template-surface removal to be recorded in a
changelog and `CLAUDE.md` names one in its workflow, so the format is the only
open part, and a named public convention is preferable to a local one.

**Rejected.** Two literals for the version, one in the help and one in the
context, which is the duplication `FR-CTX-027` would otherwise permit; and
bumping `schema_version` or `cache_format` together, which `FR-CDOC-005`
forbids outright — "Neither SHALL be incremented on account of a change that
affects only the other."

---

## OD-04 — One package, or a workspace

**Status: settled, with a residual.**

**Decision.** **One Cargo package**, carrying a library and a binary. The logic
lives in the library and is testable without launching a process; the binary
parses, dispatches, and maps the error to an exit code.

**Rationale.** It is what `CLAUDE.md` *Estrutura do Projecto* describes — one
manifest at the root, `src/` beneath it — and what *Organização* requires of
the split. `DIV-032` removes the usual reason to split further: the library
carries no compatibility guarantee, so there is no published crate boundary to
protect.

**Rejected.** A workspace of several crates, which buys a boundary nothing
consumes and multiplies the manifests that must agree on the release profile
and the MSRV.

**Residual, open under `technical-writer`.** Whether `benches/` needs a package
of its own to keep the measurement crates — `criterion`, `dhat` — out of the
shipped dependency graph. `CLAUDE.md` *Orçamento de dependências* requires
knowing what each dependency drags in, and a dev-dependency of the same package
still appears in `cargo tree`. To be settled in `operations`, with the
dependency graph of both arrangements recorded.

---

## OD-05 — The module decomposition

**Status: open. Owner: `technical-writer`.**

**Question.** `CLAUDE.md` names `main.rs`, `cli/`, `project/`, `mariadb/`,
`model/`, `render/` and `error.rs`. Six responsibilities have no home among
them. Where does each live?

| Responsibility | Requirements | Homes visible |
|---|---|---|
| The catalogue cache | `FR-CACHE-*`, `FR-CDOC-*` | a `cache/` module; under `project/`; under `mariadb/` |
| Output formatting, the envelope, `text` layout, escaping | `FR-OUT-*` | an `output/` module; under `cli/`; under `model/` |
| Help text and the typed examples and exit-codes table | `FR-HELP-022` | under `cli/`; a `help/` module |
| The privilege cross-checks | `FR-PRIV-011`, `FR-PRIV-017`, `FR-PRIV-019` | under `mariadb/`; under `model/` |
| The configuration reader and writer | `FR-CONF-*`, `FR-CFG-*` | under `project/`; a `config/` module |
| The diagnostic renderer and the suggestion machinery | `FR-ERR-008`, `FR-ERR-019` | under `error.rs`; a `diag/` module |

**Constrained by.** `CLAUDE.md` *Convenções de Código Rust*: `snake_case`, no
module name repeating its parent, `foo.rs` beside `foo/` and never `mod.rs`,
and the visibility ladder private → `pub(super)` → `pub(crate)` → `pub`.

**Still to settle, with written rationale.** All six placements, and whether
`cli/` holds one module per porcelain command that delegates the work, or the
work itself.

**Blocks.** `architecture`, `interfaces`.

---

## OD-06 — The error type's shape and the exit-code derivation

**Status: open. Owner: `technical-writer`.**

**Question.** One error enum or several? How is the exit code obtained from an
error value, and how are the four labelled lines built?

**Visible from the repository.**

- `FR-ERR-001` fixes ten codes; `FR-ERR-002` forbids collapsing two conditions onto one code where the caller's next step differs.
- `FR-ERR-034` fixes, **per code**, what the `cause` line must name, and bans a `cause` "whose wording would be equally true of a different failure". The error value must therefore carry the instance, not the category.
- `FR-ERR-015` is withdrawn, and its *Rejected* note refuses "retaining `kind` as an internal taxonomy with no external carrier", on the ground that a classification nothing outside the process can observe cannot be tested.
- `CLAUDE.md`: `thiserror` in the library, `anyhow` in the binary; `#[non_exhaustive]` on public enums.
- `FR-ERR-030`: `70` is produced by a caught top-level panic and by a detected invariant violation. Whether the first condition survives the release profile is `OD-28`, which carries `DIV-045`; this entry takes that answer as given and decides only how the error value yields the code.

**Still to settle, with written rationale.** One enum against per-module enums
composed by `From`; whether the exit code is a method on the error or a table
in the binary; whether the four lines come from `Display` or from a separate
renderer, which `FR-ERR-024`'s escaping and `FR-ERR-022`'s character set both
argue for; and whether the discriminant an exit-code mapping needs is the
taxonomy `FR-ERR-015` rejected or is distinguishable from it.

**Blocks.** `interfaces`, `architecture`.

---

## OD-07 — Help: the parser's renderer, or `tpl`'s own

**Status: settled.**

**Decision.** **`tpl` renders all seven sections itself**, with
`disable_help_flag` and `disable_help_subcommand` set on the parser. `clap` is
kept for parsing and for the tree introspection `FR-HELP-021` requires.

**Rationale.** Help is contract: `FR-HELP-006` fixes seven sections in a fixed
order, two of which — `EXAMPLES` and `EXIT CODES` — no argument parser
generates; `FR-HELP-009` and `FR-HELP-010` fix an 80-column layout with the
breaks written into the text, forbid reading `COLUMNS`, and forbid reflow; and
`FR-HELP-002` with `BR-HELP-001` require `tpl help <path>` and
`tpl <path> --help` to be **byte-identical at every depth**. Rendering it in
`tpl` makes all four properties the project's own to hold. `FR-HELP-022`
already requires the examples and exit codes to come from a typed table
indexed by command path, so the renderer has its source.

**Rejected.** `clap`'s `help_template` with `after_help` carrying the two extra
sections. It makes a contract output depend on the parser's renderer staying
byte-stable across versions — a dependency upgrade could then change a
contract output with nothing in this project having changed. It is the same
argument `FR-CONF-037` makes against inheriting a driver's TLS default.

**Composition.** `clap`'s `color` feature is on by default and must be off:
`NFR-DET-004` forbids an ANSI escape sequence on either stream. Its terminal
wrapping lives behind the optional `wrap_help` feature, which is left off (clap
feature-flags documentation, clap 4.6.6, verified 2026-09-10). `OD-08` remains
open for the parser's error messages, which this decision does not reach.

---

## OD-08 — The parser's own diagnostics

**Status: open. Owner: `technical-writer`.**

**Question.** Are `clap`'s error messages intercepted and re-rendered, or are
its diagnostic features disabled?

**Visible from the repository, and verified.**

- `FR-ERR-008` fixes four labelled lines; `FR-ERR-033` makes them the whole of what a caller receives.
- `FR-ERR-019` and `FR-ERR-020` fix the suggestion rule; `FR-ERR-022` and `FR-ERR-023` restrict a runnable hint to literals and `[A-Za-z0-9_]{1,64}` and refuse a candidate outside that set entirely.
- `FR-CLI-014` requires a repeated single-value flag to be `64` **naming both values**.
- `clap`'s default features include `error-context` and `suggestions`, and `color` (clap feature-flags documentation, verified 2026-09-10). Its message shape cannot be emitted as it stands.

**Still to settle, with written rationale.** Intercept `clap::Error` and
re-render from its kind and context, against turning `error-context` and
`suggestions` off and producing every diagnostic in `tpl`. The second forfeits
the context that would let a re-rendering name both values of a repeated flag;
whether the parser exposes enough to do that at all was **not verified**.

**Blocks.** `interfaces`.

---

## OD-09 — TOML: the read path and the write path

**Status: settled.**

**Decision.** **Read path `toml` with `serde`; write path `toml_edit` 0.25**,
which preserves comments, spacing and the relative order of items.

**Rationale.** Six requirements make format preservation a functional need, not
a nicety. `FR-PROJ-017` and `FR-PROJ-018` put a commented-out `[database.*]`
entry in the `.cfg` that `tpl init` writes, so that "the correct shape is in
front of the reader without a trip to documentation"; `FR-CFG-020` requires
`database update` to change the named fields "leaving the rest of the entry
untouched"; `FR-CFG-013` prints the file "literally". The `toml` crate is
serde-oriented and its own documentation points elsewhere for this — "For
format-preserving editing or finer control over output, see `toml_edit`" (`toml`
1.1.5+spec-1.1.0, docs.rs, verified 2026-09-10) — while `toml_edit`
"allows you to parse and modify toml documents, while preserving comments,
spaces *and relative order* of items" (`toml_edit` 0.25.13+spec-1.1.0,
docs.rs, verified 2026-09-10). Splitting the paths keeps the serde mapping,
which `FR-CONF-002`'s typed key space wants on the read side, and keeps the
comments, which the write side must not destroy.

**Rejected.**

- **`toml_edit` for both paths.** It loses the serde mapping, so the fifteen typed keys of `FR-CONF-002` would be validated against a document tree by hand — more code on the path that reads untrusted input, which `BR-CONF-004` is written to keep small.
- **`toml` alone**, which is what `CLAUDE.md`'s stack table names. The first `tpl cfg set` would delete the commented example that `FR-PROJ-018` requires the file to carry, so a stated requirement would stop holding on the second invocation, silently.

**Consequence.** The `Stack` table of `CLAUDE.md` gains `toml_edit`. That table
is an architecture decision by that file's own terms, and the change is
prepared for the user rather than made here.

---

## OD-10 — Cache filenames, and the case collision

**Status: settled.**

**Decision.** A cached object's path is the **catalogue object kind plus the
literal object name**, exactly as `FR-CACHE-001` and `FR-CDOC-014` already
shape them: `tables/<name>.json`, `views/<name>.json`,
`routines/<kind>.<name>.json`. **No encoding layer and no hash.**

On a **case collision** — two objects of one kind whose names differ only in
case, mapping to one path on a case-insensitive filesystem — `cache refresh`
**detects it and fails, naming both objects**.

**Rationale.** `FR-CDOC-014` already fixes one of the three path forms
literally, so the other two follow the same rule and there is one naming rule
rather than two. The collision is real rather than theoretical: MariaDB permits
two such objects on a case-sensitive filesystem, and two of the four targets of
`NFR-PERF-018` are macOS, whose default filesystem is case-insensitive.
Detecting and failing is the only outcome consistent with the stance the
functional corpus takes everywhere else — a wrong answer wearing the appearance
of a right one is the failure it works hardest to prevent, per `BR-CDOC-002`
and `BR-SEM-004`. Silently serving one object's bytes under the other's name
would be exactly that.

**Rejected.**

- **A disambiguating suffix** on one of the two colliding names. It creates two naming rules where `FR-CDOC-014` fixed one, and the suffix would then have to be derivable by every reader of the cache.
- **Documenting the limitation** and carrying on. MariaDB permits such schemas on Linux, so the failure would be reachable and silent, which is the class of defect this project refuses.

**Still to record in `data-model`.** Whether a cached object file carries the
envelope of `FR-OUT-024` or a bare object. The `.json` suffixes of
`FR-CDOC-001` and `FR-CDOC-014` fix the encoding as JSON; they do not fix the
outer shape.

---

## OD-11 — The scope of the async runtime

**Status: settled.**

**Decision.** **The process is synchronous.** The current-thread `tokio`
runtime is built **lazily inside `mariadb/`**, with `block_on` at that
boundary. Nothing outside that module is async.

**Rationale.** `NFR-PERF-005` requires four commands — `tpl init`, every form
of `help`, every form of `version` — to perform no discovery, read no
configuration file and open no connection, and requires it to be verified from
**outside** the process: no `stat` of an ancestor, no open of `.tpl/.cfg`, no
socket. A runtime built for every invocation is startup work that the invoked
command did not need, which is what `CLAUDE.md` forbids under "Inicialização
preguiçosa por defeito". Confining the runtime to the one module that needs it
also keeps `NFR-PERF-004` — at most one connection — local to the component
that opens it.

**Rejected.** A `#[tokio::main(flavor = "current_thread")]` attribute over the
whole entrypoint. It starts a runtime for `tpl --version` too, and cannot
satisfy `NFR-PERF-005`'s outside-the-process observation as directly: the
absence of runtime setup then has to be argued rather than observed.

**Composition.** The driver decision is unchanged and rests on measurement:
`sqlx` 0.9.0 with `tokio` 1.53.1 on a current-thread runtime, `mysql` 28.0.2
rejected, per `FR-CONF-036` and `BENCHMARKS.md` (2026-09-10). `BENCHMARKS.md`
also records an unexplained 44 ms blocking cost in `connect_with` on musl over
raw loopback, tracked as roadmap task `#8`; it bears on `OD-12`.

---

## OD-12 — How six phase deadlines are enforced

**Status: open. Owner: `technical-writer`. Flagged as the entry likeliest to
turn out to be a requirement that cannot be met as written.**

**Question.** `FR-CONF-005` requires a deadline on six distinct phases, and
`FR-ERR-034` requires the `cause` to name **which** of DNS resolution, TCP
connect, TLS handshake or catalogue query failed. How are the phases
separated, and how is a deadline applied to a synchronous template render?

**Visible from the repository.**

- `FR-CONF-005`: deadlines on DNS resolution, TCP connect, TLS handshake, catalogue query, `password_command`, and render.
- `FR-GLOB-012`: a phase ends at the first of its own deadline and what remains of the overall budget, measured from process start.
- `FR-GLOB-013` and `FR-ERR-027`: the exit code is the phase's — `69`, `78` or `65`.
- `FR-ERR-034`, row `69`: the `cause` names the phase that failed, the host and port attempted, and what that phase returned.
- `FR-CONF-005` note with `NFR-PERF-018`: a static `musl` build resolves names differently, and "a name that did not resolve is not a host that refused a connection".
- `FR-RND-033`: exceeding the render deadline is `65`. Whether the template engine offers any timeout or fuel mechanism was **not verified**.

**Why it may be unmeetable.** A driver `connect` covers resolution, connection
and handshake in one call, so naming which of the three failed may not be
obtainable without `tpl` resolving and connecting itself and handing the driver
a socket — and a synchronous render has no obvious interruption point at all.
If either turns out to be unobtainable it is a defect owed to
`specification-manager`, not a design to be invented here.

**Still to settle, with written rationale.** Whether the connect phase is
decomposed and how; and how a render is bounded — a watchdog, a cooperative
check inside a registered filter, or a stated limit.

**Blocks.** `architecture`, `interfaces`, `quality-attributes`.

---

## OD-13 — The engine pin, and `minijinja-contrib`

**Status: settled. Recorded in [`ADR-001`](../adr/adr-001-template-engine-pin.md).**

**Decision.** Pin the stable line of `minijinja` named by `ADR-001`, and retain
`minijinja-contrib` on the same line. Its filters are therefore **group 3** of
`FR-ENV-019` — available, and guaranteed by nobody.

The version number, the rationale, the options rejected, and the two deliberate
shadowings of engine built-ins are recorded in `ADR-001` and are **not restated
here**, per rule R3 of [`docs/adr/README.md`](../adr/README.md). `FR-ENV-003`
requires the pin to live in an architecture decision record and to be cited from
there; `ADR-001` is that record.

---

## OD-14 — Which undefined behaviour the engine is configured with

**Status: settled, with one observation owed.**

**Decision.** `UndefinedBehavior::Strict`.

**Rationale.** `FR-SEM-012` requires a render to fail when a template reads a
field that does not exist, and `FR-PRIV-016` states the cost of that rule in
its own text: `{% if table.restricted %}` "fails on every complete table".
Of minijinja's four variants — `Lenient` (the default), `Chainable`,
`SemiStrict`, `Strict` — only `Strict` fails a truthiness test on an undefined
value; `Lenient` and `SemiStrict` both permit `{% if missing %}`, and
`Chainable` permits the attribute access as well (docs.rs
`minijinja::UndefinedBehavior`, verified 2026-09-10). `Strict` is therefore the
only variant consistent with the consequence the functional corpus already
accepted, and `FR-SEM-012` is load-bearing for three requirements elsewhere —
`FR-CTX-034`, `FR-PRIV-016` and `FR-ENV-017` — so it may not be weakened by
the choice of setting.

**Rejected.** `Lenient` and `SemiStrict`, which contradict `FR-PRIV-016`'s
stated cost; `Chainable`, which permits the very field access `FR-SEM-012`
fails and would turn a misspelled field into an empty string in a generated
file.

**Observation owed against the pin, and it must be recorded before
`architecture` asserts it.** That a **defined** `null` interpolates as the
empty string under `Strict`, rather than failing, as `FR-SEM-010` requires and
`FR-SEM-011` reinforces by forbidding the words `none` and `null` in the
output. `Strict` governs *undefined* values, and a defined null is a different
thing — but the distinction has **not been verified** against minijinja 2.24,
and the two requirements are contradicted outright if it does not hold.

---

## OD-15 — The template loader

**Status: open. Owner: `technical-writer`.**

**Question.** Is the engine given the stock path loader, or a loader written
for the containment rules?

**Visible from the repository, and verified.**

- `FR-TMPL-023`: the template root is the boundary of every lookup.
- `FR-TMPL-024`: a symbolic link inside `templates/` is refused — not listed, shown, checked, rendered, or included.
- `FR-TMPL-025` and `FR-TMPL-026`: every resolved path is canonicalised and re-checked against the root, and an escape is `65`.
- `FR-TMPL-008`: inside a template a name is literal, so `tpl` adds no resolution layer the engine does not have.
- `FR-SEC-017` names the threat: `ln -s ../.cfg .tpl/templates/leak.jinja`.
- minijinja's `path_loader` is documented as refusing names that start with a dot or sit in a dot-directory. The documentation does **not** state that it rejects `..`, an absolute path, or a symlink (docs.rs `minijinja::path_loader`, verified 2026-09-10). That is a gap in the documentation, not a verified behaviour.

**Still to settle, with written rationale.** Wrap `path_loader` with the checks
in front of it, against writing a loader that performs them; and where the same
checks live for the three `template` subcommands, which resolve paths without
the engine.

**Blocks.** `architecture`, `security`.

---

## OD-16 — The TLS backend and the root store

**Status: settled. Recorded in [`ADR-002`](../adr/adr-002-tls-mode-mapping.md).**

**Decision.** Bundled `webpki-roots` trust anchors, and the mapping of the five
modes of `FR-CONF-013` onto the driver's five named variants.

The mapping table, the trust-anchor rationale, the options rejected, and the
composition with `FR-CONF-037` and `FR-CONF-039` are recorded in `ADR-002` and
are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). `FR-CONF-038` requires the mapping to
live in an architecture decision record and to be cited from there; `ADR-002` is
that record.

---

## OD-17 — Observability

**Status: open. Owner: `technical-writer`.**

**Question.** How is the diagnostic stream produced, and how is the prohibition
on six content categories enforced by structure rather than by review?

**Visible from the repository, and verified.**

- `FR-GLOB-014` and `FR-GLOB-015`: three `-v` levels mapping to `INFO`, `DEBUG`, `TRACE`, saturating; `-q` is errors only.
- `FR-GLOB-017`: at `INFO`, which phases ran and how long each took, and **exactly one line per catalogue query, distinguishable from every other diagnostic line**; at `DEBUG`, each cache hit and miss.
- `NFR-PERF-008`: that line is what makes the query count observable from outside the process, which is how `NFR-PERF-001` and `NFR-PERF-002` are checked at all.
- `NFR-DET-001`: stderr is neither deterministic nor contract — so the line must be stable enough to count and free enough to change.
- `FR-GLOB-018`: six categories never appear at any level, the raw driver error among them.
- `CLAUDE.md`: `tracing` 0.1.44 with `tracing-subscriber` 0.3.23 (crates.io, verified 2026-09-10).

**Still to settle, with written rationale.** A bespoke minimal layer against
`tracing-subscriber`'s `fmt` layer with a custom formatter; how the query line
is made distinguishable, and whether a test may depend on that while stderr
stays outside the contract; how `FR-GLOB-018` is made structural, given that a
raw driver error is one logging call away at every call site; and whether
`tracing` earns its place under the dependency budget for four levels and one
structured line.

**Blocks.** `operations`, `architecture`, `technology-stack` — the last because
whether `tracing` earns its place is a dependency-budget question, and
[README.md](README.md#the-documents) already listed that document against this
entry.

---

## OD-18 — Serialisation, key order, and the two omissions

**Status: open. Owner: `technical-writer`.**

**Question.** Are the seventeen documents emitted by derived serialisation or by
a hand-written writer, and how are the two exceptions to "absent is `null`"
expressed?

**Visible from the repository, and verified.**

- `FR-OUT-013` and `FR-HELP-023`: fixed key order per structure, and no unordered map on the emitting path.
- `FR-OUT-012`: absent is `null` and never omitted, with exactly two exceptions — `FR-CFG-037`, where an unset key is absent from `cfg list`, and `FR-PRIV-016`, where `restricted` appears only on an incomplete object.
- `FR-OUT-007` and `FR-OUT-008`: compact by default; `--pretty` is a two-space indent, which is `serde_json`'s own default — "Construct a pretty printer formatter that defaults to using two spaces for indentation" (docs.rs `serde_json::ser::PrettyFormatter`, verified 2026-09-10).
- `FR-OUT-017`: invalid UTF-8 becomes U+FFFD, so catalogue values are read as bytes and converted lossily.
- `FR-OUT-018`: C0 escaping over every interpolated value in read output — a transformation on the way out, not a property of the model.
- `FR-CTX-026` and `FR-CFG-037` are maps whose order no requirement fixes; `serde_json`'s default map is ordered by key unless `preserve_order` is enabled.
- `serde` 1.0.229, `serde_json` 1.0.151 (crates.io, verified 2026-09-10).

**Still to settle, with written rationale.** Derived `Serialize` against a
bespoke writer — the C0 escaping and the mid-document pipe state of
`FR-ERR-026` both argue for a writer wrapper; whether `preserve_order` is
enabled, and what order `vars` and the `cfg list` document take; whether the
two omissions use `skip_serializing_if` or a distinct type; and whether `serde`
becomes a public dependency of the library, which `DIV-032` names as an
architecture decision.

**Blocks.** `interfaces`, `data-model`.

---

## OD-19 — Whether the two embeddings are materialised

**Status: settled.**

**Decision.** **Materialise them.** The object graph **is** the document.

**Rationale.** `FR-CTX-006` embeds the referenced table one level deep with its
columns, indexes and primary key in full, and `FR-CTX-010` embeds the
referencing table to the same depth; `FR-CTX-009` cuts both at the first hop,
so no traversal can fail to terminate. A model whose shape is the document's
shape has one representation to get right, and `FR-SCH-022`'s round-trip —
dump, feed back through `--context`, render byte-identically — is a property of
that one representation rather than of an emitter that reconstructs it.

**The consequence, stated openly.** Peak resident memory scales with the
quadrupled column volume: `BR-CTX-001` records that `FR-CTX-006` alone "roughly
doubles the column volume" over 200 tables with 180 foreign keys, and
`FR-CTX-010` records that it "doubles it again". The provisional `< 32 MiB`
figure of `NFR-PERF-014` over `WL-001` is therefore **the figure most likely to
be superseded upward by the first real measurement** — exactly as `FR-CTX-010`
anticipates, and exactly what `NFR-PERF-019` exists to permit without the
figure having been a limit in the meantime.

**Rejected.**

- **Emit by reference** at serialisation time, keeping one owned copy of each table. The bytes would be identical and the memory lower, at the price of an emitter that has to reproduce a one-hop cut correctly in both directions, including the self-reference and cycle cases `FR-CTX-009` enumerates. A defect there is a wrong document at exit `0`.
- **Streaming the dump**, so that peak memory tracks the largest table rather than the whole document. `FR-SCH-016` makes the dump one document and `FR-CTX-023` promises referential integrity over it, so the whole model has to be in hand before the first byte can be trusted.

---

## OD-20 — Edit distance, and the other small algorithms

**Status: settled for the distance, with a residual.**

**Decision.** **Damerau-Levenshtein, hand-rolled, no dependency.**

**Rationale.** A transposition is one error, and it is the commonest typing
mistake; counting it as two misreports the caller's actual distance from the
name they wanted. This is **observable output**, so the worked example belongs
in the specification rather than in a comment:

| Supplied | Candidate | Damerau-Levenshtein | Plain Levenshtein |
|---|---|---|---|
| `ordres` | `orders` | **1** | 2 |

`FR-ERR-019` admits candidates "within an edit distance of two", so the choice
changes the candidate set as well as its order: under Damerau-Levenshtein more
names qualify, and a transposed name ranks above a name one substitution away.
Hand-rolled and with no dependency because `CLAUDE.md` *Orçamento de
dependências* prefers `std` and refuses a crate used for one function, and
because `BR-PERF-004` makes this a budgeted path — 200 comparisons for a `66`
over `WL-001` — so the implementation must be ours to measure.

**Rejected.** Plain Levenshtein, which reports the commonest typo as two
errors and would drop a transposed name from the candidate set whenever a
second error is present; and a distance crate, for the dependency budget.

**Residual, open under `technical-writer`.** Four further specified algorithms
were not reached by this decision and each needs its own rationale in
`interfaces`: the `LIKE` matcher of `FR-SCH-012` through `FR-SCH-014`; the
POSIX word splitting of `FR-CONF-025`; the word-list tokeniser of
`FR-ENV-030`, which has a published eight-row vector; and the ASCII-only case
folding of `FR-ENV-031` and `FR-SCH-014`, where a locale-aware or Unicode
folding would be wrong.

---

## OD-21 — Two test seams that must not be on the published surface

**Status: settled by the eighth edition, with a residual. Owner of the
residual: `technical-writer`.**

**What the conflict was.** `FR-ERR-031` required a deliberate trigger for `70`
and `FR-SRV-035` a seam that presents the reader with a series above its own
window, both barred from every help text and from both command trees — while
`BR-ERR-001` demanded an **integration** test per exit code and `FR-SRV-035`
an assertion on the exit code, neither observable without running the binary.
Every mechanism a running binary could reach collided with a requirement in
force.

**The resolution, made in the corpus and not here.** Both seams move **inside
the process**, reachable from nothing a caller can write. The four requirements
`BR-ERR-001` enumerates hold unchanged — `FR-CLI-002`, `FR-CLI-021`,
`FR-HELP-021`, `NFR-PERF-018`, to which `FR-ERR-031`'s own table adds
`FR-CLI-023` and `NFR-DET-001` — and the two that yield say so in their own
text.

| Requirement | What it now fixes |
|---|---|
| `FR-ERR-031` | The trigger is reachable **only from within the system's own test configuration**, is reachable from **no invocation of the binary the project distributes**, and appears in no help text, in the JSON tree of `FR-HELP-016`, or in the tree of `FR-CLI-002`. The requirement enumerates the three rejected candidates and the requirement each collides with |
| `BR-ERR-001` | Yields **for `70` alone**, stated in its own text: `70` is exercised in process through the trigger and **not** by an integration test. The nine other codes are unchanged |
| `FR-SRV-035` | Yields the assertion on the exit code. The test asserts that the read completes without error and that `standing` is `newer_than_supported`; the seam is the one `FR-ERR-031` names. Building an impostor server to make the observation external is rejected in the requirement |
| `BR-SRV-003` | States what it reaches: the three promises about what the process **sends** (`FR-SRV-012` … `FR-SRV-014`), and not `FR-SRV-035`, which is a promise about what the reader **emits** |

**What this decides for `verification`.** Two tests, both in process, each
described with the limit its own requirement states rather than a limit this
folder invents: what is executed is the guard, and separately the step from an
error condition to an exit status — `BR-CLI-004` for `FR-SRV-035`, and the nine
integration-tested codes for `FR-ERR-031`. The composition of the two is
reasoned rather than executed, and both requirements say so. `verification`
cites that limit and does not restate it.

**No longer part of this entry.** The tension between `FR-ERR-030`'s caught
panic and an aborting release profile is not a seam question, and the eighth
edition recorded it as `DIV-045` rather than resolving it. It is carried in
`OD-28`. `FR-ERR-031`'s trigger exercises the **other** producing condition of
`70` — the detected invariant violation — so the exception `BR-ERR-001` grants
does not depend on that answer.

**Residual, open under `technical-writer`.** Which in-process construct
realises a seam that the project's own tests reach and the distributed binary
does not — for both seams, since `FR-SRV-035` now points at `FR-ERR-031`'s —
and, following from it, which test kind each of the two tests is. To be
settled in `verification`, with the construct's visibility from each test kind
verified against the toolchain's own documentation rather than assumed.

**Blocks.** `verification`.

---

## OD-22 — The test harness, and the fixture certificate

**Status: settled by the eighth edition, with a residual. Owner of the
residual: `technical-writer`.**

**What the conflict was.** `FR-CONF-013` defaults `tls` to `verify-identity`,
and `FR-CONF-038` recorded that `tpl` with default configuration cannot reach
the fixture of `scripts/mariadb/` over TCP on any series. The default mode was
therefore the one cell of a ten-cell table with no acceptance test, and the
fixture could not supply one.

**The resolution.** `FR-CONF-038` now states the fixture obligation as a
requirement: the fixture **SHALL** be able to present, at each series of
`FR-SRV-015`, a server whose certificate names the host by which the project's
tests reach it, and **SHALL** retain a server that offers no TLS. Two options
are rejected in the requirement's own text — dropping the acceptance test and
stating the cost, because the default is the mode a caller meets without asking
for it; and configuring the certificate on the three TLS-capable series alone,
because `FR-SRV-029` requires the test against every series and `10.11` is
supported until 2028-02-16. None of the ten cells changed, and
`verify-identity` is not relaxed: a certificate naming the host is what the
mode always required.

**What the requirement hands to this folder.** `FR-CONF-038` states that how
the certificate is generated, where the fixture keeps it, and how the no-TLS
server is retained beside it "are the fixture's own work and are not specified
here". The distance to be covered is recorded in the fixture's own
documentation: `scripts/mariadb/README.md` gives four images, four containers,
four host ports and one build per series, records that `10.11` reports
`have_ssl=DISABLED` and needs `--skip-ssl` over TCP, and that the other three
present the self-signed certificate MariaDB generates automatically — which
`FR-CONF-038` observed to carry no `subjectAltName`.

**What the harness must still serve, unchanged by the resolution.**

- `CLAUDE.md`: validation needing a database uses the containers of `scripts/mariadb/` — never mocks, never external instances — launched before and stopped after; `scripts/mariadb/README.md` adds "Leave no container running after a validation run."
- `FR-SRV-029`: the cross-series equivalence test runs against every series, and the refusal test against at least one series outside the window.
- `NFR-PERF-007` and `BR-SRV-003`: nine properties are verified from **outside** the process — the statements the server receives, the connections it accepts, the files the process opens.
- `FR-SRV-012`: the closed statement list is checked by observing what the server actually receives, expecting four kinds and no fifth, with the three connection-start statements issued once each in the stated order.
- `BR-SEC-003`: the sentinel test runs every command of the tree at maximum verbosity and asserts the sentinel appears in no byte of either stream.

**Residual, open under `technical-writer`.** Two halves, each with its own
document.

| Half | Settled in |
|---|---|
| How the fixture satisfies `FR-CONF-038`: how a certificate naming the host is produced and kept for each series, and how a server offering no TLS is retained beside it | `operations` |
| How the four containers are driven, whether server-dependent tests are gated, and how each outside-the-process observation above is instrumented | `verification` |

**Blocks.** `verification`, `operations`.

---

## OD-23 — Packaging, artefacts, and the musl build path

**Status: settled.**

**Decision.** **`cargo-zigbuild` for the two `musl` targets, native builds for
the two Darwin targets** — reproducing what `BENCHMARKS.md` measured. **No
continuous integration for now**: the targets and the toolchain are described,
no pipeline is prescribed, and CI's absence is recorded as future work.

**Rationale.** `NFR-PERF-018` fixes the four targets and the static `musl`
linkage; `DIV-041` records that the `gnu` triples are not targets.
`BENCHMARKS.md` names the reason a cross toolchain is needed at all — "the
Apple linker cannot emit ELF" — and records the combination that produced the
measured artefacts: `cargo-zigbuild` 0.23.4 with zig 0.16.0. Reproducing the
measured path means a later measurement is comparable with the recorded one,
which `NFR-PERF-012` requires of every baseline. Prescribing a pipeline before
one exists would put a description of something imaginary in a specification
whose whole discipline is to describe what is true today.

**Rejected.** `cross` or a container build, either of which may be right later
but neither of which produced the recorded figures; and prescribing a CI
pipeline now, which would be aspiration rather than specification.

**Recorded as future work.** There is no CI. Four consequences follow and each
is currently carried by whoever runs the pipeline by hand: `CLAUDE.md`'s
five-command validation sequence, `NFR-PERF-018`'s "no target is second class"
across four targets, `NFR-PERF-017`'s no-regression rule, and `FR-SRV-019`'s
release gate on the supported-series table. The release artefact itself — bare
binary, archive, checksums, signature — is not fixed by this entry.

---

## OD-24 — The discovery boundary, and the process uid

**Status: settled by the eighth edition, with a residual. Owner of the
residual: `technical-writer`.**

**What the conflict was.** `FR-PROJ-005` made the user's home directory a
boundary of project discovery. It can only be located from `HOME`, which
`FR-CLI-021` forbids reading to determine the location of the project, and a
shell exporting a different `HOME` falsified `BR-CLI-002`.

**The resolution.** `FR-PROJ-005` is the requirement that yields. The upward
walk stops at **the mount point alone**, and the boundary "SHALL be determined
without reading any environment variable". Three statements follow it.

| Requirement | What changed |
|---|---|
| `BR-CLI-002` | Gains the clause it was missing: nothing a shell can set may decide which project is discovered, which database entry is selected, or which server is reached |
| `FR-SEC-013` | The walk stops at the mount point; the threat it closes is narrower than the first edition claimed, and a `.tpl` planted in a world-writable ancestor is refused by `FR-SEC-014`, not by any boundary |
| `DIV-024` | Shortened by one clause: the correction owed to `CLAUDE.md` no longer names a home boundary |

Locating the home directory from the system's own account record rather than
from `HOME` was rejected in `FR-PROJ-005`'s own text: the boundary would then
sit wherever that record says, which need not be an ancestor of the working
directory, so the rule could silently never fire.

**Technical consequence.** No home-directory lookup remains anywhere in
discovery, so `std::env::home_dir` and every substitute for it leave the
implementation's path entirely. The one boundary that remains is
`std::os::unix::fs::MetadataExt::dev()`, compared between a directory and its
parent: in `std`, and no `unsafe`.

**The accepted cost, stated by `FR-PROJ-005` and inherited here.** A `.tpl`
folder above the caller's home directory and on the same filesystem — at
`/home`, at `/Users`, or at `/` — is now within the walk. Three things bound
it: such a directory is not ordinarily writable by the caller, so a `.tpl`
there is either the caller's own or is refused by `FR-PROJ-010`; `tpl init`
warns on stderr when it creates a project that shadows one above it, per
`FR-PROJ-016`; and `FR-PROJ-007` admits no fallback, so a walk that reaches the
boundary without finding a `.tpl` fails with `78` per `FR-PROJ-006` rather than
reading settings from anywhere else.

**Residual, open under `technical-writer`.** The process's own uid.
`FR-PROJ-010` requires `.tpl/.cfg` to be owned by the current user, so the
check compares two values: `std::os::unix::fs::MetadataExt::uid()` gives the
**file's** uid, while the process's uid comes from `getuid()`, which is
`unsafe` through `libc` — and `CLAUDE.md` forbids `unsafe` and requires
`#![forbid(unsafe_code)]`. A safe wrapper is needed and none is in the stack
table. To be settled in `technology-stack`, under the dependency budget, with
what the candidate drags in recorded. `FR-PROJ-011`'s mode check needs nothing
further: `MetadataExt` supplies `mode()` on the same metadata (`std` API
documentation, `std::os::unix::fs::MetadataExt`, Rust 1.98.1, verified
2026-09-11).

**Blocks.** `technology-stack`.

---

## OD-25 — The clock source for `now`

**Status: open. Owner: `technical-writer`.**

**Question.** `FR-CTX-028` requires an RFC 3339 timestamp in UTC with second
precision. Where does the calendar conversion come from, and does it also
parse?

**Visible from the repository.**

- `FR-CTX-028`: the form is exactly `2026-09-10T08:14:22Z` — UTC, `Z` offset, second precision. Its rationale rejects a structured value because no date filter exists.
- `FR-CTX-029`: evaluated once per invocation.
- `FR-ENV-025`: no template function may read a clock; `now` is a template's only source of time.
- `FR-CDOC-013`: `loaded_at` appears in `meta.json` and in `cache status`, in the same form — so the same conversion serves both, and the cache **reads one back**.
- `std::time::SystemTime` yields a duration since the Unix epoch; a civil date needs either arithmetic in the crate or a dependency. `CLAUDE.md` prefers `std` and refuses a crate used for one trivial function. No date crate is in the stack table.

**Still to settle, with written rationale.** Hand-rolled civil-from-days
arithmetic, which must also parse for `loaded_at`, against a date crate — and
if a crate, which, and whether two uses justify it.

**Blocks.** `technology-stack`, `architecture`.

---

## OD-26 — The boundary against the knowledge graph

**Status: settled.**

**Decision.** `docs/spec-technical/` becomes the **fourth row** of the
coordination file's sources-of-truth table, owned by `technical-writer`,
answering **how the system is built**. The knowledge graph stays **descriptive**
for code-level facts. The change to `CLAUDE.md` is **prepared for the user's
approval and is never made unilaterally**.

**Rationale.** `specification/README.md` hands the crate layout, the module
layout, the types and the library API to architecture without naming where
architecture lives, and two requirements — `FR-ENV-003` and `FR-CONF-038` —
already cite a home outside the functional corpus. A source of truth that is
not in the coordination table is a source of truth agents will not consult.
`CLAUDE.md` currently assigns *"Onde e **como**"* to the graph, which overlaps
this folder's scope, so the third row narrows as the fourth is added.

**The two rows, in the language of the file they are destined for.** For the
user to apply, or not:

```
| Knowledge Graph | **Onde** — que código existe, como se articula, e que requisito cada componente satisfaz | skill `knowledge-authority` |
| `docs/spec-technical/` | **Como** o `tpl` é construído — arquitectura, interfaces, dados, segurança, operação, qualidade | subagente `technical-writer` |
```

**The scope addition the user made, and its consequence.** The graph will also
represent the component architecture, the application flows, and
requirement-satisfaction edges, so that it answers *which component satisfies
`FR-X`* and *what breaks if I change this*. The architecture is therefore
carried twice — as prose here and as a graph model there — and **the two must
not drift**. The division is recorded in
[README.md](README.md#the-architecture-is-carried-twice): this folder
prescribes and traces to requirements; the graph describes what exists; where
they differ, both readings are reported and neither is silently corrected. A
graph fact is never cited here as authority for a decision.

**Rejected.** Leaving the technical specification out of the table, which
leaves two requirements citing a home no coordination document acknowledges;
and folding it into the graph's row, which would put a prescription and a
description under one owner and one scope — the confusion `CLAUDE.md` warns is
"o erro previsível".

---

## OD-27 — `seed-bench.sql` and `WL-001`

**Status: settled.**

**Decision.** `scripts/mariadb/seed-bench.sql` is written in a **later sprint —
the one that implements the catalogue reader**. Until then the five budgets that
depend on it are stated as **provisional**, in the vocabulary `NFR-PERF-019`
already provides.

**Rationale.** `WL-001` fixes the fixture's content — 200 tables, 2 400
columns, 600 indexes, 180 foreign keys, 40 generated columns, 25 triggers, 30
views, 40 routines, comments on 60% of the tables — and `BR-PERF-002` keeps it
separate from `seed.sql` on purpose, because one fixture serving both "would
hide an N+1, which is invisible at ten tables". `BR-PERF-007` already records
that it is the one file of `scripts/mariadb/` still absent, and that `WL-001`
is what needs it. Writing it beside the reader it measures is the point at
which an N+1 becomes detectable; writing it earlier produces a fixture nothing
can be run against. `NFR-PERF-019` and `NFR-PERF-020` are exactly the mechanism
for carrying a named, unmeasured budget without it being a limit, so
`quality-attributes` can state all nine budgets today.

**Rejected.** Writing it in this sprint, which produces a 200-table fixture
with no reader to exercise it and no measurement to validate it against; and
merging it into `seed.sql`, which `BR-PERF-002` forbids and which would make
the correctness suite pay for 200 tables on every run.

**Consequence to record.** `WL-002`'s byte scalar `N` cannot be computed until
the fixture exists, and `DIV-036` still owes `CLAUDE.md` one line of the
project-structure tree and one of the testing section naming the file.

---

## OD-28 — The release profile against the caught-panic condition of `70`

**Status: open. Owner: `technical-writer`.**

**Question.** `FR-ERR-030` makes **a panic caught at the top level of the
process** one of exactly two producing conditions of `70`, and `FR-ERR-032`
requires that `70` to carry the four labelled lines of `FR-ERR-008`. The
release profile `CLAUDE.md` fixes sets `panic = "abort"`, under which a panic
terminates the process abnormally: no code of `FR-ERR-001` reaches the caller
and no message is written. Which yields — the profile, or the requirement?

**Visible from the repository, and verified.**

- `DIV-045`, recorded in the eighth edition, states the contradiction and names both admissible resolutions: the profile leaves the panic path catchable, **or** `FR-ERR-030` is amended first — through `specification-manager` — to drop the caught-panic condition and to state the resulting limit in its own text. Leaving both statements standing is the one outcome it refuses, because a caller reading `FR-ERR-001` would branch on a code the binary cannot produce.
- `FR-ERR-030` is unchanged and carries a note pointing at `DIV-045`, because the specification precedes the implementation.
- `FR-ERR-031`'s in-process trigger exercises the **other** producing condition, the detected invariant violation, so `BR-ERR-001`'s exception for `70` stands whichever way this entry goes. See `OD-21`.
- `CLAUDE.md` fixes the profile as `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`, and makes a change to that table an architecture decision to be recorded before it is implemented.
- The cost of unwinding is **not measured**. `BENCHMARKS.md` records the same five settings for the artefacts it measured, so every figure in it was taken under an aborting profile and none of them separates the two.

**Still to settle, with written rationale.** Whether the profile unwinds — at a
cost in binary size and speed to be measured on the targets of `NFR-PERF-018`
before it is accepted, since no recorded figure separates the two profiles — or
whether the profile stands and the amendment `DIV-045` names is requested of
`specification-manager` first. The order is not interchangeable: an
implementation choice never narrows a contractual code without the corpus
saying so.

**Blocks.** `architecture`, `technology-stack`, `operations`.

---

## Editorial defects, reported and corrected

Two statements in `specification/` were stale when this register was written.
Both were corrected in the eighth edition, at commit `9efa791` of 2026-09-10,
and neither is outstanding. No document of this folder ever relied on either.

| Id | Where | Defect reported | Correction verified 2026-09-11 |
|---|---|---|---|
| **ED-01** | `glossary.md`, entry *DSN* | The form ended `[?params]`, which `FR-CONF-009` removed in the fifth edition and of which `FR-CONF-011` admits nothing | The entry gives `scheme://[user[:password]@]host[:port]/database` and states that it carries no query parameters, per `FR-CONF-011` |
| **ED-02** | `upstream-divergences.md`, `DIV-034` | The body said thirteen volatile catalogue fields were excluded by `FR-CAT-024` while its own seventh-edition note said sixteen | The clause reads sixteen, the note beside it records the growth from thirteen as history, and both agree with the sixteen rows of `FR-CAT-024`'s table |

Both identifiers are retired rather than deleted, and neither is reused: a
reader who meets `ED-01` or `ED-02` in the history is bounced here rather than
left hunting for an open defect.

## Order of work

Recomputed after the eighth edition settled the three conflicts. `OD-01` leaves
the table: the register it called for exists, and `ADR-001` and `ADR-002` are
accepted in it.

| Order | Entries | Why |
|---|---|---|
| 1 | `OD-28` | The only entry whose answer may change a requirement. `DIV-045` leaves two statements standing beside each other, and one of its two resolutions must reach `specification-manager` before any code is written against it |
| 2 | `OD-12` | Flagged as the likeliest to turn out to be unmeetable as written; the sooner it is examined, the sooner it becomes a conflict or a design |
| 3 | `OD-14`'s owed observation | A settled decision resting on one unverified engine behaviour, which `FR-SEM-010` and `FR-SEM-011` would contradict outright if it does not hold |
| 4 | `OD-05`, `OD-06`, `OD-08`, `OD-15`, `OD-17`, `OD-18`, `OD-25` | The remaining open entries, each with its options already enumerated |
| 5 | The residuals of `OD-04`, `OD-20`, `OD-21`, `OD-22`, `OD-24` | Narrow points inside settled entries, each naming the document that settles it |
