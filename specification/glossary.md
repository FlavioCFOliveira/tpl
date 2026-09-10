---
title: Glossary
status: approved
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

A named performance measurement, listed in `NFR-PERF-014`, which states what is
measured, over which reference workload, and whether a server is needed. A
budget's **ratified** figure lives in `BENCHMARKS.md` and never in this
specification, per `BR-PERF-006`. Until a budget is measured it may carry a
**provisional figure** here instead.

## provisional figure

A working target carried in the table of `NFR-PERF-014` and marked as
provisional. It is not a limit: it fails no change, is not a baseline, and is
superseded by the first valid measurement of that budget under the gate of
`NFR-PERF-020`. See `NFR-PERF-019`. A number in this corpus that is neither
marked provisional nor recorded in `BENCHMARKS.md` is a defect.

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

## divergence register

The table in [server-contract.md](server-contract.md) that records every
difference between the supported series which the model accommodates: the
field, the treatment applied, and what each series was observed to return. Its
form is fixed by `FR-SRV-036` and its content is required by `FR-SRV-027`. It
holds two rows, both fields whose value differs because the servers' own
default collations differ and which are therefore passed through verbatim
under `FR-SRV-039`. Every difference observed between the series, whether or
not the model accommodates it, is recorded separately under `FR-SRV-038`.

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

One of the seven flags accepted by every node of the tree, in any position, per
`FR-CLI-024`. Five of the seven carry a short form, and those five are the
whole short-flag set of the tool: no local flag has one, per `FR-GLOB-024`. See
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
in full. It is an array of strings naming the properties that could not be read,
present only on an incomplete object, per `FR-PRIV-016`. An object requested by
name never carries it, because that case fails with `77` instead; a document
carrying it is refused as a `--context`; and an object carrying it is never
written to the cache, per `FR-CACHE-037`. See `FR-PRIV-005`, `FR-PRIV-008`, and
`FR-PRIV-016`.

## routine

A stored procedure or a stored function. MariaDB distinguishes the two in
`INFORMATION_SCHEMA.ROUTINES.ROUTINE_TYPE`; `tpl` reports both under the single
term "routine" and states the kind per object.

## series / supported series / major family

A **major family** is a MariaDB release line identified by its leading number:
`10`, `11`, `12`. A **series** is a release line within a family, identified by
two parts: `10.11`, `11.4`, `12.3`. A **supported series** is one that
`FR-SRV-001` admits — it belongs to one of the three most recent major families
and is under MariaDB community maintenance. The four supported series on the
verification date are named by `FR-SRV-015`, which is the only place in this
corpus that names them, per `BR-SRV-005`.

A **rolling release** is a series MariaDB does not maintain after GA. No rolling
release is a supported series, per `FR-SRV-016`.

Version comparison in this specification is always by series, never by point
release: `10.11.14` and `10.11.2` are the same series, per `FR-SRV-021`. A
series **below** the window is refused, per `FR-SRV-020`; one **above** it is
read and marked, per `FR-SRV-031` — see *standing* below.

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

## standing

The third key of the `server` object of `FR-CTX-031`, stating where the server
the read was made against sits relative to the supported window: `supported`, or
`newer_than_supported`. It is enumerated, always present, and never `null`, per
`FR-CTX-034`. A series *below* the window has no value because it never reaches
a document — `FR-SRV-020` refuses it before the catalogue is read.

## target

One of the four build targets of `NFR-PERF-018`: Linux on amd64 and arm64,
statically linked against `musl`, and macOS on amd64 and arm64. Every
measurement, every baseline, and every budget is stated against one of them,
and measurements on different targets are never compared, per `NFR-PERF-012`.
None is second class.

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
Sixteen are excluded from the model as a closed list by `FR-CAT-024`, because
carrying one would put `NFR-DET-001` in permanent conflict with the server.
Three of the sixteen are the creation and alteration timestamps of a routine
and of a trigger, which differ between two servers of the same series and
would make `FR-SRV-026` unsatisfiable.

## entry label

Synonym for the name of a database entry. Used where "database name" would be
ambiguous with the server-side database.
