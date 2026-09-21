---
title: CLI Contract
status: approved
last-reviewed: 2026-09-21
related: [global-flags.md, help-and-version.md, errors-and-exit-codes.md, output-formats.md]
---

# CLI Contract

## Overview

This file defines the shape of the command line itself: the invocation grammar,
the complete command tree, how a node with children behaves, how arguments and
flags are parsed, and the guarantees that make an invocation reproducible. Every
other module in this specification adds commands and flags inside the frame set
here.

## Scope

In scope: invocation grammar, the closed command tree, group-node behaviour,
aliases, strict parsing rules including where a global flag may appear, the
absence of environment configuration, and determinism of the surface.

Out of scope: what each command does with what it parses. That belongs to the
module that owns the command.

## Actors

- **Calling agent**, reading the surface this file fixes through help text,
  exit code, and output. This is the primary consumer.
- **Operator**, invoking that same surface at a shell prompt.
- **Project**, supplying the configuration and the templates the commands of
  this tree reach.

All three are defined in [glossary.md](glossary.md); the lines above state
each one's stake in this file, which is the form every other module's *Actors*
section uses.

Both callers are served by the same surface. Where the two would pull in
opposite directions, the calling agent decides the outcome.

*Amended in the thirty-third edition: the three definitions move to the
glossary and these lines state a stake instead.* This was the one *Actors*
section in the corpus that said what its actors **are** rather than what they
want from the file, and it was the only definition of *calling agent* and of
*operator* anywhere — two terms used in twenty module files and sixteen. Its
third line was worse than a second convention: it defined *project* a second
time, and not the same way. This file had it as *the `.tpl` folder supplying
configuration and templates*, where [glossary.md](glossary.md#project) has a
project as any directory **containing** a `.tpl` folder, with that folder as
the project root. `FR-PROJ-001` is with the glossary. The glossary governs,
and the sentence that is wrong does not survive the move.

*Rejected: leaving this section and adding two glossary entries beside it.* It
buys one file a fuller answer and leaves the corpus with two places to look
for it, which is the defect the thirty-second edition's rule names. It would
also have left the `project` line standing, and that line is false.

## Invocation grammar

```
tpl [global flags] <command> [<subcommand> …] [<arguments>] [local flags]
```

- **FR-CLI-001**: The system SHALL accept invocations of the form
  `tpl [global flags] <command> [<subcommand> …] [<arguments>] [local flags]`,
  where `<command>` and each `<subcommand>` are exact names drawn from the
  command tree defined in this file. This form is canonical, not restrictive:
  `FR-CLI-024` accepts a global flag in any position.

- **FR-CLI-002**: The command tree SHALL be closed. `tpl` SHALL accept only the
  canonical command names and the aliases declared in this specification.

  *Rationale.* A closed surface is a guarantee, not an accident of the parser. A
  prefix that is unique today stops being unique when a command is added, so a
  working invocation would begin to fail in a minor version, silently.

- **FR-CLI-003**: IF the first non-flag token is not a command of the tree, THEN
  the system SHALL exit `64` (`EX_USAGE`) and SHALL offer a nearest-match
  suggestion as defined in `FR-ERR-019`.

- **FR-CLI-004**: The system SHALL NOT infer a command from a prefix of its
  name. `tpl sch tables` is `64`.

- **FR-CLI-005**: The system SHALL NOT infer a long flag from a prefix of its
  name. `tpl --data shop schema tables` is `64`.

- **FR-CLI-006**: The system SHALL NOT execute external subcommands. A token
  that is not a declared command SHALL NOT cause a lookup of a `tpl-<token>`
  executable on `PATH`.

  *Rationale.* External subcommands would make the tree unclosable in
  `tpl help --format json`, and would let `PATH` choose executable code.

## Command tree

```
tpl
├── schema                       first arm — read the database structure
│   ├── info
│   ├── tables      (tbls)
│   ├── table       (tbl)
│   ├── views       (vws)
│   ├── view        (vw)
│   ├── routines    (rtns)
│   ├── routine     (rtn)
│   └── dump
├── template                     second arm — read the project's templates
│   ├── list
│   ├── show
│   ├── check
│   └── path
├── render                       third arm — render one template, once
├── cache                        the catalogue cache
│   ├── load
│   ├── clean
│   └── status
├── cfg                          the .tpl/.cfg file
│   ├── get
│   ├── set
│   ├── unset
│   ├── list
│   └── database  (db)
│       ├── add
│       ├── list
│       ├── show
│       ├── update
│       ├── remove
│       └── test
├── init                         create a .tpl project
├── help                         help, and the whole tree as JSON
└── version
```

- **FR-CLI-007**: WHEN a group node is invoked with no child, the system SHALL
  print that node's own help to stdout and SHALL exit `0`.

  *Rationale.* A bare `tpl` reads as an implicit request for help, not as a
  usage error. The rule is uniform so that a caller need not know which nodes
  are groups. The accepted cost is that `tpl > f.txt` writes a help dump into
  the file, and that the `0` does not mean work was done.

- **FR-CLI-008**: The group nodes SHALL be exactly `tpl`, `tpl schema`,
  `tpl template`, `tpl cache`, `tpl cfg`, and `tpl cfg database`.

- **FR-CLI-009**: A group node SHALL NOT have a default action. No group node
  performs work of its own under any circumstances.

- **FR-CLI-010**: The top-level commands SHALL be exactly `schema`, `template`,
  `render`, `cache`, `cfg`, `init`, `help`, and `version`.

## Aliases

- **FR-CLI-011**: The system SHALL accept the following aliases, and no others:

  | Node | Canonical | Alias |
  |---|---|---|
  | `tpl schema` | `tables` | `tbls` |
  | `tpl schema` | `table` | `tbl` |
  | `tpl schema` | `views` | `vws` |
  | `tpl schema` | `view` | `vw` |
  | `tpl schema` | `routines` | `rtns` |
  | `tpl schema` | `routine` | `rtn` |
  | `tpl cfg` | `database` | `db` |

  *Rationale.* A MariaDB routine is a procedure or a function, so the alias
  `procs` would misstate the scope of what the command returns; `rtns` and `rtn`
  are used instead.

- **FR-CLI-012**: The system SHALL NOT accept any top-level alias. `tpl s`,
  `tpl t`, and `tpl r` are `64`.

  *Rationale.* A one-letter top-level alias would be frozen forever, and `d` is
  already taken by `-d/--database`.

- **FR-CLI-013**: The system SHALL show every alias in the help of its parent
  node and in the JSON command tree.

- **BR-CLI-001**: Aliases are never ambiguous. `tbl` and `tbls` differ by one
  character, so a mistyped alias SHALL be resolved through the nearest-match
  rule of `FR-ERR-019` rather than by inference.

## Parsing rules

- **FR-CLI-014**: IF a flag that carries a single value is given more than once
  in one invocation, THEN the system SHALL exit `64` and SHALL name both values
  in the error.

  *Rationale.* Last-wins would let a script that appends a flag twice keep
  working, with the result depending on how the script grew.

- **FR-CLI-025**: WHEN a flag that carries **no value** is given more than once
  in one invocation, the system SHALL accept the invocation and SHALL give the
  flag the effect of one occurrence. `tpl -q -q schema tables` is
  `tpl -q schema tables`, and `tpl schema tables --direct --direct` is
  `tpl schema tables --direct`.

  Six flags of this specification are reached: `-q/--quiet`, `--pretty`,
  `--direct`, `--no-cache`, `-h/--help` and `-V/--version`. `-v/--verbose` is
  **excluded**, and it is the one exclusion: `FR-CLI-016` counts its
  repetitions up to three levels and saturates above three, so a second
  occurrence of that flag carries meaning and there is nothing to make
  idempotent. A valueless flag added to this specification later is governed by
  this requirement without amendment; the six are named so that an implementer
  can see which exist today and not to close the class.

  Where `FR-HELP-013` obliges help to state whether a flag is repeatable, the
  fact for such a flag is that it is accepted more than once and that further
  occurrences have the effect of the first.

  *Rationale.* `FR-CLI-014` refuses a repeated flag that carries a value, and
  the ground it gives is that last-wins would let a script that appends a flag
  twice keep working with the result depending on how the script grew. That
  ground does not transfer here, because there are no two values to disagree.
  What decides it instead is the primary consumer: an agent that appends a flag
  to a command line it has already built is the ordinary way a command line
  gets built programmatically, and refusing it makes the count of a flag's
  occurrences a fact the agent has to learn from a failure. That is the
  argument `FR-CLI-024` already made for the position of a global flag, and it
  applies here unchanged. Two of the six make it sharpest: `-h/--help` is the
  flag a caller reaches for **to recover from a failure**, so refusing
  `tpl -h -h` refuses the recovery path itself; and `-v/--verbose` is already
  accepted any number of times, so a caller who may write `-v -v -v` and may
  not write `-q -q` is holding a distinction nothing on the surface explains.

  *What is unchanged.* `FR-CLI-015` still refuses `-q` together with `-v` with
  `64`: that is two flags disagreeing and not one flag repeating.
  `FR-CLI-014` is untouched and still reaches every flag that carries a single
  value. Neither requirement reaches `--set`, which `FR-RND-008` declares
  repeatable with distinct keys and `FR-RND-014` refuses on a repeated key.
  `FR-CLI-019` still rejects at each node every flag that node does not
  declare, so `tpl init --pretty --pretty` is `64` for the flag being unknown
  there and not for being written twice.

  *Rejected: `64`, which is what the parser produces today.* It is the outcome
  and not a decision — the refusal was given a message in the sixth sprint
  without the question being put — and the only ground available for it is
  `FR-CLI-014`'s, which is about a value. Refusing also costs the caller a
  whole invocation to learn something that changes nothing about what they
  asked for: the meaning of `tpl -q -q` is not in doubt to anybody, including
  the parser that refuses it.

  *Rejected: `64` for the four flags a command declares and acceptance for the
  two that end the invocation.* It splits one rule over two sets with no
  difference a caller can see from the command line, and it leaves
  `tpl --pretty --pretty` and `tpl -h -h` on opposite branches of a rule about
  writing a flag twice.

  *Accepted cost.* A caller whose command line carries a flag twice because its
  generator has a defect is not told so. The corpus does not treat a refusal as
  a defect detector anywhere else — `FR-CLI-014` refuses for ambiguity and not
  for duplication — and `BR-CLI-002` is satisfied either way, because what the
  invocation does is still fully determined by what is visible of it.

- **FR-CLI-015**: IF `-q/--quiet` and `-v/--verbose` are both given, THEN the
  system SHALL exit `64`.

- **FR-CLI-016**: The system SHALL count repetitions of `-v/--verbose` up to
  three levels and SHALL saturate above three without error.

- **FR-CLI-017**: The system SHALL accept `--` as an argument terminator on
  every command. Tokens after `--` SHALL be treated as positional arguments.

- **FR-CLI-018**: IF a flag value begins with `-` and is supplied as a separate
  token, THEN the system SHALL exit `64` and the hint SHALL show the corrected
  `--flag=value` form. A value beginning with `-` SHALL be accepted in the
  `--flag=value` form, or as a positional argument after `--`.

- **FR-CLI-019**: A command SHALL reject any flag it does not declare, with `64`
  and the unknown-flag message.

  *Rationale.* There is no "known but inapplicable" category. `tpl init -o /tmp/x`
  and `tpl schema dump --format text` are both unknown-flag errors, not special
  cases needing their own explanation.

- **FR-CLI-020**: The system SHALL treat a flag and its value as case-sensitive
  and SHALL NOT normalise the case of either.

- **FR-CLI-024**: A global flag of `FR-GLOB-001` SHALL be accepted in any
  position on the command line: before the command, between a command and its
  subcommand, after the positional arguments, and among the local flags. The
  grammar of `FR-CLI-001` SHALL be read as the canonical form in which this
  specification and the help write an invocation, and SHALL NOT be read as a
  constraint on where a global flag may be written.

  ```
  tpl -d shop schema tables
  tpl schema tables -d shop
  tpl schema -d shop tables --pattern '%_log'
  ```

  All three are the same invocation.

  *Rationale.* Every example in this corpus puts a global flag before the
  command, and nothing said whether the other positions were accepted. The two
  candidate answers differ sharply for the primary consumer: an agent that
  appends `-d shop` to a command line it has already built is the ordinary way
  a command line gets built programmatically, and refusing it with `64` would
  make the position of a flag a fact the agent has to learn from a failure. A
  global flag is global, per `FR-GLOB-002`; a flag that is accepted only in one
  position is a positional argument wearing a flag's spelling.

  *What is unchanged.* `FR-CLI-017` continues to govern `--`: a token after the
  argument terminator is a positional argument, so `tpl render x -- -d` passes
  `-d` to the command as an argument and does not select a database entry.
  `FR-CLI-014` continues to make a repeated single-value flag `64` wherever the
  two occurrences appear, and `FR-CLI-019` continues to reject at each node
  every **local** flag that node does not declare — this requirement frees the
  position of the seven global flags and of nothing else.

  *Closes* `OQ-015`, now listed under [Closed](open-questions.md#closed).

  *Accepted cost.* A value that looks like a command name cannot be
  distinguished from a command name by position alone, so `tpl -d schema
  tables` selects the database entry named `schema` and then fails on `tables`
  as an unknown top-level command. `FR-CLI-003` gives that a `64` with a
  nearest-match suggestion, and `FR-CLI-018` already refuses a separate-token
  flag value beginning with `-`.

## Configuration surface

- **FR-CLI-021**: The system SHALL NOT read any environment variable to
  determine its behaviour, its defaults, or the location of the project.

- **FR-CLI-022**: The system SHALL resolve every setting through exactly two
  configuration layers above the built-in default, strongest first: the command
  line, then `.tpl/.cfg`, then the built-in default declared in this
  specification. `FR-CONF-029` owns this rule.

- **FR-CLI-023**: WHERE a value is written as `${VAR}` inside `.tpl/.cfg`, the
  system SHALL read the environment to expand it, as defined in `FR-CONF-015`.
  This is the only circumstance in which `tpl` reads the environment.

  *Rationale.* The ban is on flags and defaults, not on the substitution
  mechanism, whose whole purpose is to keep secrets out of the file.

- **BR-CLI-002**: An invocation is fully described by what is visible of it. Two
  identical command lines run in two different shells, against the same project
  state, cannot read different databases. Nothing a shell can set may decide
  which project is discovered, which database entry is selected, or which
  server is reached.

  *Amended in the eighth edition.* The third sentence is new, and it is the
  clause this rule was missing. Project discovery stopped at the home
  directory of `FR-PROJ-005`, which could only be located from `HOME`, so a
  shell exporting a different `HOME` made two identical command lines discover
  two different projects — this rule failing in its own terms while every
  requirement it relied on stood. `FR-PROJ-005` is amended to drop that
  boundary, and this rule now states what a shell may not decide rather than
  leaving it to be inferred from whichever requirements happen to exist.

## Behavioural invariants

- **BR-CLI-003**: `tpl` is never interactive. It SHALL NOT prompt for
  confirmation, SHALL NOT prompt for a password, SHALL NOT invoke a pager, and
  SHALL NOT read stdin except for an explicitly requested `--context -`. When
  information is missing, it fails at once with the code that names what is
  missing.

- **BR-CLI-004**: Success is silent. A successful command writes only its
  expected result to stdout and exits `0`. There is no `OK`, no summary, no
  count, no elapsed time, no emoji, no progress indicator, and no spinner. A
  command that produces no data — `tpl init`, `tpl cfg database add` — writes
  nothing, and the `0` is the message.

- **BR-CLI-005**: Help output is a legitimate stdout payload and the one
  deliberate exception to `BR-CLI-004`. `FR-CLI-007`, and every explicit
  `--help`, write to stdout and exit `0`.

- **BR-CLI-006**: Results go to stdout; everything else goes to stderr,
  including warnings and log output.

## Determinism

- **NFR-DET-001**: The same invocation against the same project state and the
  same database state SHALL produce byte-identical **stdout**. The diagnostic
  output written to stderr is neither deterministic nor contract.

  *Amended in the second edition.* The first edition said "output" without
  qualification, which contradicted `FR-GLOB-017`: a phase timing differs on
  every run by construction. Naming stdout resolves the contradiction in the
  direction that was always intended — stdout is the result a caller parses and
  compares between runs, and stderr is diagnosis.

- **NFR-DET-002**: Orderings SHALL be explicit and stable, and SHALL NOT depend
  on the order in which the server or the filesystem returns rows or entries.
  Every collection the system presents SHALL be ordered by name, ascending,
  compared byte by byte, except where this specification names another order:

  | Collection | Order | Fixed by |
  |---|---|---|
  | A table's columns | Ordinal position | This requirement |
  | An index's columns | The order the catalogue states | `FR-CAT-010` |
  | A primary key's columns | The order the catalogue states | `FR-CAT-043` |
  | A foreign key's columns, and the referenced columns paired with them | The order the catalogue states | `FR-CAT-045` |
  | An `ENUM` or `SET` member list | The order the catalogue states | `FR-CTX-016` |
  | A routine's parameters | Declaration order | `FR-CAT-018` |
  | Every other collection | Name, ascending, byte-wise | This requirement |

  *Amended in the third edition.* The default rule and the table of exceptions
  are new. The first edition named three orders — tables by name, columns by
  ordinal, indexes by name — and the model carries seven further collections:
  views, routines, outgoing foreign keys, `referenced_by`, triggers, `CHECK`
  constraints, and routine parameters. Two of the seven were fixed elsewhere by
  accident rather than by rule, and five had no stated order at all, so five
  collections could not satisfy `NFR-DET-001` and `FR-CTX-005` had nothing
  observable to point at.

  *Rationale.* One default plus a short list of exceptions is checkable in a
  single test and holds for a collection this specification has not thought
  of yet. An enumeration of every collection would be incomplete again the
  next time the model grows.

  *Amended in the seventh edition: three exceptions added, and every one of
  them is a collection the default rule would have corrupted.* The exceptions
  were written when the catalogue field lists were unobserved, so three
  ordered collections the model carries had no entry and fell to the default.

  - **A primary key's columns.** `voyage_leg`'s key is
    `(vessel_imo, voyage_number, leg_sequence)`; sorted by name it becomes
    `(leg_sequence, vessel_imo, voyage_number)`, which is a different key.
  - **A foreign key's columns.** The referencing list and the referenced list
    are **paired positionally**, per `FR-CAT-045`, so sorting either one
    independently pairs each column with the wrong counterpart, and sorting
    both pairs them wrongly whenever the two orders differ.
  - **An `ENUM` or `SET` member list.** The order **is the meaning**: a
    member's position is the ordinal it is stored as. Sorting
    `enum('Draft','Booked','Loaded',…)` by name renumbers every member, and a
    generator emitting a target-language enumeration from it would assign
    every value the wrong discriminant, at exit `0`.

  All three are silent corruptions of correct-looking output, which is the
  class of failure this corpus works hardest to prevent, and none of them was
  reachable before the field lists were recorded.

  *Rejected.* Enumerating an order for each of the ten collections, for the
  reason above; and ordering by the catalogue's own ordinal wherever one exists,
  which would make the order of a listing depend on a field the caller cannot
  see and which `FR-CAT-024` may exclude from the model.

  *Accepted cost.* Byte-wise comparison, chosen for the same reason
  `FR-SCH-014` folds case over ASCII only, puts `Orders` before `customers`
  and `_internal` after `Zebra`. The order is the same on every machine and in
  every locale, which is what a caller comparing two runs needs.

- **NFR-DET-003**: The system SHALL NOT consult `isatty()` or any other terminal
  detection to decide output format, colour, pagination, or content.

- **NFR-DET-004**: The system SHALL emit no colour and no ANSI escape sequence,
  on stdout or on stderr, under any circumstances. This requirement governs
  what the system itself composes for presentation. It does not govern a byte
  the system carries from elsewhere — a catalogue value, a rendered template, a
  `--context` value, or the argument vector — which is content and not
  decoration; what may reach a reader from such a value is fixed by
  `FR-ERR-024` in a diagnostic message, by `FR-OUT-018` in the output of a read
  command, and by `FR-OUT-019`, which emits the result of `tpl render` and the
  source printed by `tpl template show` byte for byte.

  *Rationale.* Colour on stdout would put escape sequences inside the result an
  agent parses; colour on stderr alone would keep a flag, an environment
  variable, and a terminal check alive purely for decoration. Removing it also
  lets the argument parser be built without its colour support.

  *Amended in the twentieth edition: the requirement says whose bytes it
  governs.* "Under any circumstances" was read against a catalogue name
  carrying `U+009B`, the single-character CSI: `FR-ERR-024` escapes the C0
  range, and therefore `ESC`, and does not escape `U+009B`, so a terminal that
  honours 8-bit controls could read a control sequence out of a name the system
  printed. Deriving the diagnostic renderer from this corpus forced the choice,
  and the answer is that this requirement is not about that byte. Read as
  reaching content, it would forbid `tpl render` to emit a template that writes
  an escape sequence deliberately — which `FR-OUT-019` guarantees byte for byte
  — so it would need an exception carved into a requirement whose subject is
  colour, to protect a behaviour another requirement already fixes; and it
  would become a third owner of escaping, with a third exception list, over the
  values `FR-ERR-024` and `FR-OUT-018` already own, which is the second copy
  nobody edits. Its subject is colour and terminal decoration, as every clause
  of its rationale says: a flag, an environment variable, a terminal check, and
  the argument parser's colour support.

  *Rejected.* Reading it as reaching every byte except the two outputs
  `FR-OUT-019` excepts, and widening the escape set of `FR-ERR-024` and
  `FR-OUT-018` to the C1 range on this requirement's authority. It is coherent,
  and it decides from here a question that belongs to the two requirements that
  own escaping, each of which has its own ground and its own exceptions. One
  consequence stands and is not closed here: a value escaped over the C0 range
  alone can still carry a C1 control to a terminal that honours it. That is a
  question for those two requirements, and it is recorded as an item in the
  [README](README.md#maintenance-debt).

- **NFR-DET-005**: The `now` render variable is the single documented source of
  non-reproducibility. A template that uses it produces output that differs
  between runs by design. Its form is fixed by `FR-CTX-028`.

## Dependencies

- [global-flags.md](global-flags.md) — the seven flags every node accepts.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — the codes referenced
  above, and the order in which conditions are evaluated.
- [help-and-version.md](help-and-version.md) — what a group node prints under
  `FR-CLI-007`.

## Open questions

None specific to this module. `OQ-014` is answered by `FR-HELP-026` and
`OQ-015` by `FR-CLI-024`; both are listed under
[Closed](open-questions.md#closed).
