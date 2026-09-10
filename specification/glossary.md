---
title: Glossary
status: draft
last-reviewed: 2026-09-10
related: [README.md, cli-contract.md, catalogue-coverage.md, context-document.md]
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
(`tpl render …`). Command groups that are not arms — `tpl cache …`,
`tpl cfg …`, `tpl init`, `tpl help`, and `tpl version` — are auxiliary.

## budget

A named performance measurement with a recorded baseline, listed in
`NFR-PERF-014`. A budget states what is measured and over which reference
workload; the figure itself lives in `BENCHMARKS.md` and in the project's
architecture decision records, never in this specification.

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

## complete read / incomplete read

A read is complete when it returns, for every object it presents, every property
[catalogue-coverage.md](catalogue-coverage.md) defines for that object;
otherwise it is incomplete. Completeness is a property of a read, not of a
server: the same database read by two users can be complete for one and
incomplete for the other. See `FR-PRIV-001`.

## context

The set of top-level variables available to a template during a render:
`database`, one of `table` / `view` / `routine`, `vars`, `tpl`, `now`. The
command line decides which object variable is bound, per `FR-RND-023`. The
content of the two that come from a server is fixed by
[context-document.md](context-document.md); so is the content of the other
three, by `FR-CTX-026` through `FR-CTX-028`.

## context document

The single JSON document that carries the model: emitted by `tpl schema dump`,
consumed by `tpl render --context`, and seen by a template as the `database`
context variable. It travels as the `data` payload of a document envelope, per
`FR-SCH-017`. Its structure is fixed by
[context-document.md](context-document.md) and it is plumbing contract. Not to
be confused with *context*, above, which is the set of variables a template
sees, three of which never come from a database.

## contract group

One of the three tiers of the template surface defined by `FR-ENV-001`: what
`tpl` registers, which is full contract; an enumerated list of filters inherited
from the engine, guaranteed against a pinned engine minor version; and
everything else the engine offers, which works and is guaranteed by nobody.

## document envelope

The three-key outer shape every JSON document `tpl` writes to stdout carries:
`schema_version`, `source`, and `data`. Only `data` differs between commands,
and the module that owns a command owns the shape of its `data`. Fixed by
`FR-OUT-024`; the seventeen documents are indexed by `BR-OUT-002`.

## coverage

Which object kinds and which of their properties enter the model, fixed by
[catalogue-coverage.md](catalogue-coverage.md). Coverage is applied before
`--pattern`, per `FR-CAT-028`, so that any difference between what a listing
shows and what the server holds is attributable to one or the other and never to
both.

## database entry

A `[database.<name>]` block in `.tpl/.cfg` describing how to reach one server
and which database on it to read. The entry name is a label local to the
project; it need not match the database name on the server. Selected with
`-d/--database`.

## discovery

Locating the project by walking up from the current directory until a `.tpl`
folder is found, within the boundary defined by `FR-PROJ-005`. See
`FR-PROJ-004`.

## DSN

A single-string connection descriptor stored in the `dsn` key of a database
entry, in the form
`scheme://[user[:password]@]host[:port]/database[?params]`. See `FR-CONF-009`.

## group node

A node of the command tree that has children and no action of its own — `tpl`,
`tpl schema`, `tpl template`, `tpl cache`, `tpl cfg`, and `tpl cfg database`.
Invoked without a child, a group node prints its own help. See `FR-CLI-007`
and `FR-CLI-008`.

## leaf

A node of the command tree that performs an action. `tpl schema tables`,
`tpl render`, and `tpl init` are leaves.

## local flag

A flag declared by one or more specific commands rather than by the whole tree.
A command rejects any flag it does not declare. Contrast global flag.

## global flag

One of the seven flags accepted by every node of the tree. See
[global-flags.md](global-flags.md).

## word list

The sequence of words a naming filter derives from its operand before rejoining
them, by the tokenisation rule of `FR-ENV-030`. Every one of the five naming
filters is a pure function of it, which is what makes all five predictable from
one rule.

## model

Everything `tpl` knows about a database: the covered object kinds and their
covered properties. The model is the same whatever the source — a live read, a
cached read, or a `--context` document — and is defined by
[catalogue-coverage.md](catalogue-coverage.md).

## nearest match

A suggestion offered when a supplied name does not exist, computed by edit
distance over the names that do exist. See `FR-ERR-019`.

## normative budget

The single budget whose target is stated in the text of its requirement and can
therefore fail a change on its own, as opposed to the budgets that carry only
the no-regression rule. There is exactly one, fixed by `NFR-PERF-015`.

## object

A table, a view, or a routine. The three kinds are named by the same flag
spellings wherever a command names one: `--table`, `--view`, `--routine`.

## plumbing

Output intended to be consumed by another program. In `tpl`, `--format json`
and the JSON-only output of `tpl schema dump` and `tpl help --format json`.
Every plumbing document shares the envelope of `FR-OUT-024` and is versioned by
`schema_version`.

## porcelain

Output intended to be read by a person. In `tpl`, `--format text`, which is
explicitly not a contract. See `FR-OUT-004`.

## pre-scan

A scan of the raw argument vector, performed before the parser runs, that
determines whether errors are emitted as text or as JSON. See `FR-ERR-017`.

## project

Any directory containing a `.tpl` folder. The `.tpl` folder is the project root
and the only source of configuration and templates.

## qualified routine name

A routine named in the form `procedure:<name>` or `function:<name>`. Procedures
and functions occupy distinct namespaces on the server, so one bare name can
denote two objects; the qualified form says which is meant. Accepted wherever a
command names one routine, per `FR-SCH-008`. A bare name matching both is `64`,
per `FR-SCH-010`.

## read-through cache

A cache consulted before the server on every read, and populated immediately
whenever the read misses. See `FR-CACHE-006` and `FR-CACHE-007`.

## reference workload

One of the three databases against which a performance budget is measured:
`WL-001`, the large workload; `WL-002`, a verification scalar over it; and
`WL-003`, the small workload that is the common path. Defined in
[performance-requirements.md](performance-requirements.md).

## render

One execution of one template against one context, producing one result on
stdout. Exactly one render happens per `tpl render` invocation. See
`FR-RND-002`.

## requirement of form

A performance requirement stated as an observable, permanent property rather
than as a figure — that the catalogue-query count does not grow with the number
of objects, that a cache hit opens no connection. It is verified from outside
the process and needs no stopwatch. The six are `NFR-PERF-001` through
`NFR-PERF-006`.

## restricted

The field by which a listing or a dump marks an object the reader could not read
in full. An object requested by name never carries it, because that case fails
with `77` instead. A document carrying it is refused as a `--context`. See
`FR-PRIV-005` and `FR-PRIV-008`.

## routine

A stored procedure or a stored function. MariaDB distinguishes the two in
`INFORMATION_SCHEMA.ROUTINES.ROUTINE_TYPE`; `tpl` reports both under the single
term "routine" and states the kind per object.

## schema (the word, two meanings)

1. The first arm, `tpl schema …`, which reads the structure of a database.
2. The `--schema` flag of `tpl cfg database add|update`, which names the
   database on the server.

Help text disambiguates the two wherever both could be meant. See `FR-CFG-028`.

## source

The second key of the document envelope, by which every JSON document states
where its bytes came from: `server`, `cache`, `project`, or `binary`. It is an
enumerated string rather than a boolean so that a value may be added without
breaking the contract, per `FR-CDOC-010` — two were added by the third edition.
`"source":"cache"` is also the signal that a document promises neither
referential integrity nor a snapshot, per `FR-CDOC-016`. The value set is fixed
by `FR-OUT-026`.

## template

A file under `.tpl/templates/` whose name ends in `.jinja`. Nothing else in that
directory is a template. See `FR-TMPL-004` and `FR-TMPL-005`.

## template name

The path of a template relative to `.tpl/templates/`, with the `.jinja`
extension optional on the command line and mandatory inside a template. See
`FR-TMPL-006`, `FR-TMPL-007`, and `FR-TMPL-008`.

## volatile field

A catalogue field the server changes without any change to the structure — a row
estimate, a data length, a modification timestamp, an index cardinality.
Thirteen are excluded from the model as a closed list by `FR-CAT-024`, because
carrying one would put `NFR-DET-001` in permanent conflict with the server.

## entry label

Synonym for the name of a database entry. Used where "database name" would be
ambiguous with the server-side database.
