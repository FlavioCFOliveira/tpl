---
title: Upstream Divergences
status: approved
last-reviewed: 2026-09-23
related: [README.md, global-flags.md, cfg-commands.md, template-environment.md, server-contract.md, performance-requirements.md]
---

# Upstream Divergences

## Overview

This specification is the single source of functional truth. The root
`README.md` and the root `CLAUDE.md` have stated functional content that this
specification either contradicts or now owns, and they have stated as present
things the repository does not yet contain. Each divergence is recorded here
with what the file said, what is in force against it, the correction owed, and
whether it is still owed.

**This file does not authorise a change to either document, and the
specification never edits them.** It is a work list for whoever holds the pen on
those two files.

**Fifty-six entries are recorded. As of 2026-09-23, five are due in full,
forty-seven are discharged, and four are partly discharged.** Nine entries
still owe something: five owe it to `CLAUDE.md` and four to `README.md`, and no
entry is in both counts.

*The counts moved in the forty-second edition, and one entry moved them.*
`DIV-056` is raised against `README.md`, because `FR-CONF-002` gained three
keys and the root file counts the key space. It joins the **due** count and the
count owed to `README.md`. No entry was re-read and none changed status; the
counts restated in `DIV-032` and `DIV-041` move with this paragraph.

*The counts moved in the thirty-seventh edition, and two entries moved them.*
`DIV-055` is discharged by `9562fb2` and `DIV-052` by `abbfe70`, so two entries
leave the **due** count and join the discharged one, and both were owed to
`CLAUDE.md`. `DIV-050` discharged one more of its lines, at `abbfe70`, and stays
**partly discharged**, so it moves no count — a status value is what these
counts count. The four that remain due in full are `DIV-032` and `DIV-041`,
owed to `CLAUDE.md`, and `DIV-053` and `DIV-054`, owed to `README.md`. The four
partly discharged are `DIV-001`, whose remaining half is `README.md`'s, and
`DIV-036`, `DIV-037` and `DIV-050`, whose remaining parts are all `CLAUDE.md`'s.

**The asymmetry has reversed, and one commit did it.** `e75996c` rewrote the
root `README.md` whole on 2026-09-17 — 192 lines in, 365 out — and it is what
`0ea5624` was to the other file: it discharges something in thirty-one entries
at a stroke, by removing the command reference, the alias table, every flag
table, the exit code table, the error envelope and the template environment
section, and by making the corrections eleven entries asked for in so many
words. `README.md` owed thirty-two corrections before it and owes three
after it. The remaining work of this register is now almost entirely
`CLAUDE.md`'s.

*Corrected in the thirty-seventh edition, in the last sentence.* It was written
when `CLAUDE.md` owed seven entries against `README.md`'s three. Two of those
seven are discharged, so the split is **five to three**, and *almost entirely*
overstates it. The asymmetry stands and is smaller: `CLAUDE.md` owes the
majority of what is left, and no longer the bulk of it.

**The pass the thirty-first edition left owed is this one, and it is made.**
All fifty-two entries then recorded were re-read on 2026-09-21 against
`README.md` at `db80114` and `CLAUDE.md` at `8f936d4`, in both directions, as
the sixteenth edition's extension of the fifth validation rule requires: for
the entries the files have discharged, and for the divergences this register
did not hold. Thirty entries were found discharged and each records the commit
that discharged it; two divergences had no entry and are `DIV-053` and
`DIV-054`. Nothing in the register now rests on a reading older than that
date.

**Two entries were read a second time the same day, against the working tree
rather than against either document, and neither changed status.**
`scripts/mariadb/seed-bench.sql` was added to the tree after the pass above
was made. It is the file `DIV-036` was raised over and half of what `DIV-052`
was waiting on, so both entries carried a clause that had stopped being true
within the day. `DIV-036` stays **partly discharged** and now owes one sentence
where it owed a sentence and a file; `DIV-052` stays **due** and now waits on
one thing where it waited on two. The counts above are unchanged, because a
status value is what they count and neither moved. *The one thing `DIV-052` was
then waiting on arrived the next day, at `abbfe70`, and the thirty-seventh
edition discharged that entry; `DIV-036` is where this paragraph left it.*

**One entry is added by the thirty-sixth edition, and it was not raised by an
edit to either file.** `DIV-055` records two passages of `CLAUDE.md` stating
that a regression against a recorded baseline fails the change. Both were exact
summaries of `NFR-PERF-017` when they were written, and that requirement is
withdrawn, so they became owed without anybody touching the file they are owed
to — the direction the thirty-second edition's extension of the fifth
validation rule covers, and the second entry raised in it after `DIV-053`. The
counts above moved with it: one more entry, one more due in full, one more owed
to `CLAUDE.md`. `DIV-052` is raised against one of the same two passages, for a
different defect and with a different correction, and it is amended in the same
edition where this one made it false; neither discharges the other, and that
entry's **Kind** is a question named there and left for the next reading of
this file.

**Both entries are discharged by the thirty-seventh edition, which is that next
reading, and they discharged apart.** `9562fb2` removed both of `DIV-055`'s
passages from `CLAUDE.md`, and `abbfe70` created `benches/` with the measurement
harness in it, which made `DIV-052`'s sentence true without anybody editing the
file. **The Kind question is settled there and is settled by that fact**: a
Kind classifies the defect an entry records and not the passage it points at,
and the proof is that one sentence carrying two defects discharged on two
different conditions, by two different commits, through events of two different
sorts. An entry with one Kind could have described only one of them. The
grounds, and the two alternatives rejected with them, are under
[DIV-052](#div-052).

**A third entry was read against the working tree, on 2026-09-22, and it did
not move either.** `0110f8c` created `examples/`, which is one of the three
lines of `CLAUDE.md`'s project tree that `DIV-050` still recorded as absent.
That entry stays **partly discharged** and now owes two lines where it owed
three. The counts above are unchanged for the same reason, and the trigger was
again an event in the repository rather than an edit to either document — the
direction the thirty-second edition widened the **Overstatement** kind to
cover.

**`DIV-050` was read against the working tree once more, later the same day,
and still did not move.** `abbfe70` created `benches/`, the second of the two
lines that entry then owed, so it now owes **one** — `templates/`, which is
still absent. It stays **partly discharged**, because an entry is never recorded
as discharged while any part of it stands, and the counts above are unchanged
for the third time and for the same reason. The trigger was again an event in
the repository. The same commit discharged `DIV-052`, which reads that directory
from the other end, and the two were read separately against their own passages,
which is what both entries give as the reason they are recorded apart.

Three kinds of entry appear:

- **Contradiction** — the file states something this specification contradicts.
  Leaving it is a defect.
- **Migration** — the file states something this specification now owns.
  Leaving it creates two sources for one truth.
- **Overstatement** — the file states, as a fact about the repository,
  something that is not true of it. Nothing is owed to this folder and no
  requirement is contradicted; what is wrong is that a reader who acts on the
  passage fails. Leaving it spends the credit of everything else the file says.

  *Widened in the thirty-second edition.* The kind read *states as present
  something the repository does not contain*, which is the direction the four
  entries of the sixteenth edition were raised in and not the whole of the
  kind. `DIV-054` is the other direction: two passages of `README.md` state
  that no command reaches a server, which was true when they were written and
  has not been since `db7337d`, and a contributor who acts on the second of
  them skips the fixture and watches a server-dependent test fail. The test the
  kind states of itself is unchanged and is what decides membership — nothing
  owed here, no requirement contradicted, and a reader who acts on the passage
  fails. *Rejected: a fourth kind for the opposite direction.* Three kinds
  distinguish **who** the statement is wrong about — this corpus, this corpus's
  ownership, the repository — and a fourth would split the third by direction,
  which is a property of the individual passage and not of what has to be done
  about it. The correction is the same either way: make the tense or the fact
  match the working tree.

*The third kind is added in the sixteenth edition*, and five entries carry it:
`DIV-046` and `DIV-054` against `README.md`, and `DIV-050`, `DIV-051` and
`DIV-052` against `CLAUDE.md`. **Three of the five are discharged in full —
`DIV-046`, `DIV-051` and `DIV-052` — and every one of the three by the
repository catching up with the document rather than by an edit to it. A
fourth, `DIV-050`, is partly discharged the same way and owes one line. The
fifth, `DIV-054`, is due.** No one of the five discharges another, because each
is corrected in the passage it was raised against. The first four were raised on
one fact — that the repository then held no crate — and `DIV-046` and `DIV-051`
are the closest pair among them: they record the same eight `cargo` commands,
under headings that are each other's translation, once in each root document,
and `d8e7e8a` discharged both. `DIV-052` needed a second fact beside the first,
and it is why that entry outlived them: a benchmark had to be written before the
directory its passage names held anything. `abbfe70` wrote one, on 2026-09-22.
`DIV-054` is the fifth and is of the opposite direction, which is what widened
the kind.

*Corrected in the thirty-seventh edition, in the counts and in the clause
naming what `DIV-052` waited on.* The closing clause said that entry *alone is
still due* because there are no benchmarks and the fixture they run against is
incomplete. The fixture was completed in the thirty-third edition and that
entry's Status was narrowed then; the benchmark arrived at `abbfe70`, and both
halves of the clause are paid. The counts are restated against the Index rather
than carried forward, which is the third validation rule of the
[README](README.md#maintenance-debt) applied here: the paragraph read *three of
the five are discharged, two of them by the repository catching up*, and the
Index of the day held two discharged, not three, each of them by the repository
catching up. Counting a paragraph's own claim correctly establishes nothing
about whether it matches the table it describes.

*The seventeenth edition closes that kind in `CLAUDE.md`* and adds no entry to
it. The sixteenth edition's sweep left one candidate in that file unjudged —
the *verificados* of *Plataformas Suportadas* — and the reading of that section
finds that it declares a policy rather than asserting a state, so nothing is
owed for it. That reading, and the sweep establishing that no other passage of
the kind is left in the file, are at *[A candidate read, and not
recorded](#a-candidate-read-and-not-recorded)*.

The two kinds above were written when every entry described the relation
between a root document and this corpus. An entry that describes the relation
between a root document and the repository has no home among them, and
labelling it a contradiction would have named a requirement that does not
exist.

## How an entry is read

Four things carry an entry, and each answers a different question.

- **Target** names the files the entry was raised against, and is the authority
  on where to look when the entry is re-checked.
- **Kind** is one of the three above.
- **Status** says what is still owed, and where, as of the date it names. It is
  the only part of an entry that speaks about the files as they now stand.
- The *Says* clause is a **quotation of the target file as it stood when the
  entry was written**. It is not a claim about the file today, and a discharged
  entry commonly carries one whose words are no longer in the file. **Status**
  is what settles whether the clause is still there; the quotation is kept
  because it is the evidence the entry was raised on, and removing it would
  leave the entry unfalsifiable.

  *Extended in the thirty-second edition to every clause but **Status**.* The
  rule was written over *Says* alone, and an entry carries more than *Says*: a
  *Specification* clause, a *Correction*, the notes an amendment leaves behind,
  and its row in the Index. Each of those states the file, the repository and
  this corpus **as they stood when that clause was written or last amended**,
  which its own edition note dates, and none of them is a claim about today.
  **Status** is, and it is the only part of an entry that is. Without the
  extension a reader would have to treat a dozen dated paragraphs as current
  and the register would decay faster than any pass could repair it; with it,
  one paragraph per entry is what a re-read has to bring forward. Where a dated
  clause is **load-bearing for an entry that is still owed**, it is corrected
  rather than left to the rule — `DIV-050` and `DIV-052` each carry such a
  correction from this edition — because a live entry's own reasoning has to
  be true of the thing it asks somebody to change.

A **Status** takes one of three values.

- **Due** — the target file still states what the entry records, and the
  correction is owed in full.
- **Discharged** — nothing the entry records is owed any longer. The status
  names the commit that discharged it, so that the entry says *when* it stopped
  being owed and not merely that it is not owed. Nothing remains.

  *Amended in the thirty-second edition: a discharge is not always an edit to
  the target.* The value read *the target file no longer states it*, which is
  true of a **contradiction** and of a **migration**, because what is wrong
  there is the sentence. It is not true of an **overstatement**, where what is
  wrong is that the sentence does not match the repository: such an entry
  discharges equally when the repository catches up with the document, and the
  passage then stands untouched and correct. `DIV-046`, `DIV-051` and four
  lines of `DIV-050` discharged that way, at `d8e7e8a`, `4014dc4` and
  `0110f8c`, with no edit to either root document. **The commit named is then
  the one that made the statement true**, and the entry says so, because an
  entry that merely stopped being listed leaves the next reader to re-derive
  it. Two of those entries carry a further clause, and it is the reason this amendment is worth
  making: **the correction they asked for must now not be made**, since a
  sentence saying the crate does not exist would today be false. A register
  whose remedies have outlived their condition is worse than one that is
  merely stale.
- **Partly discharged** — the correction has separable parts, some made and
  some not. This is the ordinary outcome for an entry whose **Target** is
  `both`: the two files are edited under separate authorisations and almost
  never in the same commit, so an entry against both is discharged in halves.
  Such a status names, part by part, what was discharged and by which commit,
  and what remains owed. **An entry is never recorded as discharged while any
  part of it stands**, because an entry reduced to its last clause is still a
  correction somebody has to make.

## A discharged entry is kept, not removed

A discharged entry keeps its heading and its identifier, in place, and carries
a status saying what discharged it. It is not deleted, and it is not moved.
This is the rule [README.md](README.md#identifier-scheme) already states for a
withdrawn requirement, applied for the same reason: a reference written before
the discharge must resolve to an explanation rather than to nothing. `DIV-045`
has been written this way since the ninth edition, so the rule was already in
force in this file before it was stated here.

**Rejected: the *Closed* table of [open-questions.md](open-questions.md).** A
closed open question leaves the index and becomes a row in a table at the foot
of its file, and that was the obvious alternative, since a divergence closes
the way an open question closes rather than standing in force the way a
requirement stands. It was rejected on two grounds. First, entries here cite
each other — `DIV-029` cites `DIV-020`, `DIV-038` cites `DIV-014`, `DIV-040`
cites `DIV-015`, `DIV-043` cites `DIV-013` — and a citation whose target has
been lifted out of its section resolves to a row instead of to the reasoning
the citing entry depends on. Second, `DIV-045` already carries the in-place
form, so adopting the table would put two treatments of one event in one file,
which is the defect this specification records against the root documents.

## When this file is re-read

The classification below was made against `README.md` at `db80114` and
`CLAUDE.md` at `9562fb2`, the last commit to touch each — the first on
2026-09-21 and the second on 2026-09-22. **It stops being true the moment
either file is edited.** That is how `DIV-031` came to ask, through four
editions and one amendment of its own, for the removal of a clause that had
been gone since `0ea5624`.

*Restated in the thirty-second edition, with the commits it now names.* The
fifteenth edition made the classification at `87dd6e3` and wrote this section
to say that it decays. It did. `README.md` was edited three times after it —
`e75996c`, `db7337d` and `db80114` — and `CLAUDE.md` five — `c6356df`,
`b066cfa`, `cd6ce7e`, `6a0cce5` and `8f936d4` — and the register went ten days
out of date against both, which the thirty-first edition recorded and did not
pay. **This edition paid it**, and the classification below is of the state
above and of no other.

*Paid again in the thirty-seventh edition, against `CLAUDE.md` at `9562fb2`.*
That file has been edited three times since `8f936d4`, all on 2026-09-22 —
`3a360d6`, then `6a66d14`, then `9562fb2` — so the classification above stopped
being true of it, on this section's own terms. The three edits were read against
every entry whose **Target** is that file. **Only `9562fb2` reaches any of
them**: it rewrote both passages of `DIV-055`, it rewrote the second half of the
sentence `DIV-052` is raised against and left that entry's own clause standing,
and it changed one comment line inside the project tree `DIV-050` targets.
`3a360d6` touched the `Erros` row of the *Stack* table and the *Tipos e erros*
bullet that restates it; `6a66d14` touched the synergy rules, the pre-flight
checklist and the open-task gate. **No entry of this register targets any
passage either of those two commits touched**, so neither discharges anything
and neither changes a status. `README.md` is untouched since `db80114`, and the
classification of every entry against it stands as the thirty-second edition
made it.

The obligation that replaces the presumption is recorded in
[README.md](README.md#maintenance-debt) as the fifth validation rule of this
corpus: **a register of corrections owed to a file this specification does not
own is re-read against that file whenever the file changes**, and an entry
found discharged records the commit that discharged it. The trigger is the
edit, not the edition. A register whose entries are true only of a state
nobody has checked since is worse than no register, because it sends a reader
to correct what is already correct and spends the standing of the entries that
are still owed.

## The commits named below

Seventeen commits are named in the statuses, and each is named by its short
hash alone after this table.

| Commit | Date | Subject |
|---|---|---|
| `3f65b5f` | 2026-09-09 | docs: describe the tpl CLI and its project conventions |
| `1352a2d` | 2026-09-09 | docs(specification): specify the complete CLI surface |
| `011c059` | 2026-09-10 | docs(specification): support a window of MariaDB series, not a floor |
| `0ea5624` | 2026-09-10 | docs: restructure CLAUDE.md around the specification |
| `50153d6` | 2026-09-11 | docs(claude): reduce four passages to citations of the records |
| `26e1739` | 2026-09-11 | docs(readme): name the five TLS modes and the default in force |
| `87dd6e3` | 2026-09-11 | docs: describe the repository the coordination documents actually have |
| `d8e7e8a` | 2026-09-12 | build(crate): stand up the cargo package and the crate root |
| `4014dc4` | 2026-09-15 | feat(cli): render the seven help sections at a fixed width of eighty columns |
| `e75996c` | 2026-09-17 | feat(project): give tpl a project it discovers, trusts, reads and writes |
| `db7337d` | 2026-09-20 | feat(schema): wire the eight schema commands and the catalogue cache |
| `db80114` | 2026-09-21 | feat(render): deliver the third arm and the context it assembles |
| `8f936d4` | 2026-09-21 | docs(coord): condition parallelism on user authorisation alone |
| `0110f8c` | 2026-09-22 | feat(examples): vendor sakila and world, and write the shared driver |
| `abbfe70` | 2026-09-22 | chore(benches): add the measurement harness for the nine budgets |
| `9562fb2` | 2026-09-22 | docs: make performance guidance design, not a gate |
| `fe428da` | 2026-09-22 | feat(benches): measure the nine points and record the observations |

`3f65b5f` wrote both root documents and is the state every entry of the first
edition was raised against. Two commits carry most of the discharges, one per
file. `0ea5624` reduced `CLAUDE.md` to agent coordination and is behind twelve
discharges and nineteen halves. `e75996c` rewrote `README.md` whole and is
behind thirty-one, which is why the majority of the remaining work of this file
is `CLAUDE.md`'s — the reverse of what the fifteenth edition found. *The
thirty-seventh edition narrowed the claim here as it narrowed it in the
[Overview](#overview), and for the same arithmetic: the split is five entries
to three, not the seven to three it was written against.*

**Eight of the seventeen are not edits to a root document at all**, and they are
named because an **overstatement** discharges when the repository catches up
with the text, on the rule at *[How an entry is read](#how-an-entry-is-read)*.
`d8e7e8a` created the crate, `4014dc4` created `tests/`, `0110f8c` created
`examples/`, `abbfe70` created `benches/` with the measurement harness in it,
`fe428da` took the first readings from that harness, and `db7337d` and
`db80114` wired the arms that made two sentences of `README.md` stop being true.
`1352a2d` is the eighth and is named for neither reason: it is where this
register was opened, and `DIV-025` cites it for a search of the history.

## Index

| Id | Target | Kind | Status | Subject |
|---|---|---|---|---|
| [DIV-001](#div-001) | both | Migration | Partly, `README.md` | All CLI-surface content moves here |
| [DIV-002](#div-002) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | "No command accepts a password in argv" |
| [DIV-003](#div-003) | `README.md` | Contradiction | Discharged, `e75996c` | The `--password` / `-p` flag |
| [DIV-004](#div-004) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `--output`, `--output-dir` and the file-writing surface |
| [DIV-005](#div-005) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Only `init` and `cfg` write inside `.tpl/` |
| [DIV-006](#div-006) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Configuration printing always redacts |
| [DIV-007](#div-007) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `EPIPE` always exits `0` |
| [DIV-008](#div-008) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `${VAR}` expands in any string value |
| [DIV-009](#div-009) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `schema dump` equals the render context |
| [DIV-010](#div-010) | `README.md` | Contradiction | Discharged, `e75996c` | TTY-dependent `--format` default |
| [DIV-011](#div-011) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | Positional render target |
| [DIV-012](#div-012) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `tpl init` takes no argument |
| [DIV-013](#div-013) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `password_command` is a shell command |
| [DIV-014](#div-014) | `README.md` | Contradiction | Discharged, `e75996c` | Aliases `procs` and `proc` |
| [DIV-015](#div-015) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | The `database` and `config` command groups |
| [DIV-016](#div-016) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `--no-color` and `NO_COLOR` |
| [DIV-017](#div-017) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `TPL_DIR` and `TPL_DATABASE` |
| [DIV-018](#div-018) | `README.md` | Contradiction | Discharged, `26e1739` and `e75996c` | Three TLS modes, defaulting to `preferred` |
| [DIV-019](#div-019) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `--all-tables` and `--pattern` on `render` |
| [DIV-020](#div-020) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | The `.tpl/` layout omits `.cache/` |
| [DIV-021](#div-021) | `README.md` | Contradiction | Discharged, `e75996c` | `template check` lints |
| [DIV-022](#div-022) | `README.md` | Contradiction | Discharged, `e75996c` | `--pattern` follows the server collation |
| [DIV-023](#div-023) | both | Migration | Discharged, `0ea5624` and `e75996c` | Global flag tables are wrong in three ways |
| [DIV-024](#div-024) | `CLAUDE.md` | Migration | Discharged, `0ea5624` | Project discovery lacks its boundary and checks |
| [DIV-025](#div-025) | `CLAUDE.md` | Migration | Discharged, never owed | The "specification does not exist yet" note |
| [DIV-026](#div-026) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | The `rust_type` and `go_type` filters |
| [DIV-027](#div-027) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | The `plural` and `singular` filters |
| [DIV-028](#div-028) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | Auto-escaping keyed on the file extension |
| [DIV-029](#div-029) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | `tpl init` creates four artefacts |
| [DIV-030](#div-030) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | The read-only session presented as prevention |
| [DIV-031](#div-031) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `SHOW` as a permitted way to read the catalogue |
| [DIV-032](#div-032) | `CLAUDE.md` | Contradiction | Due | `model/` as the documented public surface |
| [DIV-033](#div-033) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | The whole `Environment` surface as contract |
| [DIV-034](#div-034) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | The `table` and `column` field lists |
| [DIV-035](#div-035) | `CLAUDE.md` | Migration | Discharged, `0ea5624` | The performance budget table |
| [DIV-036](#div-036) | `CLAUDE.md` | Migration | Partly, `CLAUDE.md` | The testing section names two of the fixture's three SQL scripts |
| [DIV-037](#div-037) | both | Contradiction | Partly, `CLAUDE.md` | A withdrawn MariaDB floor, no ceiling, and no refusal of MySQL |
| [DIV-038](#div-038) | both | Migration | Discharged, `0ea5624` and `e75996c` | Routine naming has no disambiguator |
| [DIV-039](#div-039) | both | Contradiction | Discharged, `0ea5624` and `e75996c` | Determinism stated over all output |
| [DIV-040](#div-040) | `CLAUDE.md` | Contradiction | Discharged, `0ea5624` | `tpl cache` is absent, and the auxiliary set is closed |
| [DIV-041](#div-041) | `CLAUDE.md` | Migration | Due | The target matrix is deferred; this specification now fixes it, and Linux is `musl` |
| [DIV-042](#div-042) | `README.md` | Contradiction | Discharged, `e75996c` | The JSON error envelope, the `kind` field, and `did_you_mean` |
| [DIV-043](#div-043) | `README.md` | Contradiction | Discharged, `e75996c` | The `.cfg` is now read strictly; an unrecognised key is fatal |
| [DIV-044](#div-044) | `README.md` | Contradiction | Discharged, `e75996c` | The four fields of `tpl schema info` |
| [DIV-045](#div-045) | `CLAUDE.md` | Contradiction | Discharged, ninth edition | The release profile aborts on panic |
| [DIV-046](#div-046) | `README.md` | Overstatement | Discharged, `d8e7e8a` | The build sequences, in a repository with no crate |
| [DIV-047](#div-047) | `README.md` | Contradiction | Discharged, `e75996c` | The `render` flag table omits `--direct` and `--no-cache` |
| [DIV-048](#div-048) | `README.md` | Contradiction | Discharged, `e75996c` | The entry flag is `--database`, which is the global flag's name |
| [DIV-049](#div-049) | `README.md` | Contradiction | Discharged, `e75996c` | The entry flag table omits `--ca-file` and `--ca-path` |
| [DIV-050](#div-050) | `CLAUDE.md` | Overstatement | Partly, `CLAUDE.md` | The project tree, in a repository with no crate |
| [DIV-051](#div-051) | `CLAUDE.md` | Overstatement | Discharged, `d8e7e8a` | The eight `cargo` commands, in a repository with no crate |
| [DIV-052](#div-052) | `CLAUDE.md` | Overstatement | Discharged, `abbfe70` | The benchmark directory, in a repository with no benchmarks |
| [DIV-053](#div-053) | `README.md` | Contradiction | Due | Four commands said to perform no discovery at all |
| [DIV-054](#div-054) | `README.md` | Overstatement | Due | Two passages saying no command reaches a server |
| [DIV-055](#div-055) | `CLAUDE.md` | Contradiction | Discharged, `9562fb2` | A regression against a recorded baseline said to fail the change |
| [DIV-056](#div-056) | `README.md` | Contradiction | Due | The `.cfg` key space counted short of the eighteen keys of `FR-CONF-002` |

One passage of `CLAUDE.md` was read and found not to be a divergence, so it
has no entry and no row above. The reading and its grounds are at
*[A candidate read, and not recorded](#a-candidate-read-and-not-recorded)*,
at the foot of this file, with the sweep that closes the **Overstatement**
class in that file.

## DIV-001

**Target**: both. **Kind**: migration.

**Status**: partly discharged, re-read on 2026-09-21 at `db80114` and
`8f936d4`. The `CLAUDE.md` half is discharged by `0ea5624`. Two of the four
parts of the `README.md` half are discharged by `e75996c`, which rewrote that
file whole: the command reference and the exit code table are gone, and *Where
the truth lives* now states that nothing in the file is the contract and that
`specification/` governs where the two disagree. **Two parts are due.** The
*Configuration* and *Failure* sections survive as summaries, and each still
states figures this corpus owns — fifteen keys, the 4096-byte output cap, `78`
for a non-zero `password_command` exit, `64` for `add` against a name that
exists, `66` for `update` against one that does not, and `74` for a pipe
closed part-way through a JSON document. A summary that cites is not a second
source; a figure restated is one, and it is the copy nobody updates.



The root documents are the two places the CLI surface was described before this
specification existed. Both must now be reduced.

- `README.md` keeps its role as the entry door: what `tpl` is, how to install
  it, a short quick start, and a pointer to `/specification`. Its command
  reference, configuration reference, exit code table, and error-message section
  are owned here.
- `CLAUDE.md` keeps agent coordination: which agent does what, the mandatory
  skills, the workflow, and the validation pipeline. Its description of the
  three arms, the CLI surface, the `.tpl` project, exit codes, and error message
  format is owned here.

Content of those files that this edition does not cover — implementation
language and conventions, performance and resource budgets, supported platforms,
the MariaDB container discipline — is untouched by this entry.

## DIV-002

**Target**: `CLAUDE.md`, CLI design rules. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the sentence quoted below
with the CLI design rules that carried it. Nothing is owed. Re-read on
2026-09-21 at `8f936d4`, the state of `CLAUDE.md` today; nothing this entry
records has returned to it.



*Says*: "Nenhum comando aceita password em `argv`."
*Specification*: `FR-CFG-030` removes the flag, but `FR-CFG-031` and
`FR-CFG-032` keep two documented paths open — `--dsn` and `tpl cfg set` — with a
warning in the help rather than a refusal.
*Correction*: rewrite the sentence to say that no flag named `password` exists,
and that two documented paths remain open.

## DIV-003

**Target**: `README.md`, the flag tables of `database add` and `update` and of
`render`. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which rewrote `README.md` and took the
flag tables of `database add` and `update` and of `render` with them. Neither
the `--password` row nor any of the four short forms survives, and the file
now states that there is no `--password` flag and no `-p`. The global flag
table it leaves carries exactly the five short forms of `FR-GLOB-001` and says
they are the whole short-flag space of the tool. Nothing is owed. Re-read on
2026-09-21 at `db80114`.



*Says*: `--password`, short `-p`, "Password, written to `.cfg`"; and short
forms `-H` for `--host`, `-P` for `--port`, `-u` for `--user`, and `-s` for
`--set` on `tpl render`.
*Specification*: `FR-CFG-030` — the `--password` flag does not exist. And
`FR-GLOB-024` — the five short forms of `FR-GLOB-001` are the whole short-flag
set of the tool, and no local flag carries one.
*Correction*: remove the `--password` row, and remove the short-form column
entry from the `--host`, `--port`, `--user`, and `--set` rows. `--dsn`,
`--schema`, `--tls`, `--password-command`, `--ca-file`, and `--ca-path` are the
other flags of those two commands, per `FR-CFG-027`, and none has a short form
either.

*Extended in the fifth edition.* The entry previously covered `-p` alone, on
the ground that it collides with `-P` by case. `OQ-016` is now closed the other
way from the root document: rather than deciding which local short forms to
keep, `FR-GLOB-024` declares that there are none, so all four of the root
document's remaining short forms are wrong and not only the one that collides.
The reasoning is in `FR-GLOB-024`; the `-P`/`-p` collision it describes is the
worked example of why the whole one-letter space is reserved.

## DIV-004

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`: the five flags, the flag table that carried them and
the "console by default" framing are all gone, and the file now states twice
that a render writes to stdout and nowhere else — once under *What `tpl` is
for* and once below the quick start, where it says `tpl render` has no
`--output`, no `--format` and no `--pretty`. The atomic-write paragraph that
survives is about the `.tpl/.cfg` rewrite of `FR-CFG-041` and not about a
render destination, so it is not this entry's subject. Nothing is owed.



*Says*: `README.md` presents `--output`, `--output-dir`, `--output-name`,
`--no-clobber`, and `--dry-run`, with a paragraph on atomic writes;
`CLAUDE.md` presents `--output` and `--output-dir` as the explicit way to save a
result, and its exit code table attaches `73` and `74` to those destinations.
*Specification*: `FR-RND-028` — none of the five flags exists, stdout is the
only destination, and `FR-ERR-003` restricts `73` to `tpl init`.
*Correction*: remove the flags, the atomic-write paragraph, the "console by
default, file on request" framing, and the destination clauses from both exit
code tables.

## DIV-005

**Target**: `CLAUDE.md`, the `.tpl` project section. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the `.tpl` project section.
Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md`
today; nothing this entry records has returned to it.



*Says*: `tpl init`, `tpl database …` and `tpl config …` are the only commands
that write in `.tpl/`; all others treat it as read-only.
*Specification*: `FR-PROJ-023` — `tpl cache load` and every cached read command
also write to `.tpl/.cache/` on a miss.
*Correction*: rewrite the sentence to list the four writers.

## DIV-006

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the secrets and versioning
section. Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of
`CLAUDE.md` today; nothing this entry records has returned to it.



*Says*: any command that prints configuration always redacts passwords.
*Specification*: `FR-CFG-021` redacts in `cfg list` and `cfg database show`;
`FR-CFG-006` and `BR-CFG-002` make `tpl cfg get` a deliberate exception.
*Correction*: state the exception rather than leave it to be discovered.

## DIV-007

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which made the correction this entry asked for: the
file now reads that `EPIPE` on stdout exits `0` silently **in the ordinary
case**, and that a pipe closing part-way through a JSON document is `74`. That
is `FR-ERR-025` and `FR-ERR-026` in the file's own words. Nothing is owed.



*Says*: `EPIPE` on stdout exits `0` silently, in every case.
*Specification*: `FR-ERR-025` keeps `0` for the ordinary case, and `FR-ERR-026`
returns `74` when the pipe closes part-way through a JSON document.
*Correction*: refine the flat rule in both files.

## DIV-008

**Target**: `CLAUDE.md`, secrets and versioning. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the secrets and versioning
section. Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of
`CLAUDE.md` today; nothing this entry records has returned to it.



*Says*: `${VAR}` is substituted "em qualquer valor string".
*Specification*: `FR-CONF-015` limits expansion to six fields, and
`FR-CONF-016` and `FR-CONF-017` forbid it in `tls` and `password_command`.
*Correction*: narrow the statement to the enumerated field set.

## DIV-009

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed both passages: the command reference
that carried the claim under `tpl schema dump`, and the *Rendering without a
database* section that repeated it. What the file says now is that the dump is
a document a later `tpl render --context` reads, and it claims no equality
between the two. Nothing is owed.



*Says*: `tpl schema dump` produces exactly the JSON the render receives as
context.
*Specification*: `FR-SCH-018` — the dump carries only the server-derived part;
`FR-RND-024` shows that `vars`, `tpl`, and `now` are always injected by the
render.
*Correction*: restate the round-trip as "the dump supplies the server-derived
part of the context", in both files.

## DIV-010

**Target**: `README.md`, global flags. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which made the correction this entry
asked for: the file now states that `--format` defaults to `text`, fixed, with
no terminal detection anywhere. Nothing is owed. Re-read on 2026-09-21 at
`db80114`.



*Says*: `--format` "Defaults to `text` on a TTY, `json` otherwise".
*Specification*: `FR-OUT-001` and `FR-OUT-002` — the default is `text`, fixed,
with no terminal detection anywhere.
*Correction*: change the default and remove the TTY clause.

## DIV-011

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the porcelain command list.
Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md`
today; nothing this entry records has returned to it.



*Says*: `tpl render <template> [alvo] [flags]` — a positional render target.
*Specification*: `FR-RND-003` — the target is always a flag, and the template
name is the only positional.
*Correction*: change the usage line.

## DIV-012

**Target**: `CLAUDE.md`, porcelain commands. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the porcelain command list.
Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md`
today; nothing this entry records has returned to it.



*Says*: `tpl init` with no argument.
*Specification*: `FR-PROJ-012` — an optional positional path, defaulting to the
current directory.
*Correction*: add the optional argument.

## DIV-013

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which made both corrections this entry asked for: the
prose now calls `password_command` an argument **array**, executed directly,
with no shell and with shell metacharacters as literal arguments, and the
`.cfg` example writes it as the array `["security", "find-generic-password",
"-s", "tpl-reporting", "-w"]` — the line this entry spelled out. Nothing is
owed.



*Says*: `README.md` describes `password_command` as a "shell command whose
trimmed stdout is used as the password"; the `.cfg` example in `CLAUDE.md`
writes it as a TOML string, `password_command = "security …"`.
*Specification*: `FR-CONF-024` — executed directly, without a shell, from an
array; shell metacharacters are literal arguments. `FR-CONF-023` — stored in
`.cfg` as an array. `FR-CONF-025` splits a string supplied to a command, not
one found in the file, and `FR-CONF-035` refuses a string found in the file
with `78`.
*Correction*: in `README.md`, remove the word "shell", state the array form,
and rewrite the `.cfg` example line as
`password_command = ["security", "find-generic-password", "-s", "tpl-reporting", "-w"]`.

*Amended in the fifth edition.* Two things changed. `OQ-019` is closed by
`FR-CONF-035`: a `password_command` that is not a TOML array of strings is
`78`, so the string in the `README.md` example is no longer merely a
non-canonical spelling — it is a `.cfg` that `tpl` refuses to read, and the
correction is required rather than tidying. And the `CLAUDE.md` half of this
entry is discharged: that file no longer carries a `.cfg` example, having been
reduced to agent coordination, so only `README.md` is left to correct.

## DIV-014

**Target**: `README.md`, alias table. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the alias table with the
command reference. Neither alias row survives, and the file states no alias at
all. Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: `routines` aliases to `procs`, `routine` to `proc`.
*Specification*: `FR-CLI-011` — `rtns` and `rtn`, because a routine is a
procedure or a function.
*Correction*: change both rows.

## DIV-015

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`: there is no top-level `tpl database …` group and no
`tpl config …` group anywhere in the file, and every invocation in the quick
start and in *Reading the configuration back* is written `tpl cfg …` or `tpl
cfg database …`. Nothing is owed.



*Says*: a top-level `tpl database …` group with alias `db`, a separate
`tpl config …` group, and `tpl database test` in the quick start.
*Specification*: `FR-CFG-001` through `FR-CFG-003` — one `tpl cfg` group, with a
`database` subgroup aliased `db`, and `tpl cfg database test`.
*Correction*: rewrite both command lists and the quick start.

## DIV-016

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed both and said why: "There is no colour
anywhere, so there is no flag and no variable to turn it off." Nothing is
owed.



*Says*: a `--no-color` global flag, colour implied off when stdout is not a TTY
or `NO_COLOR` is set.
*Specification*: `NFR-DET-004` — no colour anywhere, so the flag does not exist
and the variable is not read.
*Correction*: remove the flag row and the variable from both files.

## DIV-017

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed both variables and corrected the
precedence to the two layers of `FR-CONF-029`: flag, then `.tpl/.cfg`, then
the built-in default, with the file stating in two places that no environment
variable configures behaviour and that `${VAR}` inside `.tpl/.cfg` is the only
environment read `tpl` performs. Nothing is owed.



*Says*: environment variables `TPL_DIR` and `TPL_DATABASE`, with `-d` overriding
`TPL_DATABASE`, which overrides `core.database`.
*Specification*: `FR-CLI-021` and `FR-CONF-030` — no environment variable is
read for behaviour, and precedence has two layers. `${VAR}` inside `.cfg`
remains the only environment read.
*Correction*: remove both variables and correct the precedence sentence in both
files.

## DIV-018

**Target**: `README.md`, TLS flag and the `.cfg` example. **Kind**:
contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114`. The flag row was
discharged by `26e1739`; the `.cfg`-example half is discharged by `e75996c`,
which rewrote the configuration section so that the prose names the five modes
of `FR-CONF-013` with `verify-identity` as the default, names `ca_file` and
`ca_path` as what supplies trust material to the two verifying modes, and
writes the example entry with `tls = "verify-ca"` and a `ca_file`. Nothing
this entry records is stated by the file.

**One residue, and it is not a divergence.** The correction asked for both keys
in the example and the example carries one. No requirement obliges an example
to exercise both, the prose beside it names both, and an entry kept open on the
shape of a remedy rather than on a statement still standing is an entry that
sends a reader to correct what is already correct — which is what the fifth
validation rule exists to stop.



*Says*: `--tls <mode>` is one of `disabled`, `preferred`, `required`, defaulting
to `preferred`. `CLAUDE.md` carries the same three-mode enumeration, as the
illustration of an enumerated flag value in its help-text rules.
*Specification*: `FR-CONF-013` — five modes, defaulting to `verify-identity`,
with `ca_file` and `ca_path` as new keys.
*Correction*: replace the enumeration and the default, and add the two keys to
the example. Correct the `CLAUDE.md` illustration to the five modes, or choose
a flag whose value set this specification does not fix.

## DIV-019

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`: `--all-tables` appears nowhere in the file,
`--pattern` appears only on `tpl schema tables`, where `FR-SCH-013` declares
it, and the render examples are one render each. Nothing is owed.



*Says*: `README.md` documents `--all-tables`, `--pattern` on `render`, and
repeatable object flags, with examples that concatenate many renders;
`CLAUDE.md` refers to `render --all-tables` in its performance budget.
*Specification*: `FR-RND-002`, `FR-RND-004`, and `FR-RND-007` — one render per
invocation, each object flag at most once, and no `--all-*` or `--pattern` on
`render`.
*Correction*: remove the flags and the multi-render examples, and replace them
with the caller-side loop of `UC-008`. The performance budget in `CLAUDE.md`
needs a different unit of measure.

## DIV-020

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which made both corrections: the `.tpl` tree now
carries `.cache/` with a comment saying a read creates it, and the generated
`.gitignore` is shown with its two lines, `.cfg` and `.cache/`. Nothing is
owed.



*Says*: the `.tpl/` tree contains `.cfg`, `.gitignore`, and `templates/`, and
the generated `.gitignore` holds one line.
*Specification*: `FR-PROJ-002` and `FR-PROJ-017` — the tree also contains
`.cache/`, and the generated `.gitignore` holds two lines.
*Correction*: update both trees and the `.gitignore` content in both files.

## DIV-021

**Target**: `README.md`, second arm. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the second arm's command
reference. The word *lint* is in no part of the file, `tpl template check` is
shown with no argument, and `tpl template path` no longer appears at all.
Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: `tpl template check <name>` "Parse and lint a template", with a required
name.
*Specification*: `FR-TMPL-017` through `FR-TMPL-019` — `check` parses only,
there is no lint, and the name is optional and repeatable.
*Correction*: remove the word "lint" and make the argument optional. The same
applies to `tpl template path`, which also takes an optional name per
`FR-TMPL-021`.

## DIV-022

**Target**: `README.md`, `--pattern`. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the sentence with the alias
table it sat under. Nothing in the file attributes case sensitivity to the
server. Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: "Case sensitivity follows the server collation."
*Specification*: `FR-SCH-013` and `FR-SCH-014` — evaluated locally, ASCII
case-insensitive, independent of the server.
*Correction*: replace the sentence.

## DIV-023

**Target**: both. **Kind**: migration.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, and all three faults are gone at once: the table
lists exactly the seven global flags of `FR-GLOB-001`, `--output`, `--format`
and `--no-color` are not among them, `--timeout` is, and the paragraph below
it names `--format`, `--pretty`, `--direct` and `--no-cache` as local flags
and points at [global-flags.md](global-flags.md) for the list. Nothing is
owed.



Both global flag tables are wrong in three ways at once: they list `--output`,
`--format`, and `--no-color` as global; they omit `--timeout`; and they do not
mention that `--direct` and `--no-cache` exist at all.
*Specification*: `FR-GLOB-001` — exactly seven global flags — and `FR-GLOB-021`,
which lists the four local flags and the commands that declare each.
*Correction*: replace both tables with a pointer to
[global-flags.md](global-flags.md).

## DIV-024

**Target**: `CLAUDE.md`, project discovery. **Kind**: migration.

**Status**: discharged by `0ea5624`, which replaced the description of the
walk with a pointer to [project-and-discovery.md](project-and-discovery.md) —
the correction this entry asked for, made as it asked for it. Nothing is owed.
Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md` today; nothing
this entry records has returned to it.



*Says*: the walk climbs until a `.tpl/` folder is found, and `TPL_DIR` skips
discovery.
*Specification*: `FR-PROJ-005` adds the boundary at the mount point;
`FR-PROJ-009` through `FR-PROJ-011` add canonicalisation and the ownership and
mode checks; `FR-GLOB-009` replaces `TPL_DIR` with `--tpl-dir`.
*Correction*: the description is incomplete rather than wrong; replace it with a
pointer to [project-and-discovery.md](project-and-discovery.md).

*Amended in the eighth edition.* This entry named a boundary at the home
directory beside the mount point. `FR-PROJ-005` drops it, because locating it
required reading `HOME` and `FR-CLI-021` admits no such read. The correction
owed to `CLAUDE.md` is unchanged in kind and shorter by one clause.

## DIV-025

**Target**: `CLAUDE.md`, functional specification section. **Kind**: migration.

**Status**: discharged, and no commit discharged it. The clause quoted below
is not in `CLAUDE.md` today and is in no committed state of it: searching the
history for the quoted words finds one occurrence in the repository, in this
file, at `1352a2d`. The subsection it asks to remove therefore never existed
in the file, and nothing is owed. Recorded rather than deleted, because an
entry raised against an unverified reading of a target is the same defect as
an entry left standing after the reading went stale, and the identifier must
resolve to that explanation. Re-read on 2026-09-21 at `8f936d4`, the state of
`CLAUDE.md` today; nothing this entry records has returned to it.



*Says*: "A pasta `/specification` ainda não existe neste repositório", with an
instruction to remove the subsection once the bootstrap is done.
*Specification*: the folder now exists, and this file is part of it.
*Correction*: remove the subsection, as that text itself instructs.

## DIV-026

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed the filter table and both template
examples. Neither filter name appears in the file, and the only mention of the
mapping is the `.tpl` tree's line for `templates/rust/_types.jinja`, described
as a column-to-Rust-type mapping delivered as a macro — which is what
`FR-ENV-009`, `FR-ENV-010` and `FR-PROJ-017` say it is. Nothing is owed.



*Says*: `CLAUDE.md` lists `rust_type` among the SQL and code filters;
`README.md` lists `rust_type` and `go_type` in its filter table and uses
`rust_type` in two template examples.
*Specification*: `FR-ENV-009` and `FR-ENV-010` — neither filter exists. The
mapping is delivered as a template macro at `.tpl/templates/rust/_types.jinja`,
written by `tpl init` per the amended `FR-PROJ-017`, so that the opinion it
encodes belongs to the project.
*Correction*: remove both filters from both tables, and rewrite the two
`README.md` examples to call the macro instead of the filter.

## DIV-027

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed the filter table; neither word appears
in the file. Nothing is owed.



*Says*: both files list `plural` and `singular` as naming filters.
*Specification*: `FR-ENV-012` and `FR-ENV-013` — neither exists. Correct English
inflection is a project in itself, and a wrong plural on a table name that is
not English is guaranteed noise in generated code.
*Correction*: remove the row from both tables.

## DIV-028

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed the template-environment section
entirely: no extension rule survives, and neither does the sentence framing
two behaviours as differing from stock Jinja2. Nothing is owed.



*Says*: `CLAUDE.md` requires auto-escaping to be enabled by extension for
`.html`, `.xml`, and `.htm`; `README.md` states that auto-escaping is off "for
every extension except `.html`, `.htm`, and `.xml`".
*Specification*: `FR-ENV-026` through `FR-ENV-028` — auto-escaping is off
always, no rule is keyed on any extension, and `escape` is an explicit filter.
*Correction*: remove the extension rule from both files. In `README.md` the
surrounding sentence, "Two behaviours differ from stock Jinja2 defaults", must
also be corrected: after this change the two are strict undefined variables and
the preserved trailing newline of `FR-SEM-003`, not auto-escaping.

## DIV-029

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which made the correction this entry asked for: the
file states that `tpl init` writes **five** artefacts and names them, and the
`.tpl` tree shows `templates/rust/_types.jinja` among them. Nothing is owed.



*Says*: both files present `tpl init` as creating four artefacts, and list them.
*Specification*: `FR-PROJ-017`, as amended — five artefacts. The fifth is
`.tpl/templates/rust/_types.jinja`.
*Correction*: add the row to both tables, and add the file to the `.tpl` tree
shown in each. `README.md` additionally states that `.tpl/.gitignore` holds one
line, which `DIV-020` already corrects to two.

## DIV-030

**Target**: `CLAUDE.md`, project invariant 1. **Kind**: contradiction.

**Status**: discharged by `0ea5624`. The promise survives in `CLAUDE.md`, but
the session statement no longer follows it: the detail is deferred to
[server-contract.md](server-contract.md), which is the correction this entry
asked for. Nothing is owed. Re-read on 2026-09-21 at `8f936d4`, the state of
`CLAUDE.md` today; nothing this entry records has returned to it.



*Says*: "O `tpl` **nunca** emite DDL, DML ou qualquer statement de escrita",
followed immediately by the read-only session statement, presenting the session
setting as the mechanism that delivers the promise.
*Specification*: `FR-SRV-006` through `FR-SRV-011` and `BR-SRV-002` — the
promise has two parts and only one of them prevents. The guarantee is the closed
list of statements `tpl` will issue. The session setting is defence in depth: it
makes a write fail, it does not stop the connection attempting one, and it
constrains the transaction rather than the session. The specification adds a
read-back of the setting, and a limit of at most one connection per invocation.
*Correction*: state the closed list as the guarantee, keep the session setting as
defence in depth, and add the read-back and the connection limit. The `78` on
failure, and the absence of a flag to disable any of it, are unchanged.

## DIV-031

**Target**: `CLAUDE.md`, project invariant 1. **Kind**: contradiction.

**Status**: discharged by `0ea5624`. **This is the entry that showed the
register was unsafe to presume current.** The `SHOW` clause was removed
several sprints before the eleventh edition amended this entry for a miscount,
and the amendment revisited the count without checking whether the clause it
corrects was still in the file. Nothing is owed, and nothing was owed then
either. Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md` today;
nothing this entry records has returned to it.



*Says*: the catalogue is read exclusively through `INFORMATION_SCHEMA` "e,
quando estritamente necessário, `SHOW`".
*Specification*: `FR-SRV-006` and `FR-SRV-007` — the closed list has four
entries and `SHOW` is not one of them. A statement outside the list is not
permitted however necessary it seems.
*Correction*: remove the `SHOW` clause. The prohibition on `mysqldump` and on
any external process is unchanged and is restated by `FR-SRV-007`.

*Amended in the eleventh edition.* The clause above said **three** entries. It
was written in the second edition, when the list had three; the fifth edition
added a fourth — the read-back of the session read-only variable, which the
tenth edition named `@@session.tx_read_only` — and this entry was not revisited
with it. `FR-SRV-006` carries four rows, counted in its own table on
2026-09-11, and `FR-SRV-012` requires a test that expects four kinds of
statement and no fifth. **Only the count changes.** `SHOW` is no more one of
the four than it was one of the three, and the correction owed to `CLAUDE.md`
is the one stated above and nothing more.

## DIV-032

**Target**: `CLAUDE.md`, project structure. **Kind**: contradiction.

**Status**: **due**, re-read on 2026-09-22 at `9562fb2`. The sentence stands
in *Estrutura do Projecto*, below the project tree and unchanged since
`3f65b5f`, through every commit that has touched `CLAUDE.md` since — the three
most recent of them `3a360d6`, `6a66d14` and `9562fb2`, none of which reaches
this passage. It is one of the nine entries this register still owes something
on, and one of the five owed to that file.

*Corrected in the thirty-sixth edition, with `DIV-041`.* The sentence read
*one of the seven corrections this register still owes and one of the six owed
to that file*, where the Overview counted nine and six. The thirty-third
edition corrected the identical sentence in `DIV-052` and swept no further, so
two entries kept a figure the Overview had left behind; the thirty-sixth
edition moved the Overview again, to ten and seven, which made the second half
wrong as well. Both halves are now the Overview's, stated in the Overview's own
terms so that a reader comparing the two is comparing one count. This is the
third validation rule of the [README](README.md#maintenance-debt) applied to a
count rather than to a record: correcting one restatement of a figure
establishes nothing about the others.

*Corrected again in the thirty-seventh edition, with `DIV-041`, and the sweep
was made first.* Discharging `DIV-052` and `DIV-055` takes the Overview to
eight entries owing and five owed to `CLAUDE.md`, and both halves here follow
it. Every restatement of either count was found before any was changed — this
entry, `DIV-041` and `DIV-052`, which is the whole of them — so that the defect
the note above records is not repeated by the edition correcting it.



*Says*: "as suas structs são a superfície pública documentada", of the `model/`
module.
*Specification*: only the JSON document and the command line are contract. The
library carries no compatibility guarantee, and the document contract lives in
[context-document.md](context-document.md) rather than in a set of Rust types.
*Correction*: state that the contract is the document, not the types that
produce it. Five questions this edition would otherwise have had to settle —
owned versus borrowed types, public fields versus accessors, newtypes for names,
whether the serialisation crate is a public dependency, and `#[non_exhaustive]`
— are architecture decisions rather than functional requirements, and belong in
an architecture decision record.

## DIV-033

**Target**: `CLAUDE.md`, render context. **Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the render context section
and replaced it with a pointer to
[template-environment.md](template-environment.md) — one of the two
corrections this entry offered. Nothing is owed. Re-read on 2026-09-21 at
`8f936d4`, the state of `CLAUDE.md` today; nothing this entry records has
returned to it.



*Says*: "Filtros, testes e funções registados no `Environment` são superfície
pública: acrescentar é permitido, renomear ou remover é uma quebra de
compatibilidade."
*Specification*: `FR-ENV-001` through `FR-ENV-004` — the surface has three
groups and only the first carries that guarantee unqualified. The named
inherited filters of `FR-ENV-018` are guaranteed only against a pinned engine
minor version, and everything else the engine offers works but is guaranteed by
nobody.
*Correction*: replace the sentence with the three groups, or with a pointer to
[template-environment.md](template-environment.md).

## DIV-034

**Target**: both. **Kind**: contradiction.

**Status**: discharged. The `CLAUDE.md` half was discharged by `0ea5624`,
which removed the render context table. The `README.md` half is discharged by
`e75996c`, which rewrote that file and took with it both sentences the *Says*
clause quotes; neither is in the file at `db80114`, which is its state today.
Nothing remains: the three further corrections the seventh edition added and
the exclusion count the eighth corrected all qualified the passage that has
gone. Re-read again on 2026-09-21, in the pass below, and unchanged.

*Re-read in the thirty-first edition, under the fifth validation rule of the
[README](README.md#maintenance-debt).* The rule obliges this register to be
re-read against a target file whenever that file changes, and an entry found
discharged to record the commit that discharged it. The root `README.md` has
been edited three times since this entry was last checked — `e75996c`,
`db7337d` and `db80114` — and the re-read was made because this edition amends
the entry, which is the trap the eleventh edition fell into with `DIV-031`:
amending an entry without checking whether the sentence it corrects is still
in the file. This entry was re-read; the register as a whole was not, and
what that leaves owed is recorded under the index above.



*Says*: `README.md` states that a `table` carries `name`, `comment`, `engine`,
`charset`, `collation`, `columns`, `primary_key`, `indexes`, and
`foreign_keys`, and that a `column` carries `name`, `position`, `data_type`,
`nullable`, `default`, `comment`, "and the raw catalogue attributes MariaDB
reports for it". `CLAUDE.md` names the same set of model types.
*Specification*: the list is incomplete and, in two places, wrong in shape. A
table also carries `table_type` per `FR-CAT-002`, `referenced_by` per
`FR-CAT-013`, its triggers per `FR-CAT-014`, and its `CHECK` constraints per
`FR-CAT-015`. A column's `default` is a discriminated structure per
`FR-CTX-012`, not a value; its type is `column_type` plus eight decomposed parts
per `FR-CTX-015`, not "the raw catalogue attributes"; and it carries
`table_name` per `FR-CTX-019`. Sixteen volatile catalogue fields are excluded
outright by `FR-CAT-024`.
*Correction*: replace both descriptions with a pointer to
[catalogue-coverage.md](catalogue-coverage.md) and
[context-document.md](context-document.md).

*Amended in the seventh edition, and the correction is now cheap to make.*
When this entry was written the specification could not offer a complete
replacement, because the field lists themselves were unobserved. They are
observed and written: `FR-CAT-039` through `FR-CAT-051` fix the catalogue
field list of every object kind an entry of
[open-questions.md](open-questions.md) had asked for, and what the model takes
from each, and `BR-CAT-005` states the rule by which one becomes the other. A
**table** is not among them, and what this entry offers for a table instead is
`FR-CAT-053`, the index naming every property a table object carries beside the
requirement that fixes it. Three further
corrections join the ones above and each is a fact the root document states
wrongly rather than incompletely: a table carries **no character set**, only
a collation, per `FR-SCH-009` as amended; a column's `default` has **three**
`kind` values and not four, per `FR-CTX-013`; and a table's comment for a
**view** is the literal string `VIEW`, which `FR-CAT-040` keeps out of the
model. The exclusion list of `FR-CAT-024` grew from thirteen fields to
sixteen in the same edition.

*Amended in the eighth edition.* The *Specification* clause above still said
thirteen while the note beside it said sixteen. `FR-CAT-024` carries sixteen
rows, counted in its own table on 2026-09-10; the clause is corrected and the
two now agree.

*Corrected in the thirty-first edition: the range cited did not cover what the
sentence claimed for it.* The seventh edition's note said `FR-CAT-039` through
`FR-CAT-051` fix the catalogue field list of **every object kind**, and it was
already false of the table when it was written — the twenty-third edition
corrected the same sentence where it appears in the
[README](README.md#seventh-edition--the-catalogue-field-lists), and this copy
of it was passed over. It has since drifted further: the table's own statement
is `FR-CAT-053`, which is outside the range, and that requirement records in
its own text that a table has no catalogue field list in this corpus and names
the observation pass that would record one. The sentence now says what the
range fixes and names what stands in its place, so the replacement this entry
offered is complete for a table too — through a property index rather than
through a field list.

*Rejected: extending the cited range to `FR-CAT-053`.* The identifiers are not
contiguous with it and never will be: `FR-CAT-052` and `FR-CAT-054` through
`FR-CAT-057` sit between and are not field lists, and a range that has to be
read as a set is not a range. Also rejected: dropping the sentence, which
would leave this entry's *Correction* clause pointing at two files without
saying that what it points at is now written down — which was the whole of
what the seventh edition added here.

## DIV-035

**Target**: `CLAUDE.md`, non-functional requirements. **Kind**: migration.

**Status**: discharged by `0ea5624`, which replaced the budget table with the
statement that the numeric targets do not live in that file and a pointer to
[performance-requirements.md](performance-requirements.md). The fifth
edition's requirement that `CLAUDE.md` keep no figure at all is met. Nothing
is owed. Re-read on 2026-09-21 at `8f936d4`, the state of `CLAUDE.md` today;
nothing this entry records has returned to it.



*Says*: a table of four performance budgets with figures in milliseconds, a
peak-memory figure in MiB, and a set of rules derived from them.
*Specification*: [performance-requirements.md](performance-requirements.md) —
the corpus keeps six requirements of form, the three reference workloads, and
the measurement protocol; the figures stay outside it. The budget set itself
changed: `--version` and `--help` are separate lines, the
`render --all-tables` line is replaced by the 200-invocation loop because
`FR-RND-007` removed the flag it measured, and four budgets are added — the
cache-served read per object, which is the one normative target, the failure
path, `tpl help --format json`, and the loop.
*Correction*: replace the four-row table with the set of `NFR-PERF-014` and
point the derived rules at
[performance-requirements.md](performance-requirements.md).

*Amended in the fifth edition.* The correction previously offered a choice —
keep the figures in `CLAUDE.md` or move them to `BENCHMARKS.md` — and cited
`OQ-051` through `OQ-060` for their being unmeasured. Those ten entries are
closed and the choice is settled by what the specification now does with them:
**the four figures `CLAUDE.md` states are adopted into `NFR-PERF-014` as
provisional**, marked under `NFR-PERF-019`, and each is removed from that table
by the same step that records a real measurement in `BENCHMARKS.md`, per
`NFR-PERF-020`. `CLAUDE.md` should therefore keep no figure at all: a third
copy would be the one nobody updates when the first measurement lands.
`NFR-PERF-014` records the provenance of each of the four, so removing them
from `CLAUDE.md` loses nothing.

## DIV-036

**Target**: `CLAUDE.md`, project structure and the MariaDB testing section.
**Kind**: migration.

**Status**: **partly discharged**, re-read on 2026-09-21 at `8f936d4`, and
re-read again the same day against the working tree. The project-tree half
remains discharged by `87dd6e3`: the tree's fixture line describes what
`scripts/mariadb/` holds instead of enumerating its files, so there is no list
for `seed-bench.sql` to be missing from. **The file itself is no longer owed.**
`scripts/mariadb/seed-bench.sql` is in the working tree, it loads on each of
the four series of `FR-SRV-015`, and it realises both `WL-001` and `WL-003` at
the counts `performance-requirements.md` states — so the discharge is the
repository catching up with the document, in the second sense a **Discharged**
value takes under *[How an entry is read](#how-an-entry-is-read)*. **One part
is still due.** The coverage paragraph of *Testes contra MariaDB* still names
`scripts/mariadb/setup.sql` and `seed.sql` and no third script, and that
sentence is now wrong about a directory that has one.



*Says*: `scripts/mariadb/` holds a `Dockerfile`, `setup.sql`, and `seed.sql`,
and the two SQL scripts must cover the read surface exhaustively.
*Specification*: `WL-001` requires a fourth file, `seed-bench.sql`, and
`BR-PERF-002` keeps it separate from `seed.sql` on purpose: `seed.sql` is
exhaustive variety at minimal volume, for correctness, and `seed-bench.sql` is
volume at minimal variety, for measurement. One fixture serving both would hide
an N+1, which is invisible at ten tables.
*Correction*: name `seed-bench.sql` in the coverage paragraph of the testing
section, which is all that remains. *Narrowed in the thirty-third edition*: the
correction read *add `seed-bench.sql` to the tree and to the testing section*,
and two of its three parts are done — the tree stopped listing files at
`87dd6e3`, and the file itself is in the working tree. A remedy that outlives
its condition sends an editor to make a change that is already made.
*Amended in the sixth edition, and largely discharged.* Three of the four
files now exist and the container is buildable at each of the four series of
`FR-SRV-015`, so the two larger parts of this entry are settled: `CLAUDE.md`
no longer describes the container in the singular, and the DDL of `setup.sql`
and `seed.sql` is DDL all four accept. What is still owed is one line of the
project-structure tree and one of the testing section, both naming
`seed-bench.sql`, and the file itself.

This entry previously recorded that none of the four files existed and that
**all twenty-four remaining open questions were blocked by that absence**.
Neither half survives. The fixture exists; a first pass against it closed five
entries; a second pass recorded the field lists themselves and closed twenty
more; and the last entry, `OQ-042`, closed on a stated limit in `FR-SRV-041`
rather than on evidence, because the server it wanted is not one a fixture of
MariaDB servers can hold. **No open question is blocked by this entry, and
none is open at all.** The mandated tests of `BR-SCH-004`, `FR-SRV-029` and
`BR-SEC-003` are not blocked by this entry either, and neither are the five
budgets that needed a fixture: `seed-bench.sql` exists, and `BR-PERF-007` now
records that no budget is blocked by the fixture.

*Corrected in the twenty-sixth edition.* The sentence above said those three
tests were "blocked only by `tpl` not existing". The binary exists, so the
clause named a condition that does not hold, and what this entry is entitled to
say about the three is only that its own absence does not block them: what each
of them waits on is the command it drives. `BR-SCH-004` records its own block,
in [schema-commands.md](schema-commands.md), and the same claim in the
*Maintenance debt* section of [README.md](README.md) is corrected with it. The
debt of this entry is unchanged — `seed-bench.sql`, and the two lines that name
it.

*Amended in the seventh edition.* What this entry still owes is unchanged and
is now the whole of it: one line of the project-structure tree, one line of
the testing section, and the `seed-bench.sql` file itself.

*Related, and not covering it: `DIV-052`.* The *Disciplina de medição*
subsection of the same file states that the benchmarks run against the
containers' dataset, which is the fixture this entry was raised over. The two
entries are not one. This one is a **migration** whose remaining correction is
a sentence naming a third script, and making it leaves that subsection saying
benchmarks live in `benches/`, a directory the repository has not got.
`DIV-052` is an **overstatement** whose correction is a qualifier on a tense,
and it produces no benchmark. Neither discharges the other, and the link is
written in both directions so that whoever pays one is told the other still
stands.

*Amended in the thirty-third edition, with the entry's Status.* The clause read
that the subsection's dataset is *in part the fixture this entry still owes*
and that *the budgets over `WL-001` wait on `seed-bench.sql`, per
`BR-PERF-007`*. Neither is true of a tree that holds the file, and
`BR-PERF-007` no longer says the second. What the two entries share is
narrower than it was and the link is kept for it: `DIV-052`'s sentence is still
false, and this entry's remaining line still describes a fixture wrongly.

*Corrected in the thirty-seventh edition, because half of the link has been
paid and this entry is still owed.* `DIV-052` is discharged: `abbfe70` created
`benches/` and put the measurement harness in it, so the sentence that entry
was raised over is true and the directory the paragraph above calls one *the
repository has not got* is in the working tree. **What this entry still owes is
unchanged**, and so is the reason the two were never one: the coverage
paragraph of *Testes contra MariaDB* still names two of the fixture's three SQL
scripts, and making that correction produces no benchmark and touches no other
section. The link is kept because it records why neither entry discharged the
other, which is the fact that decided how both were raised.

## DIV-037

**Target**: both. **Kind**: contradiction.

**Status**: **partly discharged**, re-read on 2026-09-21 at `db80114` and
`8f936d4`, and only `CLAUDE.md` now owes anything. The `README.md` version row
was discharged by `011c059`; **the second `README.md` part is discharged by
`e75996c`**, which removed the claim that the generated example template
renders without error against any table of any MariaDB database, and which
states instead that MySQL is not a target and that a server which is not
MariaDB is refused rather than read. `CLAUDE.md`'s silence on the window
remains discharged by `0ea5624`. **One part is due.** *Testes contra MariaDB*
still observes that MariaDB and MySQL diverge in `INFORMATION_SCHEMA` without
saying that a server which is not MariaDB is refused with `78`, per
`FR-SRV-003` — the half of this entry that was never about a version number.



*Says*: `README.md` states, in its requirements table, `MariaDB 10.6 or later`,
and states that the generated example template "renders without error against
any table of any MariaDB database". `CLAUDE.md` states no version at all, and
observes that MariaDB and MySQL diverge in the catalogue without saying what
`tpl` does about it. The `README.md` line is wrong in both directions: `10.6`
left community maintenance on 2026-07-06, and `or later` states no ceiling.
*Specification*: `FR-SRV-001` — the supported servers are the MariaDB series
that belong to one of the three most recent major families **and** are under
community maintenance, which on 2026-09-10 is `12.3`, `11.8`, `11.4` and
`10.11`, per `FR-SRV-015`. `FR-SRV-020` — a MariaDB below that window is
refused with `78`, carrying the message of `FR-SRV-030`. `FR-SRV-003` — a
server that is not MariaDB is refused with `78`, its `cause` naming the product
the server reported, because three verified divergences would make the model
silently wrong rather than empty.

*Amended in the sixth edition*: this entry cited `kind: server_version_unsupported`
and `kind: server_not_mariadb` as the specification's position. `FR-ERR-015`
withdrew the `kind` field in the fifth edition and the two owning requirements
were amended with it; this entry was missed.
*Correction*: qualify the `README.md` sentence to a supported server, state that
MySQL is refused rather than attempted, and — wherever either file needs to name
the supported versions — cite `FR-SRV-001` rather than copy the table of
`FR-SRV-015`, per `BR-SRV-005`. A second copy of four version numbers in a file
this specification does not own is the copy nobody will re-verify.
*Amended in the fourth edition*: this entry previously recorded the correction
against a minimum of MariaDB 10.6. That floor is withdrawn — 10.6 left community
maintenance on 2026-07-06 — and the correction owed is now the window, not a
floor.

## DIV-038

**Target**: both. **Kind**: migration.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which removed the command reference and the exit-code
discussion together: `tpl schema routine` appears only as a name in the
implementation-status table, `tpl render --routine` appears nowhere, and no
passage of the file names a routine in either form. Nothing is owed.



*Says*: `README.md` documents `tpl schema routine <name>` and the repeatable
`--routine <name>` flag with a bare name; `CLAUDE.md` does the same in its
command list.
*Specification*: `FR-SCH-008`, as amended — wherever a command names one
routine, the qualified forms `procedure:<name>` and `function:<name>` are
accepted; and `FR-SCH-010`, as amended — a bare name matching both a procedure
and a function is `64`, never resolved in favour of either.
*Correction*: document the qualified form on `tpl schema routine`,
`tpl render --routine`, and both `tpl cache` subcommands that name an object,
and add the ambiguity case to the exit-code discussion. `DIV-014`, which
corrects the aliases of the same commands, applies to the same rows.

## DIV-039

**Target**: both. **Kind**: contradiction.

**Status**: discharged, re-read on 2026-09-21 at `db80114` and `8f936d4`. The
`CLAUDE.md` half was discharged by `0ea5624`. The `README.md` half is
discharged by `e75996c`, which made the correction this entry asked for word
for word: "**stdout is byte-identical for the same invocation against the same
state**; stderr is not, and is not contract — at raised verbosity it carries
phase timings, which differ on every run." Nothing is owed.



*Says*: "The same invocation against the same database produces byte-identical
output", without qualification, in `README.md`; `CLAUDE.md` states the same rule
in its determinism section.
*Specification*: `NFR-DET-001`, as amended — byte-identical **stdout**. The
diagnostic output written to stderr is neither deterministic nor contract, and
cannot be: `FR-GLOB-017` requires phase timings at `INFO`, which differ on every
run by construction.
*Correction*: add the word stdout, and state that stderr is excluded. The `now`
exception stated in both files is unaffected and remains correct.

## DIV-040

**Target**: `CLAUDE.md`, the three arms and the porcelain command list.
**Kind**: contradiction.

**Status**: discharged by `0ea5624`, which removed the three arms and the
porcelain command list together. `tpl cache` is absent from `CLAUDE.md` still,
but so is every other command, and the sentence that closed the auxiliary set
with "apenas" is gone. Nothing is owed. Re-read on 2026-09-21 at `8f936d4`,
the state of `CLAUDE.md` today; nothing this entry records has returned to it.



*Says*: "Fora dos três braços existe apenas a gestão do projecto (`tpl init`,
`tpl database …`, `tpl config …`)" — and the porcelain command list names no
other group.
*Specification*: `FR-CLI-010` — the top-level commands are `schema`,
`template`, `render`, `cache`, `cfg`, `init`, `help`, and `version`.
`FR-CLI-008` makes `tpl cache` a group node, and
[cache-commands.md](cache-commands.md) gives it three subcommands. The
auxiliary set is therefore four groups and not three, and `tpl cache` is absent
from `CLAUDE.md` altogether — including from the sentence that closes the set
with "apenas".
*Correction*: add `tpl cache load|clean|status` to the porcelain command list,
and open the auxiliary sentence to include it. `DIV-015`, which replaces
`tpl database …` and `tpl config …` with `tpl cfg …`, applies to the same
sentence.

## DIV-041

**Target**: `CLAUDE.md`, supported platforms. **Kind**: migration.

**Status**: **due**, re-read on 2026-09-22 at `9562fb2`. The deferral stands
verbatim under *Plataformas Suportadas*, unmoved by `3a360d6`, `6a66d14` or
`9562fb2`, and the two rules beside it still restate what `NFR-PERF-018` and
`NFR-PERF-012` carry. It is one of the nine entries this register still owes
something on.

*Corrected in the thirty-sixth edition, with `DIV-032`*, and for the reason
given there.

*Corrected again in the thirty-seventh edition, with `DIV-032`*, and for the
reason given there. The re-read against `9562fb2` covered the whole of this
entry's **Target**: the deferral is untouched, and so are the two rules beside
it — including the one this entry quotes as *correctly* stating that a
performance baseline always names the target it was measured on, which that
commit left exactly as it stood.



*Says*: "A matriz concreta de alvos — target triples, escolha de libc,
linkagem e forma de empacotar o binário — é decisão de arquitectura em curso e
**não se fixa aqui**." It also states, correctly, that no target is second
class and that a performance baseline always names the target it was measured
on.
*Specification*: `NFR-PERF-018` — the target set is exactly four, and it is
fixed:

| System | Architecture | Target |
|---|---|---|
| Linux | amd64 | `x86_64-unknown-linux-musl`, statically linked |
| Linux | arm64 | `aarch64-unknown-linux-musl`, statically linked |
| macOS | amd64 | `x86_64-apple-darwin` |
| macOS | arm64 | `aarch64-apple-darwin` |

The libc question the sentence defers is answered: Linux is `musl`, linked
statically, so one artefact per architecture runs wherever that architecture
does with no dependency on the host's C library version. The `gnu` triples are
not targets.
*Correction*: replace the deferral with a pointer to `NFR-PERF-018`, and drop
the two sentences that describe the matrix as an open architecture decision.
The two rules beside it are correct and this specification now carries them
both — no target is second class, per `NFR-PERF-018`, and a baseline names its
target, per `NFR-PERF-012` — so they should point there rather than restate it.

*What the correction must not lose.* The `musl` static linkage has an
observable consequence, and it is recorded in `NFR-PERF-018` and again at
`FR-CONF-005`: a statically linked `musl` binary resolves names through
`musl`'s own `getaddrinfo`, which does not load the platform's name-service
modules, so a name resolvable only through such a module does not resolve for
`tpl` on a Linux target. The outcome is an ordinary `69`, per `FR-ERR-027`, and
what it obliges is precision in the `cause` line, per `FR-ERR-034`. Whoever
edits `CLAUDE.md` should not present the linkage as a pure packaging decision.

## DIV-042

**Target**: `README.md`, the exit codes and error messages section. **Kind**:
contradiction.

**Status**: discharged by `e75996c`, which made the correction this entry
asked for. The sentence, the JSON envelope and the sentence about `kind` are
gone with the exit codes and error messages section that carried them, and
what stands in their place is `FR-ERR-033` in the file's own words:
"`--format` applies to a result and never to a failure … the **exit code** is
the machine-comparable signal", above a pointer to
[errors-and-exit-codes.md](errors-and-exit-codes.md) for the table and for
what each `cause` names. The four-line example survives and is correct.
Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: "Under `--format json`, errors are JSON too", followed by the envelope
verbatim —
`{"error":{"exit":66,"code":"EX_NOINPUT","kind":"table_not_found", … ,"did_you_mean":["orders"]}}`
— and then "`kind` is a stable enumerated identifier meant to be compared
programmatically; `message` is meant to be read."
*Specification*: `FR-ERR-033` — **no error is ever emitted as JSON**, from any
command, at any verbosity, under any value of `--format`. When the outcome is
an error, `--format` is ignored, the four labelled lines of `FR-ERR-008` go to
stderr, and stdout is empty. Three of the four things the passage states are
withdrawn with the document that carried them: `FR-ERR-014`, the document;
`FR-ERR-015` and `FR-ERR-016`, the `kind` field and its compatibility rule;
and the `did_you_mean` array, per the amendment to `FR-ERR-023`. Nearest-match
suggestion survives, in the text `hint` line.
*Correction*: delete the sentence, the JSON block, and the sentence about
`kind`. Replace them with the statement that `--format` applies to a result and
never to a failure, that the exit code is the machine-comparable signal, and
that `FR-ERR-034` states per code what the `cause` line names. Whatever remains
of the four-line example in that section is correct and stays.

*Note on scope.* The dossier for the fifth edition recorded this envelope as
appearing in `CLAUDE.md`. It no longer does: that file has been reduced to
agent coordination and now defers the whole error contract to
`specification/errors-and-exit-codes.md`, which is `DIV-001` discharged for
that section. The envelope survives only in the root `README.md`, and this
entry is written against the files as they now stand.

## DIV-043

**Target**: `README.md`, the configuration section. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which made all three corrections. The
`.cfg` example was rewritten and every key in it — `core.database`, and `dsn`,
`host`, `port`, `user`, `database`, `tls`, `ca_file` and `password_command`
under `[database.<name>]` — is a key of `FR-CONF-002`, checked key by key on
2026-09-21. The file states that the file is read strictly and what an
unrecognised key costs, and it states that a DSN carries no query parameters
and that a `?` is refused whatever follows it. Nothing is owed. Re-read on
2026-09-21 at `db80114`.



*Says*: a `.tpl/.cfg` example, and prose describing the file, written at a time
when an unrecognised key had no stated outcome.
*Specification*: `FR-CONF-034` — a key outside the enumerated space of
`FR-CONF-002`, **anywhere** in `.tpl/.cfg`, is `78` (`EX_CONFIG`) with a
nearest-match suggestion. `FR-CONF-011` — a DSN carries no query parameters,
and any `?` is `78`. `BR-CONF-004` gives the reason for both: the file is
untrusted input that decides which host is contacted, which credential is
used, and which child process is executed, so a reader that accepted what it
did not understand would be guessing at those three.
*Correction*: three things follow, and the first is the one that matters.

1. **Check every `.cfg` example in the file against the key space of
   `FR-CONF-002`, key by key.** A key shown in an example and absent from that
   table is no longer undocumented — it is a file `tpl` refuses to read, and a
   reader who copies the example gets `78` on every subsequent invocation. The
   root document is the first place a user copies a `.cfg` from.
2. State that the file is read strictly, so that a user who adds a key of their
   own knows the outcome before they meet it.
3. State that a DSN carries no query parameters. No example in the root
   document shows one today, so this is a gap rather than a contradiction, but
   a DSN copied from another tool commonly carries them and the document is
   where a reader would look for permission.

`DIV-013` corrects the one line of that example this specification refuses
outright — `password_command` as a string — and applies to the same block.

## DIV-044

**Target**: `README.md`, the command-surface listing. **Kind**:
contradiction.

**Status**: discharged by `e75996c`, which removed the command-surface
listing. No passage of the file describes what `tpl schema info` reports, and
the string this entry quotes is not in it. Nothing is owed. Re-read on
2026-09-21 at `db80114`.



*Says*: `tpl schema info` reports "Database metadata: name, version, charset,
collation".

*Specification*: three of the four are confirmed and the fourth does not
exist. `FR-CTX-036` fixes the metadata fields of the `database` object as
`name`, `charset` and `collation`, observed against the schema catalogue on
all four series of `FR-SRV-015`. There is **no `version` field**: the server
version reaches the document as the `server` **object** of `FR-CTX-031`,
carrying three keys — `version`, `series`, and `standing` — of which the last
two have no counterpart in the root document at all. A caller that reads
`data.database.version` finds nothing there, and `FR-SEM-012` fails a render
that reads it.

*Correction*: change the line to name `name`, `charset` and `collation`, and
either drop `version` or replace it with a pointer to the `server` object of
[context-document.md](context-document.md). The listing is the first place a
caller looks for the shape of that document.

## DIV-045

**Target**: `CLAUDE.md`, the release profile. **Kind**: contradiction.

**Status**: discharged in the ninth edition, by amendment rather than by an
edit to `CLAUDE.md`: nothing was ever owed, so no commit could discharge it.
Recorded here for completeness, and with one observation the re-check turned
up: the profile table this entry quotes is itself gone, reduced to a citation
of `ADR-004` by `50153d6`. The *Says* clause below therefore no longer matches
the file, which changes nothing about an entry that owes nothing. Re-read on
2026-09-21 at `8f936d4`, the state of `CLAUDE.md` today; nothing this entry
records has returned to it.



*Says*: the release profile is tuned in `Cargo.toml` with `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`, and `opt-level = 3`.

*Specification*: `FR-ERR-030` — `70` (`EX_SOFTWARE`) is produced by exactly
two conditions, one of which is **a panic in the process**; `FR-ERR-032`
requires that `70` to carry the four labelled lines of `FR-ERR-008`; and
`FR-ERR-001` makes the code table contract. What `panic = "abort"` settles is
how a panic ends the process, not whether the process may report it first. By
**default** it ends it abnormally: the caller receives no code of
`FR-ERR-001` and no message, which removes one of the two producing conditions
of a contractual code in the only artefact a caller ever runs. That default is
not the whole of what the setting admits. A process that installs its own
handler for a panic writes the message and exits with the status it chooses,
before the aborting runtime is reached, and the requirement is then met under
the profile exactly as written.

*Correction*: none is owed to the profile table. The five settings stand as
written, and `FR-ERR-030` as amended in the ninth edition obliges the process
to report a panic and exit `70` without obliging the panic path to be
catchable.

*Why this was recorded rather than resolved, and how it closed.* The eighth
edition reconciled three contradictions **within** this corpus, and this one
was not within it: nothing in `/specification` states a release profile, and
the choice between an unwinding profile and a narrower promise for `70` was an
architecture decision with a cost this corpus cannot weigh. The decision came
back narrower than either option — the profile stands and the requirement
states an outcome rather than a mechanism — and the ninth edition writes the
amendment in.

*Amended in the ninth edition, and this entry is discharged.* The
*Specification* clause rested on a premise that was too strong. It said without
qualification that under `panic = "abort"` "a panic terminates the process
abnormally, the caller receives no code of `FR-ERR-001`, and no message is
written"; that is true of the **default** behaviour of such a profile and not
of a process that handles the panic itself. The clause is corrected and now
separates the two. The *Correction* clause is corrected with it: it required
the profile to leave the panic path catchable, and offered as its alternative
that `FR-ERR-030` drop the caught-panic condition and state the resulting
limit. Neither is what happened. `FR-ERR-030` keeps both producing conditions
and states the first as the outcome a caller observes — the message and the
code — which the profile as written can produce, so the two statements this
entry refused to leave standing together no longer conflict. Nothing is owed to
`CLAUDE.md` under this entry. The identifier is retained rather than removed,
so that a reference written before the ninth edition resolves to this
explanation.

## DIV-046

**Target**: `README.md`, *Installation* and *Development*. **Kind**:
overstatement.

**Status**: **discharged by `d8e7e8a`**, which created the crate on
2026-09-12: `Cargo.toml`, `Cargo.lock` and `src/` are in the working tree,
verified on 2026-09-21, and both sequences this entry records now run. The
passage stands and is no longer false, which is how an **overstatement**
discharges when the repository catches up with the document rather than the
other way about — the rule is at *[How an entry is
read](#how-an-entry-is-read)*.

**The correction this entry asked for must now not be made.** It asked for a
sentence saying the crate does not exist yet, at the head of *Installation* and
of *Development*. Written today that sentence would be false, and the entry
would have turned from a work list into a defect.



*Says*: *Installation* gives `git clone`, `cd tpl`, and `cargo build
--release`, and states that "the binary is produced at `target/release/tpl`".
*Development* gives `cargo build`, `cargo run -- --help` and `cargo test`,
followed by the five-command validation pipeline that opens with `cargo fmt`.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder: `DIV-001`
leaves *Installation* to `README.md` as part of its role as the entry door, so
the section belongs where it is and only its content is false. It is false of
the repository rather than of a requirement — `cargo build --release` fails
for want of a manifest, and every command of *Development* fails with it.
*Correction*: say that the crate does not exist yet, at the head of both
sequences or in place of them. The document-scope banner at the top of the file
does not do it. That banner says no command described below is implemented yet,
which a reader takes to mean the binary builds and the commands are not
finished; it does not say there is nothing to build. *Installation* is the
first instruction in the file a reader acts on, so it is the first place the
file spends its credit, and *Development* is where a contributor goes next.

*Why this entry names two sections.* The survey that raised it named
*Installation* alone. *Development* states the same thing about the same file
under a second heading, and an entry that corrected one would leave a reader
running `cargo fmt --all -- --check` against a directory with no manifest. The
two are one correction and are recorded as one.

*One observation that this entry does not own, and where it now lives.* The
project-structure tree of `CLAUDE.md` also names `Cargo.toml`, `src/`,
`tests/`, `benches/`, `templates/` and `examples/`, none of which the
repository has. `CLAUDE.md` is not this entry's **Target** and nothing is owed
there under it, so the divergence was named here only to keep it from being
lost between the two files. It has since been raised against that file as
`DIV-050`, in the same edition. The two are one divergence written twice, and
neither discharges the other: each is corrected by an edit to its own file,
under its own authorisation, so a reader who corrects both sequences of
`README.md` leaves the tree of `CLAUDE.md` still saying the crate is there.

*The same commands stand in `CLAUDE.md`, under `DIV-051`.* That file's
*Desenvolvimento* section and the validation pipeline under it give the same
three commands and the same five, and they are recorded against that file as an
entry of their own, in the same edition and for the reason written there. The
relation is the one above: neither entry discharges the other, and correcting
*Development* here leaves the other file's copy standing.

## DIV-047

**Target**: `README.md`, the flag table of `render`. **Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the render flag table with
the command reference. No table in the file presents itself as the flag list
of any command, so there is none to omit `--direct` and `--no-cache` from; the
paragraph under the global flag table names both and says which commands
declare them. Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: the flags of `tpl render` are `--table`, `--view`, `--routine`,
`--all-tables`, `--pattern`, `--set`, `--context`, `--output`, `--output-dir`,
`--output-name`, `--no-clobber` and `--dry-run`.
*Specification*: `FR-GLOB-021` — `--direct` and `--no-cache` are declared by
`render`, by the eight `schema` subcommands and by `cache load` — and
`FR-GLOB-022`, which requires each declaring command to list both in its own
help and in its `options` array in the JSON command tree. A table that
presents itself as the flag list of one command and omits two flags that
command declares tells a reader those two would be rejected there under
`FR-CLI-019`. They are declared, so they are accepted; what the omission costs
is a caller who never reaches for them.
*Correction*: add a row for `--direct` and one for `--no-cache`, or replace the
table with a pointer to [global-flags.md](global-flags.md) and
[render-command.md](render-command.md). Seven of the twelve rows are removed
outright by `DIV-004` and `DIV-019`, so the table is being rewritten in any
case.

*Not covered by `DIV-023`.* That entry records the same two flags as missing,
but from the **global** flag tables, and its correction replaces those tables
with a pointer to [global-flags.md](global-flags.md). Applied exactly as
written it leaves this table untouched and still silent: a pointer put where
the global table stood says nothing about a local table further down the file,
and the reader who consults a command's own flag list is the one this entry is
about.

## DIV-048

**Target**: `README.md`, the flag table of `database add` and `update`.
**Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the flag table of `database
add` and `update`. The row is gone, and `--database` appears in the file only
as the global flag of `FR-GLOB-001`, described as the entry an invocation uses
— which is what `FR-GLOB-004` gives it. Nothing is owed. Re-read on 2026-09-21
at `db80114`.



*Says*: `--database`, "Database name on the server", among the flags of
`database add` and `database update`. The group is spelled `tpl database …`
there rather than `tpl cfg database …`, which is `DIV-015`'s subject and not
this entry's.
*Specification*: `FR-CFG-027` names that flag `--schema`, and `FR-CFG-028`
records it as the only flag of the tool whose name differs from the key it
writes — `database.<name>.database` — and gives the reason. The name the table
uses is already taken and means something else: `FR-GLOB-001` declares
`-d/--database` as one of the seven global flags, and `FR-GLOB-004` gives it
the `.cfg` entry label, not the database on the server. `FR-GLOB-002` makes
every global flag acceptable at every node, so the two definitions meet on the
very two commands this table describes, and `FR-CFG-028` states in as many
words that they cannot coexist at one node.
*Correction*: rename the row to `--schema` and keep its description, which is
right. This is not a synonym a reader may pick either way. `tpl cfg database
add shop --database shop` written from this table is accepted, because the
global flag is declared at that node, and `FR-GLOB-004` reads the value as an
entry label — so the invocation silently fails to write the key the table
promised, and writes nothing wrong enough to be refused.

## DIV-049

**Target**: `README.md`, the flag table of `database add` and `update`.
**Kind**: contradiction.

**Status**: discharged by `e75996c`, which removed the same table. There is no
flag list for `add` and `update` to omit `--ca-file` and `--ca-path` from; the
configuration section names `ca_file` and `ca_path` as the keys that supply
trust material to the two verifying modes, and shows `ca_file` in the `.cfg`
example. Nothing is owed. Re-read on 2026-09-21 at `db80114`.



*Says*: the flags of `add` and `update` are `--dsn`, `--host`, `--port`,
`--user`, `--password`, `--password-command`, `--database` and `--tls`.
*Specification*: `FR-CFG-027` declares nine flags for those two commands, and
the two the table has no row for are `--ca-file`, writing
`database.<name>.ca_file`, and `--ca-path`, writing `database.<name>.ca_path`.
The fifth edition added the last three rows of that mapping to close `OQ-017`,
on the ground that registering an entry against a private certificate
authority otherwise took three invocations where the mapping promised one; the
table carries the first of the three and neither of the others. `FR-CFG-047`
adds that none of the three carries a short form.
*Correction*: add a row for `--ca-file` and one for `--ca-path`. The table's
own `--tls` row makes the omission cost more than it looks: that row already
names `verify-ca` and `verify-identity`, and `FR-CONF-014` makes those the two
modes the trust material serves, so the table sends a reader to a mode and
withholds the flags that configure it.

*Not covered by `DIV-003` or `DIV-018`.* `DIV-003`'s correction does name
`--ca-file` and `--ca-path`, among "the other flags of those two commands",
but it names them only to say that none of them carries a short form and asks
for nothing to be added. `DIV-018` asks for `ca_file` and `ca_path` to be
added to the `.cfg` example, which is the configuration file and not this
table; that half of `DIV-018` is still due, and discharging it in full would
leave this entry standing.

## DIV-050

**Target**: `CLAUDE.md`, *Estrutura do Projecto*. **Kind**: overstatement.

**Status**: **partly discharged**, re-read on 2026-09-22 against the working
tree and against `CLAUDE.md` at `9562fb2`. **Five of the six lines are
discharged** by the repository catching up with the document, on the rule at
*[How an entry is read](#how-an-entry-is-read)*: `d8e7e8a` created
`Cargo.toml` and `src/` with all seven of the children this tree names, on
2026-09-12; `4014dc4` created `tests/` on 2026-09-15; `0110f8c` created
`examples/` on 2026-09-22; and `abbfe70` created `benches/` the same day. All
eleven paths are in the working tree, verified on 2026-09-22, and the paragraph
under the tree now speaks of a `model/` that exists. **One line is due**:
`templates/` is still absent, and the tree still names it with a comment saying
what it holds. The correction is narrower than it was and is unchanged in
kind — a qualifier on what is not yet there, and **neither the tree nor the
decomposition under `src/` is to be removed**.

*The `benches/` line, and what makes it true, recorded in the thirty-seventh
edition.* The tree calls that directory *benchmarks*, and `abbfe70` put the
measurement harness there — `run.sh`, `protocol.sh`, `fixture.sh`, `loop200.sh`
and a `README.md` of its own — with `fe428da` taking the first readings from it
and recording them in `BENCHMARKS.md`. The line is therefore true of what is
there and not merely of a path that exists, which is what an **overstatement**
discharges on. `DIV-052` reads the same absence from the other end and is
discharged by the same commit, in the same edition; neither discharged the
other, and each was read against its own passage, which is the ground both
entries give for being recorded apart.

*The entry stays partly discharged, and no count of the Overview moves.* One
line is still owed, and **an entry is never recorded as discharged while any
part of it stands**. A status value is what the Overview counts, and this one
did not change.

*The `examples/` line, and what makes it true.* The tree calls that directory
*pipelines completos: schema → template → output*, and in the working tree of
2026-09-22 it holds the shared driver of `examples/_driver/`, a `README.md` of
its own, and the four worked examples `FR-EX-001` obliges — `go-data-layer`,
`rust-data-layer`, `python-data-layer` and `node-data-layer` — each carrying
its templates, its type-mapping macro, its driver script and the data layer it
renders. The line
is therefore true of what is there and not merely of a path that exists, which
is what an **overstatement** discharges on. Two lines of the same tree were
untouched by this: `templates/` and `benches/` were both absent, and `DIV-052`
read `benches/` from the other end and was neither changed nor read here. *The
verbs of the last sentence are put in the past by the thirty-seventh edition,
which discharged the `benches/` line and `DIV-052` with it; `templates/` is
where the entry now stands alone.*

*Corrected in the thirty-fifth edition.* The *Specification* clause below said
that an agent taking the tree for a map finds nothing at **three** of the
paths, and named `examples/` among them; it names two. That clause is dated and
would otherwise stand as written under the rule at *[How an entry is
read](#how-an-entry-is-read)*, and it is corrected because it is load-bearing
for an entry still owed: it is where this entry states what the reader loses,
and a cost counted over a path that now exists is a cost nobody has to pay.

*Corrected again in the thirty-seventh edition, on the same ground and for the
same clause.* `abbfe70` created `benches/`, so the clause names **one** path
where it named two. The entry is still owed, so the clause is still
load-bearing, and the same reasoning applies: a reader is told exactly which
path the tree promises and the repository does not have, and that is now
`templates/` alone.



*Says*: a fenced tree headed `tpl/`, listing `Cargo.toml`; `src/` with
`main.rs`, `cli/`, `project/`, `mariadb/`, `model/`, `render/` and `error.rs`;
`templates/`, `tests/`, `benches/` and `examples/`, each with a comment naming
what it holds — starter templates, end-to-end CLI integration tests,
benchmarks, complete pipelines; and `BENCHMARKS.md`, `knowledge-model.md`,
`scripts/mariadb/`, `specification/` and `docs/`. Nothing in the block or
around it carries a tense or a marker: it reads as the contents of the
directory, and the paragraph directly under it speaks of `model/` in the
present indicative.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. `DIV-001`
leaves this section where it is — it reduces `CLAUDE.md` to agent coordination
and names implementation conventions among the content it does not touch — so
the section belongs to that file exactly as *Installation* belongs to
`README.md` under `DIV-046`, and only its tense is false. It is false of the
repository rather than of a requirement, and false to the reader that file
declares for itself: an agent that takes the tree for the map of what it may
open finds nothing at one of them — `templates/` — where when this entry was
raised it found nothing at any of the six. `87dd6e3` makes that
reading the natural one, because it shows the tree being maintained against the
repository — three artefacts were added to it because they exist — which tells
a reader the list is kept current.
*Correction*: say at the head of the tree that no code artefact exists yet and
that what follows is the structure the project will have, or mark the lines
that are already there and leave the rest unmarked. **Neither the tree nor the
decomposition under `src/` is to be removed**, and that distinction is the
whole of this entry.

*Planned decomposition and assertion of present state.* The six lines do two
different jobs. `Cargo.toml`, `templates/`, `tests/`, `benches/` and
`examples/` say the project will be an ordinary Rust project, and cost a reader
nothing beyond the paths that are not there. The seven lines under `src/` are a
decision: they are the module decomposition, they tell an implementer where
each responsibility goes before any of it is written, and a document whose
stated job is to say how work is executed here is the right place for them.
That decomposition is an architecture decision, recorded as one outside this
corpus, and this entry takes no position on its content — only on the tense it
is stated in. Removing the six lines would therefore make the document worse in
the name of making it true: the project would lose, from the file every agent
reads first, its only statement of where code belongs, and the next
implementer would invent a structure rather than follow one. What the section
may not do is state a plan in the indicative of the present. A direction and a
claim about the working tree are different things, and the tree is read as the
second because nothing in the section says it is the first. That is why the
correction above is a qualifier and not a deletion.

*There is no banner to argue about here.* `DIV-046` had to explain why
`README.md`'s document-scope banner does not discharge the same defect in that
file: it says no command described below is implemented yet, which a reader
takes to mean the binary builds and the commands are unfinished. `CLAUDE.md`
carries no equivalent. It opens by saying that it is agent coordination and
that it holds no functional requirements, and it says nothing anywhere about
how much of the repository exists. Every statement it makes about the working
tree is therefore taken at face value, and the qualifier this entry asks for
has nothing standing in for it.

*The same divergence as `DIV-046`, and not a widening of it.* That entry
records the build sequences of `README.md` in a repository with no crate; this
one records the tree of `CLAUDE.md` in the same repository. It is one disease
in two files, and the two are recorded apart because **Target** is the
authority on where to look when an entry is re-checked: `DIV-046` was raised by
a reading of `README.md` alone, and widening its target to `both` would have
made it assert of `CLAUDE.md` what nobody had read there. The consequence is
that neither entry discharges the other. Each is corrected by an edit to its
own file under its own authorisation, and the link is written in both
directions so that whoever holds the pen on one file is told the other is still
saying it.

*Not covered by `DIV-032` or `DIV-036`.* Both name the project structure of
`CLAUDE.md` in their **Target**, and both are the nearest thing to an entry
that already reads this section. Neither reaches the tree's six false lines.
`DIV-032` is about the sentence below the tree, which calls the structs of
`model/` the documented public surface; it contradicts a requirement, its
correction rewrites that sentence, and applied exactly as written it leaves the
tree as it stands. `DIV-036` did target the tree, and asked for a line to be
**added** to it — `seed-bench.sql`, which the fixture line then enumerated
around — and `87dd6e3` discharged that half by making the line describe the
directory instead of listing it. So the tree has been read against this corpus
under an entry and corrected under one, and both times the subject was a line
that ought to be there rather than the six that ought not to be stated in the
present. An addition and a tense are different corrections, and neither entry
carries the other.

*One observation that this entry does not own.* *Desenvolvimento* gives `cargo
build`, `cargo run -- --help` and `cargo test`, and the mandatory validation
pipeline under it adds `cargo fmt --all -- --check`, `cargo clippy`, `cargo
build --release`, `cargo test --all-features` and `cargo audit` — eight
commands, every one of which fails in this repository for want of a manifest,
which is what `DIV-046` records of *Development* in the other file. It is not
recorded under this entry: the reading that raised this one was authorised over
the project tree, and an entry that reached past its own reading is the defect
the paragraph above keeps `DIV-046` clear of. `DIV-001` leaves the validation
pipeline with `CLAUDE.md`, so that section belongs where it is too, and as here
only its tense would be in question. It needed an entry of its own, and has
since been raised as `DIV-051`, in the same edition; the reason the two are
recorded apart is written there.

## DIV-051

**Target**: `CLAUDE.md`, *Desenvolvimento* and the *Pipeline de validação
obrigatório* under it. **Kind**: overstatement.

**Status**: **discharged by `d8e7e8a`**, which created the crate on
2026-09-12. Every one of the eight commands runs in this repository, verified
on 2026-09-21, and the sentence between the two blocks gates work on a
pipeline that can now be passed. The passage stands and is no longer false,
which is how an **overstatement** discharges when the repository catches up
with the document — the rule is at *[How an entry is
read](#how-an-entry-is-read)*, and `DIV-046` records the same discharge for
the same eight commands in the other file.

**The correction this entry asked for must now not be made**, for the reason
`DIV-046` states: a sentence saying the crate does not exist yet would be
false.



*Says*: *Desenvolvimento* opens with a fenced block giving `cargo build`,
`cargo run -- --help` and `cargo test`. The subsection under it, *Pipeline de
validação obrigatório*, states that no work is complete until every command
below it passes, in the order given, and gives five:
`cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --release`, `cargo test --all-features` and `cargo audit`.
Neither block carries a tense or a marker, and the sentence between them states
an obligation in the present indicative.
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. `DIV-001`
leaves the validation pipeline with `CLAUDE.md` — it names the pipeline among
the content that file keeps — so the section belongs where it is, exactly as
*Installation* belongs to `README.md` under `DIV-046`, and only its tense is
false. It is false of the repository rather than of a requirement, and false to
the reader that file declares for itself: an agent that runs the pipeline
before reporting a task complete gets five failures that say nothing about the
work it did, and one that follows the section above it gets the other three. The cost is larger here than in a passage a reader merely
consults, because the sentence between the blocks is a gate on work in the
present. Four sprints of work have been closed in this repository, and the gate
as written could not have been passed in any of them.
*Correction*: say at the head of the section that the crate does not exist yet
and that the commands become runnable with it, or mark the two blocks. **The
eight commands are not to be removed, and neither is the obligation stated
between them.** They are the right commands, they are this file's to state
under `DIV-001`, and they run unchanged the moment the manifest exists. What is
wrong is the tense and not the substance — which is the whole of this entry, as
the same distinction is the whole of `DIV-050`.

*Why this is an entry of its own and not a widening of `DIV-050`.* Both entries
are raised against `CLAUDE.md`, both record the same fact about the repository,
and both ask for a qualifier rather than a deletion, so the question is whether
they are one correction the way `DIV-046`'s two sections are. They are not, on
three grounds.

First, the criterion `DIV-046` used is the correction and not the
neighbourhood. Its two sections are about four hundred lines apart in
`README.md` — *Installation* near the head of the file and *Development* near
its foot — so proximity is not what joined them. What joined them is that both
are sequences of build commands, and one sentence written at the head of each
discharges both.

Second, the correction asked for here is not the one `DIV-050` asks for. That
entry qualifies a tree of paths and says in as many words that neither the tree
nor the decomposition under `src/` may be removed, because the decomposition is
an architecture decision the document is entitled to state. This entry
qualifies two blocks of commands and the obligation stated between them. An
editor who applies `DIV-050` exactly as written — a line at the head of the
tree, or a mark on the lines that are already there — leaves the eight commands
where they are, under a heading and a subheading that entry does not name, and
leaves a rule that gates the completion of work on commands that cannot run.

Third, the two passages are edited apart and must therefore be re-checked
apart. **Target** is the authority on where to look when an entry is
re-checked, and `87dd6e3` is the demonstration: it rewrote the tree, adding
three artefacts because they exist, and did not touch *Desenvolvimento*. An
entry whose target named both sections would have been re-read against that
commit and recorded as partly discharged on the strength of an edit that did
nothing for the commands.

What a merge would buy is that whoever holds the pen on `CLAUDE.md` sees one
item instead of two, and both corrections are likely to be made in one commit.
That is not enough to merge them. The index groups by **Target**, so a reader
working through what `CLAUDE.md` owes sees both entries together and loses
nothing; while an entry carrying two corrections of different shapes under one
status is what the *partly discharged* machinery exists to describe, and
describing as one thing what is two makes the register a worse work list rather
than a shorter one.

The reading that raised `DIV-050` was authorised over the project tree and
named this passage without recording it, which left the placement open rather
than settled. It is settled here, by a reading of the section itself and on the
three grounds above.

*The same eight commands as `DIV-046` records of `README.md`.* *Development* in
that file gives `cargo build`, `cargo run -- --help` and `cargo test`, and then
the same five-command pipeline under the sentence *Before any change is
considered complete*. The two passages are one list written twice, once in each
root document, under headings that are each other's translation. This is the
relation `DIV-046` and `DIV-050` already carry, and it holds the same way here:
neither entry discharges the other, because each is corrected by an edit to its
own file under its own authorisation, so a reader who qualifies *Development*
leaves `CLAUDE.md` still telling an agent that no work is complete until eight
commands pass that cannot run. The link is written in both directions.

*Three passages elsewhere in the file cite the pipeline and need nothing.* The
rules table at the head of `CLAUDE.md` makes it a rule that the mandatory
pipeline passes before work is complete; the platforms section requires what
passes it to pass on every supported target; and the code-conventions section
names `rustfmt` and `clippy` as the arbiters that run in it. Each states an
obligation and names the section that holds the commands, and none of them says
anything about the working tree, so the qualifier this entry asks for at the
head of that section reaches them where they stand. It is the reading `DIV-050`
gives the lines of the tree it leaves alone: a direction and a claim about the
working tree are different things, and only the second is false.

*One observation that this entry does not own.* The *Disciplina de medição*
subsection states that the benchmarks live in `benches/` and run against the
dataset of the MariaDB containers, so as to be reproducible. `benches/` is one
of the three paths `DIV-050` still records as absent, and there are no
benchmarks to live in it; the sentence is the same class of claim as the two
blocks above, in the present indicative with nothing around it to carry a
tense. It is not
recorded under this entry: the reading that raised this one was authorised over
the eight commands, and reaching past its own reading is the defect `DIV-046`
and `DIV-050` each keep themselves clear of. No entry covers it. `DIV-035` was
the performance budget table and is discharged; `DIV-036` targets the project
tree and the MariaDB testing section, and what it still owes is the sentence
that ought to name `seed-bench.sql`. It needs an entry of
its own, and has since been raised as `DIV-052`, in the same edition; the three
grounds on which it is an entry apart from this one, rather than a widening of
it, are written there.

## DIV-052

**Target**: `CLAUDE.md`, the *Disciplina de medição* subsection of *Desempenho
e Eficiência*. **Kind**: overstatement.

**Status**: **discharged** by `abbfe70`, re-read on 2026-09-22 against the
working tree and against `CLAUDE.md` at `9562fb2`. **The repository caught up
with the sentence**, which is the second sense a *Discharged* value takes under
*[How an entry is read](#how-an-entry-is-read)*: `abbfe70` created `benches/`
and put the measurement harness in it — `run.sh`, `protocol.sh`, `fixture.sh`,
`loop200.sh` and a `README.md` of its own — and that harness runs against the
MariaDB containers, which it stands up through `scripts/mariadb/` and loads with
`seed-bench.sql`. `fe428da` then took the first readings from it, so the
sentence is true twice over: benchmarks live there, and they have run against
that dataset and produced a record. **The commit named is `abbfe70`, the one
that made the statement true.**

**The clause this entry was raised against survives the edit to that
subsection.** `9562fb2` rewrote the sentence's second half — `DIV-055`'s
business, discharged there — and left *Os benchmarks vivem em `benches/` e
correm contra o dataset dos containers MariaDB, para serem reproduzíveis*
standing. That is this entry's clause, unchanged and now true, so the discharge
is of the passage this entry actually records and not of a sentence somebody
else rewrote.

*Corrected in the thirty-third edition.* The last sentence read *one of the
seven corrections this register still owes*, where the Overview counts nine
entries owing, six to `CLAUDE.md` and three to `README.md`. The figure is now
the Overview's, stated in the Overview's own terms so that a reader comparing
the two is comparing one count.

*Corrected again in the thirty-sixth edition, which moved the Overview.*
`DIV-055` takes the counts to ten owing and seven owed to `CLAUDE.md`, and this
sentence follows them. The same edition found the thirty-third's correction had
been made here and nowhere else, and corrected `DIV-032` and `DIV-041` with
it.

*Both corrections are spent, in the thirty-seventh edition.* This entry owes
nothing, so it is in no count of entries owing, and the sentence the two notes
above were correcting is gone with the **due** status. They are kept because
they are the record of a figure restated in four places and corrected in one,
which is the lesson the Overview's counts are maintained under.



*Says*: the subsection opens with the rule that there is no performance claim
without numbers and no optimisation without a measurement before and after,
gives a table assigning a tool to each kind of measurement, and then states:
"Os benchmarks vivem em `benches/` e correm contra o dataset dos containers
MariaDB, para serem reproduzíveis. As baselines são registadas em
`BENCHMARKS.md`, identificando o alvo em que foram medidas; **uma regressão
face à baseline reprova a alteração** e tem de ser justificada ou corrigida
antes de o trabalho ser dado por concluído." Nothing in the subsection carries
a tense or a marker.
*Specification*: none **for what this entry records**, and that is what the
third kind means. Nothing is owed to this folder for the claim this entry was
raised over. Of what the subsection states, `NFR-PERF-009`, `NFR-PERF-010` and
`NFR-PERF-012` fix the measurement protocol and `NFR-PERF-012` obliges a
measurement to name the target it was taken on, all three in force. Where a
benchmark lives is no requirement of this corpus — it names no directory — and
`DIV-035` left the discipline with `CLAUDE.md` when it took the figures out of
that file, so the subsection belongs where it is, exactly as *Installation*
belongs to `README.md` under `DIV-046`. What this entry finds false is the
tense of one clause, and it is false of the repository rather than of a
requirement: an agent that reads the sentence for what to re-run before
claiming a speed-up is sent to a directory that is not there, and one that
reads it for what produced the baselines in `BENCHMARKS.md` is told something
that file denies of itself.
*Correction*: say at the head of the subsection that no benchmark exists yet
and that the sentence says where they will live, or mark that one sentence.
**Neither the discipline nor the location is removed by this entry.** No claim
without numbers, and the figure recorded in `BENCHMARKS.md` against a named
target, are obligations in force — they governed both campaigns
`BENCHMARKS.md` already holds, and each of those entries says where it departs
from the normative protocol — and `benches/` is where a benchmark should go the
day one is written. What this entry finds wrong is the assertion that
benchmarks are there. **The subsection's last clause is wrong for a second and
unrelated reason, which `DIV-055` records**, and the two corrections are made
together or in either order.

*Amended in the thirty-sixth edition, in the two paragraphs above and nowhere
else.* Three sentences had stopped being true. The *Specification* paragraph
said flatly that **no requirement of this corpus is contradicted** and that
what the subsection states **is in force**, and cited `NFR-PERF-017` for the
clause that a regression against the baseline reproves the change. That
requirement is withdrawn and `BR-PERF-008` contradicts the clause, so the
paragraph asserted the opposite of what this corpus now holds. The *Correction*
paragraph carried the same claim from the other side, listing a regression
failing the change among the obligations *in force today*, and it was the
sentence an editor would have read as a reason to keep the clause. `DIV-055` is
raised for the contradiction, against this same sentence and against a second
passage of the file, because the correction differs: this entry asks for a
tense to be qualified and that one asks for a clause to go.

**The Kind is left at overstatement and the question is named, not settled.**
This passage now carries one defect of each kind, and whether an entry whose
target sentence is contradicted keeps a kind describing a different defect in
it is a question about how this register classifies, not about what either
correction is. It is left for the next reading of this file, on the rule the
sixteenth and seventeenth editions used for a candidate a sweep could not
judge. Nothing turns on it while it stands: both defects are recorded, each
with its own correction, and `DIV-055` names this entry as `DIV-052` names it.

**Settled in the thirty-seventh edition, which is that next reading: the Kind
stays `overstatement`, and it stays because a Kind classifies the defect the
entry records and never the passage the entry points at.** That is what
*[How an entry is read](#how-an-entry-is-read)* already implies of every clause
of an entry — **Target** names where to look, **Kind** names what is wrong,
**Status** names what is still owed — and it is the same criterion `DIV-051`
settled for where an entry goes: the shape of the correction decides, not the
proximity of the passages. One sentence carrying two defects is two entries with
two kinds, and it always was.

**What settles it is not the argument but what happened to the two entries.**
They discharged separately, by different commits, through events of different
sorts.
This one discharged when the repository caught up with the text, at `abbfe70`,
with nobody editing `CLAUDE.md` — which is what an **overstatement** discharges
on, and only an overstatement. `DIV-055` discharged when somebody deleted a
clause, at `9562fb2` — which is what a **contradiction** discharges on. A single
entry carrying one Kind could not have described both discharge conditions, and
whoever made either edit would have been left holding an entry that was half
paid. The separation was right, and the Kind belongs to the entry that records
the defect.

*Rejected: moving this entry's Kind to `contradiction` because its sentence was
contradicted.* It would have made the Kind a property of the passage, and a
passage can carry any number of defects while an entry records one. Under that
reading this entry would have become a contradiction it never asserted, would
have been read as discharged by `9562fb2`, which did not touch its clause, and
the false tense it was raised over would have gone unrecorded until somebody
raised it again. *Also rejected: merging the two entries once both were
discharged*, on the ground that the distinction stopped mattering. A discharged
entry is kept in place so that a reference written before the discharge resolves
to an explanation, and merging two would leave one of the two identifiers
resolving to nothing.

*Why this is an entry of its own and not a widening of `DIV-051`.* The
criterion is the one `DIV-051` settled: **the shape of the correction decides
where an entry goes, and not the proximity of the passages.** It reached that
by reading `DIV-046`, whose two sections lie about four hundred lines apart in
`README.md`, and finding that what joined them was not neighbourhood but that
one sentence written at the head of each discharges both. This entry is the
same criterion applied against the grain. *Disciplina de medição* is close to
the two blocks `DIV-051` records — a subsection of the very next top-level
section, some fifty lines below the head of *Desenvolvimento* — so proximity
argues for a merge here as loudly as it argued against one there, and it is not
what decides.

What decides is that no one correction serves both. A qualifier at the head of
*Desenvolvimento*, which is what `DIV-051` asks for, does not reach a
subsection of *Desempenho e Eficiência*, and an editor who writes it leaves a
reader in the next section still told that benchmarks live in a directory
nothing has said is absent. The two corrections also discharge on different
conditions. `DIV-051`'s eight commands run unchanged the moment the manifest
exists, and that entry says so in as many words; this sentence does not become
true with a manifest. A benchmark has to be written. One entry carrying both
would be settled in one half by an event that leaves the other waiting.

*Amended in the thirty-third edition, because half of what this entry waited on
has arrived.* The passage continued: *and what it is said to run against is not
complete: `WL-001` is realised by `scripts/mariadb/seed-bench.sql`, which does
not exist, and `BR-PERF-007` in
[performance-requirements.md](performance-requirements.md) records that the
budgets over it cannot be measured until it does*. The file is in the working
tree, `DIV-036` no longer owes it, and `BR-PERF-007` is restated over a fixture
that is complete. The ground this clause serves is untouched, and it is why the
clause is narrowed rather than dropped: `DIV-051`'s correction still does not
reach this subsection, and this sentence still becomes true only when a
benchmark is written.

Third, the two passages are edited apart and must be re-checked apart, which is
the ground `DIV-051` gives against `DIV-050` and which holds here on the same
demonstration: `87dd6e3` rewrote the project tree, adding three artefacts to it
because they exist, and touched neither *Desenvolvimento* nor this subsection.
**Target** is the authority on where to look when an entry is re-checked, and
an entry naming two sections that move independently records a discharge it has
not had.

*Where the entry sits in this file.* It takes the next identifier and stands at
the foot, in identifier order, rather than beside the entries it resembles. An
identifier is stable once assigned and is cited from other entries, so the
order of this file is the order the entries were raised in; the Index is what
groups them, and a reader working through what `CLAUDE.md` owes reads it by
**Target**. That is the reason `DIV-051` gave for declining to merge into
`DIV-050`, and it applies to position as it applies to merging: the index
already puts the four entries against that file in one place, so nothing is
bought by moving this one next to them.

*The entry `DIV-051` said this passage needed.* That entry records the eight
`cargo` commands of this same file and closes by naming this sentence as an
observation it does not own, because the reading that raised it was authorised
over the commands, and reaching past its own reading is the defect `DIV-046`
and `DIV-050` each keep clear of. It also did the checking this entry rests
on — that no entry covers the sentence, that `DIV-035` is discharged, and that
`DIV-036` targets other sections — and left the entry to be raised. This is
that entry, and the link is written in both directions. Neither discharges the
other, for the three grounds above.

*The relation to `DIV-036`, which was raised over the fixture this sentence
invokes.* `DIV-036` is the nearest entry by subject. It is partly discharged,
and what it still owes is one sentence of the MariaDB testing section, which
names two of the fixture's three SQL scripts. The third is
`scripts/mariadb/seed-bench.sql`, which is what `WL-001` is realised by and
which `BR-PERF-002` keeps separate from `seed.sql` on purpose, so the dataset
this sentence promises reproducibility against is the one that entry describes
incompletely. The two do not cover each other in either direction. `DIV-036` is
a **migration** whose correction names a file in a paragraph, and made exactly
as written it leaves this sentence saying that benchmarks live in `benches/`;
this entry's correction is a qualifier on a tense and produces no benchmark.
What the relation does establish is that this entry is not discharged by a
manifest alone: the sentence becomes true when benchmarks exist.

*Amended in the thirty-third edition, with the Status above.* The clause said
`DIV-036` still owes the file itself, called the correction an **addition** of
*a file, and the two lines that name it*, counted *an addition, a tense and a
missing file* as three corrections, and closed by saying a benchmark over
`WL-001` becomes possible when `DIV-036` is paid. The file is in the working
tree, so none of that holds: `DIV-036` owes a sentence and not a file, and a
benchmark over `WL-001` is possible today. What the clause was written to
establish — that neither entry discharges the other — is unchanged, which is
why it is corrected rather than removed.

*Not covered by `DIV-035` or `DIV-050` either.* `DIV-035` is the entry that
last read this part of the file: it moved the four-row budget table out of
`CLAUDE.md` and is discharged by `0ea5624`, which replaced the table with a
pointer to [performance-requirements.md](performance-requirements.md). Its
subject was the figures, its correction was made, and a discharged entry owes
nothing; the discipline it deliberately left behind is what this entry finds a
false tense in front of. `DIV-050` records `benches/` as one of the three
lines of the project tree the repository still does not have, which is the
same absence seen from the other end, but its correction is a qualifier on the
tree or a mark on the lines already there, and applied exactly as written it
leaves a
subsection a hundred lines below asserting that the directory holds something.
An absent path and a claim about what it holds are two statements, and marking
the first does not correct the second.

*What the sweep of this file found, and the one thing it does not settle.*
This entry's passage was given to it; the rest of `CLAUDE.md` was read to
answer whether any passage of this kind is left, because a claim that a class
is exhausted is worth only the sweep behind it. Every other path, file and
artefact that file names exists: the twenty-four files of this folder its
subject table points to and the `README.md` here it starts from,
`BENCHMARKS.md`, `knowledge-model.md`, `docs/adr/` with the decisions its Stack
table cites, and `scripts/mariadb/` with the fixture, the TLS material and the
harness its testing section describes. The
passages that state `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, the
release profile, the module conventions and the separation of library from
binary assert nothing about the working tree: each directs how code is to be
written, and a direction and a claim about the working tree are different
things, which is the reading `DIV-050` gives the lines of the tree it leaves
alone. The tool table directly above this entry's sentence is of the same sort,
and so is the overview, which says what `tpl` is rather than what the
repository holds.

**One candidate stands, and this entry does not record it.** *Plataformas
Suportadas* says "Os sistemas suportados e verificados são o **Linux** e o
**macOS**", and says of the other Unixes that they "não entram na matriz, não
correm em validação", which presupposes that the four targets do. Nothing has
been verified on any target, because the validation that would do it is the
pipeline `DIV-051` records as unable to run at all. It is one adjective and one
clause inside a declaration of policy, not a block a reader executes, and
whether *suportados e verificados* asserts a state or names a class is a
judgement for a reading of that section — which `DIV-041` targets on other
grounds, for its deferral of the target matrix, and does not reach. It is named
here and not recorded, on the rule the three entries before this one each kept.
**So the class is not closed in this file by this entry**: four of its passages
are recorded, and one candidate is left for a reading of its own.

*That reading was made in the seventeenth edition* and is recorded at
*[A candidate read, and not recorded](#a-candidate-read-and-not-recorded)*.
It finds the candidate declares a policy rather than asserting a state, so no
entry is owed for it, and it closes the class in this file by a sweep of its
own rather than by this entry.

## DIV-053

**Target**: `README.md`, *Discovery*. **Kind**: contradiction.

**Status**: **due**, raised on 2026-09-21 at `db80114`. The sentence stands,
unchanged since `e75996c`.



*Says*: "Four entries of the tree need no project and perform no discovery at
all: `tpl init`; `tpl help` in its three forms; `-h/--help` at any node; and
`tpl version` with `-V/--version`. Each runs where no project exists, reads no
file under `.tpl` and opens no socket."
*Specification*: `FR-PROJ-025`, as amended in the thirty-first edition. The
four commands are the right four and the clause over them is no longer right:
**`tpl init` does look upward, and a requirement obliges it to.**
`FR-PROJ-016` requires it to warn that the project it is about to create
shadows one in an ancestor directory, and that warning cannot be written
without looking for the ancestor. What `FR-PROJ-025` forbids is that a project
above the invocation decide the outcome — none is required, none supplies
configuration, none selects an entry, and none decides where `tpl init`
creates what it creates — and it says so in its own text. *No discovery at
all* is the reading that requirement stopped having. `NFR-PERF-005` states the
file-open observable per command for the same reason.
*Correction*: qualify the clause. Say that the four need no project and that
no project above the invocation decides what any of them does, and either name
the one upward look `tpl init` makes and the single line on stderr it
produces, or drop *at all* and leave the detail to `FR-PROJ-025`. The rest of
the sentence is correct and stays: each of the four runs where no project
exists, and none of them opens a socket.

*Raised by an amendment to this corpus, and not by an edit to the target — and
that is what makes it worth recording twice over.* The fifth validation rule
was written for a register that decays because somebody else edits the file it
describes. This entry decays from the other end. The sentence in `README.md`
has not moved since `e75996c` and was an exact summary of `FR-PROJ-025` when it
was written; what moved is the requirement, amended four days later by the
thirty-first edition of this corpus, which found that `FR-PROJ-016` obliges the
very walk `FR-PROJ-025` had forbidden without qualification. **A register of
corrections owed to a file this specification does not own decays whenever
either side changes**, and an edition that amends a requirement a root document
paraphrases owes this register a look at the paraphrase. The rule is extended
to say so, in [README.md](README.md#maintenance-debt).

## DIV-054

**Target**: `README.md`, *Keeping a password off disk* and *Development*.
**Kind**: overstatement.

**Status**: **due**, raised on 2026-09-21 at `db80114`. Both clauses stand.
`db7337d` made both of them false and left both; `db80114` rewrote the status
banner that contradicts them and left both again.



*Says*: under *Keeping a password off disk*, of `${VAR}` and
`password_command`, "Both take effect when the configuration is **resolved for
a connection**, which no written command does yet: today they are stored,
printed and validated, and not performed." Under *Development*, of the
containers of `scripts/mariadb/`, "No test needs one today, because no command
that reaches a server is written."
*Specification*: none, and that is what the third kind records. No requirement
of this corpus is contradicted and nothing is owed to this folder. Both clauses
are false of the repository, verified on 2026-09-21. `db7337d` wired the eight
`tpl schema` subcommands and `tpl cache load` to open a connection to the
server the selected entry names, and `db80114` wired `tpl render`; the
configuration is resolved for a connection on every one of those paths, and
`src/project/password.rs` spawns the `password_command` child and reads its
output. Tests that need a server exist and drive the fixture through its own
harness — `tests/schema_and_cache.rs` and `tests/render_command.rs`, by way of
`tests/support/fixture.rs`. **The status banner at the head of the same file
says so**, listing every one of those commands as working, so the file
contradicts itself across two hundred and fifty lines.
*Correction*: delete the clause in each. Under *Keeping a password off disk*,
the sentence reads correctly as "Both take effect when the configuration is
resolved for a connection." Under *Development*, the paragraph reads correctly
without its last sentence, since everything before it describes a fixture that
exists and is used. Nothing else in either passage is wrong.

*Why one entry for two sections, and the criterion that decides it.*
`DIV-051` settled that the **shape of the correction** decides where an entry
goes and not the proximity of the passages, and `DIV-046` is the case where two
sections some four hundred lines apart are one entry because one reading
discharges both. This is that case and not `DIV-051`'s. The two clauses are one
claim — that no command of `tpl` reaches a server — written once in each
section, and they became false at the same instant and will become true again
at none: whoever establishes the fact for one has established it for the other,
and the deletion is the same deletion made twice. `DIV-051`'s third ground,
that passages edited apart must be re-checked apart, is what argues against;
it is outweighed here because what discharges this entry is not an edit to
either passage but a fact about the repository, and a fact does not arrive at
one section before the other.

*The kind is widened to hold it.* The **overstatement** kind was written as
*states as present something the repository does not contain*, and this entry
is the opposite direction: the file states as absent a capability the
repository has. The test the kind states of itself is unchanged and this entry
meets it — nothing owed here, no requirement contradicted, and a reader who
acts on the passage fails, which the *Development* clause does squarely, since
a contributor who acts on it skips the fixture and watches a server-dependent
test fail. The widening, and the fourth kind rejected in its place, are in the
[Overview](#overview).

## DIV-055

**Target**: `CLAUDE.md`, the *Pipeline de validação obrigatório* subsection of
*Desenvolvimento*, and the closing sentence of *Disciplina de medição* in
*Desempenho e Eficiência*. **Kind**: contradiction.

**Status**: **discharged** by `9562fb2`, re-read on 2026-09-22 against
`CLAUDE.md` at that commit. **Both passages are corrected, each in the way this
entry asked for.** Under *Pipeline de validação obrigatório*, the sentence that
added the benchmarks and a comparison against the baseline to the pipeline is
replaced by one saying that nothing of performance is added to it — that a
change to a hot path is an occasion to take a reading and record it, and never a
condition for the work being done — which is the second of the two corrections
offered below. Under *Disciplina de medição*, the clause **uma regressão face à
baseline reprova a alteração** and the *justificada ou corrigida* that hung on
it are gone; what the sentence now states is where readings are recorded, that
each names its target, and that figures from different targets are never
compared, which is `NFR-PERF-012` and is true. Nothing this entry records is
owed any longer.

*What the same commit added, which this entry did not ask for and does not
object to.* The section now states in its own words that no figure reproves a
change, citing `BR-PERF-008`, and that what does reprove are the requirements of
form. Both are this corpus's own statements, correctly attributed, and neither
creates a second source: the file cites the requirement rather than restating
any figure, which is what `DIV-035` asked of it and got.

*Raised and discharged within one sprint, which is the fifth validation rule
working as written.* This entry became owed on 2026-09-22 when `a7332fc`
withdrew `NFR-PERF-017`, and it was paid the same day by an edit to the file it
was owed to. The trigger for raising it was an amendment inside this corpus, the
direction the thirty-second edition's extension covers; the trigger for
discharging it was an edit to the target, the direction the rule was written
for. Both halves of the rule are exercised by one entry.



*Says*: under *Pipeline de validação obrigatório*, after the five commands,
"Quando a alteração toque num caminho quente — leitura de catálogo, construção
do contexto, render, arranque do processo — acresce a este pipeline a execução
dos benchmarks e a comparação com a baseline em `BENCHMARKS.md`, conforme a
**Disciplina de medição**." Under *Disciplina de medição*, closing the
subsection, "As baselines são registadas em `BENCHMARKS.md`, identificando o
alvo em que foram medidas; **uma regressão face à baseline reprova a
alteração** e tem de ser justificada ou corrigida antes de o trabalho ser dado
por concluído."
*Specification*: [performance-requirements.md](performance-requirements.md) —
`BR-PERF-008`, under which no figure named in this corpus and no figure
recorded against it in `BENCHMARKS.md` fails, blocks, rejects or gates a
change. `NFR-PERF-017`, which both passages summarise, is withdrawn, as are
`NFR-PERF-015` and `NFR-PERF-016` with it. The first passage adds a comparison
against a baseline to the pipeline that decides whether work is finished, which
is a gate on a figure; the second states the gate itself. **What is not
contradicted is the rest of either passage**: the five commands of the
mandatory pipeline are correctness checks and are untouched, no claim without
numbers stands, a figure recorded against a named target stands per
`NFR-PERF-012`, and the ambition of *Desempenho e Eficiência* — that a correct
but slow or wasteful implementation does not satisfy the requirement — is
exactly what `BR-PERF-008` keeps, as design and architecture.
*Correction*: in the first passage, remove the sentence, or reduce it to
saying that a change to a hot path is an occasion to take a reading and record
it. In the second, remove the clause **uma regressão face à baseline reprova a
alteração** and what hangs on it — *e tem de ser justificada ou corrigida antes
de o trabalho ser dado por concluído* — leaving the sentence stating where
baselines are recorded and that each names its target, which is true and is
`NFR-PERF-012`. Nothing else in either subsection is wrong on this ground.

*Why this is an entry of its own and not a widening of `DIV-052`.* The
criterion is the one `DIV-051` settled and `DIV-054` applied: **the shape of
the correction decides where an entry goes**, not the proximity of the
passages. `DIV-052` is raised against the same sentence as this entry's second
passage, and the two corrections are different operations with different
discharge conditions. That entry asks for a **tense** to be qualified, and it
discharges when a benchmark exists to live in the directory the sentence names;
this entry asks for a **clause** to be deleted, and it discharges only when
somebody deletes it. An editor who makes either is left with the other. They
also have different reach: this entry's first passage is in another top-level
section, which `DIV-052` does not target at all, and **Target** is the
authority on where to look when an entry is re-checked.

*Why one entry for two sections.* This is `DIV-054`'s case and not
`DIV-051`'s. The two passages state one claim — that a regression against a
recorded baseline fails the change — written once in each section, and they
stopped being true at the same instant and for one reason. Whoever establishes
the fact for one has established it for the other, and the deletion is the same
deletion made twice.

*What this entry does not ask for, stated because a reader of `DIV-052` was
told the opposite.* That entry says in as many words that neither the
discipline nor the location is to be removed, and lists a regression failing
the change among the obligations in force. That was true when it was written
and is not now; the sentence is corrected there, and this entry is what
replaces it. The discipline this entry leaves alone is the two halves of it
that survive: no claim without numbers, and a figure recorded against a named
target.

## DIV-056

**Target**: `README.md`, *Configuration*, the first paragraph of *The key
space is closed*. **Kind**: contradiction.

**Status**: **due**, raised on 2026-09-23 at `d89ffc4`, and re-read the same
day against the working tree. At `d89ffc4` the sentence counted fifteen keys,
five under `[core]`. An uncommitted edit to the working tree changed it to
seventeen and seven, which matched `FR-CONF-002` before the third render key
was added and matches it no longer. No commit has
discharged any part of it.



*Says*, in the working tree: "`.tpl/.cfg` admits exactly seventeen keys —
seven under `[core]` and ten per `[database.<name>]` block — each with a
declared type and a declared default."
*Specification*: `FR-CONF-002`, as amended in the forty-second edition, which
adds `core.render_fuel`, `core.render_output_limit` and
`core.render_memory_limit` for the render bounds of `FR-RND-036`, `FR-RND-037`
and `FR-RND-039`. The key space is now eighteen key forms, eight under
`[core]` and ten per entry.
*Correction*: change the two counts to eighteen and eight, or drop the counts
and keep the pointer to `FR-CONF-002`, which the same sentence already gives
and which cannot fall behind the table. The second is the one that stops this
entry recurring. The rest of the paragraph is correct and stays.

*Raised by an amendment to this corpus, and not by an edit to the target*, the
direction `DIV-053` first recorded. The sentence was an exact summary of
`FR-CONF-002` when it was written. Its paragraph on deadlines is unaffected:
the four `[core]` keys it names still supply every deadline, and the three
new keys set bounds that are not deadlines.

## A candidate read, and not recorded

The sixteenth edition's sweep of `CLAUDE.md` named one passage and did not
record it, on the ground that the judgement it needed was a reading of the
section it sits in. This is that reading, made in the seventeenth edition.
**It finds that the passage declares an intention rather than asserting a
state, so there is no entry to open.** The finding is written here, beside the
entries, because a conclusion that nothing is owed leaves no trace otherwise
and the question is re-opened from scratch by the next reader — which is the
failure the fifth validation rule of [README.md](README.md#maintenance-debt)
was written against, seen from its other end.

*The passage.* *Plataformas Suportadas* states that "Os sistemas suportados e
**verificados** são o **Linux** e o **macOS**, nas arquitecturas **arm64** e
**amd64**", and says of the other Unixes that they "devem funcionar ... mas
**não são testados nem garantidos**: não entram na matriz, não correm em
validação, e um problema que só neles se manifeste não reprova uma alteração."
The second clause presupposes that the four targets do enter the matrix and do
run in validation. **Nothing has been run on any target.** The validation that
would do it is the five-command pipeline `DIV-051` records as unable to run at
all for want of a manifest, and `docs/adr/adr-008-packaging-and-build-path.md`
records separately that no `musl` artefact has been run outside a container and
marks that expectation **unverified**.

*The judgement rests on four grounds.*

**First, the section says two paragraphs later that the matrix is not
decided.** "A matriz concreta de alvos — target triples, escolha de libc,
linkagem e forma de empacotar o binário — é decisão de arquitectura em curso e
**não se fixa aqui**." A verification is of an artefact on a target, and a
target is a triple with a libc and a linkage. A section that declines to name
the triples cannot, earlier in itself, be reporting that artefacts were
verified on them. The state reading makes the section contradict itself; the
policy reading makes it coherent.

**Second, the presupposition sits inside a rule, and the rule's payload is its
third limb.** "Não entram na matriz, não correm em validação, e um problema que
só neles se manifeste não reprova uma alteração" is one thought in three parts,
and the third states what fails a change. A statement of what gates a change is
deontic throughout; the two limbs before it state the policy that yields the
third, not a history of runs.

**Third, the section says in its first sentence what it is for**, and it is not
reporting: "É essa a família de sistemas para que se escreve, e é a fronteira
que delimita o que o código pode assumir." Every other sentence does that job —
Windows is out of scope, the BSDs are not guaranteed, the triples are
deferred — and the subsection under it is six rules in the deontic. So does the
adjective: it marks the four as the systems that carry the project's
undertaking of verification, against the Unixes that carry none, which is the
contrast the sentence is built on. Read as a report it would be a report naming
no target, no date, no artefact and no result, which is not a report.

**Fourth, this corpus states verification in the same mode, in the requirement
the passage would be measured against.** `NFR-PERF-005` in
[performance-requirements.md](performance-requirements.md) says of a clause
of itself that it "SHALL be verified on every target of `NFR-PERF-018`", and
the eleventh-edition note
under `NFR-PERF-018` states that "every requirement of this file is still
verified on all four targets" — written when nothing had been run, and true
today for the same reason, because what it states is the reach of the
verification obligation and not a history of runs. This corpus cannot record as
an overstatement in a document it does not own a mode of statement it uses
itself, in force, in the requirement that document's sentence would be checked
against.

*Why the decision record does not carry the other way.* `ADR-008` marks the
expectation unverified, and if `CLAUDE.md` asserted a verification the two
would contradict each other on a matter of fact. That record does not read the
file that way. It calls the five-command sequence one that the root
coordination document **mandates**, and lists the no-second-class rule of
`NFR-PERF-018` among four obligations **carried by hand** until a pipeline
exists — the deontic reading, taken by the record best placed to take it. What
it marks unverified is also narrower than the passage: that no `musl` artefact
has run outside a container, and that the two `x86_64` targets have never been
measured. Both are claims about baselines under `NFR-PERF-012`, and the
sentence of `CLAUDE.md` that carries that rule is in another section, where
`DIV-041` has already read it and found it correct.

*What this kind is, and why the passage is not of it.* The **Overstatement**
kind states its own test in the Overview: nothing is owed to this folder, no
requirement is contradicted, and what is wrong is that **a reader who acts on
the passage fails**. The four entries that carry the kind each name something
to act on — a build sequence that fails, a tree of paths that open onto
nothing, eight commands that cannot run, a directory that is not there. This
passage names no path, no command and no artefact. There is nothing in it to
act on, and what a state reading of it would produce is a false belief rather
than a failed action. Manufacturing an entry for it would put a **Status** of
*due* on a correction nobody can make and weaken the four that stand.

*The relation to `DIV-041`, which targets the same section.* That entry is a
**migration** and its subject is the deferral of the target matrix: its *Says*
quotes the deferral sentence and the two rules beside it, its correction
replaces the deferral with a pointer to `NFR-PERF-018`, and nothing in it turns
on whether verification has happened. It does not reach the adjective, and the
adjective does not reach it. **What this reading owes `DIV-041` is one
caution.** The correction that entry asks for fixes the matrix in the section,
which removes the first of the four grounds above; whoever makes it re-reads
the adjective in the section it leaves behind, because a sentence calling four
named triples *verificados* is a stronger claim than the one read here, and it
would be read against a repository in which the pipeline may by then run on
some targets and not others.

*One observation this reading does not own.* The sentence naming Linux and
macOS on two architectures states, coarsely, what `NFR-PERF-018` now fixes
exactly, and `DIV-041`'s correction names the deferral and the two rules beside
it rather than that sentence. Whether a coarse statement of the supported
systems left standing beside a pointer to `NFR-PERF-018` is two sources for one
truth is a question about that entry's **Target** and **Kind**, not about this
one: this reading was authorised over the adjective and over the sweep below,
and reaching past its own reading is the defect the four entries of the kind
each keep clear of. It is named here so that it is not lost.

*The class is exhausted in `CLAUDE.md`, and this is the sweep behind the
claim.* The whole file was read again against the question rather than the
sixteenth edition's answer taken on trust, for the reason that edition gave:
a claim that a class is exhausted is worth only the sweep behind it, and the
sixteenth edition's own sweep is what found the candidate the fifteenth had
missed. Every path, file and artefact `CLAUDE.md` names was tested against the
working tree. All exist but the ones already recorded: the twenty-four files of
this folder its subject table points to, the `README.md` of this folder it
starts from, `BENCHMARKS.md`, `knowledge-model.md`, `docs/adr/`, the `OD-09`,
`OD-17` and `OD-24` of `docs/spec-technical/open-decisions.md` that its Stack
table cites, and `scripts/mariadb/` with its Dockerfile, `setup.sql`,
`seed.sql` and TLS material. What is absent is `templates/`, `benches/` and
`examples/` — `DIV-050` for the tree, and `DIV-052` for the benchmark
directory. `DIV-032` reads the paragraph under the tree on other grounds,
where it calls the structs of `model/` the documented public surface. The
passages stating `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, the
release profile, the module conventions and the separation of library from
binary direct how code is to be written and assert nothing about the working
tree, which is the reading `DIV-050` gives the lines of the tree it leaves
alone.

*Re-run in the thirty-second edition, and two of its statements were wrong.*
The sweep above was made against the tree at `87dd6e3` and said so, and it also
said that `db80114` had since edited **both** root documents. It had not:
`db80114` touched `README.md` alone, and the last commit to touch `CLAUDE.md`
is `8f936d4`. The clause is corrected here and in the
[Overview](#overview), and the thirty-first edition's account of the same fact
is corrected in [README.md](README.md#maintenance-debt). The sweep itself was
re-run on 2026-09-21 against `CLAUDE.md` at `8f936d4` and the working tree of
that date, by enumerating every path the file names and testing each — the form
the sixth validation rule requires of a negative observation, because a wrong
path fails the test rather than passing it in silence. **Six paths have arrived
since the first sweep** — `Cargo.toml`, `src/` and its seven children, and
`tests/` — which is what discharges `DIV-051` outright and `DIV-050` in part.
**Three remain absent**, and they are the three named above. **No passage of
the class has been added.** The five commits that have edited `CLAUDE.md` since
`87dd6e3` added the Regra Zero, the synergy section, the split of `rmp` between
two skills, and the ownership of the technical specification and the decision
records; every path any of them names — `docs/spec-technical/` with its
`README.md` and `open-decisions.md`, and `docs/adr/` with its `README.md` —
exists.

**Two further passages were read on their own and are not of the kind.**

- **The changelog.** Step 4 of *Fluxo de Trabalho* says to update "README, doc
  comments e CHANGELOG", and *Documentação* names a `CHANGELOG` among the
  documents written in English. Both passages are directions rather than
  claims: step 4 is reached only after step 2 has written code, and the other
  is a rule about the language all project documentation is written in. Neither
  sends a reader to a path, so neither was of the kind then and neither is now.

  *Corrected in the thirty-first edition: two statements about the repository
  were true when they were written and have stopped being so.* This passage
  said *The repository holds no `CHANGELOG.md`*, and cited
  `docs/spec-technical/data-model.md` for neither `Cargo.toml` nor
  `CHANGELOG.md` existing at the commit it was written against. Both files
  exist in the working tree, verified on 2026-09-21: `Cargo.toml` has been in
  the repository since the fifth sprint, and `CHANGELOG.md` was created on
  2026-09-21 by `f2d19ac`. The reading this passage records is untouched by
  that — a direction is not a claim whether or not the path it names exists —
  and what is removed is only the two statements about the repository that the
  reading did not need. The citation of `docs/spec-technical/data-model.md` is
  dropped with them, because it was a citation of that file's record of an
  absence rather than of its reading, and a statement about a file this corpus
  does not own decays exactly as the fifth validation rule says. `OD-03` of
  `docs/spec-technical/open-decisions.md` is kept: it says `CLAUDE.md` "names
  one in its workflow" and settles only its format, which is a reading and not
  a claim about the tree.
- **The routing sentence of *Desempenho e Eficiência*.** It states that the
  required properties live in
  [performance-requirements.md](performance-requirements.md) and that "as
  baselines efectivamente medidas vivem em `BENCHMARKS.md`". It lies outside
  the subsection `DIV-052` targets and was therefore read on its own. It names
  a file that exists and describes it as that file describes itself — the
  register of measured baselines for `tpl`, carrying only what was actually
  measured — so it sends a reader somewhere real and tells them truly what is
  there. That no figure in it is yet a baseline for `tpl` is that file's own
  statement, made entry by entry; the sentence of `CLAUDE.md` that does assert
  benchmarks exist is in the *Disciplina de medição* subsection below it, and
  is `DIV-052`'s.

**So the class is closed in `CLAUDE.md`**: three passages are recorded there —
`DIV-050`, `DIV-051` and `DIV-052` — `DIV-046` and `DIV-054` hold the two in
the root `README.md`, and the candidate the sixteenth edition left is read here
and is not one. The claim is true of `CLAUDE.md` at `8f936d4` and of no later
state of it: by the fifth validation rule it stops being true the moment the
file is edited, and the sweep is owed again then.
