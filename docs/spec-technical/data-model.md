---
title: Data Model
status: draft
last-reviewed: 2026-09-25
related: [README.md, traceability.md, open-decisions.md, overview.md, quality-attributes.md]
---

# Data Model

## What this document is

What enters the model and by what rule, and everything the process persists:
`.tpl/.cfg`, `.tpl/.cache/`, `meta.json`, the four version numbers, and what a
migration is. Every statement cites the requirement that forces it; no
requirement text is reproduced here.

**The JSON document contract is not here.**
`specification/context-document.md` owns the document's keys, depths and cuts,
and `specification/output-formats.md` owns the envelope every document shares.
This file cites both and reproduces neither. The emitter that applies them is
`interfaces.md`; the components that produce them are `architecture.md`; the
memory cost of the shape is
[quality-attributes.md](quality-attributes.md#peak-memory-and-the-two-embeddings).

The term *model* is `specification/glossary.md`'s and is used here unchanged.

## The covered set as a predicate

Coverage is a predicate over the catalogue's table-type string, not a list of
objects. `FR-CAT-031` fixes the six strings a supported series can emit, three
of which are covered — `FR-CAT-001` for the two table types, `FR-CAT-007` for
views — and three excluded, by `FR-CAT-004`, `FR-CAT-005` and `FR-CAT-006`.
Routines are covered by `FR-CAT-008` and reached through a catalogue table of
their own. `FR-CAT-002` puts the surviving distinction in the model, so a
template reads which of the two covered table types it has rather than
inferring it.

| Obligation on the built system | Forced by |
|---|---|
| The predicate is evaluated against the string, and a temporary table is filtered rather than assumed absent — one supported series omits it and three report it | `FR-CAT-006`, `FR-CAT-032` |
| The same predicate is applied to the **column** read, not only to the object read; the column catalogue is not restricted to covered objects | `FR-CAT-052` |
| A view never appears among tables; the three covered kinds are three collections | `FR-CAT-003`, `FR-CTX-035` |
| Coverage is applied **before** `--pattern`, so a count is attributable to one or the other and never to their interaction | `FR-CAT-028`, `BR-CAT-003` |

The five features excluded whole are
[overview.md](overview.md#what-is-excluded-from-the-model-by-decision)'s; the
order in which the reader evaluates the predicate against the query it sends is
`architecture.md`.

## From a catalogue field list to a model property list

`BR-CAT-005` is the derivation rule: carry every field the catalogue provides
for an object, less the four stated grounds of exclusion, with *nothing
observed* named as a fifth case that is not one. The rule is cited, not
restated; what it obliges of the built system is that the reader's column list
per kind is **specified material**, not an implementation choice, and that a
field admitted later needs an observation rather than a decision.

Seven object kinds have their catalogue field list fixed by a requirement:

| Kind | Field list fixed by |
|---|---|
| Index | `FR-CAT-042`, folded from one row per column into one object with an ordered column list, per `FR-CAT-010` |
| Primary key | `FR-CAT-043`, which names the one authoritative source and forbids the other two |
| Foreign key | `FR-CAT-045`, across two catalogue tables, the incoming direction filtered from the other end of one of them |
| `CHECK` constraint | `FR-CAT-046` |
| View | `FR-CAT-047` |
| Routine | `FR-CAT-048`, and its parameters `FR-CAT-049` |
| Trigger | `FR-CAT-050` |

One invariant spans all of them: `FR-CAT-044` forbids a key to name a column
absent from the same table's column list, which the catalogue does not supply
for free. It is an emitter obligation and is `interfaces.md`'s to realise.

**Recorded gap — two kinds have no field list.** No requirement fixes the
catalogue field list for the **table** row or for the **column** row. Their
property lists therefore follow from `BR-CAT-005` applied to an observation the
corpus does not enumerate, supplemented by the requirements that name
individual properties — `FR-CAT-002`, `FR-CAT-009`, `FR-CAT-039` and
`FR-CAT-041` for the table and column rows, `FR-CAT-051` for a generated
column, `FR-CTX-014` through `FR-CTX-022` for a column's type, identity and
derived facts. This folder may not close the gap: re-deriving a catalogue fact
is forbidden to it (`specification/README.md` *Provenance*, item 4, carried in
[overview.md](overview.md#what-this-document-defers-and-to-what)). The two
missing lists are an observation pass whose result belongs to
`specification/catalogue-coverage.md`, and the gap is reported to its owner
rather than filled here.

Behaviour the reader encodes rather than rediscovers is fixed by `FR-CAT-033`
through `FR-CAT-038` and `FR-CAT-040`, and each is cited at the point it binds
a component in `interfaces.md`.

## The two closed exclusion lists

`BR-CAT-004` keeps them apart because their grounds, their tests and their
futures differ. Both are closed: a field is excluded by appearing on its list
and by nothing else.

| List | Ground | Test | Future |
|---|---|---|---|
| `FR-CAT-024`, sixteen fields | The server changes the field without the structure changing (`BR-CAT-002`) | Two reads of an unchanged database differ; one server suffices | Permanent. `FR-CAT-025` closes it |
| `FR-CAT-029`, **empty today** | Two supported series disagree about the field's meaning (`FR-SRV-025`) | Needs all four series at once | Expires as the window of `FR-SRV-001` moves. `FR-CAT-030` closes it and fixes how a field leaves |

Two consequences for the built system:

- The exclusion binds **every** consumer — the model, both output formats, the
  dump, the cache and the render context (`FR-CAT-026`) — so it is enforced
  where the model is built and nowhere else. A per-consumer filter would be
  four places to forget.
- The auto-increment **column attribute** stays, and only the table-level
  counter is excluded (`FR-CAT-027`); the two are different fields of different
  rows.

An entry on the second list without a row in the divergence register of
`FR-SRV-036` is a defect, per `FR-CAT-029`: the exclusion is normative there
and the evidence for it lives in the register.

## A column type: eight parts and the raw string

`FR-CTX-014` keeps the raw type string and `FR-CTX-015` adds the eight
decomposed parts, each read from the catalogue field `FR-CTX-040` names for it
and from no other. Four rules fix what the built system may and may not do with
them:

| Rule | Forced by |
|---|---|
| A part is `null` **exactly when** the field behind it returns SQL `NULL`. Applicability is never decided by `tpl`, so a type the corpus has not seen produces the parts the catalogue populates and no others | `FR-CTX-017`, `FR-CTX-041` |
| The two precision fields merge into one part, and the merge is lossless because no type populates both | `FR-CTX-040` |
| `unsigned` is derivable **only** from the raw string; the catalogue has no field for it | `FR-CTX-038` |
| An `ENUM` or `SET` member list is read from the raw string by quote state, never by splitting on the comma | `FR-CTX-039`, with `FR-CAT-034` |

**The raw string is the safety net, and it is the only one.** `FR-CTX-018`
keeps `column_type` intact for an unrecognised type and nulls every part, so a
type introduced by a later server reaches the template rather than being
guessed at. The column **default** has no equivalent: `FR-CTX-012` rejected a
raw field beside the classified structure, and `FR-CTX-037` records the
residual risk that follows. The asymmetry is deliberate and is not to be
repaired by adding a raw default field here.

## A column default: one discriminant, eight ordered rows

`FR-CTX-011` makes a default a structure or `null`; `FR-CTX-013` closes the
discriminant at three values; `FR-CTX-012` fixes the four emitted forms.
`FR-CTX-037` is the classification: eight rows over the raw catalogue value,
applied **in the order written**, first match wins, with the `kind` and the
`value` derived from that classification alone.

Three obligations on the built system follow, and all three are about order and
totality:

- The classifier is a single ordered pass. Reordering the rows changes the
  answer, so the order is part of the code's contract and is tested as such —
  `verification.md`.
- The final row makes the classification **total**: a value matching no earlier
  row is an expression. `FR-CTX-037` states why that direction fails safely,
  and no other default may be invented for it.
- A doubled apostrophe collapses to one, and there is no backslash escape, in a
  literal default and in an `ENUM` member alike (`FR-CTX-037`, `FR-CTX-039`).
  One unescaping rule serves both, which is what keeps them from drifting.

## The `database` object

`FR-CTX-036` fixes three metadata fields, `FR-CTX-031` a `server` object of
exactly three keys, and `FR-CTX-035` three collections. One property of the
built system is decided by where each comes from:

| Part | Source | Consequence |
|---|---|---|
| The three metadata fields | The schema catalogue row | Read with the rest of the catalogue |
| `server` | The version probe of `FR-SRV-002`, **not** the catalogue | Available before any catalogue read, and derived from a statement of the closed list of `FR-SRV-006` (`BR-CTX-006`) |
| `series` | Derived from `version` as `FR-SRV-040` requires, by `<major>.<minor>` alone | A field of its own; a template cannot derive it, per `FR-CTX-032` |
| `standing` | The resolved series against the window of `FR-SRV-015` | Unconditionally present, because `FR-SEM-012` fails a render that reads an absent field, so a conditional marker would fail the guard that looks for it (`FR-CTX-034`, `BR-SRV-008`) |

A cached `server` object describes the read that produced it and not any server
reachable now (`BR-CTX-006`), which is the same statement `FR-CDOC-015` makes
about the rest of a cached document.

## The four treatments of a cross-series difference

`BR-SRV-006` makes the set exhaustive over differences of presence, of
representation and of value. Which treatment a field receives is a property of
the model and is decided once, where the model is built.

| Treatment | Applies to | Fixed by |
|---|---|---|
| Normalise | A fact present on more than one series and reported in a different place, name or spelling | `FR-SRV-024` |
| Mark `null` | A field the connected series does not provide at all; composes with normalisation over one field | `FR-SRV-004` |
| Exclude | A field whose **meaning** differs and cannot be normalised; entered on the closed list of `FR-CAT-029` | `FR-SRV-025` |
| Pass through | A field present, identically named and identically meant everywhere, whose **value** differs because the servers' own defaults differ — every character-set and every collation value | `FR-SRV-039` |

The fourth treatment is the one place the corpus prefers fidelity to the server
over determinism across servers, and `FR-SRV-039` states the cost. What keeps
that preference from swallowing the equivalence promise is
[quality-attributes.md](quality-attributes.md#cross-series-equivalence-and-its-exceptions),
which owns the promise and its three exceptions.

**A difference between the series' field lists does not by itself reach the
model.** `FR-CAT-049` is the observed case: the routine-parameter list is 16
fields on three series and 17 on one, and the seventeenth is not carried —
*nothing observed*, under `BR-CAT-005` — so no treatment applies, no row is
owed to the register of `FR-SRV-036`, and nothing in the model is marked. What
the difference does bind is the statement the reader sends: `FR-SRV-037`
requires a fixed column list to be common to all four series or selected per
series, because naming an absent column fails hard at the server. That is
`interfaces.md`.

## The model in memory

| Question | Answer | Recorded in |
|---|---|---|
| Are the two foreign-key embeddings owned copies, or reproduced at serialisation time | Materialised. The object graph **is** the document | [`ADR-009`](../adr/adr-009-foreign-key-embedding-representation.md), registered as [`OD-19`](open-decisions.md#od-19--whether-the-two-embeddings-are-materialised) |
| How is a document produced from it | Derived `Serialize` through `serde_json`; the struct's field declaration order **is** the key order of `FR-OUT-013`, stated once, in the type | [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions) |
| What is never materialised | Any fact the table already states. `is_primary_key` and `is_unique` are not fields; the two tests resolve `table_name` against the render context instead | `FR-CTX-021`, `FR-CTX-022`, `BR-CTX-003` |
| What the model does not record | Which reader produced it. Completeness is a property of a **read**, not of a server, so the same database read by two users yields a complete model for one and a marked one for the other, and nothing distinguishes them afterwards | `BR-PRIV-003` |

Neither record's rationale is restated here, per rule R3 of
[`docs/adr/README.md`](../adr/README.md). One consequence of the first row
bears on the rest of this document: because the graph is the document, a
marking under `FR-PRIV-005` is a property of the object it qualifies, which is
what lets `FR-CACHE-037` keep a marked object out of the cache one object at a
time rather than discarding the read.

Owned versus borrowed types, newtypes for names, public fields versus
accessors, and `#[non_exhaustive]` are four of the five shape questions
`DIV-032` hands to architecture; they are `interfaces.md`'s and are not decided
here.

## `.tpl` on disk, and its five writers

`FR-PROJ-001` makes any directory containing `.tpl` a project;
`FR-PROJ-002` fixes what `.tpl` holds; `FR-PROJ-003` divides it into what is
versioned and what is not. Two of the three parts are local to the machine and
are dotfiles; the one part that travels with the repository is not
(`FR-CACHE-001`).

`FR-PROJ-023` names five writers and no others, and `FR-PROJ-024` admits one
exception outside `.tpl`:

| Writer | What it writes |
|---|---|
| `tpl init` | The five artefacts of `FR-PROJ-017`, plus the destination directory and its missing parents (`FR-PROJ-013`) — the one write outside `.tpl`, and directories only |
| `tpl cfg …` | `.tpl/.cfg` |
| `tpl cache load` | `.tpl/.cache/` |
| Any cached read command, on a miss | `.tpl/.cache/` |
| `tpl cache clean` | Removes from `.tpl/.cache/` what `FR-CACHE-023` and `FR-CACHE-041` name, and rewrites `meta.json` under `FR-CACHE-043` |

The fourth writer is why `BR-PROJ-002` denies that a read command is read-only
with respect to the filesystem unless it is invoked with `--direct --no-cache`.
`DIV-005` is the register entry that carried this count against the root
coordination document, discharged by commit `0ea5624`; it is cited, not
restated.

**Recorded discrepancy — the temporary files.** `FR-PROJ-002` reads that `.tpl`
holds *"exactly the artefacts above and nothing else that `tpl` reads or
writes"*, and the layout it refers to names no temporary file. `FR-CFG-041`
nonetheless requires a rewrite of `.cfg` to write a temporary file **in
`.tpl/`**, and `FR-CACHE-030` requires each cached object to be written through
a temporary file in its own directory. Both readings are recorded. This folder
takes the specific requirements as governing and reads `FR-PROJ-002` as an
enumeration of the **durable** artefacts, because a rename is atomic only
within one filesystem and therefore only from a sibling path. Two consequences
are prescribed so that the two readings cannot diverge in practice: a temporary
file exists only within the invocation that creates it, and a failed write
removes it. The wording is the functional owner's to settle.

## `.tpl/.cfg`

| Property | Decision | Forced by |
|---|---|---|
| Format | TOML, one file. No global configuration, no home, XDG or `/etc` fallback, so the project alone determines behaviour | `FR-CONF-001`, `FR-CONF-003`, `BR-PROJ-001` |
| Key space | Exactly eighteen key forms, eight under `[core]` and ten per database entry, each with a declared type and a declared default. One table serves two jobs: the validator's population, and the source of every built-in default a phase deadline or a render bound resolves from | `FR-CONF-002`, `FR-CFG-009`, `FR-CFG-010`, `FR-CONF-004`, `FR-CONF-045` |
| The three render-bound keys | Integers within the range `FR-CONF-002` declares for each; none admits `0` or a value meaning "no bound". A value outside it is `78` from the file and `64` from `tpl cfg set`. Each range is a type in `render/bounds.rs`, so an out-of-range value cannot be held once read | `FR-CONF-045`, `FR-CFG-010` |
| Mode | Created `0600`; refused at any looser mode and when not owned by the current user; retained at `0600` across every rewrite, including the temporary file | `FR-PROJ-019`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-CFG-034`, `FR-CFG-041` |
| Rewrite | Temporary file in `.tpl/`, renamed over the target; a failure part-way leaves the previous file unchanged and removes the temporary; **no lock**, so two writers yield one whole file or the other | `FR-CFG-041`, `FR-CFG-042` |
| Read path and write path | Separate: a span-carrying document tree to read the typed key space, a format-preserving editor to write | [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path) |
| Opening the file | Opened once, relative to `.tpl`, with `O_NOFOLLOW` and `O_NONBLOCK`, in `src/project/trust.rs`. Its type is tested on the descriptor: anything but a regular file — a symbolic link and a socket among them — is `78`, and a FIFO does not block. The ownership and mode checks run on the same descriptor's metadata, and the text is read through the same descriptor, so the file judged is the file read | `FR-PROJ-030`, `FR-PROJ-010`, `FR-PROJ-011`, `FR-SEC-027`, [`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid) |

**The temporary file is created at `0600` by the open itself, not chmod'd into
it afterwards.** There is otherwise an instant at which a new file holding the
credentials of the old one exists at whatever the process umask allows. The
mode is then set again on the path before the rename, because a temporary file
left by an earlier run keeps the mode it already had and `FR-CFG-034` requires
`0600` whatever the previous run left behind. The rename is the whole of the
concurrency story `FR-CFG-042` admits: two processes yield one whole file or the
other, and a killed process leaves nothing locked and nothing half-written
(`FR-PROJ-019`, `FR-CFG-034`, `FR-CFG-041`, `FR-CFG-042`).

The write path is format-preserving because six requirements make it a
functional need rather than a courtesy — chiefly `FR-PROJ-017` and
`FR-PROJ-018`, which put a commented-out example entry in the generated file,
and `FR-CFG-020` with `BR-CFG-001`, which require an update to change the named
fields and leave the rest of the entry untouched. A serde round trip would
delete the comment on the first `tpl cfg set`, so a stated requirement would
stop holding on the second invocation, silently. The crates and the versions
are `technology-stack.md`.

**The file is strict in both directions.** An unrecognised key anywhere is `78`
with a suggestion (`FR-CONF-034`), and `password_command` that is not an array
is `78` with the array form in the hint (`FR-CONF-035`). `BR-CONF-004` gives
the ground: `.cfg` is untrusted input that decides which host is contacted,
which credential is used and which child process runs. Two consequences for the
built system: the reader must retain the **line** of the fault, because
`FR-CONF-035` and `FR-ERR-034` both require the diagnostic to name it; and
forward compatibility is given up deliberately, so a binary that does not know
a key refuses the whole file rather than ignoring the key.

**Those two consequences decide the read path's shape, not just its crate.** A
mapping onto declared fields answers *this document does not fit* and cannot
answer *which key* or *at which byte*, because the offending key is precisely
the one no field is declared for. The reader therefore walks the parsed document
as a tree of spanned values, which yields both facts directly; the tree is also
what the literal printing of `FR-CFG-013` needs, since the redaction of
`FR-CFG-021` is spliced into the file's own bytes at the spans the reader
recorded, leaving every comment, key order and space outside a credential
untouched. [`OD-09`](open-decisions.md#od-09--toml-the-read-path-and-the-write-path)
records the choice and what it costs.

Credential handling, `${VAR}` expansion and the child process are
`security.md`; the commands that read and write the file are `interfaces.md`.

## `.tpl/.cache/`

| Property | Decision | Forced by |
|---|---|---|
| Location | `.tpl/.cache/<entry>/`, one folder per database entry | `FR-CACHE-001` |
| Key | The database entry **name** and nothing else — not the host, not the resolved DSN | `FR-CACHE-002` |
| Creation | On the first read that populates it, never by `tpl init`; excluded from version control by the generated `.gitignore` | `FR-CACHE-003`, `FR-PROJ-020`, `FR-CACHE-004` |
| Filenames | The object kind plus the literal object name: `tables/<name>.json`, `views/<name>.json`, `routines/<kind>.<name>.json`. No encoding layer and no hash | `FR-CDOC-014`, with [`OD-10`](open-decisions.md#od-10--cache-filenames-and-the-case-collision) |
| Encoding | JSON, UTF-8, compact, no trailing newline, written by the serialiser of [`OD-18`](open-decisions.md#od-18--serialisation-key-order-and-the-two-omissions) | The `.json` suffix of `FR-CDOC-001` and `FR-CDOC-014`; `BR-CACHE-001` keeps the form outside the contract |
| Write of an object file | One file per object, through a temporary file in the same directory, renamed over the target; **no lock**. A symbolic link at the target is replaced by the rename, never followed and never left in place. `meta.json` differs in one case: a clean given an object flag does not write an unusable `meta.json`, a link among them, and leaves it in place | `FR-CACHE-030`, `FR-CACHE-031`; `FR-CACHE-043` for `meta.json` |
| A link on the path to the cache | `.tpl` is canonicalised first; every component below it is examined without being followed. A symbolic link at `.tpl/.cache`, at the selected entry's folder, or at a collection folder the invocation would read, write or clean beneath it is refused with `78` before anything is read, written, removed or connected. A link that is itself the thing a clean removes, or that lies beneath a folder it removes, is removed as a link and never followed | `FR-CACHE-042`, `FR-CACHE-044`, `FR-SEC-026` |
| Read of an object file | Opened relative to its directory's descriptor with `O_NOFOLLOW` and `O_NONBLOCK`, and its type read from that descriptor: anything but a regular file — a symbolic link or a FIFO among them — is a miss, and a FIFO does not block the open. What is judged is what is then read | `FR-CACHE-033`, `FR-SEC-026` |
| Read of `meta.json` or `database.json` | The object-file guard above, plus a size bound of **1 MiB** (`RECORD_CAP` in `src/cache.rs`), checked on the inspected length and again on the bytes read, so a file that grows after inspection is refused rather than read short. A record that is a link, is not a regular file, or exceeds the bound is unusable: a miss for a read, and not written by a clean given an object flag | `FR-CDOC-017`, `FR-SEC-026`, `FR-CACHE-043` |
| Read of one named object | A hit only where the file holds exactly that object: the same name byte for byte and, for a routine, the same kind. Checked over the member already decoded, so it costs a comparison and no second parse | `FR-CACHE-033`, `FR-CDOC-008` |
| Never written | Any object marked `restricted`; a collection holding one is never recorded whole | `FR-CACHE-037` |
| How every row above resolves a path | **Directory-relative**, in `src/at.rs`. Each component below the canonical `.tpl` is opened relative to its parent's descriptor with `O_DIRECTORY` and `O_NOFOLLOW`. A file is read through `openat` with `O_NOFOLLOW` and `O_NONBLOCK` and typed on the descriptor; a temporary is created with `O_CREAT`, `O_EXCL` and `O_NOFOLLOW` and renamed over its target with `renameat`; removal is `unlinkat`, each child of a removed folder reached without following it; creation is `mkdirat`. A component swapped for a link between two operations fails the next one instead of redirecting it | `FR-SEC-026`, `FR-PROJ-024`, [`OD-24`](open-decisions.md#od-24--the-discovery-boundary-and-the-process-uid) |

**One step resolves a path by name, and it is benign.** A directory is listed
with `std::fs::read_dir` on its path, because `rustix::fs::Dir` needs `rustix`'s
`alloc` feature, which was not added. Every name listed is then examined,
opened and removed relative to the directory descriptor already held, and a
name that descriptor's directory does not hold is dropped. A directory swapped
between the open and the listing can therefore only mislist names **inside**
the directory the descriptor holds; nothing outside it is read, written or
removed. The anchor, the canonical `.tpl` of `FR-PROJ-009`, is opened by path,
with `O_NOFOLLOW` on its last component; redirecting it needs write access
above the project. Both limits were stated by the security review of rmp `#306`
and `#307`, 2026-09-24, as relayed by the session coordinator.

**Evidence.** The swap proof of concept of rmp `#305`'s review, which had
deleted outside the project, was run 10,000 times by the implementer and 5,000
times by the reviewer against the implementation, with no deletion outside the
project (relayed by the session coordinator, 2026-09-24; not re-run here).

**Why the record bound is 1 MiB.** `FR-CDOC-017` requires a bound no record
`tpl` writes reaches, and leaves its value to this folder. Neither record grows
with the catalogue: `meta.json` holds two versions, a timestamp and three
completeness flags, and `database.json` three identifiers and the `server`
object, so each is under 1 KiB (the committed caches under `examples/` hold
records of 158 to 183 bytes, measured on 2026-09-24). 1 MiB is over a thousand
times that, and it keeps the read of a planted file far below the 8 MiB lower
bound `FR-CONF-002` sets for the render memory limit.

Compact rather than indented, because nothing reads these files but `tpl`:
`BR-CACHE-001` makes `tpl cache status` the supported way to learn the cache's
state, and `FR-OUT-007`'s forms govern stdout, which a cache file is not.
Indenting for a human reader was rejected as bytes paid on every read of a file
no contract exposes.

**What a cached object file contains.** The model object alone, without the
envelope of `FR-OUT-024`. `OD-10` left this point to this document, and it is
settled here.

- *Rejected — the whole envelope.* `source` states where the bytes of an
  emitted document came from (`FR-OUT-026`) and is therefore a property of the
  **read**: a file storing `server` would have to be rewritten as `cache` on
  every read served from it, and `FR-OUT-029` forbids `source` on an object in
  the first place. The `data` key is named for the collection or the kind
  (`FR-OUT-030`, `FR-OUT-031`), which the path already states.
- *Rejected — the object with `schema_version` beside it.* `FR-CDOC-001` puts
  that version in `meta.json` and `FR-CDOC-004` requires it to be read before
  anything under it is trusted, so a per-file copy is a second statement of one
  fact with no rule to break a tie between them.
- *Consequence.* A cached file deserialises into the same model type the server
  path produces, which is what keeps one representation and makes the round
  trip of `FR-SCH-022` a property of that representation rather than of a
  second reader.

**Recorded ambiguity — whether a cached file is a document.** `FR-CDOC-003`
reads that `schema_version` *"SHALL be the same version the documents in the
cache carry under `FR-OUT-011`"*, and `FR-CACHE-005` that *"each cached
document SHALL carry a format version"*. Read literally, both put a version
inside each cached file. Read against `FR-OUT-011` and `FR-OUT-024`, which are
scoped to documents `tpl` emits and writes to **stdout**, neither reaches a
file in `.tpl/.cache/`, and `FR-CACHE-005`'s own second sentence defers where
each version is written to `FR-CDOC-001` — which writes both to `meta.json`.
This document takes the second reading, for the grounds above. If the
functional owner intends the first, the decision recorded here changes and
`OD-10`'s residual is settled the other way. Both readings are recorded; the
wording is the functional owner's to settle.

**The case collision.** Two objects of one kind whose names differ only in case
map to one path on a case-insensitive filesystem, which is the default on two
of the four targets of `NFR-PERF-018`. **No collision is detected and none
fails** (`OD-10`, superseded on 2026-09-23 by the user's decision for rmp
`#254`, which records the rejected detect-and-fail and collision-free
encoding). `FR-CACHE-033` governs: a named read whose file holds the other
object is a miss, on every filesystem; two names sharing one file leave the
directory's count short, so `replace()` in `src/cache.rs` records the collection
as not whole; and either object can miss on every invocation, which is that
requirement's accepted cost.

*Superseded on 2026-09-23 — the paragraph below applied `OD-10`'s
detect-and-fail to `tpl cache load`. It is kept as history and no longer
prescribes anything.*

**Recorded discrepancy — the command that detects it.** `OD-10` assigns the
detection to *"`cache refresh`"*. `FR-CLI-010` and `FR-CACHE-021` through
`FR-CACHE-023` admit no such command: the cache group is `load`, `clean` and
`status`. The decision's substance is unaffected and applies at the point a
cache write would create the collision, which under `FR-PROJ-023` is
`tpl cache load` and a cached read on a miss. Which of the two outcomes governs
the second of those writers is **not settled**: `FR-CACHE-036` makes a cache
that cannot be written a silent success at exit `0` with stdout unchanged,
while `OD-10` fails and names both objects. This document applies `OD-10` to
`tpl cache load`, whose purpose is to populate the cache, and records the
opportunistic-write case as owed to the register's owner rather than answering
it here.

## `meta.json`

`FR-CDOC-001` puts it at `.tpl/.cache/<entry>/meta.json` and requires two
version fields. What the file carries, in full:

| Field | Content | Forced by |
|---|---|---|
| `cache_format` | The version of the on-disk arrangement | `FR-CDOC-001`, `FR-CDOC-002` |
| `schema_version` | The version of the model content | `FR-CDOC-001`, `FR-CDOC-003` |
| `loaded_at` | The load time, in the fixed twenty-character form `YYYY-MM-DDTHH:MM:SSZ`, written and parsed against that one grammar | `FR-CDOC-013`, `FR-CACHE-034`, [`OD-25`](open-decisions.md#od-25--the-clock-source-for-now) |
| Per-collection completeness | For each of the three collections, whether it was loaded whole | `FR-CDOC-006`, `FR-CTX-035` |

Four rules govern how it is used:

- **It is parsed first.** `FR-CDOC-004` makes either version unknown to the
  running binary a miss, so nothing under the folder may be trusted before
  `meta.json` has been read and accepted.
- **Completeness decides a listing, not an object.** A listing is served only
  where its collection is recorded whole (`FR-CDOC-007`); an individual object
  is served whenever present (`FR-CDOC-008`). *Present* is narrowed twice:
  `FR-CACHE-037` keeps a marked object out of the cache entirely, and
  `FR-CACHE-033` makes it a regular file holding that object.
- **The object count is not stored.** `FR-CACHE-034` requires `tpl cache status`
  to report a count per collection; it is derived by listing that collection's
  directory. Recording it in `meta.json` was rejected: it is a second statement
  of a fact the files already make, and a partial or failed write would leave
  the two disagreeing with nothing to break the tie.
- **`loaded_at` goes nowhere else.** It appears in this file and in
  `tpl cache status` and in no read (`FR-CDOC-012`, `FR-CDOC-013`), because a
  load time in a read would make two identical invocations against an unchanged
  project differ (`BR-CDOC-003`, `NFR-DET-001`).

`meta.json` is not plumbing contract (`BR-CDOC-005`, `BR-CACHE-001`). Its
fields exist so the binary can decide what it may serve, and the supported way
to learn the cache's state is `tpl cache status`, whose document is
`interfaces.md`'s.

## The four version numbers

Four numbers move independently, and the fourth is a file rather than an
integer. All four are settled in
[`OD-03`](open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog),
whose rationale and rejected options are not restated here.

| Number | Starting value | What moves it | Fixed by |
|---|---|---|---|
| Binary version | `0.0.1` | A release. Full semantic versioning; while below 1.0 a breaking change is a **minor** bump | `OD-03`; the value is the user's choice of first release, and the `0.1.0` of `FR-HELP-005` and `FR-CTX-027` is illustrative |
| `schema_version` | `1` | A change on one of the three breaking rows of `FR-OUT-014` — removing a field, renaming one, changing a field's type. Adding a field or an enumerated value does **not** move it | `FR-OUT-011`, `FR-OUT-014`, `FR-CDOC-003` |
| `cache_format` | `1` | A change to the on-disk arrangement alone: which files exist, where they sit, how they are named | `FR-CDOC-002`, `FR-CDOC-005` |
| Changelog | `CHANGELOG.md`, Keep a Changelog format | A release, and every change `FR-ENV-029` requires to be recorded there — renaming or removing a guaranteed template name | `FR-ENV-029`; `CLAUDE.md` *Fluxo de Trabalho* names a changelog in the workflow |

Three properties bind the built system:

- **The binary version has one source.** `tpl version` and the `tpl.version`
  context variable both read `CARGO_PKG_VERSION`, so `FR-HELP-005` and
  `FR-CTX-027` are incapable of disagreeing.
- **The two cache versions are independent.** `FR-CDOC-005` forbids
  incrementing either on account of a change that affects only the other, and
  `BR-CDOC-001` is why one number cannot carry both facts.
- **`schema_version` is the document's, not the binary's.** `FR-OUT-011`
  versions the contract independently of the binary, so a binary release moves
  the first number and leaves the second where it stands.

**One shape change on 2026-09-21 left `schema_version` at `1`, deliberately.**
A column's decomposed type moved from a nested object to nine keys sitting
beside the column's own
([interfaces.md](interfaces.md#the-two-directions-over-the-document)). One
ground decides it, and the second row states what that ground spares.

| Ground | What holds |
|---|---|
| The contract did not change; the emitter did | `FR-CTX-014` and `FR-CTX-015` put `column_type` and the eight parts on the **column**. The nested form never matched them, so what happened is a divergence from the contract being closed, not a contract being revised. `FR-OUT-014` has no case to classify, and the release gate over it ([operations.md](operations.md#what-a-breaking-change-is)) has nothing to catch |
| What the first ground spares | Had it not held, the gate would have obliged a bump. No requirement fixes the **value** of `schema_version`; six files of `/specification` publish documents carrying `1`, and every one of them would have had to move with it |

A cached document written before the change answers nobody wrongly: it carries
the parts nested where this binary's types read them beside the column, so it
does not decode, and a file that cannot be decoded is a **miss**
(`FR-CACHE-033`, `FR-CDOC-004`). The read goes to the server and rewrites it.

**Where the consequence of a classification lives is worth stating once.**
`FR-OUT-014` marks a change breaking or not breaking and attaches no
consequence to either; `FR-OUT-011` fixes only what the field versions. The
consequence — that a breaking change moves the number versioning the surface it
broke — is this folder's, settled in
[`OD-03`](open-decisions.md#od-03--versioning-the-binary-the-document-the-cache-the-changelog)
and enacted in `operations.md`, and it is not a requirement.

Where a bump is enacted in a release, and what a release gate checks, are
`operations.md`.

**Note on the state of the repository, read at commit `243c4d6`.** All four
numbers now have a site, and no row is prescription alone. The binary version is
the manifest's `version` field; `schema_version` is the one constant `output/`
writes into the envelope; `cache_format` is `const CACHE_FORMAT: u32 = 1` in
`src/cache/meta.rs`, which reads `SCHEMA_VERSION` from `output/` rather than
writing a second copy of it; and `CHANGELOG.md` was created at commit `f2d19ac`
on 2026-09-21, in the Keep a Changelog format this table names. Each of the
three integers carries the starting value above, and nothing built contradicts
any of the four.

## Migration

Nothing persisted is ever migrated. Three different mechanisms are why, one per
persisted thing.

| Persisted thing | Migration rule | Forced by |
|---|---|---|
| `.tpl/.cache/` | None. A version the binary does not know is a miss: the data is re-read from the server and rewritten, with no error and no warning. The binary accepts the versions it writes and treats every other value as unknown | `FR-CDOC-004`, `FR-CACHE-033` |
| `.tpl/.cfg` | None, and no version field. The key space is closed and strict in both directions, so a file is either within the key space a binary knows or is refused whole with `78` | `FR-CONF-002`, `FR-CONF-034` |
| The JSON document | Versioned, not migrated. `FR-OUT-014`'s five rows decide whether a change moves `schema_version`; a consumer reads the field and branches | `FR-OUT-011`, `FR-OUT-014` |

Maintaining a reader for a superseded `cache_format` was rejected: the cache is
an optimisation whose whole content is recoverable by reading the server again
(`FR-CACHE-007`), so a migration path would be code carrying a risk to recover
something already recoverable. The `.cfg` case is the opposite and is the
accepted cost `FR-CONF-034` states in its own text: an older `tpl` cannot read
a project written by a newer one, and in exchange every key present in a `.cfg`
is a key in force.

## What a cached document does not promise

| Promise | Server read | Cache-served |
|---|---|---|
| Referential integrity — every object referenced from another is present | Yes, `FR-CTX-023` | **No**, `FR-CTX-025`, `FR-CDOC-015` |
| A point-in-time snapshot | No, and the corpus states so rather than implying it, `FR-CTX-024` | **No**, `FR-CDOC-015` |
| The signal a consumer recognises it by | — | `"source":"cache"`, `FR-CDOC-016` |

`BR-CDOC-004` gives the mechanism: one file per object, renamed into place,
with no lock and no expiry, so the cache can hold one table read on Monday
beside another read on Friday, and a document assembled from it is one that
never existed on any server at any instant. That is the direct consequence of a
cache that changes only when it is told to, which `FR-CACHE-008`,
`FR-CACHE-028` and `BR-CACHE-004` establish deliberately.

Two obligations on the built system follow, and both are about not claiming
more than is held:

- The assembler must not present a cache-served document as carrying either
  promise (`BR-CDOC-004`), and `source` is the only place the distinction is
  recorded.
- Repointing an entry invalidates nothing (`FR-CACHE-029`), so a read after a
  repoint serves the previous server's catalogue at exit `0` with nothing in
  the output saying so. `BR-CACHE-003` accepts the cost and names
  `tpl cache status`'s load time as the only signal.

## What this document defers, and to what

| Subject | Where |
|---|---|
| The document's keys, depths, cuts and envelope | `specification/context-document.md`, `specification/output-formats.md` |
| The emitter, the orderings, the seventeen payload shapes, `restricted`, and the cache-status document | `interfaces.md` |
| The reader, the cache as a read-through layer, and where the model is assembled | `architecture.md` |
| The crates and versions behind the TOML paths, the serialiser and the clock | `technology-stack.md` |
| File permissions as a trust boundary, credentials in `.cfg`, and what a cache file may hold | `security.md` |
| Where a version bump is enacted, and the release gates | `operations.md` |
| The memory cost of the materialised embeddings, and cross-series equivalence | [quality-attributes.md](quality-attributes.md) |
| The tests over the classification order, the exclusion lists and the round trip | `verification.md` |
| Three rows of [traceability.md](traceability.md) that name this document and are answered in `interfaces.md` because [README.md](README.md#data-modelmd) excludes them here: the help table of `FR-HELP-022`, the five context variables of `FR-RND-023`, and the three type families of `FR-ENV-042` | `interfaces.md` |
| Why a settled decision went the way it did | [`docs/adr/`](../adr/README.md), or [open-decisions.md](open-decisions.md) where no record holds it |
