---
title: Template Commands (Second Arm)
status: approved
last-reviewed: 2026-09-24
related: [cli-contract.md, render-command.md, project-and-discovery.md, security.md]
---

# Template Commands (Second Arm)

## Overview

The second arm reads and presents the templates available to the project. It
never contacts a database, never evaluates a template, and never leaves the
template root.

## Scope

In scope: the four subcommands, template naming and extension resolution, what
counts as a template, and the containment rules of the template root.

Out of scope: template authoring, the template language, the render context, and
the set of filters and tests available to a template.

## Actors

- **Calling agent** or **operator**, invoking the command.
- **Project**, supplying `.tpl/templates/` as the template root.

## Command surface

```
tpl template list                       List the project's templates
tpl template show  <name>               Print a template's source
tpl template check [<name> …]           Parse templates without rendering them
tpl template path  [<name>]             Print the template root, or one template's path
```

- **FR-TMPL-001**: `tpl template` SHALL be a group node, per `FR-CLI-007`.

- **FR-TMPL-002**: The system SHALL provide exactly the four subcommands listed
  above. They have no aliases.

- **FR-TMPL-003**: No `template` subcommand SHALL open a database connection,
  read the cache, or require a database entry to be selected.

## What is a template

- **FR-TMPL-004**: A template SHALL be a regular file under `.tpl/templates/`
  whose name ends in `.jinja`.

- **FR-TMPL-005**: The system SHALL ignore every file under `.tpl/templates/`
  that is not a template: it is not listed, not renderable, and not includable.

  *Rationale.* Treating every regular file as a template would let a `LICENSE`
  or a `.DS_Store` appear as renderable.

- **FR-TMPL-006**: A template name SHALL be the path of the file relative to
  `.tpl/templates/`. `rust/struct.jinja` names
  `.tpl/templates/rust/struct.jinja`.

- **FR-TMPL-007**: On the command line, the system SHALL accept a template name
  with or without its `.jinja` extension. `rust/struct` and `rust/struct.jinja`
  name the same template.

- **FR-TMPL-008**: Inside a template, a name SHALL be literal. `{% include %}`,
  `{% import %}`, and `{% extends %}` SHALL NOT resolve the extension
  implicitly.

  *Rationale.* The loader behaves exactly as the template engine documents, with
  no intermediate resolution layer. The collision between a file named `x` and a
  file named `x.jinja` disappears by construction, because a file named `x` is
  not a template at all.

- **FR-TMPL-009**: IF a template refers to another template by a name that does
  not resolve literally — for example `{% include "_header" %}` where the file
  is `_header.jinja` — THEN the system SHALL fail the render with `65`
  (`EX_DATAERR`), naming the template, the line, and the column.

  *Accepted cost.* A name copied from `tpl template list` into an
  `{% include %}` fails, because the listing omits the extension the include
  requires.

- **FR-TMPL-010**: The help of `tpl template list` SHALL state that listed names
  omit the `.jinja` extension and that an `{% include %}` requires it.

## `tpl template list`

- **FR-TMPL-011**: `tpl template list` SHALL list every template of the project,
  by name, with the `.jinja` extension removed.

  ```
  tpl template list

  docs/table.md
  example
  rust/_types
  rust/struct
  ```

  *Corrected in the fifth edition.* The listing previously showed
  `rust/struct` before `rust/_types`, which is not the order `FR-TMPL-013`
  fixes: byte-wise, `_` (0x5F) sorts before `s` (0x73), so `rust/_types` comes
  first. The example was the evidence that no order had been stated, and it is
  now the worked case of the one that has.

- **FR-TMPL-012**: Listed names SHALL be usable verbatim as the positional
  argument of `tpl render`, `tpl template show`, `tpl template check`, and
  `tpl template path`.

- **FR-TMPL-013**: The system SHALL list templates by the **displayed name** —
  the path of the file relative to `.tpl/templates/` with the `.jinja`
  extension removed, per `FR-TMPL-011` — ascending, compared byte by byte,
  under the default rule of `NFR-DET-002`. The order SHALL NOT depend on
  directory iteration order.

  *Rationale.* Sorting by the displayed name rather than by the filename on
  disk is what makes the listing self-evidently ordered: a reader sees the
  column it is sorted on. The two orders differ, and not only cosmetically —
  `a.jinja` and `a/b.jinja` display as `a` and `a/b`, and `.` (0x2E) sorts
  before `/` (0x2F), so a sort on the filename places `a.jinja` first while a
  sort on the displayed name places `a` first for a different reason and would
  place them differently the moment a third name fell between. One rule, over
  the strings actually printed, has no such case.

  Byte-wise and not by any collation, for the reason `NFR-DET-002` gives and
  `FR-SCH-014` gives for `--pattern`: the order must be the same on every
  machine and in every locale. The consequence, as there, is that `Docs` sorts
  before `docs` and `_partials` after `Zebra`.

  *Closes* `OQ-013`, now listed under [Closed](open-questions.md#closed).

  *Composition.* The same order governs `--format json`, per `FR-TMPL-028`:
  the `templates` array holds objects carrying `name`, and it is ordered by
  that `name`.

- **FR-TMPL-014**: `tpl template list` SHALL list partials — templates intended
  only to be included — alongside every other template, without a flag to hide
  or reveal them.

  *Accepted cost.* A template whose name ends `.md.jinja` lists as
  `docs/table.md`, which reads like a Markdown file rather than a template.

## `tpl template show`

- **FR-TMPL-015**: `tpl template show <name>` SHALL print the source of the
  named template, unaltered, to stdout. The escaping of `FR-OUT-018` SHALL NOT
  apply to it, per `FR-OUT-019`.

  *Amended in the third edition.* The exemption from `FR-OUT-018` is new. As
  first written, this requirement and `FR-OUT-018` contradicted each other:
  `template` was named among the read commands whose output is escaped, and a
  template containing a tab, a form feed, or an escape sequence in a literal
  could not be both escaped and unaltered.

- **FR-TMPL-016**: `tpl template show` SHALL take exactly one positional
  argument.

## `tpl template check`

- **FR-TMPL-017**: `tpl template check` SHALL perform syntax analysis only. It
  SHALL NOT evaluate an expression, SHALL NOT call a function or a filter, and
  SHALL NOT connect to a database.

  *Rationale.* Validating variables would mean evaluating the template, so a
  command called `check` would execute an unknown template arriving in a clone.

- **FR-TMPL-018**: WHEN `tpl template check` is invoked with no positional
  argument, the system SHALL check every template of the project.

- **FR-TMPL-019**: WHEN `tpl template check` is invoked with one or more
  positional names, the system SHALL check exactly those templates.

- **FR-TMPL-020**: IF a checked template contains a syntax error, THEN the
  system SHALL exit `65`, naming the template, the line, and the column.

  *Note added in the forty-sixth edition.* Where more than one checked
  template contains a syntax error, `FR-TMPL-032` states how every one of them
  is reported.

- **FR-TMPL-032**: WHEN `tpl template check` finds a syntax error in one or
  more of the templates `FR-TMPL-018` or `FR-TMPL-019` selects, the system
  SHALL check every selected template before it reports, SHALL then write to
  stderr one message of `FR-ERR-008` for each template that failed, and SHALL
  exit `65` (`EX_DATAERR`). In detail:

  1. Each message SHALL report the first syntax error the parser meets in its
     template, and SHALL carry what `FR-TMPL-020` and `FR-ERR-011` oblige: the
     template name, the line, the column, and the chain of engine errors.
  2. The messages SHALL follow one another with no blank line, no separator
     and no summary line, so each message begins with its own `error:` line
     and the number of `error:` lines is the number of failing templates.
     Every message SHALL end with the same line, `exit:  65 (EX_DATAERR)`.
  3. The order SHALL be the order of checking: with no positional argument,
     the order of `FR-TMPL-013`; with positional names, the order in which
     they are written on the command line. A template named more than once
     SHALL be checked once and reported at most once, at its first position.
  4. stdout SHALL stay empty, per `FR-ERR-033`.
  5. IF checking a template raises a condition other than a syntax error —
     the file cannot be read, for example — THEN the system SHALL stop, SHALL
     write the message of that condition alone, and SHALL exit with that
     condition's code. The messages of templates that failed before it SHALL
     NOT be written, so the `exit` line of every message written is the code
     the process returns.

  ```
  error: template 't/syn' has a syntax error at line 3, column 9
  cause: ...
  hint:  ...
  exit:  65 (EX_DATAERR)
  error: template 't/syn2' has a syntax error at line 1, column 4
  cause: ...
  hint:  ...
  exit:  65 (EX_DATAERR)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012` and `FR-ERR-034`. The example fixes only the shape.

  *Rationale.* The help of the command says that it checks every template, and
  a check that stops at the first failure checks one and reports one, so a
  project with several broken templates took one invocation per template to
  learn what one invocation could tell. A calling agent fixes all of them in
  one pass when it is shown all of them.

  *Weighed against `FR-ERR-006`.* That requirement orders conditions and
  reports the first that fails. A syntax error in two templates is one
  condition of one step, met by two objects, and not two conditions, so the
  order is unchanged: template resolution, step 4, still runs first over every
  positional name, and a name that does not resolve is reported alone, per
  `FR-TMPL-026` and `FR-TMPL-027`, before any template is checked.

  *Rejected: stopping at the first failure and saying so in the help.* It is
  honest, and it makes the caller loop: fix, run, read the next failure, run
  again, for a fact the command held at its first run.

  *Rejected: one message carrying every failure.* `FR-ERR-008` fixes a message
  at four labelled lines, and `FR-ERR-024` escapes the newline in every
  interpolated value so that no value can forge a fifth. Several failures in
  one `cause` line would be one long line that a caller has to split on a
  separator this corpus does not fix.

  *Added in the forty-sixth edition,* for rmp `#274`, from finding S-05 of the
  second re-audit.

- **BR-TMPL-001**: `tpl template check` is safe to run against a template you
  have not read. This is a guarantee, not a side effect.

## `tpl template path`

- **FR-TMPL-021**: WHEN `tpl template path` is invoked with no positional
  argument, the system SHALL print the absolute path of the template root.

- **FR-TMPL-022**: WHEN `tpl template path <name>` is invoked, the system SHALL
  print the absolute path of that template's file.

  ```
  tpl template path                    /proj/.tpl/templates
  tpl template path rust/struct        /proj/.tpl/templates/rust/struct.jinja
  ```

## `json` output

- **FR-TMPL-028**: `tpl template list` SHALL emit its `json` output in the
  envelope of `FR-OUT-024`, with a `data` of one key, `templates`, per
  `FR-OUT-030`, whose value is an array of objects each carrying `name`:

  ```json
  {"schema_version":1,"source":"project","data":{"templates":[{"name":"rust/struct"}]}}
  ```

  *Rationale.* An array of objects rather than an array of bare strings,
  because a string cannot gain a field and `FR-OUT-014` makes adding one the
  only non-breaking way for a listing to grow. It is the same argument
  `FR-CDOC-010` made for `source`.

- **FR-TMPL-029**: `tpl template path` SHALL emit its `json` output in the
  envelope of `FR-OUT-024`, with a `data` of one key, `path`, whose value is
  the absolute path `FR-TMPL-021` or `FR-TMPL-022` would print.

- **FR-TMPL-030**: `source` on a `template` document SHALL be `project`, per
  `FR-OUT-026`, because no `template` subcommand reads a catalogue, per
  `FR-TMPL-003`.

- **FR-TMPL-031**: WHEN a project has no template, `tpl template list` SHALL
  exit `0` with an empty `templates` array, per `FR-OUT-033` through
  `FR-OUT-035`, and `tpl template check` with no positional argument SHALL
  check nothing and exit `0`.

## Containment

- **FR-TMPL-023**: The template root SHALL be `.tpl/templates/` of the resolved
  project, and SHALL be the boundary of every template lookup.

- **FR-TMPL-024**: The system SHALL refuse a symbolic link inside
  `.tpl/templates/`. A symlinked entry is not a template and is not listed,
  shown, checked, rendered, or included.

- **FR-TMPL-025**: The system SHALL canonicalise every resolved template path
  and SHALL re-check it against the template root.

- **FR-TMPL-026**: IF a resolved path lies outside the template root, THEN the
  system SHALL exit `65`.

  *Rationale.* `ln -s ../.cfg .tpl/templates/leak.jinja` must not turn
  `tpl template show` into a credential dump.

- **FR-TMPL-027**: IF a named template does not exist, THEN the system SHALL
  exit `66` (`EX_NOINPUT`) with a nearest-match suggestion over the template
  names that do exist.

## Business rules

- **BR-TMPL-002**: The second arm is read-only with respect to both the database
  and the filesystem. No `template` subcommand writes anything, anywhere.

- **BR-TMPL-003**: Only the command line resolves an omitted extension. Inside a
  template, `tpl` adds no layer the engine does not have.

## Dependencies

- [project-and-discovery.md](project-and-discovery.md) — where the template root
  comes from.
- [render-command.md](render-command.md) — the same naming rules apply to the
  positional argument of `render`.
- [security.md](security.md) — the containment rules restated as a cross-cutting
  security guarantee.

## Open questions

None specific to this module. `OQ-013` is answered by `FR-TMPL-013` and is
listed under [Closed](open-questions.md#closed).
