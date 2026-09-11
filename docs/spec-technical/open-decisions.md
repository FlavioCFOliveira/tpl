---
title: Decision Register
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md]
---

# Decision Register

## What this is

Twenty-eight entries, each a decision the repository could not settle on its
own. **All twenty-eight are settled. None is open.** Nineteen were settled by
the product owner in the interview of 2026-09-10, by the establishment of the
decision register, and by the eighth edition of `/specification`; the remaining
nine were settled on 2026-09-11, together with all five residuals the eighth
edition left inside settled entries. The fifth, `OD-22`'s, was work rather than
a decision: it was executed on 2026-09-11 by tasks #15 and #25, and the entry
records what each produced.

One obligation survives the settlement, and it is named in its own entry rather
than left to be inferred:

| Entry | What is owed | To whom |
|---|---|---|
| `OD-14` | An observation about a **defined** `null` under `UndefinedBehavior::Strict`, unverified against the engine pin | `technical-writer` |

**Two further obligations were discharged by the ninth edition of
`/specification`**, at commit `4ad5e8c` of 2026-09-11: `OD-28`'s amendment to
`FR-ERR-030`, and `OD-12`'s observation on the referent of `FR-CONF-004`.
Neither entry is reopened and neither history is dropped; each records what
landed. One obligation of `OD-12`'s survives inside that entry and falls to
`verification` rather than to this register, as `OD-08`'s does.

**No entry is a conflict.** The three that were — `OD-21`, `OD-22` and
`OD-24` — were resolved in the eighth edition, which read requirement against
requirement and named the requirement that yields in each case. `OD-28` was the
one entry whose settlement obliged the functional corpus to move, and it moved
first: the ninth edition amended `FR-ERR-030` before any implementation was
written against it, which is the order an implementation choice may never
invert when a contractual code is at stake.

Each settled entry records the decision, and with it either its rationale and
**the options rejected** or a citation of the record that carries them, for the
reason [`docs/adr/README.md`](../adr/README.md) gives for the same section in a
record.

The architecture decision record register now exists (`OD-01`). A settled
entry's rationale belongs there where rule R4 of
[`docs/adr/README.md`](../adr/README.md) admits it, and this file cites it by
`ADR-NNN`; everything R4 does not admit is carried here. **Eight entries are so
reduced today**, each naming its record in its own status line; `OD-01` cites
the register itself rather than a record.

## Status legend

| Status | Meaning |
|---|---|
| **Settled** | Decided. The rationale and the rejected options are recorded in the entry, or — where a record holds them — in the architecture decision record the entry's status line names |
| **Settled, with a residual** | Decided in substance. One narrow point remains, named in the entry with its owner and the document that settles it. **No entry carries this status today**: `OD-22`'s residual was discharged on 2026-09-11 |
| **Settled, with an observation owed** | Decided. One statement the entry rests on is unverified, or one wording of the corpus is imprecise; the entry names it, names its owner, and states what changes if it does not hold |
| **Settled, with an amendment owed** | Decided. The decision obliges `/specification` to move before any code is written against it. The entry names the requirement and the order. **No entry carries this status today**: `OD-28`'s amendment landed in the ninth edition |
| **Open** | Not decided. The entry names the options and the owner. **No entry carries this status today** |
| **Conflict** | Two requirements, or a requirement and a mandated constraint, cannot both be honoured. Not a choice: a defect owed to `specification-manager`, and the documents it blocks wait for the correction rather than being written around it. **No entry carries this status today** |

## Index

| Entry | Subject | Status | Owner |
|---|---|---|---|
| [OD-01](#od-01--where-the-architecture-decision-records-live) | Where the architecture decision records live | Settled | — |
| [OD-02](#od-02--the-msrv) | The MSRV | Settled | — |
| [OD-03](#od-03--versioning-the-binary-the-document-the-cache-the-changelog) | Versioning: binary, document, cache, changelog | Settled | — |
| [OD-04](#od-04--one-package-or-a-workspace) | One package, or a workspace | Settled | — |
| [OD-05](#od-05--the-module-decomposition) | The module decomposition | Settled | — |
| [OD-06](#od-06--the-error-types-shape-and-the-exit-code-derivation) | The error type's shape and the exit-code derivation | Settled | — |
| [OD-07](#od-07--help-the-parsers-renderer-or-tpls-own) | Help: the parser's renderer, or `tpl`'s own | Settled | — |
| [OD-08](#od-08--the-parsers-own-diagnostics) | The parser's own diagnostics | Settled | — |
| [OD-09](#od-09--toml-the-read-path-and-the-write-path) | TOML: the read path and the write path | Settled | — |
| [OD-10](#od-10--cache-filenames-and-the-case-collision) | Cache filenames, and the case collision | Settled | — |
| [OD-11](#od-11--the-scope-of-the-async-runtime) | The scope of the async runtime | Settled | — |
| [OD-12](#od-12--how-six-phase-deadlines-are-enforced) | How six phase deadlines are enforced | Settled | — |
| [OD-13](#od-13--the-engine-pin-and-minijinja-contrib) | The engine pin, and `minijinja-contrib` | Settled | — |
| [OD-14](#od-14--which-undefined-behaviour-the-engine-is-configured-with) | Which undefined behaviour the engine is configured with | Settled, with an observation owed | `technical-writer` |
| [OD-15](#od-15--the-template-loader) | The template loader | Settled | — |
| [OD-16](#od-16--the-tls-backend-and-the-root-store) | The TLS backend and the root store | Settled | — |
| [OD-17](#od-17--observability) | Observability | Settled | — |
| [OD-18](#od-18--serialisation-key-order-and-the-two-omissions) | Serialisation, key order, and the two omissions | Settled | — |
| [OD-19](#od-19--whether-the-two-embeddings-are-materialised) | Whether the two embeddings are materialised | Settled | — |
| [OD-20](#od-20--edit-distance-and-the-other-small-algorithms) | Edit distance, and the other small algorithms | Settled | — |
| [OD-21](#od-21--two-test-seams-that-must-not-be-on-the-published-surface) | Two test seams that must not be on the published surface | Settled | — |
| [OD-22](#od-22--the-test-harness-and-the-fixture-certificate) | The test harness, and the fixture certificate | Settled | — |
| [OD-23](#od-23--packaging-artefacts-and-the-musl-build-path) | Packaging, artefacts, and the musl build path | Settled | — |
| [OD-24](#od-24--the-discovery-boundary-and-the-process-uid) | The discovery boundary, and the process uid | Settled | — |
| [OD-25](#od-25--the-clock-source-for-now) | The clock source for `now` | Settled | — |
| [OD-26](#od-26--the-boundary-against-the-knowledge-graph) | The boundary against the knowledge graph | Settled | — |
| [OD-27](#od-27--seed-benchsql-and-wl-001) | `seed-bench.sql` and `WL-001` | Settled | — |
| [OD-28](#od-28--the-release-profile-against-the-caught-panic-condition-of-70) | The release profile against the caught-panic condition of `70` | Settled | — |

Twenty-seven entries are settled outright; `OD-14` alone carries an observation
owed. Twenty-seven and one are the whole of the twenty-eight.

Two editorial defects were reported at the end as `ED-01` and `ED-02`. Both
were corrected in the eighth edition; neither is outstanding.

## Verification note

Every version number and every library behaviour cited below was verified
against the source named beside it — vendor documentation on `docs.rs`, the
crate index, the Rust Edition Guide, the Rust Reference, the Rust Book, the
Cargo Book, or a file of this repository. Each claim carries the date it was
verified on: **2026-09-10** for the nineteen entries settled that day,
**2026-09-11** for everything added since. Anything not verified says so in its
own text. No claim rests on recollection.

**An entry reduced to a citation carries no source of its own**, and neither
its sources nor its unverified points are restated here: both live in the
record its status line names, under the same rule. `OD-02` is the case that
matters, because the record it cites moved the number this entry carried.

The three entries the eighth edition settled — `OD-21`, `OD-22`, `OD-24` —
were re-read against the corpus at commit `9efa791` on **2026-09-11**,
requirement by requirement. Every identifier they cite was confirmed to exist in
that corpus.

The two entries the **ninth** edition discharged — `OD-12` and `OD-28` — were
re-read the same way against commit `4ad5e8c` of 2026-09-11, over
`errors-and-exit-codes.md`, `configuration-model.md`, `output-formats.md`,
`upstream-divergences.md` and the edition's own section of `README.md`. Every
identifier cited in the paragraphs that changed was confirmed to exist in that
corpus.

---

## OD-01 — Where the architecture decision records live

**Status: settled.**

**Decision.** The register lives at `docs/adr/` and is cited by `ADR-NNN`. The
convention and the authoritative index are documented in
[`docs/adr/README.md`](../adr/README.md), and are **not restated here**, per
rule R3 of that document.

---

## OD-02 — The MSRV

**Status: settled. Recorded in [`ADR-007`](../adr/adr-007-msrv.md).**

**Decision.** The MSRV is the floor `ADR-007` states, and `CLAUDE.md`'s
deferral — "MSRV a fixar no `Cargo.toml`" — is settled by it.

**The number this entry carried is superseded.** It stated a floor under an
explicit "not verified" caveat over the dependency floors. Those floors have
since been read from the crate index and one of them exceeds it, so the rule
this entry already stated — the pin rises to it — applies, and `ADR-007`
carries the result.

The number, the rule that yields it, the verified floors, the options rejected,
and the one point that remains unverified are recorded in `ADR-007` and are
**not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

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

**Status: settled. Recorded in [`ADR-006`](../adr/adr-006-package-layout.md).**

**Decision.** One Cargo package, carrying a library and a binary, and no
package of its own for `benches/`.

The rationale, the options rejected, the `cargo tree` invocation that answers
the dev-dependency question, and the `dhat` caveat that belongs to `operations`
are recorded in `ADR-006` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). The residual this entry carried was
settled on 2026-09-11 and is part of the decision that record holds.

---

## OD-05 — The module decomposition

**Status: settled.**

**Decision.** Eleven modules under `src/`, the seven the project already
sketches plus four new ones. Each new module exists because a responsibility
crosses every command and would otherwise be copied per command, or because
placing it under an existing module would make that module own a subject its
name denies.

```
src/
├── main.rs        parse, dispatch, map the error to an exit status, and nothing else
├── lib.rs         the crate root and its deliberate re-exports
├── cli/           the parser tree, one module per porcelain command, and the help renderer
├── project/       discovery, the trust checks, and the configuration reader and writer
├── mariadb/       the connection, the catalogue reader, and the privilege cross-checks
├── model/         the model read from the catalogue — the published surface
├── cache/         the read-through cache and its on-disk arrangement
├── render/        the engine, the loader, and the registered template surface
├── output/        the envelope, the JSON emitter, the text layouts, escaping, the writer
├── diagnostics/   the four-line renderer, the suggestion machinery, the verbosity gate
├── deadline.rs    the phase clock and the timer thread of OD-12
└── error.rs       the error type and the exit-code derivation
```

**The six placements the entry asked for, and the seventh the answer to
`OD-12` created.**

| Responsibility | Home | Why there | Home rejected, and why |
|---|---|---|---|
| The catalogue cache | `cache/` | It sits **between** the reader and every consumer, and owns a subject neither neighbour owns: an on-disk arrangement with its own version, `cache_format`, which `FR-CDOC-005` makes independent of the model's | Under `mariadb/`, which would make a module named after the server own a filesystem format and a version the server knows nothing about; under `project/`, which would make the project module own catalogue semantics and per-collection completeness |
| Output formatting, the envelope, `text` layout, escaping | `output/` | One envelope governs all seventeen documents, per `FR-OUT-032`, and `FR-OUT-018` escapes on the way out of four command groups. A single owner is what makes "the envelope is the same everywhere" a property of the code rather than of review | Under `cli/`, which would put the envelope in as many places as there are commands that emit; under `model/`, which would make the model own its own presentation and put an escaping rule inside the type `FR-SCH-022` requires to round-trip unchanged |
| Help text and the typed examples and exit-codes table | `cli/help.rs` | `FR-HELP-021` derives the JSON command tree by introspecting the parser's tree, which lives here, and `FR-HELP-022`'s table is indexed by command path — `cli/`'s own vocabulary. It emits through `output/` for `help --format json` | A top-level `help/`, which would have to reach into `cli/` for the tree and the paths that are its only inputs, inverting the dependency for no gain |
| The privilege cross-checks | `mariadb/privileges.rs` | The three checks read the **shape of the rows the server returned** — an empty `VIEW_DEFINITION` (`FR-PRIV-011`), a `NULL` `ROUTINE_DEFINITION` (`FR-PRIV-017`), zero rows from three catalogue tables (`FR-PRIV-019`). None is a property of the model; each is a property of a read | Under `model/`, which would make the published model type know about grants, and would put a check on a shape the model no longer carries by the time it is built |
| The configuration reader and writer | `project/config.rs` | `FR-PROJ-010` and `FR-PROJ-011` make the ownership and mode of `.tpl/.cfg` a precondition of reading it, so the file and the folder that holds it are one subject. One module owns both paths over one key space, which is what keeps the fifteen keys of `FR-CONF-002` in one place | A top-level `config/`, which separates the file from the discovery that found it and the trust checks that gate it, and puts the key space one module away from the rule that decides whether it may be read at all |
| The diagnostic renderer and the suggestion machinery | `diagnostics/` | It holds transformations, not a taxonomy: the escaping of `FR-ERR-024`, the character set of `FR-ERR-022` and `FR-ERR-023`, the candidate selection of `FR-ERR-019`, and the four-line layout of `FR-ERR-008`. It also owns the verbosity gate of `FR-GLOB-014` and the typed diagnostic sinks of `OD-17` | Inside `error.rs`, which would put presentation beside the taxonomy and make the error type depend on an edit-distance implementation. `OD-06` separates the two for the same reason |
| The phase clock and the timer thread | `deadline.rs` | `OD-12` gives one construct three users — the runtime inside `mariadb/`, the child process, and the render — and `FR-GLOB-012` composes every phase deadline with one budget measured from process start. A budget shared by three modules belongs to none of them | Inside `project/config.rs` beside the four `[core]` keys, which resolves the values but cannot hold the construct that applies them; and inside each of the three users, which is the same rule written three times |

**The name `diagnostics` rather than `diag`.** The project's own convention
refuses obscure abbreviations in module names. The register named `diag/` as a
candidate; it is spelled out.

**`cli/` delegates; it does not do the work.** Each porcelain command is a
module under `cli/` that validates its own arguments, calls the library, and
hands the result to `output/`. The work itself lives in the module that owns
the subject.

**Rationale.** Three requirements decide this rather than taste. The project's
own organisation rule puts the logic in the library and reduces `main.rs` to
parse, dispatch and map. `NFR-PERF-005` requires `tpl init`, every form of
`help` and every form of `version` to perform no discovery, read no
configuration and open no connection — a property that is *observable* when a
command module is a thin adapter over a lazily reached subject, and that has to
be argued when a command module contains the work. And the same subject is
reached from several commands — a table is read by `schema table`, by
`schema dump`, by `render` and by `cache load` — so work placed in a command
module is work that is either duplicated or reached sideways.

**Rejected — a flat module per command with the work inside it.** It is the
shape a CLI takes when it grows without a decomposition, and it makes each of
the four rules above unenforceable: the envelope would exist per command, the
catalogue read would exist per command, and `NFR-PERF-005` would be a review
item rather than an observation.

**Rejected — folding `output/` and `diagnostics/` into one module.** They
share nothing: `FR-OUT-018` excepts tab in `text` read output and
`FR-ERR-024` escapes it in every message, on streams with opposite contracts —
stdout is byte-identical under `NFR-DET-001` and stderr is explicitly neither
deterministic nor contract. One module holding two
opposite escaping rules over two opposite promises is the shape in which the
wrong one gets applied.

**Visibility.** Everything is private by default; `pub(crate)` for what crosses
a module boundary; `pub` for `model/` and `error.rs` alone, which are the
surface the project documents. `lib.rs` re-exports those two deliberately and
re-exports no module whole.

**Unblocks.** `architecture`, `interfaces`.

---

## OD-06 — The error type's shape and the exit-code derivation

**Status: settled.**

**Decision.** Three answers, in the order the entry asked them.

| Question | Answer |
|---|---|
| One enum or several | **One** public `#[non_exhaustive] enum Error` in `error.rs`, derived with `thiserror`. A module may carry a private error for its own convenience and converts it at its own boundary with `From`; no module error is public and none carries an exit code |
| Where the exit code comes from | **An inherent method on the error**, `Error::exit_code`, in the library beside the enum. `main.rs` calls it and returns the status |
| Where the four labelled lines come from | **A renderer in `diagnostics/`**, not `Display`. `Display` carries the `error:` line's content and nothing else |

**Why one enum.** `FR-ERR-001` fixes ten codes and `FR-ERR-002` forbids
collapsing two conditions onto one code where the caller's next step differs,
so every condition the program can reach has to be assigned a code by someone
who can see all of them at once. One enum puts that assignment in one
exhaustive match, and the project's own rule against a `_ =>` arm that swallows
future variants turns a new condition without a code into a **compile error**.
Per-module enums composed by `From` spread the assignment across the `From`
implementations, where a new variant in a leaf module reaches the boundary with
whatever code the conversion happened to pick.

**Why the method and not a table in the binary.** The project's conventions
name the error enum as a typical carrier of `#[non_exhaustive]`, and it is
applied here. The Rust Reference states the effect plainly: "Cannot match on a non-exhaustive enum
without including a wildcard arm", because "matching on a variant does not
contribute towards the exhaustiveness of the arms" (Rust Reference, *Type
system attributes*, verified 2026-09-11). The binary is a **downstream crate**
of the library, so a table there could not be exhaustive: it would need the
wildcard arm, and the wildcard arm is exactly the construct that lets a new
condition ship with the wrong code. Inside the defining crate the match is
exhaustive and the compiler enforces the assignment.

**Why a renderer and not `Display`.** Four requirements pull the four lines
apart from the type:

- `FR-ERR-010` forbids `cause` to restate `error`, so one value owes **two** distinct strings and `Display` supplies one.
- `FR-ERR-034` fixes, per code, what `cause` must name, so the second string is derived from a per-code obligation rather than from the variant's own wording.
- `FR-ERR-024` escapes `\n`, `\r`, `\t` and every C0 control in **every value interpolated into a message**. The renderer escapes each composed line as a whole, which makes the rule hold for interpolations nobody remembered to escape — a property that review cannot supply and that a `Display` implementation per variant cannot either.
- `FR-ERR-022` and `FR-ERR-023` restrict what may enter a runnable `hint` and require a candidate outside `[A-Za-z0-9_]{1,64}` to be dropped entirely. That is a filter over a candidate set, not a property of the error value.

`Display` is still implemented, because `thiserror` derives it from the
`#[error(...)]` attribute and because the project's conventions ask for it where
the type justifies one. Its output is the `error:` line's content, unescaped;
the renderer escapes it on the way out.

**Why the variant set is not the taxonomy `FR-ERR-015` rejected.** That
requirement withdrew an emitted `kind` field, and its *Rejected* note refuses
"retaining `kind` as an internal taxonomy with no external carrier", on the
ground that "a classification nothing outside the process can observe cannot be
tested". The variant set has two external carriers and is tested through both:
the **exit code**, which `FR-ERR-001` makes contract and `BR-ERR-001` requires
an integration test for, code by code; and the **`cause` line**, whose content
`FR-ERR-034` obliges per code. What `FR-ERR-015` refuses is a *second*
classification that nothing observes. This is the first, and its projection is
what a caller branches on.

**The structural consequence for `FR-GLOB-018`, recorded here because it is the
type that enforces it.** No variant carries the database driver's error. The
driver's failure is classified at the `mariadb/` boundary into a phase, a host,
a port and a classification, and the original value is dropped; `#[from]` is
not used on `sqlx::Error`. A raw driver error therefore has no route to any
stream, because it has no home in the value that reaches one. The template
engine is the deliberate asymmetry: `FR-ERR-011` requires "the chain of
underlying template-engine errors", so a render variant carries that chain, and
`FR-GLOB-018` forbids the **driver** error alone.

**Rejected.**

- **Per-module enums composed by `From`,** for the reason above; and additionally because `FR-ERR-034` requires the `cause` to name an instance rather than a category, so each conversion would have to carry the instance forward through every layer, which is where instances are lost.
- **`anyhow::Error` in the library.** The project fixes `thiserror` there, and an opaque error cannot carry the per-code obligations of `FR-ERR-034` or the exhaustive match `exit_code` depends on.
- **An exit code stored as a field on the error.** It makes two variants able to disagree with the table by construction, and it moves the assignment from a compiler-checked match to a value someone writes at each construction site.

**One observation for `technology-stack`.** With `main.rs` reduced to calling
the library, reading `exit_code`, and returning, the binary has no dynamic error
to carry, so `anyhow` earns nothing under the dependency budget. Whether it
stays in the stack table is a dependency question for that document; nothing in
this entry depends on the answer, and the correction to the coordination
document, if any, is prepared for the user rather than made here — as `OD-09`
did for `toml_edit`.

**Unblocks.** `interfaces`, `architecture`.

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

**Status: settled.**

**Decision.** **Intercept `clap::Error` and re-render.** The parser keeps
`error-context`; `suggestions` and `color` are turned off; `clap`'s own
renderer is never invoked, so no byte it produces reaches a caller.

| clap feature | State | Why |
|---|---|---|
| `error-context` | **on** | It is the only source of the token `FR-ERR-034` row `64` obliges the `cause` line to name |
| `suggestions` | **off** | `FR-ERR-019` and `FR-ERR-020` fix the suggestion rule — at most three, within an edit distance of two, ordered by distance then by name — and `OD-20` fixes the distance as Damerau-Levenshtein. A second candidate generator with different rules would be dead weight whose output is discarded |
| `color` | **off** | `NFR-DET-004` forbids colour and every ANSI escape sequence on stdout and on stderr, "under any circumstances", and names removing colour as what "lets the argument parser be built without its colour support" |
| `wrap_help` | off (default) | `FR-HELP-009` and `FR-HELP-010` put the line breaks in the text and forbid reading `COLUMNS`. Settled in `OD-07` |

Feature names and their documented descriptions: clap feature-flags
documentation, clap 4.6.6, verified 2026-09-11 — `error-context` is "Include
contextual information for errors (which arg failed, etc)" and `suggestions`
"Turns on the `Did you mean '--myoption'?` feature".

**Why interception rather than local production.** `FR-ERR-034` row `64`
obliges the `cause` line to name "the token rejected as written, and why it was
rejected: the unknown command or flag, the value that did not conform together
with the type expected, or both members of the mutually exclusive pair". With
`error-context` off, `clap::Error` carries a kind and nothing else, so `tpl`
would have to recover the token by re-reading `argv` — parsing untrusted input a
second time, by a second set of rules, to answer a question the parser already
answered. With the feature on, the token is read from typed context:
`clap::error::ContextKind` is "available only when the `error-context` crate
feature is enabled" and carries `InvalidArg` ("the cause of the error"),
`InvalidValue` ("rejected values") and `PriorArg` ("existing arguments") among
its variants (docs.rs `clap::error::ContextKind`, clap 4.6.6, verified
2026-09-11).

**Why this does not repeat the dependency `OD-07` rejected.** That entry
refused making a contract output depend on the parser's renderer staying
byte-stable. Nothing here depends on rendered text: `ErrorKind` and
`ContextKind` are typed API, checked by the compiler, and the four lines are
composed by `tpl`. What remains is an API-level dependency on **which** context
kinds clap populates for a given kind of failure, which no documentation
promises. `verification` therefore owes one test per `ErrorKind` that `tpl`
maps, asserting the four lines it produces — the same discipline `FR-HELP-002`
already imposes on the help surface through snapshots.

**`ContextKind` is `#[non_exhaustive]`** (same source and date), so the mapping
carries a wildcard arm by force of the language. That arm produces a `64` whose
`cause` names the token and states that the invocation was rejected, which is
the minimum `FR-ERR-034` row `64` admits; it never produces a code other than
`64`, so no unmapped context can move a caller onto a different branch.

**`FR-CLI-014` is answered without the parser's context at all.** A flag that
carries a single value is declared with `ArgAction::Append`, and `tpl` rejects
a second occurrence itself, naming **both values**. The alternative is not
available: `ArgAction::Set` "will result in an `ArgumentConflict`" on a second
occurrence, which yields the `64` but reports a conflict between arguments
rather than the two values the requirement demands (docs.rs `clap::ArgAction`,
clap 4.6.6, verified 2026-09-11). Declaring the flag as repeatable and refusing
the repetition locally is what puts both values in hand — and it makes
`FR-CLI-014` independent of anything clap chooses to place in its context.

**Rejected.**

- **Turning `error-context` and `suggestions` off and producing every diagnostic in `tpl`.** It forfeits the token that `FR-ERR-034` row `64` requires, and buys back only a feature flag. Recovering the token would mean a second parse of `argv` inside `tpl`, on the path `FR-ERR-006` places first in the validation order, where a disagreement between the two parses is a wrong `cause` line for a correctly rejected invocation.
- **Letting `clap` render its own errors.** `FR-ERR-008` fixes four labelled lines and `FR-ERR-033` makes them the whole of what a caller receives; clap's message shape is not that shape, and adopting it would make a contract output move when the parser's renderer moves. This is the argument `OD-07` already made, and it applies unchanged.
- **Keeping `suggestions` on and using clap's candidates.** `FR-ERR-019` fixes the count, the distance and the ordering, and `FR-ERR-023` refuses a candidate outside `[A-Za-z0-9_]{1,64}` entirely. A candidate set produced by another rule would have to be filtered and re-ordered into the specified one, so the feature would compute a set that is then discarded.

**Composition.** `FR-ERR-024` escapes every value interpolated into a message,
including the argument vector, and the token this entry recovers is such a
value: it reaches the reader through the renderer of `OD-06`, escaped, and never
through clap. `FR-GLOB-018` forbids the argument **vector** on a diagnostic
stream; one rejected token is not the vector, and `FR-ERR-034` row `64`
requires it.

**Unblocks.** `interfaces`.

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

**Status: settled. Recorded in [`ADR-005`](../adr/adr-005-async-runtime-scope.md).**

**Decision.** The process is synchronous, and the runtime is confined to the
boundary `ADR-005` names.

The rationale, the requirements it serves, and the options rejected are
recorded in `ADR-005` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). The driver decision this entry
composed with is [`ADR-003`](../adr/adr-003-database-driver.md), which also
records the unexplained musl blocking cost this entry mentioned; it bears on
`OD-12`.

---

## OD-12 — How six phase deadlines are enforced

**Status: settled.** The observation it owed `specification-manager` landed in
the ninth edition; one behaviour is still owed a verification, which falls to
`verification` rather than to this register, exactly as `OD-08`'s does. It was
flagged as the entry likeliest to prove a requirement unmeetable. It did not:
five of the six phases separate cleanly, and the sixth pair separates in the
report rather than in the call.

**Decision.** Three mechanisms, chosen by what the phase is waiting on.

| Phase | Bounded by | Deadline |
|---|---|---|
| DNS resolution | `tokio::time::timeout` around `tokio::net::lookup_host`, performed by `tpl` before the driver is called | the connection deadline |
| TCP connect **and** TLS handshake | one `tokio::time::timeout` around the driver's `connect_with`, which receives an address the resolution already produced | the remainder of the connection deadline |
| Catalogue query | `tokio::time::timeout` around each query | `core.query_timeout` |
| `password_command` | a timer thread that kills the child; the parent reports the deadline | `core.password_timeout` |
| Render | a timer thread that writes the `65` diagnostic and exits the process | `core.render_timeout` |

The **connection deadline** is one instant, set at `core.connect_timeout` from
the start of connection establishment and shared by the three connection
phases. Every deadline above composes with the overall budget as `FR-GLOB-012`
requires: a phase ends at the first of its own deadline and what remains of
`--timeout` measured from process start.

**How the phases are separated.**

1. **DNS is separated because `tpl` performs it.** This is obliged rather than chosen: `FR-CONF-005`'s note and `NFR-PERF-018`'s accepted cost both require the `cause` line to say that a name did not resolve rather than that a host refused a connection, and that distinction cannot be recovered from a driver call that resolves internally. `MySqlConnectOptions` exposes `host` and `port` and no pre-resolution hook, so handing it the address the resolution produced leaves it nothing to resolve (docs.rs `sqlx::mysql::MySqlConnectOptions`, sqlx 0.9.0, verified 2026-09-11).
2. **TCP connect and TLS handshake are not separable in the call.** `MySqlConnectOptions` has **no method that accepts an already-connected stream or socket**; `socket()` takes the path of a Unix socket and changes the transport rather than supplying a connection (same source and date). One call therefore covers both phases, which is what `BENCHMARKS.md` measured when it attributed 44.18 ms to `connect_with` as a whole.
3. **They are separated in the report, by the driver's own discriminant.** `sqlx::Error::Tls` is documented as "Error occurred while attempting to establish a TLS connection" and `sqlx::Error::Io` as "Error communicating with the database backend" (docs.rs `sqlx::Error`, sqlx 0.9.0, verified 2026-09-11). The phase named in the `cause` line is derived from the variant, not from the call site.
4. **Catalogue query, `password_command` and render are separate calls** and need no argument.

**What the `cause` line may carry.** `FR-ERR-034` row `69` requires it to name
the phase, the host and port attempted, and what that phase returned, while
`FR-GLOB-018` forbids the raw driver error on any diagnostic stream at any
level. Both hold only if the driver's error is **classified and re-worded**
rather than rendered: the error value carries the phase, the host, the port and
a classification of the failure, and the driver error's `Display` is never
reached. `OD-06` makes that structural by refusing the driver error a home
inside the error type.

**How a render is bounded.** A **timer thread**. The render runs on the calling
thread; a thread created immediately before it waits on a channel with
`std::sync::mpsc::Receiver::recv_timeout`, whose signature is
`recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError>` and
which returns `Err(RecvTimeoutError::Timeout)` when the duration is exceeded
before a message arrives (Rust standard library documentation,
`std::sync::mpsc::Receiver`, verified 2026-09-11). The render signals the
channel when it completes. If the deadline arrives first the timer
writes the four labelled lines of `FR-ERR-008` for a `65` — naming which
deadline expired and its resolved value, per `FR-ERR-034` row `65` — and
terminates the process with status `65`.

Three requirements make this admissible rather than merely convenient.
`FR-RND-034` already admits that stdout carries at most one incomplete result
when a render fails, so an interrupted render does not violate a promise about
stdout. `NFR-DET-001` keeps stderr outside the contract, so the timer writing
to it while the render writes to stdout interleaves nothing that is contract.
And `FR-GLOB-013` requires the exit code of the **phase in progress**, which a
timer holding the phase it was created for supplies directly; the same
construct therefore realises the overall budget of `FR-GLOB-011` when
`--timeout` is supplied.

**Why a timer thread is not the speculative parallelism the project forbids.**
The rule refuses concurrency adopted for speed without a measurement. A timer
performs no work of the invocation and makes nothing faster; it is the only
construct that can bound a computation with no interruption point. It is
created only on the two paths that need it, so the four commands of
`NFR-PERF-005` — `tpl init`, every form of `help`, every form of `version` —
create no thread, open no socket and read no file, exactly as before.

**Rejected.**

- **A pre-flight TCP connect by `tpl`, to attribute the connect phase exactly.** It would resolve the ambiguity of point 3 outright, at the price of a second connection per invocation, which `NFR-PERF-004` forbids: "One invocation SHALL open at most one connection."
- **Reporting TCP connect and TLS handshake as one `connect` phase.** `FR-ERR-034` row `69` enumerates four phases and obliges the `cause` to name the one that failed; a `cause` reading "connect failed" would be equally true of two different failures, which the same requirement forbids in its own words.
- **`minijinja`'s `set_fuel`.** Fuel is an instruction budget consumed per instruction, gated behind the `fuel` crate feature (docs.rs `minijinja::Environment`, verified 2026-09-11). `FR-CONF-002` states every deadline in **seconds** and `FR-GLOB-012` composes them with a wall-clock budget measured from process start, so a fuel figure would have to be calibrated into seconds — per target, since `NFR-PERF-012` forbids carrying a figure from one target to another. A budget that has to be re-derived on four targets to mean what a requirement already states in seconds is not the mechanism. `set_recursion_limit` stays at its documented default of 500, which bounds recursion and not time.
- **A cooperative clock check inside the output writer.** It bounds a template that emits and not one that loops without emitting, so it would bound some renders rather than the render — and which ones would depend on the template, which is caller input.
- **Rendering on a worker thread while the calling thread waits with `recv_timeout`.** The same construct inverted. It moves the hot path off the calling thread for no gain and puts the writer on the thread that is abandoned.

**The observation owed to `specification-manager`, discharged by the ninth
edition.** It reported that `FR-CONF-005` named six phases, `FR-CONF-002`
supplies four `[core]` timeout keys, and `FR-CONF-004` resolved "each phase
deadline from the `[core]` key for that phase", which left the three connection
phases with no unique referent. Two readings were available: three independent
timers of `connect_timeout` each, whose sum is three times the key the caller
set; or one budget of `connect_timeout` shared by the three, which is what the
key's name states. **This entry took the second**, because a caller who writes
`connect_timeout = 10` is stating how long connecting may take, and because the
first reading makes the configured value unable to bound the thing it is named
after. `FR-CONF-005` now carries the phase-to-key mapping and the shared
connection budget in its own text, and `FR-CONF-004` points at that mapping
instead of implying a key per phase — so the shared connection deadline above is
the corpus's rule rather than this entry's reading. Nothing in the decision
changes.

**The behaviour owed a verification, and what it costs if it does not hold.**
Whether a TLS handshake failure reaches `tpl` as `sqlx::Error::Tls` — for an
untrusted certificate, for a name mismatch, and for a server that offers no
TLS — is **not confirmed in sqlx's documentation**, which states what each
variant means and not which failures map to it. If a handshake failure arrives
as `Error::Io`, point 3 fails and `FR-ERR-034` row `69` cannot be met as
written; that is then a defect owed to `specification-manager`, naming
`FR-ERR-034`. The test that decides it belongs to `verification` and runs
against the fixture, whose obligation under `FR-CONF-038` already provides all
three failure modes, all three of which the fixture now presents — see
[`OD-22`](#od-22--the-test-harness-and-the-fixture-certificate).

**One consequence recorded for `architecture`.** `tokio::net::lookup_host` is
gated behind tokio's `net` feature (docs.rs `tokio::net::lookup_host`, tokio
1.53.1, verified 2026-09-11) and resolves through the platform resolver. The
deadline bounds `tpl`'s **wait**, not the resolver's work: a resolution that
outlives its deadline is abandoned and ends with the process. That is the
honest statement of what a deadline on DNS can be, and it is what
`NFR-PERF-018`'s musl note already assumes when it says a name simply does not
resolve.

**Unblocks.** `architecture`, `interfaces`, `quality-attributes`.

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

**Status: settled, with an observation owed.**

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

**Status: settled.**

**Decision.** **`tpl` writes the loader.** `minijinja::path_loader` is not used.
One function in `render/` resolves a template name to a path, and it is the
**only** resolution in the crate: the loader closure calls it, and so do
`template list`, `template show` and `template path`, which resolve paths
without the engine.

The resolution is one sequence, and it is stated here because five requirements
constrain it and none of them may be applied twice:

1. Reject a name that is not a template under `FR-TMPL-004` and `FR-TMPL-005` — a name whose file does not end in `.jinja` is not a template and is not resolved.
2. Join the name to the template root of `FR-TMPL-023`.
3. Canonicalise, per `FR-TMPL-025`.
4. Re-check the canonical path against the canonicalised root; an escape is `65`, per `FR-TMPL-026`.
5. Refuse a symbolic link, per `FR-TMPL-024`, by reading the entry's own metadata rather than following it — `std::fs::symlink_metadata`, which "queries the metadata about a file without following symlinks" and "corresponds to the `lstat` function on Unix" (Rust standard library documentation, `std::fs::symlink_metadata`, verified 2026-09-11).
6. Open the path that was checked, and no other.

**Why not wrap `path_loader`.** Wrapping runs `tpl`'s checks on one path and
lets the engine's helper resolve a second one from the same name, by a rule
that is not ours. Two consequences follow, and either is disqualifying. The
path that was **checked** would not be the path that is **opened**, which is the
shape of defect `FR-TMPL-025` exists to close by requiring the resolved path to
be canonicalised and re-checked. And the inner rule is not knowable: the helper
is documented to refuse templates that "start with a dot (`.`) or are contained
in a folder starting with a dot", and its documentation states **nothing** about
`..`, about an absolute path, or about a symbolic link (docs.rs
`minijinja::path_loader`, verified 2026-09-11). Silence is not a behaviour. A
containment property that `FR-SEC-017` names by exploit —
`ln -s ../.cfg .tpl/templates/leak.jinja` — may not rest on an undocumented
one.

The engine asks for exactly what a written loader supplies:
`Environment::set_loader` takes `Fn(&str) -> Result<Option<String>, Error>`, and
"once loaded, templates are cached, so the loader is invoked only once per
template name" (docs.rs `minijinja::Environment`, verified 2026-09-11) — which
is also what the project's rule that each template is parsed once per process
requires.

**How the two codes stay apart.** `FR-ERR-028` makes a **missing template**
`66` with a nearest-match suggestion under `FR-TMPL-027`, while `FR-TMPL-009`
makes an `{% include %}` that does not resolve literally a `65` naming the
template, the line and the column. The split is decided by **who asks**:

| Asked by | Mechanism | Code |
|---|---|---|
| The command line, before the engine is built | `tpl` resolves the named template itself and produces the suggestion over the names that exist | `66` |
| A template, through `{% include %}`, `{% import %}` or `{% extends %}` | The loader returns `Ok(None)`; the engine raises its own not-found error, which carries the line and the column `FR-TMPL-009` requires | `65` |
| Either, with a path that escapes the root | The loader returns `Err`, and the escape is reported as an escape | `65` |

`FR-TMPL-008` is preserved by construction: inside a template a name is
literal, and step 1 adds no extension. The optional extension of `FR-TMPL-007`
is a **command-line** affordance and is applied in `cli/` before the resolution
is asked for, so the engine never sees a name `tpl` completed.

**Rejected.**

- **Wrapping `path_loader` with the checks in front of it**, for the two reasons above.
- **Loading every template into the environment at startup with `add_template_owned`.** It makes containment a property of a single enumeration and would be simple to verify — and it reads and parses every template in the project for an invocation that renders one, which is the startup work `NFR-PERF-005` and the project's lazy-initialisation rule both refuse, and which would make `tpl render` pay for a template directory it does not use.
- **Placing the checks in each of the four `template` subcommands and again in the loader.** Five copies of a security rule is five places for it to differ; `FR-TMPL-023` makes the root "the boundary of every template lookup", which is one boundary and therefore one implementation.

**Composition with `security`.** The same function is the single place where
`FR-TMPL-024`, `FR-TMPL-025` and `FR-TMPL-026` are enforced, so `security.md`
cites one containment point rather than describing four. Which of the four
subcommands reaches it, and in what order relative to the trust checks of
`FR-PROJ-010`, is `architecture`'s to state.

**Unblocks.** `architecture`, `security`.

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

**Status: settled.**

**Decision.** **No subscriber is installed, and `tracing-subscriber` is not a
dependency.** `tpl`'s diagnostics are written by a small module in
`diagnostics/`: a level held once, a locked and buffered handle on stderr, and
a **closed set of typed emission functions**. There is no general-purpose sink
that accepts arbitrary text.

**How `FR-GLOB-018` becomes structural rather than reviewed.** Two properties,
and together they close the two routes a forbidden category can take.

1. **Nothing a dependency emits can reach the stream.** `tracing` states it: "Any trace events generated outside the context of a subscriber will not be collected", and the crate "does not contain any `Subscriber` implementations" (docs.rs `tracing` 0.1.44, verified 2026-09-11). With no subscriber installed, every event any dependency emits — including whatever the database driver chooses to record about a statement or a failure — is discarded before it exists. The raw driver error therefore has no route to stderr at all, which is what `FR-GLOB-018` names first among the six.
2. **Nothing in `tpl` can emit an arbitrary string.** The emission functions take typed arguments — a phase and a duration, a query identity, a cache key and a hit or a miss — and compose the line themselves. There is no `debug!("{e}")` to write, because there is no function that takes a formatted message. This is the same move `OD-06` makes in the error type: the forbidden content is denied a **home**, not denied by a rule someone must remember.

`OD-06` supplies the third leg: the driver's error is classified at the
`mariadb/` boundary and the original value is dropped, so it is not present in
the process to be logged even by a function that would take it.

**How the one line per catalogue query is distinguishable.** It is emitted by
**one** function, and that function is the only writer of a fixed leading token
on the line. `FR-GLOB-017` constrains the existence of the line and its
distinguishability, not its wording, and `NFR-DET-001` keeps stderr outside the
contract. A test may therefore depend on the **token** without depending on the
stream being contract: what it asserts is the two properties the requirement
states — that exactly one such line exists per query, and that no other line
carries the token — which is precisely what `NFR-PERF-008` needs to make
`NFR-PERF-001` and `NFR-PERF-002` checkable. The wording after the token stays
free to change, and no test reads it.

**How the phase timings are produced.** From the deadline machinery of `OD-12`,
which already holds the start instant and the elapsed time of every phase
because it has to enforce a deadline on each and compose it with the overall
budget of `FR-GLOB-012`. `FR-GLOB-017`'s "which phases ran and how long each
took" is a report of data the program already has; measuring it a second time
through instrumentation spans would be two clocks for one fact.

**The levels.** `-v` raises the level to `INFO`, `DEBUG`, `TRACE` and saturates
(`FR-GLOB-014`); `-q` lowers it to errors only (`FR-GLOB-015`); neither touches
stdout (`FR-GLOB-016`). The level is resolved once, during argument handling,
and read from an ordinary shared value. `NFR-DET-004` forbids colour and every
ANSI escape sequence on either stream, so the writer emits none and has no
terminal detection to perform — `NFR-DET-003` forbids that too.

**Rejected.**

- **`tracing-subscriber`'s `fmt` layer with a custom `FormatEvent`.** It supplies a formatter and a span-timing facility that this program does not need — the timings come from the deadline machinery — and it costs the property that decides this entry: with a subscriber installed, every dependency's events become emittable, so `FR-GLOB-018` would be restored to a review item over every crate in the graph rather than a consequence of the architecture.
- **A minimal `Subscriber` written in `tpl`, keeping `tracing` as the front end.** It keeps the dependency-budget question open for a facade this program does not otherwise use, and it re-opens route 1: a subscriber that filters foreign events by target is a rule that can be got wrong, where installing none cannot.
- **Keeping `tracing` and `tracing-subscriber` because they are the ecosystem's default.** The project's dependency budget refuses a crate used for a trivial function, and what is used here is four levels and a handful of typed lines on one stream, with no asynchronous context to correlate and no structured consumer to serve.

**Consequence, prepared for the user and not made here.** The coordination
document's stack table names `tracing` and `tracing-subscriber` for logging.
This decision removes the second outright and leaves the first with no role of
its own; if the driver brings `tracing` transitively it stays in the graph as a
transitive dependency and not as a facility `tpl` uses. That table is an
architecture decision by that file's own terms, and the change is prepared for
the user rather than applied, exactly as `OD-09` prepared the addition of
`toml_edit`.

**Not verified.** Whether the database driver depends on `tracing` or on `log`,
and what it records at which level. It does not bear on the decision: with no
subscriber and no logger installed, both facades discard what they are given.

**Unblocks.** `operations`, `architecture`, `technology-stack`.

---

## OD-18 — Serialisation, key order, and the two omissions

**Status: settled.**

**Decision.** **Derived `Serialize`**, written through `serde_json` into a
writer `tpl` owns. Five answers follow, one per question the entry asked.

| Question | Answer |
|---|---|
| Derived or bespoke | Derived. The emitted types are the model's, and their **field declaration order is the key order** |
| `preserve_order` | **Off**, and `indexmap` is not in the graph |
| The order of the two map-shaped documents | Byte-wise ascending by key, because `NFR-DET-002` already fixes it |
| The two omissions | One is a map that is never given the key; the other is one `skip_serializing_if`, used once in the crate |
| `serde` in the library's public signature | Yes, and it costs nothing |

**Why derived rather than a bespoke writer.** `FR-OUT-013` requires a fixed key
order per structure. Under derived serialisation that order **is** the struct's
declaration order, so it is stated once, in the type, and cannot drift from the
type. A bespoke writer states it a second time, in the writer, and a document
whose key order lives in two places is a document whose two statements
eventually disagree — the failure this folder's own conventions name. The same
argument decides `FR-SCH-022`'s round trip: `--context` is deserialised into
the same types, so the dump and its re-emission are inverse by construction
rather than by a pair of hand-written routines that must be kept inverse.

**Why the two arguments for a bespoke writer do not hold.**

- **C0 escaping.** On the JSON path it is the encoder's already: `serde_json::ser::CharEscape` enumerates `Backspace`, `FormFeed`, `LineFeed`, `CarriageReturn`, `Tab` and `AsciiControl(u8)` — "an escaped ASCII plane control character (usually escaped as `\u00XX`)" (docs.rs `serde_json` 1.0.151, verified 2026-09-11). JSON admits no raw control character inside a string, so a tab is emitted as `\t`, which is the escape. The ninth edition settled the reading this entry took: `FR-OUT-018`'s tab exception is now confined to `text`, and on the `json` path the requirement states that the escape the format defines is what satisfies it. On the **`text`** path the escaping is `tpl`'s own, with tab excepted for the column alignment `FR-OUT-006` needs, and it lives in `output/` beside the layouts it exists for. Neither path needs a serialiser wrapper.
- **The mid-document pipe state.** `FR-ERR-026` makes a stdout closed part-way through a JSON document a `74`. That is a property of the **writer**, not of the encoder: `tpl` writes through a buffered writer that records whether any byte of a document has been emitted, and reports `74` when a write fails after the first. `FR-OUT-021` and the project's own rule that I/O is aggregated require that writer regardless, so the state costs nothing extra.

**The order of `vars` and of the `cfg list` document is not a free choice.**
`NFR-DET-002` states the default and its exceptions: "Every collection the
system presents SHALL be ordered by name, ascending, compared byte by byte,
except where this specification names another order", and its table names six
exceptions, of which neither `FR-CTX-026`'s `vars` nor `FR-CFG-037`'s document
is one. Both therefore fall to the default. The model carries them as ordered
maps keyed by `String`, whose iteration order is byte-wise ascending, and
`serde_json::Value` never appears on the emitting path — so `preserve_order`
would change nothing if it were enabled, and it is not enabled. `indexmap` does
not enter the dependency graph.

**The two exceptions to "absent is `null`", expressed differently because they
are different things.**

| Exception | Mechanism | Why |
|---|---|---|
| `FR-CFG-037` — a key absent from `.cfg` is absent from the `cfg list` document | The document is **built from the keys the file carries**. An absent key is a key never inserted | `FR-CFG-014` forbids applying defaults, so there is no value to omit. Modelling every key as an `Option` and omitting the `None`s would put fifteen omissions in the type to express one rule about a file |
| `FR-PRIV-016` — `restricted` appears only on an incomplete object | One `#[serde(skip_serializing_if = "Option::is_none")]`, on that field alone | The property belongs to the object and is genuinely optional. `FR-PRIV-016` also requires the array never to be empty, which the `Option` states and an empty `Vec` would not |

The attribute appears **exactly once** in the crate. That is the enforceable
form of `FR-OUT-012`: every other `Option` serialises as `null`, by serde's
default, and a second appearance of the attribute is a visible change rather
than a silent one. `verification` owes the test that counts the omissions in
the seventeen documents.

**`--pretty`.** `FR-OUT-008` fixes a two-space indent, which is
`serde_json`'s own default — "construct a pretty printer formatter that
defaults to using two spaces for indentation" (docs.rs
`serde_json::ser::PrettyFormatter`, verified 2026-09-10). Compact is the
default form under `FR-OUT-007` and is `serde_json`'s ordinary serialiser.

**`serde` is a public dependency of the library, deliberately.** The model
derives `Serialize` and `Deserialize`, so serde's traits appear in the public
signature. `DIV-032` records that the library carries no compatibility
guarantee, which removes the cost a public dependency ordinarily has: there is
no consumer whose build a serde major version could break. `technology-stack`
records it as a fact about the surface, not as a risk.

**Where the lossy conversion happens.** `FR-OUT-017` replaces an invalid UTF-8
byte sequence with U+FFFD. That happens in `mariadb/`, where catalogue values
are read as bytes, so the model holds only valid UTF-8 and every consumer —
JSON, `text`, the render context, the cache — inherits the substitution once.
It is not an emitting-path transformation and does not belong to `output/`.

**Rejected.**

- **A bespoke writer for the seventeen documents.** It duplicates the key order that the types already state, and it puts the round trip of `FR-SCH-022` in the hands of two routines that have to stay inverse.
- **Enabling `preserve_order`.** It contradicts `NFR-DET-002` for the two map-shaped documents — insertion order is neither name order nor a named exception — and it changes the ordering of every map globally to answer a question about two.
- **`skip_serializing_if` as a general convention.** It would silently omit every future `Option`, which is exactly what `FR-OUT-012` forbids: "An absent value SHALL be emitted as `null` and SHALL NOT be omitted, so that the shape of a document is constant."
- **A distinct type to express absence for the two exceptions.** It states the rule in the type system at the price of two parallel model shapes for two fields, and `FR-OUT-014` already makes adding a field non-breaking, so the shape has to stay one shape.

**Unblocks.** `interfaces`, `data-model`.

---

## OD-19 — Whether the two embeddings are materialised

**Status: settled. Recorded in [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md).**

**Decision.** Materialise both embeddings: the object graph is the document.

The rationale, the memory consequence, and the two options rejected — emitting
by reference at serialisation time, and streaming the dump — are recorded in
`ADR-009` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

---

## OD-20 — Edit distance, and the other small algorithms

**Status: settled.** The distance was settled in the eighth edition; the four
further algorithms are settled below.

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

**Residual, settled 2026-09-11. All four are implemented in the crate, with no
dependency.** The four are decided together because one argument decides all of
them: each has a **closed grammar written into the corpus**, so a library would
have to be constrained back to that grammar rather than consulted for it, and
the dependency budget refuses a crate used for one function.

| Algorithm | Requirements | Decision | Rejected |
|---|---|---|---|
| The `LIKE` matcher | `FR-SCH-012` … `FR-SCH-014` | Implemented in the crate over the two metacharacters and the two escapes the requirement enumerates — `%`, `_`, `\%`, `\_` — matched in memory and never sent to the server | A regular-expression crate, which brings a full engine and a translation step for a two-metacharacter grammar, and whose own escaping rules would have to be got right over a name that is free text on the server; a glob crate, whose semantics are a different language |
| POSIX word splitting | `FR-CONF-025` | Implemented in the crate, honouring single and double quotes as the requirement states. It runs **only** on a string supplied to a command, never on a value read from `.cfg`: `FR-CONF-035`'s rationale refuses "a quoting engine on the path that reads an untrusted `.cfg`", and `FR-CONF-024` makes one unnecessary by executing the stored array directly and without a shell | A shell-words crate, for the dependency budget, and because the splitting a crate performs is the shell's whole grammar rather than the two quoting forms the requirement names |
| The word-list tokeniser | `FR-ENV-030`, with the eight-row vector of `FR-ENV-032` | Implemented in the crate. The five rules are stated in the requirement, applied once, left to right, and `FR-ENV-032` publishes the expected output for eight operands — so the implementation is testable against the corpus rather than against a library's idea of a word | An inflection or case-conversion crate, whose rules are its own: `FR-ENV-033`'s accepted cost fixes `HTTP_server` → `HttpServer`, and a crate that preserves acronyms would produce a different generated identifier at exit `0` |
| ASCII-only case folding | `FR-ENV-031`, `FR-SCH-014` | `std`: the ASCII-restricted `str::eq_ignore_ascii_case` and `str::to_ascii_lowercase`, which exist beside the Unicode-aware `str::to_lowercase` for exactly this distinction (Rust standard library documentation, `str`, verified 2026-09-11; the exact wording of their guarantee was not retrievable from the rendered page and is not quoted here) | Unicode or locale-aware folding, which both requirements refuse in their own text, because it would make the same template produce different output on two machines and break `NFR-DET-001` |

**What still belongs to `interfaces`.** The surface of each — the signature,
the inputs it accepts, and the errors it can produce — is described there.
What is settled here is the choice and its rejection, which is what this
register exists to carry.

---

## OD-21 — Two test seams that must not be on the published surface

**Status: settled.** The conflict was resolved by the eighth edition; the
residual it left is settled below.

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

**No longer part of this entry.** The tension between the condition
`FR-ERR-030` carried before the ninth edition — a panic **caught** at the top
level — and an aborting release profile is not a seam question; the eighth
edition recorded it as `DIV-045` rather than resolving it, and the ninth
amended the requirement to state the outcome instead. It is carried in
`OD-28`. `FR-ERR-031`'s trigger exercises the **other** producing condition of
`70` — the detected invariant violation — so the exception `BR-ERR-001` grants
does not depend on that answer.

**Residual, settled 2026-09-11. The construct is `#[cfg(test)]`, and both tests
are therefore unit tests inside the library.**

`FR-ERR-031` requires the trigger to be "reachable only from within the
system's own test configuration". `#[cfg(test)]` *is* that configuration, and
the toolchain's own documentation states both halves of what the residual asked
to be verified rather than assumed (The Rust Programming Language, ch. 11.3,
verified 2026-09-11):

- **Absent from the distributed binary.** "The `#[cfg(test)]` annotation on the `tests` module tells Rust to compile and run the test code only when you run `cargo test`, not when you run `cargo build`. This saves compile time … and saves space in the resultant compiled artifact because the tests are not included."
- **Absent from an integration test.** "Each file in the *tests* directory is a separate crate, so we need to bring our library into each test crate's scope." A separate crate is compiled without `cfg(test)` for the library it links, so an item behind `#[cfg(test)]` is not there to be reached.

The second half decides the test kind, and it decides it the same way for both
seams: an integration test **cannot** see the construct, so each test is a
**unit test in the library crate**. That is consistent with what the corpus
already granted — `BR-ERR-001` yields the integration test for `70` alone and
says so in its own text, and `FR-SRV-035` asserts on what the reader emits
rather than on a process exit status.

**Rejected.**

- **A `#[doc(hidden)] pub` item.** It satisfies `FR-ERR-031` literally — a library item is not reachable from an invocation of the binary — and it puts a trigger on the surface the project publishes, which is the property this entry's title refuses. `DIV-032` removes the compatibility cost, not the surface.
- **A cargo feature.** `FR-ERR-031` rejects it by name: "the artefact verified would not be the artefact distributed", and `NFR-PERF-018` makes every distributed artefact first class.
- **An environment variable or a hidden command.** Both are rejected in `FR-ERR-031`'s own table, against `FR-CLI-021`, `FR-CLI-023`, `NFR-DET-001`, `FR-CLI-002` and `FR-HELP-021`.

**What `verification` still owes.** The register of the two tests, each
described with the limit its own requirement states — not a limit this folder
invents — and the naming of both by requirement identifier.

**Unblocks.** `verification`.

---

## OD-22 — The test harness, and the fixture certificate

**Status: settled by the eighth edition. The residual was discharged on
2026-09-11 by tasks #15 and #25.**

**What the conflict was.** `FR-CONF-013` defaults `tls` to `verify-identity`,
and `FR-CONF-038` recorded that `tpl` with default configuration could not then
reach the fixture of `scripts/mariadb/` over TCP on any series. The default mode
was therefore the one cell of a ten-cell table with no acceptance test, and the
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

**The residual was work, and the work was run.** `FR-CONF-038` handed both
halves to the fixture in its own text — how the certificate is generated, where
the fixture keeps it, and how the no-TLS server is retained beside it "are the
fixture's own work and are not specified here" — and this entry refused to
record an arrangement of files and commands nobody had executed. Both were then
executed, and `scripts/mariadb/README.md` is their record. What follows names
what landed; the arrangement itself is not restated here.

| Half | Discharged by | What it produced |
|---|---|---|
| The fixture certificate | Task #15, commit `4bce12e` | `scripts/mariadb/tls/`: a root whose private key is destroyed at generation, a leaf naming `DNS:localhost`, `IP:127.0.0.1` and `IP:::1`, the `ssl_ca`/`ssl_cert`/`ssl_key` settings that put it into service, and `generate.sh` to reproduce it. All four series report `have_ssl=YES` and accept a `verify-identity` connection; the no-TLS server survives as a fifth container started `--skip-ssl` |
| The harness | Task #25, commit `de7ed1e` | `up.sh`, `down.sh`, `status.sh` — the three-valued gate — `observe.sh`, `series.env`, `probe-session.sql` and `observer.Dockerfile`, each established against a substitute client because `tpl` does not exist yet |

**Both halves of `FR-CONF-038` are therefore satisfied**, and no passage of this
folder may still describe the fixture as unable to present a named certificate
or `tpl` as unable to reach it by default. What the discharge did **not**
establish is recorded by the fixture with what was tried in each case: the
failing outcome of `FR-SRV-013`, which no real MariaDB produces; `NFR-PERF-001`
and `NFR-PERF-002` conclusively, which wait on `WL-001` and therefore on
[`OD-27`](#od-27--seed-benchsql-and-wl-001); and the file-open observation on
either Darwin target, which `NFR-PERF-005` now bars from being inferred from a
Linux build traced in a container.

**What the harness must serve, unchanged by the resolution.**

- `CLAUDE.md`: validation needing a database uses the containers of `scripts/mariadb/` — never mocks, never external instances — launched before and stopped after; `scripts/mariadb/README.md` adds "Leave no container running after a validation run."
- `FR-SRV-029`: the cross-series equivalence test runs against every series, and the refusal test against at least one series outside the window.
- `NFR-PERF-007` and `BR-SRV-003`: nine requirements are verified from **outside** the process, by the four instruments `NFR-PERF-007` names, each on the targets of `NFR-PERF-018` its row admits.
- `FR-SRV-012`: the closed statement list is checked by observing what the server actually receives, expecting four kinds and no fifth, with the three connection-start statements issued once each in the stated order.
- `BR-SEC-003`: the sentinel test runs every command of the tree at maximum verbosity and asserts the sentinel appears in no byte of either stream.

**Unblocks.** `operations` and `verification`, which waited on this residual and
on nothing else in this register.

---

## OD-23 — Packaging, artefacts, and the musl build path

**Status: settled. Recorded in [`ADR-008`](../adr/adr-008-packaging-and-build-path.md).**

**Decision.** `cargo-zigbuild` for the two `musl` targets, native builds for the
two Darwin targets, and no continuous integration prescribed.

The rationale, the options rejected, the four obligations carried by hand while
there is no pipeline, and the two targets that have never been measured are
recorded in `ADR-008` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

---

## OD-24 — The discovery boundary, and the process uid

**Status: settled.** The conflict was resolved by the eighth edition; the
residual it left is settled below.

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

**Residual, settled 2026-09-11. The safe wrapper is `rustix`, with default
features off and the `process` feature alone.** `rustix::process::getuid` is
declared `pub fn getuid() -> Uid`, is a safe function, and is "available on
crate feature `process` only" (docs.rs `rustix` 1.1.4, verified 2026-09-11).
That is the one value `std` does not supply: `std::os::unix::fs::MetadataExt`
gives the **file's** `uid()` and its `mode()` on the same metadata (`std` API
documentation, `std::os::unix::fs::MetadataExt`, Rust 1.98.1, verified
2026-09-11), so `FR-PROJ-011`'s mode check needs nothing further and
`FR-PROJ-010` needs exactly one call.

**Why a dependency at all.** `getuid()` through `libc` is an `unsafe` call, and
the project forbids `unsafe` and keeps `#![forbid(unsafe_code)]` at the top of
the crate. That is not a rule to be weighed against a dependency: it is one of
the project's non-negotiable rules, and the crate that satisfies it is the
smallest one that does.

**Rejected.**

- **`libc` with a local `unsafe` block.** Forbidden outright, and the attribute that forbids it is required to stay.
- **`nix`.** It answers the same question and carries a far larger surface for it; the budget prefers the narrower crate.
- **A crate that resolves the user account** — `uzers` and its predecessors. `FR-PROJ-010` compares two uids and needs no user name, and `OD-24` has already removed every account lookup from discovery.
- **Inferring ownership by effect**, by attempting a write. `.tpl/.cfg` is read-only to every command that checks it, and `BR-TMPL-002` and the project's own scope statement keep `tpl` from writing where it was not asked to.

**One consequence for `technology-stack`.** `rustix` is a new normal
dependency and the stack table does not name it. What it drags in on each of
the four targets of `NFR-PERF-018` is recorded there, under the dependency
budget; the correction to the coordination document's stack table, if any, is
prepared for the user rather than made here, as `OD-09` did for `toml_edit`.

**Unblocks.** `technology-stack`.

---

## OD-25 — The clock source for `now`

**Status: settled.**

**Decision.** **Hand-rolled, in `std`, no date dependency.** One clock read per
invocation, one civil-date conversion, and **one grammar used in both
directions** — the fixed twenty-character form `YYYY-MM-DDTHH:MM:SSZ` and
nothing else.

- The instant is `std::time::SystemTime::now`, converted with `duration_since(UNIX_EPOCH)`, whose seconds are documented as "the number of non-leap seconds since the start of 1970 UTC", equivalent to a POSIX `time_t` (Rust standard library documentation, `std::time::SystemTime`, verified 2026-09-11).
- The seconds are split into days and seconds-of-day, and the days are converted to a proleptic Gregorian civil date by integer arithmetic. `FR-CTX-029` reads the clock once per invocation, so the conversion runs at most twice — once to write, once to read a cached value back.
- The reverse direction parses **only** that form, position by position, and rejects everything else.

**Why one grammar in both directions is the whole of the argument.**
`FR-CTX-028` fixes the form exactly — UTC, `Z` offset, second precision — and
`FR-CDOC-013` puts the same value in `meta.json` and in the output of
`tpl cache status`, where `FR-CACHE-034` shows it in that form. So `tpl` writes
one form and must read back the form it wrote. A date crate reads the whole of
RFC 3339, which admits a fractional part (`time-secfrac = "." 1*DIGIT`), a
numeric offset (`time-offset = "Z" / time-numoffset`), and lower-case `t` and
`z` — "the 'T' and 'Z' characters in this syntax may alternatively be lower
case 't' or 'z' respectively" (RFC 3339, §5.6, verified 2026-09-11). A reader
that accepts all of that and a writer that emits one of them are **not
inverse**: a hand-edited `meta.json` carrying `2026-09-10t08:14:22.5+01:00`
would parse, and `tpl cache status` would then re-emit a `loaded_at` in a form
`FR-CDOC-013` does not fix, or silently shift the value into UTC. Writing both
directions against one grammar removes the case rather than handling it.

**What a value that does not parse means.** The cache file is unreadable, and
`FR-CACHE-033` already fixes the outcome: treat it as a miss, read from the
server, rewrite the file, and report neither an error nor a warning. No new
rule is needed, and no `70` is reachable from a file the caller can edit —
which `FR-ERR-031` refuses in its own *Rejected* note.

**What the conversion must be, so that `verification` can test it.** Proleptic
Gregorian, no leap seconds — the epoch seconds are non-leap by the
documentation quoted above — and correct across the range a `SystemTime` can
carry. It is a closed function of one integer with a published expected value
per input, so it is testable as a vector rather than against a clock.

**One property recorded, because it is a property of the clock and not of the
conversion.** `SystemTime` "is not monotonic" and `duration_since` returns a
`Result` because "an earlier `SystemTime` may actually be later than a later
one" (same source and date). A system clock behind 1970 therefore has no
representation in this form. `now` is documented by `NFR-DET-005` as the single
source of non-reproducibility, and a clock that cannot be converted is a defect
in the host rather than in the input — `FR-ERR-030`'s detected invariant
violation, not a caller-facing condition.

**Rejected.**

- **A date crate — `chrono`, `time`, or `jiff`.** Each brings a formatting and parsing engine, and each brings or optionally brings a timezone database, to answer a question with no timezone, no locale, no offset, no fractional part and no alternative form. The project's dependency budget refuses a crate used for one trivial function, and this is one integer-to-civil-date conversion and one twenty-character format.
- **A date crate confined to the write path, with a hand-rolled reader.** It is the worst of both: a dependency *and* two grammars, which is the failure the decision above exists to avoid.
- **Storing `loaded_at` as an epoch integer in `meta.json` and formatting it only on the way out.** It removes the parse, and it contradicts `FR-CDOC-013`, which puts the value in `meta.json` and in `cache status` as one value in one form; `BR-CDOC-005` keeps `meta.json` outside the plumbing contract but does not license a second representation of a field the corpus names in both places.
- **Emitting the stored string verbatim without parsing it.** It removes the conversion and admits into a contract-shaped document whatever a hand-edited `meta.json` carries, which `FR-CDOC-013` and `FR-CACHE-034` between them do not admit.

**Unblocks.** `technology-stack`, `architecture`.

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

**Status: settled. Recorded in [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md).**

**Decision.** The profile stands, and the process installs a panic hook that
produces the outcome `FR-ERR-030` requires.

The five profile settings, the hook, what the `cause` line carries and what it
withholds, the options rejected, and the reasoned step over `strip = true` are
recorded in `ADR-004` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

**What this entry settled beyond the decision, and which stays here.** The
amendment it owed `specification-manager` landed in the ninth edition of
`/specification`, at commit `4ad5e8c` of 2026-09-11: `FR-ERR-030` now states the
outcome a caller observes rather than a mechanism, `FR-ERR-034`'s `70` row loses
the same word, and the code table of `FR-ERR-001` is unchanged. `DIV-045` is
discharged with nothing owed to `CLAUDE.md` under it. The order was not
interchangeable: the requirement moved first, before any implementation was
written against it.

**Consequence for the coordination document.** Its release-profile bullet cites
`ADR-004` rather than listing the five settings, per rule R3. No setting
changes.

**Unblocks.** `architecture`, `technology-stack`, `operations`.

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

## What remains

No entry is open, so there is no order of work over the register. What remains
is one obligation, and it blocks one sentence of one document.

| Order | What | Owner | Blocks |
|---|---|---|---|
| 1 | `OD-14`'s owed observation — that a **defined** `null` interpolates as the empty string under `UndefinedBehavior::Strict`. It is unverified, and `FR-SEM-010` and `FR-SEM-011` are contradicted outright if it does not hold | `technical-writer` | `architecture` may not assert the behaviour until it is verified |

`OD-22`'s residual — the fixture certificate and the harness — was discharged on
2026-09-11 by tasks #15 and #25, and `operations` and `verification` no longer
wait on it.

The two obligations this table carried for `specification-manager` are
discharged: the ninth edition amended `FR-ERR-030` (`OD-28`) and gave
`FR-CONF-005` the phase-to-key mapping with its shared connection budget
(`OD-12`). Each entry records what landed, and neither is reopened.

Two further obligations fall to `verification` rather than to this register, and
are recorded in the entries that create them: one test per mapped
`clap::ErrorKind` (`OD-08`), and the phase attribution of a TLS handshake
failure against the fixture (`OD-12`).
