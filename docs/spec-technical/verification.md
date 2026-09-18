---
title: Verification
status: draft
last-reviewed: 2026-09-18
related: [README.md, traceability.md, open-decisions.md, overview.md, architecture.md, interfaces.md, data-model.md, security.md, operations.md, quality-attributes.md]
---

# Verification

## What this document is

What must be tested, what each test asserts, and what each test needs in order
to run: the tests the corpus mandates as contract, the tables that are
specification and test vector at once, the properties that may only be observed
from outside the process, the two seams that must not reach the published
surface, and the twelve flows that are the acceptance skeleton.

**No target and no budget appears here.** Every budget, its standing and the
protocol that ratifies a figure are
[quality-attributes.md](quality-attributes.md#the-nine-budgets); every measured
figure is `BENCHMARKS.md`'s. **No build command appears here.** The pipeline
that runs the suite, the gates a release passes and the fixture's operational
standing are [operations.md](operations.md#the-mandatory-validation-pipeline).

**The harness exists.**
[The container harness](#the-container-harness) records what
`scripts/mariadb/` supplies as of 2026-09-11, which discharged the harness half
of [`OD-22`](open-decisions.md#od-22--the-test-harness-and-the-fixture-certificate)'s
residual, what the suite added to it at commit `a6d1fed`, and what remains
unestablished beside both.

**Six rows of the register below are written or partly written; every other row
is obligation alone.** Row 2 is a unit test over the seam `FR-ERR-031` requires,
and it asserts what that row asserts: the guard produces the condition of
`FR-ERR-030` carrying the message `FR-ERR-032` requires. Rows 3 to 6 were
written with the command surface, at commit `f8f335d` of 2026-09-15, and
[Help](#help-snapshots-at-every-depth) names each. Row 1 is reached for two of
its nine codes — `0`, by every successful form of help and version, and `64`, by
every refusal of step 1 — because the commands that produce the other seven are
later sprints. Row 15 is reached for two of its six requirements, at commit
`a6d1fed`, and [the register](#mandated-by-the-corpus) says which, on what
evidence, and what is written but runs on neither Darwin target.

**The suite reaches the containers.** `tests/outside_the_process.rs` carries six
tests: two drive the four series of `FR-SRV-015`, two are differential runs that
need no server, one is the syscall trace the two Linux targets admit, and one
exercises the gate's own mapping with no fixture in reach. What is recorded of
every other test is its obligation, its trace, and, where the test exists, where
it lives — never an outcome.

## The suite as it stands

At commit `fd51ca2`, and recorded with its point in history because it is a
count that moves: **621 tests**, made of 564 unit tests inside the library, 55
integration tests over the binary — 25 in `tests/project_and_configuration.rs`,
13 in `tests/help_surface.rs`, 7 in `tests/invocation_surface.rs`, 6 in
`tests/outside_the_process.rs`, 4 in `tests/help_environment.rs` — and 2
doc-tests. It was 504 in the working tree of 2026-09-17 and 272 at commit
`f8f335d` of 2026-09-15; the model work of `0cdc539` and the harness work of
`a6d1fed` are the whole of the difference, at 111 tests and 6.

`tests/support/` holds two modules and no test target: `fixture.rs`, which is
the gate and the three server-side instruments, and `differential.rs`, which is
the fourth. Both are reached by `#[path]` from the files that use them.

**Two tests need a server**, both in `tests/outside_the_process.rs`, and both
are gated on the fixture: they run where it is up and are skipped, with a
printed reason, where it is not. No other test reaches one, because no other
command does — the connectivity subcommand is the one `cfg` leaf still
unwritten.

## The four kinds of test, and what each needs

The kind is not a taxonomy for its own sake: it decides what the harness must
supply, and it is the column that says whether a mandated test is blocked on the
fixture, on the binary, or on neither.

| Kind | Where it runs | Needs a process launch | Needs a server |
|---|---|---|---|
| **Vector** | A pure function, against a table published in `/specification` | no | no |
| **Unit** | Inside the library crate | no | no |
| **Integration** | The distributed binary is launched | yes | no |
| **Server** | The binary is launched against a fixture container | yes | yes |

One package carries the library and the binary
([`ADR-006`](../adr/adr-006-package-layout.md)), so a unit test addresses the
logic without launching a process and an integration test links the library as a
separate crate. That distinction is load-bearing for
[the two seams](#the-two-test-seams) and is not restated there.

## Test naming binds a test to a requirement

`specification/README.md` (*Identifier scheme*) makes the requirement identifier
the citation unit in commit messages, task descriptions **and test names**, and
states that an identifier is stable once assigned and is never renumbered. A
name built on one therefore survives every edition of the corpus.

| Rule | Consequence |
|---|---|
| A test name begins with the identifier that **mandates** it, lower-cased, hyphens replaced by underscores, followed by a phrase naming the case | `fr_env_033_pascal_of_http_server`, `br_help_001_cfg_database_add_at_depth_three` |
| Where one requirement mandates several tests, the phrase distinguishes them and the identifier does not change | The three of `BR-HELP-003`, the two of `FR-SRV-029` |
| Where one test verifies several requirements, the name carries the one that **mandates the test**; the others are cited in the test's own doc comment | `BR-SCH-004` names the test; `FR-SCH-016`, `FR-SCH-021`, `FR-SCH-022` are cited in it |
| A test that verifies a **row** of a published vector names the row, not a new identifier | `BR-ENV-007` makes a cell the contract, so the cell is the case |
| No test that verifies a **single** requirement is named without one | *Which test covers `FR-X`?* is then answerable by search alone, which is what makes the register below auditable against the suite |

The reverse direction — which requirement a document answers — is
[traceability.md](traceability.md)'s and is not duplicated in test names.

**Recorded change — the suite now carries the identifier in the name.** This
section recorded a divergence on 2026-09-17: the suite carried 502 test
functions, none of which began with an identifier, so the rule was met in
substance — every test bound to its requirements in its own doc comment — and
unmet in form. It was closed where it was owed, in the code: at commit
`bfa043d`, **604 of the 619 test functions** were renamed so that the mandating
identifier leads, and *which test covers `FR-X`?* is now answerable by search
alone. **Every test name this document prints is read from the tree at commit
`fd51ca2`**, except the two illustrations of the form in the table above, which
name no test.

**Fifteen tests keep a descriptive name, and the rule's last row is narrowed to
say so.** The row read *no test is named without an identifier*, which those
fifteen do not satisfy. It was this folder's own strengthening rather than the
corpus's: `specification/README.md` makes the identifier **the reference used
in** a test name and does not require every test to reference a requirement, so
a test that verifies none has no identifier to be named after. The row now
states the rule the suite holds to, and the criterion that decides which tests
fall outside it is stated here rather than left to the next reader: **a test
verifies no single requirement when what it asserts is a property of the crate's
internals or of the test corpus rather than of `tpl`'s observable behaviour**.
The fifteen fall into four families, each searchable by the absence of a
prefix.

| Family | Count | Why no identifier leads |
|---|---|---|
| Allocation and borrowing | 8 | The assertion is that a value is borrowed rather than copied. `CLAUDE.md` requires minimal allocation on a hot path; no requirement of `/specification` states it, and a caller cannot observe it |
| The test corpus, or a private type | 4 | One walks the sample set the exit-code test is driven from, two exercise a type no document publishes, and `display_is_one_line_and_carries_no_labelled_line` in `src/error.rs` cites [`OD-06`](open-decisions.md#od-06--the-error-types-shape-and-the-exit-code-derivation) — an open decision, and not a requirement identifier |
| `text` output | 2 | `FR-OUT-004` makes `text` output explicitly not a contract, so line termination and trailing whitespace are properties of a layout no requirement fixes |
| The suite's own gate | 1 | It asserts how `scripts/mariadb/status.sh`'s exit code is mapped, which is a property of the harness and not of `tpl` |

A better criterion replaces this one. What may not happen is a test named for a
requirement it does not verify: that makes the search above answer wrongly
rather than not at all.

## The register of mandated tests

A **mandated** test is one the corpus requires, as against one an implementer
chooses to write. It is part of the definition of done for the feature that can
produce the behaviour, on the same footing as the behaviour itself.

### Mandated by the corpus

| # | Test | What it asserts | Kind | Mandated by |
|---|---|---|---|---|
| 1 | Exit code per condition | One integration test per exit code of `FR-ERR-001`; nine codes, `70` excepted | Integration; the conditions that reach a server need one | `BR-ERR-001` |
| 2 | The `70` trigger | The guard on a violated internal invariant produces the condition of `FR-ERR-030` carrying the message `FR-ERR-032` requires | Unit | `FR-ERR-031`, `BR-ERR-001` |
| 3 | Help equivalence | Byte-identical output across the three forms of each node, at every depth | Integration, snapshot | `BR-HELP-001`, `FR-HELP-002` |
| 4 | Tree completeness | Every command, alias and flag appears in the JSON command tree | Unit | `BR-HELP-003` (1) |
| 5 | Example presence | Every command carries at least one example | Unit | `BR-HELP-003` (2) |
| 6 | Example validity | Every example parses through the command parser itself | Unit | `BR-HELP-003` (3) |
| 7 | The dump round-trip | A render over a dump fed back through `--context` is byte-identical to the same render against a live read | Server | `BR-SCH-004` |
| 8 | The sentinel | A known sentinel password appears in no byte of either stream, from any command of the tree, at maximum verbosity | Integration; the commands that reach a server need one | `BR-SEC-003` |
| 9 | The closed statement list | The server receives the four kinds of statement of `FR-SRV-006` and no fifth, with the three connection-start statements issued once each in that order | Server, observed on the server | `FR-SRV-012` |
| 10 | The read-only read-back | Both outcomes of the read of `@@session.tx_read_only`: the session setting taking effect, and failing to take effect | Server, **every series** — exactly one of the four discriminates the spelling | `FR-SRV-013`, `FR-SRV-009` |
| 11 | The connection count | At most one connection per invocation, counted on the server | Server, observed on the server | `FR-SRV-014`, `NFR-PERF-004` |
| 12 | Cross-series equivalence | Byte-identical documents from identical DDL, on every series of `FR-SRV-015`, with the three exceptions | Server, four containers | `FR-SRV-029` (1), `FR-SRV-026` |
| 13 | The refusal | A series outside the window is refused with `78` and no catalogue is read | Server, one container outside the window | `FR-SRV-029` (2), `FR-SRV-020` |
| 14 | The newer-than-window read | The read completes without error and `standing` is `newer_than_supported` | Unit | `FR-SRV-035` |
| 15 | The six requirements of form | Each observed from outside the process, never by reading the source, by the four instruments of `NFR-PERF-007` and each only on the targets its row admits | Server or integration, per row | `NFR-PERF-007`, `NFR-PERF-018` |
| 16 | The word list | The eight rows of the published table | Vector | `FR-ENV-032`, `FR-ENV-030`, `FR-ENV-031` |
| 17 | The five naming filters | The forty cells of the published table | Vector | `FR-ENV-033` |
| 18 | The seven tests | The answer of each test, and the disjointness of the three families | Vector | `FR-ENV-041`, `FR-ENV-042` |
| 19 | Family membership | The four rows partitioning the observed `data_type` values | Vector | `FR-ENV-046` |
| 20 | Identifier quoting | Every backtick doubled, the whole enclosed in one pair | Vector | `FR-ENV-045`, `FR-ENV-035` |
| 21 | Member-list parsing | An `ENUM` or `SET` member list read by quote state, never by splitting on the comma | Vector | `FR-CTX-039` |
| 22 | The shipped example | `.tpl/templates/example.jinja` renders without error against any table of any supported series | Server, four containers | `FR-PROJ-021` |
| 23 | The key invariant | No key the model carries names a column absent from the same table's column list | Server | `FR-CAT-044`, `FR-CAT-043` |
| 24 | Context strictness | A test resolving a column's `table_name` against a table absent from the context fails with `65`, identically for a column arriving from `--context` and one read from a server | Integration | `FR-SEM-018`, `FR-SEM-017`, `FR-ENV-017` |
| 25 | The three detections | Each shape of a privilege-driven absence is detected as the shape the catalogue gives it | Server, reduced-privilege reader | `FR-PRIV-011`, `FR-PRIV-017`, `FR-PRIV-019`, `FR-PRIV-012` |
| 26 | Type non-coercion | A naming filter applied to a non-string fails the render rather than coercing; a test applied to a non-column fails rather than answering `false` | Integration | `FR-ENV-034`, `FR-SEM-008`, `FR-SEM-009`; `FR-ENV-040`, `FR-SEM-005` |

**Recorded change — the kind of rows 4 to 6.** This document gave the three
tests of `BR-HELP-003` as integration tests; they are written as unit tests, and
the column now says so. `BR-HELP-003` fixes the three properties and no kind, so
nothing of the corpus moves. The reason the kind moved is that each of the three
compares the document against **the tree the binary parses with**: an
integration test reads the emitted bytes and would have to re-derive the tree in
order to compare, which is the second source `FR-HELP-021` exists to prevent.
Row 3 is unaffected and stays an integration test, because the equivalence it
asserts is between two routes **through the process** and cannot be observed
from inside it.

**Recorded reading — the count.** [traceability.md](traceability.md) records
*"fourteen mandated tests"*, naming `BR-HELP-003` once with *(three)* beside it
and `FR-SRV-029` once with *(two)*, and counting the second expansion but not
the first. The table above expands both, and adds the rows the corpus mandates
outside that list — `FR-SRV-014`, `NFR-PERF-007`, `FR-ENV-045`, `FR-CTX-039`,
`FR-PROJ-021`, `FR-CAT-044`, `FR-SEM-018`, the three privilege detections and
the non-coercion rules. The two are not in conflict about any test; they count
differently. Nothing is corrected in the other file.

**What row 15 reaches, and what it does not.** Two of the six requirements of
form are asserted, at commit `a6d1fed`, in `tests/outside_the_process.rs`. Every
negative assertion carries a control, on the terms
[the invocation surface](#the-invocation-surface-observed-on-the-process) states
and for the same reason.

| Requirement, or clause of one | Instrument | Where it runs |
|---|---|---|
| `NFR-PERF-006`, and the connection clause of `NFR-PERF-005` | The server's connection record, corroborated over the same window by the server's statement record | Every series of `FR-SRV-015`, once each, gated on the fixture |
| The discovery clause of `NFR-PERF-005` | A differential run, with its inversion | All four targets: it needs no server |
| The configuration clause of `NFR-PERF-005` | A differential run, with its inversion | All four targets |
| The two clauses above, **as syscalls** | A syscall trace | **Written, and run on neither Darwin target.** The body skips, with a printed notice, on a host that is not Linux or that has no `strace` |

The trace is written and does not run on either Darwin target, and that is the
requirement's arrangement rather than a gap in the suite: `NFR-PERF-005` forbids
crediting a trace taken inside a Linux container to a Darwin target, because the
artefact observed would not be the artefact distributed. Where the trace does
not exist the differential run is the whole of the evidence, which
`NFR-PERF-007` provides for in terms and `NFR-PERF-005` states in its own
text.

`NFR-PERF-001` through `NFR-PERF-004` are the four of the six that are not
reached, and each waits on the same thing: a catalogue reader, so that there is
a query to count and a connection to attribute.

### Owed, and not yet observable

Two verifications are owed by a settled decision rather than by a requirement.
Each decides a point on which a settled entry declined to assert, and each names
what it costs if the expected answer does not hold. A third was owed by
[`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) and is
discharged: the tests exist, and
[The parser's mapping](#the-parsers-mapping-one-test-per-kind) names them.

| Owed | What must be observed | What it costs if it fails | Kind | Recorded in |
|---|---|---|---|---|
| The phase attribution of a TLS handshake failure | Whether an untrusted certificate, a name mismatch, and a server offering no TLS each reach `tpl` as the driver's TLS error rather than its I/O error | `FR-ERR-034` row `69` cannot be met as written, and a defect is owed to the functional owner naming `FR-ERR-034` | Server; needs the three failure modes `FR-CONF-038` obliges the fixture to present | [`OD-12`](open-decisions.md#od-12--how-six-phase-deadlines-are-enforced) |
| A **defined** `null` under the strict undefined-behaviour variant | Whether it interpolates as the empty string rather than failing the render | `FR-SEM-010` and `FR-SEM-011` are contradicted outright | Unit, against the engine pinned by [`ADR-001`](../adr/adr-001-template-engine-pin.md) | [`OD-14`](open-decisions.md#od-14--which-undefined-behaviour-the-engine-is-configured-with) |

The second is why
[architecture.md](architecture.md#the-render-component) asserts nothing about a
defined `null`: until the observation is made, **no passage of this folder may
rely on either answer**, and this register carries the obligation so that it is
not lost between the entry that owes it and the run that discharges it.

## The published test vectors

`BR-ENV-007` makes the behaviour tables of `specification/template-environment.md`
**the test vector**: each row is a case the implementation shall satisfy, and a
change to any cell is a change to the contract of `FR-ENV-002` and is breaking.
What a breaking change obliges is
[operations.md](operations.md#what-a-breaking-change-is)'s.

| Vector | Rows | Why it can be a vector at all |
|---|---|---|
| The word list, `FR-ENV-032` | 8 | `FR-ENV-030` states the derivation as five ordered rules and `FR-ENV-031` folds case over ASCII alone, independently of server, collation and locale |
| The five naming filters, `FR-ENV-033` | 8 × 5 | Every filter is a pure function of the word list, so the whole surface is decided by one input |
| The seven tests, `FR-ENV-041` | 7 | Five answer from the operand alone; `primary_key` and `unique` resolve against the render context (`FR-ENV-015`), so those two need a context and not a server |
| Family membership, `FR-ENV-046` | 4 rows over 39 observed `data_type` values | The memberships were observed, not derived from taxonomy, and `FR-ENV-042` makes disjointness the property a template depends on |
| Identifier quoting, `FR-ENV-045` | 1 worked case | `FR-ENV-035` alone is testable only against a server; naming the mechanism is what makes it testable without one, which `FR-ENV-045` states as its reason |
| Member-list parsing, `FR-CTX-039` | 6 rules, with observed examples | The rules are stated over the raw type string, so the parse is exercisable from a literal |

**A vector needs neither the binary's process nor a container**, which makes
these the first mandated tests that can run: they need the library and nothing
beside it. `FR-ENV-046`
carries a **bounded claim** and the bound is part of the vector: the memberships
are fixed over the 39 values the fixture produced, and `VECTOR` — which `10.11`
and `11.4` reject — and any type a series newer than the window introduces are
assigned to no family without a further observation.

## The nine observations made outside the process

`NFR-PERF-007` forbids verifying a requirement of form by reading the source,
and `BR-SRV-003` says why: a promise about what a process sends that can only be
checked by reading that process's own source is not a promise a caller can rely
on. Nine requirements are held to that standard.

**Four instruments, and each is bound to the targets it exists on.**
`NFR-PERF-007` names them and `NFR-PERF-018` supplies the target set; the
binding is part of the rule, not a property of the machine a run happens on.

| Instrument | What it observes | Targets |
|---|---|---|
| The server's statement record | The statements the server receives | all four |
| The server's connection record | The connections the server accepts | all four |
| A syscall trace of the process | The files the process opens | the two Linux targets |
| A differential run | The invocation's own outcome — exit code, stdout bytes, artefacts left on disk — under an arrangement the operation, had it been performed, would not have survived | all four |

Where both the trace and the differential run exist, the trace establishes a
clause stated as a syscall and the differential run corroborates it; where only
the differential run exists, it is the whole of the evidence. The order is
`NFR-PERF-007`'s and is not this document's to vary.

| # | Requirement | The property | What is observed |
|---|---|---|---|
| 1 | `NFR-PERF-001` | No query per object on a full read | The catalogue queries the server receives, over two workloads of different size |
| 2 | `NFR-PERF-002` | Reading one named object does not scale with the database | The same, for a named-object read |
| 3 | `NFR-PERF-003` | A cache hit opens no connection and issues no query | The connections the server accepts, and the queries it receives: none of either |
| 4 | `NFR-PERF-004` | At most one connection per invocation | The connections the server accepts |
| 5 | `NFR-PERF-005` | The commands of `FR-PROJ-025` touch nothing | Per clause, per target — see below |
| 6 | `NFR-PERF-006` | A command needing no catalogue opens no connection | The connections the server accepts |
| 7 | `FR-SRV-012` | The closed statement list | The statements the server receives: four kinds, no fifth, three of them once each in order |
| 8 | `FR-SRV-013` | The read-back of `@@session.tx_read_only`, in both outcomes | The statements the server receives, and what the session reports |
| 9 | `FR-SRV-014` | The connection count | The connections the server accepts |

**Row 5 is the only row whose evidence differs by target**, and `NFR-PERF-005`
states the split in its own text rather than leaving it to the harness.

| Clause of `NFR-PERF-005` | Evidence | On which targets |
|---|---|---|
| Opens no connection | The server's connection record | all four |
| Performs no project discovery | A differential run: `tpl init` inside a subdirectory of an existing project, which `FR-PROJ-012` and `FR-PROJ-013` make create a project there and report no ancestor | all four |
| Reads no configuration file | A differential run: a command of `FR-PROJ-025` inside a project whose `.tpl/.cfg` would fail `FR-CONF-034`, asserting exit `0` and stdout byte-identical to the same command run outside any project | all four |
| The two clauses above, as syscalls | A syscall trace: no `stat` of an ancestor, no open of `.tpl/.cfg` | **the two Linux targets only** |

A Linux observation **may not be credited to either Darwin target**, which
`NFR-PERF-005` forbids in terms and for the reason `FR-ERR-031` already gave:
the artefact observed would not be the artefact distributed. The fixture's
`observe.sh` traces inside a Linux container when the host has no `strace`, and
that trace is evidence for the Linux targets alone. The wrapper in
`tests/support/fixture.rs` therefore names the backend rather than leaving it at
`auto`, so that the fallback cannot happen silently and produce a trace no
caller may use. What the two Darwin targets consequently do not catch is named
by the requirement: a build that opened `.tpl/.cfg`, read it and discarded what
it read.

**Recorded contradiction — `tpl init`, the ancestor, and the discovery clause.**
Three requirements in force cannot all hold, and the built command satisfies the
first of them.

| Requirement | What it states about `tpl init` inside an existing project |
|---|---|
| `FR-PROJ-016` | It **requires** a warning on stderr that the project just created shadows the one above; the built command names both paths in it |
| `NFR-PERF-005` | Every command of `FR-PROJ-025`, `tpl init` among them, performs **no project discovery**. Finding the ancestor is discovery |
| `NFR-PERF-007` | Its worked arrangement for the discovery clause asserts that the same invocation creates the project there **and does not report the ancestor** |

Nothing is corrected in `/specification`, and no reading of it is chosen here:
the wording is the functional owner's, and the defect is registered as task #120
for `specification-manager`. What belongs to this document is what the
contradiction costs the instrument. The differential run reads the exit code,
the stdout bytes and the artefacts left on disk, and stderr is none of the
three, so it neither sees the warning nor is falsified by it: it passes
honestly, and what it establishes is narrower than the clause's words — that the
ancestor did not decide where the project went, not that no ancestor was looked
at. The test states that limit in its own doc comment, which is where a reader
of a run meets it. The syscall trace is taken over `tpl help`, so no instrument
in the suite is presently pointed at the invocation the three requirements
disagree about.

Rows 1 and 2 are the only two that need a **second, larger** database to be
conclusive, which is `WL-001` and therefore `seed-bench.sql`
([quality-attributes.md](quality-attributes.md#the-three-reference-workloads),
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001)). The query count is
additionally observable on the diagnostic stream, through the fixed leading token
of the per-query line (`NFR-PERF-008`, `FR-GLOB-017`); that a test may depend on
the token and on nothing after it is
[operations.md](operations.md#observability)'s.

**Recorded reading — nine requirements, not nine distinct observations.** Rows 4
and 9 name the same property: `FR-SRV-014` fixes the connection count *as
`NFR-PERF-004` fixes it* and adds that it shall be verifiable from the server
side. One observation of the connections a server accepts discharges both. The
nine is a count of requirements, and the instruments are four, so neither number
counts the observations a run makes.

**Recorded reading — which requirements `NFR-PERF-007` reaches.** It reads
*"each requirement of this section"*, and the section it sits in also contains
`NFR-PERF-007` itself, `NFR-PERF-008` and `BR-PERF-001`, none of which is a
property of the running system that an outside observation could reach. The six
above are the ones it can reach, which is the reading
[quality-attributes.md](quality-attributes.md#the-six-requirements-of-form)
already takes. The wording is the functional owner's to settle; nothing in the
built system differs between the readings.

## The invocation surface, observed on the process

Four properties of the argument vector are properties of the **process** rather
than of a function, so none can be established from inside the crate: a unit
test over the renderer shows that the renderer escapes, not that the vector
reaches it, that nothing else writes to the stream, or that the parser consulted
no `PATH` on the way. They are asserted in `tests/invocation_surface.rs`, at
commit `f8f335d` of 2026-09-15 and under the names `bfa043d` gave them — five
tests for the four properties, and two regressions beside them. **Every negative
test carries a control**: the decoy executable is run directly, and the
environment is shown to reach a child process, because a test that asserts an
absence is worth nothing until the same harness is shown to observe the
presence.

| Property | Requirement | Test |
|---|---|---|
| A rejected token reaches stderr escaped, whatever it carries, over the three untrusted populations | `FR-ERR-024` | `fr_err_024_a_command_token_a_flag_token_and_a_help_path_segment_reach_stderr_escaped` |
| A token cannot forge a labelled line of its own | `FR-ERR-008`, `FR-ERR-024` | `fr_err_024_a_token_cannot_forge_a_labelled_line_of_its_own` |
| One rejected token reaches the message, and the argument vector never does, at any verbosity | `FR-GLOB-018`, with [`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) | `fr_glob_018_one_rejected_token_reaches_the_message_and_never_the_argument_vector` |
| No `tpl`-prefixed executable is looked for on `PATH` | `FR-CLI-006` | `fr_cli_006_no_tpl_prefixed_executable_is_searched_for_on_path` |
| No environment variable decides a form of help, of version, or a refusal | `FR-CLI-021`, `NFR-DET-004` | `fr_cli_021_no_environment_variable_decides_a_form_of_help_of_version_or_a_refusal` |
| The argument terminator is never read as the value of the flag before it, so the `cause` names the node the invocation reached | `FR-CLI-017`, and `FR-ERR-009` through it | `fr_cli_017_the_argument_terminator_is_never_read_as_the_value_of_the_flag_before_it` |
| A token written on both sides of the terminator is refused where it stands **first** | `FR-CLI-017`, `FR-CLI-019` | `fr_cli_019_a_token_written_on_both_sides_of_the_terminator_is_refused_where_it_stands_first` |

The last two are regression tests over defects a security audit of the command
surface found, and each was proved to fail against the defect it guards before
being kept. They are recorded here rather than left in the file because a
regression test whose defect is not named is a test the next reader cannot
judge.

## The two test seams

Two behaviours can be exercised only from a seam, and both seams are barred from
the published surface. The resolution is the corpus's and is settled in
[`OD-21`](open-decisions.md#od-21--two-test-seams-that-must-not-be-on-the-published-surface);
the rationale and the three rejected mechanisms are not restated.

| Seam | What it makes reachable | Mechanism that keeps it off the surface |
|---|---|---|
| The deliberate `70` trigger (`FR-ERR-031`) | The guard on a violated internal invariant, which no correct invocation can reach | `#[cfg(test)]`. The construct is compiled only under the test configuration, so it is absent from the artefact a build produces, and an integration test is a separate crate linking the library compiled **without** `cfg(test)`, so it cannot see the construct either |
| The narrowed supported window (`FR-SRV-035`) | A reader presented with a series above its own window, for which no server can exist by construction | The same construct, for the same two reasons |

Both seams are therefore **unit tests inside the library crate**, and neither is
a command, a flag, an environment variable or a cargo feature — each of those
collides with a requirement in force, which `FR-ERR-031` records candidate by
candidate.

**What each test does not establish, in the words of its own requirement.**

- `BR-ERR-001` yields the integration test for `70` **alone** and says so in its
  own text. What is observed is the guard, and separately the step from an error
  condition to an exit status, which each of the nine other codes does exercise;
  the composition of the two is reasoned rather than executed.
- `FR-SRV-035` yields the assertion on the exit code. The test asserts that the
  read completes without error and that `standing` is `newer_than_supported`;
  the step from a completed read to exit `0` is `BR-CLI-004`, observed by the
  integration test of every successful command.
- `BR-SRV-003` does **not** reach `FR-SRV-035`, and says so: it governs promises
  about what the process *sends*, and this is a promise about what the reader
  *emits*.

These are the limits the requirements state. This document cites them and
invents none of its own.

## Exit codes: nine integration tests and one exception

`FR-ERR-001` fixes ten codes. `BR-ERR-001` requires at least one integration
test per code, part of the definition of done for the feature that can produce
it; `70` is the single exception, exercised in process through the seam above.

| Code | Reached by | Needs a server |
|---|---|---|
| `0` | Any successful command | Only where the command reads a catalogue |
| `64` | An unknown command or flag, a mutually exclusive pair, a malformed value | no |
| `65` | A template syntax error, a malformed `--context`, a path escaping the root | no |
| `66` | A named object that does not exist | Only for a catalogue object |
| `69` | An unreachable server | no |
| `70` | **Excepted.** The seam above | no |
| `73` | `tpl init` against an existing `.tpl` | no |
| `74` | An I/O failure, including a pipe closed mid-document | no |
| `77` | Authentication refused, or a privilege shortfall on a named object | yes for both |
| `78` | A configuration that does not describe a usable connection; an unsupported series; a read-only session that could not be enforced | Only for the last two |

The message every one of them carries is four labelled lines (`FR-ERR-008`),
identical in shape whatever `--format` was requested (`FR-ERR-033`), with the
per-code obligation on the `cause` line fixed by `FR-ERR-034`. The renderer and
the suggestion machinery are
[interfaces.md](interfaces.md#the-diagnostic-renderer)'s; the suggestion rule a
test asserts against is `FR-ERR-019`, `FR-ERR-020` and `FR-ERR-023`.

### The parser's mapping: one test per kind

The parser's own diagnostics reach `64` through a mapping
[`OD-08`](open-decisions.md#od-08--the-parsers-own-diagnostics) owed a test per
kind, because no documentation promises **which** context a given refusal
carries. The tests exist, in `src/cli/intercept.rs`, and each asserts the
`clap::ErrorKind` against this tree before asserting the four lines: a parser
version that reclassified one of these refusals fails where the refusal is
mapped, rather than silently producing the wildcard's message.

| `clap::ErrorKind` | Test |
|---|---|
| `InvalidSubcommand` | `fr_cli_003_an_unknown_subcommand_names_the_token_and_the_nearest_matches` |
| `UnknownArgument` | `fr_cli_019_an_unknown_flag_names_the_token_and_the_nearest_matches`, and `fr_cli_017_a_token_the_command_takes_no_argument_for_is_named_as_written` for the condition `FR-CLI-017` separates from it |
| `MissingRequiredArgument` | `fr_err_034_a_missing_required_argument_names_the_command_and_the_argument` |
| `InvalidValue` | `fr_err_034_a_value_outside_an_enumeration_names_the_value_and_the_values_accepted`, `fr_err_034_a_flag_given_without_any_value_says_so_rather_than_naming_a_value`, and `fr_cli_018_a_separate_token_value_beginning_with_a_dash_shows_the_corrected_form` for the two kinds the one condition of `FR-CLI-018` arrives under |
| `ValueValidation` | `fr_err_034_a_value_of_the_wrong_type_names_the_value_and_the_type_expected` |
| `ArgumentConflict` | `fr_cli_014_a_flag_that_carries_no_value_is_refused_on_its_second_occurrence` |
| The wildcard arm, reached by `InvalidUtf8` | `fr_err_034_a_refusal_this_crate_does_not_classify_is_a_sixty_four_naming_what_it_can` |

Two further tests hold the other half of that entry.
`fr_err_033_no_byte_the_parsers_own_renderer_composes_reaches_the_caller`
renders what the parser's own renderer would have written for eight refusals and
asserts that nothing it composes survives into the four labelled lines, and
`fr_err_034_every_flag_whose_value_is_parsed_names_the_type_it_expects` walks
the tree feeding every flag a value no parser accepts, so a flag whose type the
`cause` line cannot name fails there rather than reaching a caller.

**What the wildcard test asserts is what the binary does, not what the entry
first assumed.** `InvalidUtf8` carries no context this crate reads, so no token
is named; the test asserts the kind, the `64`, and the `cause` that says the
parser named none. `OD-08` records the wording and why the degradation is the
safe direction.

## Help: snapshots at every depth

`FR-HELP-002` requires byte-identical output between `tpl help <path>` and
`tpl <path> --help` — two distinct code paths — and `FR-HELP-003` makes `-h`
produce the same bytes as `--help` rather than a summary. `BR-HELP-001` makes a
byte-for-byte snapshot test the guarantee, **at every depth of the tree**.

| Population, derived from the tree of `FR-CLI-002` | Count |
|---|---|
| Nodes, root included | 35 |
| Depths, root at depth 0 | 4, the deepest being `tpl cfg database add` |
| Help forms per node (`tpl help <path>`, `tpl <path> --help`, `tpl <path> -h`) | 3 |
| Group nodes, which print their own help when invoked with no child (`FR-CLI-007`, `FR-HELP-025`) | 6, adding a fourth form for each |
| Version forms, byte-identical to one another (`FR-HELP-002`, `FR-HELP-005`) | 3 |

The two counts are **derived from the tree and the group-node list, not stated
by any requirement**: 35 nodes over the eight top-level commands of
`FR-CLI-010`, and the six group nodes of `FR-CLI-008`. They are recorded because
a snapshot suite that covers fewer nodes than the tree holds is the failure
`BR-HELP-001` exists to prevent, and the arithmetic is the only thing that says
so.

Aliases (`FR-CLI-011`) are not nodes and add no snapshot; what they add is a row
to the tree-completeness test of `BR-HELP-003`, which asserts that every command,
**alias** and flag appears in the JSON document.

**The JSON tree is derived by introspecting the parser** (`FR-HELP-021`), which
is what makes the third test of `BR-HELP-003` — every example parses through the
command parser itself — a check of the parser against its own documentation
rather than of one text against another. The help surface as built is
[interfaces.md](interfaces.md#the-help-surface)'s.

### Which tests exist

At commit `f8f335d` of 2026-09-15, under the names `bfa043d` gave them. Every
test in `tests/` launches the binary in an empty directory with no `.tpl` above
it and under a cleared environment, because a run that happened to stand inside
a project would pass for the wrong reason.

| Test | What it covers |
|---|---|
| `tests/help_surface.rs::fr_help_002_the_three_help_forms_are_byte_identical_at_every_node_of_the_tree` | `BR-HELP-001` and `FR-HELP-002`: the three forms compared byte for byte at all 35 nodes, the root among them, at every depth |
| `tests/help_surface.rs::fr_help_025_a_group_node_with_no_child_prints_what_its_help_form_prints` | The fourth form, at all six group nodes, the root among them (`FR-CLI-007`, `FR-HELP-025`) |
| `tests/help_surface.rs::fr_help_005_the_three_version_forms_write_exactly_the_line_of_the_requirement` | The third equivalence, and the line itself (`FR-HELP-005`) |
| `tests/help_surface.rs::fr_help_003_the_short_help_flag_is_not_a_summarised_long_one` | `FR-HELP-003`, on content: the seven sections and the four facts a leaf with an operand and an enumerated flag states |
| `tests/help_surface.rs::fr_glob_019_a_leaf_that_requires_an_operand_still_answers_both_flag_forms` | All fourteen required operands answer `-h`, `--help`, `-V` and `--version`, and the node still refuses a missing operand with `64` |
| `tests/help_surface.rs::fr_help_027_an_alias_prints_the_same_bytes_as_the_node_it_names` | The seven aliases of `FR-CLI-011`, in the text form and the JSON one, over eight comparisons — `cfg db add` being the alias read at depth three |
| `tests/help_surface.rs::fr_help_028_a_segment_that_names_no_child_is_refused_with_the_node_it_was_sought_under` | `FR-HELP-028`, on the process: `64`, an empty stdout, the suggestion, and a `cause` naming both the segment and the node |
| `tests/help_surface.rs`, six further tests | That every form succeeds where no project exists and nothing is in the environment (`FR-PROJ-025`, `FR-CLI-021`); the envelope and the four keys of `data`; the compact form and the indented one; the reduction of `FR-HELP-029`; the global flags carried once and repeated by no command; and a document byte-identical between runs |
| `tests/help_environment.rs`, four tests | `FR-HELP-009`, `FR-HELP-010` and `NFR-DET-004`: the same bytes under two environments that disagree about `COLUMNS`, `TERM`, `CLICOLOR_FORCE` and `FORCE_COLOR`; exit `0` on stdout; no line over 80 columns; no ANSI escape on either stream |
| `src/cli/help/document.rs`, three tests | The three properties of `BR-HELP-003`, each over the document `surface` builds, compared against the tree the binary parses with |

**The population the walk covers is read from the document rather than
listed**, so a node added to the tree is covered without any of these tests
changing — which is the property a snapshot suite that enumerates nodes cannot
have, and the failure `BR-HELP-001` exists to prevent.

**No stored reference file exists.** The comparison is between the forms, which
is the equivalence `FR-HELP-002` states and `BR-HELP-001` binds; nothing here
compares today's bytes with a copy taken earlier, so a change to a help text is
a change nobody has to re-bless, while a change that made two forms diverge
fails.

## The dump round-trip

`BR-SCH-004` mandates the test that keeps rendering without a database safe to
rely on: dump the reference database, feed the dump back through
`tpl render --context`, and assert that the rendered result is **byte-identical**
to the result of the same render against a live read of the same database.

| What the test depends on | Requirement |
|---|---|
| The dump is one complete document; `--pattern` on it is `64` | `FR-SCH-016`, `FR-SCH-021` |
| The document the dump emits is accepted by `--context` | `FR-SCH-022`, `FR-RND-016` |
| The three always-injected variables are injected by the render and not taken from the document | `FR-RND-024` |
| A marked document is refused as a context, whatever the render selects | `FR-PRIV-008`, `FR-PRIV-009` |
| Every object referenced from another is present | `FR-CTX-023` |

It is a **server** test on both halves: the dump and the live read must come
from the same database in the same state. `BR-SCH-004`'s own note records that
the container it needs now exists at all four series, and attributes the
remaining block to `tpl` not existing; the binary exists at this commit, and
what blocks the test is the two commands it runs, neither of which is written.

## Cross-series equivalence across four series

`FR-SRV-026` is the strongest testable form of *supported*, and `FR-SRV-029`
mandates two tests for it: the equivalence, executed against **every** series of
`FR-SRV-015`, and the refusal of `FR-SRV-020` against **at least one series
outside** the window. The equivalence, its three exceptions and their three
bounds are
[quality-attributes.md](quality-attributes.md#cross-series-equivalence-and-its-exceptions)'s
and are not restated; what belongs here is what the tests must not assume.

| Caution | Source |
|---|---|
| A difference may fall **anywhere** in the window. Of the fourteen differences `FR-SRV-038` records, nine separate `10.11` from the rest and two separate `12.3` from the rest; three — differences 2, 8 and 14 — put `10.11` and `11.4` on one side and `11.8` and `12.3` on the other. A test that models the window as one old server and three modern ones is right for nine of the fourteen and wrong for five: differences 2, 6, 8, 9 and 14 | `FR-SRV-038`, differences 2, 8, 14 |
| A statement naming a fixed `INFORMATION_SCHEMA` column list must be common to all four series or selected per series. Naming an absent column is a hard `ERROR 1054`, not a `NULL` and not a warning | `FR-SRV-037`; differences 5, 6, 7 |
| A field that differs between series without a row in the register is a **failure** of `FR-SRV-026`, not an instance of its exception | `FR-SRV-027`, `FR-SRV-036`, `FR-SRV-039` |
| The catalogue comparison already recorded under `FR-SRV-038` is evidence, not the test: it compares catalogue material, and `FR-SRV-026` compares the document `tpl` emits | `FR-SRV-038`, `FR-SRV-029` |
| The series table decays on a calendar, and a re-derivation that changes the set changes what these tests run against | `FR-SRV-019`, `BR-SRV-004`; the gate is [operations.md](operations.md#the-release-gates)'s |

**Recorded contradiction — the refusal test has no container.**
`FR-SRV-029` requires an integration test of the refusal against at least one
series **outside** the window. `FR-SRV-020`'s own rationale states that every
such series "is a series the project will never observe against the container of
`scripts/mariadb/`, because observation follows support", and
`scripts/mariadb/README.md` defines exactly four images, one per **supported**
series, and names none outside the window. Both readings are recorded:

- **The rationale is about catalogue observation**, and a container outside the
  window would exist only to be refused — `FR-SRV-020` refuses before the
  catalogue is read, so nothing about its catalogue is ever observed. On this
  reading the two requirements are consistent and the fixture is simply short a
  container.
- **The rationale is about the fixture's contents**, in which case
  `FR-SRV-029`'s second test has nothing to run against and the corpus asks for
  a test it also declines to provision.

Nothing is corrected in either file. What the refusal test runs against is **not
established**, and it is named among what the harness does not supply below.

**The greeting gap is discharged, and what it obliges a test is not.** The
harness observed that the version a server sends in its initial handshake
greeting carries a `5.5.5-` prefix on `10.11` and on no other series, while the
probe of `FR-SRV-002` answers without it everywhere
(`scripts/mariadb/README.md`, difference 9), and this folder reported the
observation to the functional owner rather than record a difference on the
corpus's behalf. The twelfth edition of `/specification` recorded it, as
**difference 13 of `FR-SRV-038`**, and the two requirements it bears on —
`FR-SRV-040` and `FR-SRV-041` — each carry a note saying it was checked against
the difference and stands. What survives the discharge is an obligation on a
test: no test named here rests on the greeting — `FR-SRV-002` determines the
version by the probe, not by the announcement — and a test, a diagnostic or a
fixture gate that reads a greeting must not assume the two readings agree,
because on one of the four supported series they do not.

## The reduced-privilege reader and the three shapes of absence

`FR-PRIV-018` fixes three shapes, observed on 2026-09-10 against all four
series, and they do not resemble one another: **a reader that looks for one
finds none of the others**. The fixture supplies the reader that produces all
three — `tpl_reader`, holding `SELECT, EXECUTE ON freight.*` and nothing more
(`scripts/mariadb/README.md`).

| Shape | Property it falls on | What the test asserts | Requirement |
|---|---|---|---|
| **The empty string**, on a row that is present | A view's definition | A named view exits `77`; a listing or a dump marks the view, with `definition` among the names in `restricted` | `FR-PRIV-011` |
| **SQL `NULL`**, on a row that is present | A routine's body | A named routine exits `77`; a listing or a dump marks the routine, with `body` in `restricted` | `FR-PRIV-017` |
| **Zero rows** from one catalogue table against full rows from another | A table's referential rules | A table whose key column names a referenced table with no referential row for its constraint exits `77` when named, and is marked in a listing or a dump | `FR-PRIV-019` |

Two further properties are part of the same test population and are what the
fixture's reduced reader makes reproducible.

- **What the reduced reader keeps must not be reported as lost.** It retains
  every check constraint, every index including the unique ones, and the primary
  key, because `FR-CAT-043` reads the last two from a table the privilege does
  not remove. A test asserting a wholesale loss of key metadata would pass
  against a wrong implementation.
- **The asymmetry is the design and both uniform answers are defects**
  (`BR-PRIV-001`): a **named** object that is incomplete fails with `77` and no
  partial object is returned (`FR-PRIV-003`, `FR-PRIV-004`); a listing or a dump
  succeeds and marks **each** incomplete object individually (`FR-PRIV-005`,
  `FR-PRIV-006`), and never marks a complete one (`FR-PRIV-007`). Two tests per
  shape, therefore, not one.

`restricted` is an ordered, never-empty array of model property names
(`FR-PRIV-016`); a marked object never reaches the cache (`FR-CACHE-037`) and a
marked document is refused as a context (`FR-PRIV-008`). The detections
themselves — which two compare two observations of one population and which is
self-announcing — are
[interfaces.md](interfaces.md#the-three-privilege-detections)'s, together with
the discrepancy recorded there about how many are cross-checks.

**A fourth absence exists and is not testable.** A hidden trigger list is
byte-identical to an empty one, on every series, and `FR-PRIV-020` states that
limit rather than leaving it to be discovered. **No test asserts that a table's
triggers were readable**, and none can; a test that appeared to would be
asserting the common case of a database with no triggers.

## The fixture's stated gaps, and what they bound

The fixture is the only database any validation may use — no mock, no stub, no
external instance — and its standing as an operational asset is
[operations.md](operations.md#the-fixture-as-an-operational-asset)'s. Since task
#15 the certificate obligation of `FR-CONF-038` is met, so a server-reaching
test named here may be written at the `verify-identity` default of
`FR-CONF-013`, and the fifth container is what gives the mode table its
server offering no TLS.

`FR-SRV-038` names three gaps so that they are not mistaken for observations.

| Gap | What it bounds |
|---|---|
| No row holds a non-`NULL` `INET4`, `INET6` or `UUID` value | The types are exercised as catalogue metadata only; no test may rest on a value of one |
| No comment carries a supplementary-plane character | The four-byte path through the `utf8mb3` comment columns is unexercised. The fixture records that such a character is silently replaced by `?` at DDL parse time with `warning_count` left at 0, so the gap cannot be closed by a test against the fixture as it stands |
| `tariff` uses implicit system versioning | `IS_SYSTEM_TIME_PERIOD_START` and `IS_SYSTEM_TIME_PERIOD_END` never read `YES`, so no test covers a column for which they do |

Three structures are **deliberately absent** rather than hidden behind a
conditional, because the identical DDL must be accepted by all four series
(`FR-SRV-029`): a `VECTOR` column or index, rejected by `10.11` and `11.4`; a
`SET` member containing a comma, rejected by all four and a property of the type
rather than a series difference (`FR-CAT-034`); and the `utf8mb4_uca1400_*`
collations, which are the default on three of the four and would make a declared
collation indistinguishable from an inherited one.

One file the fixture still owes bounds what can be measured rather than what can
be asserted: `seed-bench.sql`, which `WL-001` needs and which
[`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001) schedules for the
sprint that implements the catalogue reader. Rows 1 and 2 of
[the nine observations](#the-nine-observations-made-outside-the-process) wait on
it, because a query count that does not scale cannot be demonstrated against ten
tables.

## The twelve end-to-end flows

`UC-001` through `UC-012` are the acceptance-test skeleton and the source of
every `EXAMPLES` section. Each composes requirements owned elsewhere and
introduces no behaviour of its own, so a flow test asserts the **composition**:
the exit code, the bytes on stdout, and the postcondition each use case states.

| Flow | What it exercises | Kind | Requirements cited by the use case |
|---|---|---|---|
| `UC-001` | Bootstrap: `tpl init` writes five artefacts, `0`, nothing on stdout; a second run is `73`; a nested project warns and succeeds | Integration | `FR-PROJ-017`, `FR-PROJ-022`, `FR-PROJ-025` |
| `UC-002` | Registering a database entry, and the two refusals: no connection detail at all, and an entry that already exists | Integration | `FR-CFG-015`, `FR-CFG-027` |
| `UC-003` | Keeping the password off disk: the command string split by POSIX quoting, stored as an array, executed without a shell, its deadline producing `78` | Integration | `FR-CONF-023`, `FR-CONF-028` |
| `UC-004` | Connectivity: four steps reported together, and the four outcomes that separate `69`, `77`, `78` and a `0` carrying `can_read_catalogue` false | Server | `FR-CFG-024`, `FR-CFG-043`, `FR-CFG-045` |
| `UC-005` | Learning the whole CLI in one call: the complete tree as one compact document, in a directory with no project | Integration | `FR-HELP-016`, `FR-OUT-024`, `FR-PROJ-025` |
| `UC-006` | Inspecting structure: a listing as text, one table as JSON, a `66` with a nearest match, and the read left cached | Server | `FR-SCH-009`, `FR-CACHE-007` |
| `UC-007` | Rendering one object to stdout, and the three failures: absent template, render failure, two object flags | Server | `FR-RND-001`, `FR-RND-028` |
| `UC-008` | Rendering every table: the caller's loop, one invocation per table, because there is no `--all-tables` | Server | `FR-RND-002`, `BR-RND-002`, `FR-SCH-029` |
| `UC-009` | Rendering without a database: dump, commit, render from the document; the pipeline form; `--context` with an explicit `-d` is `64` | Server for the dump, integration for the render | `FR-SCH-022`, `BR-SCH-004`, `FR-RND-016` |
| `UC-010` | Working offline from a warm cache: a load, reads that open no connection, an interrupted load leaving what was stored unchanged | Server, then integration | `FR-CACHE-006`, `FR-CACHE-032`, `FR-CACHE-022` |
| `UC-011` | Repointing an entry, and the accepted failure mode: a stale cache answers with exit `0` and says nothing, the load time being the only signal | Server | `FR-CACHE-028`, `FR-CACHE-029`, `BR-CACHE-003` |
| `UC-012` | Recovering from a mistyped name: the four-line error, a runnable `hint`, a candidate suppressed when it falls outside the permitted character set, and a text diagnostic under `--format json` | Server | `FR-ERR-019`, `FR-ERR-023`, `FR-ERR-033` |

Three of the twelve carry an obligation beyond their own flow. `UC-005` is the
only flow that must run where **no project exists**, which is what makes
`FR-PROJ-025`'s exemption observable. `UC-009` is the round-trip of
`BR-SCH-004`, listed once in the register above and exercised here as a flow.
`UC-011` is the only flow whose expected outcome is a **wrong answer with exit
`0`**: the test asserts the documented failure mode, not its absence.

**Which flows the built tree can reach, at 2026-09-17.** Three, and one of them
only in part. `UC-001` and `UC-005` run whole. `UC-002` runs whole, and
`tests/project_and_configuration.rs` exercises both of its refusals and the
two `FR-CFG-048` adds. `UC-003` runs only as far as the file: the array the
quoting rule produces is asserted on the write path, and the execution half —
no shell, the cap, the deadline, the `78` — is reachable from no command,
because the resolution that runs the child is entered only by a command that
opens a connection. It is covered by unit tests against the child until
`UC-004`'s subcommand exists. The other eight need a server or a command that
is not written.

## The container harness

**The harness exists.** Task #25 discharged the harness half of
[`OD-22`](open-decisions.md#od-22--the-test-harness-and-the-fixture-certificate)'s
residual by running it rather than describing it, and `scripts/mariadb/README.md`
is its record: the scripts, their output, and the observations each instrument
actually produced. That file is not restated here. What belongs here is which
obligation each part discharges, and what a test suite still has to decide.

### What the harness must achieve, and what discharges it

| Obligation | Source | Discharged by |
|---|---|---|
| Every validation that needs a database uses the containers of `scripts/mariadb/` — never a mock, never a stub, never an external instance | `CLAUDE.md`, *Testes contra MariaDB* | The fixture is the only database; nothing else is provisioned |
| Containers are launched before a run and stopped after it; **no container is left running** | `scripts/mariadb/README.md` | `up.sh` and `down.sh`, the latter asking the daemon what is left rather than asserting it |
| All four series run **side by side**, each with its own image, container and published port, so that a cross-series comparison is one run rather than four | `FR-SRV-029`, `FR-SRV-015` | Four images, four containers, four ports — **and a fifth container that is not a fifth series**: the `--skip-ssl` server `FR-CONF-038` obliges, on a fifth port |
| A container is confirmed to have initialised cleanly before any test runs against it, because a failure inside the init directory leaves the schema incomplete while the container still reports itself up | `scripts/mariadb/README.md` | `up.sh` checks each server twice — the published port answering, and the log and object count — and removes a server it cannot make healthy |
| A server-dependent test can ask whether the fixture is up, without a client installed | `scripts/mariadb/README.md` | `status.sh`, whose exit code distinguishes **no** fixture from **half** a fixture |
| A container of a series **outside** the window is available for the refusal test | `FR-SRV-029`, `FR-SRV-020` — and see the contradiction recorded above | **Nothing.** The fixture defines no such container |
| The reduced-privilege reader is reachable beside the privileged one, on every series | `FR-PRIV-018`, `FR-PRIV-011`, `FR-PRIV-017`, `FR-PRIV-019` | `tpl_reader`, with the three shapes of absence observed identically on all four series |
| The statements a server receives, and the connections it accepts, are observable **from the server side** | `FR-SRV-012`, `FR-SRV-013`, `FR-SRV-014`, `BR-SRV-003` | `observe.sh statements` and `observe.sh connections`, each established against a substitute client and each with its own observer baseline measured rather than assumed |
| The files a process opens are observable from outside it | `NFR-PERF-005`, `NFR-PERF-007` | `observe.sh opens`, on a Linux host. See the target limit below |
| Every command of the tree can be run at maximum verbosity with both streams captured whole | `BR-SEC-003` | The fixture supplies the servers the commands that reach one need; capturing both streams whole is the suite's, and the suite does it — every launch of the binary in `tests/` captures both. The sentinel test itself waits on the commands that carry a credential to a server |

**The instruments were established against a substitute client**, `tpl` having
no catalogue reader to send a statement with. That is the correct order for an
instrument whose whole purpose is to observe from outside the process under
test: what was established is that the instrument sees what a client sends, not
what any particular client sends. **Two tests now drive `tpl` itself** against
the four series, and what each establishes is an absence — no connection
accepted, no statement received — which is why each carries the control the
absence is worthless without: the same record is shown to count a connection
that was made, and to hold the observer's own statements, over the same window.

### The gate, and the fourth instrument

Two of the open points this section recorded were discharged at commit
`a6d1fed`, by building them rather than by settling them on paper. Both are the
**suite's** and neither is the fixture's, which is what the open points
predicted: neither needs a container.

**The gate is `scripts/mariadb/status.sh --quiet`, and its exit code is mapped
by value.** The suite reaches it through `tests/support/fixture.rs`, which also
wraps the three server-side instruments, reads the inventory from
`status.sh --export` and the series from `series.env`, and issues no `docker`
command of its own. No port, credential, server name or schema name is restated
in Rust.

| Exit code | What the suite does |
|---|---|
| `0` | Runs the body |
| `1` | Skips it, and prints why |
| `2`, every other code, and a gate that was signalled rather than exited | **Fails the run** |

The third row is the row that earns the three-valued code. Half a fixture is not
an absent one, and a gate that could not be asked has answered nothing: mapping
either onto a skip would report a real failure as a pass, which is the failure
the code exists to prevent.

**A skip is written to `/dev/stderr`, past the test harness's capture.**
`libtest` captures both streams of every test — its own `--no-capture` is
documented as *"don't capture stdout/stderr of each task, allow printing
directly"* and `--show-output` as *"Show captured stdout of successful tests"*
(`libtest` command-line help, rustc 1.98.1, read 2026-09-18) — so a notice
written with `println!` or `eprintln!` is shown for no passing test, and **a
skip a caller cannot see is a skip a caller reads as a pass**. A second handle
on the process's own file descriptor `2` is outside the capture; the notice goes
through it, opened for append and written in one call, because the tests that
report one run on parallel threads.

**The fourth instrument of `NFR-PERF-007` exists, in
`tests/support/differential.rs`.** It records the three channels the requirement
names — the exit code, the bytes on stdout, and the paths left under an observed
root — and no fourth. It needs no container and no privilege, so it is available
on all four targets of `NFR-PERF-018`, which is what `NFR-PERF-005` relies on
where no syscall tracer exists. **Every run made through it carries its
inversion**: the same comparison over a command that *does* perform the
operation, whose two outcomes are shown to differ — because an equality between
two outcomes establishes nothing until the arrangement is shown to be potent.

### What is not established

Two rows of this table were discharged at commit `a6d1fed` and are recorded
above rather than deleted: the fourth instrument, and what the suite does with
each value of the gate.

| Open | Why it is not answered here |
|---|---|
| **The file-open instrument does not exist on either Darwin target.** On a host with no `strace` the harness traces a Linux build inside a container and says so; `NFR-PERF-005` forbids crediting that observation to macOS | The limit is the platform's, recorded by `NFR-PERF-005` with the three commands that established it. What would lift it is named there, and is not this folder's to take |
| **What the refusal test of `FR-SRV-029` runs against** | The contradiction is recorded under [cross-series equivalence](#cross-series-equivalence-across-four-series) and is the functional owner's; the fixture provisions no container outside the window |
| **Two observations the fixture could not produce**: the failing outcome of `FR-SRV-013`, and rows 1 and 2 conclusively | The first needs a server that accepts the read-only statement and does not apply it, which no real MariaDB does — a fault-injection seam in `tpl`, not a container, and no such seam is decided. The second waits on `WL-001` ([`OD-27`](open-decisions.md#od-27--seed-benchsql-and-wl-001)) |

## What this document defers, and to what

| Subject | Where |
|---|---|
| Every budget, its standing, the measurement protocol and the three workloads | [quality-attributes.md](quality-attributes.md) |
| Every measured figure and every recorded baseline | `BENCHMARKS.md` |
| The validation pipeline, the release gates and the fixture's operational standing | [operations.md](operations.md) |
| The fixture's contents, its credentials, its deliberate omissions, the harness scripts and their recorded output, and the nine differences its own passes observed between the series | `scripts/mariadb/README.md` |
| The record of every difference observed between the series — fourteen | `FR-SRV-038` |
| The privilege detections as built, the diagnostic renderer, the emitter and the help surface | [interfaces.md](interfaces.md) |
| The engine construction the seams and the vectors run against | [architecture.md](architecture.md) |
| What a mandated test asserts **about**, requirement by requirement | `/specification`, cited here by identifier and never reproduced |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
