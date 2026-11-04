---
id: ADR-005
title: The scope of the async runtime
status: accepted
decided: 2026-11-04
last-reviewed: 2026-11-04
requirements: [FR-PROJ-025, NFR-PERF-004, NFR-PERF-005, NFR-PERF-006, NFR-PERF-007, FR-MYSQL-020, FR-MYSQL-042]
supersedes: []
superseded-by: null
---

# ADR-005 — The scope of the async runtime

## Status

Accepted, 2026-09-10.

**Amended in place on 2026-11-03**, for rmp `#315`: the SQLite provider of
`ADR-013` opens a connection outside `mariadb/` and bounds its statements with a
thread. The record then stated that the runtime was MariaDB's alone and where a
deadline may use the runtime's timer, and quoted the root coordination document
in the English it is now written in. The scope did not change, so `decided` is
unchanged; `last-reviewed` carries the amendment, and the claims re-read that
day are those of the *Sources* rows dated 2026-11-03.

**Amended in place on 2026-11-04**, for rmp `#327`, by the user's decision of
that day, relayed by the session coordinator: the `mysql` provider reads through
the asynchronous driver of `ADR-014`, in a module of its own, `mysql/`, and the
runtime's boundary now covers both modules that open a connection to a server,
`mariadb/` and `mysql/`. The decision — a synchronous process, with a lazy
current-thread runtime confined to the module that opens the connection to a
server — is unchanged; its parameter, the set of such modules, moved from one to
two. Under the register's rule that is an edit in place with a new `decided`
date, not a supersession. The claims re-read that day are those of the
*Sources* rows dated 2026-11-04.

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
need, in the words "No heavy static initialisation" and "Lazy initialisation by
default" — a rule it states as a demand on design and architecture rather than
a preference, because the tool is "invoked repeatedly and often inside loops and
build pipelines".

**A second provider reads without a runtime.** `ADR-013` selects `rusqlite`,
which is synchronous, for a `sqlite` entry, and `FR-LITE-009` bounds every
statement on that connection by the catalogue-query deadline. Its reader,
`sqlite/`, opens the one connection `NFR-PERF-004` allows for such an entry,
outside `mariadb/`, so where the runtime lives and where a deadline may use its
timer must be stated for a reader that has none.

**A third provider reads through the asynchronous driver.** `ADR-014` has a
`mysql` entry read through the driver `ADR-003` pins. `FR-MYSQL-020` connects
such an entry exactly as a `mariadb` entry, under the deadlines of
`FR-CONF-005`, and `FR-MYSQL-042` holds it to the one connection of
`NFR-PERF-004`. Its reader lives in a module of its own, `mysql/`, because each
engine is a provider module of its own, so the runtime is needed in a second
module that opens a connection to a server.

## Decision

**The process is synchronous.** Nothing outside the two modules that open a
connection to a server, `mariadb/` and `mysql/`, is asynchronous, and no
signature elsewhere in the crate returns a future.

**The runtime is built lazily inside each of those modules, with `block_on` at
its boundary.** It is a current-thread runtime, and it comes into existence only
when a command actually reaches one of them: `mariadb/` for a `mariadb` entry,
`mysql/` for a `mysql` entry. The two halves compose: for either entry, the
module that needs the runtime is also the module that opens the one connection
`NFR-PERF-004` allows, so the connection and the runtime have the same owner.
An invocation opens at most that one connection, so it reaches at most one of
the two modules and builds at most one runtime.

**The runtime is the server providers' alone.** A command that reads a
`sqlite` entry builds no runtime: `rusqlite` blocks on the calling thread, and
`sqlite/` is synchronous like the rest of the process. Its catalogue-query deadline is
enforced by a watchdog thread in `sqlite/` holding `rusqlite`'s
`InterruptHandle`, with the library's `busy_timeout` set to what remains of the
bound (`OD-36`); neither needs a runtime timer.

**The boundary is the unit of verification.** Because no runtime is built for a
command that reaches neither `mariadb/` nor `mysql/`, `NFR-PERF-005` is
satisfied by observation rather than by argument: there is no socket, no
configuration read and no discovery to observe, and `NFR-PERF-007` can be met
without reading the source.

## Alternatives rejected

- **`#[tokio::main(flavor = "current_thread")]` over the entrypoint.** The
  obvious shape, and the one the driver's own examples suggest. Refused twice
  over: it starts a runtime for `tpl --version` too, which is startup work the
  invoked command did not need; and it makes `NFR-PERF-005` harder to
  demonstrate, because the absence of runtime setup then has to be **argued**
  from the source rather than observed from outside — which is the one thing
  `NFR-PERF-007` forbids.

- **An asynchronous process throughout**, propagating `async` out of the
  server providers' modules into the library's own signatures. Refused because it buys nothing this
  process can spend: there is at most one connection, no concurrent work to
  overlap, and the cost is paid by every caller of every function in the
  library, including the paths that never touch a server.

- **A multi-thread runtime flavour.** Refused on the same ground and on the root
  coordination document's rule against speculative parallelism: concurrency
  enters only with a measured benefit over representative load, and with one
  connection and one query sequence there is no work to distribute. A worker
  pool started for an ephemeral process is startup cost with no counterpart.

- **The `mysql` reader inside `mariadb/`**, which would have kept the runtime in
  one module and this record's parameter unchanged. Refused by the user on
  2026-11-04, on the ground that each engine is a provider module of its own, as
  `sqlite/` already is.

## Consequences

**`block_on` at each module boundary is a synchronisation point, and those two
are the only ones.** Everything above `mariadb/` and `mysql/` reads as ordinary
synchronous Rust, and a reader auditing startup cost has two places to look, of
the same shape.

**The runtime's timers stay inside the boundary.** The phase deadlines of
`FR-CONF-005` that a `mariadb` or a `mysql` read crosses — DNS resolution, TCP
connect, the TLS handshake and the catalogue query — are enforced with the
runtime's own timers, which exist only where the runtime is. The phases that run
outside the two modules — `password_command`, the render, and every statement
on a SQLite connection — are bounded by threads of the crate's own and build no
runtime. `docs/spec-technical/open-decisions.md` records the mechanisms, in
entry `OD-12` and, for a `sqlite` entry, `OD-36`; for a `mysql` read it records
none yet, and that record is owed by its owner. Each is subordinate to this
record on the question of *where*: a deadline mechanism that used the runtime's
timer outside the two modules, or built a runtime to obtain one, would
contradict this record and would be the defect. A thread outside them that keeps
time without the runtime, as the watchdog in `sqlite/` does, does not.

**The runtime's cost is not paid by the commands that cannot use it.** This is
what `NFR-PERF-005` asks for, and it is also what puts the reference figures
`NFR-PERF-014` carries for `tpl --version` and `tpl --help` within reach, since
neither command can reach either module at all.

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
| No startup work beyond what the invoked command needs, in the words "No heavy static initialisation" and "Lazy initialisation by default"; no speculative parallelism without a measured benefit; performance is a demand on "design and architecture", for a tool "invoked repeatedly and often inside loops and build pipelines" | `CLAUDE.md`, *Performance and Efficiency* and its *Implementation rules*, at `6203216` | 2026-11-03 |
| `rusqlite` is synchronous, and a command that reads a `sqlite` entry builds no runtime | `ADR-013`, *Consequences* | 2026-11-03 |
| Every statement on a SQLite connection runs under the catalogue-query deadline | `specification/sqlite-contract.md`, `FR-LITE-009`, at `5805b5d` | 2026-11-03 |
| The `password_command` and render phases are bounded by threads of the crate's own; the statements on a SQLite connection by a watchdog thread in `sqlite/` holding the interrupt handle, with the busy timeout set to the remaining bound, and no runtime timer | `docs/spec-technical/open-decisions.md`, `OD-12` and `OD-36`, at `ad653f5` | 2026-11-03 |
| A `mysql` entry reads through the asynchronous driver `ADR-003` pins | `ADR-014`, *Decision* | 2026-11-04 |
| A `mysql` entry is connected exactly as a `mariadb` entry, under the deadlines of `FR-CONF-005`, and opens at most one connection per invocation | `specification/mysql-contract.md`, `FR-MYSQL-020` and `FR-MYSQL-042`, at `d4eaf76` | 2026-11-04 |
| The reader of a `mysql` entry lives in `mysql/`, a module of its own, and not inside `mariadb/`, because each engine is a provider module of its own | The user's decision of 2026-11-04, relayed by the session coordinator for rmp `#327` | 2026-11-04 |
| `docs/spec-technical/` records no deadline mechanism, and no module, for a `mysql` read | `rg` for the `mysql` provider over `docs/spec-technical/`, at `d4eaf76` | 2026-11-04 |
