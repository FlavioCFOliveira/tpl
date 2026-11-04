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
| `doc-layer.py` | The document-layer scanner. Emits that layer; verifies it against the tree — see [The document layer](#the-document-layer-doc-layerpy) |
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

## The document layer: `doc-layer.py`

```sh
scripts/knowledge/doc-layer.py
```

The second scanner re-derives the document half of the graph: every node and
edge `knowledge-model.md` defines that `rust-layer.py` does not emit. It keeps
the same contract. It emits and never writes; it speaks the model's vocabulary
without extending it; it stamps every element with `HEAD`, and reports on
standard error when a covered file has uncommitted changes; and its output has
the format described under *The output format*, with one `//` section per label
and per predicate.

```sh
scripts/knowledge/doc-layer.py --summary        # counts per label and predicate
scripts/knowledge/doc-layer.py --format json    # the same records, for a diff
scripts/knowledge/doc-layer.py --commit <hash> --date <YYYY-MM-DD>
scripts/knowledge/doc-layer.py --verify         # every record, derived twice
```

There is no `--sample`: the exhaustive check takes under a second.

### What it derives

| Label | Derived from |
|---|---|
| `File` | Every tracked file the coverage table below gives a kind. `title` and `status` come from a Markdown file's front matter and are absent where it has none; nothing else is read into the node |
| `Directory` | Every directory above a covered file; the root is `.` |
| `Requirement` | A definition site in `specification/`: an `FR`, `NFR` or `BR` bullet `- **ID**:`, a `WL` bullet `- **WL-NNN** —`, a `## UC-NNN — title` heading, an `OQ` row of the table headed `Id`, `Question`, and a bare `## DIV-NNN` heading |
| `Decision` | A `## OD-NN — title` heading in `docs/spec-technical/`, and the front-matter `id` of a record in `docs/adr/` |
| `Baseline` | A `## <date> — <subject>` entry of `BENCHMARKS.md` |
| `Crate` | The `[dependencies]` table of `Cargo.toml`, and that table alone |

| Predicate | Derived from |
|---|---|
| `CONTAINS` | The directory tree above the covered files |
| `DEFINES` | The definition site, carrying its 1-based `line` |
| `CITES` | An occurrence inside a definition block, from the block's owner, carrying `firstLine` and `path` |
| `REFERENCES` | An occurrence outside every definition block, from the file, carrying `firstLine` |
| `TARGETS` | The `**Target**:` field of a divergence record |
| `RELATED_TO` | A front-matter `related:` entry, resolved against the file's own directory |
| `DECLARES` | `BENCHMARKS.md` to each baseline it records |
| `DEPENDS_ON` | The component named after the package, `tpl`, to each crate, carrying `optional` |

The model fixes the set of `File.kind` values but not where each kind lives.
This table is that mapping; a tracked path no row matches is not covered.

| Path | `kind` |
|---|---|
| `specification/**/*.md` | `spec` |
| `docs/spec-technical/**/*.md` | `techspec` |
| `docs/adr/**/*.md`, its `README.md` included | `adr` |
| `CLAUDE.md`, `README.md` | `coordination` |
| `BENCHMARKS.md`, `benches/**` | `benchmark` |
| `scripts/mariadb/**`, `scripts/sqlite/**` | `fixture` |
| `src/**/*.rs` | `source` |
| `tests/**/*.rs` | `test` |
| `templates/**` | `template` |

Three rows reach past the layer last loaded: `scripts/sqlite/`, which its own
`README.md` names the SQLite fixture; `benches/`, which its own `README.md`
names the benchmark harness; and `templates/`, which does not exist yet.

### What it deliberately does not derive

- **Every other tracked file**: `examples/`, `skill/`, `vendor/`,
  `release-notes/`, `scripts/knowledge/`, `.github/`, `.cargo/`, and every root
  file but the three above. No kind in the model names them.
- **The contents of a `.pem` file.** It gets a node and nothing of it is opened:
  `knowledge-model.md` names the fixture's private key as the file its
  no-content rule exists for.
- **A `Component`.** The one `DEPENDS_ON` binds is the binary `rust-layer.py`
  emits. This script binds it with `MATCH` and never creates it.
- **A decision named inside a requirement's block.** The partition makes it a
  `CITES` from the requirement, and the model defines `CITES` only from a
  requirement to a requirement and from a decision to either. The occurrence is
  emitted under no other predicate, and `--verify` counts it. At `099dfba` there
  is one: `DIV-045` names `ADR-004` at `specification/upstream-divergences.md:1743`.
- **The removal of a property.** A node is upserted with the properties it has,
  so a `title` a file no longer declares stays on its node until the
  `knowledge-authority` skill reconciles it, as it does every record this scan
  no longer produces.

### How a key and a property are derived

- An identifier is written as found. `ordinal` is its trailing number as an
  integer, and `area` its middle segment, present for `FR`, `NFR` and `BR`
  only. A placeholder with no digits — `FR-X`, `OD-nn`, `ADR-NNN` — is not an
  identifier.
- `title` is taken verbatim, code spans included, from the source the model
  names: the `UC` heading after its em dash, the `OQ` row's *Question* cell,
  the `DIV` index's *Subject* cell, the `OD` heading after its em dash, and the
  `ADR`'s front-matter `title`. `FR`, `NFR`, `BR` and `WL` carry none.
- A divergence record's target is the code span its `**Target**:` field opens
  with; the rest of the field names a section of that file. A field opening
  with *both* names the two `coordination` files.
- A baseline's key is `<date>#<slug>`, the slug being the subject lower-cased,
  backticks dropped, and every run of other characters a single hyphen. Its
  `target` is every code span of the entry's *Target of record:* clause, in
  order and without repeats, joined by `; `.
- A crate's `version` is the requirement as declared, `source` is `crates.io`,
  and `features` is the declared list joined by `, `, absent where none is
  declared. `justification` is the crate's *Chosen because* cell in
  `docs/spec-technical/technology-stack.md`, verbatim. That column states why
  each crate was chosen; the *What it does here* column beside it is the
  crate's purpose, from which the model forbids inferring a justification, and
  it is never read.

### The partition

`CITES` and `REFERENCES` follow the rule `knowledge-model.md` states. A
definition block is:

- for `FR`, `NFR`, `BR` and `WL`, the bullet and every following line up to the
  next non-blank line at column 0;
- for `UC`, `DIV` and `OD`, the `##` section, up to the next heading of level 1
  or 2;
- for `OQ`, the row;
- for an `ADR`, the whole file below its front matter.

A line inside fenced code is read for identifiers, but it can neither end a
block nor open one. An identifier at its own definition site is the definition,
not an occurrence, and an occurrence of a block's owner inside that block is a
self-reference; neither produces an edge. Both edges keep the first occurrence
of their endpoint pair.

An `ADR`'s front-matter `requirements:` list sits above its block, so each entry
is a `REFERENCES` from the record's file, where the rule's wording places it. At
`099dfba` those lists yield 74 references, and the bodies of the records cite
all but two of the same requirements as well.

### Writing it

Every edge statement binds both endpoints with `MATCH`, `MERGE`s the
relationship alone, and re-binds it by type before it sets a property:

```cypher
MATCH (a:File {path: '…'}) MATCH (b:Requirement {id: '…'}) MERGE (a)-[:DEFINES]->(b) WITH DISTINCT a, b MATCH (a)-[e:DEFINES]->(b) SET e.gitCommit = '…', e.gitDate = '…', e.line = 151
```

A `MATCH` that binds nothing makes the statement a no-op, never a stub. The
re-bind matters because of the engine trap `knowledge-model.md` records: a
`MERGE` between a pair already joined by an edge of another type matches that
edge and creates nothing, and without the re-bind the `SET` would land on that
other edge. **The trap is live in this layer**: at `099dfba`, 278 `REFERENCES`
edges share their `(File, identifier)` pair with a `DEFINES` edge. Load those by
diffing `--format json` against the live set, as the model prescribes.

Every statement is checked against a ceiling of 1 MB before it is printed; the
longest at `099dfba` is 860 bytes.

### Verifying it

```sh
scripts/knowledge/doc-layer.py --verify
```

The verifier derives the whole layer a second time, with code that shares
nothing with the scan: its own identifier pattern, its own front-matter and
fence readers, its own definition-site patterns, and blocks walked forward from
each site rather than assigned by a state machine. It compares the two
derivations record by record, in both directions, and then reopens every
`CITES` and `REFERENCES` record at its `file:line` and confirms the line names
the target. A record emitted wrong, emitted twice, emitted with an endpoint this
layer does not declare, or not emitted at all, is one mismatch under its own
name. The one endpoint it accepts from outside the layer is the `Component` of
`DEPENDS_ON`, for which it confirms that `src/main.rs` exists.

Exit `0` when every record matched, `1` when any did not.

#### What it reported, on 2026-11-04 at `099dfba`

```
$ scripts/knowledge/doc-layer.py --verify
checked 15739 records at file:line; 0 mismatched
0 occurrences name an identifier nothing defines
1 occurrences inside a requirement's block name a decision, for which the model has no predicate

$ scripts/knowledge/doc-layer.py --summary
LABEL          NODES
Baseline       18
Crate          14
Decision       51
Directory      34
File           246
Requirement    1069

PREDICATE      EDGES
CITES          4437
CONTAINS       279
DECLARES       18
DEFINES        1120
DEPENDS_ON     14
REFERENCES     8124
RELATED_TO     238
TARGETS        77

total          1432 nodes, 14307 edges
```

The 15 739 records are the 1 432 nodes and 14 307 edges, each read once.

#### That the check is not vacuous

The layer was perturbed in memory and read back each time, against a copy of
the records and never against the tree. Each perturbation draws the records it
touches with a seeded generator, from records sorted by identity.

| Perturbation | Records touched | Mismatches reported |
|---|---|---|
| none | 0 | `0` |
| every `DEFINES` `line`, `CITES` `firstLine` and `REFERENCES` `firstLine` moved by one | 13 681 | `13 681` |
| fifty identifier titles altered | 50 | `50` |
| fifty identifier ordinals moved by one | 50 | `50` |
| fifty `File` kinds changed | 50 | `50` |
| fifty `File` titles removed | 50 | `50` |
| fifty `REFERENCES` dropped | 50 | `50` |
| fifty `CITES` given the wrong `path` | 50 | `50` |
| fifty `RELATED_TO` dropped | 50 | `50` |
| every `Baseline` target altered | 18 | `18` |
| every `Crate` version altered | 14 | `14` |
| every `DEPENDS_ON` made optional | 14 | `14` |

Each figure is the number of records the perturbation touched, and no more. The
scan itself was then broken in memory, to show that the second derivation does
not inherit the first one's logic:

| The scan altered so that | Mismatches reported |
|---|---|
| a bullet block ends on its own first line | `5 099`: 3 265 `CITES` and 1 834 `REFERENCES` |
| the `WL` family is not recognised | `120`: the three nodes and their `DEFINES`, 48 `CITES`, 65 `REFERENCES`, and the count of dangling occurrences |
| fenced code is not recognised | `0` |

The last row is not a pass. No fence in the tree at `099dfba` hides a heading or
a column-0 line inside a block, so the fence handling changes nothing today: it
is a guard, and the current tree does not exercise it.

#### Against the Rust layer

Every `File` and `Requirement` that `rust-layer.py` binds with `MATCH` has to be
a node this layer emits, or the edge that binds it is a silent no-op:

```sh
rust=$(mktemp) doc=$(mktemp)
scripts/knowledge/rust-layer.py --format json > "$rust"
scripts/knowledge/doc-layer.py --format json > "$doc"
python3 -I - "$rust" "$doc" <<'EOF'
import json, sys
rust, doc = (json.load(open(path, encoding="utf-8")) for path in sys.argv[1:])
declared = {(node["label"], node["key"]) for node in doc["nodes"]}
bound = {
    (end["label"], end["key"])
    for edge in rust["edges"]
    for end in (edge["source"], edge["target"])
    if end["label"] in ("File", "Requirement")
}
print(f"{len(bound)} endpoints bound; {len(bound - declared)} not declared")
EOF
rm -f "$rust" "$doc"
```

At `099dfba` it printed `664 endpoints bound; 0 not declared`: the 140 files of
`DECLARES` and `IMPLEMENTED_BY`, and the 524 requirements of `VERIFIES`.

### Known limits

- The 2026-09-11 entry of `BENCHMARKS.md` gives its target as "`aarch64` Linux
  … on an `aarch64-apple-darwin` host": an architecture and a host, not two
  target triples. Its `target` reads `aarch64; aarch64-apple-darwin`, which is
  the clause's code spans as written.
- A structure the rules cannot place stops the scan with exit `1`, naming the
  file and line: an identifier defined twice, a definition inside another's
  block, a divergence record with no target field or a target that is not a
  coordination file, a `related:` entry that is not a covered file, a baseline
  with no *Target of record*, and a dependency that is not from `crates.io`.
