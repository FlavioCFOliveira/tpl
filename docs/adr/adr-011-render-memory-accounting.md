---
id: ADR-011
title: The heap count behind the render memory limit
status: accepted
decided: 2026-09-23
last-reviewed: 2026-09-24
requirements: [FR-RND-039, FR-RND-038, FR-RND-033, FR-RND-034, FR-CONF-045]
supersedes: []
superseded-by: null
---

# ADR-011 — The heap count behind the render memory limit

## Status

Accepted, 2026-09-23.

**The mechanism was chosen by the user for rmp `#255`**: the count comes from a
counting global allocator provided by a crate, and the render's existing
deadline thread polls it and ends the render with `65` when it crosses the
limit. **The two parts this record proposed — the crate (Decision, point 1)
and the hard-limit policy (Decision, point 3) — were confirmed by the user on
2026-09-23**, as relayed by the session coordinator. This record does not
re-open either.

## Context

`FR-RND-039` bounds the heap the process holds while a render runs, "as counted
by its allocator", observed periodically, and ends the render with `65` when
the observed count crosses the value `FR-CONF-045` resolves for
`core.render_memory_limit`. `FR-RND-036`, as amended within the forty-second
edition, **delegates the mechanism out of the corpus**: "which allocator counts
is a matter for the technical specification and the architecture decision
records, not for this corpus". The record is also admitted under rule R4's
second limb: it adds a dependency and replaces the process-wide allocator, and
the alternatives below must survive the choice.

Four forces fix the shape of any answer.

- **`unsafe` is forbidden** (`#![forbid(unsafe_code)]`). Implementing
  `GlobalAlloc` requires an `unsafe impl`, so the counting wrapper must come
  from a crate. Declaring `#[global_allocator] static …` of a type that
  implements it is safe code, and the `unsafe_code` lint does not fire on it
  (verified: Sources).
- **An allocation failure cannot be turned into `65` on stable Rust.** The hook
  that would intercept it, `std::alloc::set_alloc_error_hook`, is nightly-only;
  the default handler prints a line to stderr and aborts. The documentation
  adds that the default "may be changed to panicking in future versions". A
  refused allocation therefore ends the process by `SIGABRT`, which is the
  residual `FR-RND-039` already states.
- **Only one thread allocates while a template runs.** `FR-RND-040` shuts down
  every driver runtime and closes the connection before the render starts, so
  the render thread and the deadline thread of `OD-12`
  (`docs/spec-technical/open-decisions.md`) are the only threads alive. Counter
  contention is not a factor.
- **The dependency budget is strict**, and `ADR-007` takes the MSRV from the
  highest floor any shipped dependency declares.

The measurement that made the limit necessary is recorded in
`SECURITY-AUDIT.md`, *Remediation*, row H-1: string doubling through a
`namespace` reached 16 981 MB of peak resident memory inside the fuel budget
and was stopped only by the 30 s render deadline.

## Decision

1. **The count comes from the crate `cap`, version 0.1.2, with no features
   enabled.** `tpl` installs `cap::Cap<std::alloc::System>` as its
   `#[global_allocator]`. The count `FR-RND-039` bounds is the value
   `Cap::allocated()` returns: the sum of `Layout::size()` over the live
   allocations of the process.

   `cap` is chosen because it is the only candidate that meets every criterion
   at once. It wraps any `GlobalAlloc`, including `System`; it keeps one
   atomic counter, adjusted by a single atomic read-modify-write per `alloc`,
   `alloc_zeroed`, `realloc` and `dealloc`, and `allocated()` is three atomic
   loads, callable from any thread; it offers an optional hard limit,
   whose use point 3 settles; it has **no dependencies** and declares no
   `rust-version`, so it adds one crate to the graph and does not move the
   floor of `ADR-007`; it is licensed `MIT OR Apache-2.0`; and it compiles
   under `#![forbid(unsafe_code)]` and clippy `-D warnings` in an edition-2024
   crate, type-checks on the four targets of `ADR-008`, and compiles on Rust
   1.87.0 and 1.98.1, which bracket the 1.94.0 floor. Its weakness is
   maintenance: the last release is of 2023-03-26 and the repository marks the
   crate `passively-maintained`. The whole crate is 478 lines, tests included,
   with no dependency and no platform-specific path, which is what makes that
   weakness acceptable; Consequences states what it costs.

   The `stats` feature is refused: it adds a `fetch_add`, a `fetch_max` and a
   call to `allocated()` to every allocation, and `FR-RND-039` needs none of
   what it returns. The `nightly` feature cannot be used on a stable
   toolchain.

2. **The deadline thread of `OD-12` polls the count.** It wakes at a fixed
   interval and at the render deadline, whichever comes first. At each wake it
   reads `allocated()`; a value above the resolved limit ends the render
   exactly as an expired deadline does — the cause of `FR-RND-039` written
   through the diagnostics path, then `std::process::exit` with `65`, which
   discards buffered stdout and keeps `FR-RND-034`. Reading the count
   allocates nothing. The interval is a parameter of the technical
   specification, not of this record: Consequences proposes its value.

3. **The crate's hard limit is not set.** The allocator is constructed with
   the limit `usize::MAX` and `set_limit` is never called, so `cap` refuses no
   allocation of its own accord and the only allocation failure left is the
   one the operating system produces.

## Alternatives rejected

- **A `GlobalAlloc` of this project's own.** Implementing the trait requires
  `unsafe impl GlobalAlloc` and `unsafe fn` methods. `#![forbid(unsafe_code)]`
  forbids both, and cannot be relaxed locally.

- **`setrlimit` with `RLIMIT_AS` or `RLIMIT_DATA`.** Its failure mode is an
  allocation refused by the kernel, which aborts with `134` and never produces
  `65` or a cause — the residual `FR-RND-039` accepts, not the bound it
  requires. It is also not usable where it was tried: on the development
  host (macOS, Darwin 25.6) lowering either limit fails with `EINVAL`. It would bound address space or the data segment, not the heap
  count `FR-RND-039` names.

- **Render fuel alone.** `FR-RND-036` bounds evaluation steps, not bytes. The
  re-verification recorded in `SECURITY-AUDIT.md` reached 16 981 MB within a
  few dozen steps. `FR-RND-036` itself states that fuel does not bound memory.

- **Polling the process's resident set through the operating system.** It
  needs one mechanism per operating system, and every one of them is outside
  `std`. It counts what `FR-RND-039` does not name — code, stacks, and pages
  the allocator has freed but not returned — and it lags behind allocation,
  because a page is counted only once it is touched. It would replace a
  portable atomic load with platform code for a less exact quantity.

- **The crate's hard limit, alone or as a backstop.** Alone, it fails the
  requirement: a refused allocation reaches `handle_alloc_error` and aborts
  with `134`, with no cause (verified: Sources). As a backstop above the
  polled limit it was weighed and refused, for three reasons.
  - **Any value close enough to protect pre-empts the poll in the shape that
    motivated the limit.** A doubling string crosses the polled limit with one
    allocation of about the size it already holds, and copying into a new
    buffer holds both. A hard limit within a small multiple of the polled one
    is reached by that allocation before the poll observes the crossing, and
    turns the `65` with a cause into `134` without one. A value far enough
    above never to do this protects little the operating system does not.
  - **The hard-limit path carries a race that is reported and unfixed.**
    Upstream issue `#2` describes a failing over-allocation that wraps the
    remaining-bytes counter, so that a concurrent over-allocation succeeds.
    With the limit at `usize::MAX` the counter cannot wrap, and the path is
    never reached.
  - **`FR-RND-039` places a hard ceiling outside `tpl`**: "A caller who needs a
    hard ceiling on the machine's memory must impose it outside `tpl`."

- **A check of the heap peak at the end of the render**, through `cap`'s
  `stats` feature, so that a completed render whose peak exceeded
  `render_memory_limit` always ends with `65`. Rejected by the user on
  2026-09-24 (rmp `#297`). The case it targets is real: in rmp `#267`, a 32 MiB
  render under a 16 MiB limit completed in some runs, idle and under load,
  because the limit is enforced only at each poll (point 2, at the interval
  `docs/spec-technical/architecture.md` fixes); the counts are recorded in
  `ADR-012`, *Consequences*. It is refused for four reasons.
  - **The limit guards against runaway renders**, and sampling stops those.
  - **`FR-RND-039` acts on the observed count and allows the escape**: the
    process "MAY hold more than the limit between one observation and the
    next".
  - **It needs the `stats` feature this record refuses** (point 1), and with it
    an atomic update on every allocation.
  - **A render that never finishes still depends on sampling**, so the check
    would not replace the poll.

- **Other crates.** Each was read at its latest release on 2026-09-23.
  - `stats_alloc` 0.1.10 (MIT, 2022-03-30): no limit; six `SeqCst` counters
    with two read-modify-writes per allocation; the held count is derived by
    subtracting two counters read separately, so it can tear.
  - `peak_alloc` 0.3.0 (MIT, 2025-06-23): no limit; two read-modify-writes per
    allocation (`fetch_add` and `fetch_max`), for a peak `FR-RND-039` does not
    use; hard-wired to `System`. The most recent of the counters, and the
    runner-up: it would serve if `cap` became unusable.
  - `tracking-allocator` 0.4.0 (MPL-2.0, 2022-07-01): a hook per allocation
    into a tracker registered at runtime, with per-thread allocation groups; a
    heavier mechanism than a counter, under a weak-copyleft licence.
  - `alloc_counter` 0.0.4 (2019-11-29): per-thread counters that the deadline
    thread cannot read, and a dependency on a pre-release `pin-utils`.
  - `alloc-track` 0.4.0 (2025-11-14): per-backtrace and per-thread tracking;
    depends on `dashmap` and `lazy_static`, and on `procfs` on Linux.
  - `tikv-jemallocator` with `tikv-jemalloc-ctl`: replaces the allocator
    rather than wrapping `System`, builds a C library for each of the four
    targets, and reads statistics through an epoch refresh. Far beyond the
    dependency budget for one counter.

## Consequences

- **A test of the memory limit is a measurement test.** Its verdict depends on
  sampling, so it is kept off the release path under `ADR-012`, Decision
  point 10, and runs on demand.

- **Every allocation of the process pays one atomic read-modify-write, and
  every deallocation one more**, for the whole invocation, not only while a
  template runs. This is a cost to measure, not an estimate: after
  implementation, the render wall time of the four worked examples under
  `examples/` (`go`, `node`, `python` and `rust` data layers) is measured
  before and after, on
  the target `NFR-PERF-012` requires a baseline to name, under the protocol of
  `BENCHMARKS.md`. The reading is informative and gates nothing, per
  `BR-PERF-008`.

- **The count is process-wide.** Heap held before the render starts — the
  model, the context document, the configuration — counts toward the limit;
  `FR-CONF-045` sets the lower bound at 8 MiB for exactly this reason. The
  count is of requested bytes, not of allocator overhead or resident pages, so
  it will not equal a resident-set reading. For `FR-RND-038`, a render
  abandoned under `FR-CACHE-039` has ended and dropped its values before the
  following render is observed, so the following render is not charged for
  it; the technical specification must preserve that order.

- **Poll interval: 10 ms is proposed for the technical specification.** The
  overshoot `FR-RND-039` admits is what the render allocates within one
  interval. Reading an atomic costs nothing measurable, and a legitimate render
  of the worked examples ends within a few intervals, so the poll costs a
  legitimate render at most a few idle wake-ups. The measured growth in H-1
  averaged 566 MB/s (16 981 MB in 30 s); 10 ms is about 5.7 MB of that
  average. That average understates the peak rate of an exponential curve,
  and a shorter interval does not help where it matters most: for a doubling
  value the overshoot is set by the size of the one allocation that crosses
  the limit, not by the interval. The realised overshoot is to be observed
  after implementation by re-running the H-1 template under the default limit
  and recording the exit status and peak resident memory.

- **The abort residual stays exactly as `FR-RND-039` states it.** An
  allocation the operating system refuses aborts, with the one line the
  standard library writes to stderr and no cause. Were the standard library to
  switch the default to a panic, as its documentation allows, that failure
  would reach the panic path of `ADR-004` and end with `70`; that change
  obliges a review of this record and of `FR-RND-039`.

- **The process has one global allocator, and it is this one.** `ADR-006` and
  `docs/spec-technical/operations.md` leave in-tree heap profiling with `dhat`
  undecided, and that profiler also requires installing its allocator as the
  program's global allocator. Both cannot be installed at once. The choice
  between composing them, gating one out, or profiling out of tree is not
  taken here.

- **Maintenance risk is carried, not removed.** `cap` is passively maintained.
  A defect found in it is resolved by the route `ADR-010` recorded for the
  driver, or by moving to `peak_alloc`, the runner-up. Either is an edit to
  this record.

- **The dependency adds no transitive crate.** `cargo tree -e normal,build`
  gains exactly one node. `cargo audit` has no advisory for `cap`.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| `cap` 0.1.2: latest and maximum stable release, published 2023-03-26, licence `MIT OR Apache-2.0`, no `rust-version` declared | crates.io API, `/api/v1/crates/cap` | 2026-09-23 |
| `cap` 0.1.2 has no dependencies; features `nightly` and `stats`; maintenance badge `passively-maintained` | `cap-0.1.2/Cargo.toml` from the published `.crate` | 2026-09-23 |
| `Cap` keeps one `remaining` counter adjusted by one atomic RMW per `alloc`, `alloc_zeroed`, `realloc` and `dealloc`; `allocated()` is `limit − remaining` from three `SeqCst` loads, retried while the two limit reads differ; `stats` adds `fetch_add`, `fetch_max` and a call to `allocated()` per allocation; a refused request returns null | `cap-0.1.2/src/lib.rs` from the published `.crate`, 478 lines, read in full | 2026-09-23 |
| Hard-limit race: a failing over-allocation wraps `remaining`, letting a concurrent one succeed; issue open | GitHub `alecmocatta/cap`, issue `#2`; last commit on the default branch 2023-03-26 | 2026-09-23 |
| No RustSec advisory for `cap` | GitHub `rustsec/advisory-db`, path `crates/cap` absent | 2026-09-23 |
| `#[global_allocator] static …: Cap<System>` compiles under `#![forbid(unsafe_code)]` in an edition-2024 crate and passes clippy `-D warnings`; `allocated()` reads the same count from a second thread; one crate added to `cargo tree -e normal,build`; `cargo check --release` passes for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-musl` and `x86_64-unknown-linux-musl`; compiles with Rust 1.87.0 and 1.98.1; linked and run on `aarch64-apple-darwin` only | Scratch probe crate outside the repository, `cap = "=0.1.2"` | 2026-09-23 |
| With a hard limit set, a refused allocation prints "memory allocation of N bytes failed" and exits `134` under `panic = "abort"` | The same probe, run on `aarch64-apple-darwin` | 2026-09-23 |
| `set_alloc_error_hook` is nightly-only (`alloc_error_hook`, tracking issue `#51245`); the default handler prints to stderr and aborts, and may become a panic in future versions | doc.rust-lang.org, `std::alloc::set_alloc_error_hook` and `std::alloc::handle_alloc_error` | 2026-09-23 |
| `ulimit -v` and `ulimit -d` (`RLIMIT_AS`, `RLIMIT_DATA`) fail with `EINVAL` on the development host, macOS Darwin 25.6 | Shell probe on that host | 2026-09-23 |
| That `setrlimit` is refused on every supported macOS release, and how Linux enforces either limit | unverified | — |
| `stats_alloc` 0.1.10 (MIT, 2022-03-30), `peak_alloc` 0.3.0 (MIT, 2025-06-23), `tracking-allocator` 0.4.0 (MPL-2.0, 2022-07-01), `alloc_counter` 0.0.4 (MIT OR Apache-2.0, 2019-11-29), `alloc-track` 0.4.0 (MIT OR Apache-2.0, 2025-11-14): release, licence and dependencies | crates.io API, and each crate's published `Cargo.toml` | 2026-09-23 |
| `stats_alloc`: six `SeqCst` counters, two RMWs per allocation, no limit; `peak_alloc`: `fetch_add` plus `fetch_max` per allocation, `System` only, no limit; `tracking-allocator`: runtime-registered tracker hooks and thread-local groups; `alloc_counter`: thread-local counters | Each crate's `src/lib.rs` from the published `.crate` | 2026-09-23 |
| `tikv-jemallocator` builds jemalloc from C and reads statistics through an epoch refresh | unverified | — |
| Per-allocation overhead of `cap` on the render hot path | unverified; to be measured after implementation | — |
| The rejection of the end-of-render peak check and its four reasons | The user's decision of 2026-09-24, relayed for rmp `#297` | 2026-09-24 |
| The process "MAY hold more than the limit between one observation and the next" | `specification/render-command.md`, `FR-RND-039` | 2026-09-24 |
| 16 981 MB peak resident memory in 30 s for the H-1 template, inside the fuel budget | `SECURITY-AUDIT.md`, *Remediation*, row H-1 | 2026-09-23 |
