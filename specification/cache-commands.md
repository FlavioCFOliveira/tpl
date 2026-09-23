---
title: Catalogue Cache
status: approved
last-reviewed: 2026-09-23
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
write atomicity, what is never written, and the failure modes the model
accepts.

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

  *Note added in the fortieth edition.* Under `FR-CACHE-038` a render no longer
  settles whether it is a hit before it starts. An invocation is a hit only if
  no file it reads is a miss. An invocation that abandons a render under
  `FR-CACHE-039` is a miss, and the connection it then opens is the first and
  only one of the invocation.

- **FR-CACHE-007**: WHEN a cached read misses, the system SHALL read the server,
  SHALL write the result to the cache except for the objects `FR-CACHE-037`
  excludes, and SHALL then answer. The write happens before the answer.

  *Checked in the thirty-first edition against an invocation that fails after
  the write, and unchanged.* A `tpl render` whose template carries a syntax
  error refuses at step 8 of `FR-ERR-006`, and the catalogue it read at steps 6
  and 7 is in the store by then, because this requirement puts the write before
  the answer. The question put was whether the store should be left untouched
  by an invocation that goes on to fail. It should not, and the cost stands
  with its ground stated here rather than an exemption being written in.

  *Why the cost stands.* What is written is a **correct read of the server**,
  and nothing downstream of it makes it wrong: the template that failed to
  compile is not a fact about the catalogue. The ordinary next action after a
  `65` from `FR-RND-030` is to correct the template and render again, against
  the same entry and the same objects — which the filled store serves without
  a connection, per `NFR-PERF-003`. An exemption would therefore spend a
  correct read in order to make the caller pay for it a second time on the very
  next invocation they make. A caller who does not want the store written has
  `--no-cache`, per `FR-CACHE-014`, and `--direct --no-cache` is the pure read
  of `FR-CACHE-016`.

  *Rejected: exempting a read whose invocation has already failed.* At the
  moment the write happens the invocation has **not** failed — step 8 has not
  run — so the exemption has no condition it could test. Making it testable
  would mean holding the read in memory until the render has succeeded and
  writing it afterwards, which is a second ordering of the same two operations
  and reaches the store only on the path where the store was least needed.

  *Rejected: writing the store only on success, as a general rule.* It moves
  the write behind every fallible step, so a `74` on stdout — a consumer that
  closed the pipe, per `FR-ERR-026` — would also discard a catalogue that was
  read correctly, and `tpl schema dump | head -1` would leave the store empty
  every time. The one case that raised the question would be answered by
  making every case worse.

- **FR-CACHE-038**: WHEN `tpl render` reads through the cache and every
  collection is recorded as whole under `FR-CDOC-006`, the system SHALL read
  `database.json`, the listing of each collection, and the bound object's file
  before the render starts. It SHALL read every other object file no earlier
  than the first time the template reaches that object. WHERE the path of every
  object file names the object that file holds, the `database` the template
  sees SHALL be the document an up-front read of the same files would produce:
  the same members, the same names, the same order under `NFR-DET-002`, and the
  same value under every filter, test and function. Any other object file is a
  miss under `FR-CACHE-033`.

  A lookup by name — the tests `primary_key` and `unique` of `FR-ENV-015` and
  the functions `table`, `view`, `routine` and `column` of `FR-ENV-020` —
  SHALL resolve the name from the collection's listing and SHALL reach only the
  object it returns. An object the lookup passes over is not reached by it, and
  a lookup that returns no object reaches none.

  *Added in the fortieth edition, as decided for rmp `#246`.* A cached render
  read and decoded every object file of its entry, whatever the template
  reached. The user chose lazy loading: the `database` a template sees stays
  whole, and an object's file is read only when the template reaches that
  object. The reading that raised the question is recorded in `BENCHMARKS.md`;
  it is informative, per `BR-PERF-008`, and the requirement rests on the
  identical document, not on the figure.

  *Why the bound object's file is read up front.* Every render that binds an
  object reaches it, so reading it first costs nothing, and a miss on it is then
  decided before the render starts, at the cache-or-connection step of
  `FR-ERR-006`, exactly as before this edition.

  *Rejected: narrowing the context to the bound object.* It removes the rest of
  `database`, which `FR-RND-023` binds whatever object the invocation names.
  *Also rejected: validating every file up front and serving lazily.* It keeps
  the cost this requirement removes.

  *Amended within the fortieth edition: the equivalence is limited to files
  whose path names what they hold.* An up-front read orders the members of a
  collection by the names inside their files. This requirement forbids reading
  any file but the bound object's before the template reaches it, so a lazy
  read names and orders the members by their paths. The two agree for every file `tpl` writes, because
  each object is written under the path its own name composes, per
  `FR-CACHE-030` and `FR-CDOC-014`. They disagree for two inputs `tpl` never
  writes: an object file whose content names an object other than the one its
  path names, and
  a file in a collection's folder whose name no object's path could take.
  `FR-CACHE-033` makes both a miss, so neither is served, and the equivalence
  holds over everything that is.

  *`BR-CACHE-001` is not contradicted.* That the path of an object file names
  what the file holds is a property of the arrangement versioned by
  `cache_format`, per `FR-CDOC-002`, which the binary checks before it serves,
  as it checks the versions under `FR-CDOC-004`. No caller relies on it, and
  the layout stays outside the plumbing contract.

  *Rejected: reverting to the up-front read.* It would restore the equivalence
  for files `tpl` never writes by reading every file of the entry on every
  render, which is the cost this requirement exists to remove.

  *Amended in the forty-first edition: a lookup by name reaches only the object
  it returns, as decided for rmp `#248`.* The requirement did not say whether a
  lookup reaches the objects it passes over on its way to the one it returns,
  and a lookup that scanned the collection read the file of every object listed
  before the wanted one. The name a lookup matches is already known without
  reading any file: under the equivalence above, the names the listing carries
  are the names the files hold. So the lookup's answer, including the answer
  that no object of that name exists, depends on the listing alone, and only
  the returned object's file is read. A damaged file the lookup passes over is
  not consulted, so it is not a miss of that invocation, per `FR-CACHE-033`. A
  damaged file the lookup returns is a miss, answered under `FR-CACHE-039`. The
  reading that raised the question is recorded in `BENCHMARKS.md`; it is
  informative, per `BR-PERF-008`, and the rule rests on the identical answer,
  not on the figure.

  *Rejected: a lookup reaches every object it passes over.* It gives the same
  answer at the cost of reading, on every lookup, the file of each object
  listed before the wanted one, and it would make a damaged file the template
  never uses a miss only because its name sorts earlier.

- **FR-CACHE-039**: IF a file read under `FR-CACHE-038` during the render is a
  miss under `FR-CACHE-033`, including a file removed after the listing was
  read, THEN the system SHALL abandon the render, SHALL treat the invocation as
  a miss under `FR-CACHE-007`, and SHALL render again from the server's
  document. No byte of the abandoned render SHALL reach stdout. The invocation
  SHALL open at most one connection, per `NFR-PERF-004`. The render that
  produces the result SHALL have the whole deadline of `FR-CONF-005`, and SHALL
  use the value of `now` evaluated for the abandoned render.

  The abandoned render SHALL keep every render bound of `FR-RND-038` — the
  deadline, render fuel, the render output limit and the render memory limit —
  until it has returned, whether or not it evaluates further after the miss.
  IF the abandoned render crosses any of them, before the miss or after it,
  THEN the system SHALL end the invocation with `65` (`EX_DATAERR`) and the
  `cause` of the first bound crossed, per `FR-RND-038`, SHALL NOT read the
  server, and SHALL let no byte of the abandoned render reach stdout, per
  `FR-RND-034`. The server read and the second render follow only an abandoned
  render that returned within every bound.

  *Added in the fortieth edition.* The miss is answered exactly as a miss found
  before the render is answered: one whole read of the server, the write of
  `FR-CACHE-007`, and a render from what the server returned. The output is
  therefore the output the invocation would have produced had the miss been
  found up front. `--no-cache` still suppresses the write, per `FR-CACHE-014`.

  *Consequence.* A condition the abandoned render raised before it reached the
  miss is reported as the first failure, per `FR-ERR-006`, and no connection is
  opened. A condition of the cache-or-connection step or of catalogue object
  resolution raised after the render is abandoned is reported with that step's
  code. A render bound the abandoned render crosses is a render condition too,
  before the miss or after it: the invocation ends with `65` and that bound's
  `cause`, with no connection opened, per the second paragraph of this
  requirement.

  *Amended in the forty-second edition.* The second paragraph of the
  requirement, and the last sentence of the consequence above, are new. A render
  that reaches the miss through a lookup function of `FR-ENV-020` — `table`,
  `view`, `routine` or `column` — can go on evaluating after the miss, and no
  requirement said what happened if it then crossed a bound. Left unwatched,
  such a render was observed to reach 8.6 GB past a one-second deadline.
  *Rejected: discarding the abandoned render and reading the server at once.*
  A running render cannot be stopped short of ending the process, so it would
  keep consuming, unbounded, beside the read and the second render.
  *Also rejected: making a lookup-function miss unwind the evaluation at
  once.* It would change the contract of the lookup functions, whose answer
  `FR-CACHE-038` and `FR-ENV-017` fix.

  *Rejected: fetching only the missing object and continuing.* The document
  would mix sources within one render, and `NFR-PERF-004` would hold only if
  the connection stayed open for the rest of the render, which `FR-RND-040`
  now forbids: the connection the read opens is closed, and the driver's
  runtime shut down, before the render that follows starts. *Also rejected:
  failing the invocation.* It fails a read that can succeed, which is what
  `FR-CACHE-033` forbids.

- **FR-CACHE-008**: The system SHALL NOT apply a time-to-live and SHALL NOT
  expire an entry automatically. A cached object stays until `tpl cache clean`
  removes it or a fresh read replaces it.

  *Rationale.* An authoritative-only cache would make the first invocation on a
  new project fail; a TTL would make the read source depend on the clock; an
  opt-in flag would mean the cache is used only by callers who remember to ask.

- **FR-CACHE-009**: The following commands SHALL read through the cache: the
  eight `schema` subcommands, and `tpl render` when it has no `--context`.

- **FR-CACHE-010**: `tpl cfg database test` SHALL always contact the server,
  SHALL neither read nor write the cache, and SHALL read nothing into the
  model. The one statement it issues against the catalogue is the privilege
  probe of `FR-CFG-044`, whose result is a boolean and not model content.

  *Rationale.* Contacting the server is the command's only reason to exist, and
  it must reach a verdict without a cache standing between it and the server.

  *Amended in the fifth edition.* The last clause is new, and it is a
  correction rather than an addition. Three passages in this corpus cite this
  requirement for the proposition that `tpl cfg database test` is the one
  command that opens a connection and reads no catalogue — `FR-CFG-024`,
  `FR-SRV-002`, and `FR-SRV-034`. `FR-CFG-044` gives the command a catalogue
  statement, so the proposition as those passages state it stopped being true
  the moment `OQ-002` was answered. What remains true, and what those passages
  rely on, is that nothing the command reads enters the model: no object, no
  field, nothing cacheable. This requirement now says that, so the three
  citations resolve to something that is the case.

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
  | `collections` | An array of objects, one per collection, each carrying `name`, `count` — the number of object files the collection holds, as defined below — and whether the collection was loaded whole, per `FR-CDOC-006` |

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

  The `count` of a collection SHALL be the number of object files present in
  that collection's folder: every file that the on-disk arrangement versioned
  by `cache_format`, per `FR-CDOC-002`, names as an object of that collection,
  whether or not its content is readable, is valid UTF-8, or decodes as a
  document. `tpl cache status` SHALL NOT open an object file to count it. A
  temporary file of a write in flight, per `FR-CACHE-030`, and any other file
  that is not an object file SHALL NOT be counted.

  *Amended in the thirty-eighth edition.* The requirement said "the count of
  objects held" and left open whether an object file whose content cannot be
  read is held. The user decided that it is.

  *Rationale.* The count reports what the cache holds, not what it can serve.
  Whether a file can serve is decided at read time, where `FR-CACHE-033` makes
  an unreadable file a miss and rewrites it. Counting by content would make a
  status report read every cached byte to produce a number.

  *Accepted cost.* A count can include a file that the next read treats as a
  miss, per `FR-CACHE-033`, until that read rewrites it.

- **FR-CACHE-035**: WHEN the cache for the selected entry is empty, the `data`
  of `tpl cache status` SHALL carry `loaded_at` `null` and `collections` an
  empty array. The exit code is `0`, per `FR-CACHE-026`.

## Invalidation

- **FR-CACHE-028**: Nothing SHALL invalidate the cache automatically: no
  clock, no configuration change and no comparison with the server SHALL
  remove or replace a cached object. What is stored SHALL change only through
  an invocation that stores what it reads or removes what is stored:
  `tpl cache load`, `tpl cache clean`, and a read command of `FR-CACHE-009`
  that writes the cache under `FR-CACHE-015`.

  *Amended in the forty-fourth edition,* for rmp `#261`. The second sentence
  named `tpl cache clean` and `tpl cache load` as the only commands that change
  what is stored, while `FR-CACHE-015` makes a read command write the cache on
  a miss and under `--direct`, unless `--no-cache` is given. The two could not
  both hold. `FR-CACHE-015` governs, as the implementation reads it, and this
  requirement now names every invocation that writes. Its point is unchanged:
  the cache changes only because an invocation changed it, per `BR-CACHE-004`.
  *Rejected: narrowing `FR-CACHE-015` so that only `tpl cache load` writes.*
  Every read would then stay live until an explicit load, which is the opt-in
  cache the rationale of `FR-CACHE-008` rejects.

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
  WHERE the target file already exists and its content is byte-identical to
  the bytes the write would produce, the system MAY leave that file in place
  and skip the temporary file and the rename. A target whose content cannot be
  read, or differs in any byte, SHALL be written through the temporary file and
  the rename. A target that is a symbolic link SHALL NOT be left in place and
  SHALL NOT be followed, whatever it points at: the rename SHALL replace the
  link itself with a regular file.

  *Amended in the forty-second edition, as decided for rmp `#256`.* The last
  sentence is new. The permission above read a byte-identical target through a
  symbolic link as a target to leave in place, which kept a link in the cache
  and left the file it reached outside the cache's control. The sentence
  states the guard on the write, and `FR-CACHE-033` states the same guard on
  the read.

  *Amended in the thirty-ninth edition.* The requirement did not say whether a
  file that already holds exactly the bytes a write would produce must still be
  replaced. It need not be, as decided for rmp `#244`.

  *Rationale.* The observable result is the same either way. The file holds
  the same bytes, every later read of it serves the same document, and
  `tpl cache status` counts the same files, per `FR-CACHE-034`. The rename
  changes only the file's modification time, and no output of `tpl` reports
  it: the one load time is `loaded_at`, which lives in `meta.json` and in the
  output of `tpl cache status` and nowhere else, per `FR-CDOC-012` and
  `FR-CDOC-013`. This permission does not reach `meta.json`, which is not an
  object file. The reading that raised the question is recorded in
  `BENCHMARKS.md`; it is informative, per `BR-PERF-008`, and the permission
  rests on the identical result, not on the figure.

  *Consequence.* Both paths conform, so neither is contract. `FR-CACHE-031`
  holds on both: a file left in place is whole, and a file renamed over is
  whole. The modification time of an object file is not part of the contract,
  per `BR-CACHE-001`, and no caller or test can rely on it changing, or on it
  staying unchanged, across a write.

  *Accepted cost.* An object file's modification time no longer tells when
  that object was last read from the server. No requirement offered that
  reading.

  *Rejected: requiring the replacement in every case*, which spends a
  temporary file and a rename to produce a state indistinguishable from the
  one already on disk.

- **FR-CACHE-031**: The system SHALL NOT take a lock over the cache. Two
  processes writing the same object yield one whole result or the other, never a
  half file, and a killed process leaves nothing locked.

- **FR-CACHE-032**: IF the server is unreachable during `tpl cache load`, THEN
  the system SHALL exit `69` (`EX_UNAVAILABLE`) and SHALL leave everything
  already stored unchanged.

- **FR-CACHE-033**: IF a cache file is unreadable, or carries an unknown format
  version, THEN the system SHALL treat it as a miss: read from the server,
  rewrite the file, and report neither an error nor a warning.

  The condition SHALL be evaluated for the files a read consults. A file the
  read never consults is not a miss of that invocation and SHALL be left as it
  is. Two reads consult fewer object files than their entry holds: a render
  under `FR-CACHE-038` consults only the objects its template reaches — for a
  lookup by name, only the object the lookup returns — and
  `tpl schema info`, which presents no member of any collection per
  `FR-SCH-031`, opens no object file.

  WHEN a render reads through the cache under `FR-CACHE-038`, two further
  object files SHALL be a miss: a file whose content names an object other than
  the one its path names, and a file in a collection's folder that carries the
  extension of an object file, is not the temporary file of a write in flight
  under `FR-CACHE-030`, and has a name the arrangement versioned by
  `cache_format` gives to no object. The first is found when the file is read:
  before the render starts for the bound object, and otherwise when the template
  reaches the object. The second is found when the listing is read, before the
  render starts, because the listing consults every name in the folder.

  IF a read that serves one named object from its own file under
  `FR-CDOC-008` finds that the object the file holds differs from the object
  requested in its kind or in its name, the names compared byte for byte, THEN
  the file SHALL be a miss. The arrangement of file names versioned by
  `cache_format` is unchanged.

  IF an object file is a symbolic link, THEN it SHALL be a miss for every
  read, whatever the link points at, and the system SHALL NOT read through
  it. The rewrite that follows the miss replaces the link, per
  `FR-CACHE-030`.

  *Amended in the fortieth edition.* The requirement did not say whether a
  file the invocation never consults is covered. Before `FR-CACHE-038` a render
  consulted every file, so the question arose only for `tpl schema info`, whose
  behaviour this clause states without changing it. *Accepted cost:* a damaged
  file that no read consults stays damaged until a read consults it, and a
  render that never reaches it is served from the cache.

  *Amended within the fortieth edition: the second paragraph is new.* It is
  what keeps the equivalence of `FR-CACHE-038` true, and it reaches only files
  `tpl` never writes. A file whose name no object's path could take is found
  before the render starts, so its miss is answered as any miss found then is.
  A file whose content names another object is found the same way when it is
  the bound object's, and otherwise during the render, where its miss is
  answered under `FR-CACHE-039`.

  *Amended in the forty-first edition.* The clause on a lookup by name is new.
  It states for this requirement what `FR-CACHE-038` now fixes: a lookup
  consults only the object it returns, so a damaged file it passes over is not
  a miss.

  *Amended in the forty-second edition: the paragraph on a read of one named
  object is new, as decided for rmp `#254`.* The paragraph before it made a
  file holding another object a miss for a render and said nothing of a read
  that serves one named object,
  and `FR-CDOC-008` serves an object "whenever it is present" without saying
  what present is. The security audit recorded in `SECURITY-AUDIT.md` at the
  repository root found, as its finding SEC-03, that on a filesystem that
  folds case or Unicode normalisation, two objects whose names differ only in
  case or in normalisation share one file, and that a read of the one the file does not hold was served
  from it as a hit and answered that the object does not exist, with a `cause`
  naming a server that was never asked. The object requested was not present,
  so the read was a miss that `FR-CACHE-007` obliges to reach the server. The
  paragraph says so. It closes the false answer on every filesystem, and it
  does not stop the two objects from sharing a file, which the file-naming
  arrangement would have to change to prevent.

  *Accepted cost.* On such a filesystem, a read of either of two colliding
  objects can miss on every invocation, because each rewrite serves one of
  them and replaces the other.

  *Amended in the forty-second edition: the paragraph on a symbolic link is
  new, as decided for rmp `#256`.* A symbolic link where an object file belongs is not
  a file `tpl` writes, per `FR-CACHE-030`, and reading through it would serve
  whatever the link reaches as though the cache held it. The read now applies
  the guard the write applies.

- **FR-CACHE-036**: IF the system cannot write to `.tpl/.cache/`, THEN it SHALL
  answer from what it read, SHALL exit `0`, SHALL leave the cache as it found
  it, and SHALL report neither an error nor a warning. A failed cache write
  SHALL NOT change the exit code and SHALL NOT change a byte of stdout.

  *Rationale.* The read succeeded. The answer the caller asked for is correct,
  complete, and live, and the only thing that did not happen is an
  optimisation. Failing the command would make `.tpl/` being read-only —
  a checked-out worktree, a container with a mounted project, a
  CI runner — turn every correct read into a failure, when `FR-CACHE-016`
  already offers `--direct --no-cache` as the form for exactly that situation
  and the caller should not have to know to reach for it. It is the symmetric
  rule to `FR-CACHE-033`, which treats an unreadable cache file as a miss and
  says nothing: the cache is an optimisation in both directions, and an
  optimisation that fails is not an event in the contract.

  *Rejected.* Exiting non-zero, which fails a correct read; and a warning on
  stderr, which a caller checking the exit code never sees, per `NFR-DET-001`,
  and which would appear on every invocation for as long as the condition
  lasts.

  *Accepted cost, and it is real.* A project whose `.tpl/` is persistently
  unwritable pays the network on every invocation and nothing in any output
  says so. The signal exists but must be sought: `tpl cache status` reports
  what the cache holds, per `FR-CACHE-025`, and a cache that is never written
  reports an empty one with `loaded_at` `null`, per `FR-CACHE-035`. That is
  where the condition shows, per `BR-CACHE-001`, which already makes
  `tpl cache status` the supported way to learn the state of the cache.

- **FR-CACHE-037**: The system SHALL NOT write to the cache any object marked
  `restricted` under `FR-PRIV-005`. A later read of that object SHALL be a
  miss.

  *Rationale.* The cache is keyed by database entry name and by nothing else,
  per `FR-CACHE-002`, and completeness is a property of a read rather than of
  a server, per `BR-PRIV-003`. Together those two make a cached restricted
  object a trap: the stub one reader produced would be served under
  `FR-CDOC-008` to a later invocation, possibly by a reader with sufficient
  privilege, who would receive a marked object where a whole one was available
  and had never been asked for. The marking would be intact and would still be
  wrong, because it would describe a privilege that is not the one in force.

  A collection containing an object excluded by this requirement SHALL NOT be
  recorded as whole under `FR-CDOC-006`. Recording it as whole would make
  `FR-CDOC-007` serve a listing from the cache with the excluded object simply
  missing, which is a wrong answer wearing the appearance of a right one —
  exactly what `BR-CDOC-002` requires the completeness record to prevent.

  *Consequence.* A read by a reader whose privileges are short is correct and
  is not cached; the same read repeated pays the network again, and the listing
  it belongs to is a miss until a reader who can see the whole object warms it.
  That is the right price: the alternative is a cache whose contents depend on
  which credentials happened to warm it.

  *Rejected.* Caching the object with the marking intact, which is the failure
  above. Also rejected: keying the cache by reader as well as by entry, which
  would answer the objection and would make `FR-CACHE-002` false, multiply the
  cache by the number of credentials a project uses, and give `tpl cache clean`
  a scope nobody can see.

## Dependencies

- [schema-commands.md](schema-commands.md) and
  [render-command.md](render-command.md) — the commands that read through the
  cache.
- [cfg-commands.md](cfg-commands.md) — `cfg database test`, the one command that
  always bypasses the cache.
- [project-and-discovery.md](project-and-discovery.md) — the `.tpl/` layout and
  the writers of it.

## Open questions

None specific to this module. `OQ-021` is answered by `FR-CACHE-036` and
`OQ-048` by `FR-CACHE-037`; both are listed under
[Closed](open-questions.md#closed).
