---
id: ADR-014
title: The driver of the mysql provider
status: accepted
decided: 2026-11-04
last-reviewed: 2026-11-04
requirements: [FR-MYSQL-045, FR-MYSQL-022, FR-MYSQL-021, FR-MYSQL-020, FR-CONF-036]
supersedes: []
superseded-by: null
---

# ADR-014 — The driver of the `mysql` provider

## Status

Accepted, 2026-11-04, for rmp `#327`.

The rule this record applies was fixed in advance by the session coordinator,
under the user's decision that became `FR-MYSQL-045`: if `sqlx-mysql` 0.9.0
requests the server's RSA public key on a session TLS does not protect, and no
option prevents it, the crate is vendored beside `vendor/sqlx-core-0.9.0/` with
the request patched out; otherwise no patch is needed. The evidence in
*Context* places it in the first branch.

**Every decision in this record is the user's, of 2026-11-04, relayed by the
session coordinator**: the rule and the branch it yields; the form of the
refusal, Decision point 3; what `tpl` does with it, point 4, including the
outcome a `mariadb` entry keeps; and the module the reader lives in, for which
`ADR-005` was amended in place the same day.

## Context

**The corpus names no driver for the `mysql` provider and hands the choice
out.** The scope of `specification/mysql-contract.md` excludes "the choice of
the database driver, which is an architecture decision under `FR-CONF-036` and
`FR-MYSQL-022`", and names no home for it. The record is admitted under rule
R4's second limb: the choice changes what the package is built against, and its
alternatives must survive it, because the patch it carries has no upstream end
in sight and the question returns at every move of `ADR-003`'s pin.

**Five requirements constrain the driver.** `FR-CONF-036` disqualifies a driver
that cannot express the five modes, and `FR-MYSQL-020` and `FR-MYSQL-021` hold a
`mysql` entry to the same modes and the same behaviour against a MySQL server.
`FR-MYSQL-022` disqualifies one that cannot authenticate `caching_sha2_password`
under every mode that encrypts. `FR-MYSQL-045` forbids requesting the server's
RSA public key on a session TLS does not protect, whatever the authentication
method, ends such a login with `78`, and lets no flag, configuration key or
environment condition change this.

**What the pinned driver does.** It was read in `transact-rs/sqlx` at tag
`v0.9.0`, the repository and tag `ADR-003` names. Paths below are under
`sqlx-mysql/`, and line numbers are at the tag. The published crate is the same
source: its checksum is the one `Cargo.lock` records, and its `src/` and manifest
are identical to the tag's (Sources).

- *`caching_sha2_password`.* When the server answers the scramble with
  `perform_full_authentication` (`0x01 0x04`), `AuthPlugin::handle` calls
  `encrypt_rsa` with the request byte `0x02` (`src/connection/auth.rs:52–53`)
  and writes what it returns (`55–56`). On a stream that is not TLS,
  `encrypt_rsa` writes that byte as a packet and flushes (`139–141`), reads the
  server's key (`143–145`), XORs the NUL-terminated password with the nonce
  (`147–155`) and encrypts it (`157–158`). Only on a TLS stream does it return
  the password in clear instead (`134–137`).
- *`sha256_password`.* `AuthPlugin::scramble` calls `encrypt_rsa` with `0x01`
  (`auth.rs:26`), both for the initial handshake response
  (`src/connection/establish.rs:102–106`) and after an authentication switch
  (`128–146`). On a stream that is not TLS it writes the same kind of request —
  on the initial path, before the handshake response itself.
- *Which streams are not TLS.* `is_tls` is true only on the stream the TLS
  upgrade returns (`src/connection/tls.rs:92–100`) and false otherwise
  (`src/connection/stream.rs:58`). Under `Disabled` the upgrade is skipped
  (`tls.rs:23–30`); under `Preferred` it is skipped where the server does not
  advertise TLS (`tls.rs:39–43`). These are the two sessions `FR-MYSQL-045`
  names.
- *No option prevents the request.* `MySqlConnectOptions` holds no field about
  the key (`src/options/mod.rs:63–83`), and neither its setters (`121–413`) nor
  the URL parameters (`src/options/parse.rs:50–79`) reach one. The crate feature
  `rsa` (`Cargo.toml:18`, not a default) selects the encryption backend and
  nothing else. This package builds without it, and then `encrypt_rsa` still
  writes the request and reads the key, and only afterwards fails with
  `Error::Configuration`, "RSA auth backend disabled; …" (`auth.rs:204–213`).
  **The package as built today requests the key and then fails the login**,
  without sending the password; the request alone is what `FR-MYSQL-045`
  forbids.
- *No release to wait for.* On `main` at `7908fe79`, of 2026-10-31,
  `encrypt_rsa` still writes the request on that path and the options gain no
  switch; `sqlx` 0.9.0 is still the maximum stable release.

**Nothing else in the crate stands in the way of `8.4` or `9.7`.** The
handshake's version string is parsed leniently, stored, and consulted by nothing
(`establish.rs:68–95`), so the window of `FR-MYSQL-006` to `FR-MYSQL-010` is
`tpl`'s alone. `caching_sha2_password` is implemented on both paths
`FR-MYSQL-022` needs: the fast path scrambles in the server's order, with
upstream's regression test for its issue `#4244` (`auth.rs:101–124`, `221–245`),
and full authentication over TLS sends the password in clear (`134–137`).
`mysql_native_password`, disabled by default in `8.4` and removed as of MySQL
9.0.0, is implemented and not needed. The client's capabilities
(`stream.rs:35–46`) and the handshake's decoding of MySQL's reserved bytes
(`src/protocol/connect/handshake.rs:57–62`) are those a `mariadb` entry already
uses. The statement the driver issues after
the handshake (`src/options/connect.rs:52–89`) is issued only if one of four
options is set; the MariaDB path sets all four off
(`src/mariadb/connect.rs:425–431`), which a `mysql` entry needs as well under
`FR-MYSQL-016`. Three facts are bounded rather than obstacles:

- the decoder refuses a result column whose wire type it does not list
  (`src/protocol/text/column.rs:261–263`). The closed list of `FR-MYSQL-015`
  reads `INFORMATION_SCHEMA` fields alone, and a `VECTOR` column reaches the
  read as text, per difference 1 of `FR-MYSQL-014`; that every field of the
  twelve tables arrives on a listed wire type is for the provider's tests to
  observe;
- after an authentication switch, the follow-up exchange uses the handshake's
  nonce rather than the switch's (`establish.rs:133` shadows it), which `main`
  corrects. Only the RSA exchange reads the nonce at that point, and the patch
  removes it;
- an empty password is scrambled rather than sent empty (`auth.rs:21`, `23`),
  which `main` also corrects. It is reached only when a password is set to the
  empty string; the MariaDB path sets none where the entry carries none
  (`src/mariadb/connect.rs:437–439`).

## Decision

1. **The `mysql` provider reads through the driver of `ADR-003`.** `sqlx-mysql`
   0.9.0 serves both providers, at the pin, in the form and with the features
   `ADR-003` records; `rsa` stays off. No crate is added. The mapping of the
   five modes is `ADR-002`'s and applies to a `mysql` entry unchanged, since
   `FR-MYSQL-020` connects one exactly as a `mariadb` entry. With the patch
   below, the driver meets the three constraints: it expresses the five modes;
   it authenticates `caching_sha2_password` under every mode that encrypts, on
   the evidence of its source, which the provider's tests observe; and it
   requests no key on a session TLS does not protect.

2. **The package patches the driver rather than living with the request.** The
   root manifest's `[patch.crates-io]` table redirects `sqlx-mysql` to
   `vendor/sqlx-mysql-0.9.0/`, which holds the published 0.9.0 source with the
   change of point 3 and nothing else. Everything else about the form is
   `ADR-010`'s, applied to a second tree and not restated: a tracked copy
   redirected by path, the version in the directory name, maintained by hand,
   excluded from the workspace, its provenance in `vendor/README.md`. The
   licence texts already at `vendor/` serve this tree as they serve that one:
   the published crate carries the same two stubs, `../LICENSE-APACHE` and
   `../LICENSE-MIT`, and was packaged from the same upstream revision as
   `sqlx-core` 0.9.0.

3. **The change, in full.** It is confined to `src/connection/auth.rs`, save
   one export:

   - in `AuthPlugin::handle`, the arm for `perform_full_authentication`
     (`0x04`, lines 52–59): before `encrypt_rsa` is called, the arm returns the
     refusal when the stream is not TLS;
   - in `AuthPlugin::scramble`, the arm for `sha256_password` (line 26): the
     same, before its call to `encrypt_rsa`;
   - the refusal is `Error::Configuration` — the variant the published crate
     already returns on this path when built without `rsa` — boxing a value of
     an error type declared for this refusal alone, carrying the method's name as
     `AuthPlugin::name` gives it (`src/protocol/auth.rs:16–23`). The type is
     exported from the crate root, beside the exports of `src/lib.rs:40–51`.

   `encrypt_rsa` is not edited. Those two arms are its only callers, so after
   the change nothing writes a public-key request on a stream that is not TLS,
   and on a TLS stream every path runs as published. The refusal is returned
   before the request is written, so the connection ends without completing the
   authentication, as `FR-MYSQL-045` requires. No other function, file, feature
   or manifest entry differs from the published crate; any other difference is a
   defect in the tree.

4. **What `tpl` does with the refusal.** It recognises the refusal by its type,
   never by its text, as the boundary of `src/mariadb/fault.rs` reads the
   driver's discriminants alone. For a `mysql` entry the refusal is the
   condition of `FR-MYSQL-045`, exit `78`: it carries the method's name, which
   the `cause` names, and since it is raised only on a stream that is not TLS,
   under `preferred` it shows that the server offered no TLS
   (`tls.rs:39–43`). For a `mariadb` entry it keeps exactly the outcome the
   same login has today (*Consequences*).

5. **The condition that retires the patch: the first `sqlx` release whose
   `sqlx-mysql` lets a caller refuse the request.** At that release the patch
   entry for `sqlx-mysql` and its tree are deleted, `tpl` sets the refusal on
   every connection it opens, and points 2 to 4 are amended in place. It was not met on
   2026-11-04. Until it is, the patch is carried across every move of
   `ADR-003`'s pin: the tree is vendored again at the release moved to, and the
   change is re-applied and checked against that release's published source.
   `main` already edits both functions the change touches, so the re-application
   will not be mechanical.

## Alternatives rejected

- **The published crate unchanged.** It is today's build, and the absent `rsa`
  backend refuses only after the request and the key have crossed the wire
  (`auth.rs:139–145`, before `158`); `FR-MYSQL-045` forbids the request itself.
  Enabling `rsa` would instead complete the login with a password encrypted
  under a key received over the unprotected session, which is what the user's
  decision forbids.

- **Wrapping the driver instead of patching it.** Unavailable. One `connect`
  covers the TCP connection, the TLS upgrade and the authentication, and
  `MySqlConnectOptions` accepts no connected socket or stream, so nothing
  outside the crate can see or withhold the request. A preliminary login to
  learn whether the server would ask for full authentication is a second
  connection, which `NFR-PERF-004` forbids, and it meets the same path.

- **A switch in the options that only a `mysql` entry sets**, leaving a
  `mariadb` entry on the published behaviour. Refused: it widens the divergence
  to the options type, a setter and the stream's state, to preserve a behaviour
  that only ever ends in failure — the published crate, built without `rsa`,
  completes no login that needs the key (*Consequences*).

- **The refusal inside `encrypt_rsa`.** It is the one function that writes the
  request, so a guard there covers every caller by construction. Refused
  because it does not know the method, only its request byte, so naming the
  method would need a mapping the crate does not hold; and because the code
  after the guard would be left unreachable, either kept as dead code or deleted
  as a second change. That the two arms of point 3 are its only callers is
  re-established at every re-application.

- **Recognising the refusal by its message, or by `Error::Configuration`
  alone.** The message is refused by the boundary `src/mariadb/fault.rs` draws,
  which reads the driver's discriminants and never its text (lines 19–27). The
  variant alone cannot name the method, and the TLS set-up returns it as well
  for client-certificate errors
  (`vendor/sqlx-core-0.9.0/src/net/tls/tls_rustls.rs:119–123`, `203–208`), so it
  would identify the condition only through an argument about which options
  `tpl` sets.

- **`mysql_async` 0.37.1**, an asynchronous MySQL driver at its maximum stable
  release. Disqualified by `FR-CONF-036` on the first of the two counts on
  which `ADR-003` refused the `mysql` crate: it negotiates TLS exactly when TLS
  options are given (`src/opts/mod.rs:1167–1168`) and then fails against a
  server that offers none (`src/conn/mod.rs:561–569`), so `preferred` cannot be
  expressed. It would not spare the patch: on a connection neither encrypted
  nor made through a socket it writes the same `0x02` request
  (`src/conn/mod.rs:812–818`), and no option prevents it — `secure_auth`
  disables `mysql_old_password` alone (`src/opts/mod.rs:1095–1100`). It would also be a second driver stack for a
  protocol the first already speaks. These facts were read at the published
  crate and not tested against servers: enough to disqualify, which is all they
  are used for here, and not enough to qualify, per `ADR-003`.

- **A fork of `sqlx` pinned by revision.** Refused on the grounds `ADR-010`
  records for the same choice. They weigh more here: this divergence has no
  upstream end, so the second repository would be kept for as long as upstream
  offers no switch.

- **Exit `78` for a `mariadb` entry, as for a `mysql` entry.** Refused by the
  user on 2026-11-04: it would change what a `mariadb` entry does, and the
  user's goal for the `mysql` provider is that nothing about MariaDB support
  changes (`FR-MYSQL-001`, *Provenance*).

## Consequences

**A `mariadb` entry runs through the patched code, and keeps exactly its present
outcome.** MariaDB offers `caching_sha2_password` too: it documents a
server-side plugin, "available from these versions: MariaDB Community Server
11.4.9 / 11.8.4 / 12.1", in the library `auth_mysql_sha2`, not installed by
default, with its RSA key pair generated by default; MariaDB Server does not
support `sha256_password`. Three of the four series of `FR-SRV-015` therefore
have releases that offer it. Assuming the plugin asks for full authentication as
MySQL's does — its documentation calls it MySQL-compatible, and nothing here
observed it — a login on such a server, by an account using it, with no cached
login and on a session TLS does not protect, ends today with the published crate
requesting the key and then failing; `src/mariadb/fault.rs` classifies the
failure on its row for anything else, `Error::ConnectionRefused`, exit `69`.
**That outcome stays exactly as it is — exit `69`, with the same message — and
the `mariadb` path SHALL map the typed refusal to it and to no other.** The one
difference is that the request is never sent. The refusal is neither an error
packet nor a TLS failure, so the classification as it stands already reaches
that row; what the obligation forbids is any change that gives the refusal an
outcome of its own for a `mariadb` entry. No login that succeeds today fails,
and none that fails succeeds.

**The reader needs the runtime, and `ADR-005` covers it.** A reader of a
`mysql` entry built on this driver is asynchronous. It lives in `mysql/`, a
module of its own, and `ADR-005`, amended in place on 2026-11-04, holds where
the runtime is built.

**`cargo audit` stops matching `sqlx-mysql`.** The path patch deletes the
crate's `source` from `Cargo.lock`, as it did for `sqlx-core` (`ADR-010`). The
compensating lookup `ADR-010` owes for `sqlx-core` is owed for `sqlx-mysql` as
well, at the same visits, for as long as this patch stands. It was made on
2026-11-04: the advisory database holds no directory for `sqlx-mysql`.

**The other obligations `ADR-010` states for a vendored tree hold for this
one**, and are not restated: the licence terms, which the files already in
`vendor/` meet; the prohibition on `unsafe`, which does not reach third-party
source; and the crate not counting as a new dependency, being one the pin
already names.

**The toolchain floor does not move.** `sqlx-mysql` 0.9.0 declares
`rust-version` 1.94.0, the floor `ADR-007` holds.

**No artefact built under this patch is the published crate**, as `ADR-010`
states for its own; a figure taken on such a build names its commit.

**The register is amended with this record.** `ADR-003` names it among the
obligations of a pin move and among what it does not decide, and `ADR-010`'s
retiring paragraph no longer counts the obligations `ADR-003` carries.
`docs/spec-technical/technology-stack.md` and the root coordination document's
stack row are to cite this record; those citations are owed by the owners of
those documents. No crate is adopted, so the *Stack* table's own rule calls for
no change to it.

**Under R3, these facts live here alone:** that the `mysql` provider reads
through this driver, the patch and its scope, and the condition that retires
it.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The constraints on the driver, and the corpus's statement that the choice is an architecture decision | `specification/mysql-contract.md`, seventy-fifth edition at `d4eaf76`: *Scope*, `FR-MYSQL-020`, `FR-MYSQL-021`, `FR-MYSQL-022`, `FR-MYSQL-045`, row 25 of `FR-MYSQL-040`; `specification/configuration-model.md`, `FR-CONF-036` | 2026-11-04 |
| The rule applied | The session coordinator's brief for rmp `#327` | 2026-11-04 |
| The user's decisions: the rule and its branch; the two guards, the typed refusal and its export; exit `78` for a `mysql` entry; exit `69` with the present message for a `mariadb` entry, and `78` refused there; the reader in `mysql/`, not inside `mariadb/` | The user's decision of 2026-11-04, relayed by the session coordinator for rmp `#327` | 2026-11-04 |
| The user's goal that nothing about MariaDB support changes | `specification/mysql-contract.md`, `FR-MYSQL-001`, *Provenance*, at `d4eaf76` | 2026-11-04 |
| The tag `v0.9.0` of `transact-rs/sqlx` is a lightweight tag on commit `75bc0487eb661da811bb7a3c5d158f1bd463fef4`, "Release 0.9.0 (#4256)"; from `003b698e99e024f3621b8043a2426fde5b741171` to it, only `README.md` changes | `git ls-remote` of the repository; a clone at the tag; `git diff --stat` between the two commits | 2026-11-04 |
| The published `sqlx-mysql` 0.9.0 has sha256 `90b8020f…acebea27`, the checksum `Cargo.lock` records; its `.cargo_vcs_info.json` names `003b698e…`; its `src/` and `Cargo.toml.orig` are identical to `sqlx-mysql/` at the tag | `static.crates.io`, `sqlx-mysql-0.9.0.crate`; `diff -r` against the clone; `Cargo.lock` at `d4eaf76` | 2026-11-04 |
| The full-authentication path, `encrypt_rsa`, the `rsa` backends, the scramble functions, and the two calls of `encrypt_rsa`, at lines 26 and 53 and nowhere else | `sqlx-mysql/src/connection/auth.rs` at `v0.9.0` | 2026-11-04 |
| The calls of `scramble` on the initial handshake and after a switch; the nonce shadowed after a switch; the version parse | `sqlx-mysql/src/connection/establish.rs` at `v0.9.0`, lines 68–95, 102–106, 128–153 | 2026-11-04 |
| `is_tls` is set only by the upgrade, which `Disabled` skips and `Preferred` skips without server TLS | `sqlx-mysql/src/connection/tls.rs` lines 17–54 and 92–100, and `src/connection/stream.rs` line 58, at `v0.9.0` | 2026-11-04 |
| No field, setter or URL parameter reaches the request | `sqlx-mysql/src/options/mod.rs` lines 63–83 and 121–413, and `src/options/parse.rs` lines 50–79, at `v0.9.0` | 2026-11-04 |
| `rsa` is an optional feature outside the default set, enabling `rand` and `rsa` | `sqlx-mysql/Cargo.toml` at `v0.9.0`, lines 13 and 18 | 2026-11-04 |
| This package enables `any`, `json`, `migrate` and `offline` on `sqlx-mysql`, and locks no `rsa` package | `Cargo.toml` lines 72–77 and `Cargo.lock`, at `d4eaf76` | 2026-11-04 |
| The checks that found no obstacle: client capabilities, handshake decoding, the statement after the handshake, the method names, column-type decoding, the crate's exports | `sqlx-mysql/src/connection/stream.rs` 35–46; `src/protocol/connect/handshake.rs` 57–78; `src/options/connect.rs` 52–89; `src/protocol/auth.rs` 8–38; `src/protocol/text/column.rs` 227–265; `src/lib.rs` 40–51; at `v0.9.0` | 2026-11-04 |
| The MariaDB path sets the four options off and sets no password where the entry carries none; the driver-error boundary reads discriminants only, and classifies anything else as `69` | `src/mariadb/connect.rs` lines 421–439, and `src/mariadb/fault.rs` lines 11–27, at `d4eaf76` | 2026-11-04 |
| `Error::Configuration` is also returned by the TLS set-up, for client-certificate errors | `vendor/sqlx-core-0.9.0/src/net/tls/tls_rustls.rs` lines 119–123 and 203–208 | 2026-11-04 |
| On `main` at `7908fe79`, of 2026-10-31: `encrypt_rsa` still writes the request (`auth.rs:153` there); the options gain no switch; `scramble` sends an empty password empty; `handle` checks the packet's length; the switch's nonce is kept | `git diff` from `v0.9.0` to `7908fe79` over `sqlx-mysql/src/connection/auth.rs`, `src/connection/establish.rs` and `src/options/mod.rs` | 2026-11-04 |
| `sqlx`, `sqlx-core` and `sqlx-mysql` 0.9.0, of 2026-05-21, are the maximum stable releases | crates.io API, the three crates | 2026-11-04 |
| `mysql_async` 0.37.1, of 2026-09-01, is the maximum stable release; it negotiates TLS exactly when TLS options are given and fails against a server that offers none; it writes the `0x02` request on a connection neither secure nor made through a socket; `secure_auth` disables `mysql_old_password` | The published `.crate`, sha256 `40d11da0…270071e4`, packaged from `blackbeam/mysql_async` at `ce4b2769`: `src/opts/mod.rs` 1095–1100 and 1167–1168, `src/conn/mod.rs` 561–569 and 812–818; crates.io API | 2026-11-04 |
| MariaDB's server-side `caching_sha2_password`: available from Community Server 11.4.9, 11.8.4 and 12.1, in `auth_mysql_sha2`, not installed by default, its RSA key pair generated by default; called MySQL-compatible | `https://mariadb.com/docs/server/reference/plugins/authentication-plugins/authentication-plugin-caching_sha2_password`, and the index page `https://mariadb.com/docs/server/reference/plugins/authentication-plugins` | 2026-11-04 |
| MariaDB Server does not support `sha256_password` | `https://mariadb.com/docs/server/reference/plugins/authentication-plugins/authentication-plugin-sha-256` | 2026-11-04 |
| MySQL 9.7 keeps `sha256_password`, deprecated, and sends its RSA public key to the client as needed | `https://dev.mysql.com/doc/refman/9.7/en/sha256-pluggable-authentication.html` | 2026-11-04 |
| `mysql_native_password` is "disabled by default in MySQL 8.4, and removed as of MySQL 9.0.0" | `https://dev.mysql.com/doc/refman/8.4/en/native-pluggable-authentication.html`, which the same path under `9.7` serves | 2026-11-04 |
| The published `sqlx-mysql` 0.9.0 carries `LICENSE-APACHE` and `LICENSE-MIT` as 17- and 14-byte stubs reading `../LICENSE-APACHE` and `../LICENSE-MIT`, and declares `MIT OR Apache-2.0` and `rust-version` 1.94.0 | The published `.crate` above | 2026-11-04 |
| The advisory database holds no directory for `sqlx-mysql` | `rustsec/advisory-db`, `crates/sqlx-mysql`: absent, HTTP 404 from the GitHub contents API | 2026-11-04 |
| Where the runtime is built, for a `mariadb` and for a `mysql` entry | `ADR-005`, *Decision*, as amended in place on 2026-11-04 | 2026-11-04 |
