---
title: Project and Discovery
status: draft
last-reviewed: 2026-09-09
related: [configuration-model.md, cfg-commands.md, cache-commands.md, security.md]
---

# Project and Discovery

## Overview

`tpl` operates on projects. A project is any directory containing a `.tpl`
folder, exactly as `git` uses `.git`. That folder is the project root and the
only source of configuration and templates. This file defines its layout, how it
is found, the checks applied before it is trusted, and what `tpl init` creates.

## Scope

In scope: the `.tpl` layout, discovery and its boundary, the ownership and mode
checks, `tpl init`, and which commands may write inside `.tpl`.

Out of scope: the content of `.cfg`, and the behaviour of the commands that
maintain it.

## Actors

- **Calling agent** or **operator**.
- **Filesystem**, which supplies the directory tree the walk traverses and the
  ownership and mode bits the checks read.

## Layout

```
<project root>/
└── .tpl/
    ├── .cfg              TOML — project options and database entries; not versioned
    ├── .cache/           read catalogue, one folder per entry; not versioned
    ├── .gitignore        excludes .cfg and .cache/
    └── templates/        the project's templates; versioned
        ├── example.jinja
        └── rust/
            └── _types.jinja
```

- **FR-PROJ-001**: A project SHALL be any directory containing a `.tpl` folder.
  That folder is the project root.

- **FR-PROJ-002**: `.tpl` SHALL contain exactly the artefacts above. `.cache/`
  is absent until the first read populates it.

- **FR-PROJ-003**: `.tpl/templates/` SHALL be versioned with the repository, and
  `.tpl/.cfg` and `.tpl/.cache/` SHALL NOT be.

  *Rationale.* `templates/` is shared team work and should be reviewed like any
  other code. `.cfg` is per-machine access configuration and may hold
  credentials. `.cache/` is per-machine state derived from a server.

## Discovery

- **FR-PROJ-004**: The system SHALL locate the project by walking up from the
  current directory until it finds a `.tpl` folder. The first one found is the
  project root, and the walk stops there.

- **FR-PROJ-005**: The upward walk SHALL stop at the user's home directory and
  at the filesystem mount point. A `.tpl` folder above either boundary SHALL NOT
  be considered.

  *Rationale.* A `.tpl` planted in a world-writable ancestor — `/tmp`,
  `/var/tmp`, a mounted share — must not be able to supply the configuration.
  The precedent is the `safe.directory` gate git added for CVE-2022-24765.

- **FR-PROJ-006**: IF no `.tpl` folder is found within the boundary, THEN the
  system SHALL exit `78` (`EX_CONFIG`) and the hint SHALL suggest `tpl init`.

- **FR-PROJ-007**: There SHALL be no fallback outside the project. A command
  that finds no `.tpl` fails; it does not read settings from anywhere else.

- **FR-PROJ-008**: `--tpl-dir <path>` SHALL name a `.tpl` folder explicitly and
  SHALL suppress the walk. It SHALL be subject to every check below without
  exemption.

## Trust checks

- **FR-PROJ-009**: The system SHALL canonicalise the resolved `.tpl` path before
  applying any check, so that a symlinked `.tpl` is verified at its real target.

- **FR-PROJ-010**: `.tpl/.cfg` SHALL be owned by the current user. IF it is not,
  THEN the system SHALL exit `78`.

- **FR-PROJ-011**: `.tpl/.cfg` SHALL carry no group and no other access bits. IF
  it does, THEN the system SHALL exit `78`:

  ```
  error: .tpl/.cfg has unsafe permissions
  cause: mode 0644; group and other must have no access
  hint:  chmod 600 .tpl/.cfg
  exit:  78 (EX_CONFIG)
  ```

  *Rationale.* A `.cfg` writable by anyone else can choose the
  `password_command` that runs with the caller's privileges.

  *Accepted cost.* A `.cfg` shared across a team through a unix group stops
  working.

## `tpl init`

```
tpl init [<path>]
```

- **FR-PROJ-012**: `tpl init` SHALL take an optional positional path, defaulting
  to the current directory.

- **FR-PROJ-013**: `tpl init` SHALL create the destination directory, including
  any missing parent directories.

- **FR-PROJ-014**: IF a `.tpl` folder already exists at the destination, THEN
  the system SHALL exit `73` (`EX_CANTCREAT`) and SHALL change nothing. It does
  not merge, complete partially, or overwrite.

- **FR-PROJ-015**: IF the destination cannot be created, THEN the system SHALL
  exit `73`.

- **FR-PROJ-016**: WHEN a `.tpl` folder exists in an ancestor directory but not
  at the destination, the system SHALL create the nested project, SHALL write a
  warning to stderr saying that it will shadow the one above, and SHALL exit
  `0`.

  *Known weakness, accepted.* A caller checking only the exit code will not see
  the warning.

- **FR-PROJ-017**: `tpl init` SHALL create exactly five artefacts:

  | Artefact | Contents |
  |---|---|
  | `.tpl/.cfg` | Mode `0600`; a `[core]` section and one commented-out `[database.*]` entry showing the exact shape a real entry takes |
  | `.tpl/.gitignore` | Two lines: `.cfg` and `.cache/` |
  | `.tpl/templates/` | The project's template directory |
  | `.tpl/templates/example.jinja` | A working example template |
  | `.tpl/templates/rust/_types.jinja` | A macro file mapping a column to a Rust type |

  *Amended in the second edition.* The fifth artefact is new. `FR-ENV-009`
  removes the `rust_type` filter from the binary, and `FR-ENV-011` delivers the
  mapping it performed as a template macro instead, so that the opinion it
  encodes — whether `DECIMAL` becomes a third-party decimal type, an `f64`, or a
  `String` — belongs to the project and can be edited there.

- **FR-PROJ-018**: The generated `.cfg` SHALL contain no active database entry.
  A fresh project knows about no database until one is added.

  *Rationale.* The commented example exists so the correct shape is in front of
  the reader without a trip to documentation.

- **FR-PROJ-019**: `tpl init` SHALL create `.tpl/.cfg` with mode `0600`.

- **FR-PROJ-020**: `tpl init` SHALL NOT create `.tpl/.cache/`.

- **FR-PROJ-021**: `.tpl/templates/example.jinja` SHALL render without error
  against any table of any supported MariaDB database, SHALL walk the columns of
  that table, SHALL use at least one filter and at least one test, and SHALL
  carry in its header the command that runs it.

- **FR-PROJ-022**: `tpl init` SHALL write nothing to stdout and SHALL exit `0`,
  per `BR-CLI-004`.

## Writers of `.tpl`

- **FR-PROJ-023**: The following, and nothing else, SHALL write inside `.tpl`:

  | Writer | What it writes |
  |---|---|
  | `tpl init` | The five artefacts of `FR-PROJ-017` |
  | `tpl cfg …` | `.tpl/.cfg` |
  | `tpl cache load` | `.tpl/.cache/` |
  | any cached read command, on a miss | `.tpl/.cache/` |

- **FR-PROJ-024**: The system SHALL NOT create, modify, or delete any file
  outside `.tpl`, under any circumstance.

- **BR-PROJ-001**: What `tpl` does is fully determined by the contents of the
  project, which is what makes its behaviour reproducible between machines and
  in CI.

- **BR-PROJ-002**: A read command writes to `.tpl/.cache/` on a miss, so no read
  command is read-only with respect to the filesystem unless it is invoked with
  `--direct --no-cache`.

## Dependencies

- [configuration-model.md](configuration-model.md) — what `.cfg` contains.
- [cache-commands.md](cache-commands.md) — what `.cache/` contains and who
  writes it.
- [template-commands.md](template-commands.md) — the template root and its
  containment rules.

## Open questions

None specific to this module.
