---
title: Interfaces
status: draft
last-reviewed: 2026-09-11
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
| 13 | The envelope and one of seventeen payloads | `output/` | stdout, from `model/`, `cli/help.rs`, `project/config.rs`, `cache/` | [The document emitter](#the-document-emitter) |
| 14 | Key order | `model/` and the other payload types | `output/` | [The document emitter](#the-document-emitter) |
| 15 | The buffered writer and the mid-document state | `output/` | `error.rs`, `main.rs` | [The document emitter](#the-document-emitter) |
| 16 | Every ordering | `output/`, with three orders preserved by `mariadb/` | stdout | [Ordering](#ordering-one-default-and-six-exceptions) |
| 17 | The parsed command tree, introspected | `cli/` | `cli/help.rs` | [The help surface](#the-help-surface) |
| 18 | The examples and exit-codes table | `cli/help.rs` | the text help, and `output/` | [The help surface](#the-help-surface) |
| 19 | A template name resolved to a path | `render/` | the engine's loader, and the three `template` subcommands in `cli/` that resolve without the engine | [The template surface](#the-template-surface) |
| 20 | The registered filters, tests and functions | `render/` | the engine, and `cli/help.rs` for publication | [The template surface](#the-template-surface) |
| 21 | The render context, and access to it from a filter or a test | `cli/`, `model/`, `project/` | `render/` | [Context access from a filter or a test](#context-access-from-a-filter-or-a-test) |
| 22 | The typed key space, read | `project/config.rs` | every consumer of a setting | [The configuration reader and the writer](#the-configuration-reader-and-the-writer) |
| 23 | The typed key space, written | `cli/` | `project/config.rs` | [The configuration reader and the writer](#the-configuration-reader-and-the-writer) |
| 24 | The password from a child process | `project/config.rs`, bounded by `deadline.rs` | `mariadb/` | [The `password_command` child](#the-password_command-child) |
| 25 | A phase deadline | `deadline.rs` | `mariadb/`, `project/config.rs`, `render/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 26 | A name matched against a pattern | `cli/` | applied over names from `model/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 27 | A routine named bare or qualified | `cli/` | `mariadb/`, `cache/` | [The shared functions and the phase clock](#the-shared-functions-and-the-phase-clock) |
| 28 | A closed set of typed diagnostic emissions | `mariadb/`, `cache/`, `deadline.rs` | `diagnostics/` | [The diagnostic renderer](#the-diagnostic-renderer) |

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
statement, and a failure of either half refuses the connection (`FR-SRV-008`,
`FR-SRV-009`, `FR-SRV-010`, `FR-SRV-011`).

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
two exceptions to "absent is `null`" and is expressed as the single
`skip_serializing_if` in the crate
([`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).
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

`main.rs` reads the exit status and returns it; it performs no classification of
its own, and the eight-step validation order that decides which code wins when
several conditions are unsatisfied is `architecture.md`'s (`FR-ERR-006`,
`FR-ERR-007`).

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
(`FR-ERR-019`, `FR-ERR-020`, `FR-ERR-021`). The distance is
Damerau-Levenshtein, implemented in the crate
([`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms));
its cost as a budgeted path is
[quality-attributes.md](quality-attributes.md#the-failure-path-is-a-budget).
The **population** is supplied by whichever component owns it — object names by
`mariadb/`, template names by `render/`, entry and key names by
`project/config.rs`, command and flag names by `cli/` — and `diagnostics/` owns
only the selection. One population is narrowed by its own requirement: a failing
segment of a help path suggests over the **children of the node reached**, not
over the tree (`FR-HELP-028`).

**The parser's own rejections are intercepted and re-rendered**, settled in
[`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics): `cli/` hands
`diagnostics/` the token as written and why it was rejected, through the error
value, and no byte the argument parser would render reaches a caller. The four
parsing rules this obliges the `cli/` surface to hold itself are:

| Rule | Forced by |
|---|---|
| No command and no long flag is inferred from a prefix, and no token is looked up on `PATH` | `FR-CLI-004`, `FR-CLI-005`, `FR-CLI-006`, `FR-CLI-002` |
| A single-value flag given twice is `64` **naming both values**, so both occurrences are retained rather than the last winning | `FR-CLI-014` |
| A global flag is accepted at any position and at every depth, while a local flag is rejected by any node that does not declare it | `FR-CLI-024`, `FR-GLOB-002`, `FR-CLI-019` |
| A separate-token flag value beginning with `-` is `64` with the corrected joined form in the hint; `--` terminates arguments; case is never normalised | `FR-CLI-018`, `FR-CLI-017`, `FR-CLI-020` |

Five short forms exist in the whole tool and no other flag declares one
(`FR-GLOB-024`), which makes the short-flag space a property of the parser tree
rather than of each node.

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
`cli/help.rs`, `project/config.rs` and `cache/`; its output is stdout.

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
(`FR-SCH-022`, `BR-SCH-004`).

**The two omissions are different mechanisms because they are different
things.** A key absent from the configuration file is a key never inserted into
the document `output/` receives — `project/config.rs` builds it from the keys
the file carries, and applies no default (`FR-CFG-037`, `FR-CFG-014`). A
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

`project/config.rs` owns one key space and two paths over it — a serde mapping
to read and a format-preserving editor to write. The split, its rationale and
the rejected options are
[`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path); the
file's format, mode, rewrite discipline and key count are
[data-model.md](data-model.md#tplcfg) and are not restated.

**The read path.**

| Obligation | Forced by |
|---|---|
| The ownership and mode checks are a **precondition** of reading, on a canonicalised path, and an explicitly named project folder is not exempt from them | `FR-PROJ-009`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-PROJ-008`, `FR-GLOB-010` |
| The value the reader hands `error.rs` on a malformed file carries the **position** of the fault as well as the key. The strictness that makes this reachable — an unrecognised key anywhere is fatal — is [data-model.md](data-model.md#tplcfg)'s and is not restated | `FR-CONF-034`, `FR-CONF-035`, `FR-ERR-034` |
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
`project/config.rs` a value already typed; the writer preserves comments,
spacing and the relative order of items, which six requirements make a
functional need
([`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path)).

| Obligation | Forced by |
|---|---|
| A value written through the key surface is validated against the enumerated key space **and** the declared type of that key | `FR-CFG-008`, `FR-CFG-009`, `FR-CFG-010`, `FR-CONF-002` |
| An unset accepts a leaf key or a whole block; an absent key or block is `66` | `FR-CFG-011`, `FR-CFG-012` |
| Create and change are distinct verbs: an existing entry refuses creation with a hint, and an update changes the named fields and leaves the rest of the entry untouched | `FR-CFG-015`, `FR-CFG-016`, `FR-CFG-017`, `FR-CFG-020`, `BR-CFG-001` |
| Removing the entry the configured default names also clears that key, silently, leaving the file coherent | `FR-CFG-022`, `FR-CFG-023` |
| Every key of an entry has a flag, each mapping to exactly one key, and one flag's name deliberately differs from the key it writes | `FR-CFG-027`, `FR-CFG-028`, `FR-CFG-029` |
| The password-command flag accepts a single string, stores the array it splits into, accepts no array on the command line and is not repeatable | `FR-CFG-046`, `FR-CONF-023`, `FR-CONF-025` |
| No `cfg` subcommand writes anywhere but the configuration file, and only one of them contacts a server | `FR-CFG-004`, `FR-CFG-005` |

**One key space, two callers, two codes.** An unknown key supplied to the write
path is `64`; the same key found in the file is `78`. The distinction is the
caller's, not the key space's — one is a malformed invocation and the other a
configuration that cannot be trusted — and both are reached through one
validator (`FR-CFG-009`, `FR-CONF-034`, `FR-ERR-001`).

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
[data-model.md](data-model.md#tpl-on-disk-and-its-four-writers).

## The `password_command` child

`project/config.rs` runs the child and hands `mariadb/` a password; `deadline.rs`
bounds it. It is the only child process this corpus admits: no other
requirement creates one, and `FR-SRV-007` forbids an external process for
reading structure.

| Obligation | Forced by |
|---|---|
| The stored form is an argument array, executed **directly, without a shell**; shell metacharacters are passed through as literal arguments | `FR-CONF-023`, `FR-CONF-024`, `FR-CONF-026` |
| A string supplied to a command is split by POSIX quoting rules on the **command** path and stored as the array; no quoting engine exists on the path that reads the file | `FR-CONF-025`, `FR-CFG-046`, [`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms) |
| The stored command is never expanded, so the environment cannot alter what is executed | `FR-CONF-017` |
| The password is the trimmed standard output, read to a cap of 4096 bytes; a child that writes more is terminated and the invocation is `78` | `FR-CONF-027`, `FR-CONF-031` |
| The child's standard error goes to the null device: not inherited, not captured, never quoted | `FR-CONF-032`, `FR-SEC-024` |
| A non-zero exit is `78`, with the `cause` naming the command as stored and the status the child returned | `FR-CONF-033`, `FR-ERR-034` |
| The deadline is enforced by a timer thread that kills the child while the parent reports the expiry; exceeding it is `78` | `FR-CONF-028`, `FR-ERR-027`, [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) |

## The shared functions and the phase clock

Each has a closed grammar written into the corpus, and each is implemented in
the crate with no dependency; the choice and the rejected libraries are
[`OD-20`](open-decisions.md#od-20--edit-distance-and-the-other-small-algorithms).
What belongs here is the surface.

| Function | Input, and where it is applied | Errors it can produce | Fixed by |
|---|---|---|---|
| The pattern matcher | A pattern from `cli/` and a name from `model/`, matched **in memory**, never sent to the server, case-folded over ASCII only; applied **after** coverage | None: a pattern matches or does not. Supplied where it is not declared it is an unknown flag, `64`; supplied to the dump, `64` | `FR-SCH-011` … `FR-SCH-015`, `FR-SCH-021`, `FR-CAT-028`, `BR-SCH-001` |
| The qualified-routine-name parser | One positional argument or one flag value in `cli/`, resolved against `mariadb/` or `cache/` | `66` with a suggestion for a name that does not exist; `64` naming both candidates for a bare name that matches a procedure **and** a function, under every circumstance | `FR-SCH-008`, `FR-SCH-010`, `FR-RND-032`, `FR-CACHE-024` |
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

## The library shape: five questions, open

`DIV-032` fixes that the contract runs through the JSON document and the command
line and **not** through the library, and hands five questions to architecture
rather than answering them.
[overview.md](overview.md#the-library-api-is-not-a-public-surface) carries the
consequence; [data-model.md](data-model.md#the-model-in-memory) records that
four of the five are not decided there.

**All five are open.** They are held by
[`OD-05`](open-decisions.md#od-05--the-module-decomposition), which settles the
module decomposition and the visibility rule and settles none of these:

| # | Question | Status |
|---|---|---|
| 1 | Owned versus borrowed types in the model | **Open** |
| 2 | Public fields versus accessors | **Open** |
| 3 | Newtypes for names | **Open** |
| 4 | Whether the serialisation crate is a public dependency | **Open** |
| 5 | `#[non_exhaustive]` on the published types | **Open** |

None is settled here. Settling one in this document would be writing a decision
the register has not taken, and the register is where a decision and its
rejected options live.

**Recorded overlap, reported and not resolved.** Two settled entries answer a
narrower form of two of the five, and neither generalises:
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)
records that the serialisation crate's traits appear in the library's public
signature because the emitted types derive them, which is question 4 asked of
those types alone; and
[`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation)
applies the non-exhaustive attribute to the error enum, which is question 5
asked of one type and is load-bearing there for the exit-code derivation.
Whether either narrow answer closes the general question is the register owner's
to decide, not this document's.

## What this document defers, and to what

| Subject | Where |
|---|---|
| The invocation grammar, the command tree, the flags and every document's keys | `/specification`, cited throughout and reproduced nowhere |
| The order an invocation runs in, the deadline machinery, lazy initialisation, the module map | `architecture.md` |
| Every version, every crate and what was rejected, including the engine pin's number | `technology-stack.md`, and [`docs/adr/`](../adr/README.md) |
| Everything persisted, the model's fields, the per-kind field lists, the default classification | [data-model.md](data-model.md) |
| Credentials, redaction, containment as a subject, transport, the injection surfaces | `security.md` |
| Observability, the content a new project ships, the build and the release gates | `operations.md` |
| Budgets, determinism as a property, and how each is measured | [quality-attributes.md](quality-attributes.md) |
| The tests over every contract above, and the two in-process seams | `verification.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
