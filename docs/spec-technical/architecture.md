---
title: Architecture
status: draft
last-reviewed: 2026-09-23
related: [README.md, traceability.md, open-decisions.md, overview.md, interfaces.md, data-model.md, quality-attributes.md]
---

# Architecture

## What this document is

The components of `tpl`, what each owns, and the order an invocation runs
through them. Every statement cites the requirement that forces it or the
settled entry that decided it; no requirement text, no command-line syntax and
no JSON syntax is reproduced.

The components named here are the modules of
[`OD-05`](open-decisions.md#od-05--the-module-decomposition), and they are the
same names [interfaces.md](interfaces.md#the-contracts) puts on both sides of
every contract. Versions and crate rationale are `technology-stack.md`; the
contracts themselves are [interfaces.md](interfaces.md); every persisted shape
is [data-model.md](data-model.md).

## The module map

[`OD-05`](open-decisions.md#od-05--the-module-decomposition) settles twelve
modules under `src/`, plus the crate root, and it settles the visibility rule.
Its rationale, the seven placements it argued and the homes it rejected are
recorded there and are not restated.

```
src/
├── main.rs        process setup (ADR-004, ADR-011), then parse, dispatch, map the error to an exit status
├── lib.rs         the crate root and its deliberate re-exports
├── cli/           the parser tree, one module per porcelain command, and the help renderer
├── project/       discovery, the trust checks, and the configuration reader and writer
├── mariadb/       the connection, the catalogue reader, and the privilege cross-checks
├── model/         the model read from the catalogue — the published surface
├── cache/         the read-through cache and its on-disk arrangement
├── render/        the engine, the loader, and the registered template surface
├── output/        the envelope, the JSON emitter, the text layouts, escaping, the writer
├── diagnostics/   the four-line renderer, the suggestion machinery, the verbosity gate
├── deadline.rs    the phase clock, and the threads OD-12 bounds two phases with
├── heap.rs        the one reading of the process's heap count, installed by the binary (ADR-011)
└── error.rs       the error type and the exit-code derivation
```

Three sub-modules are named by
[`OD-05`](open-decisions.md#od-05--the-module-decomposition)'s placement table
and are load-bearing for a boundary in [interfaces.md](interfaces.md#the-contracts):
`cli/help.rs`, `project/config.rs` and `mariadb/privileges.rs`.

**`cli/` delegates; it does not do the work.** Each porcelain command validates
its own arguments, calls the library, and hands the result to `output/`; the
work lives in the module that owns the subject. That is what makes
`NFR-PERF-005` an observation rather than an argument, and it is why a subject
reached by four commands — a table is read by a listing, by the dump, by a
render and by a cache load — exists once.

### Layout conventions

These are the project's own rules, recorded in `CLAUDE.md` (*Convenções de
Código Rust*, *Regras Inegociáveis*), and they bind this map.

| Convention | Consequence for the map above |
|---|---|
| One file-module style: `foo.rs` beside a `foo/` directory, never `mod.rs` | Every directory above has a sibling `.rs` file of the same name |
| Module names in `snake_case`, no obscure abbreviation, no repetition of the parent | `diagnostics/`, not `diag/`; `mariadb::reader`, not `mariadb::mariadb_reader` |
| Visibility minimal: private by default, `pub(crate)` for what crosses a module, `pub` for `model/` and `error.rs` alone; `lib.rs` re-exports those two and no module whole | [`OD-05`](open-decisions.md#od-05--the-module-decomposition) |
| The library holds the logic; the binary parses, dispatches and maps | `main.rs` performs no classification of its own ([interfaces.md](interfaces.md#the-error-type-and-the-exit-code)) |
| `#![forbid(unsafe_code)]` stays at the top of the crate | It is what forces a safe wrapper for the one value `std` does not supply in the trust checks ([`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid)) |

## The invocation pipeline

`FR-ERR-006` fixes eight conditions in one order and `FR-ERR-007` makes that
order decide which code wins when more than one is unsatisfied. The order is
therefore the shape of the process, not a convention of the error module.

| # | Step | Component | Code if it fails |
|---|---|---|---|
| 1 | Parse the invocation | `cli/` | `64` |
| 2 | Discover the project and apply the trust checks | `project/` | `78` |
| 3 | Read and validate the configuration file | `project/config.rs` | `78` |
| 4 | Resolve the database entry | `project/settings.rs` | `78` or `66` |
| 5 | Consult the cache; open a connection if it does not answer | `cache/`, then `mariadb/` | `69`, `77` or `78` |
| 6 | Resolve the named catalogue object | `mariadb/` or `cache/`, or `cli/` where a `--context` document is the source | `66` |
| 7 | Resolve the template | `render/` | `66` |
| 8 | Render | `render/` | `65` |

Two properties of the order are architectural rather than diagnostic. Step 1
runs for every command without exception, so a malformed invocation is rejected
before anything is read from the filesystem (`FR-ERR-006`). And no step may be
reached by a command that does not need it: steps 5 to 8 are entered by need,
not by dispatch, which is [Lazy initialisation](#lazy-initialisation) below.

### The four commands that skip steps 2 and 3

`FR-PROJ-025` exempts four entries from project discovery, and `FR-ERR-006`
turns that exemption into the skipping of steps 2 and 3. `NFR-PERF-005` makes
the exemption observable from outside the process — no `stat` of an ancestor,
no open of the configuration file, no socket — and `NFR-PERF-007` forbids
verifying it by reading the source. Which instrument establishes which clause,
and on which of the four targets, is
[verification.md](verification.md#the-nine-observations-made-outside-the-process)'s;
the design owes the same absence on all four either way.

| Exempt | Why, per `FR-PROJ-025` |
|---|---|
| `tpl init` | It creates the project folder, so it cannot require one to be found |
| The three `help` command forms | The command tree is derived from the binary (`FR-HELP-021`) |
| The help flag at any node of the tree | It is byte-identical to the help command at that node (`FR-HELP-002`) |
| `tpl version` and the version flag | It prints a constant (`FR-HELP-005`) |

**The classification happens in `cli/`, on the parsed invocation, before any
filesystem access.** It cannot be a check inside `project/`, because reaching
`project/` to ask the question is already the `stat` that `NFR-PERF-005`
forbids.

**Recorded reading — how the exempt set is counted.**
[README.md](README.md#architecturemd) fixes this document's scope as "the four
commands that skip two of them", which is the row count of `FR-PROJ-025`;
`NFR-PERF-005` states the same set as three, because it groups the help command
and the help flag together. The two describe one set, and nothing in the built
system turns on the count. The table above follows `FR-PROJ-025`.

## Step 1 in three parts, and the shape of `cli/`

Step 1 runs for every command without exception (`FR-ERR-006`), and it is three
things in one, in this order: whatever the **parser** refused, which
[`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) re-renders; then
the repeated single-value flag of `FR-CLI-014`; then the pair of `FR-CLI-015`.
The order among the three is `cli/`'s own, for the reason
[interfaces.md](interfaces.md#the-diagnostic-renderer) gives, and every one of
them is `64`.

`cli/` is therefore divided by subject and not by command: two of the three
parts are properties of the whole tree rather than of any node under it.

The table below lists every file under `src/cli/` as the directory stands at
commit `243c4d6`.

| Under `cli/` | Owns | Traced to |
|---|---|---|
| `cli.rs` | The tree and the five settings that close it at every node; the one route from the process to the parser; the dispatch, with the one arm a leaf without an implementation still takes | `FR-CLI-002`, `FR-CLI-004`, `FR-CLI-005`, `FR-CLI-006`, [`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own), [`OD-30`](open-decisions.md#od-30--a-parsed-leaf-with-no-implementation) |
| `globals.rs`, `local.rs` | The seven global flags, declared once and accepted at any position; the flags more than one node declares, written once and flattened by each | `FR-GLOB-001`, `FR-GLOB-002`, `FR-CLI-024` |
| `schema.rs`, `template.rs`, `cache.rs`, `cfg.rs` | The nodes, positional arguments and local flags of each group, taken from the module of `/specification` that owns the command | `FR-CLI-010`, `FR-CLI-008` |
| `schema/named.rs`, `schema/pattern.rs`, `schema/text.rs` | Naming one object — the qualified routine form, the lookup shared by the first arm, two `cache` subcommands and `render`, the `66` a name that reaches nothing produces and the `77` a short object owes; the `LIKE` filter evaluated in memory; and the `text` half of the first arm | `FR-SCH-005`, `FR-SCH-008`, `FR-SCH-010`, `FR-CACHE-024`, `FR-RND-003`, `FR-PRIV-003`; `FR-SCH-011` … `FR-SCH-015`, `BR-SCH-001`; `FR-SCH-026`, `FR-SCH-027` |
| `cfg/keys.rs`, `cfg/entries.rs`, `cfg/coherence.rs` | The dotted-key arm; the entry arm; and the one refusal the three writing subcommands share, applied before any of them touches the file | `FR-CFG-007` … `FR-CFG-012`, `FR-ERR-035`; `BR-CFG-001`, `FR-CFG-015` … `FR-CFG-029`; `FR-CFG-048`, `FR-CONF-007` |
| `source.rs` | Where a catalogue read comes from — the cache first, the server on a miss — for the eight `schema` subcommands and for `tpl cache load`, which is the same pipeline with the lookup skipped | `FR-CACHE-006`, `FR-CACHE-007`, `FR-SCH-025`, `FR-ERR-006` |
| `layout.rs` | The `text` cells and the multi-part payload two arms share; it composes no line, and every cell is escaped by `output/` on the way out | `FR-OUT-006`, `FR-OUT-018`, `FR-OUT-019`, `FR-OUT-034` |
| `render.rs`, with `render/context.rs` | The third arm's own step 1, the choice between the two context sources, and the assembly of the five context variables from four sources | `FR-RND-005`, `FR-RND-011` … `FR-RND-018`, `FR-RND-023`, `FR-RND-024`, `FR-RND-026` |
| `rules.rs` | The refusals `tpl` makes itself — the repeated single-value flag, the verbosity pair, `--pretty` without `--format json` — and the diagnostic level the two verbosity flags resolve to | `FR-CLI-014`, `FR-CLI-015`, `FR-OUT-009`, `FR-GLOB-014`, [`OD-17`](open-decisions.md#od-17--observability) |
| `intercept.rs` | What a parser refusal becomes, read as typed API and never as rendered text | [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) |
| `help.rs`, with `help/render.rs` and `help/document.rs` | The typed table, the seven-section renderer, and the JSON command tree | `FR-HELP-006`, `FR-HELP-016`, `FR-HELP-022`, [`OD-05`](open-decisions.md#od-05--the-module-decomposition) |

**Recorded reading — the map's phrase against the code.**
[`OD-05`](open-decisions.md#od-05--the-module-decomposition) writes `cli/` as
one module per porcelain command. As built the **argument declarations** are one
module per command **group** — `schema`, `template`, `cache`, `cfg` — while
`render`, `init`, `help` and `version` declare theirs where they are declared as
nodes: a group's leaves share flags that are written once and flattened by each,
and a top-level leaf sharing none has nothing to put in a module of its own.
Where a leaf's **work** needs a module it has one regardless of that split:
`render/context.rs`, `schema/named.rs`, `schema/pattern.rs`, `schema/text.rs`,
`cfg/keys.rs`, `cfg/entries.rs` and `cfg/coherence.rs`. Two further modules sit
beside the tree and belong to no node: `source.rs`, which every catalogue read
goes through, and `layout.rs`, which two arms share. Nothing of the
decomposition moves — `cli/` delegates and does not do the work, which is what
that entry decided — and the granularity is recorded rather than silently read
as the same thing.

**Parsing yields one of three forms, and a command is only one of them.** The
two flag forms are answered at whatever node they were given at (`FR-GLOB-019`,
`FR-GLOB-020`), and a node that declares a required operand has none to supply
when the caller is asking for the help in order to learn what the operand is. So
what parsing yields is the seven global flags together with a form that is a
help path, the version, or the command the vector names — the last being absent
for a bare `tpl`, which `FR-CLI-007` answers with the top-level help. No value
representing a complete command exists for an invocation whose required operand
is absent, which is what keeps the two flag forms reachable at every node
without weakening a declaration.

**The diagnostic level is fixed between parsing and dispatch**, once, and
nothing afterwards reads the two flags
([`OD-17`](open-decisions.md#od-17--observability), `FR-GLOB-014`,
`FR-GLOB-015`, `FR-CLI-016`). A failure to parse leaves it at the level of a run
that supplied neither flag, which costs nothing: the four labelled lines are
written at every level (`FR-ERR-008`).

**A leaf whose implementation is a later sprint reports `70`**, naming its
command path, and that arrangement is recorded as
[`OD-30`](open-decisions.md#od-30--a-parsed-leaf-with-no-implementation) rather
than left in the code that carries it.

## Inside `project/`

`project/` is the whole of the path from a working directory to a validated
configuration, and to the settings a connection will need. It is divided by the
step of `FR-ERR-006` each part serves, so that the order of the steps is a
property of the code rather than a convention: a project value exists only once
discovery and both trust checks have passed, and reading the file is a method on
it.

| Part | What it owns | Serves |
|---|---|---|
| The locator | The upward walk, the mount-point boundary, the explicitly named folder, canonicalisation | Step 2 (`FR-PROJ-004` … `FR-PROJ-009`) |
| The trust checks | Ownership and mode, judged over metadata already read | Step 2 (`FR-PROJ-010`, `FR-PROJ-011`) |
| The reader | Parse, key space, declared types, coherence, DSN grammar — in that order, over the whole file | Step 3 (`FR-CONF-001`, `FR-CONF-002`, `FR-CONF-006` … `FR-CONF-014`, `FR-CONF-034`, `FR-CONF-035`) |
| The key space | The enumerated keys as a type, with the declared type of each, and what `unset` may be given | `FR-CONF-002`, `FR-CFG-009`, `FR-CFG-010`, `FR-CFG-011` |
| The entry | One `[database.<name>]` block typed, and the predicate that decides `FR-CONF-007` | `FR-CONF-002`, `FR-CONF-006`, `FR-CONF-007` |
| The resolution | Entry selection, `${VAR}` expansion, the child, the four deadlines, the three render bounds, the settings a connection takes | Step 4 (`FR-CONF-004`, `FR-CONF-029`, `FR-CONF-045`, `FR-GLOB-004` … `FR-GLOB-008`) |
| The writer | The format-preserving rewrite of `.tpl/.cfg` | `FR-CFG-034`, `FR-CFG-041`, `FR-CFG-042` |
| The creator | The five artefacts of a new project | `FR-PROJ-012` … `FR-PROJ-024` |

Four divisions inside it are decisions rather than arrangement.

- **Reading and resolving are separate modules, not two methods of one.**
  `FR-CFG-014` forbids the whole-file listing to expand a variable, run the
  child or apply a default, and the way to hold that is for the reader to be
  unable to: the environment lookup and the child live outside it, reached only
  from the resolution. A reader that could resolve would leave the prohibition
  resting on every future call site.
- **Reading and writing are separate modules.** They are different problems over
  one file — reading validates and refuses, writing preserves and must not
  reformat — and
  [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path)
  gives them two different parsers for that reason.
- **The environment is a parameter of expansion, never a call inside it.**
  `FR-SEC-007` treats the environment as untrusted, so the expansion has to be
  exercisable against a hostile value without the process carrying it; and in
  edition 2024 setting a variable is an `unsafe` operation, which
  `#![forbid(unsafe_code)]` denies this crate. One function reads the real
  environment, and it is the only read of it in the crate (`FR-CLI-021`,
  `BR-CONF-003`).
- **A credential is a type, not a string.** It is carried in a value with no
  display implementation and no serialisation, whose one accessor is named so
  that every use of a credential is one search away; the prohibition then
  travels with the value rather than with the caller. What the type denies, and
  why each denial is owed, is `security.md`'s.

**The four deadlines and the three render bounds are resolved here and applied
elsewhere.** `project/` resolves the seven `[core]` values, taking the built-in
default of `FR-CONF-002` for each key the file omits (`FR-CONF-004`,
`FR-CONF-045`); `deadline.rs` owns the construct the deadlines are applied
through, which is [The six phase deadlines](#the-six-phase-deadlines) below, and
`render/` applies the bounds, which is [The render bounds](#the-render-bounds).

## Project discovery and the trust checks

`project/` owns the walk and the checks; nothing else in the crate locates a
project.

| Step | Rule | Forced by |
|---|---|---|
| Start | The current directory, unless the invocation names a `.tpl` folder explicitly — which suppresses the walk and exempts nothing from the checks | `FR-PROJ-004`, `FR-PROJ-008` |
| Walk | Upward, stopping at the first directory that holds a `.tpl` folder | `FR-PROJ-001`, `FR-PROJ-004` |
| Boundary | The mount point of the filesystem the walk starts on, determined without reading any environment variable | `FR-PROJ-005`, `FR-CLI-021`, `BR-CLI-002`, `FR-SEC-013` |
| Failure | No fallback anywhere outside the project: a walk that reaches the boundary without finding one fails | `FR-PROJ-006`, `FR-PROJ-007` |
| Check 1 | Canonicalise the resolved path **before** any check, so a symlinked folder is verified at its target | `FR-PROJ-009`, `FR-SEC-015` |
| Check 2 | The configuration file is owned by the current user | `FR-PROJ-010` |
| Check 3 | The configuration file carries no group and no other access bits | `FR-PROJ-011` |

Three properties of the checks as built are decisions the requirements leave
open, and each is stated rather than inferred.

- **Ownership is judged before the mode.** A file belonging to another user is
  refused whatever its mode says, because the caller's next step is to stop
  using it rather than to change its permissions (`FR-PROJ-010`,
  `FR-PROJ-011`).
- **An absent configuration file passes.** There is nothing to own and nothing
  to grant. `FR-PROJ-001` makes the project the folder rather than the file, and
  `FR-CFG-004` lets the directed write surface create it again, so a project
  whose file was removed by hand is a project with an empty configuration and
  not a project that cannot be used.
- **The judgment is separable from the metadata read.** The ownership half
  cannot be exercised otherwise: a test process cannot give a file to another
  user, and a check only ever called with its own identifier is a check nothing
  has watched fire (`FR-PROJ-010`).

**The boundary is a comparison, not a lookup.** It is decided by comparing a
directory's filesystem device identifier with its parent's, which is why no
home directory is located and no environment variable is read on this path
([`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid),
which also settles how the process's own user identifier is obtained for check
2, and states the accepted cost `FR-PROJ-005` carries).

**Checks 2 and 3 are a precondition of reading, not a validation of what was
read**, so they run before the configuration file is opened
([interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)).
Containment of template paths under the project root is a different boundary,
enforced at one point in `render/` and owned as a subject by `security.md`.

## Configuration resolution

`FR-CONF-029` gives the resolver exactly three inputs and `FR-CONF-030` denies
it a fourth: the flag layer, the configuration file, and the built-in default
declared for the key. Two components supply them and neither owns the rule.

| Layer | Supplied by | Forced by |
|---|---|---|
| The flag | `cli/`, from the parsed invocation | `FR-CONF-029`, `FR-GLOB-001` |
| The file | `project/config.rs`, after the trust checks | `FR-CONF-029` |
| The built-in default | Applied in `project/settings.rs`, in `deadline.rs` for the four deadlines, and in `render/bounds.rs` for the three render bounds | `FR-CONF-029`, `FR-CONF-002`, `FR-CONF-045` |
| **No environment layer** | — | `FR-CONF-030`, `FR-CLI-021`, `BR-CLI-002` |

**A default is applied where it is used, never where the file is read.** The
typed document carries a key the file omits as absent rather than as its
declared default, because `FR-CFG-014` forbids the whole-file listing to apply
one and `FR-CONF-004` requires the deadlines to resolve through one: a document
that carried defaults would make the two disagree. The declared key space
carries each key's declared **type**, which the reader and the writer both
validate against; the declared **value** a key falls back to is held by the
construct that consumes it.

`${VAR}` expansion inside the file is not a fourth layer: it supplies the value
of a key the file already carries (`FR-CONF-030`, `FR-CONF-015`), and it is the
single point at which the environment reaches an invocation at all
([overview.md](overview.md#what-tpl-is-not)). What the reader owes each caller,
and the three distinct read paths over the one file, are
[interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)'s; the
key space and the file's shape are [data-model.md](data-model.md#tplcfg)'s.

**A connection string resolves to the same fields a discrete entry does.** The
resolution parses the URL, expands within each already-delimited field, and
hands on a host, a port, a user, a password and a server-side database name —
whichever way the entry was written — so nothing downstream distinguishes the
two shapes and an expanded value has no delimiter to move within (`FR-CONF-018`,
`FR-SEC-009`). Percent-encoding, the third step of that order, is built:
`Field::encode` in `project/config/dsn.rs` leaves the unreserved set of RFC 3986
and escapes every delimiter of the grammar. **It has no caller on any path the
binary takes**, because nothing composes a URL for the driver: `mariadb/`
builds `MySqlConnectOptions` field by field — host, port, user, password,
database, TLS mode, trust material — so the guarantee `FR-SEC-009` names rests
on the shape of the parsed type rather than on the order of two calls, and the
encoder stands ready for the first consumer that has to put a field back into a
URL (read 2026-09-21).

## The connection lifecycle

One connection at most, per invocation (`NFR-PERF-004`). There is no pool: the
process is ephemeral and issues a handful of queries, which is the root
coordination document's rule (`CLAUDE.md`, *Desempenho e Eficiência*) and also
what `NFR-PERF-004` makes verifiable from the server side (`FR-SRV-014`).

The lifecycle is five ordered stages inside `mariadb/`, and no other module
sends a statement.

| # | Stage | Rule | Forced by |
|---|---|---|---|
| 1 | Open | Opened late — when the reader is about to read, never at dispatch | `NFR-PERF-006`, `NFR-PERF-003`, [`ADR-005`](../adr/adr-005-async-runtime-scope.md) |
| 2 | Set the session read-only | Once, at connection start | `FR-SRV-008` |
| 3 | Read the session state back | Once, immediately after stage 2, reading `@@session.tx_read_only` and nothing else — the spelling `transaction_read_only` does not exist on `10.11`; a failure of either half refuses the connection and reads no catalogue | `FR-SRV-009`, `FR-SRV-010`; `FR-SRV-038`, difference 12 |
| 4 | Probe the product and version | Before any statement other than stages 2 and 3; the series is derived from it and decides the treatment of every known difference | `FR-SRV-002`, `FR-SRV-034`, `FR-SRV-040`, `FR-SRV-022` |
| 5 | Read, then close | Closed as soon as the read ends, and before any render starts: the protocol's quit is sent, the connection released, then the runtime dropped | `CLAUDE.md`, *Desempenho e Eficiência*; `NFR-PERF-004`; `FR-RND-040` |

Stages 2 and 3 are not disableable by any flag, key or environment condition
(`FR-SRV-011`), and they detect rather than prevent: only the closed statement
list of `FR-SRV-006` prevents, which is
[overview.md](overview.md#the-three-properties-that-bound-the-artefact)'s first
property and `security.md`'s subject. The verdicts stages 4 and 5 produce — an
unsupported product, a series below the window, a series above it — are
[interfaces.md](interfaces.md#the-catalogue-reader)'s.

**Stage 5 leaves nothing of the read alive** (`FR-RND-040`). `Session::close` in
`src/mariadb.rs` sends the protocol's quit, releases the connection, and then
drops the runtime, in that order. A failed quit is not a condition — the answer
is already produced — and the socket still closes when the connection is
dropped. The runtime is **dropped**, not shut
down with a timeout: dropping it waits for all spawned work to stop, while a
timeout leaks work and threads that did not stop in time (docs.rs,
`tokio::runtime::Runtime`, tokio 1.53.1, *Shutdown*, consulted 2026-09-23).
Each connection and each runtime is counted, per thread, from creation to drop;
`mariadb::quiescent()` answers whether both counts are zero on the calling
thread. The count is per thread because the runtime of
[`ADR-005`](../adr/adr-005-async-runtime-scope.md) is current-thread and the
counted values cannot leave the thread that built them; a process-wide count
would see another test's connection in the suite. The mechanism and the option
rejected are
[`OD-11`](open-decisions.md#od-11--the-scope-of-the-async-runtime)'s
amendment.

**The order of stages 2 to 4 is `FR-SRV-042`'s.** That requirement fixes the
three connection-start statements in one order and in no other, states it once
for the whole functional corpus, and obliges every passage that depends on it to
cite it rather than repeat it. The table above conforms to it and adds nothing
of its own.

This section recorded a discrepancy until 2026-09-21: the summary in
[README.md](README.md#architecturemd) ordered the three the other way, and
`FR-SRV-002` admitted both readings by requiring the product and version to be
determined before any statement **other than** the read-only pair. `FR-SRV-042`
removes that latitude, so there is no discrepancy left to carry, and the index's
wording was corrected in the same pass as this paragraph.

## The catalogue reader and the query-count invariants

`mariadb/` holds two entry points and a fixed statement repertoire. The reader's
surface and its obligations are
[interfaces.md](interfaces.md#the-catalogue-reader)'s; what belongs here is
where the two entry points sit and what bounds the number of statements.

| Entry point | Shape | Invariant | Forced by |
|---|---|---|---|
| The full read | One statement per object **kind**, each returning every object of that kind | The count does not grow with the number of objects, and is equal over the smallest and the largest reference workload | `NFR-PERF-001` |
| The named-object read | A second path in which the name restricts the statement the server receives | The count does not grow with the number of objects in the database | `NFR-PERF-002` |

Both invariants are properties of form rather than figures, and the decision
each forces is recorded once, in
[quality-attributes.md](quality-attributes.md#the-six-requirements-of-form).
Two consequences are architectural:

- **A per-object query is a defect of the reader, not of a caller's pattern.**
  Coverage is applied by the reader before any filtering, to the object read and
  to the column read alike (`FR-CAT-052`, `FR-CAT-028`), and the pattern is
  matched in memory afterwards and never sent to the server (`FR-SCH-012`,
  `FR-SCH-015`).
- **The count is observable without reading the source.** One diagnostic line
  per catalogue query carries a fixed leading token (`FR-GLOB-017`,
  `NFR-PERF-008`), which is what lets `NFR-PERF-007` be satisfied from outside
  the process. Observability as a subject is `operations.md`, settled in
  [`OD-17`](open-decisions.md#od-17--observability).

The three privilege detections read the shape of the rows the server returned
and are `mariadb/privileges.rs`'s, running on every read that presents the
property they guard (`FR-PRIV-012`); their outcomes cross two different
boundaries, which is
[interfaces.md](interfaces.md#the-three-privilege-detections)'s.

## The cache as a read-through layer

`cache/` sits **in front of** the reader, not beside it. Every `schema`
subcommand reads through it, and so does a render that has no supplied context
(`FR-SCH-025`, `FR-CACHE-009`).

| Path | What happens | Forced by |
|---|---|---|
| Hit | Served from disk; no connection opened, no catalogue query issued | `FR-CACHE-006`, `NFR-PERF-003` |
| Miss | The server is read, the result is written, **then** the answer is produced | `FR-CACHE-007` |
| Unreadable file, or a version the binary does not know | A **silent** miss: read from the server, rewrite, report nothing | `FR-CACHE-033`, `FR-CDOC-004` |
| An object file that is not a regular file, or a named read whose file holds another object | The same silent miss; a link is never read through | `FR-CACHE-033`, `FR-CDOC-008`, with [data-model.md](data-model.md#tplcache) |
| The write fails | A **silent** success: the answer stands, the exit code and stdout are unchanged | `FR-CACHE-036` |

The layer has two independently suppressible halves — consulting and populating
— selected by the two cache flags, and the form in which neither runs is the one
that touches no file at all (`FR-CACHE-013`, `FR-CACHE-014`, `FR-CACHE-015`,
`FR-CACHE-016`, `BR-SCH-003`). A marked object is never written and a collection
holding one is never recorded whole (`FR-CACHE-037`). Which of the two catalogue
values of the `source` discriminant a document carries is decided here, because
this is the component that knows which served the read
([interfaces.md](interfaces.md#the-document-emitter)).

Everything the layer puts on disk — the location, the keying, the encoding, the
filenames, the atomic write, the two independent versions — is
[data-model.md](data-model.md#tplcache)'s and is not repeated.

## The model: one junction, three producers, three consumers

A live read, a cached read and a supplied context document must present the same
objects and the same fields, so `model/` is the single junction of the corpus
(`specification/catalogue-coverage.md` *Overview*; `BR-SCH-001`, `FR-SCH-022`).

| Producers | Consumers |
|---|---|
| `mariadb/`, on a server read | The `text` layouts, in `output/` |
| `cache/`, on a cached read | The JSON emitter, in `output/` |
| `cli/`, from a supplied context document | The render, in `render/` |

Two design consequences follow from the junction being single.

- **An invariant over an assembled model is checked by each of the three
  producers and not on the emitting path**, because `render/` consumes the same
  model without passing through `output/` (`FR-CAT-044`;
  [interfaces.md](interfaces.md#the-catalogue-reader)).
- **The lossy conversion of catalogue bytes to text happens at the `mariadb/`
  boundary and nowhere else**, so all three consumers inherit one substitution
  (`FR-OUT-017`; [interfaces.md](interfaces.md#the-catalogue-reader)).

The model's own shape, what it materialises and what it refuses to materialise,
are [data-model.md](data-model.md#the-model-in-memory)'s. That the library API
carrying it is not a public surface is
[overview.md](overview.md#the-library-api-is-not-a-public-surface)'s.

### Inside `model/`

`model/` is divided by **object kind**, one module per kind, beside four modules
that are not kinds — the two decompositions a column carries, the marking an
incomplete object carries, and the document — and a module root holding what two
of those decompositions share. Each part is named here by what it owns and by
the requirement that fixes it; which fields each kind carries is
[data-model.md](data-model.md#from-a-catalogue-field-list-to-a-model-property-list)'s
and is not repeated.

| Part | What it owns | Serves |
|---|---|---|
| `model.rs` | The one unescaping convention the two quoted forms share — a literal default's value, and a member of an `ENUM` or a `SET` | `FR-CTX-037` with `FR-CTX-039` |
| `database.rs` | The root: three metadata fields, the `server` object, the three collections the three covered kinds stand in | `FR-CAT-001`, `FR-CAT-007`, `FR-CAT-008`, `FR-CTX-035`, `FR-CTX-036` |
| `server.rs` | The three keys, the series derivation, and the two constructors that keep a probe apart from a supplied document | `FR-CTX-031` … `FR-CTX-034`, `BR-CTX-006`, `FR-SRV-040` |
| `table.rs` | The coverage predicate over the table-type string, everything a table carries, and the one constructor a table has | `FR-CAT-001` … `FR-CAT-006`, `FR-CAT-009` … `FR-CAT-015`, `FR-CAT-031`, `FR-CAT-032`, `FR-CAT-043`, `FR-CAT-044`, `FR-SCH-009` |
| `column.rs` | A column's own row, the table it names, and the static attributes read from one catalogue field | `FR-SCH-009`, `FR-CAT-027`, `FR-CAT-035`, `FR-CAT-041`, `FR-CAT-051`, `FR-CTX-019`, `FR-CTX-020`, `FR-CTX-021` |
| `column_type.rs` | The raw string, the eight parts, and the member scan by quote state | `FR-CAT-034`, `FR-CAT-038`, `FR-CTX-014` … `FR-CTX-018`, `FR-CTX-038` … `FR-CTX-041` |
| `column_default.rs` | The three-way discriminant and the eight-row classifier applied in order | `FR-CTX-011` … `FR-CTX-013`, `FR-CTX-037`, `BR-CTX-002` |
| `index.rs` | The fold to one object with an ordered column list, and the name a primary key is told by | `FR-CAT-010`, `FR-CAT-042`, `FR-CAT-043` |
| `foreign_key.rs` | One list of column **pairs**, the four reachable rules, and the incoming direction | `FR-CAT-012`, `FR-CAT-013`, `FR-CAT-033`, `FR-CAT-045` |
| `check_constraint.rs`, `trigger.rs`, `view.rs`, `routine.rs` | One kind each, at the field list its own requirement fixes | `FR-CAT-037`, `FR-CAT-046`; `FR-CAT-050`; `FR-CAT-007`, `FR-CAT-040`, `FR-CAT-047`; `FR-CAT-008`, `FR-CAT-016` … `FR-CAT-018`, `FR-CAT-048`, `FR-CAT-049` |
| `restricted.rs` | The marking: never empty, and present only on an incomplete object | `FR-PRIV-005` … `FR-PRIV-007`, `FR-PRIV-016` |
| `document/` | The document in both directions, over the four shapes that differ from the model | `FR-CTX-001` … `FR-CTX-010`, `FR-CTX-023`, `FR-CTX-033`, `FR-CTX-042`, `NFR-DET-002` |

**Two rows rest on a recorded gap and are marked so here.** No requirement fixes
the catalogue field list for the **table** row or for the **column** row, so the
two rows above name the requirements that fix individual properties and not a
field list. The gap is
[data-model.md](data-model.md#from-a-catalogue-field-list-to-a-model-property-list)'s
and is the functional owner's to close; nothing in this section may be read as
closing it.

**Three requirements are answered across the block rather than by one part**,
and each is named here so that the absence of a row is not read as an omission.
`FR-CAT-036` and `FR-CAT-039` bound what a comment and a default may be taken
for, and each is recorded on the field it reaches — a table's comment, a
column's, and a column's default. `FR-CTX-022` answers the two facts
`FR-CTX-021` refuses to materialise, from the table a column names, on the
ground `BR-CTX-003` gives — one statement of each fact; the tests themselves are
`render/`'s. `BR-CAT-005` is the rule every field list above was derived by, and
it is the first of the four construction choices below.

**Only the object graph is published.** `model/` is one of the two `pub`
modules ([`OD-05`](open-decisions.md#od-05--the-module-decomposition)) and the
document submodule under it is not: the document is contract and the types that
write it are not, which is
[overview.md](overview.md#the-library-api-is-not-a-public-surface)'s reading of
`DIV-032` applied one level down.

**Four things the block does not do**, each because a component named in the
map above owns it.

| Not here | Where | Why |
|---|---|---|
| Any statement to a server, and the lossy conversion of catalogue bytes | `mariadb/` | `FR-SRV-006`, `FR-SRV-007`, `FR-OUT-017`; the model holds only valid text ([interfaces.md](interfaces.md#the-catalogue-reader)) |
| The envelope, the two forms, escaping, the writer | `output/` | `FR-OUT-024`, `FR-OUT-032`; the document is handed over as a value and no byte of it is composed here |
| Opening the file a supplied document is read from | The caller | The read-back direction is given bytes, so the one path that opens a file stays outside the model |
| The three privilege detections | `mariadb/privileges.rs` | Each is a property of a **read** and not of the model (`BR-PRIV-003`, [`OD-05`](open-decisions.md#od-05--the-module-decomposition)); only the marking they produce is carried here |

### Four construction choices inside `model/`

Each is a choice the corpus leaves open, and each is made once so that a
requirement holds by construction rather than by every later caller
remembering it.

| Choice | What it is | Ground |
|---|---|---|
| **Refusal is an absent field, not a filter** | A field excluded on one of the four grounds has no field in any type, so there is nothing for a later reader, emitter or caller to write into | `BR-CAT-005`, `FR-CAT-024`, `FR-CAT-026`, `FR-CAT-029` — whose list is empty, so it excludes nothing today — and `FR-CTX-021`. `FR-CAT-026` binds six consumers, and a filter would be six places to forget |
| **Every collection is ordered where it is built** | One byte-wise comparator, called once per collection at the point that collection of the document is assembled; an excepted collection says so at its own call site | `NFR-DET-002`. The three exceptions the default rule would corrupt are unreachable by it, because the default rule is bounded by a trait their member types do not implement. **Which component applies an ordering** is recorded as a discrepancy in [interfaces.md](interfaces.md#ordering-one-default-and-six-exceptions) and is not settled here |
| **The one-hop cut is a type parameter** | One table shape, instantiated twice: at the first hop a reference resolves to a table, one hop in it resolves to a name. A traversal terminates because a name carries no table | `FR-CTX-006` … `FR-CTX-010`, `BR-CTX-001`, [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md). There is no depth counter and no visited set, so the three shapes `FR-CTX-009` enumerates are one rule, which is the constant depth `BR-CTX-001` chose |
| **Key order is the type's field order** | Serialisation is derived throughout, with two projections onto a private shape; no unordered map appears on the emitting path | `FR-OUT-013`, [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions). The two projections, and the four shapes that are not the model's own types, are [interfaces.md](interfaces.md#the-two-directions-over-the-document)'s |

Three further refusals are structural in the same way and are stated so they
are not read as omissions: a referential action the server discards silently
has no variant (`FR-CAT-033`); a table type the model does not cover has none
either (`FR-CAT-003` … `FR-CAT-006`, `FR-CAT-031`); and a key cannot name a
column its table does not carry, which the one constructor a table has enforces
(`FR-CAT-044`).

### The document, in both directions

One subject, two directions, and they are inverse because they are two
directions over one set of types rather than two routines kept in step
(`FR-SCH-022`, `BR-SCH-004`,
[`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions)).

| Direction | What it does | What it may not do |
|---|---|---|
| Outward | Materialises both embeddings from a model whose references are names, orders every collection, and yields the value `output/` emits | Compose a byte. A foreign key naming a table the model does not carry is `70`, which `FR-CTX-023` makes unreachable on this path |
| Inward | Reads a supplied document back as a model, undoing the embedding by taking the name out of each embedded object | Repair anything, and validate `series` against the supported window or `standing` against `series` (`FR-CTX-033`). A document that fails the contract is `65` (`FR-RND-020`), a foreign key naming a table `tables` does not carry included (`FR-CTX-042`) |

The checks the inward direction makes, and the four it is forbidden or has no
reason to make, are
[interfaces.md](interfaces.md#the-two-directions-over-the-document)'s.

**Both directions have their caller.** `tpl schema dump` emits the document and
`tpl render --context` consumes it; the second arrived with the render work of
2026-09-21, and from that commit the suite feeds one command's output to the
other over a live server. The two directions were written ahead of both
commands, because the block that owns the model owns the document it is written
as.

### What `model/` does not answer of the two files it is built from

Eighteen requirements of the two files are answered elsewhere, and none of them
is an omission. Every other requirement of
`specification/catalogue-coverage.md` and
`specification/context-document.md` is answered by a part of the table above.

| Not answered here | Which | Where it is answered |
|---|---|---|
| The rules that close the two exclusion lists, and the four grounds behind them | `FR-CAT-025`, `FR-CAT-030`, `BR-CAT-001` … `BR-CAT-004` | They govern what may **enter** a list, which is a change to the corpus and not a code path. [data-model.md](data-model.md#the-two-closed-exclusion-lists) |
| Coverage applied before `--pattern`, and applied to the column read as well as the object read | `FR-CAT-028`, `FR-CAT-052` | The reader, `mariadb/` ([interfaces.md](interfaces.md#the-catalogue-reader)) |
| The document's obedience to the four transport rules | `FR-CTX-002` | The envelope and the emitter, `output/` |
| What a cache-served document does not promise | `FR-CTX-024`, `FR-CTX-025`, `BR-CTX-004` | `cache/` ([data-model.md](data-model.md#what-a-cached-document-does-not-promise)) |
| The three context variables that come from no server | `FR-CTX-026` … `FR-CTX-030`, `BR-CTX-005` | [Context assembly](#context-assembly), in `cli/` and `render/` |

Two groups are answered here **by absence** rather than by a part, and are
named so that the absence is not read as a gap: the five features excluded
whole have no type at all (`FR-CAT-019` … `FR-CAT-023`), and the two table
types excluded by kind have no variant (`FR-CAT-004`, `FR-CAT-005`, with
`FR-CAT-003` and `FR-CAT-006` in the same predicate).

## The render component

`render/` builds the engine, owns the one template-name resolution in the crate,
and registers the template surface. The surface as published is
[interfaces.md](interfaces.md#the-template-surface)'s; the engine's identity and
its pin are [`ADR-001`](../adr/adr-001-template-engine-pin.md) and
`technology-stack.md`. What belongs here is what construction fixes.

| Construction step | What it fixes | Forced by |
|---|---|---|
| The engine is built lazily | No engine exists for an invocation that does not render | `NFR-PERF-006`, `CLAUDE.md` *Desempenho e Eficiência* |
| Templates are loaded and compiled at render time from disk | A template changes without the binary being rebuilt | `CLAUDE.md`, *Invariantes de Implementação*; `FR-TMPL-004` |
| The loader closure calls the one resolution function | The three `template` subcommands that resolve without the engine call the same function, so one boundary is enforced once | [`OD-15`](open-decisions.md#od-15--the-template-loader), `FR-TMPL-023` … `FR-TMPL-026` |
| Undefined behaviour is set to the strict variant | Reading a field that does not exist fails the render | [`OD-14`](open-decisions.md#od-14--which-undefined-behaviour-the-engine-is-configured-with), `FR-SEM-012`, `FR-SEM-013` |
| An output formatter of `tpl`'s own is installed | A `null` writes nothing and a boolean writes `true` or `false`, whatever the engine would have written | `FR-SEM-010`, `FR-SEM-011`, `FR-SEM-021` |
| Auto-escaping is set explicitly, to off, and is never left at the engine's default | No property of a template's name can turn escaping on; escaping happens only where a template asks for it | `FR-ENV-026`, `FR-ENV-027`, `FR-ENV-028` |
| Each template is compiled once per process and reused | A loop in a template does not reparse it | `CLAUDE.md`, *Desempenho e Eficiência* |
| The engine is given the resolved render fuel | Every render is held to a count of evaluation steps | `FR-RND-036`, `FR-CONF-045` |
| A render writes into a counting writer of `tpl`'s own, never to stdout | The output limit is counted as bytes are produced, and the text is held until the render returns | `FR-RND-037`, `FR-RND-034`, `FR-CACHE-039` |

**The formatter is load-bearing, and the observation
[`OD-14`](open-decisions.md#od-14--which-undefined-behaviour-the-engine-is-configured-with)
owed is what establishes that.** Made on 2026-09-21, it contradicted the answer
that entry expected: a **defined** `null` does not fail under the strict
variant, and the engine's own rendering of it is the word `None`, which
`FR-SEM-011` forbids by name. Strictness governs *undefined* values and decides
nothing about a defined `null`, so three requirements rest on the formatter row
above and on nothing else — `FR-SEM-010`, `FR-SEM-011` and `FR-SEM-021`, the
engine's stock rendering of a boolean being `True`.
`FR-SEM-012` and `FR-SEM-013` are unaffected: an undefined
fails before any formatter is reached. The two tests that hold this are in
`src/render/engine.rs` and are named in `OD-14`; removing the formatter as a
restatement of what the engine already does would break the three at once.

**Parsing is separable from rendering.** The check command performs syntax
analysis only — no expression evaluated, no filter called, no connection opened
— so the parse step is reachable without the render step (`FR-TMPL-017`,
`BR-TMPL-001`, `FR-SEC-018`).

### Context assembly

`cli/` assembles the context from four sources and hands it to `render/`
(`FR-RND-023`). Three of the five variables are **always** injected and a value
supplied for them in a context document is ignored (`FR-RND-024`), which is what
keeps the context's shape independent of where the model came from.

| Source | Supplied by |
|---|---|
| The context source — a catalogue read or a supplied document | `cache/` or `mariadb/`, or `cli/` |
| The caller's variables | `cli/`, from this invocation |
| The binary's own object | The binary |
| The render time | The clock, read once per invocation |

The render time is the single documented source of non-reproducibility
(`NFR-DET-005`, `FR-CTX-028`, `FR-CTX-029`); the clock, the conversion and the
one grammar used in both directions are
[`OD-25`](open-decisions.md#od-25--the-clock-source-for-now). Supplying a
context document opens no connection and touches no cache (`FR-RND-022`), and
two of the registered tests reach back into the assembled context rather than
answering from their operand, which is
[interfaces.md](interfaces.md#context-access-from-a-filter-or-a-test)'s.

## The six phase deadlines

`FR-SEC-022` requires a deadline on every blocking phase, because a hung process
is the failure the never-interactive invariant exists to prevent (`BR-CLI-003`).
`FR-CONF-005` names six phases and maps each onto one of four keys. The
mechanism for each, and the rejected alternatives, are
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) and are
not restated.

| # | Phase | Budget | Bounded by | Imposed by |
|---|---|---|---|---|
| 1 | DNS resolution | The shared connection budget | The runtime's own timer, around a resolution `tpl` performs itself | `FR-CONF-005`, `FR-ERR-027` |
| 2 | TCP connect | What remains of the same budget | The runtime's own timer, around the driver's connect call | `FR-CONF-005`, `FR-ERR-027` |
| 3 | TLS handshake | What remains of the same budget | The same timer and the same call; the phase is separated in the **report**, from the driver's own discriminant | `FR-CONF-005`, `FR-ERR-027` |
| 4 | Catalogue query | The query budget, per query | The runtime's own timer | `FR-CONF-005`, `FR-ERR-027` |
| 5 | `password_command` | The password budget, until the child has exited **and** its standard output has closed | A reader thread draining the child's pipe and a polling loop in the parent, which kills the child's whole process group and reports the expiry | `FR-CONF-005`, `FR-CONF-028` |
| 6 | Render | The render budget | A watchdog thread that writes the diagnostic and terminates the process; the same thread observes the render memory limit ([The render bounds](#the-render-bounds)) | `FR-CONF-005`, `FR-RND-033`, `FR-RND-039` |

Four rules bind the table.

- **Phases 1 to 3 share one budget**, measured from the start of the first of
  them that runs and consumed in the order they run; they are not given one
  budget each (`FR-CONF-005`). `FR-CONF-004` resolves that budget, and the three
  other budgets, from the declared key or its built-in default.
- **Every phase deadline composes with the overall budget** measured from
  process start, and a phase ends at the first of the two to expire
  (`FR-GLOB-011`, `FR-GLOB-012`).
- **The expiry is reported against the phase in progress**, which is why the
  construct that applies a deadline carries the phase it was created for
  (`FR-CONF-005`, `FR-GLOB-013`, `FR-ERR-027`).
- **The DNS phase is separated because `tpl` performs the resolution itself**,
  which is what lets the diagnostic say that a name did not resolve rather than
  that a host refused a connection (`FR-CONF-005`, `NFR-PERF-018`,
  `FR-ERR-034`). A deadline there bounds `tpl`'s wait and not the resolver's
  work.

**`deadline.rs` owns the construct and three modules use it** — the runtime
inside `mariadb/`, the child process run from `project/password.rs`, and the
render in `render/` — because a budget measured from process start and shared by
three modules belongs to none of them
([`OD-05`](open-decisions.md#od-05--the-module-decomposition),
[interfaces.md](interfaces.md#the-shared-functions-and-the-phase-clock)).

**It is at the crate root and not under `project/`.** `project/` resolves the
four values from `[core]`; it does not own the clock they are applied through,
because two of the three users are not in it and a budget measured from process
start is not a property of the configuration file
([`OD-05`](open-decisions.md#od-05--the-module-decomposition),
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced)).

**A thread here is not the speculative parallelism the project forbids.** It
performs no work of the invocation and makes nothing faster; it is created only
on the paths that need it, so the commands of `NFR-PERF-005` create no thread at
all ([`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced)).

## The render bounds

`FR-RND-038` holds every render to four bounds at once, and the first crossed
ends it with `65` and that bound's `cause`. Three are resolved from `[core]`
(`FR-CONF-045`); the mechanism of the memory count is
[`ADR-011`](../adr/adr-011-render-memory-accounting.md)'s and is not restated.

| Bound | Counted by | Observed | How it ends the render | Forced by |
|---|---|---|---|---|
| Deadline | The phase clock | By the watchdog of `bounded()` in `src/cli/render.rs`, at the deadline | The watchdog reports and terminates the process | `FR-RND-033` |
| Render fuel | The engine, per evaluation step | By the engine | The engine returns its out-of-fuel error; `render/` maps it | `FR-RND-036` |
| Render output limit | The writer the render writes into, per byte | At each write | The write that would pass the limit is refused whole; `render/` maps the failed render | `FR-RND-037` |
| Render memory limit | The binary's counting allocator | By the same watchdog, every **10 ms** and at the deadline | The watchdog reports and terminates the process | `FR-RND-039` |

**The poll interval is fixed here at 10 ms**, as
[`ADR-011`](../adr/adr-011-render-memory-accounting.md) delegates and proposes;
the grounds are that record's. It is the constant `HEAP_POLL` in
`src/cli/render.rs`. The overshoot `FR-RND-039` admits is therefore what a
render allocates within one interval, or the one allocation that crosses the
limit, whichever is larger.

Five properties of the built arrangement follow.

- **The allocator is the binary's and the reading is the library's.**
  `src/main.rs` declares the `#[global_allocator]` and hands the library a
  function reading its count through `tpl::install_heap_counter`; `heap.rs`
  holds that function and nothing else. Where no counter is installed — an
  in-process test — the memory limit observes nothing and the other three
  bounds apply unchanged. The placement and the option rejected are
  [`OD-05`](open-decisions.md#od-05--the-module-decomposition)'s refinement of
  2026-09-23.
- **The count is process-wide**, so heap held before the render — the model,
  the context — counts toward the limit (`FR-CONF-045`'s lower bound;
  [`ADR-011`](../adr/adr-011-render-memory-accounting.md)).
- **The output is held in memory until the render returns**, because
  `FR-RND-034` with `FR-CACHE-039` forbids any byte of an abandoned render
  reaching stdout. It is not held in order to be counted (`FR-RND-037`), and it
  counts toward the memory limit, which is why the output default sits below the
  memory default (`FR-CONF-045`).
- **An abandoned render keeps all four bounds until it returns** (`FR-RND-038`,
  `FR-CACHE-039`). The watchdog does not stand down at a miss. Fuel and the
  output limit crossed by an abandoned render are returned as conditions and
  end the invocation with `65` and no server read; the deadline and the memory
  limit end it from the watchdog. A render that returned within every bound has
  its values dropped, not leaked, before the server read, so the render that
  follows is not charged for them under the memory limit.
- **A render refuses to start while anything of a catalogue read is alive.**
  `bounded()` checks `mariadb::quiescent()` first and returns the `70` of a
  violated invariant otherwise (`FR-RND-040`, `FR-ERR-030`); every caller reaches
  it with the read already closed, so the check is the in-process observation
  `FR-RND-040` names rather than a reachable path.

## Lazy initialisation

Nothing is initialised for a command that cannot use it. The rule is the root
coordination document's — no heavy static initialisation, lazy initialisation by
default (`CLAUDE.md`, *Desempenho e Eficiência*) — and three requirements make
it observable rather than reviewable.

| Nothing is built | Unless | Forced by |
|---|---|---|
| A discovery walk, a configuration read, a socket | The command is not one of the four `FR-PROJ-025` exempts | `NFR-PERF-005`, `FR-PROJ-025` |
| A connection | The command needs catalogue data and the cache did not answer | `NFR-PERF-006`, `NFR-PERF-003`, `FR-CACHE-011`, `FR-RND-022` |
| The template engine | The invocation renders | `NFR-PERF-006` |
| The asynchronous runtime | `mariadb/` is reached | [`ADR-005`](../adr/adr-005-async-runtime-scope.md) |
| A thread of the crate's own | A phase bounded without the runtime is entered | [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) |

Each row is verified by an observation made outside the process and never by
reading the source (`NFR-PERF-007`), which is why laziness is placed at a module
boundary in every row: the absence of a socket, of an open file and of a thread
can be observed from outside, while the absence of a code path cannot. Which of
the four instruments reaches which absence, and on which targets, is
[verification.md](verification.md#the-nine-observations-made-outside-the-process)'s.

## The synchronous process, and the runtime boundary

The process is synchronous. Nothing outside `mariadb/` is asynchronous and no
signature elsewhere in the crate returns a future; the runtime is built lazily
inside `mariadb/`, is a current-thread runtime, and is entered at that one
boundary. The decision, the requirements it serves and the three rejected shapes
are [`ADR-005`](../adr/adr-005-async-runtime-scope.md), registered as
[`OD-11`](open-decisions.md#od-11--the-scope-of-the-async-runtime), and are not
restated.

Two consequences bear on the rest of this document:

- **The module that owns the runtime is the module `NFR-PERF-004` constrains**,
  so at most one connection and exactly one runtime have one owner, and neither
  outlives the read that needed it (`FR-RND-040`).
- **The runtime's timers are available only inside the boundary**, which is what
  divides the six deadlines of
  [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) into
  the four enforced with them and the two enforced with a thread of the crate's
  own. A deadline mechanism that needed a runtime timer outside `mariadb/` would
  contradict [`ADR-005`](../adr/adr-005-async-runtime-scope.md) and would be the
  defect, and it is why the child of `project/password.rs` is bounded by a
  thread and a polling loop and not by a task.

## Two exits that do not return through `main.rs`

`main.rs` reads the exit status from the error value and returns it, performing
no classification of its own
([interfaces.md](interfaces.md#the-error-type-and-the-exit-code)). Two paths
end the process without reaching it, and both are settled:

| Path | What it does | Settled in |
|---|---|---|
| The render deadline, and the render memory limit | The watchdog thread writes the four labelled lines for the `65` of the bound crossed and terminates the process with that status | [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), [`ADR-011`](../adr/adr-011-render-memory-accounting.md) |
| A panic | The panic hook writes the four labelled lines, carrying the location and not the payload, and terminates the process with the internal-error status | [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md) |

Both remain inside the stdout contract. `FR-RND-034` already admits that stdout
carries at most one incomplete result when a render fails — and the render
writes nothing to stdout until it returns — and `FR-ERR-033`
keeps stdout empty on the panic path; stderr is outside the contract in both
cases (`NFR-DET-001`). Both producing conditions of the internal-error code
exist in the distributed binary (`FR-ERR-030`, `FR-ERR-032`), and the release
profile that makes the hook the whole of the reporting path is
[`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md)'s.

## What this document defers, and to what

| Subject | Where |
|---|---|
| The invocation grammar, the command tree, the flags, and every requirement's text | `/specification`, cited throughout and reproduced nowhere |
| Every contract crossing a boundary above, and every signature | [interfaces.md](interfaces.md) |
| Every technology, its version, what was rejected, and the dependency budget | `technology-stack.md`, and [`docs/adr/`](../adr/README.md) |
| The model's fields, the cache on disk, the configuration file, migration | [data-model.md](data-model.md) |
| Trust boundaries as a subject, credentials, containment, transport, the injection surfaces | `security.md` |
| The build, the targets, the release gates, observability, the content a new project ships | `operations.md` |
| The measurement points, determinism as a property, and how each is measured | [quality-attributes.md](quality-attributes.md) |
| The tests over every component above, the harness, and the two in-process seams | `verification.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
