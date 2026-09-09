---
title: Use Cases
status: draft
last-reviewed: 2026-09-09
related: [schema-commands.md, render-command.md, cache-commands.md, cfg-commands.md]
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
  2. `tpl` creates `.tpl/.cfg` at mode `0600`, `.tpl/.gitignore`,
     `.tpl/templates/`, and `.tpl/templates/example.jinja`.
  3. `tpl` writes nothing to stdout and exits `0`.
- **Alternate flows**:
  - A `.tpl` already exists at the destination: exit `73`, nothing changed.
  - A `.tpl` exists in an ancestor: the nested project is created, a warning
    goes to stderr, exit `0`.
- **Postconditions**: the project is usable; no database is known to it
- **Requirements**: `FR-PROJ-012` … `FR-PROJ-022`

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
- **Postconditions**: `-d shop` resolves
- **Requirements**: `FR-CFG-015` … `FR-CFG-017`, `FR-CFG-027`

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
  2. `tpl` connects, enforces the read-only session, and reports.
- **Alternate flows**:
  - Server unreachable: exit `69`. Authentication refused: exit `77`. Read-only
    session cannot be established: exit `78`.
- **Postconditions**: the cache is untouched, whatever the outcome
- **Requirements**: `FR-CFG-024`, `FR-CFG-025`, `FR-CACHE-010`

## UC-005 — Learn the whole CLI in one call

- **Actor**: calling agent
- **Trigger**: first contact with the tool
- **Main flow**:
  1. Run `tpl help --format json`.
  2. `tpl` emits the complete command tree — commands, subcommands, aliases,
     arguments, options, examples, and exit codes — as one compact document with
     the global flags listed once.
- **Alternate flows**:
  - `tpl help schema --format json` for one subtree.
  - `tpl help --format json --pretty` for a readable form.
- **Requirements**: `FR-HELP-016` … `FR-HELP-024`

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
- **Notes**: the dump carries only the server-derived part; `vars`, `tpl`, and
  `now` are always injected by the render
- **Requirements**: `FR-SCH-016` … `FR-SCH-022`, `FR-RND-016` … `FR-RND-024`

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
  2. The agent reads the four-line error and finds a runnable command in
     `hint`, plus `did_you_mean` when the output format is JSON.
  3. The agent runs the suggested command and retries with the corrected name.
- **Alternate flows**:
  - The name contains characters outside `[A-Za-z0-9_]`: no executable
    suggestion is offered, and the name appears only as data in
    `did_you_mean`.
- **Requirements**: `FR-ERR-008`, `FR-ERR-009`, `FR-ERR-019` … `FR-ERR-024`

## Dependencies

Every use case above is a composition of requirements owned by the module files
listed in the specification [README](README.md#file-index).
