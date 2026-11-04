# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

It is **agent coordination**: it states what the project is, where the truth about it lives, and how work is executed here. **It contains no functional requirements.**

---

> # ⛔ RULE ZERO — ONE SUBAGENT AT A TIME
>
> **YOU ARE NOT AUTHORISED TO RUN MORE THAN ONE SUBAGENT AT THE SAME TIME.**
>
> The default is **a single subagent running alongside the main conversation**. Use **every** subagent
> the objective requires, but **in series** — never in parallel.
>
> **Parallelism has one condition only: the user's express and clear authorisation**, given for the
> request at hand. Without it, parallelism is forbidden.
>
> Once authorised, parallelism is an **exception**: it holds for that request only, and the authorisation
> **is always revoked at the end of the task**, restoring the serial default at once. An authorisation
> sets no precedent for later tasks.
>
> **Before launching a subagent, verify that no other is alive.** Verify by **listing the agents** — not
> from memory of what was launched, nor from the previous agent's notification: an agent can keep
> background work of its own and **resume by itself** after notifying. If another is alive — **STOP** and wait.
>
> This rule **precedes every other rule in this file** and overrides any default heuristic that favours
> parallelism. It is developed in *One at a time — never in parallel*.

---

## Overview

`tpl` is a **command-line application written in Rust** that reads the structure (DDL / schema) of **MariaDB** and **MySQL** databases and **SQLite** database files — the provider each database entry names — and applies **MiniJinja templates** to produce text or code from that structure.

The interaction model is **inspired by `git`**: a single executable, subcommands, short aliases, and read commands with stable output, designed for composition in pipelines.

**Product scope.** `tpl` explores the database, explores the project's templates, and renders one from the other. It is **read-only** on the database, and the only disk writes it makes are those explicitly requested on the command line, plus the management of its own project folder, plus the write-ahead-log files a SQLite read may cause beside the database, which `FR-LITE-008` states. A feature that does not fit this does not belong in `tpl`.

**Primary consumer: AI agents.** The expected user is not a person at an interactive shell — it is an agent (Claude Code, Codex and equivalents) invoking the tool programmatically and reading the result. An agent has three channels to understand a CLI, and only three: the **help text**, the **exit code**, and **stdout/stderr**. This is a first-order design requirement, and the reason help, exit codes, output formats and determinism are treated as contract. **The content of that contract belongs to the specification, not to this file.**

## Sources of Truth

The project has four sources of truth, with scopes that do not overlap. Confusing them is the predictable mistake:

| Source | Answers | Owner |
|---|---|---|
| `/specification` | **What** `tpl` does — requirements, rules, use cases | subagent `specification-manager` |
| `rmp` | **When and by whom** — sprints, tasks, status, decisions | skill `roadmap-manager` |
| Knowledge Graph | **Where** — what code exists, how it fits together, and which requirement each component satisfies | skill `knowledge-authority` |
| `docs/spec-technical/` | **How** `tpl` is built — architecture, interfaces, data, security, operations, quality | subagent `technical-writer` |

A task in `rmp` implements a requirement of the specification; it does not replace it. A fact in the graph describes the code that exists; it does not legitimise code the specification did not ask for. The technical specification prescribes how to build what the functional specification asks for; where they disagree, `/specification` governs. The architecture decision records, in `docs/adr/`, add no fifth source: they ground the fourth, and none is an authority for a requirement.

**This file is not a source of functional truth.** It does not describe commands, flags, output formats, exit codes, context variables, filters, or the structure of the configuration. For any question about *what* `tpl` does, read the specification — never answer from this file, and never reintroduce here what the specification already says. A second copy stops being true without giving any sign of it.

### Where to look for each subject

The complete, current index is `specification/README.md`, and that is where to start. The subjects this file no longer describes are distributed as follows:

| Subject | File |
|---|---|
| Invocation grammar, command tree, aliases, arguments | `specification/cli-contract.md` |
| Global flags, environment variables, configuration precedence | `specification/global-flags.md` |
| Help forms, help-text structure, JSON command tree, version | `specification/help-and-version.md` |
| Database exploration | `specification/schema-commands.md` |
| Template exploration | `specification/template-commands.md` |
| Rendering | `specification/render-command.md` |
| Catalogue cache | `specification/cache-commands.md`, `specification/cache-documents.md` |
| Managing the configuration and the database entries | `specification/cfg-commands.md` |
| Project configuration file: keys, types, expansion, connection definitions | `specification/configuration-model.md` |
| The project folder, its discovery and its initialisation | `specification/project-and-discovery.md` |
| Text and JSON output, envelope, encoding, output destination | `specification/output-formats.md` |
| Exit codes, validation order, error-message format, suggestions | `specification/errors-and-exit-codes.md` |
| Cross-cutting security rules and credential handling | `specification/security.md` |
| What enters the model from the catalogue, and what stays out | `specification/catalogue-coverage.md` |
| Structure of the document that carries the model — the render context | `specification/context-document.md` |
| Filters, tests and global functions available to the template | `specification/template-environment.md` |
| Render semantics | `specification/render-semantics.md` |
| Supported MariaDB series, differences between series, the read-only promise of a `mariadb` entry | `specification/server-contract.md` |
| The SQLite provider: the library, the database file, hardening, the closed statement list, how the catalogue fills the model, the read-only promise of a `sqlite` entry, the fixture | `specification/sqlite-contract.md` |
| The MySQL provider: supported series and their rule, product and version checks, the closed statement and table lists, the read-only promise of a `mysql` entry, connection and TLS, how MySQL fills the model, its divergences from MariaDB, the fixture | `specification/mysql-contract.md` |
| Incomplete reads caused by insufficient privileges | `specification/privileges-and-completeness.md` |
| Required performance properties, reference workloads, measurement protocol | `specification/performance-requirements.md` |
| Terms, end-to-end use cases, open questions | `specification/glossary.md`, `specification/use-cases.md`, `specification/open-questions.md` |

## Non-Negotiable Rules

These rules are not weighed case by case. Each has a section that develops it; the table exists so that none is lost for being far away.

| Rule | If you are about to break it |
|---|---|
| Work happens only on an **open task**: in the `OPEN` sprint and in `DOING` with `--commit-open` | **STOP** and open it through the `roadmap-manager` skill |
| **No task is executed directly** — all work is delegated to the most suitable subagent, each piece of work to its own | **STOP** and choose the subagent |
| **RULE ZERO — one subagent at a time**, never in parallel. List the agents before every launch | **STOP** and serialise. Parallelism requires the user's **express authorisation** — the only condition — and is an exception revoked at the end of the task |
| **Synergy and convergence are sought by default**, never awaited from the user, and **never an `rmp` task**. Requested tasks with **complementary objectives** or **verifiable functional or technical proximity** are delivered as **one effort**, each kind of work done at once; the tasks themselves **stay separate** | **STOP** and regroup. Joining a task the user did not request requires the user's decision |
| Git writes **only** through the `gitflow` skill | **STOP**. Never a stray `git commit`, however trivial |
| Tasks, sprints and comments **only** through the `roadmap-manager` skill | **STOP**. Never `rmp` invoked from Bash |
| Knowledge about the code **only** through the `knowledge-authority` skill | **STOP**. Never `rmp graph …` directly |
| The **work log** is written in `rmp` comments as the work happens — on the sprint and on each task, each entry with the type that fits it | **STOP** and write the entry before going on |
| The **specification precedes implementation**, and `/specification` is written only by the `specification-manager` subagent | **STOP** and formalise the requirement first |
| `docs/spec-technical/` is written only by the `technical-writer` subagent; `docs/adr/`, only by `adr-guardian` | **STOP** and delegate. Neither is edited on one's own initiative |
| **Look for open-source prior art before designing**; never reinvent what a reference project already solves | **STOP** and search |
| **Nothing outside the repository root is read, listed or referenced** — except a reference open-source project at its public source | **STOP**. Ask the user for what is missing |
| **`unsafe` is forbidden**; `#![forbid(unsafe_code)]` stays at the top of the crate | **STOP** and solve it in safe Rust |
| **Every change carries the tests that prove it**, and the **validation tier the moment calls for** passes before the work is complete — no wider, no narrower | The work **is not complete**. Fix and repeat |
| `tpl` **never writes to the database** | **STOP**. The catalogue is read only through `INFORMATION_SCHEMA` for a `mariadb` and a `mysql` entry — for a `mysql` entry, the tables of `FR-MYSQL-015` alone — and through `sqlite_schema` and the pragma functions of `FR-LITE-015` for a `sqlite` entry |
| Templates load **at runtime**, never embedded at compile time | **STOP**. Compile-time engines are excluded |
| Validation that needs a database uses the **project fixture**, operated by its harness | **STOP** and start it through the harness. Never by hand, never mocks or external instances |
| **Nothing is started that was not explicitly requested** | **STOP**. Record the finding and ask the user how to proceed |
| **No work is delivered half-done** | The work **is not complete**. Finish it, or state explicitly what was left undone and why |
| All **documentation is written in English** — this file included — and every instruction is explicit, objective, closed and concise | **STOP** and rewrite |

## Before Starting

Run this check before any work. A "no" at any point halts the work — it does not authorise proceeding with a note.

1. **Is the requirement in the specification?** If not, or if it is ambiguous, incomplete or contradictory — **STOP**. The first task is to take it to the `specification-manager` subagent.
2. **Does an `rmp` task exist for this work?** If not — **STOP** and create it through the `roadmap-manager` skill. No work is executed without a task.
3. **Is the task in the `OPEN` sprint?** If it is in `BACKLOG`, or in a `PENDING` or `CLOSED` sprint — **STOP**. Bringing it into the open sprint is a planning action and **requires the user's confirmation**.
4. **Is the task in `DOING`, opened with `--commit-open <hash>`?** If not, open it now, with the real hash from `git rev-parse HEAD`.
5. **Is there synergy or convergence to exploit?** Check **always**, without waiting for the user to ask, which tasks — in `rmp` or requested ad hoc — have complementary objectives or verifiable functional or technical proximity to this one. Tasks the user requested are joined into one effort; a task the user did not request is proposed, never started. The check **creates no task of its own** — see *The search is not a task*.
6. **What is the shortest path?** Determine the strategy that reaches the objective with the least work and no loss of quality, counting synergies, convergences, and reference open-source projects that already solve the problem (*Open Source and Prior Art*). Then follow it.
7. **Which subagent executes each piece of the work?** Decompose the work by nature and choose per piece, evaluating the agents actually installed. With no obvious specialist, the general-purpose one — **never** direct execution.
8. **Is the scope closed?** The briefing is what the task defines — title, description, requirements, comments — and nothing else. Whatever is found outside it **is recorded; it is not executed**.
9. **During the work**, write the work log as it happens, on the task and on the sprint (*The work log*).
10. **On closing**, commit first through the `gitflow` skill, then `--commit-close <hash>` with the real hash of that commit.

## Synergy and Convergence of Effort

> **Make the effort pay off: deliver the most with the least work.**

**Seeking synergy and convergence is the default way of working.** It is permanent, and it applies to every way of working in this project. Before planning, before decomposing and before delegating, the question is the same: what work can be done **at once**, to the benefit of more than one task?

**Objectives are reached in the fewest possible tasks and iterations**, making the most of the internal resources available, so that delivery is faster and costs the user less.

**The user MUST NOT have to ask for synergy and convergence to be sought.** Catching yourself executing task by task without having looked for what joins the tasks is reason to **STOP** and look.

### The search is not a task

**Seeking synergy and convergence is NEVER recorded as a task in `rmp`.** It is a way of working, not work: it runs inside what is already open, consumes no task of its own, opens no sprint, and needs no authorisation to happen. Being about to create a task to "look for synergies" is reason to **STOP**.

What is recorded is the **result**, and only when there is one: a `DECISION` comment on each task the effort joins, naming the other tasks and the convergence that joins them. *No work outside an open task* governs the work — not the search, which precedes it.

### Synergy improves the work — never worsens it

The grouped effort delivers **better** work than task-by-task execution, and **NEVER worse**: faster **without costing quality**. Work of the same nature is aggregated where that **shortens the work without harming the delivery**; where it would harm it, it is not aggregated.

### Across tasks

**Convergence is sought in the individual objectives of each task.** Complementary objectives, functional proximity and technical proximity are all turned into **one optimised effort**.

When tasks — in `rmp` or requested ad hoc by the user — have **complementary objectives** or **verifiable functional or technical proximity**, they are joined into **one development effort**, so that one piece of work serves several. Letting close tasks follow separate paths is waste, and reason to **STOP** and regroup.

**The tasks stay separate; only the effort joins them.** Each task keeps its own identity, definition, work log and closing gate in `rmp`. No task is merged into another or replaced by a combined task. The joining exists only in development and in testing.

**Within the requested work, joining is the default** and needs no further confirmation: the tasks the user asked to execute — by naming them, or by naming the sprint that holds them — are joined whenever they converge. **A task the user did not request is never joined on one's own initiative**: it is proposed to the user, naming it and the convergence that would join it, and its work starts only on the user's decision — *Initiative*, below.

The conditions of *No work outside an open task* are not waived: every task in the effort belongs to the `OPEN` sprint and is in `DOING`, opened with `--commit-open <hash>`.

The effort's scope is the **union of the scopes of its tasks**, and that union is itself closed.

### Within a task

Within a task the demand is the same: **group the work by nature and execute each nature at once.** Write **all** the code at once — for every task in the effort — and test **all** the changes at once, instead of writing a fragment, testing it, writing the next and testing it. The alternation multiplies delegations, contexts and validations, and adds nothing.

The same holds for documentation, and for every other nature of work: it is handled **all at once**. When the scope is too large for a single pass, **blocks are identified** and each block is handled whole, at once — never piece by piece.

The validation of the *Test execution policy* runs over the complete effort, not after each fragment.

### Synergy is not parallelism

Synergy is gained through **fewer, larger delegations** — never through simultaneous ones. *One at a time — never in parallel*, below, stays intact, and nothing in this section loosens it. Making the most of the internal resources never means running subagents concurrently.

### The effort is complete only when all its tasks are

An effort that joins several tasks satisfies *Completeness* in full: no task is declared complete while the work that belongs to it is not done and validated. Each closes through its own gate, under *Closing: commit first, then the task*.

## Task Execution by Subagents

**No task is executed directly.** **All** work in this project is delegated to the **specialised subagent** best suited to the objective the task must reach. Catching yourself executing directly is reason to **STOP** and delegate.

### Choosing the subagent

The set of available subagents **is evaluated at the moment**, on the machine where the work takes place. There is no fixed list here, and there **NEVER** will be: installed agents change, and a list written in this file would go stale without giving any sign of it. Before delegating, **ALWAYS** check which agents are actually available and choose the most suitable one for the task's purpose.

If no subagent is clearly the most suitable, choose the general-purpose one. The absence of an obvious choice is **NEVER** authorisation to execute directly.

The agent indications written in other sections of this file — `rust-elite-developer` for Rust code, `rust-perf-engineer` and `extreme-code-profiler` for performance, `security-researcher` for attack surface, `specification-manager` for the specification — are particular cases of this rule, not exceptions to it.

### The rule applies to every piece of work

**Delegation is not only per task — it is per piece of work within it.** Once the task is open, its work is **decomposed**, and for **each piece** the most suitable subagent is chosen, evaluated against the agents actually installed on the machine at that moment, as in the previous section.

**Decomposition is by nature of work, never by fragment.** All work of the same nature goes in **one delegation** to the same subagent — never in successive delegations over pieces of the same problem. That is what *Synergy and Convergence of Effort* demands.

A task that spans several kinds of work — specification, code, tests, performance, security, documentation — uses **several subagents**, one per kind, each with **all** the work of that kind, and **ALWAYS in series**, under *One at a time — never in parallel*, below.

**The subagent chosen for each piece is recorded on the task**, through the `roadmap-manager` skill, so that the execution is traceable to whoever did it.

### Closed scope

Each task is executed under a **closed, objective scope, focused exclusively on its purpose**. The briefing given to the subagent is bounded by what the task defines — **title, description, requirements and comments** — and by nothing else. The briefing also states the validation tier that applies (*Test execution policy*); without it, a subagent runs the full battery by default.

The subagent **NEVER** widens the scope, **NEVER** uses the pass to fix what it finds along the way, and **NEVER** anticipates the next task. What it finds outside the scope is recorded as a comment or as a new task, through the `roadmap-manager` skill; it is not executed. Faced with the temptation to fix it now, **STOP** and record.

An effort that joins several tasks has as its scope the union of their scopes, fixed when the effort is formed and closed from then on. *Synergy and Convergence of Effort* is decided before executing; during execution it is **NEVER** a door to widen the scope.

What this section demands of the subagent, *Initiative*, below, demands of the whole session.

### One at a time — never in parallel

> This section develops **RULE ZERO**, at the top of this file. Where they seem to diverge, Rule Zero governs.

**NEVER run more than one subagent at the same time.** Use every subagent the objective requires, but ALWAYS **in series**: launch one, wait for it to finish, evaluate the result, and only then launch the next. Catching yourself about to launch two — **STOP** and serialise.

#### The launch gate

Before **every** delegation, without exception and however trivial it seems:

1. **List the agents** and confirm that **none** is running.
2. Only then launch.

**The previous agent's completion notification is not enough.** An agent can stop with background work of its own still running and **resume by itself** after notifying — in which case two agents are active without any decision having wanted it. Trusting the notification is trusting a fact that can stop being true; the listing is the real state, at that moment.

If another agent is alive — **STOP** and wait for it to finish. **NEVER** launch "just one more" while waiting.

This rule overrides any default heuristic that favours parallelism, including the habit of grouping several independent invocations in the same message so that they run concurrently, and workflow orchestration, which fans agents out. In this project the default is serial execution, and it is the default that prevails in doubt.

Only the user can authorise parallel execution, and that **express and clear authorisation, given for the request at hand, is the only condition** that lifts the prohibition — as **RULE ZERO** fixes. Without it, the series stands, whatever the argument.

Once authorised, parallelism is still **exceptional**: it holds for the request at hand, and the authorisation **is revoked at the end of the task**, restoring the serial default at once. An authorisation of parallelism sets no precedent for later tasks.

**Never invoke cost as an argument for parallelising.** The rule exists to protect the user's cost; using it the other way round — *it is faster, therefore cheaper* — is breaking it.

## Language

Language is an instrument of coordination: what is written here is what another agent will execute. A vague sentence produces vague work. The four demands apply to everything that is **written** and everything that is **interpreted** — briefings to subagents, task titles and descriptions, work-log comments, commit messages, answers to the user, and documentation:

| Demand | What it imposes |
|---|---|
| **Explicit** | State what is intended, in full. Nothing is left implicit, understood, or to the reader's inference |
| **Objective** | State what is to be executed, in verifiable terms. No generic intentions, no wording whose fulfilment cannot be checked |
| **Closed** | Bound the scope: what is in and, when not evident, what is out. An instruction without a boundary is an instruction not yet written |
| **Concise** | Few words. Cut what adds nothing — never what bounds |

Being about to write an instruction that admits two readings is reason to **STOP** and rewrite it. Faced with a **received** instruction that admits two readings, **STOP** and ask the user: **NEVER** pick one of them on your own.

### English

**All documentation is written in English** — the README, this `CLAUDE.md`, the functional specification, the technical specification, the decision records, the CHANGELOG and the code's doc comments, without exception. Flawless English is required: spelling, grammar and syntax without error, in a professional tone.

The four demands above apply to documentation with the same weight. Correct but vague English **does not** meet the requirement.

## Initiative

**Action is directed at the objective of the work in progress, and at nothing else.** Starting on your own initiative any work that was not **explicitly requested** is **forbidden**. No fix is too small, too obvious or too timely to escape this rule.

When a need outside the scope of the work in progress is identified:

1. **STOP.** It is not started.
2. **Record** the finding through the `roadmap-manager` skill, as a comment on the task or as a new task.
3. **Ask the user** how to proceed, and wait for the answer.

Resuming the work in progress is the default behaviour; starting the discovered work requires the user's decision. Using the pass to fix what is found along the way, anticipating the next task, and adding what nobody asked for are violations of the same rule.

**Seeking synergy and convergence is not starting work**, and so does not fall under this prohibition: it is done **always**, by default, without waiting for the user to ask. What requires the user's decision is joining a task **the user did not request**, under *Synergy and Convergence of Effort*.

## Completeness

**Delivering partial work is forbidden.** What is started is executed in full: **NEVER** leave a task half-done, **NEVER** declare complete what is unfinished.

This imposes three things, and none is waived:

- **Nothing is declared complete that is not.** Work ends when everything the task defines is done, the tests that prove it exist, and the validation tier the moment calls for passes (*Test execution policy*).
- **No skeleton is delivered.** No `todo!()`, `unimplemented!()`, unwritten branches, unwritten tests, or documentation left un-updated for later.
- **The scope is not narrowed in silence.** When part of the work is blocked, **all** the rest is executed, and what was left undone, and why, is stated **explicitly**. Reducing the scope is the user's decision, never the executor's.

Work that does not fit entirely in the task's scope is not delivered by halves: **STOP** and take the question to the user.

An effort that joins several tasks is complete only when **all** of them are: *Synergy and Convergence of Effort* groups the work, never waives part of it.

## Development Policy

Development follows every policy in this file, and is focused and objective: it serves the purpose of the task or of the user's request, and nothing else.

- **Tasks are written and read objectively, specifically and concretely**, without digressions or assumptions. This applies equally to generating tasks and to interpreting them; it is *Language* applied to tasks.
- **Shortest path first.** Before any development, determine the shortest path to the objective and its requirements — the strategy with the least work that does not reduce the quality of the delivery — weighing synergies, convergences and open-source prior art. Only then proceed, along that path.

## Open Source and Prior Art

### Look for a reference open-source solution

At every moment of analysis, exploration and development, check whether an open-source project already implements or solves what is needed. Use it to copy from — in full or in part, where its licence allows, and with the notices that licence requires — or to study as an example. Look for the **reference** projects in the domain.

**Only reference projects are used**: projects that are demonstrably useful and trustworthy for the specific contribution sought. A project that is not, is not used.

### Do not reinvent

When analysing an existing problem, a new feature, or the refactoring of an existing feature, do not invent a solution from scratch. Start from the information already at hand, and use — wherever possible — the knowledge, engineering, architecture or approach of an open-source project that already solves the problem, as a whole or in part. Copying and studying such projects is authorised.

The source project is only the **starting point**. `tpl` takes from it what benefits `tpl` and is never bound to the original code: what is taken is adapted to the specification and to this file's rules.

### Boundaries

- **Open source informs *how*, never *what*.** A behaviour found in another project is not a requirement until `/specification` states it — *Functional Specification*.
- **Taken code obeys this file in full**: the Rust conventions, the `unsafe` prohibition, the implementation invariants and the *Test execution policy*.
- **Adopting a project as a dependency** passes the dependency budget (*Performance and Efficiency*) and, when it changes the *Stack* table, is an architecture decision recorded before it is implemented.
- **Reading a reference project at its public source** — its official repository, documentation or published package — is the single exception to *Working Scope*.

## Investigation and Optimisation

When investigating the cause of a problem in the code, or optimising a component, first decide which technique — or combination of techniques — will identify and locate it: profiling, tracing, metrics, logs, benchmarks. Then dissect the problem until its cause is entirely clear. A cause that is not yet clear has not yet been found.

- **These techniques are means of investigation**, and none changes the *Stack*: tracing a process does not add `tracing` to the crate.
- **The performance instruments, and the agents for this work**, are in *Measurement discipline*.

## Mandatory Skills

Three domains of this project are **NEVER** operated directly: each has a dedicated skill that is the only way in. The rule is the same in all three — **NEVER invoke the underlying CLI from Bash**; invoke the skill, which operates it. Being about to write one of these CLIs in Bash is reason to **STOP** and call the skill.

| Domain | Skill | CLI the skill operates |
|---|---|---|
| Git writes | `gitflow` | `git` (write operations) |
| Sprints, tasks and comments | `roadmap-manager` | `rmp`, every command except `rmp graph …` |
| Knowledge about the project | `knowledge-authority` | `rmp graph …` |

### 1. Git — skill `gitflow`

**All Git write operations go through the `gitflow` skill**, following the good practices of that branching methodology. This includes, without exception: `commit`, `add`, creating and deleting branches, `merge`, `rebase`, `tag`, `push`, `stash`, and any history rewrite.

The repository follows the **gitflow** model (Vincent Driessen): opening and closing sprints, recording a closed task as a commit, creating releases and hotfixes — all of that is the skill's decision, not improvisation.

Reading is free: `git status`, `git log`, `git diff`, `git show` and equivalents may be run directly, because they change no state.

**NEVER** commit directly to `main`.

### 2. Sprints, tasks and comments — skill `roadmap-manager`

**All management and coordination of sprints, tasks and comments goes through the `roadmap-manager` skill**, the only operator of the `rmp` CLI (Groadmap) for **every command except `rmp graph …`**, which belong to the `knowledge-authority` skill. This covers creating, listing, inspecting and editing tasks; the sprint lifecycle (plan, start, close, reopen); status transitions; priorities and severities; dependencies and subtasks; reordering; the audit log and statistics; and the typed comment log.

`rmp` is the **only source of truth** for planning and executing this project's tasks. **NEVER** use ad-hoc files, chat lists, TODOs in the code or any other tool as a substitute.

**This project's roadmap:** `tpl`.

#### No work outside an open task

**All work is executed on an open task in `rmp`, and only on it.** Starting work on a task that was not opened is a **process violation**, not a shortcut — and it is not justified by the change being small, obvious or urgent.

**The two sets of states are not confused.** `OPEN` is a *sprint* state, never a task state:

| Entity | States |
|---|---|
| Task | `BACKLOG`, `SPRINT`, `DOING`, `TESTING`, `COMPLETED` |
| Sprint | `PENDING`, `OPEN`, `CLOSED` |

A task is executable when **both** conditions hold:

1. It belongs to the sprint whose state is `OPEN`.
2. It was itself moved to `DOING` with `--commit-open <hash>`.

Failing either is enough for the task **not** to be executable. Concretely:

- A task in `BACKLOG` is **NEVER** executed.
- A task in a `PENDING` or `CLOSED` sprint is **NEVER** executed.
- Bringing a task into the open sprint is a **planning action** and **requires the user's confirmation**. It is **NEVER** done on one's own initiative to unblock work.

If either condition fails, **STOP** and regularise the task through the `roadmap-manager` skill before writing the first line.

**Work discovered midway that has no task of its own is recorded as a task or as a comment — it is never executed.** That is *Closed scope*, in *Task Execution by Subagents*, and it applies here unchanged.

**This rule governs the work, not the search.** Seeking synergy and convergence is not work, is not recorded as a task, and does not go through this gate — that is *The search is not a task*, in *Synergy and Convergence of Effort*.

The task lifecycle is **NEVER** an exception to the rule at the top of this section: every status transition and every comment goes through the `roadmap-manager` skill.

#### Closing: commit first, then the task

The commit gate imposes the order, and the order is not inverted:

1. The work is **recorded in a commit**, through the `gitflow` skill.
2. The task is **closed** with `--commit-close <hash>`, with the real hash of that commit.

The hash comes from the commit actually created. **NEVER invent, guess or reuse a hash** to satisfy the gate: with no commit created, **STOP** and create it through the `gitflow` skill.

In an effort that joins several tasks, each closes with the hash of the commit that **records its work**; when a single commit records the work of several, that hash closes each of them. What the rule forbids is the hash of a commit that does not contain the task's work.

#### The work log

The typed `rmp` comments are this project's **work log**. They are written **as the work happens**, never reconstructed at the end, so that what was done — and why — can later be recovered from them alone.

- **On the sprint in progress**: its progression towards the sprint's purpose — material advances, sprint-level decisions, findings that affect more than one task, and changes to its scope with their reason.
- **On each task**: the progression of that task.

Every entry uses the type that matches what it records — never `NOTE` where another type applies. The `roadmap-manager` skill owns the lexicon; the types are:

| Type | Task | Sprint | Written when |
|---|---|---|---|
| `HYPOTHESIS` | yes | no | Before testing a proposition: the proposition, and what would confirm or refute it |
| `TEST` | yes | no | A test, benchmark or verification ran: the exact command, and what it showed |
| `FINDING` | yes | yes | Something was discovered: behaviour observed, a cause identified, a hypothesis refuted |
| `DECISION` | yes | yes | A choice was made between viable options: the choice, the reasoning, and the options rejected |
| `PROGRESS` | yes | yes | The work advanced materially: what is done, and what remains |
| `UPDATE` | yes | yes | The definition of the task or sprint changed: why it changed |
| `NOTE` | yes | no | Something worth keeping fits no other type: context, a pointer, an out-of-scope follow-up |

A task is **not ready to close** until its log holds a `DECISION` for every non-obvious choice, a `TEST` for the verification of its acceptance criteria, a `FINDING` for whatever a future reader would need, and an `UPDATE` for every change to its definition. **STOP** and write what is missing before closing.

### 3. Project knowledge — skill `knowledge-authority`

**All querying and management of knowledge about the project goes through the `knowledge-authority` skill**, the only operator of `rmp graph …` and of the knowledge graph behind it. The graph is managed **exclusively** by this skill.

It is the **first place to consult** for any factual question about this repository's code — which file or function implements a feature, which modules, types, tests or dependencies exist, what depends on what, how the architecture fits together, the reach and impact of a change. **ALWAYS** consult the graph **before** reading files, even when the answer seems one `grep` away. Being about to open a file to answer a factual question without having consulted the graph is reason to **STOP** and consult it.

Use it also to sync, refresh or audit the graph after a commit or when the project's files change, and to maintain `./knowledge-model.md` — a file written exclusively by this skill.

### Boundaries between the three

The three skills do not substitute for each other, and confusing them is the usual mistake:

- The boundary between `roadmap-manager` and `knowledge-authority` is the subcommand: `rmp graph …` is `knowledge-authority`; all the rest of `rmp` is `roadmap-manager`.
- Managing tasks and sprints is `roadmap-manager` — **never** `knowledge-authority`, even though both use the `rmp` binary.
- Querying the knowledge graph is `knowledge-authority` — **never** `roadmap-manager`.
- Recording work in Git is `gitflow` — **never** a stray `git commit`, even when the change is trivial.

## Functional Specification

**All work rests on a specification.** No feature is implemented, changed or removed without first being described in the project's functional specification, in `/specification`.

The specification is `tpl`'s **only source of functional truth**: what is not there is not a requirement, and is **NEVER** implemented on one's own initiative, by analogy with another project, because it seems obvious, or because it was mentioned in passing in a conversation. Faced with a request the specification does not cover, **STOP** and formalise it first.

### Coordination

Coordinating the specification is the **exclusive** responsibility of the `specification-manager` subagent. It is the only one that writes in `/specification`, and all work on that folder goes through it:

- bootstrapping the structure and maintaining the navigation `README.md`;
- creating and changing functional modules, business rules, actors, use cases and the glossary;
- assigning and maintaining stable requirement identifiers;
- detecting inconsistencies, ambiguities, contradictions and gaps between files;
- maintaining cross-references;
- deprecating and archiving obsolete content;
- structured audits.

**No other agent edits `/specification`** — not even to fix a typo.

### The specification precedes implementation

Without exception:

1. The request arrives. Check whether it is **covered by the specification**.
2. If it is not — or is covered ambiguously, incompletely or contradictorily — **STOP**: the **first** task is to have the `specification-manager` subagent formalise it. Only then is it implemented.
3. Implement exactly what the specification defines: no less, no more.
4. If, during implementation, the specification turns out to be wrong or incomplete, **STOP**. Correct it through the `specification-manager` subagent, and only then resume. **NEVER** implement against the specification, and **NEVER** leave code and specification diverging.

This is step 1 of the *Workflow*, and the reason it is first.

## Technical Specification and Decision Records

The functional specification says **what** `tpl` does; **how** it is built lives in `docs/spec-technical/`, written **exclusively** by the `technical-writer` subagent. **No other agent edits it** — not even to fix a typo.

Nothing there is a requirement: every statement traces to a requirement in `/specification` or to an explicit constraint. Where the two disagree, the divergence is a defect to report — it is **never** closed by rewriting the functional specification, and **never** left standing. The index, what each document answers, and where to start are in `docs/spec-technical/README.md`; read them there — they are not copied here.

### Architecture decision records — `docs/adr/`

The architecture decision records (`ADR-NNN`, in `docs/adr/`) keep the **why** of a decision and the alternatives that were rejected — the rationale the technical specification does not carry. The `adr-guardian` subagent writes them, and nobody else.

**A record is required** in two cases: when a requirement in force delegates the fact outside the functional specification, and when the decision is architectural and the rejected alternatives must survive the choice. Outside these cases no record is opened — the decision stays in `docs/spec-technical/`. Admission is narrow on purpose, and rule R4 of `docs/adr/README.md` fixes it.

**What a record keeps is repeated nowhere else.** This file cites `ADR-NNN` — as the *Stack* rows and the release-profile rule already do — and never rewrites what the record says. Being about to bring here a version, a figure or a rationale that a record already keeps is reason to **STOP** and cite.

The lifecycle of a record, its states, numbering, format, and the four rules that govern it are in `docs/adr/README.md`. Read them there — **they are not copied here**.

## Workflow

1. **Specify** — formalise the requirement in the functional specification, through the `specification-manager` subagent. No later step starts without this one.
2. **Implement** — write the code that satisfies it, with the tests that prove it.
3. **Test** — run the validation tier the moment calls for (*Test execution policy*), against real MariaDB when the requirement touches the catalogue.
4. **Document** — update the README, doc comments and CHANGELOG.

The steps run in this order and **NEVER** is one skipped. Catching yourself implementing without step 1 done — **STOP** and go back to it.

The four steps run over the **whole effort**, not over each fragment: everything it requires is specified, then everything is implemented, then everything is tested, then everything is documented. The order does not change because of this.

## Working Scope

**This project's scope is restricted to the repository's root folder.**

**NEVER** read, list, search, inspect or reference anything outside `tpl/`. This includes sibling directories, the user's other projects, and any path above the repository root. Looking for conventions, seeing how another of the user's projects solved something, or confirming a pattern are **NEVER** exceptions.

**The single exception** is reading a reference open-source project at its public source, under *Open Source and Prior Art*. It never extends to the user's own projects or directories.

Every other piece of information the work needs lives in this repository. If something is missing, **STOP** and ask the user — **NEVER** go looking for it elsewhere.

## Implementation Invariants

Two permanent constraints on implementation choices. They take priority over any other design consideration, and neither is negotiated case by case.

### 1. Absolute read-only

`tpl` **never** issues DDL, DML or any write statement against the database it connects to, and no flag turns this behaviour off. The catalogue is read through `INFORMATION_SCHEMA` for a `mariadb` and a `mysql` entry — for a `mysql` entry, the tables of `FR-MYSQL-015` alone — and through `sqlite_schema` and the pragma functions of `FR-LITE-015` for a `sqlite` entry — never through `mysqldump` or any external process.

The detail of the guarantee — how read-only mode is imposed, what happens when it cannot be imposed, and which statements are admissible — belongs to `specification/server-contract.md` for a `mariadb` entry, to `specification/mysql-contract.md` for a `mysql` entry, and to `specification/sqlite-contract.md` for a `sqlite` entry, and is read there.

### 2. Runtime rendering, with MiniJinja

Templates are **files on disk, loaded and compiled at render time** — never embedded in the binary at compile time. The engine is **[MiniJinja](https://docs.rs/minijinja)** (`minijinja` + `minijinja-contrib`).

**Do not use Askama, Tera, Handlebars, `std::fmt`, or any other engine.** Askama in particular is explicitly excluded for being *compile-time*: it would force `tpl` to be recompiled whenever a template changed, which defeats the reason this tool exists.

The configuration required of the `Environment` — loader, behaviour on undefined variables, auto-escape, path-resolution boundary, and the content of a render error — belongs to `specification/template-environment.md` and `specification/render-semantics.md`.

## Stack

The record of binding technology choices. Any change to this table is an architecture decision and must be recorded before it is implemented.

| Area | Choice | Notes |
|---|---|---|
| Language | Rust (edition 2024) | MSRV fixed in `ADR-007`, in `docs/adr/` |
| CLI | `clap` v4 (derive) | Command tree, aliases, `--help` per subcommand |
| Templates | `minijinja` + `minijinja-contrib` | Runtime, always |
| MariaDB and MySQL access | `sqlx-core` + `sqlx-mysql`, with `tokio` | Taken directly, not through the `sqlx` facade, for both server providers; versions, features, runtime scope and the rejected candidate: `ADR-003` and `ADR-005`; the `mysql` provider's driver and the patch it carries: `ADR-014`, in `docs/adr/` |
| SQLite access | `rusqlite`, with the bundled SQLite library built with extension loading compiled out | Driver, library release, build options and rejected alternatives: `ADR-013`, in `docs/adr/` |
| Serialisation | `serde` + `serde_json` | The render context is `serde`-serialisable |
| Configuration | `toml` + `serde` for reading; `toml_edit` for writing | Writing preserves comments and order; `OD-09`, in `docs/spec-technical/open-decisions.md` |
| Errors | `thiserror` in the library; the binary carries no error type of its own | `main.rs` reads the library's `exit_code` and returns it; `OD-32`, in `docs/spec-technical/open-decisions.md` |
| Logging | Own diagnostics, no subscriber installed | No `tracing` or `tracing-subscriber`; controlled by the verbosity flag; `OD-17`, in `docs/spec-technical/open-decisions.md` |
| Process uid, helper process group, and directory-relative file operations | `rustix`, `default-features = false`, `features = ["fs", "process"]` | Safe `getuid`, `kill_process_group`, `waitid`, `openat`, `unlinkat`, `renameat` and `mkdirat`, and `O_NOFOLLOW`/`O_NONBLOCK` opens; through `libc` they would require `unsafe`; `OD-24` and `OD-12`, in `docs/spec-technical/open-decisions.md` |
| Heap accounting | `cap` over `std::alloc::System`, as `#[global_allocator]` | Render memory limit (`FR-RND-039`), read by the deadline thread; version, hard-limit policy and rejected alternatives: `ADR-011`, in `docs/adr/` |

> **Closed decision — MariaDB driver.** The choice, the rule that decided it, the rejected candidate, and the measurement that confirmed the choice — refuting the suspicion that used to be written here — are in `ADR-003`, in `docs/adr/`. The runtime scope is in `ADR-005`.

## Supported Platforms

`tpl` is a **Unix** tool. That is the family of systems it is written for, and the boundary of what the code may assume.

The supported and verified systems are **Linux** and **macOS**, on the **arm64** (`aarch64`, Apple Silicon included) and **amd64** (`x86_64`) architectures. Other Unix systems — the BSDs, for example — should work, since nothing in `tpl` depends on a specific system beyond what `std` already abstracts, but they are **neither tested nor guaranteed**: they are not in the matrix, they do not run in validation, and a problem that shows only on them does not fail a change.

**Windows is out of scope.** It is not a target, no code is written to accommodate it, and no dependency is accepted because of it.

The concrete target matrix — target triples, libc choice and linkage — is fixed in `NFR-PERF-018`, in `specification/performance-requirements.md`, which also records the observable consequence of static linking against `musl` on name resolution. The build path of each target is `ADR-008`, and the validation and packaging of the distributed artefacts are `ADR-012`, both in `docs/adr/`. **They are not repeated here.**

### Rules that follow

- **No target is second-class.** What passes the full battery (*Test execution policy*) must pass on every supported target; a failure on any of them fails the change, whatever the target.
- **Platform-specific code is the exception**, and is isolated behind `#[cfg(unix)]`. `#[cfg(windows)]` has no place in the crate. The restrictive permissions required of the configuration file depend on `std::os::unix::fs::PermissionsExt`, and are on their own reason enough for the boundary to be Unix.
- **Paths always through `Path` and `PathBuf`**, never through string concatenation with a hand-written separator.
- **No CPU features assumed.** No `target-cpu=native` in the release profile: it would break the portability and reproducibility of the distributed binary.
- **Dependencies must compile and pass tests on every target.** A crate that does not support `aarch64` does not enter — a criterion added to the dependency budget, not a replacement for it.
- **A performance baseline always identifies the target it was measured on.** Figures measured on different targets are not compared.

## Project Structure

```
tpl/
├── Cargo.toml
├── BENCHMARKS.md            # record of performance readings, with the target each was measured on
├── knowledge-model.md       # graph model — written only by the knowledge-authority skill
├── src/
│   ├── main.rs              # entry point: parse, dispatch, exit-code mapping
│   ├── cli/                 # clap tree — one module per porcelain command
│   ├── project/             # project-folder discovery and configuration reading
│   ├── provider.rs          # dispatch of a read to the selected entry's provider
│   ├── mariadb/             # INFORMATION_SCHEMA reader (read-only)
│   ├── mysql/               # MySQL catalogue reader (read-only)
│   ├── sqlite/              # SQLite catalogue reader (read-only, hardened)
│   ├── model/               # the model read from the catalogue
│   ├── render/              # MiniJinja Environment, loader, filters, tests, functions
│   └── error.rs             # error type + exit codes
├── templates/               # starter templates
├── tests/                   # integration tests (end-to-end CLI)
├── benches/                 # benchmarks
├── scripts/mariadb/         # MariaDB fixture: images, SQL, TLS material, harness
├── scripts/mysql/           # MySQL fixture: images, SQL, TLS material, harness
├── scripts/sqlite/          # SQLite fixture: SQL text built into databases at test time
├── examples/                # complete pipelines: schema → template → output
├── specification/           # functional specification
└── docs/                    # technical specification and decision records (ADR)
```

`model/` is the project's boundary: it is at once the result of introspection and what a template sees. Its shape is not decided in code — the specification fixes it (`specification/catalogue-coverage.md` and `specification/context-document.md`), and `model/` implements it. The contract is that document, not the types that produce it: the `model/` structs are not public surface, and the library carries no compatibility promise — `DIV-032`, in `specification/upstream-divergences.md`, and `docs/spec-technical/overview.md`.

## Rust Code Conventions

The code of this project — and the way it is organised — follows the **good practices and idiomatic conventions of the Rust language**. This is not a style preference: it is a project rule, and it applies to new code as much as to refactored code.

### Organisation

- **Modules.** Names in `snake_case`, without obscure abbreviations and without repeating the parent's name (`mariadb::reader`, never `mariadb::mariadb_reader`). One module per concept, aligned with the tree in the previous section.
- **One module-file style.** Always use the `foo.rs` form together with the `foo/` directory; do **not** use `mod.rs`. Mixing the two styles in the same tree is forbidden.
- **Minimal visibility.** Everything is private by default. `pub(crate)` for what crosses modules, `pub(super)` for what only the parent needs, and `pub` reserved for what the library publishes — `model/`, `error.rs` and the four entry functions (`docs/spec-technical/interfaces.md`, *The library shape*), with no stability promise, per `DIV-032`.
- **Deliberate re-exports.** `lib.rs` re-exports a coherent API with `pub use`; a whole module is not re-exported just to save a `use` path.
- **Library and binary separated.** The logic lives in the library and is testable without launching a process; `main.rs` only parses, dispatches, and maps the error to an exit code.

### Naming

Conformance with the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and RFC 430:

- `snake_case` for functions, variables, fields and modules; `UpperCamelCase` for types, traits and variants; `SCREAMING_SNAKE_CASE` for constants and statics.
- Conversion conventions, chosen by cost: `as_` (cheap borrow), `to_` (costly, allocates), `into_` (consumes the receiver).
- Getters without a `get_` prefix: `table.name()`, never `table.get_name()`.
- Iterators through the conventional trio: `iter`, `iter_mut`, `into_iter`.

### Types and errors

- **`Result` and `Option` with the `?` operator.** `unwrap`/`expect` outside tests only with the invariant documented in the message itself. The error type follows the *Stack* table: `thiserror` in the library, and the binary carries no error type of its own.
- **Newtypes for invariants** — an already validated table name is not just any `String`.
- **Generics or `impl Trait`** instead of needless indirection; `Box<dyn Trait>` only when the heterogeneity is real.
- **Conversions through `From`/`TryFrom`**, not through stray functions when the trait serves.
- **`std` traits implemented where they make sense**: `Debug` always, and `Display`, `Default`, `FromStr`, `AsRef` where the type justifies it.
- **Signatures in borrowed types** — `&str`, `&[T]`, `&Path` in parameters, never `String`, `Vec<T>` or `PathBuf`. It is also what the minimal-allocation rule demands.

### Expression

- **Iterators and combinators** instead of indexed loops with a mutable accumulator, when they cost neither clarity nor performance.
- **Exhaustive pattern matching**, without a `_ =>` that silently swallows future variants of an `enum` of the crate itself.
- **`#[non_exhaustive]`** on public types expected to grow — typically the error `enum` and the `model/` structs. It is a code convention, not a compatibility promise.
- **Derives instead of manual implementations** whenever they are equivalent.

### Tools as arbiter

`rustfmt` with the default configuration is the formatting authority, and `clippy` with `-D warnings` is the idiom authority. Both run in the validation tiers of the *Test execution policy*, and their verdict is not debated case by case.

In doubt, the reference is the language's convention and the Rust API Guidelines — never habit brought from another language.

## Development

```bash
cargo build
cargo run -- --help
cargo test
```

### Test execution policy

**Every delivered change carries the tests that prove its conformance to the requirements it implements** — unit, integration, end-to-end, or whichever kind the case calls for.

**Tests run logically and efficiently.** What runs is chosen by what changed and by the moment: close to development, only the code the change touched; at sprint close and before a release, the whole. A test that the change cannot make fail is not run, and the time it would take is saved.

| Moment | What changed | What runs |
|---|---|---|
| Development | Documentation only — Markdown files, with no change to source, tests, manifests, templates, scripts or fixture | **Nothing.** No code changed, so there is nothing to test |
| Development | Rust code (`src/`, `tests/`, `benches/`) or the Cargo manifests | `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`; the tests that exercise the changed code, selected by test target and name filter; `cargo audit` only when `Cargo.toml` or `Cargo.lock` changed |
| Development | Anything else — templates, examples, scripts, fixture | The tests that exercise what changed |
| Sprint close, `git push`, pre-release | — | **The full battery**, below |

A doc comment inside a `.rs` file is Rust code: it compiles, and may hold a doctest. Tests that need the fixture run in the development tiers only when the changed code reaches the server.

#### The full battery

On sprint close, before `git push`, and before a release, these commands run in this order and all must pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --all-features
cargo audit
```

**Nothing performance-related is added to any tier.** A change to a hot path — catalogue reading, context construction, render, process start-up — is an occasion to take a reading and record it, under *Measurement discipline*; it is never a condition for the work to be complete.

`clippy` with `-D warnings` is not negotiable. A lint whose suppression is justified requires a local `#[allow(...)]` with a comment stating the reason — never a crate-level suppression.

### Rust development

**Always** use the `rust-elite-developer` agent for Rust programming tasks: creating, editing, refactoring or reviewing code. For performance work on hot paths, use `rust-perf-engineer`. For attack-surface analysis (parsing untrusted input, credentials, executing external commands), use `security-researcher`.

`unsafe` is forbidden in this project. Keep `#![forbid(unsafe_code)]` at the top of the crate.

## Performance and Efficiency

Performance and economy of resources are **design and architecture** demands: they weigh on every construction decision of `tpl` — what it does at start-up, how often it goes to the server, how much it holds in memory. `tpl` is a command-line tool, invoked repeatedly and often inside loops and build pipelines — every millisecond is paid many times, and it is for that use that it is built fast and frugal.

**No figure fails a change.** No figure in the specification, and no reading recorded in `BENCHMARKS.md`, fails, blocks or holds back a change, a release or a piece of work — that is `BR-PERF-008`, in `specification/performance-requirements.md`. The rules in this section are construction rules, not acceptance rules.

**The figures do not live here.** The required properties, the reference workloads, the measurement points and the protocol belong to `specification/performance-requirements.md`; the readings actually measured live in `BENCHMARKS.md`.

**What fails a change are the shape requirements.** Deterministic counts and absences — catalogue queries do not depend on the number of objects, a cache hit opens no connection, an invocation opens at most one connection — are correctness invariants, asserted by the test suite wherever the instrument each assertion needs is within the run's reach: those observed on the server side need the fixture up, and the syscall shape of the discovery and configuration clauses needs a host that allows tracing, which none of the Darwin targets does — that is `NFR-PERF-007` and the section *What an assertion needs in order to run*, in `specification/performance-requirements.md`. A skipped assertion is not a weakened requirement: every clause holds on all four targets and is verified on all four, and a run that cannot reach the instrument skips the assertion stating the reason, and never reports it as passed in silence. Breaking them is a functional defect, not a slow run.

### Implementation rules

- **Catalogue round-trips do not scale with the number of objects.** An `N+1` in the catalogue reader is a defect, not a matter of style.
- **Each template is parsed once per process.** MiniJinja's `Environment` keeps the compiled template and reuses it; reparsing per iteration is forbidden.
- **No start-up work that the invoked command does not need.** No heavy static initialisation, no building the `Environment` for an invocation that does not render, no connecting to the database for a command that does not use it. Lazy initialisation by default.
- **Aggregated I/O.** Writing through buffered writers; never one syscall per line.
- **Minimal allocation on hot paths.** Borrow instead of cloning; `&str` instead of `String`; `Cow<'_, str>` when it actually avoids a copy. A `.clone()` on a hot path requires justification.
- **Pre-size collections** whose cardinality is already known (`with_capacity`).
- **Streaming wherever the operation allows it.** Memory must scale with the largest single object, not with the whole database, in everything that does not require the complete document.
- **A single database connection**, opened as late as possible and closed as soon as the read ends. No pool: the process is ephemeral and makes a handful of queries.
- **Dependency budget.** Every crate must justify its presence. Prefer `std`. Before adding a dependency, check what it pulls in (`cargo tree`) and what it costs (`cargo bloat`, start-up time). A dependency used only for a trivial function does not enter.
- **Release profile** tuned in `Cargo.toml`: the five settings, and the composition with the panic hook that satisfies `FR-ERR-030`, are in `ADR-004`, in `docs/adr/`.
- **No speculative parallelism.** Concurrency enters only with a benefit measured, in a benchmark, on a representative workload. Parallelising because it is possible is forbidden — the cost of synchronisation and of starting threads is real, and on small workloads it loses.

### Measurement discipline

**No performance claim without figures, and no optimisation without measuring before and after.** Intuition about what is fast is not evidence. The instruments below are informative: they answer questions, at the request of whoever asks them, and fail nothing.

| Tool | Use |
|---|---|
| `hyperfine` | End-to-end wall time of the CLI, start-up included |
| `criterion` | Function micro-benchmarks (parsing, type mapping, render) |
| `dhat-rs` | Allocation profile and heap peak |
| `samply` / `cargo flamegraph` | Attribution of CPU to call sites |
| `cargo bloat` | Each crate's contribution to the binary size |

Benchmarks live in `benches/` and run against the dataset of the MariaDB containers, so that they are reproducible. Readings are recorded in `BENCHMARKS.md`, always identifying the target they were measured on, and figures measured on different targets are not compared. **`BENCHMARKS.md` is a record of observations** — informative, consulted on request, and never a gate.

For optimisation work, use the `rust-perf-engineer` agent; for cause-and-effect investigation between implementation and measured behaviour (RAM, CPU, vacuous or redundant code), use `extreme-code-profiler`. The choice of technique follows *Investigation and Optimisation*.

## Testing against MariaDB

Tests or validations that need a real database **must** use the fixture of the provider they read: `scripts/mariadb/` for MariaDB, `scripts/mysql/` for MySQL, and `scripts/sqlite/` for SQLite, below. **Never** use external instances, mocks or stubs as a substitute.

`scripts/mariadb/` does not hold only server images and SQL: the servers present the TLS certificate versioned in `scripts/mariadb/tls/`, and beside them runs a server that offers no TLS at all. What is there, how it is launched and how it is verified is in `scripts/mariadb/README.md` — **read it there; it is not copied here**.

**The fixture is not operated by hand.** It brings its own harness — a set of scripts that starts it and verifies it server by server, stops it and proves that nothing was left behind, answers whether it is up before a server-dependent test runs, and instruments the observations the specification requires to be made outside the process. The fixture is always operated through them: a hand-written Docker command in place of the harness is reason to **STOP**. Which scripts they are, how they are invoked and what each does is read there.

`tpl` supports **more than one MariaDB series**. Which series, and what a difference between series obliges, belongs to `specification/server-contract.md` and **is not copied here**.

Whenever the content, structure or types returned by an `INFORMATION_SCHEMA` query must be confirmed:

1. Start the fixture through the harness, on **every supported series**, and confirm through it that all of them responded.
2. Run the real query on each series and observe the actual response.
3. Confirm the behaviour in the official MariaDB documentation.
4. Document the code according to what was observed **and** confirmed.
5. Stop the fixture through the harness, which confirms that nothing of it was left running.

**A divergence between series is itself a finding** and is recorded as such. It is the reason `tpl` reads more than one version, and it is what the specification requires to be handled — not a footnote.

**It is forbidden** to assume, infer or document the catalogue's behaviour without executing these steps. Generic knowledge of MySQL is not enough: the divergences between MariaDB and MySQL in `INFORMATION_SCHEMA` are real and relevant.

The scripts `scripts/mariadb/setup.sql` and `seed.sql` must exhaustively cover the catalogue surface `tpl` reads: every native MariaDB data type, simple and composite primary keys, unique and composite indexes, foreign keys with distinct `ON UPDATE`/`ON DELETE` rules, generated columns, views, procedures and functions, triggers, and comments. The modelled domain must be realistic, never `id=1, name='test'`. The DDL must be accepted by **every** supported series.

The demand for exhaustiveness does not end at the catalogue: it applies equally to the transport surface, and it is `FR-CONF-038`, in `specification/configuration-model.md`, that fixes what the TLS material and the server without TLS must make demonstrable.

**MySQL.** A test that reads a MySQL server uses the fixture of `scripts/mysql/`, operated through its own harness on the same terms as the MariaDB fixture — never by hand. What it holds, how it is launched and how it is verified is in `scripts/mysql/README.md`; what it must cover is `FR-MYSQL-044`, in `specification/mysql-contract.md`. **Neither is copied here.**

**SQLite.** A test that reads a SQLite database uses the fixture of `scripts/sqlite/`: SQL text that the tests build into database files at test time, through the SQLite library the binary carries. No binary database file is committed, and there is no container and no harness to start. What each file builds, and how the tests build it, is in `scripts/sqlite/README.md`; what the fixture must cover is `FR-LITE-038`, in `specification/sqlite-contract.md`. **Neither is copied here.**

## Documentation

- The language and the writing demands are those of *Language*: flawless English, explicit, objective, closed and concise. Read them there; they are not repeated here.
- The register is technical and addressed to human readers.
- Documentation must be **faithful to the code**: never describe commands, flags, filters or structures that are not implemented exactly as described. While the implementation does not exist, the README must say so explicitly.
- Every public (`pub`) item carries a doc comment. `#![warn(missing_docs)]` in the crate.
