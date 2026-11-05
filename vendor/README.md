# `vendor/`

Third-party source vendored into this repository. It is **not** this project's
code: the conventions, lints and prohibitions that govern `src/` do not reach
it, and `Cargo.toml` excludes it from the workspace so that the mandatory
validation pipeline never runs over it.

## `sqlx-core-0.9.0/`

| | |
|---|---|
| Crate | `sqlx-core` |
| Version | `0.9.0`, published 2026-05-21 |
| Taken from | `sqlx-core-0.9.0.crate`, the crates.io release artefact, sha256 `05b44e85bf579a8eeb4ceaa77a3a523baf2bf0e9bac7e40f405d537b5d2d5ccb` — the checksum this repository's `Cargo.lock` recorded for the registry source it replaces |
| Upstream revision | `003b698e99e024f3621b8043a2426fde5b741171`, per the crate's own `.cargo_vcs_info.json` |
| Upstream repository | `https://github.com/transact-rs/sqlx` (the `repository` field of the published manifest still names the former `launchbadge/sqlx` path) |
| Licence | `MIT OR Apache-2.0`, per the published manifest |
| Why it is here | `ADR-010` — the TLS connect stall in the pinned driver |

### The divergence, in full

One statement, in `src/net/socket/mod.rs`, inside the `_rt-tokio` branch of
`connect_tcp`:

```rust
socket.set_nodelay(true)?;
```

It reproduces the first hunk of upstream PR **#4336** (`transact-rs/sqlx`,
merged 2026-08-17) verbatim, context and all, including the rebinding of the
connected stream that the statement needs. Nothing else in this tree differs
from the published crate; any other difference is a defect in it.

The remaining hunks of that PR are **not** reproduced here. They set the same
option on the `_rt-async-io` paths, a feature this package does not enable, and
`ADR-010` prescribes one statement and quotes this one.

### Licence files

`LICENSE-APACHE` and `LICENSE-MIT` inside `sqlx-core-0.9.0/` are preserved byte
for byte as published. In the published artefact each is a 17- and a 14-byte
file whose whole content is the relative path `../LICENSE-APACHE` and
`../LICENSE-MIT`: upstream keeps them as symbolic links into its own workspace
root, and the packaged crate carries the link text rather than the licence
text. That is what "as published" is, and `ADR-010` forbids moving this source
onto other terms — so the stubs stay as they are. The terms are the ones the
manifest declares: `MIT OR Apache-2.0`.

Preserving them is not the whole of the obligation, and `ADR-010` says so: the
repository must also **carry the terms those files name**. Both offers require
it — Apache-2.0 §4(a) obliges a redistributor to give any other recipient of
the work a copy of the License, and the MIT permission notice obliges the
copyright notice and the permission notice to travel in all copies — and this
repository redistributes `sqlx-core` 0.9.0. The two texts therefore sit beside
the crate directory, in this one:

| File | Bytes | sha256 |
|---|---|---|
| `LICENSE-APACHE` | 10297 | `c8f5453612253e8ea8bad241617fe74c4a6d5d592ba64ff881668c211b50ac99` |
| `LICENSE-MIT` | 1109 | `5abbdd842310c8f1650fcde3a5a51a3e09412b9e5a6eea87519e0db9e32b339d` |

Both are taken from `transact-rs/sqlx` at revision
`003b698e99e024f3621b8043a2426fde5b741171` — the one the crate's own
`.cargo_vcs_info.json` names — and not from a licence index. `ADR-010`
requires that provenance for a reason: upstream's `LICENSE-MIT` opens on
`Copyright (C) SQLx Contributors` and `Portions of this work Copyright (C)
LaunchBadge, LLC`, and that is the notice the MIT permission notice requires to
travel. No generic copy of either licence supplies it.

The placement is load-bearing rather than tidy. `../LICENSE-APACHE` resolves,
from `sqlx-core-0.9.0/`, to exactly `vendor/LICENSE-APACHE`, and
`../LICENSE-MIT` to `vendor/LICENSE-MIT`: the stubs stop dangling, and this
directory reproduces upstream's own layout instead of diverging from it.
Adding the texts inside the crate directory would have been a second
divergence from the published source, in the very diff `ADR-010` keeps cheap
to run.

### When this directory goes

`ADR-010` retires on the first `sqlx` release whose `sqlx-core` carries PR
`#4336`. On that release this directory is deleted, along with the
`sqlx-core` entry of the `[patch.crates-io]` table and the `[workspace]`
exclusion that point at it. The table itself, and the licence texts beside
this directory, stay for as long as `sqlx-mysql-0.9.0/` stands; each vendored
tree goes on its own condition.

## `sqlx-mysql-0.9.0/`

| | |
|---|---|
| Crate | `sqlx-mysql` |
| Version | `0.9.0`, published 2026-05-21 |
| Taken from | `sqlx-mysql-0.9.0.crate`, the crates.io release artefact at `https://static.crates.io/crates/sqlx-mysql/sqlx-mysql-0.9.0.crate`, sha256 `90b8020fe17c5f2c245bfa2505d7ef59c5604839527c740266ad2214acebea27` — the checksum this repository's `Cargo.lock` recorded for the registry source it replaces, verified before the tree was copied |
| Upstream revision | `003b698e99e024f3621b8043a2426fde5b741171`, per the crate's own `.cargo_vcs_info.json` — the revision `sqlx-core-0.9.0/` was packaged from |
| Upstream repository | `https://github.com/transact-rs/sqlx` |
| Licence | `MIT OR Apache-2.0`, per the published manifest |
| Why it is here | `ADR-014` — four changes the published crate does not offer: it requests the server's RSA public key over a session TLS does not protect, which `FR-MYSQL-045` forbids (point 3); it sends a scramble of an empty password, and for an entry with no password an empty first response whatever the method, where `FR-CONF-055` requires the response the servers' own clients send (point 5); it does not let a caller learn the phase a connection reached, which `FR-CONF-005` decides by (point 6); and it panics on a short or malformed packet from the server, where `FR-ERR-047` and `FR-ERR-048` require the second outcome, exit `69` (point 8) |

### The divergence, in full

Thirteen files differ from the published crate, and nothing else: the five of
points 3, 5 and 6 — `src/connection/auth.rs`, `src/connection/establish.rs`,
`src/connection/mod.rs`, `src/options/mod.rs` and `src/lib.rs` — and the nine of
point 8, which share `src/connection/auth.rs` with them:
`src/connection/auth.rs`, `src/connection/executor.rs`,
`src/connection/stream.rs`, `src/io/buf.rs`,
`src/protocol/connect/handshake.rs`, `src/protocol/response/eof.rs`,
`src/protocol/response/err.rs`, `src/protocol/statement/row.rs` and
`src/protocol/text/row.rs`.

**Point 3 — the refusal of the key request.** Two guards and one typed error,
in `src/connection/auth.rs`:

- in `AuthPlugin::scramble`, the arm for `sha256_password` returns the refusal
  when the stream is not TLS, before it calls `encrypt_rsa`;
- in `AuthPlugin::handle`, the arm for `perform_full_authentication` (`0x04`)
  does the same;
- the refusal is `Error::Configuration`, boxing `PublicKeyRequestRefused`, a
  type declared beside the two guards for this refusal alone, which carries the
  name `AuthPlugin::name` gives the method.

The type is exported from the crate root, `src/lib.rs`, beside the crate's
other exports. Rust's privacy rules require one more line for that export to
compile: `auth` is a private module of `connection`, so `src/connection/mod.rs`
re-exports the type with `pub use auth::PublicKeyRequestRefused;`, which the
root then names.

`encrypt_rsa` is not edited. The two arms are its only callers, so nothing
writes a public-key request on a stream that is not TLS, and on a TLS stream
every path runs as published.

**Point 5 — the empty password.** Two changes:

- in `src/connection/auth.rs`, `AuthPlugin::scramble` returns first, where the
  password is the empty string, before it computes, guards or requests
  anything: an empty response for `mysql_native_password`, and a single `0x00`
  byte, as MySQL's client library sends, for `caching_sha2_password` and
  `sha256_password` — ahead of point 3's guard, because a login with no
  password needs no key. The arm for `mysql_clear_password` already sends a
  single `0x00` for the empty string and is unchanged;
- in `src/connection/establish.rs`, `DoHandshake::do_handshake`, the first
  response of the login is computed, where the server names a method, from the
  entry's password or, where the entry supplies none, from the empty string —
  `options.password.as_deref().unwrap_or_default()`, the form the response to
  a switch already uses. Where the server names no method the response stays
  absent, as published.

The response to a switch and `AuthPlugin::handle` are not edited: the first
already presents an absent password as the empty string, so the change to
`scramble` reaches it.

**Point 6 — the phase the connection reached.** Three changes:

- in `src/options/mod.rs`: `ConnectPhaseHandle`, a public `Clone` and `Debug`
  handle on a shared atomic cell, whose one reader, `phase`, returns
  `ConnectPhase` — `TcpConnect`, which a new handle holds, `TlsHandshake` or
  `Login`; one field of `MySqlConnectOptions` holding an optional handle,
  absent in `new()`; and one setter, `connect_phase`, that sets it;
- in `src/connection/establish.rs`, `DoHandshake::do_handshake`, where a
  handle is set: the login is recorded on entry, because `sqlx-core` calls into
  `with_socket` only once the TCP connect has completed; the TLS handshake
  immediately before `tls::maybe_upgrade`; and the login again immediately
  after it returns;
- in `src/lib.rs`, the existing export of the options also names
  `ConnectPhase` and `ConnectPhaseHandle`.

**Point 8 — the hardening against short and malformed packets.** Upstream
commit `4161768c9c077ea5dff6debcadeb0382d9f027df` of `transact-rs/sqlx`,
pull request **#4458**, merged on `main` on 2026-10-16, "fix(mysql): harden
protocol decoders against malformed packets from hostile servers", carried
whole: its 19 hunks are applied verbatim — context, error texts and unit tests
included — to the nine files named above, and nothing of the commit is left
out or added to. Its diff, as the GitHub API serves it for the commit, is 618
lines with sha256
`e14d5e1169de3a4939c11481b7f83f1f3c04cd5fef8019e8492427367e930ec5`. It was
applied with `patch -p2` over the tree as points 3, 5 and 6 leave it, every hunk
succeeding with no rejection and no fuzz: `auth.rs` at an offset of 21 lines,
`stream.rs` at −3 and `statement/row.rs` at −1, every other hunk at its own
line. The lines it adds and removes are exactly the commit's: the sorted `+`
and `-` lines of `git diff` over the nine files equal those of the upstream
diff. Every check it adds returns `Error::Protocol`; among them is
`MAX_PACKET_PAYLOAD_LEN`, 1 GiB, which bounds the join of continuation packets
in `recv_packet`. The guards of point 3 and the first return of point 5 are
untouched by it, and it touches none of the files of point 6.

Nothing else in this tree differs from the published crate; any other
difference is a defect in it. The check is a recursive `diff` of this
directory against the published artefact, unpacked after its sha256 is
verified, which names exactly the thirteen files above.

### Licence files

`LICENSE-APACHE` and `LICENSE-MIT` inside `sqlx-mysql-0.9.0/` are the same
17- and 14-byte stubs `sqlx-core-0.9.0/` carries, preserved byte for byte, and
they resolve to the same two texts in this directory. The crate was packaged
from the same upstream revision, so those texts carry its notices too.

### When this directory goes

`ADR-014` point 7 retires each part on its own condition:

- point 3, at the first `sqlx` release whose `sqlx-mysql` lets a caller refuse
  the request;
- point 5, at the first whose `sqlx-mysql` sends, for an empty or an absent
  password, at the first response and at every switch, the responses point 5
  sends;
- point 6, at the first whose `sqlx-mysql` lets a caller learn the phase a
  connection reached;
- point 8, at the first whose `sqlx-mysql` contains upstream commit
  `4161768c9c077ea5dff6debcadeb0382d9f027df`, pull request `#4458`.

At each, that part leaves this tree and `tpl` uses what the release offers.
The directory is deleted with the last part, along with the `sqlx-mysql` entry
of the `[patch.crates-io]` table and its `[workspace]` exclusion. Until then,
every move of `ADR-003`'s pin vendors this crate again at the release moved to
and re-applies every remaining part.
