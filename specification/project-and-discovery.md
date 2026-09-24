---
title: Project and Discovery
status: approved
last-reviewed: 2026-09-24
related: [configuration-model.md, cfg-commands.md, cache-commands.md, security.md, errors-and-exit-codes.md, global-flags.md, help-and-version.md]
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

  *Note added in the forty-eighth edition.* `.cfg` may also be absent, which
  is the state of every clone under `FR-PROJ-003`. `FR-PROJ-028` states what
  such a project is.

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

  IF a command that requires a project is given a `--tpl-dir` path that does
  not exist or is not a directory, THEN the system SHALL exit `78`
  (`EX_CONFIG`). The `error` and `cause` lines SHALL name the path and state
  why it is not usable, and the `cause` SHALL state that `--tpl-dir` disabled
  the upward search, per the `78` row of `FR-ERR-034`. Neither line SHALL
  describe a walk. The `hint` SHALL name `--tpl-dir` as the value to correct,
  and SHALL NOT suggest a `tpl init` that creates a project anywhere but at
  the path named: WHERE the path's last segment is `.tpl`, the `hint` SHALL
  carry `tpl init` with the path's parent directory, built under
  `FR-ERR-041`.

  ```
  error: the folder named by --tpl-dir does not exist: /srv/shop/.tpl
  cause: --tpl-dir disabled the upward search; nothing exists at /srv/shop/.tpl
  hint:  correct --tpl-dir, or create the project with: tpl init /srv/shop
  exit:  78 (EX_CONFIG)
  ```

  *Amended in the forty-third edition.* The requirement did not say what a
  `--tpl-dir` naming nothing produces, and the implementation reported
  `the walk upward ended at /nonexistent/.tpl`, a walk the flag had
  suppressed, with a hint to run `tpl init` in the current directory, per
  finding E-03 of the audit of rmp `#259`. `FR-PROJ-006` still governs the
  failed walk, and this clause governs the case with no walk. The code is
  unchanged: `78`, as for a project that is not found.

  *Note added in the forty-eighth edition.* A path that exists and is a
  directory is not thereby a `.tpl` folder. `FR-PROJ-027` states which
  directories are, and what a `--tpl-dir` naming any other one produces.

- **FR-PROJ-027**: A directory SHALL be usable as the `.tpl` folder of a
  project only WHERE the last segment of its path is `.tpl`, either as the
  path is written or after the canonicalisation of `FR-PROJ-009`. The segment
  SHALL be compared with `.tpl` without regard to the case of ASCII letters,
  so `.TPL` and `.Tpl` match. The walk of
  `FR-PROJ-004` finds only such directories, so this requirement is observable
  only through `--tpl-dir`.

  IF a command that requires a project is given a `--tpl-dir` path that names
  an existing directory whose last segment is `.tpl` in neither form, THEN the
  system SHALL exit `78` (`EX_CONFIG`), and SHALL read no configuration and no
  template from that directory and write nothing into it. The message SHALL
  be as follows:

  1. **`error`** SHALL name the path as written, and SHALL state that it is
     not a `.tpl` folder.
  2. **`cause`** SHALL state that `--tpl-dir` disabled the upward search, per
     the `78` row of `FR-ERR-034`, and SHALL state one of two facts. WHERE the
     directory holds a directory named `.tpl`, it SHALL state that it does.
     Otherwise it SHALL state that `--tpl-dir` names the `.tpl` folder of a
     project, not the directory that holds it.
  3. **`hint`**, WHERE the directory holds a directory named `.tpl`, SHALL
     name the corrected value: the path as written, followed by `/.tpl`. WHERE
     the invocation carried no positional operand, the `hint` SHALL carry the
     command `tpl --tpl-dir <corrected value>` followed by the command path of
     the invocation, the nodes from `tpl` to the leaf, with `-d` carried under
     `FR-ERR-043` and no other flag. WHERE it carried an operand, the `hint`
     SHALL state in words that the same invocation is to be run again with the
     corrected value, and SHALL NOT reproduce the operand. The corrected value
     is built under `FR-ERR-041`; IF that set refuses it, THEN the placeholder
     rule of `FR-ERR-043` applies.
  4. **`hint`**, otherwise, SHALL name `--tpl-dir` as the value to correct and
     SHALL show the form `--tpl-dir <project>/.tpl`. It SHALL NOT suggest
     `tpl init`, per `FR-PROJ-008`.

  ```
  tpl --tpl-dir ../shop cfg database list
  error: --tpl-dir names ../shop, which is not a .tpl folder
  cause: --tpl-dir disabled the upward search; ../shop holds a .tpl folder, and --tpl-dir must name that folder
  hint:  name the .tpl folder itself: tpl --tpl-dir ../shop/.tpl cfg database list
  exit:  78 (EX_CONFIG)

  tpl --tpl-dir /tmp/empty template list
  error: --tpl-dir names /tmp/empty, which is not a .tpl folder
  cause: --tpl-dir disabled the upward search; --tpl-dir names the .tpl folder of a project, not the directory that holds it, and /tmp/empty holds none
  hint:  correct --tpl-dir to name a project's .tpl folder, as in --tpl-dir <project>/.tpl
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The examples fix the facts named, the command carried
  and the code.

  The condition is evaluated at step 2 of `FR-ERR-006`, after the checks of
  `FR-PROJ-008` and the canonicalisation of `FR-PROJ-009`, and before the
  ownership and mode checks of `FR-PROJ-010`, `FR-PROJ-011` and `FR-PROJ-028`.

  *Rationale.* Naming the project directory instead of its `.tpl` folder is
  the most likely mistake with this flag. Before this requirement the system
  accepted any existing directory: `tpl cfg database list` and
  `tpl template list` reported an empty project and exited `0`, and
  `tpl cfg database add` wrote a `.cfg` beside the real `.tpl`, where no later
  command reads it. This is finding V-01 of the fifth re-audit, recorded for
  rmp `#265`.

  *Why the name, and not the contents.* A `.tpl` folder need not hold `.cfg`,
  per `FR-PROJ-028`, and need not hold `templates/`, so no child identifies
  it. Its name is what the walk looks for, and a test on the name makes
  `--tpl-dir` accept exactly what discovery accepts. The canonical form is
  admitted so that a symbolic link to a `.tpl` folder is accepted; the written
  form is admitted so that a `.tpl` that is itself a link is accepted, as the
  walk accepts it.

  *Rejected: finding a `.tpl` folder inside the named directory and using
  it.* It gives the flag two meanings, and a caller that named the wrong
  directory by mistake would act on a project it did not name. Also rejected:
  testing for `.cfg`, which refuses every clone.

  *Accepted cost.* A `.tpl` folder reached through a link whose name and
  target name are both something other than `.tpl` is refused. The caller
  names the target instead.

  *Added in the forty-eighth edition,* for rmp `#265`.

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
  hint:  chmod 600 /home/ana/shop/.tpl/.cfg
  exit:  78 (EX_CONFIG)
  ```

  The `hint` SHALL carry the absolute path of the file, built under
  `FR-ERR-041`, so that it succeeds from any directory, per `BR-ERR-004`. The
  same holds for the `hint` of `FR-PROJ-010`.

  *Amended in the forty-third edition.* The example showed the relative form
  `chmod 600 .tpl/.cfg`, which fails when run from a subdirectory of the
  project, per finding E-22 of the audit of rmp `#259`.

  *Rationale.* A `.cfg` writable by anyone else can choose the
  `password_command` that runs with the caller's privileges.

  *Accepted cost.* A `.cfg` shared across a team through a unix group stops
  working.

  *Note added in the forty-ninth edition.* This check reads the group and
  other bits only. Every mode that clears all six passes it, whatever the
  owner bits: `0600`, `0400` and `0700` alike. A file the owner cannot read
  then fails its read with `74` (`EX_IOERR`), per `FR-ERR-001`. The `cause`
  SHALL name the mode found and SHALL state that group and other must have no
  access. It SHALL NOT state that `0600` is the only mode accepted. A `cfg`
  command that rewrites the file leaves it at `0600`, per `FR-CFG-034`. The
  message stated "only at mode 0600" while a file at `0400` was read, per
  finding W-08 of the sixth re-audit, recorded for rmp `#281`.

- **FR-PROJ-028**: WHERE the project's `.tpl` folder holds no `.cfg`, the
  system SHALL use the project with an empty configuration, and SHALL NOT
  refuse it for the absence:

  1. **Reading.** Step 3 of `FR-ERR-006` SHALL pass. Every key of
     `FR-CONF-002` takes its default, and no database entry is declared, so a
     command that requires an entry fails at step 5 as it does for a `.cfg`
     that declares none.
  2. **Trust.** `FR-PROJ-010` and `FR-PROJ-011` have no file to check. The
     `.tpl` folder itself SHALL instead be owned by the current user, checked
     at its canonical path. IF it is not, THEN the system SHALL exit `78`
     (`EX_CONFIG`). The `cause` SHALL name the folder, state that it holds no
     `.cfg`, and state that it is owned by another user. The `hint` SHALL name
     `--tpl-dir` as the way to name the caller's own project.
  3. **Writing.** A `cfg` command that writes `.tpl/.cfg`, per `FR-PROJ-023`,
     SHALL create the file at mode `0600` by the procedure of `FR-CFG-041`,
     and SHALL write only what the command sets. A `cfg` command that names a
     key or an entry the absent file cannot hold — `tpl cfg unset`,
     `tpl cfg database update` or `tpl cfg database remove` — SHALL end as it
     ends for a `.cfg` that does not hold it, and SHALL create nothing.

  ```
  error: the .tpl folder at /tmp/.tpl cannot be used as a project
  cause: /tmp/.tpl holds no .cfg, and the folder is owned by another user
  hint:  name your own project's .tpl folder with: tpl --tpl-dir <project>/.tpl <command>
  exit:  78 (EX_CONFIG)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named and the code.

  *Rationale.* `FR-PROJ-003` keeps `.cfg` out of version control, so every
  clone of a repository holds a `.tpl` folder with `templates/` and no `.cfg`.
  `tpl init` refuses such a folder with `73`, per `FR-PROJ-014`. Refusing the
  project for the absence would leave the caller no `tpl` command that makes
  it usable. With an empty configuration, the templates are listed, shown and
  checked at once, `tpl render --context` renders, and the first
  `tpl cfg database add` creates the file. This pins the behaviour observed by
  finding V-01 of the fifth re-audit, recorded for rmp `#265`.

  *Why the folder's owner.* `FR-SEC-013` states that a `.tpl` planted in a
  world-writable ancestor such as `/tmp` is refused by the checks on `.cfg`. A
  planted folder without `.cfg` passes both of them, and would then supply
  templates to the caller and receive the `.cfg` that the caller's next
  `tpl cfg` command writes, credentials included. The owner check restores
  the refusal for exactly the case the two checks cannot reach. A clone is
  owned by the user who made it, so the check refuses none.

  *Rejected: exiting `78` for an absent `.cfg`.* It refuses every clone, and
  its `hint` could name only a shell command. Also rejected: letting
  `tpl init` complete a `.tpl` folder without `.cfg`, which `FR-PROJ-014`
  forbids. Also rejected: checking the owner of the folder for every project,
  which adds a refusal where `FR-PROJ-010` already establishes trust.

  *Added in the forty-eighth edition,* for rmp `#265`.

  *Amended in the fiftieth edition,* for rmp `#282`. The comparison ignores
  ASCII case, as `FR-PROJ-029` does. On a case-insensitive filesystem the
  walk of `FR-PROJ-004` finds a folder named `.TPL` when it looks for `.tpl`,
  so a case-sensitive test here refused a folder that discovery accepts. One
  rule on every host keeps the two requirements in step without testing the
  filesystem. *Accepted cost.* On a case-sensitive filesystem, `--tpl-dir`
  accepts a folder named `.TPL` that the walk would not find. The caller
  named it, and every trust check still applies.

  *Read against `FR-CLI-020`, and unchanged.* That requirement bars
  normalising the case of a flag value. The path is still used as written,
  and its case is never changed; only the test of its last segment ignores
  case.

## `tpl init`

```
tpl init [<path>]
```

- **FR-PROJ-012**: `tpl init` SHALL take an optional positional path, defaulting
  to the current directory.

  *Note added in the forty-ninth edition.* The path names the directory that
  will hold `.tpl`, never the `.tpl` folder itself. `FR-PROJ-029` refuses a
  path that names a `.tpl` folder.

- **FR-PROJ-029**: IF the destination of `tpl init` names a `.tpl` folder,
  THEN the system SHALL exit `64` (`EX_USAGE`) and SHALL create, change and
  delete nothing. The destination names a `.tpl` folder in either of two
  cases:

  1. **As written.** The last segment of the path as written, after any
     trailing separators and any trailing `.` segments are dropped, is `.tpl`.
  2. **Canonical.** The destination exists, and the last segment of its
     canonical path is `.tpl`. This is the case of `tpl init` with no operand,
     run inside a `.tpl` folder.

  In both cases the segment SHALL be compared with `.tpl` without regard to
  the case of ASCII letters, per `FR-PROJ-027`.

  The message SHALL be as follows:

  1. **`error`** SHALL state that `tpl init` takes the directory that will
     hold `.tpl`, not the `.tpl` folder.
  2. **`cause`** SHALL name the path as written, or `.` where no operand was
     given, and SHALL state that its last segment is `.tpl`.
  3. **`hint`** SHALL carry `tpl init` followed by the parent directory of the
     folder named: the path as written with its last segment removed, in the
     first case, and the canonical parent, in the second. WHERE that parent is
     the current directory, the `hint` SHALL carry `tpl init` with no operand.
     The parent is built under `FR-ERR-041`. IF that set refuses it, THEN the
     `hint` SHALL carry `tpl init <path>` and SHALL state in words that
     `<path>` stands for the directory that holds the folder named. The `hint`
     SHALL carry no flag.

  ```
  tpl init proj/.tpl
  error: tpl init takes the directory that will hold .tpl, not the .tpl folder
  cause: the path proj/.tpl ends in .tpl, so a project there would be nested inside that folder
  hint:  create the project in the parent directory: tpl init proj
  exit:  64 (EX_USAGE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code.

  The condition is evaluated when the destination is first examined: before
  the check of `FR-PROJ-014`, before any directory is created under
  `FR-PROJ-013`, and after the warning of `FR-PROJ-026`, where that warning
  is written.

  *Rationale.* `FR-PROJ-008`, `FR-PROJ-027` and every `hint` that names a
  project teach the form `<project>/.tpl`, so `tpl init <project>/.tpl` is
  the next mistake a caller makes. Before this requirement it exited `0` and
  created `<project>/.tpl/.tpl`. Discovery from `<project>` then found the
  outer folder, which held no `.cfg` and no templates: `tpl template list`
  listed nothing, `tpl cfg database add` wrote `.cfg` into the outer folder,
  and `tpl render example` exited `66`. The warning of `FR-PROJ-016` also
  named a project that did not exist, because `tpl init` had created the
  outer folder itself as a missing parent. This is finding W-01 of the sixth
  re-audit, recorded for rmp `#281`.

  *Why the name, as in `FR-PROJ-027`.* A `.tpl` folder is recognised by its
  name, by discovery and by `--tpl-dir` alike. The same test here refuses
  exactly the paths at which a project would be created inside what every
  other command reads as a `.tpl` folder.

  *Why `64`.* The operand is a value the caller wrote and can rewrite, and the
  destination could be created. `73` sends the caller to check permissions or
  to choose another destination, per its row of `FR-ERR-001`, and the fault
  is in neither. `FR-ERR-003` is unchanged: `73` is still produced only by
  `tpl init`.

  *Rejected: creating the project in the parent directory.* It gives the
  operand two meanings, and a caller that meant a directory literally named
  `.tpl` would receive a project it did not name. Also rejected: refusing any
  path with a `.tpl` segment anywhere in it. No finding reached that case, and
  `FR-PROJ-016` governs a destination beneath an existing project.

  *Accepted cost.* A project cannot be created in a directory whose own name
  is `.tpl`. The caller renames the directory or chooses another.

  *Added in the forty-ninth edition,* for rmp `#281`, from finding W-01 of the
  sixth re-audit.

  *Amended in the fiftieth edition,* for rmp `#282`. On a case-insensitive
  filesystem, `tpl init x/.TPL` passed the test, created `x/.TPL/.tpl`, and
  discovery from `x` then took `x/.TPL` as the project, which is the outcome
  of finding W-01 with no message. This is finding X-04 of the seventh
  re-audit of rmp `#263`. The segment is now compared without regard to ASCII
  case. *Rejected: comparing only the canonical path.* A destination that
  does not exist yet has no canonical path, and `x/.TPL` is created by the
  same invocation. *Accepted cost.* On a case-sensitive filesystem, a project
  cannot be created in a directory named `.TPL` in any case.

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

  *Amended in the forty-ninth edition.* A `.tpl` folder in an ancestor
  directory engages this requirement only WHERE it existed before the
  invocation. A `.tpl` folder that the same `tpl init` created as a missing
  parent, under `FR-PROJ-013`, is not a project above the destination, and
  the system SHALL NOT write the warning for it. The warning named a project
  that did not exist, per finding W-01 of the sixth re-audit, recorded for
  rmp `#281`. `FR-PROJ-029` refuses the commonest path that produced it; this
  amendment covers the paths that remain, such as `tpl init a/.tpl/b`.

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

- **FR-PROJ-026**: WHEN `tpl init` is given `--tpl-dir`, the system SHALL
  accept the flag, SHALL give it no effect, and SHALL write exactly one warning
  line to stderr, per `FR-OUT-020`:

  ```
  warning: --tpl-dir has no effect on tpl init; it takes its destination as an operand: tpl init <path>
  ```

  1. **No effect.** The destination SHALL be the one `FR-PROJ-012` gives: the
     positional path, or the current directory. The system SHALL NOT resolve,
     examine or create the path `--tpl-dir` names, and SHALL apply to it none
     of the checks of `FR-PROJ-008` through `FR-PROJ-011`. Those checks govern
     a folder an invocation uses as its project, and `tpl init` uses none, per
     `FR-PROJ-025`.
  2. **The line.** It SHALL name `--tpl-dir`, SHALL state that the flag has no
     effect on `tpl init`, and SHALL carry the form `tpl init <path>`. It SHALL
     NOT reproduce the value given to `--tpl-dir`: the value is not examined,
     so it is not written back.
  3. **When.** The line SHALL be written once, after step 1 of `FR-ERR-006`
     passes and before the destination is examined, so it is the first line on
     stderr. It SHALL precede the message of `FR-PROJ-014` or `FR-PROJ-015`
     and the warning of `FR-PROJ-016`. An invocation refused at step 1 —
     `--tpl-dir` given twice, per `FR-CLI-014`, for instance — writes the
     error of that step and no warning.
  4. **Exit code.** The exit code SHALL be the one the same invocation without
     `--tpl-dir` returns: `0`, or `73` per `FR-PROJ-014` and `FR-PROJ-015`.
  5. **Verbosity.** The line is a warning, so `-q/--quiet` suppresses it, per
     `FR-GLOB-015`. `-v/--verbose` does not change it.

  *Note added in the forty-ninth edition.* The line also precedes the message
  of `FR-PROJ-029`, and the exit code of item 4 may also be `64`, per that
  requirement.

  ```
  tpl init --tpl-dir /srv/shop/.tpl       warning, then 0 or 73 for ./.tpl
  tpl init /srv/shop --tpl-dir x          warning, then 0 or 73 for /srv/shop/.tpl
  tpl -q init --tpl-dir x                 no line; 0 or 73 for ./.tpl
  ```

  *Rationale.* `FR-GLOB-002` makes every global flag acceptable at every node,
  and `BR-GLOB-001` states that a global flag may have no effect on a node and
  is never refused alone. `FR-GLOB-007` applies the same rule to `-d` on a
  command that requires no entry. What is added here is the line. `--tpl-dir`
  names a location, and `tpl init` writes at one, so a caller that gave the
  flag looks for the project where the flag pointed. Before this requirement
  the invocation acted on the current directory in silence, and a `73` then
  named a `.tpl` the caller had not pointed at. This is finding R-11 of the
  re-audit recorded for rmp `#269`.

  *Rejected: exiting `64`.* It refuses a global flag given alone, which
  `FR-GLOB-002` and `BR-GLOB-001` forbid. It also breaks the case
  `FR-CLI-024` protects: an agent that appends the same global flags to every
  command it builds.

  *Rejected: using `--tpl-dir` as the destination.* It would give `tpl init`
  two ways to name one place and amend `FR-PROJ-012` and `FR-PROJ-017`. It
  would also need a rule for a named path whose last segment is not `.tpl`.
  `FR-PROJ-008` already points a caller from `--tpl-dir` to
  `tpl init <parent>`, not to the flag.

  *Accepted cost.* A caller that passes `-q`, or checks only the exit code,
  does not see the line. `FR-PROJ-016` accepts the same cost for its warning,
  and the help of `tpl init` states the fact where it is read, per
  `FR-HELP-034`.

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
  | `tpl cfg …` | `.tpl/.cfg`, created at mode `0600` where it is absent, per `FR-PROJ-028` |
  | `tpl cache load` | `.tpl/.cache/` |
  | any cached read command, on a miss | `.tpl/.cache/` |

  *Amended in the forty-eighth edition.* The `tpl cfg …` row states that the
  file is created where it is absent, per `FR-PROJ-028`.

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
