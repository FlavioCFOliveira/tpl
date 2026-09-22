---
title: Worked Examples
status: approved
last-reviewed: 2026-09-22
related: [use-cases.md, template-environment.md, server-contract.md, render-command.md, cfg-commands.md, project-and-discovery.md, template-commands.md]
---

# Worked Examples

## Overview

A **worked example** is a complete, runnable demonstration held in `examples/`
that builds an application's data layer for one target language out of a known
database schema, using nothing but the command line this specification fixes.
It is this specification's answer to a question no requirement answers on its
own:
whether the commands compose into the thing the tool exists to produce.

Its acceptance signal is not that `tpl` exited `0`. It is that the code the
example renders is accepted by the target language's own toolchain, per
`FR-EX-009`.

## Scope

In scope: what a worked example is, how many there are, which schemas they
read, which server they read them from, what artefacts each one holds, the
workflow each one drives, where that workflow runs, where its type mapping
comes from, and what makes one correct.

Out of scope: the content of any template, the content of any type mapping, the
code any example renders, the tools a compile gate invokes, and how the server
of `FR-EX-006` is provisioned. The first four are the examples' own; the last is
fixture work, with an owner outside this corpus.

**The subject of the requirements in this file is the worked example and not
`tpl`.** What they fix is an artefact of the repository rather than a behaviour
of the binary, on the precedent `FR-CONF-038` sets for the fixture of
`scripts/mariadb/`. The [README](README.md#writing-conventions) names the
exception and its two members.

## Actors

- **Calling agent** or **operator**, reading a worked example to learn how the
  commands compose, and running it to confirm that they still do.
- **Project**, holding the examples, the schemas they read, and the compile
  gates that accept or reject them.

## What a worked example is

- **FR-EX-001**: `examples/` SHALL hold exactly four worked examples, one for
  each of the four target languages — Go, Rust, Python and Node.js.

  *Rationale.* Four languages is what makes the demonstration a demonstration
  rather than an advertisement for one of them: a type mapping is an opinion
  that belongs to the project holding it, per `BR-ENV-002`, and one example
  would leave that claim untested against any language but its own. One example
  per language, and not several, keeps the set comparable — what differs
  between two examples is then the language and nothing else.

- **FR-EX-002**: A worked example SHALL take a known database schema as its
  input and SHALL produce, as its output, a data layer for its target language:
  source files that declare the schema's tables as types of that language and
  that that language's own toolchain accepts, per `FR-EX-009`. `UC-013` is the
  flow it realises end to end.

- **FR-EX-003**: A worked example SHALL hold exactly four kinds of artefact,
  and SHALL be complete with them:

  | Artefact | What it is |
  |---|---|
  | Templates | The templates that produce the data layer's source files, held in a directory of the example's own beside the driver script and placed into the project's `.tpl/templates/` by the workflow, per `FR-EX-010` |
  | The type-mapping macro | The example's mapping from a catalogue type to a type of the target language, per `FR-EX-008` |
  | The driver script | One program that runs the workflow of `FR-EX-004` from beginning to end, written in Python for every example whatever the target language is |
  | The compile gate | The step that submits the rendered files to the target language's compiler and reports whether they were accepted, per `FR-EX-009` |

  *Why the driver script is one language for all four.* The driver is not part
  of what is being demonstrated — it is the harness that drives the
  demonstration — so a driver written in the target language would make each
  example's harness a second thing a reader has to learn before reaching the
  first. One language across the four leaves exactly one axis of difference
  between them, which is the templates and the type mapping, and that axis is
  the subject.

  *Rejected: a driver written in each example's own target language.* It reads
  as the more natural arrangement and it was rejected because it costs the set
  its comparability and buys nothing: the driver invokes a command line, which
  every one of the four languages does equally badly and equally well.

  *Amended in the thirty-fifth edition: the templates are the example's and the
  project is the run's.* The first row read *held in the example's own
  `.tpl/templates/`, per `FR-TMPL-004`*, and that could not hold together with
  `FR-EX-004`, whose first step is `tpl init`: `FR-PROJ-014` exits `73` against
  a destination that already has a `.tpl` and changes nothing, so an example
  carrying one could never run the workflow this file obliges it to drive. The
  two requirements were written in one edition and neither was read against the
  other; the defect was found by the work that built the four examples, all of
  which resolved it the same way. `FR-EX-010` writes that resolution down, and
  this row now names where the templates are kept and cites it for where they
  are put. `FR-TMPL-004` is unchanged and is where the requirement lands: a
  template is a file under `.tpl/templates/` at the moment `tpl render` reads
  it, which is what `FR-EX-010` obliges the workflow to arrange.

## The workflow a worked example drives

- **FR-EX-004**: A worked example SHALL drive the whole workflow through the
  command line alone, and SHALL NOT call any library interface of `tpl`. The
  workflow SHALL be, in order:

  | Step | Command | Fixed by |
  |---|---|---|
  | Create the project | `tpl init` | `FR-PROJ-012` … `FR-PROJ-022` |
  | Register the access | `tpl cfg database add` | `FR-CFG-015`, `FR-CFG-027` |
  | Verify the access | `tpl cfg database test` | `FR-CFG-024`, `FR-CFG-039` |
  | Read the schema | `tpl schema …` | `FR-SCH-001` … `FR-SCH-015` |
  | Render | `tpl render` | `FR-RND-001` … `FR-RND-006` |

  **Where this workflow runs, and how the example's templates reach
  `.tpl/templates/` between its first step and its last, is `FR-EX-010`.** The
  first step is possible only in a directory that holds no project when it
  runs, and the fifth reads templates the first step did not write.

  *Rationale, and it is the whole reason the examples exist.* The primary
  consumer of `tpl` is an agent that has three channels and only three — the
  help text, the exit code, and the streams — and an example that reached past
  them into a library would demonstrate a surface no such consumer can use. It
  would also demonstrate a surface this specification does not fix: the
  [README](README.md#still-out-of-scope) keeps the library out of scope
  entirely, so an example built on one would be unspecified in its most
  load-bearing part and would break without any requirement having changed.

  *Rejected: allowing the driver to import the library where doing so is
  shorter.* It is shorter, and it was rejected because the shortness is bought
  from the one property the example is for. An example is read as a statement
  of what the tool can be asked to do; a single library call inside it makes
  that statement false, and nothing in the rendered output would say so.

- **FR-EX-005**: A worked example SHALL obtain each rendered file by
  redirecting the standard output of one `tpl render` invocation, and SHALL NOT
  expect `tpl` to write a file. `FR-RND-028` is why: `tpl render` writes its
  result to standard output and nowhere else, and declares no `--output`.

  *Consequence, stated because it is the shape of every driver script.* One
  file rendered is one invocation, and a data layer of *n* tables is *n*
  invocations plus whatever the example renders once for the whole database.
  `UC-008` and `BR-RND-002` already fix that iteration is the caller's job;
  here the caller is the driver script.

- **FR-EX-010**: A worked example SHALL drive the workflow of `FR-EX-004` in a
  **workspace**: a directory that holds no `.tpl` when the workflow starts and
  in which `tpl init` creates the project. IF a previous run left a `.tpl`
  there, THEN the workflow SHALL remove it before `tpl init` runs. The project
  SHALL NOT be an artefact of the example: the example SHALL hold its templates
  outside that project and SHALL place them under `.tpl/templates/` after
  `tpl init` and before the first `tpl render`.

  Three requirements decide those clauses. `FR-PROJ-014` exits `73` against a
  destination that already has a `.tpl` and changes nothing, so an example that
  kept its project could not run a second time. The project holds the database
  entries the workflow's second step registers, and `FR-PROJ-019` gives its
  `.cfg` mode `0600`. And `FR-TMPL-004` fixes where `tpl render` reads a
  template from, which is why the templates have to be moved rather than merely
  held: everything the example is — the four artefacts of `FR-EX-003` — and
  every file it renders lives outside the `.tpl`, and only a copy of the
  templates is inside it while the workflow runs.

  *Rationale.* `FR-EX-003` and `FR-EX-004` could not both be satisfied
  literally: one put the example's templates in its own `.tpl/templates/` and
  the other began the workflow with the command that creates `.tpl` and refuses
  to create it twice. What the two were reaching for is one arrangement, and it
  is the one this requirement states: the project is a **product** of the
  workflow rather than a part of the example, and the templates are a part of
  the example that the workflow puts into the product. Placing them after
  `tpl init` is also what makes `FR-EX-008`'s rationale hold: the Rust example
  begins from the macro `FR-PROJ-017` writes and is free to **replace** it,
  which it could not do if its own files were there first.

  *Rejected: an example that carries a committed `.tpl/templates/`, and a
  workflow that finds it already there.* It is the reading `FR-EX-003` invited
  and it fails on `FR-PROJ-014`, which exits `73` and changes nothing: the
  example could not drive the workflow `FR-EX-004` obliges it to drive, and the
  demonstration would begin by stepping around the first command it exists to
  demonstrate. Its mirror — dropping `tpl init` from `FR-EX-004` so that a
  committed project becomes admissible — fails on that requirement's own
  rationale, and costs more besides: the project carries `.tpl/.cfg`, so
  committing it commits the entries of a run against a real server.

  *Accepted cost.* A reader who opens a worked example before running it sees a
  `templates/` directory that is not where `FR-TMPL-004` says a template lives,
  and has to read this requirement to learn why. The alternative costs the
  example its first command, which is the more expensive of the two.

## The schemas a worked example reads

- **FR-EX-006**: All four worked examples SHALL read the same three schemas —
  `sakila`, `world` and `freight` — from one server, and that server SHALL be
  of the most recent series named by `FR-SRV-015`. `BR-SRV-005` governs the
  series: this requirement names the criterion and never the number.

  *Why three, and why these three.* `sakila` and `world` are published
  datasets, so a reader arrives already knowing what they contain and can judge
  the rendered data layer against a schema they recognise rather than against
  one the project designed for the occasion. `freight` is the project's own,
  and it exists for the part the other two cannot reach.

  *Rejected: covering every native type from `sakila` and `world` alone.* It is
  the option that needs no schema of the project's own, and it fails on the
  facts: neither dataset carries a `JSON`, `UUID`, `INET6` or `BIT` column, a
  generated column, a system-versioned table or a sequence. A type mapping
  written against those two alone would therefore satisfy `FR-EX-008` while
  leaving untested exactly the types a mapping gets wrong — `FR-CAT-051` fixes
  what a generated column carries, `FR-CTX-038` records that `inet6` and `uuid`
  return a `column_type` equal to their `data_type` and carry no width, and a
  mapping that never meets either is a mapping nobody has exercised. `freight`
  is the third schema for that reason and for no other, and what it must carry
  is the complement: the native types and the catalogue features the two
  published datasets do not.

  *What `freight` is obliged to carry.* Every native type and every catalogue
  feature that [catalogue-coverage.md](catalogue-coverage.md) admits into the
  model and that `sakila` and `world` do not declare, so that the union of the
  three schemas exercises the model's whole type surface. Its DDL is obliged to
  be accepted by the one series this requirement names, and by no other: the
  four-series obligation belongs to the fixture of `scripts/mariadb/`, which is
  what `FR-SRV-029` binds its tests to, and a worked example reads one server.

- **FR-EX-007**: The four worked examples SHALL read those schemas under the
  same conditions: the same server, the same three schemas at the same state,
  and database entries that differ in nothing a read can observe.

  *Rationale.* The set is comparable only if one axis varies. Where two
  examples render different data layers, the difference must be attributable to
  their templates and their type mappings, because those are what the set
  exists to compare; a difference traceable to two servers, two schema states
  or two privilege sets would make the comparison worthless and would not
  announce itself. `FR-PRIV-001` is the sharp case: a read is complete or
  incomplete according to the reader's privileges, so two entries differing
  only in their user can hand two examples two different models of one
  database.

  *Out of scope, and named so that nobody looks for it here.* How that server
  is provisioned, and by what, is fixture work with an owner outside this
  corpus, in the terms the eighth edition used for `FR-CONF-038`. This
  requirement fixes that there is one, and what must be true of it.

## The type mapping

- **FR-EX-008**: Each worked example SHALL carry its own type mapping,
  delivered as a template macro in the form `FR-ENV-011` fixes, and that macro
  SHALL produce a type of the target language for every value of `data_type`
  the model reports for a column of the three schemas of `FR-EX-006`. No worked
  example SHALL expect `tpl` to supply a mapping for its language.

  *Rationale.* `FR-ENV-011` and `BR-ENV-002` put the mapping in the project
  rather than in the binary, and a worked example is the demonstration that the
  extension point is usable — an example that received its mapping from `tpl`
  would demonstrate the opposite of what this specification decided. The Rust example
  begins from the macro `tpl init` writes under `FR-PROJ-017` and is free to
  replace it; the other three have none to begin from, which is the case the
  extension point exists for and the case nothing has yet exercised.

  *The coverage obligation is over `data_type` and is decidable.* `FR-CTX-014`
  gives a column both `data_type` and `column_type`, and `FR-CTX-038` makes
  `column_type` the raw string a template can always fall back to. The mapping
  is obliged over `data_type`, whose values are enumerable from the three
  schemas, and is not obliged over `column_type`, whose values are not.

  *Accepted cost.* A mapping complete over three schemas is not a mapping
  complete over MariaDB, and this requirement does not claim it is. What it
  buys is that no type the examples actually meet is left to a fallback nobody
  wrote.

## What makes a worked example correct

- **FR-EX-009**: A worked example SHALL carry a **compile gate**: a step that
  submits every file the example rendered to the toolchain that decides whether
  the target language accepts that file — its compiler where the language has
  one, and its own static type checker where it has not — and that reports
  failure whenever any rendered file is not accepted. **The gate's verdict is
  the example's acceptance signal, and nothing earlier in the workflow is.**

  *Rationale.* Generated code that parses as text and does not compile is the
  failure this whole set exists to catch, and it is invisible to every other
  signal available: `tpl render` exits `0` on any template that evaluated, so
  an exit code says nothing about the file, and a non-empty output says less.
  A toolchain that must accept the file is the one reader that cannot be
  satisfied by output that merely looks right. `FR-SEM-021` is the case this
  specification already knows — an interpolated boolean rendered `True` is a
  token Rust, Go and JSON all refuse, and nothing short of that reader would
  have reported it.

  *Why the clause names two kinds of reader.* Two of the four target languages
  are compiled and two are not, and a rule written over compilers alone would
  be unsatisfiable for half the set — which is the shape of defect the
  twenty-eighth edition corrected in `FR-SRV-013`. What the clause fixes is the
  property, that the reader is the language's own and that its rejection is the
  gate's failure; which tool answers to that per language is the example's
  choice, because naming four toolchains here would oblige this specification
  to track four it does not use.

  *Rejected: accepting a successful render as the signal.* It is what every
  intermediate artefact of the workflow already reports, and it was rejected
  because it is the signal that cannot fail on the defect the examples exist to
  find. A render succeeds precisely when the template evaluated, which is a
  fact about the template and not about the file.

## Business rules

- **BR-EX-001**: A worked example introduces no behaviour of its own. Every
  command it runs, every flag it passes and every field it reads is fixed by a
  requirement elsewhere in this specification; where an example appears to need
  something no requirement gives it, the defect is in this specification and
  is corrected here, never worked around in the example.

- **BR-EX-002**: The four examples are one set and are maintained as one. A
  change to the schemas of `FR-EX-006`, or to the model a schema produces,
  reaches all four; an example left behind is a broken example, not a stale
  one, because the compile gate of `FR-EX-009` is the only thing that says so
  and it says so for each example separately.

## Dependencies

- [use-cases.md](use-cases.md) — `UC-013`, the flow a worked example realises.
- [cfg-commands.md](cfg-commands.md) — the two access commands of `FR-EX-004`.
- [schema-commands.md](schema-commands.md) — the read commands of `FR-EX-004`.
- [render-command.md](render-command.md) — `FR-RND-028`, which is why
  `FR-EX-005` redirects.
- [project-and-discovery.md](project-and-discovery.md) — `FR-PROJ-014` and
  `FR-PROJ-017`, which are why `FR-EX-010` reads as it does.
- [template-commands.md](template-commands.md) — `FR-TMPL-004`, which fixes
  where a template is read from.
- [template-environment.md](template-environment.md) — `FR-ENV-011` and
  `BR-ENV-002`, which put the type mapping in the project.
- [context-document.md](context-document.md) — `FR-CTX-014` and `FR-CTX-038`,
  which fix the fields a type mapping reads.
- [server-contract.md](server-contract.md) — `FR-SRV-015` and `BR-SRV-005`,
  which fix the series `FR-EX-006` names by criterion.
- [catalogue-coverage.md](catalogue-coverage.md) — what the model carries, and
  therefore what `freight` is obliged to exercise.

## Open questions

None specific to this module. The index of
[open-questions.md](open-questions.md) is empty.
