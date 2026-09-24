# Worked examples

Four complete demonstrations that `tpl`'s commands compose into the thing the
tool exists to produce: an application's data layer, built from a database
schema, through the command line and nothing else.

`specification/examples.md` defines what a worked example is — `FR-EX-001` to
`FR-EX-009`, `BR-EX-001` and `BR-EX-002` — and `UC-013` in
`specification/use-cases.md` is the flow each one realises end to end. This file
is how a reader runs them.

**An example is correct when the code it renders compiles.** Not when `tpl`
exited `0`: a render succeeds precisely when the template evaluated, which is a
fact about the template and not about the file. Each example therefore ends in a
**compile gate** — the target language's own toolchain reading every rendered
file — and that verdict is the acceptance signal, per `FR-EX-009`.

---

## The four

| Directory | Language | Database driver | Compile gate | A reader installs |
|---|---|---|---|---|
| [`go-data-layer/`](go-data-layer/README.md) | Go | `github.com/go-sql-driver/mysql` | `go build ./...`, `go vet ./...` | Go 1.24 or newer |
| [`rust-data-layer/`](rust-data-layer/README.md) | Rust | `sqlx` 0.9 | `cargo build`, `cargo clippy -- -D warnings` | Rust 1.85 or newer |
| [`python-data-layer/`](python-data-layer/README.md) | Python | PyMySQL | `python -m compileall`, `mypy --strict` | Python 3.11 or newer, plus `mypy`, `PyMySQL`, `types-PyMySQL` |
| [`node-data-layer/`](node-data-layer/README.md) | Node.js | `mysql2` | `node --check`, `tsc --checkJs --noEmit` | Node.js 20 or newer, plus `npm install` in the example |

Each example's own README states its toolchain floor, the versions it was
verified at, and what its templates decided. None of that is repeated here.

The Rust example is excluded from the repository's Cargo workspace, in the root
`Cargo.toml`, so `cargo build` and `cargo clippy` at the repository root neither
compile the rendered crate nor fetch what it depends on.

---

## What the four share

`FR-EX-007` requires them to read the same schemas from the same server under
the same conditions, because the set is comparable only while one axis varies:
where two examples render different data layers, the difference must be
attributable to their templates and their type mappings, which is what the set
exists to compare.

| | |
|---|---|
| **One server** | The fixture's most recent supported series, `12.3`, on `127.0.0.1:13309`, over TLS at `verify-identity` |
| **Three schemas** | `sakila` (16 tables), `world` (3) and `freight` (17) — **36 tables and 362 columns** |
| **One driver** | [`_driver/`](_driver/README.md), the five-command workflow of `FR-EX-004` written once, in Python, for all four |

The driver holds the server, the schemas and the access, and asks each example
for one thing only: which template renders into which file. Everything else it
does — checking every exit code, keeping the two streams apart, redirecting a
render's bytes into a file, reading `can_read_catalogue` rather than inferring
it from exit `0` — is documented in its own README and is the same for all four.

### Why three schemas and not two

`sakila` and `world` are published datasets, so a reader arrives already knowing
what they contain and can judge a rendered data layer against a schema they
recognise. They are also not enough.

The model reports **39 distinct `data_type` values across the three schemas, and
25 of them are carried by `freight` alone** — a figure each of the four examples
arrived at independently, and each of their type-mapping tables lists. A
mapping written against the two published datasets would have been complete over
14 types and would never have met a spatial column, a `BIT`, an `INET6`, a
`UUID`, a generated column or a system-versioned table. `freight` is the
project's own schema and exists to carry exactly that complement, per
`FR-EX-006`.

---

## Running one

The fixture must be up and carrying the two published datasets, which are not in
its image. From the repository root:

```sh
cargo build --release                       # the binary the driver looks for first
./scripts/mariadb/up.sh 12.3                # the one server FR-EX-006 names
./scripts/mariadb/seed-datasets.sh 12.3     # load sakila and world, then verify
export FIXTURE_PW=tpl-root                  # the password; see below
```

Then any one of the four:

```sh
python3 examples/go-data-layer/run.py
python3 examples/rust-data-layer/run.py
python3 examples/python-data-layer/run.py

npm --prefix examples/node-data-layer install   # once, before the first Node run
python3 examples/node-data-layer/run.py
```

Each `run.py` takes no arguments. It renders the example's files and then runs
that example's compile gate, and its exit code is the verdict.

What the driver writes into `.tpl/.cfg` is the literal string `${FIXTURE_PW}`,
never the password, and no password is ever passed on a command line: `tpl`
expands the variable from the environment when it resolves the entry for a
connection. A value written into an argv is visible in the process table for the
life of the invocation, which is why
[`specification/security.md`](../specification/security.md) makes this more than
a stylistic preference.

When you are done:

```sh
./scripts/mariadb/down.sh 12.3
```

[`scripts/mariadb/README.md`](../scripts/mariadb/README.md) documents the
fixture, and its *published datasets* section documents `seed-datasets.sh` — what
it loads, what it verifies, and why loading the two schemas by hand is refused.
The driver script and `_driver` take no dependency beyond the Python standard
library; the packages in the table above are needed by the *rendered* code and by
the gate that reads it.

---

## What each one produces

The same 36 tables, four times over. Every file is one invocation of
`tpl render`, because `tpl render` writes to standard output and nowhere else —
`FR-RND-028` — so a rendered file is a redirection the caller performs, which is
`FR-EX-005`.

| Example | Rendered files | Lines | Committed under |
|---|---:|---:|---|
| `go-data-layer` | 111 | 20 049 | `out/` |
| `rust-data-layer` | 111 | 26 137 | `src/` |
| `python-data-layer` | 114 | 26 132 | `datalayer/` |
| `node-data-layer` | 114 | 25 001 | `src/` |

The rendered tree is committed in each case, so an example can be read without
being run. None of the four records a timestamp: `now` is the single documented
source of non-reproducibility, per `FR-CTX-030`, and a generated file carrying
one would produce a diff on every run for a reason that is not a change.

Where the files land differs by language and each example's README says why: a
Python module's name *is* its import path, a Rust module's path *is* its file's
path, and Go and Node.js constrain neither.

---

## The type mapping is the point of the exercise

`tpl` ships no `go_type`, `rust_type`, `python_type` or `js_type` filter and
never will. `FR-ENV-009` to `FR-ENV-011` say so and `BR-ENV-002` says why: a type
mapping is an opinion, and an opinion belongs to the project holding it. The
extension point the specification offers instead is a template macro, and these
four examples are the demonstration that it is usable.

Each example therefore carries its own `templates/<language>/_types.jinja`,
covering all 39 `data_type` values, per `FR-EX-008`. Nothing falls through: the
final branch of each macro calls `fail()`, so a type the macro has never met ends
the render with `65` and names the column.

**Each of the four proved that arm non-vacuous** — delete a branch, re-render the
table that needs it, and the render fails `65` naming the column — and each
README shows the failure it produced.

The four mappings disagree, which is the result worth having. A `DECIMAL` is a
`decimal.Decimal` in Python, a `string` in Go and in Node.js, and a rendered
adapter in Rust; a MariaDB `TIME` is a `datetime.timedelta`, a `string`, a
`string` and `sqlx`'s `MySqlTime`. Only the Rust example replaces a starter macro
rather than writing one from nothing, because `tpl init` writes one for Rust
alone, per `FR-PROJ-017`.

---

## What the examples found

The examples have a second purpose and they earned it. Exercising templates
against a real catalogue found three defects that no unit test would have, and
every one of them is in **binding** — the value going back to the server — rather
than in rendering. Each example's README carries the measurement.

- **`sqlx` decodes a `DECIMAL` into no standard type, and a spatial column into
  nothing at all.** Measured against this server over a battery of candidate
  types, `String` and `Vec<u8>` included. The Rust example therefore renders two
  adapters of its own.
- **An equality filter over a `BIT` column matches nothing**, on both the Python
  and the Node.js drivers. MariaDB compares a `BIT` with a binary string by
  converting the string to a number, and the bytes the driver returns are not a
  numeral. It is the one condition those rendered searches build that cannot
  succeed, and both examples say so.
- **`mysql2` parses a geometry column into plain objects that do not bind back.**
  An equality filter matched nothing, silently, and an insert would have written
  JSON text into a geometry column. The Node.js example converts through
  `ST_AsWKB` and `ST_GeomFromWKB` at both ends.

None of the three is visible to a render that exits `0`, and none of them to a
compile gate: all three are failures at the database, found by putting the
rendered code in front of one. They are what reading a real catalogue buys.

---

## Where the rest is written

| Question | Where |
|---|---|
| What a worked example is, and what makes one correct | [`specification/examples.md`](../specification/examples.md) |
| The flow, step by step, with its alternate flows | `UC-013`, in [`specification/use-cases.md`](../specification/use-cases.md) |
| The workflow, the access, and what the driver asserts | [`_driver/README.md`](_driver/README.md) |
| The server, the datasets, and how to operate the fixture | [`scripts/mariadb/README.md`](../scripts/mariadb/README.md) |
| One example's templates, type mapping, gate and known limits | that example's own `README.md` |

`BR-EX-002` maintains the four as one set: a change to the schemas of
`FR-EX-006`, or to the model a schema produces, reaches all four, and an example
left behind is a broken example rather than a stale one — which only its own
compile gate says.
