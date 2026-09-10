---
title: Configuration Commands
status: approved
last-reviewed: 2026-09-10
related: [configuration-model.md, cache-commands.md, security.md, errors-and-exit-codes.md, server-contract.md]
---

# Configuration Commands

## Overview

`tpl cfg` is the single top-level group that manages `.tpl/.cfg`. It has two
arms of its own: dotted keys, for any single value, and an entry subgroup, so
that registering a database is one invocation rather than five.

## Scope

In scope: the `cfg` command tree, the semantics of each subcommand, the
flag-to-key mapping, redaction rules, and which of these commands may contact a
server.

Out of scope: the content and validation rules of the `.cfg` file itself, which
belong to [configuration-model.md](configuration-model.md).

## Actors

- **Calling agent** or **operator**.
- **Project**, whose `.tpl/.cfg` file is the only thing these commands write.

## Command surface

```
tpl cfg get   <key>
tpl cfg set   <key> <value>
tpl cfg unset <key>
tpl cfg list

tpl cfg database add    <name> [flags]        alias: tpl cfg db …
tpl cfg database list
tpl cfg database show   <name>
tpl cfg database update <name> [flags]
tpl cfg database remove <name>
tpl cfg database test   <name>
```

- **FR-CFG-001**: `tpl cfg` and `tpl cfg database` SHALL be group nodes, per
  `FR-CLI-007`.

- **FR-CFG-002**: `tpl cfg database` SHALL carry the alias `db`.

- **FR-CFG-003**: The system SHALL NOT provide a top-level `database` group.
  `tpl database …` is `64` with a nearest-match suggestion, per `FR-ERR-019`.

  *Rationale.* A top-level group named `database`, whose verbs are `add`,
  `update`, and `remove`, reads as a tool that mutates a database — in a tool
  whose central invariant is that it never issues a write statement. These
  commands touch `.tpl/.cfg` and nothing else. `cfg` is named after the file it
  owns.

- **FR-CFG-004**: No `cfg` subcommand SHALL write anywhere but `.tpl/.cfg`.

- **FR-CFG-005**: No `cfg` subcommand SHALL contact a server, with the single
  exception of `tpl cfg database test`.

## Key commands

- **FR-CFG-006**: `tpl cfg get <key>` SHALL print the value stored under that
  key, as written in the file, without expanding `${VAR}` and without redaction.

  ```
  tpl cfg get database.reporting.password
  hunter2

  mysql -u reader -p"$(tpl cfg get database.reporting.password)"
  ```

  *Rationale.* `cfg get` is a directed read: whoever types the key name knows
  what they are asking for. Redacting here would leave no way to feed a password
  to another command.

- **FR-CFG-007**: IF the key supplied to `tpl cfg get` is absent from the file,
  THEN the system SHALL exit `66` (`EX_NOINPUT`) with a nearest-match suggestion
  over the keys that do exist.

  *Rationale.* Exiting `0` with empty output would be indistinguishable from a
  key whose value is empty.

- **FR-CFG-008**: `tpl cfg set <key> <value>` SHALL write the value under that
  key.

- **FR-CFG-009**: `tpl cfg set` SHALL accept only the enumerated key space of
  `FR-CONF-002`. IF the key is not in that space, THEN the system SHALL exit
  `64` (`EX_USAGE`) with a nearest-match suggestion over the known keys.

- **FR-CFG-010**: `tpl cfg set` SHALL validate the supplied value against the
  type declared for that key in `FR-CONF-002`, and SHALL exit `64` if it does
  not conform.

- **FR-CFG-011**: `tpl cfg unset <key>` SHALL accept either a leaf key, such as
  `database.shop.host`, or a whole block, such as `database.shop`, and SHALL
  delete what it is given.

- **FR-CFG-012**: IF the key or block supplied to `tpl cfg unset` is absent,
  THEN the system SHALL exit `66`.

- **FR-CFG-013**: `tpl cfg list` SHALL print the contents of `.tpl/.cfg`
  literally, with passwords redacted per `FR-CFG-021`.

- **FR-CFG-014**: `tpl cfg list` SHALL NOT resolve the configuration. It SHALL
  NOT expand `${VAR}`, SHALL NOT run `password_command`, and SHALL NOT apply
  defaults.

  *Rationale.* Printing the effective configuration would be better for
  diagnosing precedence, but resolving in order to print means running a child
  process and requiring the environment, so `cfg list` could fail with `78` or
  spawn something.

## Entry commands

- **FR-CFG-015**: `tpl cfg database add <name>` SHALL create a
  `[database.<name>]` block from the flags supplied.

- **FR-CFG-016**: `tpl cfg database add` SHALL require either `--dsn` or at
  least one of the discrete connection flags. The two groups are mutually
  exclusive, and at least one is required. IF neither is supplied, THEN the
  system SHALL exit `64`.

- **FR-CFG-017**: IF `tpl cfg database add` names an entry that already exists,
  THEN the system SHALL exit `64`, with a hint pointing at
  `tpl cfg database update`.

- **FR-CFG-018**: `tpl cfg database list` SHALL print the names of the entries
  defined in the file.

  *Provenance.* Root `README.md`; not contradicted by any decision.

- **FR-CFG-019**: `tpl cfg database show <name>` SHALL print that entry, with
  passwords redacted per `FR-CFG-021`, and SHALL NOT expand `${VAR}`.

- **FR-CFG-020**: `tpl cfg database update <name>` SHALL change the fields named
  by the flags supplied, leaving the rest of the entry untouched.

- **BR-CFG-001**: `add` creates and `update` changes. Neither silently does the
  other's job: there is no `--force` that replaces wholesale, and no idempotent
  `add` that would make the two verbs synonyms.

- **FR-CFG-021**: WHEN printing configuration, `tpl cfg list` and
  `tpl cfg database show` SHALL redact secrets as follows:

  | Stored value | Printed as |
  |---|---|
  | a literal password | `***` |
  | the password inside a DSN | `***`, with user, host, port and database left visible |
  | `${VAR}` in any field | `${VAR}`, exactly as written |

  *Rationale.* Printing `${VAR}` unexpanded means a password living in an
  environment variable never reaches stdout through these two commands.

- **BR-CFG-002**: `tpl cfg get` is the one deliberate exception to redaction,
  and it is written down here rather than left to be discovered.

- **FR-CFG-022**: `tpl cfg database remove <name>` SHALL delete that entry.

- **FR-CFG-023**: WHEN `tpl cfg database remove` deletes the entry named by
  `core.database`, the system SHALL also clear `core.database`, silently,
  leaving the file coherent.

  *Rationale.* The next invocation without `-d` then fails with `78`, "no
  database entry selected", which is the correct message. A stderr warning would
  not be seen by a caller checking only the exit code, and refusing with `64`
  until the reference is cleared by hand would be worse.

- **FR-CFG-024**: `tpl cfg database test <name>` SHALL perform exactly the
  following four steps, in this order, and SHALL report the outcome of each:

  1. connect to the server described by that entry and authenticate;
  2. enforce the read-only session of `FR-SRV-008` and confirm it under
     `FR-SRV-009`;
  3. verify that the server is a supported MariaDB series, per `FR-SRV-034`;
  4. run the catalogue privilege probe of `FR-CFG-044`.

  *Amended in the fourth edition.* The third step is new. `FR-SRV-034` attaches
  the product check of `FR-SRV-003` and the version check of `FR-SRV-020` to
  opening a connection rather than to reading a catalogue, and this command is
  the only one that opens a connection and reads no catalogue into the model,
  per `FR-CACHE-010` — so it was the only one the earlier wording of
  `FR-SRV-002` let through. An entry that reaches a server every other command
  refuses must not be reported as working here.

  *Amended in the fifth edition.* The fourth step is new, and the steps are
  numbered because the command reports one field per step, per `FR-CFG-039`.
  `OQ-002` asked whether this command reports the reader's effective
  privileges; the answer is yes, as the single boolean `can_read_catalogue`
  produced by the probe of `FR-CFG-044`.

- **FR-CFG-043**: IF the server the entry reaches is not a supported MariaDB
  series, THEN `tpl cfg database test` SHALL exit `78` with the message of
  `FR-SRV-030`, whose `cause` states that the connection and the authentication
  succeeded.

  *Rationale.* This command exists to tell a caller which of three things is
  wrong, and `78` is now one of four outcomes it can report — `0`, `69`, `77`,
  `78`. Without the `cause` line of `FR-SRV-030` the caller could not separate
  "your host is unreachable" from "your credentials are refused" from "your
  network and credentials are fine and your server is too old", which are three
  different next steps. The distinction is stated in the message rather than in
  a fourth exit code because `FR-ERR-001` fixes the code set and `FR-SRV-020`
  already places this condition on `78`.

  *Amended in the fifth edition.* The requirement previously also named
  `kind: server_version_unsupported`. `FR-ERR-015` withdraws that field, so the
  `cause` line of `FR-SRV-030` is the whole of the distinction — which is what
  this rationale already said carried it, and what `FR-ERR-034` now makes an
  obligation rather than an intention.

  A server newer than the supported window is **not** a failure here: it is read
  under `FR-SRV-031`, so this command exits `0` and reports it, per
  `FR-CFG-039`.

- **FR-CFG-025**: `tpl cfg database test` SHALL NOT read or write the cache, per
  `FR-CACHE-010`.

- **FR-CFG-026**: `tpl cfg database test` SHALL declare `--format` and
  `--pretty`.

### The catalogue privilege probe

- **FR-CFG-044**: The fourth step of `FR-CFG-024` SHALL be exactly one
  `SELECT` against `INFORMATION_SCHEMA`, restricted to the server-side database
  the entry names, whose result the system SHALL NOT read as model content and
  SHALL read only for two facts: whether the statement was answered without a
  privilege error, and whether it returned at least one row. The system SHALL
  set `can_read_catalogue` to true WHEN both hold, and to false otherwise.

  *What the probe proves.* That this reader, on this connection, can see the
  named database in the catalogue at all. That is the fact a caller needs from
  a connectivity check and cannot obtain any other way short of attempting the
  read it is about to make.

  *What the probe does not prove.* That any particular object is readable, that
  any particular property of an object is readable, or that a later read will be
  complete under `FR-PRIV-001`. Completeness is a property of a read, per
  `BR-PRIV-003`, and only a read establishes it.

  *Rationale.* The probe is a boolean and not a privilege listing because
  anything finer is a field list, and a field list about a catalogue nobody has
  observed cannot be written: `scripts/mariadb/` does not exist in this
  repository, and `OQ-009`, `OQ-010` and `OQ-024` are blocked by that absence.
  A probe whose answer depended on the container would have dragged this
  question behind the same block that holds those three, which is exactly what
  answering `OQ-002` was meant to avoid. One statement also keeps the command
  inside `NFR-PERF-002`: the probe's cost does not grow with the number of
  objects.

  *Within the closed list.* The probe is a `SELECT` against
  `INFORMATION_SCHEMA.*` and is therefore already the first entry of
  `FR-SRV-006`. It widens nothing, and `FR-SRV-007` is unaffected.

  *Not an attempt-and-fall-back read.* `FR-SRV-023` forbids discovering a
  difference between server series by attempting a read and handling its
  failure, and this probe attempts a read and reports whether it failed. The
  two are distinct on both grounds `FR-SRV-023` gives. The probe issues one
  statement, always the same one, always issued — so nothing is sent that the
  closed list does not contain, and the statement count does not vary with the
  server, which is what `NFR-PERF-001` and `NFR-PERF-002` fix. And what it
  reports is a property of the **reader**, which no other statement can
  establish, rather than a property of the **series**, which `FR-SRV-022`
  determines from the version probe without attempting anything.

  *Rejected.* Probing by reading a listing — the tables of the selected
  database — which answers the same boolean and makes the command's cost, and
  its correctness, depend on the field list `OQ-024` holds open. Also rejected:
  reading the reader's granted privileges directly, which is a different
  catalogue whose shape is equally unobserved and which reports what was granted
  rather than what this connection can see.

- **FR-CFG-045**: A `can_read_catalogue` of false SHALL NOT change the exit
  code. `tpl cfg database test` SHALL exit `0` and report it.

  *Rationale.* This command's purpose is to say which of several things is
  wrong with an entry, and a diagnostic that refuses to answer is less useful
  than one that answers and says so — the asymmetry `BR-PRIV-001` already
  settles for a listing. A false that exited `77` could never be observed in
  the success document of `FR-CFG-039`, which would make the field dead
  surface.

  `FR-PRIV-003` does not apply: it fails a request that **named** an object,
  and the probe names none. A `77` from this command therefore still means what
  it meant before — the server refused the authentication, at step 1 of
  `FR-CFG-024`.

  *Accepted cost.* Exit `0` no longer means "this entry is fully usable"; it
  means "the four steps ran and here is what each returned". A caller that
  branches on the exit code alone and needs the fourth answer must read
  `can_read_catalogue`. The help of the command states this, and the `EXAMPLES`
  section shows the field being read.

## Flags of `add` and `update`

- **FR-CFG-027**: `tpl cfg database add` and `tpl cfg database update` SHALL
  declare the following flags, each mapping to one key of the entry:

  | Flag | Key |
  |---|---|
  | `--dsn <url>` | `database.<name>.dsn` |
  | `--host <host>` | `database.<name>.host` |
  | `--port <port>` | `database.<name>.port` |
  | `--user <user>` | `database.<name>.user` |
  | `--schema <name>` | `database.<name>.database` |
  | `--tls <mode>` | `database.<name>.tls` |
  | `--password-command <command>` | `database.<name>.password_command` |
  | `--ca-file <path>` | `database.<name>.ca_file` |
  | `--ca-path <path>` | `database.<name>.ca_path` |

  *Amended in the fifth edition.* The last three rows are new, and close
  `OQ-017`. The three keys were already in the key space of `FR-CONF-002` and
  were reachable only through `tpl cfg set`, so registering an entry that used
  a keychain lookup and a private certificate authority took three invocations
  where the mapping promised one. Every key of an entry now has a flag.

- **FR-CFG-046**: `--password-command` SHALL accept a single string and SHALL
  store the array `FR-CONF-025` splits it into. It SHALL NOT accept an array on
  the command line, and SHALL NOT be repeatable.

  *Rationale.* `FR-CONF-023` fixes the stored form as an array and `FR-CONF-025`
  already fixes the splitting rule for a string supplied to a command; this flag
  is that rule's caller. A repeatable flag accumulating one argument per
  occurrence would be a second way to build the same array, and `FR-CLI-014`
  makes a repeated single-value flag `64` in any case.

- **FR-CFG-047**: None of the three flags of the amendment above SHALL carry a
  short form, per `FR-GLOB-024`.

- **FR-CFG-028**: `--schema` SHALL name the database on the server. It is the
  only flag whose name differs from the key it writes.

  *Rationale.* `table_schema` is the catalogue's own word for it, so `--schema`
  reads as the server side without inventing vocabulary, and it leaves the
  global `-d/--database` free to mean the entry label across the whole tree.
  Two definitions of `--database` cannot coexist at one node.

  *Accepted cost.* The word `schema` now names two things: the first arm and
  this flag. Help text must disambiguate wherever both could be meant.

- **FR-CFG-029**: `--dsn` SHALL be mutually exclusive with the discrete
  connection flags in one invocation.

- **FR-CFG-030**: The system SHALL NOT declare a `--password` or `-p` flag on
  any command.

- **FR-CFG-031**: `--dsn` SHALL accept whatever the caller writes, including a
  literal password, and SHALL store it verbatim.

- **FR-CFG-032**: `tpl cfg set` SHALL accept a literal password written to
  `database.<name>.password`.

- **FR-CFG-033**: The help of `--dsn`, `--password-command`, `--ca-file`,
  `--ca-path`, and `tpl cfg set` SHALL state that a value given on the command
  line is visible in the process table. WHERE the key the flag writes admits
  `${VAR}` expansion under `FR-CONF-015`, that help SHALL also recommend
  `${VAR}` instead.

  *Amended in the fifth edition.* The three flags `FR-CFG-027` gained inherit
  this warning, which is why they are named here. The `${VAR}` recommendation
  is now conditional because it cannot be given for all of them: `FR-CONF-017`
  forbids expansion in `password_command`, and `FR-CONF-015` does not admit it
  in `ca_file` or `ca_path` either. For those three the honest advice is the
  bare fact: the value is visible in the process table for the life of the
  invocation, and no expansion alternative exists. `BR-CFG-003` governs — `tpl`
  warns; it does not prevent.

- **BR-CFG-003**: `tpl` warns; it does not prevent. Putting a secret in the
  argument vector is the caller's decision. What `tpl` guarantees is that no
  flag named `password` exists, and that the two remaining paths are documented
  rather than accidental.

- **FR-CFG-034**: WHEN a `cfg` command rewrites `.tpl/.cfg`, the file SHALL
  retain mode `0600`.

  *Rationale.* `FR-PROJ-019` creates the file at `0600` and `FR-PROJ-011`
  refuses to read it at any looser mode; a command that loosened it would break
  the next invocation.

  *Moved in the fifth edition.* This rationale stood under `FR-CFG-042`, where
  it argued for a mode rather than for the absence of a lock. It is the
  argument for this requirement and now stands under it.

- **FR-CFG-041**: WHEN a `cfg` command rewrites `.tpl/.cfg`, it SHALL write a
  temporary file in `.tpl/` at mode `0600` and SHALL rename it over the target.
  A failure part-way through SHALL leave the previous `.cfg` in place,
  unchanged.

  *Rationale.* This is the rule `FR-CACHE-030` already applies to the other
  thing `tpl` writes, and it matters more here: a truncated `.cfg` is a `78` on
  every subsequent invocation, and it holds the credentials without which the
  project cannot reach a server. A cached object lost to a truncated write is
  recovered by reading the server again; a lost `.cfg` is not recovered at all.

- **FR-CFG-042**: The system SHALL NOT take a lock over `.tpl/.cfg`. Two
  processes rewriting it yield one whole file or the other, never a half file,
  and a killed process leaves nothing locked.

  *Rationale.* The same argument `FR-CACHE-031` makes for the cache, and it
  applies with more force here: a stale lock on `.cfg` would make every command
  fail, including the `tpl cfg unset` that would clear it.

  *Accepted cost.* Two concurrent `tpl cfg set` invocations on different keys
  can lose one of the two writes. Both files are whole and valid; the later
  rename wins.

## `json` output

- **FR-CFG-035**: Every `cfg` subcommand that declares `--format` SHALL emit
  its `json` output in the envelope of `FR-OUT-024`, with `source` set to
  `project` per `FR-OUT-026` — except `tpl cfg database test`, whose `source`
  SHALL be `server`, because contacting the server is what the command does,
  per `FR-CACHE-010`.

- **FR-CFG-036**: The `data` of `tpl cfg get` SHALL carry `key` and `value`,
  the value as written in the file, unexpanded and unredacted per `FR-CFG-006`:

  ```json
  {"schema_version":1,"source":"project","data":{"key":"database.shop.host","value":"db.example.com"}}
  ```

- **FR-CFG-037**: The `data` of `tpl cfg list` SHALL mirror the key space of
  `FR-CONF-002` as nested objects — `core`, and `database` keyed by entry name
  — with the redaction of `FR-CFG-021` applied and `${VAR}` left exactly as
  written, per `FR-CFG-014`. A key absent from the file SHALL be absent from
  the document rather than emitted as its default, because `FR-CFG-014`
  forbids applying defaults.

  *Rationale.* This is the one place `FR-OUT-012` does not apply. Emitting an
  unset key as `null` would be indistinguishable from a key written with an
  empty value, and `tpl cfg get` already answers `66` for the difference, per
  `FR-CFG-007`.

- **FR-CFG-038**: The `data` of `tpl cfg database list` SHALL carry one key,
  `entries`, per `FR-OUT-030`, whose value is an array of objects each
  carrying `name`. The `data` of `tpl cfg database show` SHALL carry one key,
  `entry`, per `FR-OUT-031`, whose value is that entry with the redaction of
  `FR-CFG-021` applied.

- **FR-CFG-039**: The `data` of `tpl cfg database test` SHALL carry `entry`,
  `connected`, `read_only_session`, `server`, and `can_read_catalogue`, in that
  order, reporting the outcome of the four steps `FR-CFG-024` requires the
  command to perform:

  ```json
  {"schema_version":1,"source":"server","data":{"entry":"shop","connected":true,"read_only_session":true,"server":{"version":"11.4.5-MariaDB","series":"11.4","standing":"supported"},"can_read_catalogue":true}}
  ```

  `server` SHALL be the object of `FR-CTX-031`, with the same three keys and the
  same meanings. `can_read_catalogue` SHALL be the boolean the probe of
  `FR-CFG-044` produces.

  *Rationale.* A `test` that reaches exit `0` reports the outcome of all four
  steps; a failure exits `69`, `77`, or `78` instead and emits the four-line
  text diagnostic of `FR-ERR-008`, per `FR-ERR-033`. The fields are named rather
  than implied so that a caller can branch on them without inferring from the
  exit code alone.

  *Amended in the fourth edition.* `server` is new, and it is required by this
  requirement's own rationale rather than by a new decision: `FR-CFG-024` gained
  a third step, and a step whose outcome is not reported cannot be branched on.
  It carries the whole object rather than a bare boolean because `standing` is
  the one field that distinguishes the two ways this command can exit `0` — a
  supported series, and a series newer than the window read under `FR-SRV-031`.
  Adding a field is non-breaking, per `FR-OUT-014`.

  *Amended in the fifth edition.* `can_read_catalogue` is new, and closes
  `OQ-002`. The command exists to tell a caller which of several things is
  wrong with an entry, and "the credentials work but this reader cannot see the
  catalogue" was the one outcome it could reach and not report. It is a single
  boolean rather than a privilege listing for the reason `FR-CFG-044` gives:
  anything finer would be a field list, and a field list cannot be written until
  the container of `scripts/mariadb/` exists. Adding a field is non-breaking,
  per `FR-OUT-014`.

  *Accepted cost.* `can_read_catalogue: true` does not promise that a
  subsequent read is complete. It promises exactly what `FR-CFG-044` says the
  probe proves and no more; completeness is `FR-PRIV-001` and only a full read
  establishes it. The help of this command states the difference.

- **FR-CFG-040**: WHEN a `cfg` listing is empty — `tpl cfg database list` in a
  project with no entry, which `FR-PROJ-018` makes the ordinary first state —
  the system SHALL exit `0` with an empty array, per `FR-OUT-033` through
  `FR-OUT-035`.

## Dependencies

- [configuration-model.md](configuration-model.md) — the key space, the value
  types, and the validation these commands apply.
- [cache-commands.md](cache-commands.md) — why repointing an entry does not
  invalidate the cache.
- [errors-and-exit-codes.md](errors-and-exit-codes.md) — `64`, `66`, `78`.
- [server-contract.md](server-contract.md) — `FR-SRV-034`, which brings
  `database test` under the two server checks; `FR-SRV-030`, the message
  `FR-CFG-043` emits; and `FR-SRV-006`, the closed statement list the probe of
  `FR-CFG-044` falls inside.
- [privileges-and-completeness.md](privileges-and-completeness.md) —
  `FR-PRIV-001` and `BR-PRIV-003`, the completeness the probe of `FR-CFG-044`
  does not establish, and `FR-PRIV-003`, which `FR-CFG-045` explains does not
  reach the probe.

## Open questions

None specific to this module. The four this file carried are all closed and
listed under [Closed](open-questions.md#closed): `OQ-002` by `FR-CFG-044`,
`FR-CFG-045` and the `can_read_catalogue` field of `FR-CFG-039`; `OQ-016` by
`FR-GLOB-024`; `OQ-017` by the three flags `FR-CFG-027` gained; and `OQ-074`,
the version gate, by `FR-CFG-043` and the `server` field of `FR-CFG-039`.

