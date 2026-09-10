---
title: tpl Functional Specification
status: draft
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

The specification has been written in four editions. All are in force; each
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
generalise it is blocked on observing `OQ-041`.

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
  differences actually exist is [OQ-045](open-questions.md#oq-045) and cannot be
  written until the container exists.
- **The server version in the model** — `FR-SRV-028` and `FR-CTX-031`. A
  template that must accommodate a difference could not previously see which
  server it was rendering against.

Three open questions are closed and none is left open. `OQ-044` — the outcome
below the window — is `FR-SRV-020`: refusal with `78` and
`kind: server_version_unsupported`. The other two were the escalations the
ceiling created, and both came back decided:

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

### Still out of scope

- The Rust implementation: its crates, its module layout, its types, and its
  library API. Only the JSON document and the command line are contract, so the
  shape of the library is an architecture decision and not a requirement.
- The text of any catalogue query. What is read is specified; how it is read is
  not.
- Every numeric performance target and every measured baseline. Those live in
  the project's architecture decision records and in `BENCHMARKS.md`, cited by
  the requirement that needs them.
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
therefore expected, not a defect: sixteen open questions are closed and their
numbers are not reused. The *Closed* table of
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
numbered within its file, from `001`, in declaration order.

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

## Status legend

| Status | Meaning |
|---|---|
| `draft` | Written from a settled decision, not yet reviewed against the corpus |
| `approved` | Reviewed and in force |
| `deprecated` | Superseded; retained for traceability, not to be implemented |

## Provenance

Every requirement in this specification derives from one of three sources:

1. Four decision logs. The first interview settled 53 points about the CLI
   surface; the second settled 28 points about the model, the document, the
   template surface, the server contract, and performance, and recorded four
   defects found in the first edition; the third is the audit of 2026-09-10,
   whose fifteen findings the user answered with fourteen decisions and one
   deliberate deferral; the fourth settled the supported version window. Each
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
4. Nothing else. Where information is missing, this specification records an
   entry in [open-questions.md](open-questions.md) rather than filling the gap.

## Maintenance debt

One item is known, deliberately deferred, and owed to a maintenance pass that
follows the third edition. It is recorded here so that it is not rediscovered
as a defect.

1. **The first edition's open questions.** `OQ-002` through `OQ-024`, less
   `OQ-008`, `OQ-020`, and `OQ-022`, remain in
   [open-questions.md](open-questions.md). The second edition settled a
   different set of subjects and did not touch them; the third closed three of
   them and narrowed `OQ-024`, and the rest are the residue.

Three items previously recorded here have been discharged.

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
