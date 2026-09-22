# Benchmarks

This file is the **register of observations** for `tpl`. A figure recorded here
has been observed on a named target under a stated protocol; a figure that has
not been observed does not belong here.

**No figure in this file fails anything.** `BR-PERF-008`, in
[`specification/performance-requirements.md`](specification/performance-requirements.md),
states that no figure named in that corpus and no figure recorded against it
here fails, blocks, rejects or gates a change, a release, or a piece of work.
This register is informative: it is consulted on demand by a reader who wants to
know what the program cost on a named machine on a named day, and never
consulted to decide that work is done.

Two rules govern every entry, and neither is negotiable:

1. **Every recorded figure names the target it was measured on.** A number
   without a target cannot be read.
2. **Figures from different targets are never compared with each other.** The
   tables below are read down a column, never across two targets.

The properties `tpl` is required to have, the reference workloads, the nine
measurement points and the measurement protocol live in
[`specification/performance-requirements.md`](specification/performance-requirements.md).
This file carries only what was actually measured. Where an entry departs from
that file's protocol, it says so, which is what `NFR-PERF-020` obliges.

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

The 61.9 ms figure is real and reproducible, and its mechanism was unknown when
this entry was written. It was established afterwards; see
[The musl anomaly](#the-musl-anomaly--resolved-on-2026-09-11) below before
drawing anything from it.

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

### The musl anomaly — resolved on 2026-09-11

**Recorded, not buried.** `sqlx` with TLS over direct Linux loopback takes
61.937 ms, of which `connect_with` alone accounts for **44.18 ms**. The query is
~1.1 ms in every variant, so the cost is entirely in connection establishment.

During those 44 ms the process is at **0% CPU with 9 voluntary context
switches** — it is blocked, not computing.

Nagle's algorithm was the obvious suspect, the ~40 ms magnitude matching a
delayed ACK almost exactly, and this entry originally recorded it as *tested and
refuted*: toggling it on the sync driver (`tcp_nodelay(false)`) gave 3.85 ms
against 3.83 ms — unchanged. **That test refuted nothing.** It exercised Nagle
in isolation, on a driver that does not reproduce the stall, and the stall
requires two ingredients at once.

**The mechanism was established on 2026-09-11 under roadmap task `#8`, and the
account is [the 2026-09-11 entry](#2026-09-11--the-tls-connect-stall-on-linux-loopback),
which supersedes this section.** In short: `sqlx-core` 0.9.0 never sets
`TCP_NODELAY`, and its `StdSocket` inherits a `write_vectored` that writes only
the first buffer, so the TLS client flight leaves split and Nagle holds the
`Finished` record for one Linux delayed ACK. The anomaly's absence on the macOS
host, where the path crosses Docker Desktop's port proxy instead of raw
loopback, is confirmed there; its cause on that path is not separated, and that
entry says so.

Nothing in this entry's decision rested on this row, and nothing in it changes:
the async candidate wins the musl comparison on the plaintext pair, and the
anomaly was a defect in the driver, not a property of the driver choice.

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

### Standing relative to the measurement protocol

This entry is a **driver-selection record**. It is not a reading of any
measurement point of `NFR-PERF-014`, and no figure in it stands as the reference
figure of one. It departs from that file's measurement protocol in three ways,
stated here so that the next entry is not modelled on it by accident: 5 warmups
per round rather than the 20 of `NFR-PERF-009`; a host that was not idle,
against `NFR-PERF-010`; and a relative standard deviation above the 5% of
`NFR-PERF-011` on several rows. It also reaches a server, which three of the
nine measurement points do and the other six do not.

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

---

## 2026-09-11 — The TLS connect stall on Linux loopback

*Sprint 4, task `#8`. This entry establishes the mechanism behind the anomaly the
2026-09-10 entry recorded as unexplained, and supersedes that entry's account of
it. Target of record: `aarch64` Linux, every figure taken inside a container
under Docker Desktop's Linux VM, on an `aarch64-apple-darwin` host; one figure
taken on `aarch64-apple-darwin` itself and labelled as such. Servers of record:
MariaDB `10.11`, `11.4`, `11.8` and `12.3`. This is a defect investigation, not a
reading of a measurement point: no figure here stands as a reference figure for
`tpl`.*

### What was established

`sqlx` 0.9.0's TLS connect stalls for ~40 ms because the client's last handshake
flight is **blocked**, not because any part of it is expensive. It is **not a
cost**, and it needs **two ingredients at the same time** — which is why task
`#7`'s test of Nagle in isolation refuted nothing.

1. **Nagle stays on.** `sqlx-core` 0.9.0 never calls `set_nodelay`. The string
   `nodelay` occurs zero times in the crate, and no `setsockopt` appears
   anywhere in the syscall trace of a connect.
2. **The TLS 1.3 client flight leaves split across separate segments.**
   `rustls` always calls `write_vectored` with all pending records at once, but
   `sqlx_core::net::tls::util::StdSocket` implements `io::Write` with `write`
   and `flush` only. It therefore inherits `std`'s default `write_vectored`,
   **which writes only the first buffer**. The gather never happens.

Chained, they produce the stall:

- the 6-byte `ChangeCipherSpec` record goes out alone;
- the `Finished` record queued behind it is held by Nagle, because there is
  small unacknowledged data in flight;
- the server has nothing to send until it sees that `Finished`, so it answers
  only with Linux's delayed ACK — `TCP_DELACK_MIN` is `HZ/25`, i.e. **40 ms**;
- that ACK releases Nagle, and everything accumulated leaves at once.

**Plaintext never stalls** because the MySQL protocol in clear is strict
ping-pong: it never issues two writes without a read between them, so there is
never unacknowledged small data in flight to hold the next write back.

### Wire evidence

`tcpdump -i lo -n -ttt`, run inside the server container's own network
namespace. The second column is the inter-packet delta:

```
 0.000122  client > 3306: [P.], seq 255:261, length 6     <- 14 03 03 00 01 01  ChangeCipherSpec
 0.042147  3306 > client: [.],  ack 261,     length 0     <- DELAYED ACK, +42.147 ms
 0.000007  client > 3306: [P.], seq 261:463, length 202   <- Finished (82) + HandshakeResponse (120)
```

`strace` agrees, and shows that nothing is computing meanwhile: three `sendto`
calls of the client's own, then `epoll_pwait(...) = 1 <0.040814>`.

### Workload, candidates and protocol

Three harnesses, each isolating a different thing.

**1. A 2x2 factorial**, purpose-built: blocking `std` sockets driving `rustls`'
synchronous API against a real MariaDB server, with **no `tokio` in the binary at
all**. The two factors are `TCP_NODELAY` and whether the client's records are
written split or coalesced. Measured: the client segment lengths seen on the
wire, and the time from the start of the flight to the first server response.

**2. The untouched `sqlx` binary**, with `TCP_NODELAY` forced on by an
`LD_PRELOAD` shim rather than by any change to the crate. Measured:
`connect_with`, five repeats per state.

**3. Real `sqlx-core` 0.9.0 carrying the candidate fixes**, applied through
`[patch.crates-io]`, each fix switchable at runtime so that one binary produces
all four cells. Measured: `connect_with`, seven repeats per variant, plus a
phase breakdown instrumented identically to task `#7`'s.

### Resolution of the instrument

No A/A twin was run for this entry, and none was needed: the effect is roughly
**30x the spread of the repeats within any cell**. Every result below is
therefore printed as its raw repeats rather than as a mean, so that the spread
is visible without a summary statistic standing between the reader and it.

### Results — the 2x2 factorial

| `TCP_NODELAY` | writes | client segments | time to first response |
|---|---|---|---|
| off | split | `[218, 6, 82, 122]` | **43.7 – 45.2 ms** |
| off | coalesced | `[218, 88, 122]` | 0.105 – 0.132 ms |
| on | split | `[218, 6, 82, 122]` | 0.101 – 0.109 ms |
| on | coalesced | `[218, 88, 122]` | 0.088 – 0.103 ms |

Only the cell that reproduces `sqlx`'s configuration stalls. **Removing either
ingredient removes the stall**, which is what makes the mechanism a conjunction
rather than a single cause. Because this binary links no `tokio`, the async
runtime is exonerated.

### Results — the untouched `sqlx` binary under the `LD_PRELOAD` shim

`connect_with`, five repeats:

| state | `connect_with` (ms) |
|---|---|
| as published | 42.9  41.7  41.9  41.5  41.9 |
| `TCP_NODELAY` forced on by the shim | 1.34  1.39  1.28  1.34  1.29 |

The crate is byte-identical across the two rows; only the socket option differs.

### Results — candidate fixes patched into `sqlx-core` 0.9.0

`connect_with`, seven repeats:

| variant | `connect_with` (ms) |
|---|---|
| both off (= published 0.9.0) | 49.9  42.9  42.6  46.7  42.6  43.4  41.9 |
| `set_nodelay(true)` (the upstream fix) | 1.22  1.36  1.38  1.36  1.37  1.31  1.34 |
| `write_vectored` on `StdSocket` only | 1.28  1.31  1.27  1.34  1.47  1.39  1.27 |
| both | 1.55  1.49  1.40  1.28  1.25  1.26  1.39 |

Either change alone removes the stall, and the two together are not measurably
better than either alone on this path. **Whether `tpl` adopts a workaround, and
which, is an architecture decision that this entry does not make**; it is raised
as roadmap task `#38`. What is recorded here is the measured effect of each
option, and nothing beyond it.

### Results — phase instrumentation

Instrumented identically to task `#7`'s phase breakdown:

| variant | connect | set session | query |
|---|---|---|---|
| TLS `verify-identity`, published 0.9.0 | **42.27 ms** | 0.22 ms | 0.28 ms |
| TLS `verify-identity`, with `set_nodelay` | **1.38 ms** | 0.11 ms | 0.17 ms |
| no TLS (`disabled`) | 0.46 ms | 0.10 ms | 0.11 ms |
| server started `--skip-ssl` | 0.59 ms | 0.10 ms | 0.13 ms |

The cost is entirely in connection establishment; the phases after it are
unaffected in every variant.

### Independent of server series and of network interface

All four supported series stall: `10.11`, `11.4`, `11.8` and `12.3` all land in
**41–52 ms**, and all drop to **~1.3 ms** with `TCP_NODELAY` forced. It is not a
property of one server series.

It is not a property of loopback either: it reproduces over the docker bridge
veth path at **41.7 – 45.1 ms**.

It is absent in exactly one measured place — the macOS host reaching Docker
Desktop's published port, at **2.2 – 2.9 ms** (`aarch64-apple-darwin`; by the
standing rule, not comparable with the Linux figures above). There a userland
TCP proxy terminates the connection. **The writes are still split there**, so
the absence is a property of that path, not of the client.

### Whose defect it is, and its status upstream

It is `sqlx`'s, and it is a **regression in a published version**:

| version | `set_nodelay` |
|---|---|
| `sqlx-core` 0.7.4 | present |
| `sqlx-core` 0.8.0 | present |
| `sqlx-core` 0.8.6 | present |
| **`sqlx-core` 0.9.0**, published 2026-05-21 | **zero occurrences** |
| `main` | present again |

Chronology, in the repository `https://github.com/transact-rs/sqlx`:

- originally fixed by issue **#3043** → PR **#3055**, merged 2024-02-16;
- dropped by the runtime rewrite, commits `6b828e698f` (PR **#3791**) and
  `66526d9c56` (PR **#4022**), both 2025-09-08;
- re-reported by a third party as issue **#4335** on 2026-07-10, with an
  identical symptom — "41-43ms", against MariaDB/MySQL, `runtime-tokio` with
  `tls-rustls`;
- fixed on `main` by PR **#4336**, merged 2026-08-17.

**No release carries the fix. 0.9.0 is still the latest on crates.io.**

### What is not reported upstream

`StdSocket` still lacks a `write_vectored` implementation on `main`, so every
`rustls` flight still leaves split. That is benign once Nagle is off — which is
why the upstream fix is sufficient — but it is the reason a missing
`TCP_NODELAY` costs 40 ms here rather than costing nothing.

### What was not measured — stated, not implied

- **No bare-metal Linux host was available.** This host is
  `aarch64-apple-darwin`, and every Linux figure above was taken inside Docker
  Desktop's Linux VM. The bare-metal case is **untested**, not inferred.
- **Why the macOS host path does not stall was not separated** between Docker
  Desktop's userland proxy and Darwin's own Nagle and delayed-ACK behaviour.
  Both remain candidate explanations.
- **Task `#7`'s synchronous candidate was not rebuilt.** The explanation that it
  coalesces because `std::net::TcpStream` implements `write_vectored` is
  **inference from the factorial**, not a measurement of that crate.
- **Figures from different harnesses are not compared number to number.** The
  1.38 ms connect above and the 4.03 ms connect from task `#7`'s phase breakdown
  come from different harnesses; so do the 42.27 ms above and the 44.18 ms
  recorded in the 2026-09-10 entry. Each is read against the other figures of
  its own harness.

### Standing relative to the measurement protocol

This entry is a **defect investigation**. It is not a reading of any measurement
point of `NFR-PERF-014`, and no figure in it stands as the reference figure of
one. It departs from that file's measurement protocol further than the
2026-09-10 entry does, and deliberately: a handful of repeats rather than pooled
hundreds, no warmups, no A/A twin, a host that was not idle against
`NFR-PERF-010`, and a server in the loop, which three of the nine measurement
points have and the other six do not. The quantity under measurement is a 40 ms
block against a ~1.3 ms baseline, and the instrument does not need to be sharper
than this to settle it. **Do not model a reading of a measurement point on this
one.**

### Reproduction

```sh
# Wire evidence: capture inside the server container's own network namespace,
# so that loopback is the loopback the client actually writes to.
docker run --rm -it --network container:<server> --cap-add NET_ADMIN \
  <image-with-tcpdump> tcpdump -i lo -n -ttt 'tcp port 3306'

# Syscall evidence: three sendto calls of the client's own, then the wait.
strace -f -tt -T -e trace=network,epoll_pwait <sqlx-binary>

# The crate itself: the option is simply never set in 0.9.0.
grep -rn nodelay <sqlx-core 0.9.0 source>          # no hits

# Toggling the option without touching the crate: an LD_PRELOAD shim that
# intercepts connect(2) and sets TCP_NODELAY on the socket.

# Toggling the fixes inside the crate: vendor sqlx-core 0.9.0 and redirect to it,
#   [patch.crates-io]
#   sqlx-core = { path = "<vendored sqlx-core>" }
# with set_nodelay(true) and StdSocket::write_vectored each behind a runtime flag,
# so that one binary produces all four cells.
```

The 2x2 factorial is a standalone binary: blocking `std::net::TcpStream`,
`rustls`' synchronous `ClientConnection`, no `tokio`. The "split" arm writes each
record with a separate `write`; the "coalesced" arm gathers them into a single
write. Segment lengths are read off the same `tcpdump` capture.

---

## 2026-09-22 — The measurement set on `aarch64-apple-darwin`

*Sprint 15, task `#218`. Target of record: `aarch64-apple-darwin`, measured
natively. Server of record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This is
the first reading of the nine measurement points of `NFR-PERF-014` and of the
`WL-002` scalar. **This entry records; it does not judge.** It contains no pass,
no fail and no regression verdict, and it compares no figure with an adopted
one except where a difference is stated as an observation and labelled as one.*

### Outcome

**All nine measurement points and the `WL-002` scalar were measured, on one of
the four targets of `NFR-PERF-018`.** The campaign completed and exited `0`.

`WL-002` is now valued: the compact `tpl schema dump` of `WL-001` is
**3 182 326 bytes**, identical across all five takes. The ±2% band that scalar
carries is therefore 3 118 680 to 3 245 972 bytes.

Two readings carry a relative standard deviation above five per cent and are
recorded with that fact stated, per `NFR-PERF-011`: point 5 at 8.294%, and the
`64` half of point 6 at 7.513%. Neither may stand as its point's reference
figure until retaken on a quiet host. Neither is a statement about `tpl`.

### Workload

`WL-001` and `WL-003`, both realised by `scripts/mariadb/seed-bench.sql` and
loaded by `scripts/mariadb/seed-bench.sh`, which verified every count the
specification states for them: 200 tables, 2 400 columns, 600 indexes, 180
foreign keys, 40 generated columns, 25 triggers, 30 views, 40 routines and
comments on 120 tables for `WL-001`; 1 table, 12 columns, 3 indexes for
`WL-003`. Four of the ten readings have a workload of `none`.

There are no candidates: this is not a comparison. One binary was measured.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, detected by the harness, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01), `cargo` 1.98.1 (797e8a9bc 2026-08-05) |
| Crate | `tpl` 0.1.0, edition 2024, MSRV 1.94.0 |
| Release profile | `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` |
| Binary | `target/release/tpl`, sha256 `16976d61065884ca3b5eac18e06efe8852410927eae7e0d232dd2eb12d33c927` |
| Instrument | `hyperfine` 1.20.0, always `--shell=none`; `/usr/bin/time -l` for point 9; `jq` 1.7.1-apple |
| Server | MariaDB `12.3.3-MariaDB-ubu2404`, the project fixture, on `127.0.0.1:13309` |
| Transport | `tls = "disabled"`; no reading here carries a TLS handshake |
| Account | `root`, for the reason stated under *Confounders* |
| Taken at | 2026-09-22T14:36:05Z, campaign start |

### Protocol

Every wall-time and memory reading was taken under `NFR-PERF-009` — the median
of **200 runs after 20 warmups** — and every record says `standing: full`. The
`WL-002` scalar is a size and not a duration; it was taken 5 times with no
warmup, which the harness records as such.

Each reading was taken over the workload `NFR-PERF-014` names for its point and
with the cache in the posture that table's `Cache` column names for it: `not
reached` for points 1, 2, 3 and 7; `bypassed, --direct --no-cache` for points 4
and 9; `empty when each run begins` for point 8; `served from`, with the server
removed through `down.sh` and the gate asked again, for points 5 and 6. The
`WL-002` scalar is not a row of that table and fixes no posture; it was taken
`--direct --no-cache`, and the record says that the choice is the harness's.

`NFR-PERF-010` was met in two of its four clauses and **departed from in two**,
which is stated here rather than implied:

| Clause | Met? |
|---|---|
| No intervening shell | **met** — `hyperfine --shell=none` on every wall-time reading |
| Warm page cache | **met** — 20 warmup runs per reading, and the first execution of the binary discarded before any sample |
| First execution of a freshly built binary discarded | **met** — `protocol_discard_first_execution`, once, before the first sample of the campaign |
| Idle host | **departed from** — see *Confounders* |
| Mains power | **departed from** — the host ran on battery throughout, 77% to 68%; macOS Low Power Mode was off (`pmset -g` reports `lowpowermode 0`), so the CPU was not throttled by the battery policy |

`NFR-PERF-012` is met: every record names `aarch64-apple-darwin`, and the four
records whose reading reached a server name series `12.3`.

### The noise floor of the instrument on this host

Two arms of the same command, `tpl --version` against `tpl --version`, at the
full protocol, interleaved by `hyperfine` in one invocation:

| arm | n | median | mean | stddev | min | max | rsd |
|---|---|---|---|---|---|---|---|
| A | 200 | 1.6946 ms | 1.7010 ms | 0.0453 ms | 1.6188 ms | 1.8559 ms | 2.66% |
| A twin | 200 | 1.6947 ms | 1.7050 ms | 0.0589 ms | 1.6100 ms | 1.9551 ms | 3.45% |

**The two medians differ by 0.0001 ms, which is 0.006%.** The median is
therefore stable on this host to well under a tenth of a per cent, even while
the host was not idle. What the load moves is the **dispersion**, not the
centre: the same unchanged binary produced 2.66% and 3.45% on two interleaved
arms. That is the quantity `NFR-PERF-011` speaks of, and it is why two readings
below cross five per cent without anything about `tpl` having changed.

A difference between two figures in this entry smaller than about 0.01 ms is
inside this floor and should not be read as a difference.

### Results

Every figure is a median. `rsd` is the relative standard deviation of the
samples behind it, and the last column is the plain answer to whether it exceeds
the five per cent of `NFR-PERF-011`.

| # | Measurement point | n | warmups | median | mean | stddev | min | max | rsd | over 5%? |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `tpl --version` | 200 | 20 | **1.7561 ms** | 1.7621 ms | 0.0620 ms | 1.6242 ms | 2.0842 ms | 3.521% | no |
| 2 | `tpl --help` | 200 | 20 | **1.8775 ms** | 1.8835 ms | 0.0544 ms | 1.7893 ms | 2.1069 ms | 2.890% | no |
| 3 | Startup to the first byte of useful work | 200 | 20 | **1.8919 ms** | 1.9053 ms | 0.0771 ms | 1.7000 ms | 2.2549 ms | 4.047% | no |
| 4 | `tpl schema dump` | 200 | 20 | **26.684 ms** | 26.867 ms | 0.8123 ms | 25.738 ms | 33.729 ms | 3.023% | no |
| 5 | A cache-served read of one object | 200 | 20 | **2.1152 ms** | 2.1668 ms | 0.1797 ms | 1.9286 ms | 3.2550 ms | **8.294%** | **yes** |
| 6 | The failure path | 200 | 20 | **2.0667 ms** | 2.0788 ms | 0.0779 ms | 1.9168 ms | 2.5344 ms | 3.745% | no |
| 7 | `tpl help --format json` | 200 | 20 | **1.9946 ms** | 1.9990 ms | 0.0625 ms | 1.8819 ms | 2.2378 ms | 3.129% | no |
| 8 | The canonical loop of 200 invocations | 200 | 20 | **6850.70 ms** | 6865.35 ms | 77.986 ms | 6743.54 ms | 7310.16 ms | 1.136% | no |
| 9 | Peak resident memory | 200 | 20 | **9 076 736 B** | 9 060 598 B | 80 709 B | 8 896 512 B | 9 191 424 B | 0.891% | no |
| — | `WL-002`, the verification scalar | 5 | 0 | **3 182 326 B** | 3 182 326 B | 0 B | 3 182 326 B | 3 182 326 B | 0.000% | no |

#### Point 6, both halves

`NFR-PERF-014` makes this one point measured by two invocations, whose figure is
the **slower** of the two, with the other recorded beside it. The `66` is the
slower half, which is what that requirement predicts, and it is the figure the
row above carries.

| half | n | median | mean | stddev | min | max | rsd | over 5%? |
|---|---|---|---|---|---|---|---|---|
| `64`, an undeclared flag | 200 | 1.8602 ms | 1.8853 ms | 0.1416 ms | 1.6580 ms | 2.9842 ms | **7.513%** | **yes** |
| `66`, with nearest match | 200 | **2.0667 ms** | 2.0788 ms | 0.0779 ms | 1.9168 ms | 2.5344 ms | 3.745% | no |

The `66` costs **0.2065 ms more** than the `64` — twenty times the noise floor,
and therefore a difference and not noise. That difference is the edit distance
of `FR-ERR-019` computed against the 200 names of `WL-001`, which is the
quantity `BR-PERF-004` says this point exists to measure.

#### The invocations, as a reader would retype them

| # | Invocation |
|---|---|
| 1 | `tpl --version` |
| 2 | `tpl --help` |
| 3 | `tpl template list` |
| 4 | `tpl -d bench_wl001 schema dump --direct --no-cache` |
| 5 | `tpl -d bench_wl003 schema table consignment` |
| 6 | `tpl -d bench_wl001 schema table accrual --no-such-flag` and `tpl -d bench_wl001 schema table accrualx` |
| 7 | `tpl help --format json` |
| 8 | `benches/loop200.sh <binary> bench_wl001 example <names-file>` |
| 9 | `tpl -d bench_wl001 schema dump --direct --no-cache` |
| — | `tpl -d bench_wl001 schema dump --direct --no-cache`, and the byte length of what it wrote |

Points 3, 5, 6, 8 and 9 and the scalar are invoked inside the projects the
harness builds under `target/bench-work`; `tpl template list` for point 3 runs
in a project holding no database entry, which is what `NFR-PERF-014` names for
that row.

### Observations, which are not verdicts

`BR-PERF-008` means no figure here decides anything. These are stated because a
reader comparing this entry with the adopted figures of `NFR-PERF-014` will
notice them, and it is better that the entry name them than that a reader infer
something from them.

- **Point 4 came in at about a nineteenth of its adopted figure.** The adopted
  figure is `< 500 ms`, taken from a root document and stated there as dominated
  by server time; the reading is 26.684 ms, which is 5.3% of it. The measured
  read of a 200-table catalogue over a plaintext loopback connection to a
  container on this host cost 26.7 ms. Nothing follows from the gap by rule.
- **The adopted figures assume point 3 costs twice what points 1 and 2 cost;
  the measurement does not show that.** `NFR-PERF-014` gives points 1 and 2
  `< 5 ms` and point 3 `< 10 ms`, on the stated ground that point 3 is useful
  work where the others are static text. Measured, point 3 is **1.8919 ms**
  against point 2's **1.8775 ms** — 0.0144 ms apart, which is 0.8% and not 100%,
  and which sits just above the 0.01 ms this host's noise floor resolves, so the
  ordering is real and the magnitude is at the edge of what the instrument can
  see. Against point 1 the gap is 0.1358 ms, or 7.7%. The project discovery and
  configuration read that distinguish point 3 cost, on this host, a small
  fraction of what the adopted ratio anticipated.
- **Point 9 came in at about 8.66 MiB against an adopted `< 32 MiB`,** which is
  27% of it, for the whole-catalogue read of a 200-table database.
- **Point 8 averages 34.25 ms per invocation** over its 200 (6850.70 / 200).
  Each run is one server read followed by 199 cache hits over `WL-001`. This is
  **not** comparable with point 5, which is a cache-served read over `WL-003`, a
  database of one table: the two carry different workloads and `NFR-PERF-012`'s
  reasoning applies to the difference.
- **`BR-PERF-004` expects a wrong invocation to cost what `tpl --version`
  costs.** Measured, the `66` half costs 2.0667 ms against `tpl --version`'s
  1.7561 ms — 17.7% more — and the `64` half 1.8602 ms, 5.9% more. The point
  exists so that this expectation can be checked against a reading instead of
  asserted, which is what this line does.

### Confounders — read this before the tables

- **The host was not idle, against `NFR-PERF-010`.** Throughout the campaign a
  Firefox tab was decoding video (`VTDecoderXPCService` at 17-19% CPU), iTerm2
  was active, and four containers belonging to other projects were running under
  Docker Desktop's Linux VM (`sapoteca-solr-1`, `sapoteca-zookeeper-1`,
  `mongodb-mongo-1`, `mongodb-mongo-express-1`), with the
  `Virtualization.VirtualMachine` process at ~5% CPU and 3.3 GB resident. Load
  average was 2.17 at campaign start and 3.39 at its end. This is the departure
  that the two dispersions above five per cent are most plausibly about, and it
  is why the A/A twin above was taken: the medians survive it, the dispersions
  do not.
- **The host ran on battery, against `NFR-PERF-010`.** 77% at the start, 68% at
  the end, discharging throughout. Low Power Mode was off, so the CPU was not
  throttled by that policy, but no reading here was taken on mains power and
  none should be described as if it were.
- **`hyperfine` reported statistical outliers** on several readings and
  recommended a quieter system. The maxima in the table bear it out: point 4's
  max of 33.73 ms sits 26% above its median, and the `64` half of point 6 has a
  max of 2.98 ms against a median of 1.86 ms. Medians are reported precisely
  because they are insensitive to those tails.
- **The server path is not raw loopback.** It crosses Docker Desktop's port
  proxy, as the 2026-09-10 entry established for this same host. Points 4, 8 and
  9 and the scalar carry that path in their figures, and a bare-metal Linux
  server would be a different measurement.
- **The benchmark entries authenticate as `root`, not as `tpl_reader`.**
  `seed-bench.sql` grants the reader nothing on `freight_wl001` or
  `freight_wl003`, so a reader-backed entry would present an empty catalogue —
  which `FR-PRIV-001` makes a silent success — and a reading over an empty
  catalogue is not a reading over `WL-001`. Registered as `#224`.
- **Points 5 and 6 are dominated by process startup.** Both read a cache file
  written while the server was up, with the server then removed; at 2.07 and
  2.12 ms against `tpl --version`'s 1.76 ms, the work they measure beyond
  starting the process is a few hundred microseconds. The `64` half of point 6
  is refused before any read at all, so it reaches neither cache nor server and
  measures argument parsing and refusal.
- **Point 8's figure includes its driver.** `benches/loop200.sh` performs a
  `read` and an exec per line; that cost is inside the 6850.70 ms, as
  `BR-PERF-005` intends, since the loop is the subject and not a wrapper.
- **Point 9 uses a different instrument from every other reading.**
  `/usr/bin/time -l` reports `maximum resident set size` in bytes on macOS;
  `hyperfine` takes no part in it, because it measures time.
- **`scripts/mariadb/up.sh` exits `2` on success** when asked for a single
  server, because it ends with an unfiltered `status.sh`. The harness does not
  read its exit code and asks the filtered gate immediately afterwards.
  Registered as `#223`.

### What was not measured — stated, not implied

- **Three of the four targets of `NFR-PERF-018` were not measured at all.**
  `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and
  `x86_64-apple-darwin` carry **no figure from this campaign**, and nothing in
  this entry may be read as covering them. `aarch64-apple-darwin` is the only
  target this host can measure natively, and `NFR-PERF-012` forbids carrying a
  figure across targets. `BR-PERF-003` records that no point is required to be
  measured on every target: a point with a figure on one target and none on the
  other three is complete rather than short.
- **Three of the four series of `FR-SRV-015` were not measured.** Points 4, 8
  and 9 and the scalar were taken against `12.3` only. `10.11`, `11.4` and
  `11.8` carry no figure here, and `NFR-PERF-012` forbids comparing across
  series as it forbids comparing across targets.
- **No reading carries a TLS handshake.** The benchmark entries asked for
  `tls = "disabled"`. What the four TLS modes cost is not measured here.
- **The `WL-002` scalar was taken 5 times, not 200.** It is a size and not a
  duration, and it was byte-identical on all five takes.

### Reproduction

```sh
# 1. The binary under measurement. The release profile is the one a user runs.
cargo build --release
shasum -a 256 target/release/tpl
#   16976d61065884ca3b5eac18e06efe8852410927eae7e0d232dd2eb12d33c927

# 2. The noise floor of the instrument on this host: two identical arms,
#    interleaved by hyperfine in one invocation, at the full protocol.
hyperfine --shell=none --style none --warmup 20 --runs 200 \
  -n A "$PWD/target/release/tpl --version" \
  -n A-twin "$PWD/target/release/tpl --version"

# 3. The whole campaign: nine measurement points and the WL-002 scalar, under
#    the full protocol, against series 12.3. The harness stands the fixture up
#    through its own scripts, loads both workloads, and leaves the fixture down.
#    It exits 0 whatever it measured.
./benches/run.sh --series 12.3 --out readings.jsonl

# 4. The table above, from the records.
jq -r '[.id, (.median|tostring), .unit, (.n|tostring),
        ((.rsd_pct*1000|round/1000)|tostring), (.rsd_over_5pct|tostring)]
       | @tsv' readings.jsonl

# 5. Both halves of point 6.
jq -r 'select(.id=="point-6") | .parts[]
       | "\(.part)  median=\(.median)  rsd=\(.rsd_pct)  over5=\(.rsd_over_5pct)"' \
  readings.jsonl

# 6. The raw samples behind every figure, one file per reading, and hyperfine's
#    own exports, are left under the work directory.
ls target/bench-work/samples/ target/bench-work/hyperfine/

# 7. The host conditions this campaign departed from, checked the same way.
pmset -g ps                 # power source
pmset -g | grep lowpowermode
uptime                      # load average
docker ps --format '{{.Names}}'
```

The fixture is operated only through `scripts/mariadb/`: `up.sh` starts and
verifies the server, `seed-bench.sh` loads `WL-001` and `WL-003` and verifies
every count they state, `status.sh` is the gate, and `down.sh` removes the
server and proves nothing is left. `benches/run.sh` calls exactly those and
writes no `docker` command of its own. The campaign leaves the fixture down,
which `./scripts/mariadb/status.sh --quiet 12.3` confirms by exiting non-zero.
