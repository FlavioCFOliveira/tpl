# `go-data-layer` — a Go data layer, built from three schemas through the command line

This is the Go member of the four worked examples `FR-EX-001` requires. It reads
`sakila`, `world` and `freight` from one MariaDB server and renders a Go package
per schema: a row struct, its primary key, a repository interface and a
`database/sql` implementation of that interface, for every one of the 36 tables.

Its acceptance signal is not that `tpl` exited `0`. It is that `go build` and
`go vet` accept every file it rendered, per `FR-EX-009`, and the example's own
script is what runs them.

```sh
export FIXTURE_PW=tpl-root
python3 examples/go-data-layer/run.py
```

## What it produces

| Path | Rendered from | Count |
|---|---|---|
| `out/<schema>/schema.go` | `go/schema` | 3, one per schema |
| `out/<schema>/<table>.go` | `go/struct` | 36, one per table |
| `out/<schema>/<table>_repository.go` | `go/repository` | 36 |
| `out/<schema>/<table>_sql.go` | `go/sql` | 36 |

111 files, from 111 invocations of `tpl render`. One file is one invocation,
because `tpl render` writes to standard output and nowhere else — `FR-RND-028`,
and `FR-EX-005` is the consequence.

`out/` is committed, so that the example can be read without being run.

### The package

`schema.go` is rendered once per schema with the whole database in context and
no object flag, per `FR-RND-006`. It carries the package documentation, the
provenance of the read, `Open`, the two sentinel errors, the ordering
`Direction`, and `Repositories`, which is one repository per table over one
pool:

```go
db, err := freight.Open(config)          // a *mysql.Config the caller owns
repositories := freight.New(db)          // one repository per table
```

### The interface

Every table's repository declares the same five methods. `Actor` stands for the
table's Go name, and every method takes a `context.Context` as its first
argument:

| Method | Signature after `ctx` | What it does |
|---|---|---|
| `Create` | `(row *Actor) error` | Inserts, naming neither generated columns nor the auto-increment key. Where the table has one, the key the server assigned is written back into `row` |
| `Read` | `(key ActorKey) (*Actor, error)` | Selects one row by its primary key, simple or composite. Returns an error wrapping `ErrNotFound` when there is none |
| `Update` | `(row *Actor) (int64, error)` | Writes every column that is neither generated nor part of the key, and reports the rows affected |
| `Delete` | `(key ActorKey) (int64, error)` | Removes one row by its primary key, and reports the rows affected |
| `Search` | `(filter ActorFilter, order []ActorOrder, limit, offset int) ([]Actor, error)` | The advanced search |

`ActorFilter` carries one field per column for equality, a `…From` and a `…To`
for every numeric and temporal column, a `…Like` for every textual column, and a
`…IsNull` for every nullable one. Each is optional, and the conditions that are
set are joined with `AND`. Which columns get which is decided by the registered
tests `numeric`, `temporal`, `textual` and `nullable` of `FR-ENV-041`.

`ActorOrder` is a column and a direction. The column is an `ActorColumn`, whose
constants are the table's own columns, and the implementation resolves it
through a closed `switch` before it reaches the statement: an ordering that
names anything else is refused with `ErrUnknownColumn` rather than interpolated.
No value a caller supplies is ever part of a statement's text.

## The type mapping

`tpl` ships no `go_type` filter and never will: `FR-ENV-010` says so and
`BR-ENV-002` says why — a type mapping is an opinion, and an opinion belongs to
the project holding it. The opinion is therefore a macro of this example,
`templates/go/_types.jinja`, in the form `FR-ENV-011` fixes.

It is obliged to cover every value of `data_type` the model reports for a column
of the three schemas, per `FR-EX-008`. That is **39 values over 362 columns**,
and all 39 are branched on. Nothing falls through: the final branch calls
`fail()`, so a type the macro has never met ends the render with `65` and names
the column. Removing the `uuid` branch and re-rendering `freight.audit_event`
produces exactly that, which is how the guard was shown not to be vacuous.

| `data_type` | Go type | Only in `freight` |
|---|---|:-:|
| `tinyint` | `int8`, `uint8` when unsigned | |
| `smallint` | `int16`, `uint16` when unsigned | |
| `mediumint` | `int32`, `uint32` when unsigned | |
| `int` | `int32`, `uint32` when unsigned | |
| `bigint` | `int64`, `uint64` when unsigned | • |
| `decimal` | `string` | |
| `float` | `float32` | • |
| `double` | `float64` | • |
| `bit` | `[]byte` | • |
| `char` | `string` | |
| `varchar` | `string` | |
| `tinytext` | `string` | • |
| `text` | `string` | |
| `mediumtext` | `string` | • |
| `longtext` | `string` | • |
| `enum` | `string` | |
| `set` | `string` | |
| `binary` | `[]byte` | • |
| `varbinary` | `[]byte` | • |
| `tinyblob` | `[]byte` | • |
| `blob` | `[]byte` | |
| `mediumblob` | `[]byte` | • |
| `longblob` | `[]byte` | • |
| `date` | `time.Time` | • |
| `datetime` | `time.Time` | |
| `timestamp` | `time.Time` | |
| `time` | `string` | • |
| `year` | `uint16` | |
| `inet4` | `string` | • |
| `inet6` | `string` | • |
| `uuid` | `string` | • |
| `geometry` | `[]byte` | • |
| `geometrycollection` | `[]byte` | • |
| `linestring` | `[]byte` | • |
| `multilinestring` | `[]byte` | • |
| `multipoint` | `[]byte` | • |
| `multipolygon` | `[]byte` | • |
| `point` | `[]byte` | • |
| `polygon` | `[]byte` | • |

**25 of the 39 are carried by `freight` alone**, which is why `FR-EX-006` puts a
third schema beside the two published ones. A mapping written against `sakila`
and `world` would have been complete over 14 types and would never have met a
spatial column, a `BIT`, an `INET6`, a `UUID`, a generated column or a
system-versioned table.

Four of the mappings are opinions worth stating outright:

- **`decimal` is a `string`.** It is the only mapping in the standard library
  that loses nothing — `float64` cannot hold `decimal(38,10)` — and it is what
  the driver returns for the type. A project wanting arithmetic replaces one
  branch of one macro, which is the whole reason the mapping lives here.
- **`time` is a `string`.** A MariaDB `TIME` is a signed interval of up to
  ±838:59:59. It is not a time of day and does not fit `time.Time`.
- **`inet4`, `inet6` and `uuid` are `string`.** Observed against the fixture
  server: the three arrive as character data — `ColumnType.DatabaseTypeName`
  reports `CHAR` — carrying their textual presentation.
- **A JSON column has no branch**, because the model never reports one: MariaDB
  reports it as `longtext`, per `FR-CAT-038`.

### Nullability

A nullable column is not the same Go type as a non-nullable one:

| Column | Go field |
|---|---|
| `first_name varchar(45) NOT NULL` | `FirstName string` |
| `changed_column varchar(64) NULL` | `ChangedColumn sql.Null[string]` |
| `client_ipv6 inet6 NULL` | `ClientIpv6 sql.Null[string]` |
| `page_preview blob NULL` | `PagePreview sql.Null[[]byte]` |

`sql.Null[T]` is the standard library's own answer, and it puts the difference
in the type rather than in a comment: it scans a `NULL` without erroring, it is
a `driver.Valuer`, and `Valid` is the one question a caller answers before
reading `V`. Verified against the fixture: `sql.Null[string]`,
`sql.Null[uint16]` and `sql.Null[[]byte]` each scan a real column of `freight`,
and `sql.Null[uint16]` is accepted as a query parameter.

The filter fields are the same type made nilable — a pointer, or the slice's own
nil where the value is already a slice.

## The compile gate

`run.py` runs both of these over the module, from the example's own directory:

```
go build ./...
go vet ./...
```

A missing toolchain is a failure that names what is missing, never a skip: the
gate's verdict is the example's acceptance signal, so a gate that did not run
has produced no verdict. The rendered package is also `gofmt`-clean, which is
not part of the gate but is what makes the committed output stable under an
editor.

The module is `tpl.example/go-data-layer`, and its one dependency is
`github.com/go-sql-driver/mysql`, which `schema.go` uses for `Open`. `go.mod`
and `go.sum` are committed; the first run needs that module in the module cache
or a network to fetch it.

## Prerequisites

| | |
|---|---|
| The binary | `cargo build --release` |
| The server | `./scripts/mariadb/up.sh`, then `./scripts/mariadb/seed-datasets.sh 12.3` |
| The password | `export FIXTURE_PW=tpl-root` |
| Go | 1.24 or newer — the floor the driver's own `go.mod` sets. `sql.Null[T]`, which the rendered code uses for every nullable column, needs 1.22 |

The server, the three schemas and the entries are the driver's, not this
example's: [`_driver/README.md`](../_driver/README.md) documents them, and
`FR-EX-007` is why all four examples read them the same way.

## What this example does not do

- **It writes nothing to the database.** `tpl` cannot, per the read-only
  invariant, and neither does the example: it renders files and compiles them.
- **It reaches no library interface of `tpl`.** Five commands and their exit
  codes, per `FR-EX-004`, which is the surface the primary consumer has.
- **It records no timestamp.** `now` is the single documented source of
  non-reproducibility, per `FR-CTX-030`, and a generated file carrying one
  produces a diff on every run for a reason that is not a change. Provenance —
  the server, its series, its standing and the version of `tpl` — is rendered
  once per package into `schema.go`.

## Known limits

Each is a property of this example's templates, not of `tpl`, and each is
visible because the compile gate is what would report it:

- **Identifiers are sanitised, not proved.** `FR-ENV-031` passes a character
  outside `A-Za-z` through the naming filters unchanged, so `pascal` renders the
  column `back`tick` as `Back`tick`, which is an identifier in no language. The
  macro replaces the ASCII punctuation a quoted MariaDB identifier may carry
  with an underscore, and `freight.legacy_edi_field` is the table that exercises
  it: `back`tick`, `space in name`, `select`, `posição` and `Mixed Case Column`
  arrive as `BackTick`, `SpaceInName`, `Select`, `Posição` and
  `MixedCaseColumn`. A name beginning with a digit, and two names whose
  sanitised forms collide, would still render code the Go compiler refuses — and
  that is when the gate reports it.
- **An acronym loses its case.** `actor_id` becomes `ActorId`, not `ActorID`.
  That is what `FR-ENV-033`'s `pascal` does, and its own *accepted cost* clause
  records it; buying it back would need an initialism list, which is a second
  opinion to maintain and another way for two columns to collide.
- **A table with no primary key is refused.** All 36 have one. A table without
  one ends the render with `65` and says why, because `Read`, `Update` and
  `Delete` would have no key to address a row by.
