# Knowledge Model — tpl

This file is the canonical description of the **shape** of the `tpl` knowledge
graph: its label dictionary, its predicate dictionary, its constraints, and its
index recommendations. It carries nothing else.

**Form, never content.** The graph holds the information; this file holds only
the vocabulary needed to query it. No node listing, no inventory, no census, no
measured selectivity, and no example row presented as a project fact appears
here. Anything that would go stale when the graph changes — without the
vocabulary itself changing — is one statement away instead, and the entry names
that statement.

The live inventory of what is populated is always:

```cypher
MATCH (n) UNWIND labels(n) AS l RETURN l, count(*) AS c ORDER BY c DESC
MATCH ()-[e]->() RETURN type(e) AS edge, count(*) AS c ORDER BY c DESC
```

**Provenance is universal.** Every node and every edge carries `gitCommit`
(string, the full 40-character hash of the commit at which the element was last
confirmed) and `gitDate` (string, that commit's ISO date, `YYYY-MM-DD`). These
two are omitted from the per-entry property lists below; assume them everywhere.
The query that finds a violation:

```cypher
MATCH (n) WHERE n.gitCommit IS NULL OR n.gitDate IS NULL RETURN labels(n)[0], count(*)
MATCH ()-[e]->() WHERE e.gitCommit IS NULL OR e.gitDate IS NULL RETURN type(e), count(*)
```

**A definition may precede its data.** Several labels and predicates below are
defined but not yet populated. Defining a term costs the graph nothing and lets a
reader compose a correct statement before the data lands. Each such entry says
plainly that it is unpopulated and why; the census above is the authority on what
is populated *now*.

---

## Label dictionary

### `File`

One tracked source file the model covers.

| Property | Type | Meaning |
|---|---|---|
| **`path`** | string | **Identity.** Repository-relative path, e.g. `specification/cli-contract.md`. Never absolute, never prefixed with `./`. |
| `name` | string | Basename, e.g. `cli-contract.md`. |
| `kind` | string | One of `spec`, `fixture`, `benchmark`, `techspec`, `adr`, `source`, `test`, `template`. |
| `title` | string | The front-matter `title`. **Absent** where the file has no front matter. |
| `status` | string | The front-matter `status`. **Absent** where the file has no front matter. |

`title` and `status` are genuinely absent, not null, on a file without front
matter — `keys(f)` does not list them. Test either with `IS NULL`, which is true
in both cases, and never assume the key exists.

A `File` node carries the path and the declared metadata **only**. Nothing
derived from a file's *contents* is ever stored as a property — a rule that is
load-bearing for `scripts/mariadb/tls/server-key.pem`, which is a private key.

### `Directory`

A directory that contains tracked files the model covers, or another such
directory.

| Property | Type | Meaning |
|---|---|---|
| **`path`** | string | **Identity.** Repository-relative path. The repository root is the single character `.`. |
| `name` | string | Basename; `.` for the root. |

### `Requirement`

One stable identifier defined by the functional specification in
`specification/`. Every family of identifier the specification issues is a
`Requirement` — the label names the role (a stable, citable unit of
specification), not a grammatical category.

| Property | Type | Meaning |
|---|---|---|
| **`id`** | string | **Identity.** The identifier exactly as written, e.g. `FR-CLI-001`, `UC-007`, `OQ-042`. |
| `family` | string | One of `FR` (functional requirement), `NFR` (non-functional), `BR` (business rule), `UC` (use case), `OQ` (open question), `DIV` (upstream divergence), `WL` (reference workload). |
| `area` | string | The middle segment where the numbering scheme has one, e.g. `CLI` in `FR-CLI-001`. **Absent** for `UC`, `OQ`, `DIV` and `WL`, whose identifiers have no middle segment. |
| `ordinal` | **integer** | The trailing number, as an integer — `1`, not `'001'`. `MERGE (r:Requirement {ordinal: 1})` and `{ordinal: '1'}` bind different nodes. |
| `title` | string | A short subject, **only where the source supplies one**. Absent for `FR`, `NFR`, `BR` and `WL`, which are defined as prose bullets with no heading. |

Where `title` comes from, by family — the sources differ and the difference is
real, not cosmetic:

- `UC` — the text after the em dash on its `## UC-0NN — …` heading.
- `OQ` — the *Question* column of its row in the open-questions table.
- `DIV` — the *Subject* column of the divergence **index table**. The `## DIV-0NN`
  headings themselves are bare and supply no title, so the index table is the only
  source. A reader expecting the title on the heading will find nothing.

### `Baseline`

One recorded performance measurement in `BENCHMARKS.md`.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** `<date>#<slug>`, e.g. `2026-09-10#mariadb-driver-selection`. |
| `date` | string | ISO date the baseline was recorded, `YYYY-MM-DD`. |
| `subject` | string | What was measured or decided. |
| `target` | string | The target triple(s) the measurement was taken on. Multiple targets are separated by `; `. A baseline without a target is not a baseline, and figures from different targets are never compared. |

### `Component` — defined, not populated

An architectural unit of the implementation: the crate root, the binary, or a
module within it.

| Property | Type | Meaning |
|---|---|---|
| **`name`** | string | **Identity.** The module path, e.g. `tpl::mariadb::reader`. |
| `path` | string | Repository-relative path of the directory or file that realises it. |
| `kind` | string | One of `crate-root`, `binary`, `module`, `submodule`. |
| `role` | string | One sentence on what the component is responsible for. |

**Zero instances after the bootstrap task.** `Component` is one of the three
widened layers. It is populated once the architecture document is turned into
graph content, which the bootstrap task deliberately deferred to separate work.

### `Flow` — defined, not populated

An end-to-end path through the system: a pipeline (`schema` → context → render)
or a control path (project discovery, error mapping).

| Property | Type | Meaning |
|---|---|---|
| **`name`** | string | **Identity.** A stable, human-readable name. |
| `kind` | string | `pipeline` or `path`. |

**Zero instances after the bootstrap task**, for the same reason as `Component`.

### `FlowStep` — defined, not populated

One ordered step within a `Flow`.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** `<flow>#<ordinal>`, e.g. `render-pipeline#3`. |
| `ordinal` | **integer** | Position within the flow, from 1. |
| `title` | string | What the step does. |
| `exitCode` | **integer** | The process exit code this step produces when it fails. **Absent** where the step cannot fail. |

**Zero instances after the bootstrap task**, for the same reason as `Component`.

### `Crate` — defined, not populated

A Rust dependency, or the standard library.

| Property | Type | Meaning |
|---|---|---|
| **`name`** | string | **Identity.** The crate name as `Cargo.toml` spells it. |
| `version` | string | The version requirement as declared. |
| `source` | string | `crates.io` or `std`. |
| `features` | string | The declared feature list, comma-separated. |
| `justification` | string | Why the crate earns its place, per the project's dependency budget. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet —
no `Cargo.toml`, no `src/`, and no `.rs` file.

### `Function` — defined, not populated

A free function.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** Fully qualified, e.g. `tpl::render::build_environment`. |
| `name` | string | The bare name. |
| `module` | string | The module path that owns it. |
| `visibility` | string | One of `pub`, `pub(crate)`, `pub(super)`, `private`. |
| `file` | string | Repository-relative path of the file that declares it. |
| `line` | **integer** | Declaration line, 1-based. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet.

### `Type` — defined, not populated

A struct, enum, union, or type alias.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** Fully qualified, e.g. `tpl::model::Table`. |
| `name` | string | The bare name. |
| `module` | string | The module path that owns it. |
| `kind` | string | One of `struct`, `enum`, `union`, `alias`. |
| `visibility` | string | As for `Function`. |
| `file` | string | Repository-relative declaring file. |
| `line` | **integer** | Declaration line, 1-based. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet.

### `Trait` — defined, not populated

A trait declaration. Identical to `Type` except that it has no `kind`.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** Fully qualified. |
| `name` | string | The bare name. |
| `module` | string | The module path that owns it. |
| `visibility` | string | As for `Function`. |
| `file` | string | Repository-relative declaring file. |
| `line` | **integer** | Declaration line, 1-based. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet.

### `Method` — defined, not populated

A method in an inherent or trait `impl`.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** `Type::method`, fully qualified on the type. |
| `name` | string | The bare name. |
| `owner` | string | `key` of the owning `Type`. |
| `traitKey` | string | `key` of the `Trait` being implemented. **Absent** for an inherent `impl`. |
| `visibility` | string | As for `Function`. |
| `file` | string | Repository-relative declaring file. |
| `line` | **integer** | Declaration line, 1-based. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet.

### `Test` — defined, not populated

One test, benchmark, or doc test.

| Property | Type | Meaning |
|---|---|---|
| **`key`** | string | **Identity.** `file::name`, e.g. `tests/cli_help.rs::help_json_is_stable`. |
| `name` | string | The test function name. |
| `kind` | string | One of `unit`, `integration`, `doc`, `bench`. |
| `file` | string | Repository-relative declaring file. |
| `line` | **integer** | Declaration line, 1-based. |

**Zero instances after the bootstrap task**: the repository holds no Rust yet.

---

## Predicate dictionary

One row per endpoint pair. The same type against a different pair is a different
assertion and is listed separately.

### Populated

| Predicate | Endpoints | Asserts | Edge properties |
|---|---|---|---|
| `CONTAINS` | `(Directory)→(Directory)` | Directory nesting: the target sits directly inside the source. | — |
| `CONTAINS` | `(Directory)→(File)` | The file lives directly in this directory, not in a descendant of it. | — |
| `DECLARES` | `(File)→(Baseline)` | This file is the single site that declares the baseline. | — |
| `DEFINES` | `(File)→(Requirement)` | This file is the **single canonical definition site** of the identifier. Exactly one per `Requirement`. | `line` (**integer**) — the 1-based line of the definition. |
| `CITES` | `(Requirement)→(Requirement)` | The source requirement's own prose names the target identifier. **This is the impact-analysis edge**: it answers "what else must be revisited if this requirement changes?". | `firstLine` (**integer**) — first line of the citation; `path` (string) — file the citation was read in. |
| `REFERENCES` | `(File)→(Requirement)` | The file names the identifier **outside** any definition block — narrative prose, an index table, a fixture comment. | `firstLine` (**integer**). |
| `RELATED_TO` | `(File)→(File)` | The source file's front-matter `related:` list declares the target adjacent. Directed and **not** automatically reciprocal: traverse both ways when adjacency in either direction matters. | — |

`CITES` and `REFERENCES` **partition** every non-definition occurrence of an
identifier, and the partition rule is one line: an occurrence inside a
definition block becomes `CITES`, from the requirement that owns the block; an
occurrence outside every definition block becomes `REFERENCES`, from the file.
A definition block is the bullet and its indented continuation for `FR`, `NFR`,
`BR` and `WL`; the whole `##` section for `UC` and `DIV`; the single table row for
`OQ`. An occurrence naming the very requirement whose block it sits in is a
self-reference and produces no edge.

Both edges are deduplicated on their endpoint pair and keep the **first**
occurrence's line, which is why the property is `firstLine` and not `line`.

**Why `DECLARES` and not `CONTAINS` for the baseline.** `CONTAINS` is reserved
for structural containment in the filesystem tree, where both endpoints are real
paths. A `Baseline` is not a filesystem child of `BENCHMARKS.md`; it is an
element the file *declares*, in exactly the sense in which a source file declares
a `Function` or a `Type`. Using `DECLARES` keeps one predicate for
"this file is the single declaration site of this element" across the whole
model, and keeps `CONTAINS` unambiguously about the tree.

### Defined, not populated

Every predicate below has **zero instances after the bootstrap task**. The two
reasons are the same as for the labels: the `Component`/`Flow` layers await the
architecture document being turned into graph content, and the Rust-symbol
predicates await the existence of Rust in the repository.

| Predicate | Endpoints | Asserts | Edge properties | Why unpopulated |
|---|---|---|---|---|
| `SATISFIES` | `(Component)→(Requirement)` | This component satisfies the requirement. **The requirement-satisfaction layer**: it answers "which component satisfies `FR-X`?" and, inverted, "is `FR-X` implemented at all?". | `kind` (string) — `full` or `partial` | no `Component` data yet |
| `IMPLEMENTED_BY` | `(Component)→(File)` | The component is realised by this file. | — | no `Component` data yet |
| `DEPENDS_ON` | `(Component)→(Component)` | The source needs the target to function. | `optional` (**boolean**) | no `Component` data yet |
| `DEPENDS_ON` | `(Component)→(Crate)` | The component depends on this external crate. | `optional` (**boolean**) | no `Component` or `Crate` data yet |
| `HAS_STEP` | `(Flow)→(FlowStep)` | The step belongs to this flow. | — | no `Flow` data yet |
| `NEXT` | `(FlowStep)→(FlowStep)` | Step ordering: the target follows the source. | — | no `Flow` data yet |
| `PERFORMED_BY` | `(FlowStep)→(Component)` | The component carries out this step. | — | no `Flow` data yet |
| `SPECIFIED_BY` | `(Flow)→(Requirement)` | The requirement specifies this flow. | — | no `Flow` data yet |
| `DECLARES` | `(File)→(Function\|Type\|Trait\|Method\|Test)` | This file is the declaration site of the symbol. Same assertion as the populated `(File)→(Baseline)` row. | `line` (**integer**) | no Rust in the repository |
| `MEMBER_OF` | `(Method)→(Type)` | The method belongs to this type. | — | no Rust in the repository |
| `IMPLEMENTS` | `(Type)→(Trait)` | The type implements this trait. | — | no Rust in the repository |
| `CALLS` | `(Function\|Method)→(Function\|Method)` | The source calls the target. | — | no Rust in the repository |
| `VERIFIES` | `(Test)→(Requirement)` | The test verifies the requirement. | — | no Rust in the repository |

---

## Constraints

Every label with an identity property carries **both** a `IS UNIQUE` and a
`IS NOT NULL` constraint on it. That is what "identity" means here, and this
engine can hold the line: `IS UNIQUE` is genuinely enforced, so a violating write
is rejected with a non-zero exit rather than silently accepted.

This matters more than tidiness. The engine's `MERGE` over a **pattern**
re-creates every node in that pattern unless the whole pattern already matches,
which produces duplicate stub nodes without any error. The UNIQUE constraint
turns that silent corruption into a hard failure at the moment it happens, and is
the strongest defence the model has against it. Declare the whole schema **before
the first write**, never after.

Read the engine's current view with:

```cypher
SHOW CONSTRAINTS
```

The DDL, for every identity property in the model:

```cypher
CREATE CONSTRAINT file_path_uniq           IF NOT EXISTS FOR (x:File)        REQUIRE x.path IS UNIQUE;
CREATE CONSTRAINT file_path_notnull        IF NOT EXISTS FOR (x:File)        REQUIRE x.path IS NOT NULL;
CREATE CONSTRAINT directory_path_uniq      IF NOT EXISTS FOR (x:Directory)   REQUIRE x.path IS UNIQUE;
CREATE CONSTRAINT directory_path_notnull   IF NOT EXISTS FOR (x:Directory)   REQUIRE x.path IS NOT NULL;
CREATE CONSTRAINT requirement_id_uniq      IF NOT EXISTS FOR (x:Requirement) REQUIRE x.id IS UNIQUE;
CREATE CONSTRAINT requirement_id_notnull   IF NOT EXISTS FOR (x:Requirement) REQUIRE x.id IS NOT NULL;
CREATE CONSTRAINT baseline_key_uniq        IF NOT EXISTS FOR (x:Baseline)    REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT baseline_key_notnull     IF NOT EXISTS FOR (x:Baseline)    REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT component_name_uniq      IF NOT EXISTS FOR (x:Component)   REQUIRE x.name IS UNIQUE;
CREATE CONSTRAINT component_name_notnull   IF NOT EXISTS FOR (x:Component)   REQUIRE x.name IS NOT NULL;
CREATE CONSTRAINT flow_name_uniq           IF NOT EXISTS FOR (x:Flow)        REQUIRE x.name IS UNIQUE;
CREATE CONSTRAINT flow_name_notnull        IF NOT EXISTS FOR (x:Flow)        REQUIRE x.name IS NOT NULL;
CREATE CONSTRAINT flowstep_key_uniq        IF NOT EXISTS FOR (x:FlowStep)    REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT flowstep_key_notnull     IF NOT EXISTS FOR (x:FlowStep)    REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT crate_name_uniq          IF NOT EXISTS FOR (x:Crate)       REQUIRE x.name IS UNIQUE;
CREATE CONSTRAINT crate_name_notnull       IF NOT EXISTS FOR (x:Crate)       REQUIRE x.name IS NOT NULL;
CREATE CONSTRAINT function_key_uniq        IF NOT EXISTS FOR (x:Function)    REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT function_key_notnull     IF NOT EXISTS FOR (x:Function)    REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT type_key_uniq            IF NOT EXISTS FOR (x:Type)        REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT type_key_notnull         IF NOT EXISTS FOR (x:Type)        REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT trait_key_uniq           IF NOT EXISTS FOR (x:Trait)       REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT trait_key_notnull        IF NOT EXISTS FOR (x:Trait)       REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT method_key_uniq          IF NOT EXISTS FOR (x:Method)      REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT method_key_notnull       IF NOT EXISTS FOR (x:Method)      REQUIRE x.key IS NOT NULL;
CREATE CONSTRAINT test_key_uniq            IF NOT EXISTS FOR (x:Test)        REQUIRE x.key IS UNIQUE;
CREATE CONSTRAINT test_key_notnull         IF NOT EXISTS FOR (x:Test)        REQUIRE x.key IS NOT NULL;
```

Constraints on an unpopulated label are declared all the same: they cost nothing
and they are in force before the first row of that label ever lands, which is the
only moment at which they can prevent the duplication trap.

### The engine's limits

Only these two kinds exist, and only on a **single** node property. Composite
`NODE KEY`, `ASSERT exists(...)` and type constraints (`IS :: STRING`) all fail
with an opaque internal error, so the model expresses no rule that needs them.
Nothing here declares a relationship constraint, because the engine has none.

### Integrity queries — the rules the engine cannot express

Three model rules are not expressible as engine constraints. Each is stated here
with the query that measures the gap; the count that query returns is graph
content and lives only in the graph.

1. **Every `Requirement` has exactly one `DEFINES` edge.** A requirement with
   zero has no definition site; one with two has contradictory definitions.

   ```cypher
   MATCH (r:Requirement) OPTIONAL MATCH (:File)-[d:DEFINES]->(r)
   WITH r, count(d) AS n WHERE n <> 1 RETURN r.id, n
   ```

2. **No stub nodes** — an identity property set while the descriptive ones are
   null, which is the signature of the pattern-`MERGE` trap.

   ```cypher
   MATCH (n:File)        WHERE n.name IS NULL OR n.kind IS NULL     RETURN count(n)
   MATCH (n:Directory)   WHERE n.name IS NULL                       RETURN count(n)
   MATCH (n:Requirement) WHERE n.family IS NULL OR n.ordinal IS NULL RETURN count(n)
   MATCH (n:Baseline)    WHERE n.date IS NULL OR n.subject IS NULL   RETURN count(n)
   ```

3. **No duplicate identity**, per label, on the declared key. Redundant while the
   UNIQUE constraints hold, and the check that proves they do:

   ```cypher
   MATCH (n:Requirement) WHERE n.id IS NOT NULL
   WITH n.id AS v, count(*) AS c WHERE c > 1 RETURN count(v) AS dup_values, sum(c) AS dup_nodes
   ```

---

## Index recommendations

**None. No standalone index is recommended for this model, and the reason is
evidence, not omission.**

The engine's index is single-property, node-only and hash — therefore
equality-only. A range predicate (`>`, `<`) ignores it and falls back to a label
scan. An index is worth recommending exactly where this skill issues an
**equality lookup on one property**, which is what every identity lookup in the
model is.

Every one of those lookups is **already served**. A UNIQUE constraint creates its
own backing index, which `SHOW INDEXES` reports as `__uniq__<Label>.<property>`
with empty `labelsOrTypes` and `properties` — a reporting quirk, not a stray
index, and never something to drop. Because the model declares a UNIQUE
constraint on every identity property, every identity property already has a
usable index, and a second index on the same property adds write cost and storage
for no read benefit.

### How this was proven, and how to re-prove it

`EXPLAIN` distinguishes the two outcomes: `NodeByLabelScan` + `Filter` means no
usable index; `NodeByIndexSeek` means one is in use.

```cypher
EXPLAIN MATCH (r:Requirement {id:'FR-CLI-001'}) RETURN r   -- NodeByIndexSeek
EXPLAIN MATCH (r:Requirement {family:'FR'})     RETURN r   -- NodeByLabelScan + Filter
```

The first is an identity property, covered by its UNIQUE constraint's backing
index. The second is a descriptive property with no constraint and no index, and
it is the **control** that proves the planner really does distinguish the two —
without it, a `NodeByIndexSeek` could not be attributed to anything.

`PROFILE` measures rather than predicts, and `dbHits` is the figure that settles
it: a seek on an identity property costs one hit, while the unindexed control
pays one hit per node of the label.

```cypher
PROFILE MATCH (r:Requirement {id:'FR-CLI-001'}) RETURN r
```

Read `timeNs` from a single run with suspicion: the run immediately after a
`DROP INDEX` is cold and reports a time an order of magnitude above the steady
state. Repeat before concluding anything from it.

### When to revisit

Add an index only for an **equality lookup on a non-identity property**, on a
label large enough for the scan to hurt, and only after `EXPLAIN` shows the plan
change. The candidates to re-measure when the Rust layers are populated:

```cypher
MATCH (n:Function) WHERE n.name IS NOT NULL RETURN count(n) AS tot, count(DISTINCT n.name) AS dv
MATCH (n:Test)     WHERE n.name IS NOT NULL RETURN count(n) AS tot, count(DISTINCT n.name) AS dv
```

A composite lookup cannot be indexed — composite indexes are unsupported — so
where a lookup matches on two properties, index the more selective single one and
let the engine filter the rest. There is no `ALTER INDEX`: changing one is a
`DROP` then a `CREATE`, two invocations, and the index is absent between them.
