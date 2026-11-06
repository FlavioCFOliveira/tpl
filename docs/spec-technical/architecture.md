---
title: Architecture
status: draft
last-reviewed: 2026-11-06
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

[`OD-05`](open-decisions.md#od-05--the-module-decomposition) settles fifteen
modules under `src/`, plus the crate root, and it settles the visibility rule.
Its rationale, the seven placements it argued and the homes it rejected are
recorded there and are not restated. `sqlite/` and `provider.rs` are the two
added for the second provider, by
[`OD-35`](open-decisions.md#od-35--the-provider-dispatch-and-the-sqlite-reader);
`mysql/` is the sixteenth, added for the third, in the place
[`ADR-005`](../adr/adr-005-async-runtime-scope.md) gives it.

```
src/
├── main.rs        process setup (ADR-004, ADR-011), then parse, dispatch, map the error to an exit status
├── lib.rs         the crate root and its deliberate re-exports
├── cli/           the parser tree, one module per porcelain command, and the help renderer
├── project/       discovery, the trust checks, and the configuration reader and writer
├── provider.rs    the one match over an entry's provider, and the quiescence of the three readers (OD-35)
├── mariadb/       for a mariadb entry: the connection, the catalogue reader, and the privilege cross-checks; the connection and the fault classification serve a mysql entry too
├── mysql/         for a mysql entry: the session checks, the window, and the catalogue reader (ADR-005, ADR-014)
├── sqlite/        for a sqlite entry: the hardened open, the catalogue reader, and the fold (OD-35)
├── model/         the model read from the catalogue — the published surface
├── cache/         the read-through cache and its on-disk arrangement
├── render/        the engine, the loader, and the registered template surface
├── output/        the envelope, the JSON emitter, the text layouts, escaping, the writer
├── diagnostics/   the four-line renderer, the suggestion machinery, the verbosity gate
├── deadline.rs    the phase clock, and the threads OD-12 bounds two phases with
├── heap.rs        the one reading of the process's heap count, installed by the binary (ADR-011)
├── at.rs          directory-relative file operations below the canonical .tpl (OD-24)
└── error.rs       the error type and the exit-code derivation
```

Three sub-modules are named by
[`OD-05`](open-decisions.md#od-05--the-module-decomposition)'s placement table
and are load-bearing for a boundary in [interfaces.md](interfaces.md#the-contracts):
`cli/help.rs`, `project/config.rs` and `mariadb/catalogue/completeness.rs`.

**`cli/` delegates; it does not do the work.** Each porcelain command validates
its own arguments, calls the library, and hands the result to `output/`; the
work lives in the module that owns the subject. That is what makes
`NFR-PERF-005` an observation rather than an argument, and it is why a subject
reached by four commands — a table is read by a listing, by the dump, by a
render and by a cache load — exists once.

### Layout conventions

These are the project's own rules, recorded in `CLAUDE.md` (*Rust Code
Conventions*, *Non-Negotiable Rules*), and they bind this map.

| Convention | Consequence for the map above |
|---|---|
| One file-module style: `foo.rs` beside a `foo/` directory, never `mod.rs` | Every directory above has a sibling `.rs` file of the same name |
| Module names in `snake_case`, no obscure abbreviation, no repetition of the parent | `diagnostics/`, not `diag/`; `mariadb::reader`, not `mariadb::mariadb_reader` |
| Visibility minimal: private by default, `pub(crate)` for what crosses a module, `pub` for `model/`, `error.rs` and the entry functions of [interfaces.md](interfaces.md#the-library-entry-points); no module is re-exported whole | [`OD-05`](open-decisions.md#od-05--the-module-decomposition) |
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
| 5 | Consult the cache; open a connection if it does not answer | `cache/`, then `provider.rs`, which reaches `mariadb/`, `mysql/` or `sqlite/` | `69`, `77` or `78` for a server entry; for a `sqlite` entry `69`, `70`, `74` or `78` (`FR-ERR-006`, note of the sixty-fifth edition) |
| 6 | Resolve the named catalogue object | The reader or `cache/`, or `cli/` where a `--context` document is the source | `66` |
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
| `cli.rs` | The tree and the five settings that close it at every node; the one route from the process to the parser; the dispatch, in which every leaf of the tree has its implementation | `FR-CLI-002`, `FR-CLI-004`, `FR-CLI-005`, `FR-CLI-006`, [`OD-07`](open-decisions.md#od-07--help-the-parsers-renderer-or-tpls-own), [`OD-30`](open-decisions.md#od-30--a-parsed-leaf-with-no-implementation) |
| `globals.rs`, `local.rs` | The seven global flags, declared once and accepted at any position; the flags more than one node declares, written once and flattened by each | `FR-GLOB-001`, `FR-GLOB-002`, `FR-CLI-024` |
| `schema.rs`, `template.rs`, `cache.rs`, `cfg.rs` | The nodes, positional arguments and local flags of each group, taken from the module of `/specification` that owns the command | `FR-CLI-010`, `FR-CLI-008` |
| `schema/named.rs`, `schema/pattern.rs`, `schema/text.rs` | Naming one object — the qualified routine form, the lookup shared by the first arm, two `cache` subcommands and `render`, the `66` a name that reaches nothing produces and the `77` a short object owes; the `LIKE` filter evaluated in memory; and the `text` half of the first arm | `FR-SCH-005`, `FR-SCH-008`, `FR-SCH-010`, `FR-CACHE-024`, `FR-RND-003`, `FR-PRIV-003`; `FR-SCH-011` … `FR-SCH-015`, `BR-SCH-001`; `FR-SCH-026`, `FR-SCH-027` |
| `cfg/keys.rs`, `cfg/entries.rs`, `cfg/coherence.rs` | The dotted-key arm; the entry arm; and the one refusal the three writing subcommands share, applied before any of them touches the file | `FR-CFG-007` … `FR-CFG-012`, `FR-ERR-035`; `BR-CFG-001`, `FR-CFG-015` … `FR-CFG-029`; `FR-CFG-048`, `FR-CONF-007` |
| `source.rs` | Where a catalogue read comes from — the cache first, the database on a miss — for the eight `schema` subcommands and for `tpl cache load`, which is the same pipeline with the lookup skipped. The read on a miss is to go through `provider.rs`, which reaches the server or the database file ([The provider dispatch](#the-provider-dispatch)) | `FR-CACHE-006`, `FR-CACHE-007`, `FR-SCH-025`, `FR-ERR-006`, `FR-CONF-051` |
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

**Every leaf of the tree has an implementation.** Until 2026-09-22 a leaf whose
implementation was a later sprint reported `70`, naming its command path; that
arrangement is recorded, and its discharge, in
[`OD-30`](open-decisions.md#od-30--a-parsed-leaf-with-no-implementation).

## Inside `project/`

`project/` is the whole of the path from a working directory to a validated
configuration, and to the settings a connection will need. It is divided by the
step of `FR-ERR-006` each part serves, so that the order of the steps is a
property of the code rather than a convention: a project value exists only once
discovery and the trust checks have passed, and parsing the text they read is a
method on it.

| Part | What it owns | Serves |
|---|---|---|
| The locator | The upward walk, the mount-point boundary, the explicitly named folder, canonicalisation | Step 2 (`FR-PROJ-004` … `FR-PROJ-009`) |
| The trust checks | The file's type, ownership and mode, judged over the metadata of the descriptor the text is then read through; the folder's ownership where the file is absent | Step 2 (`FR-PROJ-030`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-PROJ-028`) |
| The reader | Parse, key space, declared types, coherence, DSN grammar — in that order, over the whole file; the two values of `provider`, and the refusal of `${` in `path` | Step 3 (`FR-CONF-001`, `FR-CONF-002`, `FR-CONF-006` … `FR-CONF-014`, `FR-CONF-034`, `FR-CONF-035`, `FR-CONF-051`, `FR-CONF-054`) |
| The key space | The enumerated keys as a type, with the declared type of each, and what `unset` may be given | `FR-CONF-002`, `FR-CFG-009`, `FR-CFG-010`, `FR-CFG-011` |
| The entry | One `[database.<name>]` block typed, and the predicate that decides `FR-CONF-007` | `FR-CONF-002`, `FR-CONF-006`, `FR-CONF-007` |
| The resolution | In two halves. At step 5, `prepare`: entry selection, the keys the provider refuses and the `path` it requires, a value holding a NUL character, the four deadlines and the three render bounds — the entry held as written, nothing expanded and no child run. On the connection branch of step 6 alone, `Prepared::connect`: `${VAR}` expansion and the child, into the settings a connection takes — for a `sqlite` entry, the resolved path alone | Steps 5 and 6 (`FR-CONF-004`, `FR-CONF-029`, `FR-CONF-045`, `FR-CONF-056`, `FR-CONF-057`, `FR-GLOB-004` … `FR-GLOB-008`, `FR-CONF-052` … `FR-CONF-054`) |
| The writer | The format-preserving rewrite of `.tpl/.cfg`, every step of it relative to the `.tpl` folder the trust checks opened and held, and made durable | `FR-CFG-034`, `FR-CFG-041`, `FR-CFG-042` |
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
through, which is [The phase deadlines](#the-phase-deadlines) below, and
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
| Check 2 | The configuration file, where it exists, is a regular file, established on the descriptor it is then read through | `FR-PROJ-030`, `FR-SEC-027` |
| Check 3 | The configuration file is owned by the current user | `FR-PROJ-010` |
| Check 4 | The configuration file carries no group and no other access bits | `FR-PROJ-011` |
| Check 5 | Where no configuration file exists, the `.tpl` folder is owned by the current user | `FR-PROJ-028` |

Three properties of the checks as built are decisions the requirements leave
open, and each is stated rather than inferred.

- **Ownership is judged before the mode.** A file belonging to another user is
  refused whatever its mode says, because the caller's next step is to stop
  using it rather than to change its permissions (`FR-PROJ-010`,
  `FR-PROJ-011`).
- **An absent configuration file is not refused for its absence.** There is
  no file to own and nothing to grant, so the folder's ownership is checked
  instead (`FR-PROJ-028`). `FR-PROJ-001` makes the project the folder rather
  than the file, and `FR-CFG-004` lets the directed write surface create it
  again, so a project whose file was removed by hand is a project with an empty
  configuration and not a project that cannot be used.
- **The judgment is separable from the metadata read.** The ownership half
  cannot be exercised otherwise: a test process cannot give a file to another
  user, and a check only ever called with its own identifier is a check nothing
  has watched fire (`FR-PROJ-010`).

**The boundary is a comparison, not a lookup.** It is decided by comparing a
directory's filesystem device identifier with its parent's, which is why no
home directory is located and no environment variable is read on this path
([`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid),
which also settles how the process's own user identifier is obtained for checks
3 and 5, and states the accepted cost `FR-PROJ-005` carries).

**Checks 2 to 4 are a precondition of reading, not a validation of what was
read.** The file is opened once, relative to `.tpl`, without following a link
and without blocking; the checks run on that descriptor's metadata, and only
then is the text read through the same descriptor
([data-model.md](data-model.md#tplcfg),
[interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)).
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
([overview.md](overview.md#what-tpl-is-not)). It runs, with the child of
`password_command`, only on an invocation that opens a connection: `prepare`
in `project/settings.rs` checks the entry as written at step 5, and
`Prepared::connect` expands and runs the child where `cli/source.rs` reaches
the server on a miss, on `--direct`, on the read a render returns to, and for
`tpl cache load`; `tpl cfg database test` calls both halves through `resolve`.
A read the cache serves therefore reads no variable and starts no child
(`FR-CONF-056`). The two checks of `FR-CONF-040` and `FR-CONF-041` run on the
value as written at step 5 and again after expansion, for a reference that
expands to the empty string (`FR-CONF-050`). What the reader owes each caller,
and the three distinct read paths over the one file, are
[interfaces.md](interfaces.md#the-configuration-reader-and-the-writer)'s; the
key space and the file's shape are [data-model.md](data-model.md#tplcfg)'s.

**A connection string resolves to the same fields a discrete entry does.** The
resolution parses the URL, expands within each already-delimited field, and
hands on a host, a port, a user, a password and a server-side database name —
whichever way the entry was written — so nothing downstream distinguishes the
two shapes and an expanded value has no delimiter to move within (`FR-CONF-018`,
`FR-SEC-009`). Nothing composes a URL for the driver: `mariadb/` builds
`MySqlConnectOptions` field by field — host, port, user, password, database,
TLS mode, trust material — so the guarantee `FR-SEC-009` names rests on the
shape of the parsed type rather than on the order of two calls. **The third
step of `FR-CONF-018`, percent-encoding, is performed by nothing**:
`Field::encode` in `project/config/dsn.rs` has no caller outside its unit
tests (read at `9d2538e`), and no part of a `dsn` is decoded either. Whether the
requirement or the code moves, and what a literal `dsn` password may hold, is
[`OD-39`](open-decisions.md#od-39--a-dsn-value-percent-encoded-by-nothing),
open.

## The connection lifecycle

One connection at most, per invocation (`NFR-PERF-004`). There is no pool: the
process is ephemeral and issues a handful of queries, which is the root
coordination document's rule (`CLAUDE.md`, *Performance and Efficiency*) and also
what `NFR-PERF-004` makes verifiable from the server side (`FR-SRV-014`).

For a `mariadb` entry the lifecycle is five ordered stages inside `mariadb/`;
for a `mysql` entry it is the same five inside `mysql/`, stage 1 run by the
code of `mariadb/`, and the read-back of stage 3 and the checks of stage 4
MySQL's own ([The MySQL reader](#the-mysql-reader)); for a `sqlite` entry it is the six
stages of [The SQLite reader](#the-sqlite-reader). No module but those three
sends a statement, and which of them is reached is
[The provider dispatch](#the-provider-dispatch)'s.

| # | Stage | Rule | Forced by |
|---|---|---|---|
| 1 | Open | Opened late — when the reader is about to read, never at dispatch. One call to the driver runs the TCP connect, the login and, where the mode negotiates one, the TLS handshake inside the login; the driver records in a handle `tpl` passes it the phase the connection reaches, and `tpl` reads that handle after every poll of the call, to time each phase, and once the call ends in an error or an expired budget, to name the phase reached | `NFR-PERF-006`, `NFR-PERF-003`, [`ADR-005`](../adr/adr-005-async-runtime-scope.md); `FR-CONF-005`, `FR-GLOB-017`, [`ADR-014`](../adr/adr-014-mysql-provider-driver.md) point 6 |
| 2 | Set the session read-only | Once, at connection start, under its own deadline of `core.query_timeout` | `FR-SRV-008`, `FR-CONF-005` |
| 3 | Read the session state back | Once, immediately after stage 2, reading `@@session.tx_read_only` and nothing else — the spelling `transaction_read_only` does not exist on `10.11`; a failure of either half refuses the connection and reads no catalogue | `FR-SRV-009`, `FR-SRV-010`; `FR-SRV-038`, difference 12 |
| 4 | Probe the product and version | Before any statement other than stages 2 and 3; the series is derived from it and decides the treatment of every known difference. Stages 3 and 4 each take their own deadline of `core.query_timeout`, as stage 2 does | `FR-SRV-002`, `FR-SRV-034`, `FR-SRV-040`, `FR-SRV-022`, `FR-CONF-005` |
| 5 | Read, then close | Closed as soon as the read ends, and before any render starts: the protocol's quit is sent, the connection released, then the runtime dropped | `CLAUDE.md`, *Performance and Efficiency*; `NFR-PERF-004`; `FR-RND-040` |

Stages 2 and 3 are not disableable by any flag, key or environment condition
(`FR-SRV-011`), and they detect rather than prevent: only the closed statement
list of `FR-SRV-006` prevents, which is
[overview.md](overview.md#the-three-properties-that-bound-the-artefact)'s first
property and `security.md`'s subject. The verdicts stages 4 and 5 produce — an
unsupported product, a series below the window, a series above it — are
[interfaces.md](interfaces.md#the-catalogue-reader)'s, and how a failure of
stage 1 becomes a condition is
[interfaces.md](interfaces.md#a-server-connection-that-fails)'s.

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
thread, and `mysql::quiescent()` answers the same of `mysql/`'s own counts. The count is per thread because the runtime of
[`ADR-005`](../adr/adr-005-async-runtime-scope.md) is current-thread and the
counted values cannot leave the thread that built them; a process-wide count
would see another test's connection in the suite. The mechanism and the option
rejected are
[`OD-11`](open-decisions.md#od-11--the-scope-of-the-async-runtime)'s
amendment.

**Each connection phase and each of stages 2 to 4 writes its line at `-v`.**
`mariadb/connect.rs` times DNS resolution where it runs and the three phases
inside the driver's call by the readings of stage 1, and writes their lines
once the call has opened or failed; `run` in `mariadb/session.rs`, through
which `mysql/session.rs` issues its three statements too, bounds each session
statement and writes its line once it has ended. A
phase that failed or expired has its line, and a session statement's line is
not a catalogue query's (`FR-GLOB-017`). The readings bound what a line is
worth: a phase entered and left within one poll is not seen, so under
`required`, `verify-ca` and `verify-identity` the handshake's line is written
for a connection that opened whether or not a reading saw it, possibly at zero,
and under `preferred` an unseen handshake has none — the limit `FR-GLOB-017`
states since the eighty-fourth edition.

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

## The provider dispatch

`FR-CONF-051` makes the provider of the selected entry decide which contract
reads it. The decision and the options rejected are
[`OD-35`](open-decisions.md#od-35--the-provider-dispatch-and-the-sqlite-reader);
what belongs here is where the provider is decided and who reaches the choice.

| Step of `FR-ERR-006` | Where | What happens | Forced by |
|---|---|---|---|
| 3 | `project/config.rs` | `provider` is read as one of three values, an absent key meaning `mariadb`; a `path` holding `${` is refused | `FR-CONF-051`, `FR-CONF-054` |
| 5 | `project/settings.rs` | The keys the provider refuses, then the presence of `path`, then a value holding a NUL character, are checked on the entry as written; nothing is expanded and `password_command` is not run here — both wait for the connection branch of step 6 (`FR-CONF-056`); a `mysql` entry is checked and resolved by the functions that serve a `mariadb` entry, into the same settings type; a `sqlite` entry's `path` is resolved against the project directory | `FR-CONF-052` … `FR-CONF-054`, `FR-CONF-056`, `FR-CONF-057`, `FR-MYSQL-020` |
| 6 | `provider.rs`, once the cache did not answer | One exhaustive match sends a `mariadb` entry to `mariadb/`, a `mysql` entry to `mysql/` and a `sqlite` entry to `sqlite/`; all three produce the same `model::Database` | `FR-CONF-051`, `BR-LITE-002`, `BR-MYSQL-003` |

Three callers reach `provider.rs`, and no other module names a reader:
`cli/source.rs`, which every catalogue read goes through — the eight `schema`
subcommands, `tpl cache load`, and `tpl render` without `--context`;
`cli/cfg/connectivity.rs`, for `tpl cfg database test` (`FR-CFG-024`,
`FR-CFG-056`); and `bounded()` in `cli/render.rs`, which asks whether anything
of any reader's read is alive (`FR-RND-040`). The two checks of
`FR-CONF-040` and `FR-CONF-041` stay in `cli/source.rs` and reach the two
server arms alone: a `sqlite` entry carries neither key (`FR-CONF-052`), and a
`mysql` entry must name both (`FR-MYSQL-020`).

**The refused keys are checked before the child runs.** `FR-CONF-052` refuses
a key because a refused key "would look in force"; running the
`password_command` of an entry whose provider refuses it would put it in force
for as long as the child runs.

**One read shape for every provider.** A read is the whole catalogue whatever
was asked for, on the three grounds `cli/source.rs` records for a server: both
ends of every foreign key are embedded (`FR-CTX-006`, `FR-CTX-010`), the
population a suggestion is drawn from is in hand (`FR-SCH-010`), and the cache
is warmed (`FR-CACHE-007`). The count still does not depend on the objects,
which is all `FR-LITE-037` and `FR-MYSQL-042` ask of a named read.

## The MySQL reader

`mysql/` is the third reader, and the only module that issues the statements
of a `mysql` entry's session and read (`FR-MYSQL-015`, `FR-MYSQL-016`). That it
is a module of its own, and builds the runtime it needs, is
[`ADR-005`](../adr/adr-005-async-runtime-scope.md)'s as amended on 2026-11-04;
that it reads through the driver of
[`ADR-003`](../adr/adr-003-database-driver.md), and the patch that driver
carries for it, are [`ADR-014`](../adr/adr-014-mysql-provider-driver.md)'s.
The code at `cc511e2` implements this section against the seventy-seventh
edition.

| Part | What it owns | Forced by |
|---|---|---|
| `mysql.rs` | `open` builds the current-thread runtime, opens the connection through the function of `mariadb/connect.rs` that opens a `mariadb` one, handing it `mysql/fault.rs`'s classification, and runs the session start; `Session::close` sends the protocol's quit, closes the socket and then drops the runtime; `quiescent` answers for this module's own counts | `NFR-PERF-004`, `FR-RND-040`, `FR-MYSQL-020` |
| `mysql/session.rs` | The three connection-start statements, once each, in the order of `FR-MYSQL-017`; the read-back of `@@session.transaction_read_only`, confirmed by the integer `1` alone and decoded as an integer of either signedness; the product check; the series | `FR-MYSQL-006` … `FR-MYSQL-012`, `FR-MYSQL-017` … `FR-MYSQL-019` |
| `mysql/window.rs` | The set of `FR-MYSQL-003` as one table compiled in, newest first; the comparison is `mariadb/window.rs`'s | `FR-MYSQL-002`, `FR-MYSQL-003`, `FR-MYSQL-009`, `FR-MYSQL-010`, `BR-MYSQL-001` |
| `mysql/fault.rs` | Hands a failed connection to the classification of `mariadb/fault.rs`, with the provider `mysql` | `FR-MYSQL-045`, `FR-ERR-047` … `FR-ERR-049` |
| `mysql/catalogue.rs` | Issues the plan of `statements.rs` and nothing besides, each statement under the catalogue-query deadline, over the text protocol, with the database name conveyed by the rule of [`ADR-015`](../adr/adr-015-mysql-catalogue-name-conveyance.md) at its one composition site; the privilege probe of `FR-CFG-044`, against `SCHEMATA`, on the same path | `FR-MYSQL-015`, `FR-MYSQL-042`, `FR-MYSQL-048`, `FR-CFG-044` |
| `mysql/catalogue/statements.rs` | Twelve constant statement texts, one per table of the closed list, each naming only fields the mapping reads and holding one `?` where the database name goes | `FR-MYSQL-015`, `FR-MYSQL-016`, `FR-MYSQL-025` … `FR-MYSQL-038` |
| `mysql/catalogue/row.rs` | The decoding of a field; an integer field accepted in either signedness, because MySQL declares the integer fields of its catalogue with both | `FR-MYSQL-025` … `FR-MYSQL-038` |
| `mysql/catalogue/expression.rs` | The phrases of the attribute field, the outer parentheses of an expression default, and the one layer of backslash escaping removed | `FR-MYSQL-029`, `FR-MYSQL-030`, `FR-MYSQL-046` |
| `mysql/catalogue/fold.rs` | The rows become the model | `FR-MYSQL-023` … `FR-MYSQL-041` |

**What it shares with `mariadb/`, and why only that.** `FR-MYSQL-020`
connects a `mysql` entry exactly as a `mariadb` entry, so one function opens
both connections — the driver's options, the name resolution, the TLS mode of
[`ADR-002`](../adr/adr-002-tls-mode-mapping.md), the trust material and the
connection budget — and takes the classification of a failure as a parameter.
The statement runner and its deadline, the counting of what is alive, the
comparison of two series, the text accessors, the classification of a failure,
the attachment of both directions of every foreign key with the cross-check of
`FR-PRIV-019`, and the assembly of each table with its primary key read from
the index named `PRIMARY` are shared on the same ground: a requirement holds
them to the same behaviour for both providers (`FR-MYSQL-032`, `FR-MYSQL-033`,
`FR-MYSQL-041`). Everything a MySQL server answers differently — the read-back,
the product check, the window, the twelve statements and every field they
map — is `mysql/`'s, so no rule of `mysql-contract.md` is written in
`mariadb/` beyond the provider the shared classification is handed.

**The read is one plan, whatever was asked.** Twelve statements, one per
table, each restricted to the database the entry names — the one value a caller
supplies to a MySQL statement, which `FR-MYSQL-048` keeps from changing any
part of it but the value its rows are compared with, by the means
[`ADR-015`](../adr/adr-015-mysql-catalogue-name-conveyance.md) records; the
MariaDB reader binds the same value as a parameter; a named read issues
the same twelve, so the reader has no second entry point, and a `CHECK`
constraint is joined to its table in the fold, by its schema and its name,
rather than by a statement per table (`FR-MYSQL-034`, `FR-MYSQL-042`). Each
statement writes the per-query line of `FR-GLOB-017` before it is sent, under
the token a `mariadb` read writes; the three connection-start statements write
none.

**The fold differs from `mariadb/`'s where MySQL reports differently**, and
`fold.rs`'s own table names each difference with its requirement: the covered
table type and the routine type `LIBRARY` (`FR-MYSQL-023`), an index holding a
functional key part (`FR-MYSQL-024`), the default (`FR-MYSQL-029`), the
attribute field (`FR-MYSQL-030`), a generated column (`FR-MYSQL-031`), three
more `data_type` values (`FR-MYSQL-028`), `ignored` (`FR-MYSQL-032`), a
`CHECK` constraint's table and level (`FR-MYSQL-034`), a view's algorithm
(`FR-MYSQL-035`), and the escaping removed from three texts (`FR-MYSQL-046`).
A short read is marked in the shapes of `FR-MYSQL-041` by the completeness
verdict `mariadb/` already applies.

**The session guarantee cannot be stepped around.** `mysql::open` issues the
three connection-start statements before it returns, behind no condition, and
the catalogue read takes only the session `open` produces, so every catalogue
statement follows all three (`FR-MYSQL-017`, `FR-MYSQL-019`).

**`tpl cfg database test` for a `mysql` entry** runs the four steps of
`FR-CFG-024` with the MySQL counterpart of each: the connection carries no
default schema, the session start of `mysql/session.rs` answers the second and
third steps, and the probe of `FR-CFG-044` reads `SCHEMATA` for the database
the entry names. An error packet carrying one of the four privilege errors of
that requirement — `1044`, `1142`, `1143`, `1227`, held once in
`mariadb/fault.rs` for both server providers — is its `false`; any other is the
`69` of `FR-ERR-036`.

## The SQLite reader

`sqlite/` is the second reader, and the only module that sends a statement for
a `sqlite` entry (`FR-LITE-015`, `FR-LITE-016`). It is shaped like `mariadb/`
and shares nothing with it; both write `model/` and both are bounded through
`deadline.rs`. It is prescribed by
[`OD-35`](open-decisions.md#od-35--the-provider-dispatch-and-the-sqlite-reader).
The reader at `19abb1f`, as rmp `#323` changes it, implements everything this
section prescribes, against the seventy-first edition.

| Part | What it owns | Forced by |
|---|---|---|
| `sqlite.rs` | The one route to a connection — kind check, open, hardening and probe, in that order and behind no condition; the one function every statement runs through; the close; the per-thread count of open connections and live watchdogs | `FR-LITE-017`, `FR-LITE-014`, `NFR-PERF-004`, `FR-RND-040` |
| `sqlite/open.rs` | The kind of the file, and the read-only open | `FR-LITE-005`, `FR-LITE-006` |
| `sqlite/harden.rs` | Each control and limit, set and read back, and the comparison of the two as a function separable from any connection | `FR-LITE-010` … `FR-LITE-012` |
| `sqlite/watch.rs` | The busy timeout and the watchdog that bound every statement | `FR-LITE-009`, [`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection) |
| `sqlite/fault.rs` | Where the library's result stops and a condition of `FR-ERR-001` begins: one pure function over the stage or statement and the result code, and the table of SQLite's names for the codes | `FR-LITE-040`, `FR-LITE-007`, `FR-LITE-012`, `FR-GLOB-018`, [`OD-38`](open-decisions.md#od-38--a-sqlite-result-code-the-contract-does-not-classify) |
| `sqlite/catalogue.rs` | The read transaction, the plan, and the read that owns its values | `FR-LITE-015`, `FR-LITE-037`, `FR-LITE-042` |
| `sqlite/catalogue/statements.rs` | The text of every statement, one constant each | `FR-LITE-015`, `FR-LITE-016` |
| `sqlite/catalogue/check.rs` | The verdict of the check over one row of the schema table | `FR-LITE-041` |
| `sqlite/catalogue/fold.rs` | The values into the model: coverage, affinity, default, rowid alias, keys, the match of names, order | `FR-LITE-019` … `FR-LITE-034`, `BR-LITE-003` |
| `sqlite/catalogue/trigger.rs` | The timing and the event, read from a trigger's stored header, and the reading of SQLite's comment syntax the check shares for its third and seventh kinds | `FR-LITE-032`, `FR-LITE-041` |

**The connection start, in the order `FR-LITE-017` fixes.** No byte of the file
is parsed before stage 5 has confirmed every control; the probe at stage 6 is
the first statement that reads the file (`FR-LITE-002`, observed there). On a
read, the `BEGIN DEFERRED` of `FR-LITE-042` and then the check of
`FR-LITE-041`, the first catalogue statement of the plan below, follow it.

| # | Stage | Through | Statements | Refused with |
|---|---|---|---|---|
| 1 | The kind of the file | `std::fs::metadata` on the resolved path, which "will traverse symbolic links" and "currently corresponds to the `stat` function on Unix" (Rust standard library documentation, consulted 2026-11-03), so it opens nothing and a FIFO does not block | none | `78` for anything but a regular file; a lookup that fails — the path names nothing, a link on it dangles, the filesystem refuses it — opens nothing and is `74` under the first row of `FR-LITE-007`, to which `FR-LITE-005` routes it, the `cause` naming what the filesystem returned |
| 2 | The open | `Connection::open_with_flags` with `SQLITE_OPEN_READ_ONLY`, `SQLITE_OPEN_NO_MUTEX` and `SQLITE_OPEN_EXRESCODE`, and without `SQLITE_OPEN_CREATE`, `SQLITE_OPEN_URI` or `SQLITE_OPEN_NOFOLLOW` | none | `74` for `SQLITE_CANTOPEN` (`FR-LITE-006`, `FR-LITE-007`); any other result code by `FR-LITE-040` |
| 3 | The statement bound | The busy timeout replaced, and the watchdog started ([`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection)) | none | — |
| 4 | The build | `PRAGMA compile_options`, which must return the row `OMIT_LOAD_EXTENSION`, for extension loading, and one of `TEMP_STORE=1`, `TEMP_STORE=2` and `TEMP_STORE=3`, for temporary storage | 1 | `70` (`FR-LITE-010`, `FR-LITE-012`) |
| 5 | The other controls, and the limits | The read-only open read back through `is_readonly` on `main`; defensive mode on, `trusted_schema` off and views disabled, through `set_db_config` and read back through `db_config`; `query_only` on, `cell_size_check` on, `mmap_size` `0` and `temp_store` `MEMORY`, read back as `2`, each set and read back through its pragma; the three limits through `set_limit`, read back through `limit` | 8 | `70` (`FR-LITE-010` … `FR-LITE-012`) |
| 6 | The version probe | `SELECT sqlite_version()` | 1 | By `FR-LITE-040`: `69`, `74` or `78` under the rows of `FR-LITE-007`, and the code its own table gives any other result code |

Why each flag of stage 2 is set or left out:

| Flag | Decision | Ground |
|---|---|---|
| `SQLITE_OPEN_READ_ONLY`, without `SQLITE_OPEN_CREATE` | Set; create left out | `FR-LITE-006`. A read-only open "returns an error if the database doesn't exist" (docs.rs, `rusqlite` 0.41.1, `OpenFlags`, consulted 2026-11-03) |
| `SQLITE_OPEN_EXRESCODE` | Set | `FR-LITE-007` separates `SQLITE_READONLY_DIRECTORY` from the rest of its class, and the library returns primary codes by default (`https://sqlite.org/rescode.html`, last updated 2026-02-18, consulted 2026-11-03); the flag enables extended codes from SQLite 3.37.0 (`OpenFlags`, as above) |
| `SQLITE_OPEN_NOFOLLOW` | Left out | `FR-LITE-005` follows links |
| `SQLITE_OPEN_URI` | Left out, and not needed to keep the path literal | The bundled build defines `SQLITE_USE_URI` (`libsqlite3-sys` 0.39.1, `build.rs`, consulted 2026-11-03), under which a filename beginning with `file:` is a URI whatever the flags; the open is handed the absolute path of `FR-CONF-054`, which begins with `/`, and SQLite "interprets any filename that does not begin with "file:" as an ordinary filename" (`https://sqlite.org/uri.html`, consulted 2026-11-03) |
| `SQLITE_OPEN_NO_MUTEX` | Set | The connection is used from one thread; it is `rusqlite`'s own default mode (`OpenFlags`, as above) |

**Three controls go through the configuration interface and four through their
pragma.** `rusqlite`'s `DbConfig` carries `SQLITE_DBCONFIG_DEFENSIVE`,
`SQLITE_DBCONFIG_TRUSTED_SCHEMA` and `SQLITE_DBCONFIG_ENABLE_VIEW`, set by
`set_db_config` and read back by `db_config` without a statement
([technology-stack.md](technology-stack.md#the-sqlite-driver-and-the-library-it-carries));
`FR-LITE-015` admits the `trusted_schema` pragma without obliging it, and names
no statement for views. `query_only`, `cell_size_check`, `mmap_size` and
`temp_store` have no variant there, so each is a pair of statements of
`FR-LITE-015`'s first row.
Temporary storage is in effect only where both its read-back at stage 5 and the
`TEMP_STORE` row of stage 4 hold (`FR-LITE-010`). The limits use the `limits`
feature [`ADR-013`](../adr/adr-013-sqlite-driver.md) enables.

**The catalogue plan.** Seven catalogue statements follow the probe, fixed
before the first is sent, which is `FR-LITE-037` by construction
(`FR-LITE-015`, observed there, for the sixty-ninth edition, as eight with the
probe). On a read they run inside the read transaction below. No statement
invokes `pragma_table_list`.

| # | Reads | Through | Forced by |
|---|---|---|---|
| 1 | The check: every row of the schema table, each of its five values as stored | `main.sqlite_schema` | `FR-LITE-041`, `FR-LITE-017` |
| 2 | The encoding | `pragma_encoding` | `FR-LITE-023` |
| 3 | The rows of type `table` and `view`, with their name, type and root page; for a table, whether `pragma_index_info` returns a row for its name | `main.sqlite_schema` joined to `pragma_index_info` | `FR-LITE-019` … `FR-LITE-021`, `FR-LITE-028` |
| 4 | The columns of every table | `main.sqlite_schema` joined to `pragma_table_xinfo` | `FR-LITE-025` … `FR-LITE-029` |
| 5 | The indexes of every table, and their members | `main.sqlite_schema` joined to `pragma_index_list` and `pragma_index_xinfo` | `FR-LITE-028` … `FR-LITE-030` |
| 6 | The foreign keys of every table | `main.sqlite_schema` joined to `pragma_foreign_key_list` | `FR-LITE-031` |
| 7 | The stored text of every view and trigger | `main.sqlite_schema` | `FR-LITE-032`, `FR-LITE-033` |

Four rules bind the text of every statement:

- **A pragma function that takes a name is passed the schema `main` and only
  the name of a row of type `table` whose root page is not `0`**, the row
  restricted in the statement before the function is reached, so no view's or
  virtual table's name is passed to one, no view is compiled and no virtual
  table is connected
  (`FR-LITE-015`, `FR-LITE-019`, `FR-LITE-020`; verified by row 32 of the
  [register](verification.md#mandated-by-the-corpus)).
- **The check comes first.** Every statement that reads through a pragma
  function follows it, inside the same read transaction, so a table or a view
  that would replace one has been refused before one runs, and none added by
  another connection after the check is read (`FR-LITE-017`, `FR-LITE-041`,
  `FR-LITE-042`).
- **No value read from the file enters a statement's text.**
- **The plan is a constant.** No path adds a statement (`FR-LITE-037`), and no
  statement reads from a view: a view is read from statements 3 and 7 alone, so
  one whose definition does not compile is read whole and unmarked
  (`FR-LITE-033`, `FR-LITE-036`).

**The read transaction.** `sqlite/catalogue.rs` holds the check and the six
statements after it in one [read
transaction](../../specification/glossary.md#read-transaction), so that all
seven read the snapshot the check read (`FR-LITE-042`). Why a deferred
transaction, why `COMMIT` rather than `ROLLBACK`, and the options rejected are
that requirement's and are not restated.

| Question | Decision | Ground |
|---|---|---|
| Where `BEGIN DEFERRED` is issued | After the probe and immediately before the check. It reads nothing; the snapshot is taken by the check | `FR-LITE-017`, `FR-LITE-042` |
| Where `COMMIT` is issued | Immediately after statement 7 completes, and on no other path | `FR-LITE-042` |
| A read that ends early — the check refuses the file, a statement fails, a bound ends one | No `COMMIT` is issued; the transaction ends with the connection, which the library rolls back when it closes | `FR-LITE-042`; `FR-LITE-041`: a refused file is read by no further statement |
| How each of the two runs | Through the one function every statement runs through, under its own bound ([`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection)); neither writes the line of `FR-GLOB-017`, because neither is a catalogue statement | `FR-LITE-009`; `FR-LITE-015` and `FR-LITE-037`, as amended and noted in the seventieth edition |
| How a failure of either is classified | As a statement of stage `Transaction`, by steps 2 and 3 of `FR-LITE-040` and never by step 1 ([interfaces.md](interfaces.md#the-sqlite-reader)) | `FR-LITE-040`; the note on `FR-ERR-006` of the seventieth edition |
| `tpl cfg database test` | Issues the check alone and no transaction statement | `FR-LITE-042`, `FR-CFG-056` |

**The check is one constant with two callers**: statement 1 of every read,
inside the read transaction, and the whole of step 4 of
`tpl cfg database test`, outside any (`FR-CFG-056`, `FR-LITE-042`). Its verdict
is decided in `tpl`, not in the statement's text.

| Question | Decision | Ground |
|---|---|---|
| Where the verdict is decided | `sqlite/catalogue/check.rs`: a pure function over one row's five values as `ValueRef`s, testable as a vector | `FR-LITE-041`; the third kind needs the word after `CREATE`, and the seventh the words `CREATE TRIGGER` and the one after them, under SQLite's comment syntax, which `trigger.rs` already reads for `FR-LITE-032`, so one reading of that syntax serves all three |
| The `pragma_` prefix | The first seven bytes of the name, of a text or a BLOB, compared with `pragma_` ignoring ASCII case. An integer or a real is written in digits and cannot begin with it | `FR-LITE-041`, first kind: "whatever its storage class" |
| The root page of the third kind | Compared with the integer `0`. A `NULL` root page, which the library refuses at the probe, is not `0` | `FR-LITE-041`, third kind; `FR-LITE-020`, *Source* |
| The type of the fourth kind | The whole value's bytes compared with each of `table`, `index`, `view` and `trigger`: a NUL byte is part of the value, and no letter case is folded | `FR-LITE-041`, fourth kind |
| The text of the fifth kind | `sql` `NULL` or the empty text, on a row whose type is not exactly `index` | `FR-LITE-041`, fifth kind |
| The NUL character of the sixth kind | A NUL byte anywhere in a text `name`, `tbl_name` or `sql`; the `cause` names the first of the three, in that order, that holds one | `FR-LITE-041`, sixth kind |
| The header of the seventh kind | On a row of type `trigger`, the words `CREATE` and `TRIGGER`, read under SQLite's comment syntax ignoring ASCII case, followed by a token that is not an unquoted `IF` | `FR-LITE-041`, seventh kind |
| Every later comparison of `type` | Exact, in each statement's text and in the fold: the fourth kind has refused every other spelling before any of them runs | `FR-LITE-041`, the paragraph after its table; `FR-LITE-020` |
| Which row is reported | The first refused row in the order the statement returns rows, under the first kind it meets in the order of `FR-LITE-041`'s table | `FR-LITE-041`: the `cause` names one row and one kind |
| What is held | Each row is judged as it is stepped; only a refused row is kept | `CLAUDE.md`, *Implementation rules*: streaming wherever the operation allows it |

**The read owns its values, typed and converted at the boundary.** A `rusqlite`
row borrows its statement, so every value the read keeps is copied out as its
row is read, and the statements are dropped before the connection closes. A
value the join repeats over consecutive rows is kept once per run of them — a
table's name in statements 4 to 6, and an index's `name`, `unique` and
`origin` in statement 5 — and rows that come apart form a run per part, so no
run assumes an order. A field of a closed set of spellings — the schema table's
`type` in statements 3 and 7 (`FR-LITE-041`, fourth kind), `origin`
(`FR-LITE-030`), and `on_update`, `on_delete` and `match` (`FR-LITE-031`) —
holds the recorded spelling it equals, uncopied, and any other value as read.
Both spare a copy and change no value (`CLAUDE.md`, *Implementation rules*:
minimal allocation on hot paths). Each is read through its
SQLite storage class and never coerced, a SQL `NULL` kept apart from every
value (`FR-LITE-004`, step 2); a pragma field of a class the library does not
document for it is a violated internal invariant, `70` (`FR-ERR-030`), as a
catalogue shape the model cannot represent is in `mariadb/`. Statements 3 to
7 read the snapshot the check read (`FR-LITE-042`), so every value of the
schema table they read holds a class the check confirmed, and no writer that
takes SQLite's locks can change that. A value of an excluded class read there
all the same — which only a process writing the file's bytes without those
locks could produce, a case `FR-LITE-042` claims nothing against — is still
the `78` of the check, never the `70` of a pragma field (`FR-LITE-040`, its
paragraph on values). Text is read
as bytes and converted with U+FFFD substituted, at this boundary and nowhere
else (`FR-OUT-017`). Where that conversion replaced invalid UTF-8 in the name of
an object, a stored row, a run's table or an index run, the name also keeps
the bytes the library returned for it (`Raw`), and elsewhere its text is those
bytes: the fold groups, matches and orders by them and converts nothing
(`FR-LITE-043`).

**The fold** applies `FR-LITE-019` … `FR-LITE-034` and `BR-LITE-003` to the
owned values through the one constructor a table has, so `FR-CAT-044` is
checked here as on every other producer; what it derives, and where, is
[data-model.md](data-model.md#a-sqlite-read)'s. Coverage is applied before any
object is folded. The trigger
header reader is a pure function of the stored text, testable as a vector,
returning one timing and one event or the `70` of `FR-LITE-032`. The version
probe's answer becomes the `server` object through a constructor of its own,
deriving `series` from the first two dot-separated parts (`FR-LITE-002`); an
answer without them is the `70` of `FR-ERR-030`, on the ground `FR-LITE-012`
gives a control: the library is part of the binary.

**The fold indexes before it looks up.** Every collection the fold reads by a
name is grouped once, and no object is found by scanning another collection.

| Collection | Read by | Grouped as |
|---|---|---|
| The column, index and key rows | Their table, by the bytes the file holds for its name | Each statement's runs, sorted once by those bytes; a table's runs found by binary search |
| A table's index runs | Their index | Sorted by the bytes of the index's name, so that the runs of one index stand together |
| The stored rows of the views | Their name's bytes | Sorted once by those bytes; a view's row found by binary search |
| The covered tables, the views, and a parent table's columns | The name `BR-LITE-003` matches | A map keyed by the name's bytes, hashed and compared with the 26 ASCII letters folded, holding no folded copy; a parent's columns mapped only for a key that names them |

**Two names that convert alike refuse the read before anything is folded**
(`FR-LITE-043`). After grouping, `alike` in `sqlite/catalogue/fold.rs` compares
the names of each set — the covered tables and the views together, then, table
by table, the columns, the indexes and the triggers of one covered table — and
returns the `78` of `Error::SqliteNamesAlike` for the first pair whose bytes
differ and whose converted texts are equal; nothing is presented and nothing
is cached.

Grouping holds whatever order the rows arrive in. No statement orders its rows:
every order the model carries is the fold's (`FR-LITE-025`, `FR-LITE-029` …
`FR-LITE-031`, `NFR-DET-002`). The fold runs after the last statement, outside
every statement's bound
([`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection)),
so a lookup that scanned would let a file of many objects cost time growing
with their square where the statements' grows with their number (`FR-LITE-009`,
*Consequence*; `FR-LITE-020`, *Provenance*). **Discharged at `82eb1d4` — the
discrepancy this paragraph recorded.** The fold at `cb99236` scanned, in
`table()`, `keys()`, `indexes()`, `views()` and the placement of triggers; at
`19abb1f` every such lookup went through a map built once, in `Grouped` and
`Names` of `sqlite/catalogue/fold.rs`. rmp `#323` keeps `Names` and replaces
the maps of `Grouped` with the sorted runs above.

**The refusal makes every order by name total.** The objects are taken in the
order of their names' bytes before they are ordered by text, so the fold is the
same whatever order the rows came in, and no two names of one set tie once
`alike` has passed. Byte order is the order SQLite's `BINARY` collation gives,
which "Compares string data using memcmp(), regardless of text encoding"
(`https://sqlite.org/datatype3.html`, consulted 2026-11-04).
The bytes come from `sqlite3_column_text` (docs.rs, `rusqlite` 0.41.1,
`statement.rs`, `value_ref`, consulted 2026-11-04), whose cases of conversion
name a conversion to UTF-8 for UTF-16 content alone
(`https://sqlite.org/c3ref/column_blob.html`, consulted 2026-11-04), so in a
UTF-8 file they are the stored bytes. That a UTF-16 file cannot hold two names
that convert alike is
stated in the code (`sqlite/catalogue.rs`, `Raw`) and not confirmed in the
official documentation.

**The close.** Every statement is dropped, the watchdog is stopped and joined,
and then the connection is closed — before any render starts (`FR-RND-040`). A
failed close is not a condition: the answer is already produced, and `close` is
"functionally equivalent to the `Drop` implementation" apart from returning the
error (docs.rs, `rusqlite` 0.41.1, `Connection::close`, consulted 2026-11-03).

### What was taken from the reference projects

Two reference projects read a SQLite catalogue through the pragma functions:
SeaQL `sea-schema`, at version 0.18.1 on `master`
(`src/sqlite/discovery.rs`, `src/sqlite/def/table.rs`), and Atlas
(`ariga/atlas`, `master`, `sql/sqlite/inspect.go`), both read at their public
sources on 2026-11-03. Neither is copied; both were studied.

| Practice | Project | Taken or refused | Ground |
|---|---|---|---|
| `pragma_table_list` as the source of an object's type, `WITHOUT ROWID` and `STRICT`, rather than `sqlite_master` alone | Atlas | **Refused** in the sixty-ninth edition; taken until then | No statement invokes it, by the user's decision recorded under `FR-LITE-020`; the schema table is the source (`FR-LITE-015`, `FR-LITE-020`, `FR-LITE-028`) |
| `table_xinfo` rather than `table_info`, so that `hidden` tells a generated column | Both | **Taken** | `FR-LITE-025` |
| `index_xinfo` rather than `index_info`, for `key` and `desc` | Atlas | **Taken** | `FR-LITE-029`, `FR-LITE-030` |
| The rows of `foreign_key_list` merged into one key per `id` | `sea-schema` | **Taken** | `FR-LITE-031` |
| One statement per table, and per index | Both | **Refused** | The count would grow with the objects (`FR-LITE-037`, `NFR-PERF-001`) |
| A table name formatted into the statement's text | Both | **Refused** | The second rule above |
| `AUTOINCREMENT` matched in the stored `CREATE TABLE` | Both | **Refused** | `FR-LITE-028` reads the rowid alias from the pragmas |
| Generation expressions, `CHECK` constraints and foreign-key names parsed from the stored text | Atlas | **Refused** | `BR-LITE-002`, `FR-LITE-034` |

## The catalogue reader and the query-count invariants

`mariadb/` holds two entry points and a fixed statement repertoire. The reader's
surface and its obligations are
[interfaces.md](interfaces.md#the-catalogue-reader)'s; what belongs here is
where the two entry points sit and what bounds the number of statements. The
MySQL reader's plan, which has one entry point, is
[The MySQL reader](#the-mysql-reader)'s, and the SQLite reader's is
[The SQLite reader](#the-sqlite-reader)'s.

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
  matched in memory afterwards and passed to the database in no statement
  (`FR-SCH-012`, `FR-SCH-013`, `FR-SCH-015`).
- **The count is observable without reading the source.** One diagnostic line
  per catalogue query carries a fixed leading token (`FR-GLOB-017`,
  `NFR-PERF-008`), which is what lets `NFR-PERF-007` be satisfied from outside
  the process. Observability as a subject is `operations.md`, settled in
  [`OD-17`](open-decisions.md#od-17--observability).

The three privilege detections read the shape of the rows the server returned
and are `mariadb/catalogue/completeness.rs`'s, running on every read that presents the
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
| Miss | The database is read, the result is written, **then** the answer is produced | `FR-CACHE-007` |
| Unreadable file, or a version the binary does not know | A **silent** miss: read the database, rewrite, report nothing | `FR-CACHE-033`, `FR-CDOC-004` |
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

**A cached render defers two parts of each table** (`FR-CACHE-038`). Under
`tpl render`, `cache.rs` decodes a table's file but keeps the members of its
`foreign_keys` and `referenced_by` as the bytes of each JSON value, and
`cli/render/context/lazy.rs` decodes a part, validating it as an up-front read
would, the first time the template reaches it. A part that fails then is a
miss found during the render, answered by `FR-CACHE-039`: the render is
abandoned, nothing of it is written, and the database is read. Every other
command that reads the cache decodes everything before its first byte of
output. The members of one collection are read relative to the collection's
folder, opened once at step 6 with no link followed and held for the render,
rather than walking the path again per file.

Everything the layer puts on disk — the location, the keying, the encoding, the
filenames, the atomic write, the two independent versions — is
[data-model.md](data-model.md#tplcache)'s and is not repeated.

## The model: one junction, five producers, three consumers

A live read, a cached read and a supplied context document must present the same
objects and the same fields, so `model/` is the single junction of the corpus
(`specification/catalogue-coverage.md` *Overview*; `BR-SCH-001`, `FR-SCH-022`).
The model is also the same whichever provider read it (`catalogue-coverage.md`,
note of the sixty-fifth edition; `BR-MYSQL-003`), so a live read has three
producers.

| Producers | Consumers |
|---|---|
| `mariadb/`, on a read through a `mariadb` entry | The `text` layouts, in `output/` |
| `mysql/`, on a read through a `mysql` entry | The JSON emitter, in `output/` |
| `sqlite/`, on a read through a `sqlite` entry | The render, in `render/` |
| `cache/`, on a cached read | |
| `cli/`, from a supplied context document | |

Two design consequences follow from the junction being single.

- **An invariant over an assembled model is checked by each of the five
  producers and not on the emitting path**, because `render/` consumes the same
  model without passing through `output/` (`FR-CAT-044`;
  [interfaces.md](interfaces.md#the-catalogue-reader)).
- **The lossy conversion of catalogue bytes to text happens at the boundary of
  the reader — `mariadb/`, `mysql/` or `sqlite/` — and nowhere else**, so all three
  consumers inherit one substitution (`FR-OUT-017`;
  [interfaces.md](interfaces.md#the-catalogue-reader)).

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
| `database.rs` | The root: four metadata fields, among them the provider, whose closed enumeration is declared here; the `server` object; the three collections the three covered kinds stand in | `FR-CAT-001`, `FR-CAT-007`, `FR-CAT-008`, `FR-CTX-035`, `FR-CTX-036`, `FR-CTX-043` |
| `server.rs` | The three keys, the series derivation, the two constructors that keep a server's probe apart from a supplied document, and a third for the SQLite library's probe | `FR-CTX-031` … `FR-CTX-034`, `BR-CTX-006`, `FR-SRV-040`, `FR-LITE-002` |
| `table.rs` | The coverage predicate over the table-type string, everything a table carries, and the one constructor a table has | `FR-CAT-001` … `FR-CAT-006`, `FR-CAT-009` … `FR-CAT-015`, `FR-CAT-031`, `FR-CAT-032`, `FR-CAT-043`, `FR-CAT-044`, `FR-SCH-009` |
| `column.rs` | A column's own row, the table it names, and the static attributes read from one catalogue field | `FR-SCH-009`, `FR-CAT-027`, `FR-CAT-035`, `FR-CAT-041`, `FR-CAT-051`, `FR-CTX-019`, `FR-CTX-020`, `FR-CTX-021` |
| `column_type.rs` | The raw string, the eight parts, and the member scan by quote state | `FR-CAT-034`, `FR-CAT-038`, `FR-CTX-014` … `FR-CTX-018`, `FR-CTX-038` … `FR-CTX-041` |
| `column_default.rs` | The three-way discriminant and the eight-row classifier applied in order | `FR-CTX-011` … `FR-CTX-013`, `FR-CTX-037`, `BR-CTX-002` |
| `index.rs` | The fold to one object with an ordered column list, and the name a MariaDB primary key is told by; a SQLite table's key is held by the table, set by its producer ([data-model.md](data-model.md#a-sqlite-read)) | `FR-CAT-010`, `FR-CAT-042`, `FR-CAT-043`, `FR-LITE-029` |
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
| Any statement to a server or a database file, and the lossy conversion of catalogue bytes | `mariadb/`, `sqlite/` | `FR-SRV-006`, `FR-SRV-007`, `FR-LITE-015`, `FR-LITE-016`, `FR-OUT-017`; the model holds only valid text ([interfaces.md](interfaces.md#the-catalogue-reader)) |
| The envelope, the two forms, escaping, the writer | `output/` | `FR-OUT-024`, `FR-OUT-032`; the document is handed over as a value and no byte of it is composed here |
| Opening the file a supplied document is read from | The caller | The read-back direction is given bytes, so the one path that opens a file stays outside the model |
| The three privilege detections | `mariadb/catalogue/completeness.rs` | Each is a property of a **read** and not of the model (`BR-PRIV-003`, [`OD-05`](open-decisions.md#od-05--the-module-decomposition)); only the marking they produce is carried here |

### Four construction choices inside `model/`

Each is a choice the corpus leaves open, and each is made once so that a
requirement holds by construction rather than by every later caller
remembering it.

| Choice | What it is | Ground |
|---|---|---|
| **Refusal is an absent field, not a filter** | A field excluded on one of the four grounds has no field in any type, so there is nothing for a later reader, emitter or caller to write into | `BR-CAT-005`, `FR-CAT-024`, `FR-CAT-026`, `FR-CAT-029` — whose list is empty, so it excludes nothing today — and `FR-CTX-021`. `FR-CAT-026` binds six consumers, and a filter would be six places to forget |
| **Every collection is ordered where it is built** | One byte-wise comparator, called once per collection at the point that collection of the document is assembled; an excepted collection says so at its own call site | `NFR-DET-002`. The three exceptions the default rule would corrupt are unreachable by it, because the default rule is bounded by a trait their member types do not implement. **Which component applies an ordering** is recorded as a discrepancy in [interfaces.md](interfaces.md#ordering-one-default-and-seven-exceptions) and is not settled here |
| **The one-hop cut is a type parameter** | One table shape, instantiated twice: at the first hop a reference resolves to a table, one hop in it resolves to a name. A traversal terminates because a name carries no table | `FR-CTX-006` … `FR-CTX-010`, `BR-CTX-001`, [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md). There is no depth counter and no visited set, so the three shapes `FR-CTX-009` enumerates are one rule, which is the constant depth `BR-CTX-001` chose |
| **Key order is the type's field order** | Serialisation is derived throughout, with two projections onto a private shape; no unordered map appears on the emitting path | `FR-OUT-013`, [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions). The two projections, and the four shapes that are not the model's own types, are [interfaces.md](interfaces.md#the-two-directions-over-the-document)'s |

Three further refusals are structural in the same way and are stated so they
are not read as omissions: a referential action the server discards silently
has no variant (`FR-CAT-033`) — the `SET DEFAULT` a SQLite read reports is
carried by the value `FR-CAT-055` gives an unrecorded rule
([`OD-35`](open-decisions.md#od-35--the-provider-dispatch-and-the-sqlite-reader));
a table type the model does not cover has none
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
| Coverage applied before `--pattern`, and applied to the column read as well as the object read | `FR-CAT-028`, `FR-CAT-052` | The reader, `mariadb/` or `sqlite/` ([interfaces.md](interfaces.md#the-catalogue-reader)) |
| The document's obedience to the four transport rules | `FR-CTX-002` | The envelope and the emitter, `output/` |
| What a cache-served document does not promise | `FR-CTX-024`, `FR-CTX-025`, `BR-CTX-004` | `cache/` ([data-model.md](data-model.md#what-a-cached-document-does-not-promise)) |
| The three context variables that do not come from the database | `FR-CTX-026` … `FR-CTX-030`, `BR-CTX-005` | [Context assembly](#context-assembly), in `cli/` and `render/` |

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
| The engine is built lazily | No engine exists for an invocation that does not render | `NFR-PERF-006`, `CLAUDE.md` *Performance and Efficiency* |
| Templates are loaded and compiled at render time from disk | A template changes without the binary being rebuilt | `CLAUDE.md`, *Implementation Invariants*; `FR-TMPL-004` |
| The loader closure calls the one resolution function | The three `template` subcommands that resolve without the engine call the same function, so one boundary is enforced once | [`OD-15`](open-decisions.md#od-15--the-template-loader), `FR-TMPL-023` … `FR-TMPL-026` |
| Undefined behaviour is set to the strict variant | Reading a field that does not exist fails the render | [`OD-14`](open-decisions.md#od-14--which-undefined-behaviour-the-engine-is-configured-with), `FR-SEM-012`, `FR-SEM-013` |
| An output formatter of `tpl`'s own is installed | A `null` writes nothing and a boolean writes `true` or `false`, whatever the engine would have written | `FR-SEM-010`, `FR-SEM-011`, `FR-SEM-021` |
| Auto-escaping is set explicitly, to off, and is never left at the engine's default | No property of a template's name can turn escaping on; escaping happens only where a template asks for it | `FR-ENV-026`, `FR-ENV-027`, `FR-ENV-028` |
| Each template is compiled once per process and reused | A loop in a template does not reparse it | `CLAUDE.md`, *Performance and Efficiency* |
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
| The context source — a catalogue read or a supplied document | `cache/`, `mariadb/`, `mysql/` or `sqlite/`, or `cli/` |
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

## The phase deadlines

`FR-SEC-022` requires a deadline on every blocking phase, because a hung process
is the failure the never-interactive invariant exists to prevent (`BR-CLI-003`).
`FR-CONF-005` names eight kinds of phase and maps each onto one of four keys;
the session statement, the fifth, is three phases on a server connection. The
mechanism for each, and the rejected alternatives, are
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) and are
not restated.

| # | Phase | Budget | Bounded by | Imposed by |
|---|---|---|---|---|
| 1 | DNS resolution | The shared connection budget | The runtime's own timer, around a resolution `tpl` performs itself | `FR-CONF-005`, `FR-ERR-027` |
| 2 | TCP connect | What remains of the same budget | The runtime's own timer, around the driver's connect call | `FR-CONF-005`, `FR-ERR-027` |
| 3 | TLS handshake | What remains of the same budget | The same timer and the same call | `FR-CONF-005`, `FR-ERR-027` |
| 4 | The login | What remains of the same budget | The same timer and the same call; the login runs before and after a TLS handshake | `FR-CONF-005`, `FR-ERR-027`, `FR-ERR-047` |
| 5 | A session statement — the read-only statement, its read-back, the version probe | The query budget, for each statement from its own start, shared with nothing | The runtime's own timer, around each statement | `FR-CONF-005`, `FR-ERR-027`, `FR-SRV-010` |
| 6 | Catalogue query | The query budget, per query — for a `sqlite` entry, per statement on the connection | The runtime's own timer; for a `sqlite` entry, the library's busy timeout and a watchdog thread holding the interrupt handle ([`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection)) | `FR-CONF-005`, `FR-ERR-027`, `FR-LITE-009` |
| 7 | `password_command` | The password budget, until the child has exited **and** its standard output has closed | A reader thread draining the child's pipe and a polling loop in the parent, which kills the child's whole process group and reports the expiry | `FR-CONF-005`, `FR-CONF-028` |
| 8 | Render | The render budget | A watchdog thread that writes the diagnostic and terminates the process; the same thread observes the render memory limit ([The render bounds](#the-render-bounds)) | `FR-CONF-005`, `FR-RND-033`, `FR-RND-039` |

Six rules bind the table.

- **Phases 1 to 4 share one budget**, measured from the start of the first of
  them that runs and consumed in the order they run; they are not given one
  budget each (`FR-CONF-005`). `FR-CONF-004` resolves that budget, and the three
  other budgets, from the declared key or its built-in default.
- **Phases 2 to 4 are one call to the driver, and the phase is the one the
  connection reached**, read from the handle the driver records it in, never
  from the call that was running (`FR-CONF-005`;
  [`ADR-014`](../adr/adr-014-mysql-provider-driver.md) point 6). An expired
  budget is reported against that phase, and a failure is classified by it
  ([interfaces.md](interfaces.md#a-server-connection-that-fails)). The driver
  resolves the host a second time inside the call, so a budget that expires
  there is reported against the TCP connect, the bound
  [`ADR-014`](../adr/adr-014-mysql-provider-driver.md) states.
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
- **A `sqlite` entry runs phases 6 and 8 alone.** DNS resolution, TCP connect,
  the TLS handshake, the login, the session statements and `password_command`
  do not run, and every statement on the connection to the file is bounded as
  a catalogue query (`FR-LITE-009`). The open of the file is bounded by
  nothing, which is `FR-LITE-005`'s accepted cost. A `mysql` entry runs all
  eight, as a `mariadb` entry does (`FR-MYSQL-020`).

**`deadline.rs` owns the construct and five modules use it** — the runtimes
inside `mariadb/` and `mysql/`, the watchdog inside `sqlite/`, the child process
run from `project/password.rs`, and the render in `render/` — because a budget
measured from process start and shared by five modules belongs to none of them
([`OD-05`](open-decisions.md#od-05--the-module-decomposition),
[interfaces.md](interfaces.md#the-shared-functions-and-the-phase-clock)).

**It is at the crate root and not under `project/`.** `project/` resolves the
four values from `[core]`; it does not own the clock they are applied through,
because four of the five users are not in it and a budget measured from process
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
  end the invocation with `65` and no read of the database; the deadline and
  the memory limit end it from the watchdog. A render that returned within
  every bound has its values dropped, not leaked, before the database is read,
  so the render that follows is not charged for them under the memory limit.
- **A render refuses to start while anything of a catalogue read is alive.**
  `bounded()` checks `provider::quiescent()` first and returns the `70` of a
  violated invariant otherwise (`FR-RND-040`, `FR-ERR-030`); every caller reaches
  it with the read already closed, so the check is the in-process observation
  `FR-RND-040` names rather than a reachable path. `provider::quiescent()`
  answers for the three readers — no MariaDB or MySQL connection or runtime, and
  no SQLite connection or watchdog, on the calling thread
  ([`OD-35`](open-decisions.md#od-35--the-provider-dispatch-and-the-sqlite-reader)).

## Lazy initialisation

Nothing is initialised for a command that cannot use it. The rule is the root
coordination document's — no heavy static initialisation, lazy initialisation by
default (`CLAUDE.md`, *Performance and Efficiency*) — and three requirements make
it observable rather than reviewable.

| Nothing is built | Unless | Forced by |
|---|---|---|
| A discovery walk, a configuration read, a socket | The command is not one of the four `FR-PROJ-025` exempts | `NFR-PERF-005`, `FR-PROJ-025` |
| A connection — to a server, or of the SQLite library to a database file | The command needs catalogue data and the cache did not answer | `NFR-PERF-006`, `NFR-PERF-003`, `FR-CACHE-011`, `FR-RND-022`, `FR-LITE-037` |
| The template engine | The invocation renders | `NFR-PERF-006` |
| The asynchronous runtime | `mariadb/` or `mysql/` is reached, and at most one of them per invocation; never for a `sqlite` entry | [`ADR-005`](../adr/adr-005-async-runtime-scope.md), [`ADR-013`](../adr/adr-013-sqlite-driver.md) |
| A thread of the crate's own | A phase bounded without the runtime is entered — the statements on a SQLite connection among them | [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced), [`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection) |

Each row is verified by an observation made outside the process and never by
reading the source (`NFR-PERF-007`), which is why laziness is placed at a module
boundary in every row: the absence of a socket, of an open file and of a thread
can be observed from outside, while the absence of a code path cannot. Which of
the four instruments reaches which absence, and on which targets, is
[verification.md](verification.md#the-nine-observations-made-outside-the-process)'s.

## The synchronous process, and the runtime boundary

That the process is synchronous, where the asynchronous runtime is built, that
it serves the two server providers alone, and which deadlines may use its timers
are [`ADR-005`](../adr/adr-005-async-runtime-scope.md)'s, registered as
[`OD-11`](open-decisions.md#od-11--the-scope-of-the-async-runtime), and are not
restated. In this document they decide the *Bounded by* column of
[The phase deadlines](#the-phase-deadlines), whose mechanisms are
[`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) and, for
a `sqlite` entry,
[`OD-36`](open-decisions.md#od-36--the-deadline-of-a-statement-on-a-sqlite-connection),
and the runtime's row of [Lazy initialisation](#lazy-initialisation).

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
