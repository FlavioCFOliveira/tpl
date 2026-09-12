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

`LICENSE-APACHE` and `LICENSE-MIT` are preserved byte for byte as published. In
the published artefact each is a 17- and a 14-byte file whose whole content is
the relative path `../LICENSE-APACHE` and `../LICENSE-MIT`: upstream keeps them
as symbolic links into its own workspace root, and the packaged crate carries
the link text rather than the licence text. That is what "as published" is, and
`ADR-010` forbids moving this source onto other terms — so the stubs stay as
they are. The terms are the ones the manifest declares: `MIT OR Apache-2.0`.

### When this directory goes

`ADR-010` retires on the first `sqlx` release whose `sqlx-core` carries PR
`#4336`. On that release this directory is deleted, along with the
`[patch.crates-io]` table and the `[workspace]` exclusion that point at it.
