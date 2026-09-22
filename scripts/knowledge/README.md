# The knowledge graph's Rust layer

One command re-derives the Rust half of the project's knowledge graph from the
source tree.

```sh
scripts/knowledge/rust-layer.py
```

That is the whole of the procedure it replaces. Until now the scan that keeps
this layer current existed only as a script written from scratch inside whatever
session happened to need it, which made the refresh reproducible **in substance**
— a reader knew what to do — and not **by one command**. `CLAUDE.md` makes the
graph the first place to consult for any factual question about the code, and a
graph nobody can cheaply refresh decays into a second, stale copy of the tree. A
procedure whose first step is *write a scanner* does not prevent that.

## Contents

| File | Purpose |
|---|---|
| `rust-layer.py` | The scanner. Emits the layer; verifies it against the source |
| `.gitignore` | Keeps the interpreter's bytecode cache out of the repository |
| `README.md` | This file |

Python 3, standard library only, no configuration and no dependencies. The tree
is the input and stdout is the output.

## It emits; it does not write

**Nothing here writes to the graph.** The script opens no connection to it,
issues no Cypher write, and never touches `~/.roadmaps/`. It writes statements to
standard output, and the `knowledge-authority` skill — the graph's only writer —
decides what to do with them. That separation is deliberate: the skill owns the
write rule, the provenance discipline and the reconciliation of what is already
there, and a scanner that loaded its own output would own none of them.

It also does not write `knowledge-model.md`. That file carries the vocabulary
this script speaks and is written by the `knowledge-authority` skill alone.

## What it derives

Everything a `.rs` file under `src/` or `tests/` declares, in the vocabulary
`knowledge-model.md` fixes:

| Label | Derived from |
|---|---|
| `Component` | One per scanned file: the binary for `src/main.rs`, a `module` for every other file under `src/`, a `test-crate` for every file under `tests/` |
| `Type` | `struct`, `enum`, `union` and a module-level `type` alias |
| `Trait` | A `trait` declaration |
| `Function` | A free function — one written in a module, never in an `impl` and never in a `trait` |
| `Method` | A function written in an inherent or trait `impl` |
| `Test` | A function carrying `#[test]` or `#[bench]`; `unit` under `src/`, `integration` under `tests/` |
| `Variant` | One variant of an `enum`, read from the enum's own body |
| `Const` | A `const` or `static`, at module level or in an `impl` |

| Predicate | Derived from |
|---|---|
| `IMPLEMENTED_BY` | `(Component)→(File)`, the file that realises the component |
| `DECLARES` | `(File)→(symbol)`, carrying the 1-based declaration `line` |
| `MEMBER_OF` | `(Method)→(Type)` and `(Variant)→(Type)`, where this crate declares the type |
| `IMPLEMENTS` | `(Type)→(Trait)`, where this crate declares the trait |
| `VERIFIES` | `(Test)→(Requirement)`, from the test's own name: `fr_cdoc_001_…` verifies `FR-CDOC-001` |

Every node and every edge is stamped with `gitCommit` and `gitDate`, which
`knowledge-model.md` requires of every element. The stamp is `HEAD`; when the
scanned tree has uncommitted changes the script says so on standard error,
because the stamp then names a state that is not what was read.

### What it deliberately does not derive

Each of these is a decision, not an omission:

- **`File`, `Directory`, `Requirement`, `Decision` and `Baseline`.** They come
  from the document layer, not from the Rust tree. Every edge this script emits
  binds such a node with `MATCH` before it `MERGE`s the relationship, so a node
  this layer does not own is never created here — a `MATCH` that binds nothing is
  a silent no-op, where a pattern `MERGE` would leave a stub.
- **`Crate` and `DEPENDS_ON`.** They come from the `[dependencies]` table of
  `Cargo.toml`, which is not a `.rs` file.
- **`CALLS` and `SATISFIES`.** `knowledge-model.md` records both as deferred by
  an explicit decision.
- **A `crate-root` component for `src/lib.rs`.** `Component.name` is an identity
  property, and the crate root's module path is `tpl` — the same name the binary
  carries. Two components cannot hold one identity, so one of them has to give
  way, and this script gives it to the binary and emits nothing for the crate
  root. **Choosing differently is a change to the model**, and the model belongs
  to the `knowledge-authority` skill; the collision is recorded here so that the
  skill can settle it rather than discover it.
- **Items written inside a function body, and the method signatures a `trait`
  declares.** Neither is a free function.

### How a key is derived

| Input | Key |
|---|---|
| `src/model/table.rs` | module `tpl::model::table` |
| `src/lib.rs`, `src/main.rs` | module `tpl` |
| `tests/support/fixture.rs` | module `support/fixture` — a test crate is not a module of the library, and it keeps the shape of its own path |
| an inline `mod tests` | the enclosing module with `::tests` appended |
| a `Test` | `<file>::<function name>`, the file being repository-relative |
| a `Method` | `<owner type key>::<method name>` |

**An `impl` is repaired against the file's `use` list.** A key is derived from
the file that writes it, so `impl Named for Table<'_>` written in
`src/model/document/order.rs` would name `tpl::model::document::order::Table` — a
type that does not exist. The file's `use crate::model::table::Table` is what
repairs it, and `knowledge-model.md` requires the repair. Only `crate::`,
`super::`, `self::` and a module the file itself declares are resolved; an import
from another crate names no node of this graph and is left unresolved, which
`--verify` reports by name.

**The `use` list is read per scope and never per file.** An inline `mod tests`
routinely writes `use super::{Thing, …}` for the very names the module around it
declares, and a file-wide map would let that import decide how the surrounding
module's own `impl` blocks resolve — one module too high, silently, for every
name the test module re-imports.

### Why the pass is brace-depth aware

Whether a function is a `Function` or a `Method` is decided by the block it sits
in, and so is whether an identifier at the head of a line is an enum variant. A
line-by-line grep cannot tell them apart. The scan therefore tracks the block
structure, and it never reads a raw line: Rust block comments nest, a raw string
ends on a quote followed by as many hashes as opened it, and `'{'` is a character
and not a block — each of those turns a naive brace count into the wrong answer.

## Verifying it

```sh
scripts/knowledge/rust-layer.py --verify                       # every record
scripts/knowledge/rust-layer.py --verify --sample 151 --seed 0 # a drawn sample
```

This is the check the layer is answerable for, and **the source tree is the
ground truth**. It reopens each record at the `file:line` it claims and confirms
that the source there declares that symbol, with that name, that kind and that
visibility, using a **single-line matcher that shares no code with the scan**.
The scan reaches a declaration through a stateful pass over a whole file; the
verifier reaches it through one line. A record the scan put on the wrong line, or
attributed to the wrong name, kind or visibility, fails. Reading the whole layer
also confirms that every `MEMBER_OF` and `IMPLEMENTS` endpoint is a node this
layer declared.

Exit `0` when every record read matched, `1` when any did not.

**The sample is drawn deterministically, which is what makes a result
reproducible rather than merely repeatable.** The records are sorted by identity
and drawn with a seeded generator, so `--sample 151 --seed 0` names the same 151
records on every run and on every machine. Re-running the command below is the
whole of the procedure.

### What it reported, on 2026-09-21 at `243c4d6`

```
$ scripts/knowledge/rust-layer.py --verify --sample 151 --seed 0
sampled 151 of 3236 records at file:line with seed 0; 0 mismatched

$ scripts/knowledge/rust-layer.py --verify
checked 3925 records at file:line; 0 mismatched
8 imported names could not be resolved to a module:
  src/deadline.rs: std::time::Duration
  src/error.rs: thiserror::Error
  src/model/document/order.rs: std::borrow::Cow
  src/output/envelope.rs: serde::Serialize
  src/output/writer.rs: serde::Serialize
  src/output/writer.rs: std::io::Write
```

The sample size is 151 because that is the size the layer's first build checked.
The exhaustive run reads all 3 236 records of the layer and makes the 689
endpoint assertions beside them — one per `MEMBER_OF` and `IMPLEMENTS` edge — so
the sample is the reproducible headline and the full run is the stronger claim.

Every unresolved name is an import from another crate, which is the stated
outcome and not a failure.

### That the check is not vacuous

A verifier that passes everything proves nothing, so the layer was perturbed in
three ways and read back each time. Each perturbation was made in memory,
against the records and never against the tree:

| Perturbation | Mismatches reported |
|---|---|
| none | `0` of 3 925 |
| every record's line moved by one | `3 125` — which is every record that carries a line, the 111 components having none |
| fifty names altered | exactly `50` |
| fifty visibilities flipped | exactly `50` |

Each figure is the number of records the perturbation could reach, and no more:
the check fails precisely what was broken and nothing beside it.

## Other invocations

```sh
scripts/knowledge/rust-layer.py --summary        # counts per label and predicate
scripts/knowledge/rust-layer.py --format json    # the same records, for a diff
scripts/knowledge/rust-layer.py --commit <hash> --date <YYYY-MM-DD>
```

`--summary` is the quickest way to see what a change to the tree did to the
layer. `--format json` is for a reader that wants to diff the desired set against
the live set rather than load statements — which is what `knowledge-model.md`
prescribes wherever an endpoint pair may already carry an edge of another type.

## The output format

One statement per line. A line beginning with `//` is a comment and an empty line
separates the sections, so a driver reads the file line by line and skips both.
Node statements come first, then edges; the header records the commit the run was
stamped with.

`rmp graph client` takes one statement per invocation, which is why the output is
one statement per line. **Running them is the `knowledge-authority` skill's job,
not this script's**, and the skill is answerable for reading the `counters` block
of each write and for reconciling records this scan no longer produces.

## Known limits

Stated so that a later reader does not mistake one for a defect:

- A single-line `enum E { A, B }` yields the type and not its variants. Variants
  are read from the enum's body, one per line, which is how this crate writes
  them.
- A macro that expands to an item declares nothing this scan can see. The crate
  declares no items through macros today.
- `use a::*` imports no name this scan can resolve, so an `impl` for a type
  reached only through a glob import keeps the writing file's module in its key.
  The crate writes no glob import outside test modules today.
- The scan reads `src/` and `tests/`. `benches/` is not scanned, because the
  repository does not carry that directory yet; adding it is a one-line change
  to `SCANNED`, and `Test.kind` already admits `bench`.
