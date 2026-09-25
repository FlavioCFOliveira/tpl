---
title: Interfaces
status: draft
last-reviewed: 2026-09-25
related: [README.md, traceability.md, open-decisions.md, overview.md, data-model.md, quality-attributes.md]
---

# Interfaces

## What this document is

Every contract that crosses a boundary inside the built system, and how each
external contract is realised at one of them. Each is described with the
component on **both** sides. Every statement cites the requirement that forces
it; no requirement text is reproduced.

**No command-line syntax and no JSON syntax appears here.**
`specification/cli-contract.md` owns the invocation grammar and
`specification/output-formats.md` the document shape; this file cites both and
reproduces neither. Persisted shapes are [data-model.md](data-model.md), which
this file does not repeat.

The components named on either side of a contract are the modules of
[`OD-05`](open-decisions.md#od-05--the-module-decomposition). The decomposition
and its rationale are not restated here; neither is the order an invocation runs
in, which is `architecture.md`, nor any version or crate rationale, which is
`technology-stack.md`.

## The contracts

| # | Contract | Producing component | Consuming component | Section |
|---|---|---|---|---|
| 1 | A catalogue read, one result set per object kind | `mariadb/` | `model/` | [The catalogue reader](#the-catalogue-reader) |
| 2 | The named-object read | `mariadb/` | `model/` | [The catalogue reader](#the-catalogue-reader) |
| 3 | Bytes to text: the lossy conversion | `mariadb/` | `model/` | [The catalogue reader](#the-catalogue-reader) |
| 4 | The resolved series and the `server` object | `mariadb/` | `mariadb/` itself, `model/` | [The catalogue reader](#the-catalogue-reader) |
| 5 | The key-to-column invariant over an assembled table | `mariadb/`, `cache/`, `cli/` | `model/` | [The catalogue reader](#the-catalogue-reader) |
| 6 | A short read: the completeness verdict | `mariadb/privileges.rs` | `error.rs` for a named object, `model/` for a listing | [The three privilege detections](#the-three-privilege-detections) |
| 7 | The `source` discriminant | `cache/` | `output/` | [The document emitter](#the-document-emitter) |
| 8 | The error value | every module | `error.rs` | [The error type and the exit code](#the-error-type-and-the-exit-code) |
| 9 | The exit status | `error.rs` | `main.rs` | [The error type and the exit code](#the-error-type-and-the-exit-code) |
| 10 | The four labelled lines | `diagnostics/` | stderr, from `error.rs` | [The diagnostic renderer](#the-diagnostic-renderer) |
| 11 | The rejected token | `cli/` | `diagnostics/`, through `error.rs` | [The diagnostic renderer](#the-diagnostic-renderer) |
| 12 | A candidate population for a suggestion | `mariadb/`, `render/`, `project/config.rs`, `cli/` | `diagnostics/` | [The diagnostic renderer](#the-diagnostic-renderer) |
| 13 | The envelope and one of seventeen payloads | `output/` | stdout, from `model/`, `cli/help.rs`, `cli/cfg/`, `cache/` | [The document emitter](#the-document-emitter) |
| 14 | Key order | `model/` and the other payload types | `output/` | [The document emitter](#the-document-emitter) |
| 15 | The buffered writer and the mid-document state | `output/` | `error.rs`, `main.rs` | [The document emitter](#the-document-emitter) |
| 16 | Every ordering | `output/`, with three orders preserved by `mariadb/` | stdout | [Ordering](#ordering-one-default-and-six-exceptions) |
| 17 | The parsed command tree, introspected | `cli/` | `cli/help.rs` | [The help surface](#the-help-surface) |
| 18 | The examples and exit-codes table | `cli/help.rs` | the text help, and `output/` | [The help surface](#the-help-surface) |
| 19 | A template name resolved to a path | `render/` | the engine's loader, and the three `template` subcommands in `cli/` that resolve without the engine | [The template surface](#the-template-surface) |
| 20 | The registered filters, tests and functions | `render/` | the engine, and `cli/help.rs` for publication | [The template surface](#the-template-surface) |
| 21 | The render context, and access to it from a filter or a test | `cli/`, `model/`, `project/` | `render/` | [Context access from a filter or a test](#context-access-from-a-filter-or-a-test) |
| 22 | The typed key space, read | `project/config.rs` | every consumer of a setting | [The configuration reader and the writer](#the-configuration-reader-and-the-writer) |
| 23 | The typed key space, written | `cli/cfg/` | `project/edit.rs` | [The configuration reader and the writer](#the-configuration-reader-and-the-writer) |
| 24 | The password from a child process | `project/password.rs`, bounded by `deadline.rs` | `mariadb/`, through `project/settings.rs` | [The `password_command` child](#the-password_command-child) |
| 25 | A phase deadline | `deadline.rs` | `mariadb/`, `project/password.rs`, `render/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 29 | The refusal of an incoherent entry write | `cli/cfg/coherence.rs`, over the predicate in `project/config/entry.rs` | `error.rs` | [The configuration reader and the writer](#the-configuration-reader-and-the-writer) |
| 26 | A name matched against a pattern | `cli/` | applied over names from `model/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 27 | A routine named bare or qualified | `cli/` | `mariadb/`, `cache/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 28 | A closed set of typed diagnostic emissions | `mariadb/`, `cache/`, `deadline.rs` | `diagnostics/` | [The diagnostic renderer](#the-diagnostic-renderer) |
| 30 | The document built from a model, embeddings materialised and every collection ordered | `model/document/` | `output/` | [The two directions over the document](#the-two-directions-over-the-document) |
| 31 | A supplied context document, read back as a model | `cli/`, from bytes it opened | `model/document/` | [The two directions over the document](#the-two-directions-over-the-document) |
| 32 | The three render bounds, resolved | `project/settings.rs` | `render/`, and `cli/render.rs` for the memory limit | [The render bounds](#the-render-bounds) |
| 33 | The heap count | `main.rs`, through the library's `install_heap_counter` | `heap.rs`, read by `cli/render.rs` | [The render bounds](#the-render-bounds) |
| 34 | Whether anything of a catalogue read is alive | `mariadb/` | `cli/render.rs` | [The render bounds](#the-render-bounds) |

## The catalogue reader

`mariadb/` returns model material to `model/`; it returns nothing else, and no
other component sends a statement.

| Obligation on the surface | Forced by |
|---|---|
| One result set per object **kind**, and a fixed set of statements whose count does not grow with the number of objects; the named-object path is a second entry point in which the name restricts the statement the server receives | `NFR-PERF-001`, `NFR-PERF-002` |
| Every statement is drawn from the closed list of four kinds; no other statement exists anywhere in the crate, and no external process is invoked | `FR-SRV-006`, `FR-SRV-007`, `BR-SRV-001` |
| A statement naming a fixed `INFORMATION_SCHEMA` column list is either common to all four series or selected from the **resolved series**; a list is never discovered by issuing a statement and handling its failure | `FR-SRV-037`, `FR-SRV-022`, `FR-SRV-023` |
| Coverage is applied by the reader, to the object read **and** to the column read, before any filtering | `FR-CAT-052`, `FR-CAT-028`, `BR-CAT-003` |
| Rows are folded into objects where the catalogue reports one row per column: an index becomes one object with an ordered column list; a foreign key is joined across two catalogue tables, its incoming direction read from the other end of one of them; the primary key is read from the index table alone | `FR-CAT-010`, `FR-CAT-042`, `FR-CAT-045`, `FR-CAT-013`, `FR-CAT-043` |
| Behaviour the catalogue does not state plainly is **encoded**, not rediscovered per read | `FR-CAT-033` … `FR-CAT-038`, `FR-CAT-040`, `FR-CAT-051` |

Which fields each kind carries is
[data-model.md](data-model.md#from-a-catalogue-field-list-to-a-model-property-list);
the reader's column list per kind is specified material rather than an
implementation choice.

**The lossy conversion is at this boundary and nowhere else.** Catalogue values
are read as bytes and converted with U+FFFD substituted for an invalid
sequence, so the model holds only valid text and all four consumers — the JSON
emitter, the `text` layouts, the cache and the render context — inherit one
substitution (`FR-OUT-017`, with
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
It is not a transformation of the emitting path and does not belong to
`output/`.

**The version probe runs before the catalogue and its result is consumed
twice.** It determines the product and the version on connecting, before any
statement but the read-only pair (`FR-SRV-002`, `FR-SRV-034`); `mariadb/`
resolves a series from it by major and minor alone (`FR-SRV-040`,
`FR-SRV-021`), uses that series to select a column list under `FR-SRV-037` and
a difference treatment under `FR-SRV-022`, and hands the three fields of the
`server` object to `model/` (`FR-CTX-031`, `FR-CTX-034`). A product that is not
MariaDB is `78` (`FR-SRV-003`); a series below the window is `78`
(`FR-SRV-020`, `FR-SRV-030`); one above it is a marked read (`FR-SRV-031`,
`FR-SRV-032`). The read-only session is set and read back before any catalogue
statement — the read-back names `@@session.tx_read_only`, the only spelling
present on all four series (`FR-SRV-038`, difference 12) — and a failure of
either half refuses the connection (`FR-SRV-008`, `FR-SRV-009`, `FR-SRV-010`,
`FR-SRV-011`).

**One invariant is checked over an assembled table, not over a query result.**
`FR-CAT-044` forbids any key — primary, unique, index, or either direction of a
foreign key — to name a column absent from that table's column list, and the
observation behind `FR-CAT-043` shows the catalogue does not supply the
property for free. It is checked once, over an assembled table, and is called by
each of the three producers of a model: `mariadb/` on a server read, `cache/` on
a cached read, and the `--context` path in `cli/`. It is not placed on the
emitting path, because `render/` consumes the same model without passing through
`output/` — the argument `FR-CAT-026` makes for the exclusion lists, applied to
an invariant.

**Recorded gap — what a violation of `FR-CAT-044` produces.** The requirement
states the property and names no exit code. `FR-ERR-030` gives `70` to a
violated internal invariant the system detects and declines to continue past,
which is the only condition in `FR-ERR-001` whose wording covers it; whether
`FR-CAT-044` is such an invariant, or a key to be dropped, is not settled by the
corpus. The gap is reported to the functional owner rather than filled here.

**What the build does with the unsettled half, and why it does not close it.**
The check reports a value of its own, naming the table, the key and the column
and **naming no exit code**, because the same violation means two things at the
two places a model is built: from a read it is an internal invariant, and from a
supplied document it is caller data. Each caller maps it — `70` on the producing
path and `65` on the `--context` path
([The two directions over the document](#the-two-directions-over-the-document))
— so the corpus's silence is carried in the type rather than answered by it. The
other half of the gap the build does decide: a key is never dropped, on either
path.

## The three privilege detections

`mariadb/privileges.rs` reads the **shape of the rows the server returned** and
produces a completeness verdict. None of the three is a property of the model,
so none is placed in `model/`: each is a property of a read (`BR-PRIV-003`).

| Detection | The shape it reads | Needs two observations compared |
|---|---|---|
| A view's definition read as the empty string (`FR-PRIV-011`) | A row that is present, carrying a zero-length definition | Yes — rows of the view kind against the readability of each definition (`FR-PRIV-015`) |
| A routine's body read as SQL `NULL` (`FR-PRIV-017`) | A row that is present, carrying `NULL` | No — the shape is self-announcing |
| A key column naming a referenced table with no referential row for its constraint (`FR-PRIV-019`) | Zero rows from one catalogue table against full rows from another | Yes (`FR-PRIV-015`) |

The three shapes of a privilege-driven absence are the empty string, SQL `NULL`
and zero rows, and a reader that looks for one finds none of the others
(`FR-PRIV-018`) — which is why the detections are three code paths and not one.
All three run on **every** read that presents the property they guard, the dump
included (`FR-PRIV-012`).

**Recorded discrepancy — how many cross-checks there are.**
`FR-PRIV-012` names three, calling the routine-body check a cross-check.
`FR-PRIV-015` names two — views and foreign keys — and states that "no
cross-check SHALL be performed for any other object kind", while `FR-PRIV-017`'s
own note records that the routine body "needed no cross-check at all". Read
literally, `FR-PRIV-012` requires of routines what `FR-PRIV-015` forbids.
[README.md](README.md#interfacesmd) fixes this document's scope as "the three
privilege cross-checks" and [traceability.md](traceability.md) records two.
Both readings are recorded. This document takes `FR-PRIV-015`'s: there are
**three detections**, of which two compare two observations of one population
and are cross-checks in `FR-PRIV-015`'s sense. Nothing in the built system
differs between the two readings; the wording is the functional owner's to
settle.

**The verdict crosses two different boundaries, and that is the asymmetry.**

| Outcome | Boundary | Forced by |
|---|---|---|
| A **named** object that is incomplete: `77`, and no partial object is returned | `mariadb/privileges.rs` → `error.rs`, before `model/` presents anything | `FR-PRIV-003`, `FR-PRIV-004`, `BR-PRIV-001` |
| A listing or a dump: success, with the object marked | `mariadb/privileges.rs` → `model/`, per object | `FR-PRIV-005`, `FR-PRIV-006`, `FR-PRIV-007` |
| The `cause` line names which property of which object could not be read | `error.rs` → `diagnostics/` | `FR-PRIV-013`, `FR-ERR-034`, and `FR-PRIV-014` for what it may not carry |
| A marked object is never written to the cache, and a collection holding one is never recorded whole | `model/` → `cache/` | `FR-CACHE-037` |
| A marked document supplied as a context is refused before the render begins, whatever the render selects | `cli/` → `error.rs` | `FR-PRIV-008`, `FR-PRIV-009`, `FR-RND-020` |

`restricted` is an ordered, never-empty array of model property names, on the
object and never on the envelope (`FR-PRIV-016`, `FR-OUT-029`); it is one of the
two exceptions to "absent is `null`" and is expressed as the one omission
attribute the crate admits, on that one field
([`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
It is written once **per markable kind** — a table, a view and a routine — and
`OD-18`'s reading of the count is amended there rather than here; every other
optional value in the model serialises as `null`, which is what `FR-OUT-012`
requires and what a fourth appearance would break visibly.
The one limit the detections do not reach — a hidden trigger list, which is
byte-identical to an empty one — is stated in `FR-PRIV-020` and carried in
[overview.md](overview.md#the-three-limits-the-system-states-rather-than-overcomes).

## The error type and the exit code

Settled in [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation):
one public non-exhaustive enum in `error.rs`, an inherent method on it for the
exit status, and a renderer in `diagnostics/` for the four lines. The rationale
and the rejected options are not restated here.

| Obligation on the type | Forced by |
|---|---|
| Ten codes and no others, each distinct condition with its own code, never collapsed where the caller's next step differs | `FR-ERR-001`, `FR-ERR-002` |
| Every variant carries the **instance**, not the category: the token, the identifier and the population, the phase with host and port, the path and the operation, the key with the value found and the value expected | `FR-ERR-034`, `FR-ERR-010`, `FR-ERR-012` |
| A render variant carries the chain of underlying engine errors, with template, line and column | `FR-ERR-011`, `FR-RND-030`, `FR-RND-031`, `FR-SEM-019` |
| No variant carries the database driver's error. The driver's failure is classified at the `mariadb/` boundary into a phase, a host, a port and a classification, and the original value is dropped | `FR-GLOB-018`, with [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) and [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) |
| `73` is constructible only on the `tpl init` path; `70` only from a panic or a detected invariant violation, its trigger reachable from nothing a caller can write | `FR-ERR-003`, `FR-ERR-030`, `FR-ERR-031`, [`OD-21`](open-decisions.md#od-21--two-test-seams-that-must-not-be-on-the-published-surface) |
| A deadline produces the code of the **phase in progress**, so the value the phase clock hands to `error.rs` names the phase | `FR-ERR-027`, `FR-GLOB-013`, `FR-GLOB-012` |
| One condition reachable over **two populations** is two variants, not one, where a single `cause` would carry wording that is false on one of them. An object sought in a catalogue read names the database entry it was read through; sought in a `--context` document it has no entry to name, because `FR-RND-019` resolves none and `FR-RND-022` opens no connection, so the document's **path** stands where the entry stands. Both name the database | `FR-ERR-034`, `FR-RND-032`, `FR-SCH-010`, `FR-RND-019`, `FR-RND-022` |
| A repetition the specification **permits** is not the repetition `FR-CLI-014` refuses: `--set` is repeatable, so a second occurrence is correct and a second occurrence of the same **key** is the fault, reported with both values | `FR-RND-008`, `FR-RND-014`, `FR-CLI-014`, `FR-ERR-034` |
| Each render bound is a variant of its own carrying the resolved value, so the `cause` names the bound, the value and the key that raises it, and the `hint` names the `tpl cfg set` that raises it | `FR-RND-036`, `FR-RND-037`, `FR-RND-039`, `FR-ERR-034` row `65`, `FR-ERR-009` |

`main.rs` reads the exit status and returns it; it performs no classification of
its own, and the eight-step validation order that decides which code wins when
several conditions are unsatisfied is `architecture.md`'s (`FR-ERR-006`,
`FR-ERR-007`).

**The interim reading of the `70` row is discharged.** Until 2026-09-22 a leaf
the parser accepted and no sprint had written was a detected invariant
violation raised through the same guard, so a caller reached `70` from an
ordinary invocation; every leaf now has its implementation
([`OD-30`](open-decisions.md#od-30--a-parsed-leaf-with-no-implementation)). The
`#[cfg(test)]` trigger of `FR-ERR-031` was untouched by it and remains absent
from the artefact.

**stdout is empty on every error path.** `--format` is ignored, the four lines
go to stderr, and no error is ever a JSON document — so `output/` is not reached
at all once an error is the outcome (`FR-ERR-033`, `FR-OUT-015`,
`FR-OUT-032`).

## The diagnostic renderer

`diagnostics/` receives an error value from `error.rs` and composes what reaches
stderr. It holds transformations, not a taxonomy.

| Obligation | Forced by |
|---|---|
| Four labelled lines, in one order, and they are the whole of what a caller receives | `FR-ERR-008`, `FR-ERR-033` |
| `cause` is factual, specific, never a restatement of `error`, and never wording that would be equally true of a different failure; its per-code content is obliged row by row | `FR-ERR-010`, `FR-ERR-034` |
| `hint` carries a runnable command wherever one exists, built only from literals and from names matching the admitted character set; a candidate outside it is presented in **no** form and the generic hint stands alone | `FR-ERR-009`, `FR-ERR-022`, `FR-ERR-023` |
| Every interpolated value is escaped — newline, carriage return, **tab**, and every C0 control — and the renderer escapes each composed line as a whole rather than each interpolation | `FR-ERR-024`, with [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) |
| No credential, resolved DSN, argument vector, child stderr, raw driver error or `.cfg` content reaches the stream at any verbosity | `FR-ERR-013`, `BR-ERR-003`, `FR-GLOB-018` |

Tab is escaped here and excepted in `text` read output; the two rules are
deliberately different, which is why `output/` and `diagnostics/` are two
modules ([`OD-05`](open-decisions.md#od-05--the-module-decomposition)).

**Suggestion selection.** At most three candidates within an edit distance of
two, ordered by distance and then by name, over eight populations; where no
candidate qualifies the suggestion is omitted rather than weakened
(`FR-ERR-019`, `FR-ERR-020`, `FR-ERR-021`). The distance is the one
`FR-ERR-039` fixes, implemented in the crate
([`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms));
its cost as a measured path is
[quality-attributes.md](quality-attributes.md#the-failure-path-is-a-measurement-point).
The **population** is supplied by whichever component owns it — object names by
`mariadb/`, template names by `render/`, entry and key names by
`project/config.rs`, command and flag names by `cli/` — and `diagnostics/` owns
only the selection. One population is narrowed by its own requirement: a failing
segment of a help path suggests over the **children of the node reached**, not
over the tree (`FR-HELP-028`).

**The parser's own rejections are intercepted and re-rendered**, settled in
[`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics): `cli/` hands
`diagnostics/` the token as written and why it was rejected, through the error
value, and no byte the argument parser would render reaches a caller. **Where
the parser names no token, none is handed on**: the refusal is still `64`, and
the `cause` says that the invocation was rejected and that no token was named.
That is the wildcard arm of the mapping, and the one refusal of this tree that
reaches it is a value that is not valid UTF-8; the entry records the observed
wording and why the degradation is the safe direction.

The four parsing rules this obliges the `cli/` surface to hold itself are:

| Rule | Forced by |
|---|---|
| No command and no long flag is inferred from a prefix, and no token is looked up on `PATH` | `FR-CLI-004`, `FR-CLI-005`, `FR-CLI-006`, `FR-CLI-002` |
| A single-value flag given twice is `64` **naming both values**, so both occurrences are retained rather than the last winning | `FR-CLI-014` |
| A global flag is accepted at any position and at every depth, while a local flag is rejected by any node that does not declare it | `FR-CLI-024`, `FR-GLOB-002`, `FR-CLI-019` |
| A separate-token flag value beginning with `-` is `64` with the corrected joined form in the hint; `--` terminates arguments; case is never normalised | `FR-CLI-018`, `FR-CLI-017`, `FR-CLI-020` |

Five short forms exist in the whole tool and no other flag declares one
(`FR-GLOB-024`), which makes the short-flag space a property of the parser tree
rather than of each node.

**Two of the four rules are held in `cli/`, and two are the parser's own
behaviour.** The repetition of `FR-CLI-014` and the pair of `FR-CLI-015` are
refused by `cli/` rather than declared on the arguments, because a refusal
written in the parser's words is a refusal the caller never reads in the four
labelled lines of `FR-ERR-008`; the repetition is read over the **declarations**
— every flag declared as appending, less the one flag a requirement makes
repeatable — so a flag added to the tree is governed without that rule changing.
The argument terminator of `FR-CLI-017` and the byte-for-byte match of
`FR-CLI-020` are what the parser already does, so each is asserted by a test
rather than implemented: a default is the one property of a dependency that can
change with nothing in this project changing. Step 1 of `FR-ERR-006` is
therefore three things in one — what the parser refused, then `FR-CLI-014`, then
`FR-CLI-015` — and the order among the three is `cli/`'s own, because
`FR-ERR-006` fixes the order between steps and not within one: a refusal that is
a property of **one** flag precedes a refusal that is a property of **two**.

**The diagnostic sink is a closed set of typed emission functions**, so the six
categories `FR-GLOB-018` forbids have no home to be written from, and the one
line per catalogue query carries a fixed leading token that makes it
distinguishable (`FR-GLOB-017`, `NFR-PERF-008`). Phase timings come from
`deadline.rs`, which already holds them. The decision and its consequences are
[`OD-17`](open-decisions.md#od-17--observability); observability as a subject is
`operations.md`.

## The document emitter

`output/` owns one envelope, seventeen payloads, one ordering rule, one
`text` layout rule, one escaping rule and one writer. Its inputs are `model/`,
`cli/help.rs`, `cli/cfg/` and `cache/`; its output is stdout.

| Obligation | Forced by |
|---|---|
| One outer shape of exactly three keys in a fixed order, governing **every** JSON document without exception; nothing is placed beside the payload and the envelope never grows a fourth key | `FR-OUT-024`, `FR-OUT-028`, `FR-OUT-032` |
| `source` is present on every document and takes one of exactly four enumerated values, never a boolean and never on an object | `FR-OUT-026`, `FR-CDOC-009`, `FR-CDOC-010`, `FR-OUT-029` |
| The payload is one key, named for the collection in the plural or for the kind in the singular | `FR-OUT-030`, `FR-OUT-031` |
| Seventeen payload shapes, each fixed by the module that owns its command and nowhere else | `BR-OUT-002`, with `FR-SCH-030` … `FR-SCH-036`, `FR-TMPL-028` … `FR-TMPL-031`, `FR-CACHE-034`, `FR-CACHE-035`, `FR-CFG-035` … `FR-CFG-040`, `FR-HELP-017` … `FR-HELP-019` |
| Compact is the default — one line, no superfluous whitespace, one terminating newline — and the indented form is a two-space indent, one key per line, requested by `--pretty` and by nothing else | `FR-OUT-007`, `FR-OUT-008`, `FR-OUT-009`, `FR-OUT-010` |
| An absent value is `null` and is not omitted, with exactly two exceptions | `FR-OUT-012`, `FR-CFG-037`, `FR-PRIV-016` |
| An empty result is success: the header row alone in `text`, the collection key with an empty array in `json` | `FR-OUT-033` … `FR-OUT-037` |

**Where the four values of `source` are decided.** `cache/` decides between
the two catalogue values, because it is the component that knows which served
the read (`FR-SCH-035`, `FR-CACHE-012`, `FR-CDOC-011`); the other two are fixed
per command and reach `output/` as constants — the project value for `template`
and `cfg` documents (`FR-TMPL-030`, `FR-CFG-035`) and for the cache-status
document, which reports *on* the cache rather than being served from it
(`FR-CACHE-034`), and the binary value for the command tree (`FR-HELP-017`). The
one command in the `cfg` group that contacts a server carries the server value
(`FR-CFG-035`, `FR-CACHE-010`).

**Key order is stated in the type and nowhere else.** The payload types'
field declaration order **is** the emitted key order, and no unordered map
appears on the emitting path; the two map-shaped documents fall to the default
ordering rule rather than to insertion order
(`FR-OUT-013`, `FR-HELP-023`,
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
The same types deserialise a supplied context document, which is what makes the
dump round trip inverse by construction rather than by two routines kept in step
(`FR-SCH-022`, `BR-SCH-004`). Which of them are the model's own, and the four
that are not, is
[The two directions over the document](#the-two-directions-over-the-document).

**The two omissions are different mechanisms because they are different
things.** A key absent from the configuration file is a key never inserted into
the document `output/` receives — `cli/cfg/` builds it from the keys the typed
document carries, and the typed document carries no default to apply
(`FR-CFG-037`, `FR-CFG-014`). A
`restricted` array is a genuinely optional property of the object it qualifies
and is the one field carrying the omission attribute (`FR-PRIV-016`). Both are
settled in
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions).

**C0 escaping has two paths and they differ.** In `text` read output `output/`
escapes every interpolated value whatever its source, excepting tab, because the
aligned columns are laid out with it; in `json` the format admits no raw control
character, so the escape the format defines is what satisfies the rule and the
encoder supplies it (`FR-OUT-018`, `FR-OUT-006`, `FR-OUT-019`). Two outputs are
exempt and are emitted byte for byte: a rendered result and a printed template
source (`FR-OUT-019`, `FR-TMPL-015`). `text` is not a contract and may change
without a version bump (`FR-OUT-004`, `FR-OUT-005`, `FR-SCH-027`).

**The writer knows whether it is inside a document.** `output/` writes through
one buffered writer that records whether any byte of a JSON document has been
emitted. stdout closed before that point ends the process silently at `0`;
closed after it, the consumer holds truncated JSON and cannot tell, so the
outcome is `74` (`FR-ERR-025`, `FR-ERR-026`, `FR-OUT-021`,
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
Results go to stdout and everything else to stderr, and a command that produces
no result writes nothing (`FR-OUT-020`, `FR-OUT-023`, `BR-CLI-004`).

## Ordering: one default and six exceptions

Every ordering is applied by `output/` and none is inherited from the server or
from the filesystem (`NFR-DET-002`, `NFR-DET-001`). The default is by name,
ascending, byte-wise. Six collections are excepted, and each exception's order
is carried **from** `mariadb/` rather than re-derived:

| Collection | Order | Fixed by |
|---|---|---|
| A table's columns | Ordinal position | `NFR-DET-002` |
| An index's columns | The order the catalogue states | `FR-CAT-010` |
| A primary key's columns | The order the catalogue states | `FR-CAT-043` |
| A foreign key's columns, and the referenced columns paired with them | The order the catalogue states | `FR-CAT-045` |
| An `ENUM` or `SET` member list | The order the catalogue states | `FR-CTX-016` |
| A routine's parameters | Declaration order | `FR-CAT-018` |

Three of the six would be **silently corrupted** by the default, which is why
the reader must preserve an order rather than hand `output/` an unordered set: a
sorted primary key is a different key, an independently sorted foreign-key
column list pairs each column with the wrong counterpart, and a sorted member
list renumbers every ordinal (`NFR-DET-002`). Two further orderings are fixed
outside that table and fall to the default rule over a **derived** string: a
template listing is ordered by the displayed name, with the extension removed,
independently of directory iteration order (`FR-TMPL-013`), and the `restricted`
array is ordered by property name (`FR-PRIV-016`). Byte-wise comparison, never a
collation, is what makes an order identical on every machine and in every locale
(`NFR-DET-002`, `FR-SCH-014`, `FR-SCH-028`).

**Recorded discrepancy — which component applies the ordering.** This section
and [quality-attributes.md](quality-attributes.md#the-six-requirements-of-form)
place every ordering in `output/`. As built, the `text` path orders there and
the **document** path orders in `model/document/`, at the point each collection
of the document is assembled. `NFR-DET-002` decides neither reading: it requires
an ordering to be explicit, stable and not inherited from the server or the
filesystem, and names no component. Both readings are recorded. Nothing
observable distinguishes them — the ordering is applied before any byte is
written, and `output/` receives a document already in order — and the model's
placement carries one property this section's does not: the default rule is
bounded by a trait the excepted member types do not implement, so a collection
`NFR-DET-002` excepts cannot be handed to it. Which placement governs is this
folder's to settle in a pass of its own, and not here.

**Recorded gap — the order of `referenced_by`.** An entry of that collection is
a pair of a referencing table and the key it declares, so it has two names and
the default rule of `NFR-DET-002` is written over one. The corpus names no
order for it and the collection is not among the six exceptions. As built the
entries are ordered by the referencing table's name and then by the constraint's,
which is total — a table may reference another more than once — and byte-wise
like every other. The gap is reported to the functional owner rather than closed
here.

## The two directions over the document

`model/document/` owns both directions over the one document that carries the
model, and they are inverse because they are two directions over **one** set of
types rather than two routines kept in step (`FR-SCH-022`, `BR-SCH-004`,
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
The document's keys, depths and cuts are `specification/context-document.md`'s
and are cited here, never reproduced.

**The types are the model's own, with four exceptions, two projections and one
flattening.**

| Shape | Whose type | Why |
|---|---|---|
| A column, an index, a trigger, a `CHECK` constraint, a view, a routine, the `server` object, and every member type nested inside them | The model's, serialised directly | Each states its own keys in its own module, so the key order of `FR-OUT-013` is stated once where the fields are |
| The decomposed type of a column | The model's, **flattened into the column** | `FR-CTX-014` gives a column `column_type` and `FR-CTX-015` gives it the eight parts *additionally*, so a serialised column reads `data_type` and never `column_type.data_type`: the nine keys are siblings of the column's own, and the decomposition is a type without being a level of the document. It stays a type because its one constructor is what makes `FR-CTX-040`'s field-by-field reading structural |
| The `database` object, a table, a foreign key, an entry of `referenced_by` | The document's own | Each differs from the model in one respect and the same one: the **embedding** of `FR-CTX-006` and `FR-CTX-010`, which the model carries as a name and the document carries as an object |
| A column default | The model's, projected onto a private tagged shape | `FR-CTX-012` puts the discriminant **inside** the object, beside a `value` the `null` form does not carry, which is neither shape a derive over the model's variants produces |
| A `restricted` marking | The model's, projected onto a bare array of names | `FR-PRIV-016` makes the document shape an array rather than an object, and the projection is where its byte-wise order is applied |

Both projections are declared on the model type, in both directions, so the
document shape is still a property of a type and no writer restates it. The
flattening is declared the same way and holds in both directions, so a document
is read back by the shape it is written in. The document's own types are
`pub(crate)`: the document is contract and the types that write it are not
(`DIV-032`).

**`FR-OUT-013`'s key order survives the flattening**, because the nine keys are
emitted where the field sits: the column's order is its own field order with the
decomposition's spliced in at that position, and it is still stated once, in the
two types, rather than in a writer
([`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).

**What the inward direction checks, and where each check lives.** The document
is untrusted input on this path, and every check is either a derived
deserialisation or a constructor the model already has.

| Checked | Where |
|---|---|
| The three envelope keys, with `source` one of the four values of `FR-OUT-026` | The envelope's derived deserialisation; a bare payload in place of the envelope is refused (`FR-SCH-036`) |
| Every key the document contract names, with the type it fixes | Each type's derived deserialisation. An absent key is a fault, because `FR-OUT-012` emits an absent **value** as `null` rather than omitting the key |
| `version`, `series` and `standing` present and strings, `standing` one of two values | The `server` object's three fields and its closed enumeration (`FR-CTX-031`, `FR-CTX-033`, `FR-CTX-034`) |
| No key of a table names a column that table does not carry | The one constructor a table has (`FR-CAT-044`) |
| Every table a foreign key names — the referenced table under `foreign_keys`, the referencing table under `referenced_by` — is a member of `tables`; a `null` referenced table names none | `references_are_carried()` in `src/model/document/read.rs`, over the tables already read back; a fault is `ContextFault::DanglingReference`, naming the carrying table, the collection, the key and the table named (`FR-CTX-042`) |
| A `restricted` marking names at least one property | The marking's fallible conversion (`FR-PRIV-016`) |

**And four things it does not check**, two of them because `FR-CTX-033` forbids
it in as many words and two because no requirement asks for them.

| Not checked | Why |
|---|---|
| `series` against the supported window | `FR-CTX-033` forbids it. No connection is opened on this path (`FR-RND-022`), so there is no server to vouch for, and validating the window would make every committed dump expire on a calendar date as the window moved |
| `standing` against `series` | `FR-CTX-033` forbids it in as many words |
| The value of `schema_version` | `FR-OUT-014` fixes what moves it and states no rule for refusing a value, so refusing one would be a check the corpus does not ask for |
| `primary_key` against `indexes` | `FR-CAT-043` makes the index collection the authoritative source and bars a second, and `BR-CTX-003` is the ground: the document presents the key twice so that a template need not match on a name, and reading both back would give the model a second place the two could disagree from. The document's `primary_key` is therefore not read at all |

A column's `table_name` is not a reference under `FR-CTX-042` and is not
checked: an absent table fails the render when a test resolves it, which
`FR-SEM-017` and `FR-SEM-018` make reachable on purpose. The embedded copies of
`FR-CTX-006` and `FR-CTX-010` are not checked apart from the members they copy.

**Nothing is repaired.** A document that is JSON and does not match the contract
is `65`, carrying the rule it failed; one that is not well-formed JSON is `65`
carrying the position the decoder stopped at. Both halves are what the `65` row
of `FR-ERR-034` obliges the `cause` line to carry, and the path is added by the
caller that opened the file (`FR-RND-020`, `FR-ERR-029`, `FR-ERR-034`).

**The outward direction can fail, and for one condition.** A foreign key naming
a table the model does not carry is a violated internal invariant, `70`, which
`FR-CTX-023` makes unreachable for a model produced by a server read and
`FR-CTX-042` unreachable for a supplied document, whose read-back refuses it
with `65` first.
Materialising the embedding is what makes it detectable at all: an emitter that
reproduced the cut at write time would have written the document and left the
caller to find the dangling reference
([`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md)).

## The help surface

`tpl` renders all seven sections itself, with the argument parser's own help
flags disabled; the parser is kept for parsing and for the tree introspection.
Settled in [`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own),
whose rationale and rejected option are not restated.

| Obligation on `cli/help.rs` | Forced by |
|---|---|
| Six forms, and three equivalences that are **byte-identical** at every depth of the tree | `FR-HELP-001`, `FR-HELP-002`, `FR-HELP-003`, `BR-HELP-001` |
| Seven sections in a fixed order; four always appear and an empty section is otherwise omitted; a node's children are listed as its first positional argument | `FR-HELP-006`, `FR-HELP-007`, `FR-HELP-008` |
| A fixed 80-column layout with the line breaks written into the text; the terminal width is never read and nothing is ever reflowed | `FR-HELP-009`, `FR-HELP-010` |
| Each flag and argument states its type, default, whether it is required, its enumerated values, whether it is repeatable, and any mutual exclusion; the global flags are listed once, at the root | `FR-HELP-013`, `FR-GLOB-003` |
| Help is self-contained, carries at least one correct example, lists only the codes its command can produce, and contains no colour, emoji or decoration | `FR-HELP-012`, `FR-HELP-014`, `FR-HELP-011`, `FR-HELP-015`, `BR-HELP-002` |
| A group node with no child prints exactly the text its help form would print, and exits `0` | `FR-CLI-007`, `FR-HELP-025`, `FR-CLI-009` |

**The renderer reads two sources and no third.** The **parser tree** `cli/`
declares, read **unbuilt**, supplies `USAGE`, `ARGUMENTS` and `OPTIONS`; the
typed table supplies `DESCRIPTION`, `EXAMPLES`, `EXIT CODES` and `SEE ALSO`.
Reading the tree unbuilt is what puts the seven global flags at the root alone,
by construction rather than by a filter: the parser propagates a global argument
into a subcommand when the tree is **built**, so no other node's `OPTIONS` can
carry one (`FR-GLOB-003`). Below the root the seven are acknowledged in `USAGE`
as `[options]`, which names no flag and is true of every node (`FR-GLOB-002`).

**Five of the six facts of `FR-HELP-013` are introspected**: the type, the
enumerated values where there are any, the default, whether the argument is
required, and whether it is repeatable. Repeatability is read from the refusal
rule of `FR-CLI-014` and never from the parser's action, because a flag carrying
a **single** value is declared as appending so that both values reach the
message that requirement obliges
([`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics)); the action
would state the reverse of what the tool enforces.

**Recorded divergence — `FR-OUT-009` was refused nowhere, and is discharged.**
That requirement makes `--pretty` without `--format json` a `64` on a command
that declares both, and the `EXIT CODES` section of every such command already
states it. In the working tree of 2026-09-17 no code refused it: `tpl cfg list
--pretty` wrote the compact document and exited `0`, while `cli/` carried the
rule in a doc comment saying it was deliberately not declared on the argument,
with no arm refusing it either. The divergence was recorded rather than closed
by narrowing: `/specification` governs, so the requirement stood and the built
behaviour was the defect.

**Discharged at commit `243c4d6`.** `rules::refuse_pretty_without_json` in
`src/cli/rules.rs` applies it, at every node that declares both flags and
recursively into each matched subcommand, and yields
`Error::MutuallyExclusiveFlags` naming both members — which is what the `64` row
of `FR-ERR-034` obliges the `cause` line to carry. `--pretty` written with no
`--format` at all is caught by the same arm, because the declaration gives
`--format` a default and the first occurrence is the format in force.

| Invocation | Exit | Verified |
|---|---|---|
| `tpl cfg list --pretty` | `64`, with the four labelled lines of `FR-ERR-008` | by invocation, 2026-09-21 |
| `tpl cfg list --pretty --format json` | `0`, and the document indented | by invocation, 2026-09-21 |

`FR-OUT-009` was last re-read against the **thirty-second edition** of
`/specification`, the current one, and carries no amendment note in any edition:
it stands exactly as first written, and the built behaviour now matches it. The
record of the divergence is kept rather than deleted, so that an identifier
resolves to what happened.

**The sixth fact, mutual exclusion, is stated on the argument's own entry.**
`FR-HELP-013` obliges help to state it and the tree has nothing to introspect:
no argument declares `conflicts_with`, because every refusal the corpus obliges
— the exclusions of `FR-CLI-015`, `FR-RND-005`, `FR-CFG-016` and `FR-CFG-029`,
and the dependency of `FR-OUT-009` — is refused away from the parser, for the
reason [the diagnostic renderer](#the-diagnostic-renderer) gives. The fact is
therefore carried by the typed table, as `Documented::excludes`, spelled as the
reader meets it — a long form with its dashes — and empty where the argument
excludes nothing. It is one place, uniformly at every node, and it reaches both
channels: the sentence in the text help, and the `excludes` array of each
argument in the JSON document. That is where `FR-HELP-022` puts a fact neither
channel may derive from the other.

**`EXIT CODES` was rejected, on three grounds.** The obligation had been met in
the prose of the `EXIT CODES` section of the command that **owns** each refusal.

| Ground | What it says |
|---|---|
| **The decisive one.** The global pair has no `EXIT CODES` line to live in | `-q/--quiet` with `-v/--verbose` is declared at the root, and the root's `64` line carries an unknown command, an unknown flag and a repeated flag value — never that pair. The route answered `FR-HELP-013` for every local pair and for the global one not at all, which is an obligation met inconsistently |
| The requirement puts the fact on the argument | It is the sixth of six facts about **the value**, beside the type, the default, whether it is required, the permitted values and repeatability. Five are introspected onto the argument's entry; the sixth belongs where they are |
| A code's section answers a different question | `EXIT CODES` says what produces a code. What an argument may be written with is not that, and is not addressable per argument in the JSON document from there |

The `EXIT CODES` prose is untouched by the decision. Built at commit `243c4d6`;
`src/cli/help/render.rs` carries the same three grounds beside the renderer, and
`tests/help_surface.rs` holds two tests to it — one over the global verbosity
pair, one over every exclusion the document declares, each asserting that the
text help of the node names it.

**`70` is listed in one help text and no other**: the root's. `FR-ERR-030` makes
its two producing conditions defects in `tpl` rather than conditions of any
command under it, `FR-HELP-011` admits it at the root because a panic is
reachable from any invocation, and `BR-HELP-002` refuses repeating at a level
what has already been said — which is the shape `FR-GLOB-003` already fixes for
the global flags.

**The command tree is introspected at runtime from the tree the parser actually
parses with** (`FR-HELP-021`), so `cli/` is the producer and `cli/help.rs` the
consumer; the document is emitted through `output/` and obeys every rule of the
JSON contract (`FR-HELP-024`, `FR-HELP-016`, `FR-HELP-029`). Three consequences
follow at this boundary:

- **A node the parser accepts is a node the document publishes.** That is why
  neither test seam can be a command or a flag, and why both live inside the
  process (`FR-ERR-031`, `FR-SRV-035`,
  [`OD-21`](open-decisions.md#od-21--two-test-seams-that-must-not-be-on-the-published-surface)).
- **Examples and exit codes cannot be introspected**, so they come from a typed
  table indexed by command path which feeds the text help and the JSON document
  alike, and neither is ever parsed out of rendered help (`FR-HELP-022`).
  [data-model.md](data-model.md#what-this-document-defers-and-to-what) defers
  that table here.
- **Declaration order is preserved throughout and no unordered map appears on
  the path** (`FR-HELP-023`), which is the same obligation `FR-OUT-013` places
  on every other document.

The per-command content is fixed by `FR-HELP-018`, `FR-HELP-019` and
`FR-HELP-020`; the template surface is published in the same document, in three
groups (`FR-ENV-005`), which makes `render/` a second producer into
`cli/help.rs`. The three properties the tree carries as tests are `BR-HELP-003`.

**The document as built.** It is the envelope of `FR-OUT-024` with `source` set
to `binary` (`FR-OUT-026`), and `data` carries the four keys `FR-HELP-017` fixes,
in that order. `commands` is flat and excludes the root; each entry carries its
`path` as an **array of segments**, which is the vector a caller hands straight
back to `tpl help` (`FR-HELP-026`), and `inherits_globals` after the seven
members `FR-HELP-019` names. The reduction of `FR-HELP-029` is a pre-order walk
from the node the path resolved to, so a subtree is the walk rather than a
filter applied to the whole array. `template_surface` carries the three
groups of `FR-ENV-005` in the shape that requirement fixes, and **every array
the corpus can enumerate is now published**: `registered` carries the eleven
filters, seven tests and five functions `render/` actually registers, and
`inherited.filters` the fourteen names of `FR-ENV-018` in the order that
requirement states them. `inherited.tests`, `inherited.functions` and the three
arrays of `other` stay as `FR-ENV-005` fixes them — two empty and three `null`
(read from `tpl help --format json`, 2026-09-21). Both shapes, and the points
the implementation derives under them, are
[`OD-29`](open-decisions.md#od-29--the-json-command-tree-two-shapes-and-what-the-binary-publishes).

**The six forms reach one renderer and one version line**, which is what makes
the three equivalences of `FR-HELP-002` hold by construction rather than by
comparison: `tpl help <path>`, `tpl <path> --help`, `tpl <path> -h` and a bare
group node arrive at the first, and `tpl version`, `tpl --version` and `tpl -V`
at the second. `tpl help <path>` and `tpl <path> --help` remain two distinct
routes **through the parser**, which is what `BR-HELP-001` binds by test:
[verification.md](verification.md#help-snapshots-at-every-depth).

**A required operand does not hide the two flag forms.** `OD-07` turns the
parser's own help flag off, so `-h`, `--help`, `-V` and `--version` are ordinary
global arguments and the parser validates required arguments first; at the
fourteen required operands the tree declares over thirteen leaves, the strict
parse refused `tpl <node> --help` before the flag was read. It is resolved by a
**second parse**, over the same tree with the requirement waived, narrowed to
the parser's missing-argument refusal and honoured only where one of the two
flags is present; a vector that names neither falls through to the first
refusal with its message intact, and the parsing rules run over the second parse
exactly as over the first, because step 1 of `FR-ERR-006` admits no exception.
The declarations are untouched, so requiredness still reaches the help of
`FR-HELP-013` and the document of `FR-HELP-021`. This is neither the argument
pre-scan `FR-ERR-017` and `FR-ERR-018` withdrew nor the second set of rules
`OD-08` rejects: it is the same parser over the same tree, differing in one
validation setting, on a path that has already failed.

**Path resolution is `cli/help.rs`'s own.** A path of any depth is accepted as a
sequence of positional arguments, each segment resolved by the rules of the
tree: an alias resolves to its canonical node and no segment is inferred from a
prefix (`FR-HELP-026`, `FR-HELP-027`, `FR-CLI-011`). A segment that names no
child is `64` with a suggestion over that node's children, and the `cause` names
the segment and the node it was looked for under (`FR-HELP-028`).

## The template surface

`render/` registers the surface and builds the engine; `cli/` reaches it for
four commands and for the render. The engine pin, the two deliberate shadowings
of engine built-ins and the guarantee that attaches to each group are
[`ADR-001`](../adr/adr-001-template-engine-pin.md), registered as
[`OD-13`](open-decisions.md#od-13--the-engine-pin-and-minijinja-contrib), and
are not restated here.

| Group | Contents | Guarantee | Fixed by |
|---|---|---|---|
| 1 | Eleven filters, seven tests and five global functions that `tpl` registers | Full contract; adding a name is not breaking, renaming or removing one is | `FR-ENV-001`, `FR-ENV-002`, `FR-ENV-006`, `FR-ENV-007`, `FR-ENV-014`, `FR-ENV-020`, `FR-ENV-029` |
| 2 | Fourteen inherited filters, enumerated and **closed** | Against the pinned engine line | `FR-ENV-018`, `FR-ENV-019`, `FR-ENV-003` |
| 3 | Everything else the engine offers | None, and the help says so | `FR-ENV-004` |

Three obligations bind the registration itself:

- **Two names are registered over engine built-ins of a different arity**, each
  taking a required argument where the engine's takes none, so a template
  written against the engine's signature fails loudly rather than emitting
  different bytes (`FR-ENV-037`, `FR-ENV-044`,
  [`ADR-001`](../adr/adr-001-template-engine-pin.md)). A third registered filter
  takes a required argument on the same ground (`FR-ENV-038`).
- **No registered name coerces.** A filter or a test handed an operand of a type
  it does not accept fails the render, naming the name, the type received and
  the location; it never returns an empty string and never answers `false`
  (`FR-SEM-005` … `FR-SEM-009`, `FR-ENV-034`, `FR-ENV-040`, `FR-ENV-039`).
- **The set of global functions is closed** (`FR-ENV-020`), which is what makes
  the four capability prohibitions structural rather than reviewed: there is no
  registered name that reads the environment, a file, the network or a clock
  (`FR-ENV-022` … `FR-ENV-025`). `security.md` owns them as a subject.

**The three type families are a table indexed by the normalised type name**,
disjoint and total: a value in none of the four rows satisfies none of the three
tests and does not fail the render (`FR-ENV-041`, `FR-ENV-042`, `FR-ENV-046`).
[data-model.md](data-model.md#what-this-document-defers-and-to-what) defers the
table here; the type decomposition it indexes is
[data-model.md](data-model.md#a-column-type-eight-parts-and-the-raw-string)'s.

**Auto-escaping is off and is never keyed on a name.** Escaping happens where a
template asks for it, through the registered filter and nowhere else
(`FR-ENV-026`, `FR-ENV-027`, `FR-ENV-028`).

**One function resolves a template name to a path, and it is the only resolution
in the crate.** The loader closure calls it and so do the three `template`
subcommands that resolve a path without the engine. The sequence, the two codes
and why the engine's own path helper is not wrapped are settled in
[`OD-15`](open-decisions.md#od-15--the-template-loader) and are not restated.
What the boundary obliges of `cli/`: the optional extension is completed on the
command line, before the resolution is asked for, so the engine never sees a
name `tpl` completed and a name inside a template stays literal (`FR-TMPL-007`,
`FR-TMPL-008`, `BR-TMPL-003`, `FR-TMPL-006`). What it obliges of `render/`: a
file that is not a regular file under the root ending in the template extension
is invisible to listing, render and include alike (`FR-TMPL-004`,
`FR-TMPL-005`), and the containment checks are enforced at that one point
(`FR-TMPL-023` … `FR-TMPL-026`).

**Parsing is separable from rendering**, because the check command performs
syntax analysis only: no expression evaluated, no filter called, no connection
opened (`FR-TMPL-017`, `FR-TMPL-018`, `FR-TMPL-019`, `FR-TMPL-020`,
`BR-TMPL-001`, `FR-SEC-018`). No `template` subcommand opens a connection, reads
the cache or requires an entry (`FR-TMPL-003`), and the root and one template's
path are both reported absolute (`FR-TMPL-021`, `FR-TMPL-022`).

## Context access from a filter or a test

`cli/` assembles the render context from four sources and hands it to `render/`;
two registered tests then need to reach **back** into it.

| Variable | Source | Fixed by |
|---|---|---|
| The database, and the bound object selected by an object flag | The context source: a catalogue read or a supplied document | `FR-RND-023`, `FR-RND-006`, `FR-RND-003` |
| The caller's variables | The `--set` flags of this invocation | `FR-RND-023`, `FR-CTX-026` |
| The binary's own object | The binary | `FR-CTX-027` |
| The render time | The clock, once per invocation | `FR-CTX-028`, `FR-CTX-029`, [`OD-25`](open-decisions.md#od-25--the-clock-source-for-now) |

The last three are **always** injected and a value supplied for them in a
context document is ignored (`FR-RND-024`), which is why a dump carries the
server-derived part alone (`FR-SCH-018`). The `--set` surface is validated in
`cli/`: split on the first `=`, an empty value admitted, a key matching the
declared pattern, a dotted key rejected rather than split, a duplicate key `64`,
and every value a string (`FR-RND-008` … `FR-RND-015`). A supplied document is
untrusted input and is validated **structurally only** — the envelope whole
rather than a bare payload, the three server keys present and strings, the
standing within its enumerated range, and never against the current window
(`FR-SCH-036`, `FR-RND-020`, `FR-RND-021`, `FR-CTX-033`); supplying it alongside
an explicit entry is `64`, and it opens no connection and touches no cache
(`FR-RND-018`, `FR-RND-019`, `FR-RND-022`).

**Two of the seven tests answer from the context and not from their operand.**
Nothing a table already states is materialised on a column, so the tests resolve
the column's table name against the render context and answer from what the
table states (`FR-CTX-019`, `FR-CTX-021`, `FR-CTX-022`, `BR-CTX-003`,
`FR-ENV-015`, `FR-ENV-016`, `FR-ENV-041`). A table that is absent from the
context fails the render rather than answering `false`, whether the column came
from a server or from a supplied document (`FR-ENV-017`, `FR-ENV-043`,
`FR-SEM-017`, `FR-SEM-018`). The other five consult the operand alone.

The engine supplies the access: "A read only reference is passed to filter
functions and similar objects to allow limited interfacing with the engine", and
its lookup method "Looks up a variable by name in the context" (docs.rs
`minijinja::State`, minijinja 2.24.0, verified 2026-09-11). The documented
limitation — that the lookup cannot find a variable unless it is a global, was
passed in the initial render context, or was referenced by a macro (same source
and date) — does not reach this case: the database is a top-level variable of
the initial context under `FR-RND-023`. The claim is bounded to that: nothing
here promises a lookup of a variable introduced inside a macro or a call block.

## The configuration reader and the writer

`project/` owns one key space and two paths over it — `project/config.rs`, which
reads into a typed document, and `project/edit.rs`, which rewrites the file
preserving its format. The split, its rationale and the rejected options are
[`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path); the
file's format, mode, rewrite discipline and key count are
[data-model.md](data-model.md#tplcfg) and are not restated. The submodule
division, and why the two paths are two modules, are
[architecture.md](architecture.md#inside-project).

**The read path.**

| Obligation | Forced by |
|---|---|
| The type, ownership and mode checks are a **precondition** of reading, made on the descriptor the file is then read through, below a canonicalised path, and an explicitly named project folder is not exempt from them | `FR-PROJ-009`, `FR-PROJ-030`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-PROJ-008`, `FR-GLOB-010` |
| The value the reader hands `error.rs` on a malformed file carries the **position** of the fault as well as the key. The strictness that makes this reachable — an unrecognised key anywhere is fatal — is [data-model.md](data-model.md#tplcfg)'s and is not restated | `FR-CONF-034`, `FR-CONF-035`, `FR-ERR-034` |
| The whole file is walked for keys outside the space **before** any value is read, so a file carrying both a misspelled key and a malformed value reports the misspelling | `FR-CONF-034`, `FR-CONF-002`, `FR-ERR-007` |
| An **absent** file is an empty document rather than a failure, once the `.tpl` folder is found to be the caller's: the project is the folder, and the directed write surface must be able to write the file again | `FR-PROJ-028`, `FR-PROJ-001`, `FR-PROJ-017`, `FR-CFG-004` |
| A setting resolves through exactly two layers above a built-in default, with no environment layer; the resolver therefore has three inputs and no fourth | `FR-CONF-029`, `FR-CONF-030`, `FR-CLI-022` |
| An entry selected on the command line is **distinguishable** from one resolved through the configured default, because two render rules depend on the distinction | `FR-GLOB-008`, `FR-RND-018`, `FR-RND-019` |
| Nothing selected is `78`; a named entry that does not exist is `66` with a suggestion | `FR-GLOB-006`, `FR-GLOB-007`, `FR-ERR-004`, `FR-ERR-005` |
| The five admitted and refused combinations of connection and password keys are decided **before** any connection is opened; a DSN carries no query parameters | `FR-CONF-006`, `FR-CONF-007`, `FR-CONF-011`, `FR-CONF-012` |
| Expansion reaches six fields only, is a single pass, and inside a DSN parses the URL **first**, expands within the delimited field, then percent-encodes; an unclosed form and an undefined variable are both `78` | `FR-CONF-015` … `FR-CONF-022` |

**Three read paths over one file produce three different outputs**, and they are
three code paths rather than one with a flag:

| Path | Expanded | Redacted | Defaults applied | Fixed by |
|---|---|---|---|---|
| The directed key read | No | No | No | `FR-CFG-006`, `FR-CFG-007`, `FR-CFG-036` |
| The whole-file listing and one entry shown | No | Yes | No | `FR-CFG-013`, `FR-CFG-014`, `FR-CFG-019`, `FR-CFG-021`, `FR-CFG-037`, `FR-CFG-038` |
| The resolution a connection uses | Yes | — | Yes | `FR-CONF-004`, `FR-CONF-029` |

The third never reaches stdout. Credentials, the sentinel property and what may
be printed are `security.md`'s.

**The write path.** `cli/` validates a command's arguments and hands
`project/edit.rs` a value already typed; the writer preserves comments, spacing
and the relative order of items, which six requirements make a functional need
([`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path)).

| Obligation | Forced by |
|---|---|
| A value written through the key surface is validated against the enumerated key space **and** the declared type of that key | `FR-CFG-008`, `FR-CFG-009`, `FR-CFG-010`, `FR-CONF-002` |
| An unset accepts a leaf key or a whole block; an absent key or block is `66` | `FR-CFG-011`, `FR-CFG-012` |
| Create and change are distinct verbs: an existing entry refuses creation with a hint, and an update changes the named fields and leaves the rest of the entry untouched | `FR-CFG-015`, `FR-CFG-016`, `FR-CFG-017`, `FR-CFG-020`, `BR-CFG-001` |
| A deletion that removes the entry the configured default names also clears that key, silently and in the same rewrite. The obligation is over the **state**, so both deletions that reach it are bound: the entry-removal command, and an unset given that entry's block. An unset given one field of the entry does not engage it — the entry survives and the reference still resolves | `FR-CFG-022`, `FR-CFG-023`, `FR-CFG-011` |
| A write that would leave a database entry in a refused combination is refused **before** the file is touched: `64`, the file unchanged, the `cause` naming both keys of the pair and the `hint` carrying a runnable repair. The entry judged is the entry as it would stand after the write, and nothing the invocation did not name is removed to make it coherent | `FR-CFG-048`, `FR-CONF-007`, `FR-CFG-020`, `BR-CFG-001`, `FR-ERR-009`, `FR-ERR-034` |
| Every key of an entry has a flag, each mapping to exactly one key, and one flag's name deliberately differs from the key it writes | `FR-CFG-027`, `FR-CFG-028`, `FR-CFG-029` |
| The connection-string flag admits exactly what the file admits — the grammar, the two schemes, no query parameter — validated before the write, stored verbatim when admitted, `64` and nothing written when not; a variable reference stays opaque and unexpanded | `FR-CFG-031`, `FR-CONF-009`, `FR-CONF-010`, `FR-CONF-011`, `FR-CONF-018` |
| The password-command flag accepts a single string, stores the array it splits into, accepts no array on the command line and is not repeatable | `FR-CFG-046`, `FR-CONF-023`, `FR-CONF-025` |
| No `cfg` subcommand writes anywhere but the configuration file, and only one of them contacts a server | `FR-CFG-004`, `FR-CFG-005` |

**One key space, two callers, two codes.** An unknown key supplied to the write
path is `64`; the same key found in the file is `78`. The distinction is the
caller's, not the key space's — one is a malformed invocation and the other a
configuration that cannot be trusted — and both are reached through one
validator (`FR-CFG-009`, `FR-CONF-034`, `FR-ERR-001`).

**One rule about a combination, two callers, two codes.** `FR-CONF-007` decides
which combinations of connection and password keys one entry may hold, and two
components ask it: the reader, for which a refused combination found in the file
is `78`, and the writer, for which a write that would produce one is `64`
(`FR-CFG-048`). Both reach **one predicate**, which carries whether each key of
the rule is declared plus the one property of a value the rule reads — whether
the connection string carries a password. Applying the rule twice would let the
file and the invocation disagree about what an entry may hold, which is the
state `FR-CFG-031`'s twenty-second-edition amendment records as unrepairable by
any `cfg` command.

The writer feeds that predicate the entry the file carries, with the keys the
invocation names declared on top; the reader feeds it the entry alone. The
difference between the two callers is therefore the input and the code, never
the rule.

**The connectivity command reports one field per step.** Four ordered steps —
connect and authenticate, enforce and confirm the read-only session, verify the
series, run the privilege probe — each separately reportable and separately
failing (`FR-CFG-024`, `FR-CFG-039`, `FR-SRV-034`, `UC-004`). The probe is
exactly one statement against the catalogue, restricted to the named database
and read for two facts only; it is not model content, and a negative answer does
**not** change the exit code (`FR-CFG-044`, `FR-CFG-045`). An unsupported series
is `78` with the `cause` stating that connection and authentication succeeded
(`FR-CFG-043`, `FR-SRV-030`); this command reads and writes no cache
(`FR-CFG-025`, `FR-CACHE-010`).

**The project writer.** Creating a project creates the destination directory and
its missing parents, refuses an existing project folder with `73` changing
nothing, reports a failure the filesystem returns with `73`, writes nothing to
stdout, and warns on stderr at `0` when the new project nests inside an existing
one (`FR-PROJ-012` … `FR-PROJ-016`, `FR-PROJ-022`). What it ships is
`operations.md`; where its five artefacts land is
[data-model.md](data-model.md#tpl-on-disk-and-its-five-writers).

## The `password_command` child

`project/password.rs` runs the child and hands `mariadb/` a password;
`project/settings.rs` is its one caller, because the child belongs to the
resolution and not to the read; `deadline.rs` bounds it. It is the only child
process this corpus admits: no other requirement creates one, and `FR-SRV-007`
forbids an external process for reading structure.

| Obligation | Forced by |
|---|---|
| The stored form is an argument array, executed **directly, without a shell**; shell metacharacters are passed through as literal arguments | `FR-CONF-023`, `FR-CONF-024`, `FR-CONF-026` |
| A string supplied to a command is split by POSIX quoting rules on the **command** path and stored as the array; no quoting engine exists on the path that reads the file | `FR-CONF-025`, `FR-CFG-046`, [`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms) |
| The stored command is never expanded, so the environment cannot alter what is executed | `FR-CONF-017` |
| The child is started as the leader of a process group of its own, with `process_group(0)` of `std::os::unix::process::CommandExt`, which uses the child's pid as the group id (Rust standard library documentation, stable since 1.64.0, consulted 2026-09-23) | `FR-CONF-028` |
| The password is the trimmed standard output, read to a cap of 4096 bytes; a child that writes more has its whole process group terminated, and the invocation is `78` | `FR-CONF-027`, `FR-CONF-031` |
| The cap is applied **at the pipe**, by a read bounded at one byte past it, so the process never holds more than that and never truncates a credential into a password it would then send | `FR-CONF-031`, `FR-SEC-024` |
| The child's standard input is the null device, so a child cannot inherit the caller's and read from it | `FR-SEC-023`, `BR-CLI-003` |
| The child's standard error goes to the null device: not inherited, not captured, never quoted | `FR-CONF-032`, `FR-SEC-024` |
| A non-zero exit is `78`, with the `cause` naming the command as stored and the status the child returned | `FR-CONF-033`, `FR-ERR-034` |
| A child that cannot be **started** is a condition of its own, also `78`: its next step is to correct the file, not to read what the command printed | `FR-CONF-033`, `FR-ERR-002` |
| The phase ends when the child has exited **and** its standard output has reached end of file. The deadline bounds the whole phase: a reader thread drains the pipe and a polling loop in the parent sends `SIGKILL` to the child's process group and reports the expiry, without waiting on the pipe; exceeding it is `78` | `FR-CONF-028`, `FR-ERR-027`, [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) |
| The child's exit is observed with `waitid` under `EXITED`, `NOHANG` and `NOWAIT`, which leaves it waitable, and it is reaped only after the group is signalled — or, on success, after the pipe has closed — so its pid, which is the group's id, cannot be reused before the group kill | `FR-CONF-028`, `FR-CONF-031` |
| A descendant that has left the group is not terminated; the reader thread is dropped rather than joined, so the invocation still ends at the deadline | `FR-CONF-028` |
| A bound already spent when the child would be started stops it from being started at all | `FR-GLOB-012`, `FR-CONF-028` |

**Why the parent polls rather than waits.** Three obligations meet on one child
— a deadline, a bounded read, and a child that may never exit — and neither a
blocking wait nor a blocking read can hold all three: either holds the process
past the deadline, and a child writing more than a pipe buffers blocks on its
own write until something drains it. The drain therefore has to run while the
deadline is watched. A child that exits while a descendant holds its standard
output is a phase still running, which is why the loop waits for both the exit
and the end of file, and why the exit is observed without reaping. The runtime of
[`ADR-005`](../adr/adr-005-async-runtime-scope.md) is scoped to `mariadb/` and
is not started for an invocation that never connects, so the watcher is a thread
and a short sleep rather than a task
([`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced)).

## The shared functions and the phase clock

Each has a closed grammar written into the corpus, and each is implemented in
the crate with no dependency; the choice and the rejected libraries are
[`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms).
What belongs here is the surface.

| Function | Input, and where it is applied | Errors it can produce | Fixed by |
|---|---|---|---|
| The pattern matcher | A pattern from `cli/` and a name from `model/`, matched **in memory**, never sent to the server, case-folded over ASCII only; applied **after** coverage | None: a pattern matches or does not. Supplied where it is not declared it is an unknown flag, `64`; supplied to the dump, `64` | `FR-SCH-011` … `FR-SCH-015`, `FR-SCH-021`, `FR-CAT-028`, `BR-SCH-001` |
| The qualified-routine-name parser | One positional argument or one flag value in `cli/`, resolved against `mariadb/`, `cache/`, or a supplied `--context` document | `66` with a suggestion for a name that does not exist; `64` naming both candidates for a bare name that matches a procedure **and** a function, under every circumstance. Each code is carried by two variants, one per population — see [the error type](#the-error-type-and-the-exit-code) | `FR-SCH-008`, `FR-SCH-010`, `FR-RND-032`, `FR-CACHE-024` |
| The word-list tokeniser | A string operand in `render/`; five rules applied once, left to right, with a published eight-row vector | None of its own; the five naming filters that consume it fail `65` on a non-string operand | `FR-ENV-030` … `FR-ENV-033`, `FR-ENV-034` |
| The edit distance | A supplied name and a population, in `diagnostics/` | None; it returns a possibly empty candidate list | `FR-ERR-019` … `FR-ERR-021` |
| ASCII-only case folding | Shared by the matcher and the tokeniser, independent of server, collation and locale | None | `FR-SCH-014`, `FR-ENV-031` |
| The series derivation | The probed version string, in `mariadb/` | A string without the product marker is refused, `78` | `FR-SRV-040`, `FR-SRV-021`, `FR-SRV-041`, `FR-CTX-032` |

One row of the six has two callers and must therefore be one implementation:
ASCII folding, because a locale-dependent second copy would make the same
template produce different output on two machines (`NFR-DET-001`). A seventh
shared rule is not listed because it belongs to another document — the
apostrophe unescaping that serves a literal default and a member list alike,
which is
[data-model.md](data-model.md#a-column-default-one-discriminant-eight-ordered-rows)'s.

**The phase clock is a shared construct with three users** — the runtime inside
`mariadb/`, the child process, and the render — because every phase deadline
composes with one budget measured from process start, and a budget belonging to
three modules belongs to none of them (`FR-GLOB-011`, `FR-GLOB-012`,
`FR-GLOB-013`, `FR-CONF-005`, `FR-SEC-022`). The three mechanisms and what each
phase is bounded by are
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced); the
machinery itself is `architecture.md`.

Four obligations shape its surface, and each is a property of the type rather
than of its callers.

| Obligation | Forced by |
|---|---|
| No deadline crosses a boundary as a bare integer: seconds are one type, carrying the unit the four keys are declared in, and a **positive** integer, so zero is refused where it is written and not where it would expire | `FR-CONF-002`, `FR-GLOB-001` |
| What bounds one phase carries three facts, because a `65` and a `69` must name two of them: **which** of the two bounds applies, its resolved value, and how long the phase may actually run. The phase deadline wins a tie, naming the more specific of two equally true answers | `FR-GLOB-012`, `FR-GLOB-013`, `FR-ERR-027`, `FR-ERR-034` |
| The origin of the overall budget is recorded once, by the crate entry point, before the argument vector is read; a second recording cannot move an origin already being spent | `FR-GLOB-011` |
| The budget the three connection phases share is an **instant**, not a duration, so the three consume one budget in the order they run and the last of them may find nothing left | `FR-CONF-005` |

The resolution of the four values from `[core]` is `project/`'s and the clock's
composition of them with `--timeout` is this module's: `--timeout` takes no part
in the resolution and composes with its result (`FR-CONF-004`, `FR-GLOB-012`).

## The render bounds

Three contracts serve the bounds of `FR-RND-038`; how they compose, and the
10 ms poll interval, are
[architecture.md](architecture.md#the-render-bounds)'s.

| Contract | Obligation | Forced by |
|---|---|---|
| The resolved bounds | `project/settings.rs` hands `render/` one value carrying render fuel, the output limit and the memory limit, each the file's value or the built-in default. Each is a type that cannot hold a value outside its range. Every render made under it — an abandoned one and the one that follows — starts with the whole of each | `FR-CONF-045`, `FR-RND-038` |
| The heap count | `tpl::install_heap_counter(fn() -> usize)` is public, beside `run` and `install_panic_hook`: the binary calls it once, before `run`, with a function reading its allocator's count, and the first installation wins. Reading the count allocates nothing. Not calling it leaves the memory limit without a count | `FR-RND-039`, [`ADR-011`](../adr/adr-011-render-memory-accounting.md) |
| Quiescence | `mariadb::quiescent()` answers whether the calling thread holds no open connection and no driver runtime; `bounded()` in `cli/render.rs` refuses to start a render with `70` when it does not | `FR-RND-040`, `FR-ERR-030` |

The first two render bounds are answered by `render/` as conditions: the
engine's out-of-fuel error, found anywhere in the error chain, becomes
`RenderFuelExhausted`, and a write the counting writer refused becomes
`RenderOutputLimitExceeded`, so neither reaches the caller as a generic render
failure (`FR-RND-036`, `FR-RND-037`). The deadline and the memory limit leave
the process from the watchdog and return nothing.

## The library entry points

The library has four public functions. The binary calls the first three;
`run_from` serves an in-process caller, a test or a fuzz harness:
`install_panic_hook` ([`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md)),
`install_heap_counter` ([The render bounds](#the-render-bounds)), `run`, and
`run_from`. Why `run_from` takes the shape it does, and the options rejected, are
[`OD-34`](open-decisions.md#od-34--an-in-process-entry-point-over-a-supplied-argument-vector)'s.

```rust
pub fn run() -> Result<(), Error>;

pub fn run_from<I, T>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>;
```

| Obligation | Forced by |
|---|---|
| `run()` is `run_from(std::env::args_os())` and nothing else, so the two paths cannot diverge | [`OD-34`](open-decisions.md#od-34--an-in-process-entry-point-over-a-supplied-argument-vector) |
| The first element of `args` is the program name and is not parsed as an argument; the command path starts at the second. A caller passes `"tpl"` first | The parser: "The first argument will be parsed as the binary name unless `Command::no_binary_name` is used" (docs.rs, `clap` 4.6.6, the version `Cargo.lock` resolves, `Command::try_get_matches_from`, whose `_mut` form `cli/` calls, consulted 2026-09-24), and `tpl` does not use `no_binary_name` |
| The four labelled lines are written by `run_from` on the way out, once, and the error is then returned for the caller to map to an exit status | `FR-ERR-008`, `FR-ERR-033`, [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) |
| Three values are set **once per process, and the first call wins**: the instant the `--timeout` budget is measured from, the heap counter, and the argument vector a `hint` writes back. A second call in the same process measures its budget from the first call's start and writes the first call's vector into its hints. The diagnostic level is set again on every call | `FR-GLOB-011`; `FR-RND-039`, [`ADR-011`](../adr/adr-011-render-memory-accounting.md); `FR-ERR-043`; `FR-GLOB-014`, `FR-GLOB-015` |
| Two paths end the process with `std::process::exit` and do not return: the render deadline and the render memory limit, from the watchdog thread. The panic path exits only where the caller installed the hook | [architecture.md](architecture.md#two-exits-that-do-not-return-through-mainrs) |
| The working directory and the environment are read from the process, not from an argument: discovery walks up from the current directory unless `--tpl-dir` is given, `tpl init` compares its destination with it, and `${VAR}` references are expanded from the process environment | `FR-PROJ-004`, `FR-CONF-015` |
| No stability promise: the signature may change in any release | `DIV-032`, [overview.md](overview.md#the-library-api-is-not-a-public-surface) |

A caller that drives `run_from` repeatedly — a fuzz harness — must therefore run
each input where a process exit is acceptable, or bound the input so neither
exit path is reached.

## The library shape: five questions, open

`DIV-032` fixes that the contract runs through the JSON document and the command
line and **not** through the library, and hands five questions to architecture
rather than answering them.
[overview.md](overview.md#the-library-api-is-not-a-public-surface) carries the
consequence; [data-model.md](data-model.md#the-model-in-memory) records that
four of the five are not decided there.

**All five are now answered over the published types**, which are those of
`model/` and `error.rs` and no others
([`OD-05`](open-decisions.md#od-05--the-module-decomposition)); the crate root
also exposes the four entry functions of
[The library entry points](#the-library-entry-points), which carry no stability
promise either (`DIV-032`). Four were
settled when the model was built and are recorded in
[`OD-31`](open-decisions.md#od-31--the-models-shape-strings-fields-and-the-attribute);
the fifth was already settled. None is settled **here**: the register is where a
decision and its rejected options live, and this table states the answer and
cites it.

| # | Question | Answer over the published types | Recorded in |
|---|---|---|---|
| 1 | Owned versus borrowed types in the model | Neither alone: one clone-on-write string type, under one lifetime parameter threaded through every type, so one shape serves a live read, a cached read and a supplied document | [`OD-31`](open-decisions.md#od-31--the-models-shape-strings-fields-and-the-attribute) |
| 2 | Public fields versus accessors | Divided by whether the type carries an invariant: public fields where every field is an independent fact, private fields and one constructor where a value relates two of them | [`OD-31`](open-decisions.md#od-31--the-models-shape-strings-fields-and-the-attribute) |
| 3 | Newtypes for names | None. A name is the string type of question 1 | [`OD-31`](open-decisions.md#od-31--the-models-shape-strings-fields-and-the-attribute) |
| 4 | Whether the serialisation crate is a public dependency | Yes, deliberately, and it costs nothing because `DIV-032` withdraws the library's compatibility guarantee | [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions) |
| 5 | `#[non_exhaustive]` on the published types | On every published type except the two that are **inputs** a caller must be able to write down | [`OD-31`](open-decisions.md#od-31--the-models-shape-strings-fields-and-the-attribute) |

[`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation)
applies the attribute of question 5 to the error enum for a reason of its own,
load-bearing there for the exit-code derivation; that answer and `OD-31`'s agree
and are not one decision.

## What this document defers, and to what

| Subject | Where |
|---|---|
| The invocation grammar, the command tree, the flags and every document's keys | `/specification`, cited throughout and reproduced nowhere |
| The order an invocation runs in, the deadline machinery, lazy initialisation, the module map | `architecture.md` |
| Every version, every crate and what was rejected, including the engine pin's number | `technology-stack.md`, and [`docs/adr/`](../adr/README.md) |
| Everything persisted, the model's fields, the per-kind field lists, the default classification | [data-model.md](data-model.md) |
| Credentials, redaction, containment as a subject, transport, the injection surfaces | `security.md` |
| Observability, the content a new project ships, the build and the release gates | `operations.md` |
| The measurement points, determinism as a property, and how each is measured | [quality-attributes.md](quality-attributes.md) |
| The tests over every contract above, and the two in-process seams | `verification.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
