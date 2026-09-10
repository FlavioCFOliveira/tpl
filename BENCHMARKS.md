# Benchmarks

This file is the register of measured baselines for `tpl`. A figure recorded
here has been observed on a named target with a stated protocol; a figure that
has not been observed does not belong here.

Two rules govern every entry, and neither is negotiable:

1. **Every baseline names the target it was measured on.** A number without a
   target is not a baseline.
2. **Figures from different targets are never compared with each other.** The
   tables below are read down a column, never across two targets.

The properties `tpl` is required to have, the reference workloads, and the
normative measurement protocol live in
[`specification/performance-requirements.md`](specification/performance-requirements.md).
This file carries only what was actually measured. Where an entry departs from
the normative protocol, it says so.

## How to read an entry

Each entry states, in this order: what was decided or established; the workload;
the candidates; the environment down to the toolchain; the protocol; the noise
floor of the instrument on that host; the results; the confounders that make the
raw tables misleading if read naively; and the commands needed to reproduce the
whole thing. An entry is written to be reproducible from itself alone.

---

## 2026-09-10 — MariaDB driver selection

*Sprint 2, task `#7`. Target of record: `aarch64-apple-darwin` and
`aarch64-unknown-linux-musl`. Server of record: MariaDB `11.4`.*

### Outcome

**`sqlx` 0.9.0 with `tokio` 1.53.1 on a current-thread runtime.** `mysql`
28.0.2 is rejected.

This closes the last architecture decision the project carried as open. The
spike that produced these figures was deleted when the task closed; this entry
is the whole surviving record of it.

### What decided it was a rule, not the numbers

`FR-CONF-036`, in
[`specification/configuration-model.md`](specification/configuration-model.md),
disqualifies a driver that cannot express all five TLS modes of `FR-CONF-013`
distinctly, and forbids settling the choice by reducing the mode set to fit a
driver. `mysql` 28.0.2 fails that requirement on two independent counts, both
established empirically against running servers rather than read off the API:

- **`preferred` is not expressible.** The driver's entire TLS surface is
  `Option<SslOpts>`, which affords two postures and no third: never negotiate,
  or require and fail. No configuration connects in plaintext to a TLS-less
  `10.11` while also encrypting against `11.4` — which is precisely what
  `preferred` means.
- **`verify-ca` collapses onto `verify-identity`,** returning a byte-identical
  error. The cause is a dead `match` arm: the verifier patterns on
  `CertificateError::NotValidForName`, while rustls 0.23.44 emits the struct
  variant `NotValidForNameContext`. The two compare equal under `PartialEq`, but
  a pattern is not a `PartialEq` call, so the arm never matches and
  `with_danger_skip_domain_validation(true)` is inert. It fails **silently**:
  the code compiles, the option exists, and the caller is handed a different
  guarantee from the one asked for.

There is no third candidate to weigh. `mysqlclient-sys` is an FFI binding,
`diesel` is an ORM over that same C library, and the crates `mariadb`,
`libmariadb-sys` and `mariadb-connector-c` do not exist on crates.io.

The numbers below agreed with the rule rather than opposing it, which is why the
decision was not a trade-off. They are recorded in full because the suspicion
they refuted — that an async runtime penalises startup, binary size and resident
memory in an ephemeral process — was written into the project's own stack notes
and deserves an evidence trail.

### Workload

Every figure was taken against `tpl-mariadb:11.4`
(`11.4.13-MariaDB-ubu2404`), `freight` schema, running one `INFORMATION_SCHEMA`
query that returns **301 rows / 14 499 bytes**. All five driver variants
produced byte-identical output.

### Candidates

| label | crate | notes |
|---|---|---|
| `nop` | — | empty binary carrying the identical release profile; the process-spawn floor |
| sync — as shipped | `mysql` 28.0.2 | default options, `ssl_opts = None` |
| sync — `prefer_socket(false)` | `mysql` 28.0.2 | same binary, socket probe disabled |
| sync — `prefer_socket(false)` + TLS | `mysql` 28.0.2 | same binary, TLS required |
| async — as shipped (TLS) | `sqlx` 0.9.0 + `tokio` 1.53.1 | default options, `ssl_mode = Preferred` |
| async — TLS disabled | `sqlx` 0.9.0 + `tokio` 1.53.1 | same binary, TLS off |

The three sync rows share one binary and the two async rows share one binary;
the variants are runtime flags, not separate builds. That is why the stripped
size repeats down each group in the tables below.

### Environment

**Host.** Apple M4, 10 cores, 32 GiB, macOS 26.6.2 (25G83). Load average
1.88–2.35 throughout — **the host was not idle**: Solr, ZooKeeper, MongoDB and
mongo-express containers were up, plus the Docker Desktop VM at 6.2 GB RSS. The
host could not be quieted, and the A/A twin described below exists precisely to
make that admissible rather than to pretend otherwise.

**Toolchain.** rustc/cargo 1.98.0, edition 2024; `cargo-zigbuild` 0.23.4 with
zig 0.16.0 for the musl target (the Apple linker cannot emit ELF); `hyperfine`
1.20.0; Docker 29.7.2; `alpine:latest` arm64; `tpl-mariadb:11.4`.

**Crates.** Sync: `mysql` 28.0.2 with `mysql_common` 0.37.3 — 135 crates in the
dependency graph. Async: `sqlx` 0.9.0 with `tokio` 1.53.1 — 134 crates. Both
link the same TLS stack: `rustls` 0.23.44, `ring` 0.17.14, `webpki-roots`
1.0.9.

**Release profile, identical in all four crates.** `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`.

### Protocol

8 rounds x 50 runs per command — **400 pooled samples per command** — with 5
warmups per round, driven by `hyperfine --shell=none`. **The command order was
rotated one position per round**, and every comparison between two commands was
made **paired per round**, so that host drift over the session cancels rather
than accumulating in whichever command happened to run first.

The entire darwin run was executed twice end to end, and the two replications
agree on every sign reported here.

### Noise floor and resolution

Recorded before any figure, because it is what makes the figures readable.

An **A/A twin** was run throughout: the same binary, executed twice per round
under two different labels, so that every difference measured between the twins
is noise by construction. It resolved to:

| target | A/A twin difference | rounds with a consistent sign |
|---|---|---|
| `aarch64-apple-darwin` | 0.039 ± 0.122 ms | 5 of 8 — a coin flip |
| `aarch64-unknown-linux-musl` | 0.002 ± 0.015 ms | 4 of 8 |

A `nop` crate carrying the identical release profile gives the process-spawn
floor: **1.339 ± 0.064 ms** on darwin, **0.104 ± 0.027 ms** on musl. Nothing
below that floor is attributable to a driver.

Read together: on darwin, a difference smaller than roughly 0.2 ms is not a
difference. On musl the instrument is an order of magnitude sharper.

### Results — `aarch64-apple-darwin`, measured natively

| candidate | wall mean ± sd | min | p50 | peak RSS | stripped binary |
|---|---|---|---|---|---|
| nop (floor) | 1.339 ± 0.064 ms | 1.194 | 1.329 | 1.41 MiB | 302 464 B |
| sync — as shipped | 4.743 ± 0.288 ms | 4.284 | 4.704 | 3.73 MiB | 2 720 416 B |
| sync — `prefer_socket(false)` | 4.462 ± 0.249 ms | 4.119 | 4.432 | 3.73 MiB | 2 720 416 B |
| sync — `prefer_socket(false)` + TLS | 7.557 ± 0.237 ms | 7.146 | 7.514 | 4.67 MiB | 2 720 416 B |
| async — as shipped (TLS) | 7.433 ± 0.220 ms | 6.624 | 7.402 | 2.81 MiB | 1 513 712 B |
| async — TLS disabled | 4.281 ± 0.182 ms | 3.614 | 4.278 | 2.20 MiB | 1 513 712 B |

### Results — `aarch64-unknown-linux-musl`, every figure taken inside a Docker container

The musl binaries were never executed on bare metal. Each figure below was taken
by a runner container sharing the server container's network namespace, which is
raw loopback — a different network path from the darwin figures above. The two
targets are not comparable with each other, for this reason on top of the
standing rule.

| candidate (in container) | wall mean ± sd | min | p50 | peak RSS | stripped binary |
|---|---|---|---|---|---|
| nop (floor) | 0.104 ± 0.027 ms | 0.066 | 0.097 | 0.75 MiB | 312 296 B |
| sync — as shipped | 1.766 ± 0.120 ms | 1.616 | 1.747 | 2.38 MiB | 2 899 848 B |
| sync — `prefer_socket(false)` | 1.686 ± 0.055 ms | 1.558 | 1.681 | 2.36 MiB | 2 899 848 B |
| sync — `prefer_socket(false)` + TLS | 4.498 ± 0.145 ms | 4.303 | 4.468 | 3.12 MiB | 2 899 848 B |
| async — as shipped (TLS) | 61.937 ± 4.105 ms | 46.045 | 62.393 | 1.75 MiB | 1 652 816 B |
| async — TLS disabled | 1.236 ± 0.075 ms | 1.069 | 1.223 | 1.12 MiB | 1 652 816 B |

The 61.9 ms figure is real, reproducible, and **not understood**. See
[The musl anomaly](#the-musl-anomaly) below before drawing anything from it.

### Peak RSS method

Peak resident set size is `getrusage(ru_maxrss)`, median of 12 invocations, on
both targets:

- **macOS** — `/usr/bin/time -l`, which reports bytes.
- **Linux** — busybox `time -v` inside the container, which reports kbytes.

Values were near-constant across the 12 invocations on every candidate.

### The TLS confounder — read this before the "as shipped" rows

Both drivers link the same TLS stack, but they do not *negotiate* the same way:
`mysql` defaults to `ssl_opts = None`, while `sqlx` defaults to
`ssl_mode = Preferred`. The "as shipped" rows therefore do not measure the same
work.

This was proved server-side, not inferred: diffing the server's `Ssl_accepts`
counter across an invocation, against a zero-delta control, gives
`tls_handshakes = 0` for the sync driver and `tls_handshakes = 1` for the async
one.

**Timed as shipped, "async is 57% slower on darwin" is entirely a TLS handshake,
not tokio.** The handshake itself costs 3.10 ms on the sync driver and 3.15 ms
on the async one — measured independently on two drivers, agreeing to within
50 µs.

### Like for like, TLS equalised, paired per round

| comparison | difference | rounds with a consistent sign |
|---|---|---|
| darwin, plaintext | async faster by 0.181 ± 0.085 ms | 8 of 8 |
| darwin, TLS on both | async faster by 0.124 ± 0.037 ms | 8 of 8 |
| musl, plaintext | async faster by 0.450 ± 0.026 ms — 27% | 8 of 8 |

The darwin advantage is real in sign, but its magnitude moved between the two
replications (0.08 ms → 0.18 ms). **The honest statement for darwin is that the
two drivers are within about 0.2 ms of each other and too close to choose on.**
The musl advantage is large relative to that target's noise floor and stable.

The size and memory columns point the same way and are not close: on both
targets the async binary is roughly 1.2 MB smaller stripped, and its peak RSS is
lower in every equalised comparison.

### `prefer_socket` isolated

`mysql` probes for a Unix socket by default, issuing an extra `SELECT @@socket`.
This was confirmed first at the protocol level — the statement appears and
disappears from the server's statement log as the option is toggled — and only
then timed.

| target | cost of the probe | rounds with a consistent sign |
|---|---|---|
| darwin | 0.281 ± 0.057 ms — 6.3% of the invocation | 8 of 8 |
| musl | 0.079 ± 0.038 ms | 8 of 8 |

### The musl anomaly

**Recorded, not buried.** `sqlx` with TLS over direct Linux loopback takes
61.937 ms, of which `connect_with` alone accounts for **44.18 ms**. The query is
~1.1 ms in every variant, so the cost is entirely in connection establishment.

During those 44 ms the process is at **0% CPU with 9 voluntary context
switches** — it is blocked, not computing.

Nagle's algorithm was the obvious suspect, the ~40 ms magnitude matching a
delayed ACK almost exactly. It was **tested and refuted**: disabling it on the
sync driver (`tcp_nodelay(false)`) gave 3.85 ms against 3.83 ms — unchanged.

**The mechanism is unexplained.** It is tracked as roadmap task `#8`. The
anomaly is absent on the macOS host, where the same path crosses Docker
Desktop's port proxy instead of raw loopback.

Nothing in the decision rests on this row: the async candidate wins the musl
comparison on the plaintext pair, and the anomaly is a defect to be understood
before TLS-over-loopback is relied upon, not a property of the driver choice.

### Caveats

- **Two instruments were used.** `hyperfine` and a custom runner (which produced
  the phase breakdown, the handshake attribution and the RSS figures) are
  different instruments. They were validated to agree within 0.05 ms on darwin,
  which is inside that target's noise floor.
- **The two targets took different network paths.** The darwin path crosses
  Docker Desktop's port proxy; the musl path is raw loopback inside a shared
  network namespace. This is a further reason, on top of the standing rule,
  never to compare the two tables with each other.
- **The host was not idle.** The A/A twin, the order rotation, the paired
  comparisons and the full second replication are what make the figures usable
  in spite of it.

### Standing relative to the normative protocol

This entry is a **driver-selection record**, not a ratified budget under
`NFR-PERF-020`, and no figure in it supersedes a provisional budget in
[`specification/performance-requirements.md`](specification/performance-requirements.md).
It departs from that file's measurement protocol in three ways, stated here so
that the next entry is not modelled on it by accident: 5 warmups per round
rather than the 20 of `NFR-PERF-009`; a host that was not idle, against
`NFR-PERF-010`; and a relative standard deviation above the 5% of
`NFR-PERF-011` on several rows. It also reaches a server, which
`NFR-PERF-013` keeps every normative budget away from.

### Reproduction

```sh
# Server: MariaDB 11.4, freight schema, on a non-default port.
docker run -d --name tpl-bench-11.4 -e MARIADB_ROOT_PASSWORD=tpl-root \
  -p 127.0.0.1:13307:13307 tpl-mariadb:11.4 --port=13307

# Binaries: native for darwin, zig-linked for musl.
cargo build    --release --target aarch64-apple-darwin
cargo zigbuild --release --target aarch64-unknown-linux-musl

# One round. Repeat 8 times, rotating the -n/command pairs one position
# per round. sync-nops and sync-nops-AA are the same binary with the same
# flags: they are the A/A twin, and the difference between them is the
# noise floor.
hyperfine --shell=none --warmup 5 --runs 50 --style none --export-json <out> \
  -n nop "<nop>" -n sync-shipped "<sync>" \
  -n sync-nops "<sync> --no-prefer-socket" -n sync-nops-AA "<sync> --no-prefer-socket" \
  -n sync-nops-tls "<sync> --no-prefer-socket --tls" \
  -n async-shipped "<async>" -n async-notls "<async> --no-tls"

# musl figures are taken from a runner sharing the server's network namespace.
docker run -d --name tpl-bench-runner --network container:tpl-bench-11.4 \
  alpine:latest sleep infinity
docker cp <binary> tpl-bench-runner:/b/

# Peak RSS: getrusage(ru_maxrss), median of 12.
/usr/bin/time -l <cmd>                               # macOS, bytes
docker exec tpl-bench-runner /usr/bin/time -v <cmd>  # Linux, kbytes
```

The TLS confounder is confirmed by diffing `Ssl_accepts` on the server across an
invocation, with a control invocation known to add zero. The `prefer_socket`
probe is confirmed by toggling the option and watching `SELECT @@socket` appear
in and disappear from the server's statement log.
