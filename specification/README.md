---
title: tpl Functional Specification
status: approved
last-reviewed: 2026-09-24
related: [glossary.md, cli-contract.md, catalogue-coverage.md, open-questions.md, upstream-divergences.md]
---

# tpl Functional Specification

This folder is the single source of functional truth for `tpl`. It is independent
and self-sufficient: no functional requirement lives outside it, and nothing here
defers to another document for its meaning.

`README.md` at the repository root is the entry door to the repository.
`CLAUDE.md` at the repository root is agent coordination. Neither carries
functional requirements. Where either still repeats functional content that this
specification now owns, contradicts it, or states as present something the
repository does not contain, the divergence is recorded in
[upstream-divergences.md](upstream-divergences.md) and the passage must be
corrected in those files. That register is re-read against both files whenever
either of them changes, and whenever this corpus amends a requirement one of
them paraphrases — for entries the files have discharged and for divergences it
does not yet hold — and every entry records whether the correction is still
owed and, where it is not, the commit that discharged it.

## Scope

The specification has been written in fifty-six editions. All are in force;
each adds to the ones before it and amends them in place, and every
amendment carries an *Amended in the nth edition* note beside the
requirement it changes.

*Corrected in the thirty-fourth edition: the count was one behind.* The line
read *thirty-two editions* while the thirty-third was written, recorded below,
and in force. It is the third time this sentence has been the thing an edition
forgot: the twenty-third left it at twenty-two, the thirty-first at thirty, and
the thirty-third at thirty-two, each corrected by the edition after it rather
than by the one that moved past it. Whoever opens an edition changes this
number in the same pass, and the check is the cheapest in this file — the
number here equals the number of edition sections below it.

*Corrected again in the forty-second edition.* The line read *thirty-nine*
while the fortieth and forty-first editions were written, recorded below, and
in force.

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
  tenth took the record to twelve, and the twelfth to thirteen; the thirteenth
  took it to fourteen, with a difference observed before any of them. The
  eighteenth added no row and records three readings of the build below the
  table, where an observation that differs across the window without being a
  difference between the series belongs.
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
them.** The field lists of every object kind **an entry had asked for** are
fixed — the twenty-third edition corrects this sentence, which read *of every
object kind* and passed over the table object, for which no entry had
asked: comments and
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
and `11.4` from `11.8` and `12.3`. It is the second difference in the record
to split the window in the middle rather than at one of its ends — difference
2 was already there — and it is recorded as a caution that a difference may
fall anywhere in the window. This edition wrote it as the only one; the
twelfth corrects it. The divergence register of `FR-SRV-036`, empty through
six editions, gains its first two rows — both created by `FR-SRV-039`.

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
  provided none, so the default of `FR-CONF-013` was the one cell of a
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
  under either. It was bound to every series of `FR-SRV-015`, in the words
  `FR-SRV-029` already used.

  *Corrected in the twenty-eighth edition.* The clause read that the test
  "now runs against every series", in the present tense and over the whole of
  `FR-SRV-013`. The twenty-eighth edition splits that requirement by the test
  form that can reach each of its two outcomes, and the binding attaches to the
  half that reaches a server: the confirming outcome runs against every series,
  and the failing outcome runs against none, in process. The binding is
  undiminished — what changed is which half carries it.
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

**One defect was found while this edition ran, and is corrected in it.**
Recomputing the tally beside the note on difference 8 showed the claim that
tally supports to be wrong: difference 2 splits `10.11` and `11.4` from `11.8`
and `12.3` exactly as difference 8 does, so the record holds two such splits
and not one. The tally was corrected with the record of thirteen and the claim
was left standing, because it predates this edition and was not what this
edition was opened to settle; it was recorded under
*[Maintenance debt](#maintenance-debt)* for a correction of its own, and that
correction is the editorial one below.

**One editorial correction, made after the record of thirteen was written.**
The note beside difference 8 called it *the only split in this record that
does not fall after `10.11`*. Two of the thirteen split the window in the
middle — difference 2 as well as difference 8 — and the note now says so,
counts the thirteen three ways, and names the differences in each count. A
second count in the same note was wrong for a reason of its own: code that
models the four series as one old server and three modern ones is right for
nine of the thirteen and not for eleven, because the two splits that isolate
`12.3` defeat that model as surely as the two that fall in the middle. The
caution the note exists to give — a difference may fall anywhere in the
window — is unchanged and is stronger for the second example. The seventh
edition's account of the same note, above, is corrected with it. No
requirement is amended, no row of the record changes, and the register of
`FR-SRV-036` keeps its two rows. Recorded under this edition rather than
opening a thirteenth: a gloss on an observation binds nothing, no requirement
rests on the claim, and nothing here is observed, decided, or amended. The
eleventh edition settled the same case the same way for `DIV-031`'s count.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Thirteenth edition — a record assembled from its own table

`FR-SRV-038` obliges every difference observed between the series to be
recorded in its own section, whether or not it reaches the model. A fourteenth
difference was observed on 2026-09-10, while the shared DDL of
`scripts/mariadb/` was being settled, and was written down beside `FR-SRV-029`,
whose fixture consequence it decided. It never reached the record. Two
recounts — the tenth edition's and the twelfth's — passed over it, because each
was computed over the rows the record already held rather than over everything
the corpus had observed. This edition records the difference, and names where
the method failed.

**Nothing here changes what `tpl` does, and no requirement is amended.**

- **The record of differences grows from thirteen to fourteen** —
  [server-contract.md](server-contract.md). Difference 14 is the refusal of a
  `VECTOR` column or a vector index at DDL time by `10.11` and by `11.4`, with
  `ERROR 4161 (HY000)`, and its acceptance by `11.8` and `12.3`. It is the only
  difference in the record that is about what a server **accepts** rather than
  what it reports. The claim is bounded to acceptance: nothing was observed
  about what the catalogue reports for such an object on the two series that
  accept one, because the shared DDL declares none for any series to report.
- **The counts beside difference 8 are recomputed over fourteen rows, and the
  twelfth edition's classification of the thirteen was checked against the rows
  rather than taken on trust.** It holds — nine differences isolate `10.11`,
  two isolate `12.3`, and two split the window in the middle. Difference 14 has
  the middle shape, so that count goes from two to three and the closing one
  from four differences to five, 2, 6, 8, 9 and 14; the other two counts do not
  move, which is the check that the recount was made over the rows.
- **The register of `FR-SRV-036` keeps its two rows, and the reason is written
  rather than left implied** — `FR-SRV-038`. A row of that register carries
  three columns — the field, the treatment, and what each series returned — and
  this difference fills none of them: it was observed in a server's answer to a
  `CREATE`, no field of the model is in question, and no catalogue reading was
  taken for a `VECTOR` object on any series. The reason is neither difference
  12's nor difference 13's; it is upstream of both, in which databases can
  exist on each series at all.
- **The method is named where the record states it** — `FR-SRV-038`. *Method
  and date* now counts five observation occasions rather than four passes, and
  says why no pass could have found this one: a pass compares what running
  servers report, and this difference is in what a server accepts, which is
  settled before any comparison can be made. The Scope of
  [server-contract.md](server-contract.md) follows it.
- **`FR-CAT-029`'s count follows and its list stays empty** —
  [catalogue-coverage.md](catalogue-coverage.md). A difference in what a server
  accepts is not a field whose meaning two series disagree about.
- **`FR-ENV-046` cites the record and not the fixture consequence alone** —
  [template-environment.md](template-environment.md). Its bounded claim already
  named `VECTOR` as a type a supported server can hold and the fixture's 39
  `data_type` values do not cover; it now cites the difference that establishes
  it.

**One defect of wording was corrected in passing.** *Method and date* said
*Three passes were made* and then described four: the twelfth edition added the
fourth pass and left the count behind it. The paragraph now states five
occasions and enumerates five.

**One defect was found by the sweep this edition ran and is not corrected in
it.** Two further readings that differ across the window — the build's source
revision and the SSL library string — are recorded under `FR-SRV-040` and
classified there as properties of the build. `FR-SRV-038` then offered an
observation two homes and no third: a row in its table if it is a difference
between the series, or a line below the table if it was observed to differ
between two servers of the same series, so that it is not counted as one.
Neither reading had either. It is recorded under
*[Maintenance debt](#maintenance-debt)* for a correction of its own, because
the correction is a decision rather than an editorial tidy — the timestamps
recorded below that table were observed to differ between two servers of the
**same** series, and the SSL library string was not. The eighteenth edition
settles it, on a rule in force rather than on a decision from outside this
corpus, and finds a third reading of the same shape that this edition's sweep
did not name.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Fourteenth edition — an observation of servers nobody had configured

`FR-SRV-038` records what four servers were observed to report, and one of its
fourteen rows recorded a reading taken before this project's own fixture
changed the thing it read. `have_ssl` is `DISABLED` on `10.11` until an
administrator configures a certificate; `FR-CONF-038` has said so since the
eighth edition, and it now obliges the fixture to configure one at every
series. The row said `DISABLED` and said nothing else, so an observation record
contradicted the fixture the observation was taken from. This edition puts the
condition in the row and checks the other thirteen rows for the same defect.

**Nothing here changes what `tpl` does, and no requirement is amended.**

- **Difference 3 names the condition it was read under** —
  [server-contract.md](server-contract.md). The row now reads `have_ssl`, on a
  server left to itself, with no TLS material configured for it, and cites
  `FR-CONF-038` for that condition and for the fixture obligation that lifts
  it, rather than restating either. The difference itself is unchanged:
  `10.11` offers no TLS until something configures it, and that is the whole
  of what makes it a difference between the series. The row also gains a
  bounded claim, in the form differences 13 and 14 already carry.
- **The other thirteen rows are checked against the same defect, and none has
  it** — `FR-SRV-038`. The check reads each row against what the fixture
  configures rather than against the other rows: three TLS settings in one
  file, two initialisation scripts, a root password and a published port, and
  no other server variable on any series. Nine rows are properties of the
  server build, two are the servers' own default collation, which the fixture
  leaves untouched and which is visible for that reason, one is a statistic
  over identical data and already states that condition, and one is not a
  reading at all. Only `have_ssl` is a value the fixture sets. The negative is
  recorded with its method, because a sweep that finds nothing is worth only
  as much as the reader can see of how far it went.
- **The record says which of its occasions the fixture had TLS material for**
  — `FR-SRV-038`, *Method and date*. The first two passes ran before it and
  the fourth after it, and difference 13's bounded claim had already recorded
  the later state. One condition of observation was therefore not constant
  across the five occasions, and the record now says so instead of leaving
  each row to be read as though it were.
- **`FR-CONF-038`'s consequence note says when it was true** —
  [configuration-model.md](configuration-model.md). It stated in the present
  tense that `tpl` with default configuration cannot reach the fixture over
  TCP on any series, which the fixture's own certificate has since made false.
  The observation and its date are unchanged; the note now bounds the claim to
  the fixture as it then stood and records the discharge. The note that states
  the condition cites difference 3 in return, so the two point at each other.
- **The eighth edition's fixture obligation is recorded as discharged** —
  *[Maintenance debt](#maintenance-debt)*. It was the one obligation that
  edition placed outside this corpus, and `scripts/mariadb/` now carries the
  material it asked for.

**One editorial correction.** The eighth edition's entry above said *the
fixture provides none*, in the present tense, inside a narrative of what that
edition found; it now says *provided*, which is what it meant.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Fifteenth edition — a register nobody re-read

[upstream-divergences.md](upstream-divergences.md) recorded forty-five
corrections owed to the root `README.md` and the root `CLAUDE.md`, and no
edition had re-read it against those files since the files changed. `DIV-031`
asks for the removal of the clause that admitted `SHOW` as a way to read the
catalogue — "e, quando estritamente necessário, `SHOW`" — which `0ea5624`
removed several sprints before the
eleventh edition amended that very entry for a miscount. The amendment
recounted the list and did not check whether the sentence it corrected was
still in the file. One entry reclaiming work already done destroys the
presumption that the other forty-four are current, so this edition classifies
every one of them and writes down the obligation that was missing.

**Nothing here changes what `tpl` does, and no requirement is amended.** No
root document is edited either: this corpus does not hold the pen on those two
files, and discharging an entry by making its correction is not what this
edition did.

- **Every entry carries a Status, and the index carries a Status column** —
  [upstream-divergences.md](upstream-divergences.md). Each of the forty-five
  was classified against the current text of the file its **Target** names, as
  that file stood at `87dd6e3`: **ten were due in full, fourteen discharged,
  and twenty-one partly discharged**. Thirty-one entries still owed something,
  twenty-eight of them to `README.md`. The sixteenth edition adds four entries
  and those figures are its own; the register's own count is in its Overview.
- **A discharged entry names the commit that discharged it.** Twelve were
  discharged outright by `0ea5624`, which reduced `CLAUDE.md` to agent
  coordination, and that same commit discharged one half of nineteen more; one
  by `26e1739`, one by `011c059` and one by `87dd6e3`, each in part. An entry
  that says when it stopped being owed can be audited; an entry that merely
  says it is not owed has to be re-derived by the next reader.
- **A discharged entry is kept in place, with its identifier** —
  [upstream-divergences.md](upstream-divergences.md). This is the rule the
  *[Identifier scheme](#identifier-scheme)* above states for a withdrawn
  requirement, and `DIV-045` has carried the in-place form since the ninth
  edition. The alternative considered and rejected was the *Closed* table of
  [open-questions.md](open-questions.md): entries of the register cite each
  other, and a citation whose target has been moved to a row resolves to a row
  rather than to the reasoning.
- **The partly discharged case is given a treatment, because it is the
  ordinary case and not the exception.** Nineteen entries name both files, and
  the two are edited under separate authorisations, so such an entry is
  discharged in halves. **No entry is recorded as discharged while any part of
  it stands**, and a partial status names, part by part, what was discharged
  and by which commit. `DIV-018` is the first to carry it: `26e1739` replaced
  the three TLS modes in the flag row of `README.md` and the `.cfg` example in
  the same file still shows neither `ca_file` nor `ca_path`.
- **The index and one entry are brought back into agreement** —
  `DIV-013`. The index said `README.md` and the entry said `both`, because the
  fifth edition discharged the `CLAUDE.md` half and narrowed the index row
  without narrowing the field. The field is authoritative and the row now
  matches it; what is owed, and to which file, is the Status column's job.
- **One entry is discharged that no commit discharged, and the finding is
  recorded rather than deleted** — `DIV-025`. Its *Says* clause quotes a
  sentence saying `/specification` does not yet exist. Searching the history
  for those words finds one occurrence in the whole repository, in the register
  itself, at `1352a2d`. The subsection it asks to be removed was never in
  `CLAUDE.md`. An entry raised against an unverified reading of a target is the
  same defect as an entry left standing after the reading went stale, and the
  identifier must resolve to that explanation.
- **Two statements about `CLAUDE.md` made outside the register are corrected
  with it** — [performance-requirements.md](performance-requirements.md). The
  provenance note of `NFR-PERF-018` said that file's matrix *still names* the
  `gnu` triples; it named them from `a8c5390` until `0ea5624` removed the
  matrix, and what stands there now is the deferral `DIV-041` quotes. The
  provenance note of `NFR-PERF-014` said a correction is owed to that file
  under `DIV-035`; `0ea5624` removed the budget table and the entry is
  discharged. Both notes keep what they record — where five provisional
  figures and four targets came from — and lose only the present-tense claim
  about a file that has moved on. A register is not the only place this corpus
  says something about a document it does not own, which is why the fifth rule
  is written over registers **and** over any such statement.

**One observation is recorded in passing and owes nothing.** `DIV-045`, which
the ninth edition discharged by amendment, quotes a release-profile table that
`50153d6` has since reduced to a citation of `ADR-004`. Its *Says* clause no
longer matches the file, which changes nothing about an entry that owes
nothing. Every discharged entry now carries a quotation that outlived its
source; this one is worth naming because the entry had already been closed, so
nothing would have brought a reader back to it.

**The rule this edition adds is the fifth of its kind**, and it is stated under
*[Maintenance debt](#maintenance-debt)* with the other four. A register is not
a requirement: it describes a file this corpus does not own, so it decays
whenever somebody else edits that file, and no reference check, recount or
sweep of this corpus can see it happen.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Sixteenth edition — the entries a swept register did not hold

The fifteenth edition re-read every entry of
[upstream-divergences.md](upstream-divergences.md) against the two root
documents and classified all forty-five. It did not ask whether forty-five was
all of them. A reading of the root `README.md` against this corpus, made for
another purpose, found four divergences that no entry asks for, and the first
of them is the first instruction in that file a reader acts on. This edition
records the four, and two more that the readings which followed found in
`CLAUDE.md` — six in all — and writes down the check that would have found
them.

**Nothing here changes what `tpl` does, and no requirement is amended.** No
root document is edited either, for the reason the fifteenth edition gave: this
corpus does not hold the pen on those two files, and recording a correction is
not making it.

- **Seven entries are added, and all seven are due** — `DIV-046` through
  `DIV-052`, each checked against its target file at `87dd6e3`, which is still
  the last commit to touch either root document and so the same state the
  fifteenth edition classified the other forty-five against. The register now
  holds fifty-two entries: **seventeen due in full, fourteen discharged,
  twenty-one partly discharged**, and thirty-eight still owe something,
  thirty-two of them to `README.md` and seven to `CLAUDE.md`.
  - `DIV-046` — *Installation* tells a reader to run `cargo build --release`,
    and *Development* gives eight `cargo` commands, in a repository that has no
    `Cargo.toml`. The document-scope banner says no command described below is
    implemented yet; it does not say the crate does not exist, so a reader who
    takes the banner at its word expects the build to work and the commands to
    be unfinished.
  - `DIV-047` — the flag table of `render` omits `--direct` and `--no-cache`,
    which `FR-GLOB-021` declares on that command and `FR-GLOB-022` requires it
    to list.
  - `DIV-048` — the entry flag is spelled `--database`, where `FR-CFG-027`
    names it `--schema`. The name the file uses is the global flag of
    `FR-GLOB-001`, which `FR-GLOB-002` makes acceptable at that very node and
    `FR-GLOB-004` gives a different meaning, so the invocation the table
    teaches is accepted and writes the wrong key.
  - `DIV-049` — the same table omits `--ca-file` and `--ca-path`, which
    `FR-CFG-027` has declared since the fifth edition, while its `--tls` row
    already names the two modes that `FR-CONF-014` says the trust material
    serves.
  - `DIV-050` — the project-structure tree of `CLAUDE.md` states a crate and
    five directories the repository does not have, in a block that carries no
    tense. It is the first of the two entries raised against that file, and
    both had to separate what the document is entitled to say from the way it
    says it: the module decomposition under `src/` is where code will go and
    belongs in a coordination document, so what is owed is a qualifier on the
    tree and not its removal.
  - `DIV-051` — the *Desenvolvimento* section of `CLAUDE.md` gives three
    `cargo` commands and the mandatory validation pipeline under it five more,
    and gates the completion of all work on those five. Every one of the eight
    fails for want of a manifest. They are the same eight commands `DIV-046`
    records of the other file's *Development*, and `DIV-001` leaves the
    pipeline with `CLAUDE.md`, so what is owed is a qualifier on the two blocks
    and not the removal of a command.
  - `DIV-052` — the *Disciplina de medição* subsection of `CLAUDE.md` states
    that the benchmarks live in `benches/` and run against the containers'
    dataset, in a repository with no such directory and no benchmark of any
    kind. The measurement discipline stated beside it is in force and is
    stated correctly, so what is owed is a qualifier on one clause. It is the
    third of this edition's entries against that file, and the only one of the
    four that needs a second fact besides the missing crate: the fixture the
    sentence invokes is in part what `DIV-036` still owes.
- **The register gains a third kind of entry** —
  [upstream-divergences.md](upstream-divergences.md). **Contradiction** and
  **Migration** both describe the relation between a root document and this
  corpus. `DIV-046` describes the relation between a root document and the
  repository: it contradicts no requirement, nothing is owed to this folder,
  and acting on it fails anyway. **Overstatement** names that, and four
  entries carry it — `DIV-046` against the root `README.md`, and `DIV-050`,
  `DIV-051` and `DIV-052` against `CLAUDE.md`. Forcing them into
  **Contradiction** was rejected, because it would have obliged a
  *Specification* clause to cite a requirement that does not exist.
- **Each new entry names the existing entry it was checked against, where one
  looks as though it covers the ground.** `DIV-023` records the same two flags
  as missing from the **global** tables and replaces those tables with a
  pointer; applied exactly as written it leaves the `render` table silent,
  which is `DIV-047`'s subject. `DIV-003` names `--ca-file` and `--ca-path`,
  but only to say that neither carries a short form, and `DIV-018` asks for
  `ca_file` and `ca_path` in the `.cfg` example rather than in the flag table;
  neither reaches `DIV-049`. `DIV-032` and `DIV-036` both name the project
  structure of `CLAUDE.md` in their **Target**, and neither reaches `DIV-050`:
  the first is about the sentence below the tree, and the second asked for a
  line to be **added** to the tree and was discharged in that half by
  `87dd6e3`, which corrected the line it named and left standing the six that
  state a crate the repository has not got. `DIV-051` is checked against
  `DIV-050`, which is the entry that looks most like it — same file, same fact
  about the repository, same shape of correction — and against `DIV-046`, which
  records the same eight commands in the other root document; the reasons
  neither absorbs it are written under it. An entry that looks covered and is
  not is how these came to be missing, so the check is written into the entries
  rather than left to the next reader.

**The divergence that had no entry now has one.** The project-structure tree
of `CLAUDE.md` names `Cargo.toml`, `src/`, `tests/`, `benches/`, `templates/`
and `examples/`, and the repository has none of them — the same divergence as
`DIV-046`, in the other root document. It was first recorded in prose, here and
in `DIV-046`, because that entry's **Target** is `README.md` and an entry is
not widened to a file the survey that raised it did not read. `DIV-050` is the
entry it needed, raised against `CLAUDE.md`, and it is added to this edition
rather than opening a seventeenth. Nothing the corpus knows changes with it:
the finding, the kind it belongs to, and the reason it could not be folded into
`DIV-046` were all settled here, and what was outstanding was the entry alone.
An edition records what this corpus learned, and one whose whole content was
*write down what the edition before it had already found* would turn the
edition record into a log of working sessions.

**What the prose above had not settled is the correction**, and it is not a
deletion. The seven lines under `src/` are the project's module decomposition —
an architecture decision, recorded as one outside this corpus — and a document
whose job is to say how work is executed here is entitled to say where code
will live. What it may not do is say it in the indicative of the present, in a
block that carries no tense and sits in a file with no banner of any kind about
how much of the repository exists. So `DIV-050` asks for a qualifier on the
tree, and says in as many words that neither the tree nor the decomposition is
to be removed. An entry read as *delete the tree* would have the project strike
its own structure from the file every agent reads first.

**The second divergence this edition recorded rather than acted on now has an
entry too.** `DIV-050` named, and did not record, the eight `cargo` commands of
`CLAUDE.md`'s *Desenvolvimento* section and of the mandatory validation
pipeline under it: three in the first block, five in the second, and a sentence between
them making the completion of all work conditional on the five. Every one of
them fails for want of a manifest, which is what `DIV-046` records of
*Development* in the other root document. `DIV-051` is the entry they needed.

**It is a second entry against `CLAUDE.md` and not a widening of `DIV-050`**,
and the reason is the one `DIV-046` acted on rather than the one it appears to
have acted on. That entry covers two sections of `README.md` because one
sentence at the head of each discharges both, not because the two sit near each
other: they are about four hundred lines apart, at the head of the file and at
its foot. Here the two passages do not take one correction. `DIV-050` qualifies
a tree of paths and forbids the removal of the decomposition under `src/`;
`DIV-051` qualifies two blocks of commands and the obligation stated between
them, and an editor who applies the first exactly as written leaves the eight
commands untouched under a heading it does not name. The two passages are also
edited apart, which `87dd6e3` demonstrates — it rewrote the tree and left
*Desenvolvimento* alone — and **Target** is the authority on where an entry is
re-checked, so one entry over both would have gone to *partly discharged* on an
edit that did nothing for the commands. What the merge would have bought is one
item instead of two for whoever holds the pen on that file; the register's
index groups by **Target**, so that reader already has both in one place.

**The correction preserves every command.** `DIV-001` leaves the validation
pipeline with `CLAUDE.md` — it names the pipeline among the content that file
keeps — so the commands are that file's to state, they are the right commands,
and they run unchanged the moment the manifest exists. What is wrong is the
tense, exactly as in `DIV-050`, and an entry read as *delete the pipeline*
would strike the project's completion gate out of the file every agent reads
first. `DIV-051` therefore asks for a qualifier at the head of the section, or
a mark on the two blocks, and says in as many words that neither the commands
nor the obligation between them is to be removed.

**This entry is added to this edition rather than opening a seventeenth**, for
the reason `DIV-050` was. What was outstanding was the entry alone: the finding,
the kind it belongs to, the file it is owed to, and the fact that it needed an
entry of its own were all recorded here when this edition was written, and its
placement was settled by applying a rule this edition had already stated twice
rather than by learning anything new. An edition records what this corpus
learned; opening one whose whole content was *write down the second of the two
findings the edition before it had already made* would turn the edition record
into a log of working sessions, and doing it twice from one edition's backlog
would make the point twice over.

**The third divergence this edition recorded rather than acted on now has an
entry too.** `DIV-051` named, and did not record, the *Disciplina de medição*
subsection of `CLAUDE.md`, which states that the benchmarks live in `benches/`
and run against the dataset of the MariaDB containers. The repository has no
such directory and no benchmark of any kind, and `BENCHMARKS.md` does not stand
in for them: its two campaigns measured probe binaries this repository does not
hold, and it says of itself that no figure in it is a baseline for `tpl`.
`DIV-052` is the entry that sentence needed.

**It is the third of this edition's entries against `CLAUDE.md`, and the
criterion that placed it is the one `DIV-051` settled**: the shape of the
correction decides where an entry goes, not the proximity of the passages. Here that criterion cuts against
proximity rather than with it. *Disciplina de medição* sits some fifty lines
below the head of *Desenvolvimento*, in the very next top-level section, so
nearness argues for a merge as loudly as distance argued against one in
`DIV-046`; what decides is that no one correction serves both. A qualifier at
the head of *Desenvolvimento* does not reach a subsection of *Desempenho e
Eficiência*, and the two discharge on different conditions: `DIV-051`'s eight
commands run unchanged the moment the manifest exists, while this sentence does
not become true with a manifest, because a benchmark has to be written and what
it is said to run against is not complete. The passages are also edited apart,
which `87dd6e3` demonstrates for both — it rewrote the project tree and touched
neither.

**The correction preserves the measurement discipline.** No performance claim
without numbers, the baseline recorded in `BENCHMARKS.md` against a named
target, and a regression failing the change were obligations in force when this
was written, and they governed both campaigns that file already holds;
`NFR-PERF-009` through `NFR-PERF-013` and `NFR-PERF-017` carried the same rules
in this corpus, and `DIV-035` left the discipline with `CLAUDE.md` when it took
the figures out.
What is wrong is the assertion of state in front of it, so `DIV-052` asks for a
qualifier on one clause and says in as many words that neither the discipline
nor the location is to be removed. An entry read as *drop the benchmark rule*
would take the project's only statement of what fails a change out of the file
every agent reads first.

*Four verbs of the paragraph above are put in the past by the thirty-sixth
edition, and its last sentence is answered by it.* `BR-PERF-008` withdraws the
third of the three obligations — a regression failing the change — and
`NFR-PERF-013` and `NFR-PERF-017` are withdrawn with it, so this corpus no
longer carries that rule and `CLAUDE.md`'s two statements of it now contradict
it; `DIV-055` is the entry raised for them, and what it asks for is exactly the
removal this paragraph argued against. The other two obligations stand — no
claim without numbers, and a figure recorded against a named target — and so
does the sixteenth edition's reasoning about why `DIV-052` is an entry of its
own, which turns on the shape of a correction and not on what any rule
requires.

**The relation to `DIV-036` is written in both directions, and neither covers
the other.** That entry is partly discharged and still owes
`scripts/mariadb/seed-bench.sql` with the two lines that name it; `WL-001` is
realised by that file, and `BR-PERF-007` records that the budgets over it
cannot be measured until it exists. So the dataset the sentence promises
reproducibility against is in part what `DIV-036` owes. But `DIV-036` is a
**migration** whose correction is an addition, and applied exactly as written
it leaves the sentence standing; `DIV-052` is an **overstatement** whose
correction is a qualifier, and it produces no fixture.

**This entry is added to this edition rather than opening a seventeenth**, for
the reason `DIV-050` and `DIV-051` were, and the reason is stronger the third
time. What was outstanding was the entry alone: the finding, the kind, the file
it is owed to, the entries checked against it, and the fact that it needed an
entry of its own were all recorded here and in `DIV-051` when this edition was
written, and its placement was settled by applying a rule this edition had
already stated three times. An edition records what this corpus learned;
opening one to hold the last of three findings the edition before it had
already made would turn the edition record into a log of working sessions.

**Whether any passage of this class is left in `CLAUDE.md` was checked, and one
candidate stands.** The whole file was read against the question rather than
trusted to the previous reading, because a claim that a class is exhausted is
worth only the sweep behind it. Every other path, file and artefact that
document names exists, and its passages on `#![forbid(unsafe_code)]`,
`#![warn(missing_docs)]`, the release profile and the module conventions direct
how code is to be written rather than assert what the tree holds. One passage
is not settled by that reading: *Plataformas Suportadas* calls Linux and macOS
"suportados e verificados" and says of the other Unixes that they do not run in
validation, which presupposes that the four targets do, where the validation
that would do it is the pipeline `DIV-051` records as unable to run at all. It
is an adjective inside a declaration of policy rather than a block a reader
executes, and whether it asserts a state or names a class is a judgement for a
reading of that section, which `DIV-041` targets on other grounds and does not
reach. **It is named and not recorded**, on the rule all four entries keep: an
entry does not reach past the reading that raised it. So the seam is not
declared closed — four passages of the class are registered, and one candidate
is left for a reading of its own.

**The fifth validation rule is extended rather than joined by a sixth**, and
the extension is stated under *[Maintenance debt](#maintenance-debt)* beside
it. As written, the rule obliges a register to be re-read against its target
whenever the target changes, and an entry found discharged to record the commit
that discharged it. That is a rule about entries that have gone stale, and it
is silent about the divergence nobody ever wrote down. Both are the same
obligation seen from each end — the register is true of the file or it is not
— so the rule states both rather than splitting one obligation across two
numbers. The extension is the third rule's lesson carried across: a record is
assembled from every place an observation was made, not from the entries it
already holds, and no amount of care in classifying forty-five entries
establishes that there are forty-five.

**One editorial correction to the edition above.** The fifteenth edition's
account of itself was written in the present tense — the register *records*
forty-five corrections, thirty-one entries *still owe* something — which was
true of the register when that edition closed and is not true now. The figures
are that edition's finding and are kept; the tense is put in the past, so that
a reader does not take an edition's account of what it found for a statement
about the register today. The register's own count is in its Overview, which is
the one place obliged to be current.

**One editorial correction inside this edition.** The account of `DIV-046`
above said that *Development* gives four `cargo` commands more than
*Installation*. That entry's own *Says* clause gives three commands and then a
five-command pipeline, which is eight, and it is the same eight `DIV-051`
records of the other file. The count is corrected here rather than left for a
later edition, because this edition is the one being written and a number it
states about the eight commands it registers should be right when it closes.
Nothing in `DIV-046` changes: the entry itself never carried the wrong figure.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Seventeenth edition — the candidate that was policy and not state

The sixteenth edition swept `CLAUDE.md` for passages that state as present
something the repository does not contain, registered four of them, and named
one candidate it did not register: *Plataformas Suportadas* calls Linux and
macOS "suportados e **verificados**" and says of the other Unixes that they do
not run in validation, where nothing has been run on any target. It left the
judgement to a reading of that section, and said in as many words that the seam
was therefore not closed. **This edition is that reading. It registers nothing,
and that is its result.**

**The passage declares an intention; it does not assert a state.** The reading
and its four grounds are recorded at *[A candidate read, and not
recorded](upstream-divergences.md#a-candidate-read-and-not-recorded)*, at the
foot of the register, and the grounds are these. The section says two
paragraphs below the adjective that the concrete matrix of targets is an
architecture decision still open, and a section that declines to name the
triples cannot earlier be reporting that artefacts were verified on them. The
presupposition sits inside a three-part rule whose payload is its third
part — a problem seen only on an unsupported Unix does not fail a change —
which is deontic throughout. The section states in its first sentence that its
job is to bound what the code may assume, and every other sentence in it does
that job. And this corpus states verification in the same mode in the very
requirement the passage would be measured against: `NFR-PERF-005` says of a
clause of itself that it "SHALL be verified on every target of
`NFR-PERF-018`", and the eleventh edition's note
under `NFR-PERF-018` says every requirement of that file is verified on all
four targets — written when nothing had been run, and true today, because what
it states is the reach of an obligation and not a history of runs. A corpus
cannot record as an overstatement, in a document it does not own, a mode of
statement it uses itself in force.

**The decision record was the strongest argument the other way, and it does not
carry.** `ADR-008` records that no `musl` artefact has been run outside a
container and marks that expectation unverified, so if `CLAUDE.md` asserted a
verification the two would contradict each other on a matter of fact. That
record does not read the file that way: it calls the five-command sequence one
the root coordination document **mandates** and lists the no-second-class rule
among four obligations **carried by hand** until a pipeline exists. What it
marks unverified is also narrower than the passage, and both halves of it are
claims about baselines under `NFR-PERF-012`, which is a different section of
`CLAUDE.md` and one `DIV-041` has already read and found correct.

**Nothing is registered, and the register says so where a reader will find
it.** The finding is recorded in three places in
[upstream-divergences.md](upstream-divergences.md), because a conclusion that
nothing is owed leaves no trace otherwise and is re-opened from scratch by the
next reader: a paragraph in the Overview beside the definition of the third
kind, a line under the Index where the absent row would be, and a forward link
at the foot of `DIV-052`, which is the entry that named the candidate. The
alternative — an entry with a **Status** of *due* for a correction nobody can
make — was rejected, because it would weaken the four entries that stand by
putting a manufactured fifth beside them.

**One caution is owed to `DIV-041`, which targets that section on other
grounds.** That entry is a migration about the deferral of the target matrix,
and its correction replaces the deferral with a pointer to `NFR-PERF-018`.
Making it removes the first of this reading's four grounds, so whoever holds
the pen re-reads the adjective in the section the correction leaves behind: a
sentence calling four named triples *verificados* is a stronger claim than the
one read here. One observation is named and not acted on for the same entry —
the sentence naming Linux and macOS on two architectures states coarsely what
`NFR-PERF-018` now fixes exactly, and whether that is two sources for one truth
is a question about `DIV-041`'s target and kind rather than about this reading.

**The class is exhausted in `CLAUDE.md`, and the claim is verified rather than
inherited.** The whole file was swept again instead of the sixteenth edition's
answer being trusted, on the rule that edition acted on — a claim that a class
is exhausted is worth only the sweep behind it, and the sixteenth edition's own
sweep is what found the entries the fifteenth had missed. Every path, file and
artefact the document names was tested against the working tree at `87dd6e3`,
which was then the last commit to touch either root document, and all existed
but those already recorded under `DIV-050`, `DIV-051` and `DIV-052`. Two
further passages were read on their own and are not of the kind: the workflow
step and the language rule that name a `CHANGELOG`, which are directions
rather than claims; and the routing sentence of *Desempenho e Eficiência*,
which names `BENCHMARKS.md` and describes it as that file describes itself.

*Corrected in the thirty-first edition.* Two statements here were true when
this edition was written and are not now, and both are about the repository
rather than about the reading. `87dd6e3` is no longer the last commit to touch
either root document — `db80114` has since edited both — so the claim that the
class is exhausted is of the file at `87dd6e3` and of no later state of it,
which is what the fifth validation rule below already says of it and what the
tense now says too. And the `CHANGELOG` the workflow step names exists:
`f2d19ac` created `CHANGELOG.md` on 2026-09-21, and `Cargo.toml` has been in
the repository since the fifth sprint. The reading is untouched by either — a
direction is not a claim whether or not the path it names exists — and the
clause about what the repository has not got is removed because the reading
never needed it. The same two corrections are made in
[upstream-divergences.md](upstream-divergences.md), where the passage this one
summarises lives. **So the seam is shut**: four passages registered
in `CLAUDE.md`, one in the root `README.md` under `DIV-046`, and the candidate
read and dismissed with its reason.

**Why this is an edition and not an addition to the sixteenth.** Three entries
were added to that edition after it was written — `DIV-050`, `DIV-051` and
`DIV-052` — each on the ground that what was outstanding was the entry alone:
the finding, the kind and the placement had all been settled there, and an
edition whose whole content was *write down what the edition before it had
already found* would turn the edition record into a log of working sessions.
That ground does not reach this reading, and it argues the other way. The
sixteenth edition did not make this finding; it recorded that it could not, and
deferred the judgement to a reading it had not made. What this edition adds is
that judgement, the criterion that produced it — a passage that directs is not
a passage that asserts, and the test of the third kind is whether a reader who
acts on it fails — and a completeness claim the edition before it declined to
make. An edition records what this corpus learned, and this corpus did not know
any of the three.

**No requirement is added, amended or withdrawn, no identifier is retired, and
no entry is added to the register.** The index of
[open-questions.md](open-questions.md) stays empty, and no root document is
edited: this corpus does not hold the pen on those two files, and judging that
a passage needs no correction is not making one.

### Eighteenth edition — two readings with no home, and a phrase doing two jobs

`FR-SRV-038` obliges every difference observed between the series to be
recorded in its own section, and offered an observation two homes: a row in
its table, or a line below the table if it was observed to differ between two
servers of the same series, so that it is not counted as one. The thirteenth edition's sweep found two
readings with neither — the build's source revision and the SSL library string,
both recorded under `FR-SRV-040` and classified there as properties of the
build — and left the correction for a task of its own, because it is a decision
and not an editorial tidy: the routine and trigger timestamps earn their line
below that table by what a wall-clock time is, and no two servers of one series
had ever been compared for the SSL library string. This edition settles it, and
finds beside it a third reading of the same shape and one phrase doing two
jobs.

**Nothing here changes what `tpl` does, and no requirement is withdrawn.** One
requirement is amended, the observation record gains a passage, one stale count
is corrected, and no row enters the table.

- **The second home is stated over what it actually holds** —
  [server-contract.md](server-contract.md). `FR-SRV-038` named one
  disqualifying test — observed to differ between two servers of the same
  series — and an observation can fail both that test and the test for a row.
  The requirement now states what a row requires, and three grounds for the
  line below the table: an observation that differs within a series, a property
  of the **build**, which the fixture selects and does not pin, and an
  observation whose entailment by the series is not established, which must
  state its bound and name the observation that would settle it. The third
  ground is what the two readings needed, and the record had no way to say it.
- **Three readings are recorded, not two.** The version string's suffix — the
  distribution each image was built on — differs between `10.11` and the other
  three exactly as the SSL library string does, is classified in the same
  sentence of `FR-SRV-040`, and had no home either. It is also what makes the
  SSL library string's classification legible, so recording the two without it
  would have recorded the weaker half of the finding. This is the thirteenth
  edition's third validation rule again, applied to the sweep that edition ran:
  a record is assembled from every place an observation was made, and its own
  list of what it lacks is not that place.
- **The classification is settled on a rule in force, not handed back.** The
  fourth validation rule decides it — a reading this project's own fixture can
  move is not credited to the series until the condition is varied — and the
  fixture names a series and not a patch release, so the build behind it is
  exactly such a condition. For the suffix and the source revision the ground
  is stronger and needs no rule: a distribution is not a MariaDB fact, and
  `FR-SRV-040` already calls the revision *a distinct hash per build*, where a
  series holds many builds. For the SSL library string the ground carries only
  as far as **not established** — its split is coextensive with the split of
  the distribution, and nothing observed separates the two explanations — which
  is exactly what a line below the table claims and what a row would exceed.
  The passage states the bound and names the observation that would settle it.
- **A row was impossible in any case, for a reason prior to the
  classification.** A row names what each series returned. The observation
  recorded that the four servers differ in the source revision and in the SSL
  library string, and did not record what any of the four returned, so four
  columns of each row could not be filled from anything this corpus then held.
  The
  outcome that would have produced new evidence was therefore not the only one
  that needed it: a row needs a new observation too, and a different one. The
  twenty-third edition records the reading that supplies those columns and
  leaves the classification, which is the ground that survives it.
- **One phrase was doing two jobs, and the record now separates them** —
  `FR-SRV-038`. The fourteenth edition's sweep calls nine rows of the table
  properties of the server build, to establish that no setting of the fixture
  can move them. The three readings below the table are properties of the build
  in a second sense: the build carries them and the series does not fix them.
  The first sense makes a row immune to the fixture; the second keeps a reading
  out of the table. Left unseparated, the record would have called the same
  phrase a difference between the series in nine places and not a difference in
  three.
- **One stale count is corrected, and a second is checked and does not move.**
  The paragraph guarding the timestamps warned a reader not to count them as a
  *twelfth* series difference, which was right when the record held eleven rows
  and has been wrong since the tenth edition; it reads fifteenth now, because
  the number a reader would wrongly reach is one past the rows the table holds
  and moves with them. `FR-CAT-029` in
  [catalogue-coverage.md](catalogue-coverage.md) is the second, and it does not
  move at all, because no row enters the table; the check is recorded there all
  the same, since a count that did not move is worth only as much as the reader
  can see of it having been recomputed.

**Why this is an edition, and not an amendment folded into the seventeenth.**
The test the sixteenth and seventeenth editions settled between them is whether
this corpus learned anything, and the seventeenth's answer turned on its having
changed no requirement: what it added was a judgement. This edition **amends a
requirement in force**. `FR-SRV-038` admitted two homes and an observation
could fail the test for both, which is a defect in the rule and not in the rows
kept under it; the requirement now states what a row requires and what the home
below the table holds, and three readings enter that home. The thirteenth
edition, which found the defect, deliberately did not fix it and said why — the
correction was a decision — so what was outstanding here was neither an entry
nor a writing-down but the decision itself and the ground for it. An edition is
what records that.

**Two editorial corrections, both of one shape: a narrative of what an earlier
edition did, written in the present tense and overtaken.** The thirteenth
edition's account above described `FR-SRV-038` as offering two homes, and this
edition gives the requirement a third; it now says *then offered*, and states
the second home as that edition read it. And five paragraphs of
*[Maintenance debt](#maintenance-debt)* — the thirteenth edition's and the four
that follow it — said that the item each of them left alone *is* or *remains*
outstanding, which was each edition's own true report and is false of the item
now; each says what it did, in the tense it did it in. The fourteenth edition
made the same correction on the same ground, to the eighth edition's *the
fixture provides none*.

**No open question is raised, and none could be.** No requirement of this
corpus reads any of the three readings, so nothing waits on the observation the
passage names; an entry would sit in an index whose emptiness means that
nothing is waiting. The index of [open-questions.md](open-questions.md) stays
empty, and the last item of *[Maintenance debt](#maintenance-debt)* is
discharged.

### Nineteenth edition — a table that did not say how to read itself

`FR-ERR-001` fixes ten exit codes and, beside each, a *Condition* cell. It has
never said whether that cell enumerates the conditions that produce the code or
characterises the class they belong to, and the two readings build different
programs from the same requirement. Deriving the project's error type from this
corpus forced the choice twice: once on the column as a whole, and once on
three cells that each name a configuration key — `64` for an unknown one, `66`
for a configuration key among the named objects that do not exist, and `78` for
a key outside the enumerated space — with nothing in the file separating them.
**Both readings were taken correctly**, and neither was taken from the file
that raises the question: the three were separated by following citations into
[cfg-commands.md](cfg-commands.md) and
[configuration-model.md](configuration-model.md), and the column was read as a
characterisation because the alternative collides with requirements in force
elsewhere. This edition writes both answers where the question is asked.

**Nothing here changes what `tpl` does, and no requirement is withdrawn.** The
ten codes, their names, their conditions and the caller's next step are
unchanged. One requirement is amended to say how its own table is read, and one
is added that routes a configuration key to the code it already produced.

- **The codes are closed and the *Condition* column is not** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-001` now says
  so in its own text, and says what follows from it: a condition is neither
  absent nor misfiled because no cell names it, every condition is stated by
  the requirement that owns the behaviour, and `FR-ERR-002` is what obliges
  that requirement to carry a code. Two requirements had already read the table
  that way without the table saying so — `FR-CFG-043`, which declines a fourth
  outcome because `FR-ERR-001` "fixes the code set", and `FR-OUT-033`, which
  reads the `66` cell as what that code is reserved for — so the amendment
  states a reading the corpus already used rather than choosing between two.
- **Six conditions are recorded as examples, and the remainder is measured
  rather than listed** — the same file. `FR-CLI-014`, `FR-CLI-018`,
  `FR-SCH-010`, `FR-CFG-017` and `FR-HELP-028` carry `64`, and `FR-CONF-021`
  carries `78`, and none appears in any cell. They were found by one reading
  made for another purpose, not by a sweep, and the amendment says so; what was
  measured instead is the scale, by a textual sweep of the twenty module files
  on 2026-09-12 — one hundred and twelve requirements outside that file name a
  code of the table. Closing the column was rejected on the fifth validation
  rule's ground in a new setting, that the second copy is the one nobody edits;
  correcting the six cells found to be short was rejected because it answers
  this reading and not the next one.
- **Three codes for one configuration key, separated where a caller reads** —
  the same file. `FR-ERR-035` routes a key by where it was met and what was
  asked of it: named to `tpl cfg set` and outside the key space is `64`, named
  to `tpl cfg get` or `tpl cfg unset` and absent from the file is `66`, and
  carried by the file and outside the key space is `78`. It cites
  `FR-CFG-009`, `FR-CFG-007` with `FR-CFG-012`, and `FR-CONF-034`, and restates
  none of them, because each belongs to the module that owns the command or the
  file. It also states why the three cannot collide, which was derivable from
  that file all along and had never been derived: the file is validated at step
  3 of `FR-ERR-006` and no `cfg` subcommand is excused from it by
  `FR-PROJ-025`, so a `.cfg` carrying a key outside the space is refused before
  any command resolves a key of its own — and `tpl cfg set` given a bad key
  against such a file exits `78` and not `64`, per `FR-ERR-007`.

**The lesson is of a shape the five validation rules do not cover, and it is
recorded here rather than made a sixth.** Each of those rules is about a
statement decaying — against a later observation, a later row, a changed
fixture, an edited file — and nothing decayed here. The table has been silent
on this point since the first edition and is silent in the same way today. What
found it was the first attempt to derive an implementation from this corpus,
which is a reader it had not had, and that same reading found the three cells.
Whether the lesson generalises — whether every table here that
characterises rather than enumerates should say which it does — is a question
about the corpus and not about one file, and it is named and left for a reading
that covers the corpus rather than settled from a single instance.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Twentieth edition — four absolutes stated wider than their ground, and a rule its reader could not find

Deriving the error type and the diagnostic renderer from this corpus produced
five statements an implementer could not apply without choosing what they
meant. Four are of one shape: an absolute stated wider than the ground the
requirement itself gives for it. *Each distinct condition SHALL have its own
code*, against ten codes. *Names matching `[A-Za-z0-9_]{1,64}`*, against a
suggestion population that includes every configuration key and five hyphenated
flags. *SHALL NOT restate the `error` line*, against a row of `FR-ERR-034` that
obliges the `cause` of a `65` to name what the `error` line has just named. *No
ANSI escape sequence, under any circumstances*, against a catalogue name
carrying the single-character CSI. Each was written to carry a purpose this
corpus states exactly beside it, and each says something wider than that
purpose, which a reader implementing it has no way to narrow. The fifth is not
of that shape: `-v` with `-q` was resolved by choosing, and the corpus had
decided it in the first edition, in the file the reader was not reading.

**One of the five changes what `tpl` does, and the edition says which.** A
suggestion may now name a configuration key and a hyphenated flag, which the
character set as written admitted in no form. The other four state what was
already in force: the codes, their names, their conditions and the caller's
next step are untouched, no requirement is withdrawn, and no identifier is
added or retired.

- **What a closed set of ten codes can promise** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-002` now
  obliges every condition that ends an invocation in failure to carry exactly
  one code of `FR-ERR-001`, named by the requirement that owns the condition.
  Its prohibition is unchanged and undiminished — two conditions never share a
  code where the caller's next step differs — and it is the whole of what the
  first sentence was reaching for. As written, the first sentence was refuted
  by `FR-ERR-006`, which routes every parsing fault to `64` by design, by the
  hundred and twelve requirements the nineteenth edition counted, and by
  `FR-ERR-035`, which routes one configuration key to three codes *because*
  three next steps differ. The *Overview* repeated the overstatement in prose
  and now agrees with the requirement. Deleting the sentence was rejected:
  `FR-ERR-001` cites this requirement for the obligation that makes its
  *Condition* column safe to read as a characterisation.
- **What the hint character set governs** — the same file. `FR-ERR-022` now
  distinguishes a spelling this specification enumerates, which is a literal —
  a command or alias of the command tree, a flag a node declares, a key of
  `FR-CONF-002` — from every other value, whatever its source, which is what
  the character set tests: a table, a view, a routine, a template, a database
  entry, the entry name inside a `database.<name>` key, and the name of an
  environment variable among them. Values are tested one at a time and the
  separators that join them are literals, so a dotted key and a command path
  are each admissible exactly as far as their own segments are.
  `FR-ERR-021` and `FR-ERR-022` contradicted each other outright: no key of
  `FR-CONF-002` matches the set, because all fifteen key forms contain a dot,
  and `FR-ERR-023` drops a candidate outside the set in every form, prose
  included. Two of the eight populations `FR-ERR-021` names could therefore
  never be suggested, and commands escaped only by accident of spelling.
  Widening the set was rejected — it answers this reading and not the next, and
  every widening is paid for by every untrusted name — and so was removing the
  two populations from `FR-ERR-021`, which withdraws the suggestion where it is
  safest. `FR-ERR-023` and `FR-SEC-019` in [security.md](security.md) carry the
  same distinction, the second because the threat rests on it.
- **Which requirement governs where a `cause` repeats its `error` line** — the
  same file. `FR-ERR-010` now states that a `cause` restates the `error` line
  when it adds nothing to it, and that where a row of `FR-ERR-034` obliges a
  fact the `error` line also carries, the row governs. The example of
  `FR-ERR-008` is this file's own model of a correct message and its `cause`
  repeats both names from the line above it, so the reading was already in use
  where the question is asked. Resolving it the other way was rejected: a
  `cause` that subtracted whatever the `error` line happened to say would make
  a testable obligation depend on the wording of an individual message, which
  this file puts out of scope.
- **The subject of `NFR-DET-004`** — [cli-contract.md](cli-contract.md). It
  governs what the system composes for presentation, and not a byte carried
  from the catalogue, a template, a `--context` value or the argument vector;
  `FR-ERR-024`, `FR-OUT-018` and `FR-OUT-019` own those, as they already did.
  A reading that reached content would forbid `tpl render` the byte-for-byte
  output `FR-OUT-019` guarantees it, and would make a requirement about colour
  a third owner of escaping with a third exception list. That reading was
  stated and rejected in the requirement, because the question comes back
  otherwise. One consequence stands: a value escaped over the C0 range alone
  can still carry a C1 control to a terminal that honours it. It is a question
  for the two requirements that own escaping, it is recorded as the one item of
  *[Maintenance debt](#maintenance-debt)*, and `FR-ERR-024` says so where a
  reader meets the gap.
- **`-v` with `-q` was never undefined** — [global-flags.md](global-flags.md).
  `FR-CLI-015` refuses the pair with `64`, and has since the first edition.
  This file declares both flags, and until now declared neither against the
  other; it is where an implementer resolving the pair reads, and `FR-GLOB-015`
  now names the refusal at the point of declaration. Stating a precedence here
  — quiet over verbose, or last-wins — was rejected as a contradiction with a
  requirement in force, and on the ground `FR-CLI-014` already gives: a result
  that depends on how a script grew.

**The lesson is the nineteenth edition's question, with four more instances.**
That edition named one and left it for a reading that covers the corpus:
whether every table here that characterises rather than enumerates should say
which it does. Four of these five are the same question about a sentence rather
than a table — whether an absolute is stated wider than the ground this corpus
gives for it — and they were found the same way, by the first reader obliged
to turn this corpus into a program. It is still not a sixth validation rule:
each of the five rules is about a statement decaying, and nothing here decayed.
These sentences were exactly this wide in the edition that wrote them, and it
took a reader who could not ask a question to find out. The question is now
recorded as recurring rather than isolated, and it is still left for a reading
that covers the corpus rather than settled from five instances.

**No requirement is withdrawn, no identifier is retired or assigned, and no
open question is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Twenty-first edition — a key nobody had named, and an array with two shapes

Deriving the help command and the JSON document it emits produced two questions
an implementer could not answer from what was written, and both are about the
shape of that one document. `FR-ENV-005` obliged `tpl help --format json` to
publish the template surface and named no place in the document for it, while
`FR-HELP-017` listed a `data` of three keys with nothing said beside them, which
reads as the whole of what `data` may carry; between them the surface had a
publisher and no home. `FR-HELP-019` fixed the members of an entry of
`data.commands` and left unstated the array those entries sit in, so the `path`
on every entry read as a flat array and *the subtree rooted at* of `FR-HELP-016`
read as a nested one, with nothing in the corpus to choose between them.

**Neither settlement withdraws a requirement or reverses a decision.** Both fix
a shape that was not fixed, in a document no implementation emits yet, and the
shape is contract from here.

- **Where the template surface is published** —
  [help-and-version.md](help-and-version.md) and
  [template-environment.md](template-environment.md). `FR-HELP-017` now carries
  a fourth key of `data`, `template_surface`, after `commands`, and states that
  `data` is an **open** set of keys to which a later edition may add, after the
  last, so that the position of every key already present is unchanged. The
  envelope of `FR-OUT-024` is untouched and stays closed at three keys;
  `FR-OUT-028` bars a fourth key beside `data` and says nothing about a fourth
  key inside it, and it is undiminished. `FR-ENV-005` fixes what the key holds
  and keeps ownership of the content: three sibling objects of one shape, one
  per group of `FR-ENV-001`, each carrying `guarantee`, `filters`, `tests`, and
  `functions`, with the names drawn from the requirements that already fix them
  and group 3 carried as the entry whose `guarantee` is `none` and whose arrays
  are `null`, per `FR-OUT-012`. Publishing the surface inside the entry of
  `tpl render` was rejected: it would not survive the reduction of
  `FR-HELP-029`, and a command's entry lists the flags that command declares,
  per `FR-HELP-020`. A document of its own was rejected too, because it adds a
  node to a tree `FR-CLI-002` closes.
- **What shape `data.commands` has** —
  [help-and-version.md](help-and-version.md). `FR-HELP-019` now states that the
  array is **flat**: one entry per node below `tpl`, group nodes included, each
  carrying its full `path` and never its children. `FR-HELP-016` and
  `FR-HELP-029` define a subtree under it as a **selection** over that array —
  the entry whose `path` is the path given, together with every entry whose
  `path` extends it segment by segment, in the order they hold in the unreduced
  document — with every other key of `data` emitted unreduced. Nesting was
  rejected because it carries twice a relation `path` already states, and
  because `FR-HELP-026` had rested on the flat reading since it was written:
  its rationale turns on every node carrying its full `path` so that a caller
  can hand the path straight back.

**The two are one act because the first is unanswerable without the second.**
A key added to `data` has to be a key the reduction of `FR-HELP-029` leaves
alone, and what that reduction does could not be said until the array it
reduces had a shape. The pair were found the way the twentieth edition's five
were, by the first reader obliged to turn this corpus into a program, and they
are of a different kind from those: nothing here decayed and nothing was stated
wider than its ground — these shapes were never fixed at all, and the corpus
read as though they had been.

**No requirement is withdrawn, no identifier is retired or assigned, and no
open question is raised or reopened.** The index of
[open-questions.md](open-questions.md) stays empty.

### Twenty-second edition — three writes the tool could not undo

Implementing [project-and-discovery.md](project-and-discovery.md),
[configuration-model.md](configuration-model.md) and the non-server part of
[cfg-commands.md](cfg-commands.md) reached three places where a `cfg` command
this corpus admits writes a `.tpl/.cfg` the same corpus refuses to read. In
each of them the invocation is legal, the file it produces is not, and every
later invocation is refused at step 3 of `FR-ERR-006` — including the `cfg`
command that would undo the write, because no `cfg` subcommand is among the
commands `FR-PROJ-025` excuses from reading and validating the file. **The
three are one defect in three places**: a rule stated over the invocation where
the state it protects is the entry, or over one command where two reach the
state.

**None of the three needed a decision from outside this corpus.** Each is
settled on a requirement already in force, and each says so in its own text.

- **`--dsn` admits what the file admits** — [cfg-commands.md](cfg-commands.md).
  `FR-CFG-031` read "SHALL accept whatever the caller writes ... and SHALL
  store it verbatim", which is a rule about secrets — `FR-SEC-002` and
  `DIV-002` cite it for exactly that — stated wide enough to be read as a rule
  about syntax. It now admits exactly what `FR-CONF-009`, `FR-CONF-010` and
  `FR-CONF-011` admit, validates before writing, and exits `64` without writing
  anything where the value is not admitted. A literal password is still
  admitted and still stored as written, because `BR-CFG-003` governs and what
  is refused is a value the reader cannot accept. The admission is an equality
  in both directions: more writes a file the reader refuses, and less leaves a
  legal `.cfg` that `tpl` cannot write.
- **A write that would make an entry incoherent is refused** — the same file.
  `FR-CFG-048` is new, and it states over one **entry** the rule `FR-CFG-016`
  and `FR-CFG-029` state over one **invocation**, which is where `FR-CONF-007`
  has always stated it. `tpl cfg set`, `tpl cfg database add` and
  `tpl cfg database update` are refused with `64`, and `.tpl/.cfg` is left
  unchanged, where the entry as it would stand after the write is a combination
  `FR-CONF-007` refuses. Repairing the entry by removing the fields the new
  value supersedes was rejected against `FR-CFG-020`, `BR-CFG-001` and
  `BR-CONF-004` — it is the reader's forbidden guess, made by the writer.
  Writing and letting the next read fail was rejected because it reports the
  fault one invocation late and against the file rather than against the
  invocation that caused it. `64` rather than `78` because the file is valid
  and the invocation is not, which is `FR-CFG-017`'s shape exactly.
- **The coherence obligation is over the state, not over one command** — the
  same file. `FR-CFG-023` clears a `core.database` whose entry
  `tpl cfg database remove` has deleted, and `tpl cfg unset database.<name>`
  reaches the identical state. It now names both commands and clears the
  reference in the same rewrite, which `FR-CFG-041` already makes atomic.
  Nothing is decided that the requirement had not decided: its own rationale
  settles silent clearing against a warning and against a refusal, for this
  state.

**One observation is recorded and not acted on.** `FR-CONF-010` states which
two schemes are accepted and names no code for refusing a third, where
`FR-CONF-011` states `78` for a query parameter in its own text. The `78` row
of `FR-ERR-001` carries the condition as *invalid entry*, so nothing is
ungoverned under `FR-ERR-002`, and no requirement of this edition rests on it —
`FR-CFG-031` cites `FR-CONF-010` for what is admitted, and not for what a file
carrying something else produces. It is named here so that the next reader of
that requirement does not take the silence for a gap this edition left behind.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** One identifier is assigned, `FR-CFG-048`. The index of
[open-questions.md](open-questions.md) stays empty.

### Twenty-third edition — a reading recorded as absent, and an object with no field list

Two instruments reached this corpus before the catalogue reader was written,
and each returned a defect the other could not have found. Re-running the
fixture's build reading found `FR-SRV-040` recording that a malloc-library
variable returns no row on any server, where it returns one on every server.
Writing worked examples against the model found that a routine's `kind` has no
stated casing to branch on, and that a table object — the one the model is
mostly made of — has no field list where a view and a routine each have one.

**Nothing here changes what `tpl` reads or how it fails.** Two requirements are
amended, one is scoped to the subject it always had, one gains a note saying
what it is not, one is added, and one observation record is corrected against
the run that contradicts it.

- **A reading recorded as absent is present on every server** —
  [server-contract.md](server-contract.md). `FR-SRV-040` said a
  malloc-library variable was requested and no row came back on any of the
  four, so no such variable exists on these servers.
  `version_malloc_library` returns a row on all four series and on the fifth
  listener, and its value is `system`; it was read three ways on each of the
  five on 2026-09-18 and all fifteen readings agreed, and the run is recorded
  with the fixture. What returned no row was the **name**: a wrong name prints
  nothing at all under a `SHOW` with a `LIKE`, header included, and exits `0`,
  while selecting the same wrong name directly fails with `ERROR 1193`. The
  requirement now records the reading and the shape that produced the wrong
  one, because that shape is what the next reader will meet. The reading takes
  neither home of `FR-SRV-038` — it agrees everywhere, and both homes hold an
  observation that differs. Striking the sentence was rejected: the reason the
  record was wrong would go with it.
- **The two readings of the build now have their values, and still take no
  row** — the same file. The passage below the table of `FR-SRV-038` declined
  a row to the source revision and the SSL library string on two grounds, and
  the same run discharges one of them: four columns of each row could not be
  filled from anything this corpus held, and now they can. The classification
  is untouched and declines the row alone — a source revision is per build by
  what it is, and the SSL library string's split is still coextensive with the
  distribution each image was built on, which the fourth validation rule below
  refuses to credit to the series while the fixture selects the build without
  pinning it. The run strengthens that refusal rather than weakening it: the
  fifth listener runs the `10.11` image and returns both readings byte for
  byte. Transcribing the values into this corpus was rejected — a per-build
  value copied here decays silently when the fixture's tag moves, and
  difference 3 declines to restate the fixture's own record for the same
  reason. Both variables are named now — `version_source_revision` and
  `version_ssl_library` — where one sentence of that passage had said that
  neither requirement named them and that nothing in the fixture read them.
  Both halves were false.
- **A routine's `kind` is one string** —
  [catalogue-coverage.md](catalogue-coverage.md), with
  [schema-commands.md](schema-commands.md). `FR-CAT-016` read "procedure or
  function" and fixed no value, so a template or a `jq` filter had no casing
  to branch on and a wrong guess matched nothing at exit `0`. It now carries
  the catalogue's own string unchanged, `PROCEDURE` or `FUNCTION`, which is
  what `FR-CAT-048` records the routine-type field returning. **It is not the
  prefix of the qualified form of `FR-SCH-008`**, which is lower case, and
  both requirements now say so: a caller composing a qualified name from
  `kind` folds the case, and nothing else separates the two. Lower-casing the
  field was rejected because it would make `kind` the only enumerated
  catalogue value the model rewrites — `table_type`, `check_option`,
  `is_updatable`, `security_type`, `body_kind`, `parameter_style`,
  `is_deterministic`, `sql_data_access`, a trigger's `event`, `timing` and
  `orientation`, and a parameter's `mode` are each the catalogue's own string.
- **A table object has an index of its properties** — the same file.
  `FR-CAT-053` is new. Views get a field list in `FR-CAT-047` and routines in
  `FR-CAT-048`; a table had coverage requirements, per-property requirements
  for its indexes, keys and constraints, and nothing that said what a table
  carries. The new requirement indexes every property against the requirement
  that fixes it, obliges a later requirement that gives a table a property to
  add it there, and fixes the two properties no requirement of that file
  stated:
  `name`, which every other named object of that section already carries under
  that key, and the indexes, which `FR-CAT-010` shapes and `FR-CAT-042`
  enumerates without either saying that a table carries them — `FR-SCH-009`
  and `FR-CTX-007` said it from outside. It
  does **not** write a catalogue field list, and says why: the observation
  pass of 2026-09-10 recorded the field list of every object kind an entry of
  [open-questions.md](open-questions.md) had asked for, no entry asked for the
  table catalogue's own row, and this corpus holds no reading of it. Writing
  one from the fields already named and from MariaDB's documentation was
  rejected on the seventh edition's validation rule below, which exists
  because four requirements were once coherent, correctly cross-referenced,
  and describing a catalogue that does not exist.
- **One enumeration names its subject** —
  [context-document.md](context-document.md). `FR-CTX-013` read "`kind` SHALL
  be an enumerated field taking exactly the three values", and with
  `FR-CAT-016` stating two strings for a field of the same name the absolute
  would have read across both. It now says *the `kind` of a column default*,
  which is the subject `FR-CTX-011`, `FR-CTX-012` and the section heading
  already gave it. This is the twentieth edition's shape — an absolute stated
  wider than its ground — found this time by a change that would have
  collided with it.

**Two editorial corrections, one to a claim that was never true and one to a
report the evidence has overtaken.** The seventh edition's entry above said
the field lists of *every object kind* were fixed and listed eleven, none of
them the table; it now says *every object kind an entry had asked for*, which
is what that edition did. And the eighteenth edition's entry said four columns
*could not be filled from anything this corpus holds*, in the present tense of
a report this edition overtakes; it reads *then held*, which is the correction
that edition made twice itself.

**None of the five validation rules below would have found any of the four
defects, and the malloc sentence is the one worth saying why about.** It cited
an observation, the observation had been made, and the record of it was
internally coherent, correctly cross-referenced, complete against the corpus
and conditioned — the four rules this corpus can run on itself each pass it.
What was wrong was the **reading**, and the form the reading took is what
hid it: a `SHOW` with a `LIKE` answers a wrong name and an absent variable
with the same silence and the same exit code, so a negative result carries no
evidence that the question was the right one. The casing and the missing index
are of the twenty-first edition's kind instead — shapes nobody had fixed,
which no rule about decay or completeness can reach. What found all four was a
re-run of a fixture reading by a second form and the first worked examples
written against the model: the same class of instrument the twentieth,
twenty-first and twenty-second editions used, applied before the code it
precedes.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** One identifier is assigned, `FR-CAT-053`. The index
of [open-questions.md](open-questions.md) stays empty.

### Twenty-fourth edition — an enumeration read as an order

The first implementation of the connection start reached two requirements in
force that order the same three statements differently. `FR-SRV-012` required
an integration test to assert the three connection-start statements "in the
order that table states", and the table of `FR-SRV-006` lists the version probe
above the read-only session statement. `FR-ERR-006` orders the three conditions
those statements settle — the read-only session of `FR-SRV-010`, then the
product check of `FR-SRV-003`, then the version-window check of `FR-SRV-020` —
and a condition cannot be evaluated before the statement that produces its
evidence, so that order puts the read-only pair before the probe. No test could
satisfy both, and the test that `FR-SRV-012` itself mandates could not be
written until one order was stated.

**Nothing here changes what `tpl` reads or how it fails**, and the order that
governs is the one three requirements in force already state for themselves.
One requirement is added, one has a clause replaced by a citation, one table is
declared the enumeration it always was, and three are checked and left as
written.

- **The defect is in the reading, and the table is the evidence** —
  [server-contract.md](server-contract.md). The table of `FR-SRV-006` fixes
  **membership**: which four kinds of statement this system may issue, and each
  kind's occasion and count. It is not a sequence and cannot be read as one,
  because its first row is the catalogue read — the statement issued last, and
  as many times as the command requires. A reader taking the rows top to bottom
  is given the one statement that must follow the other three first. The table
  now says which kind of table it is, and the only order any cell of it ever
  carried, the adjacency of the read-only statement and its read-back, is cited
  from the requirement that now states it. No row moves and no count changes.
- **The order is stated once, in `FR-SRV-042`** — the same file. The read-only
  session statement of `FR-SRV-008`; its read-back under `FR-SRV-009`,
  immediately after it; then the version probe of `FR-SRV-002`; and every
  `SELECT` against `INFORMATION_SCHEMA.*` after all three. The requirement
  carries the discipline `BR-SRV-005` states for the supported set — stated
  here, cited everywhere else — and it derives the order from `FR-ERR-006`
  rather than asserting it independently, so the two cannot drift apart.
- **`FR-SRV-012` cites it and states nothing of it itself** — the same file.
  The clause "in the order that table states" becomes "in the order
  `FR-SRV-042` fixes". What the test expects is otherwise unchanged: the four
  kinds, no fifth, and the three connection-start statements exactly once each,
  observed on the server under `BR-SRV-003`.
- **Three requirements are checked and left as written.** `FR-ERR-006`, in
  [errors-and-exit-codes.md](errors-and-exit-codes.md), orders conditions and
  not statements, and its eight steps and the ordering among the three
  conditions of the cache-or-connection step are as the fourth edition left
  them. `FR-SRV-002` admits both orders — it defers to the read-only pair
  without saying whether that pair is issued before the probe — which is
  exactly why the intent its fourth-edition amendment states needed a
  requirement of its own. And `FR-CFG-024`, in
  [cfg-commands.md](cfg-commands.md), whose step 2 is the read-only pair and
  step 3 the series check, fixes the four outcomes that
  command reports and not the order of the statements. Each now records that it
  was checked, and cites `FR-SRV-042` rather than repeating it.

**The rejected order is recorded with the argument for it, because the argument
is real.** Identifying the server before setting its session is the order the
table was read as stating, and it gives the more accurate diagnosis for the
commonest fault this check meets: an entry pointed at the wrong server is
reported as the wrong product rather than as a session that could not be set
read only. It loses on the exchange. What it buys is a `cause` line separating
two conditions that already share the code `78`, the name of the entry and the
caller's next step; what it spends is the read-only guarantee, on the one
statement every connection issues, since the version probe is itself a read.
Taking it would also reverse the fourth edition's decision in `FR-ERR-006`,
whose stated ground is that the strongest guarantee is confirmed before the
server is characterised, and a better-targeted `cause` for one class of
misconfiguration is not a ground that edition failed to weigh. A third option —
probing first and deferring every verdict until all three answers are held —
satisfies `FR-ERR-006` literally and is refused for what it does in between: a
server the probe has already shown `tpl` refuses still receives the read-only
pair, so the diagnosis is bought at a higher price than the rejected order
pays, not a lower one.

**One question the nineteenth edition named is settled for one table and left
open in general.** That edition asked whether every table in this corpus that
characterises rather than enumerates should say which it does, and left it for
a reading that covers the corpus. `FR-SRV-006`'s table now says it, on its own
ground: a requirement in force read it wrongly, and a mandated test could not
be written while it did. Nothing here is a reading of the other tables, and the
question stands where the nineteenth edition left it.

**One editorial correction.** The Scope above said this specification has been
written in *twenty-two* editions, which was the twenty-second edition's count;
the twenty-third added a section and left the number behind it. It reads
twenty-four, which is the number of edition sections above.

**None of the five validation rules below would have found this defect.** Both
requirements resolve their cross-references correctly, neither describes an
observation, no record of observations is in question, no condition of
observation has moved, and nothing outside this corpus had decayed. The two
were coherent apart and contradictory together, which is the eighth edition's
shape; what found it was the first implementation of the connection start,
written against both.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** One identifier is assigned, `FR-SRV-042`. The index of
[open-questions.md](open-questions.md) stays empty.

### Twenty-fifth edition — four refusals with no condition, a prefix with no casing, and a listing no rule produced

The first arm is the first block whose output a calling agent consumes
directly, and three instruments reached this corpus before it was built. The
first implementation of the connection and the catalogue reader returned four
conditions it could reach and no requirement named. Writing worked examples
against the qualified routine form found a token this corpus neither admitted
nor refused. And reading the one worked `text` listing this corpus carries
against the layout that now exists found that no single rule produces it.

**Nothing here adds an exit code.** The set of `FR-ERR-001` is closed and is
untouched; four requirements are added, four are amended, and every new
condition carries a code the table already holds.

- **An entry that names no host, and an entry that names no database** —
  [configuration-model.md](configuration-model.md). `FR-CONF-002` gives `host`
  and `database` no default, and `FR-CFG-016` requires one discrete flag and
  neither of those two, so
  `tpl cfg database add reporting --user reader`
  writes an entry that is legal in the file, passes `FR-CONF-007`, and
  describes no connection and no read. `FR-CONF-040` and `FR-CONF-041` refuse
  the invocation that selects such an entry with `78`, at the entry-resolution
  step of `FR-ERR-006`, before a connection is opened. The second of the two
  also states the question nothing had answered — **which database a read
  covers**: it is the one the entry names, by the `database` key or the
  `/database` segment of a DSN, and from no other source. Every `schema`
  subcommand depends on it. Composing either refusal where the connection is assembled
  was rejected, because that layer cannot satisfy the `78` row of `FR-ERR-034`
  — it holds neither the file nor the position the `cause` must name; refusing
  the whole file at step 3 was rejected, because it makes `.tpl/.cfg`
  repairable only by hand, which is the defect the twenty-second edition
  closed from the writing side; taking the session's default schema, guessing
  among the databases a reader can see, and adding a flag beside the `-d` that
  already names the entry were each rejected in `FR-CONF-041`'s own text.
- **A session that opens and does not hold** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). The four conditions of
  `69` this corpus stated were a name that did not resolve, a refused
  connection, a failed TLS handshake, and a deadline. A connection dropped
  while a statement is in flight is none of them, and was reported as the
  second — the right code under a `cause` line that names the TCP connect for
  a session that had already connected, authenticated, been set read only and
  been probed. `FR-ERR-036` states the condition, and the `69` row of
  `FR-ERR-034` gains a fifth phase, the version probe of `FR-SRV-002`, which
  belonged to none of the four. A tenth code was rejected against `FR-ERR-001`
  and `FR-ERR-002`: the caller's next step is the one every `69` carries, and
  what two conditions on one code owe a reader is a `cause` that separates
  them.
- **A database the reader cannot see** —
  [privileges-and-completeness.md](privileges-and-completeness.md). A schema
  catalogue returning no row for the database a read covers was reported as a
  violated internal invariant, exit `70`, because no condition fitted.
  `FR-ERR-030` closes `70` to a panic and to a defect this system detects in
  itself, and this is neither: nothing about `tpl` is wrong when a server
  declines to show a schema, and `70` tells a caller the condition is not
  theirs to fix when a grant or a corrected key fixes it. `FR-PRIV-021` makes
  it `77`, chosen on the caller's next step as `FR-ERR-002` requires, and
  states in its own text that it does not claim which of the two explanations
  holds — the shape is the zero-rows shape of `FR-PRIV-018`, arriving for the
  object the whole document describes. `66` was rejected because its next step
  is to list what exists, and the population of databases is one this system
  never reads: obtaining it is a second statement against the counts
  `NFR-PERF-001` and `NFR-PERF-002` fix.
- **The qualified prefix is matched as written** —
  [schema-commands.md](schema-commands.md). The twenty-third edition fixed
  `kind` at `PROCEDURE` and `FUNCTION` and said a caller composing a qualified
  name from the field folds the case. `procedure:calc_vat` was admitted and
  `PROCEDURE:calc_vat` — what a caller that reads `kind` and concatenates
  actually produces — was governed by nothing. `FR-SCH-008` now matches the
  prefix case-sensitively and refuses a non-matching case with `64`, decided
  at step 1 of `FR-ERR-006` from the token alone, with a `hint` carrying the
  same invocation in lower case. `FR-CAT-016` gains the pointer that makes the
  two readable from either side. Case-insensitive matching was rejected
  because no spelling this corpus fixes is stated to be folded and because it
  multiplies the names the prefix shadows; letting the token fall
  through as a bare name was rejected because the resulting `66` diagnoses the
  wrong fault, over a population that cannot hold the name the caller meant.
- **The worked listing is now the rule applied to its own data** — the same
  file. `FR-SCH-026`'s listing could not be reproduced by any single layout
  rule: its `NAME` column was twelve wide against a widest cell of eleven, its
  `COLUMNS` column was seven wide in the header row and eight beneath it — so
  the header's `COMMENT` began one column left of every comment under it — and
  it right-aligned a numeric column that no requirement mentioned. This is the
  only worked `text` listing this corpus carries, and `FR-OUT-006` fixes that
  the output is aligned columns under a header row and fixes nothing further,
  so nothing else here could settle it. The requirement now states the rule in
  six clauses — column order, width from the widest cell including the header,
  left alignment for every cell including a column of numbers, a two-space
  separator, a row ending at its last non-empty cell, one `\n` per line — and
  the listing is those clauses applied to its own three tables. Right-aligning
  a numeric column was rejected: it obliges an alignment per column and obliges
  this corpus to say which columns hold numbers, a second field list written
  for a surface `FR-OUT-004` declares is not a contract.

**Three readings produced this edition and none of them is one of the five
validation rules below.** Four of the six defects were found by writing the
connection and the catalogue reader against this corpus — the instrument that
produced the twentieth, twenty-first, twenty-second and twenty-fourth editions;
one by composing a qualified routine name from the field this corpus fixes, as
a caller does; and one by reading this corpus's only worked listing back
against the layout that now exists. All six were shapes nobody had fixed rather
than statements that had decayed, which is why no rule that re-reads a register
would have found any of them.

**No requirement is withdrawn, no identifier is retired, and no open question
is raised or reopened.** Four identifiers are assigned — `FR-CONF-040`,
`FR-CONF-041`, `FR-ERR-036` and `FR-PRIV-021` — and none in
[schema-commands.md](schema-commands.md), whose two defects are amendments to
requirements in force, as is the pointer `FR-CAT-016` gains. The index of
[open-questions.md](open-questions.md) stays empty.

### Twenty-sixth edition — a block that had gone, a measure with two members, and two facts a command must print

The first arm is being built, and five statements it needs were either missing
from this corpus or had stopped being true. Three of them govern the one path a
calling agent meets most often — the message a wrong name produces. One is a
pair of facts a command must print and no observation had recorded. One is the
casing of a path segment, where two requirements in force each offered a
spelling and neither claimed the segment.

**Nothing here adds an exit code, withdraws a requirement or retires an
identifier.** Four requirements are added, four are amended, and two statements
this corpus makes about the repository are corrected.

- **A mandated test whose block had gone** —
  [schema-commands.md](schema-commands.md). `BR-SCH-004` mandates the dump
  round-trip test, and its *Accepted cost* said the test was blocked only by
  `tpl` not existing. The binary exists, so the note named no block at all, on
  a test this corpus obliges someone to write. What blocks it is that neither
  half of the round-trip is implemented: `tpl schema dump` and
  `tpl render --context` are both declared in the command tree and neither
  executes. The two halves belong to different arms, so the test becomes
  writable when the later of the two lands and not before. Recording the block
  as gone was rejected, and so was writing the dump half against a stored
  snapshot — that asserts a determinism property `NFR-DET-001` already owns,
  where this rule's subject is that a render from a dump and a render from a
  live read agree.
- **A suggestion with one worked cardinality out of three** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-008` shows one
  candidate, `FR-ERR-019` admits three, and how two or three are written was
  fixed nowhere, so the form was the implementation's rather than this
  corpus's. `FR-ERR-037` fixes it: every candidate inside the one `did you
  mean` question, quoted, `, ` between each pair but the last and ` or ` before
  the last, in the order `FR-ERR-019` gives. Separating every pair with `, `
  was rejected, because the caller must choose exactly one and ` or ` is the
  word that says so, on the line `FR-ERR-009` makes the one they act on; one
  line per candidate was rejected, because `FR-ERR-008` fixes the message at
  four labelled lines and `FR-ERR-024` escapes the newline in every
  interpolated value precisely so that nothing can forge a fifth.
- **A comparison that folds case, or does not** — the same file. Nothing said
  which, over a population of names a server owns, and the two readings build
  different programs. `FR-ERR-038` compares the characters as written and folds
  nothing; it names the two places this corpus does fold — `FR-SCH-014` for
  `--pattern` and `FR-ENV-031` for the word-list tokeniser — so that a reader
  arriving from either is told once, and it states the cost: a name differing
  in more than two letters' case alone is outside the threshold and is not
  offered, and the caller recovers it from the generic hint in one further
  invocation. Folding was rejected on two grounds. `--pattern` selects a set
  the caller then reads in full, where folding widens a listing; this is a
  ranking under a threshold, where folding changes which candidates are offered
  **at all**. And it would place a name differing only in case at distance
  zero, beneath an `error` line stating that the name does not exist.
- **A measure named by its family and not by its member** — the same file.
  `FR-ERR-019` admits a candidate by an edit distance of two, and the two forms
  of the Damerau-Levenshtein distance disagree inside exactly that threshold:
  `ca` reaches `abc` in two steps unrestricted and three restricted. The
  variant therefore decides whether a candidate is offered at all, not merely
  where it ranks, and it was recorded outside this corpus, as a technical open
  decision. `FR-ERR-039` names the restricted form — optimal string alignment.
  The unrestricted form was rejected: what it buys is the candidates in which a
  caller transposed two characters **and** edited between them, and what it
  costs is that the distance can no longer be computed from a bounded window of
  the comparison, on the path `BR-PERF-004` budgets at 200 names.
- **Two facts a command prints and no observation had recorded** —
  [catalogue-coverage.md](catalogue-coverage.md). `FR-SCH-009` has required
  `tpl schema table` to carry a table's engine, collation and comment since the
  first edition; nothing fixed the first two as model properties, neither had
  ever been observed in this repository, and `FR-CAT-053` — the index added one
  edition earlier to make a table's properties visible — showed it. It blocked
  the command outright. `FR-CAT-054` records both from the evidence: the
  catalogue field, its declared type, `InnoDB` and `utf8mb4_unicode_520_ci` on
  every covered table and SQL `NULL` together on every view row, observed on
  2026-09-18 against all four series of `FR-SRV-015` through the harness of
  `scripts/mariadb/`, with the conditions the reading was taken under and a
  bounded claim, because the fixture declares one engine and one table
  collation. Carrying neither was rejected: there is nowhere else the two can
  come from, and deriving the collation from the schema's default is the
  inference the seventh edition refused when it removed the table character
  set. `FR-CAT-053` gains the row and a fourth fragment; the table catalogue's
  own field list is still unrecorded and nothing waits on it.
- **A cache path with two plausible casings** —
  [cache-documents.md](cache-documents.md). `FR-CDOC-014` fixes
  `routines/<kind>.<name>.json` and fixed no casing for `<kind>`, while calling
  itself the same rule at two layers as `FR-SCH-008` — which, since the
  twenty-fifth edition, has a casing and it did not. A path builder could read
  `<kind>` from `FR-CAT-016`, in upper case, or from `FR-SCH-008`, in lower,
  and a cache written under one and read under the other neither collides nor
  fails: it **misses**, silently, at exit `0`, which is `FR-CACHE-033`
  behaving exactly as written. The requirement now fixes lower case and names
  `FR-SCH-008` as the rule it follows, so its two-layers claim is true of
  something a reader can check. Taking the casing from `FR-CAT-016` was
  rejected: that field carries the catalogue's own string because it is a value
  of a document a caller reads, and a path segment is a name this system
  composes.

**What found them.** Three instruments, and none is one of the five validation
rules below. `FR-ERR-037`, `FR-ERR-038` and `FR-ERR-039` were found by writing
the diagnostic that suggests a nearest match against this corpus — the
instrument that produced the twentieth, twenty-first, twenty-second,
twenty-fourth and twenty-fifth editions — which had to make three statements
this corpus did not. `FR-CAT-054` and the casing of `FR-CDOC-014` were found by
reading requirements in force against two things written one edition earlier,
the index of `FR-CAT-053` and the casing `FR-SCH-008` gained, which is the third
rule's lesson in a third setting: a statement that is correct about itself
establishes nothing about whether it is complete against what it points at. And
the block of `BR-SCH-004` was found by reading a note against the repository it
describes, which is the fifth rule's shape applied to a file this corpus does
not own — the note was true when it was written, and the repository moved under
it.

**Two statements about the repository are corrected with it**, both of them the
same clause. The *Maintenance debt* section below said of `FR-SRV-013` that it,
"like every test this corpus mandates", was blocked only by `tpl` not existing;
`DIV-036` said it of three mandated tests. Both are corrected, and neither by
deciding what blocks a test this edition did not read: what a mandated test
waits on is the command it drives, and `BR-SCH-004` is the one this edition
read.

**Four identifiers are assigned** — `FR-ERR-037`, `FR-ERR-038`, `FR-ERR-039`
and `FR-CAT-054` — and none is retired. No requirement is withdrawn, no open
question is raised or reopened, and the index of
[open-questions.md](open-questions.md) stays empty. One decision that had been
recorded outside this corpus is brought inside it, because it decides an
observable message and is therefore a requirement by the test
[Writing conventions](#writing-conventions) states.

### Twenty-seventh edition — four answers the first arm had to choose for itself

The first arm was built. Its eight `tpl schema` subcommands and the catalogue
cache beneath them landed at `db7337d`, and building them ran into four
questions this corpus does not answer. Each was resolved by a choice no
requirement authorises, each choice is now visible in code, and each was
reported rather than written in — which is what made this edition possible: the
four are answered here, and the code follows.

**Three requirements are added, thirteen are amended, and no identifier is
retired.** No exit code is added or withdrawn, no requirement is withdrawn, and
the index of [open-questions.md](open-questions.md) stays empty.

- **A catalogue value outside a closed enumeration was refused** —
  [catalogue-coverage.md](catalogue-coverage.md). Five value sets are fixed
  from an observation of all four series — the referential action of
  `FR-CAT-045`, the level of `FR-CAT-046`, the event and the timing of
  `FR-CAT-050`, the kind of `FR-CAT-016` — and the model has a shape for the
  recorded values and no other. The one foreseeable trigger for a sixth is a
  server newer than the window, which `FR-SRV-031` says shall be **read and
  marked**. A refusal there contradicts the promise outright, and it does it
  with `70`, which `FR-ERR-030` closes to a panic and to an invariant the
  system detects **in itself**. `FR-CAT-055` carries the catalogue's own string
  unchanged instead, changing no exit code, and states the one consequence that
  reaches beyond the document: a routine kind outside the two is not reachable
  by the qualified name of `FR-SCH-008`. `FR-CAT-016` is amended with it,
  because it said the value SHALL "therefore" be one of two strings — true of
  every supported series and not a property of a field the catalogue declares
  `varchar(13)`. Refusing was rejected for the two reasons above; substituting
  the nearest recorded value and dropping the object were rejected because both
  are wrong documents at exit `0`.
- **Six fields the catalogue declares nullable had no shape for an absent
  value** — the same file. `FR-CAT-056` records the declaration of each, over
  all four series, and fixes what the model carries: `null`, per `FR-CTX-005`
  and `FR-OUT-012`, for a trigger's definer and statement, a routine's body,
  and a foreign key's referenced key and referenced table; and, for a key
  column's referenced column, the observation that the read's own population
  excludes every row on which the field is absent. Substituting the empty
  string was rejected — it is what the first implementation does for five of
  the six, and it makes an absent value indistinguishable from a present empty
  one, which is the distinction `FR-PRIV-011` depends on and which matters most
  for the routine body, where SQL `NULL` is the missing privilege of
  `FR-PRIV-017`. Declaring the five `NOT NULL` on the strength of the fixture
  was rejected as the inference the fourth [provenance](#provenance) forbids,
  pointing the other way. `FR-CTX-006` gains the one structural case: a key
  that names no table has no first hop.
- **The cross-schema foreign key is excluded, and the observation that would
  have recorded it could not be made** — the same file. `FR-CAT-057` excludes
  such a key in both directions and forbids reading a second schema to carry
  one, because `FR-CTX-006` and `FR-CTX-010` embed the table at **each** end in
  full and `FR-CTX-023` requires every referenced object to be present. Reading
  the referenced schema was rejected against `FR-CONF-041` and `FR-PRIV-001`;
  carrying the key as a bare name was rejected as the depth that adapts to the
  graph `BR-CTX-001` refused; refusing the read was rejected as a loss larger
  than the one it prevents. The exclusion is a decision and needs no
  observation — but the attempt is recorded with it, because the requirement
  would otherwise look like one that had been observed. The fixture declares
  **one** user schema and carries no cross-schema key, verified on 2026-09-20
  against all four series, and the DDL that would settle the behaviour is named
  in the requirement's *What would change this*.
- **`FR-PRIV-019` said "the table" and its antecedent named two** —
  [privileges-and-completeness.md](privileges-and-completeness.md). A key column
  that names a referenced table under no referential-constraint row costs the
  **referencing** table its `foreign_keys`, per `FR-CAT-012`, and the
  **referenced** table its `referenced_by`, per `FR-CAT-013`. Both are marked.
  `FR-PRIV-002` is unconditional, and marking the referencing end alone would
  present an empty `referenced_by` as a complete answer at exit `0` — the exact
  failure this file exists to prevent, arriving from the end nobody was looking
  at. Marking either end alone was rejected; a single marking on the document
  was already forbidden by `FR-PRIV-006`. The accepted cost is stated: one lost
  rules row marks two objects, which over the fixture is 14 marked tables for
  15 lost rules.
- **Two populations wore one number** — the same file. `FR-PRIV-018`'s counts
  and `FR-PRIV-019`'s *all 54 rows* are taken over the **unfiltered**
  key-column table, while the read `FR-CAT-045` specifies sees **17** — the rows
  naming a referenced table. Both numbers are correct over the population each
  was taken over, and nothing said which. `FR-PRIV-018` now names the
  population beside every count: the key-column table has two, and every other
  catalogue table this file names has one.
- **A rationale argued against the only implementation its own corpus allows** —
  [performance-requirements.md](performance-requirements.md). `NFR-PERF-002`
  fixes a **statement count** and its rationale named whole-catalogue *reading*
  as the defect, which `FR-CTX-006`, `FR-CTX-010` and `FR-CTX-023` between them
  make unavoidable: a read returning one table's rows returns that table's keys
  without the tables they name, and no document can be built from it. The
  requirement now states that the rows MAY be the whole catalogue, replaces the
  rationale rather than softening it, and states what a narrow read would have
  to return so that the option stays open — every table at either end of one of
  the named table's keys, in full, and no further, because `FR-CTX-008` cuts at
  the first hop. Obliging a narrow read was rejected: the neighbours are not
  known until the key rows have been read, and choosing between a second
  dependent round and a self-join is an architecture decision this corpus does
  not make.
- **Two commands were emitting the same bytes** —
  [schema-commands.md](schema-commands.md). `FR-SCH-031` read `info`'s `data`
  as the `database` object, and the fifth edition's amendment had put the three
  collections of `FR-CTX-035` into it — which is the whole of the model, so
  `tpl schema info --format json` emitted, byte for byte, what
  `tpl schema dump` emits. The step was mechanical and its consequence was
  never stated. `FR-SCH-031` now fixes four members and no others — `name`,
  `charset`, `collation` and the `server` object — says in its own text that
  the two commands SHALL NOT emit the same bytes, and states how the reduced
  object relates to the one `FR-CTX-001` fixes: every member it carries is that
  member, under that name, with that value. `FR-CTX-001`, `FR-CTX-035` and
  `FR-CTX-036` are unchanged, and `FR-CTX-035` names the one reduction so that
  a reader arriving from either file is told once. Keeping the collections and
  accepting the byte-identity was rejected — it costs a caller the whole model
  to ask a database's name, on the command whose help line reads *Database
  metadata*; inventing a count field was rejected as surface no requirement of
  [catalogue-coverage.md](catalogue-coverage.md) fixes; a different key was
  rejected because it would cost the one property the reduction preserves.

**What found them.** Not one of the five validation rules below. All four
questions were raised by **writing the first arm against this corpus** — the
instrument that produced the twentieth, twenty-first, twenty-second,
twenty-fourth and twenty-fifth editions, reaching further than it had before
because this time the arm was finished. Each of the four is a place where two
requirements in force gave an implementer two readings and no rule, which is
the shape no rule that re-reads a register can find. The second and third rules
did the rest of the work once the questions were in hand: `FR-CAT-055` and
`FR-CAT-056` cite the observation that establishes them, and `FR-PRIV-018` is
assembled from the populations its numbers were taken over rather than from the
numbers.

**One observation occasion, and one observation that could not be made.** The
four images of `scripts/mariadb/` were run on 2026-09-20, through the harness,
as the privileged reader and as the reduced-grant reader in turn. It recorded
the declared type and nullability of every field the three new requirements
name; the two populations of the key-column table, filtered and unfiltered; and
the schema population of each server. It re-confirmed difference 9 of
`FR-SRV-038` — the trigger event column at `varchar(20)` on `12.3` and
`varchar(6)` on the other three — which is now cited by `FR-CAT-055` as
evidence that a supported series has already widened a field behind a closed
set. It adds **no row** to the record of `FR-SRV-038` and **no row** to the
register of `FR-SRV-036`: every difference it met was already recorded, and
every field it names was already carried. What it could not observe is the
cross-schema foreign key, for the reason `FR-CAT-057` states.

**Three identifiers are assigned** — `FR-CAT-055`, `FR-CAT-056` and
`FR-CAT-057` — and none is retired.

### Twenty-eighth edition — a test with no route to half of what it demanded

The first arm's verification from outside the process was written, and it met a
requirement it could not satisfy. `FR-SRV-013` demanded **one integration
test** exercising **both** outcomes of the read-back of `FR-SRV-009`, and
neither route to the failing outcome existed. No server produces it: a server
that accepts the read-only session statement and does not apply it is the case
`FR-SRV-009` exists to catch, and no supported MariaDB behaves that way, which
three attempts against the fixture confirmed. No admissible seam reaches it
either: a seam on `FR-ERR-031`'s terms is reachable only from within the
system's own test configuration, and an integration test drives the distributed
binary, which carries no such seam. The confirming half was delivered, on all
four series, observed from the server; the other half was reported rather than
faked, which is what made this edition possible.

**Nothing here changes what `tpl` does.** One requirement is amended, one rule
yields for one clause, one count follows it, and **no identifier is assigned or
retired**. No requirement is withdrawn, no exit code is added or withdrawn, and
the index of [open-questions.md](open-questions.md) stays empty.

- **`FR-SRV-013` is split by the test form that can reach each outcome** —
  [server-contract.md](server-contract.md). The confirming outcome keeps the
  integration test, keeps the tenth edition's binding to every series of
  `FR-SRV-015`, and is observed on the server. The failing outcome is verified
  **in process**, through a seam the requirement now authorises in its own
  text, on `FR-ERR-031`'s terms — reachable from no invocation of the
  distributed binary and absent from every published surface — which presents
  the read-back with an answer that does not confirm the setting, so that the
  test can assert the condition of `FR-SRV-010` and that no catalogue statement
  follows. What yields is the **form** of that half's test and nothing else.
  This is `FR-SRV-035`'s resolution of the eighth edition, in the same shape
  and for the same collision: name the seam, name the test form, and state what
  the form does not establish.
- **`BR-SRV-003` states the exception in its own text** — the same file. That
  rule names `FR-SRV-012` through `FR-SRV-014` and requires all three to be
  observed outside the process, on the server, so `FR-SRV-013` is inside it and
  the answer could not be left to inference. It yields for the failing outcome
  alone, in the shape `BR-ERR-001` uses for `70`. It is unchanged over
  `FR-SRV-012`, over `FR-SRV-014`, and over the confirming outcome — which
  carries the whole of what `FR-SRV-013` promises about the statement the
  process **sends**. What yields is the verification of what the process does
  with the answer it **receives**, and no server can show that.
- **The count in `BR-SCH-004`'s precedent list follows** —
  [schema-commands.md](schema-commands.md). It said `FR-SRV-012` and
  `FR-SRV-013` "mandate two for the read-only promise", one test each. They
  mandate three. Only the count changes, and nothing about the round-trip that
  rule mandates or what it still waits on.
- **`FR-SRV-013` joins the requirements that state a limit on evidence** —
  *[Writing conventions](#writing-conventions)*. It carries the note shapes
  that family uses and states what its in-process half leaves unobserved: no
  invocation of the distributed binary is observed refusing on a read-back that
  did not confirm, and no server is observed producing one. The enumeration
  there goes from three to four.
- **The tenth edition's entry above is corrected where it spoke in the present
  tense.** It said the test "now runs against every series", over the whole of
  `FR-SRV-013`. The binding attaches to the half that reaches a server, and the
  entry now says so. The binding itself is undiminished.

**One observation, and one that cannot be made.** The three fixture conditions
were tried on 2026-09-20, through the harness, by the pass that wrote the
confirming half — an open transaction before the read-only statement, the same
as the reduced-grant reader, and the global read-only flag — and under each the
read-back still confirmed the setting. The first was tried on all four series.
`FR-SRV-013` records them with their bound, and records the one further
candidate that was reasoned against rather than observed, so that it is not
tried again. It adds **no row** to the record of `FR-SRV-038` and **no row** to
the register of `FR-SRV-036`: a condition no server enters is not a difference
between the series, and no field of the model is in question. What cannot be
observed is the failing outcome itself, for the reason `FR-SRV-013` states.

**What found it.** Not one of the five validation rules below. The instrument
was **writing the verification against this corpus** — the outside-the-process
observation the first arm owes — which is the instrument of the twentieth,
twenty-first, twenty-second, twenty-fourth, twenty-fifth and twenty-seventh
editions, turned for the first time on the tests rather than on the commands.
The eighth edition met this defect's twin by reading requirement against
requirement, and did not meet this one: `FR-SRV-035` authorised a seam and
demanded an integration test in the same breath, so its collision was on the
page, while `FR-SRV-013` authorises no seam at all and its two halves read as
one coherent demand until somebody has to produce the second. A requirement
that mandates a test can be internally coherent, correctly cross-referenced,
counted right and impossible to satisfy, and the reading that finds it is an
attempt to write the test.

**Three options were rejected and each is recorded in `FR-SRV-013` with its
reason.** Withdrawing the failing half, which would leave the branch of
`FR-SRV-010` that this read-back decides with no test of any kind, on
`BR-ERR-001`'s ground for `70`. Naming a fixture condition, which no
configuration of a conforming server can supply, because the outcome is a
defect in a server rather than a state a server can be put into. And a stand-in
between the reader and the server rewriting the read-back's answer, refused on
`FR-SRV-035`'s ground — an impostor of the wire protocol, maintained across
four series, for one assertion — and because an observation made against an
artefact this project wrote satisfies `BR-SRV-003`'s letter while abandoning
its substance.

### Twenty-ninth edition — a boolean the engine spelled with a capital, and a rationale its own table disproved

The two arms were joined — a context assembled, a template rendered from it —
and the first thing the joined tool printed for `{{ column is nullable }}` was
`True`. No requirement of this corpus said otherwise, so the engine decided it.
`tpl` generates code, and `True` is a token Rust, Go, JSON and SQL all refuse:
the tool's default output for one of its own registered tests was a token that
does not build. The user decided on 2026-09-21 that it must not be, and
this edition writes the decision in; it is the seventh decision log of
*[Provenance](#provenance)*. The same pass read `FR-ENV-033` back against its
own table and found its rationale claiming an identity the table disproves.

**One requirement is added and one rationale is corrected.** `FR-SEM-021` is
the only identifier assigned; none is retired and none is renumbered, no exit
code is added or withdrawn, no cell of any table in this corpus moves, and the
index of [open-questions.md](open-questions.md) stays empty.

- **A boolean interpolated into the output renders `true` or `false`** —
  [render-semantics.md](render-semantics.md). `FR-SEM-021`, placed beside
  `FR-SEM-010` and for that rule's reason: what an interpolated value produces
  is render semantics, and not one of the names the three groups of
  `FR-ENV-001` partition, so nothing in
  [template-environment.md](template-environment.md) changes with it. The
  requirement forbids `True`, `False`, and every other casing. Leaving the
  rendering to the engine and obliging the template author to write `| lower`
  or `| json` at each interpolation is rejected in the requirement's own text:
  that option carries the guarantee group 3 carries, which is none, and the
  author who forgets the filter receives no signal, because the render succeeds
  and the wrong token reaches the generated file.
- **`FR-ENV-033` stops claiming an identity its own table disproves** —
  [template-environment.md](template-environment.md). The rationale said
  `snake(pascal(x))` returns `snake(x)` for every `x`. It does not for
  `order_2_items`, which is a row of that requirement's table: `pascal` gives
  `Order2Items`, a single word under `FR-ENV-030`, so `snake` gives
  `order2items`. The table is right, keeps every cell, and `BR-ENV-007` is
  untouched — the prose was the only thing wrong. The claim is dropped rather
  than qualified, because the condition under which it holds turns on digits at
  a boundary and on single-letter words alike — `a_b` gives `AB` and then `ab`,
  while `a_bc` survives as `ABc` and then `a_bc` — so a qualification short
  enough to read would have replaced a false claim with one a reader could
  carry just as far. What stands in its place is the rule: derive the word list
  of the operand the filter is handed.

**What found them.** Not one of the five validation rules below. Both came from
**running the joined tool**, which is the first reading of this corpus made
against output a user can see rather than against another document. The first
is invisible to every check this corpus can run on itself, because no
requirement was wrong: what a boolean renders as was unwritten, and an absence
contradicts nothing — it is found by looking at the bytes, or not at all. The
second was on the page from the moment the requirement was written, and the
readings of that file since passed over it, because the sentence that was false
sat directly beneath a table that was right and read as that table's summary. A
rationale is prose, and prose beside a test vector is read as description of
it; this one made a claim the vector refutes in a row the reader has just
scanned.

### Thirtieth edition — a condition decidable from the project, evaluated after the catalogue

`tpl render nosuch --table orders` against a live database read the whole
catalogue and filled the store before refusing on a template name that could
never have rendered. Nothing was diverging: `FR-ERR-006` put template
resolution seventh, after the entry, the cache and the catalogue, and the
implementation did exactly that. What had never been asked is whether that
position is the one the performance family wants, and the certification of the
write path on 2026-09-21 supplied the facts the question needed — that the read
path has no partial form, and that every fallible step runs before the single
write of a document already whole. This edition puts the question and answers
it.

**No identifier is assigned, none is retired, and no requirement is added or
withdrawn.** One step of one requirement changes position; the step count, the
codes and every other requirement stay as they were, and the index of
[open-questions.md](open-questions.md) stays empty.

- **Template resolution moves from the seventh position to the fourth** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-006` resolves a
  template name immediately after `.tpl/.cfg`, and before the entry, the cache,
  the connection and the catalogue. The condition needed nothing the old
  position gave it: `FR-TMPL-023` makes the template root a property of the
  resolved project and `FR-TMPL-003` keeps a template name away from an entry,
  a cache and a connection, so it was decidable as soon as step 2 had run —
  the ground on which `FR-SCH-008` already placed one condition at step 1. What
  the old position cost was everything an invocation that cannot render paid
  before being told so, which `BR-PERF-004` says a wrong invocation must not
  pay, and whose connection `NFR-PERF-006` obliges a command needing no
  catalogue data not to open. It remains one order for every command:
  `tpl render` is the only command that reaches both sides of the move, and
  `FR-ERR-007` therefore hands a doubly faulted invocation the other of its two
  codes. The requirement records the two cases that change as an accepted cost,
  beside the one visible outside the process — a producer behind `--context -`
  is now cut off rather than drained.
- **Eight sentences numbered a step that has moved, and each is corrected** —
  two in `FR-ERR-006`'s own notes, two elsewhere in
  [errors-and-exit-codes.md](errors-and-exit-codes.md), two in
  [configuration-model.md](configuration-model.md), and two in the edition
  records of this file. Each now names the step it means, or drops a number it
  never needed. Numbering a step in prose is what let one move falsify eight
  sentences, and the four steps of `FR-CFG-024`, which are its own and not
  these, are untouched.
- **`NFR-PERF-006` is checked and left as written** —
  [performance-requirements.md](performance-requirements.md). Its obligation —
  a command requiring no catalogue data opens no connection — is what the new
  position lets a render whose template does not exist satisfy, and the clause
  naming what that requirement covers does not name this invocation. The clause
  enumerates and the obligation governs, so nothing in that file is false and
  nothing in it changes here.

**What found it.** Not a reading of this corpus against itself: the order, every
requirement it orders, and the code all agreed with one another. It was found by
asking from the performance side, rather than the correctness side, what an
invocation costs when it cannot succeed — and it was answerable only because the
write path had been certified the same day.

### Thirty-first edition — the backlog read back, and fourteen statements nobody had made

Sprint 17 was opened to clear a backlog thirty-one tasks deep, set aside while
the three arms were built, and fourteen of those tasks were questions for this
corpus. They are not one finding. They arrived from the first derivation of the
diagnostic renderer, from the parser, from the help renderer, from a
measurement taken across a pipe, from a run of `tpl init`, and from reading
this corpus back against a repository that has moved under it. What they share
is a shape: in each of the fourteen a requirement in force was silent, or
stated wider than the ground it gives for itself, or true when it was written
and not now. They are answered here in one pass over the corpus rather than in
fourteen visits, because nine of the fourteen land in three files —
[errors-and-exit-codes.md](errors-and-exit-codes.md),
[global-flags.md](global-flags.md) and this one.

**Six identifiers are assigned, none is retired and none is renumbered** —
`FR-CLI-025`, `FR-GLOB-025`, `FR-ERR-040`, `FR-HELP-030`, `FR-OUT-038` and
`FR-RND-035`. No code is added to the table of `FR-ERR-001` or withdrawn from
it; one cell of that table is widened, and one row of `FR-ERR-034` gains a
fourth population. The index of [open-questions.md](open-questions.md) stays
empty.

**Two of the fourteen change what `tpl` does.** A valueless flag written twice
is accepted where the parser then refused it, and every flag and every
positional argument must now say what it does in its help. Everything else
states what was already in force, qualifies a rule that reached further than
its ground, or corrects a sentence that had stopped being true.

- **The `cause` of a failed `password_command` names the command, and two
  general prohibitions yield to it** —
  [configuration-model.md](configuration-model.md),
  [global-flags.md](global-flags.md), [security.md](security.md) and
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-CONF-033` obliged
  the `cause` to name the command as stored while `FR-GLOB-018` barred the
  `password_command` from every diagnostic stream and `BR-ERR-003` barred the
  contents of `.tpl/.cfg` from every error message. `FR-CONF-033` governs: it
  is the specific rule, the array is the one part of an entry whose purpose is
  **not** to hold a credential, and without it the `cause` names a category
  where an instance is available. `FR-GLOB-018`, `FR-SEC-005` and `BR-ERR-003`
  now state the one exception, over the array wherever a requirement obliges a
  `cause` to name it — which reaches `FR-CONF-031` and `FR-CONF-028` as well.
  The residual is **accepted, with its ground**: `FR-CONF-017` keeps `${VAR}`
  out of the array, so the environment cannot inject a secret, and a caller who
  writes one there literally sees it printed. The implementation's reading was
  correct and nothing in `src/diagnostics/cause.rs` is owed a change.
- **A valueless flag given twice is accepted, with the effect of one
  occurrence** — [cli-contract.md](cli-contract.md). `FR-CLI-025`, placed
  beside `FR-CLI-014` and not inside it, because that requirement's ground is
  two values disagreeing and a valueless flag has none. Six flags are reached —
  `-q/--quiet`, `--pretty`, `--direct`, `--no-cache`, `-h/--help` and
  `-V/--version` — and `-v/--verbose` is excluded, because `FR-CLI-016` counts
  its repetitions. Refusing with `64`, which is what the parser then produced
  and produced by accident, is rejected in the requirement's own text: the
  meaning of `tpl -q -q` is not in doubt to anybody, and `tpl -h -h` refuses
  the very path a caller uses to recover from a refusal. `FR-GLOB-015` carries
  the cross-reference where the flag is declared.
- **A failed rewrite of `.tpl/.cfg` is `74`** —
  [cfg-commands.md](cfg-commands.md) and
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-CFG-041` named the
  failure and no code; it now names `74`, and the row that carries it is the
  `74` row of `FR-ERR-001`, whose cell is widened from *reading `.tpl`* to the
  class it characterises. `73` is rejected because `FR-ERR-003` reserves it for
  `tpl init` and the destination here exists; `78` because the file is intact
  and the message would send the caller to correct what is correct. The `74`
  the sixth sprint wrote into the help of the five `cfg` writers is
  **confirmed**, and nothing in that help changes.
- **`FR-GLOB-007` is qualified as `FR-GLOB-006` is qualified** —
  [global-flags.md](global-flags.md). `tpl -d nope cfg list` is `0`, and
  `FR-GLOB-007` as amended decides it. `FR-GLOB-025` names the commands that
  require an entry — the eight `schema` subcommands, `tpl render` without
  `--context`, and the three `cache` subcommands — so both requirements read
  one set from one place. The case that settles it is
  `tpl -d nope cfg database add nope --host h`, which the literal reading
  refuses for the absence of the entry it exists to create. The reading written
  into roughly thirty commands' help is **confirmed**, and `FR-ERR-006` now
  says that a step whose condition an invocation does not raise is not a step
  it skips — which is the contrast that let the literal reading stand.
- **A flag and an argument say what they do** —
  [help-and-version.md](help-and-version.md). `FR-HELP-013` enumerates six
  facts about a **value**, and a renderer satisfying it emits
  `Type: string. No default. Optional. Not repeatable.` for `-d/--database`
  without saying that the flag selects a `[database.<name>]` entry.
  `FR-HELP-030` adds one sentence of purpose, held in the typed table of
  `FR-HELP-022` — which is where it must live, because the declarations' doc
  comments name requirement identifiers that `FR-HELP-014` bars from help. It
  reaches the seven global flags `FR-GLOB-003` lists once at the root, and it
  is already satisfied by the one-line summaries `FR-HELP-008` puts beside a
  node's children. The cost is one line per flag at the one place that flag is
  declared, which is not the multiplication `BR-HELP-002` and `FR-HELP-007`
  refuse.
- **A separate-token flag value in a `hint` is governed, by a set of its own** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-022`'s closing
  clause reaches it and `FR-ERR-022`'s set cannot hold it: every value reaching
  `FR-CLI-018` begins with `-`, so `[A-Za-z0-9_]{1,64}` refuses all of them and
  the corrected form could never show a value. `FR-ERR-040` admits
  `[A-Za-z0-9_-]{1,64}` over the **whole** value, which is the half the
  admission test could not enforce — it bounds each `-`-separated segment and
  so bounds nothing, as `tpl -d -a-a-a-a-a-a-a-a version` shows. Widening
  `FR-ERR-022`'s own set is rejected on the twentieth edition's ground, and
  `FR-CLI-018` is satisfiable either way and says so.
- **`tpl init` looks upward because a requirement obliges it to** —
  [project-and-discovery.md](project-and-discovery.md) and
  [performance-requirements.md](performance-requirements.md). `FR-PROJ-016`
  requires the shadowing warning, `FR-PROJ-025` and `NFR-PERF-005` forbade the
  discovery that produces it, and `NFR-PERF-007` wrote a differential
  arrangement asserting the warning is not emitted. `FR-PROJ-025` is the
  requirement that yields, and it now states what its discovery clause forbids:
  no project above the invocation decides its outcome. `NFR-PERF-005` states
  the file-open observable per command, so that the one command that walks its
  ancestors is not described as one that does not; and `NFR-PERF-007`'s
  arrangement now asserts the three things the differential run actually reads
  — the exit code, stdout, and the artefacts on disk — where it asserted the
  absence of a warning on a stream that instrument does not observe. The code
  at `src/project/init.rs`, observed on 2026-09-18, conforms.
- **Two statements the twenty-third edition made false** —
  [upstream-divergences.md](upstream-divergences.md) and
  [context-document.md](context-document.md). `DIV-034` claimed that
  `FR-CAT-039` through `FR-CAT-051` fix the field list of every object kind; it
  now says *every object kind an entry of
  [open-questions.md](open-questions.md) had asked for* and names `FR-CAT-053`
  for the table, which is outside the range. `FR-CTX-035` sent a reader to
  "`catalogue-coverage.md` and the open questions it carries", an index that has
  been empty since the seventh edition; it names the file alone.
- **A sixth validation rule, for a negative observation** — under
  *[Maintenance debt](#maintenance-debt)*. A record that something is absent
  must rest on a form that would fail on a wrong question. It is the rule none
  of the five would have found the defect of `FR-SRV-040` with, and it is of
  the first and third rules' family rather than the fourth and fifth's, which
  is why the nineteenth and twentieth editions' ground for declining a sixth
  does not reach it.
- **`FR-SCH-009`'s note counted three places and named two** —
  [schema-commands.md](schema-commands.md). The count is dropped rather than a
  third place found, and the sentence now reads as
  [catalogue-coverage.md](catalogue-coverage.md) states the same fact below the
  table of `FR-CAT-054`.
- **The *Provenance* section counted three sources above a list of five** —
  below. It meant three when it was written, and the fourth and sixth editions
  each added a source without moving the count. The opening says **four**, the
  fifth item is named as the rule that closes the list, and **no item is
  renumbered**, because seven passages of this corpus cite *the fourth
  provenance* and every one means item 4.
- **Four facts the render path left unsettled** —
  [output-formats.md](output-formats.md),
  [errors-and-exit-codes.md](errors-and-exit-codes.md),
  [render-command.md](render-command.md) and
  [upstream-divergences.md](upstream-divergences.md). `FR-OUT-038` says what
  `FR-OUT-014`'s *Breaking* column obliges — `schema_version` moves by one, and
  the obligation binds from the **first release**, so the flattening of
  `Column::column_type` this sprint correctly left it at `1` and the six files
  carrying `schema_version: 1` in a worked example are untouched. The `66` row
  of `FR-ERR-034` gains the `--context` document as a fourth population, which
  `FR-RND-032` had always produced. `FR-RND-035` fixes `74` for a `--context`
  file or stream that cannot be read and `FR-RND-020` fixes `65` for bytes that
  are not UTF-8, on RFC 8259's ground; both confirm what the code chose. And
  two statements about the repository are corrected: `Cargo.toml` and
  `CHANGELOG.md` both exist, verified in the working tree on 2026-09-21.
- **The performance family, and work on a doomed invocation** —
  [performance-requirements.md](performance-requirements.md) and
  [cache-commands.md](cache-commands.md). `NFR-PERF-006`'s coverage clause now
  names the invocation the thirtieth edition created, a `tpl render` whose
  template does not resolve; the thirtieth edition checked that clause and left
  it, and an enumeration short by exactly the case the edition before it
  decided from is the enumeration worth lengthening. `NFR-PERF-014` gains **no**
  live-database row: what one would measure is the catalogue read, which
  `NFR-PERF-001` governs as a form and the `tpl schema dump` row as a figure.
  `FR-CACHE-007` keeps its ordering and records why — what the store holds is a
  correct read of the server, the author's next act is to fix the template and
  render again, and an exemption would spend a correct read to make them pay
  for it twice.
- **`FR-ERR-026` is reached only when bytes are already in flight** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). The thirtieth edition's
  note gave the condition as *where its document does not fit the pipe buffer*,
  which is necessary and not sufficient. Measured with the fixture up, eight
  runs of eight of `tpl schema dump --direct --no-cache | tpl render nosuch
  --context -` gave `producer=0 consumer=66 pair=66`: the consumer refuses in
  about 3 ms, at the step the thirtieth edition moved, before the producer has
  written a byte. The note is corrected, and `FR-ERR-025` and `FR-ERR-026`
  needed the same qualification rather than only the note — a document is **in
  flight** from its first byte on stdout to its last, and that line divides the
  two codes.

**What found them.** Not one of the six validation rules, and not one reading.
Five came from deriving an implementation from this corpus and finding it
silent — the `cause` of a failed child, the code of a failed `.cfg` write, the
qualifier on `-d`, the purpose of a flag, and what a breaking change obliges.
Three came from running the joined tool and looking at what it did: the
parser's refusal of `-q -q`, the walk `tpl init` makes, and the unbounded value
in a `hint`. One came from **measuring** rather than reading, across a pipe,
and it corrected a condition the edition immediately before had stated
confidently and loosely. The remaining five came from reading this corpus back
against something that had moved beneath it — a range of identifiers and an
index that emptied, two counts nobody recomputed, a finding read back for what
would have caught it, and a performance family read back against a step the
thirtieth edition moved. That last group is the one the fifth validation rule
exists for, and it is the group that keeps arriving.

### Thirty-second edition — a register paid, two conditions nobody owned, and a key that skipped its own convention

Sprint 17 was opened to clear a backlog thirty-one tasks deep, and the
thirty-first edition answered fourteen of them in one pass. Four remained,
three of them findings that edition recorded and deliberately did not act on.
They are answered here in one pass for the reason that edition gave: two of the
four land in the same file, and the two corpus-wide rules the others set — one
over where a defined term lives and one over when a register decays — are read
together or not at all.

**Three requirement identifiers are assigned, none is retired and none is
renumbered** — `FR-CONF-042`, `FR-CONF-043` and `FR-CONF-044`, all in
[configuration-model.md](configuration-model.md). Two entries are raised in
[upstream-divergences.md](upstream-divergences.md), `DIV-053` and `DIV-054`,
whose sequence is its own. No code is added to the table of `FR-ERR-001` or
withdrawn from it, and no cell of it moves. The index of
[open-questions.md](open-questions.md) stays empty.

**Two of the four change what `tpl` does**, and both are implemented by the
Rust delegation that follows this edition. `ca_path` follows symbolic links,
and a `ca_path` that supplies nothing is refused rather than ignored. And two
`cause` lines change: the one for a `password_command` ended by a signal now
names the signal, and the one for a child whose status could not be read stops
saying the child could not be started. The other two put a definition where
this corpus says definitions go, and bring a register back into agreement with
two files that had moved under it.

- **Two `password_command` conditions the corpus did not reach** —
  [configuration-model.md](configuration-model.md). The diagnostic renderer at
  `src/diagnostics/cause.rs` produces a message for a child that could not be
  started and for a child ended by a signal, and neither fell inside
  `FR-CONF-028`, `FR-CONF-031` or `FR-CONF-033`, which `FR-ERR-002` says must
  not happen. `FR-CONF-042` governs the first and `FR-CONF-043` the second,
  both at `78`, which is what the implementation already chose and what puts
  them with the three conditions beside them: the configured way of obtaining a
  password failed to produce one, which is a fault in `.tpl/.cfg` and not in
  the network or the credentials. `77` is rejected in both — it would tell the
  caller a server refused an authentication that was never attempted — and `70`
  in the first, which would name a defect in `tpl` for a program the caller
  chose. Two things follow that the code does not yet do. `FR-CONF-043` obliges
  the `cause` to name **the signal**, because `FR-ERR-034` bans a `cause` that
  names a category where an instance is available and every target of
  `NFR-PERF-018` is a Unix; and it excludes the signals `tpl` itself sends,
  because `FR-CONF-028` and `FR-CONF-031` terminate the child and own their own
  outcomes, so without the clause the requirement would swallow two conditions
  in force. The exception the thirty-first edition wrote over the
  `password_command` array reaches both without amending `FR-GLOB-018`,
  `FR-SEC-005` or `BR-ERR-003`, because it was stated over the array wherever a
  requirement obliges a `cause` to name it.
- **`ca_path` follows symbolic links, and a `ca_path` that yields nothing is
  refused** — [configuration-model.md](configuration-model.md). `FR-CONF-014`
  had one clause and answered neither question. A `CApath` directory in
  OpenSSL's and MariaDB's sense is conventionally a set of hash-named symbolic
  links, and `trust` at `src/mariadb/connect.rs` **then** selected entries on
  `DirEntry::file_type`, which does not traverse one — so the arrangement the
  key exists to serve yielded an empty bundle, silently. The weighing is written
  into the requirement: this corpus states links twice and in opposite
  directions, refusing them inside `.tpl/templates/` under `FR-TMPL-024`
  because that directory is versioned, shared and printed, and following them
  at `.tpl` under `FR-PROJ-009` because following is what makes the ownership
  and mode checks land on the real file. `ca_path` has the second shape: the
  directory is named by a `.tpl/.cfg` that is the caller's alone at mode
  `0600`, what is read is certificates, and no byte of the bundle is ever
  printed. `FR-CONF-044` answers the second question at `78`, because a
  declared key that contributes nothing is a fault in the file and its remedy
  is the file. A **warning** was the close alternative and is rejected in the
  requirement's own text — stderr is not contract, `-q` lowers it out of sight,
  and the caller redirecting it into a build log is the caller this condition
  exists for — and `69` and `74` are rejected for naming a server and an I/O
  failure that are not at fault. The order is untouched, and the determinism
  with it; the thirty-fourth edition moved the statement of that order out of
  the requirement and into the record that owns it.
- **A term defined in two places, and the rule being broken was the glossary's
  own** — [glossary.md](glossary.md),
  [errors-and-exit-codes.md](errors-and-exit-codes.md) and
  [performance-requirements.md](performance-requirements.md). That file opens
  by saying a term used in a requirement without being defined there is a
  defect, and two terms were defined elsewhere: *differential run* beside
  `NFR-PERF-007` since the eleventh edition, and *in flight* beside `FR-ERR-025`
  and `FR-ERR-026` since the thirty-first, which followed it. **The glossary
  governs**, and both are moved into it with the two requirements citing them.
  The alternative — a term defined where it is used, with the glossary carrying
  a pointer — was live, because both are load-bearing at the point of use, and
  is rejected on three grounds stated at the head of that file: it turns an
  absolute into a two-case rule whose new case has no test, it legitimises the
  drift that produced the defect, and it is the weaker form of one copy where
  the file already has the stronger one in `budget`, `provisional figure` and
  `cache hit / cache miss` — the first two renamed by the thirty-sixth edition
  to `measurement point` and `reference figure`, which is where a reader looking
  for them now finds the same shape. **Nothing about `tpl` changes**, and neither
  requirement is amended: what moved is where the sentence lives. One
  cross-reference follows it, in [output-formats.md](output-formats.md), which
  named the two requirements as defining the term.
- **Fifty-two entries re-read, and forty-five of them are discharged** —
  [upstream-divergences.md](upstream-divergences.md). The obligation the
  thirty-first edition recorded and did not pay is paid. Every entry was read
  against `README.md` at `db80114` and `CLAUDE.md` at `8f936d4`, in both
  directions. **Thirty entries are discharged and each names the commit.**
  `e75996c` is what `0ea5624` was to the other file: it rewrote `README.md`
  whole, and twenty-eight of the thirty name it. With `DIV-034`, which the
  thirty-first edition discharged, the `README.md` half of `DIV-037` and two of
  the four parts of `DIV-001`, it discharges something in thirty-one entries.
  The asymmetry the fifteenth edition found has reversed:
  thirty-two corrections were owed to `README.md` and three are, while six are
  owed to `CLAUDE.md`. **Two
  divergences the register did not hold are raised**, and each says something
  the register had not had to say. `DIV-053` is the sentence of *Discovery*
  saying four commands perform no discovery **at all**, which was an exact
  summary of `FR-PROJ-025` until the thirty-first edition amended that
  requirement to admit the upward look `FR-PROJ-016` obliges: the register
  decayed from **this** side, and the fifth validation rule is extended below to
  say so. `DIV-054` is two passages saying no command reaches a server, left
  standing by the very commits that made them false. Three rules of that file
  move with the pass: the **Overstatement** kind is widened to both directions,
  a **Discharged** status is allowed to name the commit that made a statement
  **true** rather than one that removed it — which is how `DIV-046` and
  `DIV-051` close, and why both now say that the correction they asked for must
  **not** be made — and the dating convention written over *Says* is extended
  to every clause but **Status**.

**What found them.** Not one reading and not one instrument. Two came from
deriving an implementation from this corpus and finding it silent — the
conditions of a failed child, and what a directory of links contributes. One
came from reading this corpus against itself, where a file's own opening rule
and a requirement in another file could not both be honoured. And one is the
fifth validation rule doing exactly what it was written for, a sprint late: a
register of corrections owed to files this corpus does not own, read against
those files after somebody else edited them. That last one also found the rule
short by a trigger, which is the second time the fifth rule has been extended
by a pass it authorised — the sixteenth edition made the first extension for
the same reason, that a rule satisfied in full was still not enough.

**One observation is named and not acted on.** The `74` cell of `FR-ERR-001`
characterises its class as I/O on "`.tpl` and what it holds, a `--context`
document, or stdout", and calls those "the three kinds of file and stream this
system touches". Trust material read from a `ca_file` or a `ca_path` outside
`.tpl` — `/etc/ssl/certs/db-ca.pem` is the example in the root `README.md` — is
a fourth, and `FR-CONF-014` now names that code for an entry it cannot read.
Nothing is ungoverned by it, because the thirty-first edition's amendment to
that table says a condition is neither absent nor misfiled because no cell
names it, and no requirement rests on the count of three. It is named here so
that the next reader of that cell does not have to find it again.

*Acted on in the thirty-third edition, and the observation is discharged.* The
cell names four kinds and trust material is among them.

### Thirty-third edition — a file that arrived, a class counted short, and the sweep the rule was worth

Sprint 17 was opened to clear a backlog thirty-one tasks deep. The thirty-first
edition answered fourteen of it in one pass and the thirty-second the four that
remained; three more were raised by the work that followed them, and they are
answered here in one pass, which is this sprint's last reading of this corpus.
Two of the three were recorded by earlier editions of this sprint and
deliberately not acted on — one because the edition that found it had no
reading to choose between, one because it needed a sweep of every file rather
than a judgement on the two instances in front of it.

**No requirement identifier is assigned, none is retired and none is
renumbered.** No entry is raised in
[upstream-divergences.md](upstream-divergences.md) and no entry of it changes
status. The index of [open-questions.md](open-questions.md) stays empty. **No
figure is ratified and no baseline is set.**

**Nothing this edition changes changes what `tpl` does.** One requirement of
form is recorded as measured, two counts are told how they are counted, one
cell of a table names a fourth member of the class it characterises, and five
definitions move to the file this corpus says definitions live in. The one
thing owed outside this corpus is one sentence of the root `CLAUDE.md`, which
was owed before and is owed more narrowly now.

- **A rule that had been waiting on a file the tree now holds** —
  [performance-requirements.md](performance-requirements.md) and
  [upstream-divergences.md](upstream-divergences.md). `BR-PERF-007` said that
  `scripts/mariadb/seed-bench.sql` *is still absent*, and it is not:
  the file loads on each of the four series of `FR-SRV-015` and realises both
  `WL-001` and `WL-003` at every count this corpus states for them. The rule is
  **restated over the condition that now holds** rather than withdrawn, and the
  two are not the same choice: five budgets needed the fixture and four did
  not, which is a permanent property of the budget set and what its live
  citations read the rule for, so withdrawing it would retire an identifier and
  leave every one of them resolving to a note. The same fact reaches the register, and
  it reaches it from the side the thirty-second edition widened the
  **Overstatement** kind to cover — the repository catching up with a document,
  rather than a document being edited. `DIV-036` was raised over that file and
  now owes one sentence instead of a sentence and a file; `DIV-052` waited on a
  benchmark **and** on that fixture, and now waits on the benchmark alone.
  Neither changes status, so no count in that register moves. Two further
  corrections travel with it: `NFR-PERF-014` rejected a tenth budget on the
  ground that it would be *the sixth budget needing a server*, where three rows
  carry `Server: yes` and five is `BR-PERF-007`'s count of the budgets needing
  the **fixture**; and `DIV-052` called itself one of *seven* corrections where
  that register's own Overview counted nine entries owing. Both are corrected
  against the figures they meant to cite. *The verb is put in the past by the
  thirty-sixth edition, which raised `DIV-055` and took that Overview to ten —
  and found the thirty-third's correction had been made in one entry of three,
  which it corrected in the other two.* **`NFR-PERF-001` is recorded as
  measured**, dated, with the condition it was read under: eleven catalogue
  statements against eleven, on all four series, from a fresh project with an
  empty cache. That record ratifies nothing, and says so — ratification was
  `NFR-PERF-009` through `NFR-PERF-012` under the gate of `NFR-PERF-020`, and a
  statement count is a requirement of form. *The verb is put in the past by the
  thirty-sixth edition, which took the gate out of that requirement; the record
  itself is unchanged and denies the same thing it denied.*
- **Two counts that did not say how they were counted** —
  [performance-requirements.md](performance-requirements.md). `WL-003` named
  three indexes and `WL-001` six hundred, and an index count admits two
  readings that differ by a third of it. The fixture had to choose, chose
  **`PRIMARY` counts**, and wrote that rule into a script — an implementation
  filling a gap this corpus left, which is the shape of defect the
  twenty-seventh edition named when the finished first arm had to choose four
  answers for itself. Both workloads now state the rule, so that they are read
  alike: what makes `NFR-PERF-001`'s comparison a comparison of two databases
  rather than of two conventions is that one convention governs both. The
  reading is not a preference. `FR-CAT-043` states that the primary key **is
  also an index** and puts the index named `PRIMARY` in the table's index
  collection, so a count that excluded it would count something the model does
  not present; `FR-CAT-042` supplies the second clause, that a composite index
  is one index and not one per column. **`PRIMARY` not counted** is rejected in
  the requirement's own text, on the ground that it moves both numbers rather
  than settling one — 600 would describe a database of 800 — and buys nothing,
  since the quantity `NFR-PERF-001` compares is a count of statements that no
  index convention moves.
- **A class characterised by three where it reaches four** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). The `74` cell of
  `FR-ERR-001` called `.tpl` and what it holds, a `--context` document and
  stdout *the three kinds of file and stream this system touches*. Trust
  material read from a `ca_file` or a `ca_path` is a fourth: nothing in this
  corpus requires either path to lie inside `.tpl`, a conventional `CApath` is
  a system trust store, and `FR-CONF-014` — as the thirty-second edition
  amended it — routes an entry it cannot read to this very code. The cell names
  the fourth. **The characterisation is not turned back into an enumeration**,
  which the thirty-first edition decided deliberately and the nineteenth closed
  the column against: a fourth member is no argument for reopening either, and
  the count is not load-bearing, so a fifth kind moves this cell and nothing
  else. A **code of its own** for trust material is rejected under
  `FR-ERR-002`, which forbids collapsing two conditions only where the caller's
  next step would differ, and it does not — checking permissions on a
  configured path is what this row already prints.
- **The sweep the thirty-second edition's rule was worth, and five definitions
  moved** — [glossary.md](glossary.md),
  [open-questions.md](open-questions.md) and
  [cli-contract.md](cli-contract.md). That edition wrote the rule that every
  defined term lives in the glossary, moved the two instances a question had
  been asked about, and recorded that no sweep for the rest had been made. The
  sweep is made, over all twenty-six files. It first needed a **boundary**, and
  the boundary is now stated in the glossary with the alternative rejected
  beside it, because the rule read at its widest empties four sections to fill
  one. Three shapes are not definitions: a requirement or business rule that
  fixes a thing, which the glossary cites rather than restates; a legend for
  the values of one field of one table, read where the table is; and a note
  shape the writing conventions own. Under it, **five definitions moved and one
  duplicate was removed.** *Closed*, *dissolved* and *closed on a stated limit*
  were defined in [open-questions.md](open-questions.md) and are used away from
  it — *dissolved* in two other files, and *stated limit* in five, three of
  which carry it as the name of the note a requirement writes where a
  guarantee stops. *Calling agent* and *operator* were defined in
  [cli-contract.md](cli-contract.md)'s *Actors* section, which was the one
  Actors section in the corpus that says what its actors **are** rather than
  what they want from the file, and the only definition of either term
  anywhere. Its third line was the duplicate, and the worse half of the finding:
  it defined *project* a second time and **not the same way**, as the `.tpl`
  folder itself where `FR-PROJ-001` and the glossary have a project as the
  directory containing one. The glossary governs, and the sentence that was
  wrong did not survive the move. One citation is repaired with them:
  the glossary's `volatile field` entry restated `BR-CAT-002` without citing
  it, so a reader was never sent to the rule that fixes the term.

**What found them.** Three different things, and none of them a rule of the
list below. The first was a fact about the working tree arriving after the
corpus had been read against it — the same shape as the register's own decay,
a sprint later and from the other side, and the reason this edition re-read two
entries against the tree rather than against either root document. The second
was a script documenting a rule this corpus had left open, which is only
visible to somebody reading the implementation and the requirement together.
The third was an edition doing what it said it would: the thirty-second wrote a
rule and recorded that it was worth exactly the sweep behind it, and the sweep
found five more terms, in two passages the two known instances did not predict
— and one of the five contradicted the glossary rather than merely repeating
it.

**Two observations are named and not acted on.** The first is that the rule the
sweep applied is a rule about **terms**, and the corpus has an editorial
vocabulary the boundary deliberately leaves in place: ten clause and status
names in [upstream-divergences.md](upstream-divergences.md), three frontmatter
values in the status legend below, and the note shapes of the writing
conventions. Each is read where it is defined and none is used elsewhere, so
none is a defect today; what would make one a defect is its being used away
from the passage that defines it, which is the test the glossary now states.
The second is that nineteen module *Actors* sections state a stake and one stated
a definition, and only the one that stated a definition was touched — the
nineteen are the convention and are left alone.

### Thirty-fourth edition — a folder the corpus had never described, and a rationale that described the code

Sprint 18 delivers `examples/`, which both root documents have named since
before there was a repository to hold it and which has never existed. This is
that sprint's first reading of this corpus, and it is first because nothing
else in the sprint can be written until the corpus says what an example is.
The second half of the edition is unrelated to the first and travels with it
because both are edits to this corpus and neither is large enough to be worth
a reading of its own.

**One file is added and one prefix with it.** No requirement identifier is
retired and none is renumbered. No entry is raised in
[upstream-divergences.md](upstream-divergences.md) and no entry of it changes
status. The index of [open-questions.md](open-questions.md) stays empty. **No
figure is ratified and no baseline is set, and nothing this edition changes
changes what `tpl` does.**

- **A deliverable with no requirement behind it** —
  [examples.md](examples.md), [use-cases.md](use-cases.md) and
  [glossary.md](glossary.md). `CLAUDE.md`'s project tree calls `examples/`
  *complete pipelines: schema to template to output* and the root `README.md`
  names it too; `DIV-050` records the overstatement, because the folder is not
  there. What no document recorded at all is what would have to be in it. The
  new file answers that in nine requirements and two business rules:
  `FR-EX-001` fixes four worked examples, one for each of Go, Rust, Python and
  Node.js; `FR-EX-002` fixes what one takes in and what it produces;
  `FR-EX-003` fixes the four artefacts each one holds — templates, the
  type-mapping macro, the driver script and the compile gate; `FR-EX-004`
  obliges the whole workflow to be driven **through the command line alone**,
  naming the five commands in order and forbidding any library interface;
  `FR-EX-005` states the consequence that a rendered file is a redirection of
  `tpl render`'s standard output, per `FR-RND-028`; `FR-EX-006` and `FR-EX-007`
  fix the three schemas, the one server, and the conditions all four examples
  share; `FR-EX-008` obliges each example to carry its own type mapping in the
  form `FR-ENV-011` fixes; and `FR-EX-009` makes the compile gate's verdict the
  acceptance signal. `UC-013` is the flow end to end. **The server is named by
  criterion and never by number**, per `BR-SRV-005` — the most recent series of
  `FR-SRV-015`.

  **Why `freight` is the third schema, and it is the rejected option that says
  so.** Covering every native type from `sakila` and `world` alone needs no
  schema of the project's own and fails on the facts: neither published dataset
  carries a `JSON`, `UUID`, `INET6` or `BIT` column, a generated column, a
  system-versioned table or a sequence. A mapping written against those two
  would satisfy `FR-EX-008` while leaving untested exactly the types a mapping
  gets wrong, which `FR-CAT-051` and `FR-CTX-038` are the corpus's own evidence
  for. `freight` carries the complement and nothing else is its purpose.

  Three terms move into [glossary.md](glossary.md) with the file, under the
  rule the thirty-second edition wrote and the thirty-third swept for: *worked
  example*, *data layer* and *compile gate*, each citing the requirement that
  fixes it rather than restating it. The adjective in *worked example* is
  load-bearing, and the entry says so: the `EXAMPLES` section of a help text
  and the `example.jinja` that `tpl init` writes are neither of them one.

- **A rejected option named by the code, which had already misled a reader** —
  [configuration-model.md](configuration-model.md). `FR-CONF-014`'s amendment
  note rejected *skipping a symbolic link, **which is the behaviour built***,
  and a second clause of the same note said a dangling link *is skipped in
  silence today*. Both asserted, in the present tense and without a date, what
  `src/mariadb/connect.rs` did; both were false in the commit that wrote them,
  because `455e48d` carries the amendment and the correction of that file
  together. On 2026-09-22 a reading made for the project's architecture
  decision records took the first as a standing claim and concluded that the
  code still skipped links, an error caught only by reading the file. Each
  clause now names the option by what it does and grounds it in the outcome it
  produces, and the reading of the implementation is kept beside them as an
  `*Observed, 2026-09-21.*` note — the shape
  [project-and-discovery.md](project-and-discovery.md) already uses for its two
  readings of `src/project/init.rs`. A rationale that describes an
  implementation stops being true the moment the implementation is corrected,
  and it stops without any signal.

  **The same note held one fact twice, and the copy goes.** It stated that
  entries are sorted into ascending path order, over the names the directory
  holds, before any is read — clause for clause the decision the architecture
  decision record for the TLS mode mapping states, and that record carries with
  it the alternative it rejected and the published documentation grounding the
  rejection, neither of which the copy carried. The requirement now keeps the
  **outcome** — one configuration produces one bundle on every run and on every
  host — and **cites** the record for the order, by role and not by number, in
  the form `FR-ENV-003` already uses for the template engine's pin. *Rejected:
  keeping the sentence here.* It costs a reader no hop and it was rejected
  because the order is a mechanism where a requirement states the outcome a
  caller observes, which is the ninth edition's ground, and because the two
  copies had already begun to differ in content rather than merely repeat each
  other. The duplicate-contribution clause **stays**, because that record cites
  this requirement for it: identifiers flow up and roles flow down, and the
  note is now written in both directions.

**Two editorial corrections to the edition above.** The thirty-second
edition's own record of `FR-CONF-014` carried the same defect as the
requirement it recorded — it said `trust` at `src/mariadb/connect.rs`
*selects* entries on `DirEntry::file_type` and that the arrangement *yields* an
empty bundle, in the present tense — and its closing sentence restated the
order this edition has stopped restating. The verbs are put in the past, which
is the treatment the thirteenth and fourteenth editions gave a closed edition's
present-tense claim, and the closing sentence now points at where the order
lives. A closed edition stays frozen as a narrative; only a verb that has
stopped being true is touched.

**What found them.** Two things, and neither a rule of the list below. The
first was a sprint reaching a deliverable the corpus had never described —
`DIV-050` had recorded for two editions that the folder was missing and no
reading had asked what belonged in it, because an entry never reaches past the
reading that raised it. The second was a reader outside this corpus acting on a
rationale and getting it wrong, which is the only instrument that finds a note
whose every internal check passes.

**Three things are named and not acted on.**

- **The survey the second half was put to found one more of the same shape, and
  it was not corrected here.** `FR-CLI-025` rejected *`64`, which is what the
  parser produces today* — an option named by the behaviour built, in the
  present tense and undated, in a requirement whose whole effect is to change
  that behaviour, so the clause became false at the moment the requirement was
  satisfied. The thirty-fifth edition corrected it. Three near instances were
  read and judged not of the class, each because an edition attribution dates
  the reading: two notes of
  [configuration-model.md](configuration-model.md) written *in the thirty-second
  edition* over what `src/diagnostics/cause.rs` already produced, and
  `FR-ERR-025`'s note written *in the thirty-first* over what
  `src/output/writer.rs` already does. `FR-CFG-041`'s rejected clause is the
  clean case and needs nothing: it is written in the past tense throughout.
- **`FR-CONF-014`'s provenance note names `ADR-002` by number**, where the
  citation this edition adds names the record by role. The two forms now sit in
  one requirement. Which form this corpus uses is a question for a reading that
  covers it, not a by-product of a scoped task, and nothing is ambiguous while
  they stand.
- **The corpus holds twenty-seven files and two passages count twenty-six.**
  Both are accounts of what the thirty-third edition's sweep covered, in the
  past tense, and both stay true of that edition; neither is amended, on the
  ground the note at the foot of this file states for the count of validation
  rules.

**`DIV-050` was checked and unchanged.** It records that `CLAUDE.md`'s tree
names `examples/` as though the directory held something, and when this edition
ran the directory held nothing: a requirement obliging the folder to exist is
not the folder existing. The entry was due exactly as it was, and the task that
created the folder is what discharged its `examples/` limb — `0110f8c`, in the
thirty-fifth edition.

### Thirty-fifth edition — a folder that arrived, a clause built to age, and two requirements that could not both hold

Sprint 18 delivered `examples/` and the four worked examples the thirty-fourth
edition described. Three things in this corpus fell due with it and are
answered here in one pass. The register holds an entry raised over the folder's
absence. One requirement carries the defect that edition found in another,
named, and deliberately did not act on. And the two requirements it wrote about
an example's templates and an example's workflow could not both be satisfied
literally — which the work that built the four examples found, and resolved the
same way in all four before this corpus said anything about it.

**One identifier is assigned — `FR-EX-010` — none is retired and none is
renumbered.** One term enters [glossary.md](glossary.md): *workspace*. No entry
is raised in [upstream-divergences.md](upstream-divergences.md) and none
changes status, though one is discharged in one more of its limbs. The index of
[open-questions.md](open-questions.md) stays empty. **No figure is ratified and
no baseline is set, and nothing this edition changes changes what `tpl` does.**

- **A limb discharged by the repository catching up with the document** —
  [upstream-divergences.md](upstream-divergences.md). `DIV-050` records that
  `CLAUDE.md`'s project tree names six paths in the present indicative, and it
  has been **partly discharged** since `d8e7e8a` and `4014dc4` made three of
  them true. `0110f8c` created `examples/` on 2026-09-22, with the shared
  driver, the four worked examples of `FR-EX-001` and a `README.md` of its own,
  so the fourth line is now true of what is there rather than of a path that
  merely exists. The entry stays **partly discharged** and owes two lines where
  it owed three; `templates/` and `benches/` are untouched, and so is
  `DIV-052`, which reads `benches/` from the other end. No count in that
  register moves, because a status value is what it counts. The trigger was an
  event in the repository and not an edit to either root document, which is the
  direction the thirty-second edition widened the **Overstatement** kind to
  cover.
- **A rejected option that named the behaviour the requirement exists to
  change** — [cli-contract.md](cli-contract.md). `FR-CLI-025` rejected *`64`,
  which is what the parser produces today* and closed on *the parser that
  refuses it*: the present tense, undated, over an implementation. It is the
  instance the thirty-fourth edition's survey found beside `FR-CONF-014`'s two
  clauses and named without correcting, and it is the worse of them — **this
  requirement's whole effect is to change the behaviour the clause named**, so
  the clause was false in the commit that wrote it and stayed false from the
  moment the requirement was satisfied, telling every later reader that `tpl`
  refuses what it accepts. The option is kept, is now named by what it does —
  refusing the invocation with `64` — and is grounded in the outcome it
  produces: no requirement of this corpus ever chose the refusal, and a caller
  pays a whole invocation to learn something that changes nothing about what
  they asked for. The reading of the implementation is kept beside it as an
  `*Observed, 2026-09-22.*` note, in the form
  [project-and-discovery.md](project-and-discovery.md) uses for its two
  readings of `src/project/init.rs`: the six flags are declared as overriding
  themselves, `tpl -q -q version` and `tpl -h -h` both exit `0`, and `455e48d`
  carries the requirement and that declaration in one commit. The three near
  instances that survey judged clean are re-read and unchanged, each dated by
  the edition it attributes its reading to.
- **Two requirements that could not both be satisfied literally** —
  [examples.md](examples.md), [use-cases.md](use-cases.md) and
  [glossary.md](glossary.md). `FR-EX-003` put a worked example's templates in
  *the example's own `.tpl/templates/`* and `FR-EX-004` began that example's
  workflow with `tpl init`, which `FR-PROJ-014` refuses with `73` against a
  destination that already has a `.tpl`, changing nothing. An example cannot
  carry a project and also create one. Both were written in the thirty-fourth
  edition and neither was read against the other; the contradiction was found
  by the work that built the driver, and all four examples resolved it
  identically. `FR-EX-010` writes that resolution down: the workflow runs in a
  **workspace** that holds no `.tpl` when it starts, the project `tpl init`
  creates there is a product of the run and not an artefact of the example, and
  the example's templates are held outside it and placed under
  `.tpl/templates/` after `tpl init` and before the first `tpl render`, which
  is where `FR-TMPL-004` obliges `tpl render` to read them from. Placing them
  after `init` is also what makes `FR-EX-008`'s rationale hold, since an
  example that may **replace** the macro `FR-PROJ-017` writes cannot have had
  its own files there first. **Rejected: an example that carries a committed
  `.tpl/templates/`** — the reading `FR-EX-003` invited — because it fails on
  `FR-PROJ-014`, and its mirror, dropping `tpl init` from the workflow, fails
  on `FR-EX-004`'s own rationale and would commit the entries of a run against
  a real server. `UC-013` states the placement in its first step, names the
  `73` as an alternate flow, and carries the two identifiers with it.

**Four sentences of two closed editions are corrected, and nothing else in
either is touched.** The thirty-fourth edition closed by naming three things it
did not act on, and two of its sentences about them were in the present tense
and have stopped being true: `DIV-050` *is checked and unchanged* and the
directory *still holds nothing*, and `FR-CLI-025` *rejects* a clause it no
longer carries. The thirty-first edition's record of `FR-CLI-025` carried the
same defect as the requirement it recorded, twice — *where the parser refuses
it today* and *which is what the parser produces today*. Every one of the four
verbs is put in the past, and the two that name a defect this edition repaired
say so. This is the treatment the thirty-third and thirty-fourth editions gave
a closed edition's present-tense claim: a closed edition stays frozen as a
narrative, and only a verb that has stopped being true is touched.

**One thing is named and not acted on.** The commits table of
[upstream-divergences.md](upstream-divergences.md) prefaces its list by saying
that a number of the commits it names are *not edits to a root document at
all*, and two of the commits it then names did edit `README.md` — `db7337d` and
`db80114`, which it names for what they made true rather than for what they
edited. This edition adds `0110f8c` to that table and moves the count from five
of thirteen to six of fourteen, which is what the addition obliges; the
looseness in the phrase predates the addition and belongs to a reading of that
sentence, and nothing turns on it, because each commit is named individually
with what it did.

### Thirty-sixth edition — the figures stop refusing, and three readings nobody had chosen

The product owner decided that the benchmarks exist as **informative
instruments**, that no gate is built on one, and that no test runs constantly
for that purpose — and, asked what becomes of the requirements this file states
as forms rather than as figures, decided that those **survive as correctness
invariants**. The obligation toward speed and toward sparing use of
the machine stays, as **design and architecture**: it shapes how `tpl` is built
and decides nothing about whether a change is accepted.
[performance-requirements.md](performance-requirements.md) is rewritten to that
decision, and three holes the first measuring harness fell into are settled with
it.

**One identifier is assigned — `BR-PERF-008` — and four are retired:
`NFR-PERF-013`, `NFR-PERF-015`, `NFR-PERF-016` and `NFR-PERF-017`.** Four more
are amended — `NFR-PERF-011`, `NFR-PERF-014`, `NFR-PERF-019` and
`NFR-PERF-020` — and none of the four touches how a number is taken, which
`NFR-PERF-009`, `NFR-PERF-010` and `NFR-PERF-012` fix and this edition leaves
alone. Two terms are renamed in
[glossary.md](glossary.md) and one is retired there. One entry is raised in
[upstream-divergences.md](upstream-divergences.md), `DIV-055`, and one is
amended, `DIV-052`. The index of [open-questions.md](open-questions.md) stays
empty. **No figure is measured and none is recorded by this edition, and
nothing it changes changes what `tpl` does.**

- **No figure fails a change** —
  [performance-requirements.md](performance-requirements.md). `BR-PERF-008` is
  the rule the rest of the edition follows from: no figure named in this corpus,
  and no figure recorded against it in `BENCHMARKS.md`, fails, blocks, rejects or
  gates a change. Three requirements are withdrawn to make it true, and a fourth
  follows them.
  `NFR-PERF-015` named the **one normative budget**, whose whole content was the
  power to refuse a change over a number; `NFR-PERF-016` existed only to say
  that the other eight carried the no-regression rule instead; and
  `NFR-PERF-017` was that rule — a measurement worse than the recorded baseline
  failed the change that produced it. `NFR-PERF-013` goes with them, not because
  it gated anything but because its subject was the normative budget and its
  rationale argued from continuous integration: the class it governed is now
  empty, and the fact it was read for — which points need a server — is in the
  `Server` column of `NFR-PERF-014`, where it always was. **`BENCHMARKS.md` is
  restated as what it now is**: a register of observations, informative,
  consulted on demand, never a gate.
- **The requirements of form survive, and the section says what they are** —
  [performance-requirements.md](performance-requirements.md). `NFR-PERF-001`
  through `NFR-PERF-008` are untouched and are now introduced as **correctness
  invariants**: deterministic counts and deterministic absences, independent of
  how fast the host is — statements over `WL-001` equal statements over
  `WL-003`, a cache hit opens no connection, one invocation opens at most one
  connection, the commands of `FR-PROJ-025` read no configuration. A cache hit
  that opened a connection is a **functional defect**, not a slow run. The
  integration suite asserts them and already did, which the section records as
  an *Observed* note naming the two files that carry them. *The clause read
  `on every run of it` until the thirty-seventh edition narrowed it to what a
  run actually asserts; the reason is under that edition.*
  Saying this is the point: without it, a reader meeting eight `NFR-PERF`
  identifiers directly above a withdrawn gate takes them for the same thing.
  `WL-002` is stated as being of the same kind for the same reason — a byte
  count is deterministic, and a departure from it is a finding about the fixture
  or the document shape and never a performance result.
- **A dispersion rule that did not say what it was about** —
  [performance-requirements.md](performance-requirements.md). `NFR-PERF-011`
  barred a run whose relative standard deviation exceeds five per cent from
  being recorded as a baseline, and called it *invalid*, which left two readings
  on top of each other: that the instrument was noisy, and that the application
  degraded. **The corpus takes the first**, and now says so — dispersion across
  repeated runs of one unchanged binary measures the machine underneath it and
  measures nothing about the program. The reading is no longer discarded but
  recorded with its dispersion, and it is never a finding about `tpl`. The five
  per cent line is kept and the requirement now carries the evidence for
  stating it as a caution: at full protocol on the measurement host,
  `tpl --version` produced 4.63% and `tpl help --format json` 3.33%, so the two
  cheapest invocations in the set sit close enough to the line that a loaded
  machine crosses it. *Rejected: moving the line to ten per cent*, which would
  have said nothing about what the number means, and *rejected: withdrawing the
  rule with the gates*, since dispersion is the one field that says how much to
  trust a figure.
- **Three readings a shell script had chosen and this corpus had not** —
  [performance-requirements.md](performance-requirements.md). Each appeared the
  moment something tried to measure against `NFR-PERF-014`, and each is settled
  as an informative reference figure, which naming an invocation does not
  undo. **The third point named a quantity and no command**: it is now
  `tpl template list` in a project holding no database entry, the cheapest
  invocation that is useful work rather than static text, where `NFR-PERF-005`
  excuses every form of `help` and of `version` from the discovery and the
  configuration read this point exists to include. *Rejected: letting any
  command satisfying the description serve*, because two readings from two
  commands are two quantities. **The sixth point is two invocations and said
  nothing about how they combine**: its figure is the **slower half**, recorded
  whole, with the other half beside it, on the ground `BR-PERF-004` gives — the
  `66` computes an edit distance against 200 names and is the half that carries
  the quantity. *Rejected: splitting the row*, which is a change to a set fixed
  at nine, and *rejected: the mean of the two*, which no invocation produces.
  **Three points said `Server: yes` and nothing about the cache**: a
  read-through read would have made the fourth and the ninth **cache figures**
  after their first run, under rows saying a server answered, so those two
  bypass the cache with `--direct --no-cache`, the pure read of `FR-CACHE-016`,
  while the eighth starts each run with the cache empty, because one server read
  followed by 199 cache hits is what a caller's loop over 200 objects does. A
  `Cache` column now carries the answer per row, and the rows are numbered,
  because three of them had to be discussed by position.
- **A word that kept a meaning the corpus had withdrawn** —
  [glossary.md](glossary.md),
  [performance-requirements.md](performance-requirements.md). A **budget** is a
  quantity one may not exceed, and this corpus keeps the word where that is
  still true — the invocation timeout of `FR-GLOB-011` and the shared phase
  budget of `FR-CONF-005`. Leaving it on nine figures that limit nothing would
  have left one word carrying two opposite forces, in one corpus, four sections
  apart. The nine are **renamed** and not redefined: they are **measurement
  points**, collectively the measurement set, and what they carry is a
  **reference figure**. The two marks are renamed with the requirements that own
  them — a *provisional* figure is an **adopted** one, per `NFR-PERF-019`, and a
  *ratified* figure is a **recorded** one, per `NFR-PERF-020`, since *ratified*
  meant binding and nothing binds. The term *normative budget* is retired in the
  glossary and its entry kept, because closed editions and the register read
  it.
  `NFR-PERF-019` keeps the job the rename leaves it — separating a figure
  somebody measured from a figure somebody wrote down — and `NFR-PERF-020`
  keeps the recording and loses the gate, stating what a record must carry so
  that a reading can be read years later by somebody who was not there.
  *Rejected: withdrawing `NFR-PERF-019` and `NFR-PERF-020` with the other
  three*, which would leave nine points, five figures nobody measured, and no
  rule about what happens when somebody measures one.
- **The register, read against what this edition amends** —
  [upstream-divergences.md](upstream-divergences.md). The fifth validation rule,
  as the thirty-second edition extended it, says an entry decays when this
  corpus amends a requirement a root document paraphrases. It does here.
  `CLAUDE.md` states, in its mandatory validation pipeline, that a change to a
  hot path adds the benchmarks and a comparison against the baseline, and its
  *Disciplina de medição* closes by stating that a regression against the
  baseline **reproves the change**. Both were exact summaries of `NFR-PERF-017`
  and are contradicted by `BR-PERF-008`, so `DIV-055` is raised for them, due in
  full. `DIV-052` is raised against the second of those two passages already,
  for a different defect and with a different correction, and it is amended
  here only where this edition made it false: its *Specification* paragraph said
  that no requirement of this corpus is contradicted, and its *Correction* that
  a regression failing the change is an obligation in force today. Neither is
  true now. The register's counts move with the new entry — ten entries owing,
  seven of them to `CLAUDE.md` — and moving them found the same figure restated
  in three entries and corrected in one: the thirty-third edition corrected
  `DIV-052` and swept no further, so `DIV-032` and `DIV-041` had kept a number
  the Overview left behind, and this edition's move made the half of theirs
  that was still right wrong as well. Both are corrected, which is the third
  validation rule applied to a count: correcting one restatement of a figure
  establishes nothing about the others. **One thing is named and not acted
  on**: `DIV-052`'s **Kind** is still *overstatement*, and whether a passage
  carrying one defect of each kind should hold one entry or two is a question
  for the next reading of that register, not for the edition that made the
  second defect appear.

### Thirty-seventh edition — the figures are measured, and a claim narrower than the suite

Sprint 15 built a measurement harness, measured the nine points of
`NFR-PERF-014` and the `WL-002` scalar for the first time, and recorded the
readings in `BENCHMARKS.md`. Three things in this corpus fell due with that and
are answered here in one pass. `NFR-PERF-020` obliges an adopted figure to be
removed by the step that records the measurement replacing it, and five were
still standing. The register holds two entries the sprint's other commits
discharged, and its counts are restated for them. And the technical
specification reported one divergence it correctly declined to settle by
rewriting this corpus.

**No identifier is assigned, none is retired and none is renumbered.** No term
enters or leaves [glossary.md](glossary.md). Two entries of
[upstream-divergences.md](upstream-divergences.md) are discharged, `DIV-052` and
`DIV-055`, one more limb of `DIV-050` is discharged, and one question that
register left named is settled. The index of
[open-questions.md](open-questions.md) stays empty. **This edition records
figures and reinstates no gate: `BR-PERF-008` is untouched, and nothing it
changes changes what `tpl` does.**

- **Five adopted figures removed, per point and per target** —
  [performance-requirements.md](performance-requirements.md),
  [context-document.md](context-document.md),
  [open-questions.md](open-questions.md). `fe428da` measured all nine points and
  the `WL-002` scalar at the full protocol of `NFR-PERF-009` — the median of 200
  runs after 20 warmups — on `aarch64-apple-darwin`, against series `12.3` for
  the three points that reach a server, and recorded them under `NFR-PERF-020`.
  Points 1, 2, 3, 4 and 9 carried an adopted figure and carry none now: an
  adopted figure that stayed beside the measurement replacing it is the second
  source `BR-PERF-006` exists to prevent. **The removal is per point and per
  target, and one target of four was measured**, so the table says so in its own
  words: the other three carry no figure for any point, not the adopted one and
  not the measured one, and `NFR-PERF-012` forbids carrying a figure across
  targets. The last column no longer holds values at all — it says where each
  point's reference figure stands — and the provenance note stops quoting the
  four numbers it was written to explain the origin of, for the reason
  `DIV-035` gave `CLAUDE.md`. Two readings do not stand, per `NFR-PERF-011`:
  point 5 at 8.294% dispersion and the `64` half of point 6 at 7.513%, because
  the host ran on battery and was not idle. Neither point had an adopted figure,
  so the removal reaches neither, and retaking them on a quiet host is rmp
  `#228` and is not settled here.
- **What supersession requires, because the first measurement departed from part
  of it** — [performance-requirements.md](performance-requirements.md).
  `NFR-PERF-019` names `NFR-PERF-009`, `NFR-PERF-010` and `NFR-PERF-012` and
  admits no departure from any, while `NFR-PERF-020` provides in terms for a
  record that states *in what respect it departed*. The campaign met two of
  `NFR-PERF-010`'s four clauses. The two are reconciled in the direction
  `NFR-PERF-020` already fixed: a measurement supersedes when it is taken at the
  protocol of `NFR-PERF-009`, names its target and its series, states every
  departure from `NFR-PERF-010`, and carries a dispersion inside
  `NFR-PERF-011`'s line. *Rejected: reading it as admitting no departure at
  all*, which would have discarded eight readings whose own dispersion says the
  host was quiet enough, and would have treated a fact about a machine as a
  verdict on a reading — which is what the thirty-sixth edition took
  `NFR-PERF-011` out of.
- **Four observations the campaign produced, and where each belongs** —
  [performance-requirements.md](performance-requirements.md),
  [context-document.md](context-document.md). Each is recorded where the corpus
  says something the reading bears on, and nowhere else. `NFR-PERF-019` records
  that every one of the five adopted figures came in below the figure adopted
  for it, the widest by more than an order of magnitude, which is the evidence
  for what its own rationale asserts about a figure somebody wrote down.
  `NFR-PERF-014` corrects the ground it gave for point 3's adopted figure being
  twice points 1 and 2's: measured, point 3 is **0.8%** above point 2, not 100%,
  and the discovery and configuration read that distinguish it cost a small
  fraction of what the ratio anticipated — **the choice of invocation is
  untouched**, because what made `tpl template list` right is what
  `NFR-PERF-005` excuses `help` and `version` from and not what the work costs.
  `BR-PERF-004` records the check it exists to make possible: a wrong invocation
  cost **17.7% more** than `tpl --version`. And `FR-CTX-010`'s accepted cost
  predicted the peak-memory figure would be superseded **upward**; it was not,
  and that bullet is corrected and stops restating the figure. Nothing follows
  from any of the four by rule, per `BR-PERF-008`.
- **A claim about the suite wider than any run** —
  [performance-requirements.md](performance-requirements.md). The *Requirements
  of form* section said the integration suite asserts them **on every run of
  it**, and `docs/spec-technical/verification.md` records the narrower
  arrangement: the assertions needing a server are gated on the fixture
  standing, and the syscall trace of `NFR-PERF-007` runs on neither Darwin
  target. That folder reported the divergence and correctly declined to settle
  it by rewriting this one. **It is settled here by narrowing the wording to
  what is verified**, with a table saying what each assertion needs and what a
  run without it does — skip, with a stated reason, and never pass silently.
  A skipped assertion is not a weakened requirement: every clause in that table
  holds on all four targets and is verified on all four, and the one place the
  evidence is genuinely weaker is the file-open observation, which
  `NFR-PERF-005` already names and bounds. *Rejected: stating the gap and
  keeping the sentence*, which leaves a reader who stops at the bold claim
  carrying away what the qualification withdraws.
- **Two entries the sprint discharged, and the question one of them left named**
  — [upstream-divergences.md](upstream-divergences.md). `DIV-055` is discharged
  by `9562fb2`, which removed both passages imposing a performance gate from
  `CLAUDE.md`; it was raised and paid inside one sprint, and it exercises both
  halves of the fifth validation rule — raised because this corpus amended a
  requirement that file paraphrased, discharged because somebody edited the
  file. `DIV-052` is discharged by `abbfe70`, which created `benches/` with the
  measurement harness in it, so the sentence saying benchmarks live there and
  run against the containers' dataset is true; that is the repository catching
  up with the document, and `fe428da` then took readings from it. The same
  commit discharges one more line of `DIV-050`, which stays **partly
  discharged** and owes `templates/` alone. **The Kind question `DIV-052` left
  named is settled**: the Kind stays *overstatement*, because a Kind classifies
  the defect an entry records and not the passage it points at — and the proof
  is that one sentence carrying two defects discharged on two conditions, by
  events of two different sorts, which one entry could not have described.
  *Rejected: moving the Kind to contradiction*, and *rejected: merging the two
  entries now both are discharged*, which would leave one identifier resolving
  to nothing.
- **The register's counts, restated against its own Index** —
  [upstream-divergences.md](upstream-divergences.md). Eight entries owe
  something where ten did, five to `CLAUDE.md` where seven did, and forty-seven
  are discharged where forty-five were. Every restatement of those counts was
  found before any was changed — the Overview, `DIV-032`, `DIV-041` and
  `DIV-052` — which is what the thirty-sixth edition's own note asks of the
  edition that moves them. Two further claims are narrowed by the same
  arithmetic: that the remaining work is *almost entirely* `CLAUDE.md`'s, which
  at five to three it is not, and the count of how many entries of the third
  kind are discharged, which is restated against the Index rather than carried
  forward and did not match it before. The fifth validation rule is paid for
  three commits that had touched `CLAUDE.md` since the last pass — `3a360d6`,
  `6a66d14` and `9562fb2` — and the first two reach no entry's passage.

**Two sentences of a closed edition are corrected, and nothing else in it is
touched.** The thirty-sixth edition named two statements outside this corpus
that it made false and did not own: the budget tables of
`docs/spec-technical/quality-attributes.md`, corrected by `66e0bbf`, and **the
four architecture decision records that argue from `NFR-PERF-017`**, reconciled
by `2c3d60f` — which changed **eight**. The count was this corpus asserting
something about a folder it does not read, and it is corrected with the
discharge recorded beside it. This is the treatment the thirty-third and
thirty-fourth editions gave a closed edition's present-tense claim.

**One thing is named and not acted on.** `9562fb2` added a sentence to
`CLAUDE.md` stating that the requirements of form are asserted by the test suite
**em cada `cargo test`**, which is the same claim this edition narrowed in
[performance-requirements.md](performance-requirements.md) and is wider than
what a run without the fixture asserts. Whether that is an **overstatement**
owed to that file is a judgement for a reading of the section it sits in, on the
rule the sixteenth and seventeenth editions used for a candidate a sweep could
not judge, and raising an entry is not what this edition was authorised to do.
It is named here so that the next reading of
[upstream-divergences.md](upstream-divergences.md) has it, it blocks nothing,
and no requirement of this corpus waits on it.

### Thirty-eighth edition — a count that did not say what it counted

`FR-CACHE-034` required `tpl cache status` to report, per collection, "the
count of objects held", and left open whether an object file whose content
cannot be read, is not valid UTF-8, or does not decode is held.
`FR-CACHE-033` answers that question for a read and not for a report. The user
decided that it is held.

**No identifier is assigned, none is retired and none is renumbered.** No term
enters or leaves [glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged, and
the index of [open-questions.md](open-questions.md) stays empty.

- **The count is of object files present, not of objects readable** —
  [cache-commands.md](cache-commands.md). `FR-CACHE-034` now defines `count` as
  the number of files in the collection's folder that the arrangement versioned
  by `cache_format` names as objects, whatever their content, and forbids
  opening an object file to count it. A temporary file of a write in flight and
  any other file that is not an object file are not counted. Whether a file can
  serve stays with `FR-CACHE-033`, at read time. *Accepted cost:* a count can
  include a file the next read treats as a miss. `FR-CACHE-025`,
  `FR-CACHE-035`, `FR-CDOC-006` and `FR-CDOC-013` were read against the
  decision and none conflicts with it, so none is amended. Neither root
  document paraphrases the count, so the fifth validation rule owes nothing.

### Thirty-ninth edition — a write that would change nothing

`FR-CACHE-030` required every cached object to be written through a temporary
file renamed over the target, and did not say whether a file already holding
exactly the bytes the write would produce must still be replaced. It need not
be, as decided for rmp `#244`.

**No identifier is assigned, none is retired and none is renumbered.** No term
enters or leaves [glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged, and
the index of [open-questions.md](open-questions.md) stays empty.

- **A byte-identical file may stay in place** —
  [cache-commands.md](cache-commands.md). `FR-CACHE-030` now permits the system
  to leave a target whose content is byte-identical to the write in place, and
  requires the temporary file and the rename for a target that cannot be read
  or differs in any byte. The observable result is the same either way: the
  rename changes only an object file's modification time, which no output
  reports, and `loaded_at` stays in `meta.json`, which the permission does not
  reach. The reading in `BENCHMARKS.md` that raised the question is
  informative, per `BR-PERF-008`. *Accepted cost:* an object file's
  modification time no longer tells when the object was last read.
- **A rule that restated the mechanism** —
  [cache-documents.md](cache-documents.md). `BR-CDOC-004` said each file was
  "renamed into place"; it now cites `FR-CACHE-030` instead, and its argument
  is unchanged.

`FR-CACHE-007`, `FR-CACHE-008`, `FR-CACHE-031`, `FR-CACHE-032`,
`FR-CACHE-033`, `FR-CACHE-034`, `FR-CACHE-035`, `FR-CACHE-036`, `FR-CDOC-013`,
`FR-CDOC-015`, `FR-CFG-041` and `UC-010` were read against the decision and
none conflicts with it, so none is amended. No use case or glossary entry says
every file is rewritten. Neither root document paraphrases `FR-CACHE-030`, so
the fifth validation rule owes nothing.

### Fortieth edition — a render that reads only what its template reaches

A cached render read and decoded every object file of its entry, even when it
was bound to one table. The user chose lazy loading, as decided for rmp `#246`:
the `database` a template sees stays whole, and an object's file is read only
when the template reaches that object. Narrowing the context to the bound
object was rejected.

**Two identifiers are assigned, `FR-CACHE-038` and `FR-CACHE-039`; none is
retired and none is renumbered.** No term enters or leaves
[glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged, and
the index of [open-questions.md](open-questions.md) stays empty.

- **What is read, and when** — [cache-commands.md](cache-commands.md).
  `FR-CACHE-038` reads `database.json`, each collection's listing and the bound
  object's file before the render, and every other object file no earlier than
  the template's first reach of it. Where the path of every object file names
  the object that file holds, the `database` a template sees is the document an
  up-front read of the same files would produce. Reverting to the up-front read
  to keep that equivalence for files `tpl` never writes was rejected.
- **A miss found during the render** — [cache-commands.md](cache-commands.md).
  `FR-CACHE-039` abandons the render, reads the server once as any miss does,
  and renders again from the server's document. No byte of the abandoned render
  reaches stdout. The second render has its own full deadline and keeps the
  first render's `now`.
- **A file no read consults** — [cache-commands.md](cache-commands.md).
  `FR-CACHE-033` now applies to the files a read consults. A damaged file the
  template never reaches is not a miss of that invocation and is left as it is.
  The same clause states, without changing it, what `tpl schema info` already
  did: it opens no object file. A second clause makes two files `tpl` never
  writes a miss for a render under `FR-CACHE-038`: an object file whose content
  names an object other than the one its path names, found when the file is
  read, and a file whose name no object's path could take, found when the listing is
  read. `BR-CACHE-001` was read against both clauses and is not contradicted:
  the check is one the binary makes before serving, and the layout stays
  outside the plumbing contract. *Accepted cost:* a damaged file stays damaged
  until a read consults it.
- **Hit and miss** — `FR-CACHE-006`, the glossary entry *cache hit / cache
  miss* and `NFR-PERF-003` in
  [performance-requirements.md](performance-requirements.md) say that an
  invocation is a hit only if no file it reads is a miss.
- **The render and its order** — `FR-RND-002` and `FR-RND-034` in
  [render-command.md](render-command.md), and `FR-ERR-006` in
  [errors-and-exit-codes.md](errors-and-exit-codes.md). An abandoned render is
  not the invocation's render, leaves nothing on stdout, and returns the
  invocation to the cache-or-connection step.
- **Deadline, `now`, coherence and diagnostics** — `FR-CONF-005` in
  [configuration-model.md](configuration-model.md), `FR-CTX-029` in
  [context-document.md](context-document.md), `BR-CDOC-004` in
  [cache-documents.md](cache-documents.md), and `FR-GLOB-017` in
  [global-flags.md](global-flags.md) each carry a note citing the two new
  requirements.

`FR-CACHE-007`, `FR-CACHE-014`, `FR-CACHE-031`, `FR-CDOC-007`, `FR-CDOC-015`,
`FR-RND-022`, `FR-RND-023`, `FR-RND-032`, `FR-ENV-015`, `FR-ENV-017`,
`NFR-PERF-004`, `NFR-DET-001` and `BR-SCH-004` were read against the decision
and none conflicts with it, so none is amended. For a template and a cache in
which nothing is missing, every output is what it was. Both root documents say
that a cache hit opens no connection, which stays true, so the fifth validation
rule owes nothing.

### Forty-first edition — a lookup that reads only what it returns

Under `FR-CACHE-038` a lookup by name read the file of every object listed
before the one it returned, because the requirement did not say whether a
lookup reaches the objects it passes over. The user decided that it reaches
only the object it returns, as decided for rmp `#248`.

**No identifier is assigned, none is retired and none is renumbered.** No term
enters or leaves [glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged, and
the index of [open-questions.md](open-questions.md) stays empty.

- **A lookup reaches only the object it returns** —
  [cache-commands.md](cache-commands.md). `FR-CACHE-038` now names the lookups
  by name — the tests `primary_key` and `unique` and the functions `table`,
  `view`, `routine` and `column` — resolves each name from the collection's
  listing, and reads only the returned object's file. A lookup that finds
  nothing reads no object file. Its equivalence now covers functions as well as
  filters and tests. *Rejected:* a lookup that reaches every object it passes
  over, which gives the same answer and reads more files.
- **A damaged file a lookup passes over** — [cache-commands.md](cache-commands.md).
  `FR-CACHE-033` says a lookup consults only the object it returns, so a
  damaged file it passes over is not a miss, as the fortieth edition decided
  for a file no read consults.
- **What a lookup depends on** — `FR-ENV-015` and `FR-ENV-020` in
  [template-environment.md](template-environment.md), and `FR-CTX-022` in
  [context-document.md](context-document.md), each carry a note: the answer,
  including absence under `FR-ENV-017` and `FR-ENV-043`, depends on the
  listing's names alone. No lookup offers a nearest-match suggestion, so none
  depends on anything else.

`FR-CACHE-039`, `FR-ENV-016`, `FR-ENV-017`, `FR-ENV-041`, `FR-ENV-043`,
`FR-SEM-017`, `FR-SEM-018` and `BR-CTX-003` were read against the decision and
none conflicts with it, so none is amended. For a template and a cache in which
nothing is missing, every output is what it was. Neither root document
paraphrases a lookup, so the fifth validation rule owes nothing.

### Forty-second edition — the bounds a hostile input found missing

The security audit of sprint 20, recorded in `SECURITY-AUDIT.md` at the
repository root, reported three findings and two hardening observations. The
user selected five fixes, as decided for rmp `#252` through `#256`, and this
edition writes the behaviour each fix needs. A re-verification of the first
of them added a render memory limit to `#255`, and `#258` separates the render
from the catalogue read; both are folded into this edition.

**Eight identifiers are assigned — `FR-RND-036` through `FR-RND-040`,
`FR-CONF-045`, `FR-SEC-025` and `FR-CTX-042`; none is retired and none is
renumbered.** One term enters [glossary.md](glossary.md), *render bound /
render fuel / render output limit / render memory limit*, and the entry *cache hit / cache miss* is amended. One entry of
[upstream-divergences.md](upstream-divergences.md) is raised, `DIV-056`, and
none is discharged. The index of [open-questions.md](open-questions.md) stays
empty.

- **A render is bounded by work, output and memory, not only by time**
  (`#255`) — [render-command.md](render-command.md) and
  [configuration-model.md](configuration-model.md). `FR-RND-036` bounds a
  render by render fuel, counted by the template engine; `FR-RND-037` by the
  render output limit, counted as bytes are produced; and `FR-RND-039` by the render memory limit, the heap the process
  holds as its allocator counts it, observed periodically. Exceeding any of
  the three is `65`, and the `cause` names the bound, its resolved value and
  the key that raises it. `FR-RND-038` composes the three with the deadline
  and gives each render of `FR-CACHE-039` whole bounds; an abandoned render
  keeps all four until it returns, and one it crosses, before or after the
  miss, ends the invocation with `65` and no server read, per `FR-CACHE-039`
  and the note on `FR-ERR-006`. `FR-CONF-045` resolves
  them from three new keys of `FR-CONF-002`: `core.render_fuel` (default
  100 000 000, range 1 to 10^12), `core.render_output_limit`
  (default 64 MiB, below the memory default because the output a render holds
  counts toward it, range 1 byte to 1 TiB) and `core.render_memory_limit` (default 128 MiB, the
  user's decision over the base recorded in `BENCHMARKS.md`, range 8 MiB to
  1 TiB), with no flag and no environment layer, as for the deadline keys.
  *Stated limit,* in `FR-RND-039`: the memory limit can be passed briefly
  between two observations, and a single allocation the operating system
  refuses aborts the process by a signal; the deadline is the backstop.
  `FR-RND-036` first stated that memory was bounded only by the deadline and
  rejected a hard cap; that text is superseded within this edition.
  `FR-ERR-001`'s `65` cell and `FR-ERR-034`'s `65` row name the bounds;
  `FR-SEC-025` records the threat closed, and `FR-SEC-022` carries a note.
- **No connection outlives the catalogue read** (`#258`) —
  [render-command.md](render-command.md). `FR-RND-040` closes the connection
  and shuts down the driver's runtime before any render starts, on every path
  of `tpl render` that reads the server, and names how it is verified: the
  server records the session ended before the first byte of output, and the
  suite asserts no connection or runtime is alive when the template begins.
  `NFR-PERF-004` and `FR-CACHE-039` carry notes.
- **The `password_command` deadline ends the whole process group** (`#252`) —
  `FR-CONF-028` now starts the child in a process group of its own, ends the
  phase only when the child has exited and its output has ended, and at the
  deadline terminates the group and stops waiting on the pipe, with `78`. It
  states where the termination stops: a descendant that leaves the group.
  `FR-CONF-031` terminates the same group at the 4096-byte output cap, still
  with `78`. `FR-SEC-012`, `FR-SEC-024` and `FR-CONF-043` follow both.
- **A `--context` document must not dangle** (`#253`) —
  [context-document.md](context-document.md). `FR-CTX-042` makes every table a
  foreign key names, in either direction, a member of `tables` for a supplied
  document, so a document that breaks it is `65` under `FR-RND-020` and
  `FR-ERR-029` and never `70`. A column's `table_name` stays with
  `FR-SEM-018`, which accepts that case on purpose. `FR-RND-020` carries a
  note.
- **A cached file holding another object is a miss** (`#254`) —
  [cache-commands.md](cache-commands.md). `FR-CACHE-033` extends to every read
  of one named object the rule it gave a render: a file whose object differs
  in kind or in name is a miss that reads the server, per `FR-CACHE-007`. File
  naming is unchanged. `FR-CDOC-008` in
  [cache-documents.md](cache-documents.md) says what "present" means.
- **A symbolic link in the cache is never followed** (`#256`) —
  `FR-CACHE-030` states the write guard, that a link at the target is replaced
  and never left in place, and `FR-CACHE-033` the read guard, that an object
  file that is a link is a miss.

`FR-CONF-005`, `FR-CACHE-007`, `FR-CACHE-038`, `FR-CACHE-039`, `FR-CTX-023`,
`FR-ENV-017`, `FR-SEM-017`, `FR-SEM-018`, `FR-ERR-006`, `FR-ERR-027`,
`FR-ERR-028`, `FR-ERR-029`, `FR-ERR-030`, `FR-CFG-009`, `FR-CFG-010` and
`FR-CFG-037` were read against the decisions and none conflicts with them, so
none is amended. `tpl cfg` validates and lists keys by reference to
`FR-CONF-002` and needs no change. By the fifth validation rule the root
`README.md` owes one correction, `DIV-056`, because it counts the key space;
`CLAUDE.md` paraphrases none of the requirements amended.

### Forty-third edition — help and errors a caller can act on alone

The audit of sprint 21, recorded in `HELP-AUDIT.md` at the repository root as
rmp `#259`, read every help text and every error message for what a calling
agent learns from help, exit code and stderr alone. Nine of its findings need
a change to this corpus, and this edition writes them for rmp `#260`. The
other findings need none and are not touched here.

**Six identifiers are assigned — `FR-HELP-031`, `FR-HELP-032`,
`FR-HELP-033`, `FR-ENV-047`, `FR-ERR-041` and `BR-ERR-004`; none is retired
and none is renumbered.** No term enters or leaves
[glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **The template surface carries signatures** (H-03) —
  [template-environment.md](template-environment.md). `FR-ENV-005` now holds
  one item object per name, and `FR-ENV-047` fixes the item: `name`,
  `signature`, `operand`, `arguments` with type, requirement and default, and
  a one-sentence `purpose`, for groups 1 and 2. The members of the arrays
  change type from string to object; no release has been made, so
  `schema_version` stays `1`, per `FR-OUT-038`.
- **The context variables are published** (H-02) —
  [help-and-version.md](help-and-version.md). `FR-HELP-017` adds a fifth key
  of `data`, `context_variables`, after the last, and `FR-HELP-032` fixes it:
  the seven top-level variables of `FR-RND-023`, each with its type, the flag
  that binds it where one does, and its meaning. `FR-HELP-022` holds both new
  sets of values in its typed table.
- **The text help of `tpl render` states the same material** (H-02, H-03) —
  `FR-HELP-033` lists the variables, every filter, test and function of groups
  1 and 2 with its signature and purpose, and the statement that group 3 is
  unguaranteed, inside `DESCRIPTION`. `FR-HELP-006` is unchanged.
- **Every leaf says what it touches** (H-10) — `FR-HELP-031` ends every leaf's
  `DESCRIPTION`, and its JSON `description`, with four statements: whether it
  connects to a server, whether it needs a database entry, which files it
  writes, and what it prints.
- **Template names and project paths can be written into a hint** (H-07,
  E-22) — [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-041`
  governs a template name and a filesystem path of the project or of
  `--tpl-dir` by the set `[A-Za-z0-9_./-]`, at most 1024 characters and not
  beginning with `-`. `FR-ERR-022`, `FR-ERR-023` and `FR-SEC-019` in
  [security.md](security.md) cite it. A nested template name can now be
  suggested, and the example of `FR-PROJ-011` in
  [project-and-discovery.md](project-and-discovery.md) carries the absolute
  path.
- **A hint must be able to succeed** — `BR-ERR-004` bars a hint that names a
  command which cannot succeed in the state found, and requires the known
  value instead of a placeholder, subject to the character sets. It sharpens
  `FR-ERR-009`, which asked for a concrete, runnable command and did not say
  it must succeed. The `hint` of `FR-CONF-011` in
  [configuration-model.md](configuration-model.md) is amended to meet it.
- **Two `cause` rows name the instance** (E-03, E-08) — `FR-ERR-034`. The `65`
  row names the key path and the expected type in a `--context` document,
  instead of citing a file of this corpus. The `78` row names the path
  `--tpl-dir` named and that no upward search was made, and `FR-PROJ-008`
  states the condition, the message and the hint.
- **Two `cfg` invocations gain a stated outcome** (E-14, E-25) —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-007` makes a block key given to
  `tpl cfg get` a `64` whose hint is `tpl cfg database show` or
  `tpl cfg list`, and `FR-ERR-035` gains the routing row. `FR-CFG-020` makes
  `tpl cfg database update` with no field flag a `64` whose hint lists the
  flags of `FR-CFG-027`.

`FR-HELP-006`, `FR-HELP-007`, `FR-HELP-014`, `FR-HELP-019`, `FR-HELP-029`,
`FR-ENV-004`, `FR-ENV-016`, `FR-ENV-020`, `FR-RND-020`, `FR-RND-023`,
`FR-CTX-026` through `FR-CTX-028`, `FR-GLOB-009`, `FR-GLOB-025`,
`FR-PROJ-006`, `FR-CFG-016`, `FR-CONF-040`, `FR-CONF-041`, and `FR-ERR-040`
were read against the changes and none conflicts with them, so none is
amended. Neither root document paraphrases a requirement this edition
amends, so the fifth validation rule owes nothing.

### Forty-fourth edition — a default with two readings, and a cache written by two rules

The implementation of the forty-third edition, for rmp `#261`, found one
requirement of that edition open to two readings, silent on arguments passed
only by name, and two requirements of the cache that could not both hold. The
first and the last are settled by the reading the implementation took; the
second by a signature a template can write, which the implementation must
follow.

**No identifier is assigned, none is retired and none is renumbered.** No term
enters or leaves [glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged, and
the index of [open-questions.md](open-questions.md) stays empty.

- **A default is carried as source text** —
  [template-environment.md](template-environment.md). `FR-ENV-047` now makes
  `default` a JSON string holding the default as a template's source writes it
  — `"2"`, `"\"\""`, `"false"` — or `null` where there is none, and never the
  JSON value that source denotes.
- **An argument passed only by name is written by name** —
  [template-environment.md](template-environment.md). `FR-ENV-047` now writes
  such an argument `name=…` in `signature`, orders `arguments` as the
  signature writes them, and lets an item describe one form of call. No field
  is added to the argument object.
- **A read command writes the cache** — [cache-commands.md](cache-commands.md).
  `FR-CACHE-028` named `tpl cache load` and `tpl cache clean` as the only
  commands that change what is stored, against the table of `FR-CACHE-015`. It
  now names a read command of `FR-CACHE-009` writing under `FR-CACHE-015` as the
  third, and keeps its point: nothing invalidates the cache automatically.

`FR-ENV-005`, `FR-ENV-018`, `FR-HELP-022`, `FR-HELP-033`, `FR-CACHE-008`,
`FR-CACHE-009`, `FR-CACHE-014`, `FR-CACHE-016`, `FR-CACHE-029`,
`FR-CACHE-033`, `BR-CACHE-002`, `BR-CACHE-004`, `BR-CDOC-004` and `UC-011` were read against
the changes and none conflicts with them, so none is amended. Neither root
document paraphrases either requirement, so the fifth validation rule owes
nothing.

### Forty-fifth edition — a global flag that does nothing says so

The re-audit of sprint 21, closed for rmp `#269`, found that `tpl init` given
`--tpl-dir` acts on its own destination and says nothing about the flag, so a
`73` names a `.tpl` the caller did not point at (finding R-11). No requirement
fixed the outcome. This edition fixes it. The flag is accepted with no effect,
as `FR-GLOB-007` already does for `-d`, and the invocation says so on stderr
and in its help.

**Two identifiers are assigned — `FR-PROJ-026` and `FR-HELP-034`; none is
retired and none is renumbered.** No term enters or leaves
[glossary.md](glossary.md), no entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **`tpl init` warns about `--tpl-dir`** —
  [project-and-discovery.md](project-and-discovery.md). `FR-PROJ-026` accepts
  the flag with no effect: the path it names is not resolved, examined or
  checked. The requirement writes one fixed warning line to stderr, naming the
  flag and the form `tpl init <path>` and never reproducing the value. The line
  comes first, before any error of `FR-PROJ-014` or `FR-PROJ-015` and before the
  warning of `FR-PROJ-016`. The exit code is the one the invocation has without
  the flag, and `-q` suppresses the line. `UC-001` in
  [use-cases.md](use-cases.md) gains the alternate flow and cites it.
- **The help of `tpl init` states it** —
  [help-and-version.md](help-and-version.md). `FR-HELP-034` puts one sentence
  before the four statements of `FR-HELP-031`, in the text `DESCRIPTION` and in
  the JSON `description`: `--tpl-dir` has no effect, and the destination is
  `PATH` or the current directory.
- **`BR-GLOB-001` states what the corpus does** —
  [global-flags.md](global-flags.md). The rule read that a flag one node would
  have to ignore or reject is local, which made `-d/--database` and `--tpl-dir`
  local under `FR-GLOB-007` and `FR-PROJ-025`. It now reads as follows. A node
  may give a global flag no effect, and a global flag is never refused on its
  own. A refusal is admitted only for a combination, as in `FR-RND-018`. A flag
  some node would have to refuse on its own is local. A table names the nodes on
  which each of the two flags has no effect.

`FR-GLOB-002`, `FR-GLOB-003`, `FR-GLOB-009`, `FR-GLOB-010`, `FR-GLOB-015`,
`FR-GLOB-021`, `FR-CLI-014`, `FR-CLI-024`, `FR-PROJ-008` through `FR-PROJ-016`,
`FR-PROJ-022`, `FR-PROJ-025`, `FR-SEC-016`, `FR-OUT-020`, `FR-OUT-023`,
`BR-CLI-004`, `FR-ERR-006`, `FR-HELP-019`, `FR-HELP-031`, `NFR-PERF-005` and
`NFR-PERF-007` were read against the changes, and none conflicts with them, so
none is amended. The trust checks of `FR-GLOB-010`, `FR-PROJ-008` and
`FR-SEC-016` govern a folder an invocation uses as its project, and `tpl init`
uses none. The warning is a line on stderr and not an error, so the validation
order of `FR-ERR-006` gains no step. Neither root document paraphrases an
amended requirement: the root `README.md` describes `--tpl-dir` under
`FR-GLOB-009`, which is unchanged. The fifth validation rule therefore owes
nothing.

### Forty-sixth edition — every broken template, a flag that could not act, and a shortened command

The second re-audit of sprint 21, closed for rmp `#274`, found three places
where the text an agent reads promised more than the command did. `tpl
template check` stopped at the first broken template while its help says it
checks every one (finding S-05). `tpl render --context <file> --direct` exited
`0` with `--direct` doing nothing (finding S-11). `tpl sch` received no
suggestion, because a prefix is too far from the whole name by edit distance
(finding S-14). No requirement fixed any of the three outcomes. This edition
fixes them.

**Three identifiers are assigned — `FR-TMPL-032`, `FR-RND-041` and
`FR-ERR-042`; none is retired and none is renumbered.** No term enters or
leaves [glossary.md](glossary.md); the entry *nearest match* is amended. No
entry of [upstream-divergences.md](upstream-divergences.md) is raised or
discharged, and the index of [open-questions.md](open-questions.md) stays
empty.

- **`tpl template check` reports every failure** —
  [template-commands.md](template-commands.md). `FR-TMPL-032` checks every
  selected template before it reports, writes one four-line message per
  failing template in the order of checking, and exits `65`. A condition other
  than a syntax error stops the check and is reported alone, with its own
  code. `FR-TMPL-020` and `FR-ERR-006` carry notes: one condition met by
  several objects is reported per object only where a requirement says so.
- **`--direct` with `--context` is refused** —
  [render-command.md](render-command.md). `FR-RND-041` exits `64` at argument
  parsing, as `FR-RND-018` does for `--context` with an explicit `-d`. The
  `cause` names both flags and why they contradict each other. The `hint`
  names the flag to remove for each outcome and does not reproduce the path.
  Where `-d` is also given, `FR-RND-018` is reported. `FR-RND-025` carries a
  note.
- **A shortened command is suggested** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-042` admits,
  for a command token only, every child of the node reached of which the token
  is a proper prefix, at any distance. Such candidates join those of
  `FR-ERR-019` under its order and its cap of three. Nothing is executed, and
  `FR-CLI-004` still makes the invocation `64`. `FR-ERR-019`, `FR-ERR-020` and
  `FR-CLI-004` carry notes, and `UC-012` in [use-cases.md](use-cases.md) gains
  an alternate flow.

`FR-TMPL-013`, `FR-TMPL-017` through `FR-TMPL-019`, `FR-TMPL-026`,
`FR-TMPL-027`, `FR-TMPL-031`, `FR-OUT-037`, `FR-ERR-008`, `FR-ERR-011`,
`FR-ERR-022`, `FR-ERR-024`, `FR-ERR-033`, `FR-ERR-034`, `FR-ERR-037`,
`FR-ERR-038`, `FR-RND-018`, `FR-RND-019`, `FR-RND-022`, `FR-CACHE-013`,
`FR-CACHE-015`, `FR-CACHE-018`, `FR-GLOB-021`, `BR-GLOB-001`, `FR-HELP-013`,
`FR-HELP-027` and `FR-HELP-028` were read against the changes, and none
conflicts with them, so none is amended. `FR-HELP-013` already obliges the
help of `--direct` and `--context` to state the new mutual exclusion. The
finding S-02 was read against `FR-CFG-016`, `FR-CONF-040` and `FR-CONF-041`,
which admit an entry created from one discrete flag and refuse it with `78`
when it is used; nothing is changed for it. Neither root document paraphrases
an amended requirement, so the fifth validation rule owes nothing.

### Forty-seventh edition — a key misspelt, an array on the command line, and a hint that acts elsewhere

The help refinement of sprint 21, for rmp `#276`, found four places where a
requirement in force kept an agent from acting correctly on first reading.
`tpl cfg get core.conect_timeout` received no suggestion, because the
suggestion was drawn over the keys the file sets. `tpl cfg set
database.shop.password_command '["pass","x"]'` stored the one word
`[pass,x]` and exited `0`. A trailing backslash in the same string was dropped
without trace. And a `hint` naming another `tpl` command did not carry the
`--tpl-dir` or `-d` the invocation was given, so the copied command could act
on another project or another entry; a hint naming a `--context` path tested
it against a set no requirement named for it.

**Two identifiers are assigned — `FR-CONF-046` and `FR-ERR-043`; none is
retired and none is renumbered.** No term enters or leaves
[glossary.md](glossary.md); the entry *nearest match* is amended. No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **A misspelt configuration key is suggested over the key space** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-007` draws the suggestion for
  `tpl cfg get` over every key of `FR-CONF-002`, with the entry segment bound
  to the entries the file declares, and the `hint` states each candidate the
  file does not set. `FR-CFG-012` applies the same rule to `tpl cfg unset`.
  The code stays `66`.
- **A `password_command` string that cannot be what the caller meant is
  refused** — [configuration-model.md](configuration-model.md).
  `FR-CONF-046` refuses with `64`, and writes nothing, a string that yields no
  word, leaves a quote unclosed, ends in a backslash outside any quoted run,
  or begins with `[`. The first two rows state what the implementation already
  refused; the last two are new. `FR-CONF-025` and `FR-CFG-046` carry notes.
- **A `hint` command acts on the same project and entry** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-043` makes a
  runnable `tpl` command in a `hint` carry the `--tpl-dir` and `-d` the
  invocation was given, immediately after `tpl`, except on a node where the
  flag has no effect and in a `hint` that exists to change or remove the flag.
  A value either set refuses leaves a placeholder, named in words.
  `BR-ERR-004` carries a note.
- **The `--context` path is a governed path** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md) and
  [security.md](security.md). `FR-ERR-041` names the path given to
  `--context`, other than `-`, among the paths its set governs, and
  `FR-SEC-019` restates it. The set is unchanged.

`FR-CFG-009`, `FR-CFG-010`, `FR-CFG-011`, `FR-CONF-023`, `FR-CONF-024`,
`FR-CONF-026`, `FR-CONF-035`, `FR-CONF-042`, `FR-ERR-009`, `FR-ERR-019`,
`FR-ERR-022`, `FR-ERR-023`, `FR-ERR-035`, `FR-ERR-037`, `FR-ERR-040`,
`BR-ERR-003`, `FR-GLOB-007`, `FR-GLOB-009`, `BR-GLOB-001`, `FR-RND-018`,
`FR-PROJ-008` and `FR-PROJ-026` were read against the changes, and none
conflicts with them, so none is amended. No security rule yields: the key
space is a literal of `FR-ERR-022`, an entry name stays under its set, a path
under the set of `FR-ERR-041`, and no credential enters a message. Neither
root document paraphrases an amended requirement: the root `README.md`
describes `password_command` as one string on the command line that `tpl`
splits, which stays true. The fifth validation rule therefore owes nothing.

### Forty-eighth edition — the folder `--tpl-dir` names, a project without `.cfg`, and a path that is not expanded

The fifth re-audit of sprint 21, rmp `#263`, found three places where the
system accepted what the caller did not mean and reported success. `tpl
--tpl-dir ../shop cfg database list` read the directory that holds `.tpl` as
the project, reported no entry, and a following `cfg database add` wrote a
`.cfg` beside the real `.tpl` (finding V-01, rmp `#265`). The four `template`
subcommands exited `0` over a malformed `.cfg` while their help listed `78`
(finding V-02, rmp `#272`). And a `${VAR}` given to `ca_file` or `ca_path` was
stored, never expanded, and failed at connection time with a `cause` that did
not say why (finding V-03). This edition fixes the three outcomes.

**Three identifiers are assigned — `FR-PROJ-027`, `FR-PROJ-028` and
`FR-CONF-047`; none is retired and none is renumbered.** No term enters or
leaves [glossary.md](glossary.md); the entry *project* is amended. No entry
of [upstream-divergences.md](upstream-divergences.md) is raised or
discharged, and the index of [open-questions.md](open-questions.md) stays
empty.

- **`--tpl-dir` names a `.tpl` folder, and nothing else** —
  [project-and-discovery.md](project-and-discovery.md). `FR-PROJ-027` admits
  a directory as the `.tpl` folder only where the last segment of its path is
  `.tpl`, as written or after canonicalisation. Any other existing directory
  named by `--tpl-dir` is `78` at step 2, with nothing read and nothing
  written. Where the directory holds a `.tpl` folder, the `cause` says so and
  the `hint` carries the corrected value and, for an invocation without
  operands, the same command with it. `FR-PROJ-008` carries a note, and
  `FR-SEC-016` restates the rule.
- **A `.tpl` folder without `.cfg` is a project with an empty
  configuration** — [project-and-discovery.md](project-and-discovery.md).
  `FR-PROJ-028` pins what the implementation did: step 3 passes, every key
  takes its default, and no entry is declared. It adds one check: such a
  folder SHALL be owned by the current user, or the invocation is `78`,
  because a planted `.tpl` without `.cfg` passed both checks of `FR-SEC-014`
  and would receive the caller's credentials. The first `cfg` writer creates
  the file at `0600` by the procedure of `FR-CFG-041`, which is amended to
  say so; a `cfg` command that changes nothing creates nothing. `FR-PROJ-002`
  and `FR-PROJ-023` carry notes, and `FR-SEC-014` is amended.
- **The `template` subcommands validate `.cfg`** —
  [template-commands.md](template-commands.md). `FR-TMPL-003` states that the
  four subcommands perform step 3 of `FR-ERR-006` and exit `78` over a `.cfg`
  that cannot be used. This was always required: `FR-ERR-006` exempts steps 2
  and 3 only for the commands of `FR-PROJ-025`. `FR-ERR-006` carries a note.
  Their help, which lists `78`, is correct and stays.
- **`ca_file` and `ca_path` refuse `${`** —
  [configuration-model.md](configuration-model.md). `FR-CONF-015` never
  expanded either key, and `FR-CONF-047` refuses a value holding `${`: `64`
  on the command line, with nothing written, and `78` at step 3 in the file.
  The `cause` states that the key is a literal path and is not expanded.
  `FR-CONF-015` carries a note, and `FR-CFG-033` obliges the help of
  `--ca-file` and `--ca-path` to state the refusal.
- **The exit-code table and the path set follow** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). The `78` cell of
  `FR-ERR-001` names the three conditions, and `FR-ERR-041` names the
  corrected `--tpl-dir` value among the paths it governs. `UC-002` in
  [use-cases.md](use-cases.md) gains three alternate flows.

`FR-PROJ-001`, `FR-PROJ-004` through `FR-PROJ-007`, `FR-PROJ-009` through
`FR-PROJ-011`, `FR-PROJ-014`, `FR-PROJ-017`, `FR-PROJ-022`, `FR-PROJ-025`,
`FR-PROJ-026`, `FR-GLOB-009`, `FR-GLOB-010`, `FR-SEC-013`, `FR-CFG-010`,
`FR-CFG-012`, `FR-CFG-027`, `FR-CFG-034`, `FR-CFG-040`, `FR-CONF-014`,
`FR-CONF-034`, `FR-ERR-034`, `FR-ERR-043`, `FR-HELP-031` and `BR-ERR-004`
were read against the changes, and none conflicts with them, so none is
amended. The `78` row of `FR-ERR-034` already obliges the `cause` of a
refused `--tpl-dir` to name the path and why it is not usable. `tpl init`
is unchanged: it uses no `--tpl-dir`, per `FR-PROJ-026`, and still writes
nothing to stdout, per `FR-PROJ-022`. The root `README.md` states that an
absent `.cfg` is not a failure and reads as an empty configuration, which
stays true: the refusal of `FR-PROJ-028` is for the folder's owner, not for
the absence. It describes `--tpl-dir` as naming the `.tpl` folder, and
`${VAR}` as expanded in six fields, both unchanged. The fifth validation rule
therefore owes nothing.

### Forty-ninth edition — a project inside its own folder, a suggestion with nothing in common, and values that could never work

The sixth re-audit of sprint 21, recorded for rmp `#281`, found places where
the system accepted a value that could never do what the caller meant, or
offered a correction that shared nothing with the mistake. `tpl init
proj/.tpl` created `proj/.tpl/.tpl` and warned about a project that did not
exist (finding W-01). With entries `n1` and `shop`,
`tpl cfg database remove zz` answered `did you mean 'n1'?` (finding W-02).
`tpl cfg database show --format json` wrote every value as a string (finding
W-03). `${VAR}` in `core.database` was stored and never expanded, and the same
in `password_command` reached the program with nothing said (finding W-04).
Empty hosts, schemas and entry names, and a reference named `${1X}`, were
stored and failed later with a DNS error or a `hint` no shell accepts
(finding W-06). And the permission message said `.cfg` was read "only at mode
0600" while a file at `0400` was read and rewritten at `0600` (finding W-08).
This edition fixes those outcomes. Findings W-05, W-07 and W-09 are wording
of help and error text that no requirement fixes, and take no change here.

**Six identifiers are assigned — `FR-PROJ-029`, `FR-ERR-044`, `FR-CFG-049`,
`FR-CONF-048`, `FR-CONF-049` and `FR-CONF-050`; none is retired and none is
renumbered.** No term enters or leaves [glossary.md](glossary.md); the entries
*database entry* and *nearest match* are amended. No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **`tpl init` refuses a `.tpl` folder as its destination** —
  [project-and-discovery.md](project-and-discovery.md). `FR-PROJ-029` exits
  `64`, creating nothing, where the last segment of the path as written, or
  of the canonical path of an existing destination, is `.tpl`. The `hint`
  carries `tpl init` with the parent directory. `FR-PROJ-016` is amended so
  that the shadow warning names only a `.tpl` folder that existed before the
  invocation. `FR-PROJ-012` and `FR-PROJ-026` carry notes.
- **A suggestion must keep a character of the name** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-044` admits a
  candidate only where its distance is also strictly less than the length of
  the longer of the two names. `zz`, `ab` and `q` no longer suggest `n1`, and
  every distance-one slip in a name of two characters or more is still
  offered. No separate rule is written for the commands that delete.
  `FR-ERR-019` and `FR-ERR-020` carry notes.
- **The `json` values of `cfg` keep their TOML types** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-049` makes a string a string, an
  integer a number and `password_command` an array in `tpl cfg get`,
  `tpl cfg list` and `tpl cfg database show`. It pins what the first two do and
  changes the third. `FR-CFG-038` carries a note.
- **An entry name is `[A-Za-z0-9_]{1,64}`** —
  [configuration-model.md](configuration-model.md). `FR-CONF-048` applies the
  rule to `cfg database add`, to the `<name>` segment of a key given to
  `cfg set`, and to `core.database`: `64` on the command line, with nothing
  written, and `78` at step 3 in the file. A `${` in `core.database` is
  refused by the rule, with a `cause` that says the key is not expanded. The
  set is the one of `FR-ERR-022`, so every entry name is printable in every
  `hint`; the consequence note of `FR-ERR-022`, `FR-CONF-008`, `FR-CONF-040`
  and `FR-CFG-015` follow.
- **A reference names a variable a shell can define** —
  [configuration-model.md](configuration-model.md). `FR-CONF-049` fixes the
  name of `${NAME}` as `[A-Za-z_][A-Za-z0-9_]*` in the six expanded fields:
  `64` on the command line, where an unclosed reference is refused as well,
  and `78` in the file when the field is expanded, as for an unclosed one.
- **An empty host or database is no host or database** —
  [configuration-model.md](configuration-model.md). `FR-CONF-050` refuses an
  empty `--host`, `--schema` or the matching key with `64`, and treats an
  empty value in the file, as written or after expansion, as the absent key
  that `FR-CONF-040` and `FR-CONF-041` refuse with `78`.
- **`password_command` passes `${VAR}` through and says so** —
  [configuration-model.md](configuration-model.md),
  [cfg-commands.md](cfg-commands.md). `FR-CONF-017` is amended: a reference
  in a word is passed as written and is not refused, because a program the
  caller chose, such as `sh -c`, may expand it. `FR-CFG-033` obliges the help
  of `--password-command` and of `tpl cfg set` to state that the command is
  not expanded and that no `[core]` key is. `FR-CONF-015` carries a note.
- **The mode of `.cfg`** — [project-and-discovery.md](project-and-discovery.md),
  [cfg-commands.md](cfg-commands.md). A note on `FR-PROJ-011` states that the
  check reads the group and other bits only, and that its `cause` does not
  name `0600` as the only mode. `FR-CFG-034` is amended: a rewrite leaves the
  file at `0600` whatever its mode before, and is not refused for a clear
  owner-write bit.
- [use-cases.md](use-cases.md): `UC-001` and `UC-002` gain alternate flows,
  and the glossary entries above are amended.

`FR-PROJ-008`, `FR-PROJ-013` through `FR-PROJ-015`, `FR-PROJ-027`,
`FR-ERR-001`, `FR-ERR-003`, `FR-ERR-005`, `FR-ERR-006`, `FR-ERR-022`,
`FR-ERR-023`, `FR-ERR-037`, `FR-ERR-038`, `FR-ERR-042`, `FR-GLOB-007`,
`FR-CFG-009`, `FR-CFG-010`, `FR-CFG-036`, `FR-CFG-037`, `FR-CFG-041`,
`FR-CONF-021`, `FR-CONF-022`, `FR-CONF-034`, `FR-CONF-046`, `FR-CONF-047`,
`FR-SEC-014` and `FR-SEC-019` were read against the changes. Apart from the
notes named above, none conflicts with them, so none is amended. The cells of
`FR-ERR-001` already characterise every new condition: a value the caller
wrote is `64`, and a `.cfg` that cannot be used is `78`. `FR-SEC-019` still
names the entry name among the values the set governs, which stays true. The
root `README.md` describes `tpl init` as taking an optional path, the mode of
`.cfg` as granting no access to group and other, and `${VAR}` as expanded in
six fields and refused when unclosed or undefined. None of those statements is
made false by this edition, so the fifth validation rule owes nothing.

### Fiftieth edition — a hint that deleted what nobody named, a flag that took the command, and a name in another case

The seventh re-audit of rmp `#263`, recorded for rmp `#282`, found a chain of
`hint` commands, each exiting `0`, that emptied an entry defined by `dsn`. The
connection `hint` offered `--host` for that entry, the conflict `hint` then
offered `tpl cfg unset database.ds.dsn`, and the unset removed the user, the
database and the password reference with nothing said (finding X-01).
`tpl -d schema tables` answered "unknown command 'tables'" (finding X-03).
`tpl init x/.TPL` passed the `.tpl` test on a case-insensitive filesystem
(finding X-04). `tpl cache clean --table nope` exited `0` (finding X-06). The
command tree gave `--port` the default `["3306"]` (finding X-07). A host of
spaces was stored (finding X-08). This edition fixes those outcomes. Findings
X-02 and X-05, and the first part of X-08, are wording of `hint` text that no
requirement fixes, and take no change here beyond the rationale sentence of
`FR-CONF-049` that quoted that text.

**Six identifiers are assigned — `BR-ERR-005`, `FR-ERR-045`, `FR-CFG-050`,
`FR-CLI-026`, `FR-CACHE-040` and `FR-HELP-035`; none is retired and none is
renumbered.** No term enters or leaves [glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **A `hint` deletes nothing the caller did not name** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `BR-ERR-005` bars a
  `hint` command that deletes or overwrites a value of `.tpl/.cfg` or a cached
  object the invocation did not name, and any command that deletes a
  nearest-match candidate. `BR-ERR-004` carries a note.
- **A dsn entry is changed through its dsn** —
  [errors-and-exit-codes.md](errors-and-exit-codes.md). `FR-ERR-045` makes
  every `hint` that repoints or completes an entry defined by `dsn` carry
  `tpl cfg database update <entry> --dsn <url>`. `FR-CONF-041` and
  `FR-SRV-030` carry notes.
- **The conflict `hint` keeps the form the entry uses** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-048` is amended: a table fixes
  the `hint` for each refused pair, and the `cause` states what switching
  between `dsn` and the discrete fields removes. `FR-CFG-050` writes a warning
  when `tpl cfg unset` deletes `dsn`, naming the facts the dsn carried.
- **A flag that took a command says so** — [cli-contract.md](cli-contract.md).
  `FR-CLI-026` reports "`-d` needs a value", with `64`, where `-d` or
  `--tpl-dir` took a command name from a separate token and the next token
  then failed as a command. `FR-CLI-024` carries a note.
- **The `.tpl` segment ignores ASCII case** —
  [project-and-discovery.md](project-and-discovery.md). `FR-PROJ-027` and
  `FR-PROJ-029` are amended.
- **A named clean that finds nothing is `66`** —
  [cache-commands.md](cache-commands.md). `FR-CACHE-040` refuses
  `tpl cache clean` with an object flag whose object is not cached, with a
  suggestion over the cached names and no clean command in the `hint`.
  `FR-CACHE-023` carries a note.
- **A flag default is one string** —
  [help-and-version.md](help-and-version.md). `FR-HELP-035` makes `default`
  `null` or a single JSON string of the command-line text, as `FR-ENV-047`
  does for a template argument. `FR-HELP-020` carries a note.
- **Whitespace alone is empty** —
  [configuration-model.md](configuration-model.md). `FR-CONF-050` is amended:
  a host or a database made only of the six ASCII whitespace characters is
  empty. No other value is trimmed.

`FR-CFG-011`, `FR-CFG-012`, `FR-CFG-020`, `FR-CFG-029`, `FR-CONF-006`,
`FR-CONF-007`, `FR-CONF-040`, `FR-CLI-003`, `FR-CLI-018`, `FR-CLI-020`,
`FR-GLOB-011`, `FR-ERR-001`, `FR-ERR-006`, `FR-ERR-009`, `FR-ERR-019`,
`FR-ERR-043`, `FR-ERR-044`, `FR-OUT-020`, `FR-GLOB-015`, `NFR-PERF-006` and
`FR-ENV-047` were read against the changes. Apart from the notes named above,
none conflicts with them, so none is amended. `FR-CLI-020` bars normalising
the case of a flag value; the path is used as written, and only the test of
its last segment ignores case, as `FR-PROJ-027` now states. The `66` row of
`FR-ERR-001` already covers a named object that does not exist. The root
`README.md` makes no statement about the hints, the cache clean of an absent
object, or the JSON help defaults that this edition makes false, so the fifth
validation rule owes nothing.

### Fifty-first edition — a global flag written for the server database

The tenth re-audit of rmp `#263`, recorded for rmp `#285`, found that
`tpl cfg database add hs4 --host h --database shop` exited `0` with an entry
lacking its server database, and that `tpl cfg database update shop --database
shop2` said that no field flag was given without saying why `--database` did
not count (finding AA-02). `--database` is the long form of the global
`-d/--database`, which has no effect on these commands, per `FR-GLOB-007` and
`BR-GLOB-001`; the server database is written by `--schema`, per `FR-CFG-028`.
This edition keeps that behaviour and makes it said.

**One identifier is assigned — `FR-CFG-051`; none is retired and none is
renumbered.** No term enters or leaves [glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **A global flag with no effect on an entry write is warned about** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-051` writes one warning line to
  stderr when `tpl cfg database add` or `tpl cfg database update` is given
  `-d/--database`, naming `--schema` as the flag that sets the server database.
  The exit code is unchanged, `-q/--quiet` suppresses the line, and the value
  is reproduced only under the set of `FR-ERR-022`. It follows the precedent of
  `FR-PROJ-026`.
- **The empty `update` names the flag it did not count** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-020` is amended: where the
  invocation refused for giving no field flag was given `-d/--database`, the
  `cause` says so and points at `--schema`.
- **The table of flags without effect cites the new requirement** —
  [global-flags.md](global-flags.md). The `-d/--database` row of `BR-GLOB-001`
  adds `FR-CFG-051`.

`FR-GLOB-002`, `FR-GLOB-007`, `FR-GLOB-015`, `FR-GLOB-025`, `FR-CLI-014`,
`FR-CLI-024`, `FR-CFG-016`, `FR-CFG-027`, `FR-CFG-028`, `FR-ERR-006`,
`FR-ERR-022`, `FR-OUT-020`, `FR-OUT-023` and `FR-PROJ-026` were read against
the changes. None conflicts with them, so none is amended. `FR-OUT-023` keeps
stdout empty; the line goes to stderr. The root `README.md` makes no statement
about `-d/--database` on the `cfg` commands that this edition makes false, so
the fifth validation rule owes nothing.

### Fifty-second edition — a warning that advised the wrong flag

The eleventh re-audit of rmp `#263`, recorded for rmp `#286`, found that the
warning of `FR-CFG-051` always advised `--schema <value>` with the
`-d/--database` value, also where `--schema` was given and where the entry is
defined by `dsn`, in which case `--schema` is refused (finding AB-02).

**No identifier is assigned, retired or renumbered.** No term enters or leaves
[glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **The warning names only the flag that can act** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-051` is amended: its last clause
  names the path part of `--dsn` where the invocation uses or targets a dsn,
  is left out where `--schema` was given, and names `--schema <value>`
  otherwise. The line stays one line, and its value rules are unchanged. It is
  written after step 3 of `FR-ERR-006` instead of after step 1, because the
  dsn condition of an `update` is known only once `.tpl/.cfg` is read.
- **The empty `update` names both ways the server database is written** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-020` is amended: where the
  refused invocation was given `-d/--database`, the `cause` says the database
  on the server is set with `--schema`, or is the path part of `--dsn` for an
  entry defined by `dsn`. The refusal is decided at step 1, before `.tpl/.cfg`
  is read, so the `cause` names both forms.

`FR-GLOB-007`, `FR-GLOB-015`, `FR-CLI-014`, `FR-CFG-016`,
`FR-CFG-028`, `FR-CFG-029`, `FR-CFG-048`, `FR-CONF-007`, `FR-CONF-009`,
`FR-ERR-006`, `FR-ERR-022` and `FR-OUT-020` were read against the changes.
None conflicts with them, so none is amended. The root `README.md` makes no
statement about the warning or the `cause` that this edition makes false, so
the fifth validation rule owes nothing.

### Fifty-third edition — a cache that outlived its entry, and a warning that corrected the right flag

The twelfth re-audit of rmp `#263`, recorded for rmp `#287`, found that
`tpl cfg database remove NAME` and `tpl cfg unset database.NAME` leave
`.tpl/.cache/NAME/` in place, that an entry added later under the same name
reads that data with exit `0`, and that `tpl -d NAME cache clean` could not
remove it while no entry declared the name, exiting `66` (finding AC-01). It
also found that the warning of `FR-CFG-051` advised `--schema <value>` where
the `-d/--database` value was the entry name itself (finding AC-02).

The preferred ruling for AC-01, deleting the cache folder with the entry, is
not adopted: `BR-CACHE-004`, `FR-CFG-004` and `FR-CACHE-011` forbid a `cfg`
command to delete or touch cached data. The alternative the brief named is
adopted: `tpl cache clean` may name the cache of a deleted entry, and the
commands that delete or repoint an entry say, without reading the cache, that
its data is kept and which command removes it.

**Four identifiers are assigned — `FR-CFG-052`, `FR-CFG-053`, `FR-CACHE-041`
and `FR-HELP-036`; none is retired and none is renumbered.** No term enters or
leaves [glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **Deleting an entry says its cache is kept** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-052` writes one warning line to
  stderr when `tpl cfg database remove` or `tpl cfg unset` of a whole entry
  exits `0`, naming `tpl -d <name> cache clean`, with nothing on stdout and no
  access to the cache. `FR-CFG-011`, `FR-CFG-022` and `FR-CFG-050` carry
  notes or a sentence that cite it.
- **Repointing an entry says its cache is kept** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-053` writes one warning line to
  stderr when `tpl cfg database update` is given a flag `FR-CACHE-029` names
  and exits `0`, naming `tpl -d <name> cache clean`.
- **The cache of a deleted entry can be cleaned** —
  [cache-commands.md](cache-commands.md). `FR-CACHE-041` lets
  `tpl cache clean` with no object flag remove `.tpl/.cache/<name>` for a
  name of `FR-CONF-048` that no entry declares, ignoring ASCII case, where
  that path exists, with exit `0` and one stderr line. `FR-CACHE-023`,
  `BR-CACHE-003` and `BR-CACHE-004` carry notes, and `FR-ERR-005` and
  `FR-GLOB-025` carry notes that name the exception.
- **The help states the cache facts** —
  [help-and-version.md](help-and-version.md). `FR-HELP-036` makes the
  `DESCRIPTION` of `remove`, `unset`, `add`, `update` and `cache clean` state
  what each does to, or with, an entry's cache, each command carrying
  `-d NAME`.
- **The warning does not correct the entry name** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-051` is amended: a new
  condition 3 ends the line with "the name argument already names the entry"
  where the `-d/--database` value equals the `<name>` operand byte for byte,
  and names neither `--schema` nor any value.
- **The quoted clean acts on the caller's project** —
  [cfg-commands.md](cfg-commands.md) and
  [cache-commands.md](cache-commands.md). Amended within this edition: the
  command in the lines of `FR-CFG-052` and `FR-CFG-053` carries the
  invocation's explicit `--tpl-dir` as `FR-ERR-043` states for a `hint`, with
  a placeholder stated in words where `FR-ERR-041` refuses the value. The
  line of `FR-CACHE-041` carries no command, and names the removed folder as
  the filesystem records it where that differs in case from the name given.
- **The repoint use case covers removal** — [use-cases.md](use-cases.md).
  `UC-011` gains an alternate flow for removing and adding an entry.

`FR-CFG-004`, `FR-CFG-012`, `FR-CFG-020`, `FR-CFG-023`, `FR-CACHE-002`,
`FR-CACHE-011`, `FR-CACHE-015`, `FR-CACHE-028`, `FR-CACHE-029`,
`FR-CACHE-036`, `FR-CACHE-040`, `FR-CONF-048`, `FR-GLOB-006`, `FR-GLOB-007`,
`FR-GLOB-015`, `BR-GLOB-001`, `FR-ERR-001`, `FR-ERR-006`, `FR-ERR-022`,
`BR-ERR-005`, `FR-ERR-041`, `FR-ERR-043`, `FR-OUT-020`, `FR-OUT-023` and
`FR-HELP-031` were read against
the changes. Apart from the notes named above, none conflicts with them, so
none is amended. The warning lines are not `hint` lines, and the cache they
name belongs to the entry the invocation names, so `BR-ERR-005` is not
engaged. The root `README.md` states that removing the entry `core.database`
names clears it silently; that remains true, so the fifth validation rule
owes nothing.

### Fifty-fourth edition — a repoint through the key of a field

The thirteenth re-audit of rmp `#263`, recorded for rmp `#289`, found that
`tpl cfg set database.NAME.host` and `tpl cfg unset` of such a field change
where an entry points with exit `0` and nothing on stderr, while
`tpl cfg database update` given the same field warns that the entry's cache is
kept (finding AD-01).

**No identifier is assigned, retired or renumbered.** No term enters or leaves
[glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **Setting or unsetting a field says the cache is kept** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-053` is amended: its line is
  also written when `tpl cfg set` or `tpl cfg unset` is given
  `database.<name>.host`, `port`, `user`, `database`, `tls` or `dsn` and
  exits `0`, with the same content, the same `--tpl-dir` rule and no access
  to the cache. Item 4 places the line of `FR-CFG-050` first where both are
  written, and a new item 7 makes the line independent of what the file held
  before. `FR-CFG-008`, `FR-CFG-011`, `FR-CFG-050` and `FR-CFG-052` gain a
  sentence that cites it.
- **The help states the fact** —
  [help-and-version.md](help-and-version.md). `FR-HELP-036` gains a row for
  `tpl cfg set`, and its `tpl cfg unset` row a clause for one field, each
  carrying `tpl -d NAME cache clean`.
- **The rule on repointing records the extension** —
  [cache-commands.md](cache-commands.md). `BR-CACHE-003` carries a note.

`FR-CFG-004`, `FR-CFG-048`, `FR-CFG-051`, `FR-CACHE-002`, `FR-CACHE-011`,
`FR-CACHE-029`, `BR-CACHE-004`, `FR-CONF-048`, `FR-ERR-022`, `FR-OUT-020`,
`FR-OUT-023`, `FR-GLOB-015`, `FR-HELP-031` and `UC-011` were read against the
changes. None conflicts with them, so none is amended: the line writes to
stderr only, reads nothing under `.tpl/.cache/` and deletes nothing, so
`FR-CFG-004`, `FR-CACHE-011` and `BR-CACHE-004` hold. The root `README.md`
makes no statement about what `tpl cfg set` or `tpl cfg unset` writes to
stderr, so the fifth validation rule owes nothing.

### Fifty-fifth edition — the whole database block, and a warning that states only what it knows

The fourteenth re-audit of rmp `#263`, recorded for rmp `#290`, found that
`tpl cfg unset database` deletes every entry with exit `0` and nothing on
stderr, leaving each entry's cache to be read by an entry added later under
the same name (finding AE-01), and that the line of `FR-CFG-053` states as
fact a repoint and a cache the command cannot know of (finding AE-02).

**No identifier is assigned, retired or renumbered.** No term enters or leaves
[glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **Unsetting the whole block warns for every entry** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-052` is amended: `tpl cfg unset
  database` that exits `0` writes one line per deleted entry, in file order,
  each with its own `tpl -d NAME cache clean` carrying the invocation's
  `--tpl-dir`, with no access to the cache; `-q` suppresses them. Refusing
  the command was rejected by the coordinator of rmp `#290`. `FR-CFG-011`
  and `FR-CFG-053` gain a clause that cites it.
- **The repoint line is true in every case** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-053` item 1 is amended: the
  line says the entry may now point at another server and that any data
  cached for it is kept and still served, whether the entry is new, its value
  unchanged, or no cache exists. The command carried and every other item are
  unchanged.
- **The help states the fact** —
  [help-and-version.md](help-and-version.md). The `tpl cfg unset` row of
  `FR-HELP-036` gains a clause for the key `database`, and the rows for
  `tpl cfg database remove`, `tpl cfg set`, `tpl cfg database add` and
  `tpl cfg database update` say "any data cached", so that each holds where
  no cache exists.
- **The rule on repointing records both changes** —
  [cache-commands.md](cache-commands.md). `BR-CACHE-003` carries a note.

`FR-CFG-004`, `FR-CFG-023`, `FR-CFG-050`, `FR-CACHE-002`, `FR-CACHE-011`,
`FR-CACHE-041`, `BR-CACHE-004`, `FR-CONF-048`, `FR-ERR-022`, `FR-ERR-043`,
`FR-OUT-020`, `FR-OUT-023` and `FR-GLOB-015` were read against the changes.
None conflicts with them, so none is amended: `FR-CFG-023` already clears
`core.database` whenever the entry it names is deleted, by any command. The
root `README.md` makes no statement about what `tpl cfg unset database`
writes, and does not quote the line of `FR-CFG-053`, so the fifth validation
rule owes nothing.

### Fifty-sixth edition — a removal warning that asserted a cache

rmp `#291` found that the line of `FR-CFG-052` states as fact that data cached
for the deleted entry is kept, which is false where no cache exists, while the
rows of `FR-HELP-036` and the line of `FR-CFG-053` say "any data cached".

**No identifier is assigned, retired or renumbered.** No term enters or leaves
[glossary.md](glossary.md). No entry of
[upstream-divergences.md](upstream-divergences.md) is raised or discharged,
and the index of [open-questions.md](open-questions.md) stays empty.

- **The removal line holds where no cache exists** —
  [cfg-commands.md](cfg-commands.md). `FR-CFG-052` item 1 and its three
  examples say "any data cached for it under `.tpl/.cache/NAME/` is kept".
  The facts named, the command carried and every other item are unchanged.

`FR-CFG-053`, `FR-HELP-036`, `FR-CACHE-011` and `BR-CACHE-004` were read
against the change. None conflicts with it, so none is amended. The root
`README.md` does not quote the line of `FR-CFG-052`, so the fifth validation
rule owes nothing.

### Still out of scope

- The Rust implementation: its crates, its module layout, its types, and its
  library API. Only the JSON document and the command line are contract, so the
  shape of the library is an architecture decision and not a requirement. The
  database driver is part of this: `FR-CONF-036` states what one must be able
  to express, and no requirement names one.
- The text of any catalogue query. What is read is specified; how it is read is
  not.
- Every **measured** performance figure and every reading. Those live in
  `BENCHMARKS.md`, cited by the requirement that needs them. A measurement point
  that has not been measured may carry an **adopted** reference figure in
  `NFR-PERF-014`, marked as such under `NFR-PERF-019` and removed from this
  corpus by the same step that records the real one, per `NFR-PERF-020`. No
  figure in either place fails a change, per `BR-PERF-008`.
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
| [examples.md](examples.md) | `EX` | The four worked examples in `examples/`: what one is, what it reads, what it holds, where it runs, and what makes it correct |
| [catalogue-coverage.md](catalogue-coverage.md) | `CAT` | What enters the model from the catalogue, and what is excluded |
| [context-document.md](context-document.md) | `CTX` | The structure of the document that carries the model |
| [template-environment.md](template-environment.md) | `ENV` | Filters, tests, global functions, and what is contract |
| [render-semantics.md](render-semantics.md) | `SEM` | Whitespace, operands, null versus absence, boolean rendering, author-signalled failure |
| [server-contract.md](server-contract.md) | `SRV` | The supported version window, differences between series, the closed statement list, the read-only promise |
| [privileges-and-completeness.md](privileges-and-completeness.md) | `PRIV` | Complete and incomplete reads, and how a short read is reported |
| [cache-documents.md](cache-documents.md) | `CDOC` | Cache versions, completeness records, and the `source` field |
| [performance-requirements.md](performance-requirements.md) | `PERF` | Requirements of form, reference workloads, measurement protocol, the measurement set |
| [open-questions.md](open-questions.md) | `OQ` | Points this specification cannot yet fix |
| [upstream-divergences.md](upstream-divergences.md) | `DIV` | Corrections owed to the root `CLAUDE.md` and `README.md`, and what discharged each |

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
  are the same actor. **One class of requirement takes another subject, and it
  is named here so that it is not read as drift.** Where a requirement fixes an
  artefact of the repository rather than a behaviour of the binary, the
  artefact is the subject: `FR-CONF-038`'s second paragraph obliges the fixture
  of `scripts/mariadb/`, and every requirement of [examples.md](examples.md)
  obliges a worked example. The test is whether the statement could be
  satisfied by a change to `tpl` at all; where it could not, `tpl` is the wrong
  subject and naming it would make the requirement unsatisfiable rather than
  conventional. *Stated in the thirty-fourth edition, which wrote the second of
  the two.*
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
- **A term that has a definition has exactly one, and it is in
  [glossary.md](glossary.md).** That file states one half of the rule of
  itself — a term used in a requirement without an entry there is a defect —
  and the thirty-second edition adds the other: a definition written anywhere
  else in this corpus is the same defect. A requirement that needs its term
  explained beside it cites the entry and does not restate it. **It is
  checkable against an edition under review**: for every term an edition
  defines, marks out or leans on, ask whether [glossary.md](glossary.md) holds
  an entry for it and whether the entry and the passage say the same thing.
  Two terms were moved under this rule in the edition that wrote it —
  *in flight* and *differential run* — and the **sweep of all twenty-six files
  for the rest was made in the thirty-third**, which moved five more and
  removed one duplicate. What the rule reaches, and the three shapes it does
  not, are stated in [glossary.md](glossary.md#what-counts-as-a-definition)
  with the alternative rejected beside them; a note shape of the two bullets
  above is one of the three, which is why naming one here is not defining a
  term elsewhere.
- **Where a verification stops, rather than a guarantee, the same note shapes
  are used and the three-way family stays three.** The family above is limits
  on what `tpl` guarantees about a server, a table, or a trust store, and its
  members cite each other. A limit on the **evidence** for a guarantee — the
  observation that cannot be made, rather than the promise that cannot be kept
  — takes the same note shapes, as many of them as it has content for, without
  joining that family, because it qualifies a different kind of claim. Four
  requirements are written this way:
  `FR-ERR-031`, where no invocation of the distributed binary is observed
  returning `70`; `NFR-PERF-005`, where the file-open observation is made
  on the Linux targets and not on the macOS ones; `FR-PRIV-021`, where no
  invocation is observed producing a schema read that returns no row; and
  `FR-SRV-013`, where no server is observed producing a read-back that does not
  confirm the read-only session, so that half of it is exercised in process.

## Status legend

| Status | Meaning |
|---|---|
| `draft` | Written from a settled decision, not yet reviewed against the corpus |
| `approved` | Reviewed and in force |
| `deprecated` | Superseded; retained for traceability, not to be implemented |

## Provenance

Every requirement in this specification derives from one of **four** sources,
numbered 1 through 4 below. The fifth item is not a fifth source: it is the
rule that closes the list.

*Corrected in the thirty-first edition: the opening counted three sources above
a list of five.* Three was true of the list when it was written — decision
logs, the root documents, and *Nothing else* — and two sources have been added
since without the count moving: the fourth edition added the published external
authority and the sixth the direct observation, each taking the number below
the one it displaced. The count is corrected and **no item is renumbered**,
because seven passages of this corpus cite *the fourth provenance* and every
one of them means item 4, the direct observation — `FR-SRV-041` and `FR-SRV-038` in
[server-contract.md](server-contract.md), `FR-CAT-053`, `FR-CAT-055` and
`FR-CAT-056` in [catalogue-coverage.md](catalogue-coverage.md), the
twenty-seventh edition's record above, and the note on recurring obligations in
[open-questions.md](open-questions.md). Renumbering to make four items into four
numbers would silently redirect all seven.

*Rejected: splitting the five into three sources with sub-items.* There is no
grouping under which the four are three. The decision logs, the root documents,
the published authority and the direct observation are four kinds of ground
with four different ways of decaying, which is the whole reason each is stated
apart. *Rejected: lifting item 5 out of the numbered list.* It is the rule that
makes the other four exhaustive, and it is read where a reader is counting
them; moving it to a paragraph below would leave the list looking open.


1. Seven decision logs. The first interview settled 53 points about the CLI
   surface; the second settled 28 points about the model, the document, the
   template surface, the server contract, and performance, and recorded four
   defects found in the first edition; the third is the audit of 2026-09-10,
   whose fifteen findings the user answered with fourteen decisions and one
   deliberate deferral; the fourth settled the supported version window; the
   fifth put every remaining decidable question to the user and settled all
   thirty-three, four of them against the recommendation; the sixth settled
   the four points the observation raised, one of which — `FR-SRV-039` —
   reversed a recommendation and reordered two values this corpus had held to
   be compatible; the seventh, of 2026-09-21, settled the one point the joined
   arms raised, that a boolean interpolated into a generated file renders
   `true` and not `True`, which `FR-SEM-021` writes in. Each
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

**One item is outstanding**, recorded by the twentieth edition and listed
below. The eight items this section has held before it are all discharged; the
most recent of those was recorded by the thirty-second edition and discharged
by the thirty-third, inside one sprint. The item this section carried through
the sixth edition — twenty catalogue field lists that no observation had
recorded — was discharged by the
seventh: the fixture was read against all four series of `FR-SRV-015` on
2026-09-10, every field list was recorded verbatim, and each is now a
requirement in [catalogue-coverage.md](catalogue-coverage.md) or
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
it. **It is discharged**: `scripts/mariadb/` carries TLS material of its own
and configures it at every series, and the fourteenth edition records what
that changes for the observation record of `FR-SRV-038`. `DIV-045`, recorded
in the same edition, was a correction owed to the root coordination document;
the ninth edition discharges it, because `FR-ERR-030` as amended and the
profile that document states no longer contradict each other.

The ninth edition adds no obligation of either kind. Its four changes are
corrections of wording, each stated beside the requirement it changed.

The tenth edition adds no obligation of either kind either. It names a variable
that two requirements had left to the implementer, records a twelfth difference
between the series, and widens one verification that was already owed —
`FR-SRV-013`, which at the time was blocked, as every test this corpus mandates
then was, by `tpl` not existing.

*Corrected in the twenty-sixth edition.* The clause read that `FR-SRV-013`
"like every test this corpus mandates is blocked only by `tpl` not existing",
in the present tense and over every mandated test at once. The binary exists,
so the clause named a condition that does not hold, and what a mandated test
waits on is the command it drives, which differs from test to test.
`BR-SCH-004` now records its own block, in
[schema-commands.md](schema-commands.md); `DIV-036`, which made the same claim
of three tests, is corrected with it.

The eleventh edition adds no obligation of either kind. It records a limit on
where one observation can be made, adds an instrument that closes the resulting
gap on every target, and leaves the requirement it qualifies saying what that
instrument does not establish. One obligation outside this corpus is narrowed
rather than created: the file-open observation of `NFR-PERF-005` is owed on the
two Linux targets and is owed on neither macOS target, so a verification suite
that skips it there is conforming and not incomplete.

The twelfth edition adds no obligation of either kind. It records a difference
`FR-SRV-038` already obliged this corpus to hold, checks the two requirements
that difference bears on and leaves both as written, and adds no row to the
register of `FR-SRV-036`. The one obligation it did record — a defect it found
while recomputing a tally and did not correct at the time — was discharged
inside the same edition and is listed below.

The thirteenth edition adds no obligation of either kind either. It records a
difference `FR-SRV-038` already obliged this corpus to hold, recomputes the
counts that follow it, checks the twelfth edition's classification of the
thirteen against the rows, and adds no row to the register of `FR-SRV-036`. It
does record one item of debt, and that item stood outstanding until the
eighteenth edition discharged it.

The fourteenth edition adds no obligation of either kind, and discharges one
recorded outside this corpus by the eighth. It corrects one row of the record
of `FR-SRV-038`, which was true of servers nobody had configured and false of
the servers the fixture now runs, checks the other thirteen rows against the
same defect and finds none, and amends one note of `FR-CONF-038` for the same
reason. The item below was untouched by it and stayed outstanding.

The fifteenth edition adds no obligation of either kind, and records none as
discharged. It classifies all forty-five entries of
[upstream-divergences.md](upstream-divergences.md) against the two root
documents as they stand, fixes what becomes of a discharged entry and of a
partly discharged one, and adds the fifth validation rule below. It changes no
requirement. It does confirm one obligation already recorded outside this
corpus and narrow another: `seed-bench.sql` is still absent from
`scripts/mariadb/` under `DIV-036`, and the project-tree half of that entry is
discharged, because the tree's fixture line now describes what the directory
holds instead of listing its files. The item below was untouched by it and
stayed outstanding.

The sixteenth edition adds no obligation of either kind, and records none as
discharged. It adds seven entries to
[upstream-divergences.md](upstream-divergences.md), all due — four owed to the
root `README.md` and three to `CLAUDE.md` — gives that register a third kind of
entry for the four of them that contradict the repository rather than a
requirement, and extends the fifth validation rule below to cover the
divergence a register does not yet hold. It changes no requirement. All three
findings it first recorded rather than acted on have been acted on. The
project-structure tree of `CLAUDE.md` names a crate and five directories the
repository does not have, and `DIV-050` is the entry that divergence needed;
the eight `cargo` commands of that file's *Desenvolvimento* section and of the
validation pipeline under it fail in this repository for the reason `DIV-046`
records of *Development* in the other file, and `DIV-051` is theirs; the
*Disciplina de medição* subsection of the same file says the benchmarks live in
`benches/`, a directory the repository has not got and where no benchmark
exists to live, and `DIV-052` is its. The three are recorded apart because the
corrections differ and the passages are edited apart, which is set out above.
One observation is named and not recorded: the sweep that asked whether any
passage of that class is left in `CLAUDE.md` found one candidate of a different
shape — *Plataformas Suportadas* calls Linux and macOS "suportados e
verificados" where nothing has been verified on any target — and whether that
asserts a state or names a class is a judgement for a reading of that section.
It is an observation about a file this corpus does not own, so it is debt only
in the sense `DIV-036` is, and it blocks nothing here. The item below was
untouched by it and stayed outstanding.

The seventeenth edition adds no obligation of either kind, adds no entry to
[upstream-divergences.md](upstream-divergences.md), and changes no
requirement. It **discharges the observation the sixteenth edition named and
could not judge**: the reading of *Plataformas Suportadas* finds that
"suportados e verificados" declares a policy rather than asserting a state, so
nothing is owed for it and no entry is opened; the reading, its four grounds
and the sweep that closes the **Overstatement** class in `CLAUDE.md` are at
*[A candidate read, and not
recorded](upstream-divergences.md#a-candidate-read-and-not-recorded)*. One
caution is left with `DIV-041`, whose correction to that same section removes
the first of those grounds and therefore obliges whoever makes it to re-read
the adjective it leaves behind; and one observation is named and not acted on,
that the section's sentence naming Linux and macOS states coarsely what
`NFR-PERF-018` fixes exactly. Both are about a file this corpus does not own,
so neither is debt here in any other sense than `DIV-036` is. The item below
was untouched by it and stayed outstanding.

The eighteenth edition adds no obligation of either kind, and discharges the
one item this section still carried. It amends `FR-SRV-038` to state what the
home below its table actually holds, records three readings of the build there,
corrects one stale count and checks a second that does not move, and adds no
row and no register entry. It records no obligation outside this corpus
either: the observation its passage names would
settle a classification that nothing in this corpus reads, so no requirement
waits on it and no test is owed for it.

The nineteenth edition adds no obligation of either kind, and records none as
discharged. It amends `FR-ERR-001` to say which of its two columns is
exhaustive, adds `FR-ERR-035` to route a configuration key to the code it
already produced, and changes nothing about `tpl`. One question is named and
not settled — whether every table in this corpus that characterises rather than
enumerates should say which it does — and it is not debt in any sense the five
rules use: nothing waits on it, no requirement is ambiguous because of it, and
the instance that raised it is closed. It is left for a reading that covers the
corpus, on the ground the sixteenth edition used when it named a candidate it
could not judge from the reading it had made — the seventeenth edition then
made the reading, and judged.

The twentieth edition adds one obligation inside this corpus and none outside
it, and records none as discharged. It settles five statements a derivation of
the error type and the diagnostic renderer could not apply without choosing:
`FR-ERR-002` now states an obligation ten codes can satisfy, `FR-ERR-022` says
what its character set governs and stops contradicting `FR-ERR-021`,
`FR-ERR-010` says which requirement governs where a `cause` repeats its `error`
line, `NFR-DET-004` says whose bytes it governs, and `FR-GLOB-015` names the
refusal `FR-CLI-015` has carried since the first edition. One of the five
changes what `tpl` does — a suggestion may now name a configuration key and a
hyphenated flag — and the other four state what was already in force. The
obligation it records is below: the decision `NFR-DET-004` deliberately did not
take from its own side.

The twenty-first edition adds no obligation of either kind, and records none as
discharged. It fixes the shape of the JSON document `tpl help --format json`
emits, for which this corpus had fixed a command and not a payload:
`FR-HELP-017` gains a fourth key of `data` and declares that set open,
`FR-ENV-005` fixes what the key holds, and `FR-HELP-019` settles that
`data.commands` is flat, which `FR-HELP-016` and `FR-HELP-029` then read as a
selection rather than a nesting. It reverses no decision an earlier edition
took: both defects were shapes nobody had fixed rather than statements that had
decayed, so none of the five rules below would have found either, and what
found them was a derivation of the command from the corpus. The item below was
untouched by it and stayed outstanding.

The twenty-second edition adds no obligation of either kind, and records none
as discharged. It closes three places where a `cfg` command this corpus admits
wrote a `.tpl/.cfg` the same corpus refuses to read: `FR-CFG-031` now admits
what `FR-CONF-009`, `FR-CONF-010` and `FR-CONF-011` admit and nothing else,
`FR-CFG-048` refuses a write that would leave an entry in a combination
`FR-CONF-007` refuses, and `FR-CFG-023` reaches `tpl cfg unset` as well as
`tpl cfg database remove`. None of the five rules below would have found any of
the three: each was a rule stated over the invocation where the state it
protects is the entry, or over one command where two reach the state, and what
found them was the first implementation written against this corpus — the
instrument that found the twentieth and twenty-first editions' defects as well.
One observation is named in that edition and not acted on, that `FR-CONF-010`
names no code for refusing a scheme it does not accept; nothing waits on it,
and no requirement of the edition rests on it. The item below was untouched by
it and stayed outstanding.

The twenty-third edition adds no obligation of either kind, and records none as
discharged. It corrects `FR-SRV-040`, which recorded a malloc-library variable
as returning no row where it returns one on every server of the fixture; it
discharges one of the two grounds on which the passage below the table of
`FR-SRV-038` declines a row to two readings of the build, leaving the
classification to decline it alone; it states the emitted value of a routine's
`kind` and its relation to the qualified form of `FR-SCH-008`; it adds
`FR-CAT-053`, the index of a table object's properties; and it scopes one
enumeration to the subject it always had. Two observations are named in it and
neither is debt. The table catalogue's own field list has never been recorded,
and `FR-CAT-053` names the pass against the fixture of `scripts/mariadb/` that
would record it; nothing waits on it, because every property a requirement in
force gives a table is named in that index. And the observation the eighteenth
edition's passage asked for is half taken: the run of 2026-09-18 supplies the
two readings' values and compares no two builds of one series, which is the
half that would settle the classification and the half that is still untaken.
The item below was untouched by this edition and stays outstanding.

The twenty-fourth edition adds no obligation of either kind, and records none
as discharged. It settles the order in which the three connection-start
statements are issued, which `FR-SRV-012` and `FR-ERR-006` ordered differently
and without which the test that `FR-SRV-012` mandates could not be written:
`FR-SRV-042` states the order once, the table of `FR-SRV-006` is declared the
enumeration it always was, `FR-SRV-012` cites rather than restates, and
`FR-ERR-006`, `FR-SRV-002` and `FR-CFG-024` are checked and left as written.
None of the five rules below would have found it, and what did was the first
implementation of the connection start. Two statements about files this corpus
does not own are named in it and neither is debt here, in any other sense than
`DIV-036` is: `docs/spec-technical/architecture.md` records the discrepancy
this edition settles and resolves it the same way, and
`scripts/mariadb/probe-session.sql` demonstrates the rejected order, both as
checked on 2026-09-18. Neither is an entry of
[upstream-divergences.md](upstream-divergences.md), whose scope is the root
`README.md` and the root `CLAUDE.md`, and nothing in this corpus waits on
either. The item below was untouched by this edition and stays outstanding.

The twenty-fifth edition adds no obligation of either kind, and records none as
discharged. It states six conditions the first arm can reach and no requirement
named: two entries that describe no connection and no read, a session that
opens and does not hold, a database whose schema row is absent, the casing of
the qualified routine prefix, and the layout rule that produces the one worked
`text` listing this corpus carries. No exit code is added, and every condition
carries a code of `FR-ERR-001` already in force. None of the five rules below
would have found any of the six: four were found by the first implementation of
the connection and the catalogue reader, and two by reading this corpus back
against the layout and the document it now has, and all six were shapes nobody
had fixed rather than statements that had decayed. Two observations are named
in that edition's own requirements and neither is debt: `FR-PRIV-021` states
that no invocation of the distributed binary is observed producing its
condition, which is a limit on evidence and joins `FR-ERR-031` and
`NFR-PERF-005` under *[Writing conventions](#writing-conventions)*; and
`FR-SCH-026` states the layout rule for the listings of the first arm alone,
because those are the only listings this corpus works. The item below was
untouched by this edition and stays outstanding.

The twenty-sixth edition adds no obligation of either kind, and records none as
discharged. It fixes five statements the first arm needs and this corpus did
not make: the wording of a suggestion that names more than one candidate,
whether the comparison behind it folds case, which Damerau-Levenshtein distance
`FR-ERR-019` measures, the catalogue source and the observed value of a table's
engine and collation, and the casing of the routine kind in a cache path. It
also corrects two statements this corpus makes about the repository, which were
true when they were written and had stopped being so — the clause above about
`FR-SRV-013`, and the same clause in `DIV-036`. One decision recorded outside
this corpus is brought inside it: the variant of the edit distance, which
decides which candidate a `66` offers, is now `FR-ERR-039`. What remains of the
technical open decision that held it is a matter for the owner of
`docs/spec-technical/`, which this corpus does not write, and nothing here waits
on that: `FR-ERR-039` is complete on its own terms. The item below was untouched
by this edition and stays outstanding.

The twenty-seventh edition adds no obligation inside this corpus and **one
outside it**, and records none as discharged. It answers the four questions the
finished first arm had to choose for itself, and each answer is a statement the
code at `db7337d` now contradicts, which is a matter for the owner of that code
and not debt of this corpus: a requirement is not owed work because an
implementation predates it. The obligation outside this corpus is the
observation `FR-CAT-057` could not make. **The fixture of `scripts/mariadb/`
declares one user schema and carries no cross-schema foreign key**, so what a
server returns for one is unknown here; the requirement excludes the feature,
which needs no observation, and names the DDL that would settle the behaviour
if the exclusion is ever revisited. It is fixture work with an owner and a
trigger, in the terms the eighth edition used for `FR-CONF-038`, and no
requirement of this corpus is waiting on it: `FR-CAT-057` is complete on its
own terms today. The item below was untouched by this edition and stays
outstanding.

The twenty-eighth edition adds no obligation of either kind, and records none
as discharged. It settles the one requirement the first arm's verification from
outside the process could not satisfy: `FR-SRV-013` demanded an integration
test over both outcomes of the read-back, and the failing outcome is reachable
from no server and from no seam an integration test can see. That half is now
verified in process, through a seam the requirement authorises on
`FR-ERR-031`'s terms; `BR-SRV-003` yields for that clause in its own text, and
`BR-SCH-004`'s count follows. None of the five rules below would have found it:
the requirement was internally coherent, correctly cross-referenced and counted
right, and what found it was writing the test it mandates. The test form it now
names is owed by the code that carries the read-back, which is work a
requirement in force obliges rather than debt of this corpus. The item below
was untouched by this edition and stays outstanding.

The twenty-ninth edition adds no obligation of either kind, and records none
as discharged. It writes down what an interpolated boolean produces, which no
requirement had fixed, and corrects one rationale that claimed an identity its
own table disproves. Neither touches a fixture, a measurement, or a file this
corpus does not own. What `FR-SEM-021` now requires is owed by the code that
renders, which is work a requirement in force obliges rather than debt of this
corpus. The item below was untouched by this edition and stays outstanding.

The thirtieth edition adds no obligation of either kind, and records none as
discharged. It moves one step of `FR-ERR-006` and corrects the eight sentences
that numbered a step the move displaced. Nothing it changes touches a fixture
or a measurement. Two things outside this corpus now follow the order it
replaced: the code that implements the old position, and every citation of a
step of `FR-ERR-006` by number in `docs/spec-technical/` and in the comments of
the implementation and its tests. Both are owed by owners outside this corpus —
work a requirement in force obliges, in the terms the twenty-eighth and
twenty-ninth editions used, rather than debt of this one. The item below was
untouched by this edition and stays outstanding.

The thirty-first edition adds no obligation of either kind, and records one as
discharged outside this corpus. It settles fourteen points the backlog had
held, of which two change what `tpl` does — a valueless flag given twice is now
accepted, per `FR-CLI-025`, and every flag and argument now states what it does
in its help, per `FR-HELP-030` — and the rest state what was already in force,
qualify a rule stated wider than its ground, or correct a sentence that had
stopped being true. Nothing it changes touches a fixture or a measurement. The
entry it discharges is `DIV-034`, whose `README.md` half `e75996c` removed when
it rewrote that file, recorded with its commit in
[upstream-divergences.md](upstream-divergences.md) under the fifth validation
rule below.

That same rule has a trigger and the trigger has fired. Both root documents
were edited since the fifteenth edition's classification of the register, so a
re-read of all fifty-two entries against both files is owed, in both
directions. This edition re-read the one entry it amends and did not make that
pass, and says so where the register states its own figures. Like `FR-SRV-019`
below it is an obligation with an owner and a trigger rather than debt of this
corpus: no requirement here waits on it, and nothing in this specification is
ambiguous while it stands. The item below was untouched by this edition and
stays outstanding.

*Corrected in the thirty-second edition, and the obligation is discharged.*
The paragraph read *Both root documents were edited at `db80114`, and the root
`README.md` twice more before it*, and `db80114` touched `README.md` alone.
`CLAUDE.md` was edited five times after `87dd6e3`, at `c6356df`, `b066cfa`,
`cd6ce7e`, `6a0cce5` and `8f936d4`, and `README.md` three, at `e75996c`,
`db7337d` and `db80114`. The trigger had fired far harder than the sentence
said. The same clause appears in the register's own sweep and is corrected
there with it. **The pass is made**, and what it found is the thirty-second
edition's record above.

The thirty-second edition adds one obligation inside this corpus and none
outside it, and discharges the one recorded immediately above. It pays the
register in full — fifty-two entries re-read against both root documents,
thirty of them discharged with the commit that discharged each, and two
divergences raised that the register did not hold — governs the two
`password_command` conditions `FR-ERR-002` left ungoverned, and decides the two
questions `FR-CONF-014` had never answered. Nothing it changes touches a
fixture or a measurement. What `FR-CONF-014` and `FR-CONF-044` now require is
owed by the code that assembles trust material, and what `FR-CONF-043` requires
of a `cause` is owed by the diagnostic renderer; both are work a requirement in
force obliges rather than debt of this corpus, in the terms the twenty-eighth
through thirtieth editions used. The obligation it does add is the sweep below.

The thirty-third edition adds no obligation of either kind, and discharges the
one the thirty-second recorded. It makes that sweep, restates `BR-PERF-007`
over a fixture that is now complete, names a fourth kind in the `74` cell of
`FR-ERR-001`, and records `NFR-PERF-001` as measured without ratifying
anything. Nothing it changes touches a fixture or a measurement, and nothing
it changes changes what `tpl` does; one sentence of the root `CLAUDE.md` is
owed under `DIV-036` and was owed before, more widely.

The thirty-fourth edition adds no obligation inside this corpus and **one
outside it**, and records none as discharged. It describes a folder the
repository does not have: `examples/`, the four worked examples in it, the
schemas they read and the server they read them from are all owed by the sprint
that writes them, which is work a requirement in force obliges rather than debt
of this corpus, in the terms the twenty-eighth through thirtieth editions used.
The obligation outside this corpus is the one `FR-EX-006` and `FR-EX-007` name
and deliberately do not own: a server of the most recent series of `FR-SRV-015`
carrying `sakila`, `world` and `freight`, with `freight` authored to carry the
native types and catalogue features the two published datasets do not. It is
fixture work with an owner and a trigger, in the terms the eighth edition used
for `FR-CONF-038`, and no requirement of this corpus waits on it: every
requirement of [examples.md](examples.md) is complete on its own terms today.
The second half of the edition — the two clauses of `FR-CONF-014` that
described the code, and the order that requirement now cites rather than
restates — touches no fixture and no measurement. The item below was untouched
by this edition and stays outstanding.

The thirty-fifth edition adds no obligation of either kind, and records none as
discharged. It discharges one limb of `DIV-050`, corrects a rejected option
that described the behaviour its own requirement exists to change, and
reconciles the two requirements of [examples.md](examples.md) that could not
both be satisfied literally. Nothing it changes touches a fixture or a
measurement, and nothing it changes changes what `tpl` does. What `FR-EX-010`
requires is already true of all four worked examples, which is where it was
read from, so nothing is owed for it inside this corpus or outside it. The
obligation the thirty-fourth edition recorded outside this corpus — the server
carrying the three schemas of `FR-EX-006` — is not read here and stays as that
edition left it. The item below was untouched by this edition and stays
outstanding.

The thirty-sixth edition adds no obligation inside this corpus and **one
outside it**, and records none as discharged. It withdraws the performance
gates on the product owner's decision, reclassifies the requirements of form as
the correctness invariants they always were, settles what the dispersion rule
of `NFR-PERF-011` is a statement about, and fixes the invocation, the
aggregation and the cache posture that three points of `NFR-PERF-014` had left
to whoever measured them. The obligation outside this corpus is `DIV-055`:
`CLAUDE.md` states, in two places, that a regression against a recorded
baseline fails the change, which `BR-PERF-008` now contradicts. Like every
entry of [upstream-divergences.md](upstream-divergences.md) it is a work list
for whoever holds the pen on that file, it blocks nothing here, and no
requirement of this corpus waits on it. **Two further statements outside this
corpus are made false by this edition and are not this register's to hold**:
the budget tables of `docs/spec-technical/quality-attributes.md`, which restate
the withdrawn vocabulary, and the four architecture decision records that argue
from `NFR-PERF-017` — each belongs to the agent that owns its folder, and the
register above covers the two root documents and nothing else. Nothing this
edition changes touches a fixture or a measurement, and nothing it changes
changes what `tpl` does. The item below was untouched by it and stays
outstanding.

**All three obligations this edition recorded outside the corpus are
discharged, and one of its counts was wrong.** `DIV-055` is discharged by
`9562fb2`, which removed both passages from `CLAUDE.md`; the register records it
and the commit. `66e0bbf` corrected `docs/spec-technical/`, including the budget
tables of `quality-attributes.md`. And `2c3d60f` reconciled the decision
records — **eight of them, not the four this paragraph counts**:
`adr-001-template-engine-pin`, `adr-003-database-driver`,
`adr-004-release-profile-and-panic-path`, `adr-005-async-runtime-scope`,
`adr-007-msrv`, `adr-008-packaging-and-build-path`,
`adr-009-foreign-key-embedding-representation` and
`adr-010-driver-tls-connect-stall`. The count of four was this corpus asserting
something about a folder it does not read, which is the defect the fifth
validation rule guards against in the other direction; the number is corrected
here and the sentence is otherwise left as the edition wrote it. The
identification of the eight is `2c3d60f`'s and not this corpus's: what is
recorded here is that the obligation was named, that it was met, and by which
commit.

The thirty-seventh edition adds no obligation of either kind, and **discharges
the three the thirty-sixth recorded outside this corpus** — the entry
`DIV-055`, the budget tables of `docs/spec-technical/quality-attributes.md` and
the decision records that argued from `NFR-PERF-017` — each named in the
paragraph above with the commit that paid it. It removes five adopted figures
that a measurement superseded, states what a superseding measurement has to
satisfy, records four observations of the first campaign where the corpus says
something each bears on, narrows one claim about the integration suite to what
is verified, and discharges two entries of
[upstream-divergences.md](upstream-divergences.md) with one more limb of a
third. Nothing it changes touches a fixture, and what it changes about a
measurement is where a figure lives rather than how one is taken. One
observation is named and not acted on, and it is about a file this corpus does
not own: `9562fb2` added to `CLAUDE.md` the claim that the requirements of form
are asserted **em cada `cargo test`**, which is the wider form this edition
narrowed here, and whether an entry is owed for it is a judgement for a reading
of that section. It is debt only in the sense `DIV-036` is, and it blocks
nothing. The item below was untouched by this edition and stays outstanding.

The section therefore carries the item the twentieth edition recorded, and the
eight it has held before are all accounted for below.

One item is outstanding.

- **Escaping stops at C0, and a C1 control can still reach a terminal.**
  Recorded by the twentieth edition. `FR-ERR-024` escapes `\n`, `\r`, `\t`
  and the C0 range in every value interpolated into a diagnostic message, and
  `FR-OUT-018` escapes the C0 range in the `text` and `json` output of the read
  commands. Neither reaches `U+009B`, the single-character CSI, nor any other
  C1 control, so a catalogue name carrying one is printed as it stands and a
  terminal that honours 8-bit controls reads a control sequence out of it. The
  twentieth edition settled that `NFR-DET-004` does not decide this — its
  subject is the decoration the system composes, not a byte carried from the
  catalogue — and deliberately did not decide it from that side, because the
  two requirements that own escaping have their own ground and their own
  exception lists and are edited together. What is owed is a decision over both
  of them: whether a composed message and a composed listing should escape the
  C1 range as they escape the C0 range, and what that means for the tab
  exception in `text` and for the byte-for-byte outputs of `FR-OUT-019`, which
  are the product and stay untouched either way. It is a decision and not an
  editorial tidy, which is why it is a task of its own, on the ground the
  thirteenth edition used for the two readings with no home. Nothing in this
  corpus is ambiguous while it stands: both requirements say exactly what they
  escape, and a reader of either is told where the gap is.

Eight items previously recorded here have been discharged. The eighth is the
sweep below, recorded and discharged inside this sprint.

- **The corpus has not been swept for terms defined outside the glossary.**
  Recorded by the thirty-second edition, which wrote the rule that every
  defined term lives in [glossary.md](glossary.md), moved the two instances its
  four pieces reached — *in flight*, from
  [errors-and-exit-codes.md](errors-and-exit-codes.md), and *differential run*,
  from [performance-requirements.md](performance-requirements.md) — and
  recorded that whether a third existed was unknown, on the ground that a rule
  is worth exactly the sweep behind it. **Discharged in the thirty-third
  edition**, which read all twenty-six files for a passage that defines a term
  rather than using one. It moved five definitions and removed one duplicate:
  *closed*, *dissolved* and *closed on a stated limit*, from
  [open-questions.md](open-questions.md), and *calling agent* and *operator*,
  from the *Actors* section of [cli-contract.md](cli-contract.md), whose third
  line defined *project* a second time and **not the same way** as the entry
  that already held it. One citation was repaired with them, in the glossary's
  own `volatile field` entry, which restated `BR-CAT-002` without sending a
  reader to it. The sweep also produced what the rule had been missing: a
  **boundary**, stated in
  [glossary.md](glossary.md#what-counts-as-a-definition) with the alternative
  rejected beside it, so that the next reading of this rule has a test and not
  a judgement.

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
- **The note on difference 8 of `FR-SRV-038`.** Recorded by the twelfth
  edition, which found while recomputing a tally that the note called
  difference 8 the only split in the record not falling after `10.11` when
  difference 2 has the same shape, and left the claim for a correction of its
  own. Discharged in the same edition —
  [server-contract.md](server-contract.md), *Differences observed between the
  series*. The note counts the differences three ways — those isolating
  `10.11`, those isolating `12.3`, and those splitting the window in the
  middle — and names the differences in each count, so that the arithmetic can
  be checked against the rows instead of recomputed from scratch; the
  thirteenth edition recomputed all three over fourteen rows when it added
  difference 14. Nothing about `tpl` turned on it: no requirement rested on the
  claim.
- **The two readings with no home under `FR-SRV-038`.** Recorded by the
  thirteenth edition, whose sweep found that `FR-SRV-040` reports a source
  revision differing between all four servers and an SSL library string
  differing between `10.11` and the other three, classifies both as properties
  of the build, and that the record of `FR-SRV-038` held neither a row for them
  nor a line below its table. It was left for a task of its own because the
  correction was a decision and not an editorial tidy: the classification
  beside theirs rested on an observation and this one did not. Discharged by
  the eighteenth edition —
  [server-contract.md](server-contract.md), *Differences observed between the
  series*. Three readings are recorded below the table rather than two, the
  requirement states the ground that admits them, and the one classification
  that is not settled says so, states its bound, and names the observation that
  would settle it. It was settled on the fourth validation rule below and not
  referred back: the fixture names a series and not a patch release, so the
  build is a condition of observation this project's own fixture moves. Nothing
  about `tpl` turned on it either: no requirement reads any of the three.

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

The thirteenth edition adds a third rule, learned from a record that was
recounted twice and was never wrong about its own rows. **A record of
observations must be assembled from every place an observation was made, not
from the entries it already holds.** Difference 14 of `FR-SRV-038` was observed
before any of the four comparison passes, was written down in the requirement
whose decision it drove, and was invisible to both recounts because each was
computed over the rows of the table. Counting a table correctly establishes
nothing about whether the table is complete. The check that finds a defect of
this shape is a sweep of the corpus for observations recorded outside the
record that owns them — every *Observed* note, every passage naming a series,
and every statement that a server accepted or refused something — and it is
what this edition ran.

The fourteenth edition adds a fourth rule, learned from a row that was accurate
on the day it was written. **An observation of what a server reports must
record the conditions it was taken under wherever this project's own fixture
can change them, and the record must be re-read whenever the fixture does
change.** Difference 3 of `FR-SRV-038` was true of four servers nobody had
configured and false of the four the fixture now runs, and nothing in the row
said which it described. No reference check and no recount could find a defect
of this shape: the row was internally coherent, cited correctly, and counted
right. What finds it is reading the record against what has since been done to
the thing observed, which is how this one was found — from the technical
specification, read against this corpus rather than from within it.

The fifteenth edition adds a fifth rule, learned from a register that asked for
work done several sprints earlier. **A register of corrections owed to a file
this specification does not own must be re-read against that file whenever the
file changes — both for the entries the file has discharged and for the
divergences the register does not yet hold — and an entry found discharged must
record the commit that discharged it. The obligation is over every statement
this corpus makes about such a file and not over a register alone**: the
fifteenth edition found two
outside the register, in the provenance notes of `NFR-PERF-014` and
`NFR-PERF-018`, each describing a passage of `CLAUDE.md` that `0ea5624` had
removed. `DIV-031` asked for the removal of a clause `0ea5624` had
already removed, and the eleventh edition amended that entry — recounting a
list inside it — without checking whether the sentence the entry corrects was
still in the file. The four rules above are all checks this corpus can run on
itself: a reference resolves or it does not, a requirement matches an
observation or it does not, a record is complete against the corpus or it is
not, a row names its conditions or it does not. This one cannot be run on this
corpus at all, because what decays is outside it and decays when somebody else
edits it. The trigger is therefore the edit and not the edition, and the
evidence is the commit: an entry that says *when* it stopped being owed can be
audited, while an entry that merely stops being listed leaves the next reader
to re-derive it. A register whose entries are true only of a state nobody has
checked since is worse than no register — it sends a reader to correct what is
already correct, and it spends the standing of the entries that are still
owed.

*Extended in the sixteenth edition.* The rule was written over one half of the
sweep. It obliged the register to be re-read for entries the target file had
discharged, and said nothing about the divergence that had never been recorded
at all, so a register could satisfy it in full and still be incomplete — which
is what the fifteenth edition left behind. Four divergences of the root
`README.md` had no entry, and the reading that found them was made for another
purpose and could as easily not have been made. The clause now names both
directions. This is the third rule's lesson in a second setting: counting a
record's own rows correctly, or classifying a register's own entries
correctly, establishes nothing about whether either is complete against the
thing it describes.

*Extended again in the thirty-second edition: the trigger is an edit on either
side.* The rule names an edit to the target file, and says of itself that it is
the one rule this corpus cannot run on itself, "because what decays is outside
it and decays when somebody else edits it". That is half of it. **An entry
decays equally when this corpus amends a requirement the target document
paraphrases**, and it then becomes owed without anybody touching the file it is
owed to. `DIV-053` is the instance. The root `README.md` says that four
commands perform no discovery **at all**, which was an exact summary of
`FR-PROJ-025` from `e75996c` until the thirty-first edition amended that
requirement four days later to admit the upward look `FR-PROJ-016` obliges.
Nothing about the sentence changed and it stopped being true. So half of this
rule **can** be run on this corpus, and it is a check an edition under review
can be put to: for every requirement the edition amends, ask whether either
root document paraphrases it, and read the paraphrase. It is the same lesson
the sixteenth edition's extension carried — a register satisfied in full in one
direction is still incomplete — arriving from a third direction.

The thirty-first edition adds a sixth rule, learned from a record of an absence
that no check here could have caught. **A record that something is absent must
rest on a form of observation that would fail on a wrong question.** A negative
observation — that a variable does not exist, that a field returns no row, that
a population holds no member of a kind — is admissible only where the form used
to take it separates *the thing is not there* from *the question was wrong*,
either because a wrong question makes that form report an error, or because the
population was enumerated and the thing shown not to be in it. A form that
answers a wrong question with the same silence it answers a true absence with
proves nothing, and the record it produces is wrong on the day it is written.

`FR-SRV-040` recorded `version_malloc_library` as returning no row where it
returns one on every server of the fixture, and the twenty-third edition
corrected it. None of the five rules above could have found it: the record
cited an observation, was internally coherent, was correctly cross-referenced,
was complete against the corpus, and named the conditions it was taken under.
What was wrong was the **reading**, and the form could not fail — a name that
matches nothing under a `SHOW … LIKE` prints no row and no header at all and
exits `0`, so an absent row is what a wrong name and a missing variable both
produce. The enumerating form of the same question, a read of the global
variables table with no `LIKE`, would have shown the variable present; and a
form that errors on an unknown name would have failed outright.

**It applies retrospectively**, on the trigger the second rule already uses: a
record of an absence is re-read against this rule the next time it is read for
any purpose, and a record that does not say what form it was taken by is not
yet a record. Two instances are named so that the rule is readable against
something. `NFR-PERF-005` satisfies it — its observation of an absence names
three commands and what each did, one absent, one refused by `csrutil` and one
demanding a password, so a wrong question there produces three distinct
refusals and not one silence. `FR-CAT-057` states an absence of a different
kind, a feature the fixture does not declare, and satisfies the rule by naming
the DDL that would create the case rather than by any reading at all.

**This rule is of the first and third rules' family and not of the fourth and
fifth's.** The nineteenth and twentieth editions each declined a sixth rule on
the ground that every rule here is about a statement decaying, and this one is
not about decay. That ground never covered the whole set: the first rule is
about a check that passes on a wrong answer — an identifier that exists is not
an identifier that is correct — and the third about a count that establishes
nothing about completeness. This is the first rule's shape applied to an
observation rather than to a reference: the form answered, and an answer is not
a result. What those two editions declined is a different question, whether a
sentence of this corpus is stated wider than the ground it gives for itself,
and it is still named, still recurring, and still left for a reading that
covers the corpus.

*Rejected: declining the rule and recording the lesson beside the
twenty-third edition's finding, as the nineteenth and twentieth editions
recorded theirs.* A lesson recorded beside one finding is read by whoever reads
that finding, and the defect it guards against is made by whoever takes the
next observation — a different reader, at a different time, with no reason to
be looking there. The five rules exist because each of them is a check that can
be applied to an edition under review, and this one can: for each record of an
absence the edition adds, name the form and ask what that form would print for
a question that is simply wrong.

**An edition record above that counts five rules is counting the set it had.**
Each such statement — *none of the five rules below would have found it* — is
an account of what an edition found against the rules that existed when it ran,
and it stays true of that edition. None is amended to say six, because amending
it would assert that an edition applied a rule nobody had yet written.
