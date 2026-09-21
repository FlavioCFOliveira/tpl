---
title: Project and Discovery
status: approved
last-reviewed: 2026-09-21
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

- **FR-PROJ-002**: `.tpl` SHALL contain exactly the artefacts above and nothing
  else that `tpl` reads or writes. `.cache/` is absent until the first read
  populates it, and `templates/` holds the project's own templates alongside
  the two `tpl init` writes, per `FR-TMPL-004`.

- **FR-PROJ-003**: `.tpl/templates/` SHALL be versioned with the repository, and
  `.tpl/.cfg` and `.tpl/.cache/` SHALL NOT be.

  *Rationale.* `templates/` is shared team work and should be reviewed like any
  other code. `.cfg` is per-machine access configuration and may hold
  credentials. `.cache/` is per-machine state derived from a server.

## Discovery

- **FR-PROJ-004**: The system SHALL locate the project by walking up from the
  current directory until it finds a `.tpl` folder. The first one found is the
  project root, and the walk stops there.

- **FR-PROJ-005**: The upward walk SHALL stop at the mount point of the
  filesystem that contains the directory the walk starts from. A `.tpl` folder
  above that boundary SHALL NOT be considered. The boundary SHALL be
  determined without reading any environment variable.

  *Amended in the eighth edition, and this requirement is the one that
  yields.* The boundary at the user's home directory is removed. It could only
  be located from `HOME`, which is an environment variable read to determine
  the location of the project — precisely what `FR-CLI-021` prohibits — and it
  made the boundary a value a shell can set. Two identical command lines run
  in two different shells could therefore discover two different projects and
  read two different databases, which falsifies `BR-CLI-002` in its own terms.
  One boundary of one walk yields to two invariants of the whole surface
  rather than the reverse: `FR-CLI-021` admits exactly one environment read,
  the `${VAR}` expansion of `FR-CLI-023`, and `BR-CLI-002` is the property
  that makes an invocation reproducible between machines and in CI, per
  `BR-PROJ-001`. The mount point needs no environment at all — it is a
  comparison between a directory and its parent — so the boundary that
  remains is the one that was already free of this defect.

  *Rationale.* A `.tpl` in an ancestor `tpl` has no reason to trust must not
  be able to supply the configuration. The mount point stops a walk that
  begins inside a mounted share from climbing out of it. The precedent is the
  `safe.directory` gate git added for CVE-2022-24765.

  *What defends the rest, and it was never this boundary.* The first edition
  named three examples, and a boundary of the walk reaches only one of them. A
  `.tpl` in `/tmp` or in `/var/tmp` is reached by a walk that starts beneath it
  long before that walk reaches any mount point, and the home directory never
  lay between the two either. What refuses such a project is the trust checks:
  `FR-PROJ-009` canonicalises the resolved path, `FR-PROJ-010` requires
  `.tpl/.cfg` to be owned by the current user, and `FR-PROJ-011` requires it
  to carry no group and no other access bits. All three apply to every project
  without exemption, including one named by `--tpl-dir`, per `FR-PROJ-008`,
  and together they give a planted `.cfg` a `78`.

  *Rejected.* Keeping the home boundary and locating it from the system's own
  account record for the invoking user rather than from `HOME`. That satisfies
  `BR-CLI-002` and would have kept the boundary, and it was rejected because
  the boundary would then sit wherever that record says, which need not be an
  ancestor of the caller's working directory — so the rule would silently
  never fire, and a rule that cannot be observed to fire is the defect
  `FR-PRIV-011` was corrected for. Also rejected: excepting discovery from
  `FR-CLI-021`, which leaves `BR-CLI-002` false and moves the contradiction
  instead of resolving it.

  *Accepted cost.* A `.tpl` folder above the caller's home directory and on
  the same filesystem — at `/home`, at `/Users`, or at `/` — is now within
  the walk, where the home boundary excluded it. Three things bound the cost.
  Such a directory is not ordinarily writable by the caller, so a `.tpl` there
  is either the caller's own or is refused by `FR-PROJ-010`. `tpl init` writes
  a warning to stderr when it creates a project that shadows one above it, per
  `FR-PROJ-016`. And `FR-PROJ-007` still admits no fallback: a walk that
  reaches the boundary without finding a `.tpl` fails with `78`, per
  `FR-PROJ-006`, rather than reading settings from anywhere else.

- **FR-PROJ-006**: IF a command that requires a project finds no `.tpl` folder
  within the boundary, THEN the system SHALL exit `78` (`EX_CONFIG`) and the
  hint SHALL suggest `tpl init`.

  *Amended in the third edition.* The qualifier "that requires a project" is
  new. As first written this requirement made a missing `.tpl` a `78` for every
  command, including `tpl init`, which creates the folder, and
  `tpl help --format json`, which exists to be callable before anything else
  does. `FR-PROJ-025` names the commands it does not reach.

- **FR-PROJ-025**: The following commands, and no others, SHALL NOT require a
  project and SHALL NOT perform discovery:

  | Command | Why |
  |---|---|
  | `tpl init` | It creates `.tpl`, so it cannot require one to be found first |
  | `tpl help`, `tpl help <command>`, `tpl help --format json` | The command tree is derived from the binary, per `FR-HELP-021` |
  | `-h/--help` at any node | Byte-identical to `tpl help <node>`, per `FR-HELP-002` |
  | `tpl version`, `-V/--version` | It prints a constant, per `FR-HELP-005` |

  Every other command SHALL perform discovery, and SHALL fail with `78` per
  `FR-PROJ-006` when it finds no project.

  **What "SHALL NOT perform discovery" forbids.** No project above the
  invocation SHALL decide its outcome: none is required for the command to
  succeed, none supplies configuration to it, none selects a database entry for
  it, and none decides where `tpl init` creates what it creates. The walk of
  `FR-PROJ-004` is the mechanism this clause exists to keep out of those four
  decisions, and it is those decisions the clause governs.

  **One of the four commands looks upward, and it is obliged to.**
  `FR-PROJ-016` requires `tpl init` to warn that the project it is about to
  create shadows one in an ancestor directory, and the warning cannot be
  written without looking for that ancestor. The look decides nothing this
  clause protects: the destination is `FR-PROJ-012`'s argument or the current
  directory, the five artefacts are `FR-PROJ-017`'s, the exit code is `0` per
  `FR-PROJ-022`, and an ancestor that is found or not found changes none of
  them. What it adds is one line on stderr. No other command of this table
  looks upward at all.

  *Amended in the thirty-first edition: the clause says what it forbids,
  because read as "no ancestor is looked at" it contradicted a requirement in
  force.* `FR-PROJ-016` has required the shadowing warning since the first
  edition, and `FR-PROJ-005`'s accepted cost names that warning as one of the
  three things that bound the cost of removing the home boundary — so the
  warning is load-bearing and cannot yield. Read literally this clause forbade
  the only way to produce it, and `NFR-PERF-005` turned the literal reading
  into an observable ("no `stat` of an ancestor directory") while
  `NFR-PERF-007` wrote a differential arrangement that asserts the warning is
  not emitted. Three requirements said three things and the code did the
  fourth. This clause is the one that yields, because it is the only one of the
  three that was reaching for something other than what it said: what a command
  that requires no project must not do is depend on one, and looking for a
  project in order to warn about it is not depending on it.

  *Observed, 2026-09-18.* `src/project/init.rs` calls `discover::locate` on the
  destination before `.tpl` exists and emits the warning where the walk finds
  an ancestor. Run inside `sub/deep` of an existing project, `tpl init` printed
  `warning: the project created at ./.tpl shadows the project at
  /private/tmp/.../.tpl` and exited `0`. It is reproduced by `tpl init`, then
  `mkdir -p sub/deep && cd sub/deep && tpl init`. The walk it makes reads no
  `.tpl/.cfg` and applies none of the trust checks of `FR-PROJ-009` through
  `FR-PROJ-011`, so the configuration clause of `NFR-PERF-005` is untouched by
  it and only the discovery clause was ever at issue.

  *Rejected: removing `tpl init` from this table for the discovery clause,
  keeping it for the rest.* It splits one table into two sets with different
  members and obliges every reader of it to ask which set a row is in.
  `tpl init` belongs here for the plainer half, stated below, and the half it
  would be removed from is the half this amendment makes true of it.

  *Rejected: withdrawing `FR-PROJ-016`, so that the clause can stand
  literally.* It reopens `FR-PROJ-005`. That requirement removed the home
  boundary and bounded the cost with three things, one of which is this
  warning; withdrawing it would leave the accepted cost naming a protection
  that no longer exists, and a nested project created inside an inherited one
  would be created in silence.

  *Rejected: emitting the warning without a walk, by looking only at the
  immediate parent.* It is cheaper and it is wrong more often than it is right:
  a project three directories up shadows just as completely as one directory
  up, and a warning that fires only sometimes is worse than none, because a
  caller who has seen it work reads its absence as an answer.

  *Rationale.* A calling agent's first invocation is `tpl help --format json`,
  which is how it loads the whole surface, per `FR-HELP-016`. Failing it with a
  code that says "fix `.tpl/.cfg` or run `tpl init`" before the agent has
  learned that `tpl init` exists is the worst outcome the exit-code table can
  produce. `tpl init` is on the list for the plainer reason that requiring a
  project in order to create one cannot work.

  *Rejected.* Running discovery for every command and making `tpl --help`
  outside a project a `78`. Also rejected: running discovery but treating its
  failure as non-fatal for the four, which adds a third state — discovered,
  absent, absent-but-tolerated — to a model that has two.

  *Accepted cost.* `tpl template list` and every other command still fail
  outside a project, so the four are a stated exception rather than a rule a
  caller can generalise. The list is in `tpl --help`, and each of the four
  omits `78` from its own `EXIT CODES` section, per `FR-HELP-011`.

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

  *Accepted cost.* A caller checking only the exit code will not see the
  warning.

  *Checked in the thirty-first edition against the two requirements that
  contradicted it, and unchanged.* This requirement obliges a look at the
  ancestors of the destination, and `FR-PROJ-025` forbade every command it
  names from performing discovery while naming `tpl init`. `FR-PROJ-025` is
  the requirement that yields, and it now states what its discovery clause
  forbids: no project above the invocation decides its outcome. This one is
  untouched — the warning, its stream and the `0` are as the first edition
  wrote them — and `NFR-PERF-005` and `NFR-PERF-007` follow `FR-PROJ-025` in
  the same edition.

  *Observed, 2026-09-18, and the observation is recorded here as well because
  this is the requirement it satisfies.* `src/project/init.rs:185` calls
  `discover::locate(None, destination)` and line 190 calls
  `emit::project_shadows_ancestor`. Run inside `sub/deep` of an existing
  project, `tpl init` printed
  `warning: the project created at ./.tpl shadows the project at
  /private/tmp/.../.tpl` and exited `0`. Reproduced by `tpl init`, then
  `mkdir -p sub/deep && cd sub/deep && tpl init`.

  *Consequence, stated plainly.* `tpl init` is the one command of
  `FR-PROJ-025` whose syscalls include a walk over its ancestors, so the
  file-open observation of `NFR-PERF-007` sees `stat` calls above the
  destination for it and for neither of the other three. `NFR-PERF-005` states
  that per command rather than leaving a reader to infer it from a clause
  written over the whole table.

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
  | `tpl init` | The five artefacts of `FR-PROJ-017`, and the destination directory and its missing parents, per `FR-PROJ-013` |
  | `tpl cfg …` | `.tpl/.cfg` |
  | `tpl cache load` | `.tpl/.cache/` |
  | any cached read command, on a miss | `.tpl/.cache/` |

- **FR-PROJ-024**: The system SHALL NOT create, modify, or delete any file
  outside `.tpl`, with exactly one exception: the destination directory of
  `tpl init` and its missing parent directories, per `FR-PROJ-013`.

  *Amended in the third edition.* The first edition admitted no exception,
  which contradicted `FR-PROJ-013` outright: `tpl init a/b/c` cannot create
  `a/b/c` without writing outside `.tpl`. The exception is enumerated rather
  than the prohibition weakened, so that the invariant still reads as an
  invariant.

  *Accepted cost.* `tpl init` is the one command that can leave a directory
  behind on a mistyped path. It creates directories only, never a file outside
  `.tpl`, and it writes nothing at all if `.tpl` already exists at the
  destination, per `FR-PROJ-014`.

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
