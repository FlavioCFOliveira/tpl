---
title: Decision Register
status: draft
last-reviewed: 2026-09-25
related: [README.md, traceability.md]
---

# Decision Register

## What this is

Thirty-four entries, each a decision the repository could not settle on its
own. **Thirty-three are settled. One, `OD-33`, is open.** Nineteen were settled by
the product owner in the interview of 2026-09-10, by the establishment of the
decision register, and by the eighth edition of `/specification`; nine more were
settled on 2026-09-11, together with all five residuals the eighth edition left
inside settled entries. The fifth, `OD-22`'s, was work rather than a decision:
it was executed on 2026-09-11 by tasks #15 and #25, and the entry records what
each produced.

**Two entries were added on 2026-09-15**, when the command surface was built and
this folder was reconciled against it: `OD-29`, the two shapes of the JSON
command tree, settled by the twenty-first edition of `/specification` after the
implementation found them unstated; and `OD-30`, the interim outcome of a leaf
the parser accepts and no sprint has implemented yet. `OD-08` was amended the
same day, against the behaviour the binary was observed to produce.

**One entry was added on 2026-09-18**, when the model was built: `OD-31`, the
shape of the model's types, which answers four of the five library-shape
questions `DIV-032` hands to architecture. `OD-05`, `OD-18` and `OD-19` were
re-read against the same build the same day; the first two carry a refinement
and two amendments, and the third carries an observation now owed to
`adr-guardian`.

**One entry was added on 2026-09-21**, when the backlog was cleared: `OD-32`,
the removal of `anyhow` from the dependency graph, settled by the user that day.
It is the fifth entry to prepare a correction to `CLAUDE.md`, and the user
applied that correction on 2026-09-22. `OD-20` was narrowed the same day:
`FR-ERR-039` names the edit-distance variant, so the entry holds only how the
distance is computed.

**One entry was added on 2026-09-22**, when sprint 18 delivered `examples/`:
`OD-33`, whether `UC-013` joins the twelve-flow acceptance skeleton. It is the
**one entry carrying `Open` today**, and it is open because the choice is not
this folder's alone. `OD-04` was corrected the same day: the
`[workspace]` table now carries a second exclusion, and the entry said it
carried one.

**Four settled entries were amended on 2026-09-23**, against the forty-second
edition of `/specification` and the code of sprint 20: `OD-05` gains `heap.rs`
and `render/bounds.rs`; `OD-10` gains two read-side misses and its collision
outcome is superseded by the user's decision for rmp `#254`; `OD-11` records when the runtime ends; and
`OD-12` records the child's process group and the render's watchdog, and
qualifies two of its rejections. No entry was added, and none was reopened. The
poll interval [`ADR-011`](../adr/adr-011-render-memory-accounting.md)
delegates is fixed in [architecture.md](architecture.md#the-render-bounds), not
here.

**Four settled entries were amended on 2026-09-24**: `OD-23` now cites
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md), which reverses the
entry's refusal to prescribe continuous integration and fixes the release
artefact. The same day, for rmp `#293`, `OD-03` was worded more precisely, and
its decision is unchanged: its binary-version rule names Semantic Versioning
2.0.0. For rmp `#294`, `OD-03` was amended: the binary version's value is
`0.0.1`, the first release the user chose. For rmp `#306` and `#307`, `OD-24`
adds `rustix`'s `fs` feature for directory-relative cache operations and
non-blocking opens, and `OD-10`'s rejection of `O_NOFOLLOW` is reversed by it;
the correction this owed `CLAUDE.md` is recorded below as applied.

**One entry was added on 2026-09-24**, for rmp `#257`: `OD-34`, the in-process
entry point `run_from`, whose rejected options have no record to live in.

**One factual claim was corrected on 2026-09-18**, in `OD-18`: the entry denied
that `indexmap` is in the dependency graph, and `cargo tree` at commit `fd51ca2`
shows that it is, under two parents that are not on the emitting path. The
decision is untouched and the entry records what moved.

One obligation survives the settlement, and it is named in its own entry rather
than left to be inferred:

| Entry | What is owed | To whom |
|---|---|---|
| `OD-19` | The record's wording — *owned copy* — against a value whose members are borrowed from the model, and the memory consequence the record draws from it | `adr-guardian` |

**`OD-14`'s obligation was discharged on 2026-09-21**, when the render work made
the observation it owed. The answer was the opposite of the expected one — a
defined `null` does not fail under `UndefinedBehavior::Strict`, and the engine
renders it `None` — and the entry records what the built system does about it.

**Two further obligations were discharged by the ninth edition of
`/specification`**, at commit `4ad5e8c` of 2026-09-11: `OD-28`'s amendment to
`FR-ERR-030`, and `OD-12`'s observation on the referent of `FR-CONF-004`.
Neither entry is reopened and neither history is dropped; each records what
landed. The one obligation of `OD-12`'s that fell to `verification` was
discharged on 2026-09-18, when the connection was built; the entry records the
two failure modes that were produced, the third that was not, and the
instruction of its own that the work overruled. `OD-08`'s obligation of the same
kind was discharged on 2026-09-15 and is recorded as discharged in that entry.

**No entry is a conflict.** The three that were — `OD-21`, `OD-22` and
`OD-24` — were resolved in the eighth edition, which read requirement against
requirement and named the requirement that yields in each case. `OD-28` was the
one entry whose settlement obliged the functional corpus to move, and it moved
first: the ninth edition amended `FR-ERR-030` before any implementation was
written against it, which is the order an implementation choice may never
invert when a contractual code is at stake.

Each settled entry records the decision, and with it either its rationale and
**the options rejected** or a citation of the record that carries them, for the
reason [`docs/adr/README.md`](../adr/README.md) gives for the same section in a
record.

The architecture decision record register now exists (`OD-01`). A settled
entry's rationale belongs there where rule R4 of
[`docs/adr/README.md`](../adr/README.md) admits it, and this file cites it by
`ADR-NNN`; everything R4 does not admit is carried here. **Eight entries are so
reduced today**, each naming its record in its own status line; `OD-01` cites
the register itself rather than a record.

### Corrections owed to `CLAUDE.md`

`CLAUDE.md` is coordination and **this folder never edits it**; six
corrections have been prepared for it, and **all six have been applied**. Four
landed on 2026-09-11, each entry recording the commit that discharged it; the
fifth was prepared on 2026-09-21 and applied by the user on 2026-09-22.

| Entry | Correction | State |
|---|---|---|
| `OD-09` | The stack table's configuration row splits into `toml` and `toml_edit` | Applied. Task #20, commit `ee7363d` |
| `OD-17` | The logging row loses `tracing` and `tracing-subscriber` | Applied. Task #20, commit `ee7363d` |
| `OD-24` | The stack table gains a row for `rustix` | Applied. Task #20, commit `ee7363d` |
| `OD-26` | *Fontes de Verdade* gains this folder as a fourth source | Applied. Task #46, commit `c6356df` |
| `OD-24`, amended 2026-09-24 | The *Stack* table's `rustix` row reads `features = ["fs", "process"]`, and its note names the directory-relative calls beside `getuid`, `kill_process_group` and `waitid` | **Applied 2026-09-24**, at line 475: the row reads `features = ["fs", "process"]` and names `openat`, `unlinkat`, `renameat` and `mkdirat` and the `O_NOFOLLOW`/`O_NONBLOCK` opens. Uncommitted in the working tree when this was read |
| `OD-32` | The *Stack* table's error row loses `anyhow`, and the *Tipos e erros* convention that repeats that row loses it with it | **Applied 2026-09-22, by the user**, at both lines. The table's row now states the positive choice — `thiserror` in the library, no error type of the binary's own — and cites this entry by name; the convention states the same. Uncommitted in the working tree when this was read |

**The whole register was re-read against `CLAUDE.md` at commit `8f936d4` on
2026-09-21**, the last commit to touch the file. The re-read before it was at
`c6356df` on 2026-09-11, and four commits have touched the file since —
`b066cfa` (2026-09-15), `cd6ce7e` (2026-09-17), `6a0cce5` (2026-09-18) and
`8f936d4` (2026-09-21), read from `git log -- CLAUDE.md`. **No ground this
register takes from that file moved.** The four sections every entry draws on —
*Stack*, *Orçamento de dependências*, *Plataformas Suportadas* and
*Desenvolvimento* — are untouched by all four commits, as are *Fontes de
Verdade*, *Invariantes de Implementação*, *Testes contra MariaDB*, *Tipos e
erros* and *Desempenho e Eficiência*, and each section name this register cites
still resolves. What the four commits added is coordination: a Regra Zero block,
the sections *Sinergia e Convergência do Esforço*, *Linguagem*, *Proactividade*
and *Completude*, the launch gate for subagents, and the split of `rmp` between
two skills. None of it grounds a technical decision, and none of it is cited
here.

**`OD-32`'s correction was applied on 2026-09-22, and no entry now claims one
is owed to `CLAUDE.md`.** The user applied it at both lines the entry named, 465
and 539; `grep -n anyhow CLAUDE.md` returns no match anywhere in the file
(2026-09-22), and the *Stack* table's error row now cites `OD-32` for the reason
the binary carries no error type of its own. The disagreement that entry
recorded — the coordination document naming a dependency the graph no longer
carried — opened when `anyhow` left `Cargo.toml` and closed here. No entry names
the root `README.md` at all.

**The register was re-read against `CLAUDE.md` again on 2026-09-22**, because
that edit is the trigger this convention names. The edit is two sentences wide
and both are in the error row's subject, so the pass was narrow: every entry
citing the file was read against it, and the nine sections named above all still
resolve. **No ground moved.** Two entries touch the edited text, and the edit
agrees with both — `OD-06`, whose reading of the entrypoint the row now states
in the file's own words, and `OD-32`, whose correction it is. The three other
rows this register has corrected in that table — `OD-09`'s configuration split,
`OD-17`'s logging row and `OD-24`'s `rustix` row — are untouched. **One caveat
the next re-read must carry**: the edit was uncommitted when it was read, so the
commit that carries it is not yet known and the log row below names the working
tree instead.

One statement about the file is corrected by this re-read rather than by any
entry: `OD-27` cited `DIV-036` for `scripts/mariadb/seed-bench.sql` as a file
that did not yet exist. **It was written on 2026-09-21** and `OD-27` records the
discharge in its own entry; what `DIV-036` still owes the root documents is
`/specification`'s register's to state, not this one's.

**A second sweep, for quotations rather than claims**, ran on 2026-09-11 over
the whole folder: every quoted span, every cited section name and every
Portuguese fragment attributed to a root document was tested against
`CLAUDE.md` at commit `c6356df` and against the root `README.md` at commit
`87dd6e3`, then the last to touch each. **`OD-02` was the only stale
quotation**, and is corrected in its own entry. **The sweep was run again on
2026-09-21 against `CLAUDE.md` at `8f936d4`, and nothing had gone stale**: this
register quotes that file twice — *"Onde e **como**"*, marked in `OD-26` as the
text `c6356df` replaced, and *"o erro previsível"*, which the file still carries
in *Fontes de Verdade* — and both hold. The one quotation of
`scripts/mariadb/README.md`, in `OD-22`, was tested the same way and still
resolves. Two passages quote the graph's former row —
[`OD-26`](#od-26--the-boundary-against-the-knowledge-graph) and
[README.md](README.md#the-four-sources-of-truth) — each explicitly as the text
`c6356df` replaced, which was confirmed against that commit. No passage of this
folder quotes the root `README.md`. This sweep is a different check from the
one above: a claim that a correction is owed says what this folder will do, and
a quotation says what another file contains, so a register can be clean of the
first while carrying the second.

**A third sweep, for summaries of the entries of `/specification`'s divergence
register**, ran on 2026-09-11 over the whole folder. Every citation of a
`DIV-NNN` identifier outside this record — forty, in eight files, naming twelve
distinct entries — was read against that entry's current state in
`specification/upstream-divergences.md`, which classifies its fifty-two entries
due, discharged or partly discharged. **Eleven passages were stale**: five rows
of section 26 of [traceability.md](traceability.md), two of
[quality-attributes.md](quality-attributes.md), two of this file, and one each
in [overview.md](overview.md) and [data-model.md](data-model.md). Each cited a
discharged or partly discharged entry and asserted in the present indicative
what `CLAUDE.md` contains or still owes; `0ea5624` and `87dd6e3` are the commits
that falsified them. All eleven now state the technical concern or the entry
instead, for the reason [README.md](README.md#conventions) gives. The other
twenty-nine citations name `DIV-024`, `DIV-032`, `DIV-033`, `DIV-034`,
`DIV-036`, `DIV-039`, `DIV-041` and `DIV-045`, and each was verified against
its entry. This sweep is a third check again: a summary claims no correction
and quotes nothing, so it passes both checks above while decaying on the same
edit.

The re-reading is a convention of this folder rather than an occasion, for the
reason [README.md](README.md#conventions) gives, and its trigger is an edit to
the target file. **Every re-read this register has made is recorded with its
commit and date**, so the next one starts from a known state rather than from
the last edition of this folder.

**An architecture decision record is a target on the same terms.** It is a file
this folder cites and does not own, so an amendment to one triggers the same
re-read as an edit to `CLAUDE.md` does. Two were amended on 2026-09-22 and both
were read against every passage of this folder that cites them; the outcomes are
the last two rows below.

| Target | Re-read at | Date | Outcome |
|---|---|---|---|
| `CLAUDE.md` | `c6356df` | 2026-09-11 | Three entries claimed a correction `ee7363d` had already applied; each now records the commit that discharged it |
| Root `README.md` | `87dd6e3` | 2026-09-11 | No passage of this folder quotes it, and no entry names it |
| `specification/upstream-divergences.md` | — | 2026-09-11 | Eleven passages stale, in five files; all eleven now state the concern or the entry |
| `CLAUDE.md` | `8f936d4` | 2026-09-21 | No ground moved. `OD-32`'s correction is confirmed still owed and still the user's |
| `CLAUDE.md` | Working tree over HEAD `455e48d`; the edit is **not yet committed** | 2026-09-22 | No ground moved. `OD-32`'s correction is applied at both lines and the *Stack* table now cites that entry; the register carries no correction owed to this file |
| `CLAUDE.md` | Working tree over HEAD `d89ffc4`; the edit is **not yet committed** | 2026-09-23 | No ground moved. The *Stack* table gains a heap-count row citing `ADR-011`, which this folder cites for the same crate, and the `rustix` row now names `getuid`, `kill_process_group` and `waitid` with `OD-24` and `OD-12`, as [technology-stack.md](technology-stack.md#the-calls-std-does-not-supply) does. No correction is owed |
| [`ADR-002`](../adr/adr-002-tls-mode-mapping.md) | Working tree over HEAD `455e48d`; the amendment is **not yet committed** | 2026-09-22 | Nothing this folder states moved. Its thirteen citations, in four files, name the five-mode mapping, the bundled anchors, the rejected platform store and the absence of a TLS version pin; what the amendment moved is the `ca_path` assembly, which no passage of this folder restates |
| [`ADR-007`](../adr/adr-007-msrv.md) | Working tree over HEAD `455e48d`; the amendment is **not yet committed** | 2026-09-22 | Nothing this folder states moved. Its fifteen citations, in three files, name the rule and the figure it yields, and neither changed; what the amendment moved — the dependency-floor table and the graph's crate counts — this folder has never restated |

## Status legend

| Status | Meaning |
|---|---|
| **Settled** | Decided. The rationale and the rejected options are recorded in the entry, or — where a record holds them — in the architecture decision record the entry's status line names |
| **Settled, with a residual** | Decided in substance. One narrow point remains, named in the entry with its owner and the document that settles it. **No entry carries this status today**: `OD-22`'s residual was discharged on 2026-09-11 |
| **Settled, with an observation owed** | Decided. One statement the entry rests on is unverified, or one wording of the corpus is imprecise; the entry names it, names its owner, and states what changes if it does not hold |
| **Settled, with an amendment owed** | Decided. The decision obliges `/specification` to move before any code is written against it. The entry names the requirement and the order. **No entry carries this status today**: `OD-28`'s amendment landed in the ninth edition |
| **Settled, interim** | Decided, and decided to be temporary. The entry states the arrangement, what a caller observes while it stands, and what removes it. **No entry carries this status today**: `OD-30`'s arrangement was discharged on 2026-09-22 |
| **Open** | Not decided. The entry names the options and the owner. **`OD-33` is the one entry carrying it**, since 2026-09-22 |
| **Conflict** | Two requirements, or a requirement and a mandated constraint, cannot both be honoured. Not a choice: a defect owed to `specification-manager`, and the documents it blocks wait for the correction rather than being written around it. **No entry carries this status today** |

## Index

| Entry | Subject | Status | Owner |
|---|---|---|---|
| [OD-01](#od-01--where-the-architecture-decision-records-live) | Where the architecture decision records live | Settled | — |
| [OD-02](#od-02--the-msrv) | The MSRV | Settled | — |
| [OD-03](#od-03--versioning-the-binary-the-document-the-cache-the-changelog) | Versioning: binary, document, cache, changelog | Settled | — |
| [OD-04](#od-04--one-package-or-a-workspace) | One package, or a workspace | Settled | — |
| [OD-05](#od-05--the-module-decomposition) | The module decomposition | Settled | — |
| [OD-06](#od-06--the-error-types-shape-and-the-exit-code-derivation) | The error type's shape and the exit-code derivation | Settled | — |
| [OD-07](#od-07--help-the-parsers-renderer-or-tpls-own) | Help: the parser's renderer, or `tpl`'s own | Settled | — |
| [OD-08](#od-08--the-parsers-own-diagnostics) | The parser's own diagnostics | Settled | — |
| [OD-09](#od-09--toml-the-read-path-and-the-write-path) | TOML: the read path and the write path | Settled | — |
| [OD-10](#od-10--cache-filenames-and-the-case-collision) | Cache filenames, and the case collision | Settled | — |
| [OD-11](#od-11--the-scope-of-the-async-runtime) | The scope of the async runtime | Settled | — |
| [OD-12](#od-12--how-six-phase-deadlines-are-enforced) | How six phase deadlines are enforced | Settled | — |
| [OD-13](#od-13--the-engine-pin-and-minijinja-contrib) | The engine pin, and `minijinja-contrib` | Settled | — |
| [OD-14](#od-14--which-undefined-behaviour-the-engine-is-configured-with) | Which undefined behaviour the engine is configured with | Settled | — |
| [OD-15](#od-15--the-template-loader) | The template loader | Settled | — |
| [OD-16](#od-16--the-tls-backend-and-the-root-store) | The TLS backend and the root store | Settled | — |
| [OD-17](#od-17--observability) | Observability | Settled | — |
| [OD-18](#od-18--serialisation-key-order-and-the-two-omissions) | Serialisation, key order, and the two omissions | Settled | — |
| [OD-19](#od-19--whether-the-two-embeddings-are-materialised) | Whether the two embeddings are materialised | Settled, with an observation owed | `adr-guardian` |
| [OD-20](#od-20--edit-distance-and-the-other-small-algorithms) | Edit distance, and the other small algorithms | Settled | — |
| [OD-21](#od-21--two-test-seams-that-must-not-be-on-the-published-surface) | Two test seams that must not be on the published surface | Settled | — |
| [OD-22](#od-22--the-test-harness-and-the-fixture-certificate) | The test harness, and the fixture certificate | Settled | — |
| [OD-23](#od-23--packaging-artefacts-and-the-musl-build-path) | Packaging, artefacts, and the musl build path | Settled | — |
| [OD-24](#od-24--the-discovery-boundary-and-the-process-uid) | The discovery boundary, and the process uid | Settled | — |
| [OD-25](#od-25--the-clock-source-for-now) | The clock source for `now` | Settled | — |
| [OD-26](#od-26--the-boundary-against-the-knowledge-graph) | The boundary against the knowledge graph | Settled | — |
| [OD-27](#od-27--seed-benchsql-and-wl-001) | `seed-bench.sql` and `WL-001` | Settled | — |
| [OD-28](#od-28--the-release-profile-against-the-caught-panic-condition-of-70) | The release profile against the caught-panic condition of `70` | Settled | — |
| [OD-29](#od-29--the-json-command-tree-two-shapes-and-what-the-binary-publishes) | The JSON command tree: two shapes, and what the binary publishes | Settled | — |
| [OD-30](#od-30--a-parsed-leaf-with-no-implementation) | A parsed leaf with no implementation | Settled; the interim arrangement is discharged | — |
| [OD-31](#od-31--the-models-shape-strings-fields-and-the-attribute) | The model's shape: strings, fields, and the attribute | Settled | — |
| [OD-32](#od-32--anyhow-in-the-shipped-graph) | `anyhow` in the shipped graph | Settled | — |
| [OD-33](#od-33--uc-013-and-the-twelve-flow-acceptance-skeleton) | `UC-013` and the twelve-flow acceptance skeleton | Open | user |
| [OD-34](#od-34--an-in-process-entry-point-over-a-supplied-argument-vector) | An in-process entry point over a supplied argument vector | Settled | — |

Thirty-two entries are settled outright; `OD-19` alone carries an observation
owed, and `OD-33` is open. Thirty-two, one and one are the whole of the
thirty-four. `OD-30` was settled and **interim** until 2026-09-22, when its last
arm was removed; it is now settled outright.

Two editorial defects were reported at the end as `ED-01` and `ED-02`. Both
were corrected in the eighth edition; neither is outstanding.

## Verification note

Every version number and every library behaviour cited below was verified
against the source named beside it — vendor documentation on `docs.rs`, the
crate index, the Rust Edition Guide, the Rust Reference, the Rust Book, the
Cargo Book, or a file of this repository. Each claim carries the date it was
verified on: **2026-09-10** for the nineteen entries settled that day,
**2026-09-11** for everything added since, **2026-09-18** for the two library
and format claims `OD-31` rests on, for the dependency-graph reading that
corrected `OD-18` and for the driver behaviour `OD-12` now records as observed,
and **2026-09-21** for the manifest, dependency-graph and source readings
`OD-32` rests on, for the reading of `src/mariadb/connect.rs` that `OD-12`'s
third correction rests on, and for the engine behaviour `OD-14` now records as
observed; and **2026-09-22** for the readings that confirm what `OD-32` decided
has landed — the crate out of the manifest, the lock file and the resolved
graph, and the correction applied to `CLAUDE.md`; and **2026-09-23** for the
`tokio`, `rustix` and `std` behaviour the amendments of `OD-11` and `OD-12`
rest on, and for the readings of `src/` behind the amendments of `OD-05` and
`OD-10`. Anything not verified says so
in its own text. No claim rests on recollection.

The engine behaviour `OD-14` records has a source of a different kind, and it is
named here because the difference matters: it is a **test of this repository**, not a page of
vendor documentation. The behaviour it records is **not confirmed in the
engine's official documentation** for the pinned line, which is why the entry
stood unverified for as long as it did; a test asserts it on every run and fails
where a release of that line changes it.

**An entry reduced to a citation carries no source of its own**, and neither
its sources nor its unverified points are restated here: both live in the
record its status line names, under the same rule. `OD-02` is the case that
matters, because the record it cites moved the number this entry carried.

The three entries the eighth edition settled — `OD-21`, `OD-22`, `OD-24` —
were re-read against the corpus at commit `9efa791` on **2026-09-11**,
requirement by requirement. Every identifier they cite was confirmed to exist in
that corpus.

The two entries the **ninth** edition discharged — `OD-12` and `OD-28` — were
re-read the same way against commit `4ad5e8c` of 2026-09-11, over
`errors-and-exit-codes.md`, `configuration-model.md`, `output-formats.md`,
`upstream-divergences.md` and the edition's own section of `README.md`. Every
identifier cited in the paragraphs that changed was confirmed to exist in that
corpus.

---

## OD-01 — Where the architecture decision records live

**Status: settled.**

**Decision.** The register lives at `docs/adr/` and is cited by `ADR-NNN`. The
convention and the authoritative index are documented in
[`docs/adr/README.md`](../adr/README.md), and are **not restated here**, per
rule R3 of that document.

---

## OD-02 — The MSRV

**Status: settled. Recorded in [`ADR-007`](../adr/adr-007-msrv.md).**

**Decision.** The MSRV is the floor `ADR-007` states.

**The deferral this entry answered is discharged, and the entry no longer
quotes it.** `CLAUDE.md` left the figure to be fixed in the `Cargo.toml` until
commit `50153d6` of 2026-09-11 reduced that row to a citation of `ADR-007`,
while this entry went on reproducing the deferral verbatim. A quotation of a
file this folder neither owns nor may edit decays when its owner edits it, and
no check the folder runs on itself detects that; the relation between the two
documents is recorded here instead, and the text of neither is reproduced. That
is the reason [README.md](README.md#conventions) gives for re-reading against
the target file, and it governs a quotation as much as a correction owed.

**The number this entry carried is superseded.** It stated a floor under an
explicit "not verified" caveat over the dependency floors. Those floors have
since been read from the crate index and one of them exceeds it, so the rule
this entry already stated — the pin rises to it — applies, and `ADR-007`
carries the result.

The number, the rule that yields it, the verified floors, the options rejected,
and the one point that remains unverified are recorded in `ADR-007` and are
**not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

---

## OD-03 — Versioning: the binary, the document, the cache, the changelog

**Status: settled.** Worded more precisely on 2026-09-24, for rmp `#293`: the
binary version's rule now names the standard and its version. The decision is
unchanged. Amended on 2026-09-24, for rmp `#294`: the binary version's value is
`0.0.1`, the first release, chosen by the user; it had been `0.1.0`.

**Decision.** Four numbers, four rules.

| Number | Value now | Rule |
|---|---|---|
| Binary version | `0.0.1` | Full [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html). While below 1.0, a breaking change is a **minor** bump |
| `schema_version` | `1` | Versions the document contract, independently of the binary, per `FR-OUT-011`; what breaks it is `FR-OUT-014` |
| `cache_format` | `1` | Versions the on-disk arrangement only, independently of `schema_version`, per `FR-CDOC-002` and `FR-CDOC-005` |
| Changelog | `CHANGELOG.md` | Keep a Changelog format |

`tpl.version` and the output of `tpl version` both come from
`CARGO_PKG_VERSION`, so the number has **one source**.

**Rationale.** The user chose `0.0.1` as the first release on 2026-09-24.
`FR-HELP-005` gives `tpl 0.1.0\n` as its worked case and `FR-CTX-027` carries
the same string; both are illustrative examples and do not fix the value.
Reading it from `CARGO_PKG_VERSION` is what makes
`FR-HELP-005` and `FR-CTX-027` incapable of disagreeing: a second literal
would be the copy that stops being true. Pre-1.0 breaking changes as minor
bumps is the semver convention for a version below 1.0 and needs no local
rule. `FR-ENV-029` requires a template-surface removal to be recorded in a
changelog and `CLAUDE.md` names one in its workflow, so the format is the only
open part, and a named public convention is preferable to a local one.

**Rejected.** Two literals for the version, one in the help and one in the
context, which is the duplication `FR-CTX-027` would otherwise permit; and
bumping `schema_version` or `cache_format` together, which `FR-CDOC-005`
forbids outright — "Neither SHALL be incremented on account of a change that
affects only the other."

---

## OD-04 — One package, or a workspace

**Status: settled. Recorded in [`ADR-006`](../adr/adr-006-package-layout.md).**

**Decision.** One Cargo package, carrying a library and a binary, and no
package of its own for `benches/`.

The rationale, the options rejected, the `cargo tree` invocation that answers
the dev-dependency question, and the `dhat` caveat that belongs to `operations`
are recorded in `ADR-006` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). The residual this entry carried was
settled on 2026-09-11 and is part of the decision that record holds.

**The manifest carries a `[workspace]` table with two exclusions, and neither
reopens this entry.** The first is the vendored directory
[`ADR-010`](../adr/adr-010-driver-tls-connect-stall.md) places in the
repository; that record states that it adds no second package and leaves
`ADR-006`'s decision untouched. The second, added in sprint 18, is
`examples/rust-data-layer` — the crate the Rust worked example of `FR-EX-001`
renders. It is generated rather than maintained, and it turns on a `sqlx`
feature this package does not. Excluding it keeps it out of the workspace, so
the mandated validation pipeline of `CLAUDE.md` *Desenvolvimento* neither
compiles it nor resolves and fetches what it needs; its own acceptance signal is
the compile gate `FR-EX-009` obliges the example to carry, run by the example's
script and outside that pipeline.

**The package count is what this entry decided.** `cargo metadata --no-deps`
returns one workspace member, `tpl`, read on 2026-09-22; a root build therefore
selects that package alone, which task #216 observed the same day of
`cargo build --release`. A second package in the repository is not a second
package in the build.

---

## OD-05 — The module decomposition

**Status: settled.**

**Decision.** Eleven modules under `src/`, the seven the project already
sketches plus four new ones. Each new module exists because a responsibility
crosses every command and would otherwise be copied per command, or because
placing it under an existing module would make that module own a subject its
name denies.

```
src/
├── main.rs        parse, dispatch, map the error to an exit status, and nothing else
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
└── error.rs       the error type and the exit-code derivation
```

**The six placements the entry asked for, and the seventh the answer to
`OD-12` created.**

| Responsibility | Home | Why there | Home rejected, and why |
|---|---|---|---|
| The catalogue cache | `cache/` | It sits **between** the reader and every consumer, and owns a subject neither neighbour owns: an on-disk arrangement with its own version, `cache_format`, which `FR-CDOC-005` makes independent of the model's | Under `mariadb/`, which would make a module named after the server own a filesystem format and a version the server knows nothing about; under `project/`, which would make the project module own catalogue semantics and per-collection completeness |
| Output formatting, the envelope, `text` layout, escaping | `output/` | One envelope governs all seventeen documents, per `FR-OUT-032`, and `FR-OUT-018` escapes on the way out of four command groups. A single owner is what makes "the envelope is the same everywhere" a property of the code rather than of review | Under `cli/`, which would put the envelope in as many places as there are commands that emit; under `model/`, which would make the model own its own presentation and put an escaping rule inside the type `FR-SCH-022` requires to round-trip unchanged |
| Help text and the typed examples and exit-codes table | `cli/help.rs` | `FR-HELP-021` derives the JSON command tree by introspecting the parser's tree, which lives here, and `FR-HELP-022`'s table is indexed by command path — `cli/`'s own vocabulary. It emits through `output/` for `help --format json` | A top-level `help/`, which would have to reach into `cli/` for the tree and the paths that are its only inputs, inverting the dependency for no gain |
| The privilege cross-checks | `mariadb/privileges.rs` | The three checks read the **shape of the rows the server returned** — an empty `VIEW_DEFINITION` (`FR-PRIV-011`), a `NULL` `ROUTINE_DEFINITION` (`FR-PRIV-017`), zero rows from three catalogue tables (`FR-PRIV-019`). None is a property of the model; each is a property of a read | Under `model/`, which would make the published model type know about grants, and would put a check on a shape the model no longer carries by the time it is built |
| The configuration reader and writer | `project/config.rs` | `FR-PROJ-010` and `FR-PROJ-011` make the ownership and mode of `.tpl/.cfg` a precondition of reading it, so the file and the folder that holds it are one subject. One module owns both paths over one key space, which is what keeps the key space of `FR-CONF-002` in one place | A top-level `config/`, which separates the file from the discovery that found it and the trust checks that gate it, and puts the key space one module away from the rule that decides whether it may be read at all |
| The diagnostic renderer and the suggestion machinery | `diagnostics/` | It holds transformations, not a taxonomy: the escaping of `FR-ERR-024`, the character set of `FR-ERR-022` and `FR-ERR-023`, the candidate selection of `FR-ERR-019`, and the four-line layout of `FR-ERR-008`. It also owns the verbosity gate of `FR-GLOB-014` and the typed diagnostic sinks of `OD-17` | Inside `error.rs`, which would put presentation beside the taxonomy and make the error type depend on an edit-distance implementation. `OD-06` separates the two for the same reason |
| The phase clock, and the threads two phases are bounded with | `deadline.rs` | `OD-12` gives one construct three users — the runtime inside `mariadb/`, the child process, and the render — and `FR-GLOB-012` composes every phase deadline with one budget measured from process start. A budget shared by three modules belongs to none of them | Inside `project/` beside the four `[core]` keys, which resolves the values but cannot hold the construct that applies them; and inside each of the three users, which is the same rule written three times |

**Refined on 2026-09-17, when `project/` was built: the reader and the writer
are two modules, not two paths of one.** The placement row above says
`project/config.rs` owns both paths over one key space, and the reason it gives
— that the file and the folder that gates it are one subject — is unchanged:
both modules are under `project/`, and neither is reachable without the trust
checks. What moved is one level down. `project/config.rs` reads and
`project/edit.rs` writes, because
[`OD-09`](#od-09--toml-the-read-path-and-the-write-path) gives them two
different parsers over two opposite obligations — reading validates and refuses,
writing preserves and must not reformat — and a module holding both would import
both parsers and hold two representations of the same document. The key space
they share is a third module, `project/config/keys.rs`, which is what keeps the
eighteen key forms in one place as the row requires — fifteen when this was
written, eighteen since the forty-second edition added three `[core]` keys. Three further modules sit beside
them for reasons stated in
[architecture.md](architecture.md#inside-project): `settings.rs`, because
`FR-CFG-014` forbids the reader to resolve; `password.rs`, because the child
belongs to the resolution; and `secret.rs`, because a credential is a type.

**Refined on 2026-09-18, when `model/` was built: the document is a submodule of
the model, and it is not a second home for presentation.** The placement row
above rejects `output/` **under** `model/`, on the ground that it would make the
model own its own presentation and put an escaping rule inside the type
`FR-SCH-022` requires to round-trip unchanged. None of that moves: the envelope,
the two forms, the escaping and the writer are `output/`'s, and `model/document/`
composes no byte and opens no file. What sits under `model/` is the document's
**shape** — its keys, and the two instantiations of the one-hop cut of
`FR-CTX-009` — which is the model's own contract and the thing the round trip of
`FR-SCH-022` is a property of. Thirteen modules sit beside it, and the
division is [architecture.md](architecture.md#inside-model)'s. One placement
inside the submodule is **not** settled by this refinement and is recorded as a
discrepancy in
[interfaces.md](interfaces.md#ordering-one-default-and-six-exceptions): which
component applies the ordering of `NFR-DET-002` to the document's collections.

**Refined on 2026-09-23, when the render memory limit was built: a twelfth
module, `heap.rs`, at the crate root.** `FR-RND-039` needs a count the
allocator keeps, and [`ADR-011`](../adr/adr-011-render-memory-accounting.md)
supplies it through a `#[global_allocator]`. The allocator is declared in
`main.rs`, which hands the library a function reading it through the public
`install_heap_counter`; `heap.rs` holds that one function in a `OnceLock` and
answers `None` where none was installed. It is at the crate root rather than
under `render/` because the count is process-wide state, as the phase clock's
budget is, and the reader is `cli/render.rs`. *Rejected — the library declaring
the `#[global_allocator]`.* Every test binary of the package would then run
under it, and the choice of the process's one allocator, which excludes an
in-tree `dhat` profiler (`ADR-011`), would be made for every program linking
the library rather than for the binary that ships. It follows the precedent of
the panic hook of `ADR-004`, which the binary installs for the same reason.
`render/` gains a submodule, `bounds.rs`, holding the three bound types and the
counting writer of `FR-RND-037`.

**Refined on 2026-09-24, when the file operations were made directory-relative:
a thirteenth module, `at.rs`, at the crate root.** `OD-24`'s amendment makes
every cache operation, and the read of `.tpl/.cfg` and of a template, start
from a directory descriptor, and three modules use it: `cache/` for every cache path,
`project/` for `.tpl/.cfg`, and `render/` for a template. It sits at the crate
root for the reason the phase clock does: a construct shared by three modules
belongs to none of them, and placed in each it is the same rule written three
times. The module's own documentation states its operations and the one step,
listing a directory, that still resolves a path by name
([data-model.md](data-model.md#tplcache)). Read at `c360c80`, 2026-09-25.

**The name `diagnostics` rather than `diag`.** The project's own convention
refuses obscure abbreviations in module names. The register named `diag/` as a
candidate; it is spelled out.

**`cli/` delegates; it does not do the work.** Each porcelain command is a
module under `cli/` that validates its own arguments, calls the library, and
hands the result to `output/`. The work itself lives in the module that owns
the subject.

**Rationale.** Three requirements decide this rather than taste. The project's
own organisation rule puts the logic in the library and reduces `main.rs` to
parse, dispatch and map. `NFR-PERF-005` requires `tpl init`, every form of
`help` and every form of `version` to perform no discovery, read no
configuration and open no connection — a property that is *observable* when a
command module is a thin adapter over a lazily reached subject, and that has to
be argued when a command module contains the work. And the same subject is
reached from several commands — a table is read by `schema table`, by
`schema dump`, by `render` and by `cache load` — so work placed in a command
module is work that is either duplicated or reached sideways.

**Rejected — a flat module per command with the work inside it.** It is the
shape a CLI takes when it grows without a decomposition, and it makes each of
the four rules above unenforceable: the envelope would exist per command, the
catalogue read would exist per command, and `NFR-PERF-005` would be a review
item rather than an observation.

**Rejected — folding `output/` and `diagnostics/` into one module.** They
share nothing: `FR-OUT-018` excepts tab in `text` read output and
`FR-ERR-024` escapes it in every message, on streams with opposite contracts —
stdout is byte-identical under `NFR-DET-001` and stderr is explicitly neither
deterministic nor contract. One module holding two
opposite escaping rules over two opposite promises is the shape in which the
wrong one gets applied.

**Visibility.** Everything is private by default; `pub(crate)` for what crosses
a module boundary; `pub` for `model/` and `error.rs` alone, which are the
surface the project documents. `lib.rs` re-exports those two deliberately and
re-exports no module whole.

**Unblocks.** `architecture`, `interfaces`.

---

## OD-06 — The error type's shape and the exit-code derivation

**Status: settled.**

**Decision.** Three answers, in the order the entry asked them.

| Question | Answer |
|---|---|
| One enum or several | **One** public `#[non_exhaustive] enum Error` in `error.rs`, derived with `thiserror`. A module may carry a private error for its own convenience and converts it at its own boundary with `From`; no module error is public and none carries an exit code |
| Where the exit code comes from | **An inherent method on the error**, `Error::exit_code`, in the library beside the enum. `main.rs` calls it and returns the status |
| Where the four labelled lines come from | **A renderer in `diagnostics/`**, not `Display`. `Display` carries the `error:` line's content and nothing else |

**Why one enum.** `FR-ERR-001` fixes ten codes and `FR-ERR-002` forbids
collapsing two conditions onto one code where the caller's next step differs,
so every condition the program can reach has to be assigned a code by someone
who can see all of them at once. One enum puts that assignment in one
exhaustive match, and the project's own rule against a `_ =>` arm that swallows
future variants turns a new condition without a code into a **compile error**.
Per-module enums composed by `From` spread the assignment across the `From`
implementations, where a new variant in a leaf module reaches the boundary with
whatever code the conversion happened to pick.

**Why the method and not a table in the binary.** The project's conventions
name the error enum as a typical carrier of `#[non_exhaustive]`, and it is
applied here. The Rust Reference states the effect plainly: "Cannot match on a non-exhaustive enum
without including a wildcard arm", because "matching on a variant does not
contribute towards the exhaustiveness of the arms" (Rust Reference, *Type
system attributes*, verified 2026-09-11). The binary is a **downstream crate**
of the library, so a table there could not be exhaustive: it would need the
wildcard arm, and the wildcard arm is exactly the construct that lets a new
condition ship with the wrong code. Inside the defining crate the match is
exhaustive and the compiler enforces the assignment.

**Why a renderer and not `Display`.** Four requirements pull the four lines
apart from the type:

- `FR-ERR-010` forbids `cause` to restate `error`, so one value owes **two** distinct strings and `Display` supplies one.
- `FR-ERR-034` fixes, per code, what `cause` must name, so the second string is derived from a per-code obligation rather than from the variant's own wording.
- `FR-ERR-024` escapes `\n`, `\r`, `\t` and every C0 control in **every value interpolated into a message**. The renderer escapes each composed line as a whole, which makes the rule hold for interpolations nobody remembered to escape — a property that review cannot supply and that a `Display` implementation per variant cannot either.
- `FR-ERR-022` and `FR-ERR-023` restrict what may enter a runnable `hint` and require a candidate outside `[A-Za-z0-9_]{1,64}` to be dropped entirely. That is a filter over a candidate set, not a property of the error value.

`Display` is still implemented, because `thiserror` derives it from the
`#[error(...)]` attribute and because the project's conventions ask for it where
the type justifies one. Its output is the `error:` line's content, unescaped;
the renderer escapes it on the way out.

**Why the variant set is not the taxonomy `FR-ERR-015` rejected.** That
requirement withdrew an emitted `kind` field, and its *Rejected* note refuses
"retaining `kind` as an internal taxonomy with no external carrier", on the
ground that "a classification nothing outside the process can observe cannot be
tested". The variant set has two external carriers and is tested through both:
the **exit code**, which `FR-ERR-001` makes contract and `BR-ERR-001` requires
an integration test for, code by code; and the **`cause` line**, whose content
`FR-ERR-034` obliges per code. What `FR-ERR-015` refuses is a *second*
classification that nothing observes. This is the first, and its projection is
what a caller branches on.

**The structural consequence for `FR-GLOB-018`, recorded here because it is the
type that enforces it.** No variant carries the database driver's error. The
driver's failure is classified at the `mariadb/` boundary into a phase, a host,
a port and a classification, and the original value is dropped; `#[from]` is
not used on `sqlx::Error`. A raw driver error therefore has no route to any
stream, because it has no home in the value that reaches one. The template
engine is the deliberate asymmetry: `FR-ERR-011` requires "the chain of
underlying template-engine errors", so a render variant carries that chain, and
`FR-GLOB-018` forbids the **driver** error alone.

**Rejected.**

- **Per-module enums composed by `From`,** for the reason above; and additionally because `FR-ERR-034` requires the `cause` to name an instance rather than a category, so each conversion would have to carry the instance forward through every layer, which is where instances are lost.
- **`anyhow::Error` in the library.** The project fixes `thiserror` there, and an opaque error cannot carry the per-code obligations of `FR-ERR-034` or the exhaustive match `exit_code` depends on.
- **An exit code stored as a field on the error.** It makes two variants able to disagree with the table by construction, and it moves the assignment from a compiler-checked match to a value someone writes at each construction site.

**One observation for `technology-stack`, answered on 2026-09-21.** With
`main.rs` reduced to calling the library, reading `exit_code`, and returning,
the binary has no dynamic error to carry, so `anyhow` earns nothing under the
dependency budget. Whether it stays was a dependency question for that document,
and [`OD-32`](#od-32--anyhow-in-the-shipped-graph) settles it: the crate is
removed, and the correction to the coordination document was prepared there and
applied by the user on 2026-09-22, this folder never editing that file — as
`OD-09` did for `toml_edit`. Nothing in this entry depended on the answer, and
nothing in it moves.

**Unblocks.** `interfaces`, `architecture`.

---

## OD-07 — Help: the parser's renderer, or `tpl`'s own

**Status: settled.**

**Decision.** **`tpl` renders all seven sections itself**, with
`disable_help_flag` and `disable_help_subcommand` set on the parser. `clap` is
kept for parsing and for the tree introspection `FR-HELP-021` requires.

**Rationale.** Help is contract: `FR-HELP-006` fixes seven sections in a fixed
order, two of which — `EXAMPLES` and `EXIT CODES` — no argument parser
generates; `FR-HELP-009` and `FR-HELP-010` fix an 80-column layout with the
breaks written into the text, forbid reading `COLUMNS`, and forbid reflow; and
`FR-HELP-002` with `BR-HELP-001` require `tpl help <path>` and
`tpl <path> --help` to be **byte-identical at every depth**. Rendering it in
`tpl` makes all four properties the project's own to hold. `FR-HELP-022`
already requires the examples and exit codes to come from a typed table
indexed by command path, so the renderer has its source.

**Rejected.** `clap`'s `help_template` with `after_help` carrying the two extra
sections. It makes a contract output depend on the parser's renderer staying
byte-stable across versions — a dependency upgrade could then change a
contract output with nothing in this project having changed. It is the same
argument `FR-CONF-037` makes against inheriting a driver's TLS default.

**Composition.** `clap`'s `color` feature is on by default and must be off:
`NFR-DET-004` forbids an ANSI escape sequence on either stream. Its terminal
wrapping lives behind the optional `wrap_help` feature, which is left off (clap
feature-flags documentation, clap 4.6.6, verified 2026-09-10). The parser's
error messages are the neighbouring subject and were settled separately, in
[`OD-08`](#od-08--the-parsers-own-diagnostics), which this decision does not
reach.

**Consequence, found in implementation and recorded 2026-09-15.** Turning the
parser's own help flag off makes `-h`, `--help`, `-V` and `--version` ordinary
global arguments of the tree, and the parser validates **required arguments
before** it reads one. At the fourteen required operands the tree declares over
thirteen leaves, `tpl <node> --help` was therefore refused for a missing operand
— against `FR-HELP-002`, which makes the three help forms identical at every
depth, and against `FR-GLOB-019` and `FR-GLOB-020`, which give both flags an
outcome at every node. The mechanism that resolves it, and the reason the
declarations are left untouched, are
[interfaces.md](interfaces.md#the-help-surface)'s. Nothing in this decision
moves: the alternative is the parser's own help flag, which this entry rejected
for four reasons that stand.

**Rejected, with its ground unverified here.** Declaring each required operand
as required *unless* one of the two flags is present, through the parser's own
`required_unless_present_any`. The task that resolved the defect records that
the parser does not produce the behaviour under this tree; the attempt left no
artefact in the repository, so this entry records the rejection and **does not
assert the reason**. What is verifiable, and is stated in `interfaces.md`, is
the mechanism that was kept and what it leaves untouched.

---

## OD-08 — The parser's own diagnostics

**Status: settled.**

**Decision.** **Intercept `clap::Error` and re-render.** The parser keeps
`error-context`; `suggestions` and `color` are turned off; `clap`'s own
renderer is never invoked, so no byte it produces reaches a caller.

| clap feature | State | Why |
|---|---|---|
| `error-context` | **on** | It is the only source of the token `FR-ERR-034` row `64` obliges the `cause` line to name |
| `suggestions` | **off** | `FR-ERR-019` and `FR-ERR-020` fix the suggestion rule — at most three, within an edit distance of two, ordered by distance then by name — while `FR-ERR-039` fixes the measure and `OD-20` how it is computed. A second candidate generator with different rules would be dead weight whose output is discarded |
| `color` | **off** | `NFR-DET-004` forbids colour and every ANSI escape sequence on stdout and on stderr, "under any circumstances", and names removing colour as what "lets the argument parser be built without its colour support" |
| `wrap_help` | off (default) | `FR-HELP-009` and `FR-HELP-010` put the line breaks in the text and forbid reading `COLUMNS`. Settled in `OD-07` |

Feature names and their documented descriptions: clap feature-flags
documentation, clap 4.6.6, verified 2026-09-11 — `error-context` is "Include
contextual information for errors (which arg failed, etc)" and `suggestions`
"Turns on the `Did you mean '--myoption'?` feature".

**Why interception rather than local production.** `FR-ERR-034` row `64`
obliges the `cause` line to name "the token rejected as written, and why it was
rejected: the unknown command or flag, the value that did not conform together
with the type expected, or both members of the mutually exclusive pair". With
`error-context` off, `clap::Error` carries a kind and nothing else, so `tpl`
would have to recover the token by re-reading `argv` — parsing untrusted input a
second time, by a second set of rules, to answer a question the parser already
answered. With the feature on, the token is read from typed context:
`clap::error::ContextKind` is "available only when the `error-context` crate
feature is enabled" and carries `InvalidArg` ("the cause of the error"),
`InvalidValue` ("rejected values") and `PriorArg` ("existing arguments") among
its variants (docs.rs `clap::error::ContextKind`, clap 4.6.6, verified
2026-09-11).

**Why this does not repeat the dependency `OD-07` rejected.** That entry
refused making a contract output depend on the parser's renderer staying
byte-stable. Nothing here depends on rendered text: `ErrorKind` and
`ContextKind` are typed API, checked by the compiler, and the four lines are
composed by `tpl`. What remains is an API-level dependency on **which** context
kinds clap populates for a given kind of failure, which no documentation
promises. `verification` therefore owes one test per `ErrorKind` that `tpl`
maps, asserting the four lines it produces — the same discipline `FR-HELP-002`
already imposes on the help surface through snapshots.

**Discharged 2026-09-15**, at commit `f8f335d`. One test exists per mapped kind
and one for the wildcard arm, and each asserts the kind **against this tree**
before asserting the four lines, so a clap version that reclassified a refusal
fails where the refusal is mapped rather than silently producing the wildcard's
message.
[verification.md](verification.md#the-parsers-mapping-one-test-per-kind) names
them.

**`ContextKind` is `#[non_exhaustive]`** (same source and date), and so is
`ErrorKind` (docs.rs `clap::error::ErrorKind`, clap 4.6.6, verified
2026-09-15), so **both** matches of the mapping carry a wildcard arm by force of
the language. Neither arm produces a code other than `64`, so no unmapped kind
and no unread context can move a caller onto a different branch.

**Amended 2026-09-15 — the wildcard arm names no token where the parser named
none.** This entry said the arm produces a `64` whose `cause` **names the
token**. It cannot always do so, and the reason is the parser's: clap populates
no context this crate reads for `ErrorKind::InvalidUtf8`, which is the one kind
that reaches the arm from a vector a caller can write. `tpl --timeout $'\xff'
version` therefore exits `64` with `cause: the invocation was rejected while it
was being parsed, and the parser named no token of it` — observed against the
binary at commit `f8f335d`, 2026-09-15. The arm names the token wherever the
refusal carries one, reading `InvalidArg`, then `InvalidSubcommand`, then
`InvalidValue`.

`FR-ERR-034` row `64` obliges the token to be named *wherever one is
available*, and none is available here, so the row is met as written rather
than narrowed. **The degradation runs in the safe direction**: the token that
was rejected is a sequence that is not valid UTF-8, so naming it would put a
lossy rendering of bytes the caller supplied into a message, and withholding it
costs a less specific `cause` and nothing else. What the entry decided is
unchanged — the arm is `64`, always.

**`FR-CLI-014` is answered without the parser's context at all.** A flag that
carries a single value is declared with `ArgAction::Append`, and `tpl` rejects
a second occurrence itself, naming **both values**. The alternative is not
available: `ArgAction::Set` "will result in an `ArgumentConflict`" on a second
occurrence, which yields the `64` but reports a conflict between arguments
rather than the two values the requirement demands (docs.rs `clap::ArgAction`,
clap 4.6.6, verified 2026-09-11). Declaring the flag as repeatable and refusing
the repetition locally is what puts both values in hand — and it makes
`FR-CLI-014` independent of anything clap chooses to place in its context.

**Rejected.**

- **Turning `error-context` and `suggestions` off and producing every diagnostic in `tpl`.** It forfeits the token that `FR-ERR-034` row `64` requires, and buys back only a feature flag. Recovering the token would mean a second parse of `argv` inside `tpl`, on the path `FR-ERR-006` places first in the validation order, where a disagreement between the two parses is a wrong `cause` line for a correctly rejected invocation.
- **Letting `clap` render its own errors.** `FR-ERR-008` fixes four labelled lines and `FR-ERR-033` makes them the whole of what a caller receives; clap's message shape is not that shape, and adopting it would make a contract output move when the parser's renderer moves. This is the argument `OD-07` already made, and it applies unchanged.
- **Keeping `suggestions` on and using clap's candidates.** `FR-ERR-019` fixes the count, the distance and the ordering, and `FR-ERR-023` refuses a candidate outside `[A-Za-z0-9_]{1,64}` entirely. A candidate set produced by another rule would have to be filtered and re-ordered into the specified one, so the feature would compute a set that is then discarded.

**Composition.** `FR-ERR-024` escapes every value interpolated into a message,
including the argument vector, and the token this entry recovers is such a
value: it reaches the reader through the renderer of `OD-06`, escaped, and never
through clap. `FR-GLOB-018` forbids the argument **vector** on a diagnostic
stream; one rejected token is not the vector, and `FR-ERR-034` row `64`
requires it.

**Unblocks.** `interfaces`.

---

## OD-09 — TOML: the read path and the write path

**Status: settled. Applied to `CLAUDE.md` on 2026-09-11. Amended on 2026-09-17,
when the two paths were built: the read path is `toml`'s **document tree**,
`toml::de::DeTable`, and not a `serde` derive.**

**Decision.** **Read path the `toml` crate; write path `toml_edit` 0.25**, which
preserves comments, spacing and the relative order of items.

**Rationale.** Six requirements make format preservation a functional need, not
a nicety. `FR-PROJ-017` and `FR-PROJ-018` put a commented-out `[database.*]`
entry in the `.cfg` that `tpl init` writes, so that "the correct shape is in
front of the reader without a trip to documentation"; `FR-CFG-020` requires
`database update` to change the named fields "leaving the rest of the entry
untouched"; `FR-CFG-013` prints the file "literally". The `toml` crate is
serde-oriented and its own documentation points elsewhere for this — "For
format-preserving editing or finer control over output, see `toml_edit`" (`toml`
1.1.5+spec-1.1.0, docs.rs, verified 2026-09-10) — while `toml_edit`
"allows you to parse and modify toml documents, while preserving comments,
spaces *and relative order* of items" (`toml_edit` 0.25.13+spec-1.1.0,
docs.rs, verified 2026-09-10). Splitting the paths gives the read side a parser
whose only job is to yield the file as written, and keeps the comments, which
the write side must not destroy. *This sentence read "keeps the serde mapping,
which `FR-CONF-002`'s typed key space wants on the read side" until 2026-09-17;
the amendment below is why.*

**Amendment of 2026-09-17 — why the read path is not a `serde` derive.** Three
requirements ask the reader for facts a derive cannot produce, and each is a
`78` the caller has to act on.

| Fact required | Requirement | Why a derive cannot give it |
|---|---|---|
| The **name** of the offending key, with a nearest-match suggestion over the space | `FR-CONF-034` | The offending key is precisely the one no field is declared for. A deny-unknown-fields derive answers *this document does not fit*; the name is inside the error's rendered text, not a value the reader can suggest over |
| The **position** of the fault | `FR-CONF-035`, `FR-ERR-034` row `78` | A span survives only where the value that carried it does |
| The file printed **literally, with passwords redacted in place** | `FR-CFG-013`, `FR-CFG-021` | The redaction is spliced into the file's own bytes at the spans of the credential-bearing values. Every byte outside them — comments, key order, spacing — must reach the reader untouched, which a re-serialisation would not leave standing |

`toml::de::DeTable` supplies all three: it is declared
`pub type DeTable<'i> = Map<Spanned<DeString<'i>>, Spanned<DeValue<'i>>>`, so
every key and every value carries its own range; `DeTable::parse` has the
signature `pub fn parse(input: &'i str) -> Result<Spanned<Self>, Error>`; and
`toml::de::Error::span` is `pub fn span(&self) -> Option<Range<usize>>`,
documented as "the start/end index into the original document where the error
occurred" (docs.rs `toml` 1.1.6+spec-1.1.0, verified 2026-09-17). The validation the derive would
have performed is not lost: it is written out once, against the key space of
`project/config/keys.rs`, which the writer already has to consult for
`FR-CFG-009` and `FR-CFG-010`. So the choice is not *derive versus hand-written
validation* but *one validator or two*, and the rejection below — that
hand-written validation is more code on the path reading untrusted input — is
answered by the one that was going to exist either way. What the amendment
costs is that the typed document is built by hand; what it buys is that the two
callers of the key space cannot disagree about it.

**Rejected.**

- **`toml_edit` for both paths.** It carries no serde mapping either, and the read path would then hold a document type built for editing: mutable, formatting-aware, and larger than the reader needs. The write path's obligations are the reverse of the read path's, and one type serving both is the shape in which a reader acquires a way to write.
- **A `serde` derive on the read path**, which is what this entry decided until the amendment above. It cannot answer the three questions in the table, and two of the three are the content `FR-ERR-034` obliges a `cause` line to carry.
- **`toml` alone**, which is what `CLAUDE.md`'s stack table named until this was applied. The first `tpl cfg set` would delete the commented example that `FR-PROJ-018` requires the file to carry, so a stated requirement would stop holding on the second invocation, silently.

**What was applied.** Task #20, at commit `ee7363d` of 2026-09-11: the `Stack`
table of `CLAUDE.md` splits its configuration row into `toml` with `serde` on
the read path and `toml_edit` on the write path, and cites this entry by file
and identifier. That table is an architecture decision by that file's own
terms, so the change was prepared for the user rather than made here and
applied under the authorisation of that day, which the commit records as not
generalising.

---

## OD-10 — Cache filenames, and the case collision

**Status: settled.**

**Decision.** A cached object's path is the **catalogue object kind plus the
literal object name**, exactly as `FR-CACHE-001` and `FR-CDOC-014` already
shape them: `tables/<name>.json`, `views/<name>.json`,
`routines/<kind>.<name>.json`. **No encoding layer and no hash.**

~~On a **case collision** — two objects of one kind whose names differ only in
case, mapping to one path on a case-insensitive filesystem — `cache refresh`
**detects it and fails, naming both objects**.~~ **Superseded on 2026-09-23**
by the user's decision for rmp `#254`, below; kept as history.

**Rationale.** `FR-CDOC-014` already fixes one of the three path forms
literally, so the other two follow the same rule and there is one naming rule
rather than two. The collision is real rather than theoretical: MariaDB permits
two such objects on a case-sensitive filesystem, and two of the four targets of
`NFR-PERF-018` are macOS, whose default filesystem is case-insensitive.
Detecting and failing is the only outcome consistent with the stance the
functional corpus takes everywhere else — a wrong answer wearing the appearance
of a right one is the failure it works hardest to prevent, per `BR-CDOC-002`
and `BR-SEM-004`. Silently serving one object's bytes under the other's name
would be exactly that.

**Rejected.**

- **A disambiguating suffix** on one of the two colliding names. It creates two naming rules where `FR-CDOC-014` fixed one, and the suffix would then have to be derivable by every reader of the cache.
- **Documenting the limitation** and carrying on. MariaDB permits such schemas on Linux, so the failure would be reachable and silent, which is the class of defect this project refuses.

**Amended on 2026-09-23, against the forty-second edition.** Two read-side rules
now sit beside the decision, both from `FR-CACHE-033`, and neither changes the
file naming. A named read whose file holds another object — a name differing
byte for byte, or a routine of the other kind — is a miss, which closes the
false answer a case-folding filesystem produced. An object file that is not a
regular file is a miss and is never read through: `read_object()` in
`src/cache.rs` takes `lstat`, opens the file, and compares device and inode
with what it inspected. *Rejected — `O_NOFOLLOW` on the open.* The flag's value
differs between the supported targets, the crate would carry it only through a
further `rustix` feature, and an open that follows no link still opens a FIFO,
which blocks, where the inspection refuses it first. **This rejection is
reversed on 2026-09-24** by `OD-24`'s amendment: `rustix`'s `fs` feature
supplies the flag per target, and `O_NONBLOCK` beside it keeps a FIFO from
blocking the open. Implemented the same day in `src/at.rs`: an object file is
opened with both flags and typed on its descriptor, which replaces the `lstat`
and the device-and-inode comparison.

**The collision outcome is superseded, 2026-09-23, by the user's decision for
rmp `#254`.** The fix is the stored-name check above and nothing else: no
collision is detected and none fails. `FR-CACHE-033` governs and accepts that a
collision leaves the collection not whole — two names sharing one file leave the
directory's count short, so `replace()` in `src/cache.rs` records the collection
as not whole — and that either colliding object can miss on every invocation.
*Rejected — detect-and-fail at `tpl cache load`*, the outcome this entry first
prescribed: it adds a failure no requirement states, where the miss already
removes the false answer. *Rejected — a collision-free file-name encoding*: it
would change the naming arrangement `cache_format` versions and add a second
naming rule where `FR-CDOC-014` fixes one, which `FR-CACHE-033` declines.

**Still to record in `data-model`.** Whether a cached object file carries the
envelope of `FR-OUT-024` or a bare object. The `.json` suffixes of
`FR-CDOC-001` and `FR-CDOC-014` fix the encoding as JSON; they do not fix the
outer shape.

---

## OD-11 — The scope of the async runtime

**Status: settled. Recorded in [`ADR-005`](../adr/adr-005-async-runtime-scope.md).**

**Decision.** The process is synchronous, and the runtime is confined to the
boundary `ADR-005` names.

The rationale, the requirements it serves, and the options rejected are
recorded in `ADR-005` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). The driver decision this entry
composed with is [`ADR-003`](../adr/adr-003-database-driver.md), which also
records the unexplained musl blocking cost this entry mentioned; it bears on
`OD-12`.

**Amended on 2026-09-23 — when the runtime ends.** `ADR-005` fixes where the
runtime lives and says nothing of when it ends; `FR-RND-040` now requires it
gone before any render starts. `Session::close` sends the protocol's quit,
releases the connection, and **drops** the runtime, which waits for all
spawned work to stop (docs.rs, `tokio::runtime::Runtime`, tokio 1.53.1,
*Shutdown*, consulted 2026-09-23). Connections and runtimes are counted per
thread from creation to drop, and `mariadb::quiescent()` reads the count.
Nothing here conflicts with `ADR-005`: the runtime is still built lazily inside
`mariadb/`, one per connection. *Rejected — `shutdown_timeout`.* On expiry it
leaks the work and the threads that did not stop, which would be a runtime alive
while the template evaluates. *Rejected — a process-wide count.* It is the same
count in the binary, which runs one command on one thread, and wrong in the
suite, where another test's connection on another thread is not this
invocation's; the runtime is current-thread and the counted values are not
`Send`, so the per-thread count is exact.

---

## OD-12 — How six phase deadlines are enforced

**Status: settled.** The observation it owed `specification-manager` landed in
the ninth edition; the behaviour it owed a verification was **observed on
2026-09-18**, when the connection was built, and the entry is discharged below
with three corrections. No obligation of this entry stands. It was flagged as
the entry likeliest to prove a requirement unmeetable. It did not: five of the
six phases separate cleanly, and the sixth pair separates in the report rather
than in the call — on two discriminants and not on one.

**Decision.** Three mechanisms, chosen by what the phase is waiting on.

| Phase | Bounded by | Deadline |
|---|---|---|
| DNS resolution | `tokio::time::timeout` around `tokio::net::lookup_host`, performed by `tpl` before the driver is called | the connection deadline |
| TCP connect **and** TLS handshake | one `tokio::time::timeout` around the driver's `connect_with`, which receives the **configured host** and not the address the resolution produced — third correction below | the remainder of the connection deadline |
| Catalogue query | `tokio::time::timeout` around each query | `core.query_timeout` |
| `password_command` | a reader thread draining the child's standard output and a polling loop in the parent, which kills the child's process group and reports the deadline — amended 2026-09-23, below | `core.password_timeout` |
| Render | a watchdog thread that writes the `65` diagnostic and exits the process; since 2026-09-23 it also observes the render memory limit, below | `core.render_timeout` |

The **connection deadline** is one instant, set at `core.connect_timeout` from
the start of connection establishment and shared by the three connection
phases. Every deadline above composes with the overall budget as `FR-GLOB-012`
requires: a phase ends at the first of its own deadline and what remains of
`--timeout` measured from process start.

**How the phases are separated.**

1. **DNS is separated because `tpl` performs it.** This is obliged rather than chosen: `FR-CONF-005`'s note and `NFR-PERF-018`'s accepted cost both require the `cause` line to say that a name did not resolve rather than that a host refused a connection, and that distinction cannot be recovered from a driver call that resolves internally. `MySqlConnectOptions` exposes `host` and `port` and no pre-resolution hook (docs.rs `sqlx::mysql::MySqlConnectOptions`, sqlx 0.9.0, verified 2026-09-11), so the driver resolves whatever it is given and the separation costs a second lookup. This entry proposed paying for it by handing the driver the address instead; the third correction below records why that was overruled.
2. **TCP connect and TLS handshake are not separable in the call.** `MySqlConnectOptions` has **no method that accepts an already-connected stream or socket**; `socket()` takes the path of a Unix socket and changes the transport rather than supplying a connection (same source and date). One call therefore covers both phases, which is what `BENCHMARKS.md` measured when it attributed 44.18 ms to `connect_with` as a whole.
3. **They are separated in the report, by the driver's own discriminants — two of them, not one.** `sqlx::Error::Tls` is documented as "Error occurred while attempting to establish a TLS connection" and `sqlx::Error::Io` as "Error communicating with the database backend" (docs.rs `sqlx::Error`, sqlx 0.9.0, verified 2026-09-11). The phase named in the `cause` line is derived from the error value and not from the call site; **the variant alone is not the discriminant**, and the first correction below records what is.
4. **Catalogue query, `password_command` and render are separate calls** and need no argument.

**What the `cause` line may carry.** `FR-ERR-034` row `69` requires it to name
the phase, the host and port attempted, and what that phase returned, while
`FR-GLOB-018` forbids the raw driver error on any diagnostic stream at any
level. Both hold only if the driver's error is **classified and re-worded**
rather than rendered: the error value carries the phase, the host, the port and
a classification of the failure, and the driver error's `Display` is never
reached. `OD-06` makes that structural by refusing the driver error a home
inside the error type.

**How the `password_command` child is bounded, amended on 2026-09-17 when it
was built.** This entry gave the mechanism as a timer thread that kills the
child while the parent reports the deadline. What is built inverts the two
roles: the **parent** watches the deadline, in a loop that sleeps a millisecond
at a time, and the thread does the one thing the parent cannot do while
watching — drain the child's standard output.

The reason is a third obligation the row did not account for. `FR-CONF-031`
caps what is read from the child at 4096 bytes and requires a child that writes
more to be terminated, and a cap is only worth what the read behind it is: a
child writing more than the pipe buffers **blocks on its own write** until
something drains it, so a parent that waits on the child while nothing reads the
pipe waits for a child that is itself waiting. The drain therefore has to run
while the deadline is watched, and one of the two has to be on another thread.
Putting the drain there rather than the timer costs nothing and buys the cap:
the reader stops one byte past 4096, which is enough to know the cap was passed
and is the whole of what is ever held in memory, so the child is killed rather
than the credential truncated — and a truncated password would be sent, refused,
and reported as a `77` naming the credentials, which is a wrong diagnosis of a
configuration fault.

Three consequences follow, and each is a property of the built path.

- **A read that fails part-way yields nothing rather than a prefix**, for the same reason: a prefix is a truncated credential wearing the appearance of a whole one, and the child's own exit status is the better diagnosis (`FR-CONF-031`, `FR-CONF-033`).
- **A bound already spent stops the child from being started at all**, rather than starting it and killing it immediately (`FR-GLOB-012`).
- **The child is reaped on every refusing path** — the cap, the deadline, a failure to inspect it — so nothing is left running behind the process (`FR-CONF-031`).

**Why a thread and not a task.** The runtime of
[`ADR-005`](../adr/adr-005-async-runtime-scope.md) is scoped to `mariadb/` and
is built only when that module is reached, so an invocation that resolves an
entry without connecting — every `cfg` subcommand but the connectivity one — has
no runtime to spawn a task onto. Bounding the child with a runtime timer would
either start a runtime for a command that connects to nothing, which
`NFR-PERF-006` refuses, or move the runtime outside `mariadb/`, which
`ADR-005` refuses. The render's timer thread below is the same constraint met at
the other end of the process.

**How a render is bounded.** A **timer thread**. The render runs on the calling
thread; a thread created immediately before it waits on a channel with
`std::sync::mpsc::Receiver::recv_timeout`, whose signature is
`recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError>` and
which returns `Err(RecvTimeoutError::Timeout)` when the duration is exceeded
before a message arrives (Rust standard library documentation,
`std::sync::mpsc::Receiver`, verified 2026-09-11). The render signals the
channel when it completes. If the deadline arrives first the timer
writes the four labelled lines of `FR-ERR-008` for a `65` — naming which
deadline expired and its resolved value, per `FR-ERR-034` row `65` — and
terminates the process with status `65`.

Three requirements make this admissible rather than merely convenient.
`FR-RND-034` already admits that stdout carries at most one incomplete result
when a render fails, so an interrupted render does not violate a promise about
stdout. `NFR-DET-001` keeps stderr outside the contract, so the timer writing
to it while the render writes to stdout interleaves nothing that is contract.
And `FR-GLOB-013` requires the exit code of the **phase in progress**, which a
timer holding the phase it was created for supplies directly; the same
construct therefore realises the overall budget of `FR-GLOB-011` when
`--timeout` is supplied.

**Why a thread is not the speculative parallelism the project forbids.**
The rule refuses concurrency adopted for speed without a measurement. Neither
thread performs work of the invocation and neither makes anything faster: one
bounds a computation with no interruption point, the other drains a pipe that
must be drained for the bound to mean anything. Each is created only on the path
that needs it, so the four commands of `NFR-PERF-005` — `tpl init`, every form
of `help`, every form of `version` — create no thread, open no socket and read
no file, exactly as before; and a `cfg` command that resolves no entry creates
none either.

**Rejected.**

- **A pre-flight TCP connect by `tpl`, to attribute the connect phase exactly.** It would resolve the ambiguity of point 3 outright, at the price of a second connection per invocation, which `NFR-PERF-004` forbids: "One invocation SHALL open at most one connection."
- **Reporting TCP connect and TLS handshake as one `connect` phase.** `FR-ERR-034` row `69` enumerates four phases and obliges the `cause` to name the one that failed; a `cause` reading "connect failed" would be equally true of two different failures, which the same requirement forbids in its own words.
- **`minijinja`'s `set_fuel`, as the render deadline.** *Qualified on 2026-09-23: this rejects fuel as the mechanism of a deadline stated in seconds, and still stands; fuel is now used, beside the deadline, as the separate count `FR-RND-036` requires.* Fuel is an instruction budget consumed per instruction, gated behind the `fuel` crate feature (docs.rs `minijinja::Environment`, verified 2026-09-11). `FR-CONF-002` states every deadline in **seconds** and `FR-GLOB-012` composes them with a wall-clock budget measured from process start, so a fuel figure would have to be calibrated into seconds — per target, since `NFR-PERF-012` forbids carrying a figure from one target to another. A budget that has to be re-derived on four targets to mean what a requirement already states in seconds is not the mechanism. `set_recursion_limit` stays at its documented default of 500, which bounds recursion and not time.
- **A cooperative clock check inside the output writer.** *Qualified on 2026-09-23: rejected as the deadline, and still rejected; the writer now counts bytes for the separate output limit of `FR-RND-037`, and checks no clock.* It bounds a template that emits and not one that loops without emitting, so it would bound some renders rather than the render — and which ones would depend on the template, which is caller input.
- **Rendering on a worker thread while the calling thread waits with `recv_timeout`.** The same construct inverted. It moves the hot path off the calling thread for no gain and puts the writer on the thread that is abandoned.

**The observation owed to `specification-manager`, discharged by the ninth
edition.** It reported that `FR-CONF-005` named six phases, `FR-CONF-002`
supplies four `[core]` timeout keys, and `FR-CONF-004` resolved "each phase
deadline from the `[core]` key for that phase", which left the three connection
phases with no unique referent. Two readings were available: three independent
timers of `connect_timeout` each, whose sum is three times the key the caller
set; or one budget of `connect_timeout` shared by the three, which is what the
key's name states. **This entry took the second**, because a caller who writes
`connect_timeout = 10` is stating how long connecting may take, and because the
first reading makes the configured value unable to bound the thing it is named
after. `FR-CONF-005` now carries the phase-to-key mapping and the shared
connection budget in its own text, and `FR-CONF-004` points at that mapping
instead of implying a key per phase — so the shared connection deadline above is
the corpus's rule rather than this entry's reading. Nothing in the decision
changes.

**The behaviour owed a verification was observed on 2026-09-18, when the
connection was built. It discharges the obligation and corrects this entry in
three places.** What was owed: whether a TLS handshake failure reaches `tpl` as
`sqlx::Error::Tls` — for an untrusted certificate, for a name mismatch, and for
a server that offers no TLS — which sqlx's documentation does not state, and on
which `FR-ERR-034` row `69` depended. It does for one of the two failures that
were produced, and not for the other. Nothing below moves the decision: the
three mechanisms, the shared connection budget and the six phases are
unchanged.

**First correction — the variant alone is not the discriminant.** Observed
against `scripts/mariadb/` on 2026-09-18, on the `11.8` server and on the
server that offers no TLS, at the driver version
[`ADR-003`](../adr/adr-003-database-driver.md) pins:

| Failure | What the driver returned |
|---|---|
| A server offering no TLS, under `required` | `sqlx::Error::Tls` |
| A certificate the trust material does not vouch for, under `verify-ca` and under `verify-identity` | `sqlx::Error::Io`, **of kind `InvalidData`** |

The second is the driver completing the handshake's own I/O and propagating what
the TLS implementation reported. `src/mariadb/fault.rs`, lines 234-235, reads
the **variant and that kind** and nothing else (read 2026-09-21), and point 3
above is corrected to match: no kind the operating system produces for a socket
is `InvalidData`, and the driver reports a malformed protocol packet on a
variant of its own, so the two discriminants separate the two phases without
reading a message. That is what keeps `FR-ERR-034` row `69` satisfiable while
`FR-GLOB-018` holds. The trust material the two validating modes use is
[`ADR-002`](../adr/adr-002-tls-mode-mapping.md)'s and is not restated here.

**Second correction — the fixture claim was overstated, and one of the three
modes is reasoning rather than observation.** This entry said the fixture
provides all three failure modes and presents all three. **Two were produced**,
and they are the two in the table. The third — a certificate that does not name
the host — was **not observed**, because the fixture's certificate names every
spelling of the loopback by which a test can reach it
([`OD-22`](#od-22--the-test-harness-and-the-fixture-certificate)), so no
mismatch can be raised against it. It is classified with the first, on the
ground that it is the same rejection by the same implementation reported through
the same call; **that is reasoning, and `src/mariadb/fault.rs` writes it down as
reasoning** under a bounded claim of its own. The two are kept apart rather than
flattened into one list: what a claim of this kind costs is knowable only while
it is still marked as one.

**Third correction — the driver is handed the configured host, and this entry's
instruction was overruled.** The mechanism table and point 1 both said the
driver receives the address the resolution produced. The connection work of
2026-09-18 deliberately did not do that, and `src/mariadb/connect.rs` records
why (read 2026-09-21): `sqlx` takes the TLS server name from `options.host`, so
a driver handed an IP address validates **the IP address** under
`verify-identity` — the default mode of `FR-CONF-013` — and fails against every
certificate that names a host. `FR-CONF-038` fixes what that mode must do, so
the instruction and the requirement could not both stand and the requirement
governs. The resolution is still `tpl`'s and still runs first, but its result is
used for **phase attribution alone**: a name that yields no address is
`FR-ERR-001`'s `69` naming DNS, which is the whole of what point 1 needs from
it. The cost is a second lookup inside the driver — a second resolution and not
a second connection, so `NFR-PERF-004` is untouched.

**Amended on 2026-09-23, against the forty-second edition — the child's group,
and the render's watchdog.** The decision stands: three mechanisms, the shared
connection budget, six phases. Two mechanisms changed under it.

*The `password_command` child.* `FR-CONF-028` now ends the phase only when the
child has exited **and** its standard output has reached end of file, and
terminates the child's whole process group; `FR-CONF-031` terminates the same
group at the cap. As built in `src/project/password.rs`:

- The child is started with `process_group(0)`, so its pid is the group's id.
- Its exit is observed with `waitid` under `EXITED`, `NOHANG` and `NOWAIT`,
  which leaves it waitable; it is reaped only after `kill_process_group(pid,
  SIGKILL)`, or on success after the pipe has closed. An unreaped child holds its
  pid, so the group id cannot be handed to another process before the kill. A pid
  of `1` or one that does not fit is never signalled as a group.
- On the deadline and the cap the reader thread is **dropped rather than
  joined**: a descendant that has left the group may still hold the pipe, and
  the invocation must not wait for it. The thread ends at that pipe's end of
  file or with the process.

*Rejected — reaping the child as soon as it exits, as the loop did before.* A
reaped child frees its pid, so a group kill at the deadline could reach another
process that took the same id; and a read waiting for end of file after the
exit waited past every deadline when a descendant held the pipe, which is
finding SEC-01 of `SECURITY-AUDIT.md`. *Rejected — joining the reader after the
kill.* It hangs the invocation on a descendant that left the group, which
`FR-CONF-028` forbids.

*The render.* The timer thread is now the watchdog of `bounded()` in
`src/cli/render.rs`. Where the binary installed a heap counter it wakes every
10 ms and at the deadline — the interval
[architecture.md](architecture.md#the-render-bounds) fixes under the delegation
of [`ADR-011`](../adr/adr-011-render-memory-accounting.md) — and ends the
render with the `65` of `FR-RND-039` when the count is above the limit.
**A render abandoned under `FR-CACHE-039` is no longer excused**: until
2026-09-23 the watchdog stood down when the render had reached a miss, and
`FR-RND-038` now keeps all four bounds on it until it returns. Render fuel and
the output limit end a render from inside it, so the first bound crossed is the
one reported. `bounded()` also refuses to start a render while a connection or
a driver runtime is alive (`FR-RND-040`).

**One consequence recorded for `architecture`.** `tokio::net::lookup_host` is
gated behind tokio's `net` feature (docs.rs `tokio::net::lookup_host`, tokio
1.53.1, verified 2026-09-11) and resolves through the platform resolver. The
deadline bounds `tpl`'s **wait**, not the resolver's work: a resolution that
outlives its deadline is abandoned and ends with the process. That is the
honest statement of what a deadline on DNS can be, and it is what
`NFR-PERF-018`'s musl note already assumes when it says a name simply does not
resolve.

**Unblocks.** `architecture`, `interfaces`, `quality-attributes`.

---

## OD-13 — The engine pin, and `minijinja-contrib`

**Status: settled. Recorded in [`ADR-001`](../adr/adr-001-template-engine-pin.md).**

**Decision.** Pin the stable line of `minijinja` named by `ADR-001`, and retain
`minijinja-contrib` on the same line. Its filters are therefore **group 3** of
`FR-ENV-019` — available, and guaranteed by nobody.

The version number, the rationale, the options rejected, and the two deliberate
shadowings of engine built-ins are recorded in `ADR-001` and are **not restated
here**, per rule R3 of [`docs/adr/README.md`](../adr/README.md). `FR-ENV-003`
requires the pin to live in an architecture decision record and to be cited from
there; `ADR-001` is that record.

---

## OD-14 — Which undefined behaviour the engine is configured with

**Status: settled. The observation this entry owed was made on 2026-09-21.**

**Decision.** `UndefinedBehavior::Strict`.

**Rationale.** `FR-SEM-012` requires a render to fail when a template reads a
field that does not exist, and `FR-PRIV-016` states the cost of that rule in
its own text: `{% if table.restricted %}` "fails on every complete table".
Of minijinja's four variants — `Lenient` (the default), `Chainable`,
`SemiStrict`, `Strict` — only `Strict` fails a truthiness test on an undefined
value; `Lenient` and `SemiStrict` both permit `{% if missing %}`, and
`Chainable` permits the attribute access as well (docs.rs
`minijinja::UndefinedBehavior`, verified 2026-09-10). `Strict` is therefore the
only variant consistent with the consequence the functional corpus already
accepted, and `FR-SEM-012` is load-bearing for three requirements elsewhere —
`FR-CTX-034`, `FR-PRIV-016` and `FR-ENV-017` — so it may not be weakened by
the choice of setting.

**Rejected.** `Lenient` and `SemiStrict`, which contradict `FR-PRIV-016`'s
stated cost; `Chainable`, which permits the very field access `FR-SEM-012`
fails and would turn a misspelled field into an empty string in a generated
file.

**Observation made on 2026-09-21, and it is the opposite of what this entry
expected.** A **defined** `null` does **not** fail under `Strict`, and the
engine writes the word `None` for it — which `FR-SEM-011` forbids by name.
`Strict` governs *undefined* values only, so it decides nothing at all about a
defined `null`.

| Observed | What follows |
|---|---|
| A defined `null` renders, and the engine's own rendering of it is `None` | `FR-SEM-010` and `FR-SEM-011` are satisfied by the output formatter `render/` installs, and by nothing in this setting |
| A field that does not exist fails before any formatter is reached | `FR-SEM-012` and `FR-SEM-013` are this setting's, as this entry decided, and the formatter cannot weaken them |

**The observation is a test, not a citation, and it stands as long as the test
does.** In `src/render/engine.rs`,
`tests::fr_sem_011_the_formatter_is_what_keeps_a_null_from_reaching_the_output`
renders a defined `null` through an engine carrying this setting **and nothing
else**, and asserts what the engine writes; and
`tests::fr_sem_012_the_strict_behaviour_is_what_fails_a_field_that_does_not_exist`
asserts the other half. A release of the pinned line that changed either fails
the suite rather than passing quietly, which a version-dated citation would not
have done.

**What it costs, and what it does not.** No requirement is contradicted: the
built system emits the empty string for a `null`, and `FR-SEM-021` likewise
emits `true` and `false` where the engine's stock rendering is `True` and
`False`. What moves is **where** the three requirements are met — the formatter
of [architecture.md](architecture.md#the-render-component), not the
undefined-behaviour setting — and one consequence follows, which is why the
observation is recorded here rather than left in the tests that make it: the
formatter is load-bearing, and removing it as a restatement of what the engine
already does would break `FR-SEM-010`, `FR-SEM-011` and `FR-SEM-021` at once.

**Discharged.** This entry owes nothing further, and no passage of this folder
is bounded by it.

---

## OD-15 — The template loader

**Status: settled.**

**Decision.** **`tpl` writes the loader.** `minijinja::path_loader` is not used.
One function in `render/` resolves a template name to a path, and it is the
**only** resolution in the crate: the loader closure calls it, and so do
`template list`, `template show` and `template path`, which resolve paths
without the engine.

The resolution is one sequence, and it is stated here because five requirements
constrain it and none of them may be applied twice:

1. Reject a name that is not a template under `FR-TMPL-004` and `FR-TMPL-005` — a name whose file does not end in `.jinja` is not a template and is not resolved.
2. Join the name to the template root of `FR-TMPL-023`.
3. Canonicalise, per `FR-TMPL-025`.
4. Re-check the canonical path against the canonicalised root; an escape is `65`, per `FR-TMPL-026`.
5. Refuse a symbolic link at **every component of the name below the canonical root, taken in order**, refusing the first that is one, per `FR-TMPL-024`. Each component's own metadata is read rather than followed — `std::fs::symlink_metadata`, which "queries the metadata about a file without following symlinks" and "corresponds to the `lstat` function on Unix" (Rust standard library documentation, `std::fs::symlink_metadata`, verified 2026-09-11).
6. Open the path that was checked, and no other.

**Step 6 was amended on 2026-09-24, for rmp `#307`.** The path is opened
relative to the canonical root, one component at a time and following none,
with `O_NOFOLLOW` and `O_NONBLOCK`, and the type is read from the descriptor it
is then read through (`FR-TMPL-033`, `FR-SEC-027`). The calls are `OD-24`'s
`fs` feature; [security.md](security.md#template-containment) states the
property.

**Step 5 was the final component alone until 2026-09-21, and that was narrower
than the requirement.** `FR-TMPL-024` refuses a symbolic link inside
`.tpl/templates/` **without qualification**, and a link is an entry wherever it
sits in a name. Four facts record the widening, and the second bounds what the
narrow reading could have cost.

| Fact | What holds |
|---|---|
| **Why it changed** | A symlinked **intermediate directory** passes step 4, because its target is inside the root, and passes a check of the final component, because the final component is an ordinary file. One name was therefore renderable and unlistable: the listing never carried it — `FR-TMPL-024` keeps a symlinked entry out of the listing too — so the lookup and the listing disagreed about what a template is |
| **Nothing escaped containment** | Step 4 is unchanged and already refuses any chain that resolves outside the canonical root. What the narrow step admitted was a link whose target is **inside** the root, which reads a file the caller may already read under its real name. This closes a **divergence** from `FR-TMPL-024`, not a hole |
| **The root itself is not walked** | Step 4 canonicalises the root, so every component above and including it is already resolved and there is no link left there to find. The walk begins below it and covers exactly the name the caller wrote |
| **The cost** | N `lstat` calls for an N-component name, where one was made before. A template name is one or two components in practice, and every entry the walk reads was already read by step 3's canonicalisation of the same path |

The decision this entry made is unchanged — one resolution function, `tpl`'s
own, six steps in this order — and what moved is the breadth of one step. That
the requirement governs unqualified, rather than the narrower reading the step
had taken, was decided by the user on 2026-09-21.
[security.md](security.md#template-containment) carries the same widening in the
row that states the property.

**Why not wrap `path_loader`.** Wrapping runs `tpl`'s checks on one path and
lets the engine's helper resolve a second one from the same name, by a rule
that is not ours. Two consequences follow, and either is disqualifying. The
path that was **checked** would not be the path that is **opened**, which is the
shape of defect `FR-TMPL-025` exists to close by requiring the resolved path to
be canonicalised and re-checked. And the inner rule is not knowable: the helper
is documented to refuse templates that "start with a dot (`.`) or are contained
in a folder starting with a dot", and its documentation states **nothing** about
`..`, about an absolute path, or about a symbolic link (docs.rs
`minijinja::path_loader`, verified 2026-09-11). Silence is not a behaviour. A
containment property that `FR-SEC-017` names by exploit —
`ln -s ../.cfg .tpl/templates/leak.jinja` — may not rest on an undocumented
one.

The engine asks for exactly what a written loader supplies:
`Environment::set_loader` takes `Fn(&str) -> Result<Option<String>, Error>`, and
"once loaded, templates are cached, so the loader is invoked only once per
template name" (docs.rs `minijinja::Environment`, verified 2026-09-11) — which
is also what the project's rule that each template is parsed once per process
requires.

**How the two codes stay apart.** `FR-ERR-028` makes a **missing template**
`66` with a nearest-match suggestion under `FR-TMPL-027`, while `FR-TMPL-009`
makes an `{% include %}` that does not resolve literally a `65` naming the
template, the line and the column. The split is decided by **who asks**:

| Asked by | Mechanism | Code |
|---|---|---|
| The command line, before the engine is built | `tpl` resolves the named template itself and produces the suggestion over the names that exist | `66` |
| A template, through `{% include %}`, `{% import %}` or `{% extends %}` | The loader returns `Ok(None)`; the engine raises its own not-found error, which carries the line and the column `FR-TMPL-009` requires | `65` |
| Either, with a path that escapes the root | The loader returns `Err`, and the escape is reported as an escape | `65` |

`FR-TMPL-008` is preserved by construction: inside a template a name is
literal, and step 1 adds no extension. The optional extension of `FR-TMPL-007`
is a **command-line** affordance and is applied in `cli/` before the resolution
is asked for, so the engine never sees a name `tpl` completed.

**Rejected.**

- **Wrapping `path_loader` with the checks in front of it**, for the two reasons above.
- **Loading every template into the environment at startup with `add_template_owned`.** It makes containment a property of a single enumeration and would be simple to verify — and it reads and parses every template in the project for an invocation that renders one, which is the startup work `NFR-PERF-005` and the project's lazy-initialisation rule both refuse, and which would make `tpl render` pay for a template directory it does not use.
- **Placing the checks in each of the four `template` subcommands and again in the loader.** Five copies of a security rule is five places for it to differ; `FR-TMPL-023` makes the root "the boundary of every template lookup", which is one boundary and therefore one implementation.

**Composition with `security`.** The same function is the single place where
`FR-TMPL-024`, `FR-TMPL-025` and `FR-TMPL-026` are enforced, so `security.md`
cites one containment point rather than describing four. Which of the four
subcommands reaches it, and in what order relative to the trust checks of
`FR-PROJ-010`, is `architecture`'s to state.

**Unblocks.** `architecture`, `security`.

---

## OD-16 — The TLS backend and the root store

**Status: settled. Recorded in [`ADR-002`](../adr/adr-002-tls-mode-mapping.md).**

**Decision.** Bundled `webpki-roots` trust anchors, and the mapping of the five
modes of `FR-CONF-013` onto the driver's five named variants.

The mapping table, the trust-anchor rationale, the options rejected, and the
composition with `FR-CONF-037` and `FR-CONF-039` are recorded in `ADR-002` and
are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). `FR-CONF-038` requires the mapping to
live in an architecture decision record and to be cited from there; `ADR-002` is
that record.

---

## OD-17 — Observability

**Status: settled. Applied to `CLAUDE.md` on 2026-09-11.**

**Decision.** **No subscriber is installed, and `tracing-subscriber` is not a
dependency.** `tpl`'s diagnostics are written by a small module in
`diagnostics/`: a level held once, a locked and buffered handle on stderr, and
a **closed set of typed emission functions**. There is no general-purpose sink
that accepts arbitrary text.

**How `FR-GLOB-018` becomes structural rather than reviewed.** Two properties,
and together they close the two routes a forbidden category can take.

1. **Nothing a dependency emits can reach the stream.** `tracing` states it: "Any trace events generated outside the context of a subscriber will not be collected", and the crate "does not contain any `Subscriber` implementations" (docs.rs `tracing` 0.1.44, verified 2026-09-11). With no subscriber installed, every event any dependency emits — including whatever the database driver chooses to record about a statement or a failure — is discarded before it exists. The raw driver error therefore has no route to stderr at all, which is what `FR-GLOB-018` names first among the six.
2. **Nothing in `tpl` can emit an arbitrary string.** The emission functions take typed arguments — a phase and a duration, a query identity, a cache key and a hit or a miss — and compose the line themselves. There is no `debug!("{e}")` to write, because there is no function that takes a formatted message. This is the same move `OD-06` makes in the error type: the forbidden content is denied a **home**, not denied by a rule someone must remember.

`OD-06` supplies the third leg: the driver's error is classified at the
`mariadb/` boundary and the original value is dropped, so it is not present in
the process to be logged even by a function that would take it.

**How the one line per catalogue query is distinguishable.** It is emitted by
**one** function, and that function is the only writer of a fixed leading token
on the line. `FR-GLOB-017` constrains the existence of the line and its
distinguishability, not its wording, and `NFR-DET-001` keeps stderr outside the
contract. A test may therefore depend on the **token** without depending on the
stream being contract: what it asserts is the two properties the requirement
states — that exactly one such line exists per query, and that no other line
carries the token — which is precisely what `NFR-PERF-008` needs to make
`NFR-PERF-001` and `NFR-PERF-002` checkable. The wording after the token stays
free to change, and no test reads it.

**How the phase timings are produced.** From the deadline machinery of `OD-12`,
which already holds the start instant and the elapsed time of every phase
because it has to enforce a deadline on each and compose it with the overall
budget of `FR-GLOB-012`. `FR-GLOB-017`'s "which phases ran and how long each
took" is a report of data the program already has; measuring it a second time
through instrumentation spans would be two clocks for one fact.

**The levels.** `-v` raises the level to `INFO`, `DEBUG`, `TRACE` and saturates
(`FR-GLOB-014`); `-q` lowers it to errors only (`FR-GLOB-015`); neither touches
stdout (`FR-GLOB-016`). The level is resolved once, during argument handling,
and read from an ordinary shared value. `NFR-DET-004` forbids colour and every
ANSI escape sequence on either stream, so the writer emits none and has no
terminal detection to perform — `NFR-DET-003` forbids that too.

**Rejected.**

- **`tracing-subscriber`'s `fmt` layer with a custom `FormatEvent`.** It supplies a formatter and a span-timing facility that this program does not need — the timings come from the deadline machinery — and it costs the property that decides this entry: with a subscriber installed, every dependency's events become emittable, so `FR-GLOB-018` would be restored to a review item over every crate in the graph rather than a consequence of the architecture.
- **A minimal `Subscriber` written in `tpl`, keeping `tracing` as the front end.** It keeps the dependency-budget question open for a facade this program does not otherwise use, and it re-opens route 1: a subscriber that filters foreign events by target is a rule that can be got wrong, where installing none cannot.
- **Keeping `tracing` and `tracing-subscriber` because they are the ecosystem's default.** The project's dependency budget refuses a crate used for a trivial function, and what is used here is four levels and a handful of typed lines on one stream, with no asynchronous context to correlate and no structured consumer to serve.

**What was applied.** Task #20, at commit `ee7363d` of 2026-09-11. The
coordination document's stack table named `tracing` and `tracing-subscriber`
as the logging choice; its logging row now states the positive choice —
`tpl`'s own diagnostics, no subscriber installed — names the two crates only to
record that neither is used, and cites this entry. The decision removed the
second outright and left the first with no role of its own; if the driver
brings `tracing` transitively it stays in the graph as a transitive dependency
and not as a facility `tpl` uses. That table is an architecture decision by
that file's own terms, so the change was prepared for the user rather than made
here and applied under the authorisation of that day, in the same commit as
`OD-09`'s addition of `toml_edit`.

**Not verified.** Whether the database driver depends on `tracing` or on `log`,
and what it records at which level. It does not bear on the decision: with no
subscriber and no logger installed, both facades discard what they are given.

**Unblocks.** `operations`, `architecture`, `technology-stack`.

---

## OD-18 — Serialisation, key order, and the two omissions

**Status: settled.**

**Decision.** **Derived `Serialize`**, written through `serde_json` into a
writer `tpl` owns. Five answers follow, one per question the entry asked.

| Question | Answer |
|---|---|
| Derived or bespoke | Derived. The emitted types are the model's, and their **field declaration order is the key order** |
| `preserve_order` | **Off.** Nothing enables it, and no unordered map reaches the emitting path |
| The order of the two map-shaped documents | Byte-wise ascending by key, because `NFR-DET-002` already fixes it |
| The two omissions | One is a map that is never given the key; the other is one `skip_serializing_if`, used once in the crate |
| `serde` in the library's public signature | Yes, and it costs nothing |

**Why derived rather than a bespoke writer.** `FR-OUT-013` requires a fixed key
order per structure. Under derived serialisation that order **is** the struct's
declaration order, so it is stated once, in the type, and cannot drift from the
type. A bespoke writer states it a second time, in the writer, and a document
whose key order lives in two places is a document whose two statements
eventually disagree — the failure this folder's own conventions name. The same
argument decides `FR-SCH-022`'s round trip: `--context` is deserialised into
the same types, so the dump and its re-emission are inverse by construction
rather than by a pair of hand-written routines that must be kept inverse.

**Why the two arguments for a bespoke writer do not hold.**

- **C0 escaping.** On the JSON path it is the encoder's already: `serde_json::ser::CharEscape` enumerates `Backspace`, `FormFeed`, `LineFeed`, `CarriageReturn`, `Tab` and `AsciiControl(u8)` — "an escaped ASCII plane control character (usually escaped as `\u00XX`)" (docs.rs `serde_json` 1.0.151, verified 2026-09-11). JSON admits no raw control character inside a string, so a tab is emitted as `\t`, which is the escape. The ninth edition settled the reading this entry took: `FR-OUT-018`'s tab exception is now confined to `text`, and on the `json` path the requirement states that the escape the format defines is what satisfies it. On the **`text`** path the escaping is `tpl`'s own, with tab excepted for the column alignment `FR-OUT-006` needs, and it lives in `output/` beside the layouts it exists for. Neither path needs a serialiser wrapper.
- **The mid-document pipe state.** `FR-ERR-026` makes a stdout closed part-way through a JSON document a `74`. That is a property of the **writer**, not of the encoder: `tpl` writes through a buffered writer that records whether any byte of a document has been emitted, and reports `74` when a write fails after the first. `FR-OUT-021` and the project's own rule that I/O is aggregated require that writer regardless, so the state costs nothing extra.

**The order of `vars` and of the `cfg list` document is not a free choice.**
`NFR-DET-002` states the default and its exceptions: "Every collection the
system presents SHALL be ordered by name, ascending, compared byte by byte,
except where this specification names another order", and its table names six
exceptions, of which neither `FR-CTX-026`'s `vars` nor `FR-CFG-037`'s document
is one. Both therefore fall to the default. The model carries them as ordered
maps keyed by `String`, whose iteration order is byte-wise ascending, and
`serde_json::Value` never appears on the emitting path — so `preserve_order`
would change nothing if it were enabled, and it is not enabled: `Cargo.toml`
requests no feature of `serde_json`, and `Cargo.lock` records that crate's
dependencies as `itoa`, `memchr`, `serde`, `serde_core` and `zmij`, among which
the feature's optional `indexmap` does not appear (verified at commit
`fd51ca2`, 2026-09-18).

**The two exceptions to "absent is `null`", expressed differently because they
are different things.**

| Exception | Mechanism | Why |
|---|---|---|
| `FR-CFG-037` — a key absent from `.cfg` is absent from the `cfg list` document | The document is **built from the keys the file carries**. An absent key is a key never inserted | `FR-CFG-014` forbids applying defaults, so there is no value to omit. Modelling every key as an `Option` and omitting the `None`s would put fifteen omissions in the type to express one rule about a file |
| `FR-PRIV-016` — `restricted` appears only on an incomplete object | One `#[serde(skip_serializing_if = "Option::is_none")]`, on that field alone | The property belongs to the object and is genuinely optional. `FR-PRIV-016` also requires the array never to be empty, which the `Option` states and an empty `Vec` would not |

The attribute appears **exactly once** in the crate. That is the enforceable
form of `FR-OUT-012`: every other `Option` serialises as `null`, by serde's
default, and a second appearance of the attribute is a visible change rather
than a silent one. `verification` owes the test that counts the omissions in
the seventeen documents.

**Amended on 2026-09-18, when the model was built: the count is once per
markable kind, not once in the crate.** `FR-PRIV-005` through `FR-PRIV-007` put
the marking on a table, a view and a routine, and each is a type of its own, so
the attribute is written three times over **one** field. The decision is
unchanged and so is what it enforces — one field of the model is omissible, and
every other `Option` serialises as `null` — and the enforceable form moves from
a count of occurrences to the name of the field they are all on. The test
`verification` owes is unaffected: it counts omissions in the documents, not
attributes in the source.

**Amended on the same day: four of the emitted shapes are not the model's own
types.** This entry reads *the emitted types are the model's*, which holds for a
column, an index, a trigger, a `CHECK` constraint, a view, a routine, the
decomposed type, the `server` object and the marking. Four shapes differ, and
all four differ for one reason — the embedding of `FR-CTX-006` and `FR-CTX-010`,
which the model carries as a **name** and the document carries as an **object**:
the `database` object, a table, a foreign key, and an entry of `referenced_by`.
Two further shapes are the model's type **projected** onto a private shape, each
because the document's shape is not one a derive over that type produces: a
column default, whose discriminant `FR-CTX-012` puts inside the object beside a
`value` the `null` form does not carry, and the marking, whose document shape
`FR-PRIV-016` makes a bare array. Both projections are declared on the model
type and apply in both directions, so key order is still a property of a type
and the round trip is still inverse by construction — which is the whole of what
this entry decided. The shapes are enumerated in
[interfaces.md](interfaces.md#the-two-directions-over-the-document).

**Amended on 2026-09-21, when the render work read the document back: the
decomposed type is emitted flattened into the column.** It is still the model's
own type and is still serialised by the derive, so nothing this entry decided
moves; what changed is that the type is no longer a **level** of the document.
`FR-CTX-014` gives a column `column_type` and `FR-CTX-015` gives it the eight
parts *additionally*, which makes all nine siblings on the column. The key order
of `FR-OUT-013` is unaffected in substance and gains one rule: the nine keys are
emitted where the field sits, so a column's order is its own field order with
the decomposition's spliced in at that position. The derive states both, and no
writer restates either. Why the change left `schema_version` where it stands is
[data-model.md](data-model.md#the-four-version-numbers)'s.

**Amended on 2026-09-18 — `indexmap` is in the dependency graph, and its
presence is irrelevant to this decision.** This entry asserted twice that it is
not, in the `preserve_order` row and at the end of the paragraph above, and the
assertion was already false when it was written.
`cargo tree --all-features --invert indexmap`, run at commit `fd51ca2` on
2026-09-18, returns `indexmap v2.14.2` under two parents: `sqlx-core v0.9.0` —
the vendored, path-patched tree
[`ADR-010`](../adr/adr-010-driver-tls-connect-stall.md) places in the
repository — and `toml_edit v0.25.15+spec-1.1.0`. `tpl` depends directly on
both.

Neither parent is on the emitting path, which is the only path this entry
decides anything about: the documents are written by `serde_json` over the
model's own types, `toml_edit` is the configuration **write** path
([`OD-09`](#od-09--toml-the-read-path-and-the-write-path)), and the driver emits
no document at all. So what the entry was reaching for is true and checkable,
and it is stated as three facts rather than as one claim about the whole graph:
`preserve_order` is not enabled, `serde_json` does not depend on `indexmap`, and
therefore no unordered map reaches the emitting path. **The decision, its ground
and the options it rejected are unchanged**; a claim about the dependency graph
was standing in for a claim about one crate's features, and only the claim
moves.

**`--pretty`.** `FR-OUT-008` fixes a two-space indent, which is
`serde_json`'s own default — "construct a pretty printer formatter that
defaults to using two spaces for indentation" (docs.rs
`serde_json::ser::PrettyFormatter`, verified 2026-09-10). Compact is the
default form under `FR-OUT-007` and is `serde_json`'s ordinary serialiser.

**`serde` is a public dependency of the library, deliberately.** The model
derives `Serialize` and `Deserialize`, so serde's traits appear in the public
signature. `DIV-032` records that the library carries no compatibility
guarantee, which removes the cost a public dependency ordinarily has: there is
no consumer whose build a serde major version could break. `technology-stack`
records it as a fact about the surface, not as a risk.

**Where the lossy conversion happens.** `FR-OUT-017` replaces an invalid UTF-8
byte sequence with U+FFFD. That happens in `mariadb/`, where catalogue values
are read as bytes, so the model holds only valid UTF-8 and every consumer —
JSON, `text`, the render context, the cache — inherits the substitution once.
It is not an emitting-path transformation and does not belong to `output/`.

**Rejected.**

- **A bespoke writer for the seventeen documents.** It duplicates the key order that the types already state, and it puts the round trip of `FR-SCH-022` in the hands of two routines that have to stay inverse.
- **Enabling `preserve_order`.** It contradicts `NFR-DET-002` for the two map-shaped documents — insertion order is neither name order nor a named exception — and it changes the ordering of every map globally to answer a question about two.
- **`skip_serializing_if` as a general convention.** It would silently omit every future `Option`, which is exactly what `FR-OUT-012` forbids: "An absent value SHALL be emitted as `null` and SHALL NOT be omitted, so that the shape of a document is constant."
- **A distinct type to express absence for the two exceptions.** It states the rule in the type system at the price of two parallel model shapes for two fields, and `FR-OUT-014` already makes adding a field non-breaking, so the shape has to stay one shape.

**Unblocks.** `interfaces`, `data-model`.

---

## OD-19 — Whether the two embeddings are materialised

**Status: settled, with an observation owed to `adr-guardian`. Recorded in
[`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md).**

**Decision.** Materialise both embeddings: the object graph is the document.

The rationale, the memory consequence, and the two options rejected — emitting
by reference at serialisation time, and streaming the dump — are recorded in
`ADR-009` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

**Observation owed — the record's word for what an embedding site holds.**
`ADR-009` reads *"Each embedding site holds its own **owned copy** of the
embedded table"*. As built, each site holds its own **value**, materialised
before any byte is written, and that value's members are **borrowed from the
model** where the model already holds them in the order the document needs: an
embedded table's columns, indexes, triggers and `CHECK` constraints are the
model's own, not copies of them. What is built per site is the **cut itself** —
the two reference collections, as names — together with the primary key the
document presents a second time beside the indexes, and any collection that had
to be reordered. The decision the record took is unaffected — the cut is a
property of the value rather than of an emitter, which is the whole of what it
decided, and it is what makes a dangling reference detectable as `70` rather
than emitted at `0`. What may be affected is a **consequence** the record
states: it names peak resident memory as the price of the choice, and a
borrowed member does not pay it. Whether the record's wording is corrected, and
whether its memory consequence is restated, is `adr-guardian`'s and not this
register's; nothing in this folder may read *owned copy* as settling how the
value holds its members until it is. Reported 2026-09-18, against commit
`0cdc539`.

---

## OD-20 — Edit distance, and the other small algorithms

**Status: settled.** The four further algorithms are settled in the residual
below. **The measure itself left this register on 2026-09-18**, and what remains
here is how it is computed.

**Narrowed on 2026-09-21: `FR-ERR-039` names the member, and this entry no
longer holds the question.** This entry chose the Damerau-Levenshtein family and
carried the argument for it, because no requirement named a measure.
`FR-ERR-039` now does — the **restricted** form, optimal string alignment —
assigned at commit `a67d656` of 2026-09-18. It belongs there rather than here
because the two forms of the distance disagree inside the threshold
`FR-ERR-019` fixes, so the member decides **which candidates a `66` offers** and
not merely how they are ranked, which makes it observable output. The measure,
the worked example that motivates the transposition, and the form rejected are
that requirement's, and are **not restated here**.

**Decision, and the whole of what is still this entry's.** The distance is
**hand-rolled in the crate, with no dependency, over a rolling window of three
rows**.

| Question | Answer | Why |
|---|---|---|
| Whose code | The crate's own | `CLAUDE.md` *Orçamento de dependências* prefers `std` and refuses a crate used for one function; and `BR-PERF-004` makes this a measured path — 200 comparisons for a `66` over `WL-001` — so the implementation has to be ours to measure |
| How much state is held | Three rows of the comparison, rolled, and the supplied name decoded once per run | The transposition rule reaches back one row further than a plain substitution does, so rows `i`, `i - 1` and `i - 2` are the whole of what the recurrence reads (`src/diagnostics/suggest.rs`, `Matrix`, read 2026-09-21). A window bounded at all is what `FR-ERR-039` buys: it refuses the unrestricted form on the ground that it "reaches back to an arbitrary earlier position" and holds the whole comparison. Three rows also let the scratch be allocated once per population rather than once per candidate, which is what a path traversed against 200 names for one refusal is worth doing (`BR-PERF-004`) |

**Rejected.** A distance crate, for the dependency budget. The measure's own
rejected alternatives — plain Levenshtein, and the unrestricted form — are
`FR-ERR-039`'s.

**Where this is built.** `src/diagnostics/suggest.rs`, in its `Matrix` scratch,
which holds rows `i`, `i - 1` and `i - 2` (read 2026-09-21).

**Residual, settled 2026-09-11. All four are implemented in the crate, with no
dependency.** The four are decided together because one argument decides all of
them: each has a **closed grammar written into the corpus**, so a library would
have to be constrained back to that grammar rather than consulted for it, and
the dependency budget refuses a crate used for one function.

| Algorithm | Requirements | Decision | Rejected |
|---|---|---|---|
| The `LIKE` matcher | `FR-SCH-012` … `FR-SCH-014` | Implemented in the crate over the two metacharacters and the two escapes the requirement enumerates — `%`, `_`, `\%`, `\_` — matched in memory and never sent to the server | A regular-expression crate, which brings a full engine and a translation step for a two-metacharacter grammar, and whose own escaping rules would have to be got right over a name that is free text on the server; a glob crate, whose semantics are a different language |
| POSIX word splitting | `FR-CONF-025` | Implemented in the crate, honouring single and double quotes as the requirement states. It runs **only** on a string supplied to a command, never on a value read from `.cfg`: `FR-CONF-035`'s rationale refuses "a quoting engine on the path that reads an untrusted `.cfg`", and `FR-CONF-024` makes one unnecessary by executing the stored array directly and without a shell | A shell-words crate, for the dependency budget, and because the splitting a crate performs is the shell's whole grammar rather than the two quoting forms the requirement names |
| The word-list tokeniser | `FR-ENV-030`, with the eight-row vector of `FR-ENV-032` | Implemented in the crate. The five rules are stated in the requirement, applied once, left to right, and `FR-ENV-032` publishes the expected output for eight operands — so the implementation is testable against the corpus rather than against a library's idea of a word | An inflection or case-conversion crate, whose rules are its own: `FR-ENV-033`'s accepted cost fixes `HTTP_server` → `HttpServer`, and a crate that preserves acronyms would produce a different generated identifier at exit `0` |
| ASCII-only case folding | `FR-ENV-031`, `FR-SCH-014` | `std`: the ASCII-restricted `str::eq_ignore_ascii_case` and `str::to_ascii_lowercase`, which exist beside the Unicode-aware `str::to_lowercase` for exactly this distinction (Rust standard library documentation, `str`, verified 2026-09-11; the exact wording of their guarantee was not retrievable from the rendered page and is not quoted here) | Unicode or locale-aware folding, which both requirements refuse in their own text, because it would make the same template produce different output on two machines and break `NFR-DET-001` |

**What still belongs to `interfaces`.** The surface of each — the signature,
the inputs it accepts, and the errors it can produce — is described there.
What is settled here is the choice and its rejection, which is what this
register exists to carry.

---

## OD-21 — Two test seams that must not be on the published surface

**Status: settled.** The conflict was resolved by the eighth edition; the
residual it left is settled below.

**What the conflict was.** `FR-ERR-031` required a deliberate trigger for `70`
and `FR-SRV-035` a seam that presents the reader with a series above its own
window, both barred from every help text and from both command trees — while
`BR-ERR-001` demanded an **integration** test per exit code and `FR-SRV-035`
an assertion on the exit code, neither observable without running the binary.
Every mechanism a running binary could reach collided with a requirement in
force.

**The resolution, made in the corpus and not here.** Both seams move **inside
the process**, reachable from nothing a caller can write. The four requirements
`BR-ERR-001` enumerates hold unchanged — `FR-CLI-002`, `FR-CLI-021`,
`FR-HELP-021`, `NFR-PERF-018`, to which `FR-ERR-031`'s own table adds
`FR-CLI-023` and `NFR-DET-001` — and the two that yield say so in their own
text.

| Requirement | What it now fixes |
|---|---|
| `FR-ERR-031` | The trigger is reachable **only from within the system's own test configuration**, is reachable from **no invocation of the binary the project distributes**, and appears in no help text, in the JSON tree of `FR-HELP-016`, or in the tree of `FR-CLI-002`. The requirement enumerates the three rejected candidates and the requirement each collides with |
| `BR-ERR-001` | Yields **for `70` alone**, stated in its own text: `70` is exercised in process through the trigger and **not** by an integration test. The nine other codes are unchanged |
| `FR-SRV-035` | Yields the assertion on the exit code. The test asserts that the read completes without error and that `standing` is `newer_than_supported`; the seam is the one `FR-ERR-031` names. Building an impostor server to make the observation external is rejected in the requirement |
| `BR-SRV-003` | States what it reaches: the three promises about what the process **sends** (`FR-SRV-012` … `FR-SRV-014`), and not `FR-SRV-035`, which is a promise about what the reader **emits** |

**What this decides for `verification`.** Two tests, both in process, each
described with the limit its own requirement states rather than a limit this
folder invents: what is executed is the guard, and separately the step from an
error condition to an exit status — `BR-CLI-004` for `FR-SRV-035`, and the nine
integration-tested codes for `FR-ERR-031`. The composition of the two is
reasoned rather than executed, and both requirements say so. `verification`
cites that limit and does not restate it.

**No longer part of this entry.** The tension between the condition
`FR-ERR-030` carried before the ninth edition — a panic **caught** at the top
level — and an aborting release profile is not a seam question; the eighth
edition recorded it as `DIV-045` rather than resolving it, and the ninth
amended the requirement to state the outcome instead. It is carried in
`OD-28`. `FR-ERR-031`'s trigger exercises the **other** producing condition of
`70` — the detected invariant violation — so the exception `BR-ERR-001` grants
does not depend on that answer.

**Residual, settled 2026-09-11. The construct is `#[cfg(test)]`, and both tests
are therefore unit tests inside the library.**

`FR-ERR-031` requires the trigger to be "reachable only from within the
system's own test configuration". `#[cfg(test)]` *is* that configuration, and
the toolchain's own documentation states both halves of what the residual asked
to be verified rather than assumed (The Rust Programming Language, ch. 11.3,
verified 2026-09-11):

- **Absent from the distributed binary.** "The `#[cfg(test)]` annotation on the `tests` module tells Rust to compile and run the test code only when you run `cargo test`, not when you run `cargo build`. This saves compile time … and saves space in the resultant compiled artifact because the tests are not included."
- **Absent from an integration test.** "Each file in the *tests* directory is a separate crate, so we need to bring our library into each test crate's scope." A separate crate is compiled without `cfg(test)` for the library it links, so an item behind `#[cfg(test)]` is not there to be reached.

The second half decides the test kind, and it decides it the same way for both
seams: an integration test **cannot** see the construct, so each test is a
**unit test in the library crate**. That is consistent with what the corpus
already granted — `BR-ERR-001` yields the integration test for `70` alone and
says so in its own text, and `FR-SRV-035` asserts on what the reader emits
rather than on a process exit status.

**Rejected.**

- **A `#[doc(hidden)] pub` item.** It satisfies `FR-ERR-031` literally — a library item is not reachable from an invocation of the binary — and it puts a trigger on the surface the project publishes, which is the property this entry's title refuses. `DIV-032` removes the compatibility cost, not the surface.
- **A cargo feature.** `FR-ERR-031` rejects it by name: "the artefact verified would not be the artefact distributed", and `NFR-PERF-018` makes every distributed artefact first class.
- **An environment variable or a hidden command.** Both are rejected in `FR-ERR-031`'s own table, against `FR-CLI-021`, `FR-CLI-023`, `NFR-DET-001`, `FR-CLI-002` and `FR-HELP-021`.

**What `verification` still owes.** The register of the two tests, each
described with the limit its own requirement states — not a limit this folder
invents — and the naming of both by requirement identifier.

**Unblocks.** `verification`.

---

## OD-22 — The test harness, and the fixture certificate

**Status: settled by the eighth edition. The residual was discharged on
2026-09-11 by tasks #15 and #25.**

**What the conflict was.** `FR-CONF-013` defaults `tls` to `verify-identity`,
and `FR-CONF-038` recorded that `tpl` with default configuration could not then
reach the fixture of `scripts/mariadb/` over TCP on any series. The default mode
was therefore the one cell of a ten-cell table with no acceptance test, and the
fixture could not supply one.

**The resolution.** `FR-CONF-038` now states the fixture obligation as a
requirement: the fixture **SHALL** be able to present, at each series of
`FR-SRV-015`, a server whose certificate names the host by which the project's
tests reach it, and **SHALL** retain a server that offers no TLS. Two options
are rejected in the requirement's own text — dropping the acceptance test and
stating the cost, because the default is the mode a caller meets without asking
for it; and configuring the certificate on the three TLS-capable series alone,
because `FR-SRV-029` requires the test against every series and `10.11` is
supported until 2028-02-16. None of the ten cells changed, and
`verify-identity` is not relaxed: a certificate naming the host is what the
mode always required.

**The residual was work, and the work was run.** `FR-CONF-038` handed both
halves to the fixture in its own text — how the certificate is generated, where
the fixture keeps it, and how the no-TLS server is retained beside it "are the
fixture's own work and are not specified here" — and this entry refused to
record an arrangement of files and commands nobody had executed. Both were then
executed, and `scripts/mariadb/README.md` is their record. What follows names
what landed; the arrangement itself is not restated here.

| Half | Discharged by | What it produced |
|---|---|---|
| The fixture certificate | Task #15, commit `4bce12e` | `scripts/mariadb/tls/`: a root whose private key is destroyed at generation, a leaf naming `DNS:localhost`, `IP:127.0.0.1` and `IP:::1`, the `ssl_ca`/`ssl_cert`/`ssl_key` settings that put it into service, and `generate.sh` to reproduce it. All four series report `have_ssl=YES` and accept a `verify-identity` connection; the no-TLS server survives as a fifth container started `--skip-ssl` |
| The harness | Task #25, commit `de7ed1e` | `up.sh`, `down.sh`, `status.sh` — the three-valued gate — `observe.sh`, `series.env`, `probe-session.sql` and `observer.Dockerfile`, each established against a substitute client, `tpl` having no catalogue reader to send a statement with |

**Both halves of `FR-CONF-038` are therefore satisfied**, and no passage of this
folder may still describe the fixture as unable to present a named certificate
or `tpl` as unable to reach it by default. What the discharge did **not**
establish is recorded by the fixture with what was tried in each case: the
failing outcome of `FR-SRV-013`, which no real MariaDB produces; `NFR-PERF-001`
and `NFR-PERF-002` conclusively, which wait on `WL-001` and therefore on
[`OD-27`](#od-27--seed-benchsql-and-wl-001); and the file-open observation on
either Darwin target, which `NFR-PERF-005` now bars from being inferred from a
Linux build traced in a container.

**What the harness must serve, unchanged by the resolution.**

- `CLAUDE.md`: validation needing a database uses the containers of `scripts/mariadb/` — never mocks, never external instances — launched before and stopped after; `scripts/mariadb/README.md` adds "Leave no container running after a validation run."
- `FR-SRV-029`: the cross-series equivalence test runs against every series, and the refusal test against at least one series outside the window.
- `NFR-PERF-007` and `BR-SRV-003`: nine requirements are verified from **outside** the process, by the four instruments `NFR-PERF-007` names, each on the targets of `NFR-PERF-018` its row admits.
- `FR-SRV-012`: the closed statement list is checked by observing what the server actually receives, expecting four kinds and no fifth, with the three connection-start statements issued once each in the stated order.
- `BR-SEC-003`: the sentinel test runs every command of the tree at maximum verbosity and asserts the sentinel appears in no byte of either stream.

**Unblocks.** `operations` and `verification`, which waited on this residual and
on nothing else in this register.

---

## OD-23 — Packaging, artefacts, and the musl build path

**Status: settled. The build path is recorded in
[`ADR-008`](../adr/adr-008-packaging-and-build-path.md); continuous integration,
the release gate and the release artefact in
[`ADR-012`](../adr/adr-012-ci-and-release-distribution.md).** Amended on
2026-09-24: the entry had recorded that no continuous integration was
prescribed, which `ADR-012` reverses.

**Decision.** `cargo-zigbuild` for the two `musl` targets and native builds for
the two Darwin targets, per `ADR-008`. Distribution through the GitHub Actions
workflows of `ADR-012`.

The rationale, the options rejected, the obligations still carried by hand, and
the two targets that have never been measured are recorded in the two records
and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

---

## OD-24 — The discovery boundary, and the process uid

**Status: settled. Applied to `CLAUDE.md` on 2026-09-11.** The conflict was
resolved by the eighth edition; the residual it left is settled below.
**Amended on 2026-09-24**, for rmp `#306` and `#307`: `rustix` gains the `fs`
feature, by the user's decision; the correction it owed `CLAUDE.md` was applied
the same day.

**What the conflict was.** `FR-PROJ-005` made the user's home directory a
boundary of project discovery. It can only be located from `HOME`, which
`FR-CLI-021` forbids reading to determine the location of the project, and a
shell exporting a different `HOME` falsified `BR-CLI-002`.

**The resolution.** `FR-PROJ-005` is the requirement that yields. The upward
walk stops at **the mount point alone**, and the boundary "SHALL be determined
without reading any environment variable". Three statements follow it.

| Requirement | What changed |
|---|---|
| `BR-CLI-002` | Gains the clause it was missing: nothing a shell can set may decide which project is discovered, which database entry is selected, or which server is reached |
| `FR-SEC-013` | The walk stops at the mount point; the threat it closes is narrower than the first edition claimed, and a `.tpl` planted in a world-writable ancestor is refused by `FR-SEC-014`, not by any boundary |
| `DIV-024` | Shortened by one clause: the correction owed to `CLAUDE.md` no longer names a home boundary |

Locating the home directory from the system's own account record rather than
from `HOME` was rejected in `FR-PROJ-005`'s own text: the boundary would then
sit wherever that record says, which need not be an ancestor of the working
directory, so the rule could silently never fire.

**Technical consequence.** No home-directory lookup remains anywhere in
discovery, so `std::env::home_dir` and every substitute for it leave the
implementation's path entirely. The one boundary that remains is
`std::os::unix::fs::MetadataExt::dev()`, compared between a directory and its
parent: in `std`, and no `unsafe`.

**The accepted cost, stated by `FR-PROJ-005` and inherited here.** A `.tpl`
folder above the caller's home directory and on the same filesystem — at
`/home`, at `/Users`, or at `/` — is now within the walk. Three things bound
it: such a directory is not ordinarily writable by the caller, so a `.tpl`
there is either the caller's own or is refused by `FR-PROJ-010`; `tpl init`
warns on stderr when it creates a project that shadows one above it, per
`FR-PROJ-016`; and `FR-PROJ-007` admits no fallback, so a walk that reaches the
boundary without finding a `.tpl` fails with `78` per `FR-PROJ-006` rather than
reading settings from anywhere else.

**Residual, settled 2026-09-11. The safe wrapper is `rustix`, with default
features off and the `process` feature alone.** `rustix::process::getuid` is
declared `pub fn getuid() -> Uid`, is a safe function, and is "available on
crate feature `process` only" (docs.rs `rustix` 1.1.4, verified 2026-09-11).
That is the one value `std` does not supply: `std::os::unix::fs::MetadataExt`
gives the **file's** `uid()` and its `mode()` on the same metadata (`std` API
documentation, `std::os::unix::fs::MetadataExt`, Rust 1.98.1, verified
2026-09-11), so `FR-PROJ-011`'s mode check needs nothing further and
`FR-PROJ-010` needs exactly one call.

**Why a dependency at all.** `getuid()` through `libc` is an `unsafe` call, and
the project forbids `unsafe` and keeps `#![forbid(unsafe_code)]` at the top of
the crate. That is not a rule to be weighed against a dependency: it is one of
the project's non-negotiable rules, and the crate that satisfies it is the
smallest one that does.

**Rejected.**

- **`libc` with a local `unsafe` block.** Forbidden outright, and the attribute that forbids it is required to stay.
- **`nix`.** It answers the same question and carries a far larger surface for it; the budget prefers the narrower crate.
- **A crate that resolves the user account** — `uzers` and its predecessors. `FR-PROJ-010` compares two uids and needs no user name, and `OD-24` has already removed every account lookup from discovery.
- **Inferring ownership by effect**, by attempting a write. `.tpl/.cfg` is read-only to every command that checks it, and `BR-TMPL-002` and the project's own scope statement keep `tpl` from writing where it was not asked to.

**One consequence for `technology-stack`.** `rustix` is a new normal
dependency. What it drags in on each of the four targets of `NFR-PERF-018` is
recorded there, under the dependency budget.

**What was applied.** Task #20, at commit `ee7363d` of 2026-09-11: the `Stack`
table of `CLAUDE.md`, which named no such dependency, gains a row for the
process uid — `rustix`, with `default-features = false` and the `process`
feature alone — and cites this entry. That table is an architecture decision by
that file's own terms, so the change was prepared for the user rather than made
here and applied under the authorisation of that day, in the same commit as
`OD-09`'s and `OD-17`'s corrections.

**Unblocks.** `technology-stack`.

**Amended on 2026-09-24 — the `fs` feature.** Decided by the user for rmp
`#306` and `#307`, as relayed by the session coordinator; `adr-guardian` judged
no architecture decision record admissible, under rules R3 and R4 of
[`docs/adr/README.md`](../adr/README.md), because the crate's feature set lives
in this entry and in `OD-12`. The manifest's `features = ["process"]` becomes
`features = ["fs", "process"]`.

*Why.* Two races remain open with the calls `std` supplies.

- **Check-then-use on cache paths** (CWE-367). A path inspected and then used
  by name can be swapped between the two; the security review of rmp `#305`
  demonstrated a deletion outside the project on run 971 of a swap loop.
  Closing it needs every component opened relative to its parent's descriptor,
  with `openat` and `O_NOFOLLOW | O_DIRECTORY`, and the file operations made
  relative to that descriptor: `unlinkat`, `renameat` and `mkdirat`.
- **A FIFO in a file's place.** A FIFO swapped in after `lstat` blocks the
  `open` of a cache record or object file; and, for rmp `#307`, a FIFO at
  `.tpl/.cfg` or at a template blocks outside every deadline. Closing both
  needs opens with `O_NONBLOCK | O_NOFOLLOW`.

*What was verified* (docs.rs and crates.io, `rustix` 1.1.4, consulted 2026-09-24). `openat`, `unlinkat`, `renameat` and `mkdirat` are
safe functions, each "available on crate feature `fs` only". `OFlags` carries
`NOFOLLOW`, `DIRECTORY`, `NONBLOCK` and `CLOEXEC`. The feature is declared
`fs = []`, so it adds no crate to the graph. The crate declares
`rust-version = "1.63"`, below the floor of
[`ADR-007`](../adr/adr-007-msrv.md), which is therefore unaffected. `windows-sys`
is a dependency only under `cfg(windows)`, which no target of `NFR-PERF-018`
sets.

*Rejected.*

- **Not adding the feature, and accepting the two races as residuals.** The
  review's proof of concept deleted outside the project, and `FR-SEC-026` and
  `FR-PROJ-024` forbid exactly that.
- **`libc`.** Each call would be `unsafe`, which `#![forbid(unsafe_code)]`
  forbids, as for `getuid` above.
- **`nix`.** A second binding to the same calls beside `rustix`, which the
  dependency budget refuses.
- **Hard-coding the flag values.** They differ between the supported targets,
  as `OD-10` recorded, and `rustix` supplies them per target.

*Consequence.* The cache's path operations, the `.tpl/.cfg` read and the
template read are directory-relative, implemented in `src/at.rs`
([data-model.md](data-model.md#tplcache),
[security.md](security.md#template-containment)); the calls are listed in
[technology-stack.md](technology-stack.md#the-calls-std-does-not-supply). One
step, listing a directory, still resolves a path by name, because the `alloc`
feature that supplies `rustix::fs::Dir` was not added; why that is benign is
data-model.md's.

---

## OD-25 — The clock source for `now`

**Status: settled.**

**Decision.** **Hand-rolled, in `std`, no date dependency.** One clock read per
invocation, one civil-date conversion, and **one grammar used in both
directions** — the fixed twenty-character form `YYYY-MM-DDTHH:MM:SSZ` and
nothing else.

- The instant is `std::time::SystemTime::now`, converted with `duration_since(UNIX_EPOCH)`, whose seconds are documented as "the number of non-leap seconds since the start of 1970 UTC", equivalent to a POSIX `time_t` (Rust standard library documentation, `std::time::SystemTime`, verified 2026-09-11).
- The seconds are split into days and seconds-of-day, and the days are converted to a proleptic Gregorian civil date by integer arithmetic. `FR-CTX-029` reads the clock once per invocation, so the conversion runs at most twice — once to write, once to read a cached value back.
- The reverse direction parses **only** that form, position by position, and rejects everything else.

**Why one grammar in both directions is the whole of the argument.**
`FR-CTX-028` fixes the form exactly — UTC, `Z` offset, second precision — and
`FR-CDOC-013` puts the same value in `meta.json` and in the output of
`tpl cache status`, where `FR-CACHE-034` shows it in that form. So `tpl` writes
one form and must read back the form it wrote. A date crate reads the whole of
RFC 3339, which admits a fractional part (`time-secfrac = "." 1*DIGIT`), a
numeric offset (`time-offset = "Z" / time-numoffset`), and lower-case `t` and
`z` — "the 'T' and 'Z' characters in this syntax may alternatively be lower
case 't' or 'z' respectively" (RFC 3339, §5.6, verified 2026-09-11). A reader
that accepts all of that and a writer that emits one of them are **not
inverse**: a hand-edited `meta.json` carrying `2026-09-10t08:14:22.5+01:00`
would parse, and `tpl cache status` would then re-emit a `loaded_at` in a form
`FR-CDOC-013` does not fix, or silently shift the value into UTC. Writing both
directions against one grammar removes the case rather than handling it.

**What a value that does not parse means.** The cache file is unreadable, and
`FR-CACHE-033` already fixes the outcome: treat it as a miss, read from the
server, rewrite the file, and report neither an error nor a warning. No new
rule is needed, and no `70` is reachable from a file the caller can edit —
which `FR-ERR-031` refuses in its own *Rejected* note.

**What the conversion must be, so that `verification` can test it.** Proleptic
Gregorian, no leap seconds — the epoch seconds are non-leap by the
documentation quoted above — and correct across the range a `SystemTime` can
carry. It is a closed function of one integer with a published expected value
per input, so it is testable as a vector rather than against a clock.

**One property recorded, because it is a property of the clock and not of the
conversion.** `SystemTime` "is not monotonic" and `duration_since` returns a
`Result` because "an earlier `SystemTime` may actually be later than a later
one" (same source and date). A system clock behind 1970 therefore has no
representation in this form. `now` is documented by `NFR-DET-005` as the single
source of non-reproducibility, and a clock that cannot be converted is a defect
in the host rather than in the input — `FR-ERR-030`'s detected invariant
violation, not a caller-facing condition.

**Rejected.**

- **A date crate — `chrono`, `time`, or `jiff`.** Each brings a formatting and parsing engine, and each brings or optionally brings a timezone database, to answer a question with no timezone, no locale, no offset, no fractional part and no alternative form. The project's dependency budget refuses a crate used for one trivial function, and this is one integer-to-civil-date conversion and one twenty-character format.
- **A date crate confined to the write path, with a hand-rolled reader.** It is the worst of both: a dependency *and* two grammars, which is the failure the decision above exists to avoid.
- **Storing `loaded_at` as an epoch integer in `meta.json` and formatting it only on the way out.** It removes the parse, and it contradicts `FR-CDOC-013`, which puts the value in `meta.json` and in `cache status` as one value in one form; `BR-CDOC-005` keeps `meta.json` outside the plumbing contract but does not license a second representation of a field the corpus names in both places.
- **Emitting the stored string verbatim without parsing it.** It removes the conversion and admits into a contract-shaped document whatever a hand-edited `meta.json` carries, which `FR-CDOC-013` and `FR-CACHE-034` between them do not admit.

**Unblocks.** `technology-stack`, `architecture`.

---

## OD-26 — The boundary against the knowledge graph

**Status: settled. Applied to `CLAUDE.md` on 2026-09-11.**

**Decision.** `docs/spec-technical/` becomes the **fourth row** of the
coordination file's sources-of-truth table, owned by `technical-writer`,
answering **how the system is built**. The knowledge graph stays **descriptive**
for code-level facts. The change to `CLAUDE.md` is **never made unilaterally**;
it waited for the user's approval, and it has it.

**Rationale.** `specification/README.md` hands the crate layout, the module
layout, the types and the library API to architecture without naming where
architecture lives, and two requirements — `FR-ENV-003` and `FR-CONF-038` —
already cite a home outside the functional corpus. A source of truth that is
not in the coordination table is a source of truth agents will not consult.
`CLAUDE.md` assigned *"Onde e **como**"* to the graph, which overlapped this
folder's scope, so the third row narrowed as the fourth was added.

**What was applied.** Task #46, under the user's authorisation of that day:
`CLAUDE.md` now opens *Fontes de Verdade* with four sources and carries the two
rows below verbatim. The same task added the governance this entry does **not**
cover — `docs/adr/`, which `OD-26` never mentions: who writes the records and
when one is required, citing [`docs/adr/README.md`](../adr/README.md) for their
rules rather than restating them.

**The two rows, in the language of the file they are destined for**, as
applied:

```
| Knowledge Graph | **Onde** — que código existe, como se articula, e que requisito cada componente satisfaz | skill `knowledge-authority` |
| `docs/spec-technical/` | **Como** o `tpl` é construído — arquitectura, interfaces, dados, segurança, operação, qualidade | subagente `technical-writer` |
```

**The scope addition the user made, and its consequence.** The graph will also
represent the component architecture, the application flows, and
requirement-satisfaction edges, so that it answers *which component satisfies
`FR-X`* and *what breaks if I change this*. The architecture is therefore
carried twice — as prose here and as a graph model there — and **the two must
not drift**. The division is recorded in
[README.md](README.md#the-architecture-is-carried-twice): this folder
prescribes and traces to requirements; the graph describes what exists; where
they differ, both readings are reported and neither is silently corrected. A
graph fact is never cited here as authority for a decision.

**Rejected.** Leaving the technical specification out of the table, which
leaves two requirements citing a home no coordination document acknowledges;
and folding it into the graph's row, which would put a prescription and a
description under one owner and one scope — the confusion `CLAUDE.md` warns is
"o erro previsível".

---

## OD-27 — `seed-bench.sql` and `WL-001`

**Status: settled, and discharged on 2026-09-21.**

**Decision.** `scripts/mariadb/seed-bench.sql` is written in a **later sprint —
the one that implements the catalogue reader**. Until then the five measurement
points that depend on it carry no measured figure, and whatever figure each
carries is marked **adopted**, in the vocabulary `NFR-PERF-019` provides.

**Discharged: the file exists, and the decision held.** It was written on
2026-09-21, in the sprint the decision named, and `scripts/mariadb/README.md`
records what it loads and how it is verified: `seed-bench.sql` carries `WL-001`
and `WL-003` as DDL alone, loaded on demand by `seed-bench.sh`, which checks
every count each workload states and is accepted by all four series and by the
`--skip-ssl` server. The measurement the decision was waiting for was made at
once: a full read of `WL-001` and a full read of `WL-003` cost **eleven
catalogue statements each, on all four series**, which is `NFR-PERF-001`
satisfied by measurement rather than by review. Which points have since been
measured, on which target and under what conditions, is
[quality-attributes.md](quality-attributes.md#the-measurement-set)'s to state;
nothing of this entry's decision or rationale changes.

*Amended on 2026-09-22, in vocabulary and in two statements that had stopped
being true.* The thirty-sixth edition of `specification/performance-requirements.md`
withdrew the performance gates, retired four identifiers and renamed what this
entry called a *budget*, a *provisional* figure and a *ratified* one. The
decision above and the rationale below are unchanged in substance: what this
entry decided was **when the fixture is written**, and that is untouched by
anything the edition did.

**Rationale.** `WL-001` fixes the fixture's content — 200 tables, 2 400
columns, 600 indexes, 180 foreign keys, 40 generated columns, 25 triggers, 30
views, 40 routines, comments on 60% of the tables — and `BR-PERF-002` keeps it
separate from `seed.sql` on purpose, because one fixture serving both "would
hide an N+1, which is invisible at ten tables". `BR-PERF-007` recorded, when
this entry was written, that it was the one file of `scripts/mariadb/` still
absent, and that `WL-001` is what needs it; **that rule was restated over the
fixture that now exists, in commit `455e48d` on 2026-09-21, so nothing is owed
to the functional owner for it** and
[quality-attributes.md](quality-attributes.md#the-measurement-set) records the
discharge. Writing it beside the reader it measures is the point at
which an N+1 becomes detectable; writing it earlier produces a fixture nothing
can be run against. `NFR-PERF-019` and `NFR-PERF-020` are exactly the mechanism
for carrying a named, unmeasured point and for recording it when somebody
measures it, so `quality-attributes` can state all nine today.

**Rejected.** Writing it in this sprint, which produces a 200-table fixture
with no reader to exercise it and no measurement to validate it against; and
merging it into `seed.sql`, which `BR-PERF-002` forbids and which would make
the correctness suite pay for 200 tables on every run.

**Consequence to record, and discharged.** `WL-002`'s byte scalar `N` could not
be computed until the fixture existed. It was computed on 2026-09-22 and
recorded in `BENCHMARKS.md` under `NFR-PERF-020`, byte-identical across five
takes. `DIV-036` is the register entry of `/specification` that tracks what
naming the file still owes the root coordination document, and what it owes is
that register's to state.

---

## OD-28 — The release profile against the caught-panic condition of `70`

**Status: settled. Recorded in [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md).**

**Decision.** The profile stands, and the process installs a panic hook that
produces the outcome `FR-ERR-030` requires.

The five profile settings, the hook, what the `cause` line carries and what it
withholds, the options rejected, and the reasoned step over `strip = true` are
recorded in `ADR-004` and are **not restated here**, per rule R3 of
[`docs/adr/README.md`](../adr/README.md).

**What this entry settled beyond the decision, and which stays here.** The
amendment it owed `specification-manager` landed in the ninth edition of
`/specification`, at commit `4ad5e8c` of 2026-09-11: `FR-ERR-030` now states the
outcome a caller observes rather than a mechanism, `FR-ERR-034`'s `70` row loses
the same word, and the code table of `FR-ERR-001` is unchanged. `DIV-045` is
discharged with nothing owed to `CLAUDE.md` under it. The order was not
interchangeable: the requirement moved first, before any implementation was
written against it.

**Consequence for the coordination document.** Its release-profile bullet cites
`ADR-004` rather than listing the five settings, per rule R3. No setting
changes.

**Unblocks.** `architecture`, `technology-stack`, `operations`.

---

## OD-29 — The JSON command tree: two shapes, and what the binary publishes

**Status: settled by the twenty-first edition of `/specification`, 2026-09-15.**

**Decision.** Neither shape is this folder's to choose, and both are now fixed
by requirement. `data.template_surface` is a **fourth key of `data`**, and
`data` is an open set to which a later edition adds after the last
(`FR-HELP-017`). `data.commands` is a **flat** array, one entry per node below
`tpl`, no entry carrying its children, and a subtree is a selection over it
(`FR-HELP-019`, with `FR-HELP-016` and `FR-HELP-029`). The rationale and the
options rejected — nesting; publishing the surface inside the entry of
`tpl render`; a document of its own — are recorded there and are **not restated
here**.

**Why the entry exists.** Both shapes were unstated, and the emitter could not
be written until each was one answer: three keys listed *in that order* read as
the whole of what `data` may carry, and the `path` on every entry read as a flat
array while *the subtree rooted at* read as a nested one. Each was reachable
only by deriving the document from the corpus. The questions were raised by this
folder's implementation and answered by the functional owner, in that order,
which is the order a contract shape may never be settled in reverse.

**What the binary derives under them.** Each row is the implementation's, not
the requirement's, and each is what a reader of
[interfaces.md](interfaces.md#the-help-surface) meets as built.

| Derived | Ground |
|---|---|
| The reduction of `FR-HELP-029` is a **pre-order walk from the node the path resolved to**, not a filter over the unreduced array | The walk yields exactly the entry whose `path` is given together with every entry extending it, in the order `FR-HELP-019` fixes, so the ordering obligation of `FR-HELP-023` is met by the container rather than by a sort |
| The root is the one node carrying no entry | `FR-HELP-019` publishes every node **below** `tpl`, and what the root's `options` would hold is `data.global_flags`, carried once (`FR-HELP-018`, `FR-GLOB-003`) |
| `inherits_globals` is emitted **after** the seven members `FR-HELP-019` names | It is not one of the seven, and last is the one position that leaves every one of them where the requirement puts it. **Rejected**: beside `options`, the member it qualifies, which reads well and interleaves an eighth member into a sequence a requirement fixes |
| A line of an example carries `text` — the line as a caller types it, shell included — and `invocation`, the argument vector, `null` where the line carries no `tpl` call | `BR-HELP-003` parses the published vector itself, so what is published is what is parsed. **Rejected**: publishing the typed table's own prefix, invocation and suffix, which exposes a layout the renderer needs and obliges every consumer to reassemble the line the caller types |
| The three arrays of `registered` are derived from the registrations, and never restated | `FR-ENV-005` requires exactly that. They stood at `null` while `render/` was unwritten, which is what `FR-OUT-012` gives for a value that is absent rather than empty; since `render/` was built they carry eleven filters, seven tests and five functions (read from `tpl help --format json`, 2026-09-21). **Rejected**: restating the names of `FR-ENV-006`, `FR-ENV-007`, `FR-ENV-014` and `FR-ENV-020` here, which creates the second source `FR-HELP-021` exists to prevent and asserts a surface the binary does not have |
| `inherited.tests` and `inherited.functions` are empty arrays, and the three arrays of `other` are `null` | `FR-ENV-005` fixes all five, permanently: group 2 enumerates filters alone, and group 3 is what the other two do not name and `tpl` cannot enumerate |

**Recorded divergence — `inherited.filters` was `null` where the corpus fixes
its names, and is discharged.** `FR-ENV-005` ties **`registered`** to the
registrations the environment performs and says nothing of the kind about
`inherited.filters`: it requires that array to carry the fourteen names of
`FR-ENV-018`, in the order that requirement states them, and admits `null` only
*where the group cannot be enumerated*. Group 2 can be enumerated — the corpus
enumerates it — so the `null` the binary published was a divergence and not the
absent value `FR-OUT-012` permits. Its cause was the same as `registered`'s: the
implementation read all four arrays as `render/`'s, and `render/` was unwritten.
Reported at commit `f8f335d`, 2026-09-15. **Discharged at commit `243c4d6`**:
`tpl help --format json` publishes the fourteen names under
`data.template_surface.inherited.filters` (read 2026-09-21), so a caller learns
which filters group 2 holds as well as that it is `pinned`. The record is kept
rather than deleted, so that an identifier resolves to what happened.

**Unblocks.** `interfaces`.

---

## OD-30 — A parsed leaf with no implementation

**Status: settled. The interim arrangement is discharged**: its last arm,
`tpl cfg database test`, was replaced at commit `a7fb45b` of 2026-09-22, and
`fr_err_030_no_leaf_of_the_tree_answers_with_the_interim_seventy_any_longer` in
`src/cli.rs` asserts that no leaf answers with it (read at `c360c80`,
2026-09-25). What follows is kept as the record of the arrangement.

**Decision.** Every node of the tree parses from the sprint that declares it. A
leaf whose work belongs to a later sprint returns the violated-invariant
condition of `FR-ERR-030`, naming its own command path, and the process exits
`70`. There is **one arm per leaf**, never one catch-all, so each later sprint
replaces its own and finds it by the path rather than by reading.

**Why the tree is complete before the commands are.** `FR-CLI-002` closes the
tree, `FR-HELP-021` derives the published document from the tree the binary
parses with, and `BR-HELP-003` makes every command, alias and flag of that
document a contract. A tree grown command by command would publish a surface
that is a subset of the specified one, and the three tests of `BR-HELP-003`
would pass over the subset; the equivalences of `FR-HELP-002` would hold only
where a node existed. The published surface is what a calling agent reads before
it invokes anything, per `FR-HELP-016`, so it is the part that cannot wait.

**Why `70`.** `FR-ERR-030` gives it to a violated internal invariant the system
detects and declines to continue past, and a command the parser accepts and the
program cannot execute is exactly that: not caused by the command line, not
correctable by the caller. The condition is raised through the one guard that
decides an invariant violation is a `70` and names where it was detected
(`FR-ERR-031`, `FR-ERR-034` row `70`) — the same guard, not the `#[cfg(test)]`
trigger of [`OD-21`](#od-21--two-test-seams-that-must-not-be-on-the-published-surface),
which remains absent from the artefact.

**What a caller observes while it stands.** At commit `fd33cdf` of 2026-09-21:
**one of the 29 leaves exits `70`**, down from 17 in the working tree of
2026-09-17 and 27 at `f8f335d` of 2026-09-15. The one is
`tpl cfg database test`, the only `cfg` leaf that contacts a server
(`FR-CFG-005`); the other 28 act, the third arm among them since this commit.
The six group nodes print their own help and exit `0` (`FR-CLI-007`,
`FR-HELP-025`); the two flag forms are answered at every node (`FR-GLOB-019`,
`FR-GLOB-020`). A `70` from an ordinary invocation is therefore still
**expected** today, on that one path, and is not the defect `FR-ERR-030`
otherwise reports — which is the reason this arrangement is recorded here rather
than left in the code that carries it.

**Rejected.**

- **`todo!()` or `unimplemented!()`.** Both panic. A panic reaches `70` through the hook of [`ADR-004`](../adr/adr-004-release-profile-and-panic-path.md), but it reports the panic path's `cause` and `hint` rather than the condition's, and the project's own completeness rule bars a delivered arm that is not written.
- **One catch-all arm for every unimplemented leaf.** The sprint that implements a command would have to find its arm by reading the match rather than by its path, and nothing would say which arms remain.
- **Declaring a node only once its command is implemented.** It makes the tree a subset of `FR-CLI-002`'s, and the document, the help and the three tests of `BR-HELP-003` all true of the subset.
- **`64` or `78`.** Both tell the caller to change something. Nothing a caller can write reaches an implementation that does not exist, and `FR-ERR-002` forbids collapsing two conditions onto one code where the caller's next step differs.

**What removes it.** Each sprint that implements a leaf replaces that leaf's
arm. The entry is discharged when no arm remains, and it is the only entry of
this register whose discharge is measured in code rather than in a decision.

**Unblocks.** Nothing. It records an arrangement `architecture` and `interfaces`
cite.

---

## OD-31 — The model's shape: strings, fields, and the attribute

**Status: settled**, on 2026-09-18, when `model/` was built.

**Decision.** Four of the five library-shape questions `DIV-032` hands to
architecture, answered over the published surface and over nothing else.

| Question | Answer |
|---|---|
| Owned or borrowed strings | **Clone-on-write**, under **one** lifetime parameter threaded through every type of the graph |
| Public fields or accessors | **Public fields** where every field is an independent fact; **private fields and one constructor** where a value relates two of them |
| Newtypes for names | **None.** A name is the string type above |
| `#[non_exhaustive]` | On every published type **except the two that are inputs** a caller has to be able to write down |

The fifth question — whether the serialisation crate is a public dependency — is
[`OD-18`](#od-18--serialisation-key-order-and-the-two-omissions)'s and is
answered *yes*.

**Why clone-on-write, and why one lifetime.** `specification/catalogue-coverage.md`
promises that *the model is the same whatever the source*, and there are three
sources: a live read, which borrows the row buffers; a document read back from
disk, which borrows the bytes where the encoding allowed it and owns the value
where it did not; and a value built with nothing behind it. One type serves all
three only if a string can be either. The document is what makes the *either*
necessary rather than merely cheap. A deserialiser can hand out a borrowed
string only where the string is present verbatim in the input — "Deserializers
commonly use transient data … when escape sequences are being processed so the
resulting string is not present verbatim in the input" (serde.rs,
*Understanding deserializer lifetimes*, consulted 2026-09-18) — and JSON
requires the quotation mark to be escaped inside a string (RFC 8259 § 7,
consulted 2026-09-18). A raw column type such as ``enum('8''6"')`` therefore
arrives **owned** while its neighbours arrive borrowed, from one document, in
one pass.

**Why a constructor on three types and on no others.** A constructor earns its
place where a requirement relates two fields, and only there.

| Type | The invariant it establishes | Forced by |
|---|---|---|
| A table | No key names a column the column list does not carry | `FR-CAT-044` |
| The `server` object | On the probe path, `series` is the leading two components of `version` and can be populated from nothing else. There are **two** constructors, because `FR-CTX-033` bars the same check on a supplied document, and a second constructor is what states that in the type rather than in a comment | `FR-SRV-040`, `FR-CTX-031`, `FR-CTX-033` |
| The `restricted` marking | It names at least one property; the empty marking has no representation | `FR-PRIV-016` |

Every other type is plain data: each field is one catalogue fact, there is
nothing for a constructor to check, and private fields would buy an accessor per
field and no property.

**Why the attribute is not on the two inputs.** `#[non_exhaustive]` obliges a
downstream construction to go through a constructor, which is exactly right for
a value the crate **produces** and exactly wrong for one a caller **writes
down**. Two types are written down — the catalogue's statement of a column type,
and the parts a table is assembled from — and each produces a value that carries
the attribute for both.

**Rejected — owned strings everywhere.** The simplest shape, and it costs a copy
of every catalogue string on the dominant path. It also does not remove the
lifetime: the decomposed type and the classified default both borrow, so a
column carrying either carries their lifetime, and an owned graph would need
owned twins of both — the second parallel type this shape exists to avoid.

**Rejected — borrowed strings everywhere.** Cheaper still, and it cannot
represent two values the model already produces: a string literal whose doubled
apostrophe was collapsed, and a document string that needed unescaping. Neither
is a slice of its source.

**Rejected — a type parameter for the string type.** It defers the choice to
every caller, infects every type with a parameter, multiplies the
monomorphisations, and leaves `render/` with two concrete models to be written
against instead of one.

**Rejected — an arena the whole graph borrows from.** It removes the per-string
allocation on the owned path, at the price of a dependency the budget would have
to justify and of a construction path the derived deserialisation of
[`OD-18`](#od-18--serialisation-key-order-and-the-two-omissions) cannot express:
the arena would have to be threaded through as a seed, which is the plain derive
this choice keeps available.

**Rejected — a newtype per name.** A *validated* name is a different thing from
an arbitrary string, and there is no validation here to hang one on. `FR-CAT-042`
records that the catalogue returns identifiers **unescaped**, hostile ones
included, and `FR-ENV-045` makes quoting a name for a target dialect the
**template**'s job; no requirement of the corpus puts a grammar on a name the
model carries. A newtype would state an invariant nothing establishes, on every
name, in both directions.

**Scope.** This entry answers the shape of the model's types. It does not settle
the library's compatibility, which `DIV-032` withdraws, and it reaches no module
but `model/`.

**Unblocks.** `interfaces`.

---

## OD-32 — `anyhow` in the shipped graph

**Status: settled 2026-09-21, by the user. Fully discharged 2026-09-22.** The
crate left the manifest, and the correction this entry prepared for `CLAUDE.md`
was applied by the user; nothing in it is outstanding.

**Decision.** **`anyhow` is removed from the dependency graph.** Nothing
replaces it: the one error the binary handles is already handled without it.

**The ground: the crate was declared and used by nothing.** `CLAUDE.md`
*Desempenho e Eficiência* admits a crate for what it does and refuses one used
for a trivial function; this one was used for no function at all. Four readings
established it, all at commit `243c4d6` on 2026-09-21, before the removal.

| What was read | How | What it returned |
|---|---|---|
| The declaration | `Cargo.toml`, line 16 | `anyhow = "1.0.104"`, a direct dependency of the package |
| Who else needs it | `cargo tree -i anyhow`, and again with `--all-features` | `anyhow v1.0.104` with `tpl` as its only parent, identically in both readings: no other crate in the graph pulls it in |
| Whether the crate names it | `grep -rn anyhow src/ tests/` | No occurrence. `benches/` and `examples/` do not exist at this commit |
| What the binary does with an error | `src/main.rs`, read whole | `tpl::install_panic_hook()`, then `tpl::run()`, then `ExitCode::from(error.exit_code())`. There is no other error handling in the file |

[`OD-06`](#od-06--the-error-types-shape-and-the-exit-code-derivation) predicted
this. With `main.rs` reduced to calling the library, reading `exit_code` and
returning it, the binary carries no dynamic error, and a crate whose purpose is
to carry one earns nothing against the budget.

**Rejected — keeping it declared as a reserve** for a dynamic error the binary
might one day have to carry. Refused because `OD-06` gives the binary no route
to classify anything: the exit code comes from an inherent method on the
library's error, the enum is `#[non_exhaustive]` so a match written in the
binary would need the wildcard arm that entry refuses, and the four labelled
lines are composed by a renderer inside the library. The reserve is for a
capability the binary may not have, and a dependency held against a use nobody
can name is the dependency the budget exists to refuse.

**`thiserror` is untouched, and no error handling leaves the crate.** It derives
the one public `#[non_exhaustive] enum Error` and its `Display`, which is
`OD-06`'s decision and is unchanged by this one. What is removed is a crate the
source does not name; the library's error type, its exit-code method and the
diagnostic renderer are all where they were.

**The MSRV is `ADR-007`'s.** [That record](../adr/adr-007-msrv.md) holds the
rule, the graph the rule runs over and the crate that sets the figure. It has
since re-run that rule over the graph this removal left, on 2026-09-22, and
carries the result; running it was that record's work and not this entry's, and
this entry restates none of it.

**The correction prepared for `CLAUDE.md` was applied on 2026-09-22, by the
user.** That file named the crate twice — in the *Stack* table's error row and
in the *Tipos e erros* convention that repeats it, at lines 465 and 539 — and
both named it as the binary's error type. Both now state the positive choice:
`thiserror` in the library, and no error type of the binary's own. The table's
row cites this entry for the reason, so the coordination document points here
rather than restating the ground. `grep -n anyhow CLAUDE.md` returns no match
(2026-09-22). **This folder never edits that file and the user is its only
writer**, so the correction was prepared here and applied there, which is the
same handover [`OD-17`](#od-17--observability) made for `tracing` and
`tracing-subscriber` over the same table.

**The manifest change was made, in the order this entry fixed.** `CLAUDE.md`
*Stack* makes an alteration to that table an architecture decision, to be
registered before it is implemented; this entry is the registration, and the
crate was removed from `[dependencies]` afterwards, at commit `455e48d`.
`grep -n anyhow Cargo.toml Cargo.lock` returns no match and
`cargo tree -i anyhow` reports no such package (2026-09-22), so the crate is out
of the manifest, out of the lock file and out of the resolved graph.

**The one disagreement this order cost is closed.** Between the manifest change
and 2026-09-22 the coordination document named a dependency the graph no longer
carried — the price of the correction being the user's alone, recorded here
while it stood rather than left for the next reader to find. The user closed it
the same sprint. What `ADR-007` had to do about the removal is that record's and
is cited above, not restated.

---

## OD-33 — `UC-013` and the twelve-flow acceptance skeleton

**Status: open.** Owner: the user, because one of the two options reaches the
mandated validation pipeline, which `CLAUDE.md` *Desenvolvimento* fixes and this
folder does not own.

**The question.** `specification/use-cases.md` now holds thirteen flows.
[verification.md](verification.md#the-twelve-end-to-end-flows) names twelve as
the acceptance-test skeleton, and [README.md](README.md#verificationmd) and
section 16 of [traceability.md](traceability.md) count the same twelve. Does
`UC-013` become the thirteenth, or does the skeleton stay at twelve for a stated
reason?

**Why it is a decision and not a harvest.** The corpus does not settle it.
`specification/use-cases.md` *Overview* says the flows exist to show how the
requirements compose and to give each help text a source for its `EXAMPLES`
section; it does not make them acceptance tests. That reading is this folder's,
and so is the count. What makes `UC-013` different from the twelve is its
acceptance signal: `FR-EX-009` puts it in a compile gate over four target
languages, run by the worked example's own script, where the twelve are each
placed under one of the two kinds that table names — *Integration* and *Server*
— both of which `cargo test` runs.

| Option | What it would cost |
|---|---|
| **A — `UC-013` joins as a thirteenth flow.** The table gains a row and the skeleton is the whole of `specification/use-cases.md` | The register of mandated tests would name a test that `cargo test --all-features` does not run, so *Kind* gains a third value beside *Integration* and *Server* and the pipeline that decides whether a mandated test passed is no longer the mandated pipeline alone. It would also bring four external toolchains — the four of `examples/README.md` — inside this folder's verification surface, each with a floor to state and to keep current |
| **B — the skeleton stays at twelve, closed by a stated criterion rather than by a count.** `UC-013` is named as excluded, with the criterion: a flow enters the skeleton when a test in `tests/` can assert its postcondition | The exclusion has to be stated wherever the count appears — `verification.md`, `README.md` and the section 16 row of `traceability.md` — or the folder reads as unharvested against a corpus that has thirteen. And the one flow that exercises whether the commands compose into a delivered artefact would have no entry in the register of mandated tests, so nothing in this folder would say what verifies it |

**Neither option is taken here, and the twelve are untouched meanwhile.** No row
of the skeleton is renumbered, reworded or removed by this entry; what the three
documents say today they say against `UC-001` … `UC-012` and remains true of
those twelve.

---

## OD-34 — An in-process entry point over a supplied argument vector

**Status: settled.** Decided for rmp `#257` on 2026-09-24; `adr-guardian`
judged that no architecture decision record is admissible under rule R4 of
[`docs/adr/README.md`](../adr/README.md), so the rejected options live here.

**The question.** `run()` read `std::env::args_os()` and nothing else. Under
libFuzzer those arguments are the fuzzer's own, so a coverage-guided harness
stopped at the parser and reached no other code (`SECURITY-AUDIT.md`,
*Limitations*). What public entry point lets an in-process caller drive the
whole invocation?

**Decision.** `pub fn run_from<I, T>(args: I) -> Result<(), Error>`, with
`I: IntoIterator<Item = T>` and `T: Into<OsString>`, and `run()` reduced to
`run_from(std::env::args_os())`. The obligations on it are
[interfaces.md](interfaces.md#the-library-entry-points)'s.

**Rejected.**

- **The name `run_with`.** In the standard library a `_with` suffix marks a
  closure argument — `Vec::resize_with` takes `f: F` where `F: FnMut() -> T`
  (doc.rust-lang.org, `std::vec::Vec`, consulted 2026-09-24) — and this function
  takes a value.
- **Returning `ExitCode`.** It moves the mapping from error to exit status into
  the library, which `OD-06` leaves to `main.rs` over `Error::exit_code`.
- **A `Clone` bound on the iterator or its items.** `run_from` collects the
  vector once and nothing iterates `args` twice, so the bound would constrain
  callers for nothing.
- **A working-directory argument.** Discovery reads the process's current
  directory under `FR-PROJ-004`; threading a directory through every reader is a
  change to discovery, outside the question.
- **An entry compiled only under `#[cfg(fuzzing)]`.** The entry serves a test
  as well as a fuzz harness, and a test build does not set that configuration.
- **A harness that spawns the binary per input.** It is the black-box fuzzing
  already delivered, one process per input; it gives the fuzzer no coverage
  feedback from inside the process, which is the gap this entry closes.

**Consequence.** The once-per-process state and the two exits that do not
return make repeated calls in one process differ from repeated invocations of
the binary; the entry states them rather than removing them, and a caller runs
each input accordingly.

---

## Editorial defects, reported and corrected

Two statements in `specification/` were stale when this register was written.
Both were corrected in the eighth edition, at commit `9efa791` of 2026-09-10,
and neither is outstanding. No document of this folder ever relied on either.

| Id | Where | Defect reported | Correction verified 2026-09-11 |
|---|---|---|---|
| **ED-01** | `glossary.md`, entry *DSN* | The form ended `[?params]`, which `FR-CONF-009` removed in the fifth edition and of which `FR-CONF-011` admits nothing | The entry gives `scheme://[user[:password]@]host[:port]/database` and states that it carries no query parameters, per `FR-CONF-011` |
| **ED-02** | `upstream-divergences.md`, `DIV-034` | The body said thirteen volatile catalogue fields were excluded by `FR-CAT-024` while its own seventh-edition note said sixteen | The clause reads sixteen, the note beside it records the growth from thirteen as history, and both agree with the sixteen rows of `FR-CAT-024`'s table |

Both identifiers are retired rather than deleted, and neither is reused: a
reader who meets `ED-01` or `ED-02` in the history is bounced here rather than
left hunting for an open defect.

## What remains

One entry is open and one obligation stands. Each blocks one statement, and
neither blocks the other.

| Order | What | Owner | Blocks |
|---|---|---|---|
| 1 | `OD-19`'s owed observation — `ADR-009`'s *owned copy* against a value whose members are borrowed, and the peak-memory consequence the record draws from that word | `adr-guardian` | No document of this folder may read *owned copy* as settling how an embedded value holds its members |
| 2 | `OD-33`'s question — whether `UC-013` joins the acceptance skeleton, on the two options that entry states | user | No document of this folder may present its flow count as closed against the corpus; the twelve stand as written and `UC-013` is neither added to them nor named as excluded until this is answered |
`OD-14`'s obligation, which stood first in this table, was discharged on
2026-09-21: the observation was made, it went the other way, and the entry
records both the answer and what in `render/` now carries the two requirements
it turned out not to decide. `architecture` asserts the behaviour accordingly.

`OD-22`'s residual — the fixture certificate and the harness — was discharged on
2026-09-11 by tasks #15 and #25, and `operations` and `verification` no longer
wait on it.

The two obligations this table carried for `specification-manager` are
discharged: the ninth edition amended `FR-ERR-030` (`OD-28`) and gave
`FR-CONF-005` the phase-to-key mapping with its shared connection budget
(`OD-12`). Each entry records what landed, and neither is reopened.

**No obligation now falls to `verification` from this register.** The phase
attribution of a TLS handshake failure (`OD-12`) was discharged on 2026-09-18,
against the fixture, and that entry records what was observed and what was
reasoned; the second, one test per mapped `clap::ErrorKind` (`OD-08`), was
discharged at commit `f8f335d` of 2026-09-15. Each is recorded as discharged in
its own entry.

**No correction is owed outside this folder.** The last one standing was
`OD-24`'s amendment of 2026-09-24, over the `rustix` row of `CLAUDE.md`'s
*Stack* table at line 475, applied the same day. All six corrections this
register has prepared for that file are applied, and each records where and
when under [Corrections owed to `CLAUDE.md`](#corrections-owed-to-claudemd).

**`OD-27`'s consequence is discharged.** `scripts/mariadb/seed-bench.sql` was
written on 2026-09-21, and `quality-attributes` and `verification` no longer
wait on it.
