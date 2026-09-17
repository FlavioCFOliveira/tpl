---
title: tpl Functional Specification
status: approved
last-reviewed: 2026-09-17
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
either of them changes — for entries the files have discharged and for
divergences it does not yet hold — and every entry records whether the
correction is still owed and, where it is not, the commit that discharged it.

## Scope

The specification has been written in twenty-two editions. All are in force;
each adds to the ones before it and amends them in place, and every
amendment carries an *Amended in the nth edition* note beside the
requirement it changes.

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
target, and a regression failing the change are obligations in force today, and
they governed both campaigns that file already holds; `NFR-PERF-009` through
`NFR-PERF-013` and `NFR-PERF-017` carry the same rules in this corpus, and
`DIV-035` left the discipline with `CLAUDE.md` when it took the figures out.
What is wrong is the assertion of state in front of it, so `DIV-052` asks for a
qualifier on one clause and says in as many words that neither the discipline
nor the location is to be removed. An entry read as *drop the benchmark rule*
would take the project's only statement of what fails a change out of the file
every agent reads first.

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
still the last commit to touch either root document, and all exist but those
already recorded under `DIV-050`, `DIV-051` and `DIV-052`. Two
further passages were read on their own and are not of the kind: the workflow
step and the language rule that name a `CHANGELOG` the repository has not got,
which are directions rather than claims and which the technical specification
already records as prescribed and not yet observable; and the routing sentence
of *Desempenho e Eficiência*, which names `BENCHMARKS.md` and describes it as
that file describes itself. **So the seam is shut**: four passages registered
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
  columns of each row could not be filled from anything this corpus holds. The
  outcome that would have produced new evidence was therefore not the only one
  that needed it: a row needs a new observation too, and a different one.
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

**One item is outstanding**, recorded by the twentieth edition and listed
below. The seven items this section held before it are all discharged; the last
of those was recorded by the thirteenth edition and discharged by the
eighteenth. The item this section carried through the sixth edition — twenty
catalogue field lists that no observation had recorded — was discharged by the
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
`FR-SRV-013`, which like every test this corpus mandates is blocked only by
`tpl` not existing.

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

The section therefore carries the one item the twentieth edition recorded, and
the seven it has held before are all accounted for below.

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

Seven items previously recorded here have been discharged.

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
