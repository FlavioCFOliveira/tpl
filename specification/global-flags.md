---
title: Global Flags
status: draft
last-reviewed: 2026-09-09
related: [cli-contract.md, configuration-model.md, cache-commands.md, output-formats.md]
---

# Global Flags

## Overview

`tpl` has exactly seven global flags. Every other flag is local: it is declared
by the commands to which it applies, and rejected as unknown everywhere else.
This file defines the seven, their values and defaults, and the precedence by
which a setting is resolved.

## Scope

In scope: the global flag set, each flag's meaning and default, the precedence
rule, and the list of flags that are deliberately not global.

Out of scope: the behaviour each flag triggers inside a particular command,
which belongs to that command's module.

## The seven global flags

- **FR-GLOB-001**: The system SHALL declare exactly seven global flags, and no
  others:

  | Flag | Short | Value | Default |
  |---|---|---|---|
  | `--database <name>` | `-d` | database entry name | none; see `FR-GLOB-004` |
  | `--tpl-dir <path>` | | filesystem path | none; discovery applies |
  | `--timeout <seconds>` | | positive integer | `30` |
  | `--verbose` | `-v` | none, repeatable | off |
  | `--quiet` | `-q` | none | off |
  | `--help` | `-h` | none | — |
  | `--version` | `-V` | none | — |

- **FR-GLOB-002**: Every node of the command tree SHALL accept every global
  flag.

- **FR-GLOB-003**: The system SHALL list the global flags once, in the `OPTIONS`
  section of `tpl --help`, and SHALL NOT repeat them in the `OPTIONS` section of
  any other node.

  *Rationale.* Seven flags repeated across roughly thirty commands would
  multiply the very document that exists to save the caller's context window.

### `-d`, `--database`

- **FR-GLOB-004**: `-d/--database <name>` SHALL select the `[database.<name>]`
  entry of `.tpl/.cfg` to be used by the invocation.

- **FR-GLOB-005**: WHEN `-d/--database` is absent, the system SHALL use the
  entry named by `core.database` in `.tpl/.cfg`.

- **FR-GLOB-006**: IF no entry is selected — neither `-d/--database` on the
  command line nor `core.database` in the file — and the command requires one,
  THEN the system SHALL exit `78` (`EX_CONFIG`) with a message naming the file.

- **FR-GLOB-007**: IF `-d/--database` names an entry that is absent from
  `.tpl/.cfg`, THEN the system SHALL exit `66` (`EX_NOINPUT`) with a
  nearest-match suggestion over the entry names that exist.

  *Rationale.* Nothing selected is a configuration problem; a named entry that
  does not exist is a named object that does not exist, like a missing table or
  template. Calling a missing entry `64` would contradict the help, since `-d`
  is genuinely optional whenever `core.database` is set.

- **FR-GLOB-008**: The system SHALL distinguish an entry selected by
  `-d/--database` on the command line from one resolved through `core.database`,
  because `FR-RND-011` depends on that distinction.

### `--tpl-dir`

- **FR-GLOB-009**: `--tpl-dir <path>` SHALL name a `.tpl` folder explicitly and
  SHALL suppress discovery.

- **FR-GLOB-010**: A `.tpl` folder named by `--tpl-dir` SHALL be subject to
  every check defined in `FR-PROJ-007` through `FR-PROJ-009`, without exemption.

### `--timeout`

- **FR-GLOB-011**: `--timeout <seconds>` SHALL set the overall budget for the
  invocation. Its default is `30`.

- **FR-GLOB-012**: The system SHALL resolve each phase deadline strongest first:
  the `--timeout` flag, then the corresponding `[core]` key in `.tpl/.cfg`, then
  the built-in default declared in `FR-CONF-004`.

- **FR-GLOB-013**: WHEN a deadline is exceeded, the system SHALL exit with the
  code of the phase that timed out: `69` (`EX_UNAVAILABLE`) for a network phase,
  `78` for `password_command`, `65` (`EX_DATAERR`) for render.

  *Rationale.* "Never interactive" exists because a blocked process hangs the
  caller with no diagnosis. A connect to a silent address, a `password_command`
  waiting on a FIFO, and a runaway loop in a template produce that same effect,
  so the invariant is not satisfied without deadlines on every blocking phase.

### `-v`, `--verbose` and `-q`, `--quiet`

- **FR-GLOB-014**: `-v/--verbose` SHALL raise the diagnostic level written to
  stderr. One occurrence is `INFO`, two `DEBUG`, three `TRACE`; further
  occurrences saturate at `TRACE` without error.

- **FR-GLOB-015**: `-q/--quiet` SHALL lower the diagnostic level to errors only.

- **FR-GLOB-016**: Neither `-v/--verbose` nor `-q/--quiet` SHALL alter stdout in
  any way.

- **FR-GLOB-017**: At `INFO` the system SHALL report which phases ran and how
  long each took. At `DEBUG` it SHALL additionally report the catalogue queries
  issued and each cache hit and miss. At `TRACE` it MAY report internal detail.

- **FR-GLOB-018**: The system SHALL NOT write any of the following to any
  diagnostic stream, at any verbosity level: the argument vector, the resolved
  DSN, the `password_command` or its stderr, the raw driver error, or the
  contents of `.tpl/.cfg`.

### `-h`, `--help` and `-V`, `--version`

- **FR-GLOB-019**: `-h/--help` SHALL print the help of the node at which it
  appears, to stdout, and SHALL exit `0`. Its output is defined in
  [help-and-version.md](help-and-version.md).

- **FR-GLOB-020**: `-V/--version` SHALL print the version and SHALL exit `0`.
  Its output is defined in `FR-HELP-005`.

## Flags that are deliberately not global

- **FR-GLOB-021**: `--format`, `--pretty`, `--direct`, and `--no-cache` SHALL
  NOT be global. Each SHALL be declared only by the commands to which it
  applies, and SHALL be rejected as an unknown flag everywhere else, per
  `FR-CLI-019`.

  | Flag | Declared by |
  |---|---|
  | `--format <text\|json>` | `schema info`, `schema tables`, `schema table`, `schema views`, `schema view`, `schema routines`, `schema routine`, `template list`, `template path`, `cfg get`, `cfg list`, `cfg database list`, `cfg database show`, `cfg database test`, `cache status`, `help` |
  | `--pretty` | every command that declares `--format`, plus `schema dump` |
  | `--direct` | the eight `schema` subcommands, `render`, `cache load` |
  | `--no-cache` | the eight `schema` subcommands, `render`, `cache load` |

  *Rationale.* A command that does not declare a flag rejects it as unknown, so
  there is no "known but inapplicable" category to explain, and the top-level
  help never advertises a flag that half the tree refuses.

- **FR-GLOB-022**: The system SHALL list `--direct` and `--no-cache` in the
  `OPTIONS` section of the help of each command that declares them, and in that
  command's `options` array in the JSON command tree.

- **FR-GLOB-023**: The system SHALL NOT declare a flag whose name is `password`
  or whose purpose is to carry a password, at any node.

## Business rules

- **BR-GLOB-001**: The global set is small on purpose. A flag becomes global
  only when it applies to every node of the tree without exception; the moment
  one node would have to ignore it or reject it, it is local.

- **BR-GLOB-002**: No global flag changes what a command reads from the
  database. `--direct` and `--no-cache` change where catalogue data comes from,
  which is precisely why they are local.

## Dependencies

- [cli-contract.md](cli-contract.md) — the parsing rules that apply to these
  flags, including `FR-CLI-014` on repetition.
- [configuration-model.md](configuration-model.md) — the `[core]` timeout keys
  referenced by `FR-GLOB-012`.
- [cache-commands.md](cache-commands.md) — the meaning of `--direct` and
  `--no-cache`.

## Open questions

- [OQ-008](open-questions.md#oq-008) — how `--timeout` composes with the
  per-phase keys.
- [OQ-015](open-questions.md#oq-015) — the permitted position of a global flag
  on the command line.
