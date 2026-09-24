---
id: ADR-005
title: The scope of the async runtime
status: accepted
decided: 2026-09-10
last-reviewed: 2026-09-22
requirements: [FR-PROJ-025, NFR-PERF-004, NFR-PERF-005, NFR-PERF-006, NFR-PERF-007]
supersedes: []
superseded-by: null
---

# ADR-005 — The scope of the async runtime

## Status

Accepted, 2026-09-10.

## Context

`ADR-003` pins an asynchronous driver. That settles what `tpl` talks to the
server with; it does not settle how much of `tpl` becomes asynchronous as a
result, and the two are separable. The question is architectural and its
alternatives must survive the choice, which is what admits this record under
rule R4's second limb.

**`NFR-PERF-005` is the binding constraint.** Every command named by
`FR-PROJ-025` — `tpl init`, every form of `help`, every form of `version` —
SHALL perform no project discovery, read no configuration file, and open no
connection. `NFR-PERF-007` requires each of those to be verified by an
observation made **outside** the process — no `stat` of an ancestor, no open of
`.tpl/.cfg`, no socket — and forbids verifying it by reading the source.
`NFR-PERF-006` widens the no-connection half to every `template` and `cfg`
subcommand except `database test`, and to any `tpl render` given `--context`.

Two further forces point the same way. `NFR-PERF-004` allows at most one
connection per invocation, so there is nothing for a runtime to multiplex. And
the root coordination document forbids startup work the invoked command does not
need, in the words "sem inicialização estática pesada" and "inicialização
preguiçosa por defeito" — a rule it states as a first-order requirement rather
than a preference, because the tool is invoked repeatedly inside build
pipelines.

## Decision

**The process is synchronous.** Nothing outside `mariadb/` is asynchronous, and
no signature elsewhere in the crate returns a future.

**The runtime is built lazily inside `mariadb/`, with `block_on` at that
boundary.** It is a current-thread runtime, and it comes into existence only
when a command actually reaches the module that opens a connection. The two
halves compose: the module that needs the runtime is also the module
`NFR-PERF-004` constrains, so at most one connection and exactly one runtime
have the same owner.

**The boundary is the unit of verification.** Because no runtime is built for a
command that does not reach `mariadb/`, `NFR-PERF-005` is satisfied by
observation rather than by argument: there is no socket, no configuration read
and no discovery to observe, and `NFR-PERF-007` can be met without reading the
source.

## Alternatives rejected

- **`#[tokio::main(flavor = "current_thread")]` over the entrypoint.** The
  obvious shape, and the one the driver's own examples suggest. Refused twice
  over: it starts a runtime for `tpl --version` too, which is startup work the
  invoked command did not need; and it makes `NFR-PERF-005` harder to
  demonstrate, because the absence of runtime setup then has to be **argued**
  from the source rather than observed from outside — which is the one thing
  `NFR-PERF-007` forbids.

- **An asynchronous process throughout**, propagating `async` out of `mariadb/`
  into the library's own signatures. Refused because it buys nothing this
  process can spend: there is at most one connection, no concurrent work to
  overlap, and the cost is paid by every caller of every function in the
  library, including the paths that never touch a server.

- **A multi-thread runtime flavour.** Refused on the same ground and on the root
  coordination document's rule against speculative parallelism: concurrency
  enters only with a measured benefit over representative load, and with one
  connection and one query sequence there is no work to distribute. A worker
  pool started for an ephemeral process is startup cost with no counterpart.

## Consequences

**`block_on` at the module boundary is a synchronisation point, and it is the
only one.** Everything above `mariadb/` reads as ordinary synchronous Rust, and
a reader auditing startup cost has one place to look.

**Deadline enforcement lives inside the boundary.** The six phase deadlines of
`FR-CONF-005` are enforced with the runtime's own timers, which is only
available where the runtime is — see `docs/spec-technical/open-decisions.md`
entry `OD-12`, which records the mechanism and the feature it requires. That
entry is subordinate to this one on the question of *where*: a deadline
mechanism that needed a timer outside `mariadb/` would contradict this record
and would be the defect.

**The runtime's cost is not paid by the commands that cannot use it.** This is
what `NFR-PERF-005` asks for, and it is also what puts the reference figures
`NFR-PERF-014` carries for `tpl --version` and `tpl --help` within reach, since
neither command can reach the module at all.

**The driver pin and the runtime scope move independently.** A change to
`ADR-003` does not by itself change this record, and a change here does not
reopen that one. What couples them is only that an asynchronous driver requires
*some* runtime; where it lives is this record's subject.

**Under R3, the scope lives here alone.**
`docs/spec-technical/architecture.md` and
`docs/spec-technical/technology-stack.md` cite `ADR-005` rather than restating
it, and `docs/spec-technical/open-decisions.md` entry `OD-11` reduces to a
citation of this record.

## Sources

| Claim | Source | Consulted |
|---|---|---|
| The four commands that perform no discovery, read no configuration file and open no connection | `specification/performance-requirements.md`, `NFR-PERF-005`; `specification/project-and-discovery.md`, `FR-PROJ-025` | 2026-09-11 |
| Each such requirement is verified by an observation made outside the process and SHALL NOT be verified by reading the source | `specification/performance-requirements.md`, `NFR-PERF-007` | 2026-09-11 |
| One invocation opens at most one connection | `specification/performance-requirements.md`, `NFR-PERF-004` | 2026-09-11 |
| No startup work beyond what the invoked command needs; lazy initialisation by default; no speculative parallelism without a measured benefit | `CLAUDE.md`, *Desempenho e Eficiência* | 2026-09-11 |
