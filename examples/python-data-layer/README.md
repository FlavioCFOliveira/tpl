# `python-data-layer` — a Python data layer, built from three schemas through the command line

This is the Python member of the four worked examples `FR-EX-001` requires. It
reads `sakila`, `world` and `freight` from one MariaDB server and renders one
Python subpackage per schema: a dataclass row, its primary key, the columns an
ordering may name, a `typing.Protocol` and a PyMySQL implementation of that
protocol, for every one of the 36 tables.

Its acceptance signal is not that `tpl` exited `0`. It is that
`python -m compileall` and `mypy --strict` accept every file it rendered, per
`FR-EX-009`, and the example's own script is what runs them.

```sh
export FIXTURE_PW=tpl-root
python3 examples/python-data-layer/run.py
```

## What it produces

| Path | Rendered from | Count |
|---|---|---|
| `datalayer/<schema>/_core.py` | `python/core` | 3, one per schema |
| `datalayer/<schema>/__init__.py` | `python/package` | 3 |
| `datalayer/<schema>/<table>.py` | `python/dataclass` | 36, one per table |
| `datalayer/<schema>/<table>_repository.py` | `python/repository` | 36 |
| `datalayer/<schema>/<table>_pymysql.py` | `python/pymysql` | 36 |

114 files and 26,132 lines, from 114 invocations of `tpl render`. One file is one
invocation, because `tpl render` writes to standard output and nowhere else —
`FR-RND-028`, and `FR-EX-005` is the consequence.

`datalayer/` is committed, so that the example can be read without being run.
Two runs of it produce a byte-identical tree: nothing here reads `now`, which
`FR-CTX-030` makes the single documented source of non-reproducibility.

### Why the rendered files are a package

A Python module's name **is** its import path. `from datalayer import freight`
resolves to `datalayer/freight/__init__.py` and nowhere else, so rendering into
an `out/` of its own — which is what the Go example does, because a Go package's
directory name and its package clause are independent — would leave the files
unimportable without a `sys.path` entry per schema. The rendered tree is
therefore a package, and `datalayer/__init__.py` is the one Python file this
example does not render: it names the three schema subpackages, and no
invocation of `tpl render` is in a position to, because a render sees one
database at a time, per `FR-RND-006`. `run.py` removes everything else under
`datalayer/` before each run, so a table dropped from a schema cannot leave a
file behind that still imports.

That constraint is weaker than Rust's. The Rust example renders into the crate's
own `src/` because a Rust module's path **is** its file's path and a `#[path]`
attribute on every declaration is the only way around it; Python needs only that
the directory be a package, which an `__init__.py` makes it.

### The schema's own material is two files, and the reason is the language

`_core.py` carries the schema name, the errors, the ordering `Direction`, the
translation of a driver failure, and `connect`. `__init__.py` carries the
package's public surface and `Repositories`. The Rust example puts the same
material in one file.

A Python package body **executes**, top to bottom. Were the errors declared in
`__init__.py`, every table module would have to import from a package that is
still executing, and the data layer would be correct only for as long as nobody
moved a statement in `__init__.py`. `_core` imports nothing from its own package
and can therefore never be in a cycle; `__init__.py` re-exports every name it
declares, so a caller never writes `_core`.

```python
from datalayer import freight

connection = freight.connect(host="127.0.0.1", port=13309, user="root", password=...)
repositories = freight.Repositories(connection)
vessel = repositories.vessel.read(freight.VesselKey(vessel_id=1))
```

`connect` passes the caller's options to PyMySQL unchanged, with `charset` and
`autocommit` as defaults under them and `database` applied over them. It
autocommits because **no repository commits**: transaction control belongs to
the caller, who passes `autocommit=False` and commits its own unit of work.

### The protocol

Every table's contract declares the same five methods. `Actor` stands for the
table's Python class name:

| Method | Signature after `self` | What it does |
|---|---|---|
| `create` | `(row: Actor) -> None` | Inserts, naming neither generated columns nor the auto-increment key. Where the table has one, `cursor.lastrowid` is written back into `row` |
| `read` | `(key: ActorKey) -> Actor` | Selects one row by its primary key, simple or composite. Raises `NotFound` when there is none |
| `update` | `(row: Actor) -> int` | Writes every column that is neither generated nor part of the key, and returns the rows the server reported as changed |
| `delete` | `(key: ActorKey) -> int` | Removes one row by its primary key, and returns the rows removed |
| `search` | `(criteria: ActorFilter, order: Sequence[ActorOrder] = (), limit: int \| None = None, offset: int \| None = None) -> list[Actor]` | The advanced search |

The filter is `criteria` and not `filter`, which is a builtin of the language.

`ActorFilter` is a dataclass carrying one optional field per column for
equality, a `…_from` and a `…_to` for every numeric and temporal column, a
`…_like` for every textual one, and a `…_is_null` for every nullable one. Each
defaults to `None`, and the conditions that are set are joined with `AND`. Which
columns get which is decided by the registered tests `numeric`, `temporal`,
`textual` and `nullable` of `FR-ENV-041`, whose memberships `FR-ENV-046` fixes.
`ActorFilter()` matches every row, and `is_empty` is how the search asks whether
to write a `WHERE` clause at all.

`ActorOrder` is a column and a direction. The column is an `ActorColumn`, an
`enum.Enum` of the table's own columns, and `.sql` resolves it through a closed
`Mapping` into a quoted identifier this example's template wrote. A caller
holding a column name as text reaches the enumeration through
`ActorColumn.from_name`, which refuses every name the table does not carry with
`UnknownColumn`; `Direction.from_name` refuses a direction the same way.
**No value a caller supplies is ever part of a statement's text.**

The five fixed statements are module-level `Final` constants. The search, whose
shape depends on the filter, is assembled from fragments this template wrote and
a list of bound arguments, and every value it carries is a placeholder.

### Where `mypy` proves the two agree

The contract is a `typing.Protocol` and the implementation names no base class:
a protocol is satisfied structurally. The proof is at the foot of every
`<table>_pymysql.py`:

```python
if TYPE_CHECKING:

    def _implements_the_protocol(
        implementation: ActorPyMySQL,
    ) -> ActorRepository:
        return implementation
```

The return is the whole of the check. A method whose signature drifted from the
protocol's — a renamed parameter, a lost default, a widened return — fails
`mypy --strict` there, rather than a caller at run time, and the block costs
nothing at run time because it is never executed. It is the failure this example
is most likely to produce and the one nothing else would report: `compileall`
accepts a repository that implements nothing at all.

## The type mapping

`tpl` ships no `python_type` filter and never will: `FR-ENV-010` says so and
`BR-ENV-002` says why — a type mapping is an opinion, and an opinion belongs to
the project holding it. The opinion is therefore a macro of this example,
`templates/python/_types.jinja`, in the form `FR-ENV-011` fixes. This example
begins from nothing: `tpl init` writes a starter macro for Rust, per
`FR-PROJ-017`, and this language has none, which is the case the extension point
exists for.

It is obliged to cover every value of `data_type` the model reports for a column
of the three schemas, per `FR-EX-008`. That is **39 values over 362 columns**,
and all 39 are branched on. Nothing falls through: the final branch calls
`fail()`, so a type the macro has never met ends the render with `65` and names
the column. Removing the `uuid` branch and re-rendering `freight.customer`
produces exactly that —

```
cause: python/_types.jinja maps no Python type onto the column customer.portal_uuid,
whose column_type is uuid. …
exit:  65 (EX_DATAERR)
```

— which is how the guard was shown not to be vacuous.

**Every row of the table below was measured against the server of `FR-EX-006`**,
not inferred: a real non-`NULL` value of a real column of each of the 39 types
was read through PyMySQL 1.2.3 and its `type()` recorded. Nothing here comes
from a driver's documentation.

| `data_type` | Python type | Only in `freight` |
|---|---|:-:|
| `tinyint` | `int` | |
| `smallint` | `int` | |
| `mediumint` | `int` | |
| `int` | `int` | |
| `bigint` | `int` | • |
| `year` | `int` | |
| `float` | `float` | • |
| `double` | `float` | • |
| `decimal` | `decimal.Decimal` | |
| `char` | `str` | |
| `varchar` | `str` | |
| `tinytext` | `str` | • |
| `text` | `str` | |
| `mediumtext` | `str` | • |
| `longtext` | `str` | • |
| `enum` | `str` | |
| `set` | `str` | |
| `inet4` | `str` | • |
| `inet6` | `str` | • |
| `uuid` | `str` | • |
| `bit` | `bytes` | • |
| `binary` | `bytes` | • |
| `varbinary` | `bytes` | • |
| `tinyblob` | `bytes` | • |
| `blob` | `bytes` | |
| `mediumblob` | `bytes` | • |
| `longblob` | `bytes` | • |
| `geometry` | `bytes` | • |
| `geometrycollection` | `bytes` | • |
| `linestring` | `bytes` | • |
| `multilinestring` | `bytes` | • |
| `multipoint` | `bytes` | • |
| `multipolygon` | `bytes` | • |
| `point` | `bytes` | • |
| `polygon` | `bytes` | • |
| `date` | `datetime.date` | • |
| `datetime` | `datetime.datetime` | |
| `timestamp` | `datetime.datetime` | |
| `time` | `datetime.timedelta` | • |

**25 of the 39 are carried by `freight` alone**, which is why `FR-EX-006` puts a
third schema beside the two published ones. A mapping written against `sakila`
and `world` would have been complete over 14 types and would never have met a
spatial column, a `BIT`, an `INET6`, a `UUID`, a `TIME`, a generated column or a
system-versioned table.

The mapping needs **no dependency beyond PyMySQL**: `decimal` and `datetime` are
the standard library's, and the eight spatial types and `BIT` reach the caller as
`bytes`.

### The opinions, and where Python's answer is the better one

- **`decimal` is `decimal.Decimal`**, which is lossless and costs nothing. This
  is where Python does better than the other two members of the set: Go had to
  settle for a `string` because `float64` cannot hold `decimal(38,10)`, and Rust
  had to render an adapter because `sqlx` decodes a `DECIMAL` into a third-party
  crate's type or into nothing.
- **`time` is `datetime.timedelta`**, and so is what PyMySQL returns: measured,
  `06:00:00` arrives as `timedelta(seconds=21600)`. A MariaDB `TIME` is a signed
  interval of up to ±838:59:59, which is not a time of day, and `timedelta` is
  the type that says so. `datetime.time` would refuse every value outside a day.
  Go mapped it to a `string` and Rust to `sqlx`'s own `MySqlTime`; the standard
  library already had the right type.
- **Width and signedness are not expressed.** A Python `int` is unbounded, so
  there is no `int8` to narrow to and no unsigned form to distinguish, and
  `column.unsigned` is read nowhere in the macro. It is the one place this
  mapping differs from Go's and Rust's in kind rather than in spelling.
- **A `tinyint(1)` is an `int` and not a `bool`.** The catalogue reports
  `tinyint` for `tinyint(1)` and for `tinyint(3)` alike — `FR-EX-008` obliges
  the mapping over `data_type`, which does not carry the display width — and
  PyMySQL returns `1`, not `True`. `freight.vessel.in_service`, declared
  `BOOLEAN`, is therefore `in_service: int`.
- **`uuid`, `inet4` and `inet6` are `str`.** Measured: a MariaDB `UUID` arrives
  as its 36 characters of text and not as 16 bytes, and the two address types
  arrive as character data. `uuid.UUID` and `ipaddress.IPv6Address` would be a
  claim about a conversion nothing performs.
- **A JSON column has no branch**, because the model never reports one: MariaDB
  reports it as `longtext`, per `FR-CAT-038`. `freight.customer.preferences` is
  that column, and it is a `str`.

### The one mapping that does not round-trip, and the measurement that says so

`bit` is `bytes`, because that is what PyMySQL returns — `bit(8)` carrying `7`
arrives as `b'\x07'`. Binding those same bytes back does **not** match:

```
SELECT COUNT(*) FROM consignment WHERE notification_flags = %s   -- b'\x07'  ->  0
SELECT COUNT(*) FROM consignment WHERE notification_flags = %s   -- 7        ->  1
```

Both measured on this server. MariaDB compares a `BIT` with a binary string by
converting the string to a number, and `b'\x07'` is not a numeral. An equality
condition over a `BIT` column therefore matches nothing, and the mapping says so
here rather than in a comment nobody reads. A project that wants arithmetic
replaces one branch of one macro with `int` and installs a decoder of its own
through PyMySQL's `conv` argument — which is the whole reason the mapping lives
in this example and not in `tpl`.

The eight spatial types do not have this problem, and that was measured too: a
geometry column arrives as a four-byte SRID followed by well-known binary, and
`WHERE reference_marker = %s` with those same bytes matches the row.

### Nullability

A nullable column is not the same Python type as a non-nullable one:

| Column | Python field |
|---|---|
| `first_name varchar(45) NOT NULL` | `first_name: str` |
| ``back`tick varchar(48) NULL`` | `back_tick: str \| None` |
| `gross_weight_kg decimal(12,3) NOT NULL` | `gross_weight_kg: decimal.Decimal` |
| `last_login_ipv6 inet6 NULL` | `last_login_ipv6: str \| None` |
| `survey_footprint geometry NULL` | `survey_footprint: bytes \| None` |

`X | None` is the language's own answer and it puts the difference in the type
rather than in a comment: PyMySQL returns `None` for a `NULL`, and
`mypy --strict` makes a caller narrow the field before reading it. A filter
field is always `X | None`, nullable column or not, because there `None` means
*no condition*; whether the column is `NULL` is asked through the `…_is_null`
field a nullable column also gets, so the two meanings never collide.

### Identifiers

`FR-ENV-031` passes a character outside `A-Za-z` through the naming filters
unchanged, so `pascal` renders the column `` back`tick `` as `` Back`tick ``,
which is an identifier in no language. The macro replaces the ASCII punctuation a
quoted MariaDB identifier may carry with an underscore, and
`freight.legacy_edi_field` is the table that exercises it:

| Column | Field | Enum member | Statement text |
|---|---|---|---|
| `` back`tick `` | `back_tick` | `BACK_TICK` | `` `back``tick` `` |
| `space in name` | `space_in_name` | `SPACE_IN_NAME` | `` `space in name` `` |
| `select` | `select` | `SELECT` | `` `select` `` |
| `posição` | `posição` | `POSIçãO` | `` `posição` `` |
| `Mixed Case Column` | `mixed_case_column` | `MIXED_CASE_COLUMN` | `` `Mixed Case Column` `` |

Three of those rows are Python-specific:

- **A non-ASCII letter is kept.** Python admits it in an identifier, under the
  XID rules of the Reference, so `posição` is a dataclass field. Removing it
  would be a loss of information for no gain, and both letters are NFKC-stable,
  which is the normalisation Python applies to an identifier.
- **The enum member keeps that letter's case.** `upper_snake` case-folds over
  `A-Za-z` alone, per `FR-ENV-031`, so `posição` becomes `POSIçãO`. It is a
  legal Python identifier and it is what determinism costs: a case-folding that
  knew about `ç` would be one that depended on the locale, and `NFR-DET-001`
  would not survive it.
- **`select` needs no escape.** It is a keyword of SQL and of Go, and it is not
  one of Python's. The macro still carries the whole keyword list: a name whose
  snake case is one of Python's 35 hard keywords takes a trailing underscore,
  which is the convention PEP 8 states for exactly this case. The soft keywords
  `match`, `case`, `type` and `_` are legal identifiers and are left alone. No
  column of these three schemas needs the escape, so it is a guard rather than a
  demonstration, and the compile gate is what would report its absence.

An enum member name is upper case and can therefore never be a Python keyword —
every keyword is lower case but for `None`, `True` and `False` — nor one of
`enum.Enum`'s reserved `_sunder_` names, because `FR-ENV-030` discards an empty
word and a leading underscore cannot survive it. A class name can collide with
those three, and `type_name` guards them.

### The `%` that PyMySQL would have read as a conversion

PyMySQL's paramstyle is `format`: it builds the final statement as
`query % arguments`. A `%` inside a quoted identifier would therefore be read as
the start of a conversion, and a `%` beside an `s` would silently consume an
argument — the one failure in this example that would not announce itself. Every
identifier that reaches a statement's text is passed through `quote` and then has
its `%` doubled, and every `execute` is handed a tuple rather than `None`,
because `query % arguments` is what undoes the doubling. No column of these three
schemas carries a `%`; the guard costs one filter and is applied anyway.

## The compile gate

`run.py` runs both of these over the rendered package, from the example's own
directory:

```
python -m compileall -q datalayer
mypy --strict --python-version 3.11 datalayer
```

`FR-EX-009` names two kinds of reader — a compiler where the language has one,
and its own static type checker where it has not — and Python is the second
case, so the gate is both tools: `compileall` decides whether each file is a
Python file at all, and `mypy --strict` decides whether the types agree.
Neither is sufficient alone, and the second is the one that catches a rendered
implementation whose signature drifted from its protocol.

`mypy` is pinned to `--python-version 3.11` rather than left to follow the
interpreter running the gate, so the verdict is about the version this example
claims to support and not about the one that happened to be installed. It is the
Rust example's `rust-version` in another language.

A missing toolchain is a failure that names what is missing, never a skip: the
gate's verdict is the example's acceptance signal, so a gate that did not run has
produced no verdict. The two ways it can be missing are reported separately,
because the remedy differs:

- **`mypy` will not run.** Neither `python -m mypy` nor a `mypy` on `PATH`
  answered `--version`.
- **`mypy` cannot resolve PyMySQL.** A one-line module importing PyMySQL is
  checked first, so an incomplete environment is reported as an incomplete
  environment rather than as 36 defects in the rendered code. PyMySQL ships no
  `py.typed`, so its stubs are a package of their own.

Neither tool writes into the tree. The bytecode cache goes to a temporary
directory through `PYTHONPYCACHEPREFIX` and the type-check cache through
`--cache-dir`, so the committed `datalayer/` stays exactly as rendered and a
rerun cannot read anything the last run left behind.

## Prerequisites

| | |
|---|---|
| The binary | `cargo build --release` |
| The server | `./scripts/mariadb/up.sh`, then `./scripts/mariadb/seed-datasets.sh 12.3` |
| The password | `export FIXTURE_PW=tpl-root` |
| Python | 3.11 or newer, which is the floor the gate pins. Verified on CPython 3.14.7 |

The three packages a reader installs, into a virtual environment of their own:

```sh
python3 -m pip install mypy PyMySQL types-PyMySQL
```

| Package | Why | Verified at |
|---|---|---|
| `mypy` | Half the compile gate | 2.3.1 |
| `PyMySQL` | What the rendered implementation imports, and what the type mapping was measured through | 1.2.3 |
| `types-PyMySQL` | PyMySQL ships no `py.typed`, so `mypy` needs its stubs to check the implementation at all | 1.2.0.20260807 |

`run.py` itself and `_driver` take no dependency: standard library only. The
three above are needed by the *rendered* package and by the gate that reads it,
and `run.py` prefers `python -m mypy` over a `mypy` on `PATH` precisely so that
the `mypy` it runs is the one sharing an environment with the stubs.

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
  the server, its series, its standing and the version of `tpl` — is rendered
  once per schema into `_core.py` and `__init__.py`.
- **It runs no query.** The gate reads; it does not connect. The type mapping was
  measured against the server while it was being written, and the rendered code
  carries that result rather than re-establishing it.

## Known limits

Each is a property of this example's templates, not of `tpl`, and each is visible
because the compile gate is what would report it:

- **Identifiers are sanitised, not proved.** A name beginning with a digit, and
  two names whose sanitised forms collide, still render code Python refuses. A
  collision is reported by `mypy` as `Name "…" already defined`, and a duplicate
  enumeration *value* would be silently folded into an alias by `enum` — which
  is why a member's value is the column's raw name, whose uniqueness within a
  table the server already guarantees.
- **An acronym loses its case.** `actor_id` becomes `ActorId`, not `ActorID`.
  That is what `FR-ENV-033`'s `pascal` does, and its own *accepted cost* clause
  records it; buying it back would need an initialism list, which is a second
  opinion to maintain and another way for two columns to collide.
- **A module's file name is the table's name unchanged.** `run.py` writes
  `datalayer/<schema>/<table>.py` while the import statement names the table in
  snake case, so a table whose name is not already snake case would render an
  import with no module behind it. All 36 are, and `mypy` is what would report
  the absence.
- **A table with no primary key is refused.** All 36 have one. A table without
  one ends the render with `65` and says why, because read, update and delete
  would have no key to address a row by. So is a table whose every column is
  generated or part of the key, which would leave an `UPDATE` with an empty
  `SET`.
- **An equality filter over a `BIT` column matches nothing**, for the measured
  reason given above. It is the one condition the rendered search builds that
  cannot succeed.
- **One connection, not a pool.** A PyMySQL connection carries one session and is
  not safe for concurrent use, so `Repositories` holds one and a caller working
  from several threads builds one `Repositories` per connection. `sqlx`'s pool
  let the Rust example share a handle; PyMySQL has no equivalent to share.
