---
title: Glossary
status: draft
last-reviewed: 2026-09-09
related: [README.md, cli-contract.md, configuration-model.md]
---

# Glossary

Terms are defined once, here, and used consistently across the corpus. A term
used in a requirement without being defined here is a defect.

## alias

An additional, shorter name accepted for a subcommand. An alias is exactly
equivalent to its canonical name and is declared in this specification; no other
short form is accepted. See `FR-CLI-011`.

## arm

One of the three read-only capabilities `tpl` provides: exploring the database
(`tpl schema …`), exploring the templates (`tpl template …`), and rendering
(`tpl render …`). Command groups that are not arms — `tpl cfg …`, `tpl cache …`,
`tpl init`, `tpl help`, `tpl version` — are auxiliary.

## cache

The catalogue data `tpl` has previously read from a server and stored under
`.tpl/.cache/`, keyed by database entry name. See
[cache-commands.md](cache-commands.md).

## cache hit / cache miss

A hit is a read served entirely from `.tpl/.cache/` with no connection opened. A
miss is a read for which the required data is absent, unreadable, or of an
unknown format version, and which therefore reaches the server.

## canonical name

The primary name of a command or subcommand, as opposed to an alias. Help text
and the JSON command tree present the canonical name first.

## catalogue

The database structure `tpl` reads: database metadata, tables, columns, indexes,
keys, views, routines, and triggers. How the catalogue is read is outside the
scope of this edition.

## command tree

The complete set of commands, subcommands, aliases, arguments, and flags `tpl`
accepts. It is closed: see `FR-CLI-002`.

## context

The set of top-level variables available to a template during a render:
`database`, one of `table` / `view` / `routine`, `vars`, `tpl`, `now`. The
command line decides which object variable is bound; the content of each
variable is outside the scope of this edition.

## database entry

A `[database.<name>]` block in `.tpl/.cfg` describing how to reach one server
and which database on it to read. The entry name is a label local to the
project; it need not match the database name on the server. Selected with
`-d/--database`.

## discovery

Locating the project by walking up from the current directory until a `.tpl`
folder is found, within the boundary defined by `FR-PROJ-004`.

## DSN

A single-string connection descriptor stored in the `dsn` key of a database
entry, in the form
`scheme://[user[:password]@]host[:port]/database[?params]`. See `FR-CONF-008`.

## group node

A node of the command tree that has children and no action of its own — `tpl`,
`tpl schema`, `tpl template`, `tpl cfg`, `tpl cfg database`, `tpl cache`.
Invoked without a child, a group node prints its own help. See `FR-CLI-007`.

## leaf

A node of the command tree that performs an action. `tpl schema tables`,
`tpl render`, and `tpl init` are leaves.

## local flag

A flag declared by one or more specific commands rather than by the whole tree.
A command rejects any flag it does not declare. Contrast global flag.

## global flag

One of the seven flags accepted by every node of the tree. See
[global-flags.md](global-flags.md).

## nearest match

A suggestion offered when a supplied name does not exist, computed by edit
distance over the names that do exist. See `FR-ERR-015`.

## object

A table, a view, or a routine. The three kinds are named by the same flag
spellings wherever a command names one: `--table`, `--view`, `--routine`.

## plumbing

Output intended to be consumed by another program. In `tpl`, `--format json`
and the JSON-only output of `tpl schema dump` and `tpl help --format json`. The
plumbing contract is versioned by `schema_version`.

## porcelain

Output intended to be read by a person. In `tpl`, `--format text`, which is
explicitly not a contract. See `FR-OUT-004`.

## pre-scan

A scan of the raw argument vector, performed before the parser runs, that
determines whether errors are emitted as text or as JSON. See `FR-ERR-013`.

## project

Any directory containing a `.tpl` folder. The `.tpl` folder is the project root
and the only source of configuration and templates.

## read-through cache

A cache consulted before the server on every read, and populated immediately
whenever the read misses. See `FR-CACHE-002`.

## render

One execution of one template against one context, producing one result on
stdout. Exactly one render happens per `tpl render` invocation. See
`FR-RND-002`.

## routine

A stored procedure or a stored function. MariaDB distinguishes the two in
`INFORMATION_SCHEMA.ROUTINES.ROUTINE_TYPE`; `tpl` reports both under the single
term "routine" and states the kind per object.

## schema (the word, two meanings)

1. The first arm, `tpl schema …`, which reads the structure of a database.
2. The `--schema` flag of `tpl cfg database add|update`, which names the
   database on the server.

Help text disambiguates the two wherever both could be meant. See `FR-CFG-019`.

## template

A file under `.tpl/templates/` whose name ends in `.jinja`. Nothing else in that
directory is a template. See `FR-TMPL-011`.

## template name

The path of a template relative to `.tpl/templates/`, with the `.jinja`
extension optional on the command line and mandatory inside a template. See
`FR-TMPL-010` and `FR-TMPL-012`.

## entry label

Synonym for the name of a database entry. Used where "database name" would be
ambiguous with the server-side database.
