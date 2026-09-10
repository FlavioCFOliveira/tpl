---
title: Catalogue Cache
status: draft
last-reviewed: 2026-09-10
related: [schema-commands.md, render-command.md, project-and-discovery.md, cfg-commands.md]
---

# Catalogue Cache

## Overview

`.tpl/` may hold catalogue data previously read from a server, so that repeated
invocations do not read the same structure again and again. The cache is
read-through, written on every miss, and never expires on its own. A fourth
command group, `tpl cache`, loads it, cleans it, and reports on it.

## Scope

In scope: the cache location and keying, read-through semantics, the `--direct`
and `--no-cache` flags, the three `cache` subcommands, invalidation policy,
write atomicity, and the failure modes the model accepts.

Out of scope: the on-disk encoding of a cached object, and how the catalogue is
read from the server on a miss.

## Actors

- **Calling agent** or **operator**.
- **Database entry**, which is the cache key.
- **Project**, whose `.tpl/.cache/` folder holds the data.

## Location and keying

- **FR-CACHE-001**: The cache SHALL live at `.tpl/.cache/`, with one folder per
  database entry:

  ```
  .tpl/
  ├── .cfg
  ├── .cache/
  │   └── shop/
  │       ├── meta.json
  │       ├── tables/
  │       ├── views/
  │       └── routines/
  ├── .gitignore
  └── templates/
  ```

  *Rationale.* Both halves of `.tpl/` that are local to a machine are dotfiles;
  the one half that travels with the repository, `templates/`, is not.

- **FR-CACHE-002**: The cache SHALL be keyed by database entry name and by
  nothing else.

- **FR-CACHE-003**: The system SHALL NOT create `.tpl/.cache/` during
  `tpl init`. It appears on the first read that populates it.

- **FR-CACHE-004**: The `.tpl/.gitignore` written by `tpl init` SHALL exclude
  `.cache/`, per `FR-PROJ-017`.

- **FR-CACHE-005**: Each cached document SHALL carry a format version. The two
  versions the cache keeps, and where each is written, are fixed by
  `FR-CDOC-001` through `FR-CDOC-005`.

- **BR-CACHE-001**: The on-disk layout of `.tpl/.cache/` is not part of the
  plumbing contract. `tpl cache status` is the supported way to learn the state
  of the cache.

## Read-through semantics

- **FR-CACHE-006**: WHEN a cached read command runs, the system SHALL consult
  the cache first. On a hit it SHALL serve the result from disk and SHALL NOT
  open a connection.

- **FR-CACHE-007**: WHEN a cached read misses, the system SHALL read the server,
  SHALL write the result to the cache, and SHALL then answer. The write happens
  before the answer.

- **FR-CACHE-008**: The system SHALL NOT apply a time-to-live and SHALL NOT
  expire an entry automatically. A cached object stays until `tpl cache clean`
  removes it or a fresh read replaces it.

  *Rationale.* An authoritative-only cache would make the first invocation on a
  new project fail; a TTL would make the read source depend on the clock; an
  opt-in flag would mean the cache is used only by callers who remember to ask.

- **FR-CACHE-009**: The following commands SHALL read through the cache: the
  eight `schema` subcommands, and `tpl render` when it has no `--context`.

- **FR-CACHE-010**: `tpl cfg database test` SHALL always contact the server and
  SHALL neither read nor write the cache.

  *Rationale.* Contacting the server is the command's only reason to exist.

- **FR-CACHE-011**: No `template` subcommand and no `cfg` subcommand other than
  `database test` SHALL contact a database or touch the cache.

- **FR-CACHE-012**: WHEN a read is served from the cache and the output format
  is `json`, the document SHALL state that the read was cached rather than live.
  The field that carries it is the `source` field of `FR-CDOC-009`, which
  [cache-documents.md](cache-documents.md) owns.

- **BR-CACHE-002**: A read command is no longer read-only with respect to the
  filesystem, while remaining absolutely read-only with respect to the database.

## The cache flags

- **FR-CACHE-013**: `--direct` SHALL cause the invocation to read from the
  database, ignoring whatever is cached.

- **FR-CACHE-014**: `--no-cache` SHALL cause the invocation not to store its
  result in the cache.

- **FR-CACHE-015**: The two flags SHALL be orthogonal and SHALL compose:

  | Invocation | Reads from | Writes cache |
  |---|---|---|
  | neither flag | cache, else database | yes, on a miss |
  | `--direct` | database | yes |
  | `--no-cache` | cache, else database | no |
  | `--direct --no-cache` | database | no |

- **FR-CACHE-016**: `--direct --no-cache` SHALL be the pure read: it touches
  neither the cache nor any other file, and is the form that works when `.tpl/`
  is not writable.

- **FR-CACHE-017**: `--direct` and `--no-cache` SHALL be declared by exactly the
  eight `schema` subcommands, `tpl render`, and `tpl cache load`. Every other
  command SHALL reject them as unknown flags, per `FR-CLI-019`.

- **FR-CACHE-018**: `tpl cache load` SHALL accept `--direct` and SHALL ignore
  it, because reading the server is what the command does.

- **FR-CACHE-019**: IF `--no-cache` is supplied to `tpl cache load`, THEN the
  system SHALL exit `64` (`EX_USAGE`).

  *Rationale.* Loading without storing is a contradiction between a flag the
  command declares and what the command does.

- **FR-CACHE-020**: `tpl cache clean` and `tpl cache status` SHALL NOT declare
  `--direct` or `--no-cache`; supplying either is an unknown-flag error under
  `FR-CLI-019`, exit `64`.

## `tpl cache`

```
tpl -d shop cache load                      everything
tpl -d shop cache load  --table   orders
tpl -d shop cache load  --view    v_sales
tpl -d shop cache load  --routine calc_vat
tpl -d shop cache clean                     everything
tpl -d shop cache clean --table   orders
tpl -d shop cache status
```

- **FR-CACHE-021**: `tpl cache` SHALL be a group node, per `FR-CLI-007`, with
  exactly the subcommands `load`, `clean`, and `status`.

- **FR-CACHE-022**: `tpl cache load` SHALL read from the server and store the
  result. With no object flag it SHALL load the whole catalogue of the selected
  entry.

- **FR-CACHE-023**: `tpl cache clean` SHALL remove cached data for the selected
  entry. With no object flag it SHALL remove all of it.

- **FR-CACHE-024**: `tpl cache load` and `tpl cache clean` SHALL name an
  individual object with `--table <name>`, `--view <name>`, or
  `--routine <name>` — the same flag spellings `tpl render` uses. `--routine`
  SHALL accept the qualified forms of `FR-SCH-008`, and a bare name matching
  both a procedure and a function SHALL be `64`, per `FR-SCH-010`.

  *Accepted cost.* The cache names an object by flag while `schema` names it
  positionally: two grammars for the same thing in one tree.

- **FR-CACHE-025**: `tpl cache status` SHALL report the database entry, when the
  cache was loaded, and the object counts it holds.

- **FR-CACHE-026**: WHEN the cache for the selected entry is empty,
  `tpl cache status` SHALL exit `0`, per `FR-OUT-033`.

  *Rationale.* Empty is a state, not a failure. This was the first case the
  specification settled that way, and `FR-OUT-033` generalises it to every
  command that returns a collection.

- **FR-CACHE-027**: `tpl cache status` SHALL declare `--format` and `--pretty`.

- **FR-CACHE-034**: `tpl cache status` SHALL emit its `json` output in the
  envelope of `FR-OUT-024`, with `source` set to `project` per `FR-OUT-026`,
  and a `data` carrying exactly the following keys:

  | Key | Value |
  |---|---|
  | `entry` | The name of the selected database entry |
  | `loaded_at` | The load time from `meta.json`, per `FR-CDOC-013`, or `null` when the cache is empty |
  | `collections` | An array of objects, one per collection, each carrying `name`, the count of objects held, and whether the collection was loaded whole, per `FR-CDOC-006` |

  ```json
  {"schema_version":1,"source":"project","data":{"entry":"shop","loaded_at":"2026-09-10T08:14:22Z","collections":[{"name":"tables","count":14,"whole":true}]}}
  ```

  *Rationale.* `source` is `project` and not `cache`, because the command
  reports *on* the cache rather than being served *from* it. `loaded_at`
  appears here and in `meta.json` and nowhere else, per `FR-CDOC-012` and
  `FR-CDOC-013`, and this is the one document in which the age of the cache is
  the answer rather than an incidental detail.

  *Known cost, accepted.* One `loaded_at` for the whole entry understates what
  `BR-CDOC-004` describes: a cache can legitimately hold one table read on
  Monday beside another read on Friday. Carrying a load time per object would
  answer more precisely and is not carried, because `FR-CDOC-013` fixes the
  field at the entry level.

- **FR-CACHE-035**: WHEN the cache for the selected entry is empty, the `data`
  of `tpl cache status` SHALL carry `loaded_at` `null` and `collections` an
  empty array. The exit code is `0`, per `FR-CACHE-026`.

## Invalidation

- **FR-CACHE-028**: Nothing SHALL invalidate the cache automatically. Only
  `tpl cache clean` and `tpl cache load` change what is stored.

- **FR-CACHE-029**: Changing where an entry points — `--host`, `--port`,
  `--user`, `--schema`, `--tls`, or `--dsn` — SHALL NOT invalidate anything.

  ```
  tpl cfg database update shop --host db-staging.example.com
  tpl -d shop cache clean
  ```

- **BR-CACHE-003**: The consequence is accepted and must be documented plainly:
  after repointing an entry, a read serves the previous server's catalogue with
  exit `0`, and nothing in the output says so. The only signal is the load time
  reported by `tpl cache status`.

- **BR-CACHE-004**: The cache changes when it is told to, never on its own. A
  configuration command must not delete cached data as a side effect, and a
  fingerprint compared on the read path would still not cover a hand-edited
  `.cfg`.

## Writing and failure

- **FR-CACHE-030**: The system SHALL write each cached object to its own file,
  through a temporary file in the same directory, renamed over the target.

- **FR-CACHE-031**: The system SHALL NOT take a lock over the cache. Two
  processes writing the same object yield one whole result or the other, never a
  half file, and a killed process leaves nothing locked.

- **FR-CACHE-032**: IF the server is unreachable during `tpl cache load`, THEN
  the system SHALL exit `69` (`EX_UNAVAILABLE`) and SHALL leave everything
  already stored unchanged.

- **FR-CACHE-033**: IF a cache file is unreadable, or carries an unknown format
  version, THEN the system SHALL treat it as a miss: read from the server,
  rewrite the file, and report neither an error nor a warning.

## Dependencies

- [schema-commands.md](schema-commands.md) and
  [render-command.md](render-command.md) — the commands that read through the
  cache.
- [cfg-commands.md](cfg-commands.md) — `cfg database test`, the one command that
  always bypasses the cache.
- [project-and-discovery.md](project-and-discovery.md) — the `.tpl/` layout and
  the writers of it.

## Open questions

- [OQ-021](open-questions.md#oq-021) — the exit code for a failed cache write.
- [OQ-048](open-questions.md#oq-048) — whether an incomplete object may be
  written to the cache.
