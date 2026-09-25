---
title: Configuration Commands
status: approved
last-reviewed: 2026-09-24
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
  over the whole key space: every key of `FR-CONF-002`, with the `<name>`
  segment of each `database.<name>.<field>` form bound to every entry the file
  declares. The `hint` SHALL state, for each candidate the file does not set,
  that `.tpl/.cfg` does not set it. The same population and the same statement
  apply to a key given to `tpl cfg unset`, per `FR-CFG-012`.

  ```
  tpl cfg get core.conect_timeout
  error: key 'core.conect_timeout' is not set in .tpl/.cfg
  cause: core.conect_timeout is not a key of tpl, and .tpl/.cfg does not carry it
  hint:  did you mean 'core.connect_timeout'? .tpl/.cfg does not set it, so its default applies; list every key, its type and its default with: tpl help cfg set
  exit:  66 (EX_NOINPUT)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the candidate, the statement that
  the file does not set it, and the code.

  *Amended in the forty-seventh edition,* for rmp `#276`. The suggestion was
  drawn over the keys the file sets, so a slip in a key the file does not set
  — `core.conect_timeout` for `core.connect_timeout` — received no suggestion,
  and the caller could not tell a misspelt key from an unset one. The key space
  is the population `FR-CFG-009` already suggests over for `tpl cfg set`, and
  its spellings are literals of `FR-ERR-022`; an entry name inside a
  `database.<name>` key stays governed by the character set of that
  requirement, per `FR-ERR-023`. The code is unchanged: a key the file does not
  carry is still `66`, per `FR-ERR-035`.

  *Why the hint says the candidate is not set.* `tpl cfg get` and
  `tpl cfg unset` of a key the file does not set exit `66` again, so a
  candidate offered without that statement sends the caller to a command that
  cannot succeed, which `BR-ERR-004` forbids a `hint` to do. With it, the
  caller learns on first reading that the key it meant is unset, which is the
  answer to both commands.

  *Rejected: keeping the population to the keys the file sets.* It offers
  nothing for the slip a caller is most likely to make in a file that sets few
  keys, and it gives the same silence for a misspelt key and for a correctly
  spelt unset one.

  IF the key supplied to `tpl cfg get` has the form of a block — `core`,
  `database`, or `database.<name>` — rather than of a key of `FR-CONF-002`,
  THEN the system SHALL exit `64` (`EX_USAGE`), whether or not the file
  carries the block. The `cause` SHALL name the key and state that it names a
  block and not one value. The `hint` SHALL carry
  `tpl cfg database show <name>` WHERE the key is `database.<name>` and the
  entry exists, and `tpl cfg list` otherwise, per `BR-ERR-004`.

  ```
  error: 'database.shop' names a whole entry, not one value
  cause: tpl cfg get reads one key; database.shop is the block of entry 'shop'
  hint:  show the entry with: tpl cfg database show shop
  exit:  64 (EX_USAGE)
  ```

  *Rationale.* Exiting `0` with empty output would be indistinguishable from a
  key whose value is empty.

  *Amended in the forty-third edition.* The block form is new. The requirement
  stated only the absent key, so `tpl cfg get database.shop`, for an entry that
  exists, reported that the key "is not set" with `66`, which is false of a
  block the file carries, per finding E-14 of the audit of rmp `#259`. The
  code is `64` because the fault is in the token the caller wrote and the
  caller can rewrite it, which is what `64` means under `FR-ERR-035`; the
  block's presence does not change the next step.

  *Rejected: `66` with a cause that says the key names a block.* `66` sends a
  caller to list what exists and choose another name, and the name the caller
  chose does exist.

- **FR-CFG-008**: `tpl cfg set <key> <value>` SHALL write the value under that
  key. `FR-CFG-048` states when a write to a key of a database entry is
  refused, and `FR-CFG-053` the line written when the key holds a field that
  changes where the entry points.

- **FR-CFG-009**: `tpl cfg set` SHALL accept only the enumerated key space of
  `FR-CONF-002`. IF the key is not in that space, THEN the system SHALL exit
  `64` (`EX_USAGE`) with a nearest-match suggestion over the known keys.

- **FR-CFG-010**: `tpl cfg set` SHALL validate the supplied value against the
  type declared for that key in `FR-CONF-002`, and SHALL exit `64` if it does
  not conform.

- **FR-CFG-054**: IF the value given to `tpl cfg set core.database` matches
  `FR-CONF-048` and no entry of `.tpl/.cfg` has that name, compared byte for
  byte, THEN the system SHALL exit `66` (`EX_NOINPUT`) and SHALL write nothing
  to `.tpl/.cfg`. A value outside `FR-CONF-048` is refused first, with `64`,
  by that requirement.

  The `cause` SHALL name the value and state that `.tpl/.cfg` declares no
  entry of that name. The `hint` SHALL offer the nearest matches among the
  entry names the file declares, per `FR-ERR-019` and `FR-ERR-044`, with the
  command `tpl cfg set core.database <candidate>`. WHERE no candidate is
  admitted, the `hint` SHALL carry `tpl cfg database list`, and WHERE the file
  declares no entry, `tpl cfg database add <name>` with its placeholder.

  ```
  tpl cfg set core.database shpo
  error: database entry 'shpo' does not exist
  cause: core.database names an entry, and .tpl/.cfg declares no entry 'shpo'
  hint:  did you mean 'shop'? set it with: tpl cfg set core.database shop
  exit:  66 (EX_NOINPUT)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the candidate, the
  command and the code.

  *Rationale.* `tpl cfg set core.database nope` exited `0`, and the next
  command that requires an entry failed with `66` under `FR-ERR-005`, far
  from the command that wrote the name. `FR-CFG-023` already keeps the file
  from reaching this state by a deletion; this requirement keeps it from
  reaching it by a write. `66` is the code `FR-ERR-035` gives a named object
  that does not exist, and the code `FR-CFG-012` gives an unset of an absent
  key. The `hint` writes the invocation's own value under the candidate, which
  `BR-ERR-005` admits. This is rmp `#271`.

  *Rejected: exiting `0` with a warning.* The state the warning describes is
  one `FR-CFG-023` exists to prevent, and a caller that checks only the exit
  code learns of it at the next command, per `BR-ERR-002`. *Rejected:
  documenting it as allowed.* It keeps the defect and the distance between
  cause and failure.

  *Accepted cost.* The default entry cannot be set before the entry exists.
  The caller adds the entry first, or passes `-d` until it does.

  *Added in the fifty-ninth edition,* for rmp `#271`.

- **FR-CFG-011**: `tpl cfg unset <key>` SHALL accept either a leaf key, such as
  `database.shop.host`, or a whole block, such as `database.shop`, and SHALL
  delete what it is given. `FR-CFG-023` states the one further change a
  deletion makes, `FR-CFG-052` the line written for each whole entry
  deleted, including every entry the block `database` holds, and `FR-CFG-053` the line written when the key holds a field that
  changes where the entry points.

- **FR-CFG-050**: WHEN `tpl cfg unset` deletes the key
  `database.<name>.dsn`, the system SHALL write exactly one warning line to
  stderr, per `FR-OUT-020`, after the rewrite succeeds, and SHALL exit `0`.
  The line SHALL name the key and the entry, SHALL state that the host, port,
  user, password and database the dsn carried are no longer in the entry, and
  SHALL NOT reproduce any part of the dsn, per `BR-ERR-003`. The line is a
  warning, so `-q/--quiet` suppresses it, per `FR-GLOB-015`. `tpl cfg unset`
  given the block `database.<name>` writes no such line: the caller named the
  whole entry. It writes the line of `FR-CFG-052` instead. `tpl cfg unset`
  given `database.<name>.dsn` also writes the line of `FR-CFG-053`, after
  this one.

  ```
  tpl cfg unset database.ds.dsn
  warning: removed database.ds.dsn; entry 'ds' no longer holds the host, port, user, password or database that dsn carried
  ```

  The wording of the line is the implementation's. The example fixes the
  facts named.

  *Rationale.* `dsn` is the one key that holds five facts. `FR-CFG-011`
  deletes what it is given, and a caller who unsets `dsn` as a step towards
  changing one of them learns of the others' loss only from a later `78` or
  `77`. This is finding X-01 of the seventh re-audit of rmp `#263`, recorded
  for rmp `#282`.

  *Rejected: refusing the unset.* It contradicts `FR-CFG-011`, and it departs
  from the precedent of `BR-CFG-003`, under which `tpl` warns about a legal
  write and does not prevent it. *Rejected: no
  line.* The deletion is legal and named, but its reach is not visible in the
  key the caller typed.

  *Added in the fiftieth edition,* for rmp `#282`.

- **FR-CFG-012**: IF the key or block supplied to `tpl cfg unset` is absent,
  THEN the system SHALL exit `66`. For a key, the nearest-match suggestion and
  its `hint` SHALL follow `FR-CFG-007`.

  For a block — `core`, `database`, or `database.<name>` — the candidates
  SHALL be the blocks the file carries, each `database.<name>` written with
  the name the file declares. The `hint` SHALL name the candidates and SHALL
  NOT carry a `tpl cfg unset` command, per `BR-ERR-005`. WHERE the candidate
  is an entry's block, the `hint` SHALL carry
  `tpl cfg database show <candidate name>`. WHERE no candidate is admitted,
  the `hint` SHALL carry `tpl cfg database list` for a `database.<name>`
  block and `tpl cfg list` otherwise.

  ```
  tpl cfg unset database.shpo
  error: block 'database.shpo' is not in .tpl/.cfg
  cause: .tpl/.cfg declares no entry 'shpo'
  hint:  did you mean 'database.shop'? show it with: tpl cfg database show shop
  exit:  66 (EX_NOINPUT)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the candidate, the command and the
  code.

  *Amended in the fifty-ninth edition,* for rmp `#277`. The requirement gave
  a suggestion for a key and none for a block, so `database.shpo` received
  none although the file declares `shop`. `FR-ERR-021` makes database entries
  and configuration keys suggestion candidates, and a block names one or the
  other. The `hint` shows the candidate rather than deleting it, because the
  caller may have meant another name and a deletion cannot be undone by the
  next command. *Rejected: a `hint` carrying
  `tpl cfg unset database.<candidate>`*, which `BR-ERR-005` forbids.

  *Amended in the forty-seventh edition,* for rmp `#276`. The requirement named
  no suggestion, and the implementation drew one over the keys the file sets,
  as `FR-CFG-007` then read. It now cites `FR-CFG-007`, so both commands
  suggest over one population and state which candidates the file does not
  set.

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

  *Note added in the forty-ninth edition.* `FR-CONF-048` states which names
  the command accepts, and `FR-CONF-050` refuses an empty `--host` or
  `--schema`.

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
  by the flags supplied, leaving the rest of the entry untouched. `FR-CFG-048`
  states what happens where a field the flags name and a field they leave alone
  cannot stand together.

  IF `tpl cfg database update` is invoked with none of the flags of
  `FR-CFG-027`, THEN the system SHALL exit `64` (`EX_USAGE`) and SHALL leave
  `.tpl/.cfg` unchanged. The condition is decided at argument parsing, step 1
  of `FR-ERR-006`, before the entry is resolved. The `cause` SHALL state that
  no field flag was given, and the `hint` SHALL list every flag of
  `FR-CFG-027`.

  ```
  error: nothing to change: tpl cfg database update needs at least one field flag
  cause: no field flag was given for entry 'shop'
  hint:  give at least one of --dsn, --host, --port, --user, --schema, --tls, --password-command, --ca-file, --ca-path
  exit:  64 (EX_USAGE)
  ```

  *Amended in the forty-third edition.* The requirement did not say what an
  invocation with no field flag does, and it exited `0` and changed nothing,
  which a caller reads as success, per finding E-25 of the audit of rmp `#259`.
  `FR-CFG-016` already refuses `tpl cfg database add` with no connection flag
  with `64`; `update` now refuses its own empty invocation with the same code.

  WHERE the refused invocation was given `-d/--database`, the `cause` SHALL
  also state that `-d/--database` was given, that it selects the entry for
  commands that connect and is not a field flag, and that the database on the
  server is set with `--schema`, or is the path part of `--dsn`, per
  `FR-CONF-009`, for an entry defined by `dsn`. The `cause` SHALL NOT
  reproduce the value given to `-d/--database`. The `hint` is unchanged.

  ```
  tpl cfg database update shop --database shop2
  error: nothing to change: tpl cfg database update needs at least one field flag
  cause: no field flag was given for entry 'shop'; -d/--database was given, and it selects the entry for commands that connect, not a field; the database on the server is set with --schema, or is the path part of --dsn for an entry defined by dsn
  hint:  give at least one of --dsn, --host, --port, --user, --schema, --tls, --password-command, --ca-file, --ca-path
  exit:  64 (EX_USAGE)
  ```

  *Amended in the fifty-first edition.* The `cause` said that no field flag was
  given to a caller who had given `--database` in the belief that it wrote the
  database on the server, and nothing said why that flag did not count, per
  finding AA-02 of the tenth re-audit of rmp `#263`, recorded for rmp `#285`.
  The refusal is decided at step 1 of `FR-ERR-006`, so the warning of
  `FR-CFG-051` is not written for it, and the `cause` carries the fact instead.
  The value is not reproduced because the `cause` needs only the flag to be
  understood, and a value written back would have to pass the set of
  `FR-ERR-022`.

  *Amended in the fifty-second edition.* The `cause` said only that the
  database on the server is set with `--schema`. For an entry defined by
  `dsn`, `--schema` is refused, per `FR-CFG-048`, and the server database is
  the path part of the dsn, per finding AB-02 of the eleventh re-audit of rmp
  `#263`, recorded for rmp `#286`. The refusal is decided at step 1, before
  `.tpl/.cfg` is read, so the `cause` cannot know the form of the entry and
  names both, which is true of either form.

- **FR-CFG-051**: WHEN `tpl cfg database add` or `tpl cfg database update` is
  given `-d/--database`, in either spelling and at any position `FR-GLOB-002`
  accepts, the system SHALL accept the flag, SHALL give it no effect, per
  `FR-GLOB-007`, and SHALL write exactly one warning line to stderr, per
  `FR-OUT-020`. The line takes one of four forms:

  ```
  warning: -d/--database has no effect on tpl cfg database add; it selects the entry for commands that connect; the database on the server is set with --schema <value>
  warning: -d/--database has no effect on tpl cfg database add; it selects the entry for commands that connect
  warning: -d/--database has no effect on tpl cfg database update; it selects the entry for commands that connect; the database on the server is the path part of --dsn
  warning: -d/--database has no effect on tpl cfg database update; it selects the entry for commands that connect; the name argument already names the entry
  ```

  1. **No effect.** The system SHALL NOT resolve the name `-d/--database`
     gives, and SHALL NOT write it to any field of the entry.
  2. **The line.** It SHALL name `-d/--database`, SHALL name the command given
     — `tpl cfg database add` or `tpl cfg database update`, in full whatever
     alias the invocation used — and SHALL state that the flag selects the
     entry for commands that connect and has no effect here. Its last clause
     SHALL be chosen by the first of these conditions that holds:
     1. **The invocation uses or targets a dsn.** `tpl cfg database add` is
        given `--dsn`, or `tpl cfg database update` is given `--dsn` or names
        an entry that `.tpl/.cfg` defines by `dsn`. The line SHALL state that
        the database on the server is the path part of `--dsn`, per
        `FR-CONF-009`, and SHALL name neither `--schema` nor any value.
     2. **`--schema` was given.** The line SHALL end after the statement that
        the flag has no effect here, and SHALL name neither `--schema` nor any
        value.
     3. **The value is the entry name.** The value given to `-d/--database`
        is byte-for-byte equal to the `<name>` operand of the command. The
        line SHALL state that the name argument already names the entry, and
        SHALL name neither `--schema` nor any value.
     4. **Otherwise.** The line SHALL state that the database on the server is
        set with `--schema`, and SHALL reproduce the value given to
        `-d/--database` in place of `<value>` only where that value matches
        the set of `FR-ERR-022` for a database entry; otherwise it SHALL write
        the placeholder `<database>` and SHALL NOT reproduce the value in any
        form.
  3. **When.** The line SHALL be written once, after step 3 of `FR-ERR-006`
     passes and before any later check, so it is the first line on stderr and
     precedes any error the invocation then raises. Step 3 is the earliest
     point at which `.tpl/.cfg` states whether the entry `update` names is
     defined by `dsn`. An invocation refused at steps 1 to 3 writes the error
     of that step and no warning: at step 1, among others, `-d` given twice,
     per `FR-CLI-014`, `tpl cfg database add` with no connection flag, per
     `FR-CFG-016`, or `tpl cfg database update` with no field flag, per
     `FR-CFG-020`.
  4. **Exit code.** The exit code SHALL be the one the same invocation without
     `-d/--database` returns.
  5. **Verbosity.** The line is a warning, so `-q/--quiet` suppresses it, per
     `FR-GLOB-015`. `-v/--verbose` does not change it.
  6. **Only the flag.** The line is written for the flag on the command line
     only. `core.database` in `.tpl/.cfg` writes no line.

  ```
  tpl cfg database add hs4 --host h --database shop            warning (--schema shop), then 0
  tpl -d nope cfg database add nope --host h                   warning (name argument names the entry), then 0
  tpl -d hs4 cfg database update hs4 --host h5                 warning (name argument names the entry), then the line of FR-CFG-053, then 0
  tpl cfg database add hs5 --host h -d Hs5                     warning (--schema Hs5), then 0
  tpl cfg database update shop --host h -d 'a b'               warning (--schema <database>), then the line of FR-CFG-053, then 0
  tpl cfg database add hs7 --host h --schema s -d shop         warning (no --schema clause), then 0
  tpl cfg database add d1 --dsn mariadb://h/shop -d shop       warning (path part of --dsn), then 0
  tpl -d ds cfg database update ds --host h2                   warning (path part of --dsn), then 64 per FR-CFG-048, where ds is defined by dsn
  tpl -q cfg database add hs4 --host h -d shop                 no line; 0
  tpl cfg database update shop --database shop2                no line; 64 per FR-CFG-020
  ```

  *Amended in the fifty-second edition.* The line always
  ended "the database on the server is set with --schema <value>", with the
  `-d/--database` value in place of `<value>`. Where `--schema` was also
  given, the `-d/--database` value is most likely the entry name, and the
  advice would overwrite the stored server database with it. Where the entry
  is defined by `dsn`, `--schema` is refused, per `FR-CFG-048`, and the server
  database is the path part of the dsn. This is finding AB-02 of the eleventh
  re-audit of rmp `#263`, recorded for rmp `#286`. The line was written after step 1 of `FR-ERR-006`;
  it moves after step 3, because the dsn condition of an `update` is known
  only once `.tpl/.cfg` is read. *Rejected: deciding the dsn condition from
  the command line alone.* `tpl -d ds cfg database update ds --host h2` names
  no `--dsn`, and the line would still advise `--schema ds`, which
  `FR-CFG-048` refuses. *Accepted cost.* An invocation refused at step 2 or
  step 3 no longer carries the line; its error concerns the project, not the
  flag.

  *Amended in the fifty-third edition.* Condition 3 is new. Where the
  `-d/--database` value equals the entry name, the caller most likely used
  `-d` for its documented meaning, which is naming the entry, and the line
  advised `--schema <value>` with that name. Obeying it overwrites the server
  database the entry stores with the entry name. This is finding AC-02 of the
  twelfth re-audit of rmp `#263`, recorded for rmp `#287`. The equality is
  byte-for-byte because no requirement makes an entry name case-insensitive.
  Conditions 1 and 2 keep precedence: their clauses name no value, so neither
  advises a write that could overwrite a field. *Rejected: keeping the
  `--schema` clause and adding that the name argument already names the
  entry.* The clause still reads as a correction, and a caller that copies it
  still overwrites the server database.

  *Rationale.* `cfg database show` prints the server database under the key
  `database`, so `--database` is the natural guess for the flag that writes it.
  `FR-CFG-028` gives that flag the name `--schema`, and `FR-GLOB-007` gives the
  global `-d/--database` no effect on a command that requires no entry. Before
  this requirement `tpl cfg database add hs4 --host h --database shop` exited
  `0` with an entry lacking its server database, and the caller learnt it only
  at the next connecting command, with a `78`. This is finding AA-02 of the
  tenth re-audit of rmp `#263`, recorded for rmp `#285`. The line follows the
  precedent of `FR-PROJ-026`: a global flag with no effect on a node is
  accepted and warned about, not refused.

  *Rejected: exiting `64`.* It refuses a global flag given alone, which
  `FR-GLOB-002` and `BR-GLOB-001` forbid, and it refuses the invocation
  `FR-GLOB-007` exists to admit: `tpl -d nope cfg database add nope --host h`.

  *Rejected: treating `--database` as `--schema` here.* It gives one flag two
  meanings at one node, which `FR-CFG-028` rejects.

  *Accepted cost.* A caller that passes `-q`, or checks only the exit code,
  does not see the line, as `FR-PROJ-026` accepts for its own.

- **FR-CFG-053**: WHEN one of the following invocations exits `0`, the system
  SHALL write exactly one warning line to stderr, per `FR-OUT-020`, after the
  rewrite of `.tpl/.cfg` succeeds, and SHALL write nothing to stdout, per
  `FR-OUT-023`:

  - `tpl cfg database update <name>` given at least one of the flags
    `FR-CACHE-029` names: `--host`, `--port`, `--user`, `--schema`, `--tls`
    or `--dsn`;
  - `tpl cfg set` given one of the keys `database.<name>.host`,
    `database.<name>.port`, `database.<name>.user`,
    `database.<name>.database`, `database.<name>.tls` or
    `database.<name>.dsn`, which hold what those flags write;
  - `tpl cfg unset` given one of those six keys.

  ```
  tpl cfg database update shop --host db-staging.example.com
  warning: entry 'shop' may now point at another server; any data cached for it under .tpl/.cache/shop/ is kept and still served; clear it with: tpl -d shop cache clean

  tpl cfg set database.shop.host db-staging.example.com
  warning: entry 'shop' may now point at another server; any data cached for it under .tpl/.cache/shop/ is kept and still served; clear it with: tpl -d shop cache clean
  ```

  1. **Content.** The line SHALL name the entry, SHALL state that the entry
     may now point at another server, SHALL state that any data cached for
     it under `.tpl/.cache/<name>/` is kept and still served, and SHALL carry
     the command `tpl -d <name> cache clean`, with the entry name in place of
     `<name>`. The line SHALL NOT state as fact that the entry was repointed,
     that it existed before the invocation, or that a cache exists for it:
     items 3 and 7 bar the system from knowing any of the three, and the
     line SHALL be true whether the entry is new, its value is unchanged, or
     no cache exists. The name has passed `FR-CONF-048`: at step 3
     of `FR-ERR-006` where the entry is read from `.tpl/.cfg`, and on the
     command line where it is the `<name>` segment of a key given to
     `tpl cfg set`. It is reproduced under `FR-ERR-022`.
  2. **The project.** The command the line carries SHALL carry the
     `--tpl-dir` of the invocation exactly as item 2 of `FR-CFG-052` states,
     with the placeholder and its statement in words where `FR-ERR-041`
     refuses the value.
  3. **No cache access.** The system SHALL NOT read, list, test for or delete
     anything under `.tpl/.cache/` to decide whether to write the line, per
     `FR-CACHE-011` and `BR-CACHE-004`. The line is written whether or not a
     cache exists for the entry.
  4. **Order.** Where the invocation also writes the line of `FR-CFG-051`, or
     the line of `FR-CFG-050`, that line comes first and this line second.
  5. **Verbosity.** The line is a warning, so `-q/--quiet` suppresses it, per
     `FR-GLOB-015`.
  6. **Exit code.** The line does not change the exit code.
  7. **No comparison.** The line does not depend on what `.tpl/.cfg` held for
     the entry before the invocation.

  An invocation refused before or at the rewrite writes no such line. An
  invocation given only `--password-command`, `--ca-file` or `--ca-path`, and
  a `tpl cfg set` or `tpl cfg unset` given any key other than the six above,
  writes no such line: those fields do not change where the entry points.
  `tpl cfg unset` given the block `database.<name>`, or the block `database`,
  writes the lines of `FR-CFG-052` instead.

  The wording of the line is the implementation's. The example fixes the
  facts named and the command carried.

  *Rationale.* `FR-CACHE-029` and `BR-CACHE-003` accept that repointing an
  entry leaves its cache in place and require the consequence to be
  documented plainly. Before this requirement the consequence was stated
  only in the help of the command, which a caller that copies a `hint` does
  not read. This is part of finding AC-01 of the twelfth re-audit of rmp
  `#263`, recorded for rmp `#287`. The line is not a `hint`, and the cache it
  names is that of the entry the invocation names, so `BR-ERR-005` is not
  engaged.

  *Rejected: writing the line only where `.tpl/.cache/<name>/` exists.* It
  makes a `cfg` subcommand touch the cache, which `FR-CACHE-011` forbids.
  *Rejected: deleting the cache.* `BR-CACHE-004` forbids a configuration
  command to delete cached data as a side effect. *Rejected: comparing the
  new value with the stored one and writing the line only when it differs.*
  An update that writes back a stored value is rare, and the line it would
  spare costs one line of stderr.

  *Accepted cost.* A caller that passes `-q`, or checks only the exit code,
  does not see the line, and a cache the line names may not exist; the line
  says "any data" so that it stays true where none does.

  *Amended within the fifty-third edition: item 2 is new.* The command the
  line carried was `tpl -d <name> cache clean` whatever `--tpl-dir` the
  invocation was given, so a caller that copied it cleaned the cache of the
  project found from the current directory, the defect class `FR-ERR-043`
  closes for a `hint`.

  *Added in the fifty-third edition,* for rmp `#287`.

  *Amended in the fifty-fourth edition: `tpl cfg set` and `tpl cfg unset` of
  one field are new.* The requirement named `tpl cfg database update` alone,
  and `tpl cfg set database.shop.host q` or
  `tpl cfg unset database.shop.database` changes the same field through its
  key, with exit `0` and nothing on stderr, after which reads serve the data
  cached before the change. This is finding AD-01 of the thirteenth re-audit
  of rmp `#263`, recorded for rmp `#289`. The content of the line, its
  `--tpl-dir` rule and its bar on cache access are unchanged; item 7 is new,
  and item 4 now names `FR-CFG-050`, because
  `tpl cfg unset database.<name>.dsn` writes both lines and the line about
  the entry's own fields comes before the line about its cache. Item 7 states
  for every command what the rejected comparison states for `update`.
  *Rejected: writing the line from `tpl cfg set` only where the entry existed
  before the invocation.* The facts the line states hold for any entry of
  that name, per `FR-CACHE-002`, and the extra condition would spare a line
  that is still true.

  *Amended in the fifty-fifth edition: item 1 states what is known.* The line
  said that the entry "was repointed" and that data cached for it "is kept",
  and wrote so for `tpl cfg set` of an entry that did not exist before, for
  a second identical `tpl cfg set`, and for an entry with no cache: three
  cases in which items 3 and 7 leave the system unable to know either fact.
  The line now says the entry may point at another server and that any data
  cached for it is kept and still served. This is finding AE-02 of the
  fourteenth re-audit of rmp `#263`, recorded for rmp `#290`. The command
  carried, its `--tpl-dir` rule, the bar on cache access, the order, the
  verbosity and the exit code are unchanged.
  *Rejected: comparing values or testing for the cache so that the old
  wording is true when written.* Items 3 and 7 reject both, for the reasons
  stated above.

- **BR-CFG-001**: `add` creates and `update` changes. Neither silently does the
  other's job: there is no `--force` that replaces wholesale, and no idempotent
  `add` that would make the two verbs synonyms.

- **FR-CFG-048**: IF a `cfg` command that writes a database entry — `tpl cfg
  set`, `tpl cfg database add`, or `tpl cfg database update` — would leave that
  entry in a combination `FR-CONF-007` refuses, THEN the system SHALL refuse
  the write, SHALL leave `.tpl/.cfg` unchanged, and SHALL exit `64`
  (`EX_USAGE`).

  The entry this rule is applied to is the entry as it would stand after the
  write: the fields `.tpl/.cfg` already carries for that entry, with the fields
  the invocation names added or changed. The system SHALL NOT remove, replace,
  or rewrite a field the invocation did not name in order to make the entry
  coherent, and SHALL NOT perform the write and leave the incoherence for a
  later invocation to fail on.

  The `cause` SHALL name both members of the pair — the key the invocation
  writes and the key the entry already carries — per the `64` row of
  `FR-ERR-034`. The `hint` SHALL carry a runnable command that makes the
  change the invocation asked for and deletes nothing the invocation did not
  name, per `FR-ERR-009` and `BR-ERR-005`:

  | The entry carries | The invocation writes | The `hint` carries |
  |---|---|---|
  | `dsn` | a discrete connection field | `tpl cfg database update <entry> --dsn <url>`, per `FR-ERR-045` |
  | discrete connection fields | `dsn` | `tpl cfg database update <entry>` with the flag of `FR-CFG-027` for each field the new value changes, each with a placeholder |
  | `password` | `password_command` | `tpl cfg unset database.<entry>.password`, then the invocation again |
  | `password_command` | `password`, or a `dsn` carrying a password | `tpl cfg unset database.<entry>.password_command`, then the invocation again |
  | a `dsn` carrying a password | `password_command` | `tpl cfg database update <entry> --dsn <url>`, where `<url>` stands for the dsn without its password, then the invocation again |

  For `tpl cfg database add`, which writes into no existing entry, the one
  refusal is the third row of `FR-CONF-007`, reached within the invocation.
  Its `hint` SHALL carry the same `add` with `--dsn <url>`, where `<url>`
  stands for the dsn without its password, and with `--password-command`
  kept.

  WHERE the pair is `dsn` and a discrete connection field, the `cause` SHALL
  also state what switching the entry to the other form removes. WHERE the
  entry carries `dsn`, it SHALL state that unsetting `dsn` removes the host,
  port, user, password and database it carries. WHERE the entry carries
  discrete connection fields, it SHALL name each of them and state that
  describing the entry by `dsn` requires unsetting them. The `hint` SHALL NOT
  carry either switch, per `BR-ERR-005`.

  ```
  tpl cfg database update ds --host 127.0.0.1 --port 3306
  error: cannot declare both database.ds.host and database.ds.dsn
  cause: entry 'ds' is defined by dsn; the host is changed inside the dsn. Unsetting dsn would also remove the port, user, password and database it carries
  hint:  write the whole connection as a new dsn: tpl cfg database update ds --dsn <url>, where <url> is the connection URL with the new host and port
  exit:  64 (EX_USAGE)
  ```

  The wording of each line is the implementation's, under `FR-ERR-008`
  through `FR-ERR-012`. The example fixes the facts named, the command
  carried and the code.

  *Amended in the fiftieth edition,* for rmp `#282`. The `hint` named "the
  `tpl cfg unset` of the field that conflicts, or `tpl cfg database remove`
  followed by `tpl cfg database add`". For a dsn entry the first is
  `tpl cfg unset database.<entry>.dsn`, which removes the user, the database
  and the password the caller never named, and the second removes the whole
  entry. Both were followed as written, per finding X-01 of the seventh
  re-audit of rmp `#263`. *Rejected: keeping the unset and warning in the
  `hint`.* `BR-ERR-005` states why a warning beside a deleting command does
  not protect a caller that copies the command. `FR-CFG-050` warns when a
  caller unsets `dsn` by its own choice.

  *What was missing.* `FR-CFG-016` and `FR-CFG-029` make the two ways of
  describing a connection mutually exclusive **in one invocation**, and
  `FR-CONF-007` makes them mutually exclusive **in one entry**. Nothing joined
  the two, so `tpl cfg set database.shop.dsn <value>` and
  `tpl cfg database update shop --dsn <value>` each wrote a legal invocation
  into an entry `FR-CONF-007` refuses — and `FR-CFG-020` positively requires
  the rest of that entry to be left in place. Every later read is then `78`
  (`EX_CONFIG`) at step 3 of `FR-ERR-006`, including the `tpl cfg unset` that
  would repair it, so the file is repairable only by hand. `add` reaches the
  same state by the third row of `FR-CONF-007`: a `--dsn` carrying a password
  beside `--password-command` is not a pair `FR-CFG-029` separates, because
  `FR-CONF-006` excludes `password_command` from the discrete connection
  fields.

  *Rejected: removing the fields the new value supersedes.* It contradicts
  `FR-CFG-020` and `BR-CFG-001`, which bar a wholesale replacement, and it is
  the guess `BR-CONF-004` refuses, made by the writer rather than by the
  reader — `tpl` would delete a host, a user, and a password the caller never
  named, and the caller would learn of it from `tpl cfg database show`.

  *Rejected: writing, and letting the next read fail.* It produces the
  hand-repairable file described above, and it reports the fault at step 3
  against `.tpl/.cfg`, one invocation after the invocation that caused it,
  which is the opposite of the instance `FR-ERR-034` requires a `cause` to
  name.

  *Why `64` and not `78`.* The file as it stands is valid, and `78` sends the
  caller to fix `.tpl/.cfg`, per its row of `FR-ERR-001`. What is refused is
  the invocation, which the caller wrote and can rewrite. `FR-CFG-017` is the
  same shape and already carries `64`: a `cfg` write refused for what
  `.tpl/.cfg` already holds.

  *Added in the twenty-second edition.*

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

  *Note added in the fifty-third edition.* The command deletes nothing under
  `.tpl/.cache/`, per `BR-CACHE-004`. `FR-CFG-052` states the line it writes
  about the cache it leaves, and `FR-CACHE-041` states how that cache is
  removed once the entry is gone.

- **FR-CFG-023**: WHEN a `cfg` command deletes the entry named by
  `core.database` — `tpl cfg database remove <name>`, or `tpl cfg unset` given
  the block of that entry — the system SHALL also clear `core.database`,
  silently and in the same rewrite, leaving the file coherent.

  A deletion that leaves the entry in place does not engage this rule.
  `tpl cfg unset database.<name>.<field>` removes one field; the entry still
  exists, and `core.database` still resolves.

  *Rationale.* The next invocation without `-d` then fails with `78`, "no
  database entry selected", which is the correct message. A stderr warning would
  not be seen by a caller checking only the exit code, and refusing with `64`
  until the reference is cleared by hand would be worse.

  *Amended in the twenty-second edition: the obligation is over the state, and
  not over one command.* It named `tpl cfg database remove` alone, and
  `tpl cfg unset database.<name>` reaches the identical state — `core.database`
  naming an entry the file no longer carries — with no requirement governing
  it. The next invocation without `-d` then selected an entry that does not
  exist, which is `66` (`EX_NOINPUT`) with a nearest-match suggestion, per
  `FR-ERR-005`, over a population the same invocation had just removed that
  name from. The rationale above states why `78` and "no database entry
  selected" is the right answer for this state, so no decision is taken here
  that this requirement had not already taken: the same rule now reaches the
  second command that produces the state it was written for. The two changes
  are one rewrite because `FR-CFG-041` makes a rewrite atomic, and a file
  carrying one of them without the other is what this requirement exists to
  prevent.

- **FR-CFG-052**: WHEN a `cfg` command deletes a database entry —
  `tpl cfg database remove <name>`, `tpl cfg unset` given the block
  `database.<name>`, or `tpl cfg unset` given the block `database`, which
  deletes every entry — and exits `0`, the system SHALL write exactly one
  warning line to stderr for each entry deleted, per `FR-OUT-020`, after the
  rewrite of `.tpl/.cfg` succeeds, and SHALL write nothing to stdout, per
  `FR-OUT-023`.

  ```
  tpl cfg database remove shop
  warning: removed entry 'shop'; any data cached for it under .tpl/.cache/shop/ is kept, and an entry added later as 'shop' reads it; clear it with: tpl -d shop cache clean
  ```

  1. **Content.** The line SHALL name the entry, SHALL state that any data
     cached for it under `.tpl/.cache/<name>/` is kept, SHALL state that an
     entry added later under the same name reads that data, and SHALL carry
     the command `tpl -d <name> cache clean`, with the entry name in place of
     `<name>`. The entry was declared in `.tpl/.cfg`, so its name has passed
     `FR-CONF-048` at step 3 of `FR-ERR-006` and is reproduced under
     `FR-ERR-022`. `FR-CACHE-041` makes that command succeed once the entry
     is gone.
  2. **The project.** WHERE the invocation was given `--tpl-dir` on the
     command line, the command the line carries SHALL carry `--tpl-dir` with
     the same value, written immediately after `tpl` and before `-d`, as
     `FR-ERR-043` states for a command in a `hint`. The value SHALL be written
     as the caller wrote it, under the set of `FR-ERR-041`. IF the set refuses
     the value, THEN the command SHALL carry `--tpl-dir <path>`, and the line
     SHALL state in words that `<path>` stands for the `--tpl-dir` this
     invocation was given. A `.tpl` found by discovery, per `FR-PROJ-004`,
     SHALL NOT be written: the same command, run from the same directory,
     finds it again.

     *Amended in the fifty-ninth edition,* for rmp `#278`. The item named a
     value taken from `TPL_DIR`, which does not exist, per `FR-CONF-030` and
     `FR-CLI-021`.

     ```
     tpl --tpl-dir /srv/shop/.tpl cfg database remove shop
     warning: removed entry 'shop'; any data cached for it under .tpl/.cache/shop/ is kept, and an entry added later as 'shop' reads it; clear it with: tpl --tpl-dir /srv/shop/.tpl -d shop cache clean
     ```
  3. **No cache access.** The system SHALL NOT read, list, test for or delete
     anything under `.tpl/.cache/` to decide whether to write the line, per
     `FR-CACHE-011` and `BR-CACHE-004`. The line is written whether or not a
     cache exists for the entry.
  4. **Verbosity.** The line is a warning, so `-q/--quiet` suppresses it, per
     `FR-GLOB-015`.
  5. **Exit code.** The line does not change the exit code.
  6. **Every entry.** WHERE the invocation is `tpl cfg unset database`, the
     system SHALL write one line for each entry the block held, in the order
     the entries appear in `.tpl/.cfg` before the rewrite. Each line SHALL
     name its own entry and carry its own `tpl -d <name> cache clean`, under
     items 1 and 2; items 3, 4 and 5 hold for every line. `-q/--quiet`
     suppresses them all. A block that held no entry writes no line.

     ```
     tpl cfg unset database
     warning: removed entry 'shop'; any data cached for it under .tpl/.cache/shop/ is kept, and an entry added later as 'shop' reads it; clear it with: tpl -d shop cache clean
     warning: removed entry 's2'; any data cached for it under .tpl/.cache/s2/ is kept, and an entry added later as 's2' reads it; clear it with: tpl -d s2 cache clean
     ```

  `FR-CFG-023` clears `core.database` in the same rewrite where it names the
  entry, and writes no line of its own. `tpl cfg unset` of one field of an
  entry deletes no entry and writes no such line; where the field is one of
  the six `FR-CFG-053` names, it writes the line of `FR-CFG-053`. An
  invocation refused before or at the rewrite writes no such line.

  The wording of the line is the implementation's. The example fixes the
  facts named and the command carried.

  *Rationale.* The cache is keyed by entry name alone, per `FR-CACHE-002`, so
  removing an entry and adding another under the same name — the obvious way
  to repoint an entry — gives the new entry the data read for the old one,
  with exit `0` at every step and nothing in the output saying so. This is
  finding AC-01 of the twelfth re-audit of rmp `#263`, recorded for rmp
  `#287`. The line is written at the one point at which the name stops
  meaning that entry. It is not a `hint`, and the cache it names belongs to
  the entry the invocation names, so `BR-ERR-005` is not engaged.

  *Rejected: deleting `.tpl/.cache/<name>/` with the entry.* It was the
  preferred ruling of the brief for rmp `#287`, and three requirements
  forbid it: `BR-CACHE-004` bars a configuration command from deleting cached
  data as a side effect, `FR-CFG-004` bars a `cfg` subcommand from writing
  anywhere but `.tpl/.cfg`, and `FR-CACHE-011` bars it from touching the
  cache. Reversing them is a decision this edition does not take.
  *Rejected: writing the line only where `.tpl/.cache/<name>/` exists.* It
  makes a `cfg` subcommand touch the cache, which `FR-CACHE-011` forbids.
  *Rejected: a warning in `tpl cfg database add` where a cache of that name
  exists.* It touches the cache for the same reason. The help of
  `tpl cfg database add` states the fact instead, per `FR-HELP-036`.

  *Accepted cost.* An entry removed by editing `.tpl/.cfg` by hand writes no
  line. A caller that passes `-q`, or checks only the exit code, does not see
  the line.

  *Amended within the fifty-third edition: item 2 is new.* The command the
  line carried was `tpl -d <name> cache clean` whatever `--tpl-dir` the
  invocation was given, so a caller that copied it cleaned the cache of the
  project found from the current directory, the defect class `FR-ERR-043`
  closes for a `hint`.

  *Added in the fifty-third edition,* for rmp `#287`.

  *Amended in the fifty-fifth edition: `tpl cfg unset database` and item 6
  are new.* The requirement named the block of one entry, and
  `tpl cfg unset database` deletes every entry with exit `0` and nothing on
  stderr, after which an entry added under an old name reads the data cached
  for the deleted one. This is finding AE-01 of the fourteenth re-audit of
  rmp `#263`, recorded for rmp `#290`. The content of each line, its
  `--tpl-dir` rule and its bar on cache access are those of the other two
  commands. *Rejected: refusing `tpl cfg unset database` with `64` and a
  hint to `tpl cfg database remove`.* It contradicts `FR-CFG-011`, which
  deletes the block it is given, and departs from the precedent of
  `FR-CFG-050`, which warns about a legal deletion rather than refusing it;
  the coordinator of rmp `#290` ruled for the warning. *Rejected: one line
  naming every entry.* One line per entry keeps the line of item 1
  unchanged and gives each entry a command a caller can copy whole.

  *Amended in the fifty-sixth edition: item 1 says "any data cached".* The
  line stated as fact that data cached for the entry is kept, which is false
  where no cache exists, and item 3 bars the system from knowing which case
  holds. The wording now matches the rows of `FR-HELP-036` and item 1 of
  `FR-CFG-053`. The facts named, the command carried and every other item
  are unchanged. This is rmp `#291`.

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

  *Checked in the twenty-fourth edition, and unchanged.* Steps 2 and 3 put the
  read-only session before the series check, and step 4 puts the catalogue
  probe after both, which is the order `FR-SRV-042` fixes for the statements
  that settle them and `FR-ERR-006` for the conditions themselves. This
  requirement states neither and restates neither: its four steps are the four
  outcomes this command reports, one field each per `FR-CFG-039`, and they are
  performed in this order because those two requirements fix it. A reader who
  needs the order of the connection-start statements reads `FR-SRV-042`, in
  [server-contract.md](server-contract.md).

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
  anything finer is a field list, and when this requirement was written no
  field list had been recorded: `OQ-009`, `OQ-010` and `OQ-024` were open for
  exactly that reason. A probe whose answer depended on such a list would
  have dragged this question behind the same block that held those three,
  which is exactly what answering `OQ-002` was meant to avoid. The seventh
  edition closed all three, and the argument still holds: a privilege listing
  would couple this command's output to a field list that can grow, where a
  boolean cannot. One statement also keeps the command
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

  *Note added in the forty-seventh edition.* `FR-CONF-046` states the strings
  the splitting rule refuses with `64`, among them one that begins with `[`,
  which is how an array written on the command line arrives. "SHALL NOT accept
  an array" is therefore a refusal, never a string stored as one word.

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

- **FR-CFG-031**: `--dsn` SHALL accept exactly the values `FR-CONF-009`,
  `FR-CONF-010` and `FR-CONF-011` admit — the grammar, the two schemes, and no
  query parameter — and no others. The system SHALL validate the value before
  writing it, SHALL store an admitted value verbatim, including a literal
  password, and SHALL exit `64` (`EX_USAGE`) without writing anything where the
  value is not admitted.

  The value SHALL be validated as the caller wrote it, with `${VAR}` left
  unexpanded and treated as opaque text within the field it occupies. That is
  the form `FR-CONF-018` parses, and no `cfg` command reads the environment.

  *A literal password is not what is refused.* `tpl` warns; it does not
  prevent, per `BR-CFG-003`. A DSN carrying a password is admitted and stored
  as written, the help of the flag states that the value is visible in the
  process table, per `FR-CFG-033`, and `FR-SEC-002` names this flag as one of
  the two paths that stay open. What this requirement refuses is a value the
  reader of `.tpl/.cfg` cannot accept.

  *Amended in the twenty-second edition: the flag admits what the file admits,
  and nothing else.* It read "SHALL accept whatever the caller writes,
  including a literal password, and SHALL store it verbatim" — a rule about
  secrets, which is what `FR-SEC-002` and `DIV-002` cite it for, stated wide
  enough to be read as a rule about syntax. Read that way,
  `tpl cfg database add shop --dsn postgres://h/d` succeeded and wrote a scheme
  `FR-CONF-010` does not accept. From that moment every invocation is refused
  at step 3 of `FR-ERR-006`, because no `cfg` subcommand is among the commands
  `FR-PROJ-025` excuses from reading and validating the file — including the
  `tpl cfg database remove shop` that would undo it. The file was repairable
  only by hand, which is the state `FR-CFG-023` and `BR-CONF-004` are both
  written to prevent.

  *The admission is an equality, in both directions.* Admitting more than the
  three requirements do writes a file the reader refuses. Admitting less would
  leave a `.cfg` that is legal and that `tpl` cannot write, against
  `FR-CFG-027`, which gives every key of an entry a flag so that it can.

  *Why `64`.* The value is a token the caller wrote on the invocation and can
  rewrite, which is what `64` means in `FR-ERR-001` and where `FR-ERR-035`
  places a value met on an invocation. The `cause` names the value as written
  and the form expected, per the `64` row of `FR-ERR-034`, and the `hint`
  carries a runnable command, per `FR-ERR-009`.

  *The same value through the other write path.* `FR-CFG-010` validates a value
  supplied to `tpl cfg set` against the type `FR-CONF-002` declares for its
  key, and exits `64` where it does not conform. For `database.<name>.dsn` the
  declared type *connection URL* is these same three requirements, so both
  write paths admit the same set and refuse with the same code.

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

  *Amended in the forty-eighth edition.* The help of `--ca-file` and
  `--ca-path` SHALL also state that the value is read as a literal path and
  that a value containing `${` is refused, per `FR-CONF-047`.

  *Amended in the forty-ninth edition.* The help of `--password-command`, and
  the row of `database.<name>.password_command` in the help of
  `tpl cfg set`, SHALL also state that `${VAR}` is not expanded in the
  command and that its words are passed to the program as written, per
  `FR-CONF-017`. The help of `tpl cfg set` SHALL state that the fields
  `FR-CONF-015` expands are fields of a database entry, and that no key under
  `[core]` is expanded. A `${DB}` written into `core.database`, and an
  `echo ${PW}` written into `password_command`, were both accepted and never
  expanded, and nothing said so, per finding W-04 of the sixth re-audit,
  recorded for rmp `#281`.

- **BR-CFG-003**: `tpl` warns; it does not prevent. Putting a secret in the
  argument vector is the caller's decision. What `tpl` guarantees is that no
  flag named `password` exists, and that the two remaining paths are documented
  rather than accidental.

- **FR-CFG-034**: WHEN a `cfg` command rewrites `.tpl/.cfg`, the file SHALL
  retain mode `0600`.

  *Rationale.* `FR-PROJ-019` creates the file at `0600` and `FR-PROJ-011`
  refuses to read it at any looser mode; a command that loosened it would break
  the next invocation.

  *Amended in the forty-ninth edition: the rewritten file is at `0600`
  whatever mode the file had before.* `FR-PROJ-011` reads a file at any mode
  that grants group and other no access, `0400` included, and the procedure of
  `FR-CFG-041` writes a new file at `0600` and renames it over the old one. A
  file at `0400` or `0700` is therefore at `0600` after the write. The system
  SHALL NOT refuse the write because the owner-write bit is clear, and SHALL
  NOT carry the previous mode over. This pins what the implementation does,
  per finding W-08 of the sixth re-audit, recorded for rmp `#281`.

  *Rejected: keeping the previous mode.* It would carry an execute bit, or a
  mode the owner cannot read, into every later rewrite, and the one mode this
  corpus names for the file is `0600`. Also rejected: refusing to rewrite a
  file whose owner-write bit is clear. A rename replaces a file whatever that
  bit says, so the bit does not protect the file from any writer, and the
  refusal would add a condition and a code to five commands.

  *Accepted cost.* A caller that sets `.tpl/.cfg` to `0400` to mark it as
  read-only finds that a `cfg` command rewrites it and restores owner write.

  *Moved in the fifth edition.* This rationale stood under `FR-CFG-042`, where
  it argued for a mode rather than for the absence of a lock. It is the
  argument for this requirement and now stands under it.

- **FR-CFG-041**: WHEN a `cfg` command rewrites `.tpl/.cfg`, it SHALL write a
  temporary file in `.tpl/` at mode `0600` and SHALL rename it over the target.
  A failure part-way through SHALL leave the previous `.cfg` in place,
  unchanged. IF the write fails at any step — the temporary file cannot be
  created, cannot be written, cannot be given mode `0600`, or cannot be renamed
  over the target — THEN the system SHALL exit `74` (`EX_IOERR`), and the
  `cause` SHALL name the path, the operation attempted on it, and what the
  filesystem returned, per the `74` row of `FR-ERR-034`.

  WHERE `.tpl/.cfg` is absent, the same procedure SHALL create it, and a
  failure part-way through SHALL leave no `.cfg`, per `FR-PROJ-028`.

  *Amended in the forty-eighth edition: the paragraph above is new,* for rmp
  `#265`. A clone holds no `.cfg`, per `FR-PROJ-003`, so the first `cfg`
  writer creates the file rather than rewriting it.

  *Rationale.* This is the rule `FR-CACHE-030` already applies to the other
  thing `tpl` writes, and it matters more here: a truncated `.cfg` is a `78` on
  every subsequent invocation, and it holds the credentials without which the
  project cannot reach a server. A cached object lost to a truncated write is
  recovered by reading the server again; a lost `.cfg` is not recovered at all.

  *Amended in the thirty-first edition: the failure this requirement names now
  carries a code.* The requirement stated the outcome of a failed write — the
  previous file survives — and named no code, so the five commands that rewrite
  the file had a condition `FR-ERR-002` obliges to carry one and none to carry.
  It is `74`, and the row of `FR-ERR-001` that carries it is the `74` row,
  whose cell is widened in the same edition to characterise I/O on `.tpl`
  rather than reading it alone. The five commands are `tpl cfg set`,
  `tpl cfg unset`, `tpl cfg database add`, `tpl cfg database update` and
  `tpl cfg database remove`, which are the writers `FR-PROJ-023` names for
  `.tpl/.cfg`.

  *Why `74`.* The fault is the filesystem refusing an operation on a file the
  project owns, which is what that code means and what its `cause` row is
  written for — the path, the operation, and what the filesystem returned. The
  caller's next step is the one the `74` row of `FR-ERR-001` states: check
  permissions and free space.

  *Rejected: `73` (`EX_CANTCREAT`).* `FR-ERR-003` reserves it for `tpl init`,
  in terms, and the ground survives the reading: `73` reports a **destination**
  that could not be brought into existence, and here the destination exists and
  is intact. A caller receiving `73` from `tpl cfg set` would be told to choose
  another destination, and there is none to choose.

  *Rejected: `78` (`EX_CONFIG`).* It is the code for a configuration that does
  not describe a usable connection, and this requirement guarantees the
  configuration is exactly as it was. The message would send the caller to
  correct a file that is correct, which is the wrong diagnosis
  `FR-CONF-033`'s rationale refuses in the same shape.

  *Rejected: leaving the code to the implementation.* It was left, and the
  implementation supplied `74` in the `EXIT CODES` section of the five
  commands' help before any requirement said so — a help text filling a gap in
  this corpus, which is the state this amendment ends. The code it chose is the
  code this requirement now states, so nothing a caller observes changes.

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

  *Note added in the forty-ninth edition.* `FR-CFG-049` fixes the JSON type
  of each value in `entry`.

- **FR-CFG-049**: In the `json` output of `tpl cfg get`, `tpl cfg list` and
  `tpl cfg database show`, each value SHALL be the JSON counterpart of the
  TOML value `.tpl/.cfg` holds, and SHALL NOT be the text of that value:

  | TOML value in `.tpl/.cfg` | JSON value |
  |---|---|
  | string | string |
  | integer | number |
  | array of strings, as `password_command` holds | array of strings, in order |

  A value the redaction of `FR-CFG-021` replaces SHALL be the string `***`,
  or, for a DSN carrying a password, the DSN string with `***` in its place.
  A value that holds `${VAR}` is a TOML string, so it SHALL be a JSON string
  carrying the reference as written, whatever key holds it.

  ```json
  {"schema_version":1,"source":"project","data":{"entry":{"host":"db.example.com","port":3307,"database":"shop","password_command":["pass","db/shop"]}}}
  ```

  *Rationale.* The help of every command that declares `--format` offers
  `json` as the stable document a program reads. `tpl cfg get` and
  `tpl cfg list` already emitted typed values, and `tpl cfg database show`
  emitted `"port":"3307"` and `password_command` as the text of a TOML array,
  which a program cannot pass back to `tpl cfg set`: a value that begins with
  `[` is refused, per `FR-CONF-046`. One key then had two JSON types across
  three commands. This is finding W-03 of the sixth re-audit, recorded for
  rmp `#281`. For `get` and `list` the requirement pins what the
  implementation does, and for `show` it changes it.

  *Rejected: stating the value types in each of the three requirements.* One
  rule over the three documents cannot drift apart, which is the defect.

  *Added in the forty-ninth edition,* for rmp `#281`.

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
  anything finer would be a field list, and no field list has been recorded
  against the four series. Adding a field is non-breaking, per `FR-OUT-014`.

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
  invalidate the cache, and `FR-CACHE-041`, the clean of a cache left by a
  deleted entry, which `FR-CFG-052` names.
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

