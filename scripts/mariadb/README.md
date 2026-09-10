# MariaDB test fixtures

The containers `tpl` is validated against. Every statement in this directory has
been executed against all four supported MariaDB series; nothing here is
described from documentation alone.

## Contents

| File | Purpose |
|---|---|
| `Dockerfile` | One image definition, parameterised by release series |
| `setup.sql` | The `freight` schema: DDL only |
| `seed.sql` | The data, plus the statements that make the triggers, the sequence and the system-versioned table actually fire |
| `README.md` | This file |

The Dockerfile copies `setup.sql` to `/docker-entrypoint-initdb.d/01-setup.sql`
and `seed.sql` to `02-seed.sql`. The rename is load-bearing: the official
entrypoint runs that directory in collation order, and `seed.sql` sorts before
`setup.sql`.

## Supported series

The four series are fixed by `specification/server-contract.md` (`FR-SRV-015`).
Each gets its own image tag, its own container and its own published port, so
all four can run side by side.

| Series | Image tag | Container | Host port |
|---|---|---|---|
| `10.11` | `tpl-mariadb:10.11` | `tpl-mariadb-10.11` | `13306` |
| `11.4` | `tpl-mariadb:11.4` | `tpl-mariadb-11.4` | `13307` |
| `11.8` | `tpl-mariadb:11.8` | `tpl-mariadb-11.8` | `13308` |
| `12.3` | `tpl-mariadb:12.3` | `tpl-mariadb-12.3` | `13309` |

The upstream `mariadb:<series>` tags all publish a `linux/arm64` image as well
as `linux/amd64`, so the fixtures run unmodified on both supported
architectures.

## Building

One build per series. `MARIADB_SERIES` selects the upstream base image:

```sh
cd scripts/mariadb
docker build --build-arg MARIADB_SERIES=10.11 -t tpl-mariadb:10.11 .
docker build --build-arg MARIADB_SERIES=11.4  -t tpl-mariadb:11.4  .
docker build --build-arg MARIADB_SERIES=11.8  -t tpl-mariadb:11.8  .
docker build --build-arg MARIADB_SERIES=12.3  -t tpl-mariadb:12.3  .
```

Add `--platform linux/arm64` (or `linux/amd64`) to pin the architecture
explicitly.

Editing `setup.sql` or `seed.sql` requires a rebuild **and** a fresh container:
the entrypoint runs the init directory only when the data directory is empty, so
an existing container will not pick the change up.

## Running

```sh
docker run -d --name tpl-mariadb-10.11 -e MARIADB_ROOT_PASSWORD=tpl-root -p 13306:3306 tpl-mariadb:10.11
docker run -d --name tpl-mariadb-11.4  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13307:3306 tpl-mariadb:11.4
docker run -d --name tpl-mariadb-11.8  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13308:3306 tpl-mariadb:11.8
docker run -d --name tpl-mariadb-12.3  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13309:3306 tpl-mariadb:12.3
```

First start takes a few seconds longer than later ones, because the entrypoint
initialises the data directory and then runs both SQL files.

### Confirming a container came up clean

A failure inside `docker-entrypoint-initdb.d` is reported in the log and leaves
the schema incomplete. Container status alone does not tell you. Read the log:

```sh
docker logs tpl-mariadb-10.11 2>&1 | grep -E 'ERROR|Ready for start up'
```

A healthy start ends with `MariaDB init process done. Ready for start up.` and
contains no `ERROR` line.

Then confirm the schema itself:

```sh
docker exec tpl-mariadb-10.11 mariadb -uroot -ptpl-root \
  -e "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA='freight'"
```

The expected answer is `23` on every series: sixteen base tables, one
system-versioned table, five views and one sequence.

Warnings about `memory.pressure` and `io_uring_queue_init()` appear on a healthy
start on all four series; both come from the container runtime's kernel. On
`10.11` alone the log also carries `[Warning] You need to use --log-bin to make
--expire-logs-days ... work.` None of them indicates a problem with the fixture.

## Connecting

### Through `docker exec`

The client inside the container connects over the Unix socket, which sidesteps
the TLS difference described below. This is the recommended way to interrogate a
fixture by hand.

```sh
docker exec -it tpl-mariadb-11.4 mariadb -uroot -ptpl-root freight
docker exec -i  tpl-mariadb-11.4 mariadb -uroot -ptpl-root freight < some-query.sql
```

### Over TCP from the host

```sh
mariadb -h 127.0.0.1 -P 13307 -u tpl_reader -ptpl-reader-pw freight
```

**The `10.11` fixture needs `--skip-ssl`.** MariaDB 11.4 introduced automatic
generation of a self-signed certificate on first start; 10.11 has none, and
reports `have_ssl=DISABLED`. The refusal is the **MariaDB client's default TLS
mode**, not a property of modern clients in general. An `11.4` MariaDB client
and a `12.3` MariaDB client both fail against `10.11` over TCP with:

```
ERROR 2026 (HY000): TLS/SSL error: SSL is required, but the server does not support it
```

Both succeed with `--skip-ssl`. A `10.11` MariaDB client reaches a `12.3` server
unmodified, and a MySQL client connects to `10.11` over TCP with no flags at
all.

So, for 10.11 only:

```sh
mariadb -h 127.0.0.1 -P 13306 -u tpl_reader -ptpl-reader-pw --skip-ssl freight
```

The other three accept a TLS connection without further configuration.

## Credentials

| User | Password | Privileges |
|---|---|---|
| `root` | `tpl-root` | Everything. Created by the entrypoint from `MARIADB_ROOT_PASSWORD`, and reachable from any host |
| `tpl_reader` | `tpl-reader-pw` | `SELECT, EXECUTE ON freight.*` and nothing more |

`tpl_reader` exists to make an incomplete catalogue read reproducible without
breaking anything. It can list every table and read every row, and the routines
appear to it, but it holds neither `SHOW VIEW` nor the privilege that exposes a
routine body, and it loses the constraint metadata entirely. Observed
identically on all four series:

| Read | As `root` | As `tpl_reader` |
|---|---|---|
| `information_schema.TABLES` rows for `freight` | 23 | 23 |
| `VIEWS.VIEW_DEFINITION` | full text | empty string, length 0 |
| `ROUTINES.ROUTINE_DEFINITION` | full body | `NULL` |
| `ROUTINES` rows | 7 | 7 |
| `TRIGGERS` rows | 6 | 0 |
| `CHECK_CONSTRAINTS` rows | 24 | 24 |
| `TABLE_CONSTRAINTS` rows | 68 | 0 |
| `REFERENTIAL_CONSTRAINTS` rows | 15 | 0 |
| `KEY_COLUMN_USAGE` rows | 54 | 54 |

The largest absence is the last group. `tpl_reader` sees no row at all in
`TABLE_CONSTRAINTS` or `REFERENTIAL_CONSTRAINTS`, so it loses every foreign-key
rule and the whole constraint list, while still seeing the foreign-key *columns*
through `KEY_COLUMN_USAGE`, which is unaffected.

The three absences take three different shapes, which matters to a reader that
has to detect them: `VIEW_DEFINITION` comes back as the empty string,
`ROUTINE_DEFINITION` as `NULL`, and `TABLE_CONSTRAINTS`,
`REFERENTIAL_CONSTRAINTS` and `TRIGGERS` as zero rows.

## What the schema contains

`freight` models a freight-forwarding and maritime logistics operation. It is
not a type gallery wearing a costume — the ports, the UN/LOCODEs, the ships and
their IMO numbers are real, the companies are invented, and the rates are
plausible. It is built to exercise, in one schema:

- every native data type available in all four series, including `BIT`,
  `ENUM`, `SET`, `JSON`, `UUID`, `INET4`, `INET6` and all eight OGC geometry
  types, with fractional-second precision variants of `TIME`, `DATETIME` and
  `TIMESTAMP`;
- simple and composite primary keys, and composite foreign keys;
- unique, composite, prefix, descending, `FULLTEXT`, `SPATIAL`, and
  generated-column indexes;
- fifteen foreign keys, no two sharing the same `(ON DELETE, ON UPDATE)` pair
  as written, between them naming all five referential actions. The catalogue
  reports only fourteen distinct pairs, because `SET DEFAULT` is downgraded to
  `RESTRICT` on the way in — see the note at the end of the next section;
- generated columns, both `VIRTUAL` and `STORED`;
- column-level and table-level `CHECK` constraints;
- an `AUTO_INCREMENT` column and an `INVISIBLE` column;
- four kinds of default on adjacent columns — literal, expression, explicit
  `DEFAULT NULL`, and none at all;
- `ENUM` members containing an apostrophe and members containing a comma, and a
  `SET` member containing an apostrophe;
- per-column character sets and collations that differ from the table default;
- table and column comments, several of them non-ASCII;
- five views: simple, updatable, `WITH LOCAL CHECK OPTION`,
  `WITH CASCADED CHECK OPTION`, and a multi-table join;
- three procedures covering `IN`, `OUT` and `INOUT`, and four functions with
  four distinct return types;
- all six trigger kinds, `BEFORE` and `AFTER` across `INSERT`, `UPDATE` and
  `DELETE`;
- one `SYSTEM VERSIONED` table and one `SEQUENCE`;
- `legacy_edi_field`, a small table whose only job is identifier edge cases: a
  column name containing a backtick, one containing a space, one that is a
  reserved word, one with non-ASCII letters, and an index name containing a
  space.

`seed.sql` ends by booking a consignment through a stored procedure, updating
two others, deleting a third and superseding a tariff, so that the triggers have
fired, the sequence has been drawn from and the system-versioned table holds a
history row by the time the container reports itself ready.

## Deliberate omissions

Anything a supported series rejects is absent, rather than hidden behind a
conditional. The identical DDL has to be accepted by all four.

| Left out | Why |
|---|---|
| `VECTOR` columns and vector indexes | Rejected by `10.11` and `11.4` with `ERROR 4161 (HY000): Unknown data type: 'VECTOR'`. Available from `11.7`, which is a rolling release and unsupported, so the first supported series to accept it is `11.8` |
| A `SET` member containing a comma | Rejected by all four with `ERROR 1367 (22007): Illegal set 'y, z' value found during parsing`. `SET` values are stored comma-separated, so the delimiter cannot appear in a member. The comma requirement is met by `ENUM` instead, which accepts it |
| `utf8mb4_uca1400_*` collations | Accepted by all four, but the default on `11.4`, `11.8` and `12.3`, which would make an explicitly declared schema collation indistinguishable from an inherited one. `utf8mb4_unicode_520_ci` is used instead: available everywhere, default nowhere |

## What the fixture does not exercise

The schema's reach is not the data's, and three gaps are worth naming so that a
later reader does not mistake one for the other.

- The `INET4`, `INET6` and `UUID` columns exist and are catalogued correctly,
  but no row anywhere holds a non-`NULL` value in one of them.
- No comment contains a supplementary-plane (four-byte) character, which is
  worth knowing because the `INFORMATION_SCHEMA` comment columns are `utf8mb3`:
  such a character is silently replaced by `?` when the DDL is parsed, with
  `warning_count` left at 0.
- `tariff` uses implicit system versioning, so its `ROW_START` and `ROW_END`
  columns stay hidden and `IS_SYSTEM_TIME_PERIOD_START` and
  `IS_SYSTEM_TIME_PERIOD_END` never read `YES` for any column.

## Differences observed between the series

The `freight` catalogue was dumped from all four servers and compared field by
field, and `INFORMATION_SCHEMA` itself was compared table by table. Seven
differences were found.

1. **Default server collation.** `10.11` runs `utf8mb4_general_ci`; `11.4`,
   `11.8` and `12.3` run `utf8mb4_uca1400_ai_ci`. The character set is `utf8mb4`
   on all four. This propagates into `VIEWS.COLLATION_CONNECTION`, which records
   the session collation in force when each view was created, and into the
   corresponding attribute of every stored routine and trigger.

2. **Position of `INVISIBLE` in `SHOW CREATE TABLE`.** `10.11` and `11.4` emit
   `varchar(24) ... INVISIBLE DEFAULT NULL COMMENT '...'`; `11.8` and `12.3`
   emit `varchar(24) ... DEFAULT NULL INVISIBLE COMMENT '...'`. The
   `INFORMATION_SCHEMA.COLUMNS.EXTRA` value is `INVISIBLE` on all four, so a
   reader that goes through `INFORMATION_SCHEMA` is unaffected and one that
   parses `SHOW CREATE TABLE` is not.

3. **TLS availability.** `10.11` reports `have_ssl=DISABLED`; the other three
   report `have_ssl=YES`, from the self-signed certificate MariaDB began
   generating automatically at first start in 11.4. Consequences: a **MariaDB**
   client of 11.4 or later cannot reach the `10.11` fixture over TCP without
   `--skip-ssl`, while a `10.11` MariaDB client reaches a `12.3` server
   unmodified and a MySQL client reaches `10.11` with no flags at all.
   Connections made through `docker exec` use the Unix socket and are
   unaffected.

4. **Temporary tables in `INFORMATION_SCHEMA.TABLES`.** After a `CREATE
   TEMPORARY TABLE`, `10.11` returns no row for it at all; `11.4`, `11.8` and
   `12.3` return one, with `TABLE_TYPE='TEMPORARY'`. The set of `TABLE_TYPE`
   values a server can emit is therefore `BASE TABLE`, `SEQUENCE`, `SYSTEM
   VERSIONED`, `SYSTEM VIEW` and `VIEW` on `10.11`, and those five plus
   `TEMPORARY` on `11.4` and later. On no series does such a table appear in
   `INFORMATION_SCHEMA.COLUMNS` or `INFORMATION_SCHEMA.STATISTICS`.

5. **Width of `INFORMATION_SCHEMA.COLUMNS`.** It has 22 columns on `10.11` and
   24 on `11.4` and later, which add `IS_SYSTEM_TIME_PERIOD_START` and
   `IS_SYSTEM_TIME_PERIOD_END`.

6. **Width of `INFORMATION_SCHEMA.PARAMETERS`.** It has 16 columns on `10.11`,
   `11.4` and `11.8`, and 17 on `12.3`, which adds `PARAMETER_DEFAULT`.

7. **`INFORMATION_SCHEMA.PERIODS` does not exist on `10.11`.** Querying it there
   fails with `ERROR 1109 (42S02): Unknown table 'PERIODS' in
   information_schema`. On `11.4`, `11.8` and `12.3` the table is present and
   empty.

Differences 5, 6 and 7 share a consequence, and it is the practical point of the
three: **a `SELECT` naming a fixed column list against
`INFORMATION_SCHEMA.COLUMNS` or `INFORMATION_SCHEMA.PARAMETERS` cannot run
unmodified on all four series.** Naming a column that a series does not have is
a hard `ERROR 1054 (42S22)`, not a `NULL` and not a warning, so a reader either
probes the shape of these tables before querying them or restricts itself to the
`10.11` intersection.

Everything else matched exactly: column types, nullability, defaults, generation
expressions, character sets and collations, comments, all fifteen referential
constraints, all twenty-four check constraints, every index including the
descending, prefix, spatial, fulltext and generated-column ones, all five views
with their updatability and check-option attributes, all seven routines with
their parameter modes and return types, all six triggers, and the sequence.
`11.8` and `12.3` produced byte-identical dumps.

`SET DEFAULT` deserves a line of its own, because it is the one referential
action that does not survive the round trip. `ON DELETE SET DEFAULT ON UPDATE
SET DEFAULT` on `fk_container_type` is **accepted without error or warning by
all four series, and then reported as `RESTRICT`** in
`REFERENTIAL_CONSTRAINTS`; `SHOW CREATE TABLE` omits the clause entirely. The
DDL keeps it because that silent downgrade is itself the behaviour worth
observing.

## Stopping and removing

```sh
docker stop tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3
docker rm   tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3
```

Or in one step, discarding the anonymous volume the entrypoint created with it:

```sh
docker rm -f -v tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3
```

Leave no container running after a validation run. To drop the images too:

```sh
docker rmi tpl-mariadb:10.11 tpl-mariadb:11.4 tpl-mariadb:11.8 tpl-mariadb:12.3
```
