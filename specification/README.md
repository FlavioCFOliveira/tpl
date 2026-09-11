---
title: tpl Functional Specification
status: approved
last-reviewed: 2026-09-11
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

The specification has been written in twelve editions. All are in force; each
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
  seventh edition took the record to eleven and the register to two rows, the
  tenth took the record to twelve, and the twelfth to thirteen.
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
decision. The ninth edition closes it, and closes it narrower than either
option this edition named: the requirement is amended to state an outcome
rather than a mechanism, the profile stands, and `DIV-045` is discharged.

**No open question is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty. Each resolution above
rests on an invariant this corpus already held, and where a choice remained
the requirement records the option rejected and why.

### Ninth edition — four defects the technical design found in the wording

Settling the project's technical specification read this corpus against the
design that has to satisfy it, and found four places where the wording says
something no implementation can do, or says nothing where an implementation
needs a referent. **None of the four changes what `tpl` does**:
three correct wording that had drifted from the behaviour the corpus intends,
and the fourth writes down a mapping the corpus always implied and had never
stated.

- **A panic is reported, not caught** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-030` made "a
  panic **caught** at the top level of the process" a producing condition of
  `70`, which obliges the process to resume execution above the panic site. The
  release profile recorded outside this corpus terminates the process on a
  panic, so nothing is caught and the condition could not exist in the
  distributed binary — the defect `DIV-045` recorded. The requirement now
  states the outcome instead: a panic is reported in the four labelled lines of
  `FR-ERR-008` and the process exits `70`, which a process can do from the
  panic site itself, before the panic ends it. Both producing conditions
  survive, the code table of `FR-ERR-001` is unchanged, and the mechanism is an
  architecture decision this corpus does not name. `FR-ERR-034`'s `70` row
  loses the same word.
- **`DIV-045` is discharged, and its premise was too strong** —
  [upstream-divergences.md](upstream-divergences.md). The entry stated without
  qualification that under an aborting profile the caller receives no code of
  `FR-ERR-001` and no message. That is the **default** behaviour of such a
  profile, not the whole of what it admits: a process that handles the panic
  itself reports it and exits with the status it chooses. The clause is
  corrected, and with `FR-ERR-030` amended the two statements no longer
  conflict, so nothing is owed to the root coordination document under this
  entry.
- **Six phases, four keys** —
  [configuration-model.md](configuration-model.md). `FR-CONF-005` requires a
  deadline on six phases while `FR-CONF-002` declares four timeout keys, so
  `FR-CONF-004`'s "the `[core]` key for that phase" had no referent for DNS
  resolution, TCP connect and TLS handshake. `FR-CONF-005` now maps every phase
  onto its key and states that the three connection phases share **one** budget
  of `core.connect_timeout` rather than taking one each. That is the reading
  the key's name already carried, and the only one under which the configured
  value bounds the thing it is named after; a deadline expiring inside the
  shared budget is still reported against the phase in progress, per
  `FR-ERR-034`.
- **A tab in a format that admits none** —
  [output-formats.md](output-formats.md). `FR-OUT-018` excepted tab from C0
  escaping across `text` and `json` alike, and no JSON document can honour
  that: the format admits no raw control character inside a string, so a
  document carrying one would not be JSON, which `FR-OUT-024` makes contract.
  The exception is now confined to `text`, where the aligned columns of
  `FR-OUT-006` need it, and the `json` path states what satisfies the rule
  there — the escape the format defines. `FR-SEC-020`'s summary table is split
  the same way. No emitted byte changes.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Tenth edition — the read-back names its variable

Reading this corpus once more before any code is written against it reached a
statement it requires on **every** connection and describes without naming.
The fourth entry of the closed list of `FR-SRV-006` read "one read of the
session read-only state", and the session read-only state has two spellings on
MariaDB. They are not interchangeable across the window: `10.11` has
`tx_read_only` and does not have `transaction_read_only`, which it refuses
with `ERROR 1193 (HY000)`. The other three series have both. An implementer
who chose the longer spelling would be told it works by three servers out of
four and would fail outright on the fourth, at connection start, on the
statement that confirms the strongest guarantee this tool makes.

**Nothing is decided here; a reading is written down.** The four containers of
`scripts/mariadb/` were run again on 2026-09-11 and each spelling was read in
its own statement on each series, so that a failure of one could not mask the
other. **No requirement changes what `tpl` does**: two name the variable they
already commanded, one verification is bound to the servers that can falsify
it, and one difference joins the observation record.

- **The variable is named, in both places that command it** —
  [server-contract.md](server-contract.md). The fourth entry of `FR-SRV-006`
  now reads `@@session.tx_read_only`, and `FR-SRV-009` names the same variable
  and forbids the other spelling. The evidence, its bound, and the rejected
  alternatives sit under `FR-SRV-006`: selecting the spelling per series, which
  `FR-SRV-037` licenses for a catalogue column list, is refused because an
  entry of a closed list that takes a different form per server is not one
  entry, and because the two spellings agree wherever both exist, so the
  spelling every series has costs nothing to prefer.
- **The verification is bound to every series** — `FR-SRV-013`. The test that
  exercises the read-back named no server, and exactly one series of the window
  discriminates the two spellings, so a test that runs anywhere else passes
  under either. It now runs against every series of `FR-SRV-015`, in the words
  `FR-SRV-029` already used.
- **The record of differences grows from eleven to twelve** — `FR-SRV-038`.
  The presence of `transaction_read_only` is difference 12. It reaches no
  field of the model, so the register of `FR-SRV-036` keeps its two rows; and
  it does not engage `FR-SRV-037`, whose subject is the column list of a
  catalogue read. A note beside the table says why, and why widening
  `FR-SRV-037` to cover it was rejected, so that the next reader does not
  re-open it. `FR-CAT-029`'s count follows it, and that list is still empty.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Eleventh edition — the verification names its platforms

`NFR-PERF-018` makes all four build targets first class, and one of the
verifications this corpus requires could not be performed on two of them. The
gap is not in what `tpl` does: it is in the evidence. `NFR-PERF-005` promises
that `tpl init`, every form of `help` and every form of `version` perform no
discovery, read no configuration file and open no connection, and
`NFR-PERF-007` requires each such promise to be verified from **outside** the
process and never by reading the source. Two of those three clauses rest on
seeing the files the process opens, and on macOS nothing can see them: `strace`
does not exist there, `dtruss` is refused while `csrutil` reports System
Integrity Protection enabled, and `fs_usage` asks for a password. The
observation was made on 2026-09-11, on Darwin, and it establishes an absence.

Until now the consequence went unstated, and the observation was in practice
made against the Linux build inside a container — an artefact no macOS user
runs. **No requirement changes what `tpl` does.** One requirement now says on
which targets its observation is made and on which it is not, one names its
instruments and binds each to the targets it exists on, one has its parity
clause reconciled with the gap rather than left to contradict it, and one rule
in another file is checked against the same gap and recorded as unaffected.

- **The file-open observation is Linux-only, and says so** —
  [performance-requirements.md](performance-requirements.md). `NFR-PERF-005`
  carries the limit in its own text: the connection clause is verified on all
  four targets from the server side; the discovery and configuration clauses
  are verified on all four by a differential run; and the file-open observation
  is made on the two Linux targets only, and may not be inferred from a Linux
  build observed in a container. The *Observed* note names `strace`, `dtruss`
  with `csrutil`, and `fs_usage`, so the constraint is not rediscovered; the
  *Consequence* note says what the two macOS targets therefore do not catch —
  a read whose result is discarded; and the *What would change this* note names
  the two things that would lift the limit.
- **The instruments are named and bound to their targets** — `NFR-PERF-007`.
  The rule listed three instruments in a parenthesis and bound none to a
  platform, which read as a promise that all three exist everywhere. It now
  lists four in a table with the targets each is used on, and adds the
  **differential run** — an invocation made in a state the operation under test
  would not have survived, compared against one that has nothing for it to
  find. It is outside the process, needs no privilege on any platform, and is
  available on all four targets, so the two macOS targets are not left with
  nothing. Where both it and the trace exist, the trace establishes the clause
  and the differential run corroborates it.
- **Parity is reconciled, not weakened** — `NFR-PERF-018`. The rule's parity
  clause is about **results**, and a note now says so: it never claimed every
  instrument exists on every target. Precisely because no target is second
  class, the Linux observation may not be credited to macOS, which is what
  `NFR-PERF-005` now forbids. Every requirement of that file is still verified
  on all four targets; two clauses of one are verified on two of them by a
  weaker instrument, and that requirement says which and why.
- **`BR-SRV-003` is checked and is unaffected** —
  [server-contract.md](server-contract.md). Both instruments that rule relies
  on are server-side, and `FR-SRV-012` through `FR-SRV-014` are promises about
  statements and connections, not about files. A note records the check so that
  the next reader does not repeat it. The rule and the three requirements are
  unchanged.

Three outcomes were open and one was taken. Moving the verification inside the
process was rejected against `NFR-PERF-007`: the eighth edition's licence for
`FR-ERR-031` to do exactly that opened only because every outside mechanism
collided with a requirement in force, and here two remain. A privileged path on
macOS was rejected as outside this specification's authority — disabling System
Integrity Protection is a configuration of a contributor's machine and a
password prompt cannot be automated — and it is recorded in *What would change
this* as a decision the project may still take, rather than one taken here.

**One editorial correction.** `DIV-031` said the closed statement list of
`FR-SRV-006` has **three** entries, which was true when the second edition
wrote it and has not been since: the fifth edition added the read-back as a
fourth entry and the tenth named the variable it reads. The table carries four
rows, so the clause was wrong and is corrected. What that entry owes the root
`CLAUDE.md` is unchanged — the removal of the `SHOW` clause — and the entry
says so in its own note.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Twelfth edition — the announced version is not the probed one

A MariaDB server states its version twice: once when the connection opens,
unasked, and once when the probe of `FR-SRV-002` asks for it. On three of the
four supported series the two strings are identical. On `10.11` they are not —
the announcement carries a `5.5.5-` prefix and the probe's answer does not —
and the fixture of `scripts/mariadb/` recorded that while it was being built.
`FR-SRV-038` obliges **every** difference observed between the series to be
recorded, whether or not it reaches the model, and this one was not. This
edition records it.

**Nothing here changes what `tpl` does, and no requirement is amended.** The
two requirements the difference bears on were checked and both stand as
written; each now carries a note saying it was checked, so that the next reader
does not check it again.

- **The record of differences grows from twelve to thirteen** —
  [server-contract.md](server-contract.md). Difference 13 states both readings
  of the version on each of the four series, verbatim, with the date and the
  instrument: the announcement was read over a plain socket by the fixture's
  own gate, which opens the connection and closes without authenticating. The
  claim is bounded — one server of each series, at the four patch releases
  `FR-SRV-040` records — and the fifth listener of the fixture, the same
  `10.11` image started without TLS, announced the same prefixed string, so the
  prefix does not depend on TLS being in force.
- **The register of `FR-SRV-036` keeps its two rows, and the reason is written
  rather than left implied** — `FR-SRV-038`. The announcement reaches no field
  of the model, because `FR-CTX-031` carries the string the probe returns and
  `FR-SRV-040` derives `series` from that string and from nothing else. The
  reason is **not** difference 12's reason, and the record says so: difference
  12 is a session variable the model could not carry whatever `tpl` did with
  it, while this one touches a field the model does carry by a reading the
  model does not take. The register records what the model accommodates; this
  is a difference the model avoids, by naming its reading.
- **`FR-SRV-040` is checked and unchanged** — the clause *and from nothing
  else* already bars the announcement, and the note records what would follow
  if it did not: `<major>.<minor>` taken from the announcement reads `5.5` on
  `10.11`, and `FR-SRV-020` would refuse a supported server as older than the
  window.
- **`FR-SRV-041` is checked and unchanged** — the marker `MariaDB` is carried
  by both readings on all four series, so the necessary condition returns the
  same verdict from either, and no server is admitted or refused by the
  difference. The limit that requirement states covers the announcement too:
  it is one more response the server composes.
- **`FR-CAT-029`'s count follows and its list stays empty** —
  [catalogue-coverage.md](catalogue-coverage.md). The announcement is not a
  catalogue field, so it cannot be a field whose meaning two series disagree
  about.

**One defect was found and is not corrected here.** Recomputing the tally
beside the note on difference 8 showed the claim that tally supports to be
wrong: difference 2 splits `10.11` and `11.4` from `11.8` and `12.3` exactly as
difference 8 does, so the record holds two such splits and not one. The tally
now follows the record of thirteen; the claim it glosses predates this edition,
is not what this edition was opened to settle, and is recorded under
*[Maintenance debt](#maintenance-debt)* for a correction of its own.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

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
- **Where a verification stops, rather than a guarantee, the same note shapes
  are used and the three-way family stays three.** The family above is limits
  on what `tpl` guarantees about a server, a table, or a trust store, and its
  members cite each other. A limit on the **evidence** for a guarantee — the
  observation that cannot be made, rather than the promise that cannot be kept
  — takes the same note shapes, as many of them as it has content for, without
  joining that family, because it qualifies a different kind of claim. Two
  requirements are written this way:
  `FR-ERR-031`, where no invocation of the distributed binary is observed
  returning `70`, and `NFR-PERF-005`, where the file-open observation is made
  on the Linux targets and not on the macOS ones.

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

**One item is outstanding**, recorded by the twelfth edition and stated below.
The item this section carried through the sixth edition
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
it. `DIV-045`, recorded in the same edition, was a correction owed to the root
coordination document; the ninth edition discharges it, because `FR-ERR-030` as
amended and the profile that document states no longer contradict each other.

The ninth edition adds no obligation of either kind. Its four changes are
corrections of wording, each stated beside the requirement it changed.

The tenth edition adds no obligation of either kind either. It names a variable
that two requirements had left to the implementer, records a twelfth difference
between the series, and widens one verification that was already owed —
`FR-SRV-013`, which like every test this corpus mandates is blocked only by
`tpl` not existing.

The eleventh edition adds no obligation of either kind. It records a limit on
where one observation can be made, adds an instrument that closes the resulting
gap on every target, and leaves the requirement it qualifies saying what that
instrument does not establish. One obligation outside this corpus is narrowed
rather than created: the file-open observation of `NFR-PERF-005` is owed on the
two Linux targets and is owed on neither macOS target, so a verification suite
that skips it there is conforming and not incomplete.

The twelfth edition adds no obligation outside this corpus and one inside it.
It records a difference `FR-SRV-038` already obliged this corpus to hold,
checks the two requirements that difference bears on and leaves both as
written, and adds no row to the register of `FR-SRV-036`. The obligation is the
defect it found while recomputing a tally and did not correct.

**Outstanding: the note on difference 8 of `FR-SRV-038` overstates itself** —
[server-contract.md](server-contract.md), *Differences observed between the
series*. The note calls difference 8 the only split in the record that does not
fall after `10.11`. Difference 2 has the same shape — `10.11` and `11.4` report
the position of `INVISIBLE` one way and `11.8` and `12.3` the other — so there
are two such splits and not one. The tally beside the claim was corrected to
follow the record of thirteen; the claim itself was left, because it predates
the edition that found it. What is owed is a correction to that note. Nothing
about `tpl` turns on it: no requirement rests on the claim, and the caution the
note exists to give — that a difference may fall anywhere in the window — is
strengthened by the second example rather than weakened.

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
