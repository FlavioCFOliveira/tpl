# `node-data-layer` — a Node.js data layer, built from three schemas through the command line

This is the Node.js member of the four worked examples `FR-EX-001` requires. It
reads `sakila`, `world` and `freight` from one MariaDB server and renders one ES
module directory per schema: a JSDoc `@typedef` for the row, its primary key, the
columns an ordering may name, the filter a search takes, a base class carrying
the contract and a `mysql2/promise` implementation of it, for every one of the 36
tables.

Its acceptance signal is not that `tpl` exited `0`. It is that `node --check` and
`tsc --checkJs --noEmit` accept every file it rendered, per `FR-EX-009`, and the
example's own script is what runs them.

```sh
npm --prefix examples/node-data-layer install
export FIXTURE_PW=tpl-root
python3 examples/node-data-layer/run.py
```

## What it produces

| Path | Rendered from | Count |
|---|---|---|
| `src/<schema>/core.js` | `node/core` | 3, one per schema |
| `src/<schema>/index.js` | `node/index` | 3 |
| `src/<schema>/<table>.js` | `node/model` | 36, one per table |
| `src/<schema>/<table>_repository.js` | `node/repository` | 36 |
| `src/<schema>/<table>_mysql2.js` | `node/mysql2` | 36 |

114 files and 25,001 lines, from 114 invocations of `tpl render`. One file is one
invocation, because `tpl render` writes to standard output and nowhere else —
`FR-RND-028`, and `FR-EX-005` is the consequence.

`src/` is committed, so that the example can be read without being run. Two runs
of it produce a byte-identical tree: nothing here reads `now`, which `FR-CTX-030`
makes the single documented source of non-reproducibility.

### Why the rendered files are `src/`

Nothing about Node forces the choice, and that is the answer. A module specifier
is either a path relative to the importing file or a package subpath resolved
through `exports` in `package.json`, and neither takes its name from a directory.
Node is therefore unlike Python, where a module's name **is** its import path and
the rendered tree had to be a package; unlike Rust, where a module's path **is**
its file's path and only a `#[path]` attribute could move it; and closer to Go,
where a package's directory and its package clause are independent.

`src/` is what a package with no build step publishes, and `exports` maps
`./sakila` onto `./src/sakila/index.js`, so the directory never appears in an
import. `run.py` removes everything under it before each run, so a table dropped
from a schema cannot leave a file behind that still parses.

The tree is JavaScript with JSDoc types and there is no compile step: what is
committed is what runs, and `tsc` only reads it.

### The schema's own material is two files, and the reason is the language

`core.js` carries the schema name, the errors, the ordering `Direction`, the
translation of a driver failure, and `connect`. `index.js` carries the package's
public surface and `Repositories`. The Rust example puts the same material in one
file.

An ES module's bindings are hoisted but its `class` declarations are not
initialised until the module body reaches them, so two modules that each need the
other's class produce a temporal-dead-zone error at import time rather than at
build time. Were the errors declared in `index.js`, every table module would
import from a module that is still evaluating. `core.js` imports nothing from its
own directory and can therefore never be in a cycle; `index.js` re-exports
everything it declares, so a caller never writes `core`.

```js
import { connect, Repositories, VesselColumns } from 'tpl-example-node-data-layer/freight';

const connection = await connect({ host: '127.0.0.1', port: 13309, user: 'root', password });
const repositories = new Repositories(connection);
const vessel = await repositories.vessel.read({ vesselId: 1 });
```

`connect` passes the caller's options to mysql2 unchanged, with `charset` as a
default under them and `supportBigNumbers`, `bigNumberStrings` and `database`
applied over them. **No repository commits**: mysql2 autocommits until a caller
calls `beginTransaction`, so transaction control stays with the caller.

### The interface

Every table's contract declares the same five methods. `Actor` stands for the
table's JavaScript type name, and every method is `async`:

| Method | Signature | What it does |
|---|---|---|
| `create` | `(row: Actor) => Promise<void>` | Inserts, naming neither generated columns nor the auto-increment key. Where the table has one, the key the server assigned is written back into `row` |
| `read` | `(key: ActorKey) => Promise<Actor>` | Selects one row by its primary key, simple or composite. Rejects with `NotFound` when there is none |
| `update` | `(row: Actor) => Promise<number>` | Writes every column that is neither generated nor part of the key, and returns the rows the server reported as affected |
| `delete` | `(key: ActorKey) => Promise<number>` | Removes one row by its primary key, and returns the rows removed |
| `search` | `(criteria: ActorFilter, order?: readonly ActorOrder[], limit?: number \| null, offset?: number \| null) => Promise<Actor[]>` | The advanced search |

`delete` needs no renaming although it is a reserved word: since ES5 a reserved
word is a legal property name, and a method is a property. The Python example had
to rename its filter parameter for the opposite reason, and this one does not.

`ActorFilter` is an object type carrying one optional property per column for
equality, a `…From` and a `…To` for every numeric and temporal column, a `…Like`
for every textual one, and a `…IsNull` for every nullable one. A property left
absent adds no condition, and the conditions that are set are joined with `AND`.
Which columns get which is decided by the registered tests `numeric`, `temporal`,
`textual` and `nullable` of `FR-ENV-041`, whose memberships `FR-ENV-046` fixes.
`{}` is the filter that matches every row.

No filter property admits `null`, and that is what keeps *no condition* and *is
NULL* apart: absence is the first and the `…IsNull` boolean is the second. It is
the same separation the Python example makes, expressed with the value JavaScript
actually uses for absence.

`ActorOrder` is a column and a direction. The column is an `ActorColumn`, a union
of the table's own column names, and it resolves through a closed frozen mapping
into a quoted identifier this example's template wrote. The resolution is checked
twice: `tsc` refuses a value outside the union, and `actorColumnSql` refuses one
at run time with `UnknownColumn` — an own-property lookup, so a caller that
reached past the type with an unchecked string is refused rather than
interpolated. `directionSql` refuses a direction the same way. **No value a caller
supplies is ever part of a statement's text.**

The five fixed statements are module-level constants. The search, whose shape
depends on the filter, is assembled from fragments this template wrote and a list
of bound arguments, and every value it carries is a placeholder.

### Where `tsc` proves the two agree

The contract is a base class whose every method throws `NotImplemented`, and the
implementation extends it. JavaScript has no structural interface a `.js` file can
declare and no `abstract` keyword, so where Go has an `interface`, Rust a `trait`
and Python a `typing.Protocol`, this example has inheritance — and the check is
stronger for it, because `noImplicitOverride` is on and every rendered method
carries `@override`:

```js
  /**
   * @override
   * @param {ActorKey} key
   * @returns {Promise<Actor>}
   */
  async read(key) {
```

A method whose signature drifted from the contract's fails with `TS2416`; a method
whose **name** drifted — a rename that would silently leave the base class's
throwing version in place — fails with `TS4113`, because a tag claiming an
override that overrides nothing is an error. Neither failure is visible to
`node --check`, which accepts a repository that implements nothing at all. It is
the failure this example is most likely to produce and the one nothing else would
report.

## The type mapping

`tpl` ships no `js_type` filter and never will: `FR-ENV-010` says so and
`BR-ENV-002` says why — a type mapping is an opinion, and an opinion belongs to
the project holding it. The opinion is therefore a macro of this example,
`templates/node/_types.jinja`, in the form `FR-ENV-011` fixes. This example begins
from nothing: `tpl init` writes a starter macro for Rust, per `FR-PROJ-017`, and
this language has none, which is the case the extension point exists for.

It is obliged to cover every value of `data_type` the model reports for a column
of the three schemas, per `FR-EX-008`. That is **39 values over 362 columns**, and
all 39 are branched on. Nothing falls through: the final branch calls `fail()`, so
a type the macro has never met ends the render with `65` and names the column.
Removing the `uuid` branch and re-rendering `freight.customer` produces exactly
that —

```
cause: node/_types.jinja maps no JavaScript type onto the column customer.portal_uuid,
       whose column_type is uuid. …
exit:  65 (EX_DATAERR)
```

— which is how the guard was shown not to be vacuous.

**Every row of the table below was measured against the server of `FR-EX-006`**,
not inferred: a real non-`NULL` value of a real column of each of the 39 types was
read through mysql2 3.24.4 and its runtime type recorded. Nothing here comes from
the driver's documentation.

| `data_type` | JavaScript type | Only in `freight` |
|---|---|:-:|
| `tinyint` | `number` | |
| `smallint` | `number` | |
| `mediumint` | `number` | |
| `int` | `number` | |
| `year` | `number` | |
| `float` | `number` | • |
| `double` | `number` | • |
| `bigint` | `string` | • |
| `decimal` | `string` | |
| `char` | `string` | |
| `varchar` | `string` | |
| `tinytext` | `string` | • |
| `text` | `string` | |
| `mediumtext` | `string` | • |
| `longtext` | `string` | • |
| `enum` | `string` | |
| `set` | `string` | |
| `time` | `string` | • |
| `uuid` | `string` | • |
| `inet4` | `string` | • |
| `inet6` | `string` | • |
| `bit` | `Buffer` | • |
| `binary` | `Buffer` | • |
| `varbinary` | `Buffer` | • |
| `tinyblob` | `Buffer` | • |
| `blob` | `Buffer` | |
| `mediumblob` | `Buffer` | • |
| `longblob` | `Buffer` | • |
| `geometry` | `Buffer` | • |
| `geometrycollection` | `Buffer` | • |
| `linestring` | `Buffer` | • |
| `multilinestring` | `Buffer` | • |
| `multipoint` | `Buffer` | • |
| `multipolygon` | `Buffer` | • |
| `point` | `Buffer` | • |
| `polygon` | `Buffer` | • |
| `date` | `Date` | • |
| `datetime` | `Date` | |
| `timestamp` | `Date` | |

**25 of the 39 are carried by `freight` alone**, which is why `FR-EX-006` puts a
third schema beside the two published ones. A mapping written against `sakila` and
`world` would have been complete over 14 types and would never have met a spatial
column, a `BIT`, a `BIGINT`, an `INET6`, a `UUID`, a `TIME`, a generated column or
a system-versioned table.

The mapping needs **no dependency beyond mysql2**: four types answer for 39, and
all four — `number`, `string`, `Buffer` and `Date` — are the platform's own.

### The `BIGINT` decision, and why it is a decision about the connection

A JavaScript `number` is an IEEE-754 double and loses integer precision above
2^53. A MariaDB `BIGINT UNSIGNED` reaches 2^64-1, and the fixture declares seven
auto-increment keys of that type.

Measured, on the same column and the same server:

| Connection | `freight.audit_event.audit_event_id` arrives as |
|---|---|
| mysql2's defaults | `number` |
| `supportBigNumbers: true`, `bigNumberStrings: true` | `string` |

`connect` sets both, always, so the mapping is `string` unconditionally.
`supportBigNumbers` alone was **rejected**: it yields a `string` only when the
value leaves the safe range, which is the type `number | string` — a type no
static checker can help a caller with, and one whose shape depends on the data
rather than on the schema. Leaving both off was rejected because the loss is
silent, which is the one property a generated data layer must not have.

The `BigInt` primitive was rejected too. mysql2 offers no option that produces
one, so it would need a `typeCast` of this example's own, and a row carrying a
`BigInt` throws on `JSON.stringify` — which is the first thing a caller of a Node
data layer is likely to do with a row. `Buffer` and `Date` both serialise; a
`BigInt` does not.

One consequence is visible in the rendered `create`: mysql2 reports
`ResultSetHeader.insertId` as a `number` whatever the column is, so a `BIGINT` key
is written back as `String(insertId)` and a smaller one as `insertId`. The type
checker is what would have reported the difference, and the template makes it
rather than waiting to be told.

### The spatial types, and the default this example overrides

Measured: **mysql2 parses a geometry column**. A `POINT` arrives as `{x, y}`, a
`LINESTRING` as an array of those, a `POLYGON` as an array of rings. That is a
genuine difference from the other three examples, all of which receive bytes.

Those objects do not bind back. mysql2 escapes a plain object as a JSON string, so
`WHERE reference_marker = ?` with the parsed point matched **0 rows** — silently —
and an insert would have stored that JSON text in a geometry column. A mapping onto
the parsed objects would therefore have type-checked and been wrong at the
database, which is the same shape of defect the Python example found in its `BIT`.

This example maps the eight spatial types to `Buffer` and asks the server to do
the conversion at both ends:

```sql
SELECT …, ST_AsWKB(`reference_marker`) AS `reference_marker` FROM `port_facility`
INSERT INTO `port_facility` (…, `reference_marker`) VALUES (…, ST_GeomFromWKB(?))
SELECT … WHERE `reference_marker` = ST_GeomFromWKB(?)
```

Measured on this server: the `SELECT` returns a `Buffer` of well-known binary, and
the equality matches the row the bytes came from. `WHERE reference_marker = ?`
with those same bytes matches nothing, which is why the function is not optional.
The rendered `port_facility` search was run against the fixture and found its row.

`ST_AsWKB` drops the SRID, which the internal representation carries as a
four-byte prefix. Every spatial column of `freight` is SRID 0, so the round trip is
exact here; a schema mixing SRIDs would need `ST_SRID` carried alongside, and that
is an opinion for the project that has one.

### The other opinions, and where JavaScript's answer is the poorer one

- **`decimal` is a `string`**, which is what mysql2 returns and what loses
  nothing: a `number` cannot hold `decimal(38,10)`. Measured to bind back —
  `WHERE gross_weight_kg = ?` with the returned `"21480.500"` matches the row it
  came from. This is where JavaScript does worse than Python, whose
  `decimal.Decimal` is lossless *and* arithmetic; it is exactly where Go landed,
  for the same reason.
- **`time` is a `string`.** Measured: `06:00:00` arrives as `"06:00:00"`. A
  MariaDB `TIME` is a signed interval of up to ±838:59:59, which is not a time of
  day, and JavaScript has no interval type at all. Python's `timedelta` is the one
  standard library of the four that had the right type.
- **`date`, `datetime` and `timestamp` are `Date`**, because `Date` is the only
  temporal type the language has. The limit was measured too: mysql2 builds the
  `Date` in the connection's time zone, so a `DATE` of `2026-04-18` arrives as
  local midnight and prints `2026-04-17T23:00:00.000Z` in a UTC+1 process. The
  instant binds back unchanged — `WHERE payment_due_on = ?` matches — so the round
  trip is exact; what is inexact is reading a calendar date off the UTC fields of
  the result. A caller that cares passes `timezone` to `connect`, which reaches
  mysql2 unchanged. Go and Python both have a date type that is not a timestamp;
  JavaScript does not.
- **Width and signedness are not expressed.** `number` covers `TINYINT` through
  `INT`, signed or unsigned, exactly, so there is no `int8` to narrow to and no
  unsigned form to distinguish. `column.unsigned` is read nowhere in the macro. It
  is the one place this mapping differs from Go's and Rust's in kind rather than
  in spelling, and it is the same place Python's does.
- **A `tinyint(1)` is a `number` and not a `boolean`.** The catalogue reports
  `tinyint` for `tinyint(1)` and for `tinyint(3)` alike — `FR-EX-008` obliges the
  mapping over `data_type`, which does not carry the display width — and mysql2
  returns `0`, not `false`. Measured on `freight.vessel.in_service`, declared
  `BOOLEAN`, which is therefore `inService: number`.
- **`uuid`, `inet4` and `inet6` are `string`.** Measured: a MariaDB `UUID` arrives
  as its 36 characters of text and not as 16 bytes, and the two address types
  arrive as character data — `203.0.113.24` and `2001:db8:3f2b::c7`.
- **A JSON column has no branch**, because the model never reports one: MariaDB
  reports it as `longtext`, per `FR-CAT-038`. `freight.customer.preferences` is
  that column, and it is a `string`. A caller parses it; the data layer does not,
  because a data layer that parsed it would be asserting a shape the catalogue
  does not carry.

### The one mapping that does not round-trip, and the measurement that says so

`bit` is `Buffer`, because that is what mysql2 returns — `bit(8)` carrying `7`
arrives as `<Buffer 07>`. Binding those same bytes back does **not** match:

```
SELECT COUNT(*) FROM consignment WHERE notification_flags = ?   -- <Buffer 07>  ->  0
SELECT COUNT(*) FROM consignment WHERE notification_flags = ?   -- 7            ->  1
```

Both measured on this server, and then confirmed through the rendered code itself:
`repositories.consignment.search({ notificationFlags: row.notificationFlags })`
returns **0 rows** for a value read out of that very table.

**This is the same defect the Python example found, and mysql2 has it too**, for
the same reason: MariaDB compares a `BIT` with a binary string by converting the
string to a number, and `\x07` is not a numeral. The cause is the server's
comparison rule rather than either driver, so the two examples agree; the fix is
the same in both, and it is a project's to make — replace one branch of one macro
with `number` and decode the column yourself, which is the whole reason the
mapping lives in this example and not in `tpl`.

The eight spatial types would have had the same problem and do not, because this
example converts them at both ends; see above.

### Nullability

A nullable column is not the same JavaScript type as a non-nullable one:

| Column | JavaScript property |
|---|---|
| `first_name varchar(45) NOT NULL` | `firstName: string` |
| ``back`tick varchar(48) NULL`` | `backTick: string \| null` |
| `gross_weight_kg decimal(12,3) NOT NULL` | `grossWeightKg: string` |
| `last_login_ipv6 inet6 NULL` | `lastLoginIpv6: string \| null` |
| `survey_footprint geometry NULL` | `surveyFootprint: Buffer \| null` |

`T | null` puts the difference in the type rather than in a comment: mysql2
returns `null` for a `NULL`, and `strict` makes a caller narrow the property
before reading it. `undefined` is deliberately not part of a row's type — every
property is present on a row — and is reserved for the filter, where it is the
only thing it can mean.

### Identifiers

`FR-ENV-031` passes a character outside `A-Za-z` through the naming filters
unchanged, so `pascal` renders the column `` back`tick `` as `` Back`tick ``,
which is an identifier in no language. The macro replaces the ASCII punctuation a
quoted MariaDB identifier may carry with an underscore, and
`freight.legacy_edi_field` is the table that exercises it:

| Column | Property | Constant | Statement text |
|---|---|---|---|
| `` back`tick `` | `backTick` | `BACK_TICK` | `` `back``tick` `` |
| `space in name` | `spaceInName` | `SPACE_IN_NAME` | `` `space in name` `` |
| `select` | `select` | `SELECT` | `` `select` `` |
| `posição` | `posição` | `POSIçãO` | `` `posição` `` |
| `Mixed Case Column` | `mixedCaseColumn` | `MIXED_CASE_COLUMN` | `` `Mixed Case Column` `` |

Four of those rows are JavaScript-specific:

- **`camel`, not `snake` or `Pascal`.** A property is camel case in this language,
  which is the only difference between this column list and the other three
  examples': all four read the same `name` field of the same model.
- **A non-ASCII letter is kept.** JavaScript identifiers are defined over the
  Unicode `ID_Start` and `ID_Continue` properties of UAX #31, which `ç` and `ã`
  both satisfy, so `posição` is a property and is read as `row.posição`. Removing
  the two letters would lose information for no gain, and there is nothing to
  normalise: the file is UTF-8 and an identifier is compared by code point.
  A search ordered by `` `posição` `` was run against the fixture and returned its
  five rows.
- **The constant keeps that letter's case.** `upper_snake` case-folds over
  `A-Za-z` alone, per `FR-ENV-031`, so `posição` becomes `POSIçãO`. It is a legal
  property name and it is what determinism costs: a case-folding that knew about
  `ç` would be one that depended on the locale, and `NFR-DET-001` would not
  survive it.
- **No keyword needs an escape, in any position.** Every name the macro produces
  is used as a property — in an object literal, in a `@property` tag, in a member
  expression — and since ES5 a reserved word is legal in all three. `select` is
  therefore the property `select`, where Python needed PEP 8's trailing underscore
  for its own keywords. The one name that would misbehave is `__proto__`, which in
  an object literal sets the prototype instead of defining a property, and it is
  out of reach: `FR-ENV-030` discards an empty word, so a column called
  `__proto__` arrives as `proto`.

A rendered file is named for the table exactly as the catalogue names it, and
every import names it the same way, so no transformation stands between a
module's name and its file's. That is one of the Python example's known limits
that this one does not have.

### Two hazards the other examples do not have, and one they have that this does not

- **A block comment can be closed by its own content.** A catalogue comment or a
  generated column's expression carrying `*/` would end a JSDoc block and put the
  rest of the text into the token stream, and a line beginning with `@` inside one
  would be read as a tag. Every piece of catalogue text that reaches a
  `/** … */` block is passed through a macro that breaks `*/` with a space and
  flattens newlines; text that must keep its shape — a table's own comment — is
  written as `//` line comments instead, through the `comment` filter of
  `FR-ENV-038`, because a line comment cannot be closed by its content and a
  newline in it produces another line comment.
- **A backtick opens a template literal.** A quoted MariaDB identifier is full of
  backticks, so no rendered statement is built with one: every statement and every
  fragment is assembled by concatenation, and every string literal holding
  catalogue text is emitted through the `json` filter of `FR-ENV-036` — a JSON
  string literal is a JavaScript string literal, so a name carrying a quote or a
  backslash is escaped rather than ending the literal early.
- **No `%` needs doubling.** PyMySQL's paramstyle is `format` and builds the final
  statement as `query % arguments`, so the Python example had to double every `%`
  in an identifier. Every statement here is run through `Connection.execute`,
  which prepares it on the server: the SQL reaches MariaDB unmodified and MariaDB
  counts the placeholders, so neither a `%` nor a `?` inside a quoted identifier is
  a hazard. `Connection.query`, which interpolates client-side, is used nowhere in
  the rendered code.

Every statement is also run with `rowsAsArray`, so a result is an array and never
an object, and a row is built by position. A column named `__proto__` or
`constructor` would otherwise reach an object literal as a key.

## The compile gate

`run.py` runs both of these over the rendered tree, from the example's own
directory:

```
node --check <file>          # once per rendered file — it takes one file
tsc --checkJs --noEmit --project tsconfig.json
```

`FR-EX-009` names two kinds of reader — a compiler where the language has one, and
its own static type checker where it has not — and JavaScript is the second case,
so the gate is both tools. `node --check` decides whether each file is a
JavaScript file at all, and it is the only one of the two that reads the file as
the runtime will: it consults `package.json` to decide the file is a module, which
is why `"type": "module"` is committed beside the rendered tree — parsed as a
script, every one of these files is a syntax error at its first `import`.
`tsc --checkJs` reads the JSDoc annotations and decides whether the types agree,
which is the half that catches a rendered implementation whose signature drifted
from its base class. Neither is sufficient alone.

The flags are passed outright as well as set in `tsconfig.json`, so the two
commands are readable from `run.py` alone. The configuration carries the rest:
`strict`, `noImplicitOverride`, and the `nodenext` module resolution a Node ES
module package needs. `target` and `lib` are `ES2023` — the floor this example
claims, which `engines` in `package.json` states as Node 20: `Array.prototype.toSorted` is the ES2023 addition Node 20 is the first to carry — rather than
whatever the installed compiler defaults to. It is the Rust example's
`rust-version` in another language.

A missing toolchain is a failure that names what is missing, never a skip: the
gate's verdict is the example's acceptance signal, so a gate that did not run has
produced no verdict. The two ways it can be missing are reported separately,
because the remedy differs:

- **`tsc` will not run.** Neither the copy under `node_modules` nor
  `npx --no-install tsc` answered `--version`. The local copy is preferred because
  it is the one sharing a tree with the type definitions the rendered code is
  checked against; `--no-install` is what keeps the fallback off the network.
- **`tsc` cannot resolve what the rendered code imports.** A `TS2307` or `TS2688`
  in the output is an incomplete environment, not 114 defects in the rendered
  code, and is reported as one.

Neither tool writes into the tree: `node --check` writes nothing, and
`tsc --noEmit` emits nothing and is configured with no `incremental` cache. The
committed `src/` stays exactly as rendered.

## Prerequisites

| | |
|---|---|
| The binary | `cargo build --release` |
| The server | `./scripts/mariadb/up.sh`, then `./scripts/mariadb/seed-datasets.sh 12.3` |
| The password | `export FIXTURE_PW=tpl-root` |
| Node.js | 20 or newer, which is the floor `engines` states. Verified on v26.9.0 |

The three packages a reader installs, into this example's own `node_modules`:

```sh
npm --prefix examples/node-data-layer install
```

| Package | Why | Verified at |
|---|---|---|
| `mysql2` | What the rendered implementation imports, what the type mapping was measured through, and the source of its own type definitions | 3.24.4 |
| `typescript` | Half the compile gate | 5.9.3 |
| `@types/node` | `Buffer` is a Node global and the rendered rows are full of it | 26.6.2 |

`package.json`, `package-lock.json` and `tsconfig.json` are committed, as `go.mod`
and `go.sum` are in the Go example and `Cargo.toml` and `Cargo.lock` in the Rust
one. `node_modules/` is not, and nothing is installed globally: the gate resolves
`tsc` from this example's own tree first and falls back to `npx --no-install`,
which reaches no network.

`run.py` itself and `_driver` take no dependency: Python standard library only.
The three above are needed by the *rendered* package and by the gate that reads
it.

The server, the three schemas and the entries are the driver's, not this
example's: [`_driver/README.md`](../_driver/README.md) documents them, and
`FR-EX-007` is why all four examples read them the same way.

## What this example does not do

- **It writes nothing to the database.** `tpl` cannot, per the read-only
  invariant, and neither does the example: it renders files and checks them.
- **It reaches no library interface of `tpl`.** Five commands and their exit
  codes, per `FR-EX-004`, which is the surface the primary consumer has.
- **It records no timestamp.** `now` is the single documented source of
  non-reproducibility, per `FR-CTX-030`, and a generated file carrying one
  produces a diff on every run for a reason that is not a change. Provenance —
  the server, its series, its standing and the version of `tpl` — is rendered once
  per schema into `core.js` and `index.js`.
- **The gate runs no query.** It reads; it does not connect. The type mapping was
  measured against the server while it was being written, and the rendered code
  carries that result rather than re-establishing it on every run.
- **It is not TypeScript.** The deliverable is JavaScript with JSDoc types, which
  is what makes the rendered tree runnable with no build step. `tsc` is a reader
  of it and never a producer.

## Known limits

Each is a property of this example's templates, not of `tpl`, and each is visible
because the compile gate is what would report it:

- **Identifiers are sanitised, not proved.** A name beginning with a digit, and
  two names whose sanitised forms collide, still render code JavaScript refuses. A
  duplicate property in a `@typedef` is reported by `tsc` as
  `Duplicate identifier`, and a duplicate export from `index.js` as an ambiguous
  re-export.
- **An acronym loses its case.** `actor_id` becomes `actorId` and `ActorId`, not
  `actorID` and `ActorID`. That is what `FR-ENV-033`'s naming filters do, and its
  own *accepted cost* clause records it; buying it back would need an initialism
  list, which is a second opinion to maintain and another way for two columns to
  collide.
- **A table with no primary key is refused.** All 36 have one. A table without one
  ends the render with `65` and says why, because read, update and delete would
  have no key to address a row by. So is a table whose every column is generated
  or part of the key, which would leave an `UPDATE` with an empty `SET`.
- **An equality filter over a `BIT` column matches nothing**, for the measured
  reason given above. It is the one condition the rendered search builds that
  cannot succeed, and it is the one the Python example found first.
- **A `Date` carries a time of day that a `DATE` column does not**, and it is
  built in the connection's time zone. The round trip is exact; reading a calendar
  date off `getUTCDate()` is not.
- **One connection, not a pool.** A mysql2 `Connection` carries one session and
  serialises the statements sent over it, so `Repositories` holds one and a caller
  wanting concurrency builds one `Repositories` per connection — or passes a
  `PoolConnection`, which is a `Connection` and is what `mysql2.createPool` hands
  out. `connect` opens a single connection because that is what the other three
  examples open, and `FR-EX-007` is why the four are kept comparable.
- **`ST_AsWKB` drops the SRID.** Every spatial column of `freight` is SRID 0, so
  nothing is lost here; a schema mixing SRIDs would lose the identifier on a round
  trip through this data layer.
