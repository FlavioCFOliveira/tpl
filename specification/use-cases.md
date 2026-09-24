---
title: Use Cases
status: approved
last-reviewed: 2026-09-24
related: [schema-commands.md, render-command.md, cache-commands.md, cfg-commands.md, examples.md]
---

# Use Cases

## Overview

These flows exercise the command surface end to end. They exist to show how the
requirements compose, and to give each help text a source for its `EXAMPLES`
section. Every command shown is defined elsewhere in this specification; nothing
here introduces behaviour of its own.

## UC-001 — Bootstrap a project

- **Actor**: calling agent or operator
- **Trigger**: a repository has no `.tpl` folder
- **Preconditions**: the destination directory is writable
- **Main flow**:
  1. Run `tpl init`.
  2. `tpl` creates the five artefacts of `FR-PROJ-017`: `.tpl/.cfg` at mode
     `0600`, `.tpl/.gitignore`, `.tpl/templates/`,
     `.tpl/templates/example.jinja`, and `.tpl/templates/rust/_types.jinja`.
  3. `tpl` writes nothing to stdout and exits `0`.
- **Alternate flows**:
  - A `.tpl` already exists at the destination: exit `73`, nothing changed.
  - A `.tpl` exists in an ancestor: the nested project is created, a warning
    goes to stderr, exit `0`.
  - `--tpl-dir` is given: it has no effect, a warning naming the form
    `tpl init <path>` goes to stderr, and the flow continues at the
    destination the invocation would have without the flag.
  - The path names a `.tpl` folder, as in `tpl init proj/.tpl`: exit `64`,
    nothing created, and the `hint` carries `tpl init proj`.
- **Postconditions**: the project is usable; no database is known to it, so
  `tpl cfg database list` answers with an empty listing and exit `0`, per
  `FR-CFG-040`
- **Requirements**: `FR-PROJ-012` … `FR-PROJ-022`, `FR-PROJ-025`, `FR-PROJ-026`,
  `FR-PROJ-029`

## UC-002 — Register a database entry

- **Actor**: operator
- **Trigger**: the project needs to reach a server
- **Preconditions**: a `.tpl` project exists
- **Main flow**:
  1. Run
     `tpl cfg database add shop --host db.example.com --user alice --schema shop`.
  2. `tpl` writes the `[database.shop]` block to `.tpl/.cfg` and exits `0`
     silently.
  3. Optionally run `tpl cfg set core.database shop` so that `-d` may be
     omitted.
- **Alternate flows**:
  - Neither `--dsn` nor any discrete flag is given: exit `64`.
  - The entry already exists: exit `64`, with a hint pointing at
    `tpl cfg database update`.
  - The project is a clone and `.tpl` holds no `.cfg`: the command creates
    `.tpl/.cfg` at mode `0600`, writes the block, and exits `0`.
  - `--tpl-dir` names the directory that holds `.tpl`, not `.tpl` itself:
    exit `78`, nothing written, and the `hint` carries the corrected
    `--tpl-dir`.
  - `--ca-file` or `--ca-path` holds `${VAR}`: exit `64`, nothing written.
  - The entry name is empty or holds a character other than a letter, a
    digit or an underscore, or `--host` or `--schema` is empty: exit `64`,
    nothing written.
- **Postconditions**: `-d shop` resolves
- **Requirements**: `FR-CFG-015` … `FR-CFG-017`, `FR-CFG-027`, `FR-PROJ-027`,
  `FR-PROJ-028`, `FR-CONF-047`, `FR-CONF-048`, `FR-CONF-050`

## UC-003 — Keep the password out of the file

- **Actor**: operator
- **Trigger**: the entry needs a password that must not be written to disk
- **Main flow**:
  1. Run
     `tpl cfg set database.shop.password_command 'security find-generic-password -s tpl-shop -w'`.
  2. `tpl` splits the string by POSIX quoting rules and stores it as an array.
  3. On the next connection, `tpl` executes the command directly, without a
     shell, and uses its trimmed stdout.
- **Alternate flows**:
  - The command exceeds `core.password_timeout`: exit `78`.
- **Requirements**: `FR-CONF-023` … `FR-CONF-028`

## UC-004 — Verify connectivity

- **Actor**: calling agent
- **Trigger**: an entry has just been registered or repointed
- **Main flow**:
  1. Run `tpl cfg database test shop`.
  2. `tpl` connects and authenticates, enforces the read-only session, checks
     the server series, runs the catalogue privilege probe, and reports the
     outcome of all four, per `FR-CFG-024`.
- **Alternate flows**:
  - Server unreachable: exit `69`. Authentication refused: exit `77`. Read-only
    session cannot be established: exit `78`.
  - Connected and authenticated, but the series is not supported: exit `78`
    with the message of `FR-SRV-030`, per `FR-CFG-043`. The `cause` line says
    that the connection and the authentication succeeded, which is what
    separates this outcome from the `69` and the `77` above.
  - Connected, authenticated, and supported, but the reader cannot see the
    catalogue: exit `0` with `can_read_catalogue` false, per `FR-CFG-045`. The
    exit code does not change, so a caller that needs this answer reads the
    field.
- **Notes**: the command performs four steps and reports all four; the `server`
  field of `FR-CFG-039` carries the version, the series, and the standing, and
  `can_read_catalogue` carries the probe. A server newer than the supported
  window exits `0` here and is reported with
  `standing: "newer_than_supported"`, per `FR-SRV-031`
- **Postconditions**: the cache is untouched, whatever the outcome
- **Requirements**: `FR-CFG-024`, `FR-CFG-025`, `FR-CFG-043`, `FR-CFG-039`,
  `FR-CFG-044`, `FR-CFG-045`, `FR-CACHE-010`, `FR-SRV-034`

## UC-005 — Learn the whole CLI in one call

- **Actor**: calling agent
- **Trigger**: first contact with the tool
- **Main flow**:
  1. Run `tpl help --format json`.
  2. `tpl` emits the complete command tree — commands, subcommands, aliases,
     arguments, options, examples, and exit codes — as one compact document in
     the envelope of `FR-OUT-024`, with `tpl_version` and the global flags
     listed once under `data`.
  3. The invocation needs no project: `FR-PROJ-025` exempts every form of
     `help` from discovery, so this works in a directory that has no `.tpl`.
- **Alternate flows**:
  - `tpl help schema --format json` for one subtree.
  - `tpl help --format json --pretty` for a readable form.
- **Requirements**: `FR-HELP-016` … `FR-HELP-024`, `FR-OUT-024`, `FR-PROJ-025`

## UC-006 — Inspect the database structure

- **Actor**: calling agent
- **Trigger**: the agent needs to know what the database contains
- **Main flow**:
  1. Run `tpl -d shop schema tables` to see the listing as aligned text.
  2. Run `tpl -d shop schema table orders --format json` for the full,
     machine-readable metadata of one table.
- **Alternate flows**:
  - The table does not exist: exit `66` with a nearest-match suggestion.
  - Narrow the listing: `tpl -d shop schema tables --pattern '%_log'`.
- **Postconditions**: the catalogue read is now cached
- **Requirements**: `FR-SCH-004`, `FR-SCH-009` … `FR-SCH-014`, `FR-CACHE-007`

## UC-007 — Render one object

- **Actor**: calling agent
- **Trigger**: a source file must be generated from one table
- **Main flow**:
  1. Run
     `tpl -d shop render rust/struct --table orders > src/models/orders.rs`.
  2. `tpl` reads the catalogue through the cache, renders once, and writes the
     result to stdout; the shell redirects it.
- **Alternate flows**:
  - The template does not exist: exit `66` with a suggestion.
  - The template has a syntax error, or the render fails: exit `65`, with
    template, line, and column.
  - Two kinds of object flag are given: exit `64`.
- **Requirements**: `FR-RND-001` … `FR-RND-006`, `FR-RND-028` … `FR-RND-031`

## UC-008 — Render every table

- **Actor**: calling agent
- **Trigger**: one file per table is needed
- **Preconditions**: a JSON-capable filter is available to the caller
- **Main flow**:
  1. Obtain the names: `tpl -d shop schema tables --format json`.
  2. Loop in the caller's shell, one `tpl render` invocation per table, each
     redirected to its own path.
- **Notes**: there is no `--all-tables`; iteration is the caller's job, and the
  `EXAMPLES` of both `schema tables` and `render` show this loop
- **Requirements**: `FR-RND-002`, `BR-RND-002`, `FR-SCH-029`

## UC-009 — Render without a database

- **Actor**: calling agent, in CI
- **Trigger**: the render must run where no server is reachable
- **Main flow**:
  1. On a machine with access: `tpl -d shop schema dump > shop.schema.json`.
  2. Commit the snapshot.
  3. Anywhere: `tpl render rust/struct --context shop.schema.json --table orders`.
- **Alternate flows**:
  - Pipeline form: `tpl -d shop schema dump | tpl render rust/struct --context - --table orders`.
  - `--context` together with an explicit `-d` on the command line: exit `64`.
  - The document is malformed: exit `65`.
- **Notes**: the dump carries only the server-derived part, inside the `data`
  of the envelope; `vars`, `tpl`, and `now` are always injected by the render,
  in the forms `FR-CTX-026` through `FR-CTX-028` fix. `BR-SCH-004` mandates the
  test that keeps this round-trip working
- **Requirements**: `FR-SCH-016` … `FR-SCH-022`, `FR-SCH-036`, `BR-SCH-004`,
  `FR-RND-016` … `FR-RND-024`

## UC-010 — Work offline from a warm cache

- **Actor**: operator
- **Trigger**: the network will be unavailable, or repeated reads should not hit
  the server
- **Main flow**:
  1. Run `tpl -d shop cache load` while the server is reachable.
  2. Subsequent `schema` and `render` invocations are served from
     `.tpl/.cache/shop/` and open no connection.
  3. Run `tpl -d shop cache status` to see what is stored and when it was
     loaded.
- **Alternate flows**:
  - The server is unreachable during `cache load`: exit `69`, and what was
    already stored is unchanged.
  - An invocation must reach the server anyway: `--direct`.
  - An invocation must touch no file at all: `--direct --no-cache`.
- **Requirements**: `FR-CACHE-006` … `FR-CACHE-016`, `FR-CACHE-022`,
  `FR-CACHE-032`

## UC-011 — Repoint an entry

- **Actor**: operator
- **Trigger**: an entry must point at a different server
- **Main flow**:
  1. Run `tpl cfg database update shop --host db-staging.example.com`.
  2. Run `tpl -d shop cache clean`.
- **Alternate flows**:
  - Step 2 is skipped: reads continue to serve the previous server's catalogue,
    with exit `0` and nothing in the output saying so. The only signal is the
    load time reported by `tpl cache status`.
- **Notes**: nothing invalidates the cache automatically; this failure mode is
  accepted and documented
- **Requirements**: `FR-CACHE-028`, `FR-CACHE-029`, `BR-CACHE-003`

## UC-012 — Recover from a mistyped name

- **Actor**: calling agent
- **Trigger**: an invocation failed
- **Main flow**:
  1. `tpl -d shop schema table ordrs` exits `66`.
  2. The agent reads the four-line error on stderr and finds a runnable command
     in `hint`, and the nearest match named beside it.
  3. The agent runs the suggested command and retries with the corrected name.
- **Alternate flows**:
  - The nearest-match candidate contains characters outside `[A-Za-z0-9_]`: it
    is not presented at all, and only the generic hint is emitted, per
    `FR-ERR-023`. The generic hint is the listing command, so the name is
    recoverable in one further invocation.
  - `--format json` was supplied: the result document would have been JSON, but
    the error is not. `FR-ERR-033` makes every diagnostic the same four lines of
    text, whatever the format, and the exit code is the machine-comparable
    signal
  - The mistyped name is a shortened command, `tpl sch tables`: it is not
    executed, and the `64` suggests `schema`, per `FR-ERR-042`.
- **Requirements**: `FR-ERR-008`, `FR-ERR-009`, `FR-ERR-019` … `FR-ERR-024`,
  `FR-ERR-033`, `FR-ERR-034`, `FR-ERR-042`

## UC-013 — Build an application's data layer from a known schema

- **Actor**: calling agent or operator, through the driver script of a worked
  example
- **Trigger**: an application needs types for one target language that match a
  database it does not own
- **Preconditions**: a server of the most recent series of `FR-SRV-015` is
  reachable and carries the three schemas of `FR-EX-006`; the workspace of
  `FR-EX-010` is writable
- **Main flow**:
  1. Run `tpl init` in the workspace. The project is created, per `UC-001`.
     The example's own templates are then placed under `.tpl/templates/`, per
     `FR-EX-010`, because that is where step 5 reads a template from, per
     `FR-TMPL-004`.
  2. Run `tpl cfg database add <name> --host … --user … --schema <schema>`, once
     per schema read, per `UC-002`.
  3. Run `tpl cfg database test <name>`. The four steps of `FR-CFG-024` are
     reported, and `can_read_catalogue` is read from the `0` rather than
     inferred from it, per `FR-CFG-045`.
  4. Run `tpl -d <name> schema tables --format json` and take the table names
     from `data`.
  5. For each name, run
     `tpl -d <name> render <language>/struct --table <name>` and redirect the
     standard output to the file that table's type belongs in. `tpl render`
     writes nowhere else, per `FR-RND-028`, so the redirection is the caller's.
  6. Run the example's compile gate over every file written. The target
     language's own compiler accepts them, or the example has failed, per
     `FR-EX-009`.
- **Alternate flows**:
  - Step 1 exits `73`: the workspace still holds the `.tpl` of an earlier run,
    per `FR-PROJ-014`, and nothing is changed. `FR-EX-010` obliges the workflow
    to remove it first, which is what makes an example re-runnable.
  - Step 3 exits `78`: the entry names a server outside the supported window,
    per `FR-CFG-043`, and no read is attempted.
  - Step 3 exits `0` with `can_read_catalogue` false: the entry's user cannot
    see the catalogue, so step 4 would return an incomplete read under
    `FR-PRIV-001` and the data layer would be short without saying so.
  - Step 5 exits `65`: the template or the type-mapping macro would not parse,
    per `FR-RND-030`, or would not evaluate — a type the macro has no branch
    for is an undefined result — per `FR-RND-031`. Either names the template,
    the line, and the column.
  - Step 6 rejects a file: the render succeeded and the code is wrong. This is
    the outcome the gate exists for, and `FR-EX-009` states why no earlier step
    can report it.
- **Postconditions**: the data layer compiles; the catalogue read is cached, per
  `FR-CACHE-007`
- **Notes**: every step is one command line and no step reaches a library
  interface, per `FR-EX-004`. Which types the rendered files declare is the
  example's type-mapping macro's, per `FR-EX-008` and `FR-ENV-011`, and not
  `tpl`'s
- **Requirements**: `FR-EX-001` … `FR-EX-010`, `FR-PROJ-014`, `FR-PROJ-017`,
  `FR-TMPL-004`, `FR-CFG-024`, `FR-CFG-045`, `FR-SCH-004`, `FR-RND-002`,
  `FR-RND-028`, `FR-ENV-011`

## Dependencies

Every use case above is a composition of requirements owned by the module files
listed in the specification [README](README.md#file-index).

## Open questions

None specific to this module. Each use case inherits the open questions of the
requirements it composes, and those requirements carry none: the index of
[open-questions.md](open-questions.md) is empty.
