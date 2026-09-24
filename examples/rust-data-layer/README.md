# `rust-data-layer` — a Rust data layer, built from three schemas through the command line

This is the Rust member of the four worked examples `FR-EX-001` requires. It
reads `sakila`, `world` and `freight` from one MariaDB server and renders one
Rust module per schema: a row, its primary key, the columns an ordering may
name, a repository trait and an `sqlx` implementation of that trait, for every
one of the 36 tables.

Its acceptance signal is not that `tpl` exited `0`. It is that `cargo build` and
`cargo clippy -- -D warnings` accept every file it rendered, per `FR-EX-009`,
and the example's own script is what runs them.

```sh
export FIXTURE_PW=tpl-root
python3 examples/rust-data-layer/run.py
```

## What it produces

| Path | Rendered from | Count |
|---|---|---|
| `src/<schema>.rs` | `rust/schema` | 3, one per schema |
| `src/<schema>/<table>.rs` | `rust/struct` | 36, one per table |
| `src/<schema>/<table>_repository.rs` | `rust/repository` | 36 |
| `src/<schema>/<table>_sqlx.rs` | `rust/sqlx` | 36 |

111 files and 26,000 lines, from 111 invocations of `tpl render`. One file is one
invocation, because `tpl render` writes to standard output and nowhere else —
`FR-RND-028`, and `FR-EX-005` is the consequence.

`src/` is committed, so that the example can be read without being run.

### Why the rendered files are the crate's own `src/`

A Rust module's path is its file's path: `pub mod sakila;` resolves to
`src/sakila.rs`, and that module's children to `src/sakila/`. Rendering into a
directory of its own would oblige every module declaration to carry a `#[path]`
attribute, because a `#[path]`-loaded module resolves its children against the
*declaring* file's directory and not its own. The rendered tree is therefore
`src/`, and `src/lib.rs` is the one Rust file this example does not render: it
carries the crate's attributes — `#![forbid(unsafe_code)]` among them — and
names the three schema modules. `run.py` removes everything else under `src/`
before each run, so a table dropped from a schema cannot leave a file behind
that still compiles.

### The module

`src/<schema>.rs` is rendered once per schema with the whole database in context
and no object flag, per `FR-RND-006`. It carries the module documentation, the
provenance of the read, `SCHEMA`, `connect`, the `Error` enum, the ordering
`Direction`, the two column adapters the type mapping names, and `Repositories`,
which is one implementation per table over one pool:

```rust
let pool = freight::connect(options).await?;      // MySqlConnectOptions the caller owns
let repositories = freight::Repositories::new(&pool);
```

Each table's public items are re-exported there, so a caller writes
`freight::Vessel` rather than `freight::vessel::Vessel`.

### The trait

Every table's repository declares the same five methods. `Actor` stands for the
table's Rust name, and every method returns
`impl Future<Output = Result<…, Error>> + Send`:

| Method | Signature after `&self` | What it does |
|---|---|---|
| `create` | `(row: &mut Actor) -> Result<(), Error>` | Inserts, naming neither generated columns nor the auto-increment key. Where the table has one, the key the server assigned is written back into `row` |
| `read` | `(key: &ActorKey) -> Result<Actor, Error>` | Selects one row by its primary key, simple or composite. Returns `Error::NotFound` when there is none |
| `update` | `(row: &Actor) -> Result<u64, Error>` | Writes every column that is neither generated nor part of the key, and reports the rows affected |
| `delete` | `(key: &ActorKey) -> Result<u64, Error>` | Removes one row by its primary key, and reports the rows affected |
| `search` | `(filter: ActorFilter, order: &[ActorOrder], limit: Option<u64>, offset: Option<u64>) -> Result<Vec<Actor>, Error>` | The advanced search |

`ActorFilter` carries one `Option` field per column for equality, a `…_from` and
a `…_to` for every numeric and temporal column, a `…_like` for every textual
one, and a `…_is_null` for every nullable one. Each is optional, and the
conditions that are set are joined with `AND`. Which columns get which is
decided by the registered tests `numeric`, `temporal`, `textual` and `nullable`
of `FR-ENV-041`, whose memberships `FR-ENV-046` fixes. `Default` is the filter
that matches every row, and `is_empty` is how the search asks whether to write a
`WHERE` clause at all.

`ActorOrder` is a column and a direction. The column is an `ActorColumn`, an
enumeration of the table's own columns, and `as_sql` resolves it through a closed
`match` into a quoted identifier this example's template wrote. A caller holding
a column name as text reaches the enumeration through `FromStr`, which refuses
every name the table does not carry with `Error::UnknownColumn`. **No value a
caller supplies is ever part of a statement's text.**

The five fixed statements are `&'static str` constants, which is what `sqlx` 0.9
requires of them: `sqlx::query` takes `impl SqlSafeStr`, and that trait is
implemented for `&'static str` and not for a `String` built at run time. The
search, whose shape depends on the filter, is therefore built with
`QueryBuilder` — the interface the same rule points a caller at — and every
value it carries is a bound parameter.

## The type mapping

`tpl` ships no `rust_type` filter and never will: `FR-ENV-009` says so and
`BR-ENV-002` says why — a type mapping is an opinion, and an opinion belongs to
the project holding it. The opinion is therefore a macro of this example,
`templates/rust/_types.jinja`, in the form `FR-ENV-011` fixes. `tpl init` writes
a starter macro at that exact path, per `FR-PROJ-017`, and this example replaces
it outright, which is what `FR-EX-008` allows and what makes the Rust example the
one that exercises replacement rather than addition.

It is obliged to cover every value of `data_type` the model reports for a column
of the three schemas, per `FR-EX-008`. That is **39 values over 362 columns**,
and all 39 are branched on. Nothing falls through: the final branch calls
`fail()`, so a type the macro has never met ends the render with `65` and names
the column. Removing the `uuid` branch and re-rendering `freight.customer`
produces exactly that —

```
cause: rust/_types.jinja maps no Rust type onto the column customer.portal_uuid,
whose column_type is uuid. …
exit:  65 (EX_DATAERR)
```

— which is how the guard was shown not to be vacuous.

**Every row of the table below was measured against the server of `FR-EX-006`**,
not inferred: a real column of each of the 39 types was read through `sqlx` and
decoded into the type named here. The types whose direction was in doubt — the
two adapters, the temporal types, the two address types and the integer a `BIT`
becomes — were bound back as query parameters as well.

| `data_type` | Rust type | Only in `freight` |
|---|---|:-:|
| `tinyint` | `i8`, `u8` when unsigned | |
| `smallint` | `i16`, `u16` when unsigned | |
| `mediumint` | `i32`, `u32` when unsigned | |
| `int` | `i32`, `u32` when unsigned | |
| `bigint` | `i64`, `u64` when unsigned | • |
| `bit` | `u64` | • |
| `decimal` | `Decimal` | |
| `float` | `f32` | • |
| `double` | `f64` | • |
| `char` | `String` | |
| `varchar` | `String` | |
| `tinytext` | `String` | • |
| `text` | `String` | |
| `mediumtext` | `String` | • |
| `longtext` | `String` | • |
| `enum` | `String` | |
| `set` | `String` | |
| `uuid` | `String` | • |
| `binary` | `Vec<u8>` | • |
| `varbinary` | `Vec<u8>` | • |
| `tinyblob` | `Vec<u8>` | • |
| `blob` | `Vec<u8>` | |
| `mediumblob` | `Vec<u8>` | • |
| `longblob` | `Vec<u8>` | • |
| `date` | `chrono::NaiveDate` | • |
| `datetime` | `chrono::NaiveDateTime` | |
| `timestamp` | `chrono::DateTime<Utc>` | |
| `time` | `MySqlTime` | • |
| `year` | `u16` | |
| `inet4` | `std::net::Ipv4Addr` | • |
| `inet6` | `std::net::Ipv6Addr` | • |
| `geometry` | `Wkb` | • |
| `geometrycollection` | `Wkb` | • |
| `linestring` | `Wkb` | • |
| `multilinestring` | `Wkb` | • |
| `multipoint` | `Wkb` | • |
| `multipolygon` | `Wkb` | • |
| `point` | `Wkb` | • |
| `polygon` | `Wkb` | • |

**25 of the 39 are carried by `freight` alone**, which is why `FR-EX-006` puts a
third schema beside the two published ones. A mapping written against `sakila`
and `world` would have been complete over 14 types and would never have met a
spatial column, a `BIT`, an `INET6`, a `UUID`, a `TIME`, a generated column or a
system-versioned table.

The chrono types are reached through `sqlx::types::chrono`, so the crate depends
on `sqlx` and on nothing else; `MySqlTime`, `Decimal` and `Wkb` are named in
full below.

### The two adapters, and why Rust needs them where Go did not

`sqlx` decodes a `DECIMAL` column into `rust_decimal::Decimal` or
`bigdecimal::BigDecimal`, each behind a feature and a crate, and into nothing
else — `String` is not in the type's compatibility list — and it decodes a
spatial column into nothing at all, `Vec<u8>` included. Both facts were measured
against this server: of a battery of every integer and float width, `bool`,
`String`, `Vec<u8>`, the chrono types, `MySqlTime`, `Uuid`, `Ipv4Addr` and
`Ipv6Addr`, not one decoded either column.

The mapping therefore names two adapters, rendered into each schema module by
`rust/schema.jinja`:

| Adapter | Carries | Why |
|---|---|---|
| `Decimal(pub String)` | The exact decimal text the server sends | The only lossless mapping that costs no dependency. `f64` cannot hold `decimal(38,10)`, and the value is text on the wire in both protocols |
| `Wkb(pub Vec<u8>)` | A four-byte SRID followed by well-known binary | What MariaDB writes for a spatial column and what `ST_GeomFromWKB` reads back. It parses nothing |

Each implements `sqlx::Type`, `Decode` and `Encode` in about thirty lines,
delegating to `String` and to `Vec<u8>` and widening only the compatibility test.
A project that wants decimal arithmetic or parsed geometry replaces one branch
of one macro with a crate's type and turns on the matching `sqlx` feature — which
is the whole reason the mapping lives here and not in `tpl`.

Four further mappings are opinions worth stating outright:

- **`time` is `MySqlTime`**, `sqlx`'s own type. A MariaDB `TIME` is a signed
  interval of up to ±838:59:59; `chrono::NaiveTime` decodes the fixture's value
  and errors on any value outside a day.
- **`timestamp` and `datetime` are different Rust types.** Measured: a
  `TIMESTAMP` decodes into `DateTime<Utc>` and not into `NaiveDateTime`, and a
  `DATETIME` into `NaiveDateTime`. It is the one place where `sqlx`'s own
  documented table and this server agree exactly.
- **`uuid` is a `String`.** MariaDB's native `UUID` arrives as its 36-character
  text, and `sqlx`'s `Uuid` decodes 16 raw bytes, so the text is what this server
  actually hands a caller.
- **A JSON column has no branch**, because the model never reports one: MariaDB
  reports it as `longtext`, per `FR-CAT-038`.

### Nullability

A nullable column is not the same Rust type as a non-nullable one:

| Column | Rust field |
|---|---|
| `first_name varchar(45) NOT NULL` | `pub first_name: String` |
| ``back`tick varchar(48) NULL`` | `pub back_tick: Option<String>` |
| `gross_weight_kg decimal(12,3) NOT NULL` | `pub gross_weight_kg: super::Decimal` |
| `last_login_ipv6 inet6 NULL` | `pub last_login_ipv6: Option<std::net::Ipv6Addr>` |
| `survey_footprint geometry NULL` | `pub survey_footprint: Option<super::Wkb>` |

`Option<T>` is the language's own answer and it puts the difference in the type
rather than in a comment: `sqlx` decodes a `NULL` into `None` and refuses to
decode one into `T`, so the compiler makes a caller open the value before
reading it. A filter field is always `Option<T>`, nullable column or not, because
there `None` means *no condition*; whether the column is `NULL` is asked through
the `…_is_null` field a nullable column also gets.

### Identifiers

`FR-ENV-031` passes a character outside `A-Za-z` through the naming filters
unchanged, so `pascal` renders the column `` back`tick `` as `` Back`tick ``,
which is an identifier in no language. The macro replaces the ASCII punctuation a
quoted MariaDB identifier may carry with an underscore, and
`freight.legacy_edi_field` is the table that exercises it:

| Column | Field | Variant |
|---|---|---|
| `` back`tick `` | `back_tick` | `BackTick` |
| `space in name` | `space_in_name` | `SpaceInName` |
| `select` | `select` | `Select` |
| `posição` | `posição` | `Posição` |
| `Mixed Case Column` | `mixed_case_column` | `MixedCaseColumn` |

Two of those rows are Rust-specific and are why the mapping does not simply copy
the Go example's:

- **A non-ASCII letter is left alone.** Rust admits it in an identifier, so
  `posição` is a field name and `Posição` an enum variant. Removing it would be
  a loss of information for no gain.
- **`select` needs no escape.** It is a keyword of SQL and of Go, and it is not
  one of Rust's. The macro still carries the whole keyword list: a name whose
  snake case is a Rust keyword is written as a raw identifier, `r#match`, and
  the four keywords that cannot be raw — `crate`, `self`, `Self`, `super` — take
  a trailing underscore instead. No column of these three schemas needs it, so
  the escape is a guard rather than a demonstration, and the compile gate is
  what would report its absence.

The statement text is a separate question and is the `quote` filter's, per
`FR-ENV-008`: the same column reaches SQL as `` `back``tick` ``, with the
backtick doubled, while `row.try_get("back`tick")` names it raw.

## The compile gate

`run.py` runs both of these over the crate, from the example's own directory:

```
cargo build
cargo clippy --all-targets --all-features -- -D warnings
```

A missing toolchain is a failure that names what is missing, never a skip: the
gate's verdict is the example's acceptance signal, so a gate that did not run has
produced no verdict. The absence of clippy is reported separately from the
absence of `cargo`, because the remedy differs.

Clippy at `-D warnings` is a sharper reader of generated code than it looks.
Two of this example's rules exist because clippy refused the alternative, and
both are recorded in `_types.jinja`: a `.clone()` on a `Copy` type is
`clippy::clone_on_copy`, and a `&` on one passed to a generic parameter is
`clippy::needless_borrows_for_generic_args`. The macro therefore knows which
four of its thirty-nine types own a heap allocation, and the set was measured by
letting clippy name every field it wanted unborrowed.

The crate is excluded from the repository's workspace, in the root `Cargo.toml`,
so `cargo build` and `cargo clippy` at the repository root neither compile it nor
fetch what it depends on.

## Prerequisites

| | |
|---|---|
| The binary | `cargo build --release` |
| The server | `./scripts/mariadb/up.sh`, then `./scripts/mariadb/seed-datasets.sh 12.3` |
| The password | `export FIXTURE_PW=tpl-root` |
| Rust | 1.85 or newer — the floor edition 2024 sets, and the `rust-version` of the rendered crate. Verified on 1.98.1 |

The crate's one dependency is `sqlx` 0.9 with `mysql`, `runtime-tokio`,
`tls-rustls-ring-webpki` and `chrono`. `Cargo.lock` is committed; the first run
needs those crates in the registry cache or a network to fetch them.

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
  once per schema into `src/<schema>.rs`.
- **It runs no query.** The gate compiles; it does not connect. The type mapping
  was measured against the server while it was being written, and the rendered
  code carries that result rather than re-establishing it.

## Known limits

Each is a property of this example's templates, not of `tpl`, and each is visible
because the compile gate is what would report it:

- **Identifiers are sanitised, not proved.** A name beginning with a digit, and
  two names whose sanitised forms collide, still render code the Rust compiler
  refuses. That is when the gate reports it.
- **An acronym loses its case.** `actor_id` becomes `ActorId`, not `ActorID`.
  That is what `FR-ENV-033`'s `pascal` does, and its own *accepted cost* clause
  records it; buying it back would need an initialism list, which is a second
  opinion to maintain and another way for two columns to collide.
- **A module's file name is the table's name unchanged.** `run.py` writes
  `src/<schema>/<table>.rs` while the module declaration is the table's name in
  snake case, so a table whose name is not already snake case would render a
  declaration with no file behind it. All 36 are.
- **A table with no primary key is refused.** All 36 have one. A table without
  one ends the render with `65` and says why, because read, update and delete
  would have no key to address a row by. So is a table whose every column is
  generated or part of the key, which would leave an `UPDATE` with an empty
  `SET`.
- **The crate-level doctest is outside the gate.** `cargo build` and
  `cargo clippy` do not compile doctests; `cargo test --doc` does, and it passes.
