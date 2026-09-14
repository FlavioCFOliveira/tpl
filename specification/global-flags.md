---
title: Global Flags
status: approved
last-reviewed: 2026-09-14
related: [cli-contract.md, configuration-model.md, cache-commands.md, output-formats.md]
---

# Global Flags

## Overview

`tpl` has exactly seven global flags. Every other flag is local: it is declared
by the commands to which it applies, and rejected as unknown everywhere else.
This file defines the seven, their values and defaults, and the precedence by
which a setting is resolved.

## Scope

In scope: the global flag set, each flag's meaning and default, the complete
short-flag set of the tool, the precedence rule, and the list of flags that are
deliberately not global.

Out of scope: the behaviour each flag triggers inside a particular command,
which belongs to that command's module.

## Actors

- **Calling agent** or **operator**, supplying the flags on the command line.
- **Project**, whose `.tpl/.cfg` supplies the layer beneath them.

## The seven global flags

- **FR-GLOB-001**: The system SHALL declare exactly seven global flags, and no
  others:

  | Flag | Short | Value | Default |
  |---|---|---|---|
  | `--database <name>` | `-d` | database entry name | none; see `FR-GLOB-004` |
  | `--tpl-dir <path>` | | filesystem path | none; discovery applies |
  | `--timeout <seconds>` | | positive integer | none; see `FR-GLOB-011` |
  | `--verbose` | `-v` | none, repeatable | off |
  | `--quiet` | `-q` | none | off |
  | `--help` | `-h` | none | — |
  | `--version` | `-V` | none | — |

- **FR-GLOB-002**: Every node of the command tree SHALL accept every global
  flag, in any position the command line admits, per `FR-CLI-024`.

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
  because `FR-RND-018` and `FR-RND-019` depend on that distinction.

### `--tpl-dir`

- **FR-GLOB-009**: `--tpl-dir <path>` SHALL name a `.tpl` folder explicitly and
  SHALL suppress discovery.

- **FR-GLOB-010**: A `.tpl` folder named by `--tpl-dir` SHALL be subject to
  every trust check defined in `FR-PROJ-009` through `FR-PROJ-011`, without
  exemption.

### `--timeout`

- **FR-GLOB-011**: `--timeout <seconds>` SHALL set the overall wall-clock
  budget for the invocation, measured from process start. It SHALL have no
  default: WHEN it is absent, the invocation carries no overall budget and is
  bounded only by the per-phase deadlines of `FR-CONF-005`.

  *Amended in the third edition.* The first edition gave the flag the default
  `30` while `FR-CONF-004` made a flag stronger than a `[core]` key. A flag
  that always has a value always wins, so `core.connect_timeout`,
  `core.query_timeout`, `core.password_timeout`, and `core.render_timeout` were
  dead keys that could never take effect. Removing the default makes the flag
  absent unless supplied, and `FR-GLOB-012` composes it with the per-phase
  deadlines rather than overriding them.

  *Rejected.* A precedence rule distinguishing a supplied flag from a defaulted
  one, which would have to be explained in every help text that mentions a
  timeout; and dropping the four `[core]` keys for a single timeout, which
  would give a slow catalogue query no more time than a DNS lookup.

- **FR-GLOB-012**: `--timeout` SHALL compose with the per-phase deadlines of
  `FR-CONF-005` rather than replace them. A phase SHALL end when the first of
  the two expires: its own deadline, resolved by `FR-CONF-004`, or what remains
  of the overall budget.

  *Amended in the third edition.* The first edition made `--timeout` the
  strongest layer of a per-phase precedence, which is not what a flag
  documented as "the overall budget for the invocation" can be. The two are now
  bounds of different kinds and both apply.

  *Accepted cost.* `tpl --timeout 1 …` can fail inside a phase whose own
  deadline is larger, and the exit code will name that phase rather than the
  flag. `FR-GLOB-013` fixes which code, and the `cause` line of `FR-ERR-010`
  states which of the two bounds expired.

- **FR-GLOB-013**: WHEN a deadline is exceeded, the system SHALL exit with the
  code of the phase that timed out: `69` (`EX_UNAVAILABLE`) for a network phase,
  `78` for `password_command`, `65` (`EX_DATAERR`) for render. WHEN the overall
  budget of `FR-GLOB-011` expires, the system SHALL exit with the code of the
  phase that was in progress, by the same table.

  *Rationale.* "Never interactive" exists because a blocked process hangs the
  caller with no diagnosis. A connect to a silent address, a `password_command`
  waiting on a FIFO, and a runaway loop in a template produce that same effect,
  so the invariant is not satisfied without deadlines on every blocking phase.

### `-v`, `--verbose` and `-q`, `--quiet`

- **FR-GLOB-014**: `-v/--verbose` SHALL raise the diagnostic level written to
  stderr. One occurrence is `INFO`, two `DEBUG`, three `TRACE`; further
  occurrences saturate at `TRACE` without error.

- **FR-GLOB-015**: `-q/--quiet` SHALL lower the diagnostic level to errors
  only. IF `-v/--verbose` is given in the same invocation, THEN the system
  SHALL exit `64` (`EX_USAGE`), per `FR-CLI-015`.

  *Amended in the twentieth edition: the pair is named where both flags are
  declared.* `FR-CLI-015` has refused `-q` together with `-v` with `64` since
  the first edition, among the parsing rules of
  [cli-contract.md](cli-contract.md). This file declares both flags, and until
  now declared neither against the other; it is where an implementer resolving
  `-v -q` reads: one did, found neither an order nor an exclusion here, and
  resolved the pair by choosing — letting quiet win — which is a behaviour this
  corpus does not have and has never had. The clause is a cross-reference and
  not a second rule. `FR-CLI-015` owns the refusal, as the parsing rule it is,
  and nothing about `tpl` changes.

  *Rejected.* Stating a precedence here — quiet over verbose, or last-wins.
  Either would contradict a requirement in force rather than settle anything,
  and the ground `FR-CLI-014` gives for refusing a repeated single-value flag
  is exactly the ground here: a result that depends on how a script grew.

- **FR-GLOB-016**: Neither `-v/--verbose` nor `-q/--quiet` SHALL alter stdout in
  any way.

- **FR-GLOB-017**: At `INFO` the system SHALL report which phases ran and how
  long each took, and SHALL write exactly one line per catalogue query it
  issues, in a form distinguishable from every other diagnostic line. At `DEBUG`
  it SHALL additionally report each cache hit and miss. At `TRACE` it MAY report
  internal detail.

  *Amended in the second edition.* The one-line-per-query rule was raised from
  `DEBUG` to `INFO` and made distinguishable so that the catalogue-query count
  is observable from outside the process. Without it, `NFR-PERF-001` and
  `NFR-PERF-002` — the requirements that forbid a query count growing with the
  number of objects — could not be checked at all. The rule constrains the
  existence and the distinguishability of the line, not its wording; the
  diagnostic stream remains outside the contract, per `NFR-DET-001`.

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

- **FR-GLOB-024**: The five short forms declared by `FR-GLOB-001` — `-d`, `-v`,
  `-q`, `-h`, and `-V` — SHALL be the complete short-flag set of the tool. No
  other flag, global or local, at any node, SHALL declare a short form.
  `--tpl-dir` and `--timeout` are the two global flags that have none, and
  SHALL NOT acquire one.

  *Rationale.* A short form is frozen the moment it ships: it cannot be
  renamed, and it cannot be reassigned to another flag without silently
  changing what an existing invocation does. Reserving the whole one-letter
  space for the seven flags that every node accepts means a short form always
  means the same thing wherever it appears, which is what makes it safe to
  read. It is also the argument `FR-CLI-012` already makes for refusing a
  one-letter top-level alias.

  *Closes* `OQ-016`, now listed under [Closed](open-questions.md#closed). The
  root `README.md` declares `-s` for `--set` on `tpl render`, and `-H`, `-P`
  and `-u` for the entry flags of `tpl cfg database add` and `update`. None of
  the four exists; `DIV-003` records the correction owed, alongside the `-p` it
  already covered.

  *Accepted cost.* Every local flag is typed in full. `tpl cfg database add
  shop --host db.example.com --port 3306 --user alice` is longer than its
  short-form equivalent, and an operator typing it at a prompt pays for the
  guarantee. The primary consumer is a calling agent, per
  [cli-contract.md](cli-contract.md), which generates the line rather than
  typing it and reads the long form more reliably than the short one.

  *Rejected.* `-H`, `-P` and `-u` on the entry flags, which would put `-P` for
  `--port` one shift key away from a `-p` that `FR-CFG-030` refuses to declare
  for a password — a collision by case alone on the one flag pair where getting
  it wrong writes a credential into the process table. Also rejected: `-s` for
  `--set`, which reads as "string", "set", or "schema" depending on which
  neighbouring tool the caller last used.

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

None specific to this module. `OQ-015` is answered by `FR-CLI-024` and `OQ-016`
by `FR-GLOB-024`; both are listed under [Closed](open-questions.md#closed).
