---
id: ADR-004
title: The release profile and the panic path
status: accepted
decided: 2026-09-11
last-reviewed: 2026-09-22
requirements: [FR-ERR-001, FR-ERR-008, FR-ERR-030, FR-ERR-031, FR-ERR-032, FR-ERR-033, FR-ERR-034, FR-GLOB-018, FR-RND-034, BR-ERR-001]
supersedes: []
superseded-by: null
---

# ADR-004 — The release profile and the panic path

## Status

Accepted, 2026-09-11.

## Context

**`FR-ERR-030` delegates the mechanism to this register.** As amended in the
ninth edition it makes `70` the outcome of exactly two conditions — a panic in
the process, and a detected invariant violation — and requires that WHEN a panic
occurs, the system write the message `FR-ERR-032` requires and terminate with
exit `70`. It names no mechanism, and its own text says why: "Which mechanism
produces the outcome is an architecture decision, and this corpus names none,
per the boundary the README draws." `specification/README.md` states the same
delegation twice, in the ninth edition's own account of the amendment and in
*Still out of scope*. **This record is that mechanism**, and it is admitted
under rule R4's first limb.

The forces are four, and three of them are contractual:

- `FR-ERR-001` makes the exit-code table contract, so `70` is a code a caller
  may branch on and the binary must be able to return it.
- `FR-ERR-032` requires a `70` to carry the four labelled lines of
  `FR-ERR-008`, with a `hint` saying the condition is a defect in `tpl` and is
  not correctable by the caller. `FR-ERR-033` requires those four lines on
  stderr and stdout **empty** on every error path.
- `FR-ERR-034`'s `70` row obliges the `cause` line to name the invariant that
  was violated, or that a panic occurred, and in either case **where**.
- `FR-GLOB-018` closes six categories of content out of every diagnostic
  stream at every verbosity level.

The fourth force is the profile the root coordination document states, which
aborts on panic. `DIV-045` recorded the apparent contradiction between that
setting and the requirement as it then stood, and is **discharged**: the ninth
edition corrected the entry's premise — an aborting profile ends the process
abnormally **by default**, which is not the whole of what the setting admits —
and amended `FR-ERR-030` to state the outcome rather than the mechanism. Nothing
is owed to the root coordination document under that entry.

## Decision

**The release profile keeps its five settings**: `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`.

**The process installs a panic hook.** The hook writes the four labelled lines
of `FR-ERR-008` to stderr and terminates the process with exit `70` through
`std::process::exit`. This is available under the profile as it stands: the hook
"is invoked when a thread panics, but before the panic runtime is invoked. As
such, the hook will run with both the aborting and unwinding runtimes"
(Rust standard library, verified 2026-09-11). The aborting runtime is never
reached.

**The `cause` line carries the panic's location and not its payload.**
`PanicHookInfo::location` returns "information about the location from which the
panic originated", which is what `FR-ERR-034`'s `70` row obliges the line to
name. The payload is deliberately withheld: it is arbitrary text composed at the
panic site, on a path no reviewer reads before it runs, and keeping it out of
the stream is the structural way to keep `FR-GLOB-018` true there rather than
the vigilant way.

**Both producing conditions of `70` exist in the distributed binary**, so no
code of `FR-ERR-001` is unreachable and the observable behaviour `FR-ERR-030`
and `FR-ERR-032` require is present in the only artefact a caller ever runs.

## Alternatives rejected

- **`panic = "unwind"` with `catch_unwind` at the top level.** It is the
  literal reading of `FR-ERR-030` as it stood before the ninth edition and needs
  no hook at all. Refused for three reasons that compose:

  - **The trade cannot be evaluated.** No recorded figure separates the two
    profiles. `BENCHMARKS.md` states one release profile, identical in all four
    crates it measured, with `panic = "abort"` among its settings, so every
    figure in this repository was taken under the aborting profile. Changing the
    profile to buy a wording that a hook already satisfies would be a trade made
    blind in both directions.
  - **It costs the comparability of every recorded figure.**
    `NFR-PERF-012` makes a measurement meaningful only against one stated
    target, and `BR-PERF-008` keeps a figure as evidence for a reader rather
    than a verdict on a change. A profile change makes every existing figure
    unreadable against every later one, so all four targets of `NFR-PERF-018`
    would have to be re-measured before a later number could be read against an
    earlier one — to obtain a behaviour already obtainable.
  - **It buys nothing observable.** `catch_unwind` and the hook produce the same
    four lines and the same exit status. What unwinding adds is the running of
    destructors on the way out, and the process is exiting.

- **Dropping the panic condition from `70` and stating the resulting limit.**
  This is the form `DIV-045` anticipated, and `FR-ERR-030` rejects it in its own
  text: the limit is not real, because the outcome is obtainable under the
  profile as it stands, and a corpus that recorded a limit it does not have
  would send a caller branching away from a code the binary does return. It is
  recorded here because it is the option an implementer meeting the aborting
  profile reaches for first.

- **Writing the panic payload into the `cause` line.** `FR-ERR-034` requires the
  location and does not require the payload, and the payload is the one part of
  a panic that no requirement constrains the content of. Refused on
  `FR-GLOB-018`. The cost is stated rather than hidden and is accepted below.

- **Leaving the profile and the requirement both standing unchanged**, which
  `DIV-045` refused outright. The profile is kept *and* the amendment was
  requested; the corpus moved first, which is the order an implementation choice
  may never invert when a contractual code is at stake.

## Consequences

**A maintainer reading a bug report sees the panic site and not the message.**
That is the accepted cost of withholding the payload. Nothing the **caller** can
act on is lost: `FR-ERR-032`'s `hint` already tells them the condition is a
defect in `tpl` and not theirs to correct.

**No destructor runs on the way out.** `std::process::exit` "will never return
and will immediately terminate the current process", passing the code "through
to the underlying OS", and "no destructors on the current stack or any other
thread's stack will be run" (Rust standard library, verified 2026-09-11). The
one destructor whose absence a caller could detect is the flush of a buffered
stdout — and `FR-ERR-033` requires stdout to be **empty** on an error path,
while `FR-RND-034` already admits at most one incomplete result on a failed
render. Nothing a caller can observe depends on a destructor here.

**`strip = true` and the location are compatible, and this composition is
reasoned rather than sourced.** `core::panic::Location` is `&'static` data the
compiler emits rather than debug information, so stripping leaves the file, line
and column available to the hook. Neither document states that sentence; it is
derived from the two definitions and is **verified when the first `70` is
exercised** through the trigger of `FR-ERR-031`. It is marked because a reader
must not take it for a cited fact.

**The cost of unwinding is unmeasured, and this record asserts no figure for
it.** The first rejection above is the reason: no figure in this repository
separates the two profiles. A later measurement that separates them does not
reopen this decision on its own — the hook satisfies the requirement either way
— but it is what would make the trade evaluable for the first time.

**`70`'s other producing condition is independent of this record.**
`FR-ERR-031`'s in-process trigger exercises the detected invariant violation, so
`BR-ERR-001`'s exception for `70` — the one code with no integration test —
rests on that trigger and not on the panic path.

**Nothing is owed to the functional corpus.** `DIV-045` is discharged, the code
table of `FR-ERR-001` is unchanged, and this record supplies no requirement: it
names one mechanism that produces an outcome the corpus already fixes, which is
what R2 permits and no more.

**Under R3, the profile lives here alone.** The root coordination document and
`docs/spec-technical/operations.md` cite `ADR-004` for the five settings rather
than restating them, and `docs/spec-technical/open-decisions.md` entry `OD-28`
reduces to a citation of this record.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| "The panic hook is invoked when a thread panics, but before the panic runtime is invoked. As such, the hook will run with both the aborting and unwinding runtimes" | Rust standard library documentation, `std::panic::set_hook` | 2026-09-11 |
| `PanicHookInfo::location` "returns information about the location from which the panic originated, if available"; its payload is arbitrary and may be any type under `panic_any` | Rust standard library documentation, `std::panic::PanicHookInfo` | 2026-09-11 |
| `std::process::exit` "will never return and will immediately terminate the current process"; the code is "passed through to the underlying OS"; "no destructors on the current stack or any other thread's stack will be run" | Rust standard library documentation, `std::process::exit` | 2026-09-11 |
| `70` has exactly two producing conditions; a panic obliges the message and exit `70`; the mechanism is an architecture decision the corpus does not name | `specification/errors-and-exit-codes.md`, `FR-ERR-030` | 2026-09-11 |
| The `cause` line of a `70` must name the invariant violated, or that a panic occurred, and in either case where | `specification/errors-and-exit-codes.md`, `FR-ERR-034` | 2026-09-11 |
| `DIV-045` is discharged; its premise about an aborting profile was too strong, and nothing is owed to the root coordination document | `specification/upstream-divergences.md`, `DIV-045` | 2026-09-11 |
| Every measured figure in this repository was taken under one release profile carrying `panic = "abort"` | `BENCHMARKS.md`, "2026-09-10 — MariaDB driver selection", Environment | 2026-09-11 |
| `core::panic::Location` survives `strip = true` because it is compiler-emitted `'static` data rather than debug information | **Reasoned, not sourced.** Verified when the first `70` is exercised through `FR-ERR-031` | — |
