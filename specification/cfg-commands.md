---
title: Configuration Commands
status: draft
last-reviewed: 2026-09-10
related: [configuration-model.md, cache-commands.md, security.md, errors-and-exit-codes.md]
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

- **FR-CFG-024**: `tpl cfg database test <name>` SHALL connect to the server
  described by that entry, enforce the read-only session, and report the result.

- **FR-CFG-025**: `tpl cfg database test` SHALL NOT read or write the cache, per
  `FR-CACHE-010`.

- **FR-CFG-026**: `tpl cfg database test` SHALL declare `--format` and
  `--pretty`.

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

- **FR-CFG-033**: The help of `--dsn` and of `tpl cfg set` SHALL state that a
  value given on the command line is visible in the process table, and SHALL
  recommend `${VAR}` instead.

- **BR-CFG-003**: `tpl` warns; it does not prevent. Putting a secret in the
  argument vector is the caller's decision. What `tpl` guarantees is that no
  flag named `password` exists, and that the two remaining paths are documented
  rather than accidental.

- **FR-CFG-034**: WHEN a `cfg` command rewrites `.tpl/.cfg`, the file SHALL
  retain mode `0600`.

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

  *Rationale.* `FR-PROJ-019` creates the file at `0600` and `FR-PROJ-011`
  refuses to read it at any looser mode; a command that loosened it would break
  the next invocation.

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
  `connected`, and `read_only_session`, reporting the outcome of the two steps
  `FR-CFG-024` requires the command to perform:

  ```json
  {"schema_version":1,"source":"server","data":{"entry":"shop","connected":true,"read_only_session":true}}
  ```

  *Rationale.* A `test` that reaches exit `0` reports that both steps
  succeeded; a failure exits `69`, `77`, or `78` instead and emits the error
  document of `FR-ERR-014`. The fields are named rather than implied so that a
  caller can branch on them without inferring from the exit code alone.

  *Known gap.* Whether `test` also reports the reader's effective privileges is
  [OQ-002](open-questions.md#oq-002) and remains open. Answering it adds a
  field, which `FR-OUT-014` makes a non-breaking change, so this document is
  fixed now and grows later if that question is answered yes.

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

## Open questions

- [OQ-002](open-questions.md#oq-002) — whether `test` reports effective
  privileges or only that the read-only session was established. Answering it
  yes adds a field to `FR-CFG-039`, which `FR-OUT-014` makes non-breaking.
- [OQ-016](open-questions.md#oq-016) — short forms for the `add` and `update`
  flags.
- [OQ-017](open-questions.md#oq-017) — whether `password_command`, `ca_file`,
  and `ca_path` have flags of their own.

