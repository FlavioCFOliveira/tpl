---
title: tpl Functional Specification
status: approved
last-reviewed: 2026-09-10
related: [glossary.md, cli-contract.md, catalogue-coverage.md, open-questions.md, upstream-divergences.md]
---

# tpl Functional Specification

This folder is the single source of functional truth for `tpl`. It is independent
and self-sufficient: no functional requirement lives outside it, and nothing here
defers to another document for its meaning.

`README.md` at the repository root is the entry door to the repository.
`CLAUDE.md` at the repository root is agent coordination. Neither carries
functional requirements. Where either still repeats functional content that this
specification now owns, the divergence is recorded in
[upstream-divergences.md](upstream-divergences.md) and the duplicate must be
removed from those files.

## Scope

The specification has been written in eight editions. All are in force; each
adds to the ones before it and amends them in place, and every amendment
carries an *Amended in the nth edition* note beside the requirement it
changes.

### First edition — the command-line surface

Command tree, subcommands, aliases, positional arguments, global and local
flags, permitted values, parsing rules, configuration precedence, output
formats, the JSON plumbing contract, help layout, exit codes, validation order,
and the security rules that cut across the surface.

It deliberately stopped at the render context, at the catalogue, and at every
guarantee that concerns the server rather than the command line.

### Second edition — the model, the document, and the guarantees

Everything the first edition named and stopped at, except the implementation:

- What `tpl` reads from a database, and what it excludes —
  [catalogue-coverage.md](catalogue-coverage.md).
- The structure of the single document that carries it —
  [context-document.md](context-document.md).
- What a template may call, and what happens when it calls it wrongly —
  [template-environment.md](template-environment.md) and
  [render-semantics.md](render-semantics.md).
- Which servers are supported, and what the read-only promise actually
  guarantees — [server-contract.md](server-contract.md).
- What happens when a reader's privileges do not reach the whole model —
  [privileges-and-completeness.md](privileges-and-completeness.md).
- What the cache stores about itself, and what a cache-served document does not
  promise — [cache-documents.md](cache-documents.md).
- The performance properties that are observable and permanent, and the
  workloads against which the rest is measured —
  [performance-requirements.md](performance-requirements.md).

### Third edition — the JSON documents and fourteen decisions

The audit of 2026-09-10 read the corpus end to end and put its findings to the
user. Fourteen decisions came back and are written into the owning modules.
The largest is the one the audit named as the blocker:

- **One envelope for every JSON document** —
  [output-formats.md](output-formats.md). `--format json` was declared the
  plumbing contract while only two of the seventeen documents `tpl` can emit
  had a specified shape. `FR-OUT-024` fixes the envelope all seventeen share,
  and each owning module fixes its own `data`, per `BR-OUT-002`.

The other thirteen close a contradiction or a gap: the behaviour of all
seventeen registered filters and tests (`FR-ENV-030` through `FR-ENV-043`);
`--timeout` as an overall budget that composes with the per-phase deadlines
rather than killing them (`FR-GLOB-011`); which commands require a project
(`FR-PROJ-025`); a default ordering rule for every collection
(`NFR-DET-002`); the outcome of an empty result set (`FR-OUT-033`); the
content of `vars`, `tpl`, and `now` (`FR-CTX-026` through `FR-CTX-030`); the
durability of a `.tpl/.cfg` rewrite (`FR-CFG-041`); the producing condition for
`70` (`FR-ERR-030`); the three password-key combinations (`FR-CONF-007`); the
one exception to "nothing is written outside `.tpl`" (`FR-PROJ-024`); the tab
in a diagnostic message (`FR-ERR-024`); `tpl template show` as byte-for-byte
output (`FR-OUT-019`); and a mandated test for the dump round-trip
(`BR-SCH-004`).

One finding was left open deliberately: `FR-PRIV-015` now states *why* the
privilege cross-check covers views alone, and records that the choice to
generalise it was blocked on observing `OQ-041`. The sixth edition observed it
and generalised the check to one further kind.

### Fourth edition — the supported version window

The product owner replaced the open-ended version floor with a window that
moves as MariaDB's own maintenance moves, and raised what `tpl` owes the
differences between the versions in it.

- **Which MariaDB versions are supported** —
  [server-contract.md](server-contract.md). `FR-SRV-001` states a two-part
  criterion: the series of the three most recent major families that are also
  under community maintenance. `FR-SRV-015` names the four series that criterion
  admits on 2026-09-10, with its source and its verification date, and
  `FR-SRV-019` makes re-verifying it a release gate. The floor of MariaDB 10.6
  is withdrawn, because 10.6 left community maintenance on 2026-07-06.
- **What a difference between two supported series obliges** — the same file.
  `FR-SRV-024` normalises a fact reported differently, `FR-SRV-004` marks a fact
  a series does not have, `FR-SRV-025` excludes a field whose meaning differs,
  and `FR-SRV-026` states the whole of it as one testable equivalence. Which
  differences actually exist was `OQ-045`, which the sixth edition answered
  against the container: seven were found and none of them is a field the
  model carries.
- **The server version in the model** — `FR-SRV-028` and `FR-CTX-031`. A
  template that must accommodate a difference could not previously see which
  server it was rendering against.

Three open questions are closed and none is left open. `OQ-044` — the outcome
below the window — is `FR-SRV-020`: refusal with `78`, carrying the message of
`FR-SRV-030`. The other two were the escalations the ceiling created, and both
came back decided:

- `OQ-073` — a server **newer** than the window is **read**, treated as the
  newest supported series, and the divergence is marked **in the document**:
  `FR-SRV-031` through `FR-SRV-033`, with the marker a permanent enumerated
  field, `FR-CTX-034`. It could not be a field that appears only when there is
  something to report — `FR-SEM-012` would then fail the very guard that looks
  for it.
- `OQ-074` — the two server checks attach to **connecting**, not to reading, so
  they reach `tpl cfg database test`: `FR-SRV-002` as amended and `FR-SRV-034`,
  which closes the list of commands that open a connection. That command was
  the only one escaping the gate.

### Fifth edition — the last of the decisions

The fourth edition left thirty-three points open that a decision could settle.
All thirty-three were put to the user, all came back decided, and the fifth
edition writes them in. **No design question remained open**: what it left in
[open-questions.md](open-questions.md) was twenty-four facts about a MariaDB
catalogue nobody had observed, and one driver mapping that waited on a
measurement.

Four of the changes reach beyond the module that owns them.

- **No error is ever JSON** — [errors-and-exit-codes.md](errors-and-exit-codes.md).
  `FR-ERR-033` makes `--format` apply to a result and never to a failure: the
  four-line text of `FR-ERR-008` is the whole of what a caller receives, and
  the exit code is the sole machine-comparable signal. Five requirements are
  withdrawn with the document they served — `FR-ERR-014`, the document itself;
  `FR-ERR-015` and `FR-ERR-016`, the `kind` field and its compatibility rule,
  which had no other carrier; and `FR-ERR-017` with `FR-ERR-018`, the argument
  pre-scan, whose only purpose was to decide the question `FR-ERR-033` now
  settles before any argument is read. Because the text is now an error's only
  channel of detail, `FR-ERR-034` raises what it must contain, stating per exit
  code what that code's `cause` line is obliged to name.
- **The performance budgets become a protocol** —
  [performance-requirements.md](performance-requirements.md). `NFR-PERF-014`
  carries a provisional figure or a stated blank for every one of its nine
  budgets, `NFR-PERF-019` says what provisional means, and `NFR-PERF-020` is
  the gate by which a measurement replaces one. `NFR-PERF-018` states the four
  build targets, which every rule about measurement had assumed and none had
  named; the Linux pair is statically linked against `musl`.
- **`tpl cfg database test` reports the reader's privileges** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-044` specifies a probe cheap
  enough not to depend on a field list, `FR-CFG-045` keeps its negative answer
  at exit `0`, and `FR-CFG-039` carries it as `can_read_catalogue`. The
  consequence is that this command now reads the catalogue, so `FR-CACHE-010`
  is corrected: what it never does is read anything *into the model*.
- **`referenced_by` embeds** — [context-document.md](context-document.md).
  `FR-CTX-010` makes the incoming direction symmetric with the outgoing one,
  and `FR-CTX-009` is restated so that one hop remains one hop in both
  directions. It roughly doubles the document again, and the accepted cost
  names which provisional figure that puts under pressure.

The rest settle a point within one module: the `password_command` failure
package (`FR-CONF-031` through `FR-CONF-033`); a `.cfg` that is strict in both
directions (`FR-CONF-034`, `FR-CONF-035`); a DSN that carries no query
parameters (`FR-CONF-011`); the five TLS modes as a criterion the driver must
meet (`FR-CONF-036`); three new entry flags (`FR-CFG-027` as amended); the
complete short-flag set of the tool (`FR-GLOB-024`); a global flag in any
position (`FR-CLI-024`); a nested command path for `tpl help` (`FR-HELP-026`);
a failed cache write that succeeds silently (`FR-CACHE-036`); a restricted
object that is never cached (`FR-CACHE-037`); the shape of `restricted`
(`FR-PRIV-016`); the three collections of the `database` object (`FR-CTX-035`);
a fourth entry in the closed statement list (`FR-SRV-006`); the ordering of
`tpl template list` (`FR-TMPL-013`); `escape` in group 1 with a required target
(`FR-ENV-044`); `sql_type` as a normalised type name (`FR-ENV-039`); the engine
pin as an obligation rather than a number (`FR-ENV-003`); and the password
sentinel test (`BR-SEC-003`).

Three defects of structure were corrected rather than decided. `FR-SRV-024`
normalises a fact present on more than one series rather than on every series,
which makes `BR-SRV-006`'s three cases exhaustive over differences of presence
and of representation — the sixth edition found a difference of **value** that
they still do not cover, `OQ-075`. `FR-SRV-025` pointed at
`FR-CAT-025`, which is a closing rule and not a list; it now writes into
`FR-CAT-029`, a second exclusion list created for it. And the register
`FR-SRV-027` required had no home; `FR-SRV-036` gives it one.

### Sixth edition — the first observations

The fifth edition left twenty-five entries open, and every one of them waited
on something outside this corpus. Both arrived on 2026-09-10: the container
fixture of `scripts/mariadb/`, buildable at each of the four series of
`FR-SRV-015`, and the driver choice, settled by measurement. The sixth edition
writes down what was observed — and, where an observation showed a requirement
already in force to be wrong, corrects it.

**Five entries close.**

- **`OQ-003`, the TLS mapping** — [configuration-model.md](configuration-model.md).
  `FR-CONF-038` states the behaviour of each of the five modes against a server
  that offers TLS and a server that does not, verified cell by cell against
  running servers with encryption read from the live session rather than from
  the configuration that asked for it. The corpus still names no driver: the
  mapping onto the chosen one is recorded in the architecture decision records
  and cited, as `FR-ENV-003` does for the engine pin.
- **`OQ-045`, the differences between the four series** —
  [server-contract.md](server-contract.md). Seven differences were found and
  **none of them was a field the model then carried**, so the divergence
  register of `FR-SRV-036` stayed empty — as an observed result rather than
  as an absence of one. `FR-SRV-038` records all seven, because six of them
  constrain the reader, the fixture, or an open question rather than the
  document, and `FR-SRV-027` records only what the model accommodates. The
  seventh edition took the record to eleven and the register to two rows.
- **`OQ-041`, how an unreadable view is reported** —
  [privileges-and-completeness.md](privileges-and-completeness.md), and it is
  the correction described below.
- **`OQ-025`, the table types** — `FR-CAT-031` fixes the set a server can
  emit, and
  `FR-CAT-032` requires the exclusion of temporary tables to be a filter,
  because on `10.11` the catalogue omits them and on the other three it does
  not.
- **`OQ-070`, the escaping of a backtick** — `FR-ENV-045`: doubling, accepted
  by all four series.

**Three requirements in force were corrected, all in one file.** The
observation of a reduced-privilege reader established that a privilege-driven
absence has **three shapes**, not one, and that `FR-PRIV-011` was written
against a shape the server does not produce.

- `FR-PRIV-011` inferred a missing privilege from an empty view collection. The
  collection is not empty: every row is present and only the definition is
  short, and it is short by being the **empty string**. The check could never
  have fired. It is rewritten per view and is now strictly stronger.
- `FR-PRIV-010` was an unqualified prohibition on reporting an absence whose
  cause is a missing privilege, and it cannot be honoured: a hidden trigger
  list and an empty one are identical in every byte a reader can obtain.
  `FR-PRIV-020` states exactly where the guarantee stops.
- `FR-PRIV-015` left open whether the cross-check should generalise. It
  generalises to exactly one further kind — `FR-PRIV-019`, the foreign key,
  because a reduced reader keeps every key column and loses every referential
  rule, so a table appears structurally whole with no relations at all.

**One requirement is new because the reader cannot work without it.**
`FR-SRV-037` forbids a statement naming a fixed list of `INFORMATION_SCHEMA`
columns that is not present on every series: three of those tables differ in
width, and naming an absent column is a hard `ERROR 1054` rather than a `NULL`
or a warning.

**One entry is new.** `OQ-075` records a case `BR-SRV-006`'s three treatments
do not cover — a field whose *value* differs between series because the
servers' own defaults differ. It cannot be closed by observing more.

**Twenty entries were left open, and the reason was worth stating
precisely.** They were catalogue field lists, and they were no longer blocked
by the absence of a container. The campaign recorded the *differences*
between the four servers, not what any of them returned, and a comparison
that found no difference is not a record of a field list. Each entry named
the query that would close it, and the seventh edition ran every one of
them.

### Seventh edition — the catalogue field lists

The sixth edition observed the four series against each other and recorded the
**differences**. It could not close the twenty entries that asked for the
field lists themselves, because a comparison that finds no difference is not a
record of what was returned. A second observation pass, on 2026-09-10 against
all four series of `FR-SRV-015`, recorded every one of them verbatim. The
seventh edition writes them in, settles the one entry that needed a decision,
and corrects what the evidence contradicts.

**Twenty entries close, and the last entry the corpus held closes with
them.** The field lists of every object kind are now fixed: comments and
defaults (`FR-CAT-039`), the column attribute string (`FR-CAT-041`), indexes
(`FR-CAT-042`), the primary key (`FR-CAT-043`), foreign keys
(`FR-CAT-045`), `CHECK` constraints
(`FR-CAT-046`), views (`FR-CAT-047`), routines (`FR-CAT-048`), routine
parameters (`FR-CAT-049`), triggers (`FR-CAT-050`), and generated columns
(`FR-CAT-051`); and on the document side the default discriminant
(`FR-CTX-037`), the raw type string (`FR-CTX-038`), the `ENUM` member list
(`FR-CTX-039`), the type parts (`FR-CTX-040`), the character set and
collation (`FR-CTX-041`), and the metadata fields of the `database` object
(`FR-CTX-036`). `FR-ENV-046` fixes the membership of the three type families.
`BR-CAT-005` states, once, the rule by which a catalogue field list becomes a
model property list — carry by default, exclude on a stated ground — so that
no omission is indistinguishable from an oversight.

**Four decisions came from the product owner, and one of them reorders two
values this specification had held to be compatible.**

- **A collation is passed through, not normalised** —
  [server-contract.md](server-contract.md). `FR-SRV-039` is a **fourth**
  treatment beside the three of `BR-SRV-006`: a field present everywhere,
  meaning the same everywhere, whose value differs because the servers' own
  defaults differ, is carried **exactly as the server returns it**. Each
  collation is a collation, and character sets and collations are preserved
  regardless of the server version. The cost is stated rather than
  discovered: `FR-SRV-026` must except such a field, so the same template
  rendered against a `10.11` and a `12.3` will differ wherever it reads one.
  This ranks **fidelity to the server above determinism across servers**, and
  it is the only place the two are traded against each other. It closes
  `OQ-075` and fills the gap the sixth edition found in `BR-SRV-006`.
- **The default discriminant is three-way** —
  [context-document.md](context-document.md). `FR-CTX-012` required four
  distinguishable cases and the catalogue draws three: `DEFAULT NULL` and a
  nullable column with no `DEFAULT` return identical bytes, because in
  MariaDB they are one state. `kind: null` covers both, the bare `null` is
  reserved for the only case that returns SQL `NULL`, and `FR-CTX-013`
  narrows the enumeration to three values — a **narrowing of contract
  surface**.
- **The primary key comes from the index table** — `FR-CAT-043`. The three
  catalogue sources disagree on a system-versioned table, and one of them
  names an implicit period column the column table does not carry at all.
  `FR-CAT-011` now names a source and `FR-CAT-044` makes the governing
  property an invariant: a key never names a column absent from the same
  table's column list.
- **Pinned trust material is additional to the public root bundle** —
  `FR-CONF-039`, accepted as the sixth edition wrote it. Requiring exclusive
  trust would leave `FR-CONF-036` with no qualifying driver, and a
  requirement no implementation can satisfy is worse than a weaker guarantee
  stated honestly.

**Four requirements in force were contradicted by the evidence, and two of
them describe an artefact that cannot exist.**

- **`FR-CTX-012` could not be implemented**, as above.
- **`FR-SCH-009` requires a table character set the catalogue does not
  report.** The table catalogue carries a collation and no character set, on
  all four series. The item is removed rather than reconstructed from the
  collation's leading segment, which would be an inference; `CHECK`
  constraints, required since the second edition and never named in that
  list, are added in the same amendment.
- **`NFR-DET-002` would have corrupted three ordered collections.** Its
  default rule sorts by name, and it excepted an index's columns but not a
  primary key's, not a foreign key's — whose two column lists are paired
  positionally — and not an `ENUM` member list, whose order **is** the
  ordinal each member is stored as. All three are silent corruptions of
  correct-looking output, and none was reachable before the field lists were
  recorded.
- **`FR-CAT-035` assumed a field holds one attribute at a time.** The column
  attribute field carries six distinct values and no column of the fixture
  carries two at once, so the multi-attribute case is unobserved.
  `FR-CAT-041` records the population and bounds the claim.

**Two exclusions grew, and one of them had to.** `FR-CAT-024` gains a
routine's creation and alteration timestamps and a trigger's creation
timestamp: they are wall-clock times recorded when the object was installed
and they differ between two servers of the **same** series, so carrying any
of them would make `FR-SRV-026` unsatisfiable outright. `FR-CAT-052` requires
the coverage filter to reach the column read as well as the object read,
because the column catalogue carries the eight columns of a sequence and the
columns of every view alongside those of the tables.

**The record of differences between the series grows from seven to eleven**,
per `FR-SRV-038`, and one of the four new ones matters beyond its content:
the declared nullability of the index table's comment column splits `10.11`
and `11.4` from `11.8` and `12.3`. It is the only difference in the record
that does not fall after `10.11`, and it is recorded as a caution that a
difference may fall anywhere in the window. The divergence register of
`FR-SRV-036`, empty through six editions, gains its first two rows — both
created by `FR-SRV-039`.

**The last open entry closes, and it closes on a limit rather than on an
answer.** `FR-SRV-040` fixes what the version probe returns and `FR-SRV-041`
the **necessary** condition for a server to be MariaDB. The **sufficient**
condition cannot be observed against a fixture of four MariaDB servers, and
it is very likely unobtainable in principle: every check available is made
over responses the server itself controls, so a server that emulates MariaDB
completely is indistinguishable from MariaDB by any wire observation.
`FR-SRV-041` therefore states the limit in its own text — a server determined
to pass as MariaDB will pass — and `OQ-042` closes rather than waiting for
evidence that will not arrive. **The index of
[open-questions.md](open-questions.md) is now empty, and this specification
is complete**: no requirement of it is waiting on a decision, a measurement,
or an observation.

### Eighth edition — three contradictions between requirements in force

Harvesting the seventh edition for the technical design read requirement
against requirement rather than requirement against evidence, and found three
places where two requirements **both in force** could not both be satisfied.
None was a gap and none needed a decision from outside the corpus: in each
case one of the two was an invariant of the whole surface and the other a rule
of one module, so the resolution was to name the requirement that yields and
to write the reason into its own text. Two stale entries were corrected in the
same pass.

- **A boundary that read the environment** —
  [project-and-discovery.md](project-and-discovery.md). `FR-PROJ-005` stopped
  project discovery at the user's home directory, which can only be located
  from `HOME`. `FR-CLI-021` reads no environment variable to determine the
  location of the project and `FR-CLI-023` names the single exception, so the
  boundary was an environment read this corpus prohibits — and it made two
  identical command lines in two shells discover two different projects, which
  is `BR-CLI-002` failing in its own terms. `FR-PROJ-005` yields: the home
  boundary is dropped, the mount point remains, and the trust checks of
  `FR-PROJ-009` through `FR-PROJ-011` are named as what actually refuses a
  planted project. `BR-CLI-002` gains the clause it was missing — nothing a
  shell can set decides which project is discovered, which entry is selected,
  or which server is reached — and `FR-SEC-013` follows.
- **Two test seams that could appear on no surface** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md) and
  [server-contract.md](server-contract.md). `FR-ERR-031` required a deliberate
  trigger for `70`, and `FR-SRV-035` a seam presenting the reader with a
  series above its own window. Every mechanism reachable from outside the
  process collides with a requirement in force: a command or a flag with
  `FR-CLI-002` and `FR-HELP-021`, an environment variable with `FR-CLI-021`
  and `NFR-DET-001`, and a feature-selected build with the property that the
  artefact verified is the artefact distributed. Both seams therefore move
  **inside the process**, reachable from nothing a caller can write, and none
  of those four requirements yields. What yields is `BR-ERR-001`, for `70`
  alone and in its own text, and `FR-SRV-035`'s assertion on the exit code.
  Both requirements state the composition that stands in — the guard is
  observed in process, and the step from an error condition to an exit status
  is observed by the nine other codes — and both state plainly that the
  composition is reasoned rather than executed.
- **An acceptance test with no fixture** —
  [configuration-model.md](configuration-model.md). `FR-CONF-038` recorded, in
  its own text, that `tpl` with default configuration cannot reach the fixture
  of `scripts/mariadb/` over TCP on any series, and that an acceptance test
  for the default mode needs a certificate that names the host. The fixture
  provides none, so the default of `FR-CONF-013` was the one cell of a
  ten-cell table with no test. The obligation is now a requirement: the
  fixture presents, at each series of `FR-SRV-015`, a server whose certificate
  names the host, and retains a server offering no TLS for the right-hand
  column. Dropping the test with the cost stated was rejected, because a
  default that nothing exercises end to end is the case a caller meets first.

**Two editorial corrections.** The glossary's *DSN* form still ended
`[?params]`, which `FR-CONF-009` removed when `FR-CONF-011` admitted no
parameter. And `DIV-034` said "thirteen" excluded catalogue fields in one
clause while its own seventh-edition note said sixteen; the table of
`FR-CAT-024` carries sixteen rows, so the clause was wrong and is corrected.

**One divergence is recorded rather than resolved.** `FR-ERR-030` makes a
caught top-level panic a producing condition of `70`, and the release profile
the root coordination document states aborts on panic, under which that
condition cannot exist in the distributed binary. The requirement is
unchanged, because the specification precedes the implementation, and the
correction owed is `DIV-045`. It is not an open question: nothing in this
corpus waits on it, and the choice it names belongs to an architecture
decision.

**No open question is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty. Each resolution above
rests on an invariant this corpus already held, and where a choice remained
the requirement records the option rejected and why.

### Still out of scope

- The Rust implementation: its crates, its module layout, its types, and its
  library API. Only the JSON document and the command line are contract, so the
  shape of the library is an architecture decision and not a requirement. The
  database driver is part of this: `FR-CONF-036` states what one must be able
  to express, and no requirement names one.
- The text of any catalogue query. What is read is specified; how it is read is
  not.
- Every **ratified** performance figure and every measured baseline. Those live
  in `BENCHMARKS.md`, cited by the requirement that needs them. A budget that
  has not been measured may carry a **provisional** figure in `NFR-PERF-014`,
  marked as such under `NFR-PERF-019` and removed from this corpus by the same
  step that records the real one, per `NFR-PERF-020`.
- The pinned version of the template engine. `FR-ENV-003` requires the pin to
  exist, to be recorded in the project's architecture decision records, and to
  be cited from there; the number never enters this corpus.
- Template authoring guidance.

Where the specification touches one of these boundaries, it names it and stops.

## File index

| File | Prefix | Responsibility |
|---|---|---|
| [glossary.md](glossary.md) | `TERM` | Terms used across the corpus |
| [cli-contract.md](cli-contract.md) | `CLI` | Invocation grammar, closed command tree, parsing rules |
| [global-flags.md](global-flags.md) | `GLOB` | The seven global flags and configuration precedence |
| [help-and-version.md](help-and-version.md) | `HELP` | Help forms, help layout, the JSON command tree, version |
| [schema-commands.md](schema-commands.md) | `SCH` | First arm: `tpl schema …` |
| [template-commands.md](template-commands.md) | `TMPL` | Second arm: `tpl template …` |
| [render-command.md](render-command.md) | `RND` | Third arm: `tpl render …` |
| [cache-commands.md](cache-commands.md) | `CACHE` | The catalogue cache and `tpl cache …` |
| [cfg-commands.md](cfg-commands.md) | `CFG` | `tpl cfg …`, including the `database` entry group |
| [configuration-model.md](configuration-model.md) | `CONF` | The `.tpl/.cfg` file: keys, types, expansion, connection settings |
| [project-and-discovery.md](project-and-discovery.md) | `PROJ` | The `.tpl` project, its discovery, and `tpl init` |
| [output-formats.md](output-formats.md) | `OUT` | `text` and `json` output, encoding, `--pretty` |
| [errors-and-exit-codes.md](errors-and-exit-codes.md) | `ERR` | Exit codes, validation order, message format, suggestions |
| [security.md](security.md) | `SEC` | Cross-cutting security rules, each pointing at its owning module |
| [use-cases.md](use-cases.md) | `UC` | End-to-end flows across the surface |
| [catalogue-coverage.md](catalogue-coverage.md) | `CAT` | What enters the model from the catalogue, and what is excluded |
| [context-document.md](context-document.md) | `CTX` | The structure of the document that carries the model |
| [template-environment.md](template-environment.md) | `ENV` | Filters, tests, global functions, and what is contract |
| [render-semantics.md](render-semantics.md) | `SEM` | Whitespace, operands, null versus absence, author-signalled failure |
| [server-contract.md](server-contract.md) | `SRV` | The supported version window, differences between series, the closed statement list, the read-only promise |
| [privileges-and-completeness.md](privileges-and-completeness.md) | `PRIV` | Complete and incomplete reads, and how a short read is reported |
| [cache-documents.md](cache-documents.md) | `CDOC` | Cache versions, completeness records, and the `source` field |
| [performance-requirements.md](performance-requirements.md) | `PERF` | Requirements of form, reference workloads, measurement protocol |
| [open-questions.md](open-questions.md) | `OQ` | Points this specification cannot yet fix |
| [upstream-divergences.md](upstream-divergences.md) | `DIV` | Corrections owed to the root `CLAUDE.md` and `README.md` |

## Identifier scheme

Identifiers are stable once assigned. They are never renumbered to tidy a file,
never reused after a requirement is withdrawn, and are the reference used in
commit messages, task descriptions, and test names. A gap in a sequence is
therefore expected, not a defect: all seventy-five open questions are closed and
their numbers are not reused. The *Closed* table of
[open-questions.md](open-questions.md#closed) records each and what answered
it.

| Form | Meaning |
|---|---|
| `FR-<MODULE>-<NNN>` | Functional requirement — observable behaviour of `tpl` |
| `BR-<MODULE>-<NNN>` | Business rule — an invariant or policy that constrains many requirements |
| `NFR-<CATEGORY>-<NNN>` | Non-functional requirement. Two categories are in use: `DET` for determinism and `PERF` for performance |
| `UC-<NNN>` | Use case, numbered across the corpus rather than per module |
| `WL-<NNN>` | Reference workload, numbered across the corpus |
| `OQ-<NNN>` | Open question, numbered across the corpus |
| `DIV-<NNN>` | Divergence owed to a file outside this folder |

`<MODULE>` is the prefix listed in the file index above. A requirement is
numbered within its file, from `001`.

**Numeric order is not reading order, and identifiers are never renumbered.** A
number is assigned when the requirement is written, taking the next unused
number in that file. Where a later edition adds a requirement, the requirement
is placed where it belongs to be read — beside the rule it qualifies, inside
the section that owns the subject — and it keeps the number it was given. A
file therefore reads in an order its numbers do not follow, and that is
correct.

*Amended in the fifth edition.* The rule read "in declaration order", which was
true of the first edition and of nothing since: four editions have inserted
requirements where the reading order wanted them. Read literally, it obliged a
renumbering that would break several hundred cross-references, every citation
in a commit message, and every test name — to buy a property no reader needs.
Stating what is actually done removes the obligation and the temptation
together. A reader looking for a requirement uses its identifier, and a reader
reading a file follows the sections.

A **withdrawn** requirement keeps its heading, in place, carrying a
*Withdrawn in the nth edition* note that says what it required, what withdrew
it, and that the identifier is retired. It is not deleted, because a
cross-reference written before the withdrawal must resolve to an explanation
rather than to nothing. `FR-ERR-014` through `FR-ERR-018` are the first five.

## Requirement style

Functional requirements use EARS phrasing, one form per requirement, never mixed
within one statement:

- Ubiquitous — `The system SHALL <action>.`
- Event-driven — `WHEN <event>, the system SHALL <action>.`
- State-driven — `WHILE <state>, the system SHALL <action>.`
- Unwanted behaviour — `IF <condition>, THEN the system SHALL <action>.`
- Optional feature — `WHERE <feature is included>, the system SHALL <action>.`

Modal verbs follow RFC 2119 and RFC 8174: `MUST` and `SHALL` are absolute
obligations, `MUST NOT` and `SHALL NOT` absolute prohibitions, `SHOULD` a strong
recommendation with a stated exception, `MAY` a genuine option. The words are
written in capitals only where they carry that meaning.

Business rules are stated as declarative invariants rather than in EARS form,
because they constrain the whole module rather than one interaction.

## Writing conventions

- The specification is written in English.
- `tpl` is the subject of every functional requirement; "the system" and "tpl"
  are the same actor.
- Exit codes are always written as the bare number and, on first mention in a
  section, with the `sysexits.h` name: `78` (`EX_CONFIG`).
- Command lines are shown in fenced blocks without a shell prompt, unless the
  block deliberately shows both an invocation and its output.
- No emoji, no decorative characters, no HTML.
- A requirement that cannot be tested or demonstrated is not a requirement.
- **Where a guarantee stops, the requirement says so in its own text**, and
  always in the same shape: the limiting clause belongs in the requirement
  rather than in a note beside it; an *Observed* note records what was seen;
  a *Consequence, stated plainly* note says what a reader would otherwise
  wrongly assume; and a *What would change this* note names what would lift
  the limit. Three requirements are written this way — `FR-SRV-041`,
  `FR-PRIV-020` and `FR-CONF-039` — and each cites the other two, so that
  three honest limits read as one pattern rather than three accidents.

## Status legend

| Status | Meaning |
|---|---|
| `draft` | Written from a settled decision, not yet reviewed against the corpus |
| `approved` | Reviewed and in force |
| `deprecated` | Superseded; retained for traceability, not to be implemented |

## Provenance

Every requirement in this specification derives from one of three sources:

1. Six decision logs. The first interview settled 53 points about the CLI
   surface; the second settled 28 points about the model, the document, the
   template surface, the server contract, and performance, and recorded four
   defects found in the first edition; the third is the audit of 2026-09-10,
   whose fifteen findings the user answered with fourteen decisions and one
   deliberate deferral; the fourth settled the supported version window; the
   fifth put every remaining decidable question to the user and settled all
   thirty-three, four of them against the recommendation; the sixth settled
   the four points the observation raised, one of which — `FR-SRV-039` —
   reversed a recommendation and reordered two values this corpus had held to
   be compatible. Each
   decision carries its own reasoning and the alternatives it rejected; where
   the reasoning explains why a requirement reads as it does, it is preserved in
   the `Rationale`, the `Rejected`, or the `Accepted cost` note under that
   requirement.
2. The root `README.md` and `CLAUDE.md`, used only where no decision contradicts
   them. Such requirements carry a provenance note.
3. A published external authority, cited by name and by date. The fourth
   edition introduced the first: `FR-SRV-015` derives its four series from
   MariaDB's own maintenance policy, and records the source and the date it was
   verified beside the table it produced. A requirement of this kind states its
   source, states when it was checked, and names what obliges a maintainer to
   check it again — `FR-SRV-019` for this one. It is not a decision this project
   is free to make, and it decays on a schedule this project does not set.
4. A **direct observation** against a running server or a measured artefact,
   dated, and stating what was observed and on which series. The sixth edition
   introduced the first of these: every requirement it adds carries an
   *Observed* note naming what was seen on each of the four series of
   `FR-SRV-015`, or on the two servers a TLS mode was verified against. The
   seventh edition rests on this provenance more heavily than any other —
   nineteen of its requirements are records of what four servers returned —
   and it adds a discipline the sixth did not need: where a fixture exercised
   only part of a field's population, the requirement carries a **bounded
   claim** naming what was not observed, so that a later reader can tell a
   settled fact from a fact that merely has not been contradicted yet. Such a
   requirement records behaviour rather than choosing it, and it is falsifiable
   by a second observation in a way a decision is not. Behaviour that has not
   been observed is never written down, however confidently it could be
   inferred from MySQL or from documentation.
5. Nothing else. Where information is missing, this specification records an
   entry in [open-questions.md](open-questions.md) rather than filling the
   gap. Where it is not merely missing but unobtainable, the entry closes on
   a **stated limit** written into the requirement, in the form the writing
   conventions above fix — never on an inference standing in for the
   observation that cannot be made.

## Maintenance debt

**None outstanding.** The item this section carried through the sixth edition
— twenty catalogue field lists that no observation had recorded — was
discharged by the seventh: the fixture was read against all four series of
`FR-SRV-015` on 2026-09-10, every field list was recorded verbatim, and each
is now a requirement in
[catalogue-coverage.md](catalogue-coverage.md) or
[context-document.md](context-document.md). The document shapes and the field
lists are frozen, and implementation is no longer blocked on them.

[open-questions.md](open-questions.md) has an empty index: no entry of this
corpus is open. The last, `OQ-042`, is closed and was never debt of this
corpus — it waited on evidence no fixture of MariaDB servers can produce, and
it is very likely unobtainable at all, so the limit is written into
`FR-SRV-041` instead. Nothing in this specification claims a detection it
does not have: that requirement states, in its own text, that a server
determined to pass as MariaDB will pass.

One obligation is not debt but recurs, and is recorded so that it is not
mistaken for either: `FR-SRV-019` requires the table of `FR-SRV-015` to be
re-verified against MariaDB's maintenance policy before every release. It
decays on a schedule this project does not set, and the earliest date on which
it is known to be wrong is 2028-02-16.

The eighth edition adds one obligation outside this corpus and none inside it.
`FR-CONF-038` requires the fixture of `scripts/mariadb/` to present, at each
series of `FR-SRV-015`, a server whose certificate names the host, without
which the default TLS mode of `FR-CONF-013` has no acceptance test. Like
`seed-bench.sql` under `DIV-036` it is fixture work with an owner and a
trigger, not an open question, and no requirement of this corpus is waiting on
it. `DIV-045`, recorded in the same edition, is a correction owed to the root
coordination document, which is what
[upstream-divergences.md](upstream-divergences.md) exists to hold.

Five items previously recorded here have been discharged.

- **The catalogue field lists.** Twenty entries, blocked first by the absence
  of a container and then by the absence of a recorded observation. Both are
  gone.

- **The first edition's open questions.** `OQ-002` through `OQ-024`, less
  `OQ-008`, `OQ-020`, and `OQ-022`, sat unanswered through the second and
  fourth editions. The fifth put every one of them to the user and closed all
  of them; `OQ-024` alone survived, narrowed twice more, because it is a
  catalogue field list that no decision could settle. The seventh observed it
  and closed it, in `FR-CTX-036`.
- **The wrong cross-reference targets.** Twenty-seven references, across
  twenty-four passages, resolved to an identifier that exists but was not the
  intended one; the offsets were small and consistent, which pointed at a late
  renumbering rather than at independent mistakes. All have been corrected.
  The lesson is recorded in the validation rule below.
- **The validation order and `--help`.** Settled by the third edition:
  `FR-PROJ-025` names the four commands that require no project, and
  `NFR-PERF-005` states the observable consequence.
- **The specified surface without specified documents.** `--format json` was
  contract in name only. Settled by the third edition: `FR-OUT-024` fixes one
  envelope for all seventeen documents.

A reference check must verify the **target** of a cross-reference, not merely
that the identifier exists. The first edition was validated as having no dead
cross-references, and that statement was accurate as measured and insufficient.
The fifth edition found the rule earning its place twice: `FR-SRV-025` cited
`FR-CAT-025`, a closing rule rather than the list it names, and `FR-ENV-044`
would have cited `FR-SEM-008` for a filter that rule did not cover. Both
identifiers existed; neither reference was correct.

The seventh edition adds a second rule of the same shape, learned from four
requirements the observation contradicted. **A requirement that names a
catalogue field, a field's value, or a field's shape must cite the
observation that established it, and a requirement written before any
observation existed must be re-read against the first one that reaches it.**
`FR-CTX-012`, `FR-SCH-009`, `FR-CAT-035` and `NFR-DET-002` were each
internally coherent, cross-referenced correctly, and describing a catalogue
that does not exist. Nothing in a reference check could have found them; only
reading them against the evidence could.
