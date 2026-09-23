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

---

## 2026-09-22 — Waste-hunting campaign: vacuous CPU and RAM across `tpl`

*Sprint 19, task `#231`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3`, with `10.11`, `11.4` and `11.8` for the catalogue read
only.*

### Outcome

Every path the tool executes was profiled, and the work that does not serve the
invoked command is ranked below by estimated gain over estimated effort. Nothing
was changed: `src/`, `Cargo.toml` and `Cargo.lock` are untouched, and every
instrumented or altered binary was built from a throwaway copy of the crate
outside the repository.

Two defects in one function dominate everything else. **Writing the cache costs
2.47 s for `WL-001`, and 97.4% of that is waste**: `store`
(`src/cache.rs:686`) serialises each object straight into an unbuffered `File`,
one `write` syscall per JSON token, and then calls `sync_all`, which on macOS is
`fcntl(F_FULLFSYNC)`, once per file. With both removed the same 272 files are
written, byte-identical, in 63.9 ms. Every read-through miss pays this cost: the
first cached read after `tpl cache clean`, every `tpl cache load`, and — with the
server up — every `66` on an absent object name, which measured 2 641 ms. It is
**33.6% of the canonical loop of 200 invocations**.

The next tier is the render context and the cached `schema` reads, which decode
or materialise the whole catalogue for commands that present a fraction of it.

### Workload

`WL-001` (200 tables) and `WL-003` (1 table), loaded by
`scripts/mariadb/seed-bench.sh`, which verified every count on all four series.
The two projects are the ones `benches/fixture.sh` builds (`startup`, with no
database entry; `server`, with `bench_wl001` and `bench_wl003`), plus the five
templates of `examples/rust-data-layer/templates/rust/` copied into the `server`
project so that a whole-database render (`rust/schema`) and a table render
(`rust/struct`) could be measured beside the stock `example`. A third project
carried one entry per series for the catalogue-read comparison. The cache was
primed with `tpl cache load`, as `fixture_prime` does.

Thirty-two invocations were measured, covering every command of the tree except
those listed under *What was not measured*. The labels used below:

| label | invocation (in the project named) |
|---|---|
| `version`, `help`, `helpjson` | `tpl --version`, `tpl --help`, `tpl help --format json` (startup) |
| `nodehelp` | `tpl schema table --help` (startup) — a help form at a leaf with a required operand |
| `tlist`, `tshow`, `tcheck`, `tpath` | `tpl template list` (startup); `template show rust/schema`, `template check rust/schema`, `template path rust/struct` (server) |
| `cfglist`, `cfgget`, `dblist`, `dbshow` | `tpl cfg list`, `cfg get core.database`, `cfg database list`, `cfg database show bench_wl001` |
| `fail64`, `fail66` | `-d bench_wl001 schema table accrual --no-such-flag`; `-d bench_wl001 schema table accrualx` |
| `cstatus`, `cload` | `-d bench_wl001 cache status`; `-d bench_wl001 cache load` |
| `info_c`, `tables_c`, `table_c`, `table3_c`, `view_c`, `routine_c`, `dump_c` | the `schema` reads over `WL-001` (`table3_c` over `WL-003`), served from the cache |
| `rstruct_c`, `rexample_c`, `rschema_c` | `-d bench_wl001 render rust/struct --table accrual`, `render example --table accrual`, `render rust/schema`, served from the cache |
| `*_d` | the same reads with `--direct --no-cache`, served by the server |

### Candidates

There is one subject, the binary of record, and five throwaway variants built
from copies of the crate for single-variable attribution. None exists in the
repository.

| variant | the one change against the binary of record |
|---|---|
| `nop` | an empty `fn main() {}` under the identical release profile: the process-spawn floor |
| `exp` | two hooks in `main.rs`: `--exp-early` writes the version line before `tpl::run`, and `TPL_EXP_TREE=N` times `N` builds of `cli::tree()` |
| `nosync` | `exp`, with `handle.sync_all()?` removed from `store` (`src/cache.rs:702`) |
| `buf` | `exp`, with the serialisation of `store` wrapped in a `BufWriter` |
| `nosyncbuf` | both of the above |
| `forget` | `exp`, with `std::mem::forget(context)` after the render in `produce` (`src/cli/render.rs:500`–`502`) |
| `dhat` | `dhat` 0.3.3 as the global allocator, `debug = 1`, `strip = false`; used for allocation counts only, never for time |

Every variant that writes the cache wrote the same 272 files byte for byte as the
binary of record (`diff -r`, `meta.json` compared without `loaded_at`), and
`forget` produced byte-identical render output.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01), `cargo` 1.98.1 (797e8a9bc 2026-08-05), LLVM 22.1.8 |
| Crate | `tpl` 0.1.0 at `bc597dad`, edition 2024, release profile `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` |
| Binary | `target/release/tpl`, 3 934 128 B, sha256 `16976d61065884ca3b5eac18e06efe8852410927eae7e0d232dd2eb12d33c927` — the binary of the 2026-09-22 measurement-set entry |
| Instruments | `hyperfine` 1.20.0 (`--shell=none`); `samply` 0.13.1 at 20 kHz; `dhat` 0.3.3; `cargo-bloat` 0.12.1; `/usr/bin/time -l`; `jq` 1.7.1-apple |
| Server | the project fixture: `12.3.3`, `11.8.9`, `11.4.13` (`-ubu2404`) and `10.11.19-MariaDB-ubu2204`, `tls = "disabled"`, account `root`, Docker 29.8.1 |
| Power | mains (`AC Power`, battery 80%, not charging); Low Power Mode off |
| Load | 1.56 to 4.22 over the session; see *Confounders* |
| Taken | 2026-09-22, 17:46Z to 19:07Z |

`cargo flamegraph` was not used: on macOS it needs `dtrace` and root. `samply`
was used in its place. Nothing was installed; every instrument was already
present.

### Protocol

- **Wall time.** Eight rounds of 40 runs after 5 warmups per invocation — 320
  samples each — with the order of all 34 invocations rotated by one position per
  round, so that host drift spreads across every invocation instead of landing on
  whichever ran last. Each attribution experiment was its own interleaved
  campaign: the variants under comparison rotated inside one `hyperfine` call per
  round.
- **CPU attribution.** `samply record --rate 20000 --iteration-count N
  --reuse-threads --unstable-presymbolicate` over a symbolised build of the same
  source (`debug = true`, `strip = false`, separate target directory), with `N`
  between 3 and 200 so that each profile holds 545 to 172 077 samples. Shares are
  of on-CPU samples of the `tpl` processes; a share is converted to milliseconds
  only by multiplying it by the median wall time of the binary of record, and
  such a figure is labelled an estimate.
- **Allocation.** One run of the `dhat` variant per invocation, reporting total
  bytes and blocks, the peak (`t-gmax`), and the allocation sites grouped by the
  first `tpl` frame and by caller. Allocation is deterministic here: repeated
  runs differed by at most 0.3% (a timestamp and the process id).
- **Peak resident memory.** `/usr/bin/time -l`, median of 7 runs.
- **Binary size.** `cargo bloat --release --crates` and `size -m`.

This campaign does not follow `NFR-PERF-009` (200 runs after 20 warmups): it
asks attribution questions, not the nine measurement points, and 320 samples per
invocation give medians that the noise floor below resolves.

### The noise floor of the instrument on this host

Two arms of the same command, `tpl --version`, measured as two separate labels
inside the rotated campaign:

| arm | n | median | mean | sd | rsd |
|---|---|---|---|---|---|
| A | 320 | 1.722 ms | 1.729 ms | 0.054 ms | 3.13% |
| A twin | 320 | 1.736 ms | 1.741 ms | 0.103 ms | 5.90% |

The medians differ by **0.014 ms (0.8%)**. Two builds of the same code with
different layouts (`orig` against `exp`, both `--version`, 400 samples each)
differ by 0.013 ms. **A difference below about 0.02 ms is not a difference in
this entry.** The spawn floor (`nop`) is 1.356 ms in the campaign and 1.331 ms in
the start-up experiment.

### Results — every path, wall time and memory

Medians of 320 samples; peak RSS is the median of 7; `dhat` is one run.

| label | median | rsd | peak RSS | heap allocated | blocks | heap peak |
|---|---|---|---|---|---|---|
| `nop` | 1.356 ms | 11.25% | 1.41 MiB | — | — | — |
| `version` | 1.722 ms | 3.13% | 2.64 MiB | 372 235 B | 1 074 | 260 202 B |
| `help` | 1.829 ms | 2.84% | 2.94 MiB | 666 243 B | 2 396 | 260 193 B |
| `helpjson` | 1.922 ms | 2.64% | 3.11 MiB | 738 399 B | 2 928 | 264 172 B |
| `nodehelp` | 1.964 ms | 2.48% | — | 1 402 995 B | 4 299 | 670 767 B |
| `tlist` | 1.826 ms | 3.36% | 2.95 MiB | 446 631 B | 1 736 | 300 028 B |
| `tshow` | 1.840 ms | 2.91% | — | 460 457 B | 1 727 | 299 987 B |
| `tcheck` | 1.938 ms | 2.86% | — | 510 347 B | 2 268 | 299 991 B |
| `tpath` | 1.812 ms | 2.41% | — | 448 029 B | 1 754 | 300 648 B |
| `cfglist` | 1.824 ms | 2.92% | 3.14 MiB | 482 108 B | 1 812 | 312 907 B |
| `cfgget` | 1.829 ms | 2.92% | — | 483 130 B | 1 841 | 313 532 B |
| `dblist` | 1.859 ms | 4.63% | — | 597 914 B | 2 286 | 379 327 B |
| `dbshow` | 1.858 ms | 4.34% | — | 600 124 B | 2 315 | 379 971 B |
| `fail64` | 1.748 ms | 4.58% | — | 478 268 B | 1 343 | 330 010 B |
| `fail66`, server up | **2 641.3 ms** | 2.42% | 8.69 MiB | 7 175 019 B | 18 818 | 4 228 392 B |
| `cstatus` | 5.526 ms | 4.46% | 6.78 MiB | 3 803 133 B | 2 735 | 3 146 171 B |
| `info_c` | 12.199 ms | 2.01% | 13.06 MiB | 23 945 725 B | 48 926 | 8 224 224 B |
| `tables_c` | 11.249 ms | 1.75% | 12.77 MiB | 23 709 970 B | 47 997 | 8 139 312 B |
| `table_c` | 1.992 ms | 2.77% | 3.69 MiB | 662 514 B | 2 338 | 342 381 B |
| `table3_c` | 1.954 ms | 2.48% | — | 609 880 B | 2 223 | 342 397 B |
| `view_c` | 1.929 ms | 3.28% | — | 541 364 B | 2 129 | 342 438 B |
| `routine_c` | 2.482 ms | 2.49% | — | 738 925 B | 2 930 | 342 467 B |
| `dump_c` | 14.740 ms | 1.89% | 13.14 MiB | 23 963 518 B | 50 096 | 8 224 216 B |
| `rstruct_c` | 22.071 ms | 1.89% | 34.53 MiB | 43 855 737 B | 128 835 | 26 893 630 B |
| `rexample_c` | 21.547 ms | 1.97% | 34.19 MiB | 42 396 380 B | 121 723 | 26 742 242 B |
| `rschema_c` | 27.923 ms | 1.29% | 34.56 MiB | 61 184 229 B | 224 668 | 26 892 157 B |
| `dump_d` | 26.071 ms | 1.54% | 8.69 MiB | 6 671 798 B | 15 388 | 4 167 658 B |
| `table_d` | 23.574 ms | 2.11% | 8.72 MiB | 6 720 338 B | 15 474 | 4 171 291 B |
| `table3_d` | 11.451 ms | 2.51% | — | 815 055 B | 4 150 | 343 245 B |
| `info_d` | 23.500 ms | 1.86% | — | 6 673 443 B | 15 414 | 4 168 105 B |
| `rstruct_d` | 33.401 ms | 1.37% | — | 26 562 181 B | 94 121 | 22 854 992 B |
| `rschema_d` | 39.186 ms | 1.24% | 30.00 MiB | 43 892 349 B | 189 958 | 22 853 519 B |
| `cload` | **2 656.2 ms** | 2.37% | 8.64 MiB | 7 101 553 B | 18 573 | 4 228 376 B |

Measured outside the rotated campaign, 60 to 100 runs each: `tpl init` 2.102 ms;
`tpl cfg set core.database …` 1.944 ms; `tpl -d bench_wl003 cache clean`
2.475 ms (20 runs); `tpl cfg database test bench_wl001` 4.010 ms; `tpl render
example --table accrual --context <dump of WL-001>` 18.290 ms against 21.691 ms
for the same render served from the cache, with byte-identical output.

### Results — attribution experiments

**The cache write** (`cload`, 40 samples per variant, 8 rotated rounds):

| variant | median | sd | against the binary of record |
|---|---|---|---|
| binary of record | 2 472.0 ms | 37.6 ms | — |
| `nosync` | 1 339.2 ms | 24.3 ms | −1 132.8 ms |
| `buf` | 1 387.4 ms | 51.8 ms | −1 084.6 ms |
| `nosyncbuf` | **63.9 ms** | 5.1 ms | **−2 408.1 ms, −97.4%** |

The profile agrees and says why: in `cload`, 57.7% of samples are in the kernel's
`write`, reached from `<std::fs::File as Write>::write_all` under
`serde_json::to_writer` (`src/cache.rs:700`), and 37.8% in `fcntl`, reached from
`sync_all` (`src/cache.rs:702`). The server read itself is under 1%.

**The canonical loop** (`benches/loop200.sh <bin> bench_wl001 example`, cache
cleaned before every run by `--prepare`, 12 samples per variant):

| variant | median | against the binary of record |
|---|---|---|
| binary of record | 6 834.9 ms | — (the measurement-set entry recorded 6 850.70 ms) |
| `nosyncbuf` | 4 537.7 ms | −2 297.2 ms, −33.6% |
| `forget` | 6 346.2 ms | −488.7 ms, −7.2% |

**The render context destructor** (`rexample_c`, 300 samples per variant):
21.832 ms against 19.481 ms for `forget` — **−2.351 ms, −10.8%**, outputs
byte-identical.

**Start-up** (400 samples per arm, 10 rotated rounds): `nop` 1.3310 ms; `exp
--exp-early` 1.5805 ms; `exp --version` 1.7248 ms; binary of record
`--version` 1.7118 ms; `--help` 1.8234 ms. Loading and linking the 3.9 MB binary
costs 0.25 ms over the spawn floor, and the whole of `tpl::run` for
`--version` — building the parser tree, parsing, writing one line and dropping
the tree — costs 0.144 ms. One build of `cli::tree()`, warm, in a loop of 2 000,
costs 25 µs.

**Across series** (`dump_d`, 180 samples each): `10.11` 25.862 ms, `11.4`
27.009 ms, `11.8` 26.886 ms, `12.3` 26.420 ms. Heap allocated: 6 666 716 B on
`10.11`, 6 683 269–6 683 273 B on the other three. Every series was read with
the same 14 statements, as `-vvv` reports them. What `tpl` spends does not
depend on the series. The one divergence in the documents is content, not cost:
`10.11` reports `utf8mb4_general_ci` where the other three report
`utf8mb4_uca1400_ai_ci`, on 2 673 `collation` and 144 `collation_connection`
values, which makes its dump 8 448 B shorter.

**Binary size.** `.text` is 3.0 MiB of the unstripped 4.8 MiB; `cargo bloat`
attributes 22.4% to `std`, 22.1% to `tpl`, 17.0% to `minijinja`, 9.7% to
`rustls`, 5.0% to `clap_builder`, 4.6% to `ring`, 3.6% to `serde_json` and 2.4%
to `webpki`. No single crate or function stands out as removable: the TLS stack
serves the modes of `FR-CONF-013`, and the largest function is 36.3 KiB
(`minijinja::vm::Vm::eval_impl`).

### The waste register, sorted by estimated gain over estimated effort

Effort is `S` (one function, no interface change), `M` (a new code path or a
hand-written serde implementation) or `L` (a new abstraction across modules).
"Established" means a single-variable experiment above measured the gain;
"estimate" means it is derived from a profile share multiplied by a measured
median, and is an upper bound unless stated otherwise. Every figure is on
`aarch64-apple-darwin`.

| # | Path | Evidence | Cause in code | Why it is vacuous for the command | Estimated gain | Effort |
|---|---|---|---|---|---|---|
| 1 | Every cache write: `cache load`, the first cached read after a clean, a `66` with the server up | `fcntl` 37.8% of `cload` samples; `nosync` −1 132.8 ms | `handle.sync_all()` once per object file, `src/cache.rs:702`; on macOS this is `F_FULLFSYNC` | No requirement asks the cache for durability. `FR-CACHE-030` and `FR-CACHE-031` ask for atomicity against concurrent writers, which the rename gives; a file torn by a power loss fails to decode and is the miss of `FR-CACHE-033` | **−1 132.8 ms per write, established** | `S` — one line; it removes a guarantee nothing requires, so the decision is recorded first |
| 2 | The same writes | kernel `write` 57.7% of `cload` samples; `buf` −1 084.6 ms | `serde_json::to_writer(&mut handle, …)` into an unbuffered `File`, `src/cache.rs:700`, one syscall per token | The bytes written are identical with a buffer; the syscall per token does nothing for the result, and `CLAUDE.md` already requires buffered I/O | **−1 084.6 ms per write, established**; with row 1, −2 408.1 ms (−97.4%) and −33.6% of the canonical loop | `S` — wrap the handle in a `BufWriter` and flush before the rename |
| 3 | `tpl schema info`, cache-served | 12.199 ms against 1.992 ms for `table_c`; 23 945 725 B in 48 926 blocks; 13.06 MiB RSS; `Loaded::document` 55.9% of samples | `Look::Everything`, `src/cli/schema.rs:362`, which reads and decodes all 272 object files (`src/cache.rs:301`) | The JSON form emits four members from `database.json` (`FR-SCH-031`); the text form adds three counts that the directory listing already gives. `FR-SCH-031` itself rejects making this "the most expensive command of this arm while presenting the least" | **≈ −10.2 ms and ≈ −9.4 MiB RSS per invocation, estimate** (bounded below by `table_c`) | `M` — a lookup that reads the metadata and counts the members |
| 4 | `tpl cache status` | 5.526 ms, 1.8 ms of which is the floor of a discovering command; `members` 93.3% and `read_to_string` 89.3% of samples; 3 182 249 B read | `members(…)`, `src/cache.rs:632`, reads the contents of every object file (`src/cache.rs:672`) to compute `held.len()` | The report is a count per collection; the file contents are read, kept in memory, and dropped unread | **≈ −3.7 ms and ≈ −3.1 MiB RSS per invocation, estimate** | `S` — count the entries `paths::is_object` admits; whether an unreadable file still counts is the one behavioural question to settle |
| 5 | `tpl schema tables` in text form, cache-served | 11.249 ms; `Loaded::document` 62.0% of samples; 23 709 970 B | `decode` of every `TableDocument` in full, `src/cache.rs:193`, reached from `src/cli/schema.rs:389` | The text form shows four fields per table (name, engine, column count, comment); columns, indexes, keys and triggers are decoded and dropped. The JSON form carries the full documents and is not vacuous | **≈ −6.9 ms per text listing, estimate** | `M` — a reduced decode for the text form |
| 6 | Every `tpl render` | `minijinja::value::serialize` 41.4% of `rexample_c` samples; heap peak 26.7 MB against 8.2 MB for `dump_c`; 34.19 MiB RSS for a 494-byte output | `Value::from_serialize(database)`, `src/cli/render/context.rs:173`, converts the whole catalogue into `minijinja` maps (a `BTreeMap` per object: `BTreeMap::insert` 24.7%, `Value::cmp` 12.1%) before the template runs | `FR-RND-023` requires the whole database to be *reachable*, not converted: the `example` template reads one table's columns and the database name, and the other 199 tables are materialised unread | **≈ −8.9 ms and ≈ −21 MiB RSS per render, estimate**; ×199 in the canonical loop | `L` — a lazy `minijinja` object over `DatabaseDocument`, with key order and strict-undefined behaviour preserved |
| 7 | Every `tpl render` | `forget` −2.351 ms (−10.8%); −488.7 ms (−7.2%) on the canonical loop | The `context` value is dropped at the end of `produce`, `src/cli/render.rs:499`–`507`, freeing the whole value tree built for the render, block by block | The process exits immediately after; the operating system reclaims the memory at once | **−2.351 ms per render, established** | `S` — leak the value, or exit, once stdout is flushed |
| 8 | Every cache-served read of a whole collection | 13.0–18.6% of samples in `info_c`, `tables_c`, `dump_c`, 9.0% in `rexample_c`, are the `Content` buffering under `Column` deserialisation; 12 094 208 B in 20 247 blocks at that site for 2 400 columns | `#[serde(flatten)] column_type`, `src/model/column.rs:139`, which makes serde buffer every column object into `Content` and defeats borrowing | The document shape is unchanged by how it is decoded; the buffering is an artefact of the derive, not of the format. The same attribute costs 8 018 048 B in 13 544 blocks on the render side (`src/model/column_type.rs:194`, `FlatMapSerializer`) | **≈ −1.9 to −2.1 ms per whole cached read, estimate (upper bound)** | `M` — a hand-written `Deserialize` for `Column` that keeps the flat shape |
| 9 | Server reads of `WL-001` | `RawVec<Column>::grow_one` 1 976 832 B in 583 reallocations; 0.4% of `dump_d` samples | `table.columns.push(column)`, `src/mariadb/catalogue/fold.rs:434`, into vectors created empty | `CLAUDE.md` asks for pre-sizing where the cardinality is known; the rows are in hand before the push | **≈ −0.10 ms per server read, estimate** | `S` — size each table's vector from its row count |
| 10 | `tpl --help`, `tpl help`, and a help form at a leaf with a required operand | +262 780 B in 697 blocks per extra tree; `nodehelp` 1.964 ms against `help` 1.829 ms against `version` 1.722 ms; the tree is built twice for `--help` and three times for `nodehelp` | `render::text(&super::tree(), path)`, `src/cli/help.rs:282`; `let tree = super::tree()`, `src/cli/help.rs:804`; `waived(tree())`, `src/cli.rs:474` | The tree `parse` built (`src/cli.rs:370`) describes the same nodes; it is dropped and rebuilt | **≈ −0.03 to −0.05 ms per extra build, estimate** (a warm build is 25 µs; near the noise floor) | `S`–`M` — hand the parse tree to the help renderer |
| 11 | Every invocation | `drop_glue::<clap_builder::Command>` 23.9% of on-CPU samples of `--version`, 7.7% of `table_c` | The parser tree dropped at the end of `parse`, `src/cli.rs:370`–`385` | The tree is freed milliseconds before the process exits | **≈ −0.03 ms per invocation, estimate**; inside the noise floor for one invocation, ≈ 7 ms over the canonical loop | `S` |
| 12 | The production binary | The narrowed plans `Scope::Table`, `Scope::View` and `Scope::Routine`, `src/mariadb/catalogue/statements.rs:641`–`676`, are reached only from `#[cfg(test)]` code (`src/mariadb/catalogue.rs:590`–`633`); every production read is `Scope::Everything`, `src/cli/source.rs:274` | As stated | Unreachable in the shipped binary, by the design `src/cli/source.rs` documents | none at run time; its `.text` share cannot be isolated under fat LTO | `S` |

### Non-findings, stated so they are not rediscovered

- **The 272 `open` calls of a cached whole read are not waste.** `__open` is
  21–27% of the samples of every cached whole read, and the same render from one
  `--context` document is 3.4 ms faster than from the cache. The one file per
  object is `FR-CACHE-030`'s.
- **The whole-catalogue server read for a named object is not waste.**
  `table_d` issues the same 14 statements as `dump_d`; `src/cli/source.rs`
  documents why (`FR-CTX-006`, `FR-CTX-010`, `FR-CTX-023`, `FR-CACHE-007`), and
  66% of `dump_d` samples are spent waiting on the server in `kevent`.
- **The `66` with the server up reading the server and rewriting the cache is
  specified behaviour** (`FR-CDOC-008`, `FR-CACHE-007`). Its 2 641 ms are rows 1
  and 2 of the register, not a separate defect; measuring point 6 with the server
  down, as the measurement set does, hides them.
- **The render timer thread is not waste.** `bounded`, `src/cli/render.rs:595`,
  spawns a thread that only waits; it is what `FR-RND-033` requires.
- **Project discovery and configuration are not waste.** `Project::current` is
  21.4% of the on-CPU samples of `tlist`, and `tlist`, `cfglist`, `cfgget` and
  `tpath` sit 0.09–0.11 ms above `--version`.
- **Binary load is not waste.** The 0.25 ms between `nop` and `--exp-early` is
  the cost of a binary that carries the TLS stack the transport modes require.

### Confounders — read this before the tables

- **The host was not idle.** Four containers of other projects ran throughout
  (`sapoteca-solr-1`, `sapoteca-zookeeper-1`, `mongodb-mongo-1`,
  `mongodb-mongo-express-1`), iTerm2 used about 18% of a core and a root
  `osascript` about 14%, and the load average ranged from 1.56 to 4.22. The
  rotation spreads that load across every invocation; it does not remove it.
- **`samply` shares carry the profiler's overhead.** At 20 kHz an iteration of
  `rexample_c` took about 31 ms of CPU under the profiler against 21.5 ms of wall
  time without it. Only shares are read from the profiles, never durations.
- **The attribution variants are separate builds.** The layout alone moves a
  median by about 0.013 ms, which is below every established gain above by two
  orders of magnitude or more.
- **`dhat` counts the heap only**, through an instrumented build, and its figures
  are never used as time.
- **`cargo bloat` attribution under fat LTO is guesswork**, as the tool says:
  it lists `cc`, `proc_macro2` and `version_check`, which are build-time crates.
- **The server path crosses Docker Desktop's port proxy**, as the earlier
  entries of this file established for this host.
- **The benchmark entries authenticate as `root`**, because `seed-bench.sql`
  grants `tpl_reader` nothing on the benchmark schemas (`#224`).
- **`scripts/mariadb/up.sh` exited `2` on success** for a partial fixture
  (`#223`); the filtered `status.sh --quiet` gate was read instead, and answered
  `0` for every series used. `down.sh 10.11 11.4 11.8` likewise printed
  `still running after teardown: tpl-mariadb-12.3` for the server it had not been
  asked to remove.

### What was not measured — stated, not implied

- **Three of the four targets.** `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl` and `x86_64-apple-darwin` carry no figure here.
  Rows 1 and 2 in particular are platform-sensitive: `sync_all` is
  `F_FULLFSYNC` only on Apple platforms, and the cost of a syscall per token
  differs by kernel.
- **Every TLS mode.** All reads used `tls = "disabled"`.
- **`password_command`**, which no benchmark entry declares.
- **`render --view` and `render --routine`**, the `--pretty` forms, `cache clean`
  with an object flag, and `cfg database add`, `update` and `remove`. Each shares
  its costly path with a form that was measured.
- **The 10.11, 11.4 and 11.8 series for anything but the direct dump.** The
  cache and render paths do not reach the server once the cache is primed.

### Reproduction

```sh
S=/path/to/scratch          # any directory outside the repository
cargo build --release       # the binary of record

# 1. The fixture, through its harness only.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3
./scripts/mariadb/seed-bench.sh 12.3

# 2. The projects and the primed cache, with the functions benches/run.sh uses.
( FIXTURE_DIR=$PWD/scripts/mariadb; . benches/fixture.sh; FIXTURE_DIR=$PWD/scripts/mariadb
  fixture_inventory; fixture_address 12.3
  fixture_startup_project "$S/work" "$PWD/target/release/tpl"
  fixture_server_project  "$S/work" "$PWD/target/release/tpl" disabled
  fixture_prime           "$S/work" "$PWD/target/release/tpl"
  fixture_subjects        "$S/work" "$PWD/target/release/tpl" )
cp examples/rust-data-layer/templates/rust/*.jinja "$S/work/server/.tpl/templates/rust/"

# 3. Wall time: for each round r of 8, for each invocation in the table above,
#    rotated by r, from the project it names:
hyperfine -N --warmup 5 --runs 40 --export-json "$S/t/<label>.r<r>.json" "<binary> <args>"

# 4. A symbolised build for samply, from the same source, without touching Cargo.toml.
CARGO_TARGET_DIR="$S/prof" CARGO_PROFILE_RELEASE_DEBUG=true \
  CARGO_PROFILE_RELEASE_STRIP=false cargo build --release
samply record -s -r 20000 --iteration-count 30 --reuse-threads \
  --unstable-presymbolicate -o "$S/rexample_c.json.gz" -- \
  "$S/prof/release/tpl" -d bench_wl001 render example --table accrual

# 5. Allocation: a copy of the crate with `dhat = "0.3.3"`, `debug = 1`,
#    `strip = false`, `#[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;`
#    and `let _profiler = dhat::Profiler::builder().build();` first in `main`.

# 6. The cache-write experiment: copies of the crate with `handle.sync_all()?;`
#    removed from `store` (src/cache.rs:702), with the serialisation of `store`
#    written through `std::io::BufWriter::new(&mut handle)` and flushed before
#    the drop, and with both; rotate the four binaries over 8 rounds of
hyperfine -N --warmup 1 --runs 5 -n <variant> "<binary> -d bench_wl001 cache load" …
diff -r -x meta.json <cache written by the binary of record> <cache written by a variant>

# 7. The canonical loop, cache emptied before each run.
hyperfine -N --warmup 1 --runs 3 \
  --prepare "target/release/tpl -d bench_wl001 cache clean" \
  "benches/loop200.sh <binary> bench_wl001 example $S/work/wl001-tables.txt"

# 8. Peak resident memory and binary size.
/usr/bin/time -l target/release/tpl -d bench_wl001 render example --table accrual >/dev/null
CARGO_PROFILE_RELEASE_STRIP=false cargo bloat --release --crates -n 25 --target-dir "$S/bloat"

# 9. The fixture down, and nothing left.
./scripts/mariadb/down.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3   # non-zero
```

For the series comparison, `up.sh`, `seed-bench.sh` and `down.sh` take the
series names `10.11 11.4 11.8`, and a project holding one entry per port of
`scripts/mariadb/series.env` is read with `schema dump --direct --no-cache`
under the same rotation.

## 2026-09-22 — Three rows of the waste register applied: the cache write, `cache status`, and the render context

*Sprint 19, tasks `#232`, `#233` and `#234`. Target of record:
`aarch64-apple-darwin`. Server of record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`).
This entry records; it does not judge, per `BR-PERF-008`.*

### Outcome

Rows 1, 2, 4 and 7 of the waste register of the previous entry were applied, one
change per row, and each was measured against the binary of record in one
interleaved campaign:

| row | change | where |
|---|---|---|
| 1 | `store` no longer calls `sync_all` on each cache file | `src/cache.rs`, `store` |
| 2 | `store` serialises through a `BufWriter`, flushed explicitly, before the rename | `src/cache.rs`, `serialise` |
| 4 | `tpl cache status` counts the object files of each collection from the directory listing instead of reading them | `src/cache.rs`, `count` |
| 7 | `tpl render` leaks its `minijinja` context instead of freeing it, where the process exits as soon as the command returns | `src/cli/render.rs`, `produce`; `src/cli.rs`, `Ending` |

**A `WL-001` cache write fell from 2 450.3 ms to 61.1 ms (−97.5%), `tpl cache
status` from 5.515 ms to 2.071 ms (−62.4%), each cached render by 2.23 to
2.33 ms (−8.2% to −10.2%), and the canonical loop of 200 renders from
6 828.6 ms to 4 042.0 ms (−40.8%).** Every output compared was byte-identical,
with the one exception stated under *What changed that a caller can see*.

Rows 1 and 2 were measured together, as the register's `nosyncbuf` variant was:
they change one function, and neither was taken apart from the other here. The
separate attribution of each is the previous entry's.

### Workload

`WL-001` (200 tables, 30 views, 40 routines: 272 cache files) and `WL-003`,
loaded into `12.3` by `scripts/mariadb/seed-bench.sh`, which verified every
count. The `startup` and `server` projects were built by the functions of
`benches/fixture.sh`, the cache was primed by `fixture_prime`, and the five
templates of `examples/rust-data-layer/templates/rust/` were copied into the
`server` project, as in the previous entry.

| label | invocation, in the `server` project |
|---|---|
| `cload` | `tpl -d bench_wl001 cache load` |
| `cstatus` | `tpl -d bench_wl001 cache status` |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual`, served from the cache |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual`, served from the cache |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema`, served from the cache |
| loop | `benches/loop200.sh <binary> bench_wl001 example <the 200 WL-001 table names>`, the cache emptied before every run |

### Candidates

| arm | binary |
|---|---|
| `before` | the binary of record at `3e4daa8c`: 3 934 128 B, sha256 `16976d61065884ca3b5eac18e06efe8852410927eae7e0d232dd2eb12d33c927`, the same binary as the two previous entries |
| `before_twin` | the same file, measured as a second label: the A/A arm |
| `after` | the working tree with the three changes: 3 950 640 B, sha256 `5a5dd0c99784509df7442cb6ebcec8c4d003e011fd509df935b64d1331020df4` |

### Environment

As the previous entry, taken the same day on the same host: Apple M4, 10 cores,
32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0 `arm64`; `aarch64-apple-darwin`,
built and run natively; `rustc` 1.98.1 (48a229cea 2026-09-01); release profile
`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`,
`strip = true`; `hyperfine` 1.20.0 with `--shell=none`; `/usr/bin/time -l`; the
project fixture's `12.3` over Docker 29.8.1, `tls = "disabled"`, account `root`;
mains power, Low Power Mode off; load average 1.74 to 2.81. Taken 2026-09-22,
19:35Z to 19:48Z.

### Protocol

- **Three arms rotated inside one `hyperfine` call per round**, the order moved
  by one position each round, so that host drift spreads over all three.
- `cstatus` and the three renders: 8 rounds of 40 runs after 5 warmups per arm,
  320 samples each. `cload`: 8 rounds of 5 runs after 1 warmup, 40 samples
  each. The loop: 4 rounds of 3 runs after 1 warmup, 12 samples each, with
  `--prepare "tpl -d bench_wl001 cache clean"`.
- **Peak resident memory**: `/usr/bin/time -l`, median of 7 runs, taken apart
  from the timing campaign.
- **Output identity**, before any timing: each command was run by both binaries
  and stdout, stderr and the exit code compared with `cmp`; the two caches
  written by `cache load` were compared with `diff -r -x meta.json`, and the two
  `meta.json` files with `loaded_at` removed.

### The noise floor of the instrument on this host

The A/A arm, `before` against `before_twin`, inside every campaign:

| label | `before` | `before_twin` | difference |
|---|---|---|---|
| `cload` | 2 450.349 ms | 2 449.979 ms | 0.370 ms (0.02%) |
| `cstatus` | 5.515 ms | 5.525 ms | 0.010 ms (0.18%) |
| `rexample_c` | 21.800 ms | 21.708 ms | 0.092 ms (0.42%) |
| `rstruct_c` | 22.400 ms | 22.570 ms | 0.170 ms (0.76%) |
| `rschema_c` | 28.220 ms | 28.195 ms | 0.025 ms (0.09%) |
| loop | 6 828.557 ms | 6 821.296 ms | 7.261 ms (0.11%) |

**A difference below 0.2 ms on a render, or below 0.8% on any label, is not a
difference in this entry.** Every change below is more than ten times the
floor of its own campaign.

### Results

Medians; `rsd` is of the `after` arm.

| label | `before` | `after` | change | `after` rsd | peak RSS, before → after |
|---|---|---|---|---|---|
| `cload` | 2 450.349 ms | **61.147 ms** | −2 389.2 ms, −97.5% | 3.16% | — |
| `cstatus` | 5.515 ms | **2.071 ms** | −3.444 ms, −62.4% | 2.97% | 6.78 → 3.30 MiB |
| `rexample_c` | 21.800 ms | **19.566 ms** | −2.234 ms, −10.2% | 1.61% | 34.20 → 33.97 MiB |
| `rstruct_c` | 22.400 ms | **20.160 ms** | −2.240 ms, −10.0% | 1.52% | — |
| `rschema_c` | 28.220 ms | **25.895 ms** | −2.325 ms, −8.2% | 1.33% | 34.53 → 34.31 MiB |
| loop | 6 828.557 ms | **4 042.036 ms** | −2 786.5 ms, −40.8% | 0.35% | — |

The p90 moved with the median on every label: `cload` 2 486.5 → 64.6 ms,
`cstatus` 5.819 → 2.150 ms, `rexample_c` 22.230 → 20.053 ms, `rstruct_c`
22.893 → 20.638 ms, `rschema_c` 28.721 → 26.397 ms. No tail widened.

Against the previous entry's attribution experiments, which were separate
builds: `nosyncbuf` measured `cload` at 63.9 ms and the change measures
61.1 ms; `forget` measured −2.351 ms on `rexample_c` and the change measures
−2.234 ms. The loop gains −40.8% where the register estimated −33.6% for rows 1
and 2 and −7.2% for row 7, measured apart: the two gains add, because they are
in different parts of the loop — the first render alone pays the cache write,
and every render pays the context destructor.

`cstatus` now sits 0.35 ms above `tpl --version` as the previous entry measured
it — an observation across two campaigns, not a comparison — which is
what a command that discovers a project, reads its configuration and walks three
directories costs; the 3.2 MB it read are no longer read, which is the 3.5 MiB
of resident memory it no longer holds.

The render's peak resident memory barely moves, as it should: the context is
leaked after the peak has been reached, and the saving is time only.

### What changed that a caller can see

- **Output.** None of the following differs in a byte, stdout, stderr or exit
  code: `cache status` in `text`, `json` and `json --pretty` over `WL-001` and
  `WL-003`; the same with one table file truncated to 100 bytes, and with it
  emptied; the same with `meta.json` emptied (an empty cache for both);
  `render example` and `render rust/struct` with `--table accrual`, and
  `render rust/schema`, from the cache and with `--direct --no-cache`;
  `render example --context <dump>`; a `66` on an absent table; a `65` from a
  template that fails during evaluation after writing text; and a render whose
  stdout is closed after 10 bytes, which exits `0` from both. The 272 files and
  their `0600` mode written by `cache load` are identical, and so is
  `meta.json` apart from `loaded_at`.
- **The one difference: an object file that cannot be read.** Before, a file
  that was not UTF-8 or could not be opened made its whole collection report a
  count of `0` beside `whole: true`; now it is counted like any other file. With
  one table file replaced by the bytes `ff fe`, `before` reports `tables 0` and
  `after` reports `tables 200`. No requirement makes the count a validation:
  `FR-CACHE-034` asks for "the count of objects held".
- **Durability.** A cache file is no longer forced to disk before its rename, so
  a crash or a power loss can leave one empty or torn. Every read of the store
  decodes what it reads and answers a miss for such a file, which `FR-CACHE-033`
  requires; a unit test locks this in for `meta.json`.

### Confounders

- **The host was not idle**: the four containers of other projects named in the
  previous entry were still running. The rotation and the A/A arm bound their
  effect; they do not remove it.
- **`cload` and the loop write to an SSD**, and their figures depend on the
  file system as much as on `tpl`. Removing `sync_all` removes an
  `F_FULLFSYNC` on this platform; on Linux, `fsync` is cheaper and the gain of
  row 1 will be smaller.
- **Two comments were edited after the campaign**, in `src/cache.rs` and
  `src/cli/render.rs`. The binary rebuilt from the final tree has the same
  sha256 as the `after` arm, so the arm measured is the tree recorded.

### What was not measured

- **Three of the four targets**, as in the previous entry. Rows 1 and 2 are the
  platform-sensitive ones.
- **Allocation counts.** None of the three changes alters what is allocated:
  row 7 changes when memory is returned, not how much is taken.
- **`schema` reads**, which none of the changes reaches.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
# tpl-before: `cargo build --release` at 3e4daa8c; tpl-after: the same with the change.
cp <binary of record> "$S/tpl-before"; cp <binary with the change> "$S/tpl-after"

# 1. The fixture and the projects, exactly as in the previous entry's steps 1
#    and 2, then the output identity checks:
cd "$S/work/server"
for v in before after; do "$S/tpl-$v" -d bench_wl001 cache load; cp -R .tpl/.cache/bench_wl001 "$S/snap-$v"; done
diff -r -x meta.json "$S/snap-before" "$S/snap-after"
diff <(jq -S 'del(.loaded_at)' "$S/snap-before/meta.json") <(jq -S 'del(.loaded_at)' "$S/snap-after/meta.json")
cmp <("$S/tpl-before" -d bench_wl001 cache status --format json) <("$S/tpl-after" -d bench_wl001 cache status --format json)
cmp <("$S/tpl-before" -d bench_wl001 render rust/schema) <("$S/tpl-after" -d bench_wl001 render rust/schema)

# 2. Each label: for round r of 8, rotate the three arms by r.
hyperfine -N --warmup 5 --runs 40 --export-json "$S/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n before_twin "$S/tpl-before <args>" -n after "$S/tpl-after <args>"

# 3. The loop: 4 rounds, the same rotation.
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-before -d bench_wl001 cache clean" \
  -n before "benches/loop200.sh $S/tpl-before bench_wl001 example $S/work/wl001-tables.txt" …

# 4. The fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet   # non-zero
```

## 2026-09-22 — Four more rows of the waste register applied: `schema info`, the `schema tables` listing, the column decode, and the column fold

*Sprint 19, tasks `#235`, `#236`, `#237` and `#238`. Target of record:
`aarch64-apple-darwin`. Server of record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`).
This entry records; it does not judge, per `BR-PERF-008`.*

### Outcome

Rows 3, 5, 8 and 9 of the waste register were applied, one change per row, and
measured against the binary of record in one interleaved campaign:

| row | change | where |
|---|---|---|
| 3 | A cached `tpl schema info` reads `database.json` and counts the object files of each collection, instead of decoding all 272 files | `src/cache.rs`, `Cache::summary`, `Summary`; `src/cli/source.rs`, `serve_summary` |
| 5 | The cached `text` listing of `tpl schema tables` decodes each table file into a four-member view that counts the columns, instead of a full `TableDocument` | `src/cache.rs`, `Listed`, `Loaded::listing`; `src/cli/source.rs`, `serve_listing` |
| 8 | `Column` is decoded by a hand-written `Deserialize` that reads the same flat object without buffering it into serde's `Content` | `src/model/column.rs`, `decode` |
| 9 | Each table's column vector is sized once, from the run of rows that belongs to it | `src/mariadb/catalogue/fold.rs`, `columns`, `attach` |

**A cached `tpl schema info` fell from 12.282 ms to 2.100 ms (−82.9%) and
from 12.94 MiB to 3.41 MiB of peak resident memory; the cached `text` listing of
`tpl schema tables` from 11.180 ms to 7.578 ms (−32.2%); every other cached read
of a whole collection by 0.905 to 0.936 ms (−3.5% to −6.6%) and 12 094 210 B
in 20 247 blocks.** The server read moved by 0.016 ms, which is inside the
floor of its own campaign; its 583 reallocations at the fold site became 4.
Every output compared was byte-identical, with the exceptions stated under
*What changed that a caller can see*.

The labels were chosen so that each reaches one row: `schema info` decodes no
column after the change, and the `text` listing counts columns without
decoding them, so row 8 is measured on the four labels that still decode every
column, and row 9 on the one label that folds a server read.

### Workload

`WL-001` (200 tables, 2 400 columns, 30 views, 40 routines: 272 cache files)
and `WL-003`, loaded into `12.3` by `scripts/mariadb/seed-bench.sh`, which
verified every count. The `startup` and `server` projects were built by the
functions of `benches/fixture.sh`, the cache primed by `fixture_prime`, and the
five templates of `examples/rust-data-layer/templates/rust/` copied into the
`server` project, as in the previous entries.

| label | invocation, in the `server` project | row |
|---|---|---|
| `info_c` | `tpl -d bench_wl001 schema info`, served from the cache | 3 |
| `infojson_c` | `tpl -d bench_wl001 schema info --format json`, served from the cache | 3 |
| `tables_c` | `tpl -d bench_wl001 schema tables`, served from the cache | 5 |
| `tablesjson_c` | `tpl -d bench_wl001 schema tables --format json`, served from the cache | 8 |
| `dump_c` | `tpl -d bench_wl001 schema dump`, served from the cache | 8 |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual`, served from the cache | 8 |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema`, served from the cache | 8 |
| `dump_d` | `tpl -d bench_wl001 schema dump --direct --no-cache` | 9 |

### Candidates

| arm | binary |
|---|---|
| `before` | `cargo build --release` at `c724b243` into a separate target directory: 3 934 112 B, sha256 `a9468af6228bb5835dff014f08aac4e95badcc4b91a0c2406b36ac1b29cef7b7` |
| `before_twin` | the same file, measured as a second label: the A/A arm |
| `after` | the working tree with the four changes: 3 967 232 B, sha256 `604f832ba919244aa1a68d185a29523f90f6f6d68fe7b6aa1b2723514dbeb812` |

### Environment

Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0 `arm64`;
`aarch64-apple-darwin`, built and run natively; `rustc` 1.98.1 (48a229cea
2026-09-01); release profile `opt-level = 3`, `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`; `hyperfine` 1.20.0 with
`--shell=none`; `/usr/bin/time -l`; `dhat` 0.3.3; the project fixture's `12.3`
over Docker 29.5.2, `tls = "disabled"`, account `root`; mains power, Low Power
Mode off; load average 2.30 to 3.24. Timing taken 2026-09-22, 22:04Z to 22:06Z.

### Protocol

- **Three arms rotated inside one `hyperfine` call per round**, the order moved
  by one position each round, as in the previous entry.
- The seven cached labels: 8 rounds of 40 runs after 5 warmups per arm, 320
  samples each. `dump_d`: 8 rounds of 20 runs after 5 warmups, 160 samples
  each.
- **Peak resident memory**: `/usr/bin/time -l`, median of 7 runs, taken apart
  from the timing campaign.
- **Allocation**: one run per label and arm of a copy of the crate built with
  `dhat` as the global allocator, outside the repository, as the campaign
  entry describes; the copies differ from each other only by the four changes.
  The fold site is attributed by the frames of `dhat-heap.json` that pass
  through `catalogue::fold::columns` or `catalogue::fold::attach` below a
  vector growth or reservation.
- **Output identity**, before any timing: stdout, stderr and the exit code of
  25 invocations were captured from both binaries and compared with
  `diff -r`, listed under *What changed that a caller can see*.

### The noise floor of the instrument on this host

The A/A arm, `before` against `before_twin`, inside every campaign:

| label | `before` | `before_twin` | difference |
|---|---|---|---|
| `info_c` | 12.282 ms | 12.266 ms | 0.016 ms (0.13%) |
| `infojson_c` | 12.187 ms | 12.158 ms | 0.030 ms (0.24%) |
| `tables_c` | 11.180 ms | 11.152 ms | 0.027 ms (0.25%) |
| `tablesjson_c` | 13.750 ms | 13.760 ms | 0.011 ms (0.08%) |
| `dump_c` | 14.802 ms | 14.849 ms | 0.047 ms (0.32%) |
| `rexample_c` | 19.238 ms | 19.221 ms | 0.018 ms (0.09%) |
| `rschema_c` | 25.804 ms | 25.837 ms | 0.033 ms (0.13%) |
| `dump_d` | 26.251 ms | 26.235 ms | 0.015 ms (0.06%) |

**A difference below 0.05 ms, or below 0.35% on any label, is not a difference
in this entry.** Every change below is more than eighteen times the floor of
its own label, except `dump_d`'s.

### Results — wall time and memory

Medians; `rsd` is of the `after` arm.

| label | `before` | `after` | change | `after` rsd | p90, before → after | peak RSS, before → after |
|---|---|---|---|---|---|---|
| `info_c` | 12.282 ms | **2.100 ms** | −10.182 ms, −82.9% | 2.81% | 12.639 → 2.185 ms | 12.94 → 3.41 MiB |
| `infojson_c` | 12.187 ms | **2.068 ms** | −10.119 ms, −83.0% | 2.59% | 12.523 → 2.152 ms | — |
| `tables_c` | 11.180 ms | **7.578 ms** | −3.601 ms, −32.2% | 1.83% | 11.484 → 7.795 ms | 12.77 → 7.05 MiB |
| `tablesjson_c` | 13.750 ms | **12.841 ms** | −0.908 ms, −6.6% | 1.64% | 14.066 → 13.162 ms | 12.72 → 12.75 MiB |
| `dump_c` | 14.802 ms | **13.866 ms** | −0.936 ms, −6.3% | 1.71% | 15.189 → 14.223 ms | 12.91 → 13.02 MiB |
| `rexample_c` | 19.238 ms | **18.333 ms** | −0.905 ms, −4.7% | 1.52% | 19.687 → 18.775 ms | 33.95 → 34.08 MiB |
| `rschema_c` | 25.804 ms | **24.892 ms** | −0.912 ms, −3.5% | 1.75% | 26.209 → 25.377 ms | — |
| `dump_d` | 26.251 ms | 26.235 ms | −0.016 ms, −0.1% | 1.97% | 26.773 → 26.759 ms | 8.62 → 8.22 MiB |

No tail widened: the p90 moved with the median on every label.

### Results — allocation

`dhat`, one run each; total heap allocated over the run, and the heap at its
peak.

| label | total, before → after | blocks, before → after | at the peak, before → after |
|---|---|---|---|
| `info_c` | 23 945 727 → **657 342 B** | 48 926 → **2 640** | 8 224 224 → 341 750 B |
| `tables_c` | 23 709 972 → **3 852 728 B** | 47 997 → **3 306** | 8 139 312 → 3 197 830 B |
| `tablesjson_c` | 23 710 696 → **11 616 486 B** | 49 002 → **28 755** | 8 139 312 → 8 139 312 B |
| `dump_c` | 23 963 520 → **11 869 310 B** | 50 096 → **29 849** | 8 224 216 → 8 224 216 B |
| `rexample_c` | 42 396 382 → **30 302 172 B** | 121 723 → **101 476** | 26 742 242 → 26 742 242 B |
| `dump_d` | 6 671 800 → **5 560 886 B** | 15 388 → **15 009** | 4 167 658 → 3 883 242 B |

**The fold site, row 9.** Before, `fold::columns` grew each table's vector from
empty: 1 976 832 B in 583 reallocations. After, `fold::attach` makes one
allocation per table, exactly sized — 844 800 B in 200 blocks, which is 2 400
columns of 352 B — and the one vector reused across runs grows 4 times,
21 120 B. The 1 110 914 B and 379 blocks `dump_d` no longer allocates are this
site's, to 2 B.

### Observations, which are not verdicts

- **Row 3 met its estimate.** The register estimated ≈ −10.2 ms and ≈ −9.4 MiB
  for `schema info`; the change measures −10.182 ms and −9.53 MiB. The command
  now sits 0.1 ms above the 1.992 ms the campaign entry measured for
  `schema table` from the same cache — an observation across two campaigns, not
  a comparison — and the JSON form costs the same as the text form, as it
  should: both make the same three directory walks.
- **Row 5 took about half its estimate.** The register estimated ≈ −6.9 ms; the
  change measures −3.601 ms and −19.9 MB allocated. The listing still opens and
  reads the 200 table files, and the reduced view still parses every byte of
  them, skipping what it does not build; no profile of the `after` arm was
  taken, so the split of the remaining 7.6 ms is not attributed here.
- **Row 8 took under half its upper bound in time and all of it in
  allocation.** The register's upper bound was ≈ −1.9 to −2.1 ms per whole
  cached read; the change measures −0.905 to −0.936 ms on each of the four
  labels, the same figure on every one of them because each decodes the same
  2 400 columns. The allocations removed, 12 094 210 B in 20 247 blocks, are
  the register's figure for the site, to 2 B.
- **Row 9 is below the floor in time.** The register estimated ≈ −0.10 ms; the
  change measures −0.016 ms against an A/A difference of 0.015 ms, on a label
  whose time is mostly spent waiting on the server. What it establishes is the
  allocation figure above.

### What changed that a caller can see

- **Output.** None of the following differs in a byte, stdout, stderr or exit
  code, over the cache `WL-001` and `WL-003` primed by the binary of record:
  `schema info` in `text`, `json` and `json --pretty`; `schema tables` in
  `text`, with `--pattern acc%` and with a pattern matching nothing, in `json`
  and `json --pretty`; `schema table accrual` in `text` and `json`;
  `schema views`; `schema routines`; `schema dump` and `schema dump --pretty`;
  `render example --table accrual`, `render rust/struct --table accrual` and
  `render rust/schema`; the same `schema info`, `schema tables`, `schema dump`
  and `render rust/schema` with `--direct --no-cache`; the three renders with
  `--context` over a dump; `schema dump --direct --no-cache` over the fixture's
  `freight` schema on `10.11`, `11.4`, `11.8` and `12.3`. The 272 files and
  `meta.json`, apart from `loaded_at`, written by `cache load` are identical.
- **A malformed `--context` document is refused identically.** Ten documents
  with one column altered were refused by both binaries with the same `65` and
  the same `cause`: a type member of the wrong kind; the same followed, in the
  same column, by text that is not JSON (reported by position, as the derive
  reported it); an own member of the wrong kind followed by the same text
  (reported as a breach of the contract); a number out of range and a lone
  surrogate under a member no field names (reported by position); a type
  member and an own member given twice; `column_type` missing; a column that
  is an array; and a document cut in the middle of a column. The unit tests
  of `src/model/column.rs` decide 1 000-odd further documents against a copy
  of the derive.
- **The one difference: an object file only a fuller read refuses.** With one
  table file cut to 100 bytes, replaced by the bytes `ff fe`, or given a column
  whose `position` is a string, each invocation run once from a fresh copy of
  the cache, with the server up:

  | invocation | `before` | `after` |
  |---|---|---|
  | `schema info`, either form | a miss: `source` `server`, the file rewritten | a hit: `source` `cache`, the file left as it is |
  | `schema tables` in `text`, the file cut or not UTF-8 | a miss, the file rewritten | the same |
  | `schema tables` in `text`, the file valid JSON with a string `position` | a miss, the file rewritten | a hit, the file left as it is |
  | `schema tables --format json`, `schema dump` | a miss, the file rewritten | the same |

  The printed bytes are the same in every row but the JSON form of
  `schema info`, whose `source` differs. The commands that still read the
  file still miss on it and rewrite it, per `FR-CACHE-033`. `FR-SCH-031` puts
  the `text` form of `schema info` outside that requirement and gives its JSON
  form no count; no file this binary writes is one of these three.

### Confounders

- **The host was not idle**: the four containers of other projects named in the
  earlier entries were still running, and the load average was 2.30 to 3.24.
  The rotation and the A/A arm bound their effect; they do not remove it.
- **One binary carries the four changes.** The attribution of each label to one
  row rests on which code the label reaches, stated under *Workload*, not on
  four separate builds.
- **The binary of record was rebuilt**, at `c724b243`, into its own target
  directory. It is 3 934 112 B, where the previous entry's `after` arm was
  3 950 640 B; the difference was not investigated, and the arms here are
  compared only with each other.
- **`dhat` counts the heap only**, through an instrumented build, one run per
  label, and its figures are never used as time.
- **`dump_d` crosses Docker Desktop's port proxy**, and most of its time is the
  server's.

### What was not measured

- **Three of the four targets**, as in the previous entries.
- **Every TLS mode.** All server reads used `tls = "disabled"`.
- **CPU profiles.** No `samply` profile of either arm was taken; the shares the
  register cites are the campaign entry's.
- **The canonical loop of 200 renders.** Row 8 removes about 0.9 ms from each
  cached render; the loop was not re-run.
- **`cache load`.** No change touches the serialisation or the write path.
- **The 10.11, 11.4 and 11.8 series** for anything but output identity.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
# tpl-before: `cargo build --release --target-dir "$S/before-target"` at c724b243.
cp "$S/before-target/release/tpl" "$S/tpl-before"; cp target/release/tpl "$S/tpl-after"

# 1. The fixture and the projects, exactly as in the campaign entry's steps 1
#    and 2, built and primed with tpl-before. Then, from "$S/work/server", for
#    each invocation of the output identity list and each binary:
"$S/tpl-$v" <args> >"$S/out-$v/<label>.out" 2>"$S/out-$v/<label>.err"; echo $? >"$S/out-$v/<label>.code"
diff -r "$S/out-before" "$S/out-after"

# 2. Each label: for round r of 8, rotate the three arms by r.
hyperfine -N --warmup 5 --runs 40 --export-json "$S/t/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n before_twin "$S/tpl-before <args>" -n after "$S/tpl-after <args>"

# 3. Peak resident memory, median of 7.
/usr/bin/time -l "$S/tpl-after" -d bench_wl001 schema info >/dev/null

# 4. Allocation: two copies of the crate outside the repository — `git archive
#    c724b243` and the working tree — each with `dhat = "0.3.3"`, `debug = 1`,
#    `strip = false`, the `dhat` global allocator and profiler first in `main`;
#    one run of each per label, from "$S/work/server".

# 5. The fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet   # non-zero
```

## 2026-09-22 — Row 6 of the waste register applied: the render context converted where a template reads it

*Sprint 19, task `#239`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

Row 6 of the waste register was applied and measured against the binary of
record in one interleaved campaign:

| row | change | where |
|---|---|---|
| 6 | The `database` variable is a `minijinja` object over a copy of the document that converts a member the first time a template reads it, instead of a value converted whole before the template runs | `src/cli/render/context/lazy.rs`, `database`, `Database`, `Members`, `Table`; `src/cli/render/context.rs`, `assemble`; `src/model.rs`, `ToStatic`, with one implementation beside each model type |

**Each cached render fell by 5.60 to 5.75 ms (−23.1% to −30.4%) and from
34.2–34.5 MiB to 17.9–18.4 MiB of peak resident memory; the direct render by
5.58 ms (−18.1%) and from 29.19 to 13.23 MiB; and the canonical loop of 200
renders from 3 848.9 ms to 2 714.7 ms (−29.5%).** The allocations of the
conversion, 18.5 MB in 72 895 blocks, became 3.65 MB in 51 975 blocks for the
copy and at most 97 KB for what the three templates read. Every one of the
5 461 invocations of the equivalence harness was byte-identical, stdout,
stderr and exit code.

Three levels are converted on demand — the database object, its three
collections and each table — and everything below a table's field is converted
by `Value::from_serialize`, as before, the first time that field is read. Each
object caches what it converted, so a member read twice is one value.

The document is copied first because `minijinja` holds an object behind an
`Arc` with a `'static` bound and the document borrows the buffers of its source
— the cache files, the `--context` bytes, the rows of a server read. The copy,
`ToStatic`, copies every string and builds no map, key or `minijinja` value.
Extending the borrow without a copy takes `unsafe`, which the crate forbids, or
a self-referential dependency.

### Workload

`WL-001` and `WL-003`, loaded into `12.3` by `scripts/mariadb/seed-bench.sh`,
which verified every count. The `startup` and `server` projects were built by
the functions of `benches/fixture.sh` with the binary of record, the cache
primed by `fixture_prime`, and the five templates of
`examples/rust-data-layer/templates/rust/` copied into the `server` project, as
in the previous entries.

| label | invocation, in the `server` project |
|---|---|
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual`, served from the cache |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual`, served from the cache |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema`, served from the cache |
| `rexample_d` | `tpl -d bench_wl001 render example --table accrual --direct --no-cache` |
| loop | `benches/loop200.sh <binary> bench_wl001 example <the 200 WL-001 table names>`, the cache emptied before every run |

### Candidates

| arm | binary |
|---|---|
| `before` | `cargo build --release` at `765dd1c9` into a separate target directory: 3 967 216 B, sha256 `9cdfe6745bb81d43402f8597c6eef3006906fc7e3f64425d2a853b9223cc8e76` |
| `before_twin` | the same file, measured as a second label: the A/A arm |
| `after` | the working tree with the change: 4 000 432 B, sha256 `1e64fb90abaf18e2b0a4521dc28ce208958be8e810b765c27750d62025e2f914` |

### Environment

Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0 `arm64`;
`aarch64-apple-darwin`, built and run natively; `rustc` 1.98.1 (48a229cea
2026-09-01); release profile `opt-level = 3`, `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`; `hyperfine` 1.20.0 with
`--shell=none`; `/usr/bin/time -l`; `dhat` 0.3.3; `samply`; the project
fixture's `12.3` over Docker 29.8.1, `tls = "disabled"`, account `root`; mains
power, Low Power Mode off; load average 2.13 to 2.66. Timing taken 2026-09-22,
22:46Z to 22:50Z.

### Protocol

- **Three arms rotated inside one `hyperfine` call per round**, the order moved
  by one position each round, as in the previous entries.
- The three cached renders: 8 rounds of 40 runs after 5 warmups per arm, 320
  samples each. `rexample_d`: 8 rounds of 20 runs after 5 warmups, 160 samples
  each. The loop: 4 rounds of 3 runs after 1 warmup, 12 samples each, with
  `--prepare "tpl-before -d bench_wl001 cache clean"`.
- **Peak resident memory**: `/usr/bin/time -l`, median of 7 runs, taken apart
  from the timing campaign.
- **Allocation**: one run per label and arm of a copy of the crate built with
  `dhat` as the global allocator, outside the repository, as the campaign
  entry describes. The copy is attributed by the frames of `dhat-heap.json`
  that pass through a `to_static`, and the conversion by the frames that pass
  through `minijinja::value::serialize`.
- **CPU attribution of the `after` arm**: one `samply` profile of `rexample_c`,
  30 iterations at 20 kHz, from a symbolised build of the same tree. Only
  shares are read from it.
- **Output identity**, before any timing, by a harness run once per binary and
  compared with `diff -r`, described under *What changed that a caller can
  see*. The harness run twice with the binary of record was identical to
  itself.

### The noise floor of the instrument on this host

The A/A arm, `before` against `before_twin`, inside every campaign:

| label | `before` | `before_twin` | difference |
|---|---|---|---|
| `rexample_c` | 18.421 ms | 18.409 ms | 0.012 ms (0.07%) |
| `rstruct_c` | 19.060 ms | 19.030 ms | 0.031 ms (0.16%) |
| `rschema_c` | 24.873 ms | 24.788 ms | 0.086 ms (0.34%) |
| `rexample_d` | 30.756 ms | 30.746 ms | 0.011 ms (0.03%) |
| loop | 3 848.874 ms | 3 849.816 ms | 0.943 ms (0.02%) |

**A difference below 0.1 ms on a render, or below 0.35% on any label, is not a
difference in this entry.** Every change below is more than sixty times the
floor of its own label.

### Results — wall time and memory

Medians; `rsd` is of the `after` arm.

| label | `before` | `after` | change | `after` rsd | p90, before → after | peak RSS, before → after |
|---|---|---|---|---|---|---|
| `rexample_c` | 18.421 ms | **12.822 ms** | −5.599 ms, −30.4% | 1.81% | 18.751 → 13.188 ms | 34.16 → 17.94 MiB |
| `rstruct_c` | 19.060 ms | **13.378 ms** | −5.683 ms, −29.8% | 1.55% | 19.426 → 13.694 ms | 34.45 → 18.31 MiB |
| `rschema_c` | 24.873 ms | **19.125 ms** | −5.749 ms, −23.1% | 1.31% | 25.286 → 19.484 ms | 34.48 → 18.41 MiB |
| `rexample_d` | 30.756 ms | **25.177 ms** | −5.579 ms, −18.1% | 1.82% | 31.147 → 25.599 ms | 29.19 → 13.23 MiB |
| loop | 3 848.874 ms | **2 714.709 ms** | −1 134.2 ms, −29.5% | 0.24% | 3 857.2 → 2 725.3 ms | — |

No tail widened: the p90 moved with the median on every label.

### Results — allocation

`dhat`, one run each; total heap allocated over the run, and the heap at its
peak. `rexample_x` is `render example --table accrual --context <the WL-001
dump>`, taken for allocation only.

| label | total, before → after | blocks, before → after | at the peak, before → after |
|---|---|---|---|
| `rexample_c` | 30 302 174 → **15 497 427 B** | 101 476 → **80 807** | 26 742 242 → 11 958 855 B |
| `rstruct_c` | 31 761 531 → **16 984 726 B** | 108 588 → **88 000** | 26 893 630 → 12 111 022 B |
| `rschema_c` | 49 090 023 → **34 381 620 B** | 204 421 → **183 959** | 26 892 157 → 12 205 114 B |
| `rexample_d` | 23 991 914 → **9 208 525 B** | 86 630 → **67 159** | 22 419 188 → 7 635 801 B |
| `rexample_x` | 31 015 780 → **16 211 033 B** | 102 571 → **81 902** | 24 261 772 → 9 478 385 B |

**Where the difference is.** Before, the conversion allocated 18 518 034 B in
72 895 blocks on every render from the cache or a document, and 18 496 600 B
in 71 693 blocks on the direct one. After, the copy allocates 3 649 349 B in
51 975 blocks on every label, and the members converted on demand, outside the
copy, are 464 B in 2 blocks for `example`, 28 406 B in 83 blocks for
`rust/struct` and 96 808 B in 209 blocks for `rust/schema`: no template of the
worked example reads `foreign_keys` or `referenced_by`, which carry the
embedded tables of `FR-CTX-006` and `FR-CTX-010`. `schema dump` served from the
cache, which does not render, was 11 869 312 → 11 869 310 B in 29 849 blocks
in both arms.

### Observations, which are not verdicts

- **The change took about two thirds of the register's estimate in time and
  three quarters of it in memory.** The register estimated ≈ −8.9 ms and
  ≈ −21 MiB per render; the change measures −5.58 to −5.75 ms and −15.96 to
  −16.22 MiB. The estimate was derived from the campaign entry's profile, taken
  before row 7 removed the destructor of the converted context and before row
  8 changed the column's decoding; and the copy remains.
- **The gain is the same on every label, to 0.17 ms**, direct, cached or
  whole-database, because each render stopped converting the same document and
  started copying it; what differs between the labels is what they read, which
  is small in all three.
- **Where the time of `rexample_c` now goes**, from the profile of the `after`
  arm: 45.7% of samples decode the 272 cache files (`Loaded::document`), 35.0%
  open and read them (`Cache::look`), 8.6% copy the decoded document
  (`to_static`), 3.0% drop the decoded document, and 0.2% run the template.
  The first two are the cache's, and the campaign entry's non-findings keep the
  one file per object of `FR-CACHE-030`.
- **The loop fell by 1 134.2 ms, which is 5.67 ms per render**, within 0.07 ms
  of the per-render gain of `rexample_c`: 199 renders of each run are served
  from the cache, and the first, which reads the server and writes the cache,
  gains what `rexample_d` gains.

### What changed that a caller can see

- **Output.** None of the 5 461 invocations of the harness differs in a byte,
  stdout, stderr or exit code. The harness renders every template of the
  project — `example`, the 22 templates of the four worked examples
  (`examples/*/templates/`), and 28 probe templates written for this change —
  under a set of bindings and sources:

  | entry or document | sources | bindings |
  |---|---|---|
  | `freight` on `12.3` as `tpl_reader`, whose tables, views and routines carry the `restricted` marking | cache, `--direct --no-cache`, `--context` | none; four tables, two views, a function and a procedure |
  | `freight` on `12.3` as `root`, whose foreign keys and `referenced_by` are populated | the same | none; five tables, a view, a function and a procedure |
  | `freight` on `10.11`, `11.4`, `11.8` and the server without TLS, as `tpl_reader` | cache, `--direct --no-cache` | none; a table, a view, a procedure |
  | the `root` dump with one column's `table_name` set to a table that does not exist | `--context` | none; two tables |
  | the `root` dump with one table given a `restricted` marking | `--context <file>`, `--context -` | none; two tables |
  | `WL-001` | cache, `--direct --no-cache`, `--context` | none; two tables |
  | `WL-003` | the same | none |

  The probes cover the key order and the iteration of the database, of every
  table, view, routine and the server, with `length`, `items` and `dictsort`;
  `|json`, `|pprint` and `|string` of the whole database and of its parts;
  `sort` on whole objects and by attribute, `groupby`, `selectattr`,
  `rejectattr`, `map` with dotted attributes, `unique`, `batch`, `slice`,
  `first`, `last`, `reverse`, `sum`, slicing, negative indexing and repetition;
  `==`, `!=`, `<`, `in` and `is sameas` over the collections and their members,
  and between the bound object and the same object read through `database`;
  `is mapping`, `is sequence` and truthiness; `namespace` over a member;
  `debug`; the four lookups of `FR-ENV-020`; `primary_key` and `unique` over
  every column of every table, and over a column whose table is absent; and,
  one per template, a failure: an absent key at each level, an index past the
  end, `restricted` on a complete table, an attribute of a collection, a call,
  two method calls, five filters and a test given the wrong operand, a filter
  that refuses its arguments, and arithmetic on the database. The exit codes,
  the same for each binary, were 1 898 × `0`, 1 877 × `65`, 1 581 × `77`,
  102 × `70`, 2 × `66` and 1 × `64`. The 102 `70`s are the document with a
  hand-added `restricted` marking, with the marked table bound: the marking
  names `triggers`, a property this reader does not record, and both binaries
  report the internal invariant of `src/cli/schema/named.rs`. It is reported
  outside this task and was not changed.
- **Resident memory** is the one difference, and it is the one this change was
  made for.

### Confounders

- **The host was not idle**: the four containers of other projects named in the
  earlier entries were still running, and the load average was 2.13 to 2.66.
  The rotation and the A/A arm bound their effect; they do not remove it.
- **The `dhat` copies were taken one edit before the `after` arm**: `clippy`
  then asked for `Value::from_serialize(table.table_type)` in place of
  `Value::from_serialize(&table.table_type)`, which serialises the same value.
  They are never used as time.
- **A comment of `src/cli/render.rs` was edited after the campaign**, with its
  line count kept. The binary rebuilt from the final tree has the same sha256
  as the `after` arm, so the arm measured is the tree recorded.
- **The binary grew by 33 216 B (+0.84%)**, the objects and the copy; the
  `.text` share of each was not attributed.
- **`rexample_d` and the first render of each loop run cross Docker Desktop's
  port proxy**, and most of their time is the server's.

### What was not measured

- **Three of the four targets**, as in the previous entries.
- **Every TLS mode.** All server reads used `tls = "disabled"`.
- **A `samply` profile of the `before` arm**; the share the register cites is
  the campaign entry's.
- **A template that reads every member of the database.** Such a template pays
  the copy and then the conversion it paid before; the harness renders three —
  `probe/json`, `probe/pprint` and `probe/string` — for identity, and they were
  not timed.
- **The 10.11, 11.4 and 11.8 series** for anything but output identity.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
# tpl-before: `cargo build --release --target-dir "$S/before-target"` at 765dd1c9.
cp "$S/before-target/release/tpl" "$S/tpl-before"; cp target/release/tpl "$S/tpl-after"

# 1. The fixture: all five servers, because the harness reads every series.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/seed-bench.sh 12.3
#    The projects of the campaign entry's step 2, with tpl-before; a `freight`
#    project holding one `tpl_reader` entry per port of scripts/mariadb/series.env
#    and one `root` entry on 12.3; every template of examples/*/templates/ and
#    the probe templates copied into both; each cache primed with
#    `tpl-before cache load`, and each dump taken with `tpl-before schema dump`.

# 2. Output identity: the harness, once per binary, then
diff -r "$S/eq-before" "$S/eq-after"

# 3. Each label: for round r of 8, rotate the three arms by r.
hyperfine -N --warmup 5 --runs 40 --export-json "$S/t/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n before_twin "$S/tpl-before <args>" -n after "$S/tpl-after <args>"

# 4. The loop: 4 rounds, the same rotation.
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-before -d bench_wl001 cache clean" \
  -n before "benches/loop200.sh $S/tpl-before bench_wl001 example $S/work/wl001-tables.txt" …

# 5. Peak resident memory, median of 7; allocation, as in the previous entry,
#    over `git archive 765dd1c9` and the working tree.
/usr/bin/time -l "$S/tpl-after" -d bench_wl001 render example --table accrual >/dev/null

# 6. The fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet   # non-zero
```

## 2026-09-22 — Second waste-hunting pass: every path at `d140084`

*Sprint 19, task `#240`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

Every path of the campaign entry (`#231`) was profiled again at `d140084`, with
the same instruments, protocol and fixture, and with `#231`'s own binary rebuilt
and measured as an arm of the same rotated campaign. Nothing was changed:
`src/`, `Cargo.toml` and `Cargo.lock` are untouched, and every instrumented or
altered binary was built from a throwaway copy of the crate outside the
repository.

**Against `#231`, measured side by side: the canonical loop of 200 renders fell
from 6 827.1 ms to 2 734.7 ms (−59.9%), a cache write from 2 485.5 ms to
62.8 ms (−97.5%), each cached render by 8.74 to 8.80 ms (−31.3% to −40.3%),
`schema info` by 10.15 ms (−83.3%), and `cache status` by 3.49 ms (−63.4%).**
Start-up, help, version, discovery, configuration, the template commands, the
single-object `schema` reads and every server read are where `#231` left them,
to within 0.03 ms.

**One regression is established.** `#239`'s copy of the document is taken on
top of the conversion for a template that reads the whole database. Such a
render is 1.02 to 1.14 ms slower than at `d140084^` (+2.6% to +5.0%), and its
peak resident memory is 4.1 to 4.4 MiB higher. The heap at its peak is
3 637 277 B higher, which is the copy. A second whole-database template got
faster (−0.99 ms). Both remain faster than at `#231`.

**What remains is concentrated in the cached reads and the render.** Four rows
are above the noise floor by an order of magnitude or more:

- the compact `schema dump` and `schema tables --format json` decode 270 files
  and re-encode them into bytes that the files already hold verbatim;
- a render decodes every foreign key and every incoming key (with the table
  embedded in each) that the template never reads;
- the render copies the document to extend a borrow that the process could
  instead leak;
- the decoded document and its buffers are freed just before the process exits.

Rows 10, 11 and 12 of `#231`, and the vectors of `fold.rs` that grow from empty,
are at or below the floor, or have no run-time cost at all.

### Workload

`WL-001` (200 tables, 2 400 columns, 600 indexes, 180 foreign keys, 30 views,
40 routines: 272 cache files) and `WL-003`, loaded into `12.3` by
`scripts/mariadb/seed-bench.sh`, which verified every count. The `startup` and
`server` projects were built by the functions of `benches/fixture.sh` with the
binary of record. The cache was primed by `fixture_prime`, and the five templates
of `examples/rust-data-layer/templates/rust/` were copied into the `server`
project, as in the earlier entries. `--context` reads the compact dump of
`WL-001` from the cache.

Two probe templates were written into the scratch project, outside the
repository, for the whole-database question. They are not in the repository:

| template | source |
|---|---|
| `probe/whole_json` | `{{ database \| json }}` — the whole database through the `json` filter; 3 182 267 B of output |
| `probe/whole_walk` | a recursive macro that visits every key of every map and every item of every sequence under `database` and prints each leaf; 2 523 580 B of output |

`probe/whole_walk` is a stress, not a realistic template: 90.4% of its samples
are `minijinja`'s evaluation of the macro.

The labels are `#231`'s, plus the following. Every label runs in the `server`
project unless `#231` placed it in `startup`.

| label | invocation |
|---|---|
| `infojson_c`, `tablesjson_c`, `views_c`, `routines_c` | `schema info --format json`, `schema tables --format json`, `schema views`, `schema routines`, from the cache |
| `view_c`, `routine_c` | `schema view v_booking_line_summary`, `schema routine fn_consignment_hazard_count`, from the cache |
| `rwjson_c`, `rwwalk_c` | `render probe/whole_json`, `render probe/whole_walk`, from the cache |
| `rexample_x`, `rschema_x`, `rwjson_x` | `render example --table accrual`, `render rust/schema`, `render probe/whole_json`, each with `--context <dump>` and no `-d` |
| `rexample_d`, `rwjson_d` | `render example --table accrual`, `render probe/whole_json`, `--direct --no-cache` |
| `init` | `tpl init <dir>`, the directory removed by `--prepare` |
| `cfgset` | `cfg set core.database bench_wl001`, the value it already holds |
| `dbtest` | `cfg database test bench_wl001` |
| `cclean` | `-d bench_wl003 cache clean`, the cache reloaded by `--prepare` |

`--context` and `-d` cannot be given together (`64`): the first run of
`rexample_x` passed both, was refused in 1.73 ms, and was discarded and re-run
without `-d`.

### Candidates

| arm | binary |
|---|---|
| `b231` | `cargo build --release` of `git archive bc597dad` in a scratch directory: the source `#231` measured; 3 934 112 B, sha256 `dc2f9ac4efcc847829f9c8a60345562b080387ec08ebafca7bea27d0c3405767` |
| `prev` | the same, of `git archive d140084^` (`765dd1c9`): 3 967 216 B, sha256 `653ca3aa2e3a3e50595f6823417ad12e3f7ad3c112bb2448b9a0573ef1e9bad7` |
| `now` | `target/release/tpl` at `d140084`, the binary of record: 4 000 432 B, sha256 `1e64fb90abaf18e2b0a4521dc28ce208958be8e810b765c27750d62025e2f914` — the `after` arm of the previous entry, byte for byte |
| `now_twin` | the same file, measured as a fourth label: the A/A arm |

The attribution variants are copies of `git archive d140084`, one change each,
each built into its own target directory:

| variant | the one change |
|---|---|
| `ctl` | none: the source of `now` built from the scratch copy, the layout control for every variant below |
| `nop` | an empty `fn main() {}` under the identical release profile: the spawn floor |
| `xtree` | `drop(self::tree());` before `let mut tree = tree();` in `parse`, `src/cli.rs:370`: one extra build and drop of the parser tree on every invocation |
| `keeptree` | the tree forgotten instead of dropped once `interpret` returns on the accepted path of `parse`, `src/cli.rs:372` |
| `copy2` | a second `document.to_static()`, forgotten, before the one `lazy::database` takes, `src/cli/render/context/lazy.rs:106` |
| `forgetdoc` | the decoded document and its `Loaded` buffers forgotten after `present` in `serve_from`, `src/cli/source.rs:358`, and the document, model and catalogue forgotten after `present` in `read_through`, `src/cli/source.rs:447` |
| `dhat` | `dhat` 0.3.3 as the global allocator, `debug = 1`, `strip = false`; a second copy over `d140084^` for the whole-database question. Used for allocation only, never for time |

**Output identity**, before any timing. `b231`, `prev` and `now` were compared
over the cache `now` had primed, on stdout, stderr and exit code, for 15
invocations: `schema dump`; `schema tables` in both forms; `schema info` in both
forms; `schema table accrual`; `schema views`; `schema routines`;
`render example --table accrual`; `render rust/struct --table accrual`;
`render rust/schema`; the two probes; and `cache status` in both forms. All were
identical, as were the three `--context` renders. The five variants were
compared with `now` on 14 invocations, across help, version, cached and direct
reads, renders from all three sources, and a `64`. All were identical.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01); release profile `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` |
| Instruments | `hyperfine` 1.20.0 (`-N`, which is `--shell=none`); `samply` 0.13.1 at 20 kHz; `dhat` 0.3.3; `cargo-bloat` 0.12.1; `/usr/bin/time -l`; `jq` |
| Server | the project fixture's `12.3` alone during timing, `tls = "disabled"`, account `root`, Docker 29.5.2 |
| Power | mains (`AC Power`, battery 80%, not charging); Low Power Mode off |
| Load | 1.36 to 4.56 over the session; see *Confounders* |
| Taken | timing 2026-09-22, 23:04Z to 23:40Z; profiles and allocation to 23:48Z |

### Protocol

- **Wall time, as `#231` and the entries after it.** 8 rounds per label. The
  order of the labels is rotated by one position per round, and the four arms
  are rotated inside one `hyperfine` call per label per round. Start-up-class,
  cached and `--context` labels: 40 runs after 5 warmups per arm per round, 320
  samples. Direct labels, `init`, `cclean`, `dbtest`: 20 after 5 (or 2),
  160 samples. `cload` and `fail66`: 5 after 1, 40 samples. The loop: 4
  rounds of 3 runs after 1 warmup, 12 samples per arm, with
  `--prepare "tpl -d bench_wl001 cache clean"`.
- **Attribution experiments** were two further rotated campaigns of the same
  shape. One ran start-up and parser labels over `nop`, `ctl`, `xtree`,
  `keeptree` and `ctl_twin`. The other ran read and render labels over `ctl`,
  `forgetdoc`, `copy2`, `keeptree` and `ctl_twin`. Each variant is compared
  with `ctl`, not with `now`, so that the rebuild is not attributed to the
  change.
- **CPU attribution.** `samply record --rate 20000 --iteration-count N
  --reuse-threads --unstable-presymbolicate` over a symbolised build of the
  same source (`debug = true`, `strip = false`). `N` was 200 for start-up-class
  labels, 30 for renders and whole reads, 20 for direct reads, 10 for
  `cache load` and 5 for `probe/whole_walk`: 568 to 24 621 on-CPU samples per
  profile. Samples with a zero thread CPU delta are excluded. A share counts a
  sample once for every function on its stack. It becomes milliseconds only by
  multiplying it by the median of `now`, and such a figure is labelled an
  estimate and is an upper bound.
- **Allocation.** One run of the `dhat` copy per label. Sites are attributed by
  the frames of `dhat-heap.json` that a record passes through.
- **Peak resident memory.** `/usr/bin/time -l`, median of 7, for `b231` and
  `now`, and for `prev` on the two probes.
- **Binary size.** `cargo bloat --release --crates` and `size -m`, over the
  scratch copy of `d140084`.
- **The whole-database question.** `prev` against `now` inside the main
  campaign, and `dhat` over both.
- **The verbatim question.** Every object file of the `WL-001` cache was
  searched for, as a byte string, in the stdout of the compact
  `schema dump` served from the cache.

### The noise floor of the instrument on this host

The A/A arm, `now` against `now_twin`, inside the main campaign:

| class | labels | largest A/A difference |
|---|---|---|
| start-up, help, configuration, templates, single-object reads, `cache status`, `schema info` | 23 | 0.020 ms (`routines_c`); 0.019 ms on `version` |
| `init`, `cclean`, `dbtest` | 3 | 0.040 ms (`dbtest`, 1.0%) |
| cached whole reads and renders | 11 | 0.046 ms (`rschema_x`), 0.266 ms on the 118 ms `rwwalk_c` (0.23%) |
| direct reads and renders | 8 | 0.093 ms (`table_d`, 0.39%) |
| `cload`, `fail66` | 2 | 0.201 ms (0.32%) |
| the loop | 1 | 11.153 ms (0.41%) |

**A difference below 0.02 ms on a start-up-class invocation, below 0.1 ms on a
cached or direct read or render, or below 0.5% on the loop, is not a
difference in this entry.** Separate builds of the same source differ by more:
`keeptree`, whose change reaches none of the read paths, sat 0.003 to 0.321 ms
below `ctl` on them (0.32 ms on `rschema_c`). A variant's gain on a large label
is therefore established only where it is several times that spread.

The spawn floor, `nop`, is 1.283 to 1.290 ms. `#231` measured 1.356 ms.

### Results — every path, against `#231`

Medians. `#231 recorded` is the campaign entry's figure, from another campaign,
and is shown as an observation. `b231` is the same source measured in this
campaign, and the change is `now` against it. `rsd` is of `now`.

| label | `#231` recorded | `b231` | `prev` | `now` | change against `b231` | `now` rsd | peak RSS, `b231` → `now` |
|---|---|---|---|---|---|---|---|
| `version` | 1.722 ms | 1.693 ms | 1.683 ms | **1.671 ms** | −0.022 ms | 3.43% | 2.63 → 2.63 MiB |
| `help` | 1.829 ms | 1.799 ms | 1.807 ms | **1.804 ms** | +0.005 ms | 2.42% | 2.94 → 2.92 MiB |
| `helpjson` | 1.922 ms | 1.889 ms | 1.894 ms | **1.893 ms** | +0.004 ms | 2.49% | 3.06 → 3.03 MiB |
| `nodehelp` | 1.964 ms | 1.940 ms | 1.941 ms | **1.931 ms** | −0.009 ms | 2.30% | — |
| `tlist` | 1.826 ms | 1.786 ms | 1.792 ms | **1.791 ms** | +0.005 ms | 2.35% | 2.97 → 2.94 MiB |
| `tshow` | 1.840 ms | 1.804 ms | 1.810 ms | **1.803 ms** | −0.001 ms | 2.82% | — |
| `tcheck` | 1.938 ms | 1.916 ms | 1.923 ms | **1.918 ms** | +0.002 ms | 2.30% | — |
| `tpath` | 1.812 ms | 1.783 ms | 1.797 ms | **1.781 ms** | −0.002 ms | 2.59% | — |
| `cfglist` | 1.824 ms | 1.795 ms | 1.821 ms | **1.810 ms** | +0.015 ms | 2.79% | 3.16 → 3.17 MiB |
| `cfgget` | 1.829 ms | 1.815 ms | 1.812 ms | **1.796 ms** | −0.019 ms | 2.54% | — |
| `cfgset` | 1.944 ms | 1.926 ms | 1.940 ms | **1.934 ms** | +0.008 ms | 2.96% | — |
| `dblist` | 1.859 ms | 1.828 ms | 1.831 ms | **1.825 ms** | −0.003 ms | 2.42% | — |
| `dbshow` | 1.858 ms | 1.838 ms | 1.836 ms | **1.835 ms** | −0.003 ms | 2.72% | — |
| `dbtest` | 4.010 ms | 3.975 ms | 3.989 ms | **3.994 ms** | +0.019 ms | 3.09% | — |
| `init` | 2.102 ms | 2.094 ms | 2.109 ms | **2.102 ms** | +0.008 ms | 3.78% | — |
| `fail64` | 1.748 ms | 1.716 ms | 1.733 ms | **1.719 ms** | +0.003 ms | 3.03% | — |
| `fail66`, server up | 2 641.3 ms | 2 554.9 ms | 62.8 ms | **63.0 ms** | −2 491.9 ms, −97.5% | 1.63% | — |
| `cstatus` | 5.526 ms | 5.510 ms | 2.023 ms | **2.017 ms** | −3.493 ms, −63.4% | 2.56% | 6.78 → 3.31 MiB |
| `cload` | 2 656.2 ms | 2 485.5 ms | 62.6 ms | **62.8 ms** | −2 422.7 ms, −97.5% | 1.90% | — |
| `cclean` | 2.475 ms | 2.398 ms | 2.412 ms | **2.371 ms** | −0.027 ms | 4.69% | — |
| `info_c` | 12.199 ms | 12.179 ms | 2.042 ms | **2.033 ms** | −10.146 ms, −83.3% | 3.40% | 13.03 → 3.41 MiB |
| `infojson_c` | — | 12.245 ms | 2.058 ms | **2.041 ms** | −10.204 ms, −83.3% | 2.27% | — |
| `tables_c` | 11.249 ms | 11.287 ms | 7.647 ms | **7.635 ms** | −3.652 ms, −32.4% | 2.31% | 12.81 → 6.94 MiB |
| `tablesjson_c` | — | 13.814 ms | 12.900 ms | **12.934 ms** | −0.880 ms, −6.4% | 1.67% | 12.78 → 12.70 MiB |
| `table_c` | 1.992 ms | 1.951 ms | 1.961 ms | **1.954 ms** | +0.003 ms | 8.91% | 3.73 → 3.73 MiB |
| `table3_c` | 1.954 ms | 1.923 ms | 1.921 ms | **1.910 ms** | −0.013 ms | 2.38% | — |
| `view_c` | 1.929 ms | 1.885 ms | 1.893 ms | **1.881 ms** | −0.004 ms | 2.13% | — |
| `views_c` | — | 2.258 ms | 2.254 ms | **2.256 ms** | −0.002 ms | 2.21% | — |
| `routine_c` | 2.482 ms | 2.454 ms | 2.441 ms | **2.439 ms** | −0.015 ms | 2.38% | — |
| `routines_c` | — | 2.449 ms | 2.445 ms | **2.453 ms** | +0.004 ms | 2.53% | — |
| `dump_c` | 14.740 ms | 14.860 ms | 13.962 ms | **13.962 ms** | −0.898 ms, −6.0% | 1.53% | 13.11 → 12.94 MiB |
| `rstruct_c` | 22.071 ms | 22.273 ms | 19.176 ms | **13.474 ms** | −8.799 ms, −39.5% | 1.73% | 34.52 → 18.34 MiB |
| `rexample_c` | 21.547 ms | 21.658 ms | 18.577 ms | **12.919 ms** | −8.739 ms, −40.3% | 1.92% | 34.14 → 17.95 MiB |
| `rschema_c` | 27.923 ms | 28.019 ms | 25.054 ms | **19.258 ms** | −8.761 ms, −31.3% | 1.16% | 34.47 → 18.41 MiB |
| `rwjson_c` | — | 29.439 ms | 26.530 ms | **27.553 ms** | −1.886 ms, −6.4% | 0.91% | 42.08 → 46.50 MiB |
| `rwwalk_c` | — | 122.591 ms | 118.793 ms | **117.805 ms** | −4.786 ms, −3.9% | 1.06% | 43.39 → 47.60 MiB |
| `rexample_x` | 18.290 ms | 18.064 ms | 15.080 ms | **9.406 ms** | −8.658 ms, −47.9% | 1.36% | 33.75 → 17.41 MiB |
| `rschema_x` | — | 24.363 ms | 21.434 ms | **15.748 ms** | −8.615 ms, −35.4% | 1.11% | — |
| `rwjson_x` | — | 25.692 ms | 22.746 ms | **23.886 ms** | −1.806 ms, −7.0% | 0.96% | — |
| `dump_d` | 26.071 ms | 26.410 ms | 26.315 ms | **26.282 ms** | −0.128 ms | 1.28% | 8.66 → 8.17 MiB |
| `table_d` | 23.574 ms | 23.777 ms | 23.703 ms | **23.711 ms** | −0.066 ms | 1.72% | — |
| `table3_d` | 11.451 ms | 11.576 ms | 11.591 ms | **11.591 ms** | +0.015 ms | 4.08% | — |
| `info_d` | 23.500 ms | 23.737 ms | 23.754 ms | **23.704 ms** | −0.033 ms | 2.10% | — |
| `rexample_d` | — | 32.950 ms | 30.781 ms | **25.240 ms** | −7.710 ms, −23.4% | 1.14% | 29.56 → 13.20 MiB |
| `rstruct_d` | 33.401 ms | 33.547 ms | 31.450 ms | **25.855 ms** | −7.692 ms, −22.9% | 1.91% | — |
| `rschema_d` | 39.186 ms | 39.492 ms | 37.289 ms | **31.640 ms** | −7.852 ms, −19.9% | 1.02% | 29.95 → 13.67 MiB |
| `rwjson_d` | — | 40.860 ms | 38.724 ms | **39.739 ms** | −1.121 ms, −2.7% | 0.77% | — |
| loop | 6 834.9 ms | 6 827.1 ms | 3 865.4 ms | **2 734.7 ms** | −4 092.4 ms, −59.9% | 0.56% | — |

**Allocation at `d140084`**, `dhat`, one run each (total allocated, blocks, heap
at its peak):

| label | total | blocks | at the peak |
|---|---|---|---|
| `version` | 372 213 B | 1 074 | 260 180 B |
| `help` | 666 221 B | 2 396 | 260 171 B |
| `helpjson` | 738 377 B | 2 928 | 264 150 B |
| `nodehelp` | 1 402 962 B | 4 299 | 670 745 B |
| `tlist` | 446 609 B | 1 736 | 300 006 B |
| `cfglist` | 482 086 B | 1 812 | 312 885 B |
| `dblist` | 597 892 B | 2 286 | 379 305 B |
| `fail64` | 478 246 B | 1 343 | 329 988 B |
| `cstatus` | 604 437 B | 2 449 | 298 909 B |
| `info_c` | 657 320 B | 2 640 | 341 728 B |
| `tables_c` | 3 852 706 B | 3 306 | 3 197 830 B |
| `tablesjson_c` | 11 616 464 B | 28 755 | 8 139 312 B |
| `table_c` | 621 276 B | 2 269 | 342 359 B |
| `table3_c` | 588 354 B | 2 187 | 342 375 B |
| `view_c` | 541 342 B | 2 129 | 342 416 B |
| `routine_c` | 738 903 B | 2 930 | 342 445 B |
| `dump_c` | 11 869 288 B | 29 849 | 8 224 216 B |
| `rexample_c` | 15 497 405 B | 80 807 | 11 958 855 B |
| `rstruct_c` | 16 984 704 B | 88 000 | 12 111 022 B |
| `rschema_c` | 34 381 598 B | 183 959 | 12 205 114 B |
| `rwjson_c` | 58 694 492 B | 227 471 | 37 688 460 B |
| `rexample_x` | 16 211 011 B | 81 902 | 9 478 385 B |
| `dump_d` | 5 560 864 B | 15 009 | 3 883 242 B |
| `table3_d` | 819 257 B | 4 151 | 343 223 B |
| `rexample_d` | 9 208 503 B | 67 159 | 7 635 801 B |
| `cload` | 8 218 843 B | 18 466 | 3 951 958 B |

Every figure shared with the previous two entries agrees with them to 22 B, the
difference in the length of the source path the build embeds.

**Binary size.** `.text` is 3.1 MiB of the unstripped 4.9 MiB (`__text`
3 254 492 B). `cargo bloat` attributes 22.7% to `std`, 22.6% to `tpl`, 17.0% to
`minijinja`, 9.5% to `rustls`, 4.9% to `clap_builder`, 4.5% to `ring` and 4.1%
to `serde_json`, and the largest function is still
`minijinja::vm::Vm::eval_impl`, 36.3 KiB. The binary of record grew by
66 304 B (+1.7%) since `#231`, across the seven applied rows. No crate or
function stands out as removable.

### Results — attribution experiments

Medians of 320 samples (160 for direct labels). Each variant is compared with
`ctl`.

**Row 10 of `#231`, one build of the parser tree.** `xtree − ctl`, one extra
build and drop per invocation:

| label | `ctl` | `xtree` | one build | A/A |
|---|---|---|---|---|
| `version` | 1.682 ms | 1.732 ms | +0.050 ms | 0.014 ms |
| `help` | 1.803 ms | 1.843 ms | +0.040 ms | 0.008 ms |
| `helpjson` | 1.892 ms | 1.940 ms | +0.048 ms | 0.013 ms |
| `nodehelp` | 1.940 ms | 1.977 ms | +0.037 ms | 0.000 ms |
| `tlist` | 1.803 ms | 1.845 ms | +0.042 ms | 0.005 ms |
| `table_c` | 1.957 ms | 2.001 ms | +0.044 ms | 0.002 ms |
| `cstatus` | 2.018 ms | 2.054 ms | +0.036 ms | 0.003 ms |

**One build and drop costs 0.036 to 0.050 ms, established.** `--help`,
`tpl help` and `help --format json` build the tree twice
(`src/cli.rs:370`, then `src/cli/help.rs:282` or `:804`). A help form at a
leaf with a required operand builds it three times (`src/cli.rs:370`,
`src/cli.rs:474`, `src/cli/help.rs:282`). The profile agrees: building the tree
is 32.6% of the on-CPU samples of `--help` and 49.8% of `nodehelp`, against
32.6% of `--version`.

**Row 11 of `#231`, the drop of the parser tree.** `keeptree − ctl`: `version`
−0.014 ms, `help` −0.008, `helpjson` −0.005, `nodehelp` +0.008 (whose path
drops the tree on the refused parse, which `keeptree` does not reach), `tlist`
−0.022, `table_c` −0.023, `cstatus` −0.016. **The gain is −0.005 to
−0.023 ms, at the floor.** The profile would suggest far more: the tree's
destructor is 31.9% of the on-CPU samples of `--version`. But the on-CPU time
inside the process is about 0.14 ms of the 1.67 ms (`#231`: `tpl::run` is
0.144 ms), and the experiment decides.

**The copy of `#239`.** `copy2 − ctl`, one more `to_static` per render:

| label | `ctl` | `copy2` | one copy |
|---|---|---|---|
| `rexample_c` | 12.927 ms | 13.973 ms | +1.046 ms |
| `rschema_c` | 19.451 ms | 20.263 ms | +0.812 ms |
| `rwjson_c` | 27.688 ms | 28.546 ms | +0.858 ms |
| `rexample_x` | 9.558 ms | 10.499 ms | +0.941 ms |
| `rexample_d` | 25.164 ms | 26.243 ms | +1.079 ms |

On the labels that do not render, the same builds moved by −0.138 to +0.136 ms
(`dump_c`, `tablesjson_c`, `tables_c`, `dump_d`). **One copy costs 0.81 to
1.08 ms per render, established**, and allocates 3 649 349 B in 51 975 blocks,
all of it live at the peak. The profile of `rexample_c` gives it 8.87% of
samples, which is 1.15 ms at its median.

**The destructor of the decoded document.** `forgetdoc − ctl`:

| label | `ctl` | `forgetdoc` | change | `keeptree − ctl`, the same builds' spread |
|---|---|---|---|---|
| `rexample_c` | 12.927 ms | 12.513 ms | −0.414 ms | −0.003 ms |
| `rwjson_c` | 27.688 ms | 27.284 ms | −0.404 ms | −0.236 ms |
| `rschema_c` | 19.451 ms | 18.999 ms | −0.452 ms | −0.321 ms |
| `tablesjson_c` | 13.047 ms | 12.707 ms | −0.340 ms | −0.168 ms |
| `dump_c` | 13.912 ms | 13.694 ms | −0.218 ms | −0.004 ms |
| `dump_d` | 26.382 ms | 26.223 ms | −0.159 ms | −0.169 ms |
| `rexample_d` | 25.164 ms | 25.052 ms | −0.112 ms | −0.028 ms |
| `rexample_x`, not reached | 9.558 ms | 9.516 ms | −0.042 ms | −0.078 ms |
| `tables_c`, not reached | 7.625 ms | 7.609 ms | −0.016 ms | −0.035 ms |

**−0.41 ms on `rexample_c` and −0.22 ms on `dump_c` are established**, each
more than fifty times the builds' spread on that label. Elsewhere the change is
within two of that spread, and is an estimate of −0.1 to −0.45 ms. The two
labels whose path the variant does not reach moved inside it. The profile gives
the destructor of `DatabaseDocument` 3.08% of the samples of `rexample_c` and
2.27% of `dump_c`.

**The whole-database templates**, `now` against `prev`, in the main campaign:

| label | `prev` | `now` | change | peak RSS, `prev` → `now` |
|---|---|---|---|---|
| `rwjson_c` | 26.530 ms | 27.553 ms | +1.023 ms, +3.9% | 42.14 → 46.50 MiB |
| `rwjson_x` | 22.746 ms | 23.886 ms | +1.140 ms, +5.0% | — |
| `rwjson_d` | 38.724 ms | 39.739 ms | +1.015 ms, +2.6% | — |
| `rwwalk_c` | 118.793 ms | 117.805 ms | −0.988 ms, −0.8% | 43.50 → 47.60 MiB |

`dhat`, `prev` → `now`: `probe/whole_json` 55 057 223 → 58 694 492 B, 175 696 →
227 471 blocks, 34 051 183 → 37 688 460 B at the peak; `probe/whole_walk`
891 872 350 → 895 509 619 B, 1 594 799 → 1 646 574 blocks, 34 160 496 →
37 797 773 B at the peak. The differences at the peak, 3 637 277 B, are the copy
(3 649 349 B), held alive beside a conversion that is unchanged:
`minijinja::value::serialize` holds 18 433 240 B at the peak in `prev` and
18 322 320 B in `now`. **The regression `#239` expected is established: about
1 ms and 4 MiB for a template that converts the whole database**, which is the
copy measured above. It does not reach `probe/whole_walk`'s time, which fell;
that attribution is not established here.

### Where the remaining cost goes

Shares of on-CPU samples at `d140084`. A share is multiplied by the median
above only where it is labelled an estimate.

- **A cached render, `rexample_c` (12.919 ms).**
  - Opening and reading the 272 files is 35.7% (`open` alone 25.3%).
    `FR-CACHE-030` requires that layout, and `#231` classed it as not waste.
  - Decoding them is 45.9%. Of that, decoding each table's `foreign_keys` is
    14.6% and its `referenced_by` 15.1%, which together hold the tables
    embedded in each key (13.9%). They allocate 5 138 152 B in 17 603 blocks,
    and no template of the worked example reads them (`#239`).
  - The copy is 8.9%, established above at about 1 ms.
  - The destructor of the decoded document is 3.1%.
  - The template itself is 0.25%, and the parse of the invocation 1.0%.
- **`schema tables` in `text` form (7.635 ms), row 5's unattributed 7.6 ms.**
  - Reading the 200 files is 49.1%: `open` 34.0%, `read` 7.3%.
  - Parsing them is 44.8%. Of that, skipping the members the listing does not
    print is 31.7% (≈ 2.4 ms), and counting each table's columns is 12.2%
    (≈ 0.9 ms).
  - The rest is the process: the parser tree and project discovery.
  - The parse is what makes a torn or non-JSON file the miss of
    `FR-CACHE-033` for this listing, as `#236` required and a unit test holds;
    the reads are `FR-CACHE-030`'s. **None of it is vacuous.**
- **`schema dump` from the cache (13.962 ms).**
  - Reading is 33.3% and decoding 41.6%; writing the output is 20.6%.
  - **All 200 table files, 30 view files and 40 routine files appear verbatim,
    as byte strings, in the compact dump's stdout.** The decode builds each
    object only to serialise it back into the bytes it was read from. The same
    holds for the `tables` array of `schema tables --format json` (decode
    44.7%, output 22.2%).
- **The server reads.** `dump_d` is 40.6% waiting on the server, 19.2% the fold
  and 34.6% writing 3.2 MB of output. `rexample_d` spends 14.9% on the copy.
- **`cache load` (62.8 ms)** is system calls over the 272 files: `rename`
  29.2%, `open` 25.9%, `close` 10.0%, `write` 9.2%. They are the atomic
  per-object writes of `FR-CACHE-030` and `FR-CACHE-031`.
- **The vectors of `fold.rs` that grow from empty.** On `WL-001`, every one of
  them allocates once, at its first push, which grows it from empty to a
  capacity of four; none reallocates afterwards:
  - 200 index vectors, 83 200 B (`fold.rs:574`);
  - 180 foreign-key vectors, 120 960 B (`:735`);
  - 180 key-column vectors, 34 560 B (`:702`);
  - 85 incoming-key vectors, 88 320 B (`:726`);
  - 25 trigger, 20 routine-parameter and 10 check vectors, 50 400 B together.

  That is 377 440 B in 700 blocks, 6.8% of what `dump_d` allocates. Sizing
  them exactly would save no block and at most the unused capacity. Row 9
  removed 579 reallocations and measured −0.016 ms, inside the floor.

### The waste register, sorted by estimated gain over estimated effort

Effort is `#231`'s: `S` (one function, no interface change), `M` (a new code
path or a hand-written serde implementation), `L` (a new abstraction across
modules). "Established" means a single-variable experiment above measured the
gain; "estimate" means a profile share multiplied by a measured median, an upper
bound unless stated otherwise. `#231`'s rows 1–9 were applied and are not
repeated. Its rows 10–12 are re-stated here as rows 5–7.

| # | Path | Evidence | Cause in code | Why it is vacuous for the command | Estimated gain | Effort |
|---|---|---|---|---|---|---|
| 1 | The compact `schema dump` and `schema tables --format json`, served from the cache | 13.962 and 12.934 ms. Decode 41.6% and 44.7% of samples, output 20.6% and 22.2%. All 270 object files appear verbatim in the dump | `Loaded::document`, `src/cache.rs:173`, decodes every file into the model, and the `json` presentation, reached from `src/cli/schema.rs:538` and `:399`, serialises it again | The bytes printed for each object are the bytes of its file. The decode builds a structure only to reproduce them. The miss of `FR-CACHE-033` needs the file parsed, not built: `schema tables` in `text` form reads and parses the same 200 files in 7.635 ms. `--pretty` re-indents and still needs a parse | **≈ −5 to −6 ms per compact cached read (≈ −40%), estimate.** Bounded below by `tables_c`. Not in the canonical loop | `M`–`L`. A spliced writer that keeps `NFR-DET-002`'s order. It weakens the miss check from the full contract to JSON syntax, the behavioural question `#236` settled for the `text` listing |
| 2 | Every cached, `--context` and direct render | `foreign_keys` 14.6% and `referenced_by` 15.1% of the samples of `rexample_c`; 5 138 152 B in 17 603 blocks, and their share of the copy | `TableShape`'s `foreign_keys` and `referenced_by`, `src/model/document/shape.rs:198` and `:202`, decoded in full with the table embedded in each key (`FR-CTX-006`, `FR-CTX-010`), then copied by `to_static` (`:348`, `:349`) | `FR-RND-023` makes them reachable, not decoded. None of the worked-example templates reads them (`#239`) | **≈ −1.4 to −3.8 ms per render, estimate.** 3.8 ms is the share; 1.4 ms subtracts an upper bound for still parsing those bytes. ≈ −280 to −760 ms (−10% to −28%) over the canonical loop | `L`. A lazy field below the table level, which `#239` rejected on the ground that "a template that reads a table's columns reads them all". It does not answer for these two fields. Contract checking of an unread key moves to its first read |
| 3 | Every render | `copy2` +0.81 to +1.08 ms per copy; 3 649 349 B in 51 975 blocks at the peak; the whole-database regression of +1.02 to +1.14 ms and +4.1 to +4.4 MiB | `Arc::new(document.to_static())`, `src/cli/render/context/lazy.rs:106` | The copy exists only to make the document `'static` for `minijinja`'s `Arc` bound. Where the process exits when the command returns (`Ending::Process`), the source buffers can be leaked instead, in safe Rust (`Box::leak`, `String::leak`), and the document then borrows `'static` with no copy. The in-process tests keep the copy | **≈ −1.0 ms and −3.65 MB per render, established** as the cost of one copy. It also removes the whole-database regression. ≈ −200 ms (−7.3%) over the canonical loop | `M`. The loaded bytes, the `--context` bytes and the server rows each reach `produce` with a `'static` lifetime on the exiting path |
| 4 | Every cached whole read and every render from the cache | `forgetdoc` −0.414 ms on `rexample_c` and −0.218 ms on `dump_c`, established. −0.1 to −0.45 ms on four more labels, estimate. Destructor 3.1% of `rexample_c`'s samples | The decoded document and the `Loaded` buffers dropped at the end of `serve_from`, `src/cli/source.rs:356`–`359`; the document, model and catalogue at the end of `read_through`, `:446`–`447` | The process exits immediately after, as row 7 of `#231` argued for the render context, which is now leaked | **−0.41 ms per cached render, established; −0.2 to −0.45 ms per cached whole read.** ≈ −80 ms (−3.0%) over the canonical loop | `S`. The `Ending` of `src/cli.rs` carried into the reader |
| 5 (`#231` row 10) | `--help`, `tpl help`, `help --format json`, and a help form at a leaf with a required operand | One build and drop established at 0.036 to 0.050 ms. One extra build for the first three forms, two for the last | `render::text(&super::tree(), path)`, `src/cli/help.rs:282`; `let tree = super::tree()`, `src/cli/help.rs:804`; `waived(tree())`, `src/cli.rs:474` | The tree `parse` built at `src/cli.rs:370` describes the same nodes. It is dropped and rebuilt | **≈ −0.04 ms per help form (2.2% of `--help`), ≈ −0.08 ms at a leaf with a required operand (4.1%), established**, against an A/A of 0.000 to 0.014 ms on the same labels. Not in the canonical loop | `S`–`M`. The waived tree differs in one setting and is not the parse tree |
| 6 (`#231` row 11) | Every invocation | `keeptree` −0.005 to −0.023 ms, against an A/A of 0.000 to 0.014 ms | The parser tree dropped when `parse` returns, `src/cli.rs:370`–`372` | The tree is freed just before the process exits | **At the floor, established.** ≤ 5 ms (≤ 0.2%) over the canonical loop, below that loop's A/A of 11.2 ms | `S` |
| 7 (`#231` row 12) | The production binary | Unchanged. The narrowed plans `Scope::Table`, `Scope::View` and `Scope::Routine`, `src/mariadb/catalogue/statements.rs:641`–`676`, are built only by the test module of `src/mariadb/catalogue.rs` (`:590`–`633`, `:1555`–`1616`). The one production read is `Scope::Everything`, `src/cli/source.rs:275` | As stated | Unreachable in the shipped binary | **None at run time.** Its `.text` share cannot be isolated under fat LTO | `S` |
| 8 | Server reads | 700 first-push allocations, 377 440 B, and no reallocation after the first | `table.indexes.push`, `src/mariadb/catalogue/fold.rs:574`; `rule.key.columns.push`, `:702`; `table.referenced_by.push`, `:726`; `table.foreign_keys.push`, `:735` | `CLAUDE.md` asks for pre-sizing where the cardinality is known; the rows are in hand | **None in blocks, and in time below row 9's measured −0.016 ms, estimate** | `S` per site |

### The verdict: does the campaign stop?

The floor this entry measured is 0.000 to 0.020 ms on a start-up-class
invocation (1.7–2.5 ms), up to 0.046 ms on a cached render (9–28 ms), up to
0.093 ms on a direct read (11–40 ms), and 11.2 ms (0.41%) on the canonical loop
(2 734.7 ms). Against that, row by row:

| # | Gain against its own invocation | Against the canonical loop | Worth a further implementation round? |
|---|---|---|---|
| 1 | ≈ 40% of a compact cached dump or JSON listing; about 100 times its floor | none; the loop does not read it | **Yes, for the cached dump.** Settle the weaker miss check first, as `#236` did |
| 2 | ≈ 11% to 29% of a cached render | ≈ 10% to 28% | **Yes, after a single-variable experiment.** This is the largest remaining item in the loop, but it is only an estimate at effort `L` |
| 3 | ≈ 8% of a cached render, 10% of a `--context` render, 4% of a direct render; 20 or more times the floor | ≈ 7.3% | **Yes.** It is established and it removes the regression. Rows 3 and 4 share the same plumbing |
| 4 | ≈ 3.2% of a cached render; about 9 times the floor | ≈ 3.0% | **Yes, with row 3**, where it costs almost nothing more; on its own it is marginal |
| 5 | 2.2% to 4.1% of a help form; above the A/A of its labels (0.000–0.014 ms), but 0.04–0.08 ms | none | **No.** It is statistically real and practically insignificant |
| 6 | at the floor | inside the loop's floor | **No.** Insignificant |
| 7 | none | none | **No.** It has no run-time cost; any change would be for tidiness, not speed |
| 8 | below the floor | below the floor | **No.** Insignificant |

**The campaign stops for everything outside the cached reads and the render.**
Start-up, help, version, discovery, configuration, the `cfg` and `template`
commands, `init`, `cache status`, `cache clean`, `cache load`, the
single-object reads and the server reads: each remaining candidate there is at
or below the noise floor, or is a cost the specification requires. **The
campaign does not stop for the render and the cached whole reads.** Rows 3 and
4 are established, and together they are worth ≈ 1.4 ms per cached render and
≈ 10% of the canonical loop. Row 2 is the largest estimate left in the loop, and
it should be turned into an established figure before anything is implemented.
Row 1 is the largest single-invocation gain left, but the loop does not
contain it.

### Refuted hypotheses and non-findings, stated so they are not rediscovered

- **The parser tree's destructor is not worth 0.5 ms**, whatever its 31.9% share
  of `--version`'s on-CPU samples suggests when multiplied by the median. It is
  worth under 0.023 ms (row 6).
- **The index and foreign-key vectors that "grow from empty" do not
  reallocate** on `WL-001` (row 8).
- **`schema tables` in `text` form holds no vacuous cost.** Its 7.6 ms are the
  per-object reads of `FR-CACHE-030` and the parse that decides the miss of
  `FR-CACHE-033`.
- **`probe/whole_walk`'s 117.8 ms and 892 MB of allocation are the template
  engine's.** 90.4% of its samples are in `minijinja`'s evaluator running a
  recursive macro. It is a property of the template, not of `tpl`.
- **Nothing on the start-up, help, configuration or template paths moved since
  `#231`.** Every such label is within 0.03 ms of `b231` in this campaign.
- **The server reads are where `#231` left them**: `dump_d` −0.128 ms,
  `table_d` −0.066 ms, `info_d` −0.033 ms, `table3_d` +0.015 ms, each within
  about one A/A difference of its label.
- `#231`'s other non-findings stand, unchanged: the one file per object, the
  whole-catalogue server read for a named object, the render timer thread, and
  the binary's load cost.

### Confounders — read this before the tables

- **The host was not idle.** Four containers of other projects ran throughout
  (`sapoteca-solr-1`, `sapoteca-zookeeper-1`, `mongodb-mongo-1`,
  `mongodb-mongo-express-1`). Two `osascript` processes used 15–17% of a core
  each, the macOS aerial wallpaper extension about 11%, and iTerm2 about 4%.
  The load average was 1.36 to 3.15 during the main campaign and rose to 4.56
  in the tail of the variant builds, before the start-up experiment. The
  rotation and the A/A arms bound this; they do not remove it. `hyperfine`
  printed 108 warnings in the main campaign about a slow first run or
  statistical outliers.
- **`b231` and `prev` are rebuilds, not the binaries those entries measured.**
  Built from `git archive` in a scratch directory, they embed a different
  source path: `b231` is 3 934 112 B where `#231`'s binary was 3 934 128 B, and
  its sha256 differs. `now` is the recorded binary, byte for byte. `#231`'s own
  figures are therefore shown beside `b231` as observations from another
  campaign, and every change in the tables is against `b231`. The two agree
  within 0.016 to 0.339 ms on every label except the two that write the cache, where
  this campaign's `b231` is 86 to 171 ms faster.
- **Separate builds differ by up to 0.32 ms on large labels.** This is why every
  variant is compared with `ctl`, and why row 4 is established only where it
  clears that spread.
- **`samply` shares are of on-CPU samples inside the process**, carry the
  profiler's overhead, and do not see `exec` or loading. A share multiplied by a
  median overstates what a small in-process phase costs, as row 6 shows.
- **`dhat` counts the heap only**, through an instrumented build, one run per
  label, and is never used as time.
- **`cargo bloat` attribution under fat LTO is guesswork**, as the tool says.
- **`cload` and `fail66` rewrite the cache on every run, from every arm.** The
  272 files each arm writes were shown identical in the earlier entries, and the
  output identity check above was taken over the cache as `now` wrote it.
  `cclean` leaves `WL-003`'s cache empty; it was reloaded after the campaign.
- **The scratch directory already held files when this task started**, from an
  earlier run in the same session directory (a `prof/` target and
  `dhat/*.before.json`, `*.after.json`). None of them was used. Every figure here
  comes from a file this task wrote.
- **The whole-database probes are this task's own templates**, and
  `probe/whole_walk` is a stress, not a template anybody writes.
- **The server path crosses Docker Desktop's port proxy**, and the benchmark
  entries authenticate as `root` (`#224`). `up.sh 12.3` exited `2` on success
  (`#223`), and the filtered `status.sh --quiet 12.3` gate, which answered `0`,
  was read instead.

### What was not measured — stated, not implied

- **Three of the four targets**, as in every entry of this sprint. Row 3's copy
  and row 4's destructor are allocator work, and their cost will differ by
  libc.
- **Every TLS mode.** All server reads used `tls = "disabled"`.
- **Row 1 and row 2 as experiments.** Each needs a new decode path, which is an
  implementation rather than an instrument. Both are estimates.
- **The series `10.11`, `11.4` and `11.8`.** They were raised only for the test
  suite. `#231` showed that what `tpl` spends does not depend on the series.
- **`render --view`, `render --routine`, the `--pretty` forms, `cache clean`
  with an object flag, and `cfg database add`, `update` and `remove`**, as in
  `#231`.
- **The canonical loop under any variant.** The loop figures in the register
  are the per-render gains multiplied by 199 or 200.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
cargo build --release         # the binary of record; tpl-now
for c in bc597da d140084^; do mkdir -p "$S/src/$c"; git archive "$c" | tar -x -C "$S/src/$c"
  (cd "$S/src/$c" && cargo build --release --target-dir "$S/t-$c"); done   # tpl-b231, tpl-prev

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3   # read the gate, not up.sh
./scripts/mariadb/seed-bench.sh 12.3

# 2. The projects and the primed cache, as the campaign entry's step 2, with tpl-now;
#    the rust/ templates and the two probes copied into the server project; the
#    dump for --context taken with `tpl-now -d bench_wl001 schema dump`.

# 3. Wall time: for round r of 8, the labels rotated by r, the four arms rotated by r:
hyperfine -N -i --warmup 5 --runs 40 --export-json "$S/c1/<label>.r<r>.json" \
  -n b231 "$S/tpl-b231 <args>" -n prev "$S/tpl-prev <args>" \
  -n now "$S/tpl-now <args>" -n now_twin "$S/tpl-now <args>"
#    the loop, 4 rounds:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-now -d bench_wl001 cache clean" \
  -n now "benches/loop200.sh $S/tpl-now bench_wl001 example $S/work/wl001-tables.txt" …

# 4. The variants: copies of `git archive d140084`, one edit each, as the
#    Candidates table states, each built with its own --target-dir; then the
#    same rotation over (nop ctl xtree keeptree ctl_twin) and
#    (ctl forgetdoc copy2 keeptree ctl_twin).

# 5. Profiles, allocation, memory, size.
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false \
  cargo build --release --target-dir "$S/t-prof"                  # from the scratch copy
samply record -s -r 20000 --iteration-count 30 --reuse-threads --unstable-presymbolicate \
  -o "$S/prof/rexample_c.json.gz" -- "$S/t-prof/release/tpl" -d bench_wl001 render example --table accrual
/usr/bin/time -l "$S/tpl-now" -d bench_wl001 render probe/whole_json >/dev/null
CARGO_PROFILE_RELEASE_STRIP=false cargo bloat --release --crates -n 12 --target-dir "$S/bloat"

# 6. The verbatim check: every file under .tpl/.cache/bench_wl001/{tables,views,routines}
#    searched for as a byte string in the stdout of `tpl -d bench_wl001 schema dump`.

# 7. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet   # non-zero
```

## 2026-09-23 — Rows 3 and 4 of the second register applied; row 2 measured and stopped

*Sprint 19, tasks `#241` and `#242`. Target of record: `aarch64-apple-darwin`.
Server of record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry
records; it does not judge, per `BR-PERF-008`.*

### Outcome

Rows 3 and 4 of `#240`'s register were applied as one change (`#241`), and row
2 was measured by two single-variable experiments before anything was
implemented (`#242`):

| row | task | change | where |
|---|---|---|---|
| 3, 4 | `#241` | Under `Ending::Process` a read leaks the buffers its document borrows — the cache files, the `--context` bytes, the rows of a server read and the model folded from them — so the document borrows `'static` data: `tpl render` holds it without copying it, and no read frees it before the process exits. Under `Ending::Caller` (the in-process tests) the owners are dropped and the render copies, as before | `src/cli/source.rs`, `Served`, `leak`, `Reader::serve_from`, `Reader::read_through`; `src/cli/render.rs`, `from_document`, `produce`; `src/cli/render/context/lazy.rs`, `database`, `Held`; `Reader::new` and the `run` of `src/cli/schema.rs` and `src/cli/cache.rs`, which now carry the `Ending` |
| 2 | `#242` | **None.** Validating `foreign_keys` and `referenced_by` without building them measured −0.11 to −0.24 ms per cached render, at the spread of two separate builds, against an estimate of −1.4 to −3.8 ms | — |

**Each cached render fell by 1.52 to 1.55 ms (−8.0% to −12.4%), the `--context`
render by 1.28 ms (−13.8%), the direct render by 1.36 ms (−5.5%), and the
canonical loop of 200 renders from 2 672.2 ms to 2 362.9 ms (−11.6%).** Peak
resident memory fell by 4.19 to 4.48 MiB on every render. The regression
`#240` established for a template that reads the whole database is gone:
`probe/whole_json` is 1.15 to 1.28 ms faster on all three sources and its peak
is 4.2 to 4.4 MiB lower. A cached `schema dump` and `schema tables --format json` are
0.26 to 0.29 ms faster, and a direct `schema dump` 0.21 ms, which is the
destructor row 4 named. Every one of the 5 839 invocations of the equivalence
harness answered alike, byte for byte, on stdout, stderr and exit code, except
for the load time a cache write records.

**Row 2 is not worth implementing as the register framed it.** Its estimate
took the decode of the two members to be the cost. The experiments separate
the two parts: not decoding them at all saves 1.49 to 1.63 ms per cached render,
but validating them as the typed decode does, and building nothing, saves 0.11
to 0.24 ms. About 1.4 ms of the 1.6 ms is the validation, and `FR-CACHE-033`
requires it before the render starts, because a file that fails it is a miss.
What remains to be saved by deferring the construction is under a quarter of a
millisecond, before the cost of deferring it.

### Workload

`WL-001` (200 tables, 2 400 columns, 600 indexes, 180 foreign keys, 30 views,
40 routines: 272 cache files) and `WL-003`, loaded into `12.3` by
`scripts/mariadb/seed-bench.sh`, which verified every count. The `startup`,
`server` and `freight` projects and the two probe templates `probe/whole_json`
and `probe/whole_walk` are `#240`'s, in the same scratch directory outside the
repository. Both caches were reloaded with `tpl-before cache load` and were
byte-identical to what `#240` measured, `meta.json` apart. `--context` reads the
compact `WL-001` dump, which `tpl-before schema dump` reproduced byte for byte,
`source` apart.

The labels are `#240`'s:

| label | invocation, in the `server` project |
|---|---|
| `rexample_c`, `rstruct_c`, `rschema_c` | `render example --table accrual`, `render rust/struct --table accrual`, `render rust/schema`, from the cache |
| `rwjson_c`, `rwwalk_c` | `render probe/whole_json`, `render probe/whole_walk`, from the cache |
| `rexample_x`, `rwjson_x` | `render example --table accrual`, `render probe/whole_json`, each with `--context <dump>` |
| `rexample_d`, `rwjson_d` | `render example --table accrual`, `render probe/whole_json`, `--direct --no-cache` |
| `dump_c`, `tablesjson_c`, `dump_d` | `schema dump` and `schema tables --format json` from the cache; `schema dump --direct --no-cache` |
| `table_c`, `tables_c` | `schema table accrual` and the `text` form of `schema tables`, from the cache: the two controls. The first reaches the changed path with one table; the second is served by `Reader::serve_listing`, which the change does not reach |
| loop | `benches/loop200.sh <binary> bench_wl001 example <the 200 WL-001 table names>`, the cache emptied before every run |

### Candidates

| arm | binary |
|---|---|
| `before` | `cargo build --release` of `git archive ab091db` in a scratch directory: 4 000 416 B, sha256 `bee72ac35b48f21a8fc74bcddced4f7eafa573ecc2007bc712e19d88f96b20b5`. `ab091db` changes only this file against `d140084`, so this is `#240`'s `now` rebuilt |
| `before_twin` | the same file, measured as a third label: the A/A arm |
| `after` | `target/release/tpl` of the working tree with `#241`: 4 000 448 B, sha256 `53ed347097269c4a53a0fd830029101ea1ab7a0a7da528ecc8dbaa09d530ab16`, the binary the validation pipeline built |

The experiments of `#242` are copies of the `#241` tree, one change each, each
built into its own target directory from a path of the same length:

| variant | the one change |
|---|---|
| `ctl241` | none: the `#241` tree built from the scratch copy, 4 000 432 B, sha256 `5e7b921a926df61d7014d08568a45e957fa77668fd6f40260881302e6f2dfc7b`; the arm every variant is compared with |
| `skipfk` | `foreign_keys` and `referenced_by` of `TableShape` read through `serde::de::IgnoredAny` and left empty: the members are parsed as JSON and nothing is built or checked. It accepts documents `ctl241` refuses, and is an instrument only |
| `validfk` | the same two members read through private mirrors of `ForeignKeyShape`, `IncomingKey`, the embedded `TableShape` and `Index`, derived with the same field names, order and attributes, in which every collection is a sequence visitor that decodes each element with its real type and drops it; the members are then left empty. Every leaf — `Column`, `IndexColumn`, `ForeignKeyColumn`, `ReferentialAction`, `TableType`, `Trigger`, `CheckConstraint`, `Restricted` — is decoded by its own implementation |

**`validfk` refuses what `ctl241` refuses, at the same point.** Over 23
documents, each the `root` dump of `freight` with one fault inside or beside a
member of `foreign_keys` or `referenced_by` (listed under *What changed that a
caller can see*), `render probe/keys --context` exited with the same code and
wrote the same stdout and stderr, position included, under both binaries on
every refusal. `skipfk` accepted 13 of the 18 documents the other two refuse:
every fault but the four that are not JSON and the one outside the members.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01); release profile `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` |
| Instruments | `hyperfine` 1.20.0 (`-N`); `samply` 0.13.1 at 20 kHz; `dhat` 0.3.3; `/usr/bin/time -l` |
| Server | the project fixture's `12.3` alone during timing, `tls = "disabled"`, account `root`, Docker 29.8.1; all five servers for the equivalence harness and the test suite |
| Power | mains (`AC Power`, battery 80%, not charging) |
| Load | 1.69 to 3.12 over the session; see *Confounders* |
| Taken | 2026-09-23: the experiments 00:08Z to 00:14Z, the equivalence harness 00:17Z to 00:20Z, the main campaign and the loop 00:21Z to 00:28Z, then memory, allocation and profiles |

### Protocol

- **Wall time, as in the previous entries.** 8 rounds per label, the labels
  rotated by one position per round and the arms rotated inside one
  `hyperfine` call per label per round. Cached and `--context` labels: 40 runs
  after 5 warmups per arm per round, 320 samples. Direct labels and
  `rwwalk_c`: 20 runs after 5 (or 3) warmups, 160 samples. The loop: 4 rounds of
  3 runs after 1 warmup, 12 samples per arm, with
  `--prepare "tpl-before -d bench_wl001 cache clean"`.
- **The experiments of `#242`** were two campaigns of the same shape: the first
  over `ctl241`, `skipfk` and `ctl241_twin` on six render labels, the second
  over `ctl241`, `validfk`, `skipfk` and `ctl241_twin` on four. Each variant is
  compared with `ctl241`, never with `after`.
- **Peak resident memory.** `/usr/bin/time -l`, median of 7, both arms.
- **Allocation.** One run per label and arm of two copies built with `dhat` as
  the global allocator, `debug = 1`, `strip = false`: `#240`'s copy of
  `d140084` for `before` and the same edit over the `#241` tree for `after`.
- **CPU attribution of `after`.** `samply` over a symbolised build of the
  `#241` tree, 30 iterations of `rexample_c` and of `dump_c`: 9 064 and 11 404
  on-CPU samples. Only shares are read from it.
- **Output identity**, before any timing, by one harness run per binary over
  the same restored caches, compared with `diff -r`; see *What changed that a
  caller can see*.

### The noise floor of the instrument on this host

The A/A arm, `before` against `before_twin`, in the main campaign:

| class | labels | largest A/A difference |
|---|---|---|
| single-object and listing controls | 2 | 0.008 ms (`tables_c`) |
| cached and `--context` renders | 7 | 0.078 ms (`rschema_c`, 0.41%); 0.221 ms on the 118 ms `rwwalk_c` (0.19%) |
| cached whole reads | 2 | 0.073 ms (`dump_c`, 0.53%) |
| direct reads and renders | 3 | 0.051 ms (`rwjson_d`, 0.13%) |
| the loop | 1 | 5.142 ms (0.19%) |

In the experiments the A/A arm, `ctl241` against `ctl241_twin`, was 0.001 to
0.134 ms. **A difference below 0.1 ms on a render or read, or below 0.5% on the
loop, is not a difference in this entry**, and a difference between two
separate builds is not established below `#240`'s spread of such builds, 0.003
to 0.321 ms on these labels.

### Results — wall time and memory

Medians. `rsd` is of `after`.

| label | `before` | `after` | change | `after` rsd | p90, before → after | p99, before → after | peak RSS, before → after |
|---|---|---|---|---|---|---|---|
| `rexample_c` | 12.528 ms | **10.980 ms** | −1.548 ms, −12.4% | 1.69% | 12.815 → 11.218 ms | 13.051 → 11.599 ms | 18.00 → 13.68 MiB |
| `rstruct_c` | 13.181 ms | **11.649 ms** | −1.532 ms, −11.6% | 2.00% | 13.516 → 11.933 ms | 13.978 → 12.302 ms | 18.34 → 14.15 MiB |
| `rschema_c` | 18.959 ms | **17.435 ms** | −1.524 ms, −8.0% | 1.47% | 19.341 → 17.841 ms | 19.653 → 18.139 ms | 18.43 → 14.17 MiB |
| `rwjson_c` | 27.182 ms | **25.903 ms** | −1.280 ms, −4.7% | 0.84% | 27.522 → 26.172 ms | 27.800 → 26.526 ms | 46.45 → 42.25 MiB |
| `rwwalk_c` | 117.827 ms | **116.753 ms** | −1.074 ms, −0.9% | 1.26% | 119.317 → 118.667 ms | 120.673 → 121.751 ms | 47.65 → 43.37 MiB |
| `rexample_x` | 9.244 ms | **7.964 ms** | −1.280 ms, −13.8% | 1.29% | 9.440 → 8.078 ms | 9.779 → 8.320 ms | 17.56 → 13.34 MiB |
| `rwjson_x` | 23.589 ms | **22.442 ms** | −1.147 ms, −4.9% | 0.88% | 23.884 → 22.731 ms | 24.270 → 23.001 ms | 46.04 → 41.81 MiB |
| `rexample_d` | 24.601 ms | **23.242 ms** | −1.358 ms, −5.5% | 1.45% | 24.974 → 23.715 ms | 25.692 → 24.223 ms | 13.12 → 8.64 MiB |
| `rwjson_d` | 39.296 ms | **38.039 ms** | −1.257 ms, −3.2% | 0.89% | 39.660 → 38.412 ms | 40.417 → 38.706 ms | 41.65 → 37.21 MiB |
| `dump_c` | 13.708 ms | **13.420 ms** | −0.288 ms, −2.1% | 1.72% | 14.051 → 13.770 ms | 14.369 → 14.176 ms | 13.07 → 13.10 MiB |
| `tablesjson_c` | 12.580 ms | **12.320 ms** | −0.260 ms, −2.1% | 1.38% | 12.826 → 12.589 ms | 13.069 → 12.912 ms | 12.82 → 12.81 MiB |
| `dump_d` | 25.692 ms | **25.481 ms** | −0.211 ms, −0.8% | 1.19% | 26.136 → 25.827 ms | 26.379 → 26.187 ms | 8.31 → 8.25 MiB |
| `table_c`, control | 1.973 ms | **1.971 ms** | −0.002 ms | 2.39% | 2.032 → 2.040 ms | 2.109 → 2.125 ms | 3.76 → 3.71 MiB |
| `tables_c`, control | 7.530 ms | **7.537 ms** | +0.007 ms | 2.12% | 7.756 → 7.797 ms | 8.034 → 8.083 ms | 7.06 → 7.12 MiB |
| loop | 2 672.238 ms | **2 362.927 ms** | −309.3 ms, −11.6% | 0.79% | 2 685.4 → 2 371.7 ms | 2 691.9 → 2 403.8 ms | — |

The p90 and p99 moved with the median on every label but one: the p99 of
`rwwalk_c` rose by 1.08 ms, over 160 samples of a 118 ms invocation whose p90
fell by 0.65 ms.

### Results — allocation

`dhat`, one run each (total allocated, blocks, heap at its peak):

| label | total, before → after | blocks, before → after | at the peak, before → after |
|---|---|---|---|
| `rexample_c` | 15 497 405 → **11 848 150 B** | 80 807 → **28 833** | 11 958 855 → 8 309 594 B |
| `rstruct_c` | 16 984 704 → **13 335 465 B** | 88 000 → **36 026** | 12 111 022 → 8 461 777 B |
| `rschema_c` | 34 381 598 → **30 733 951 B** | 183 959 → **131 985** | 12 205 114 → 8 557 461 B |
| `rwjson_c` | 58 694 492 → **55 046 861 B** | 227 471 → **175 497** | 37 688 460 → 34 040 823 B |
| `rexample_x` | 16 211 011 → **12 561 860 B** | 81 902 → **29 928** | 9 478 385 → 8 244 432 B |
| `rexample_d` | 9 208 503 → **5 559 720 B** | 67 159 → **15 186** | 7 635 801 → 3 987 012 B |
| `dump_c` | 11 869 288 → 11 869 590 B | 29 849 → 29 851 | 8 224 216 → 8 224 312 B |
| `tablesjson_c` | 11 616 464 → 11 616 766 B | 28 755 → 28 757 | 8 139 312 → 8 139 408 B |
| `dump_d` | 5 560 864 → 5 561 638 B | 15 009 → 15 012 | 3 883 242 → 3 884 010 B |

**Every render allocates 3 647 631 to 3 649 255 B less, in 51 973 or 51 974
fewer blocks**, which is the copy `#240` measured (3 649 349 B in 51 975
blocks) less the boxes the leak takes.
The reads that do not render allocate 302 B more in two blocks, and 774 B in
three on the direct read: the boxes that move each leaked owner to the heap. The
heap at the end of the run is where the difference now shows: the leaked
document and its buffers are 8.06 to 8.31 MB still held when a cached read
exits, where `before` had freed all but 1 132 B on the reads that do not render.

### Results — the experiments of `#242`

Medians of 320 samples (160 for `rexample_d`). Each variant against `ctl241`:

| label | `ctl241` | `skipfk` | change | `validfk` | change | A/A |
|---|---|---|---|---|---|---|
| `rexample_c` | 11.042 ms | 9.422 ms | −1.621 ms, −14.7% | 10.803 ms | **−0.240 ms, −2.2%** | 0.013 ms |
| `rstruct_c` | 11.625 ms | 10.067 ms | −1.558 ms, −13.4% | 11.385 ms | **−0.240 ms, −2.1%** | 0.017 ms |
| `rschema_c` | 17.281 ms | 15.793 ms | −1.488 ms, −8.6% | 17.176 ms | **−0.106 ms, −0.6%** | 0.065 ms |
| `rexample_x` | 8.008 ms | 6.131 ms | −1.877 ms, −23.4% | 7.467 ms | −0.541 ms, −6.8% | 0.018 ms |

The first campaign agreed with the second to 0.06 ms on every shared figure,
and measured `skipfk` at −0.150 ms on `rexample_d`, which no cache file reaches,
and at −11.0 ms on `rwjson_c`, whose output `skipfk` shortens; neither is a
figure of row 2.

- **The upper bound of row 2 is −1.5 to −1.6 ms per cached render**, not the
  register's −1.4 to −3.8 ms: `skipfk` still parses the members, and the copy
  and the destructor that the register's share included were removed by `#241`.
- **The part that deferring the construction could save is −0.11 to −0.24 ms**:
  `validfk` performs every check the typed decode performs and builds nothing.
  That is 2% of a cached render and at the spread of two separate builds, and
  it is an upper bound: a deferred member must also be built on first read,
  from bytes the document would have to keep.
- **The rest, about 1.4 ms, is the validation**, and above all the decode of the
  column lists of the 360 tables embedded in those members.
- **`rexample_x` gains more, −0.54 ms**, because the `--context` path reads each
  embedded table back only for its name, and `validfk` also spares the build of
  the embedding that the model no longer carries. It is not a figure a lazy
  member would reach.

### Where the remaining cost goes

Shares of on-CPU samples of `after`; inclusive, so they overlap.

- **A cached render, `rexample_c` (10.980 ms).** Reading the 272 files is 40.8%
  (`open` 28.8%), decoding them 53.7%. Of the decode, the column lists are
  40.6%, the tables embedded in the two reference members 32.4%, and the two
  members themselves 34.3%. The template and its context are 2.4%. The copy and
  the destructor of the document no longer appear.
- **`schema dump` from the cache, `dump_c` (13.420 ms).** Reading is 33.6% and
  decoding 43.1%; the serialisation of the output is the largest self share,
  10.6%. The destructor no longer appears; row 1 of `#240` stands as it was.

### The register after this entry

`#240`'s rows 5 to 8 stand, at or below the floor, and are not repeated.

| # | Path | State after this entry | Gain left | Effort |
|---|---|---|---|---|
| 1 | The compact `schema dump` and `schema tables --format json`, from the cache | Unchanged. The bytes of every object file are printed verbatim | ≈ −5 to −6 ms per compact cached read, estimate, as `#240` stated; not in the loop | `M`–`L` |
| 2 | Every cached render | **Measured and stopped.** Deferring the construction is worth −0.11 to −0.24 ms. The 1.4 ms of validation is the miss check of `FR-CACHE-033`, and only a weaker check could remove it — JSON syntax alone, as `#236` settled for the `text` listing — which is a question for the specification, not an optimisation | ≤ −0.24 ms as specified; ≤ −1.6 ms (≈ −320 ms over the loop) under a syntax-only miss check | `L` |
| 2a | The `--context` render | New, from row 2. `crate::model::document::read` decodes every embedded table in full and keeps only its name | ≤ −0.54 ms per `--context` render, estimate; not in the loop | `M` |
| 3 | Every render | **Applied** (`#241`): −1.07 to −1.55 ms and −3.65 MB per render, the destructor of row 4 included for a cached render | — | — |
| 4 | Every cached whole read and cached render | **Applied** (`#241`): −0.26 to −0.29 ms per cached whole read, the render's share inside row 3's figure | — | — |

### What changed that a caller can see

- **Output.** Nothing. The harness was run once per binary over the same
  restored caches, 5 839 invocations each, and `diff -r` of the two trees found
  14 differing files: 8 `cache status` outputs and 6 copies of the `meta.json` a
  miss rewrote, each carrying the `loaded_at` a cache write records. With
  `loaded_at` set aside, every line matched.
  The exit codes, the same under both binaries, were 2 313 × `0`, 1 672 × `65`,
  1 725 × `77`, 110 × `70`, 18 × `66` and 1 × `64`. The harness has three parts:
  - **Every render** of `#239`'s harness — `example`, the 22 templates of
    `examples/*/templates/`, `#239`'s 28 probes, `#240`'s two whole-database
    probes, and two new probes that read every member of `foreign_keys` and
    `referenced_by`, embedded tables included, and the bound table's — over
    `freight` on `12.3` as `tpl_reader` and as `root`, on `10.11`, `11.4`,
    `11.8` and the server without TLS as `tpl_reader`, `WL-001` and `WL-003`,
    from the cache, `--direct --no-cache`, `--context <file>` and
    `--context -`: 5 349 invocations. The 110 `70`s are `#239`'s document with
    a hand-added `restricted` marking, now reached by eight more probes.
  - **Every `schema` read** in both forms and with `--pretty`, including a
    table that does not exist, from the cache and `--direct --no-cache`, over
    the same eight entries; `cache status --format json`; and, last, a
    `schema dump --direct` that rewrites the store, `cache load` and
    `cache load --table`, with the store compared afterwards: 259 invocations.
  - **Corrupted input**, 231 invocations. The `freight` table file of
    `consignment`, which carries three outgoing and three incoming keys, was
    replaced by each of 23 variants, and a view file (given an unknown key, and
    truncated), `database.json` (without `server`) and `meta.json` (an unknown
    format) by four more, each followed by six commands — `render
    probe/keys`, `render probe/fk`, `render example --table consignment`,
    `render probe/fkbound --table consignment`, `schema dump` and
    `schema table consignment --format json` — and the file compared
    afterwards. The same 23 faults were applied to the `root` dump, rendered
    with `--context <file>` twice and with `--context -` once. The faults,
    inside a member of `foreign_keys` or `referenced_by` unless stated: a
    column of an embedded table given a string position; a key's `name`
    removed; a rule given a number; an embedded index replaced by a number; an
    embedded table given an empty `restricted`; `foreign_keys` set to `null`;
    a key's `columns` given an object; an embedded column's `nullable` given a
    string; an incoming key's `key` removed; its `referenced_table` given a
    number; an embedded table's `foreign_keys` given a number; `referenced_by`
    given an object; an unknown key holding a number, and one holding an
    object; a syntax error; a duplicated `name`; the file truncated inside the
    member; an unknown key holding a lone surrogate, and one holding `1e999`; a
    syntax error in `referenced_by`; a fault of type followed by a syntax error
    later in the file; and, outside the members, a column's `nullable` given a
    string and an optional `engine` removed. Every corrupted cache file was a
    miss under both binaries, answered from the server and rewritten byte for
    byte as it was before the fault, except the six the decode accepts — the
    five unknown keys and the removed optional — which both binaries served
    from the file as it was. Every malformed document was the same `65`,
    with the same position where the text is not JSON, or the same render.
- **Resident memory** falls on every render, by the copy.
- **In-process callers** — the unit and integration tests, under
  `Ending::Caller` — free every owner and copy the document, as before.

### Confounders

- **The host was not idle.** The four containers of other projects named in
  `#240` ran throughout, and the load average was 1.69 to 3.12. The rotation
  and the A/A arms bound their effect; they do not remove it. `hyperfine`
  printed 30 warnings about statistical outliers or a slow first run.
- **`before` and `after` are separate builds from different paths**, a scratch
  directory and the repository: `#240` measured up to 0.32 ms between two such
  builds on these labels. Every render figure is more than four times that,
  and the two controls moved by 0.002 and 0.007 ms. The cached whole reads,
  −0.26 to −0.29 ms, are at that spread, and are consistent with `#240`'s
  `forgetdoc` experiment (−0.22 to −0.34 ms) rather than established by this
  campaign.
- **The experiments' `ctl241` is not `after`**: it is the same tree built from a
  scratch copy, so that each variant differs from its control by one change and
  not also by the path. It measured 11.042 to 11.086 ms on `rexample_c` where
  `after` measured 10.980 ms in the main campaign.
- **`rwjson_c` of `before` is `#240`'s `now` rebuilt, not `prev`.** The
  regression that entry measured against `prev` (26.530 ms) is gone in the sense
  that `after` is 25.903 ms here, but the two figures come from two campaigns.
- **`dhat` counts the heap only**, through an instrumented build, one run per
  label, and is never used as time. The `before` copy is `#240`'s, of
  `d140084`, whose `src/` is `ab091db`'s.
- **The `samply` shares carry the profiler's overhead and overlap**; the
  experiments, not the shares, decide row 2. The shares put the two reference
  members at 34.3% of `rexample_c` (≈ 3.8 ms) where skipping them entirely saves
  1.6 ms.
- **Timing ran against `12.3` alone.** The other four servers were brought
  down through `scripts/mariadb/down.sh` for the campaigns and brought back up
  through `scripts/mariadb/up.sh` for the harness and the test suite, whose
  gate, `status.sh --quiet`, answered `0` each time.

### What was not measured

- **Three of the four targets.** A leak and a copy are allocator work, and
  their cost differs by libc.
- **Every TLS mode.** All server reads used `tls = "disabled"`.
- **The series `10.11`, `11.4` and `11.8`** for anything but output identity.
- **`cache load`**, which the change does not reach: it reads the server and
  writes the store without presenting a document.
- **A lazy implementation of row 2.** `validfk` bounds it from above and was not
  taken further; the syntax-only miss check was not measured apart from
  `skipfk`, which also skips the members' construction.
- **Row 2a as an experiment**; its figure is `validfk`'s on `rexample_x`, an
  upper bound.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
mkdir -p "$S/src/ab091db"; git archive ab091db | tar -x -C "$S/src/ab091db"
(cd "$S/src/ab091db" && cargo build --release --target-dir "$S/t-ab091db")
cp "$S/t-ab091db/release/tpl" "$S/tpl-before"; cargo build --release; cp target/release/tpl "$S/tpl-after"

# 1. The fixture, through its harness only: all five servers for the harness.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet      # read the gate, not up.sh
./scripts/mariadb/seed-bench.sh 12.3
#    #240's projects and probes; every cache reloaded with `tpl-before cache load`
#    and kept as a snapshot, restored before each binary's harness run.

# 2. Output identity: render, schema and corruption harnesses per binary, then
diff -r "$S/q241/before" "$S/q241/after"

# 3. Timing against 12.3 alone.
./scripts/mariadb/down.sh 10.11 11.4 11.8 notls; ./scripts/mariadb/status.sh --quiet 12.3
#    for round r of 8, the labels rotated by r, the three arms rotated by r:
hyperfine -N -i --warmup 5 --runs 40 --export-json "$S/m241/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n after "$S/tpl-after <args>" -n before_twin "$S/tpl-before <args>"
#    the loop, 4 rounds:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-before -d bench_wl001 cache clean" \
  -n before "benches/loop200.sh $S/tpl-before bench_wl001 example $S/work/wl001-tables.txt" …

# 4. The experiments: copies of the #241 tree, one edit each, as the Candidates
#    table states, each with its own --target-dir; the same rotation over
#    (ctl241 skipfk ctl241_twin) and (ctl241 validfk skipfk ctl241_twin).

# 5. Memory, allocation and profiles.
/usr/bin/time -l "$S/tpl-after" -d bench_wl001 render example --table accrual >/dev/null
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false \
  cargo build --release --target-dir "$S/t-prof241"                # from the scratch copy
samply record -s -r 20000 --iteration-count 30 --reuse-threads --unstable-presymbolicate \
  -o "$S/prof241/rexample_c.json.gz" -- "$S/t-prof241/release/tpl" -d bench_wl001 render example --table accrual

# 6. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Third waste-hunting pass: the whole command surface at `14d129f`

*Sprint 19, task `#243`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

Every node of the command tree the binary reports, every alias, both output
forms of every command that has two, every help form, the version forms and the
error paths were measured at `14d129f`, with `#240`'s instruments, protocol and
fixture and with `#240`'s binary rebuilt as an arm of the same rotated
campaign. Nothing was changed: `src/`, `Cargo.toml` and `Cargo.lock` are
untouched, and every instrumented or altered binary was built from a throwaway
copy of the crate outside the repository.

**Against `#240`'s binary, measured side by side, every render is 1.11 to
1.76 ms faster, a cached `schema dump` or `schema tables --format json`, in either form, 0.26
to 0.47 ms, and the canonical loop of 200 renders fell from 2 727.8 ms to
2 397.1 ms (−12.1%).** That is `#241`, confirmed. Every other invocation is
where `#240` left it: the 62 start-up-class labels are within 0.024 ms of it,
the 10 writers and server probes under 5 ms within 0.048 ms, and the direct
reads within 0.23 ms, each at the floor of its class.

**Three new rows are above the noise floor by an order of magnitude or more, and
each was established by a single-variable experiment:**

- **A cached render with an object bound reads and decodes all 272 cache files
  of `WL-001`.** Reading only the bound table's file took a `--table` render
  from 11.33 ms to 2.21 ms (−80.5%), its peak resident memory from 13.69 MiB to
  4.27 MiB, and the canonical loop from 2 412.2 ms to 579.1 ms (−76.0%). That
  figure is an upper bound: the instrument drops the rest of `database`, which
  `FR-RND-023` binds. Keeping it reachable without reading it first is a
  question for the specification (`FR-RND-023`, `FR-CTX-023`, `FR-CACHE-033`).
- **A cache write rewrites every object whose file already holds the same
  bytes.** Skipping those halves `cache load` (62.3 → 31.3 ms), a `66` on an
  absent name with the server up (62.9 → 31.1 ms) and `schema dump --direct`
  (67.3 → 33.8 ms). It does not reach the loop, whose one write follows
  `cache clean`.
- **`--pretty` writes each indent two bytes at a time.** Writing each newline
  and its indent in one call took the pretty cached dump from 17.70 to 15.03 ms
  (−15.1%) and the pretty table listing from 16.13 to 14.01 ms (−13.1%), with
  the same bytes out.

**The campaign does not stop**: the first row alone is 76% of the canonical
loop, 164 times `#240`'s floor for that loop (11.2 ms). It stops for every other
path; see *The verdict*.

### The surface

The inventory is the binary's own: `tpl help --format json` (`FR-HELP-016`),
whose `data.commands` lists 34 nodes. Five are groups (`schema`, `template`,
`cache`, `cfg`, `cfg database`), which print their help when invoked bare; 29
are leaves. Seven aliases are declared: `tbls`, `tbl`, `vws`, `vw`, `rtns`,
`rtn` and `db`. The seven global flags were exercised where they change a path:
`-d`, `--tpl-dir`, `--timeout`, `-v`, `-q`, `-h` and `-V`.

| leaf | measured as |
|---|---|
| `schema info` | `info_c`, `infojson_c`, `infopretty_c`, `info_d` |
| `schema tables` (`tbls`) | `tables_c`, `tables_alias_c`, `tablesjson_c`, `tablespretty_c`, `tablespat_c`, `tables_d` |
| `schema table` (`tbl`) | `table_c`, `table_alias_c`, `tablejson_c`, `tablepretty_c`, `table3_c`, `tpldir_c`, `table_d`, `table3_d` |
| `schema views` (`vws`) | `views_c`, `views_alias_c`, `viewsjson_c`, `views_d` |
| `schema view` (`vw`) | `view_c`, `view_alias_c`, `viewjson_c`, `view_d` |
| `schema routines` (`rtns`) | `routines_c`, `routines_alias_c`, `routinesjson_c`, `routines_d` |
| `schema routine` (`rtn`) | `routine_c`, `routine_alias_c`, `routinejson_c`, `routine_d` |
| `schema dump` | `dump_c`, `dumppretty_c`, `dump_d`, `dump_dw` |
| `template list` | `tlist`, `tlistjson`, `tlist_srv` |
| `template show` | `tshow`, `fail66_tpl` |
| `template check` | `tcheck` |
| `template path` | `tpath`, `tpathjson` |
| `render` | 23 labels: the cached, `--context` (file and standard input) and direct sources; `--table`, `--view`, `--routine` and no object; `--set`, `--timeout`, `-vvv`, `-q`; `fail66_rnd`; and the loop |
| `cache load` | `cload`, `cloadtable` |
| `cache clean` | `cclean`, `ccleantable` |
| `cache status` | `cstatus`, `cstatusjson` |
| `cfg get` | `cfgget`, `cfggetjson` |
| `cfg set` | `cfgset` |
| `cfg unset` | `cfgunset` |
| `cfg list` | `cfglist`, `cfglistjson` |
| `cfg database add` | `dbadd` |
| `cfg database list` (`db`) | `dblist`, `dblist_alias`, `dblistjson` |
| `cfg database show` | `dbshow`, `dbshowjson` |
| `cfg database update` | `dbupdate` |
| `cfg database remove` | `dbremove` |
| `cfg database test` | `dbtest`, `dbtestjson` |
| `init` | `init` |
| `help` | `help_cmd`, `helpjson`, `helpjson_p`, `helppath`, `helppathjson` |
| `version` | `version_cmd` |

Beside the leaves: the five groups bare (`g_schema`, `g_template`, `g_cache`,
`g_cfg`, `g_cfgdb`, and `g_cfgdb_alias` for `cfg db`); `tpl` with no argument
(`bare`); `--help`, `-h` and a help flag at a leaf with a required operand
(`help`, `help_h`, `nodehelp`); `--version` and `-V`; and the error paths:
`64` for an unknown flag and an unknown command, `78` for no project, `69` for
a refused connection on a cached and a direct read, and `66` for an absent
template, an absent render template and an absent table with the server up.
**All 29 leaves, all 5 groups and all 7 aliases were measured: 116 labels and
the loop.**

### Workload

`WL-001` (200 tables, 2 400 columns, 600 indexes, 180 foreign keys, 30 views,
40 routines: 272 cache files) and `WL-003`, loaded into `12.3` by
`scripts/mariadb/seed-bench.sh`, which verified every count. The `startup` and
`server` projects were built by the functions of `benches/fixture.sh` with the
binary of record, the cache primed by `fixture_prime`, and the five templates of
`examples/rust-data-layer/templates/rust/` copied into the `server` project, as
in the earlier entries. `--context` reads the compact dump of `WL-001`,
3 182 325 B, taken from the cache.

Four more projects were built beside them, in the scratch directory:

| project | what it is for |
|---|---|
| `mut` | the `server` configuration without a cache, for the commands that write `.tpl/.cfg`. `--prepare` restores the file before every run: the base file for `cfgset`, `dbadd`; the base file plus `core.query_timeout = 30` for `cfgunset`; the base file plus an entry `scratch` for `dbupdate` and `dbremove` |
| `dead` | the `server` configuration with the port changed to `13399`, where nothing listens: the `69` of a refused connection |
| `empty` | a directory with no `.tpl` above it: the `78` of project discovery, and `--tpl-dir` pointed at the `server` project |
| `initdir` | the target of `tpl init`, removed by `--prepare` |

Probe templates, written into the `server` project and not in the repository:

| template | source | output |
|---|---|---|
| `probe/whole_json` | `{{ database \| json }}`, as `#240` | 3 182 267 B |
| `probe/whole_walk` | a recursive macro that visits every value under `database` and prints each leaf | 979 214 B. **Not `#240`'s probe of the same name**, which printed 2 523 580 B; its figures are comparable within this entry only |
| `probe/view` | `{{ view.name }} {{ view.definition \| length }}`, for `render --view` | 27 B |
| `probe/routine` | `{{ routine.name }} {{ routine.kind }}`, for `render --routine` | 37 B |

### Candidates

| arm | binary |
|---|---|
| `b240` | `cargo build --release` of `git archive ab091db` in a scratch directory: 4 000 416 B, sha256 `bee72ac35b48f21a8fc74bcddced4f7eafa573ecc2007bc712e19d88f96b20b5` — `#241`'s `before`, byte for byte, and `#240`'s `now` rebuilt |
| `now` | `target/release/tpl` at `14d129f`, the binary of record: 4 000 448 B, sha256 `53ed347097269c4a53a0fd830029101ea1ab7a0a7da528ecc8dbaa09d530ab16` — `#241`'s `after`, byte for byte |
| `now_twin` | the same file, measured as a third label: the A/A arm |

The experiments are copies of `git archive 14d129f`, one change each, each built
into its own target directory from a path of the same length:

| variant | the one change | binary |
|---|---|---|
| `ctl243` | none: the arm every variant is compared with | 4 000 432 B, sha256 `f4985b3e…6e6f` |
| `bnd243` | `from_catalogue` serves `Look::Table(name)` instead of `Look::Everything` when the invocation binds a table, `src/cli/render.rs:404`: the cache is asked for `database.json`, `meta.json` and the bound table's file, and nothing else. **An instrument only**: the `database` it binds holds one table | 4 000 432 B, sha256 `b083f734…8b2a` |
| `ind243` | `--pretty` serialised through a formatter with `PrettyFormatter`'s logic, in which each newline and its indent are one `write_all` of a slice of a static buffer instead of one call per indent level, `src/output/json.rs:71` | 4 016 944 B, sha256 `51ff1446…412d` |
| `skp243` | `store` serialises the object into a buffer first, and returns without writing where the target file already holds exactly those bytes, `src/cache.rs:945` | 4 033 504 B, sha256 `15769397…ddce` |
| `dhat` | `dhat` 0.3.3 as the global allocator, `debug = 1`, `strip = false`, over `14d129f`. Used for allocation only, never for time | — |

**Output identity**, before any timing. Every label was run once with `now` and
once with `b240`, and stdout, stderr and the exit code compared: the 114 of the
main campaign differed only in the `loaded_at` of the two `cache status` forms,
which records the last write, and the two JSON forms added later matched. Every
variant was compared with `now` in the same way over the same 114 labels: every
file matched except the `cache status` outputs and the stderr of `-vvv`, which
reports the render's duration (`phase: render took 0.2ms`). **The outputs of
`bnd243` matched on every render it reaches** — `example`, `rust/struct`, with
and without `--set` — because none of those templates reads more of `database`
than its name; a template that walks `database.tables` would not.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01); release profile `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` |
| Instruments | `hyperfine` 1.20.0 (`-N`); `samply` 0.13.1 at 20 kHz; `dhat` 0.3.3; `/usr/bin/time -l`; `jq` |
| Server | the project fixture's `12.3` alone during timing, `tls = "disabled"`, account `root`, Docker 29.5.2; all five servers for the test suite |
| Power | mains (`AC Power`, battery 80%, not charging); Low Power Mode off |
| Load | 1.98 to 3.86 over the session; see *Confounders* |
| Taken | 2026-09-23 (UTC): the main campaign 06:24Z to 06:38Z, the loop to 06:41Z, profiles, allocation and memory to 06:50Z, the experiments 06:50Z to 06:57Z, the two JSON forms of the single-object reads 06:58Z |

`cargo flamegraph` was not used, as in `#231`: on macOS it needs `dtrace` and
root. `samply` was used in its place.

### Protocol

- **Wall time, as `#240`.** 8 rounds per label; the labels rotated by one
  position per round, and the three arms rotated inside one `hyperfine` call
  per label per round. Start-up-class, cached and `--context` labels: 40 runs
  after 5 warmups per arm per round, 320 samples. Direct reads and renders,
  `init`, `cclean`, `ccleantable`, `cloadtable`, `dbtest` and the five `cfg`
  writers: 20 after 5 (2 for `init`), 160 samples. `cload`, `dump_dw` and
  `fail66`, which write the whole store on every run: 5 after 1, 40 samples.
  `rwwalk_c`: 20 after 3. The loop: 4 rounds of 3 runs after 1 warmup, 12
  samples per arm, with `--prepare "tpl -d bench_wl001 cache clean"`. The
  writers' `--prepare` is listed under *Workload*; `rexample_xin` reads the
  dump from `hyperfine --input`.
- **The experiments** were one further rotated campaign of the same shape over
  five arms — `ctl243`, `bnd243`, `ind243`, `skp243`, `ctl243_twin` — on 18
  labels chosen so that each variant has labels it reaches and labels it does
  not, and a loop campaign over `ctl243`, `bnd243` and `ctl243_twin`. Each
  variant is compared with `ctl243`, never with `now`.
- **CPU attribution.** `samply record --rate 20000 --iteration-count N
  --reuse-threads --unstable-presymbolicate` over a symbolised build of
  `14d129f` (`debug = true`, `strip = false`), for 29 labels across every
  family, `N` from 5 (`rwwalk_c`) to 200 (start-up class). Samples with a zero
  thread CPU delta are excluded, and a share counts a sample once for every
  function on its stack. The presymbolicated tables are function-level, so an
  inlined function is counted in its caller. A share becomes milliseconds only
  by multiplying it by the median of `now`, and such a figure is labelled an
  estimate and is an upper bound.
- **Allocation.** One run of the `dhat` copy per label, all 116.
- **Peak resident memory.** `/usr/bin/time -l`, median of 7, for `b240` and
  `now` on 35 labels, and for the experiments against `ctl243`.
- **The open question.** A throwaway harness, built in the scratch directory
  and not in the repository, reads the 272 files of the `WL-001` cache with
  `fs::read_to_string` by absolute path, by bare name after changing into each
  collection's directory, and under a copy 16 directories deeper; 200 runs
  after 20 warmups each.

### The noise floor of the instrument on this host

The A/A arm, `now` against `now_twin`, inside the main campaign:

| class | labels | largest A/A difference |
|---|---|---|
| start-up, help, errors, templates, configuration, single-object reads, `cache status`, `schema info` | 62 | 0.021 ms (`view_c`, 1.05%) |
| `init`, `cclean`, `ccleantable`, `dbtest` in both forms, the five `cfg` writers | 10 | 0.069 ms (`dbtestjson`, 1.68%) |
| cached listings and whole reads, cached renders but `rwwalk_c`, `--context` renders | 22 | 0.088 ms (`rschema_x`, 0.60%) |
| `rwwalk_c` | 1 | 0.049 ms (0.05%) |
| direct reads and renders, `cloadtable` | 16 | 0.193 ms (`table3_d`, 1.69%) |
| `cload`, `dump_dw`, `fail66` | 3 | 0.547 ms (`cload`, 0.86%) |
| the loop | 1 | 6.221 ms (0.26%) |

In the experiments, `ctl243` against `ctl243_twin` was 0.001 to 0.121 ms on the
labels under 30 ms, 0.18 to 0.32 ms on the store writers and 1.273 ms on
`dump_dw`; 23.5 ms (0.97%) on the loop. A variant on a label it does not reach
moved by up to 0.155 ms (and 2.16 ms on `dump_dw`). **`#240`'s floor stands and
is the one this entry is read against: below 0.02 ms on a start-up-class
invocation, below 0.1 ms on a cached or direct read or render, below 0.5% on
the loop.** A variant's gain is established only where it is several times the
spread of the builds on that label.

### Results — every path, against `#240`

Medians; `rsd` is of `now`. A change is in bold where it is at least 0.1 ms and
more than twice the A/A difference of its label. `<w>` is the scratch work
directory. The heap is `dhat`'s, one run: total allocated, and held at the peak.

| label | invocation | exit | `b240` | `now` | change | A/A | `now` rsd | heap total / at peak, `now` | peak RSS, `b240` → `now` |
|---|---|---|---|---|---|---|---|---|---|
| `version` | `tpl --version` | 0 | 1.738 ms | **1.754 ms** | +0.016 ms | 0.014 ms | 4.24% | 372 213 / 260 180 B | 2.66 → 2.62 MiB |
| `version_V` | `tpl -V` | 0 | 1.741 ms | **1.731 ms** | −0.010 ms | 0.001 ms | 2.81% | 372 199 / 260 166 B | — |
| `version_cmd` | `tpl version` | 0 | 1.741 ms | **1.744 ms** | +0.003 ms | 0.006 ms | 2.48% | 379 195 / 262 454 B | — |
| `bare` | `tpl` | 0 | 1.856 ms | **1.854 ms** | −0.001 ms | 0.017 ms | 2.35% | 665 133 / 259 205 B | — |
| `help` | `tpl --help` | 0 | 1.860 ms | **1.844 ms** | −0.016 ms | 0.001 ms | 2.50% | 666 221 / 260 171 B | 2.92 → 2.94 MiB |
| `help_h` | `tpl -h` | 0 | 1.859 ms | **1.847 ms** | −0.012 ms | 0.000 ms | 2.26% | 666 213 / 260 163 B | — |
| `help_cmd` | `tpl help` | 0 | 1.866 ms | **1.870 ms** | +0.004 ms | 0.004 ms | 2.51% | 676 511 / 264 078 B | — |
| `helpjson` | `tpl help --format json` | 0 | 1.940 ms | **1.931 ms** | −0.009 ms | 0.001 ms | 2.27% | 738 377 / 264 150 B | — |
| `helpjson_p` | `tpl help --format json --pretty` | 0 | 1.997 ms | **2.001 ms** | +0.004 ms | 0.000 ms | 2.26% | 739 552 / 264 667 B | 3.09 → 3.08 MiB |
| `helppath` | `tpl help schema table` | 0 | 1.859 ms | **1.860 ms** | +0.000 ms | 0.012 ms | 2.06% | 668 012 / 264 698 B | — |
| `helppathjson` | `tpl help schema table --format json` | 0 | 1.821 ms | **1.823 ms** | +0.001 ms | 0.002 ms | 2.43% | 658 544 / 264 818 B | — |
| `nodehelp` | `tpl schema table --help` | 0 | 1.995 ms | **1.994 ms** | −0.002 ms | 0.007 ms | 2.41% | 1 402 962 / 670 745 B | 3.55 → 3.61 MiB |
| `g_schema` | `tpl schema` | 0 | 1.836 ms | **1.849 ms** | +0.013 ms | 0.003 ms | 2.37% | 766 040 / 332 387 B | — |
| `g_template` | `tpl template` | 0 | 1.824 ms | **1.837 ms** | +0.013 ms | 0.009 ms | 2.49% | 701 951 / 295 025 B | — |
| `g_cache` | `tpl cache` | 0 | 1.843 ms | **1.837 ms** | −0.006 ms | 0.003 ms | 2.60% | 697 981 / 291 674 B | — |
| `g_cfg` | `tpl cfg` | 0 | 1.846 ms | **1.861 ms** | +0.014 ms | 0.002 ms | 2.73% | 722 495 / 307 928 B | — |
| `g_cfgdb` | `tpl cfg database` | 0 | 1.872 ms | **1.873 ms** | +0.001 ms | 0.001 ms | 3.19% | 838 529 / 374 556 B | — |
| `g_cfgdb_alias` | `tpl cfg db` | 0 | 1.858 ms | **1.867 ms** | +0.009 ms | 0.002 ms | 2.95% | 838 517 / 374 544 B | — |
| `fail64` | `tpl -d bench_wl001 schema table accrual --no-such-flag` | 64 | 1.772 ms | **1.777 ms** | +0.005 ms | 0.013 ms | 2.93% | 478 246 / 329 988 B | — |
| `fail64_cmd` | `tpl schemx` | 64 | 1.722 ms | **1.726 ms** | +0.003 ms | 0.007 ms | 2.83% | 358 069 / 256 298 B | — |
| `fail78_noproj` | `tpl schema tables` | 78 | 1.877 ms | **1.881 ms** | +0.004 ms | 0.012 ms | 3.88% | 502 198 / 338 572 B | — |
| `fail69_conn` | `tpl schema tables` | 69 | 1.965 ms | **1.964 ms** | −0.000 ms | 0.009 ms | 3.01% | 561 989 / 338 572 B | 3.56 → 3.48 MiB |
| `fail69_conn_d` | `tpl schema table accrual --direct --no-cache` | 69 | 1.992 ms | **1.976 ms** | −0.016 ms | 0.006 ms | 2.47% | 566 764 / 340 917 B | — |
| `fail66_tpl` | `tpl template show nosuch` | 66 | 1.899 ms | **1.894 ms** | −0.005 ms | 0.004 ms | 6.68% | 442 485 / 299 945 B | — |
| `fail66_rnd` | `tpl render nosuch` | 66 | 1.909 ms | **1.901 ms** | −0.008 ms | 0.006 ms | 2.18% | 399 689 / 264 997 B | — |
| `fail66` | `tpl -d bench_wl001 schema table accrualx` | 66 | 62.832 ms | **62.167 ms** | −0.665 ms | 0.361 ms | 3.23% | 8 293 077 / 3 952 742 B | 8.34 → 8.45 MiB |
| `tlist` | `tpl template list` | 0 | 1.848 ms | **1.854 ms** | +0.006 ms | 0.012 ms | 2.96% | 446 609 / 300 006 B | 2.98 → 2.97 MiB |
| `tlistjson` | `tpl template list --format json` | 0 | 1.846 ms | **1.856 ms** | +0.010 ms | 0.010 ms | 2.72% | 447 077 / 300 102 B | — |
| `tlist_srv` | `tpl template list` | 0 | 1.869 ms | **1.872 ms** | +0.003 ms | 0.007 ms | 2.35% | 448 454 / 300 006 B | — |
| `tshow` | `tpl template show rust/schema` | 0 | 1.858 ms | **1.867 ms** | +0.009 ms | 0.011 ms | 2.56% | 460 435 / 299 965 B | — |
| `tcheck` | `tpl template check rust/schema` | 0 | 1.971 ms | **1.965 ms** | −0.006 ms | 0.006 ms | 2.22% | 510 325 / 299 969 B | — |
| `tpath` | `tpl template path rust/struct` | 0 | 1.844 ms | **1.842 ms** | −0.002 ms | 0.001 ms | 2.22% | 448 007 / 300 626 B | — |
| `tpathjson` | `tpl template path rust/struct --format json` | 0 | 1.845 ms | **1.853 ms** | +0.008 ms | 0.008 ms | 3.82% | 448 355 / 300 746 B | — |
| `cfglist` | `tpl cfg list` | 0 | 1.858 ms | **1.863 ms** | +0.005 ms | 0.009 ms | 2.95% | 482 086 / 312 885 B | 3.16 → 3.20 MiB |
| `cfglistjson` | `tpl cfg list --format json` | 0 | 1.857 ms | **1.867 ms** | +0.010 ms | 0.003 ms | 4.43% | 483 907 / 312 981 B | — |
| `cfgget` | `tpl cfg get core.database` | 0 | 1.873 ms | **1.869 ms** | −0.004 ms | 0.003 ms | 3.38% | 483 108 / 313 510 B | — |
| `cfggetjson` | `tpl cfg get core.database --format json` | 0 | 1.858 ms | **1.871 ms** | +0.012 ms | 0.002 ms | 2.91% | 483 447 / 313 630 B | — |
| `cfgset` | `tpl cfg set core.database bench_wl001` | 0 | 1.994 ms | **1.992 ms** | −0.003 ms | 0.002 ms | 2.53% | 496 876 / 313 753 B | — |
| `cfgunset` | `tpl cfg unset core.query_timeout` | 0 | 1.987 ms | **2.002 ms** | +0.014 ms | 0.010 ms | 2.49% | 495 687 / 312 874 B | — |
| `dblist` | `tpl cfg database list` | 0 | 1.893 ms | **1.883 ms** | −0.009 ms | 0.003 ms | 2.58% | 597 892 / 379 305 B | — |
| `dblist_alias` | `tpl cfg db list` | 0 | 1.879 ms | **1.878 ms** | −0.001 ms | 0.006 ms | 2.40% | 597 880 / 379 293 B | — |
| `dblistjson` | `tpl cfg database list --format json` | 0 | 1.881 ms | **1.884 ms** | +0.003 ms | 0.001 ms | 2.52% | 598 384 / 379 425 B | — |
| `dbshow` | `tpl cfg database show bench_wl001` | 0 | 1.880 ms | **1.882 ms** | +0.002 ms | 0.015 ms | 2.40% | 600 102 / 379 949 B | 3.31 → 3.30 MiB |
| `dbshowjson` | `tpl cfg database show bench_wl001 --format json` | 0 | 1.896 ms | **1.890 ms** | −0.006 ms | 0.001 ms | 2.23% | 600 114 / 380 069 B | — |
| `dbadd` | `tpl cfg database add scratch --host 127.0.0.1 --port 13309 --user root --schema freight_wl003 --tls disabled` | 0 | 2.049 ms | **2.028 ms** | −0.022 ms | 0.016 ms | 2.04% | 629 198 / 385 131 B | 3.38 → 3.39 MiB |
| `dbupdate` | `tpl cfg database update scratch --port 13309` | 0 | 2.019 ms | **2.021 ms** | +0.003 ms | 0.006 ms | 5.80% | 624 573 / 380 449 B | — |
| `dbremove` | `tpl cfg database remove scratch` | 0 | 2.015 ms | **2.024 ms** | +0.009 ms | 0.003 ms | 2.33% | 620 291 / 379 278 B | — |
| `dbtest` | `tpl cfg database test bench_wl001` | 0 | 4.040 ms | **4.043 ms** | +0.003 ms | 0.003 ms | 2.81% | 675 194 / 379 949 B | — |
| `dbtestjson` | `tpl cfg database test bench_wl001 --format json` | 0 | 4.036 ms | **4.084 ms** | +0.048 ms | 0.069 ms | 3.11% | 675 206 / 380 069 B | — |
| `init` | `tpl init <w>/initdir` | 0 | 2.151 ms | **2.147 ms** | −0.003 ms | 0.000 ms | 4.58% | 379 346 / 264 393 B | 2.77 → 2.78 MiB |
| `cstatus` | `tpl -d bench_wl001 cache status` | 0 | 2.085 ms | **2.082 ms** | −0.003 ms | 0.004 ms | 2.88% | 604 437 / 298 909 B | 3.31 → 3.31 MiB |
| `cstatusjson` | `tpl -d bench_wl001 cache status --format json` | 0 | 2.067 ms | **2.064 ms** | −0.003 ms | 0.004 ms | 2.63% | 587 738 / 299 029 B | — |
| `cload` | `tpl -d bench_wl001 cache load` | 0 | 63.737 ms | **63.650 ms** | −0.087 ms | 0.547 ms | 2.84% | 8 218 843 / 3 951 958 B | 8.33 → 8.28 MiB |
| `cloadtable` | `tpl -d bench_wl001 cache load --table accrual` | 0 | 23.521 ms | **23.538 ms** | +0.017 ms | 0.009 ms | 1.84% | 5 517 630 / 3 885 608 B | 8.22 → 8.11 MiB |
| `cclean` | `tpl -d bench_wl003 cache clean` | 0 | 2.419 ms | **2.373 ms** | −0.045 ms | 0.015 ms | 5.36% | 456 075 / 297 985 B | — |
| `ccleantable` | `tpl -d bench_wl003 cache clean --table consignment` | 0 | 2.306 ms | **2.263 ms** | −0.043 ms | 0.011 ms | 4.35% | 469 219 / 299 080 B | — |
| `info_c` | `tpl -d bench_wl001 schema info` | 0 | 2.102 ms | **2.107 ms** | +0.005 ms | 0.000 ms | 3.23% | 657 320 / 341 728 B | 3.45 → 3.44 MiB |
| `infojson_c` | `tpl -d bench_wl001 schema info --format json` | 0 | 2.116 ms | **2.098 ms** | −0.017 ms | 0.001 ms | 2.90% | 657 389 / 341 848 B | — |
| `infopretty_c` | `tpl -d bench_wl001 schema info --format json --pretty` | 0 | 2.105 ms | **2.109 ms** | +0.004 ms | 0.015 ms | 3.31% | 658 564 / 342 365 B | — |
| `tables_c` | `tpl -d bench_wl001 schema tables` | 0 | 7.682 ms | **7.691 ms** | +0.009 ms | 0.015 ms | 2.36% | 3 852 706 / 3 197 830 B | 7.11 → 7.16 MiB |
| `tables_alias_c` | `tpl -d bench_wl001 schema tbls` | 0 | 7.636 ms | **7.655 ms** | +0.019 ms | 0.011 ms | 2.05% | 3 852 702 / 3 197 830 B | — |
| `tablesjson_c` | `tpl -d bench_wl001 schema tables --format json` | 0 | 12.888 ms | **12.625 ms** | **−0.264 ms, −2.0%** | 0.043 ms | 2.33% | 11 616 760 / 8 139 408 B | 12.84 → 12.83 MiB |
| `tablespretty_c` | `tpl -d bench_wl001 schema tables --format json --pretty` | 0 | 16.567 ms | **16.172 ms** | **−0.395 ms, −2.4%** | 0.034 ms | 1.32% | 11 617 935 / 8 139 408 B | 12.84 → 12.86 MiB |
| `tablespat_c` | `tpl -d bench_wl001 schema tables --pattern acc%` | 0 | 7.704 ms | **7.706 ms** | +0.002 ms | 0.035 ms | 3.23% | 3 832 808 / 3 183 012 B | — |
| `table_c` | `tpl -d bench_wl001 schema table accrual` | 0 | 1.999 ms | **1.999 ms** | −0.000 ms | 0.002 ms | 2.77% | 621 572 / 342 359 B | 3.77 → 3.77 MiB |
| `table_alias_c` | `tpl -d bench_wl001 schema tbl accrual` | 0 | 1.998 ms | **1.990 ms** | −0.008 ms | 0.000 ms | 2.70% | 621 568 / 342 355 B | — |
| `tablejson_c` | `tpl -d bench_wl001 schema table accrual --format json` | 0 | 1.999 ms | **1.999 ms** | −0.000 ms | 0.003 ms | 6.70% | 575 974 / 342 479 B | — |
| `tablepretty_c` | `tpl -d bench_wl001 schema table accrual --format json --pretty` | 0 | 2.011 ms | **2.022 ms** | +0.011 ms | 0.013 ms | 2.53% | 577 149 / 342 996 B | — |
| `table3_c` | `tpl -d bench_wl003 schema table consignment` | 0 | 1.990 ms | **1.978 ms** | −0.011 ms | 0.008 ms | 2.43% | 850 126 / 342 375 B | — |
| `views_c` | `tpl -d bench_wl001 schema views` | 0 | 2.320 ms | **2.314 ms** | −0.006 ms | 0.006 ms | 2.43% | 606 936 / 341 795 B | — |
| `views_alias_c` | `tpl -d bench_wl001 schema vws` | 0 | 2.314 ms | **2.308 ms** | −0.006 ms | 0.007 ms | 2.29% | 606 932 / 341 791 B | — |
| `viewsjson_c` | `tpl -d bench_wl001 schema views --format json` | 0 | 2.318 ms | **2.335 ms** | +0.017 ms | 0.016 ms | 2.79% | 604 324 / 341 915 B | — |
| `view_c` | `tpl -d bench_wl001 schema view v_booking_line_summary` | 0 | 1.942 ms | **1.952 ms** | +0.010 ms | 0.021 ms | 2.59% | 541 638 / 342 416 B | 3.48 → 3.53 MiB |
| `view_alias_c` | `tpl -d bench_wl001 schema vw v_booking_line_summary` | 0 | 1.955 ms | **1.931 ms** | −0.024 ms | 0.015 ms | 2.49% | 541 634 / 342 412 B | — |
| `viewjson_c` | `tpl -d bench_wl001 schema view v_booking_line_summary --format json` | 0 | 1.961 ms | **1.942 ms** | −0.019 ms | 0.001 ms | 5.30% | — | — |
| `routines_c` | `tpl -d bench_wl001 schema routines` | 0 | 2.498 ms | **2.495 ms** | −0.004 ms | 0.005 ms | 2.56% | 734 125 / 341 804 B | — |
| `routines_alias_c` | `tpl -d bench_wl001 schema rtns` | 0 | 2.509 ms | **2.495 ms** | −0.014 ms | 0.011 ms | 1.95% | 734 117 / 341 796 B | — |
| `routinesjson_c` | `tpl -d bench_wl001 schema routines --format json` | 0 | 2.513 ms | **2.513 ms** | +0.000 ms | 0.005 ms | 2.57% | 729 513 / 341 924 B | — |
| `routine_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count` | 0 | 2.504 ms | **2.486 ms** | −0.018 ms | 0.008 ms | 5.76% | 739 199 / 342 445 B | 3.81 → 3.86 MiB |
| `routine_alias_c` | `tpl -d bench_wl001 schema rtn fn_consignment_hazard_count` | 0 | 2.492 ms | **2.486 ms** | −0.006 ms | 0.010 ms | 2.38% | 739 191 / 342 437 B | — |
| `routinejson_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --format json` | 0 | 2.484 ms | **2.489 ms** | +0.004 ms | 0.030 ms | 9.58% | — | — |
| `dump_c` | `tpl -d bench_wl001 schema dump` | 0 | 13.934 ms | **13.652 ms** | **−0.282 ms, −2.0%** | 0.016 ms | 1.65% | 11 869 584 / 8 224 312 B | 13.12 → 13.11 MiB |
| `dumppretty_c` | `tpl -d bench_wl001 schema dump --pretty` | 0 | 18.084 ms | **17.612 ms** | **−0.472 ms, −2.6%** | 0.034 ms | 1.74% | 11 872 839 / 8 224 312 B | 13.09 → 13.12 MiB |
| `tpldir_c` | `tpl --tpl-dir <w>/server/.tpl -d bench_wl001 schema table accrual` | 0 | 2.020 ms | **2.009 ms** | −0.011 ms | 0.002 ms | 2.81% | 626 798 / 344 168 B | — |
| `info_d` | `tpl -d bench_wl001 schema info --direct --no-cache` | 0 | 23.855 ms | **23.777 ms** | −0.078 ms | 0.005 ms | 2.28% | 5 563 303 / 3 884 483 B | — |
| `tables_d` | `tpl -d bench_wl001 schema tables --direct --no-cache` | 0 | 23.912 ms | **23.782 ms** | −0.131 ms | 0.088 ms | 3.05% | 5 601 690 / 3 922 760 B | — |
| `table_d` | `tpl -d bench_wl001 schema table accrual --direct --no-cache` | 0 | 23.943 ms | **23.734 ms** | **−0.208 ms, −0.9%** | 0.053 ms | 1.69% | 5 610 172 / 3 887 643 B | 8.33 → 8.25 MiB |
| `table3_d` | `tpl -d bench_wl003 schema table consignment --direct --no-cache` | 0 | 11.707 ms | **11.476 ms** | −0.231 ms | 0.193 ms | 2.73% | 820 025 / 343 223 B | — |
| `views_d` | `tpl -d bench_wl001 schema views --direct --no-cache` | 0 | 23.859 ms | **23.755 ms** | **−0.103 ms, −0.4%** | 0.016 ms | 3.14% | 5 566 301 / 3 887 378 B | — |
| `view_d` | `tpl -d bench_wl001 schema view v_booking_line_summary --direct --no-cache` | 0 | 23.876 ms | **23.816 ms** | −0.060 ms | 0.178 ms | 4.37% | 5 564 680 / 3 884 472 B | — |
| `routines_d` | `tpl -d bench_wl001 schema routines --direct --no-cache` | 0 | 23.910 ms | **23.757 ms** | **−0.153 ms, −0.6%** | 0.026 ms | 4.05% | 5 568 402 / 3 889 458 B | — |
| `routine_d` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --direct --no-cache` | 0 | 23.976 ms | **23.799 ms** | **−0.177 ms, −0.7%** | 0.084 ms | 3.36% | 5 573 316 / 3 884 865 B | — |
| `dump_d` | `tpl -d bench_wl001 schema dump --direct --no-cache` | 0 | 26.383 ms | **26.322 ms** | −0.061 ms | 0.043 ms | 2.21% | 5 561 632 / 3 884 010 B | 8.28 → 8.11 MiB |
| `dump_dw` | `tpl -d bench_wl001 schema dump --direct` | 0 | 65.278 ms | **65.088 ms** | **−0.190 ms, −0.3%** | 0.015 ms | 1.82% | 8 296 599 / 3 952 726 B | — |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual` | 0 | 12.873 ms | **11.276 ms** | **−1.597 ms, −12.4%** | 0.045 ms | 2.21% | 11 848 144 / 8 309 594 B | 17.94 → 13.69 MiB |
| `rexample_vvv_c` | `tpl -vvv -d bench_wl001 render example --table accrual` | 0 | 13.063 ms | **11.300 ms** | **−1.763 ms, −13.5%** | 0.023 ms | 3.15% | 11 850 858 / 8 309 658 B | — |
| `rexample_q_c` | `tpl -q -d bench_wl001 render example --table accrual` | 0 | 12.883 ms | **11.313 ms** | **−1.570 ms, −12.2%** | 0.054 ms | 6.07% | 11 848 864 / 8 309 594 B | — |
| `rexample_to_c` | `tpl --timeout 30 -d bench_wl001 render example --table accrual` | 0 | 12.915 ms | **11.233 ms** | **−1.683 ms, −13.0%** | 0.068 ms | 3.70% | 11 850 861 / 8 309 626 B | — |
| `rexample_set_c` | `tpl -d bench_wl001 render example --table accrual --set title=Accrual` | 0 | 13.007 ms | **11.303 ms** | **−1.704 ms, −13.1%** | 0.011 ms | 4.19% | 11 850 485 / 8 310 615 B | — |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual` | 0 | 13.453 ms | **11.963 ms** | **−1.491 ms, −11.1%** | 0.014 ms | 5.78% | 13 335 459 / 8 461 777 B | — |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema` | 0 | 19.405 ms | **17.852 ms** | **−1.552 ms, −8.0%** | 0.005 ms | 3.59% | 30 733 945 / 8 557 461 B | 18.47 → 14.20 MiB |
| `rview_c` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary` | 0 | 12.864 ms | **11.156 ms** | **−1.708 ms, −13.3%** | 0.036 ms | 4.23% | 11 778 114 / 8 245 937 B | 17.78 → 13.55 MiB |
| `rroutine_c` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count` | 0 | 12.818 ms | **11.277 ms** | **−1.541 ms, −12.0%** | 0.018 ms | 3.38% | 11 778 674 / 8 246 674 B | — |
| `rwjson_c` | `tpl -d bench_wl001 render probe/whole_json` | 0 | 27.586 ms | **26.234 ms** | **−1.352 ms, −4.9%** | 0.002 ms | 1.01% | 55 046 855 / 34 040 823 B | 46.45 → 42.27 MiB |
| `rwwalk_c` | `tpl -d bench_wl001 render probe/whole_walk` | 0 | 106.186 ms | **104.860 ms** | **−1.326 ms, −1.2%** | 0.049 ms | 0.99% | 824 308 505 / 29 547 590 B | 41.88 → 38.64 MiB |
| `rexample_x` | `tpl render example --table accrual --context <w>/wl001-dump.json` | 0 | 9.492 ms | **8.160 ms** | **−1.332 ms, −14.0%** | 0.018 ms | 1.53% | 12 561 854 / 8 244 432 B | 17.52 → 13.41 MiB |
| `rexample_xin` | `tpl render example --table accrual --context - < <w>/wl001-dump.json` | 0 | 9.610 ms | **8.227 ms** | **−1.383 ms, −14.4%** | 0.038 ms | 1.72% | 17 775 500 / 9 264 544 B | 19.66 → 15.44 MiB |
| `rschema_x` | `tpl render rust/schema --context <w>/wl001-dump.json` | 0 | 15.749 ms | **14.639 ms** | **−1.110 ms, −7.0%** | 0.088 ms | 1.28% | 31 447 655 / 8 244 333 B | — |
| `rview_x` | `tpl render probe/view --view v_booking_line_summary --context <w>/wl001-dump.json` | 0 | 9.438 ms | **8.065 ms** | **−1.373 ms, −14.5%** | 0.030 ms | 1.67% | 12 491 824 / 8 244 450 B | — |
| `rwjson_x` | `tpl render probe/whole_json --context <w>/wl001-dump.json` | 0 | 23.942 ms | **22.693 ms** | **−1.248 ms, −5.2%** | 0.037 ms | 1.07% | 55 760 565 / 31 560 457 B | 46.03 → 41.83 MiB |
| `rexample_d` | `tpl -d bench_wl001 render example --table accrual --direct --no-cache` | 0 | 25.305 ms | **23.978 ms** | **−1.326 ms, −5.2%** | 0.017 ms | 2.18% | 5 559 714 / 3 987 012 B | 13.14 → 8.64 MiB |
| `rstruct_d` | `tpl -d bench_wl001 render rust/struct --table accrual --direct --no-cache` | 0 | 25.925 ms | **24.642 ms** | **−1.282 ms, −4.9%** | 0.062 ms | 1.51% | 7 046 991 / 4 139 195 B | — |
| `rschema_d` | `tpl -d bench_wl001 render rust/schema --direct --no-cache` | 0 | 31.638 ms | **30.487 ms** | **−1.150 ms, −3.6%** | 0.082 ms | 1.55% | 24 447 191 / 4 234 879 B | — |
| `rview_d` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary --direct --no-cache` | 0 | 25.275 ms | **23.820 ms** | **−1.455 ms, −5.8%** | 0.047 ms | 1.86% | 5 489 760 / 3 923 355 B | — |
| `rroutine_d` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count --direct --no-cache` | 0 | 25.131 ms | **23.908 ms** | **−1.222 ms, −4.9%** | 0.061 ms | 2.68% | 5 490 320 / 3 924 092 B | — |
| `rwjson_d` | `tpl -d bench_wl001 render probe/whole_json --direct --no-cache` | 0 | 39.894 ms | **38.686 ms** | **−1.208 ms, −3.0%** | 0.020 ms | 1.49% | 48 738 743 / 29 718 241 B | 41.64 → 37.30 MiB |

`viewjson_c` and `routinejson_c` come from a second campaign of the same shape,
run after the main one (see *Confounders*). The canonical loop:

| label | `b240` | `now` | change | A/A | `now` rsd |
|---|---|---|---|---|---|
| loop | 2 727.8 ms | **2 397.1 ms** | **−330.8 ms, −12.1%** | 6.2 ms | 0.64% |

### Results — the experiments

Medians of 320 samples (40 for the three store writers, 160 for `cloadtable`
and `dump_d`, 12 for the loop). Each variant against `ctl243`:

| label | `ctl243` | variant | change | A/A of `ctl243` | peak RSS, `ctl243` → variant |
|---|---|---|---|---|---|
| **`bnd243`**, the bound table's file only | | | | | |
| `rexample_c` | 11.327 ms | 2.206 ms | **−9.121 ms, −80.5%** | 0.044 ms | 13.69 → 4.27 MiB |
| `rstruct_c` | 11.916 ms | 2.834 ms | **−9.083 ms, −76.2%** | 0.037 ms | — |
| `rexample_set_c` | 11.369 ms | 2.199 ms | **−9.169 ms, −80.7%** | 0.121 ms | — |
| loop | 2 412.2 ms | 579.1 ms | **−1 833.1 ms, −76.0%** | 23.5 ms | — |
| `rview_c`, not reached | 11.194 ms | 11.226 ms | +0.032 ms | 0.025 ms | — |
| `rschema_c`, not reached | 17.751 ms | 17.846 ms | +0.095 ms | 0.065 ms | — |
| `table_c`, not reached | 2.008 ms | 2.016 ms | +0.009 ms | 0.009 ms | — |
| **`ind243`**, one write per indent | | | | | |
| `dumppretty_c` | 17.698 ms | 15.030 ms | **−2.668 ms, −15.1%** | 0.024 ms | 13.06 → 13.14 MiB |
| `tablespretty_c` | 16.130 ms | 14.014 ms | **−2.116 ms, −13.1%** | 0.011 ms | — |
| `helpjson_p` | 2.000 ms | 1.965 ms | −0.035 ms, −1.8% | 0.004 ms | — |
| `tablepretty_c` | 2.014 ms | 2.022 ms | +0.009 ms | 0.017 ms | — |
| `infopretty_c` | 2.095 ms | 2.100 ms | +0.005 ms | 0.001 ms | — |
| `dump_c`, compact, not reached | 13.585 ms | 13.734 ms | +0.149 ms | 0.111 ms | — |
| **`skp243`**, identical objects not rewritten | | | | | |
| `cload` | 62.325 ms | 31.327 ms | **−30.998 ms, −49.7%** | 0.318 ms | 8.28 → 8.80 MiB |
| `fail66` | 62.934 ms | 31.095 ms | **−31.838 ms, −50.6%** | 0.178 ms | — |
| `dump_dw` | 67.256 ms | 33.768 ms | **−33.489 ms, −49.8%** | 1.273 ms | — |
| `cloadtable`, one file | 23.488 ms | 23.780 ms | +0.293 ms | 0.151 ms | — |
| `dump_d`, `--no-cache`, not reached | 26.298 ms | 26.216 ms | −0.082 ms | 0.082 ms | — |

- **`bnd243` is an upper bound on row 1, not a design.** It binds a `database`
  holding one table, which `FR-RND-023` does not allow; the worked-example
  templates render the same bytes only because they read nothing of `database`
  but its name. What it measures is the cost of reading and validating the 271
  files the template never reaches: 9.1 ms per render, and the remaining
  2.2 ms is `schema table`'s own 2.0 ms plus the render. A `--view` or
  `--routine` render reads the whole store the same way (`rview_c` 11.194 ms
  against `view_c` 1.952 ms, `rroutine_c` 11.277 ms against `routine_c`
  2.486 ms), so the same saving is expected there; that is an estimate.
- **`skp243` saves 114 µs per object whose file is unchanged**, of the rename,
  the open of the temporary and its write and close. On a catalogue whose every
  object changed it would add a read per file instead: about 11 µs each, by the
  harness below, or ≈ +3 ms for `WL-001`, an estimate. Its peak resident memory
  is 0.52 MiB higher, which is the object serialised into a buffer before the
  comparison. `cloadtable` rewrites one file, and its +0.29 ms is twice the A/A
  of that label and is not attributed.
- **`ind243` changes where the bytes are written from, not which bytes.** Every
  pretty output of the 114 labels matched `now` byte for byte. The gain is in
  proportion to the indentation written: 7.2 MB of pretty dump and 6.7 MB of
  pretty listing gain 2.1 to 2.7 ms, the 130 kB of pretty help 0.035 ms, and a
  single object nothing measurable.

**The open question, answered by the harness** (mean ± σ of 200 runs, reading
the 272 files of the `WL-001` cache):

| read | wall time |
|---|---|
| by absolute path, as `tpl::cache::members` does | 4.5 ± 0.2 ms |
| by bare name, after changing into each directory | 4.5 ± 0.4 ms |
| by absolute path, 16 directories deeper | 5.1 ± 0.1 ms |
| by bare name, 16 directories deeper | 4.9 ± 0.1 ms |
| the three directory walks alone | 1.5 ± 0.1 ms |

Reading a file costs about 11 µs, and the path is not where that goes: a
relative open, which is what `openat` on a directory descriptor would give, is
no faster. The `open` share of a cached read is the operating system's
per-file cost, and only reading fewer files reduces it.

### Where the remaining cost goes

Shares of on-CPU samples of `now`; inclusive, so they overlap. A figure in
milliseconds is an estimate and an upper bound.

- **A cached render with an object bound, `rexample_c` (11.276 ms).** Reading
  the 272 files is 42.6% (`open` alone 30.7%), decoding them 51.0%: the column
  lists 37.8%, and the two reference members 32.2% (`foreign_keys` 15.9%,
  `referenced_by` 16.3%).
  `malloc` is 9.8% and the growth of vectors 6.2%. The template and its
  context are 1.75%. `bnd243` puts 9.1 ms of the 11.3 ms on the 271 files the
  template does not reach.
- **`schema dump` from the cache, `dump_c` (13.652 ms).** Reading is 34.0%
  (`open` 24.3%), decoding 42.6%, serialising the output 20.9%. `#240`'s row 1
  stands.
- **`schema dump --pretty` from the cache, `dumppretty_c` (17.612 ms).**
  Serialising is 39.6%. `_platform_memmove` is 17.5% self, against 7.1% in
  the compact form, and almost all of it is called from `serde_json`'s
  serialiser: the indent, written two bytes per level (`ind243`).
- **A `--context` render, `rexample_x` (8.160 ms).** `model::document::read`
  is 87.6%; `#242`'s row 2a stands.
- **A cache write, `cload` (63.650 ms).** `store` is 81.6%: `rename` 28.4%,
  `open` 25.4%, `write` 12.5%, `close` 7.4%. The server read and the fold are
  12.4%. `fail66` and `dump_dw` have the same profile, to within 2.1 points on
  each of those five.
- **The server reads.** `dump_d` is 38.8% waiting on the server and in `sqlx`,
  and 18.0% self in the output serialiser. `cloadtable` is 55.3% waiting; it
  reads the whole catalogue to write one file, which is `#231`'s non-finding.
- **`cfg database test`, `dbtest` (4.043 ms)**, is 75.7% the connection and
  43.3% `kevent`: waiting on the server, which `FR-CACHE-010` requires.
- **The start-up class** (1.73 to 2.00 ms, against `#240`'s spawn floor of
  1.283 to 1.290 ms) is the parser tree and, where a project is read, its
  discovery and configuration (14.0% of `template check`'s samples): `#240`'s
  rows 5 and 6.
- **Allocation.** `rexample_xin` allocates 5 213 646 B more than `rexample_x`
  and holds 1 020 112 B more at its peak: `read_document`,
  `src/cli/render.rs:573`, grows a `Vec` from empty to 4 MiB for the 3.18 MB
  document on standard input (8 388 576 B in 18 blocks). Its time is 0.067 ms
  above `rexample_x`'s, at the floor. In `rexample_c`, the sequence visitor of
  `Vec<Column>` allocates 5 916 336 B, of which 3 269 552 B is held at the peak.

### The waste register, sorted by estimated gain

Effort is `#231`'s: `S` (one function, no interface change), `M` (a new code
path or a hand-written serde implementation), `L` (a new abstraction across
modules). "Established" means a single-variable experiment above measured the
gain; "estimate" means a profile share multiplied by a measured median, an upper
bound unless stated otherwise. `#240`'s rows 5 to 8 stand at or below the floor
and are not repeated.

| # | Path | Evidence | Cause in code | Why it is vacuous for the command | Estimated gain | Effort | Functional risk |
|---|---|---|---|---|---|---|---|
| 1 | A cached render with `--table`, `--view` or `--routine` | `bnd243` −9.08 to −9.17 ms per `--table` render; −1 833 ms on the loop; 13.69 → 4.27 MiB. A/A 0.04 to 0.12 ms; 23.5 ms on the loop | `reader.serve_from(&opened, &Look::Everything, …)`, `src/cli/render.rs:404`, which reads every file through `Cache::everything`, `src/cache.rs:470`, and decodes them all in `Loaded::document`, `src/cache.rs:173` | The template reads the bound object and `database.name`; the other 271 files are read and validated and never reached | **−9.1 ms per `--table` render (−80%), and −1.83 s (−76%) on the loop, established as an upper bound**; ≈ −8.8 to −9.2 ms per `--view` or `--routine` render, estimate | `L`, after the specification | `FR-RND-023` binds `database` whole, so the rest must stay reachable, read on first access; `FR-CACHE-033` makes a file that fails validation a miss, decided today before the render starts; `FR-CTX-023` requires every referenced object present; `FR-CDOC-007`. Whether a miss discovered mid-render is acceptable is a question for the specification |
| 2 | Every whole write of the store: `cache load`, a read-through miss (`66` with the server up, `--direct`), and the first render of the loop | `skp243` −31.0 to −33.5 ms (−50%) per whole write of an unchanged catalogue; A/A 0.18 to 1.27 ms | `store`, `src/cache.rs:945`–`973`, writes a temporary and renames it over the target whatever the target holds | An object whose file already holds the same bytes is rewritten with the same bytes | **−31 ms per whole write of an unchanged `WL-001`, established**; ≈ +3 ms per whole write of a catalogue that changed throughout, estimate; +0.52 MiB peak. None on the loop, whose write follows `cache clean` | `S` | `FR-CACHE-030` requires each object to be written through a temporary renamed over the target; skipping it leaves the file's modification time and inode unchanged. `FR-CACHE-007` |
| 3 (`#240` row 1) | The compact `schema dump` and `schema tables --format json`, from the cache | Unchanged: 13.652 and 12.625 ms; decode 42.6% of `dump_c` | `Loaded::document`, `src/cache.rs:173`, and the JSON presentation reached from `src/cli/schema.rs:403` and `:542` | As `#240` stated | ≈ −5 to −6 ms per compact cached read, estimate, as `#240` stated; not in the loop | `M`–`L` | As `#240` stated |
| 4 | Every `--pretty` form of a whole read: `schema dump --pretty`, `schema tables --format json --pretty` | `ind243` −2.67 and −2.12 ms; `memmove` 17.5% self of `dumppretty_c`'s samples, against 7.1% of `dump_c`'s | `serde_json::to_writer_pretty`, `src/output/json.rs:71`: `PrettyFormatter` writes the indent with one `write_all` per level | The same bytes can be written in one call per newline | **−2.1 to −2.7 ms per pretty whole read (−13% to −15%), established**; −0.035 ms on `help --format json --pretty`; nothing on one object. Not in the loop | `S` | `FR-OUT-008` fixes the two-space indent. `src/output/json.rs:50`–`52` rests that on the encoder's default; a formatter of the project's own moves the guarantee to a test |
| 5 | Every decode of the store and of a `--context` document | `finish_grow` 6.18% of `rexample_c` and 7.00% of `rexample_x`; `Vec<Column>` 5 916 336 B allocated for 3 269 552 B held | `serde`'s sequence visitor grows each `Vec` from empty, since `serde_json` gives no length: `columns`, `src/model/document/shape.rs:179`, and the other collections of the shape | The reallocations copy what the final vector holds | ≈ −0.7 ms per cached render, estimate, upper bound; row 1 would remove most of it for a bound render | `M` | None |
| 6 | Whole reads printed to stdout | `write` self 1.45% of `dump_c` and 3.62% of `dumppretty_c` | `BufWriter::new(Tracked::new(stream))`, `src/output/writer.rs:191`: the default 8 KiB, about 389 writes for the 3.18 MB dump | A larger buffer issues fewer `write` calls for the same bytes | ≤ −0.20 ms per compact and ≤ −0.64 ms per pretty dump, estimate, upper bound | `S` | None: the bytes are the same |
| 7 (`#242` row 2a) | The `--context` render | Unchanged: `model::document::read` 87.6% of `rexample_x` | As `#242` stated | As `#242` stated | ≤ −0.54 ms per `--context` render, estimate | `M` | As `#242` stated |
| 8 | `render --context -` | +0.067 ms against the same document from a file (A/A 0.018 and 0.038 ms); +5 213 646 B allocated, +1 020 112 B at the peak | `Vec::new()` and `read_to_end`, `src/cli/render.rs:570`–`573` | The growth from empty reallocates 18 times for a document whose size a redirected file states | **At the floor in time, established in bytes** | `S` | None |

### The verdict: does the campaign stop?

`#240`'s floor is 0.000 to 0.020 ms on a start-up-class invocation, up to
0.046 ms on a cached render, up to 0.093 ms on a direct read, and 11.2 ms
(0.41%) on the canonical loop. Against it, row by row:

| # | Gain against its own invocation | Against the canonical loop | Worth a further round? |
|---|---|---|---|
| 1 | ≈ 80% of a bound cached render; about 200 times its floor | 76%, 164 times the loop's floor | **Yes — after the specification.** It is by far the largest item left, and the only one in the loop. The first step is `specification-manager`, not code |
| 2 | ≈ 50% of a whole store write; about 60 times this entry's A/A for that class | none | **Yes.** Established, `S`, one function; the wording of `FR-CACHE-030` is settled first |
| 3 | ≈ 40% of a compact cached dump, estimate | none | **Yes**, as `#240` stated; still an estimate |
| 4 | 13% to 15% of a pretty whole read; about 20 times the floor | none | **Yes.** Established, `S`, identical bytes |
| 5 | ≤ 6% of a cached render, estimate | ≤ 6% today, ≤ 0.1% after row 1 | **Only if row 1 is not taken**, and after an experiment |
| 6 | ≤ 1.5% of a compact dump, ≤ 3.6% of a pretty one | none | **No.** At most twice the floor |
| 7 | ≤ 6.6% of a `--context` render, estimate | none | **Marginal**, as `#242` left it |
| 8 | at the floor | none | **No.** Insignificant |

**The campaign does not stop.** Its largest remaining gain, row 1, is 76% of
the canonical loop — 164 times `#240`'s floor for that loop — and rows 2 to 4
are each 20 to 100 times the floor of their own invocations. **It stops for
every other path**: start-up, help and version in all their forms, the error
paths, discovery, the `cfg` and `template` commands and every configuration
write, `init`, `cache status` and `cache clean`, the single-object and
listing reads in both forms and through every alias, the direct reads and
renders, and the `--context` render beyond `#242`'s row 2a. Each candidate
left there is at or below the floor, or is a cost the specification requires.

### Refuted hypotheses and non-findings, stated so they are not rediscovered

- **The `open` share of a cached read is not path resolution.** A relative open
  is as slow as an absolute one (4.5 against 4.5 ms for 272 files), so
  `openat` on a directory descriptor would save nothing measurable.
- **The seven aliases are their canonical commands.** Each is within 0.021 ms
  of it on the start-up-class labels and within 0.036 ms on `schema tables`
  (7.7 ms, whose floor is 0.088 ms), and allocates within 12 B of it.
- **The group nodes, `tpl` bare, `-h` and `help` are the help path**: 1.84 to
  1.87 ms, against 1.844 ms for `--help`.
- **The error paths are start-up class**: the two `64`s 1.73 to 1.78 ms, the
  `78` 1.88 ms, the refused `69`s 1.96 to 1.98 ms, the two template `66`s 1.89
  to 1.90 ms. The `66` of an absent table with the server up is a read-through
  miss and a whole write, 62.2 ms, and is row 2's.
- **The `json` form of every command other than the whole reads costs what its
  `text` form costs**, to within 0.021 ms (0.041 ms for `cfg database test`,
  inside its 0.069 ms floor). `help --format json` is 0.087 ms above `tpl help`
  because it writes the whole tree, 74 234 B, where the text writes one node.
- **`-vvv`, `-q`, `--timeout` and `--set` add nothing measurable** to a cached
  render: +0.024, +0.037, −0.043 and +0.027 ms against 11.276 ms, inside the
  0.1 ms floor.
- **`render --view` and `render --routine` are `render --table`**: 11.156 and
  11.277 ms. All three read the whole store (row 1).
- **`schema routine` with a bare name reads the 40 routine files** (2.486 ms
  against 1.999 ms for `schema table`), because a bare name is a collection
  read under `FR-SCH-010`'s ambiguity (`src/cache.rs:432`). It is required.
- **`cache load --table` reads the whole catalogue to write one file**
  (23.5 ms), `#231`'s non-finding, unchanged.
- **`probe/whole_json` is `minijinja`'s**: the serialisation under its `json`
  filter is 57.8% of the samples. `probe/whole_walk` is the evaluator's, as `#240` found.
- **Nothing on the start-up, help, configuration or template paths moved since
  `#240`**, and nothing regressed anywhere.

### Confounders — read this before the tables

- **The host was not idle.** The four containers of other projects named in
  `#240` ran throughout; two `osascript` processes used 14% to 26% of a core
  each. The load average was 1.98 to 3.18 during the main campaign and 3.86 at
  the start of the experiments. The rotation and the A/A arms bound this; they
  do not remove it. `hyperfine` printed 241 warnings in the main campaign and
  63 in the experiments, about statistical outliers or a slow first run.
- **The first run of the two JSON forms added after the main campaign was
  discarded**: its rsd was 9.6% to 10.7%, against 2% to 3% on the same labels
  in the main campaign, and `view_c`, measured beside them as a control, read
  1.999 ms against 1.952 ms. It was re-run at once, with an rsd of 5.3% to 9.6%
  and `view_c` at 1.954 ms, and those are the figures shown. They carry more
  spread than the main campaign's.
- **`b240` is a rebuild, and is `#241`'s `before` byte for byte**; `now` is
  `#241`'s `after` byte for byte. The change against `b240` is therefore
  `#241`'s, measured a second time.
- **The experiments' `ctl243` is not `now`**: the same source, built from a
  scratch path, with another sha256. It measured `rexample_c` at 11.327 ms where
  `now` measured 11.276 ms in the main campaign. Every variant is compared with
  it, never with `now`.
- **The loop's A/A in the experiments was 23.5 ms (0.97%)**, twice the main
  campaign's 6.2 ms and above `#240`'s 0.5%. Row 1's −1 833 ms is 78 times it.
- **`dump_dw` is noisy**: its A/A was 1.27 ms in the experiments. Row 2's
  −33.5 ms on it is 26 times that; its figure in the main table carries that
  spread.
- **`samply` shares carry the profiler's overhead, overlap, and are
  function-level**: the presymbolicated tables name no inlined function, so the
  parser tree appears under its callers. The error paths drew 4 to 32 on-CPU
  samples over 200 iterations, too few to attribute anything; their wall time
  is `hyperfine`'s.
- **`dhat` counts the heap only**, one run per label, through an instrumented
  build, and is never used as time. The `dhat` runs and the memory readings
  were taken while the variants were building; neither is a time.
- **The probes are this task's own**, and `probe/whole_walk` is not `#240`'s
  template of that name.
- **The server path crosses Docker Desktop's port proxy**, and the benchmark
  projects authenticate as `root` (`#224`). `up.sh 12.3` exited `2` on success
  (`#223`), and the filtered gate `status.sh --quiet 12.3`, which answered `0`,
  was read instead.

### What was not measured — stated, not implied

- **Three of the four targets.** Row 2's saving is system calls and row 1's is
  mostly `open`, whose cost differs by operating system and filesystem.
- **Every TLS mode.** All server reads used `tls = "disabled"`.
- **The series `10.11`, `11.4` and `11.8`**, raised only for the test suite.
- **A connection that times out** rather than being refused, which
  `core.connect_timeout` bounds at 10 s by design; `password_command`; the
  `--dsn`, `--password-command`, `--ca-file` and `--ca-path` forms of
  `cfg database add` and `update`; `cache load` and `cache clean` with `--view`
  or `--routine`; the single-object and listing reads with `--direct` and
  without `--no-cache`; and the `65` of a template that fails to compile or of
  an expired render deadline.
- **`bnd243` for `--view` and `--routine`**, and any lazy implementation of
  row 1: the figure is an upper bound from one instrument.
- **`skp243` over a changed catalogue**, whose cost is an estimate from the
  harness.
- **Rows 3, 5, 6 and 7 as experiments.** They are estimates.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
cargo build --release; cp target/release/tpl "$S/tpl-now"           # the binary of record
mkdir -p "$S/src/ab091db"; git archive ab091db | tar -x -C "$S/src/ab091db"
(cd "$S/src/ab091db" && cargo build --release --target-dir "$S/t-ab091db"); cp "$S/t-ab091db/release/tpl" "$S/tpl-b240"

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3   # read the gate, not up.sh
./scripts/mariadb/seed-bench.sh 12.3

# 2. The inventory, from the binary.
"$S/tpl-now" help --format json | jq -r '.data.commands[] | (.path | join(" ")) + "\t" + (.aliases | join(","))'

# 3. The projects: startup and server as #231's step 2, with tpl-now; the rust/
#    templates and the four probes copied into server; mut, dead, empty as the
#    Workload table states; the dump for --context with `tpl-now -d bench_wl001 schema dump`.

# 4. Wall time: for round r of 8, the 114 labels rotated by r, the three arms rotated by r:
hyperfine -N -i --warmup 5 --runs 40 --export-json "$S/c1/<label>.r<r>.json" \
  [--prepare "<restore>"] [--input "$S/work/wl001-dump.json"] \
  -n b240 "$S/tpl-b240 <args>" -n now "$S/tpl-now <args>" -n now_twin "$S/tpl-now <args>"
#    the loop, 4 rounds:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-now -d bench_wl001 cache clean" \
  -n b240 "benches/loop200.sh $S/tpl-b240 bench_wl001 example $S/work/wl001-tables.txt" …

# 5. The experiments: copies of `git archive 14d129f` under paths of equal length,
#    one edit each as the Candidates table states, each with its own --target-dir;
#    the same rotation over (ctl243 bnd243 ind243 skp243 ctl243_twin) on 18 labels,
#    and the loop over (ctl243 bnd243 ctl243_twin).

# 6. Profiles, allocation, memory.
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false \
  cargo build --release --target-dir "$S/t-prof"                  # from the scratch copy
samply record -s -r 20000 --iteration-count 30 --reuse-threads --unstable-presymbolicate \
  -o "$S/prof/rexample_c.json.gz" -- "$S/t-prof/release/tpl" -d bench_wl001 render example --table accrual
/usr/bin/time -l "$S/tpl-now" -d bench_wl001 render example --table accrual >/dev/null

# 7. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Row 2 of the third register applied: an identical object file is left in place

*Sprint 19, task `#244`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

`store_object`, `src/cache.rs`, now serialises an object into a reused buffer
and leaves the target in place where it is a regular file of mode `0600`, of
the same length, holding the same bytes; any other target is written through
the temporary file and the rename, as before. `meta.json` is still written on
every load, through `store`. This is `FR-CACHE-030` as amended in the
thirty-ninth edition.

**Over an unchanged `WL-001`, every whole write of the store halves:** `cache
load` 61.58 → 30.74 ms (−50.1%), a `66` with the server up 63.05 → 30.60 ms
(−51.5%), `schema dump --direct` 64.47 → 33.17 ms (−48.6%). That is `#243`'s
`skp243` reproduced on the shipped code, within 0.6 ms of each of its figures.
**Over a store whose every object differs in its last byte, with the length
unchanged, `cache load` is 2.52 ms slower (+4.0%)**: the worst case, where every
file is read and compared and then rewritten. Where the lengths differ, the
comparison is one `lstat` per file, and the change is inside the noise.

### Candidates

| arm | binary |
|---|---|
| `before` | `target/release/tpl` at `7c0fdaf`: 4 000 448 B, sha256 `53ed3470…ab16` — `#243`'s `now`, byte for byte (`7c0fdaf` changes only `BENCHMARKS.md`) |
| `after` | `target/release/tpl` with this change: 4 000 448 B, sha256 `74c73ace…a6a2` |
| `before_twin` | the `before` file, measured as a third label: the A/A arm |

### Environment

As `#243`'s: Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0
`arm64`; `rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of
`ADR-004`; `hyperfine` 1.20.0 (`-N`); `/usr/bin/time -l`. The fixture's `12.3`
alone during timing, seeded by `scripts/mariadb/seed-bench.sh`, `tls =
"disabled"`, account `root`; the `server` project built by
`benches/fixture.sh` and primed with `before`. Mains power, not charging; load
1.72 to 2.60. Taken 2026-09-23, 07:25Z to 07:27Z (UTC).

### Protocol

`#243`'s: 8 rounds, the labels rotated by one position per round and the three
arms rotated inside one `hyperfine` call per label per round. The store writers
5 runs after 1 warmup per arm per round, 40 samples; `cloadtable` 20 after 5,
160 samples. Between rounds the store was rewritten by `before`, so every
unchanged-catalogue label starts from the store a load writes.

The two changed-store labels restore, in `--prepare`, a copy of the `WL-001`
store whose 271 object files were altered, `meta.json` untouched:
`cload_changed` changes each file's closing `}` to `]`, the same length;
`cload_grown` adds one byte before the final newline. The restore writes each
file to a temporary and renames it over the target, as `tpl` does.

**The restore is load-bearing, and `cp -Rp` must not be used for it.** Over a
store restored by `cp -Rp`, reading the files costs about 20 ms more on the
first pass: `after` over an **identical** store took 50.8 ms after `cp -Rp`
against 31.5 ms after the rename-based restore, and the changed store +25 to
+29 ms against +4.9 ms. The cause was not investigated; the figures below use
the rename-based restore only.

**Byte identity**, before any timing. The `WL-001` store, 272 files, was
snapshotted by path, mode and sha256 (`meta.json` without `loaded_at`) after
five sequences: `before` from empty; then `after` over it; `after` from empty;
then `before` over it; then `after schema dump --direct` over it. All five
snapshots matched, every file at mode `600`, and no temporary was left. The
inode of `tables/accrual.json` was unchanged by `after`'s load over `before`'s
store. The stdout of `schema dump --direct` matched between the arms.

### Results

Medians; the A/A column is `before` against `before_twin`.

| label | command | `before` | `after` | change | A/A | p90, `before` → `after` |
|---|---|---|---|---|---|---|
| `cload` | `tpl -d bench_wl001 cache load` | 61.578 ms | **30.735 ms** | **−30.843 ms, −50.1%** | 0.206 ms | 64.54 → 31.34 ms |
| `fail66` | `tpl -d bench_wl001 schema table accrualx`, server up | 63.054 ms | **30.602 ms** | **−32.452 ms, −51.5%** | 0.920 ms | 65.02 → 31.01 ms |
| `dump_dw` | `tpl -d bench_wl001 schema dump --direct` | 64.467 ms | **33.167 ms** | **−31.300 ms, −48.6%** | 0.176 ms | 66.83 → 33.72 ms |
| `cloadtable` | `tpl -d bench_wl001 cache load --table accrual`, two objects | 23.466 ms | 23.359 ms | −0.107 ms | 0.035 ms | 23.99 → 23.89 ms |
| `cload_changed` | `cache load` over a store differing in every object, same lengths | 62.541 ms | 65.063 ms | **+2.522 ms, +4.0%** | 0.193 ms | 64.55 → 68.20 ms |
| `cload_grown` | `cache load` over a store differing in every object's length | 62.676 ms | 62.403 ms | −0.273 ms | 0.269 ms | 64.61 → 64.70 ms |

Peak resident memory, `/usr/bin/time -l`, median of 7: `cload` 8.22 →
8.52 MiB (+0.30 MiB), `dump_dw` 8.42 → 8.56 MiB (+0.14 MiB). The two buffers
grow to the largest object and are reused; they replace the 8 KiB `BufWriter`
the object write used before.

- **An unchanged object costs about 114 µs less**: 30.84 ms over the 271
  object files, `database.json` among them. `cloadtable` writes two unchanged
  objects, `database.json` and the table's, and its −0.107 ms is three times
  its A/A; it is not attributed further.
- **The worst case is +9.3 µs per object**: 2.52 ms over 271 files, the read
  and comparison of a file that is then rewritten anyway. `#243` estimated
  ≈ +3 ms; this is the measurement. It is reached only when every object keeps
  its length and changes its bytes.
- **A changed length costs one `lstat`**, and `cload_grown` is inside its A/A.
- **The spread falls with the median.** The standard deviation of `cload` was
  6.81 ms for `before` and 0.43 ms for `after`: the renames carried the tail.

### Not measured

- A store evicted from the page cache: the comparison would then read from the
  disk, for files whose length matches.
- The canonical loop, whose one write follows `cache clean`, so every target
  is absent and costs one failed `lstat`.
- The series `10.11`, `11.4` and `11.8`, raised only for the test suite.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
mkdir -p "$S/src/7c0fdaf"; git archive 7c0fdaf | tar -x -C "$S/src/7c0fdaf"
(cd "$S/src/7c0fdaf" && cargo build --release --target-dir "$S/t-7c0fdaf"); cp "$S/t-7c0fdaf/release/tpl" "$S/tpl-before"
cargo build --release; cp target/release/tpl "$S/tpl-after"

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3
./scripts/mariadb/seed-bench.sh 12.3

# 2. The server project, with benches/fixture.sh's functions:
#    fixture_inventory; fixture_address 12.3;
#    fixture_server_project "$S/work" "$S/tpl-before" disabled; fixture_prime "$S/work" "$S/tpl-before"

# 3. The altered stores: copies of .tpl/.cache/bench_wl001, every *.json but
#    meta.json edited in place (`}\n` -> `]\n`, and `}\n` -> `} \n`), restored in
#    --prepare by a script that writes each file to a temporary and renames it.

# 4. For round r of 8, the six labels rotated by r, the three arms rotated by r:
hyperfine -N -i --warmup 1 --runs 5 [--prepare "<restore>"] --export-json "$S/c/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n after "$S/tpl-after <args>" -n before_twin "$S/tpl-before <args>"
#    cloadtable: --warmup 5 --runs 20. After each round: tpl-before -d bench_wl001 cache load.

# 5. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Row 4 of the third register applied: each indent written in one call

*Sprint 19, task `#245`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

`--pretty` is now serialised through `Indented`, a formatter of the project's
own in `src/output/json.rs`, instead of `serde_json::to_writer_pretty`. It is
`PrettyFormatter`'s logic with the default two-space indent, except that the
comma, the newline and the whole indent of a separator are one `write_all` of
a slice of a static run covering 32 levels; a deeper level is written in more
than one call. `FR-OUT-008` is now held by a test that pins the formatter to
`PrettyFormatter` at every depth up to 131, rather than by the encoder's
default.

**The pretty cached dump of `WL-001` fell from 17.76 to 15.16 ms (−14.7%), and
the pretty table listing from 16.26 to 14.12 ms (−13.2%), with the same bytes
out.** That is `#243`'s `ind243` reproduced on the shipped code, within 0.13 ms
of each of its figures. The compact control is inside its noise.

### Candidates

Both arms were built from `git archive 31fd7d7` in a scratch directory, from
paths of the same length, each into its own target directory; `after` carries
this change's `src/output/json.rs` and nothing else.

| arm | binary |
|---|---|
| `before` | `31fd7d7`: 4 000 432 B, sha256 `68ec49bb…1482` |
| `after` | `31fd7d7` with this change: 4 000 432 B, sha256 `a82d977a…9ce5` |
| `before_twin` | the `before` file, measured as a third label: the A/A arm |

The in-repository `target/release/tpl` with this change is 4 000 448 B, sha256
`ee48fd0a…de93`; the 16 B are the longer source path embedded in it.

### Environment

As `#243`'s: Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0
`arm64`; `rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of
`ADR-004`; `hyperfine` 1.20.0 (`-N`); `/usr/bin/time -l`. The fixture's `12.3`
alone during timing, seeded by `scripts/mariadb/seed-bench.sh`, `tls =
"disabled"`, account `root`; the `server` project built by
`benches/fixture.sh` and primed with `before`. Mains power, not charging; load
2.49 to 2.75. Taken 2026-09-23, 07:57Z to 07:58Z (UTC).

### Protocol

`#243`'s: 8 rounds, the three labels rotated by one position per round and the
three arms rotated inside one `hyperfine` call per label per round; 40 runs
after 5 warmups per arm per round, 320 samples per arm and label.

**Byte identity**, before any timing. Every command that declares `--pretty`
was run with both arms and stdout, stderr and the exit code compared: 975
invocations, every one identical.

| sweep | invocations | what it covers |
|---|---|---|
| `WL-001` and `WL-003` on `12.3` | 617 | `help --format json --pretty` bare and at each of the 34 node paths; for both entries, cached and `--direct --no-cache`: `schema info`, `tables`, `tables --pattern`, `views`, `routines`, `dump`, and `schema table`, `view`, `routine` for every object, plus an absent table (`66`); `cache status`, `cfg database show`, `cfg database test` per entry; `template list`, `template path` (and its `66`), `cfg get`, `cfg list`, `cfg database list` |
| `freight` on all five servers | 358 | per server, cached and `--direct --no-cache`: the four listings, `dump`, and every table, view and routine; `cache status`, `cfg database show`, `cfg database test`; `cfg list`, `cfg database list`, `template list` |

41.2 MB and 25.7 MB of pretty output respectively; the deepest indent reached
is 12 levels.

### Results

Medians; the A/A column is `before` against `before_twin`.

| label | command | `before` | `after` | change | A/A | p90, `before` → `after` | per-round change |
|---|---|---|---|---|---|---|---|
| `dumppretty_c` | `tpl -d bench_wl001 schema dump --pretty` | 17.762 ms | **15.155 ms** | **−2.607 ms, −14.7%** | 0.185 ms | 18.31 → 15.86 ms | −2.78 to −2.35 ms |
| `tablespretty_c` | `tpl -d bench_wl001 schema tables --format json --pretty` | 16.263 ms | **14.116 ms** | **−2.146 ms, −13.2%** | 0.045 ms | 16.71 → 14.52 ms | −2.22 to −2.01 ms |
| `dump_c` | `tpl -d bench_wl001 schema dump`, the compact control | 13.752 ms | 13.790 ms | +0.038 ms | 0.023 ms | 14.64 → 14.56 ms | −0.45 to +0.26 ms |

Peak resident memory of `dumppretty_c`, `/usr/bin/time -l`, median of 7:
13.13 → 13.05 MiB, inside the spread of the samples (12.94 to 13.17 MiB for `before`, 13.03 to 13.14 MiB for `after`).

- **The gain is the calls, not the bytes.** Every round moved the two pretty
  labels by 12 to 49 times their A/A, and the compact form, which does not
  reach the formatter, by less than its per-round spread.
- **The binary did not grow**: both arms are 4 000 432 B. `ind243`, `#243`'s
  instrument of the same change, was 16 512 B larger; it was not examined why.

### Not measured

- `help --format json --pretty`, which `#243` put at −0.035 ms, and the
  single-object pretty reads, where it found nothing.
- The series `10.11`, `11.4` and `11.8`, raised for the identity sweep and the
  test suite, not for timing.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
mkdir -p "$S/src/31fd7d7" "$S/src/aft0245"
git archive 31fd7d7 | tar -x -C "$S/src/31fd7d7"
git archive 31fd7d7 | tar -x -C "$S/src/aft0245"; cp src/output/json.rs "$S/src/aft0245/src/output/json.rs"
(cd "$S/src/31fd7d7" && cargo build --release --target-dir "$S/t-31fd7d7"); cp "$S/t-31fd7d7/release/tpl" "$S/tpl-before"
(cd "$S/src/aft0245" && cargo build --release --target-dir "$S/t-aft0245"); cp "$S/t-aft0245/release/tpl" "$S/tpl-after"

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3
./scripts/mariadb/seed-bench.sh 12.3

# 2. The server project, with benches/fixture.sh's functions:
#    fixture_inventory; fixture_address 12.3;
#    fixture_server_project "$S/work" "$S/tpl-before" disabled; fixture_prime "$S/work" "$S/tpl-before"

# 3. For round r of 8, the three labels rotated by r, the three arms rotated by r:
hyperfine -N --warmup 5 --runs 40 --export-json "$S/c/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n after "$S/tpl-after <args>" -n before_twin "$S/tpl-before <args>"

# 4. The identity sweep over freight and the pipeline need all five servers;
#    then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Row 1 of the third register applied: a cached render reads an object file when the template reaches it

*Sprint 19, task `#246`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

`tpl render` served from the cache now reads `meta.json`, `database.json`, the
listing of each collection and the bound object's file before the render, and
every other object file the first time the template reaches that object
(`FR-CACHE-038`). The `database` the template sees is still whole. A file that
is a miss when it is reached abandons the render, which has written nothing,
and the invocation is answered as any miss is: one server read, the cache
write, and a render from the server's document (`FR-CACHE-039`). An abandoned
render reads no further file, and its deadline no longer ends the process: the
render that follows has its own.

**A cached render bound to one object fell from 10.9–11.7 ms to 2.3–3.0 ms
(−8.6 to −8.8 ms, −75% to −80%), its peak resident memory from 13.5–14.2 MiB
to 4.2–4.9 MiB, and the canonical 200-render loop from 2 377.9 to 624.2 ms
(−73.8%).** That is `#243`'s `bnd243` upper bound (−9.1 ms, −76.0% on the
loop) within 0.5 ms per render, with the whole `database` still bound. A
template that reads the whole database pays for the bookkeeping: `{{ database |
json }}` is 0.42 ms (+1.6%) slower. A miss found during the render costs the
part of the render that preceded it.

### Candidates

Both arms were built from `git archive b9a33df` in a scratch directory, from
paths of the same length, each into its own target directory; `after` carries
this change's `src/` and nothing else.

| arm | binary |
|---|---|
| `before` | `b9a33df`: 4 000 432 B, sha256 `a82d977a…9ce5` — `#245`'s `after`, byte for byte (`b9a33df` is that change) |
| `after` | `b9a33df` with this change: 4 017 024 B, sha256 `2d5ca3c5…8e02` |
| `before_twin` | the `before` file, measured as a third label: the A/A arm |

The binary grew by 16 592 B. The in-repository `target/release/tpl` with this
change is also 4 017 024 B, sha256 `5f49ef9a…38fa`; the bytes differ where the
source path is embedded.

The figures are those of a second campaign, run after the abandoned render was
made to read no further file and to outlive its deadline. A first campaign over
the change without those two properties gave the same results within 0.25 ms
on every label but the loop, whose absolute level differed between the two
sessions (`before` 2 519.3 ms, `after` 638.9 ms, −74.6%); `rmiss_reach` was
+19.08 ms there.

### Environment

As `#243`'s: Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0
`arm64`; `rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of
`ADR-004`; `hyperfine` 1.20.0 (`-N`); `/usr/bin/time -l`. All five fixture
servers were up; timing reached only `12.3`, seeded by
`scripts/mariadb/seed-bench.sh`, `tls = "disabled"`, account `root`. The
`server` project was built by `benches/fixture.sh` and primed with `before`.
Mains power, not charging; load 2.34 to 2.90. Taken 2026-09-23 (UTC): the main
campaign 09:21Z to 09:25Z, the loop 09:25Z to 09:27Z, peak memory after. The
fixture was taken down and raised again between the two campaigns, and
`WL-001` and `WL-003` seeded again.

Probe templates, written into the `server` project and not in the repository,
as `#243` wrote them: `probe/whole_json` (`{{ database | json }}`, 3 182 266 B),
`probe/whole_walk` (the recursive walk, 979 214 B — `#243`'s output size),
`probe/view` and `probe/routine`.

### Protocol

`#243`'s: 8 rounds, the 17 labels rotated by one position per round and the
three arms rotated inside one `hyperfine` call per label per round. 40 runs
after 5 warmups per arm per round (320 samples); `rwwalk_c` 20 after 3 and
`rexample_d` 20 after 5 (160); the two miss labels 5 after 1 (40), with a
`--prepare` that replaces one object file with `{ torn` by a write to a
temporary and a rename, never by `cp`. The loop: `benches/loop200.sh`, 4 rounds
of 3 runs after 1 warmup (12 samples per arm), `--prepare "tpl -d bench_wl001
cache clean"`, as `#243`. Peak resident memory: median of 7.

**Byte identity**, before any timing. Every invocation was run with both arms
and stdout, the exit code and stderr compared; stderr after masking the
duration `-vvv` prints (`phase: render took …ms`). 10 393 invocations, every
one identical.

| sweep | invocations | what it covers |
|---|---|---|
| `freight` on `10.11`, `11.4`, `11.8`, `12.3` and the server without TLS | 4 180 | every worked template of `examples/*/templates/` (`go/`, `node/`, `python/`, `rust/`) and the `example` of `tpl init`, 19 in all: whole-database, bound with `--table`, `--view` and `--routine` (bare and qualified) to every object, with `-vvv`, `-q` and `--set`, and to an absent table; 2 590 exit `0`, 1 495 exit `65` (a template bound to an object it was not written for), 95 exit `66` |
| `WL-001` and `WL-003` on `12.3` | 6 213 | the same, over 200 tables, 30 views and 40 routines; 4 641 exit `0`, 1 534 exit `65`, 38 exit `66` |

25.2 MB and 138.7 MB of stdout respectively. No `meta.json` was rewritten
during either sweep, so every render was served from the cache and none was
abandoned. Both sweeps were run for each of the two `after` builds, with the
same result.

### Results

Medians; the A/A column is `before` against `before_twin`.

| label | command | `before` | `after` | change | A/A | p90, `before` → `after` | per-round change |
|---|---|---|---|---|---|---|---|
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual` | 11.021 ms | **2.336 ms** | **−8.684 ms, −78.8%** | 0.007 ms | 11.36 → 2.42 ms | −8.83 to −8.61 ms |
| `rexample_vvv_c` | the same with `-vvv` | 10.972 ms | **2.347 ms** | **−8.625 ms, −78.6%** | 0.132 ms | 11.32 → 2.42 ms | −8.84 to −8.58 ms |
| `rexample_q_c` | the same with `-q` | 11.061 ms | **2.342 ms** | **−8.719 ms, −78.8%** | 0.043 ms | 11.34 → 2.40 ms | −8.88 to −8.61 ms |
| `rexample_to_c` | the same with `--timeout 30` | 10.978 ms | **2.338 ms** | **−8.640 ms, −78.7%** | 0.025 ms | 11.31 → 2.43 ms | −8.81 to −8.51 ms |
| `rexample_set_c` | the same with `--set title=Accrual` | 10.973 ms | **2.337 ms** | **−8.636 ms, −78.7%** | 0.115 ms | 11.19 → 2.45 ms | −8.73 to −8.55 ms |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual` | 11.685 ms | **2.983 ms** | **−8.703 ms, −74.5%** | 0.023 ms | 11.97 → 3.06 ms | −8.89 to −8.59 ms |
| `rview_c` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary` | 11.015 ms | **2.254 ms** | **−8.761 ms, −79.5%** | 0.047 ms | 11.39 → 2.34 ms | −8.96 to −8.58 ms |
| `rroutine_c` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count` | 10.931 ms | **2.272 ms** | **−8.659 ms, −79.2%** | 0.049 ms | 11.26 → 2.37 ms | −8.86 to −8.57 ms |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema` | 17.493 ms | **16.761 ms** | **−0.732 ms, −4.2%** | 0.114 ms | 17.87 → 17.20 ms | −0.91 to −0.53 ms |
| `rwjson_c` | `tpl -d bench_wl001 render probe/whole_json` | 26.124 ms | **26.539 ms** | **+0.415 ms, +1.6%** | 0.174 ms | 26.43 → 26.92 ms | +0.17 to +0.81 ms |
| `rwwalk_c` | `tpl -d bench_wl001 render probe/whole_walk` | 105.232 ms | 104.915 ms | −0.317 ms | 0.069 ms | 107.57 → 106.63 ms | −1.11 to +0.52 ms |
| `rmiss_reach` | `probe/whole_json`, with the last table's file damaged | 50.142 ms | **68.003 ms** | **+17.861 ms, +35.6%** | 0.019 ms | 50.75 → 68.72 ms | +17.42 to +18.42 ms |
| `rmiss_bound` | `example --table accrual`, with the bound table's file damaged | 36.012 ms | **31.244 ms** | **−4.767 ms, −13.2%** | 0.195 ms | 36.55 → 31.57 ms | −5.29 to −4.27 ms |
| `rexample_x` | `render example --table accrual --context <dump>`, control | 8.028 ms | 7.991 ms | −0.037 ms | 0.024 ms | 8.21 → 8.14 ms | −0.12 to +0.01 ms |
| `rexample_d` | the same `--direct --no-cache`, control | 23.586 ms | 23.618 ms | +0.032 ms | 0.006 ms | 24.07 → 24.06 ms | −0.15 to +0.15 ms |
| `table_c` | `tpl -d bench_wl001 schema table accrual`, control | 1.949 ms | 1.950 ms | +0.001 ms | 0.021 ms | 2.02 → 2.02 ms | −0.04 to +0.04 ms |
| `dump_c` | `tpl -d bench_wl001 schema dump`, control | 13.354 ms | 13.370 ms | +0.017 ms | 0.016 ms | 13.65 → 13.80 ms | −0.10 to +0.11 ms |

The canonical loop (12 samples per arm):

| label | `before` | `after` | change | A/A | per-round change |
|---|---|---|---|---|---|
| loop | 2 377.9 ms | **624.2 ms** | **−1 753.7 ms, −73.8%** | 0.8 ms | −1 766.2 to −1 743.3 ms |

Peak resident memory, `/usr/bin/time -l`, median of 7:

| label | `before` | `after` |
|---|---|---|
| `rexample_c` | 13.70 MiB | 4.52 MiB |
| `rstruct_c` | 14.17 MiB | 4.92 MiB |
| `rview_c` | 13.50 MiB | 4.16 MiB |
| `rroutine_c` | 13.56 MiB | 4.17 MiB |
| `rschema_c` | 14.27 MiB | 13.77 MiB |
| `rwjson_c` | 42.27 MiB | 43.08 MiB |
| `rwwalk_c` | 37.58 MiB | 36.48 MiB |
| `rexample_x`, control | 13.47 MiB | 13.27 MiB |
| `rmiss_reach` | 41.92 MiB | 70.48 MiB |

- **A bound render now costs what `schema table` costs plus the render.**
  2.25 to 2.98 ms against `table_c`'s 1.95 ms: the three files read up front
  (`meta.json`, `database.json` and the bound object's), three directory
  listings, and the template. Every round moved every bound
  label by more than 100 times its A/A.
- **`rust/schema` reads only `database.tables`**, so the 70 view and routine
  files are no longer read: −0.73 ms, about 10 µs per file, an attribution by
  arithmetic and not by profile.
- **A template that reads every file pays for the bookkeeping.** `rwjson_c`
  reads all 270 object files as before and is 0.42 ms slower, positive in every
  round, and 0.8 MiB heavier at its peak. The cause was
  not attributed; the candidates are the per-member path and name kept from the
  listing, the per-member `Arc` and slot, and the file reads interleaved with
  the render instead of preceding it. `rwwalk_c` reads the same files and its
  change is inside its per-round spread.
- **A miss found during the render costs the render it abandoned.**
  `rmiss_reach` is the costly case: `probe/whole_json` reads 199 of the 200
  tables and converts them before it reaches the damaged last one, then reads
  the server and renders again; the abandoned render's memory is not freed, so
  the peak is 28.6 MiB higher. The `json` filter already fails at the miss, so
  stopping the abandoned render saves nothing here: the cost is the work before
  the miss, and the change against the first campaign (+19.08 → +17.86 ms) is
  not attributed. `rmiss_bound` is the miss found before the render, which
  `before` found only after reading all 272 files: it is now 4.8 ms cheaper.

### Not measured

- A miss removed rather than damaged, and a miss reached near the render
  deadline; the second is held by a unit test of the deadline, not timed.
- The series `10.11`, `11.4` and `11.8`, raised for the identity sweep and the
  test suite, not for timing.
- A profile of `rwjson_c`'s +0.51 ms.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
mkdir -p "$S/src/b9a33df" "$S/src/aft0246"
git archive b9a33df | tar -x -C "$S/src/b9a33df"
git archive b9a33df | tar -x -C "$S/src/aft0246"
for f in $(git diff --name-only b9a33df -- src); do cp "$f" "$S/src/aft0246/$f"; done
(cd "$S/src/b9a33df" && cargo build --release --target-dir "$S/t-b9a33df"); cp "$S/t-b9a33df/release/tpl" "$S/tpl-before"
(cd "$S/src/aft0246" && cargo build --release --target-dir "$S/t-aft0246"); cp "$S/t-aft0246/release/tpl" "$S/tpl-after"

# 1. The fixture, through its harness only.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/seed-bench.sh 12.3

# 2. The server project, with benches/fixture.sh's functions:
#    fixture_inventory; fixture_address 12.3;
#    fixture_server_project "$S/work" "$S/tpl-before" disabled; fixture_prime "$S/work" "$S/tpl-before";
#    fixture_subjects "$S/work" "$S/tpl-before"; the rust/ templates and the four probes copied in.

# 3. For round r of 8, the 17 labels rotated by r, the three arms rotated by r:
hyperfine -N -i --warmup 5 --runs 40 --export-json "$S/c/<label>.r<r>.json" \
  [--prepare "<replace one object file with '{ torn' by temporary and rename>"] \
  -n before "$S/tpl-before <args>" -n after "$S/tpl-after <args>" -n before_twin "$S/tpl-before <args>"
#    the loop, 4 rounds:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-before -d bench_wl001 cache clean" \
  -n before "benches/loop200.sh $S/tpl-before bench_wl001 example $S/work/wl001-tables.txt" …

# 4. Peak memory:
/usr/bin/time -l "$S/tpl-after" -d bench_wl001 render example --table accrual >/dev/null

# 5. The identity sweep: a project with one entry per series on freight and two on
#    WL-001 and WL-003, every template of examples/*/templates/ copied in, and each
#    invocation run with both arms, stdout, stderr and the exit code compared.

# 6. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Fourth waste-hunting pass: the whole command surface at `ba10788`

*Sprint 19, task `#247`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

Every node of the command tree, every alias, both output forms of every command
that has two, every help form, the version forms and the error paths were
measured again at `ba10788`, after `#244`, `#245` and `#246`, with `#243`'s
instruments, protocol, labels and fixture, and with `#243`'s binary of record
as the baseline arm of the same rotated campaign. Nothing was changed: `src/`,
`Cargo.toml` and `Cargo.lock` are untouched, and every instrumented or altered
binary was built from a throwaway copy of the crate outside the repository.

**Against `#243`'s binary, measured side by side, the three changes are
confirmed and nothing else moved.** A cached render bound to one object fell
from 11.13–11.86 ms to 2.32–3.03 ms (−8.76 to −8.87 ms, −74% to −79%), and the
canonical 200-render loop from 2 434.7 to 628.8 ms (−74.2%); every whole write
of an unchanged store halved (`cache load` 64.50 → 30.57 ms); the two pretty
whole reads fell by 2.27 and 2.64 ms (−14%). The 64 start-up-class labels are
within 0.026 ms of `#243`, the 10 writers and server probes within 0.047 ms,
and the direct reads within 0.17 ms, each at the floor of its class.

**One new row is above the noise floor by two orders of magnitude, and was
established by a single-variable experiment:**

- **A lookup of a table by name reads the file of every table listed before
  it.** `is primary_key`, `is unique` and the global `column()` resolve the
  column's table by a linear scan of `database.tables`, and under `#246` each
  member the scan passes is read, validated and decoded from its file. Answering
  a lookup of the bound table from the bound table took `rust/struct --table
  yard_position`, the last of 200, from 11.41 to 3.37 ms (−70.5%), and a
  `rust/struct` loop over all 200 tables from 1 604.9 to 785.6 ms (−51.0%).
  The canonical loop renders `example`, which makes no lookup, and is not
  affected.

**The campaign does not stop**: that row is 259 times the A/A of the
invocation it affects and 167 times the A/A of the loop it affects, on eight
of the worked templates of `examples/`. Its first step is a question for the
specification, not code. It stops for every other path; see *The verdict*.

### The surface

`#243`'s inventory, unchanged: `tpl help --format json` lists 34 nodes, 5
groups and 29 leaves, and 7 aliases. `#243`'s 116 labels were measured under
the same names, with the same invocations, projects and probe templates, and
three labels were added:

| label | what it is for |
|---|---|
| `rstruct_last_c` | `render rust/struct --table yard_position`: the same template as `rstruct_c` bound to the **last** table of the listing instead of the first, which is where the lookup of the new row costs most |
| `rmiss_reach` | `#246`'s label: `probe/whole_json` with the last table's file replaced by `{ torn`, a miss reached during the render (`FR-CACHE-039`) |
| `rmiss_bound` | `#246`'s label: `example --table accrual` with the bound table's file damaged, a miss found before the render |

Beside the canonical loop, a second loop, `loop_struct`, renders
`rust/struct` once per table with `benches/loop200.sh`, as the canonical loop
renders `example`.

**How the leaves map to labels** is `#243`'s table, *The surface*, row for row;
`render` now has 26 labels and the two loops. Where a family is reported as
one line below, it is because its labels run one path; that is said in each
line.

### Results — per family, against `#243`

Medians; the full per-label table follows. A/A is `now` against `now_twin`.

| family | labels | `b243` → `now` | change | largest A/A in the family |
|---|---|---|---|---|
| `version`, `-V`, `version` | 3 | 1.721–1.738 → 1.718–1.736 ms | within 0.003 ms | 0.004 ms |
| `help` in every form, `tpl` bare, the 5 groups bare and `cfg db` | 15 | 1.820–1.996 → 1.816–1.991 ms | within 0.025 ms | 0.010 ms |
| error paths: `64` twice, `78`, `69` twice, the two template `66`s | 7 | 1.723–1.973 → 1.717–1.971 ms | within 0.026 ms | 0.022 ms |
| `template list`, `show`, `check`, `path`, both forms | 7 | 1.836–1.964 → 1.831–1.962 ms | within 0.015 ms | 0.011 ms |
| `cfg list`, `get`, `database list`, `show`, both forms and the alias | 9 | 1.856–1.883 → 1.851–1.883 ms | within 0.020 ms | 0.017 ms |
| `cfg set`, `unset`, `database add`, `update`, `remove`, `init` | 6 | 1.983–2.150 → 1.988–2.131 ms | within 0.018 ms | 0.020 ms |
| `cfg database test`, both forms | 2 | 3.976–4.006 → 3.959–3.988 ms | within 0.047 ms | 0.017 ms |
| `cache status`, `cache clean` with and without `--table` | 4 | 2.051–2.394 → 2.050–2.362 ms | within 0.042 ms | 0.014 ms |
| `schema info`, three forms | 3 | 2.082–2.090 → 2.074–2.097 ms | within 0.016 ms | 0.013 ms |
| cached single-object reads: `table`, `view`, `routine`, every form, alias and `--tpl-dir`; the two collection listings `views` and `routines` | 18 | 1.933–2.506 → 1.930–2.511 ms | within 0.016 ms | 0.023 ms |
| cached `schema tables` (text, alias, `--pattern`) and compact `--format json`; compact `schema dump` | 5 | 7.61–13.59 → 7.63–13.57 ms | within 0.077 ms | 0.088 ms |
| **pretty whole reads**: `schema dump --pretty`, `schema tables --format json --pretty` | 2 | 16.14–17.66 → 13.88–15.02 ms | **−2.27 and −2.64 ms (−14%), `#245`** | 0.055 ms |
| direct reads, `--direct --no-cache`, every `schema` leaf | 9 | 11.55–25.98 → 11.38–25.87 ms | within 0.17 ms | 0.093 ms |
| **whole store writes**: `cache load`, the `66` with the server up, `schema dump --direct` | 3 | 64.2–66.8 → 30.5–32.9 ms | **−33.7 to −33.9 ms (−51% to −53%), `#244`** | 0.094 ms |
| `cache load --table` | 1 | 23.31 → 23.00 ms | −0.31 ms, 2.8 times its A/A; `#244`'s two unchanged objects | 0.111 ms |
| **cached renders bound to one object**: `example` with `-vvv`, `-q`, `--timeout`, `--set` and without, `rust/struct` on the first table, `--view`, `--routine` | 8 | 11.13–11.86 → 2.32–3.03 ms | **−8.76 to −8.87 ms (−74% to −79%), `#246`** | 0.021 ms |
| cached `rust/struct` on the last table | 1 | 12.40 → 11.67 ms | −0.73 ms (−5.9%): `#246`'s saving, mostly spent again by the new row | 0.042 ms |
| cached whole-database renders: `rust/schema`, `probe/whole_json`, `probe/whole_walk` | 3 | 17.72 / 26.18 / 105.04 → 17.05 / 26.64 / 105.47 ms | −0.67, **+0.46** and +0.43 ms | 0.324 ms |
| `--context` renders, file and standard input | 5 | 8.08–22.74 → 8.07–22.72 ms | within 0.049 ms | 0.088 ms |
| direct renders | 6 | 23.54–38.39 → 23.46–38.42 ms | within 0.10 ms | 0.112 ms |
| a miss found before the render, `rmiss_bound` | 1 | 70.06 → 31.30 ms | −38.8 ms: `#244`'s write and `#246`'s early miss | 0.168 ms |
| a miss reached during the render, `rmiss_reach` | 1 | 83.70 → 67.89 ms | −15.8 ms: `#244`'s write, less `#246`'s +17.9 ms | 0.037 ms |
| **the canonical loop**, 200 × `example` | 1 | 2 434.7 → 628.8 ms | **−1 805.8 ms (−74.2%), `#246`** | 0.7 ms |
| the `rust/struct` loop, 200 × `rust/struct` | 1 | 2 586.0 → 1 618.1 ms | **−967.8 ms (−37.4%)** | 2.2 ms |

### Results — every path, against `#243`

Medians; `rsd` is of `now`. A change is in bold where it is at least 0.1 ms and
more than twice the A/A difference of its label. `<w>` is the scratch work
directory. The heap is `dhat`'s, one run: total allocated, and held at the
peak. Peak RSS is `/usr/bin/time -l`, median of 7 per arm.

| label | invocation | exit | `b243` | `now` | change | A/A | `now` rsd | heap total / at peak, `now` | peak RSS, `b243` → `now` |
|---|---|---|---|---|---|---|---|---|---|
| `version` | `tpl --version` | 0 | 1.723 ms | **1.725 ms** | +0.002 ms | 0.004 ms | 3.32% | 372 219 / 260 186 B | 2.62 → 2.59 MiB |
| `version_V` | `tpl -V` | 0 | 1.721 ms | **1.718 ms** | −0.003 ms | 0.001 ms | 2.62% | 372 205 / 260 172 B | — |
| `version_cmd` | `tpl version` | 0 | 1.738 ms | **1.736 ms** | −0.003 ms | 0.003 ms | 2.48% | 379 201 / 262 460 B | — |
| `bare` | `tpl` | 0 | 1.835 ms | **1.839 ms** | +0.004 ms | 0.005 ms | 2.57% | 665 139 / 259 211 B | — |
| `help` | `tpl --help` | 0 | 1.843 ms | **1.840 ms** | −0.002 ms | 0.005 ms | 2.52% | 666 227 / 260 177 B | 2.97 → 3.00 MiB |
| `help_h` | `tpl -h` | 0 | 1.839 ms | **1.843 ms** | +0.005 ms | 0.007 ms | 2.64% | 666 219 / 260 169 B | — |
| `help_cmd` | `tpl help` | 0 | 1.864 ms | **1.864 ms** | +0.001 ms | 0.007 ms | 2.54% | 676 517 / 264 084 B | — |
| `helpjson` | `tpl help --format json` | 0 | 1.933 ms | **1.941 ms** | +0.008 ms | 0.003 ms | 2.26% | 738 383 / 264 156 B | — |
| `helpjson_p` | `tpl help --format json --pretty` | 0 | 1.996 ms | **1.972 ms** | −0.025 ms | 0.009 ms | 2.50% | 739 558 / 264 673 B | 3.06 → 3.17 MiB |
| `helppath` | `tpl help schema table` | 0 | 1.853 ms | **1.861 ms** | +0.008 ms | 0.003 ms | 2.80% | 668 018 / 264 704 B | — |
| `helppathjson` | `tpl help schema table --format json` | 0 | 1.820 ms | **1.818 ms** | −0.002 ms | 0.009 ms | 2.36% | 658 550 / 264 824 B | — |
| `nodehelp` | `tpl schema table --help` | 0 | 1.988 ms | **1.991 ms** | +0.002 ms | 0.010 ms | 2.34% | 1 402 971 / 670 751 B | 3.66 → 3.64 MiB |
| `g_schema` | `tpl schema` | 0 | 1.838 ms | **1.843 ms** | +0.005 ms | 0.004 ms | 2.32% | 766 046 / 332 393 B | — |
| `g_template` | `tpl template` | 0 | 1.827 ms | **1.825 ms** | −0.002 ms | 0.003 ms | 3.57% | 701 957 / 295 031 B | — |
| `g_cache` | `tpl cache` | 0 | 1.839 ms | **1.816 ms** | −0.023 ms | 0.007 ms | 2.44% | 697 987 / 291 680 B | — |
| `g_cfg` | `tpl cfg` | 0 | 1.827 ms | **1.834 ms** | +0.007 ms | 0.000 ms | 2.91% | 722 501 / 307 934 B | — |
| `g_cfgdb` | `tpl cfg database` | 0 | 1.840 ms | **1.853 ms** | +0.013 ms | 0.007 ms | 4.00% | 838 535 / 374 562 B | — |
| `g_cfgdb_alias` | `tpl cfg db` | 0 | 1.853 ms | **1.856 ms** | +0.003 ms | 0.005 ms | 2.67% | 838 523 / 374 550 B | — |
| `fail64` | `tpl -d bench_wl001 schema table accrual --no-such-flag` | 64 | 1.761 ms | **1.762 ms** | +0.001 ms | 0.000 ms | 2.58% | 478 252 / 329 994 B | — |
| `fail64_cmd` | `tpl schemx` | 64 | 1.723 ms | **1.717 ms** | −0.006 ms | 0.002 ms | 2.94% | 358 075 / 256 301 B | — |
| `fail78_noproj` | `tpl schema tables` | 78 | 1.858 ms | **1.884 ms** | +0.026 ms | 0.022 ms | 3.67% | 502 204 / 338 578 B | — |
| `fail69_conn` | `tpl schema tables` | 69 | 1.962 ms | **1.966 ms** | +0.004 ms | 0.012 ms | 2.61% | 561 995 / 338 578 B | 3.50 → 3.56 MiB |
| `fail69_conn_d` | `tpl schema table accrual --direct --no-cache` | 69 | 1.973 ms | **1.971 ms** | −0.002 ms | 0.003 ms | 3.61% | 566 770 / 340 923 B | — |
| `fail66_tpl` | `tpl template show nosuch` | 66 | 1.889 ms | **1.890 ms** | +0.001 ms | 0.000 ms | 2.59% | 442 491 / 299 951 B | — |
| `fail66_rnd` | `tpl render nosuch` | 66 | 1.901 ms | **1.894 ms** | −0.007 ms | 0.007 ms | 2.45% | 399 695 / 265 003 B | — |
| `fail66` | `tpl -d bench_wl001 schema table accrualx` | 66 | 64.225 ms | **30.511 ms** | **−33.714 ms, −52.5%** | 0.016 ms | 1.54% | 6 411 483 / 4 200 767 B | 8.38 → 8.78 MiB |
| `tlist` | `tpl template list` | 0 | 1.856 ms | **1.848 ms** | −0.008 ms | 0.003 ms | 3.20% | 446 615 / 300 012 B | 3.00 → 3.00 MiB |
| `tlistjson` | `tpl template list --format json` | 0 | 1.847 ms | **1.862 ms** | +0.015 ms | 0.008 ms | 3.22% | 447 083 / 300 108 B | — |
| `tlist_srv` | `tpl template list` | 0 | 1.865 ms | **1.860 ms** | −0.005 ms | 0.011 ms | 2.44% | 448 460 / 300 012 B | — |
| `tshow` | `tpl template show rust/schema` | 0 | 1.845 ms | **1.859 ms** | +0.015 ms | 0.011 ms | 4.30% | 460 441 / 299 971 B | — |
| `tcheck` | `tpl template check rust/schema` | 0 | 1.964 ms | **1.962 ms** | −0.002 ms | 0.003 ms | 2.54% | 510 331 / 299 975 B | — |
| `tpath` | `tpl template path rust/struct` | 0 | 1.836 ms | **1.831 ms** | −0.005 ms | 0.000 ms | 2.43% | 448 013 / 300 632 B | — |
| `tpathjson` | `tpl template path rust/struct --format json` | 0 | 1.840 ms | **1.837 ms** | −0.003 ms | 0.001 ms | 2.42% | 448 361 / 300 752 B | — |
| `cfglist` | `tpl cfg list` | 0 | 1.856 ms | **1.851 ms** | −0.005 ms | 0.016 ms | 2.50% | 482 092 / 312 891 B | 3.17 → 3.23 MiB |
| `cfglistjson` | `tpl cfg list --format json` | 0 | 1.862 ms | **1.861 ms** | −0.001 ms | 0.014 ms | 2.30% | 483 913 / 312 987 B | — |
| `cfgget` | `tpl cfg get core.database` | 0 | 1.856 ms | **1.877 ms** | +0.020 ms | 0.017 ms | 2.63% | 483 114 / 313 516 B | — |
| `cfggetjson` | `tpl cfg get core.database --format json` | 0 | 1.862 ms | **1.865 ms** | +0.003 ms | 0.008 ms | 3.31% | 483 453 / 313 636 B | — |
| `cfgset` | `tpl cfg set core.database bench_wl001` | 0 | 1.988 ms | **1.988 ms** | −0.000 ms | 0.003 ms | 2.28% | 496 882 / 313 759 B | — |
| `cfgunset` | `tpl cfg unset core.query_timeout` | 0 | 1.983 ms | **1.997 ms** | +0.015 ms | 0.020 ms | 2.96% | 495 693 / 312 880 B | — |
| `dblist` | `tpl cfg database list` | 0 | 1.880 ms | **1.874 ms** | −0.005 ms | 0.002 ms | 2.32% | 597 898 / 379 311 B | — |
| `dblist_alias` | `tpl cfg db list` | 0 | 1.883 ms | **1.866 ms** | −0.017 ms | 0.008 ms | 2.60% | 597 886 / 379 299 B | — |
| `dblistjson` | `tpl cfg database list --format json` | 0 | 1.872 ms | **1.879 ms** | +0.007 ms | 0.001 ms | 2.80% | 598 390 / 379 431 B | — |
| `dbshow` | `tpl cfg database show bench_wl001` | 0 | 1.876 ms | **1.883 ms** | +0.007 ms | 0.005 ms | 2.43% | 600 108 / 379 955 B | 3.30 → 3.33 MiB |
| `dbshowjson` | `tpl cfg database show bench_wl001 --format json` | 0 | 1.880 ms | **1.882 ms** | +0.002 ms | 0.006 ms | 2.47% | 600 120 / 380 075 B | — |
| `dbadd` | `tpl cfg database add scratch --host 127.0.0.1 --port 13309 --user root --schema freight_wl003 --tls disabled` | 0 | 2.047 ms | **2.033 ms** | −0.014 ms | 0.003 ms | 2.18% | 629 204 / 385 137 B | 3.36 → 3.45 MiB |
| `dbupdate` | `tpl cfg database update scratch --port 13309` | 0 | 2.028 ms | **2.021 ms** | −0.007 ms | 0.001 ms | 2.96% | 624 579 / 380 455 B | — |
| `dbremove` | `tpl cfg database remove scratch` | 0 | 2.022 ms | **2.032 ms** | +0.010 ms | 0.009 ms | 2.84% | 620 297 / 379 284 B | — |
| `dbtest` | `tpl cfg database test bench_wl001` | 0 | 4.006 ms | **3.959 ms** | −0.047 ms | 0.017 ms | 3.00% | 675 200 / 379 955 B | — |
| `dbtestjson` | `tpl cfg database test bench_wl001 --format json` | 0 | 3.976 ms | **3.988 ms** | +0.013 ms | 0.001 ms | 2.90% | 675 212 / 380 075 B | — |
| `init` | `tpl init <w>/initdir` | 0 | 2.150 ms | **2.131 ms** | −0.018 ms | 0.005 ms | 2.24% | 379 352 / 264 399 B | 2.75 → 2.75 MiB |
| `cstatus` | `tpl -d bench_wl001 cache status` | 0 | 2.051 ms | **2.063 ms** | +0.012 ms | 0.002 ms | 2.50% | 604 443 / 298 915 B | 3.31 → 3.36 MiB |
| `cstatusjson` | `tpl -d bench_wl001 cache status --format json` | 0 | 2.064 ms | **2.050 ms** | −0.014 ms | 0.003 ms | 2.55% | 587 744 / 299 035 B | — |
| `cload` | `tpl -d bench_wl001 cache load` | 0 | 64.501 ms | **30.572 ms** | **−33.930 ms, −52.6%** | 0.094 ms | 1.14% | 6 337 249 / 4 199 983 B | 8.41 → 8.59 MiB |
| `cloadtable` | `tpl -d bench_wl001 cache load --table accrual` | 0 | 23.305 ms | **22.995 ms** | **−0.309 ms, −1.3%** | 0.111 ms | 1.70% | 5 544 427 / 3 913 154 B | 8.23 → 8.19 MiB |
| `cclean` | `tpl -d bench_wl003 cache clean` | 0 | 2.394 ms | **2.362 ms** | −0.032 ms | 0.014 ms | 4.81% | 456 081 / 297 991 B | — |
| `ccleantable` | `tpl -d bench_wl003 cache clean --table consignment` | 0 | 2.258 ms | **2.216 ms** | −0.042 ms | 0.011 ms | 4.63% | 469 225 / 299 086 B | — |
| `info_c` | `tpl -d bench_wl001 schema info` | 0 | 2.082 ms | **2.097 ms** | +0.014 ms | 0.009 ms | 2.74% | 657 326 / 341 734 B | 3.44 → 3.47 MiB |
| `infojson_c` | `tpl -d bench_wl001 schema info --format json` | 0 | 2.090 ms | **2.074 ms** | −0.016 ms | 0.013 ms | 2.25% | 657 395 / 341 854 B | — |
| `infopretty_c` | `tpl -d bench_wl001 schema info --format json --pretty` | 0 | 2.090 ms | **2.087 ms** | −0.003 ms | 0.011 ms | 2.46% | 658 570 / 342 371 B | — |
| `tables_c` | `tpl -d bench_wl001 schema tables` | 0 | 7.613 ms | **7.690 ms** | +0.077 ms | 0.049 ms | 2.02% | 3 852 712 / 3 197 830 B | 7.14 → 7.14 MiB |
| `tables_alias_c` | `tpl -d bench_wl001 schema tbls` | 0 | 7.667 ms | **7.652 ms** | −0.016 ms | 0.003 ms | 2.02% | 3 852 708 / 3 197 830 B | — |
| `tablesjson_c` | `tpl -d bench_wl001 schema tables --format json` | 0 | 12.532 ms | **12.567 ms** | +0.036 ms | 0.030 ms | 1.92% | 11 616 766 / 8 139 408 B | 12.80 → 12.84 MiB |
| `tablespretty_c` | `tpl -d bench_wl001 schema tables --format json --pretty` | 0 | 16.144 ms | **13.875 ms** | **−2.269 ms, −14.1%** | 0.055 ms | 1.55% | 11 617 941 / 8 139 408 B | 12.83 → 12.86 MiB |
| `tablespat_c` | `tpl -d bench_wl001 schema tables --pattern acc%` | 0 | 7.626 ms | **7.631 ms** | +0.006 ms | 0.048 ms | 2.13% | 3 832 814 / 3 183 012 B | — |
| `table_c` | `tpl -d bench_wl001 schema table accrual` | 0 | 1.986 ms | **2.001 ms** | +0.014 ms | 0.009 ms | 2.62% | 621 578 / 342 365 B | 3.73 → 3.80 MiB |
| `table_alias_c` | `tpl -d bench_wl001 schema tbl accrual` | 0 | 1.987 ms | **1.982 ms** | −0.005 ms | 0.004 ms | 3.97% | 621 574 / 342 361 B | — |
| `tablejson_c` | `tpl -d bench_wl001 schema table accrual --format json` | 0 | 1.989 ms | **1.988 ms** | −0.002 ms | 0.003 ms | 2.41% | 575 980 / 342 485 B | — |
| `tablepretty_c` | `tpl -d bench_wl001 schema table accrual --format json --pretty` | 0 | 2.002 ms | **1.993 ms** | −0.009 ms | 0.006 ms | 2.50% | 577 155 / 343 002 B | — |
| `table3_c` | `tpl -d bench_wl003 schema table consignment` | 0 | 1.968 ms | **1.981 ms** | +0.013 ms | 0.018 ms | 2.44% | 849 845 / 342 381 B | — |
| `views_c` | `tpl -d bench_wl001 schema views` | 0 | 2.304 ms | **2.302 ms** | −0.002 ms | 0.010 ms | 2.15% | 606 942 / 341 801 B | — |
| `views_alias_c` | `tpl -d bench_wl001 schema vws` | 0 | 2.297 ms | **2.313 ms** | +0.016 ms | 0.014 ms | 2.47% | 606 938 / 341 797 B | — |
| `viewsjson_c` | `tpl -d bench_wl001 schema views --format json` | 0 | 2.302 ms | **2.301 ms** | −0.001 ms | 0.009 ms | 2.97% | 604 330 / 341 921 B | — |
| `view_c` | `tpl -d bench_wl001 schema view v_booking_line_summary` | 0 | 1.937 ms | **1.930 ms** | −0.007 ms | 0.009 ms | 2.41% | 541 644 / 342 422 B | 3.50 → 3.52 MiB |
| `view_alias_c` | `tpl -d bench_wl001 schema vw v_booking_line_summary` | 0 | 1.933 ms | **1.933 ms** | +0.001 ms | 0.005 ms | 2.38% | 541 640 / 342 418 B | — |
| `routines_c` | `tpl -d bench_wl001 schema routines` | 0 | 2.486 ms | **2.475 ms** | −0.010 ms | 0.005 ms | 1.85% | 734 131 / 341 810 B | — |
| `routines_alias_c` | `tpl -d bench_wl001 schema rtns` | 0 | 2.476 ms | **2.480 ms** | +0.004 ms | 0.006 ms | 2.30% | 734 123 / 341 802 B | — |
| `routinesjson_c` | `tpl -d bench_wl001 schema routines --format json` | 0 | 2.506 ms | **2.511 ms** | +0.005 ms | 0.005 ms | 2.60% | 729 519 / 341 930 B | — |
| `routine_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count` | 0 | 2.481 ms | **2.478 ms** | −0.003 ms | 0.010 ms | 1.97% | 739 205 / 342 451 B | 3.84 → 3.86 MiB |
| `routine_alias_c` | `tpl -d bench_wl001 schema rtn fn_consignment_hazard_count` | 0 | 2.477 ms | **2.493 ms** | +0.016 ms | 0.023 ms | 3.44% | 739 197 / 342 443 B | — |
| `dump_c` | `tpl -d bench_wl001 schema dump` | 0 | 13.590 ms | **13.574 ms** | −0.016 ms | 0.088 ms | 1.51% | 11 869 590 / 8 224 312 B | 13.11 → 13.14 MiB |
| `dumppretty_c` | `tpl -d bench_wl001 schema dump --pretty` | 0 | 17.659 ms | **15.019 ms** | **−2.640 ms, −14.9%** | 0.019 ms | 1.40% | 11 872 845 / 8 224 312 B | 13.12 → 13.19 MiB |
| `tpldir_c` | `tpl --tpl-dir <w>/server/.tpl -d bench_wl001 schema table accrual` | 0 | 1.996 ms | **1.994 ms** | −0.002 ms | 0.002 ms | 2.32% | 626 804 / 344 174 B | — |
| `info_d` | `tpl -d bench_wl001 schema info --direct --no-cache` | 0 | 23.383 ms | **23.323 ms** | −0.060 ms | 0.059 ms | 1.53% | 5 563 309 / 3 884 483 B | — |
| `tables_d` | `tpl -d bench_wl001 schema tables --direct --no-cache` | 0 | 23.461 ms | **23.474 ms** | +0.014 ms | 0.092 ms | 1.66% | 5 601 696 / 3 922 760 B | — |
| `table_d` | `tpl -d bench_wl001 schema table accrual --direct --no-cache` | 0 | 23.452 ms | **23.411 ms** | −0.041 ms | 0.079 ms | 2.20% | 5 610 178 / 3 887 643 B | 8.45 → 8.34 MiB |
| `table3_d` | `tpl -d bench_wl003 schema table consignment --direct --no-cache` | 0 | 11.549 ms | **11.382 ms** | −0.167 ms | 0.093 ms | 2.53% | 820 031 / 343 229 B | — |
| `views_d` | `tpl -d bench_wl001 schema views --direct --no-cache` | 0 | 23.295 ms | **23.377 ms** | +0.081 ms | 0.009 ms | 1.56% | 5 566 307 / 3 887 378 B | — |
| `view_d` | `tpl -d bench_wl001 schema view v_booking_line_summary --direct --no-cache` | 0 | 23.381 ms | **23.398 ms** | +0.017 ms | 0.045 ms | 1.57% | 5 564 686 / 3 884 472 B | — |
| `routines_d` | `tpl -d bench_wl001 schema routines --direct --no-cache` | 0 | 23.348 ms | **23.371 ms** | +0.023 ms | 0.001 ms | 1.40% | 5 568 408 / 3 889 458 B | — |
| `routine_d` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --direct --no-cache` | 0 | 23.308 ms | **23.268 ms** | −0.040 ms | 0.066 ms | 2.01% | 5 573 322 / 3 884 865 B | — |
| `dump_d` | `tpl -d bench_wl001 schema dump --direct --no-cache` | 0 | 25.976 ms | **25.865 ms** | **−0.111 ms, −0.4%** | 0.041 ms | 1.86% | 5 561 638 / 3 884 010 B | 8.23 → 8.28 MiB |
| `dump_dw` | `tpl -d bench_wl001 schema dump --direct` | 0 | 66.811 ms | **32.928 ms** | **−33.883 ms, −50.7%** | 0.066 ms | 1.41% | 6 415 005 / 4 200 751 B | — |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual` | 0 | 11.279 ms | **2.405 ms** | **−8.873 ms, −78.7%** | 0.015 ms | 2.39% | 762 108 / 269 312 B | 13.69 → 4.48 MiB |
| `rexample_vvv_c` | `tpl -vvv -d bench_wl001 render example --table accrual` | 0 | 11.247 ms | **2.404 ms** | **−8.843 ms, −78.6%** | 0.007 ms | 2.13% | 764 822 / 270 157 B | — |
| `rexample_q_c` | `tpl -q -d bench_wl001 render example --table accrual` | 0 | 11.162 ms | **2.401 ms** | **−8.761 ms, −78.5%** | 0.021 ms | 2.60% | 762 828 / 269 623 B | — |
| `rexample_to_c` | `tpl --timeout 30 -d bench_wl001 render example --table accrual` | 0 | 11.236 ms | **2.408 ms** | **−8.828 ms, −78.6%** | 0.005 ms | 2.12% | 764 825 / 270 395 B | — |
| `rexample_set_c` | `tpl -d bench_wl001 render example --table accrual --set title=Accrual` | 0 | 11.238 ms | **2.394 ms** | **−8.844 ms, −78.7%** | 0.013 ms | 2.23% | 764 449 / 269 958 B | — |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual` | 0 | 11.862 ms | **3.032 ms** | **−8.830 ms, −74.4%** | 0.004 ms | 1.66% | 2 249 423 / 410 201 B | 14.12 → 4.92 MiB |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema` | 0 | 17.720 ms | **17.049 ms** | **−0.671 ms, −3.8%** | 0.071 ms | 1.71% | 30 380 819 / 8 512 524 B | 14.16 → 13.97 MiB |
| `rview_c` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary` | 0 | 11.130 ms | **2.320 ms** | **−8.809 ms, −79.2%** | 0.004 ms | 2.41% | 658 018 / 269 381 B | 13.53 → 4.05 MiB |
| `rroutine_c` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count` | 0 | 11.189 ms | **2.318 ms** | **−8.871 ms, −79.3%** | 0.011 ms | 2.91% | 659 409 / 269 422 B | 13.50 → 4.09 MiB |
| `rwjson_c` | `tpl -d bench_wl001 render probe/whole_json` | 0 | 26.180 ms | **26.641 ms** | **+0.462 ms, +1.8%** | 0.059 ms | 1.13% | 48 462 722 / 34 115 196 B | 42.20 → 43.20 MiB |
| `rwwalk_c` | `tpl -d bench_wl001 render probe/whole_walk` | 0 | 105.037 ms | **105.466 ms** | +0.429 ms | 0.324 ms | 0.89% | 824 088 933 / 29 621 960 B | 37.66 → 36.80 MiB |
| `rexample_x` | `tpl render example --table accrual --context <w>/wl001-dump.json` | 0 | 8.089 ms | **8.123 ms** | +0.034 ms | 0.021 ms | 1.97% | 12 561 860 / 8 244 432 B | 13.36 → 13.44 MiB |
| `rexample_xin` | `tpl render example --table accrual --context - < <w>/wl001-dump.json` | 0 | 8.200 ms | **8.215 ms** | +0.016 ms | 0.051 ms | 1.71% | 17 775 506 / 9 264 544 B | 15.25 → 15.34 MiB |
| `rschema_x` | `tpl render rust/schema --context <w>/wl001-dump.json` | 0 | 14.519 ms | **14.470 ms** | −0.049 ms | 0.046 ms | 1.32% | 31 447 661 / 8 244 333 B | — |
| `rview_x` | `tpl render probe/view --view v_booking_line_summary --context <w>/wl001-dump.json` | 0 | 8.079 ms | **8.066 ms** | −0.013 ms | 0.026 ms | 1.69% | 12 491 795 / 8 244 450 B | — |
| `rwjson_x` | `tpl render probe/whole_json --context <w>/wl001-dump.json` | 0 | 22.741 ms | **22.715 ms** | −0.025 ms | 0.088 ms | 1.03% | 49 396 005 / 31 560 455 B | 41.81 → 41.81 MiB |
| `rexample_d` | `tpl -d bench_wl001 render example --table accrual --direct --no-cache` | 0 | 23.730 ms | **23.640 ms** | −0.091 ms | 0.012 ms | 1.56% | 5 559 720 / 3 987 012 B | 8.69 → 8.77 MiB |
| `rstruct_d` | `tpl -d bench_wl001 render rust/struct --table accrual --direct --no-cache` | 0 | 24.338 ms | **24.288 ms** | −0.050 ms | 0.022 ms | 1.70% | 7 046 997 / 4 139 195 B | — |
| `rschema_d` | `tpl -d bench_wl001 render rust/schema --direct --no-cache` | 0 | 30.061 ms | **30.070 ms** | +0.010 ms | 0.098 ms | 1.34% | 24 447 197 / 4 234 879 B | — |
| `rview_d` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary --direct --no-cache` | 0 | 23.556 ms | **23.455 ms** | −0.100 ms | 0.112 ms | 2.59% | 5 489 731 / 3 923 354 B | — |
| `rroutine_d` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count --direct --no-cache` | 0 | 23.544 ms | **23.495 ms** | −0.049 ms | 0.061 ms | 2.10% | 5 490 291 / 3 924 091 B | — |
| `rwjson_d` | `tpl -d bench_wl001 render probe/whole_json --direct --no-cache` | 0 | 38.392 ms | **38.420 ms** | +0.027 ms | 0.099 ms | 1.29% | 42 374 183 / 29 718 239 B | 37.25 → 37.33 MiB |
| `viewjson_c` | `tpl -d bench_wl001 schema view v_booking_line_summary --format json` | 0 | 1.940 ms | **1.943 ms** | +0.003 ms | 0.001 ms | 3.00% | 541 560 / 342 542 B | — |
| `routinejson_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --format json` | 0 | 2.485 ms | **2.479 ms** | −0.006 ms | 0.003 ms | 2.37% | 730 541 / 342 571 B | — |
| `rstruct_last_c` | `tpl -d bench_wl001 render rust/struct --table yard_position` | 0 | 12.401 ms | **11.673 ms** | **−0.728 ms, −5.9%** | 0.042 ms | 2.12% | 14 222 619 / 8 543 872 B | 14.30 → 14.02 MiB |
| `rmiss_reach` | `tpl -d bench_wl001 render probe/whole_json`, `tables/yard_position.json` damaged | 0 | 83.700 ms | **67.887 ms** | **−15.813 ms, −18.9%** | 0.037 ms | 0.72% | 90 726 660 / 56 124 837 B | 41.86 → 70.62 MiB |
| `rmiss_bound` | `tpl -d bench_wl001 render example --table accrual`, `tables/accrual.json` damaged | 0 | 70.060 ms | **31.299 ms** | **−38.762 ms, −55.3%** | 0.168 ms | 1.16% | 6 619 898 / 4 314 745 B | 13.38 → 9.50 MiB |

The two loops (12 samples per arm):

| label | `b243` | `now` | change | A/A | `now` rsd | per-round change |
|---|---|---|---|---|---|---|
| loop, 200 × `example` | 2 434.7 ms | **628.8 ms** | **−1 805.8 ms, −74.2%** | 0.7 ms | 2.69% | −1 812.9 to −1 749.7 ms |
| `loop_struct`, 200 × `rust/struct` | 2 586.0 ms | **1 618.1 ms** | **−967.8 ms, −37.4%** | 2.2 ms | 0.72% | −1 018.4 to −951.9 ms |

**Output identity**, before any timing. Every label was run once with `now` and
once with `b243`, and stdout, stderr and the exit code compared: all 119
matched except the `loaded_at` of the two `cache status` forms, which records
the last write, and the stderr of `-vvv`, which reports the render's duration.

### Results — the experiments

Copies of `git archive ba10788`, one change each, each built into its own
target directory from a path of the same length:

| variant | the one change | binary |
|---|---|---|
| `ctl247` | none: the arm every variant is compared with | 4 017 024 B, sha256 `1adaa747…3ad8` |
| `lkp247` | `lookup::member`, `src/render/lookup.rs:60`, answers a lookup in `tables` from the `table` variable when that variable is a table carrying the name sought, before scanning `database.tables`. **An instrument**: it covers the bound table only, which is every lookup the worked templates make under `--table` | 4 017 024 B, sha256 `b64a41ba…652c` |
| `buf247` | `BufWriter::with_capacity(64 * 1024, …)` in place of `BufWriter::new`, `src/output/writer.rs:191` | 4 017 024 B, sha256 `320ace09…b236` |

Every variant matched `ctl247` on all 119 labels, as `now` matched `b243`.
`lkp247` was also compared with `ctl247` over every worked template of
`examples/*/templates/` (`go/`, `node/`, `python/`, `rust/`) and `example`, 19
in all, each rendered bound to every table of `WL-001` and unbound: 3 819
invocations, stdout, stderr and the exit code identical on every one (3 806
exit `0`, 13 exit `65`).

One rotated campaign of the main campaign's shape over four arms — `ctl247`,
`lkp247`, `buf247`, `ctl247_twin` — on 12 labels, 320 samples per arm and
label, and the `rust/struct` loop over `ctl247`, `lkp247` and `ctl247_twin`,
12 samples per arm. Each variant against `ctl247`:

| label | `ctl247` | variant | change | A/A of `ctl247` | per-round change |
|---|---|---|---|---|---|
| **`lkp247`**, the bound table answered without a scan | | | | | |
| `rstruct_last_c` | 11.408 ms | 3.365 ms | **−8.043 ms, −70.5%** | 0.031 ms | −8.213 to −7.918 ms |
| `loop_struct` | 1 604.9 ms | 785.6 ms | **−819.3 ms, −51.0%** | 4.9 ms | −826.8 to −809.2 ms |
| `rstruct_c`, the first table: the scan stops at once | 3.013 ms | 3.002 ms | −0.011 ms | 0.014 ms | −0.041 to +0.052 ms |
| `rexample_c`, no lookup | 2.354 ms | 2.356 ms | +0.003 ms | 0.007 ms | −0.036 to +0.024 ms |
| `rschema_c`, not bound | 16.754 ms | 16.704 ms | −0.050 ms | 0.019 ms | −0.359 to +0.246 ms |
| **`buf247`**, a 64 KiB output buffer | | | | | |
| `dumppretty_c`, 7.2 MB out | 14.698 ms | 14.050 ms | **−0.648 ms, −4.4%** | 0.024 ms | −0.895 to −0.310 ms |
| `tablespretty_c`, 6.7 MB out | 13.692 ms | 13.137 ms | **−0.555 ms, −4.1%** | 0.046 ms | −0.732 to −0.374 ms |
| `dump_c`, 3.18 MB out | 13.274 ms | 13.154 ms | −0.120 ms, −0.9% | 0.006 ms | −0.326 to +0.025 ms |
| `tablesjson_c` | 12.302 ms | 12.200 ms | −0.102 ms, −0.8% | 0.015 ms | −0.307 to +0.110 ms |
| `rwjson_c`, 3.18 MB out through `emit_verbatim` | 26.392 ms | 26.544 ms | +0.151 ms | 0.023 ms | −0.617 to +0.395 ms |
| `helpjson_p`, 130 kB out | 1.948 ms | 1.942 ms | −0.006 ms | 0.003 ms | −0.028 to +0.029 ms |
| `table_c`, `rexample_x`, controls | 1.965 / 7.992 ms | 1.973 / 7.999 ms | +0.008 / +0.007 ms | 0.012 / 0.029 ms | inside ±0.21 ms |

- **`lkp247` is an upper bound on row 1, not a design.** It removes the scan
  only where the name sought is the bound table's; a template that resolves
  another table by name still scans. It measures the cost of reading and
  decoding the files of the tables listed before the one sought: 8.0 ms for
  199 tables, about 40 µs per table, and on the loop, whose table positions are
  0 to 199, 819 ms over 200 renders, about 4.1 ms per render on average.
  `rstruct_c` binds the first table, so its scan stops at once and it gains
  nothing.
- **`buf247` is established on the pretty whole reads only.** Every round moved
  both by 13 to 40 times their A/A. On the compact dump and the compact table
  listing the median moved by −0.10 to −0.12 ms, but at least one round of eight moved
  the other way, so it is not established there. On `probe/whole_json` it is
  inside the per-round spread.

### Where the remaining cost goes

Shares of on-CPU samples of `now`'s code in a symbolised build of `ba10788`
(`debug = true`, `strip = false`), `samply` at 20 kHz; inclusive, so they
overlap. A figure in milliseconds is a share multiplied by the median of
`now`: an estimate, and an upper bound. Folded stacks of the five profiles
named *flame graph* below are in the scratch directory, and `samply load` on
each `.json.gz` renders its flame graph.

**The start-up floor is measured, not estimated.** An empty `fn main() {}`
under the release profile, `nop`, run rotated with three labels in one further
campaign (320 samples each):

| label | median | above `nop` |
|---|---|---|
| `nop`, the process-spawn floor: `exec`, `dyld`, and nothing of `tpl` | 1.301 ms (A/A 0.011 ms) | — |
| `version` | 1.692 ms | 0.391 ms: the parser tree, 83.4% of its samples in `clap` |
| `table_c` | 1.960 ms | 0.659 ms |
| `rexample_c` | 2.348 ms | 1.047 ms |

- **A cached render bound to one object, `rexample_c` (2.405 ms; flame
  graph).** 1.30 ms of it is the spawn floor. Of the samples: the three
  directory listings of `FR-CACHE-038`, `tpl::cache::shelve`, 27.0% — the
  `getdirentries` and directory `open` calls 14.6%, and the 12.4% above them
  building a `Shelf` per entry, joining its path (3.0%) and sorting the members
  (2.3%); the parser tree 13.6%; project discovery and configuration 12.6%
  (a `canonicalize` 3.1%); the template's path resolution, `Root::find`, 8.1%
  (a `canonicalize` 7.3%); the render 15.9%, of which loading and compiling the
  template 6.2%; the bound table's read, decode and conversion 7.3%; `malloc`
  19.6% self. The render deadline's timer thread is 5.1% of the samples, on
  another core. The profile of the canonical loop driven by a child-following
  driver (`loop_example`; flame graph) has the same shape: within 0.8 points on
  each of those rows but the parser tree, 10.5% against 13.6%.
- **`rust/struct` bound to the last table, `rstruct_last_c` (11.673 ms; flame
  graph).** `lookup::member` is 82.6%, and the 199 table files it reads, `Shelf`
  reads and decodes, 53.1%. The `rust/struct` loop (`loop_struct`; flame graph)
  is 72.3% `lookup::member`. That is row 1.
- **`probe/whole_json` from the cache, `rwjson_c` (26.641 ms, +0.46 ms against
  `b243`; flame graph).** Against a profile of `#243`'s code on the same label:
  4.1% more samples in all, and the `open` system call's self share 14.20%
  against 12.62%, which is about half the extra samples. The files are opened
  one at a time, interleaved with the render, where `#243`'s code opened them
  all before it; that the interleaving costs the extra `open` time is
  **hypothesised, not established**. `FR-CACHE-038` forbids reading a file
  before the template reaches it.
- **A miss reached during the render, `rmiss_reach` (67.887 ms).** The render
  it abandons reads 199 files and converts them before it reaches the damaged
  one, then the server is read, the store written and the render made again, as
  `FR-CACHE-039` requires. Its peak resident memory is 70.62 MiB against
  41.86 MiB for `b243`: the abandoned render's context is released through
  `Ending::release`, `src/cli/render.rs:611`, which leaks it for the process
  exit, so it is still held while the second render runs.
- **The cached dumps, `dump_c` (13.574 ms) and `dumppretty_c` (15.019 ms).**
  Reading is 33.4% and 30.2% (`open` 24.0% and 22.0%), decoding
  (`Loaded::document`) 43.2% and 38.6%, serialising 21.3% and 29.3%. The
  `write` calls are 1.15% and 4.66%: row 2, which `buf247` measured.
- **A `--context` render, `rexample_x` (8.123 ms).** `model::document::read`
  is 88.2%; `finish_grow` 7.2%.
- **Allocation.** A bound cached render now allocates 762 108 B and holds
  269 312 B at its peak, against 11 848 144 B and 8 309 594 B at `#243`.
  `rstruct_last_c` allocates 14 222 619 B and holds 8 543 872 B, the 199 tables
  its lookup decodes. `rexample_xin` still allocates 5 213 646 B more than
  `rexample_x` and holds 1 020 112 B more.

### The waste register, sorted by estimated gain

Effort is `#231`'s: `S` (one function, no interface change), `M` (a new code
path or a hand-written serde implementation), `L` (a new abstraction across
modules). "Established" means a single-variable experiment above measured the
gain; "estimate" means a profile share multiplied by a measured median, an
upper bound unless stated otherwise. `#243`'s rows 1, 2 and 4 were applied by
`#246`, `#244` and `#245` and are confirmed above; its rows 3, 5, 6, 7 and 8 are
restated here with this pass's figures.

| # | Path | Evidence | Cause in code | Why it is vacuous for the command | Estimated gain | Effort | Functional risk |
|---|---|---|---|---|---|---|---|
| 1 (new) | A cached render that resolves a table by name: `is primary_key`, `is unique`, `column()`, `table()` — eight of the worked templates of `examples/`: `go/struct`, `go/sql`, `node/model`, `node/mysql2`, `python/dataclass`, `python/pymysql`, `rust/struct`, `rust/sqlx` | `lkp247` −8.04 ms on `rstruct_last_c` (A/A 0.031 ms); −819.3 ms (−51.0%) on `loop_struct` (A/A 4.9 ms); `lookup::member` 82.6% of `rstruct_last_c`'s samples and 72.3% of the loop's | The linear scan `members.try_iter().ok()?.find(…)`, `src/render/lookup.rs:63`, reads each member it passes through `Members::convert`, `src/cli/render/context/lazy.rs:517`, and `Store::table`, `:285`, which reads, validates and decodes its file | The member sought is found by the name its path already holds; the files of the members passed are read only to compare their names, and for a lookup of the bound table the member is already read | **−8.0 ms (−70%) per render bound to the last of 200 tables, and −819 ms (−51%) on a 200-render `rust/struct` loop, established as an upper bound**; about 40 µs per table listed before the one sought. None on the canonical loop | `S` for the bound table; `M` for a lookup by the listing's names | `FR-CACHE-038` reads a file "the first time the template reaches that object", and `lazy.rs:48`–`51` records *reading a member's name from the listing* as rejected because it reads "reaches" loosely. Whether a lookup by name reaches the members it passes — and so whether a damaged file listed earlier must still abandon the render under `FR-CACHE-039` — is a question for the specification. `FR-ENV-015`, `FR-ENV-020` and `FR-CTX-022` fix what the lookup answers, not how |
| 2 (`#243` row 3) | The compact `schema dump` and `schema tables --format json`, from the cache | Unchanged: 13.574 and 12.567 ms; decode 43.2% and read 33.4% of `dump_c` | `Loaded::document`, `src/cache.rs:175`, over `Cache::everything`, `:609`, and the presentation reached from `src/cli/schema.rs:543` | As `#240` stated | ≈ −5 to −6 ms per compact cached whole read, estimate, as `#240` stated; not in either loop | `M`–`L` | As `#240` stated |
| 3 (`#243` row 6) | Whole reads written to stdout, `--pretty` above all | `buf247` −0.648 ms on `dumppretty_c` and −0.555 ms on `tablespretty_c` (A/A 0.024 and 0.046 ms); −0.10 to −0.12 ms on the compact forms, not established | `BufWriter::new(Tracked::new(stream))`, `src/output/writer.rs:191`: 8 KiB, about 880 `write` calls for the 7.2 MB pretty dump | A larger buffer issues fewer `write` calls for the same bytes | **−0.55 to −0.65 ms per pretty whole read (−4%), established**; ≤ −0.12 ms per compact whole read, not established; nothing on one object or on `help` | `S` | None to the bytes. A consumer that closes the stream early is detected after up to 64 KiB instead of 8 KiB (`FR-ERR-025`); the outcome is the same |
| 4 (new) | Every cached render, bound or not; the canonical loop | `tpl::cache::shelve` 27.0% of `rexample_c`'s samples: the listing system calls 14.6%, and 12.4% above them | `shelve`, `src/cache.rs:1081`: a `PathBuf` and a `String` per entry, a `Vec` grown from empty, and `order::sort_by_name` | The listing is required; the per-entry allocations and the growth are not | ≤ −0.30 ms per bound cached render (12.4% of 2.405 ms), and ≤ −78 ms on the canonical loop (12.4% of 628.8 ms), estimate, upper bound. Not an experiment | `S`–`M` | `FR-CACHE-038` requires the listing of each collection before the render; `NFR-DET-002` requires the order, so the sort stays |
| 5 (`#243` row 5) | Every decode of the store and of a `--context` document | `finish_grow` 7.2% of `rexample_x`, 4.9% of `dump_c`, 5.9% of `rexample_c` (2.9 points of it inside `shelve`, row 4) | `serde`'s sequence visitor grows each `Vec` from empty: `columns`, `src/model/document/shape.rs:179`, and the other collections of the shape | The reallocations copy what the final vector holds | ≤ −0.58 ms per `--context` render, ≤ −0.66 ms per cached dump, ≤ −0.07 ms per bound cached render beyond row 4, estimate, upper bound | `M` | None |
| 6 (`#243` row 7, `#242` row 2a) | The `--context` render | Unchanged: `model::document::read`, `src/model/document.rs:137`, 88.2% of `rexample_x` | As `#242` stated | As `#242` stated | ≤ −0.54 ms per `--context` render, estimate, as `#242` stated | `M` | As `#242` stated |
| 7 (new) | A cached render that reads the whole database | `rwjson_c` +0.462 ms (+1.8%) against `b243`, A/A 0.059 ms; `rwwalk_c` +0.43 ms inside its A/A of 0.32 ms | Hypothesised: the files opened one at a time during the render, `open` 14.20% of the samples against 12.62% at `#243` | Not attributed | ≤ −0.46 ms per whole-database cached render, estimate; nothing on a bound render | `M` | If the hypothesis holds, the cost is `FR-CACHE-038`'s, which forbids reading a file before the template reaches it |
| 8 (new) | A miss reached during the render | `rmiss_reach` 67.887 ms and 70.62 MiB peak, against 41.86 MiB for `b243`; `#246` measured +17.9 ms against the up-front read | The abandoned render's work before the miss; its context released through `Ending::release`, `src/cli/render.rs:611`, which leaks it until the process exits | The time is the work `FR-CACHE-039` abandons; the 28.8 MiB is held while the second render runs | Time: none reducible without reading up front, which `FR-CACHE-038` forbids. Memory: ≈ −28.8 MiB at the peak by freeing the abandoned context, estimate; its time not measured | `S` for the memory | `FR-CACHE-039` requires the abandon and the restart. Only a damaged cache reaches this path |
| 9 (`#243` row 8) | `render --context -` | +0.092 ms against the same document from a file (A/A 0.051 and 0.021 ms); +5 213 646 B allocated, +1 020 112 B at the peak | `Vec::new()` and `read_to_end`, `src/cli/render.rs:782`–`785` | The growth from empty reallocates for a document whose size a redirected file states | **At the floor in time, established in bytes** | `S` | None |

### The verdict: does the campaign stop?

The floor of this pass: 0.000 to 0.023 ms on a start-up-class invocation, up
to 0.088 ms on a cached read or render, up to 0.112 ms on a direct read or
render, 0.7 ms (0.1%) on the canonical loop and 2.2 ms (0.1%) on the
`rust/struct` loop. **Start-up floor and code are distinguished**: 1.301 ms of
every invocation is the spawn floor (`nop`: `exec` and `dyld`), which no change
to `tpl`'s code reaches. Above it, 0.39 ms is the parser tree every invocation
builds, 83% `clap`; `#240`'s rows 5 and 6 found at most 0.08 ms of it
removable, and the rest is the parser the stack names. Everything else in the
register is code.

| # | Gain against its own invocation | Against a loop | Worth a further round? |
|---|---|---|---|
| 1 | 70% of `rust/struct` bound to the last table; 259 times its A/A | 51% of the `rust/struct` loop, 167 times its A/A; none on the canonical loop | **Yes — after the specification.** It is the largest established item, on the path eight of the worked templates of `examples/` take. The first step is `specification-manager`, on what "reaches" means for a lookup |
| 2 | ≈ 40% of a compact cached dump, estimate | none | **Yes**, as `#240` stated; still an estimate after four passes |
| 3 | 4% of a pretty whole read; 13 to 40 times its A/A in every round | none | **Marginal.** Established and `S`, but 0.6 ms on a 14 ms command |
| 4 | ≤ 12% of a bound cached render, estimate | ≤ 12% of the canonical loop, estimate | **Only after an experiment.** It is the largest candidate left on the canonical loop, and an upper bound |
| 5 | ≤ 7% of a `--context` render, estimate | ≤ 3% of the canonical loop, inside row 4 | **Only after an experiment** |
| 6 | ≤ 6.6% of a `--context` render, estimate | none | **Marginal**, as `#242` left it |
| 7 | ≤ 1.8% of a whole-database render | none | **No.** Unattributed, and likely required by `FR-CACHE-038` |
| 8 | memory only, on a damaged cache | none | **No** for time, which `FR-CACHE-039` requires; the memory is a question of taste on a path only a damaged store reaches |
| 9 | at the floor | none | **No.** Insignificant |

**The campaign does not stop.** Its largest remaining gain, row 1, is 70% of
the invocation it affects and 51% of a 200-render loop over the worked
`rust/struct` template — 259 and 167 times the noise floors of those
commands — and it is neither start-up floor nor a cost the specification
requires as it stands. **For the canonical loop the campaign is close to
stopping**: 2.35 ms per render, of which 1.30 ms is the spawn floor and
0.39 ms the parser tree, and the largest candidate left there, row 4, is an
estimate of at most 0.30 ms. **It stops for every other path**: start-up, help
and version in all their forms, the error paths, discovery, the `cfg` and
`template` commands and every configuration write, `init`, `cache status` and
`cache clean`, the single-object and listing reads in both forms and through
every alias, the direct reads and renders, and the whole store writes. Each
candidate left there is at or below the floor, or is a cost the specification
requires.

### Refuted hypotheses and non-findings, stated so they are not rediscovered

- **The render deadline's timer thread is not a wall-time cost.** Its samples,
  5.1% of `rexample_c`'s, are on another core; creating it is 0.8%.
- **`rust/struct` bound to the first table pays nothing for the lookup**:
  `lkp247` moved `rstruct_c` by −0.011 ms against an A/A of 0.014 ms. The cost
  is in proportion to the position of the table in the listing.
- **The canonical loop does not reach row 1**: `example` makes no lookup, and
  `lkp247` moved `rexample_c` by +0.003 ms.
- **The seven aliases are still their canonical commands**, within 0.019 ms on
  the start-up-class labels and 0.038 ms on `schema tables`.
- **The `json` form of every command other than the whole reads still costs
  what its `text` form costs**, within 0.043 ms (`help schema table`, whose
  `json` form is the faster).
- **`-vvv`, `-q`, `--timeout` and `--set` still add nothing measurable** to a
  cached render: 2.394 to 2.408 ms against 2.405 ms.
- **`#244` reaches `cache load --table`**: −0.31 ms, 2.8 times its A/A, two
  unchanged objects left in place, as `#244` measured (−0.107 ms there).
- **Nothing on the start-up, help, configuration, template, direct-read or
  `--context` paths moved since `#243`**, and nothing regressed but `rwjson_c`,
  which `#246` recorded (+0.42 ms there, +0.46 ms here).

### Confounders — read this before the tables

- **The host was not idle.** The four containers of other projects named in
  `#240` ran throughout. The load average was 4.04 at the start of the main
  campaign, just after the builds, and 2.11 to 3.14 for the rest of it; 1.83 to
  2.70 during the experiments. The rotation and the A/A arms bound this; they
  do not remove it. `hyperfine` printed 460 warnings in the main campaign,
  about statistical outliers or a slow first run.
- **`b243` is `#243`'s `now` itself**, the file measured then, sha256
  `53ed3470…ab16`, kept in the scratch directory, not a rebuild. `now` is
  `target/release/tpl` at `ba10788`, sha256 `5f49ef9a…38fa`, `#246`'s
  in-repository binary byte for byte.
- **The experiments' `ctl247` is not `now`**: the same source, built from a
  scratch path, with another sha256. It measured `rexample_c` at 2.354 ms
  where `now` measured 2.405 ms in the main campaign. Every variant is compared
  with it, never with `now`.
- **The profiles of the loops were driven by a Python script**, not by
  `benches/loop200.sh`: `bash` is a protected system binary on macOS, and
  `samply` cannot follow its children. The script execs the same 200
  invocations; its own samples are excluded. It ran over a primed store, so
  unlike the timed loop it contains no first-render miss.
- **`samply` shares carry the profiler's overhead, overlap, and are
  function-level**; an inlined function is counted in its caller.
- **`dhat` counts the heap only**, one run per label, through an instrumented
  build, and is never used as time. It and the memory readings were taken
  between campaigns, never during one.
- **The server path crosses Docker Desktop's port proxy** (Docker 29.8.1, where
  `#243` ran 29.5.2), and the benchmark projects authenticate as `root`
  (`#224`). `up.sh 12.3` exited `2` on success (`#223`), and the filtered gate
  `status.sh --quiet 12.3`, which answered `0`, was read instead.

### What was not measured — stated, not implied

- **Three of the four targets.** Rows 1 and 4 are file-system and allocator
  costs, which differ by operating system.
- **Every TLS mode**, and the series `10.11`, `11.4` and `11.8`, raised only
  for the test suite.
- **A lookup of a table other than the bound one**, of a view or of a routine
  (`view()`, `routine()`), which scan their collections the same way; and
  `lkp247`'s peak memory.
- **Rows 2, 4, 5, 6, 7 and 8 as experiments.** They are estimates.
- `#243`'s list of untested forms stands: a connection that times out,
  `password_command`, the `--dsn` and TLS-file forms of `cfg database add` and
  `update`, `cache load` and `cache clean` with `--view` or `--routine`, and
  the `65` of a template that fails to compile or of an expired deadline.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
cargo build --release; cp target/release/tpl "$S/tpl-now"            # ba10788, the binary of record
cp /path/to/243/tpl-now "$S/tpl-b243"                                # #243's binary of record, 53ed3470…ab16;
#   or: git archive 7c0fdaf into "$S/src/7c0fdaf" and cargo build --release there (another sha256)

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3   # read the gate, not up.sh
./scripts/mariadb/seed-bench.sh 12.3

# 2. The projects, with benches/fixture.sh's functions and tpl-now: fixture_startup_project,
#    fixture_server_project (disabled), fixture_prime, fixture_subjects; the rust/ templates and
#    #243's four probes copied into server; mut, dead and empty as #243's Workload table states;
#    the --context dump with `tpl-now -d bench_wl001 schema dump`.

# 3. Wall time: for round r of 8, the 119 labels rotated by r, the three arms rotated by r;
#    40 runs after 5 warmups (A), 20 after 5 (D; 2 for init), 20 after 3 (rwwalk_c), 5 after 1 (W):
hyperfine -N -i --warmup 5 --runs 40 --export-json "$S/c1/<label>.r<r>.json" \
  [--prepare "<restore, or: replace one object file with '{ torn' by temporary and rename>"] \
  [--input "$S/work/wl001-dump.json"] \
  -n b243 "$S/tpl-b243 <args>" -n now "$S/tpl-now <args>" -n now_twin "$S/tpl-now <args>"
#    the two loops, 4 rounds of 3 after 1, template example and then rust/struct:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-now -d bench_wl001 cache clean" \
  -n b243 "benches/loop200.sh $S/tpl-b243 bench_wl001 <template> $S/work/wl001-tables.txt" …

# 4. The experiments: copies of `git archive ba10788` under paths of equal length, one edit each
#    as the table states, each with its own --target-dir; the same rotation over
#    (ctl247 lkp247 buf247 ctl247_twin) on 12 labels, and the rust/struct loop over
#    (ctl247 lkp247 ctl247_twin). The identity sweep: every examples/*/templates/ template
#    and example, bound to each of the 200 tables and unbound, run with ctl247 and lkp247.

# 5. The spawn floor: an empty `fn main() {}` under the release profile, rotated with
#    tpl-now --version, schema table accrual and render example --table accrual, 8 × 40 after 5.

# 6. Profiles, allocation, memory.
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false \
  cargo build --release --target-dir "$S/t-prof"                   # from the scratch copy
samply record -s -r 20000 --iteration-count 200 --reuse-threads --unstable-presymbolicate \
  -o "$S/prof/rexample_c.json.gz" -- "$S/t-prof/release/tpl" -d bench_wl001 render example --table accrual
samply record -s -r 20000 --reuse-threads --unstable-presymbolicate -o "$S/prof/loop_struct.json.gz" \
  -- python3 loopdrv.py "$S/t-prof/release/tpl" "$S/work/wl001-tables.txt" rust/struct   # one exec per table
samply load "$S/prof/rexample_c.json.gz"                                                  # the flame graph
/usr/bin/time -l "$S/tpl-now" -d bench_wl001 render example --table accrual >/dev/null

# 7. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Row 1 of the fourth register applied: a cached lookup by name reads only the object it returns

*Sprint 19, task `#248`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

The lookups by name — the tests `primary_key` and `unique` and the functions
`table`, `view`, `routine` and `column` — no longer scan a collection served
from the cache. `crate::render::lookup` puts the name to the collection as a
query key no template can construct, and the lazy collection resolves it from
the listing and reaches only the member it returns, per the forty-first
edition of `FR-CACHE-038`. Over a whole document the query goes unanswered and
the scan runs as before.

**`rust/struct` bound to the last of 200 tables fell from 11.60 to 3.40 ms
(−8.20 ms, −70.7%), its peak resident memory from 13.86 to 5.11 MiB, and the
200-render `rust/struct` loop from 1 621.3 to 798.9 ms (−50.7%).** That is
`#247`'s `lkp247` upper bound (−8.04 ms, −51.0%) reproduced on the shipped
code. The canonical loop, which renders `example`, reaches no lookup and did
not move beyond its A/A; neither did the whole-database control.

### Candidates

Both arms were built from `git archive 59e7680` in a scratch directory, from
paths of the same length, each into its own target directory; `after` carries
this change's `src/` and nothing else.

| arm | binary |
|---|---|
| `before` | `59e7680`: 4 017 024 B, sha256 `2d5ca3c5…8e02` — `#246`'s `after`, byte for byte (`59e7680` changes only `BENCHMARKS.md` since `ba10788`) |
| `after` | `59e7680` with this change: 4 017 040 B, sha256 `19cb1ef5…63c7` |
| `before_twin` | the `before` file, measured as a third label: the A/A arm |

The in-repository `target/release/tpl` with this change is 4 033 568 B, sha256
`10a617e7…3bc2`, built from the same sources under a longer path; the 16 528 B
difference was not examined.

### Environment

As `#243`'s: Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0
`arm64`; `rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of
`ADR-004`; `hyperfine` 1.20.0 (`-N`); `/usr/bin/time -l`. All five fixture
servers were up; timing reached only `12.3`, seeded by
`scripts/mariadb/seed-bench.sh`, `tls = "disabled"`, account `root`. The
`server` project of `#246`'s entry, primed again with `before`. Mains power,
not charging; load 2.08 to 2.43. Taken 2026-09-23 (UTC): the campaign 11:03Z
to 11:04Z, the two loops 11:04Z to 11:06Z, peak memory after.

### Protocol

`#243`'s: 8 rounds, the 7 labels rotated by one position per round and the
three arms rotated inside one `hyperfine` call per label per round; 40 runs
after 5 warmups per arm per round, 320 samples per arm and label. The two
loops: `benches/loop200.sh` with `example` (the canonical loop) and with
`rust/struct` (`#247`'s `loop_struct`), 4 rounds of 3 runs after 1 warmup
each, 12 samples per arm, `--prepare "tpl -d bench_wl001 cache clean"`. Peak
resident memory: median of 7.

**Byte identity**, before any timing: `#246`'s sweep, unchanged — every worked
template of `examples/*/templates/` and the `example` of `tpl init`,
whole-database and bound with `--table`, `--view` and `--routine` (bare and
qualified) to every object, with `-vvv`, `-q`, `--set` and an absent table.
4 180 invocations over `freight` on the five servers and 6 213 over `WL-001`
and `WL-003` on `12.3`: stdout, the exit code and stderr (the `-vvv` duration
masked) identical in all 10 393. No `meta.json` was rewritten during the sweep,
so every render was served from the cache.

### Results

Medians; the A/A column is `before` against `before_twin`.

| label | command | `before` | `after` | change | A/A | p90, `before` → `after` | per-round change |
|---|---|---|---|---|---|---|---|
| `rstruct_last_c` | `tpl -d bench_wl001 render rust/struct --table yard_position` | 11.595 ms | **3.397 ms** | **−8.198 ms, −70.7%** | 0.021 ms | 11.99 → 3.49 ms | −8.35 to −8.01 ms |
| `rstruct_c` | the same bound to `accrual`, the first table | 2.988 ms | 3.005 ms | +0.018 ms | 0.012 ms | 3.08 → 3.12 ms | −0.02 to +0.07 ms |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual`, no lookup | 2.335 ms | 2.343 ms | +0.009 ms | 0.005 ms | 2.41 → 2.41 ms | −0.02 to +0.03 ms |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema` | 16.910 ms | 16.903 ms | −0.007 ms | 0.099 ms | 17.30 → 17.25 ms | −0.20 to +0.22 ms |
| `rwjson_c` | `tpl -d bench_wl001 render probe/whole_json`, the whole-database control | 26.623 ms | 26.548 ms | −0.075 ms | 0.038 ms | 26.99 → 27.17 ms | −0.53 to +0.56 ms |
| `rexample_x` | `render example --table accrual --context <dump>`, control | 8.052 ms | 8.050 ms | −0.003 ms | 0.050 ms | 8.25 → 8.24 ms | −0.10 to +0.08 ms |
| `table_c` | `tpl -d bench_wl001 schema table accrual`, control | 1.948 ms | 1.976 ms | +0.027 ms | 0.003 ms | 2.04 → 2.05 ms | −0.02 to +0.09 ms |

The loops (12 samples per arm):

| loop | `before` | `after` | change | A/A | per-round change |
|---|---|---|---|---|---|
| `loop_struct`, 200 × `rust/struct` | 1 621.3 ms | **798.9 ms** | **−822.4 ms, −50.7%** | 0.4 ms | −834.7 to −815.9 ms |
| the canonical loop, 200 × `example` | 622.5 ms | 626.3 ms | +3.9 ms | 4.2 ms | −9.7 to +11.8 ms |

Peak resident memory, `/usr/bin/time -l`, median of 7: `rstruct_last_c`
13.86 → 5.11 MiB; `rstruct_c` 4.94 → 5.02 MiB; `rwjson_c` 43.09 → 43.22 MiB.

- **The saving is the files listed before the one sought.** `rstruct_last_c`
  gains 8.2 ms, about 41 µs for each of the 199 tables before `yard_position`,
  and now costs what `rstruct_c` costs within 0.4 ms. `rstruct_c`, whose table
  is listed first, passed over nothing before and gains nothing.
- **The query costs nothing measurable where it is not answered.** The
  `--context` control and the whole-database control, whose collections are a
  whole document or are never looked up, are inside their per-round spread;
  `table_c`, which reaches none of this code, moved +0.027 ms, which is the
  noise of this session at that level.

### Not measured

- A lookup through `view()` or `routine()` on `WL-001`: no worked template
  calls either; both are covered by the tests, not timed.
- The series `10.11`, `11.4` and `11.8`, raised for the identity sweep and the
  test suite, not for timing.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
mkdir -p "$S/src/59e7680" "$S/src/aft0248"
git archive 59e7680 | tar -x -C "$S/src/59e7680"
git archive 59e7680 | tar -x -C "$S/src/aft0248"
for f in $(git diff --name-only 59e7680 -- src); do cp "$f" "$S/src/aft0248/$f"; done
(cd "$S/src/59e7680" && cargo build --release --target-dir "$S/t-59e7680"); cp "$S/t-59e7680/release/tpl" "$S/tpl-before"
(cd "$S/src/aft0248" && cargo build --release --target-dir "$S/t-aft0248"); cp "$S/t-aft0248/release/tpl" "$S/tpl-after"

# 1. The fixture, through its harness only.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/seed-bench.sh 12.3

# 2. The server project and the identity sweep, as #246's entry states.

# 3. For round r of 8, the 7 labels rotated by r, the three arms rotated by r:
hyperfine -N --warmup 5 --runs 40 --export-json "$S/c/<label>.r<r>.json" \
  -n before "$S/tpl-before <args>" -n after "$S/tpl-after <args>" -n before_twin "$S/tpl-before <args>"
#    each loop, 4 rounds, <template> rust/struct and then example:
hyperfine -N --warmup 1 --runs 3 --prepare "$S/tpl-before -d bench_wl001 cache clean" \
  -n before "benches/loop200.sh $S/tpl-before bench_wl001 <template> $S/work/wl001-tables.txt" …

# 4. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Row 3 of the fourth register applied: a 64 KiB stdout buffer

*Sprint 19, task `#249`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

The one buffer every result reaches stdout through, `src/output/writer.rs`, now
holds 64 KiB instead of the standard library's 8 KiB. **The pretty cached dump
of `WL-001` fell from 14.73 to 14.02 ms (−0.71 ms, −4.8%), and the pretty table
listing from 13.62 to 13.02 ms (−0.59 ms, −4.4%), with the same bytes out.**
That is `#247`'s estimate (−0.65 and −0.56 ms). The compact forms gained
0.15 to 0.19 ms; `--version`, a bound cached render and a single-object read
did not move beyond their per-round spread.

### Choosing the size

Six builds of `22d420f`, identical but for the capacity, measured together in
one rotated campaign of the protocol below with seven arms (the control twice,
as the A/A). Medians of 320 samples, change against the 8 KiB control:

| capacity | `dumppretty_c` | `tablespretty_c` | `dump_c` | `tablesjson_c` | `version` |
|---|---|---|---|---|---|
| 8 KiB (control) | 14.629 ms | 13.524 ms | 13.236 ms | 12.266 ms | 1.665 ms |
| 16 KiB | −0.395 ms | −0.284 ms | −0.088 ms | −0.095 ms | −0.005 ms |
| 32 KiB | −0.513 ms | −0.398 ms | −0.140 ms | −0.126 ms | −0.012 ms |
| **64 KiB** | **−0.587 ms** | **−0.519 ms** | −0.164 ms | −0.158 ms | −0.006 ms |
| 128 KiB | −0.586 ms | −0.516 ms | −0.126 ms | −0.190 ms | −0.017 ms |
| 8 MiB, the whole output | −0.253 ms | −0.147 ms | −0.090 ms | −0.098 ms | +0.007 ms |
| 8 KiB, A/A | +0.026 ms | +0.067 ms | −0.013 ms | −0.013 ms | −0.008 ms |

64 KiB is the smallest capacity that takes the whole gain on the two pretty
reads: 128 KiB takes the same, and 32 KiB about 80% of it. A buffer as large
as the whole output gains less; why was not examined. A first run of the same
campaign, whose raw exports were lost with the scratch directory, gave the
same ranking (64 KiB −0.64 and −0.51 ms, 128 KiB −0.52 and −0.58 ms, 8 MiB
−0.19 and −0.29 ms).

The buffer is reserved, not filled: a command that prints little touches only
the pages its bytes reach, and `--version`'s peak resident memory moved by one
16 KiB page.

### Candidates

Both arms were built from `git archive 22d420f` in a scratch directory, from
paths of the same length, each into its own target directory; `after` carries
this change's `src/output/writer.rs` and nothing else.

| arm | binary |
|---|---|
| `before` | `22d420f`: 4 017 040 B, sha256 `19cb1ef5…63c7` — `#248`'s `after`, byte for byte |
| `after` | `22d420f` with this change: 4 017 040 B, sha256 `43e7bc94…` — the 64 KiB candidate above, byte for byte |
| `before_twin` | the `before` file, measured as a third label: the A/A arm |

The in-repository `target/release/tpl` with this change is 4 033 568 B, sha256
`4995c368…7804`.

### Environment

As `#243`'s: Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0
`arm64`; `rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of
`ADR-004`; `hyperfine` 1.20.0 (`-N`); `/usr/bin/time -l`. All five fixture
servers were up; timing reached only `12.3`, seeded by
`scripts/mariadb/seed-bench.sh`, `tls = "disabled"`, account `root`; the
`server` project built by `benches/fixture.sh` and primed with `before`. Mains
power, not charging; load 2.06 to 4.22. Taken 2026-09-23 (UTC): the candidates
11:29Z to 11:31Z, the two arms 11:35Z to 11:36Z, peak memory after.

### Protocol

`#243`'s: 8 rounds, the labels rotated by one position per round and the arms
rotated inside one `hyperfine` call per label per round; 40 runs after 5
warmups per arm per round, 320 samples per arm and label. Peak resident memory:
median of 7.

**Byte identity**, before any timing. Every invocation was run with both arms
and stdout, the exit code and stderr (the `-vvv` duration masked) compared:
7 160 invocations and 126.4 MB of stdout, every one identical.

| sweep | invocations | what it covers |
|---|---|---|
| `freight` on the five servers | 1 233 | `help` bare and at each of the 34 node paths, in `text`, `json` and `json --pretty`; per server, cached and `--direct --no-cache`, in all three forms: `schema info`, `tables`, `tables --pattern`, `views`, `routines`, every table, view and routine, and an absent table (`66`); `schema dump` compact and `--pretty`; `cache status`, `cfg database show`, `cfg database test` in the three forms; `cfg list`, `cfg database list`, `template list`, `template path`, `template show`, `version`, `--version` |
| `WL-001` and `WL-003` on `12.3` | 1 842 | the same over the two entries |
| renders of `freight` on the five servers | 4 085 | `#246`'s render sweep: the 19 worked templates, whole-database and bound to every object, with `-vvv`, `-q`, `--set` and an absent table |

A consumer that closes early was checked on a real pipe with both arms, three
times each: `schema dump --pretty | head -c 100` exits `74`
(`FR-ERR-026`), `schema tables | head -1` and `schema dump | true` exit `0`
(`FR-ERR-025`), identically.

### Results

Medians; the A/A column is `before` against `before_twin`.

| label | command | `before` | `after` | change | A/A | p90, `before` → `after` | per-round change |
|---|---|---|---|---|---|---|---|
| `dumppretty_c` | `tpl -d bench_wl001 schema dump --pretty` | 14.725 ms | **14.017 ms** | **−0.708 ms, −4.8%** | 0.008 ms | 14.99 → 14.36 ms | −0.82 to −0.54 ms |
| `tablespretty_c` | `tpl -d bench_wl001 schema tables --format json --pretty` | 13.615 ms | **13.021 ms** | **−0.594 ms, −4.4%** | 0.004 ms | 13.94 → 13.27 ms | −0.71 to −0.53 ms |
| `dump_c` | `tpl -d bench_wl001 schema dump` | 13.287 ms | **13.099 ms** | **−0.187 ms, −1.4%** | 0.009 ms | 13.59 → 13.44 ms | −0.42 to +0.03 ms |
| `tablesjson_c` | `tpl -d bench_wl001 schema tables --format json` | 12.248 ms | **12.097 ms** | **−0.151 ms, −1.2%** | 0.004 ms | 12.45 → 12.38 ms | −0.23 to +0.04 ms |
| `version` | `tpl --version`, control | 1.668 ms | 1.679 ms | +0.011 ms | 0.012 ms | 1.73 → 1.74 ms | −0.02 to +0.06 ms |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual`, control | 2.978 ms | 2.963 ms | −0.015 ms | 0.001 ms | 3.08 → 3.06 ms | −0.06 to +0.01 ms |
| `table_c` | `tpl -d bench_wl001 schema table accrual`, control | 1.921 ms | 1.939 ms | +0.018 ms | 0.001 ms | 1.99 → 2.00 ms | −0.05 to +0.05 ms |

Peak resident memory, `/usr/bin/time -l`, median of 7: `dumppretty_c`
13.14 → 12.91 MiB; `tablespretty_c` 12.77 → 12.61 MiB; `version`
2.58 → 2.59 MiB; `rstruct_c` 5.05 → 4.98 MiB; `table_c` 3.77 → 3.77 MiB.

- **The gain is the writes.** The two pretty whole reads, 7.2 MB and 6.7 MB,
  gain 0.59 to 0.71 ms in every round; the compact forms, about half the bytes,
  gain less and not in every round.
- **Small outputs pay nothing measurable.** `version`, `table_c` and
  `rstruct_c` moved by less than their per-round spread.

### Not measured

- The series `10.11`, `11.4` and `11.8`, raised for the identity sweep and the
  test suite, not for timing.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
for n in ctl0249 k016249 k032249 k064249 k128249 m008249; do
  mkdir -p "$S/src/$n"; git archive 22d420f | tar -x -C "$S/src/$n"; done
# In each candidate but the control, src/output/writer.rs builds the buffer with
# BufWriter::with_capacity(<16384 | 32768 | 65536 | 131072 | 8388608>, …).
for n in ctl0249 k016249 k032249 k064249 k128249 m008249; do
  (cd "$S/src/$n" && cargo build --release --target-dir "$S/t-$n"); done

# 1. The fixture, through its harness only.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/seed-bench.sh 12.3

# 2. The server project, with benches/fixture.sh's functions, primed with the control.

# 3. For round r of 8, the labels rotated by r, the arms rotated by r:
hyperfine -N --warmup 5 --runs 40 --export-json "$S/c/<label>.r<r>.json" \
  -n ctl "$S/t-ctl0249/release/tpl <args>" -n k16 … -n ctl_twin "$S/t-ctl0249/release/tpl <args>"

# 4. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Closing pass: the whole command surface at `2e21918`, and every estimate of the fourth register measured

*Sprint 19, task `#250`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

Every node of the command tree, every alias, both output forms, every help
form, the version forms, the error paths and both loops were measured again at
`2e21918`, after `#248` and `#249`, with `#247`'s instruments, protocol, labels
and fixture, and with `#247`'s code as the baseline arm of the same rotated
campaign. Nothing was changed: `src/`, `Cargo.toml` and `Cargo.lock` are
untouched, and every instrumented or altered binary was built from a throwaway
copy of the crate outside the repository.

**Against `#247`'s code, only the two changes moved anything.** `rust/struct`
bound to the last of 200 tables fell from 11.40 to 3.42 ms (−70.0%), and the
200-render `rust/struct` loop from 1 592.9 to 793.5 ms (−50.2%): `#248`. The
pretty whole reads fell by 0.51 and 0.66 ms (−3.7% and −4.5%), and the compact
dump and the other large outputs by 0.12 to 0.65 ms: `#249`. The canonical loop
did not move beyond its A/A. None of the 110 other labels moved by both 0.1 ms
and twice its A/A; the two largest changes among them, `cload` −0.295 ms and
`routines_d` +0.171 ms, are 1.9 times their A/A, each with rounds of both signs.

**Every estimate of `#247`'s register is now a measured figure**, each from a
single-variable experiment over `2e21918`:

| `#247` row | what was measured | figure |
|---|---|---|
| 2, the compact cached whole reads | the output spliced from the files' bytes: with no decode (the upper bound, against `FR-CACHE-033`), and after the decode that validates them | **−7.70 ms (−58%)** unvalidated; **−2.06 ms (−16%)** validated, identical bytes |
| 4, the per-entry work of the listing | the three listings made a second time, and their directory reads alone made a second time | the whole listing 0.20 ms per bound render, 0.14–0.15 ms of it the system calls: **0.05–0.07 ms of per-entry work** |
| 5, `Vec` growth in the decode | the column list decoded into a vector of capacity 16 | **−0.19 ms (−2.4%)** on a `--context` render; nothing established elsewhere |
| 6, the `--context` read | the embedded tables decoded for their names only, unvalidated | **−1.49 to −1.56 ms (−10% to −19%)** on the `--context` renders |
| 7, the whole-database +0.46 ms | every member read before the render instead of when reached | **−0.57 ms (−2.1%)** on `probe/whole_json`: the interleaved read is the cost, established |
| 8, the memory of an abandoned render | its context and store freed instead of leaked | **−23.8 MiB** at the peak (70.62 → 46.78 MiB), at **+2.18 ms (+3.2%)** |
| start-up | an empty `main`, and `--version` answered before the parser is built | spawn floor 1.302 ms; the binary's own load 0.253 ms; the parser for `--version` 0.139–0.149 ms |

**The campaign stops.** The two largest gains left, rows 2 and 6, are
established and far above their noise floors, but each is reached only by
removing a check the specification requires as it stands (`FR-CACHE-033` and
`FR-OUT-007` for row 2, `FR-RND-020` for row 6). Every other item is below the
run-to-run spread of its command, is a cost `FR-CACHE-038` or `FR-CACHE-039`
requires, or is the process floor. See *The verdict*.

### Candidates

| arm | binary |
|---|---|
| `b247` | `cargo build --release` of `git archive ba10788` in a scratch directory: 4 017 024 B, sha256 `ecace5fe…3524`. `#247`'s binary of record, `5f49ef9a…38fa`, was lost with the scratch directory; this is its rebuild from the same source, a different path embedded |
| `now` | `target/release/tpl` at `2e21918`, the binary of record: 4 033 568 B, sha256 `4995c368…7804` |
| `now_twin` | the same file, measured as a third label: the A/A arm |

**Output identity**, before any timing. Every label was run once with `now` and
once with `b247`, and stdout, stderr and the exit code compared: all 119
matched except the `loaded_at` of the two `cache status` forms.

### Workload and protocol

`#247`'s, unchanged: `WL-001` and `WL-003` in `12.3`, seeded by
`scripts/mariadb/seed-bench.sh`, which verified every count; the `startup`,
`server`, `mut`, `dead`, `empty` and `initdir` projects and `#243`'s four probe
templates, rebuilt with `now`; the same 119 labels, invocations and classes;
8 rounds, labels and arms rotated by one position per round, 320 samples per
arm for the start-up class and the cached and `--context` labels, 160 for the
direct reads and the writers, 40 for the whole store writes and the two miss
labels; each loop 4 rounds of 3 after 1 warmup, 12 samples per arm.

### Environment

| | |
|---|---|
| Host | Apple M4, 10 cores, 32 GiB |
| System | macOS 26.6.2 (build 25G83), Darwin 25.6.0 `arm64` |
| Target | `aarch64-apple-darwin`, built and run natively |
| Toolchain | `rustc` 1.98.1 (48a229cea 2026-09-01); the release profile of `ADR-004` |
| Instruments | `hyperfine` 1.20.0 (`-N`); `samply` 0.13.1 at 20 kHz; `dhat` 0.3.3; `/usr/bin/time -l` |
| Server | the fixture's `12.3` alone during timing, `tls = "disabled"`, account `root`, Docker 29.8.1; all five servers for the test suite |
| Power | mains, not charging; Low Power Mode off |
| Load | one-minute average 2.10 to 3.20; see *Confounders* |
| Taken | 2026-09-23 (UTC): the main campaign 12:00Z to 12:12Z, the loops to 12:14Z, the experiments 12:14Z to 12:28Z, the validated splice 12:28Z to 12:31Z, memory, allocation and profiles after |

### Results — per family, against `#247`

Medians; A/A is `now` against `now_twin`. A family is one line where its labels
run one path.

| family | labels | `b247` → `now` | change | largest A/A |
|---|---|---|---|---|
| `version`, `-V`, `version` | 3 | 1.708–1.734 → 1.717–1.733 ms | within 0.011 ms | 0.013 ms |
| `help` in every form, `tpl` bare, the 5 groups bare and `cfg db` | 15 | 1.803–1.962 → 1.812–1.977 ms | within 0.021 ms | 0.018 ms |
| error paths: `64` twice, `78`, `69` twice, the two template `66`s | 7 | 1.698–1.953 → 1.702–1.952 ms | within 0.010 ms | 0.013 ms |
| `template list`, `show`, `check`, `path`, both forms | 7 | 1.816–1.956 → 1.825–1.951 ms | within 0.020 ms | 0.019 ms |
| `cfg list`, `get`, `database list`, `show`, both forms and the alias | 9 | 1.838–1.873 → 1.844–1.869 ms | within 0.013 ms | 0.014 ms |
| `cfg set`, `unset`, `database add`, `update`, `remove`, `init` | 6 | 1.982–2.127 → 1.983–2.135 ms | within 0.013 ms | 0.011 ms |
| `cfg database test`, both forms | 2 | 3.966–3.982 → 3.976–4.023 ms | within 0.041 ms | 0.048 ms |
| `cache status`, `cache clean` with and without `--table` | 4 | 2.038–2.308 → 2.053–2.283 ms | within 0.026 ms | 0.065 ms |
| `schema info`, three forms | 3 | 2.074–2.089 → 2.076–2.080 ms | within 0.008 ms | 0.002 ms |
| cached single-object reads in every form, alias and `--tpl-dir`; the `views` and `routines` listings | 18 | 1.909–2.478 → 1.910–2.478 ms | within 0.036 ms | 0.016 ms |
| cached `schema tables` (text, alias, `--pattern`) | 3 | 7.431–7.475 → 7.445–7.482 ms | within 0.013 ms | 0.033 ms |
| compact cached `schema dump`, `schema tables --format json` | 2 | 13.239 / 12.208 → 13.087 / 12.166 ms | **−0.151 ms** and −0.042 ms: `#249` | 0.043 ms |
| **pretty whole reads** | 2 | 14.728 / 13.669 → 14.067 / 13.162 ms | **−0.661 and −0.507 ms (−4.5%, −3.7%): `#249`** | 0.038 ms |
| direct reads, every `schema` leaf | 9 | 11.378–25.547 → 11.289–25.429 ms | within 0.17 ms; `dump_d` **−0.119 ms**, `#249` | 0.089 ms |
| whole store writes: `cache load`, the `66` with the server up, `schema dump --direct` | 3 | 29.978–32.368 → 29.781–32.089 ms | −0.02 to −0.30 ms; `dump_dw` **−0.279 ms**, `#249` | 0.152 ms |
| `cache load --table` | 1 | 22.772 → 22.763 ms | −0.009 ms | 0.066 ms |
| cached renders bound to one object | 8 | 2.284–3.006 → 2.292–3.027 ms | within 0.022 ms | 0.018 ms |
| **cached `rust/struct` on the last table** | 1 | 11.396 → 3.421 ms | **−7.975 ms (−70.0%): `#248`** | 0.015 ms |
| cached whole-database renders: `rust/schema`, `probe/whole_json`, `probe/whole_walk` | 3 | 16.680 / 26.525 / 104.887 → 16.756 / 26.477 / 104.875 ms | within 0.076 ms | 0.319 ms |
| `--context` renders, file and standard input | 5 | 7.949–22.776 → 7.936–22.635 ms | within 0.141 ms | 0.088 ms |
| direct renders | 6 | 23.231–38.381 → 23.247–38.243 ms | within 0.14 ms; `rwjson_d` **−0.138 ms**, `#249` | 0.228 ms |
| a miss found before the render, `rmiss_bound` | 1 | 30.983 → 30.874 ms | **−0.109 ms**, not attributed: it writes 494 B | 0.019 ms |
| a miss reached during the render, `rmiss_reach` | 1 | 67.836 → 67.186 ms | **−0.650 ms (−1.0%)**, `#249`: it writes 3.18 MB | 0.092 ms |
| the canonical loop, 200 × `example` | 1 | 616.5 → 623.1 ms | +6.6 ms; `now_twin` +1.8 ms: **not a finding** | 4.8 ms |
| **the `rust/struct` loop**, 200 × `rust/struct` | 1 | 1 592.9 → 793.5 ms | **−799.4 ms (−50.2%): `#248`** | 2.2 ms |

The canonical loop: `now` measured 623.1 ms and its twin 618.3 ms; every round
put `now` 6.1 to 13.0 ms above `b247` and the twin −0.2 to +5.9 ms. The
difference between two runs of one file is 4.8 ms, so the +1.8 to +6.6 ms is
not attributed to the code.

### Results — every path, against `#247`

Medians; `rsd` is of `now`. A change is in bold where it is at least 0.1 ms and
more than twice the A/A difference of its label. `<w>` is the scratch work
directory. The heap is `dhat`'s, one run: total allocated, and held at the
peak. Peak RSS is `/usr/bin/time -l`, median of 7 per arm.

| label | invocation | exit | `b247` | `now` | change | A/A | `now` rsd | heap total / at peak, `now` | peak RSS, `b247` → `now` |
|---|---|---|---|---|---|---|---|---|---|
| `version` | `tpl --version` | 0 | 1.719 ms | **1.730 ms** | +0.011 ms | 0.002 ms | 4.78% | 429 561 / 260 184 B | 2.61 → 2.64 MiB |
| `version_V` | `tpl -V` | 0 | 1.708 ms | **1.717 ms** | +0.010 ms | 0.013 ms | 2.50% | 429 547 / 260 170 B | — |
| `version_cmd` | `tpl version` | 0 | 1.734 ms | **1.733 ms** | −0.000 ms | 0.009 ms | 3.10% | 436 543 / 262 458 B | — |
| `bare` | `tpl` | 0 | 1.823 ms | **1.825 ms** | +0.003 ms | 0.002 ms | 3.93% | 722 481 / 259 209 B | — |
| `help` | `tpl --help` | 0 | 1.830 ms | **1.831 ms** | +0.001 ms | 0.003 ms | 2.36% | 723 569 / 260 175 B | 3.00 → 2.92 MiB |
| `help_h` | `tpl -h` | 0 | 1.830 ms | **1.826 ms** | −0.004 ms | 0.009 ms | 2.16% | 723 561 / 260 167 B | — |
| `help_cmd` | `tpl help` | 0 | 1.838 ms | **1.840 ms** | +0.002 ms | 0.001 ms | 2.00% | 733 859 / 264 082 B | — |
| `helpjson` | `tpl help --format json` | 0 | 1.914 ms | **1.915 ms** | +0.001 ms | 0.002 ms | 2.13% | 795 725 / 316 614 B | — |
| `helpjson_p` | `tpl help --format json --pretty` | 0 | 1.950 ms | **1.944 ms** | −0.006 ms | 0.000 ms | 2.30% | 796 900 / 316 614 B | 3.20 → 3.12 MiB |
| `helppath` | `tpl help schema table` | 0 | 1.837 ms | **1.849 ms** | +0.011 ms | 0.004 ms | 2.44% | 725 360 / 264 702 B | — |
| `helppathjson` | `tpl help schema table --format json` | 0 | 1.803 ms | **1.814 ms** | +0.011 ms | 0.001 ms | 2.17% | 715 892 / 265 643 B | — |
| `nodehelp` | `tpl schema table --help` | 0 | 1.962 ms | **1.977 ms** | +0.015 ms | 0.005 ms | 2.16% | 1 460 312 / 670 749 B | 3.66 → 3.58 MiB |
| `g_schema` | `tpl schema` | 0 | 1.820 ms | **1.840 ms** | +0.021 ms | 0.004 ms | 2.78% | 823 388 / 332 391 B | — |
| `g_template` | `tpl template` | 0 | 1.810 ms | **1.822 ms** | +0.012 ms | 0.018 ms | 2.45% | 759 299 / 295 029 B | — |
| `g_cache` | `tpl cache` | 0 | 1.815 ms | **1.812 ms** | −0.003 ms | 0.001 ms | 2.13% | 755 329 / 291 678 B | — |
| `g_cfg` | `tpl cfg` | 0 | 1.811 ms | **1.819 ms** | +0.008 ms | 0.006 ms | 2.38% | 779 843 / 307 932 B | — |
| `g_cfgdb` | `tpl cfg database` | 0 | 1.838 ms | **1.838 ms** | −0.000 ms | 0.008 ms | 2.58% | 895 877 / 374 560 B | — |
| `g_cfgdb_alias` | `tpl cfg db` | 0 | 1.831 ms | **1.844 ms** | +0.013 ms | 0.008 ms | 2.40% | 895 865 / 374 548 B | — |
| `fail64` | `tpl -d bench_wl001 schema table accrual --no-such-flag` | 64 | 1.741 ms | **1.750 ms** | +0.008 ms | 0.004 ms | 2.52% | 478 250 / 329 992 B | — |
| `fail64_cmd` | `tpl schemx` | 64 | 1.698 ms | **1.702 ms** | +0.004 ms | 0.008 ms | 2.61% | 358 073 / 256 300 B | — |
| `fail78_noproj` | `tpl schema tables` | 78 | 1.834 ms | **1.844 ms** | +0.010 ms | 0.002 ms | 2.13% | 502 202 / 338 576 B | — |
| `fail69_conn` | `tpl schema tables` | 69 | 1.937 ms | **1.946 ms** | +0.010 ms | 0.012 ms | 2.40% | 561 993 / 338 576 B | 3.59 → 3.62 MiB |
| `fail69_conn_d` | `tpl schema table accrual --direct --no-cache` | 69 | 1.953 ms | **1.952 ms** | −0.001 ms | 0.003 ms | 2.22% | 566 768 / 340 921 B | — |
| `fail66_tpl` | `tpl template show nosuch` | 66 | 1.866 ms | **1.863 ms** | −0.002 ms | 0.013 ms | 2.20% | 442 489 / 299 949 B | — |
| `fail66_rnd` | `tpl render nosuch` | 66 | 1.881 ms | **1.885 ms** | +0.004 ms | 0.002 ms | 2.11% | 399 693 / 265 001 B | — |
| `fail66` | `tpl -d bench_wl001 schema table accrualx` | 66 | 29.978 ms | **29.956 ms** | −0.022 ms | 0.081 ms | 1.61% | 6 411 481 / 4 200 767 B | 8.72 → 8.69 MiB |
| `tlist` | `tpl template list` | 0 | 1.841 ms | **1.844 ms** | +0.002 ms | 0.009 ms | 2.88% | 503 957 / 300 010 B | 2.98 → 3.02 MiB |
| `tlistjson` | `tpl template list --format json` | 0 | 1.829 ms | **1.831 ms** | +0.001 ms | 0.002 ms | 2.98% | 504 425 / 300 106 B | — |
| `tlist_srv` | `tpl template list` | 0 | 1.849 ms | **1.856 ms** | +0.007 ms | 0.010 ms | 2.76% | 505 802 / 300 010 B | — |
| `tshow` | `tpl template show rust/schema` | 0 | 1.836 ms | **1.846 ms** | +0.011 ms | 0.002 ms | 2.20% | 517 783 / 299 969 B | — |
| `tcheck` | `tpl template check rust/schema` | 0 | 1.956 ms | **1.951 ms** | −0.005 ms | 0.003 ms | 2.41% | 510 329 / 299 973 B | — |
| `tpath` | `tpl template path rust/struct` | 0 | 1.816 ms | **1.825 ms** | +0.009 ms | 0.002 ms | 2.23% | 505 355 / 300 630 B | — |
| `tpathjson` | `tpl template path rust/struct --format json` | 0 | 1.824 ms | **1.843 ms** | +0.020 ms | 0.019 ms | 2.98% | 505 703 / 300 750 B | — |
| `cfglist` | `tpl cfg list` | 0 | 1.841 ms | **1.852 ms** | +0.011 ms | 0.003 ms | 2.61% | 539 434 / 312 889 B | 3.23 → 3.22 MiB |
| `cfglistjson` | `tpl cfg list --format json` | 0 | 1.849 ms | **1.859 ms** | +0.010 ms | 0.014 ms | 3.20% | 541 255 / 312 985 B | — |
| `cfgget` | `tpl cfg get core.database` | 0 | 1.838 ms | **1.844 ms** | +0.006 ms | 0.000 ms | 2.30% | 540 456 / 313 514 B | — |
| `cfggetjson` | `tpl cfg get core.database --format json` | 0 | 1.842 ms | **1.849 ms** | +0.006 ms | 0.001 ms | 2.27% | 540 795 / 313 634 B | — |
| `cfgset` | `tpl cfg set core.database bench_wl001` | 0 | 1.986 ms | **1.985 ms** | −0.001 ms | 0.009 ms | 2.04% | 496 880 / 313 757 B | — |
| `cfgunset` | `tpl cfg unset core.query_timeout` | 0 | 1.982 ms | **1.983 ms** | +0.000 ms | 0.003 ms | 2.22% | 495 691 / 312 878 B | — |
| `dblist` | `tpl cfg database list` | 0 | 1.860 ms | **1.865 ms** | +0.005 ms | 0.006 ms | 2.25% | 655 240 / 379 309 B | — |
| `dblist_alias` | `tpl cfg db list` | 0 | 1.873 ms | **1.860 ms** | −0.013 ms | 0.003 ms | 2.08% | 655 228 / 379 297 B | — |
| `dblistjson` | `tpl cfg database list --format json` | 0 | 1.857 ms | **1.861 ms** | +0.004 ms | 0.004 ms | 2.74% | 655 732 / 379 429 B | — |
| `dbshow` | `tpl cfg database show bench_wl001` | 0 | 1.872 ms | **1.869 ms** | −0.003 ms | 0.006 ms | 1.94% | 657 450 / 379 953 B | 3.33 → 3.31 MiB |
| `dbshowjson` | `tpl cfg database show bench_wl001 --format json` | 0 | 1.863 ms | **1.866 ms** | +0.002 ms | 0.001 ms | 2.27% | 657 462 / 380 073 B | — |
| `dbadd` | `tpl cfg database add scratch --host 127.0.0.1 --port 13309 --user root --schema freight_wl003 --tls disabled` | 0 | 2.019 ms | **2.023 ms** | +0.004 ms | 0.002 ms | 1.98% | 629 202 / 385 135 B | 3.45 → 3.38 MiB |
| `dbupdate` | `tpl cfg database update scratch --port 13309` | 0 | 2.003 ms | **2.003 ms** | −0.000 ms | 0.005 ms | 1.90% | 624 577 / 380 453 B | — |
| `dbremove` | `tpl cfg database remove scratch` | 0 | 2.006 ms | **1.993 ms** | −0.013 ms | 0.011 ms | 2.19% | 620 295 / 379 282 B | — |
| `dbtest` | `tpl cfg database test bench_wl001` | 0 | 3.982 ms | **4.023 ms** | +0.041 ms | 0.048 ms | 3.79% | 732 542 / 379 953 B | — |
| `dbtestjson` | `tpl cfg database test bench_wl001 --format json` | 0 | 3.966 ms | **3.976 ms** | +0.010 ms | 0.007 ms | 2.95% | 732 554 / 380 073 B | — |
| `init` | `tpl init <w>/initdir` | 0 | 2.127 ms | **2.135 ms** | +0.008 ms | 0.002 ms | 2.34% | 379 350 / 264 397 B | 2.80 → 2.77 MiB |
| `cstatus` | `tpl -d bench_wl001 cache status` | 0 | 2.046 ms | **2.060 ms** | +0.015 ms | 0.006 ms | 2.22% | 776 473 / 298 913 B | 3.38 → 3.36 MiB |
| `cstatusjson` | `tpl -d bench_wl001 cache status --format json` | 0 | 2.038 ms | **2.053 ms** | +0.014 ms | 0.003 ms | 2.30% | 645 086 / 299 033 B | — |
| `cload` | `tpl -d bench_wl001 cache load` | 0 | 30.076 ms | **29.781 ms** | −0.295 ms | 0.152 ms | 1.04% | 6 337 247 / 4 199 983 B | 8.59 → 8.52 MiB |
| `cloadtable` | `tpl -d bench_wl001 cache load --table accrual` | 0 | 22.772 ms | **22.763 ms** | −0.009 ms | 0.066 ms | 1.87% | 5 544 425 / 3 913 154 B | 8.14 → 8.11 MiB |
| `cclean` | `tpl -d bench_wl003 cache clean` | 0 | 2.308 ms | **2.283 ms** | −0.026 ms | 0.024 ms | 3.70% | 456 079 / 297 989 B | — |
| `ccleantable` | `tpl -d bench_wl003 cache clean --table consignment` | 0 | 2.160 ms | **2.142 ms** | −0.018 ms | 0.065 ms | 3.21% | 469 223 / 299 084 B | — |
| `info_c` | `tpl -d bench_wl001 schema info` | 0 | 2.089 ms | **2.080 ms** | −0.008 ms | 0.002 ms | 2.43% | 714 668 / 341 732 B | 3.48 → 3.47 MiB |
| `infojson_c` | `tpl -d bench_wl001 schema info --format json` | 0 | 2.077 ms | **2.080 ms** | +0.003 ms | 0.001 ms | 2.79% | 714 737 / 341 852 B | — |
| `infopretty_c` | `tpl -d bench_wl001 schema info --format json --pretty` | 0 | 2.074 ms | **2.076 ms** | +0.002 ms | 0.002 ms | 2.52% | 715 912 / 342 369 B | — |
| `tables_c` | `tpl -d bench_wl001 schema tables` | 0 | 7.475 ms | **7.482 ms** | +0.007 ms | 0.033 ms | 1.65% | 3 910 054 / 3 255 174 B | 7.02 → 7.11 MiB |
| `tables_alias_c` | `tpl -d bench_wl001 schema tbls` | 0 | 7.470 ms | **7.466 ms** | −0.004 ms | 0.007 ms | 2.27% | 3 910 050 / 3 255 174 B | — |
| `tablesjson_c` | `tpl -d bench_wl001 schema tables --format json` | 0 | 12.208 ms | **12.166 ms** | −0.042 ms | 0.013 ms | 1.34% | 11 674 108 / 8 139 408 B | 12.83 → 12.81 MiB |
| `tablespretty_c` | `tpl -d bench_wl001 schema tables --format json --pretty` | 0 | 13.669 ms | **13.162 ms** | **−0.507 ms, −3.7%** | 0.029 ms | 1.55% | 11 675 283 / 8 139 408 B | 12.69 → 12.78 MiB |
| `tablespat_c` | `tpl -d bench_wl001 schema tables --pattern acc%` | 0 | 7.431 ms | **7.445 ms** | +0.013 ms | 0.006 ms | 1.84% | 3 890 156 / 3 232 813 B | — |
| `table_c` | `tpl -d bench_wl001 schema table accrual` | 0 | 1.978 ms | **1.991 ms** | +0.013 ms | 0.006 ms | 2.28% | 965 640 / 342 363 B | 3.77 → 3.73 MiB |
| `table_alias_c` | `tpl -d bench_wl001 schema tbl accrual` | 0 | 1.986 ms | **1.977 ms** | −0.009 ms | 0.006 ms | 2.28% | 965 636 / 342 359 B | — |
| `tablejson_c` | `tpl -d bench_wl001 schema table accrual --format json` | 0 | 1.971 ms | **1.982 ms** | +0.011 ms | 0.016 ms | 2.24% | 633 322 / 342 483 B | — |
| `tablepretty_c` | `tpl -d bench_wl001 schema table accrual --format json --pretty` | 0 | 1.974 ms | **1.979 ms** | +0.005 ms | 0.001 ms | 2.02% | 634 497 / 343 000 B | — |
| `table3_c` | `tpl -d bench_wl003 schema table consignment` | 0 | 1.959 ms | **1.956 ms** | −0.003 ms | 0.001 ms | 2.22% | 1 079 219 / 342 379 B | — |
| `views_c` | `tpl -d bench_wl001 schema views` | 0 | 2.266 ms | **2.273 ms** | +0.008 ms | 0.002 ms | 2.02% | 664 284 / 341 799 B | — |
| `views_alias_c` | `tpl -d bench_wl001 schema vws` | 0 | 2.264 ms | **2.268 ms** | +0.004 ms | 0.001 ms | 2.00% | 664 280 / 341 795 B | — |
| `viewsjson_c` | `tpl -d bench_wl001 schema views --format json` | 0 | 2.286 ms | **2.278 ms** | −0.008 ms | 0.014 ms | 2.23% | 661 672 / 341 919 B | — |
| `view_c` | `tpl -d bench_wl001 schema view v_booking_line_summary` | 0 | 1.926 ms | **1.910 ms** | −0.016 ms | 0.010 ms | 2.03% | 598 986 / 342 420 B | 3.50 → 3.50 MiB |
| `view_alias_c` | `tpl -d bench_wl001 schema vw v_booking_line_summary` | 0 | 1.909 ms | **1.910 ms** | +0.001 ms | 0.004 ms | 2.21% | 598 982 / 342 416 B | — |
| `routines_c` | `tpl -d bench_wl001 schema routines` | 0 | 2.443 ms | **2.453 ms** | +0.009 ms | 0.006 ms | 2.01% | 791 473 / 341 808 B | — |
| `routines_alias_c` | `tpl -d bench_wl001 schema rtns` | 0 | 2.450 ms | **2.456 ms** | +0.006 ms | 0.001 ms | 1.91% | 791 465 / 341 800 B | — |
| `routinesjson_c` | `tpl -d bench_wl001 schema routines --format json` | 0 | 2.478 ms | **2.472 ms** | −0.006 ms | 0.002 ms | 2.46% | 786 861 / 341 928 B | — |
| `routine_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count` | 0 | 2.443 ms | **2.478 ms** | +0.036 ms | 0.012 ms | 2.20% | 853 891 / 342 449 B | 3.91 → 3.89 MiB |
| `routine_alias_c` | `tpl -d bench_wl001 schema rtn fn_consignment_hazard_count` | 0 | 2.444 ms | **2.449 ms** | +0.004 ms | 0.008 ms | 1.65% | 853 883 / 342 441 B | — |
| `dump_c` | `tpl -d bench_wl001 schema dump` | 0 | 13.239 ms | **13.087 ms** | **−0.151 ms, −1.1%** | 0.043 ms | 1.64% | 11 926 932 / 8 263 955 B | 13.06 → 13.12 MiB |
| `dumppretty_c` | `tpl -d bench_wl001 schema dump --pretty` | 0 | 14.728 ms | **14.067 ms** | **−0.661 ms, −4.5%** | 0.038 ms | 1.34% | 11 930 187 / 8 263 955 B | 13.05 → 13.20 MiB |
| `tpldir_c` | `tpl --tpl-dir <w>/server/.tpl -d bench_wl001 schema table accrual` | 0 | 1.976 ms | **1.989 ms** | +0.014 ms | 0.002 ms | 2.50% | 970 866 / 344 172 B | — |
| `info_d` | `tpl -d bench_wl001 schema info --direct --no-cache` | 0 | 23.085 ms | **23.060 ms** | −0.026 ms | 0.000 ms | 1.20% | 5 620 651 / 3 941 827 B | — |
| `tables_d` | `tpl -d bench_wl001 schema tables --direct --no-cache` | 0 | 23.084 ms | **23.098 ms** | +0.014 ms | 0.012 ms | 1.37% | 5 659 038 / 3 980 104 B | — |
| `table_d` | `tpl -d bench_wl001 schema table accrual --direct --no-cache` | 0 | 23.120 ms | **23.050 ms** | −0.069 ms | 0.050 ms | 1.32% | 5 954 240 / 3 944 987 B | 8.19 → 8.14 MiB |
| `table3_d` | `tpl -d bench_wl003 schema table consignment --direct --no-cache` | 0 | 11.378 ms | **11.289 ms** | −0.090 ms | 0.086 ms | 1.95% | 1 049 405 / 343 227 B | — |
| `views_d` | `tpl -d bench_wl001 schema views --direct --no-cache` | 0 | 23.096 ms | **23.124 ms** | +0.028 ms | 0.035 ms | 2.23% | 5 623 649 / 3 944 722 B | — |
| `view_d` | `tpl -d bench_wl001 schema view v_booking_line_summary --direct --no-cache` | 0 | 23.073 ms | **23.096 ms** | +0.023 ms | 0.020 ms | 1.75% | 5 622 028 / 3 941 816 B | — |
| `routines_d` | `tpl -d bench_wl001 schema routines --direct --no-cache` | 0 | 23.010 ms | **23.181 ms** | +0.171 ms | 0.089 ms | 1.74% | 5 625 750 / 3 946 802 B | — |
| `routine_d` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --direct --no-cache` | 0 | 23.060 ms | **23.120 ms** | +0.060 ms | 0.057 ms | 1.33% | 5 688 008 / 3 942 209 B | — |
| `dump_d` | `tpl -d bench_wl001 schema dump --direct --no-cache` | 0 | 25.547 ms | **25.429 ms** | **−0.119 ms, −0.5%** | 0.036 ms | 1.21% | 5 618 980 / 3 941 354 B | 8.19 → 8.11 MiB |
| `dump_dw` | `tpl -d bench_wl001 schema dump --direct` | 0 | 32.368 ms | **32.089 ms** | **−0.279 ms, −0.9%** | 0.044 ms | 1.11% | 6 472 347 / 4 200 751 B | — |
| `rexample_c` | `tpl -d bench_wl001 render example --table accrual` | 0 | 2.377 ms | **2.389 ms** | +0.011 ms | 0.003 ms | 2.70% | 819 450 / 315 354 B | 4.52 → 4.59 MiB |
| `rexample_vvv_c` | `tpl -vvv -d bench_wl001 render example --table accrual` | 0 | 2.388 ms | **2.392 ms** | +0.004 ms | 0.012 ms | 1.87% | 822 164 / 315 418 B | — |
| `rexample_q_c` | `tpl -q -d bench_wl001 render example --table accrual` | 0 | 2.383 ms | **2.388 ms** | +0.005 ms | 0.003 ms | 2.36% | 820 170 / 315 354 B | — |
| `rexample_to_c` | `tpl --timeout 30 -d bench_wl001 render example --table accrual` | 0 | 2.377 ms | **2.389 ms** | +0.012 ms | 0.013 ms | 4.11% | 822 167 / 315 386 B | — |
| `rexample_set_c` | `tpl -d bench_wl001 render example --table accrual --set title=Accrual` | 0 | 2.391 ms | **2.382 ms** | −0.009 ms | 0.004 ms | 2.06% | 821 791 / 316 375 B | — |
| `rstruct_c` | `tpl -d bench_wl001 render rust/struct --table accrual` | 0 | 3.006 ms | **3.027 ms** | +0.022 ms | 0.018 ms | 2.97% | 2 310 167 / 457 092 B | 4.92 → 5.03 MiB |
| `rschema_c` | `tpl -d bench_wl001 render rust/schema` | 0 | 16.680 ms | **16.756 ms** | +0.076 ms | 0.009 ms | 1.45% | 30 438 161 / 8 560 327 B | 13.92 → 13.91 MiB |
| `rview_c` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary` | 0 | 2.284 ms | **2.292 ms** | +0.008 ms | 0.012 ms | 2.62% | 715 360 / 269 379 B | 4.12 → 4.17 MiB |
| `rroutine_c` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count` | 0 | 2.287 ms | **2.301 ms** | +0.014 ms | 0.004 ms | 2.07% | 716 751 / 269 420 B | 4.17 → 4.17 MiB |
| `rwjson_c` | `tpl -d bench_wl001 render probe/whole_json` | 0 | 26.525 ms | **26.477 ms** | −0.047 ms | 0.136 ms | 1.07% | 48 520 064 / 34 115 196 B | 43.19 → 43.11 MiB |
| `rwwalk_c` | `tpl -d bench_wl001 render probe/whole_walk` | 0 | 104.887 ms | **104.875 ms** | −0.012 ms | 0.319 ms | 1.14% | 824 146 275 / 29 621 960 B | 36.73 → 36.47 MiB |
| `rexample_x` | `tpl render example --table accrual --context <w>/wl001-dump.json` | 0 | 7.994 ms | **8.026 ms** | +0.032 ms | 0.004 ms | 7.69% | 12 619 202 / 8 244 432 B | 13.47 → 13.45 MiB |
| `rexample_xin` | `tpl render example --table accrual --context - < <w>/wl001-dump.json` | 0 | 8.109 ms | **8.126 ms** | +0.017 ms | 0.003 ms | 1.23% | 17 832 848 / 9 264 544 B | 15.34 → 15.33 MiB |
| `rschema_x` | `tpl render rust/schema --context <w>/wl001-dump.json` | 0 | 14.331 ms | **14.360 ms** | +0.029 ms | 0.006 ms | 1.18% | 31 505 003 / 8 244 333 B | — |
| `rview_x` | `tpl render probe/view --view v_booking_line_summary --context <w>/wl001-dump.json` | 0 | 7.949 ms | **7.936 ms** | −0.012 ms | 0.031 ms | 1.58% | 12 549 137 / 8 244 450 B | — |
| `rwjson_x` | `tpl render probe/whole_json --context <w>/wl001-dump.json` | 0 | 22.776 ms | **22.635 ms** | −0.141 ms | 0.088 ms | 0.92% | 49 453 347 / 31 560 455 B | 41.86 → 41.88 MiB |
| `rexample_d` | `tpl -d bench_wl001 render example --table accrual --direct --no-cache` | 0 | 23.317 ms | **23.306 ms** | −0.012 ms | 0.043 ms | 6.55% | 5 617 062 / 4 044 356 B | 8.75 → 8.73 MiB |
| `rstruct_d` | `tpl -d bench_wl001 render rust/struct --table accrual --direct --no-cache` | 0 | 24.045 ms | **24.057 ms** | +0.012 ms | 0.072 ms | 1.08% | 7 108 317 / 4 186 094 B | — |
| `rschema_d` | `tpl -d bench_wl001 render rust/schema --direct --no-cache` | 0 | 29.744 ms | **29.691 ms** | −0.053 ms | 0.101 ms | 0.96% | 24 504 539 / 4 282 690 B | — |
| `rview_d` | `tpl -d bench_wl001 render probe/view --view v_booking_line_summary --direct --no-cache` | 0 | 23.287 ms | **23.247 ms** | −0.040 ms | 0.228 ms | 3.08% | 5 547 073 / 3 980 698 B | — |
| `rroutine_d` | `tpl -d bench_wl001 render probe/routine --routine fn_consignment_hazard_count --direct --no-cache` | 0 | 23.231 ms | **23.280 ms** | +0.049 ms | 0.095 ms | 1.20% | 5 547 633 / 3 981 435 B | — |
| `rwjson_d` | `tpl -d bench_wl001 render probe/whole_json --direct --no-cache` | 0 | 38.381 ms | **38.243 ms** | **−0.138 ms, −0.4%** | 0.011 ms | 0.84% | 42 431 525 / 29 718 239 B | 37.27 → 37.31 MiB |
| `viewjson_c` | `tpl -d bench_wl001 schema view v_booking_line_summary --format json` | 0 | 1.922 ms | **1.929 ms** | +0.007 ms | 0.008 ms | 2.95% | 598 902 / 342 540 B | — |
| `routinejson_c` | `tpl -d bench_wl001 schema routine fn_consignment_hazard_count --format json` | 0 | 2.437 ms | **2.465 ms** | +0.027 ms | 0.010 ms | 2.33% | 787 883 / 342 569 B | — |
| `rstruct_last_c` | `tpl -d bench_wl001 render rust/struct --table yard_position` | 0 | 11.396 ms | **3.421 ms** | **−7.975 ms, −70.0%** | 0.015 ms | 2.02% | 3 464 675 / 491 711 B | 14.05 → 5.09 MiB |
| `rmiss_reach` | `tpl -d bench_wl001 render probe/whole_json`, `tables/yard_position.json` damaged | 0 | 67.836 ms | **67.186 ms** | **−0.650 ms, −1.0%** | 0.092 ms | 0.77% | 90 784 002 / 56 124 837 B | 70.55 → 70.56 MiB |
| `rmiss_bound` | `tpl -d bench_wl001 render example --table accrual`, `tables/accrual.json` damaged | 0 | 30.983 ms | **30.874 ms** | **−0.109 ms, −0.4%** | 0.019 ms | 1.05% | 6 677 240 / 4 314 745 B | 9.56 → 9.50 MiB |

The two loops (12 samples per arm):

| label | `b247` | `now` | `now_twin` | change | A/A | `now` rsd | per-round change |
|---|---|---|---|---|---|---|---|
| loop, 200 × `example` | 616.5 ms | 623.1 ms | 618.3 ms | +6.6 ms (twin +1.8 ms) | 4.8 ms | 1.49% | +6.1 to +13.0 ms |
| `loop_struct`, 200 × `rust/struct` | 1 592.9 ms | **793.5 ms** | 791.3 ms | **−799.4 ms, −50.2%** | 2.2 ms | 0.60% | −814.6 to −796.5 ms |

### Results — the experiments

Copies of `git archive 2e21918`, one change each, each built into its own
target directory from a path of the same length. Each is an **instrument**,
never a proposed change; where it removes a check, it says so.

| variant | the one change | binary |
|---|---|---|
| `ctl250` | none: the arm every variant is compared with | 4 017 040 B, sha256 `9f2f9cec…` |
| `dbl250` | `Cache::shelved`, `src/cache.rs:638`, makes the three listings of `shelve`, `:1081`, a second time and forgets them: the **whole** cost of a listing | 4 017 040 B, `36f0f26a…` |
| `sys250` | the same place makes the three `fs::read_dir` walks a second time, draining every entry and building nothing: the **system-call** part | 4 033 552 B, `d70e8ee9…` |
| `grw250` | `TableShape::columns`, `src/model/document/shape.rs:179`, decodes through a visitor that starts from `Vec::with_capacity(16)` (`WL-001` has 12 columns per table) | 4 017 040 B, `143b2c3c…` |
| `ctx250` | an embedded table — `ForeignKeyShape::referenced_table`, `IncomingKey::table` — decodes its `name` alone; every other key is skipped by `serde` without being validated, and the table carries empty defaults. **It removes the structural check of `FR-RND-020` on embedded tables**, and, the shape being shared, empties the embedded tables of every cached read too: it is valid on the `--context` labels only | 4 017 040 B, `6dee4892…` |
| `raw250` | the compact `schema dump` and the unfiltered compact `schema tables --format json`, served from the cache, are the envelope around each file's bytes, in listing order, with **no decode**: the upper bound of row 2, **against `FR-CACHE-033`** | 4 033 568 B, `7fa3b185…` |
| `vld250` | `raw250`, after every file read has been decoded by `Loaded::document` exactly as a hit requires; a file that fails is a miss, as today | 4 033 568 B, `03b3164f…` |
| `eag250` | `attempt`, `src/cli/render.rs:585`, reads every member of the store before the render starts. **Against `FR-CACHE-038`** | 4 017 040 B, `7179acc9…` |
| `fre250` | an abandoned render's context and store are dropped instead of released through `Ending::release`, `src/cli/render.rs:611` and `:575` | 4 017 040 B, `d49e8cdb…` |
| `ver250` | `main` answers exactly `tpl --version` before `tpl::run`, with the same bytes | 4 033 568 B, `c1769c88…` |
| `nop` | an empty `fn main() {}` under the release profile: the spawn floor | 302 464 B |

**Identity.** Every variant was run over the 119 labels and compared with
`now`. `ctl250`, `dbl250`, `sys250`, `grw250`, `raw250`, `vld250`, `eag250`,
`fre250` and `ver250` matched on every label; `raw250` and `vld250` print the
compact dump and table listing **byte for byte**. `ctx250` matched on every
`--context` label and differed, as expected, on the eight cached labels that
print an embedded table; over the `--context` document it was also compared
with `ctl250` on every worked template of `examples/*/templates/` and
`example`, bound to every tenth table and unbound: 399 invocations, all
identical. **With the bound table's file replaced by `{ torn`**, `vld250`
answered as `ctl250` does — a miss, the server read, `"source":"server"` —
while `raw250` served the damaged file from the cache: the check it removes is
real.

One rotated campaign of the main campaign's shape over nine arms — the eight
variants and `ctl250_twin` — on 15 labels; one over `ctl250`, `vld250`,
`raw250` and `ctl250_twin` on 3; the canonical loop over `ctl250`, `dbl250`,
`sys250`, `grw250` and `ctl250_twin`; and the start-up campaign over `nop`,
`ver250 --version`, `ctl250 --version`, `table_c`, `rexample_c` and the two
twins. 320 samples per arm and label, 160 for `rwwalk_c`, 40 for the two miss
labels, 12 for the loop. Each variant against `ctl250`:

| row | label | `ctl250` | variant | change | A/A | per-round change |
|---|---|---|---|---|---|---|
| **2** | `dump_c` | 13.213 ms | `vld250` 11.151 ms | **−2.062 ms, −15.6%** | 0.000 ms | −2.295 to −1.942 ms |
| | `tablesjson_c` | 12.222 ms | `vld250` 10.153 ms | **−2.069 ms, −16.9%** | 0.055 ms | −2.306 to −1.947 ms |
| | `dump_c` | 13.213 ms | `raw250` 5.512 ms | **−7.701 ms, −58.3%**, unvalidated | 0.000 ms | −7.904 to −7.581 ms |
| | `tablesjson_c` | 12.222 ms | `raw250` 4.643 ms | **−7.579 ms, −62.0%**, unvalidated | 0.055 ms | −7.751 to −7.486 ms |
| | `table_c`, control | 1.970 ms | `vld250` / `raw250` 1.973 / 1.971 ms | +0.003 / +0.000 ms | 0.003 ms | inside ±0.043 ms |
| **4** | `rexample_c` | 2.361 ms | `dbl250` 2.557 ms | **+0.196 ms**: the whole listing | 0.009 ms | +0.159 to +0.236 ms |
| | `rexample_c` | 2.361 ms | `sys250` 2.511 ms | **+0.150 ms**: its system calls | 0.009 ms | +0.122 to +0.202 ms |
| | `rview_c` | 2.281 ms | `dbl250` / `sys250` 2.482 / 2.422 ms | +0.201 / +0.141 ms | 0.013 ms | all rounds positive |
| | `rstruct_last_c` | 3.377 ms | `dbl250` / `sys250` 3.584 / 3.516 ms | +0.207 / +0.139 ms | 0.001 ms | all rounds positive |
| | loop | 614.8 ms | `dbl250` / `sys250` 656.9 / 649.9 ms | +42.1 / +35.1 ms | 7.7 ms | +29.2 to +47.7 ms |
| | `table_c`, control | 1.975 ms | `dbl250` / `sys250` 1.974 / 1.985 ms | −0.000 / +0.011 ms | 0.014 ms | both signs |
| **5** | `rexample_x` | 8.044 ms | `grw250` 7.850 ms | **−0.194 ms, −2.4%** | 0.017 ms | −0.289 to −0.116 ms |
| | `rexample_xin` | 8.123 ms | 8.018 ms | −0.105 ms | 0.005 ms | −0.256 to +0.032 ms |
| | `dump_c` / `tablesjson_c` | 13.079 / 12.144 ms | 12.963 / 12.027 ms | −0.117 / −0.117 ms | 0.027 / 0.011 ms | both signs |
| | `rwjson_c` | 26.439 ms | 26.107 ms | −0.332 ms | 0.036 ms | −0.645 to +0.083 ms |
| | `rexample_c`; loop | 2.361 ms; 614.8 ms | 2.377 ms; 616.1 ms | +0.017 ms; +1.3 ms | 0.009 ms; 7.7 ms | both signs |
| **6** | `rexample_x` | 8.044 ms | `ctx250` 6.482 ms | **−1.562 ms, −19.4%**, unvalidated | 0.017 ms | −1.740 to −1.424 ms |
| | `rexample_xin` | 8.123 ms | 6.613 ms | **−1.511 ms, −18.6%** | 0.005 ms | −1.614 to −1.384 ms |
| | `rschema_x` | 14.337 ms | 12.849 ms | **−1.488 ms, −10.4%** | 0.000 ms | −1.693 to −1.353 ms |
| **7** | `rwjson_c` | 26.439 ms | `eag250` 25.872 ms | **−0.566 ms, −2.1%** | 0.036 ms | −0.839 to −0.100 ms |
| | `rwwalk_c` | 104.874 ms | 104.053 ms | −0.820 ms | 0.068 ms | −2.045 to −0.110 ms |
| | `rschema_c`, which reaches no view or routine | 16.749 ms | 17.546 ms | **+0.797 ms** | 0.021 ms | +0.432 to +1.038 ms |
| | `rexample_c`, bound | 2.361 ms | 11.063 ms | +8.702 ms | 0.009 ms | — |
| **8** | `rmiss_reach` | 67.167 ms | `fre250` 69.344 ms | **+2.177 ms, +3.2%** | 0.140 ms | +1.591 to +3.419 ms |
| | `rmiss_bound`, `rexample_c`, `table_c` | 30.932 / 2.361 / 1.975 ms | 30.907 / 2.379 / 1.999 ms | −0.026 / +0.018 / +0.024 ms | ≤ 0.036 ms | both signs |
| start-up | `nop` | 1.302 ms | — | — | 0.016 ms | — |
| | `tpl --version`, answered before `tpl::run` | — | `ver250` 1.555 ms | 0.253 ms above `nop` | — | +0.228 to +0.273 ms |
| | `tpl --version` | `ctl250` 1.694 ms, twin 1.704 ms | — | 0.139–0.149 ms above `ver250` | 0.010 ms | — |
| | `table_c`, `rexample_c` | 1.971, 2.367 ms | — | 0.669, 1.065 ms above `nop` | — | — |

Peak resident memory, median of 7:

| label | `ctl250` | `fre250` | `eag250` | `grw250` | `ctx250` | `vld250` |
|---|---|---|---|---|---|---|
| `rmiss_reach` | 70.62 MiB | **46.78 MiB** | 70.22 MiB | 70.83 MiB | 53.30 MiB | 70.62 MiB |
| `rwjson_c` | 43.22 MiB | 43.05 MiB | 42.22 MiB | 43.36 MiB | 21.98 MiB | 43.25 MiB |
| `rexample_x` | 13.55 MiB | 13.38 MiB | 13.42 MiB | 13.50 MiB | 10.50 MiB | 13.52 MiB |
| `dump_c` | 13.17 MiB | 13.00 MiB | 13.16 MiB | 13.25 MiB | 10.12 MiB | 16.20 MiB |

- **Row 2.** Re-encoding the decoded document costs 2.06 ms of a compact
  cached whole read; the decode that validates the files costs the other
  5.64 ms of `raw250`'s 7.70 ms. `vld250` holds `FR-CACHE-033` — a damaged file
  is still a miss — and prints the same bytes for every file `tpl` writes,
  because each file is the compact encoding the dump would produce. It would
  not for a file that decodes but is not in that form (a file re-indented by
  hand): the output would then break `FR-OUT-007`, which the re-encode
  guarantees today. Its peak memory is 3 MiB higher: the output is built in
  memory before it is written.
- **Row 4.** A listing costs 0.20 ms of a bound render and 0.21 ms per render
  of the loop; 0.14 to 0.15 ms of it is the directory reads, which
  `FR-CACHE-038` requires. The per-entry work — a `PathBuf`, a `String`, the
  sort — is the difference, **0.05 to 0.07 ms**, below the run-to-run standard
  deviation of a bound render (2.8% of `ctl250`'s 2.361 ms, 0.066 ms), and inside the
  loop's A/A (35.1 against 42.1 ms, A/A 7.7 ms). The profile puts 12.2% of a
  bound render's samples in that per-entry work (≈ 0.29 ms, an upper bound);
  the experiment measures a second listing in a warm process, and that the
  gap is the first listing's page faults and allocator growth is
  **hypothesised, not established**.
- **Row 5.** A presized column list is established on `rexample_x` alone,
  −0.19 ms, 1.7 times that label's run-to-run standard deviation (1.4% of
  8.044 ms, 0.112 ms); on every other label it is inside the per-round spread.
  The capacity is a guess fitted to `WL-001`, since `serde_json` gives no
  length.
- **Row 6.** Decoding an embedded table for its name alone is worth 1.5 to
  1.6 ms of a `--context` render and 3.05 MiB of its peak. The skipped keys
  are skipped **unvalidated**: a `--context` document whose embedded table
  breaks the structural rules of `context-document.md` would render instead
  of exiting `65`, which `FR-RND-020` requires. The decode is the cost of that
  check.
- **Row 7, attributed.** Reading every member before the render makes a
  template that reaches all of them 0.57 to 0.82 ms faster, and one that does
  not — `rust/schema`, which reaches no view or routine — 0.80 ms slower, and
  a bound render 8.7 ms slower. `#246`'s +0.42 ms and `#247`'s +0.46 ms on
  `probe/whole_json` are therefore the files read interleaved with the render,
  **established**. `FR-CACHE-038` forbids reading a file before the template
  reaches it: the cost is the requirement's.
- **Row 8.** Freeing the abandoned render lowers the peak of a mid-render miss
  by 23.8 MiB, and costs 2.18 ms, because the process frees block by block
  what it would otherwise leave to the exit. It is a trade of time for memory
  on a path only a damaged store reaches, and `FR-CACHE-039` requires the
  abandon and the second render themselves.
- **Start-up.** Of `tpl --version`'s 1.70 ms, 1.302 ms is the spawn floor
  (`exec` and `dyld` of an empty binary), 0.253 ms is loading `tpl`'s own
  4 MB binary and its start-up before `main`'s first line of work — the
  panic hook installed — and 0.139 to 0.149 ms is `tpl::run`: building the
  command tree, parsing, and writing. `clap` is 78.4% of `--version`'s on-CPU
  samples and `malloc` 51.2%; `#240`'s rows 5 and 6 found at most 0.08 ms of
  the parser removable.

### Where the remaining cost goes

Shares of on-CPU samples of a symbolised build of `2e21918` (`debug = true`,
`strip = false`), `samply` at 20 kHz, inclusive; a figure in milliseconds is a
share times the median of `now`, an estimate and an upper bound. The two flame
graphs asked for are `prof/rexample_c.json.gz` and
`prof/rstruct_last_c.json.gz` in the scratch directory, with their folded
stacks beside them; `samply load` renders each.

- **A cached render bound to one object, `rexample_c` (2.389 ms; flame
  graph).** 1.302 ms is the spawn floor and 0.253 ms the binary's load. Of the
  samples: the three listings 27.3% (15.1% in `libc` and the kernel, 12.2% in
  `tpl`'s per-entry work); the parser 13.8%; project discovery and
  configuration 11.8%; the template's path resolution 8.4%, and the three
  `canonicalize` calls 10.6% in all (6.0% `Root::find` from `resolve`, 3.0%
  project discovery, 1.6% `Root::find` from the loader); the render 16.5%, of
  which loading and compiling the template 6.6%; the bound table's read and
  decode 7.4%; the timer thread 4.7%, on another core; `malloc` 18.1% self.
- **`rust/struct` bound to the last table, `rstruct_last_c` (3.421 ms; flame
  graph).** The lookup of row 1 is gone: `crate::render::lookup` is 1.2% of
  the samples, and the member files the render reads 2.8%. The render is
  65.5%, and it is the template's own evaluation: `minijinja`'s VM 55.5%, the
  macros of `rust/_types.jinja` 36.5%, the `replace` filter 14.4%, the
  template's load and compile 14.3%. Nothing there is `tpl`'s code.
- **`schema dump` from the cache, `dump_c` (13.087 ms).** Row 2: the decode
  that validates is 5.6 ms of it by experiment, the re-encode 2.1 ms.
- **A `--context` render, `rexample_x` (8.026 ms).** Row 6: the decode of the
  embedded tables is 1.5 to 1.6 ms of it by experiment.
- **Allocation.** A bound cached render allocates 819 450 B and holds
  315 354 B at its peak; `rstruct_last_c` 3 464 675 B and 491 711 B, against
  14 222 619 B and 8 543 872 B at `#247`.

### The waste register, sorted by measured gain

Effort is `#231`'s: `S` (one function, no interface change), `M` (a new code
path or a hand-written serde implementation), `L` (a new abstraction across
modules). Every figure is now an experiment's; none is an estimate. `#247`'s
rows 1 and 3 were applied by `#248` and `#249` and are confirmed above; its
row 9 is restated with this pass's figures.

| # | Path | Measured gain, with its noise | Cause in code | Effort | Functional risk | Required by the specification? |
|---|---|---|---|---|---|---|
| 2 | The compact cached `schema dump` and `schema tables --format json` | Unvalidated splice: **−7.70 and −7.58 ms (−58%, −62%)**, A/A 0.000 and 0.055 ms. Validated splice: **−2.06 and −2.07 ms (−16%, −17%)**, every round −1.94 to −2.31 ms; +3 MiB peak | `Loaded::document`, `src/cache.rs:175`, decodes every file and the presentation, `src/cli/schema.rs:543` and `:403`, re-encodes it | `M` | The unvalidated splice serves a damaged file (`FR-CACHE-033`). The validated one prints a file's bytes, which match the compact encoding only for a file `tpl` wrote: a hand-reformatted file that decodes would be printed as it is, against `FR-OUT-007` | **Yes, as it stands.** The 5.6 ms decode is `FR-CACHE-033`'s. The 2.06 ms re-encode is what makes `FR-OUT-007` hold for any file that decodes; removing it needs the specification to make a file not in compact form a miss, or accept it |
| 6 | The `--context` render | **−1.49 to −1.56 ms (−10% to −19%)**, A/A ≤ 0.017 ms, every round negative; −3.05 MiB peak | `model::document::read`, `src/model/document.rs:137`: every embedded table, `ForeignKeyShape::referenced_table` and `IncomingKey::table` in `src/model/document/shape.rs`, is decoded in full and only its name is kept | `M` | An embedded table that breaks the structural rules would no longer be refused | **Yes.** `FR-RND-020` makes a document that does not match the contract a `65`, and the embedded tables are part of it |
| 8 | A miss reached during the render | **−23.8 MiB** at the peak (70.62 → 46.78 MiB), at **+2.18 ms (+3.2%)**, every round +1.59 to +3.42 ms, A/A 0.140 ms | The abandoned context and store released through `Ending::release`, `src/cli/render.rs:611` and `:575` | `S` | None to the output; it is time traded for memory | The abandon and the second render are `FR-CACHE-039`'s; the leak is a choice. Only a damaged store reaches it |
| 7 | A cached render that reads the whole database | Reading every member up front: **−0.57 ms (−2.1%)** on `probe/whole_json`, −0.82 ms on `probe/whole_walk`; **+0.80 ms** on `rust/schema` and +8.7 ms on a bound render | The member files read as the template reaches them, `Members::convert`, `src/cli/render/context/lazy.rs` | — | — | **Yes.** `FR-CACHE-038` forbids reading a file before the template reaches it |
| 5 | Every decode of a whole document | **−0.19 ms (−2.4%)** on `rexample_x`, every round −0.12 to −0.29 ms, A/A 0.017 ms; 1.7 times its run-to-run σ. Inside the per-round spread on every other label, and on the loop | `serde`'s sequence visitor grows `columns`, `src/model/document/shape.rs:179`, from empty | `M` | None; the capacity would be a guess, `serde_json` giving no length | No. **Below the noise of practice**: 1.7 σ of one command, on no loop |
| 4 | Every cached render: the per-entry work of the listing | **0.05 to 0.07 ms** per bound render (whole listing 0.20 ms, its system calls 0.14–0.15 ms), below the run-to-run σ of 0.066 ms; 7 ms over the canonical loop, inside its A/A of 7.7 ms | `shelve`, `src/cache.rs:1081`: a `PathBuf` and a `String` per entry, and the sort | `S`–`M` | None | The listing and its order are `FR-CACHE-038`'s and `NFR-DET-002`'s; the per-entry work is **below the noise floor** |
| 9 | `render --context -` | +0.100 ms against the same document from a file (8.126 against 8.026 ms, A/A 0.003 and 0.004 ms); +5 213 646 B allocated, +1 020 112 B at the peak | `Vec::new()` and `read_to_end`, `src/cli/render.rs:782`–`785` | `S` | None | No. **1 σ of the command** (1.2% of 8.126 ms, 0.100 ms) |
| — | Start-up | The spawn floor is 1.302 ms; `tpl`'s binary load 0.253 ms; `tpl::run` for `--version` 0.139–0.149 ms (A/A 0.010 ms), of which `#240` found at most 0.08 ms removable | `exec`, `dyld`; the 4 MB binary; the `clap` tree, `src/cli.rs` | — | — | **Process floor**, and a parser the stack names |

### The verdict: does the campaign stop?

**It stops**, on the criterion set for it: every gain left is insignificant
against the noise floor of the command it affects, or is a cost the
specification requires, or is the process floor.

- **Required by the specification**: row 2 (5.6 ms of validation under
  `FR-CACHE-033`, and 2.06 ms of re-encode that is what holds `FR-OUT-007` for
  any file that decodes), row 6 (1.5 ms of validation under `FR-RND-020`), row
  7 (the interleaved read of `FR-CACHE-038`), and the time of row 8 (the
  second render of `FR-CACHE-039`). These are the two largest gains left, and
  they are established far above their floors — row 2 at 37 times its A/A, row
  6 at 90 times — but each is reached only by weakening a check. Whether to
  weaken one is a decision about the specification, for the user and
  `specification-manager`, not an optimisation.
- **Insignificant against the noise floor**: row 4 (0.05–0.07 ms, below one
  σ of a bound render and inside the loop's A/A), row 5 (0.19 ms, 1.7 σ of one
  command and nothing on any loop), row 9 (1 σ).
- **Process floor**: 1.302 ms of every invocation is the spawn floor and
  0.253 ms the binary's load; 0.14 ms more is the parser of `--version`.
- **A trade, not a waste**: row 8's memory, 23.8 MiB at the peak of a path
  only a damaged store reaches, costs 2.18 ms to recover.

**On the canonical loop the campaign has nothing left to take**: 3.1 ms per
render, of which 1.30 ms is the spawn floor, 0.25 ms the binary's load,
0.14 ms the parser, 0.20 ms the listing `FR-CACHE-038` requires, and the rest
the project, the template and the bound table's read. **On the `rust/struct`
loop**, 4.0 ms per render, the largest share is now the template's own
evaluation in `minijinja` (65.5% of `rstruct_last_c`'s samples), which is
not `tpl`'s code.

### Refuted hypotheses and non-findings, stated so they are not rediscovered

- **`#247`'s row 4 estimate was a profile artefact**: 12.2% of the samples,
  ≈ 0.29 ms, against 0.05–0.07 ms measured.
- **`#247`'s row 6 estimate was too low**: ≤ 0.54 ms estimated, 1.5–1.6 ms
  measured — and the whole of it is the check `FR-RND-020` requires.
- **`#247`'s row 7 hypothesis holds**: the interleaved read is the cost.
- **Freeing the abandoned render is not free**: it lowers the peak and costs
  2.2 ms.
- **The canonical loop did not move with `#248` or `#249`**: `example` makes
  no lookup and writes 494 bytes.
- **`#249` reaches every large output, not only the pretty ones**: `dump_c`
  −0.15 ms, `dump_d` −0.12 ms, `dump_dw` −0.28 ms, `rwjson_d` −0.14 ms,
  `rmiss_reach` −0.65 ms, each more than twice its A/A.
- **Nothing on the start-up, help, configuration, template, single-object,
  listing, direct-read, `--context` or store-write paths moved since `#247`**,
  and nothing regressed.

### Confounders — read this before the tables

- **The host was not idle.** The four containers of other projects named in
  `#240` ran throughout. The one-minute load average was 2.10 to 3.20 over the
  session. `hyperfine`
  printed 460 warnings in the main campaign, 52 in the experiments and 6 in the
  validated-splice campaign, about statistical outliers or a slow first run.
  `rexample_x`'s rsd in the main campaign, 7.69%, is such an outlier; its
  median and A/A (0.004 ms) are not affected.
- **`b247` is a rebuild of `ba10788`**, not `#247`'s file, which was lost with
  the scratch directory; its size matches that file's, 4 017 024 B.
- **The experiments' `ctl250` is not `now`**: the same source, built from a
  scratch path, 16 528 B smaller. Every variant is compared with it.
- **`dbl250` and `sys250` measure a second listing** in a process that has
  made one; the first can cost more.
- **`samply` shares carry the profiler's overhead, overlap, and are
  function-level**; an inlined function is counted in its caller. The first
  profiles of `rexample_c`, `rstruct_last_c`, `dump_c` and `rexample_x` were
  discarded: in the recording loop every iteration exited `64`, for a cause
  not investigated. They were recorded again one by one, every iteration exiting `0`.
- **`dhat` counts the heap only**, one run per label, through an instrumented
  build; it and the memory readings were taken between campaigns.
- **The server path crosses Docker Desktop's port proxy**, and the benchmark
  projects authenticate as `root` (`#224`). `up.sh 12.3` exits `2` on success
  (`#223`); the filtered gate `status.sh --quiet 12.3` was read instead.

### What was not measured — stated, not implied

- **Three of the four targets**, every TLS mode, and the series `10.11`,
  `11.4` and `11.8`, raised only for the test suite.
- **`vld250` over a hand-reformatted cache file**, where its output would
  differ; the risk is stated from the code, not observed.
- **`ctx250` over a `--context` document whose embedded table is malformed**;
  that it would render instead of exiting `65` follows from `serde` skipping
  unknown keys unvalidated.
- **A realistic implementation of rows 4, 5 or 8**: each figure is an
  instrument's.
- `#243`'s list of untested forms stands.

### Reproduction

```sh
S=/path/to/scratch            # any directory outside the repository
cargo build --release; cp target/release/tpl "$S/tpl-now"            # 2e21918, the binary of record
mkdir -p "$S/src/b247"; git archive ba10788 | tar -x -C "$S/src/b247"
(cd "$S/src/b247" && cargo build --release --target-dir "$S/t-b247"); cp "$S/t-b247/release/tpl" "$S/tpl-b247"

# 1. The fixture, through its harness only; 12.3 alone for timing.
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh --quiet 12.3   # read the gate, not up.sh
./scripts/mariadb/seed-bench.sh 12.3

# 2. The projects: #247's step 2, with tpl-now.

# 3. Wall time: #247's step 3 over the 119 labels with the arms b247, now, now_twin;
#    the two loops with template example and then rust/struct.

# 4. The experiments: copies of `git archive 2e21918` under paths of equal length, one edit
#    each as the Candidates table of the experiments states, each with its own --target-dir;
#    the rotations over (ctl250 dbl250 sys250 grw250 ctx250 raw250 eag250 fre250 ctl250_twin)
#    on 15 labels, over (ctl250 vld250 raw250 ctl250_twin) on 3, the canonical loop over
#    (ctl250 dbl250 sys250 grw250 ctl250_twin), and the start-up rotation over nop,
#    ver250 --version, ctl250 --version, table_c, rexample_c and the twins, 8 × 40 after 5.

# 5. Profiles, allocation, memory: #247's step 6, over a symbolised build of 2e21918.
samply record -s -r 20000 --iteration-count 100 --reuse-threads --unstable-presymbolicate \
  -o "$S/prof/rstruct_last_c.json.gz" -- "$S/t-prof/release/tpl" -d bench_wl001 render rust/struct --table yard_position
samply load "$S/prof/rstruct_last_c.json.gz"                                               # the flame graph

# 6. The pipeline needs all five servers; then the fixture down, and nothing left.
./scripts/mariadb/up.sh; ./scripts/mariadb/status.sh --quiet
./scripts/mariadb/down.sh; ./scripts/mariadb/status.sh --quiet    # non-zero
```

## 2026-09-23 — Peak resident memory of every render the four worked examples perform

*Sprint 20, task `#255`. Target of record: `aarch64-apple-darwin`. Server of
record: MariaDB `12.3` (`12.3.3-MariaDB-ubu2404`). This entry records; it does
not judge, per `BR-PERF-008`.*

### Outcome

**The largest peak resident set size of any `tpl render` process the four
worked examples run is 6 094 848 B (5.81 MiB)**: `rust/schema`, the whole
`freight` database in context, entry `freight`. Four times that is
24 379 392 B (23.25 MiB). This is the base the render memory limit of `#255` is
derived from; the choice of the rounding and of the limit itself is not this
entry's.

| quantity | bytes | MiB |
|---|---|---|
| base: the maximum over 3 150 render processes | 6 094 848 | 5.81 |
| 4 × base | 24 379 392 | 23.25 |
| 4 × base, rounded up to the next MiB | 25 165 824 | 24 |
| 4 × base, rounded up to the next power-of-two MiB | 33 554 432 | 32 |

### Workload

The five-command workflow of `FR-EX-004`, as `examples/_driver` performs it,
for each of the four examples (`go`, `rust`, `python`, `node`): `init`, then
per schema (`sakila`, `world`, `freight`) `cfg database add`, `cfg set …password`,
`cfg database test` and `schema tables`, then every render the example's
`plan` returns — 111 renders for `go` and `rust`, 114 for `python` and `node`,
450 in all per round. No example passes `--direct` or `--no-cache`: every
render reads through the cache the preceding `schema tables` filled, and a
`-vvv` run of the heaviest render shows no connection phase. The compile gates
were not run; the rendered trees were compared with the tracked ones instead
and were byte-identical.

### Environment

Apple M4, 10 cores, 32 GiB; macOS 26.6.2 (25G83), Darwin 25.6.0 `arm64`;
`rustc` 1.98.1 (48a229cea 2026-09-01), the release profile of `ADR-004`;
`/usr/bin/time -l`. Tree: `d89ffc4` with the uncommitted working tree of
sprint 20 (33 modified files and the untracked `src/render/bounds.rs`);
`target/release/tpl` 4 033 680 B, sha256 `4096ec45…ed09`. Only the `12.3`
server was up, seeded by `scripts/mariadb/seed-datasets.sh 12.3`, reached over
TLS at `verify-identity` as the driver configures it. Load 1.82 before and
2.68 after. Taken 2026-09-23, 18:54Z to 18:55Z.

### Protocol

Copies of `examples/` under `target/security-lab/mem/examples/`, so that no
tracked rendered file was rewritten; each copy's `plan` and `reset_output` run
through `examples/_driver.run`, with `TPL_BIN` naming a wrapper that runs
`target/release/tpl` under `/usr/bin/time -l -o` and appends the argv, exit
code, `maximum resident set size` and `peak memory footprint` of every process.
One pilot round, discarded; then 7 measured rounds, the order of the four
examples rotated by one position per round. 3 514 processes, every one exit
`0`; 3 150 of them renders, each distinct render sampled 7 times.

### Results — per template

Peak resident set size. *Median* is the median over the template's renders of
each render's median of 7; *max* is the largest single sample.

| example | template | form | processes | median | max | max at |
|---|---|---|---|---|---|---|
| go | `go/schema` | whole database | 21 | 5.09 MiB | 5.59 MiB | `freight` |
| go | `go/struct` | per table | 252 | 4.78 MiB | 5.38 MiB | `freight --table consignment` |
| go | `go/repository` | per table | 252 | 4.66 MiB | 5.17 MiB | `freight --table consignment` |
| go | `go/sql` | per table | 252 | 4.88 MiB | 5.39 MiB | `freight --table consignment` |
| rust | **`rust/schema`** | whole database | 21 | 5.22 MiB | **5.81 MiB** | **`freight`** |
| rust | `rust/struct` | per table | 252 | 4.92 MiB | 5.62 MiB | `freight --table consignment` |
| rust | `rust/repository` | per table | 252 | 4.73 MiB | 5.22 MiB | `freight --table consignment` |
| rust | `rust/sqlx` | per table | 252 | 4.95 MiB | 5.55 MiB | `freight --table consignment` |
| python | `python/core` | whole database | 21 | 4.39 MiB | 4.41 MiB | `sakila` |
| python | `python/package` | whole database | 21 | 5.19 MiB | 5.69 MiB | `freight` |
| python | `python/dataclass` | per table | 252 | 4.91 MiB | 5.58 MiB | `freight --table consignment` |
| python | `python/repository` | per table | 252 | 4.73 MiB | 5.22 MiB | `freight --table consignment` |
| python | `python/pymysql` | per table | 252 | 4.94 MiB | 5.55 MiB | `freight --table consignment` |
| node | `node/core` | whole database | 21 | 4.41 MiB | 4.44 MiB | `freight` |
| node | `node/index` | whole database | 21 | 5.20 MiB | 5.64 MiB | `freight` |
| node | `node/model` | per table | 252 | 4.88 MiB | 5.56 MiB | `freight --table consignment` |
| node | `node/repository` | per table | 252 | 4.75 MiB | 5.22 MiB | `freight --table consignment` |
| node | `node/mysql2` | per table | 252 | 4.95 MiB | 5.59 MiB | `freight --table consignment` |

The heaviest render, `rust/schema` over `freight`, 7 samples: 5 996 544 to
6 094 848 B, median 6 012 928 B (5.73 MiB). Every render's median lies between
4.39 and 5.73 MiB; the median over all 3 150 is 4.83 MiB. Between samples of the
same render the figure moves by whole 16 KiB pages, up to 98 304 B.

The non-render steps, for comparison: `init` 2.81 MiB, `cfg database add`
3.44 MiB, `cfg database test` 4.34 MiB, `schema tables` on `freight` 5.64 MiB
(maxima).

### Results — the heaviest render off the examples' path

The examples never render on a cold cache. The same render, 7 interleaved
rounds of three arms in the rust example's project:

| arm | how | RSS median | RSS max | footprint median |
|---|---|---|---|---|
| `warm` | the examples' path: a cache hit | 5.75 MiB | 5.83 MiB (6 111 232 B) | 3.39 MiB |
| `cold` | `cache clean` before each render: a miss | 6.25 MiB | 6.33 MiB (6 635 520 B) | 3.30 MiB |
| `direct` | `--direct --no-cache` | 6.12 MiB | 6.23 MiB (6 537 216 B) | 3.19 MiB |

A render that reads the server peaks about 0.5 MiB higher in resident memory
and about 0.1 to 0.2 MiB lower in footprint. Four times the `cold` maximum is
26 542 080 B (25.31 MiB): above the next-MiB rounding of the base, below the
power-of-two one.

### Confounders — read this before the tables

- **Resident set size is not heap.** It counts the binary's touched text, the
  shared libraries' resident pages and every stack, and moves in 16 KiB pages.
  `peak memory footprint`, the same `/usr/bin/time -l` report's figure for the
  process's dirty and compressed memory, is 3.3 to 3.5 MiB for the heaviest
  render: the heap is at most that. A limit counted by the allocator and set to
  four times the resident base is therefore more than four times the heap the
  heaviest example render uses.
- **Heap at exit is not peak heap.** `leaks --atExit` over the heaviest render
  reports 3 668 live blocks, 1 130 KB, when the process ends; it is a lower
  bound, and a peak heap figure needs an allocator instrument this build does
  not carry.
- **One target.** On any other target the figures differ and are not compared
  with these.

### Not measured

- The series `10.11`, `11.4` and `11.8`: the examples read only `12.3`,
  per `FR-EX-007`.
- The peak heap: `dhat` is not wired and `xctrace` is not installed on this
  host.
- The compile gates, which do not run `tpl`.

### Reproduction

```sh
cargo build --release
./scripts/mariadb/up.sh 12.3; ./scripts/mariadb/status.sh 12.3
./scripts/mariadb/seed-datasets.sh 12.3
export FIXTURE_PW=tpl-root
M=target/security-lab/mem          # git-ignored scratch
rsync -a --exclude node_modules --exclude target --exclude .tpl examples/ "$M/examples/"
# $M/tpl-wrap.sh: /usr/bin/time -l -o "$tmp" target/release/tpl "$@", then append
#   the argv, the exit code and the two peak figures to $TPL_MEM_LOG.
# $M/harness.py <example dir>: load its run.py, call reset_output() and
#   run(WORKSPACE, plan, templates=TEMPLATES) — no compile gate.
export TPL_BIN="$PWD/$M/tpl-wrap.sh" TPL_MEM_LOG="$PWD/$M/logs/rounds.tsv"
for r in 1 2 3 4 5 6 7; do      # the four examples, rotated by one per round
  for e in <go rust python node, rotated by r>; do
    TPL_MEM_TAG="$e:$r" python3 "$M/harness.py" "$M/examples/$e-data-layer"; done; done
python3 "$M/analyse.py" "$M/logs/rounds.tsv"
./scripts/mariadb/down.sh 12.3; ./scripts/mariadb/status.sh    # 12.3 down
```
