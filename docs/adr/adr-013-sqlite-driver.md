---
id: ADR-013
title: The SQLite driver and the library it carries
status: accepted
decided: 2026-11-03
last-reviewed: 2026-11-04
requirements: [FR-LITE-001, FR-LITE-004, FR-LITE-010, FR-LITE-011, FR-LITE-018, FR-LITE-039]
supersedes: []
superseded-by: null
---

# ADR-013 — The SQLite driver and the library it carries

## Status

Accepted, 2026-11-03.

The selection rule of `FR-LITE-004`, the omission of extension loading from the
carried library, and the demand for the most recent secure SQLite release that
became `FR-LITE-039` are the user's decisions of 2026-11-03, relayed by the
session coordinator for rmp `#314`, `#315` and `#319`. Decision points 3 and 4
— where the release number lives, and how the version conflict with the MariaDB
driver is resolved — were taken on rmp `#315` on the same day, within those
decisions, and were **confirmed by the user on 2026-11-03**, as was the
condition that ends point 4. Every decision in this record is the user's.

**Amended in place on 2026-11-03**, for rmp `#315`, by the user's decision of
that day: `rusqlite`'s feature `trace` is enabled for tests alone, through a
`[dev-dependencies]` entry, and not on the dependency the binary is built from.
The driver, its release and the library carried did not move. The same pass
brought the record's account of the hardening controls to the sixty-seventh
edition of the specification, which added an eighth control, temporary storage.

**Amended in place on 2026-11-04**, for rmp `#317`, once the reader was built:
the record's account of the hardening controls is brought to the sixty-ninth
edition, which added a ninth control, views disabled; the reading of the
shipped graph in *Consequences* is corrected, because three crates it named as
leaving the graph did not leave it; and the accounts of `cargo audit`, of the
fixture run and of the documents that cite this record are brought to commit
`229edc8`. The driver, its release, the library carried and every decision
point are unchanged, so `decided` is unchanged; the claims re-read that day are
those of the *Sources* rows dated 2026-11-04.

## Context

**Two requirements in force name this register, by role, as the home of a fact
the corpus refuses to hold.** `FR-LITE-001` makes the binary carry its own
SQLite library and states that which release it carries is "recorded in the
project's decision records and cited from there". `FR-LITE-039` chooses that
release and requires the project's decision records to record, beside it, the
judgement of its clause 3 for every vulnerability listed with a later fix, and
the reason a newer release is not carried wherever its clause 4 applies. The
record is admitted under rule R4's first limb. `FR-LITE-004` fixes the rule that
selects the driver and names none; it hands this register only a choice among
candidates that survive its last step, and the candidates it disqualified are
the alternatives this record keeps.

**`FR-LITE-004` is the rule, applied in order, and by no other.** Step 0
selects a Rust driver published by the SQLite developers, if one exists and can
apply every control. Step 1 disqualifies any candidate that cannot apply every
hardening control of `FR-LITE-010` and every limit of `FR-LITE-011`, and confirm
each by reading it back, through an interface that requires no `unsafe` code in
`tpl`. Step 2 requires every catalogue fact the specification reads, with
SQLite's own value types and SQL `NULL` preserved. The rule may not be satisfied
by removing a control.

**Applied strictly, the rule first selected no driver.** rmp `#315` applied it
on 2026-11-03, in compiled spikes under `#![forbid(unsafe_code)]`, against the
seven hardening controls and three limits of the specification's sixty-fifth
edition. No official driver exists. `rusqlite` failed one control, the crate
`sqlite` five, the
SQLite driver of `sqlx` 0.9.0 six, and `turso` eight. `rusqlite`'s one failure
was extension loading: its safe configuration interface, `DbConfig`, offers no
`SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION`, so the switch could be set off and not
read back; and the library as `libsqlite3-sys` bundles it is compiled with
`SQLITE_ENABLE_LOAD_EXTENSION=1`, so every connection starts with the switch
on.

**The user decided to compile the mechanism out**, and the specification's
sixty-sixth edition made that the control of `FR-LITE-010`: the carried library
is built with `SQLITE_OMIT_LOAD_EXTENSION`, and before the file is read
`PRAGMA compile_options` must return the row `OMIT_LOAD_EXTENSION`, or the
invocation ends in `70` under `FR-LITE-012`. The confirmation is a statement,
which a candidate issues through its safe interface. `FR-LITE-010` records why
the control was not removed: the mechanism absent is a stronger setting than
the mechanism off.

**The sixty-seventh edition added an eighth control, after the evaluation.**
`FR-LITE-010` now also sets `temp_store` to `MEMORY`, so that a statement's
temporary data is not written to a file outside `.tpl`, and confirms it twice:
`PRAGMA temp_store` must read back `2`, and `PRAGMA compile_options` must return
the row `TEMP_STORE=1`, `TEMP_STORE=2` or `TEMP_STORE=3`, because how the
library is built decides whether the pragma is obeyed. A control the evaluation
did not apply can disqualify a candidate and cannot qualify one, so the outcome
stands if the selected driver passes it. Point 1 records that it does, and
point 2 what the carried build reports.

**The sixty-ninth edition added a ninth control, also after the evaluation.**
`FR-LITE-010` now disables views — `SQLITE_DBCONFIG_ENABLE_VIEW` set to `0` and
read back through the same option — as defence in depth beside the closed
statement list. The same rule holds: point 1 records that the selected driver
passes it.

**`FR-LITE-039` then constrains the release, and the MariaDB driver stood in
the way.** The most recent SQLite release is 3.54.1, of 2026-10-16, and the
`rusqlite` release that carries it, 0.41.1, requires `libsqlite3-sys`
`^0.39.0`. The `sqlx` 0.9.0 facade, through which `ADR-003` consumed the
MariaDB driver, has an optional `sqlx-sqlite` that requires `libsqlite3-sys`
`>=0.30.1, <0.38.0`. Both releases of `libsqlite3-sys` declare
`links = "sqlite3"`, and Cargo admits one package per `links` name. A
dependency's optional dependencies enter the lock file whether or not anything
enables them: this repository's `Cargo.lock` at `833bc35` carries
`libsqlite3-sys` 0.37.0, which no crate of the build compiles. So the facade's
unused SQLite driver held the lock file below the release `FR-LITE-039`
requires; the newest `rusqlite` that resolved beside it, 0.39.0, bundles 3.51.3.

## Decision

1. **The driver is `rusqlite` 0.41.1**, with its default features off. The
   build the binary is made from enables two features: `bundled`, which
   compiles the SQLite source `libsqlite3-sys` carries into the binary, as
   `FR-LITE-001` requires; and `limits`, the interface that sets and reads back
   the limits of `FR-LITE-011`. Tests alone enable a third, `trace`, the
   statement trace `FR-LITE-018` records through. `load_extension` stays off:
   it binds the two library interfaces the omission removes.

   ```toml
   [dependencies]
   rusqlite = { version = "0.41.1", default-features = false, features = ["bundled", "limits"] }

   [dev-dependencies]
   rusqlite = { version = "0.41.1", default-features = false, features = ["trace"] }
   ```

   **`trace` reaches the test build and not the shipped one.** The package's
   edition, 2024, makes its resolver `"3"`, whose one stated change from `"2"`
   is the default of `resolver.incompatible-rust-versions`; under the feature
   resolution of `"2"`, features enabled on a dev-dependency "will not be
   unified when those same dependencies are used as a normal dependency, unless
   those dev-dependencies are currently being built". `cargo test`,
   `cargo clippy --all-targets` and `cargo build --all-targets` therefore
   compile `rusqlite` with `trace`, and `cargo build --release`, from whose
   output `ADR-012`'s release workflow packages the binary, compiles it with
   `bundled` and `limits` alone. The hook that installs the trace is
   `#[cfg(test)]` (`OD-37`). The dev entry keeps `default-features = false`
   because the two entries' features are unified in every test build: without
   it, the tests would also compile `rusqlite`'s defaults, `cache` and the
   crate `hashlink` it brings, which the shipped build does not.

   Under `FR-LITE-004`: step 0 selects nothing, because no Rust driver is
   published by the SQLite developers; step 1 disqualifies the crate `sqlite`,
   `sqlx`'s SQLite driver and `turso`, and `rusqlite` passes it once the
   extension-loading control of `FR-LITE-010` is the compile-time omission.
   The temporary-storage control is applied and confirmed by statements alone —
   `PRAGMA temp_store`, set and read back, and `PRAGMA compile_options` — which
   `rusqlite` issues through its safe interface, `pragma_update`,
   `pragma_query_value` and `prepare`, as it issues the confirmation of the
   omission; it leaves `rusqlite` passing step 1. The views control is set and
   read back through the safe configuration interface that also sets defensive
   mode and `trusted_schema` — `set_db_config` and `db_config` with
   `DbConfig::SQLITE_DBCONFIG_ENABLE_VIEW` — and it too leaves `rusqlite`
   passing step 1. Step 2 is passed by `rusqlite`, the one candidate left. No
   choice among survivors remains for this register.

2. **The library carried is SQLite 3.54.1**, the source `libsqlite3-sys` 0.39.1
   bundles, built with extension loading omitted. `.cargo/config.toml` carries
   the build options:

   ```toml
   [env]
   LIBSQLITE3_FLAGS = { value = "-USQLITE_ENABLE_LOAD_EXTENSION -DSQLITE_OMIT_LOAD_EXTENSION", force = true }
   ```

   The build script of `libsqlite3-sys` appends these tokens after its own
   flags, so the `-U` undoes the `SQLITE_ENABLE_LOAD_EXTENSION=1` it defines and
   the `-D` omits the mechanism; `force` makes the value win over one already in
   the environment.

   **The file is a means, not the guarantee.** Cargo reads it only where it
   discovers the project's configuration, and SQLite's page on compile-time
   options calls the `SQLITE_OMIT_*` options "mostly unsupported" and warns that
   they "may not work with the amalgamation" — the single-file form
   `libsqlite3-sys` bundles. The guarantee is the read-back `FR-LITE-010`
   requires at every open: a binary built without the option ends every
   `sqlite` read in `70`, and reads nothing from the file.

   **The build sets no `SQLITE_TEMP_STORE`, and the temporary-storage control
   needs none.** The build script of `libsqlite3-sys` defines it only under its
   feature `bundled-sqlcipher` and for Android, and this build reaches neither,
   so the carried library keeps SQLite's default `1`, under which the pragma
   decides. The amalgamation defines that default before it compiles the list
   `PRAGMA compile_options` reports, so the list carries `TEMP_STORE=1`, a row
   `FR-LITE-010` admits. That is read from the source and not observed through
   the build; as for the omission, the guarantee is the read-back at every
   open, and a build that answered otherwise would end every `sqlite` read in
   `70`.

   **The judgement `FR-LITE-039` requires.** Clause 1: 3.54.1 is the most recent
   release in SQLite's release history. Clause 2: `rusqlite` 0.41.1 carries it
   and satisfies every step of `FR-LITE-004`, per point 1. Clause 3: SQLite's
   vulnerability page, last updated 2026-08-01, lists no vulnerability fixed in
   a release later than 3.54.1 — the latest release it names as a fix is 3.53.2
   — so no entry calls for a judgement. Clause 4 does not apply: the release
   carried is the most recent one.

3. **The release number lives in `Cargo.lock`, and the test does not restate
   it.** Both manifest entries of point 1 require `rusqlite` `0.41.1`; the lock
   file resolves it, and `libsqlite3-sys` 0.39.1 beneath it, which fixes the
   SQLite release. No `=` requirement is placed on `libsqlite3-sys`. The test
   that guards the build
   asserts that `PRAGMA compile_options` lists `OMIT_LOAD_EXTENSION` and no
   `ENABLE_LOAD_EXTENSION` option, and that the library answering
   `sqlite_version()` is the one compiled in; it asserts no literal release, so
   a release `FR-LITE-039` admits does not fail it.

4. **The MariaDB driver is consumed as two of its crates, so that
   `libsqlite3-sys` 0.39 resolves.** The package depends on `sqlx-core` and
   `sqlx-mysql` directly instead of on the `sqlx` facade, whose optional
   `sqlx-sqlite` is what held the lock file below 0.38.0. The driver and its pin
   do not change. The form — the exact requirement and the feature list of each
   crate — belongs to the driver's record and is held in `ADR-003`; this record
   holds why the form changed and the alternatives refused.

   **The condition that ends this point: the first `sqlx` release whose
   `sqlx-sqlite` admits the `libsqlite3-sys` release carrying the SQLite
   release `FR-LITE-039` requires.** At that release `tpl` returns to the `sqlx`
   facade, with facade features that give `sqlx-core` and `sqlx-mysql` the same
   effective feature set `ADR-003` records, and this point and `ADR-003`'s
   consumption paragraph are amended in place. The return is owed, not merely
   permitted, because the two crates are ones upstream marks not for direct
   use — "Not intended to be used directly" (`sqlx-core`) and "Not for direct
   use; see the `sqlx` crate for details" (`sqlx-mysql`) — and the
   `_`-prefixed features enabled on them are internal to `sqlx`, listed in its
   manifests "for conditional compilation". Returning at that release also
   moves `ADR-003`'s pin to it, under the obligations that record carries for a
   version move.

## Alternatives rejected

- **Reading *official* as the crate named `sqlite`, or as the reference crate
  `rusqlite`.** Refused by the user: *official* means published by the SQLite
  developers, and neither crate is, whatever its name or the source it carries.

- **Switching extension loading off without reading it back.** Refused by the
  user as the weakest option: it leaves the one control nothing confirms.

- **`trace` on the normal dependency**, the form this record first gave.
  Refused by the user on 2026-11-03. Only the `#[cfg(test)]` hook of `OD-37`
  calls the trace, from the seam `FR-LITE-018` names and from no invocation of
  the distributed binary, so the feature on the normal entry would compile into
  the shipped build an interface nothing in it calls. The dev entry of point 1
  gives the tests the feature at no cost to that build.

- **A patched `rusqlite` that reads the switch back.** Refused by the user: it
  is a maintained fork, and it moves the `unsafe` code into the fork rather than
  removing it.

- **The crate `sqlite`, `sqlx`'s SQLite driver, and `turso`.** Disqualified at
  step 1, with five, six and eight controls failed. The omission concerns one
  control, so each still fails at least four, and `FR-LITE-039` makes no
  disqualified driver eligible by the library it carries. Two further facts
  were recorded on the task and would refuse `sqlx`'s driver on their own: it
  always sends `PRAGMA foreign_keys`, a statement `FR-LITE-016` forbids, and at
  0.9.0 it cannot carry 3.54.1.

- **`rusqlite` 0.39.0, which resolves beside the facade.** It bundles 3.51.3,
  and 3.54.1 is obtainable, so clause 4 of `FR-LITE-039` refuses it whatever
  3.51.3 carries. The refusal does not rest on a reachable vulnerability. Of the
  entries listed with a fix after 3.51.3, `CVE-2026-11822` and `CVE-2026-11824`,
  fixed in 3.53.2, need arbitrary SQL, defensive mode off and FTS5, and
  defensive mode is on under `FR-LITE-010`; `CVE-2025-70873`, fixed in 3.52.0,
  is in the `zipfile` extension, which the bundled amalgamation does not
  contain.

- **A stub `sqlx-sqlite` patched in over the registry crate.** A fake crate
  under a real name, and a path patch deletes the lock file's `source`, which
  hides the crate from `cargo audit` — the gap `ADR-010` already carries for
  `sqlx-core`.

- **A vendored `sqlx-sqlite` with its bound widened.** 10,642 lines of
  third-party source, by the task's count, untested against `libsqlite3-sys`
  0.39 and carried for a driver the package never compiles.

- **`sqlx` from its git repository.** Its `sqlx-sqlite` on `main` still requires
  `<0.39.0`, and a git dependency would replace the published release `ADR-003`
  pins.

- **An `=` requirement on `libsqlite3-sys` in the manifest.** A second place for
  the one fact the lock file already fixes, and a second edit each time
  `FR-LITE-039` moves the release.

- **A literal release asserted by the test.** It fails a compliant upgrade and
  proves nothing the version comparison of point 3 does not.

## Consequences

**The SQLite path needs no asynchronous runtime, and `ADR-005` is kept by
construction.** `rusqlite` is synchronous. A command that reads a `sqlite`
entry builds no runtime and reaches nothing in `mariadb/`, so nothing outside
that module becomes asynchronous and no runtime is built for a command that
does not reach it. The deadline of `FR-LITE-009` is reached through the
library's own wait for a lock, `busy_timeout`, and through `rusqlite`'s
`InterruptHandle`, which interrupts a statement running on another thread;
neither needs a runtime timer. `rusqlite` gives every new connection a busy
timeout of 5000 ms, which is not the bound `FR-LITE-009` sets, so it SHALL be
replaced by that bound. How the deadline is built is the technical
specification's.

**The build option is in force only where Cargo reads the project's
configuration.** Cargo probes for configuration from the directory it is invoked
in and its parents, and `cargo install` ignores local configuration unless given
`--path`. The workflows of `ADR-012` invoke Cargo at the repository root and
read it. `cargo build --manifest-path` from outside the tree, or an install from
a registry, does not; a binary built that way carries the mechanism, and every
`sqlite` read it attempts ends in `70` — it fails safe, and never reads.

**The omission was observed on three targets and is guarded on all four.** On
`aarch64-apple-darwin`, `sqlite_version()` answered `3.54.1`,
`PRAGMA compile_options` listed `OMIT_LOAD_EXTENSION`, and `load_extension()`
failed with "no such function". Cross-built with `cargo-zigbuild` for both
`musl` targets, release and test builds, the bundled library carried no
`sqlite3_load_extension` symbol. `x86_64-apple-darwin` was not observed. The
test of point 3 runs in the battery that `ADR-012`'s workflows run on all four
targets of `NFR-PERF-018`. The `musl` readings were taken with zig 0.17.0, not
the zig 0.16.0 `ADR-008` records for the build path; they are readings of the
option's effect, and no figure is taken from them.

**The binary changes; the MariaDB driver it links does not.** Over the shipped
graph, `cargo tree -e normal,build` with features, taken once per target of
`ADR-008`, point 4 and this record together:

- remove the facade `sqlx` and its compile-time macro side — `sqlx-macros`,
  `sqlx-macros-core`, and `hex`, the one crate only they used — together with
  what was compiled only for that side: `sha2` 0.10.9, `digest` 0.10.7 and
  `crypto-common` 0.1.6 with `std`, and `sqlx-mysql` and `thiserror` under the
  feature sets that side enabled. The three hashing crates stay in the graph,
  compiled without `std`, because `sqlx-core`'s feature `migrate` requires
  `sha2` 0.10;
- add `rusqlite`, `libsqlite3-sys`, `fallible-iterator` and
  `fallible-streaming-iterator`, and the build-only `pkg-config` and `vcpkg`;
- leave every other crate at its version and features. `sqlx-core` and
  `sqlx-mysql` compile with the features they had in the binary.

The commit that implemented the change, `2fd98f8`, also moved `tokio`, `rustls`,
`rustls-webpki` and `webpki-roots` to later patch releases, for rmp `#320`; that
move is not this record's. The first readings were taken with `trace` on the
normal entry; re-derived at `229edc8`, with `trace` on the dev entry alone, they
show the same crates leaving and entering, and `rusqlite` compiled in the
shipped graph with `bundled`, `limits` and `modern_sqlite` alone.

The lock file falls from 182 packages to 154: the facade's unused drivers, and
what only they pulled in, leave it. In the evaluation the package's suite
passed, 1392 tests and none failed, with the 120 tests that need the MariaDB
fixture skipped; the message of commit `2fd98f8` states that 1481 tests pass
with the MariaDB fixture up, so the equivalence on a running server no longer
rests on the identity of the compiled crates alone. The figures in
`BENCHMARKS.md` were taken on a binary without SQLite; a figure taken after the
change comes from a different artefact.

**The suite tests the library the binary carries.** `trace` reaches no feature
of `libsqlite3-sys`, and no flag its build script passes depends on the
profile, so the test build compiles the SQLite library with the compile-time
options of the shipped one, and the test of point 3 and every observation over
the fixture of `FR-LITE-038` read the options the binary carries. The feature
gates `rusqlite`'s binding, its module `trace`; SQLite's own trace interface is
compiled into the library in both builds, whose options do not omit it.

**The compiler keeps the trace out of the shipped build.** The module `trace`
exists only where the feature is enabled, so a call to it outside
`#[cfg(test)]` fails to compile under `cargo build --release`, and the boundary
`OD-37` draws is enforced rather than observed.

**The two MariaDB crates are ones upstream marks not for direct use.** Their
crates.io descriptions read "Not intended to be used directly" (`sqlx-core`) and
"Not for direct use; see the `sqlx` crate for details" (`sqlx-mysql`), and the
`_`-prefixed features the facade forwards to them are listed in upstream's
manifests "for conditional compilation". Nothing upstream promises them across
releases.
The exact requirement — the one the facade itself places on each of its crates
— freezes them at the pinned release, and the cost falls on a move of
`ADR-003`'s pin, which that record states.

**The condition that ends point 4 has not been met.** On 2026-11-03 neither
the maximum stable release of `sqlx`, 0.9.0, nor its `main` branch admits
`libsqlite3-sys` 0.39, so the return to the facade is not yet due. The
condition is checked at every dependency review, as `ADR-010`'s is, and at
every move of the release carried, since that move can change the
`libsqlite3-sys` release the condition names.

**The dependency cost is six crates, and the toolchain floor does not move.**
`rusqlite` 0.41.1 declares a floor of 1.89.0, `libsqlite3-sys` 0.39.1 1.88.0
and `pkg-config` 0.3.34 1.63; `fallible-iterator`,
`fallible-streaming-iterator` and `vcpkg` declare none, and so constrain
nothing. All three declared floors are below the floor `ADR-007` holds, which
the MariaDB driver sets, so this record does not move it; a `rusqlite` move is
an MSRV question under `ADR-007`'s rule. `cargo audit` over the resolved lock
file reports no advisory against any of the six. The advisories it reported on
2026-11-03 named crates of the MariaDB TLS path that this change does not move;
rmp `#320` moved those crates in the same commit, and over `Cargo.lock` at
`229edc8` it reports none.

**A move of the release carried is an edit to this record.** The manifest's
two `rusqlite` entries admit later `0.41` releases, and `rusqlite` admits later
`0.39` releases of `libsqlite3-sys`, so a lock-file update can move the SQLite
release. A move of the requirement itself edits both entries. That moves this record's parameter: it is an edit in place with
a new `decided` date, with the judgement of point 2 made again. `FR-LITE-039`
separately requires the choice to be re-verified against SQLite's release
history and vulnerability page before every release of `tpl`.

**Under R3, these facts live here alone.** The driver, its features, the
release carried, the build option and the judgement of `FR-LITE-039` are cited
as `ADR-013` from `docs/spec-technical/`. Neither the root coordination document
nor the root `README.md` names the SQLite driver at `229edc8`, so neither cites
this record; that citation is owed by the owners of those documents, not by
this register. The form in which the MariaDB driver is consumed is `ADR-003`'s.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The selection order, the reading of *official*, and the decision to compile extension loading out are the user's | rmp `#315` work log, `DECISION` entries of 2026-11-03 (comments 1389 and 1405) | 2026-11-03 |
| Points 3 and 4 were decided on the task; the alternatives of each | rmp `#315` work log, `DECISION` entries of 2026-11-03 (comments 1418 and 1419) | 2026-11-03 |
| The user confirmed points 3 and 4, and chose the return to the facade at the release named in point 4 | The user's answers of 2026-11-03, relayed by the session coordinator for rmp `#315` | 2026-11-03 |
| The selection rule, the extension-loading control and its read-back, the limits, the trace, the recency rule and its clauses | `specification/sqlite-contract.md`, `FR-LITE-001`, `FR-LITE-004`, `FR-LITE-010`, `FR-LITE-011`, `FR-LITE-012`, `FR-LITE-018`, `FR-LITE-039` | 2026-11-03 |
| The sixty-fifth edition's seven controls; the sixty-seventh edition's eighth, `temp_store` at `MEMORY`, confirmed by its read-back of `2` and by a row `TEMP_STORE=1`, `TEMP_STORE=2` or `TEMP_STORE=3` of `PRAGMA compile_options` | `specification/sqlite-contract.md`, `FR-LITE-010`, at `349560e` and at `5805b5d` | 2026-11-03 |
| The sixty-ninth edition's ninth control: views disabled, `SQLITE_DBCONFIG_ENABLE_VIEW` set to `0` and read back through the same option, as defence in depth | `specification/sqlite-contract.md`, `FR-LITE-010`, at `f2d1753` and at `229edc8` | 2026-11-04 |
| `DbConfig` declares `SQLITE_DBCONFIG_ENABLE_VIEW`, "Enable or disable views", with no feature noted on it | docs.rs, `rusqlite` 0.41.1, `config::DbConfig` | 2026-11-04 |
| The reader sets views off with `set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_VIEW, false)` and reads the option back with `db_config`, beside defensive mode and `trusted_schema`; a test reads it back on the reader's connection | `src/sqlite.rs`, `Session::harden`; `src/sqlite/tests.rs`, `fr_lite_010_every_control_and_limit_reads_back_as_set_on_the_connection`; at `229edc8` | 2026-11-04 |
| `trace` is enabled for tests alone, through a `[dev-dependencies]` entry, and not on the normal dependency | The user's decision of 2026-11-03, relayed by the session coordinator for rmp `#315` | 2026-11-03 |
| Edition 2024 defaults to resolver `"3"`, whose stated change from `"2"` is the default of `resolver.incompatible-rust-versions`; the resolver is a workspace-wide option taken from the top-level package; features enabled on dev-dependencies "will not be unified when those same dependencies are used as a normal dependency, unless those dev-dependencies are currently being built", and `cargo test` or `cargo build --all-targets` unify them | The Cargo Book, *Dependency Resolution*, *Feature resolver version 2* and *Resolver versions*; *Features*, *Feature resolver version 2* (`doc.rust-lang.org/cargo/reference/`) | 2026-11-03 |
| With the two entries of point 1 under edition 2024, `rusqlite` resolves with `bundled`, `limits` and `modern_sqlite` when no dev-dependency is built and with `trace` besides when one is; `libsqlite3-sys` 0.39.1 has the same features in both; a dev entry without `default-features = false` adds `cache`, `default`, `ffi-sqlite-wasm-rs` and `hashlink` | `cargo tree -e normal` and `-e normal,dev`, `-f '{p} [{f}]'`, cargo 1.99.1 on `aarch64-apple-darwin`, over a throwaway package declaring only those entries | 2026-11-03 |
| `trace` gates `rusqlite`'s module `trace`, which holds `Connection::trace_v2`; `pragma_update` and `pragma_query_value` are not gated | The published `.crate` of `rusqlite` 0.41.1, sha256 `da9bba6f…d91a594`: `src/lib.rs`, `src/pragma.rs`, `src/trace.rs` | 2026-11-03 |
| The hook that installs the trace is `#[cfg(test)]` in `sqlite/` | `docs/spec-technical/open-decisions.md`, `OD-37`, at `ad653f5` | 2026-11-03 |
| `release.yml` packages `target/<target>/release/tpl`, which `build --release` produces; its `test` step runs without `--release` | `.github/workflows/release.yml` at `5805b5d` | 2026-11-03 |
| No Rust driver is published by the SQLite developers: the source tree's `ext/` holds the language bindings `jni` and `wasm` and none for Rust; the owners of `rusqlite` and `libsqlite3-sys` (Thom Chiovoloni, gwenn), `sqlite` (Ivan Ukhov), `sqlx-sqlite` (Austin Bonander, Ryan Leckey) and `sqlite3` (Jovansonlee Cesar) are not SQLite developers | `https://sqlite.org/src/dir?ci=trunk&name=ext`; crates.io API, `/owners` of each crate | 2026-11-03 |
| Step 1 and step 2 per candidate: controls failed (`rusqlite` 1, `sqlite` 5, `sqlx` 6, `turso` 8); catalogue readings with value types and `NULL` preserved; `sqlx` always sends `PRAGMA foreign_keys`; the build with the omission answering `3.54.1`, listing `OMIT_LOAD_EXTENSION` and refusing `load_extension()`; no `sqlite3_load_extension` symbol in either `musl` build, taken with zig 0.17.0; 1392 tests passed and 120 needing the fixture skipped | rmp `#315` work log, `TEST` and `FINDING` entries of 2026-11-03 (comments 1401, 1402, 1404, 1417 and 1420) | 2026-11-03 |
| `DbConfig` has 21 variants and no `SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION`; `limit` and `set_limit` require the `limits` feature and `trace_v2` the `trace` feature; `get_interrupt_handle` returns an `InterruptHandle` whose `interrupt` interrupts the query running on another thread; new connections "currently have a default busy timeout of 5000ms" | docs.rs, `rusqlite` 0.41.1, `config::DbConfig`, `Connection`, `InterruptHandle` | 2026-11-03 |
| `rusqlite` 0.41.1, published 2026-10-28, is the maximum stable release; it declares `rust-version` 1.89.0; features `bundled`, `limits`, `trace` and `load_extension` exist, and the default set is `cache` and `ffi-sqlite-wasm-rs`; `trace` enables no other feature and no dependency, and `cache` enables `hashlink`; it requires `libsqlite3-sys` `^0.39.0` | crates.io API, `rusqlite` 0.41.1, version, features and dependencies | 2026-11-03 |
| `libsqlite3-sys` 0.39.1, published 2026-10-25, is the maximum stable release, declares `rust-version` 1.88.0 and `links = "sqlite3"`; it bundles the amalgamation `sqlite3/sqlite3.c` with `SQLITE_VERSION` `3.54.1`, source identifier `2026-10-16 14:45:20 555e07ef…`; its build script defines `SQLITE_ENABLE_LOAD_EXTENSION=1` and `SQLITE_ENABLE_FTS5`, and appends each `-D` or `-U` token of `LIBSQLITE3_FLAGS` after its own flags; it defines `SQLITE_TEMP_STORE` only under the feature `bundled-sqlcipher` and for Android, and no flag it passes depends on the profile; the amalgamation contains no `zipfile`, and defines the default `SQLITE_TEMP_STORE` `1` before `ctime.c`, whose list carries `TEMP_STORE=` and the value wherever the macro is defined | The published `.crate`, sha256 `14a5f01b…f9a2b3`: `Cargo.toml`, `build.rs`, `sqlite3/sqlite3.h`, `sqlite3/sqlite3.c` | 2026-11-03 |
| 3.54.1, of 2026-10-16, is the most recent SQLite release | `https://sqlite.org/changes.html` | 2026-11-03 |
| The vulnerability page was last updated 2026-08-01; the latest release it names as a fix is 3.53.2; `CVE-2026-11822` and `CVE-2026-11824` need arbitrary SQL, `SQLITE_DBCONFIG_DEFENSIVE` disabled and FTS5, fixed in 3.53.2; `CVE-2025-70873` is in the `zipfile` extension, fixed in 3.52.0 | `https://sqlite.org/cves.html` | 2026-11-03 |
| `SQLITE_OMIT_LOAD_EXTENSION` "omits the entire extension loading mechanism", including `sqlite3_enable_load_extension()` and `sqlite3_load_extension()`; the `SQLITE_OMIT_*` options "are mostly unsupported" and "may not work with the amalgamation". Page last updated 2026-10-01. `SQLITE_TEMP_STORE` `1` is the default and means "Use files by default but allow the PRAGMA temp_store command to override" | `https://sqlite.org/compile.html` | 2026-11-03 |
| Cargo probes for configuration in the current directory and its parents; an `[env]` value does not override the environment unless `force` is set; `cargo install` ignores local configuration unless installing with `--path` | The Cargo Book, *Configuration* (hierarchical structure, `[env]`) and *cargo install* (configuration discovery) | 2026-11-03 |
| For the purpose of generating `Cargo.lock`, the resolver builds the graph as if all features of all workspace members were enabled; it admits one package per `links` name and otherwise fails | The Cargo Book, *Dependency Resolution*, *Features* and *links* | 2026-11-03 |
| `Cargo.lock` carries `libsqlite3-sys` 0.37.0, and `cargo tree -e normal,build -i libsqlite3-sys` prints nothing | This repository at `833bc35` | 2026-11-03 |
| `sqlx-sqlite` 0.9.0 requires `libsqlite3-sys` `>=0.30.1, <0.38.0`; on `main` at `7908fe79`, committed 2026-10-31, it requires `>=0.30.1, <0.39.0`; `sqlx` 0.9.0 is the maximum stable release and requires each of its crates at `=0.9.0` | crates.io API, `sqlx-sqlite` and `sqlx` 0.9.0 dependencies; `transact-rs/sqlx`, `sqlx-sqlite/Cargo.toml` on `main` | 2026-11-03 |
| `sqlx-core` is "Not intended to be used directly" and `sqlx-mysql` "Not for direct use; see the `sqlx` crate for details"; the facade's manifest lists its `_rt-*` features under "for conditional compilation", as `sqlx-core`'s does its `_rt-*` and `_tls-*` features | crates.io, crate descriptions; `transact-rs/sqlx` at tag `v0.9.0`, `Cargo.toml` and `sqlx-core/Cargo.toml` | 2026-11-03 |
| The crates entering the shipped graph, `sqlx-core`'s and `sqlx-mysql`'s versions and features unchanged, and the lock file at 182 and 154 packages | `cargo tree -e normal,build -f '{p} [{f}]'`, once per target of `ADR-008`, at `833bc35` and over a copy of its manifest carrying point 4 and `rusqlite`, its lock re-resolved from `833bc35`'s | 2026-11-03 |
| The crates and compilations leaving the shipped graph: `sqlx`, `sqlx-macros`, `sqlx-macros-core` and `hex` leave; `sha2` 0.10.9, `digest` 0.10.7 and `crypto-common` 0.1.6 stay, compiled without `std`, and only their compilation with `std`, and those of `sqlx-mysql` and `thiserror` for the macro side, leave; `rusqlite` compiles with `bundled`, `limits` and `modern_sqlite`; besides these, only `tokio`, `rustls`, `rustls-webpki` and `webpki-roots` change, by patch release | `cargo tree --locked --offline -e normal,build --target <triple> --prefix none -f '{p} [{f}]'`, once per target of `ADR-008`, over this repository at `229edc8` and over a copy of the manifest and `Cargo.lock` at `833bc35`, the patch pointed at this repository's vendored tree | 2026-11-04 |
| `sqlx-core`'s feature `migrate` enables `sha2`, which it requires at `0.10.0` | `vendor/sqlx-core-0.9.0/Cargo.toml`, `[features]` and `[dependencies.sha2]` | 2026-11-04 |
| The version moves of `tokio`, `rustls`, `rustls-webpki` and `webpki-roots` are rmp `#320`'s; 1481 tests pass with the MariaDB fixture up | The message of commit `2fd98f8`. The test run was not repeated here | 2026-11-04 |
| No advisory against `rusqlite` 0.41.1, `libsqlite3-sys` 0.39.1, `fallible-iterator` 0.3.0, `fallible-streaming-iterator` 0.1.9, `pkg-config` 0.3.34 or `vcpkg` 0.2.15; the advisories reported name `rustls`, `rustls-webpki`, `webpki-roots` and, as a warning, `tokio` | `cargo audit` 0.22.2, advisory database of 2026-11-03 holding 1403 advisories, over the evaluation's lock file, re-resolved from `833bc35`'s | 2026-11-03 |
| Over `Cargo.lock` at `229edc8`, no vulnerability and no warning is reported | `cargo audit` 0.22.2, `--no-fetch`, over the local advisory database holding 1403 advisories | 2026-11-04 |
| Declared floors: `pkg-config` 0.3.34 → 1.63; `fallible-iterator` 0.3.0, `fallible-streaming-iterator` 0.1.9 and `vcpkg` 0.2.15 declare none | crates.io API, one request per crate | 2026-11-03 |
| The workflows invoke Cargo at the repository root and run `cargo test` for each of the four targets | `.github/workflows/ci.yml` and `.github/workflows/release.yml` at `833bc35` | 2026-11-03 |
| `docs/spec-technical/` cites `ADR-013`; neither `CLAUDE.md` nor `README.md` names SQLite | `rg ADR-013 docs/spec-technical/` and `rg -i sqlite README.md CLAUDE.md`, at `229edc8` | 2026-11-04 |
