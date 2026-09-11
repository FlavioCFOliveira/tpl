---
title: Security
status: draft
last-reviewed: 2026-09-11
related: [README.md, traceability.md, open-decisions.md, overview.md, architecture.md, interfaces.md, data-model.md]
---

# Security

## What this document is

`specification/security.md` collects the rules that exist for security reasons,
and `BR-SEC-001` makes that file a cross-reference rather than a second source:
each rule is stated once, in the module that enforces it. This document mirrors
it on the technical side under the same discipline. For every rule it names, it
records **how the built system realises it and at which component boundary** —
and nothing else. No rule is restated, no requirement text is reproduced, and
no command-line or JSON syntax appears. The identifier beside an entry is the
authority for it.

Components are the modules of
[`OD-05`](open-decisions.md#od-05--the-module-decomposition), the names
[architecture.md](architecture.md#the-module-map) and
[interfaces.md](interfaces.md#the-contracts) already use. Mechanisms owned
elsewhere are cited, not repeated: the invocation order, the discovery walk and
the deadline machinery are [architecture.md](architecture.md); every contract
and its obligations are [interfaces.md](interfaces.md); the configuration
file's shape, mode and rewrite discipline are
[data-model.md](data-model.md#tplcfg); versions, features and trust-anchor
crates are [technology-stack.md](technology-stack.md).

## The six untrusted inputs

`specification/security.md` *Trust boundaries* enumerates six. Each enters the
process at exactly one component, and that component is where the input stops
being raw.

| # | Untrusted input | First seen by | What that boundary does with it | Forced by |
|---|---|---|---|---|
| 1 | The argument vector | `cli/` | Parsed and classified before any filesystem access; a rejected token leaves the parser as a typed value in an error, never as text the parser rendered | `FR-ERR-006`, `FR-CLI-006`, [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) |
| 2 | `.tpl/.cfg` | `project/` | The resolved path is canonicalised and both trust checks run as a **precondition** of opening the file; the bytes are then parsed strictly in `project/config.rs`, an unrecognised key being fatal | `FR-PROJ-009`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-CONF-034`, `BR-CONF-004` |
| 3 | The environment | `project/config.rs` | Reached only by expanding the fields that admit expansion, in a single pass, at the one point in the crate that reads a variable at all | `FR-CLI-021`, `FR-CLI-023`, `FR-CONF-015`, `FR-CONF-019` |
| 4 | Catalogue values | `mariadb/` | Read as bytes and converted to text with the lossy substitution at that one boundary; no value is interpreted, and none reaches a statement | `FR-OUT-017`, `FR-SRV-006` |
| 5 | A supplied context document | `cli/` | Validated structurally and then handed to `render/`; the standard-input form is the one stdin read the tool admits, and the path opens no connection and touches no cache | `FR-RND-017`, `FR-RND-020`, `FR-SCH-036`, `FR-CTX-033`, `FR-RND-022`, `BR-CLI-003` |
| 6 | Files under the template root | `render/` | Reached only through the one resolution function, which is where containment is enforced; the three `template` subcommands that resolve without the engine call the same function | [`OD-15`](open-decisions.md#od-15--the-template-loader), `FR-TMPL-023`, `FR-TMPL-024`, `FR-TMPL-025`, `FR-TMPL-026` |

Rows 1, 4 and 5 are the populations `FR-ERR-024` and `FR-OUT-019` name as
sources of an interpolated value, which is why escaping is a property of the
emitting and the diagnostic paths and not of these boundaries.

**Recorded gap — the cache is not one of the six.** `.tpl/.cache/` holds
documents `tpl` itself wrote (`FR-PROJ-023`), so the corpus does not place it
among the untrusted inputs, and neither does this document. `FR-CACHE-033` and
`FR-CDOC-004` fix the treatment of a file that cannot be read or carries an
unknown version — a silent miss — and no requirement subjects a cached document
that **does** parse to the structural validation `FR-RND-020` requires of a
supplied one, although the assembled-model invariant of `FR-CAT-044` is checked
on that path as on the other two
([interfaces.md](interfaces.md#the-catalogue-reader)). The gap is reported to
the functional owner rather than filled here.

## Credentials

### The two paths that reach the argument vector

The parser tree declares no flag that carries a password, so `cli/` has no
credential-bearing option to build (`FR-SEC-001`, `FR-CFG-030`). Two surfaces
still accept one a caller writes by hand — the connection-string flag and the
directed key write — and each is a stated affordance rather than an oversight
(`FR-SEC-002`, `FR-CFG-031`, `FR-CFG-032`). What the build owes them is text:
five help surfaces carry the caution, and the substitute is recommended only
where the key it writes admits one (`FR-CFG-033`). Because `BR-CFG-003` settles
that the tool cautions and does not refuse, nothing in `cli/` inspects a value
for secret-like content — a filter that admitted some values and rejected
others would rest the guarantee on a heuristic.

### What each read path over the configuration file may disclose

`project/config.rs` holds three read paths over one file, and they are three
code paths rather than one with a flag; their obligations are
[interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)'s. What
belongs here is the disclosure property of each.

| Path | Disclosure property | Forced by |
|---|---|---|
| The directed key read | Unredacted and unexpanded, by decision, because the caller asked for one key by name | `FR-SEC-004`, `BR-CFG-002` |
| The whole-file listing, and one entry shown | Redacted, with a variable reference emitted verbatim, so what the variable holds is never disclosed by either command | `FR-SEC-003`, `FR-CFG-021` |
| The resolution a connection uses | Never reaches stdout at all; its only consumer is `mariadb/` | `FR-CONF-004`, `FR-CONF-029`, `FR-GLOB-018` |

The third row is what makes *the resolved DSN* a category a diagnostic cannot
write: the value exists only between `project/config.rs` and `mariadb/`, and no
component between them emits.

### The six categories that reach no diagnostic stream

`FR-GLOB-018` names them and `FR-SEC-005` restates the prohibition as
cross-cutting. The design does not enforce them by rule: it denies each a place
to be written from. Two decisions carry most of that —
[`OD-17`](open-decisions.md#od-17--observability), which installs no subscriber
and admits only a closed set of typed emission functions, and
[`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation),
which keeps the forbidden values out of the error type.

| # | Category | Why it has no home | Forbidden by |
|---|---|---|---|
| 1 | The argument vector | No emission function takes a formatted message, so the vector cannot be dumped; what `cli/` hands `diagnostics/` is a typed rejection, not the invocation | `FR-GLOB-018`, `FR-SEC-005`, with [`OD-17`](open-decisions.md#od-17--observability) and [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) |
| 2 | The resolved DSN | It is produced by the third read path above and consumed by `mariadb/`; no variant of the error type carries it | `FR-GLOB-018`, `FR-SEC-005`, `BR-ERR-003`, with [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) |
| 3 | The `password_command` | Nothing emits it as diagnostic content; the stored array is named in the `cause` of a child failure, which `FR-CONF-033` obliges and admits because `FR-CONF-017` keeps that array unexpanded | `FR-GLOB-018`, `FR-SEC-005` |
| 4 | The child's standard error | It is directed to the null device, so the bytes never exist in the process to be written | `FR-GLOB-018`, `FR-SEC-005`, `FR-CONF-032`, `FR-SEC-024` |
| 5 | The raw driver error | It is classified at the `mariadb/` boundary and the original value is dropped; with no subscriber installed, what the driver itself emits is discarded before it exists | `FR-GLOB-018`, `FR-SEC-005`, with [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) and [`OD-17`](open-decisions.md#od-17--observability) |
| 6 | The contents of `.tpl/.cfg` | The reader hands `error.rs` the key and the position of a fault, never the text of the file | `FR-GLOB-018`, `FR-SEC-005`, `FR-CONF-034`, `FR-CONF-035` |

Two prohibitions bound the whole set rather than one category: no credential
appears in an error message at any verbosity (`FR-SEC-006`, `FR-ERR-013`), and
`FR-SEC-024` is the case the corpus itself holds up as the model, where the
design enforces the prohibition instead of the code having to observe it.

**Recorded reading — the argument vector and the rejected token.**
`FR-GLOB-018` and `FR-SEC-005` forbid writing the argument vector to a
diagnostic stream at any verbosity. `FR-ERR-034` obliges the `cause` of a usage
error to name the token that was rejected as written, which is a value taken
from that vector, and `FR-ERR-024` names the argument vector among the
populations a message interpolates and escapes. Both readings are recorded. The
built system takes the reading that satisfies every requirement literally: the
forbidden category is the **vector**, the invocation reproduced as a whole,
while a single token a diagnostic is obliged to name is required content, bound
by `FR-SEC-006` and `FR-ERR-013`, which bar a credential wherever it appears.
The corpus resolves the sibling case explicitly — `FR-CONF-033` admits the
stored command in a `cause` and gives the ground — and leaves this one implicit.
The wording is the functional owner's to settle; nothing in the built system
differs between the two readings.

### The sentinel property

`BR-SEC-003` is the one rule `specification/security.md` owns outright, and its
subject is the surface entire, where every other credential rule binds a single
path. Three consequences for the design, none of them a restatement of it:

- **It is not satisfied by the per-path prohibitions.** Each of those is
  testable alone and proves only its own path; the property they exist to
  produce is the conjunction, and it is the form of the check that survives a
  path being added (`BR-SEC-003`).
- **What makes it hold by construction is the denial of a home**, category by
  category, in the table above. A design in which a formatted message could be
  emitted would leave the property true only until someone wrote one.
- **The register of mandated tests, the harness and the container arrangement
  are `verification.md`'s.**

**Recorded discrepancy — the sentinel property and the directed key read.**
`BR-SEC-003` asserts that the sentinel appears in no byte written by any
command of the tree. `FR-SEC-004` and `BR-CFG-002` make the directed key read a
deliberate exception to redaction, so that command, asked for the key that
holds a literal sentinel, prints it by design. Both readings are recorded. They
are compatible only if the entry the test configures holds the sentinel in a
form that command does not disclose, or if the arguments the test chooses do
not name that key; the corpus states neither, and the choice belongs to the
harness, which is `verification.md`'s. The wording is the functional owner's to
settle.

## The environment

The environment is an input of one component and one purpose: `${VAR}`
expansion inside the configuration file, which is the single circumstance in
which `tpl` reads a variable (`FR-CLI-021`, `FR-CLI-023`, `BR-CLI-002`). It is
not a configuration layer (`FR-CONF-030`;
[architecture.md](architecture.md#configuration-resolution)).

| Property | Where it is realised | Forced by |
|---|---|---|
| Expansion reaches the six fields that admit it and no others | `project/config.rs`, from the declared key space | `FR-SEC-007`, `FR-CONF-015` |
| Two fields are excluded by the same table, so the exclusion is structural: neither the encryption mode nor the child command is expandable, and no variable can therefore weaken transport or choose the program that runs | `project/config.rs` | `FR-SEC-008`, `FR-CONF-016`, `FR-CONF-017` |
| Inside a connection string, substitution happens after the URL has been broken into fields, and what it substitutes is encoded for the field it lands in | `project/config.rs`, before any connection exists | `FR-SEC-009`, `FR-CONF-018` |
| An undefined variable ends the invocation; it is never an empty substitution | `project/config.rs` → `error.rs` | `FR-SEC-010`, `FR-CONF-022`, `BR-CONF-002` |

The third row is an ordering, not a sanitisation, and the order is what does
the work: the field boundaries exist before any substitution occurs, so a
substituted value is confined to the field it was written into, and encoding it
afterwards keeps that value from re-entering the grammar it sits in. One pass
only, so nothing substituted is substituted again (`FR-CONF-019`).

## The child process

`password_command` is the only child process the corpus admits, and reading
structure through an external process is forbidden outright (`FR-SRV-007`). Its
contract is [interfaces.md](interfaces.md#the-password_command-child)'s; the
security properties it carries are these.

| Property | How the build holds it | Forced by |
|---|---|---|
| No shell is involved | `project/config.rs` executes the stored argument array directly, so metacharacters are arguments and a `.cfg` inherited or cloned cannot choose code to run | `FR-SEC-011`, `FR-CONF-024`, `FR-CONF-026` |
| The read is bounded | A fixed cap on what is read from the child; a child that writes past it is killed and the invocation fails, refusing rather than truncating | `FR-SEC-024`, `FR-CONF-031` |
| The child is silent | Its standard error goes to the null device — not inherited, not captured | `FR-SEC-024`, `FR-CONF-032` |
| A failure is diagnosable without the child's own words | The exit status and the stored command are what the `cause` carries | `FR-CONF-033` |
| It cannot hang the caller | A deadline, enforced by the timer thread of [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) | `FR-SEC-012`, `FR-CONF-028` |

The stored form is decided on the write path and never on the read path
(`FR-CONF-023`, `FR-CONF-025`), so no quoting engine exists where the untrusted
file is read — the file supplies an array, and an array is executed.

## Project discovery: the boundary and the trust checks

The walk, the boundary and the three checks are
[architecture.md](architecture.md#project-discovery-and-the-trust-checks)'s
table. What belongs here is which threat each closes and what the build
therefore contains.

| Property | What the built system does | Forced by |
|---|---|---|
| The walk has a boundary | It stops at the mount point, decided by comparing a directory's filesystem device identifier with its parent's — a comparison, not a lookup, so no environment variable is read and no home directory is located | `FR-SEC-013`, `FR-PROJ-005`, [`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid) |
| A path is checked at its target | The resolved path is canonicalised **before** any check | `FR-SEC-015`, `FR-PROJ-009` |
| The file is the caller's | The process's own user identifier is obtained through a safe call and compared with the file's | `FR-SEC-014`, `FR-PROJ-010`, [`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid) |
| Nobody else can write it | The mode check reads the same metadata as the ownership check | `FR-SEC-014`, `FR-PROJ-011` |
| Naming the folder explicitly exempts nothing | The explicit path suppresses the walk and enters the same check sequence | `FR-SEC-016`, `FR-PROJ-008` |

Two consequences are worth stating because a reader would otherwise assume more
or less than the design gives. The boundary is **not** what refuses a project
left in a directory anyone may write to — the two checks on the configuration
file are, and `FR-SEC-013` records the correction. And the checks are a
precondition of reading rather than a validation of what was read, which is why
they sit before the open in `project/` and not inside the parser
([interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)).

## Template containment

`BR-ENV-004` classes a template as semi-trusted and gives the ground.
Containment is enforced at **one** point: the single resolution function in
`render/`, settled in [`OD-15`](open-decisions.md#od-15--the-template-loader),
which the engine's loader closure and the three `template` subcommands that
resolve without the engine all call. The engine's own path helper is not wrapped, for
the reasons that entry records.

| Property | Where | Forced by |
|---|---|---|
| The template root is the boundary of every lookup | The one resolution function | `FR-TMPL-023` |
| A symbolic link inside the root is refused, read from the entry's own metadata rather than by following it | The same function | `FR-SEC-017`, `FR-TMPL-024`, [`OD-15`](open-decisions.md#od-15--the-template-loader) |
| The path that was checked is the path that is opened: one canonical form is compared with the root's, and nothing else is opened afterwards | The same function | `FR-SEC-017`, `FR-TMPL-025`, `FR-TMPL-026` |
| Syntax analysis is reachable without evaluation: no expression evaluated, no function called, no connection opened | `render/`, on the check path | `FR-SEC-018`, `FR-TMPL-017`, `BR-TMPL-001` |

### The four capability prohibitions

The set of global functions is closed (`FR-ENV-020`), and that is what makes the
four prohibitions a property of the registration rather than a review item:
`render/` registers a fixed surface at construction and registers nothing else,
so there is no name for a template to reach.

| # | No registered function | Forbidden by |
|---|---|---|
| 1 | reads the environment | `FR-ENV-022` |
| 2 | reads a file | `FR-ENV-023` |
| 3 | performs network access | `FR-ENV-024` |
| 4 | reads a clock | `FR-ENV-025` |

Why the four exist, and why the corpus writes them as prohibitions instead of
leaving them as an absence, is `BR-ENV-004`'s and is not restated. The
registered surface itself, and the guarantee attached to each of its three
groups, are [interfaces.md](interfaces.md#the-template-surface)'s; a template's
only source of time is the context variable the binary injects (`FR-ENV-025`).

## The injection surfaces

Seven surfaces exist where an untrusted value could re-enter a grammar. Six are
closed by the design; the seventh is stated rather than closed.

| Surface | Closed by | Forced by |
|---|---|---|
| A variable moving a connection's host, port or database | The parse-expand-encode order of *The environment* above | `FR-SEC-009`, `FR-CONF-018` |
| A shell interpreting a value from an untrusted `.cfg` | Direct execution from an argument array; no shell exists on the path | `FR-SEC-011`, `FR-CONF-024` |
| A catalogue name formatted into a runnable suggestion | A `hint` is built from literals and from names inside the admitted character set; a candidate outside it is presented in no form, and the generic hint stands alone | `FR-SEC-019`, `FR-ERR-022`, `FR-ERR-023` |
| A control character rewriting a terminal, or forging a line of the four-line diagnostic format | C0 escaping applied in `output/` and in `diagnostics/`, by the two rules that differ between them, with the two emissions that are exempt and byte-for-byte | `FR-SEC-020`, `FR-OUT-018`, `FR-OUT-019`, `FR-ERR-024` |
| A caller's pattern reaching the server | The pattern is matched in memory against names already read and is never sent | `FR-SCH-013` |
| A path chosen from catalogue data | There is no file-writing surface to choose a path for | `FR-RND-028`, `BR-RND-003` |

**The surface that is stated rather than closed.** Identifiers reach the model
as the catalogue holds them, hostile ones included; a template that emits one
must quote it, and the registered filter is what it quotes with
(`specification/catalogue-coverage.md` `FR-CAT-042`, `FR-ENV-035`,
`FR-ENV-045`). `tpl` generates text; what that text is fed to is outside its
boundary, and no requirement claims otherwise
([overview.md](overview.md#what-tpl-is-not)).

**Recorded gap — the one caller-supplied value that reaches the server.**
`NFR-PERF-002` establishes a named-object read in which the name restricts the
statement the server receives, and `FR-CFG-044` restricts the privilege probe to
the configured database; the qualified-routine-name parser resolves its operand
against `mariadb/` ([interfaces.md](interfaces.md#the-shared-functions-and-the-phase-clock)).
The closed statement list of `FR-SRV-006` fixes **which** statements exist and
`FR-SCH-013` keeps the pattern out of them, but no requirement and no settled
entry of [open-decisions.md](open-decisions.md) fixes **how** a caller-supplied
name is conveyed into a statement — as a value bound by the driver, or composed
into the statement text. Settling it here would be taking a decision the
register has not taken, which this folder does not admit; it is reported to the
register owner.

## Transport

`FR-SEC-021` fixes which mode a connection takes when nobody chose one, and
`BR-CONF-001` fixes where the authority over encryption sits. The first and the
fourth rows below are the two limits `FR-SEC-021` carries as observed rather
than reasoned, and both bear on the build.

| Property | How the build holds it | Forced by |
|---|---|---|
| `mariadb/` sets the mode on **every** connection it opens, whichever mode it is | The driver's own default is never the operative one. What an inherited default would amount to is [`ADR-002`](../adr/adr-002-tls-mode-mapping.md)'s consequence and is not restated | `FR-CONF-037`, `FR-SEC-021` |
| The mode-to-driver mapping lives in exactly one place | Recorded in [`ADR-002`](../adr/adr-002-tls-mode-mapping.md), which `FR-CONF-038` requires and rule R3 of [`docs/adr/`](../adr/README.md) obliges; this document cites it and carries none of it | `FR-CONF-038`, `FR-CONF-013` |
| The mode set is an admission test over the dependency table | Carried by [technology-stack.md](technology-stack.md#the-database-driver-and-the-five-tls-modes), where the criterion belongs to the stack rather than to this subject | `FR-CONF-036` |
| Pinned trust material joins the root store rather than replacing it | A property of how the store is assembled, not a choice available here: pinning therefore enlarges what passes verification instead of restricting it, and no output or document may describe it as exclusive | `FR-CONF-039`, `FR-CONF-014`, [`OD-16`](open-decisions.md#od-16--the-tls-backend-and-the-root-store) |

The trust anchors themselves, the rejection of the platform store and the
consequence for reproducibility across hosts are
[`ADR-002`](../adr/adr-002-tls-mode-mapping.md)'s and are not restated. The
third limit of [overview.md](overview.md#the-three-limits-the-system-states-rather-than-overcomes)
is the fourth row above, stated where a reader would otherwise assume more.

## The read-only promise in two parts

`BR-SEC-002` divides the promise in two and settles which half is which. The
artefact-level property is
[overview.md](overview.md#the-three-properties-that-bound-the-artefact)'s; what
belongs here is which half does what, and where each sits.

| Part | What it is in the build | What it does | Forced by |
|---|---|---|---|
| The closed statement list | Every statement `mariadb/` can send is written into the crate; no component composes one elsewhere, and no external process is run to read structure | **Prevents.** Nothing outside the list leaves the process, whatever grants the session carries | `FR-SRV-006`, `FR-SRV-007`, `BR-SRV-001` |
| The read-only session | Stages 2 and 3 of the connection lifecycle: applied once at connection start, confirmed immediately by reading `@@session.tx_read_only` and no other spelling of it, and a failure of either half refuses the connection before any catalogue statement | **Detects.** Defence in depth over the first part, which is what `BR-SRV-002` states it is and all it states it is | `FR-SRV-008`, `FR-SRV-009`, `FR-SRV-010`, `BR-SRV-002` |

Neither half is disableable by a flag, a configuration key or an environment
condition (`FR-SRV-011`), which is why no such switch exists to be reviewed. The
ordering of the two stages relative to the version probe, and the verdicts they
produce, are
[architecture.md](architecture.md#the-connection-lifecycle)'s. Describing the
session setting as prevention overstates it, and no passage of this folder may
(`BR-SRV-002`).

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every rule's normative statement | `/specification`, cited by identifier throughout and reproduced nowhere |
| The discovery walk, the connection lifecycle, the deadline machinery, the module map | [architecture.md](architecture.md) |
| Every contract crossing a boundary named above, and its obligations | [interfaces.md](interfaces.md) |
| The configuration file's format, key space, mode and rewrite discipline, and everything the cache puts on disk | [data-model.md](data-model.md) |
| The TLS crates, their features, and the one call `std` does not supply | [technology-stack.md](technology-stack.md) |
| The sentinel test, the harness, the containers, and every other mandated test | `verification.md` |
| Observability, the release gates, and what a new project ships | `operations.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
