---
title: CLI Contract
status: draft
last-reviewed: 2026-09-09
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
aliases, strict parsing rules, the absence of environment configuration, and
determinism of the surface.

Out of scope: what each command does with what it parses. That belongs to the
module that owns the command.

## Actors

- **Calling agent** — an AI coding agent invoking `tpl` programmatically and
  reading its help text, exit code, and output. This is the primary consumer.
- **Operator** — a person invoking `tpl` at a shell prompt.
- **Project** — the `.tpl` folder supplying configuration and templates.

Both callers are served by the same surface. Where the two would pull in
opposite directions, the calling agent decides the outcome.

## Invocation grammar

```
tpl [global flags] <command> [<subcommand> …] [<arguments>] [local flags]
```

- **FR-CLI-001**: The system SHALL accept invocations of the form
  `tpl [global flags] <command> [<subcommand> …] [<arguments>] [local flags]`,
  where `<command>` and each `<subcommand>` are exact names drawn from the
  command tree defined in this file.

- **FR-CLI-002**: The command tree SHALL be closed. `tpl` SHALL accept only the
  canonical command names and the aliases declared in this specification.

  *Rationale.* A closed surface is a guarantee, not an accident of the parser. A
  prefix that is unique today stops being unique when a command is added, so a
  working invocation would begin to fail in a minor version, silently.

- **FR-CLI-003**: IF the first non-flag token is not a command of the tree, THEN
  the system SHALL exit `64` (`EX_USAGE`) and SHALL offer a nearest-match hint
  as defined in `FR-ERR-015`.

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
  rule of `FR-ERR-015` rather than by inference.

## Parsing rules

- **FR-CLI-014**: IF a flag that carries a single value is given more than once
  in one invocation, THEN the system SHALL exit `64` and SHALL name both values
  in the error.

  *Rationale.* Last-wins would let a script that appends a flag twice keep
  working, with the result depending on how the script grew.

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

## Configuration surface

- **FR-CLI-021**: The system SHALL NOT read any environment variable to
  determine its behaviour, its defaults, or the location of the project.

- **FR-CLI-022**: The system SHALL resolve every setting through exactly two
  layers, strongest first: the command line, then `.tpl/.cfg`, then the built-in
  default declared in this specification.

- **FR-CLI-023**: WHERE a value is written as `${VAR}` inside `.tpl/.cfg`, the
  system SHALL read the environment to expand it, as defined in `FR-CONF-013`.
  This is the only circumstance in which `tpl` reads the environment.

  *Rationale.* The ban is on flags and defaults, not on the substitution
  mechanism, whose whole purpose is to keep secrets out of the file.

- **BR-CLI-002**: An invocation is fully described by what is visible of it. Two
  identical command lines run in two different shells, against the same project
  state, cannot read different databases.

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
  same database state SHALL produce byte-identical output.

- **NFR-DET-002**: Orderings SHALL be explicit and stable — tables by name,
  columns by ordinal position, indexes by name — and SHALL NOT depend on the
  order in which the server or the filesystem returns rows or entries.

- **NFR-DET-003**: The system SHALL NOT consult `isatty()` or any other terminal
  detection to decide output format, colour, pagination, or content.

- **NFR-DET-004**: The system SHALL emit no colour and no ANSI escape sequence,
  on stdout or on stderr, under any circumstances.

  *Rationale.* Colour on stdout would put escape sequences inside the result an
  agent parses; colour on stderr alone would keep a flag, an environment
  variable, and a terminal check alive purely for decoration. Removing it also
  lets the argument parser be built without its colour support.

- **NFR-DET-005**: The `now` render variable is the single documented source of
  non-reproducibility. A template that uses it produces output that differs
  between runs by design.

## Dependencies

- [global-flags.md](global-flags.md) — the seven flags every node accepts.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — the codes referenced
  above, and the order in which conditions are evaluated.
- [help-and-version.md](help-and-version.md) — what a group node prints under
  `FR-CLI-007`.

## Open questions

- [OQ-014](open-questions.md#oq-014) — whether `tpl help` accepts a nested
  command path.
- [OQ-015](open-questions.md#oq-015) — the permitted position of a global flag
  relative to the command name.
