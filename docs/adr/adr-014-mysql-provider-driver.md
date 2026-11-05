---
id: ADR-014
title: The driver of the mysql provider
status: accepted
decided: 2026-11-05
last-reviewed: 2026-11-05
requirements: [FR-MYSQL-045, FR-MYSQL-022, FR-MYSQL-021, FR-MYSQL-020, FR-CONF-036, FR-CONF-055, FR-MYSQL-047, FR-CONF-005, FR-ERR-027, FR-ERR-047, FR-ERR-048, FR-ERR-049]
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
refusal, Decision point 3; what `tpl` does with it, point 4; the two further
parts of the patch, points 5 and 6; and the module the reader lives in, for
which `ADR-005` was amended in place the same day.

**Decision point 3 was corrected in place on 2026-11-04**, for rmp `#329`, once
the tree was vendored. It said the change was confined to `auth.rs` save one
export from the crate root. The type cannot be exported from the root without a
second line, a re-export in `src/connection/mod.rs`, because `auth` is a private
module of `connection`; the point now names that line, the type, and the private
function that builds the refusal. The guards, the type's purpose and the error
it is boxed in did not change, so the decision did not change, and `decided` is
unchanged.

**Amended in place on 2026-11-04**, for the joined effort of rmp `#332` to
`#338`, against the seventy-seventh edition of the specification, by the user's
decisions of that day. Point 4 named, for a `mariadb` entry, the classification
the same login had before the `mysql` provider existed, with its message; that
outcome is now the specification's, the third outcome of `FR-ERR-047`, and the
point cites it. Points 5 and 6 are new: the patch now also presents an empty or
absent password as `FR-CONF-055` requires, and reports the phase a connection
reached, as `FR-CONF-005` requires and no solution without a patch delivers.
Point 7, the condition that retires the patch, was point 5 and now holds one
condition per part. The driver, the vendored form and the refusal of point 3
did not change; the record's subject — the driver and the patch it carries —
is the same, and the new parts were decided the same day, so this is an edit in
place, as `ADR-013` was for its test-only feature, and `decided` is unchanged.

**Amended in place on 2026-11-05**, for rmp `#342`, by the user's decision of
that day: the patch also carries the whole of upstream pull request `#4458`,
point 8, so that a short or malformed packet from the server ends in an
`Error::Protocol` and not in a panic, as `FR-ERR-047` and `FR-ERR-048` require.
Point 7 gains that part's retiring condition. The driver, the vendored form
and points 3 to 6 are unchanged; the record's subject is the same, and a new
part of the patch is a parameter of point 2, so this is an edit in place, and
`decided` moves to the day it was decided. The claims re-read that day are
those of the *Sources* rows dated 2026-11-05.

**Amended in place on 2026-11-05**, for rmp `#340`, against the code of commit
`f7c5385`: `tpl` now also reads the handle of point 6 after every poll of the
call to `connect`, to time each phase as `FR-GLOB-017` requires. The handle,
the three recordings and the vendored tree are unchanged; what changed is how
often `tpl` reads its own clone, and with it the retiring condition of point 6,
which point 7 now states in full.

## Context

**The corpus names no driver for the `mysql` provider and hands the choice
out.** The scope of `specification/mysql-contract.md` excludes "the choice of
the database driver, which is an architecture decision under `FR-CONF-036` and
`FR-MYSQL-022`", and names no home for it. The record is admitted under rule
R4's second limb: the choice changes what the package is built against, and its
alternatives must survive it, because three parts of the patch it carries have no upstream end
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

**The seventy-seventh edition reaches the login twice more.** `FR-CONF-055`,
which `FR-MYSQL-047` holds for a `mysql` entry, requires an empty password, and
an entry that supplies none, to be presented as the server's own clients present
an empty password, at the first response of the login and at every switch of
the authentication method, and never as a scramble of the empty string.
`FR-CONF-005` requires the phase a connection had reached to decide where an
expired budget or a failure is reported, and not the call that was running;
`FR-ERR-027` and `FR-ERR-049` report against it. The pinned driver meets
neither, at the same tag:

- *The empty password.* `AuthPlugin::scramble` computes a scramble from
  whatever password it is given, the empty string included (`auth.rs:21`,
  `23`); for `sha256_password` it reaches `encrypt_rsa` (`26`), which on a TLS
  stream sends the empty string NUL-terminated and otherwise meets the refusal
  of point 3. The first response of the login is computed only where the entry
  has a password (`establish.rs:102–106`); without one it is empty, whatever the
  method (`src/protocol/connect/handshake_response.rs:50–62`). The response to a
  switch presents an absent password as the empty string
  (`establish.rs:139`, `unwrap_or_default()`), and so scrambles it.
- *The phase reached.* One call, `MySqlConnectOptions::connect`
  (`src/options/connect.rs:21–25`), covers the TCP connect, the greeting, the
  TLS handshake and the authentication. Where the budget expires, `tpl`'s
  timeout drops that call's future, and nothing survives to say how far it got.
  Where the call fails, an error of the TCP connect and an I/O error of the
  login arrive on one variant: sqlx-core converts the connect's error with `?`
  (`sqlx-core/src/net/socket/mod.rs:190–193`) into `Error::Io`, which carries
  the `io::Error` unchanged (`sqlx-core/src/error.rs:49`), so a kind such as
  `TimedOut` does not say which phase it arose in. The driver offers no way to
  learn it. Its options hold no hook (`src/options/mod.rs:63–83`); its log
  settings reach only the statements of an open session
  (`src/connection/executor.rs:110`); the only events on the path are two
  `tracing::debug!` calls, made where no TLS is attempted (`tls.rs:35`, `41`);
  and the login cannot be run on a socket `tpl` opens itself, although sqlx-core
  exports `connect_tcp` and `WithSocket` (`sqlx-core/src/net/mod.rs:4–6`),
  because `MySqlConnection::establish` is crate-private (`establish.rs:15`),
  `DoHandshake` is private (`37`), and the options take a Unix socket path and
  no connected stream (`src/options/mod.rs:138`). **No solution without a patch
  exists.**
- *What the published API already gives.* The `io::Error` inside `Error::Io`
  carries `kind()` and `raw_os_error()`, and an error packet arrives as a
  `MySqlDatabaseError` (`src/connection/stream.rs:141–170`) whose `number()` is
  public (`src/error.rs:24`). Classifying by them, as rmp `#337` and `#338` do,
  needs no patch. Attributing an `Error::Io` to the TCP connect rather than to
  the login needs the phase of the bullet above.

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
`FR-MYSQL-016`. Two facts are bounded rather than obstacles:

- the decoder refuses a result column whose wire type it does not list
  (`src/protocol/text/column.rs:261–263`). The closed list of `FR-MYSQL-015`
  reads `INFORMATION_SCHEMA` fields alone, and a `VECTOR` column reaches the
  read as text, per difference 1 of `FR-MYSQL-014`; that every field of the
  twelve tables arrives on a listed wire type is for the provider's tests to
  observe;
- after an authentication switch, the follow-up exchange uses the handshake's
  nonce rather than the switch's (`establish.rs:133` shadows it), which `main`
  corrects. Only the RSA exchange reads the nonce at that point, and the patch
  removes it.

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

2. **The package patches the driver rather than living with what it does.** The
   root manifest's `[patch.crates-io]` table redirects `sqlx-mysql` to
   `vendor/sqlx-mysql-0.9.0/`, which holds the published 0.9.0 source with the
   changes of points 3, 5, 6 and 8 and nothing else. Everything else about the
   form is `ADR-010`'s, applied to a second tree and not restated: a tracked copy
   redirected by path, the version in the directory name, maintained by hand,
   excluded from the workspace, its provenance in `vendor/README.md`. The
   licence texts already at `vendor/` serve this tree as they serve that one:
   the published crate carries the same two stubs, `../LICENSE-APACHE` and
   `../LICENSE-MIT`, and was packaged from the same upstream revision as
   `sqlx-core` 0.9.0.

3. **The refusal of the key request, in full.** Two guards and one type in
   `src/connection/auth.rs`, and the two lines that export the type. Line
   numbers in points 3, 5 and 6 are those of the published files, and point 8's
   those of the vendored tree before it is applied:

   - in `AuthPlugin::handle`, the arm for `perform_full_authentication`
     (`0x04`, lines 52–59): before `encrypt_rsa` is called, the arm returns the
     refusal when the stream is not TLS;
   - in `AuthPlugin::scramble`, the arm for `sha256_password` (line 26): the
     same, before its call to `encrypt_rsa`;
   - the refusal is `Error::Configuration` — the variant the published crate
     already returns on this path when built without `rsa` — boxing a
     `PublicKeyRequestRefused`. That public error type is declared in
     `auth.rs` for this refusal alone. It carries the method's name as
     `AuthPlugin::name` gives it (`src/protocol/auth.rs:16–23`) and returns it
     from `plugin()`, and it implements `Display` and `std::error::Error`. One
     private function beside it builds the boxed refusal for both guards;
   - `src/connection/mod.rs` re-exports the type,
     `pub use auth::PublicKeyRequestRefused;`, and `src/lib.rs` exports it from
     the crate root, `pub use connection::PublicKeyRequestRefused;`, beside the
     exports of lines 40–51. Both lines are needed: `auth` is a private module
     of `connection` (`src/connection/mod.rs:18`), and the crate root can name
     the type only through a re-export at that level.

   `encrypt_rsa` is not edited. Those two arms are its only callers, so after
   the change nothing writes a public-key request on a stream that is not TLS,
   and on a TLS stream every path runs as published. The refusal is returned
   before the request is written, so the connection ends without completing the
   authentication, as `FR-MYSQL-045` requires.

4. **What `tpl` does with the refusal.** It recognises the refusal by its type,
   never by its text, as the boundary of `src/mariadb/fault.rs` reads the
   driver's discriminants alone. The refusal carries the method's name, and
   since it is raised only on a stream that is not TLS, under `preferred` it
   shows that the server offered no TLS (`tls.rs:39–43`); both are facts the
   `cause` names. For a `mysql` entry the refusal is the condition of
   `FR-MYSQL-045`, exit `78`. For a `mariadb` entry it is the third outcome of
   `FR-ERR-047`, exit `69`, whose `cause` and `hint` state the facts and the
   command `FR-MYSQL-045` states; the note on `FR-SEC-021` records why the code
   differs.

5. **The empty password, in full.** Two changes, which `FR-CONF-055` requires
   for both providers:

   - in `src/connection/auth.rs`, `AuthPlugin::scramble` (lines 13–34): where
     the password is the empty string, the function returns, before it
     computes, guards or requests anything, the response for an empty password
     that the user fixed from the prior art `FR-CONF-055` records — an empty
     response for `mysql_native_password` (the arm of line 23), and a single
     `0x00` byte, as MySQL's client library sends, for `caching_sha2_password`
     (line 21) and for `sha256_password` (line 26), ahead of that arm's guard of
     point 3, because a login with no password needs no key. The arm for
     `mysql_clear_password` (lines 28–32) already sends a single `0x00` for the
     empty string and is unchanged;
   - in `src/connection/establish.rs`, `DoHandshake::do_handshake`, the first
     response of the login (lines 102–106): where the server names a method, the
     response is computed from the entry's password or, where the entry
     supplies none, from the empty string —
     `options.password.as_deref().unwrap_or_default()`, the form the switch
     already uses — so an entry without a password sends what an entry with an
     empty one sends. Where the server names no method, the response stays
     absent, as published.

   The response to a switch (lines 128–146) is not edited: it already presents
   an absent password as the empty string (line 139), so the change to
   `scramble` is the change to it. `AuthPlugin::handle` is not edited: the
   steps `FR-CONF-055` names are the first response and the response to a
   switch.

6. **The phase the connection reached, in full.** Three changes, in
   `sqlx-mysql` alone:

   - in `src/options/mod.rs`: a public handle declared for this alone, `Clone`
     and `Debug`, holding a shared atomic cell, whose one reader returns the
     phase reached as a public enumeration of three values — the TCP connect,
     which a new handle holds, the TLS handshake, and the login; one field of
     `MySqlConnectOptions` (lines 63–83) holding an optional handle, absent in
     `new()` (lines 93–115); and one setter that sets it;
   - in `src/connection/establish.rs`, `DoHandshake::do_handshake` (from line
     55), where a handle is set, three recordings: the login, on entry, because
     sqlx-core calls into `with_socket` only once the TCP connect has completed
     (`sqlx-core/src/net/socket/mod.rs:190–193`); the TLS handshake, immediately
     before the call to `tls::maybe_upgrade` (line 100); and the login again,
     immediately after that call returns. `maybe_upgrade` awaits nothing where
     it negotiates no TLS (`tls.rs:23–53`), so the second recording is observed
     outside a handshake only on the refusal of `tls.rs:46–53`, which is a
     failure of the TLS handshake;
   - in `src/lib.rs`, the existing export of the options (line 45) also names
     the handle and the enumeration.

   `tpl` makes the handle and passes a clone to the options through the
   setter. It reads its own clone at two moments, for two requirements:

   - **once the call to `connect` has ended** in an expired budget or an
     error, for `FR-CONF-005`: that reading alone names the phase the deadline
     of `FR-ERR-027` and the classification of the failure report;
   - **after every poll of the call**, and once more when it ends, for
     `FR-GLOB-017`: the time between two readings is charged to the phase the
     first one saw, and each phase seen has its line once the call has ended
     (`src/mariadb/connect.rs`, `open_classified` and `Ledger`). The
     recordings are made synchronously inside a poll, so a phase is seen at the
     end of the poll that recorded it, and a phase recorded and left within one
     poll is not seen.

   Every read is an atomic load: it needs no `unsafe`, reads no driver text,
   and never writes the handle. sqlx-core is not touched, so the tree of
   `ADR-010` keeps its one statement.

   Nothing differs from the published crate but what points 3, 5, 6 and 8 name —
   no other function, file, feature or manifest entry; any other difference is
   a defect in the tree.

7. **The conditions that retire the patch, one per part.** The refusal of point
   3 goes at the first `sqlx` release whose `sqlx-mysql` lets a caller refuse
   the request; the presentation of point 5, at the first whose `sqlx-mysql`
   sends, for an empty or an absent password, at the first response and at
   every switch, the responses point 5 sends; and the handle of point 6, at the
   first whose `sqlx-mysql` lets a caller learn the phase a connection reached
   and observe each change of phase while the call runs, since `tpl` reads both
   (point 6);
   and the hardening of point 8, at the first whose `sqlx-mysql` contains
   upstream commit `4161768c9c077ea5dff6debcadeb0382d9f027df`, pull request
   `#4458`. At each, that part leaves the tree, `tpl` uses what the release offers, and
   its point is amended in place; the tree and its patch entry go with the last
   part. None of the first three was met on 2026-11-04, and the fourth was not
   met on 2026-11-05: `#4458` is on `main` and in no release. Until then the patch is carried across
   every move of `ADR-003`'s pin: the tree is vendored again at the release
   moved to, and every remaining part is re-applied and checked against that
   release's published source. `main` already edits both functions points 3
   and 5 touch, so the re-application will not be mechanical.

8. **The hardening against short and malformed packets, in full: upstream pull
   request `#4458`, carried whole.** Commit
   `4161768c9c077ea5dff6debcadeb0382d9f027df` of `transact-rs/sqlx`, merged on
   `main` on 2026-10-16, "fix(mysql): harden protocol decoders against
   malformed packets from hostile servers". Its 19 hunks are applied to the
   vendored tree verbatim, context, error texts and unit tests included, and
   nothing of the commit is left out or added to. They touch nine files under
   `src/`:

   - `connection/auth.rs`: in `AuthPlugin::handle`, a packet beginning with
     `0x01` shorter than 2 bytes is refused before `packet[1]` is read
     (1 hunk);
   - `connection/executor.rs`: a result-set column count is bounded before it
     sizes an allocation (1 hunk);
   - `connection/stream.rs`: `MAX_PACKET_PAYLOAD_LEN`, 1 GiB, and its check in
     the loop of `recv_packet` that joins continuation packets (2 hunks);
   - `io/buf.rs`: length-encoded reads bounded by the bytes remaining
     (2 hunks);
   - `protocol/connect/handshake.rs`: every fixed-width read of the greeting,
     and the length of the second part of the scramble, checked against the
     bytes remaining, an empty packet first (4 hunks);
   - `protocol/response/eof.rs`: a short EOF packet (1 hunk);
   - `protocol/response/err.rs`: an error packet that ends before its error
     number or its SQL state (3 hunks);
   - `protocol/statement/row.rs`: the binary row — the null bitmap, the length
     byte of a temporal value, fixed-width and length-encoded values, and a
     `NULL`-typed column that reached an `unreachable!()` (3 hunks);
   - `protocol/text/row.rs`: a text row whose field count does not match its
     columns (2 hunks).

   Every check returns `Error::Protocol`, the remedy class of
   `protocol/response/ok.rs:30–35`, which the commit extends to the rest of the
   protocol. In the login, `src/mariadb/fault.rs` maps it to the second outcome
   of `FR-ERR-047`, exit `69` (line 178), where an `Error::Io` would map to the
   first (line 175).

   The commit applies to the tree as it stands after points 3, 5 and 6 with no
   rejected hunk and no fuzz: `auth.rs` at an offset of 21 lines, because of
   point 5's lines above it; `stream.rs` and `statement/row.rs` at offsets of
   3 and 1 lines; every other hunk at its own line. The guards of point 3 and
   the first return of point 5 are untouched by it, and it touches none of the
   files of point 6.

   **The continuation-packet limit enters with the commit**, as one of its
   hunks, and is not a choice of this record. Whether an unbounded join of
   continuation packets is reachable is tested and recorded on rmp `#342`
   either way; the result moves nothing in this point.

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
  disables `mysql_old_password` alone (`src/opts/mod.rs:1095–1100`). It would
  also be a second driver stack for a protocol the first already speaks. These
  facts were read at the published crate and not tested against servers:
  enough to disqualify, which is all they are used for here, and not enough to
  qualify, per `ADR-003`.

- **A fork of `sqlx` pinned by revision.** Refused on the grounds `ADR-010`
  records for the same choice. They weigh more here: this divergence has no
  upstream end, so the second repository would be kept for as long as upstream
  offers no switch.

- **For a `mariadb` entry, exit `78` as for a `mysql` entry, or the
  classification the same login had before, with its message.** Both refused by
  the user on 2026-11-04; the second was this record's prescription until the
  seventy-seventh edition. `FR-ERR-047` and the note on `FR-SEC-021` record the
  grounds, and they are not restated here.

- **For an empty password, upstream's change on `main`** (`7908fe79`): an empty
  response wherever the password is empty, for `caching_sha2_password` and
  `mysql_native_password` alike (`auth.rs:20–25` there). Refused: it sends an
  empty response where point 5 sends `0x00` for `caching_sha2_password`, and it
  leaves `sha256_password` and the first response of an entry without a
  password as published. Taking it would have given point 5 an upstream end.

- **For an empty password, an edit of the response to a switch as well.**
  Unneeded: that response already presents an absent password as the empty
  string (`establish.rs:139`), and the change to `scramble` reaches it.

- **For the phase reached, any solution without a patch.** The driver offers no
  option, hook or event that marks a phase, and its login cannot be run on a
  socket `tpl` opens (*Context*). A TCP connection `tpl` opens first, to learn
  whether the TCP connect succeeds, is a second connection, which `NFR-PERF-004`
  forbids and `FR-MYSQL-042` holds a `mysql` entry to. Inferring the phase from
  the error refers it to the call, which `FR-CONF-005` forbids: on an expired
  budget there is no error, and on a failure the kind does not name the phase.
  Inferring it from the time elapsed is not what the connection had completed.
  Reading the socket's state from the operating system depends on the target
  and reads state the driver owns.

- **For the phase reached, the deadline moved into the driver**, which would
  apply the budget at each step and return an error naming the step. Refused:
  it reaches every wait of the login and moves the runtime's timers into the
  vendored crate, a wider divergence than three recordings, for a fact the
  recordings already give.

- **For the phase reached, a patch of `sqlx-core`**, marking the end of the TCP
  connect inside `connect_tcp`. Unneeded: the call into `sqlx-mysql`'s
  `with_socket` is that mark, and the tree of `sqlx-core` is `ADR-010`'s, whose
  divergence is one statement by its own rule.

- **For short packets, only the hunks of `#4458` at the sites confirmed on rmp
  `#342`** — the greeting, the error packet and the one-byte `0x01` packet.
  Refused by the user on 2026-11-05: a part of the commit is a divergence from
  the release that will carry all of it, so its retirement would have to be
  re-derived hunk by hunk, and it would leave the short EOF packet
  (`eof.rs:31–32`), the rows and the column count panicking, which the commit
  already closes.

- **For short packets, a local minimal patch**: after the server version,
  refuse fewer than the 31 bytes every branch of the greeting consumes; before
  the second part of the scramble, fewer than its length plus one; and the
  checks of the error packet and of `handle`, each written for this tree.
  Refused by the user on 2026-11-05 on the ground `ADR-010` gives for the same
  choice: a fix upstream has written disappears at the release that carries
  it, and a fix of this project's own has to be carried, re-applied and
  re-justified at every move of `ADR-003`'s pin. It also closes fewer
  panics.

- **For the phase reached, a thread-local or static cell** in place of a handle
  on the options. Refused: a global the connection does not own, correct only
  while the runtime runs the connection on the thread that reads it.

## Consequences

**A `mariadb` entry runs through every part of the patch.** MariaDB offers a
server-side `caching_sha2_password` on releases of three of the four series of
`FR-SRV-015`, not installed by default, as the note on `FR-SEC-021` records from
MariaDB's documentation; MariaDB Server does not support `sha256_password`.
Where the login of a `mariadb` entry needs the server's RSA public key on a
session TLS does not protect, the published crate as built requested the key
and then failed; the patched crate fails it without the request, and the
`mariadb` path recognises the refusal by its type and reports it as the third
outcome of `FR-ERR-047`, at the code the same login had before. No login that
succeeded before the patch fails, and none that failed succeeds. Points 5 and 6
reach a `mariadb` entry as they reach a `mysql` entry, because `FR-CONF-055` and
`FR-CONF-005` hold for both.

**For `caching_sha2_password`, point 5 sends what MySQL's client library
sends.** `FR-CONF-055` records the two reference clients: MySQL's sends a single
`0x00` for that method and MariaDB Connector/C an empty packet. On a MariaDB
server that offers the method, a `mariadb` entry with an empty or absent
password therefore sends `0x00` where MariaDB's own client sends nothing. No
observation records what either server answers to the other form.

**The handle answers both questions `FR-CONF-005` asks.** After an expired
budget it names the phase that was in progress, which `FR-ERR-027` reports.
After a failure it says whether the TCP connect had completed, which separates a
failure of the TCP connect, `FR-ERR-049`, from a failure of the login,
`FR-ERR-047`, where the kind of an I/O error cannot: `TimedOut` and an error the
operating system reports without a name of its own can arise on either side.
One bound remains. The driver hands the host name, not an address, to
`TcpStream::connect` (`sqlx-core/src/net/socket/mod.rs:192` at the tag), which
resolves it again after `tpl`'s own resolution has completed, so a budget that
expires during that second resolution is reported against the TCP connect.

**Three parts of the patch have no upstream end in sight; point 8 has one.** On
`main` at `7908fe79` the options still offer no refusal of the key request and
no record of the phase, and the change to the empty password sends what point 5
does not (*Alternatives rejected*). Point 8 is a commit already on `main`, and
it leaves the tree at the first release that contains it.

**Reading the handle after every poll changes nothing the server sees, and
the lines it yields depend on how the call yields.** The reads are in-process
loads of a cell the driver writes; no packet and no statement follows from
them, so the read-only promise and the count of `NFR-PERF-008` are untouched,
and the classification still rests on the one reading at the end. The timing
lines are diagnostics, outside the output contract under `NFR-DET-001`, and
their durations vary from run to run. Which lines appear also rests on the
call's polls: a TLS handshake is seen only if the call yields while the handle
names it, which it does whenever the handshake waits on the network. Where no
TLS is negotiated, the driver records the handshake and leaves it within one
poll, and no line is written, as `FR-GLOB-017` requires.

**Point 8 reaches every read, not the login alone.** Its checks in the rows,
the column count, the EOF packet and the join of continuation packets run on
every statement of both providers. A packet they refuse after the login ends
the statement with `Error::Protocol`, where the published crate panicked.
Which outcome that error is after the login is the specification's, and is not
restated here.

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
through this driver, each part of the patch and its scope, and the condition
that retires each. What either provider reports is the specification's, and is
cited here, not restated.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The constraints on the driver, and the corpus's statement that the choice is an architecture decision | `specification/mysql-contract.md`, seventy-fifth edition at `d4eaf76`: *Scope*, `FR-MYSQL-020`, `FR-MYSQL-021`, `FR-MYSQL-022`, `FR-MYSQL-045`, row 25 of `FR-MYSQL-040`; `specification/configuration-model.md`, `FR-CONF-036` | 2026-11-04 |
| The rule applied | The session coordinator's brief for rmp `#327` | 2026-11-04 |
| The user's decisions for rmp `#327`: the rule and its branch; the two guards, the typed refusal and its export; exit `78` for a `mysql` entry; for a `mariadb` entry, exit `69` with the message the same login had before, which the next row replaces, and `78` refused; the reader in `mysql/`, not inside `mariadb/` | The user's decision of 2026-11-04, relayed by the session coordinator for rmp `#327` | 2026-11-04 |
| The user's decisions for rmp `#332` to `#338`: for a `mariadb` entry, the refusal is the third outcome of `FR-ERR-047`, exit `69`, with the facts of `FR-MYSQL-045`; the patch presents an empty or absent password as `FR-CONF-055` requires, with an empty response for `mysql_native_password` and a single `0x00` for `caching_sha2_password` and `sha256_password`, at the first response and at every switch; the patch reports the phase reached only if no solution without a patch exists; rmp `#337` and `#338` classify by what the published API exposes | The user's decisions of 2026-11-04, relayed by the session coordinator for the joined effort | 2026-11-04 |
| The empty password, the phase reached and the outcomes the record cites | `specification/configuration-model.md`, `FR-CONF-005` and `FR-CONF-055`; `specification/mysql-contract.md`, `FR-MYSQL-047`; `specification/errors-and-exit-codes.md`, `FR-ERR-027`, `FR-ERR-047`, `FR-ERR-048`, `FR-ERR-049`; `specification/security.md`, the note on `FR-SEC-021`; the seventy-seventh edition, uncommitted in the working tree over `2d652bd` | 2026-11-04 |
| `scramble` computes a scramble from any password, the empty string included, and sends `sha256_password` to `encrypt_rsa`; the first response is computed only where the entry has a password; the response to a switch presents an absent password as the empty string | `sqlx-mysql/src/connection/auth.rs` lines 13–34, and `src/connection/establish.rs` lines 102–106 and 128–146, at `v0.9.0` | 2026-11-04 |
| An absent first response is encoded as empty data; the response to a switch is a packet of exactly the bytes given | `sqlx-mysql/src/protocol/connect/handshake_response.rs` lines 50–62, and `src/protocol/connect/auth_switch.rs` lines 58–66, at `v0.9.0` | 2026-11-04 |
| One call covers the connection: `connect` calls `establish`, which calls sqlx-core's `connect_tcp`; `connect_tcp` calls `with_socket` on the stream only once `TcpStream::connect` has returned, and converts its error with `?`; `Error::Io` carries the `io::Error` unchanged; sqlx-core exports `connect_tcp` and `WithSocket` | `sqlx-mysql/src/options/connect.rs` lines 21–25 and `src/connection/establish.rs` lines 15–21; `sqlx-core/src/net/socket/mod.rs` lines 184–194, `src/error.rs` line 49 and `src/net/mod.rs` lines 4–6; at `v0.9.0` | 2026-11-04 |
| The driver offers no hook on the path: the options hold no such field and derive `Debug` and `Clone` alone; the log settings reach the executor only; the path's only events are two `tracing::debug!` calls where no TLS is attempted; `establish` is crate-private, `DoHandshake` private, and the options take a Unix socket path and no stream | `sqlx-mysql/src/options/mod.rs` lines 62–83 and 138, `src/connection/executor.rs` line 110, `src/connection/tls.rs` lines 35 and 41, `src/connection/establish.rs` lines 15 and 37, at `v0.9.0` | 2026-11-04 |
| `maybe_upgrade` awaits nothing where it negotiates no TLS, and refuses with `Error::Tls` under the three modes that require TLS where the server offers none | `sqlx-mysql/src/connection/tls.rs` lines 17–54, at `v0.9.0` | 2026-11-04 |
| An error packet arrives as a `MySqlDatabaseError`, whose `number()` is public | `sqlx-mysql/src/connection/stream.rs` lines 141–170 and `src/error.rs` line 24, at `v0.9.0` | 2026-11-04 |
| On `main` at `7908fe79`: an empty response for `caching_sha2_password` and `mysql_native_password` where the password is empty (`auth.rs` lines 20–25 there); the first response of an entry without a password unchanged (`establish.rs` lines 104–108 there); no option for the phase | `git show 7908fe79:sqlx-mysql/src/connection/auth.rs` and `…/establish.rs`, and `git diff` from `v0.9.0` over `src/options/mod.rs` | 2026-11-04 |
| `TcpStream::connect` tries each address `addr` yields and returns the last attempt's error; `ToSocketAddrs` for string types requires a DNS lookup | docs.rs, `tokio` 1.53.1, `tokio::net::TcpStream::connect` and `tokio::net::ToSocketAddrs` | 2026-11-04 |
| The vendored tree differs from the published crate in exactly the lines of Decision point 3: in `src/connection/auth.rs`, the two guards, `PublicKeyRequestRefused` with `plugin()`, `Display` and `std::error::Error`, and the private function that builds the refusal; `pub use auth::PublicKeyRequestRefused;` in `src/connection/mod.rs`; `pub use connection::PublicKeyRequestRefused;` in `src/lib.rs`. Every other file, the manifests, `.cargo_vcs_info.json` and the licence stubs included, is identical. `auth` is declared private in `connection`, and `connection` private in the crate | `diff -r` between `vendor/sqlx-mysql-0.9.0/` in the working tree over `90881b1`, before commit, and the published `.crate`, sha256 `90b8020f…acebea27`; `src/connection/mod.rs:18` and `src/lib.rs:19` of the published crate | 2026-11-04 |
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
| The user's decision for rmp `#342`: carry the whole of upstream `#4458`, retiring at the first `sqlx` release containing it; the confirmed sites alone, and a local minimal patch, refused | The user's decision of 2026-11-05, relayed by the session coordinator | 2026-11-05 |
| The panics confirmed: a truncated greeting exits `70` in 34 of 44 runs; a short error packet and a one-byte `0x01` packet panic by code reading; the continuation join untested | rmp `#342` work log, `TEST` and `FINDING` entries of 2026-11-05 (comments 1683, 1684 and 1685) | 2026-11-05 |
| A truncated or malformed packet of the login is the second outcome, exit `69`, never `70`; an error packet that cannot be read is the same | `specification/errors-and-exit-codes.md`, `FR-ERR-047` and `FR-ERR-048`, eightieth edition at `a4f81c9` | 2026-11-05 |
| Commit `4161768c9c077ea5dff6debcadeb0382d9f027df`, parent `39eff06a`, merged to `main` on 2026-10-16 as pull request `#4458`, title as quoted; its 19 hunks over the nine files of point 8, with the per-file counts given there; `MAX_PACKET_PAYLOAD_LEN` is 1 GiB; the result-set column count is capped at 4096; every check returns `err_protocol!` | GitHub API, `transact-rs/sqlx`, `commits/4161768c9c` and `pulls/4458`; the commit's diff, sha256 `e14d5e11…930ec5`, 618 lines | 2026-11-05 |
| No release contains it: `v0.9.0` is the latest tag | GitHub API, `transact-rs/sqlx`, `tags` | 2026-11-05 |
| The diff applies to `vendor/sqlx-mysql-0.9.0/` as it stands with every hunk succeeding, no rejection and no fuzz, `auth.rs` at offset 21, `stream.rs` at −3 and `statement/row.rs` at −1 | `patch -p2 --verbose` over a scratch copy of the vendored tree at `a4f81c9`; not compiled | 2026-11-05 |
| `recv_packet` refuses an empty packet with `Error::Protocol`; `src/mariadb/fault.rs` maps an error of the login phase that is not `Io` to the second outcome and an `Io` error to the first | `vendor/sqlx-mysql-0.9.0/src/connection/stream.rs` lines 160–162; `src/mariadb/fault.rs` lines 175–178, at `a4f81c9` | 2026-11-05 |
| `tpl` reads the handle after every poll of the call to `connect` and once when it ends, charges the time between readings to the phase the first saw, and reports one line per phase seen; the classification and the deadline read the handle once the call has ended; nothing under `vendor/` changed for it | `src/mariadb/connect.rs`, `open_classified` (lines 544–644) and `Ledger` (lines 646–735), at `f7c5385`; `git diff a4f81c9 f7c5385 -- vendor/`, whose changes are point 8's and `vendor/README.md` | 2026-11-05 |
| At `INFO`, one line per phase of `FR-CONF-005` the connection ran, the TLS handshake where the mode negotiated one, written once the connection has opened or failed | `specification/global-flags.md`, `FR-GLOB-017`, as amended in the seventy-ninth edition | 2026-11-05 |
