# MariaDB test fixtures

The containers `tpl` is validated against, and the harness that drives and
instruments them. Every claim in this directory was produced by a command that
was run, and nothing here is described from documentation alone; where an
observation was made against one series rather than all four, it says which.

## Contents

| File | Purpose |
|---|---|
| `Dockerfile` | One image definition, parameterised by release series |
| `setup.sql` | The `freight` schema: DDL only |
| `seed.sql` | The data, plus the statements that make the triggers, the sequence and the system-versioned table actually fire |
| `seed-bench.sql` | The two benchmark workloads of `NFR-PERF-001`, `WL-001` and `WL-003`: DDL only, loaded on demand and not in the image |
| `datasets/sakila/` | The Sakila sample database, vendored verbatim, with its licence and its provenance note: loaded on demand and not in the image |
| `datasets/world/` | The World sample database, vendored verbatim, with its provenance note: loaded on demand and not in the image |
| `tls/generate.sh` | Regenerates the TLS material below |
| `tls/openssl.cnf` | The certificate profile: what the certificate says, including the names it carries |
| `tls/ca.pem` | The fixture's root certificate, and the file to pass as `ca_file` |
| `tls/server-cert.pem` | The certificate the server presents |
| `tls/server-key.pem` | Its private key |
| `tls/server-tls.cnf` | The three server settings that put the material into service |
| `up.sh` | Starts every server and does not return until each is listening and verified |
| `down.sh` | Stops and removes them, and proves nothing of the fixture is left |
| `status.sh` | The gate: whether the fixture is up, answered without a client |
| `seed-bench.sh` | Loads `seed-bench.sql` into a running server and counts what arrived |
| `seed-datasets.sh` | Loads `sakila` and `world` into a named running server and counts what arrived |
| `observe.sh` | The three instruments of `NFR-PERF-007` that need a server or a tracer, and the reading that identifies a build |
| `series.env` | The inventory — one record per server — and the helpers the scripts share |
| `probe-session.sql` | The connection-start sequence of `FR-SRV-006`, for a substitute client |
| `observer.Dockerfile` | The tracer image `observe.sh opens` falls back to |
| `README.md` | This file |

The Dockerfile copies `setup.sql` to `/docker-entrypoint-initdb.d/01-setup.sql`
and `seed.sql` to `02-seed.sql`. The rename is load-bearing: the official
entrypoint runs that directory in collation order, and `seed.sql` sorts before
`setup.sql`.

It copies the three `tls/` certificate files to `/etc/mysql/tls/` and
`tls/server-tls.cnf` to `/etc/mysql/conf.d/90-tls.cnf`, which
`/etc/mysql/my.cnf` includes last. See [TLS](#tls).

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

Port `13310` is reserved for a fifth container that is not a fifth series: the
server offering no TLS, described under [TLS](#tls). It runs the `10.11` image
with one extra flag.

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

No preparatory step is needed: the TLS material the build copies in is
committed. Changing it is a rebuild too — see [TLS](#tls).

## Running

`./up.sh` starts all of it and `./down.sh` stops all of it; see
[The harness](#the-harness). The commands below are what those two scripts run,
and are here because a fixture whose only documentation is a script is a fixture
nobody can check.

```sh
docker run -d --name tpl-mariadb-10.11 -e MARIADB_ROOT_PASSWORD=tpl-root -p 13306:3306 tpl-mariadb:10.11
docker run -d --name tpl-mariadb-11.4  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13307:3306 tpl-mariadb:11.4
docker run -d --name tpl-mariadb-11.8  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13308:3306 tpl-mariadb:11.8
docker run -d --name tpl-mariadb-12.3  -e MARIADB_ROOT_PASSWORD=tpl-root -p 13309:3306 tpl-mariadb:12.3
```

Every one of those four offers TLS, with the certificate described under
[TLS](#tls). The fifth container is the one that offers none:

```sh
docker run -d --name tpl-mariadb-notls -e MARIADB_ROOT_PASSWORD=tpl-root -p 13310:3306 tpl-mariadb:10.11 --skip-ssl
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

## The harness

Eight files drive the fixture and instrument it. They exist because `NFR-PERF-007`
makes the instrument *the* verification: a requirement of form is checked by an
observation made outside the process and never by reading the source, so each
observation has to be a command somebody can run and an output somebody can
read. Everything below was run against this fixture, and every output shown is
the output it produced.

| File | What it is |
|---|---|
| `series.env` | The inventory and the shared helpers. Sourced, never run |
| `up.sh` | Starts the servers and verifies each one |
| `down.sh` | Removes them and proves nothing is left |
| `status.sh` | The gate |
| `seed-bench.sh` | Loads the benchmark workloads, and verifies every count they state |
| `observe.sh` | The three instruments, and the build reading beside them |
| `probe-session.sql` | The connection-start sequence of `FR-SRV-006`, for a substitute client |
| `observer.Dockerfile` | The tracer image the third instrument falls back to |

`series.env` carries the inventory the table under [Supported
series](#supported-series) states in prose — four series, one `--skip-ssl`
server, a container and a published port each — in the form the scripts read.
The scripts read nothing else, so a port changes in one executable place; the
prose table is the second copy, and it has to be changed with it.

### Bringing it up

```sh
cd scripts/mariadb
./up.sh                # all five
./up.sh 11.8 notls     # only the named servers
```

It is idempotent: it builds an image only when it is missing and starts a
container only when it is not already running. From nothing:

```
10.11
  image   tpl-mariadb:10.11 already built
  server  tpl-mariadb-10.11 started on :13306
  listen  tpl-mariadb-10.11 answers on :13306 as 5.5.5-10.11.19-MariaDB-ubu2204
  ready   tpl-mariadb-10.11: 23 catalogue objects in freight, no [ERROR] in log
11.4
  image   tpl-mariadb:11.4 already built
  server  tpl-mariadb-11.4 started on :13307
  listen  tpl-mariadb-11.4 answers on :13307 as 11.4.13-MariaDB-ubu2404
  ready   tpl-mariadb-11.4: 23 catalogue objects in freight, no [ERROR] in log
11.8
  image   tpl-mariadb:11.8 already built
  server  tpl-mariadb-11.8 started on :13308
  listen  tpl-mariadb-11.8 answers on :13308 as 11.8.9-MariaDB-ubu2404
  ready   tpl-mariadb-11.8: 23 catalogue objects in freight, no [ERROR] in log
12.3
  image   tpl-mariadb:12.3 already built
  server  tpl-mariadb-12.3 started on :13309
  listen  tpl-mariadb-12.3 answers on :13309 as 12.3.3-MariaDB-ubu2404
  ready   tpl-mariadb-12.3: 23 catalogue objects in freight, no [ERROR] in log
notls
  image   tpl-mariadb:10.11 already built
  server  tpl-mariadb-notls started on :13310 (--skip-ssl)
  listen  tpl-mariadb-notls answers on :13310 as 5.5.5-10.11.19-MariaDB-ubu2204
  ready   tpl-mariadb-notls: 23 catalogue objects in freight, no [ERROR] in log
```

19.7 seconds on an Apple Silicon host with the four images already built.

Each server is checked twice, because neither check alone is the check. The
`listen` line is the published port answering; the `ready` line is the log
carrying no `[ERROR]` and the schema holding its 23 catalogue objects, which is
what a failure inside `/docker-entrypoint-initdb.d` would break while the
container still reported itself up.

**Readiness is the published port, never a client inside the container.** The
official entrypoint runs a temporary server with `--skip-networking` while the
init scripts execute: it answers `SELECT 1` over the Unix socket, and it then
goes away to be replaced by the real one. A `docker exec` readiness check
therefore returns true in the middle of initialisation and hands the caller a
server about to restart — observed here as

```
ERROR 2002 (HY000): Can't connect to local server through socket '/run/mysqld/mysqld.sock' (2)
```

from the step that ran immediately after it. The published port is up only when
the server under test is, so that is what `up.sh` waits for, and it waits for it
from outside every container.

**A server that cannot be made healthy is removed, not left behind.** `up.sh`
prints the container's last 30 log lines and then removes it: a failed run is
still a run, and the rule that a run leaves nothing running does not have an
exception for the runs that went wrong.

### Taking it down

```sh
./down.sh              # all five
./down.sh 11.8         # only the named servers
./down.sh --images     # ...and drop the four images too
```

```
  removed   tpl-mariadb-10.11
  removed   tpl-mariadb-11.4
  removed   tpl-mariadb-11.8
  removed   tpl-mariadb-12.3
  removed   tpl-mariadb-notls

removed 5 container(s); docker ps lists no tpl-mariadb container
```

The last line is not an intention. `down.sh` asks the daemon what is left of the
fixture and fails if the answer is not nothing:

```sh
docker ps --filter 'name=tpl-mariadb' --format '{{.Names}}' | wc -l
```
```
0
```

It removes exactly the containers `series.env` names, because the host may be
running containers that belong to other projects, and it removes the anonymous
volume each one created with it: the entrypoint populates a data directory only
when it is empty, so a volume that outlives its container is a stale schema
waiting to be mistaken for a fresh one. It is safe to run when nothing is up and
safe to run twice.

### The gate

`status.sh` answers the question a server-dependent test has to ask before it
runs. It probes each published port the way a test reaches it — from the host,
over TCP — and needs **no client installed**, which is the point: a contributor
may have none and must still be able to ask.

The mechanism is the MariaDB handshake. A server sends its greeting the moment
the socket opens, and the greeting carries the version, so what is reported is
"a MariaDB of this series answered here", not "something is listening". Bash's
`/dev/tcp` opens the socket, one byte is read with a timeout to bound a port
that accepts and then says nothing, and the version is taken from the greeting.

```sh
./status.sh
```
```
SERVER  PORT    TLS   STATE VERSION
10.11   13306   yes   up    5.5.5-10.11.19-MariaDB-ubu2204
11.4    13307   yes   up    11.4.13-MariaDB-ubu2404
11.8    13308   yes   up    11.8.9-MariaDB-ubu2404
12.3    13309   yes   up    12.3.3-MariaDB-ubu2404
notls   13310   no    up    5.5.5-10.11.19-MariaDB-ubu2204
```

The exit code is the gate, and it has three values rather than two:

| Exit | Meaning | What a test runner does |
|---|---|---|
| `0` | Every requested server answered | Run the server tests |
| `1` | None answered | Skip them, and say why |
| `2` | Some answered and some did not | A broken fixture, not an absent one. Do **not** skip |

All three were observed. With nothing running:

```
SERVER  PORT    TLS   STATE VERSION
10.11   13306   yes   down  -
11.4    13307   yes   down  -
11.8    13308   yes   down  -
12.3    13309   yes   down  -
notls   13310   no    down  -
exit=1
```

and with one of the five stopped:

```
10.11   13306   yes   up    5.5.5-10.11.19-MariaDB-ubu2204
11.4    13307   yes   down  -
11.8    13308   yes   up    11.8.9-MariaDB-ubu2404
12.3    13309   yes   up    12.3.3-MariaDB-ubu2404
notls   13310   no    up    5.5.5-10.11.19-MariaDB-ubu2204
exit=2
```

The third value is the one that earns its keep. A runner that treats "no
fixture" and "half a fixture" alike either skips silently over a real failure or
fails the build of every contributor who has no Docker.

For a runner that needs the answer as data rather than as an exit code:

```sh
eval "$(./status.sh --export)"
```
```
TPL_MARIADB_10_11=127.0.0.1:13306; export TPL_MARIADB_10_11
TPL_MARIADB_11_4=127.0.0.1:13307; export TPL_MARIADB_11_4
TPL_MARIADB_11_8=127.0.0.1:13308; export TPL_MARIADB_11_8
TPL_MARIADB_12_3=127.0.0.1:13309; export TPL_MARIADB_12_3
TPL_MARIADB_NOTLS=127.0.0.1:13310; export TPL_MARIADB_NOTLS
TPL_MARIADB_READY=all; export TPL_MARIADB_READY
```

`TPL_MARIADB_READY` is `all`, `none` or `partial`, matching the three exit
codes. `./status.sh --quiet` prints nothing and exits with the code alone.

**The gate is visible on the server, and a test that counts connections must
know it.** One `status.sh` run adds 1 to `Connections` and 1 to
`Aborted_connects` on each server it probes, because it opens a TCP connection,
reads the greeting and closes without authenticating. Measured on `12.3` around
one run, with the other four deltas accounted for by the observer's own
connections:

```
before:  Aborted_connects 3   Connections 24
after:   Aborted_connects 4   Connections 28
```

The general log is not affected: an unauthenticated probe produces no `Connect`
row, so the log-based connection count is immune to the gate while the status
counter is not. Take the counter baseline **after** the gate, not before it.

### The benchmark workloads

`NFR-PERF-001` states its requirement over two reference workloads, `WL-001`
and `WL-003`, and requires the number of catalogue statements a full read issues
over one to **equal** the number it issues over the other. Neither workload is
`freight`: `BR-PERF-002` keeps them apart, because `setup.sql` and `seed.sql`
are exhaustive variety at minimal volume, for correctness, and the benchmark
fixture is volume at minimal variety, for measurement. One fixture serving both
would hide an N+1 — invisible at 23 objects — or would make the correctness
suite pay for 200 tables on every run.

`seed-bench.sql` realises both, as two schemas:

| Schema | Workload | What it holds |
|---|---|---|
| `freight_wl001` | `WL-001`, the large workload | 200 tables, 2 400 columns, 600 indexes, 180 foreign keys, 40 generated columns, 25 triggers, 30 views, 40 routines, and a comment on 120 of the 200 tables |
| `freight_wl003` | `WL-003`, the small workload | one table, 12 columns, 3 indexes |

**It is not in the image**, and that is deliberate on the same ground: the
Dockerfile copies `setup.sql` and `seed.sql` into
`/docker-entrypoint-initdb.d`, so every container carries `freight` from the
moment it starts, and a correctness run must not pay for the benchmark
workload as well. `seed-bench.sh` is how the workload gets in.

**It seeds no rows.** Both workloads are defined by catalogue volume, every
measurement stated over them reads `INFORMATION_SCHEMA`, and a row changes no
count the file is answerable for. What the file seeds is the catalogue.

#### Loading it

```sh
./up.sh                        # the workload needs a server to go into
./seed-bench.sh                # load into all five, then verify
./seed-bench.sh 11.8 notls     # only the named servers
./seed-bench.sh --verify       # count what is there; load nothing
./seed-bench.sh --drop         # drop both schemas
```

Loading is idempotent — the file drops each schema before creating it — and the
script asks the gate before it starts, because a load against a server that is
not up fails halfway and leaves a partial schema behind. Its exit code is `0`
when every requested server holds both workloads at every stated count, `1` when
one does not, and `2` when the invocation is wrong or the fixture is down.

Never load it by hand. A `docker exec … < seed-bench.sql` puts the DDL in and
skips the twelve counts below, which is the whole of the verification that the
schema it created is the workload the specification names.

#### What it verifies, and how each quantity is counted

The counting rule is stated once, here and beside the statement in
`seed-bench.sh` that applies it, because several of these can be counted more
than one way and a figure whose rule is unstated is not a figure:

| Quantity | Counted as |
|---|---|
| tables | `TABLES` rows with `TABLE_TYPE = 'BASE TABLE'`; a view is not a table |
| columns | `COLUMNS` rows belonging to those base tables, generated columns included |
| indexes | **distinct** `(TABLE_NAME, INDEX_NAME)` pairs in `STATISTICS`; `PRIMARY` is one of them, and a composite index is one index and not one per column |
| foreign_keys | `REFERENTIAL_CONSTRAINTS` rows |
| generated | `COLUMNS` rows of those base tables whose `EXTRA` names a generated-column storage |
| triggers | `TRIGGERS` rows |
| views | `TABLES` rows with `TABLE_TYPE = 'VIEW'` |
| routines | `ROUTINES` rows, procedures and functions together |
| commented_tables | base tables whose `TABLE_COMMENT` is not empty |

The index rule is the one that decides a design in the file. An index on a
foreign-key column is declared **explicitly, before the constraint that needs
it**, so InnoDB adopts it instead of creating one of its own: the 180 foreign
keys contribute exactly 180 indexes and not 360, and the 600 are 200 primary,
200 secondary, 20 composite and those 180.

`WL-003` is counted on the same rules, which is what makes it comparable: its
three indexes are `PRIMARY`, one unique key and one composite key.

#### The run, on all five servers

```sh
./seed-bench.sh
```
```
10.11
  load    seed-bench.sql into tpl-mariadb-10.11
  ok      freight_wl001 (WL-001): tables=200 columns=2400 indexes=600 foreign_keys=180 generated=40 triggers=25 views=30 routines=40 commented_tables=120
  ok      freight_wl003 (WL-003): tables=1 columns=12 indexes=3 foreign_keys=0 generated=0 triggers=0 views=0 routines=0 commented_tables=1
11.4
  load    seed-bench.sql into tpl-mariadb-11.4
  ok      freight_wl001 (WL-001): tables=200 columns=2400 indexes=600 foreign_keys=180 generated=40 triggers=25 views=30 routines=40 commented_tables=120
  ok      freight_wl003 (WL-003): tables=1 columns=12 indexes=3 foreign_keys=0 generated=0 triggers=0 views=0 routines=0 commented_tables=1
11.8
  load    seed-bench.sql into tpl-mariadb-11.8
  ok      freight_wl001 (WL-001): tables=200 columns=2400 indexes=600 foreign_keys=180 generated=40 triggers=25 views=30 routines=40 commented_tables=120
  ok      freight_wl003 (WL-003): tables=1 columns=12 indexes=3 foreign_keys=0 generated=0 triggers=0 views=0 routines=0 commented_tables=1
12.3
  load    seed-bench.sql into tpl-mariadb-12.3
  ok      freight_wl001 (WL-001): tables=200 columns=2400 indexes=600 foreign_keys=180 generated=40 triggers=25 views=30 routines=40 commented_tables=120
  ok      freight_wl003 (WL-003): tables=1 columns=12 indexes=3 foreign_keys=0 generated=0 triggers=0 views=0 routines=0 commented_tables=1
notls
  load    seed-bench.sql into tpl-mariadb-notls
  ok      freight_wl001 (WL-001): tables=200 columns=2400 indexes=600 foreign_keys=180 generated=40 triggers=25 views=30 routines=40 commented_tables=120
  ok      freight_wl003 (WL-003): tables=1 columns=12 indexes=3 foreign_keys=0 generated=0 triggers=0 views=0 routines=0 commented_tables=1

every requested server holds WL-001 and WL-003 at the counts the specification states
```

Observed on 2026-09-21. **The identical DDL is accepted by all four series of
`FR-SRV-015` and by the `--skip-ssl` server**, with no `[ERROR]` line in any of
the five logs afterwards, which is the same standard `setup.sql` is held to and
is what makes a later difference between two servers a difference between the
servers.

#### The comparison `NFR-PERF-001` asks for

With both schemas loaded, the requirement's own instrument is a read of each,
counted with the [statements](#the-statements-a-server-receives) instrument. The
window is bracketed per read, and the client is `tpl` itself:

```sh
./observe.sh statements on 11.8
tpl schema dump > /dev/null            # in a project whose entry names freight_wl001
./observe.sh statements off 11.8
./observe.sh statements dump 11.8 --catalogue --count
```

Repeated for `freight_wl003`, and repeated on each of the four series, against a
project whose cache was empty so that the read reached the server:

```
SERVER   SCHEMA           COUNT
10.11    freight_wl001    11
10.11    freight_wl003    11
11.4     freight_wl001    11
11.4     freight_wl003    11
11.8     freight_wl001    11
11.8     freight_wl003    11
12.3     freight_wl001    11
12.3     freight_wl003    11
```

**Eleven against eleven, on all four series.** A database of 200 tables and one
of a single table cost the reader the same eleven catalogue statements, which is
`NFR-PERF-001` satisfied and measured rather than reviewed. The figure is also
the one `tests/schema_and_cache.rs` asserts for a full read of `freight`, whose
23 objects are a third size again — so the count is now observed across three
databases differing by two orders of magnitude in object count.

### The published datasets

`FR-EX-006` has all four worked examples read the same three schemas from one
server of the most recent supported series: `sakila`, `world` and `freight`.
`freight` is the fixture's own and is in the image. The other two are the
published MySQL sample databases, vendored under `datasets/` and loaded on
demand by `seed-datasets.sh`.

Why two datasets nobody here wrote: a reader arrives at `sakila` and `world`
already knowing what they contain, so the data layer an example renders can be
judged against a schema they recognise rather than against one the project
designed for the occasion. `freight` is the third schema because those two
reach only part of the catalogue — between them they declare no generated
column, no table comment, no sequence, no system-versioned table and no `JSON`,
`UUID`, `INET6` or `BIT` column, and a type mapping that never meets one of
those is a mapping nobody has exercised.

| Directory | Schema | What it holds | Upstream |
|---|---|---|---|
| `datasets/sakila/` | `sakila` | 16 tables, 89 columns, 41 indexes, 22 foreign keys, 6 triggers, 7 views, 6 routines; 47 268 rows | `https://downloads.mysql.com/docs/sakila-db.tar.gz` |
| `datasets/world/` | `world` | 3 tables, 24 columns, 5 indexes, 2 foreign keys, no view, routine or trigger; 5 302 rows | `https://downloads.mysql.com/docs/world-db.tar.gz` |

Each directory carries a `NOTICE.md` recording the source URL, the date the
archive was taken, its SHA-256 and the checksum of every file vendored out of
it, and a `LICENSE` where upstream ships one. **`sakila` does and `world` does
not**: the Sakila SQL files carry the BSD 3-clause text in their header, and
`world-db.tar.gz` holds a single plain `mysqldump` with no copyright notice and
no licence text at all. The absence is recorded in `datasets/world/NOTICE.md`
rather than filled in with a guess.

The SQL is vendored **byte for byte and unmodified**, which is what lets
`NOTICE.md` state a checksum that can be checked against a fresh download. The
one archive member left out is `sakila.mwb`, a MySQL Workbench binary model: it
is not SQL and nothing loads it.

**Neither is in the image**, on the same ground that keeps the benchmark
workload out of it: only the worked examples read them, and a correctness run
must not pay for 20 schemas and 113 catalogue objects it never touches.
`seed-datasets.sh` is how they get in.

#### Loading them

```sh
./up.sh                            # the datasets need a server to go into
./seed-datasets.sh 12.3            # load into 12.3, then verify
./seed-datasets.sh 12.3 11.8       # load into both
./seed-datasets.sh --verify 12.3   # count what is there; load nothing
./seed-datasets.sh --drop 12.3     # drop both schemas
```

**The server is required**, and that is the one place this script departs from
`seed-bench.sh`, which loads into all five when told nothing. `FR-EX-006` reads
one server of one series, so loading five is work no requirement asks for;
naming the server also makes a typo an error instead of a silent no-op, and an
unknown name exits `2` before anything is touched.

Loading is idempotent — each dataset drops its schema before creating it — and
the script asks the gate before it starts, because a load against a server that
is not up fails halfway and leaves a partial schema behind. Its exit code is `0`
when every named server holds both datasets at every stated count, `1` when one
does not, and `2` when the invocation is wrong or the fixture is down.

Never load them by hand. A `docker exec … < world.sql` puts the DDL in and skips
the ten checks below, which are the whole of the verification that what
arrived is what the file promised and that the examples can read it.

Nine of the ten checks are catalogue counts. The quantities and the rule each
is counted on are
[the same ones `seed-bench.sh` uses](#what-it-verifies-and-how-each-quantity-is-counted),
unchanged, so that both loaders answer the same question the same way. The
expected figures live in `series.env` beside the benchmark ones.

#### The grant, and the tenth check

Both dataset files create their schema from nothing — `sakila-schema.sql` runs
`DROP SCHEMA IF EXISTS sakila`, `world.sql` runs
`DROP DATABASE IF EXISTS world` — and a dropped schema takes its grants with
it. `seed-datasets.sh` therefore re-grants `tpl_reader` on every load, with the
same `SELECT, EXECUTE` that `setup.sql` grants it on `freight`.

That is not tidiness. `FR-EX-007` obliges all four worked examples to read the
three schemas of `FR-EX-006` through entries that "differ in nothing a read can
observe", and `INFORMATION_SCHEMA` shows a user only the objects it holds some
privilege on. Ungranted, the reader sees **nothing** of a schema that is
otherwise perfectly loaded, and an example would render an empty data layer,
exit `0` and say nothing — which is the `FR-PRIV-001` hazard `UC-013` names in
its second alternate flow.

So the script asks a tenth question, and asks it **as the reader**, because a
question asked as root answers about root. Here is the same `--verify` run
against schemas that were loaded and not granted:

```
12.3
  ok      sakila: tables=16 columns=89 indexes=41 foreign_keys=22 generated=0 triggers=6 views=7 routines=6 commented_tables=0
  ok      world: tables=3 columns=24 indexes=5 foreign_keys=2 generated=0 triggers=0 views=0 routines=0 commented_tables=0
  MISMATCH sakila: tpl_reader sees 0 objects, expected 23
  MISMATCH world: tpl_reader sees 0 objects, expected 3

seed-datasets.sh: 2 schema(s) did not match the dataset they carry
```

Nine checks pass and the tenth fails, which is the whole reason it exists. The
expected figure is the schema's `tables + views`, counted as root and compared
with what the reader can see.

#### The gate, and an unknown name

Three refusals, all before anything is touched. Against a fixture that is down:

```sh
./down.sh && ./seed-datasets.sh 12.3
```
```
seed-datasets.sh: the fixture is not up for the named servers; run ./up.sh first
```

`--verify` is refused by the same gate and for the same reason — a count taken
against a server that is not answering is not a count. Both exit `2`.

With no server named, and with one that is not in the inventory:

```
$ ./seed-datasets.sh
seed-datasets.sh: name the server to work on, for instance: seed-datasets.sh 12.3

$ ./seed-datasets.sh 12.4
seed-datasets.sh: unknown server: 12.4 (the names are in series.env)
```

Both exit `2`. The second is why the server is a required argument rather than a
filter over all five: a filter that matches nothing loads nothing and exits `0`.

#### The run

```sh
./seed-datasets.sh 12.3
```
```
12.3
  load    datasets/sakila/sakila-schema.sql into sakila on tpl-mariadb-12.3
  load    datasets/sakila/sakila-data.sql into sakila on tpl-mariadb-12.3
  grant   SELECT, EXECUTE on sakila to tpl_reader
  load    datasets/world/world.sql into world on tpl-mariadb-12.3
  grant   SELECT, EXECUTE on world to tpl_reader
  ok      sakila: tables=16 columns=89 indexes=41 foreign_keys=22 generated=0 triggers=6 views=7 routines=6 commented_tables=0
  ok      world: tables=3 columns=24 indexes=5 foreign_keys=2 generated=0 triggers=0 views=0 routines=0 commented_tables=0
  ok      sakila: tpl_reader sees 23 of 23 objects
  ok      world: tpl_reader sees 3 of 3 objects

every named server holds sakila and world at the counts series.env states
```

Observed on 2026-09-22 against `12.3.3-MariaDB-ubu2404`. **Both datasets are
accepted unmodified**, and the container log gained no `[ERROR]` line across the
load — the same standard `setup.sql` and `seed-bench.sql` are held to. The log
gained one line, and it is not from the load:

```
2026-09-22  8:31:28 259 [Warning] Aborted connection 259 to db: 'unconnected' user: 'unauthenticated' host: '172.17.0.1' (This connection closed normally without authentication)
```

That is `status.sh` asking the gate, which opens the published port, reads the
handshake and closes without authenticating. Every invocation of the gate leaves
one.

The rows are there too, which the counts above do not say:

| Table | Rows |
|---|---|
| `sakila.actor` | 200 |
| `sakila.film` | 1 000 |
| `sakila.film_text` | 1 000 |
| `sakila.payment` | 16 044 |
| `sakila.rental` | 16 044 |
| `world.city` | 4 079 |
| `world.country` | 239 |
| `world.countrylanguage` | 984 |

`sakila.film_text` is worth a glance: it is filled by the `ins_film` trigger and
not by an `INSERT` of its own, so its 1 000 rows are evidence that the triggers
loaded and fired.

#### Three things about `sakila` on MariaDB

**It declares 6 triggers, not the 3 its schema file shows.** `sakila-data.sql`
adds `customer_create_date`, `payment_date` and `rental_date` to the
`ins_film`, `upd_film` and `del_film` of `sakila-schema.sql`.

**`sakila.address` has 8 columns on MariaDB and 9 on MySQL.** Its `location`
column and the spatial index over it sit behind executable comments:

```sql
  /*!50705 location GEOMETRY */ /*!80003 SRID 0 */ /*!50705 NOT NULL,*/
```

MariaDB ignores every executable comment whose version number lies between
`50700` and `99999`, so it declares neither, and errors on neither. The 603
geometry values in `sakila-data.sql` sit behind the same `/*!50705 */` comment
and are skipped with the column, so the two files stay consistent either way —
which is why the comment is in both. The threshold was measured on this fixture
rather than taken from documentation:

```sh
. ./series.env
tpl_mariadb_sql 12.3 -N -B -e "
    SET @a=1; /*!40000  SET @a=2*/; SELECT '40000',  @a;
    SET @b=1; /*!50610  SET @b=2*/; SELECT '50610',  @b;
    SET @c=1; /*!50699  SET @c=2*/; SELECT '50699',  @c;
    SET @d=1; /*!50700  SET @d=2*/; SELECT '50700',  @d;
    SET @e=1; /*!50705  SET @e=2*/; SELECT '50705',  @e;
    SET @f=1; /*!80003  SET @f=2*/; SELECT '80003',  @f;
    SET @g=1; /*!99999  SET @g=2*/; SELECT '99999',  @g;
    SET @h=1; /*!100000 SET @h=2*/; SELECT '100000', @h;
    SET @i=1; /*!120303 SET @i=2*/; SELECT '120303', @i;
    SET @j=1; /*!120304 SET @j=2*/; SELECT '120304', @j;"
```
```
40000   2
50610   2
50699   2
50700   1
50705   1
80003   1
99999   1
100000  2
120303  2
120304  1
```

`2` means the comment ran. The server is `12.3.3`, so `120303` is its own
version and `120304` is beyond it. The ignored band is exactly `[50700, 99999]`
— the range MySQL 5.7 and 8.0 write their version-gated syntax into. Without
it, `/*!80003 SRID 0 */` would reach the parser as a column attribute MariaDB
does not have, and `sakila-schema.sql` would fail on `CREATE TABLE address`.

**`world.sql` has no comment in that band at all.** It uses `40000`, `40014`,
`40101`, `40103`, `40111` and `50503`, so MariaDB runs every one of them, and
none guards a statement MariaDB lacks.

#### What the two do not exercise

This is `FR-EX-006`'s reason for a third schema, restated as a count. Over the
three schemas on `12.3`, a column reports **39 distinct `data_type` values**.
`sakila` and `world` between them reach 15 of them; `freight` reaches all 39.

The 15 the published datasets do reach:

```
blob  char  datetime  decimal  enum  int  mediumint  mediumtext
set  smallint  text  timestamp  tinyint  varchar  year
```

The 24 they do not, each of which only `freight` declares:

| Group | Values |
|---|---|
| Fixed-width and floating numerics | `bigint`, `double`, `float` |
| Temporal | `date`, `time` |
| Binary | `binary`, `varbinary`, `tinyblob`, `mediumblob`, `longblob` |
| Text | `tinytext`, `longtext` |
| MariaDB's own | `bit`, `uuid`, `inet4`, `inet6` |
| Spatial | `geometry`, `point`, `linestring`, `polygon`, `multipoint`, `multilinestring`, `multipolygon`, `geometrycollection` |

Beyond the type list, neither published dataset declares a generated column, a
table comment, a sequence or a system-versioned table — the `generated=0` and
`commented_tables=0` in the run above are that, measured.

There is no `json` in the 39, and its absence is not an omission in `freight`.
MariaDB's `JSON` is an alias, so a column declared `JSON` never reports a
`data_type` of `json`:

```sh
tpl_mariadb_sql 12.3 -B -e "
    CREATE DATABASE tpl_probe_json;
    CREATE TABLE tpl_probe_json.t (id INT, doc JSON);
    SELECT COLUMN_NAME, DATA_TYPE, COLUMN_TYPE FROM information_schema.COLUMNS
      WHERE TABLE_SCHEMA='tpl_probe_json' AND TABLE_NAME='t';
    SELECT CONSTRAINT_NAME, CHECK_CLAUSE FROM information_schema.CHECK_CONSTRAINTS
      WHERE CONSTRAINT_SCHEMA='tpl_probe_json';
    DROP DATABASE tpl_probe_json;"
```
```
COLUMN_NAME     DATA_TYPE       COLUMN_TYPE
id              int             int(11)
doc             longtext        longtext

CONSTRAINT_NAME CHECK_CLAUSE
doc             json_valid(`doc`)
```

What survives into the catalogue is `longtext` plus a `json_valid()` check
constraint, so a type mapping written over `data_type` — which is what
`FR-EX-008` obliges — will never be handed a `json` to map.

## Connecting

### Through `docker exec`

The client inside the container connects over the Unix socket, which sidesteps
every TLS question. This is the recommended way to interrogate a fixture by
hand.

```sh
docker exec -it tpl-mariadb-11.4 mariadb -uroot -ptpl-root freight
docker exec -i  tpl-mariadb-11.4 mariadb -uroot -ptpl-root freight < some-query.sql
```

### Over TCP from the host

```sh
mariadb -h 127.0.0.1 -P 13307 -u tpl_reader -ptpl-reader-pw \
  --ssl-ca=tls/ca.pem --ssl-verify-server-cert freight
```

Run it from this directory, so that `tls/ca.pem` resolves. That is the full
check — chain and host name — and it succeeds on all four series, by
`127.0.0.1` and by `localhost` alike. What happens when the two flags are
dropped depends on the client, and is the subject of the rest of this section.

**A modern MariaDB client with no flags at all fails against `10.11`.** It
succeeds against the other three. The message is:

```
ERROR 2026 (HY000): TLS/SSL error: Certificate verification failure: The certificate is NOT trusted.
```

Observed with the `11.4` and the `12.3` client, against the `10.11` fixture,
and not against `11.4`, `11.8` or `12.3`. What the flagless client is missing
is a way to establish trust without being given the certificate authority, and
what it uses on the other three arrives **after authentication**: with a wrong
password every series answers `ERROR 1045`, and only with the right one does
`10.11` answer `NOT trusted`. Whatever the mechanism is called, `10.11` does
not have it and the three later series do.

Three flags get such a client through to `10.11`: `--ssl-ca` together with
`--ssl-verify-server-cert`, as above, which is also the only one of the three
that verifies anything; `--disable-ssl-verify-server-cert`, which encrypts and
trusts blindly; and `--skip-ssl`, which does neither.

A `10.11` MariaDB client reaches all four servers with no flags and an
encrypted session, because it does not verify by default, and reaches the
no-TLS server in plaintext. A MySQL client reaches all four with
`--ssl-mode=VERIFY_IDENTITY --ssl-ca=tls/ca.pem`, and refuses the no-TLS server
with `SSL is required but the server doesn't support it`.

## TLS

`FR-CONF-038`, in `specification/configuration-model.md`, obliges this fixture
to present, at each supported series, a server whose certificate names the host
the project's tests reach it by, and to keep a server that offers none. The
first is what an acceptance test for `verify-identity` — the default `tls` mode
of `FR-CONF-013` — needs; the second is the right-hand column of that
requirement's mode table.

Nothing the servers produce by themselves satisfies the first. `10.11` offers
no TLS at all unless a certificate is configured. `11.4` and later generate one
when none is, and it names nothing: `CN=MariaDB Server`, self-signed, serial
`0`, **zero X509v3 extensions** and therefore no `subjectAltName` for any host
name to match. It is also regenerated on every start — two consecutive starts
of the same container served two different SHA-256 fingerprints. The fixture
therefore carries its own certificate.

### The certificate

| | |
|---|---|
| Names | `DNS:localhost`, `IP:127.0.0.1`, `IP:::1` |
| Subject | `O=tpl MariaDB test fixture, CN=localhost` |
| Issuer | `O=tpl MariaDB test fixture, CN=tpl fixture root CA`, in `tls/ca.pem` |
| Key | RSA 2048, signed with SHA-256 |
| Validity | 3650 days from generation |

The three names are the three spellings of the loopback a test can write, so
the certificate matches whichever one it uses. `tls/ca.pem` is the file to pass
as `ca_file` (`FR-CONF-014`); it is the only trust material needed, and the
private key that signed it was destroyed at generation time and is not in this
repository.

The material is committed, so a fresh clone can build and run the fixture with
no preparatory step. It is not secret: the fixture's passwords are in this
directory too, and a certificate that names `localhost` is worth nothing
anywhere else.

### In the image

```
/etc/mysql/tls/ca.pem              0644 root:root
/etc/mysql/tls/server-cert.pem     0644 root:root
/etc/mysql/tls/server-key.pem      owned by mysql, which is the user the server runs as
/etc/mysql/conf.d/90-tls.cnf       ssl_ca, ssl_cert, ssl_key
```

`/etc/mysql/my.cnf` includes `conf.d` last, so those three settings win over
the packaged `50-server.cnf`, where they ship commented out. The group name is
`[server]`, which every series reads: `10.11` keeps its own settings under
`[mysqld]` and `12.3` under `[mariadbd]`, so neither of those is portable.

All four series then report `have_ssl=YES`, `10.11` included:

```sh
docker exec tpl-mariadb-10.11 mariadb -uroot -ptpl-root \
  -e "SHOW VARIABLES WHERE Variable_name IN ('have_ssl','ssl_cert')"
```

`require_secure_transport` is deliberately **not** set. Two cells of the
`FR-CONF-038` table — `disabled` and `preferred` against a server offering TLS
— expect a plaintext connection to be accepted, and requiring transport
security here would turn both into failures.

### The server that offers no TLS

`--skip-ssl`, passed to the container as a server argument, disables TLS
whatever `90-tls.cnf` says. `have_ssl` then reads `DISABLED` while `ssl_cert`
and `ssl_key` still show the configured paths.

```sh
docker run -d --name tpl-mariadb-notls -e MARIADB_ROOT_PASSWORD=tpl-root \
  -p 13310:3306 tpl-mariadb:10.11 --skip-ssl
```

Observed on `10.11` and on `11.4`, where it also suppresses the automatically
generated certificate. `10.11` is the canonical one, because it is the series
that has no TLS of its own. The schema is the same 23 tables; only the
transport differs.

Clearing the paths instead of using `--skip-ssl` does not work, and does not
fail the same way on every series. `--ssl-cert= --ssl-key= --ssl-ca=` makes
`10.11` refuse to start:

```
SSL error: Unable to get certificate from ''
[ERROR] Failed to setup SSL
[ERROR] Aborting
```

while `11.4` starts, ignores the empty values and falls back to its own
generated certificate, reporting `have_ssl=YES` with `ssl_cert` empty. Neither
is the no-TLS server, and `--skip-ssl` is the only spelling that is.

### Regenerating

```sh
cd scripts/mariadb
tls/generate.sh
docker build --build-arg MARIADB_SERIES=10.11 -t tpl-mariadb:10.11 .   # and the other three
```

`tls/generate.sh` needs `openssl` and nothing else. It takes everything the
certificate says from `tls/openssl.cnf` — change the names there, not in the
script — makes a root, signs the leaf with it, and destroys the root's private
key. Two consequences follow from that last step: the material cannot be
extended later without regenerating all of it, and nobody can mint a second
certificate that the committed `ca.pem` would vouch for.

Regeneration is not byte-reproducible; each run makes fresh keys, a fresh root
and fresh dates. What this directory reproduces is the material's meaning — the
same names, the same key type, the same extensions, every run.

The images carry the material, so a regeneration takes effect only after a
rebuild and a new container started from the rebuilt image.

### Verifying it

Chain and host name, from the host, with `openssl`:

```sh
cd scripts/mariadb/tls
openssl s_client -starttls mysql -connect 127.0.0.1:13306 \
  -CAfile ca.pem -verify_return_error -verify_hostname localhost -brief </dev/null
```

A healthy answer contains `Verification: OK` and `Verified peername:
localhost`. Two controls are worth running alongside it, because they are what
prove the check is real: `-verify_hostname something.else` fails with
`hostname mismatch`, and omitting `-CAfile` fails with `self-signed certificate
in certificate chain`.

The same check with each series' own client, over TCP, inside the container:

```sh
docker exec tpl-mariadb-10.11 mariadb --ssl-ca=/etc/mysql/tls/ca.pem \
  --ssl-verify-server-cert -h 127.0.0.1 -P 3306 -u tpl_reader -ptpl-reader-pw \
  -e "SHOW SESSION STATUS LIKE 'Ssl_cipher'"
```

`Ssl_cipher` is read from the live session, so a non-empty value is the
session's own evidence that it is encrypted, rather than a restatement of what
was asked for. Against the no-TLS server the same variable comes back empty.

#### Recorded run

Both halves of `FR-CONF-038` — a server whose certificate names the host at
every series, and a server offering none beside it — observed in one pass, with
the fixture up.

Full verification from the host, chain and host name, against all four published
ports:

```sh
cd scripts/mariadb/tls
for p in 13306 13307 13308 13309; do
  openssl s_client -starttls mysql -connect 127.0.0.1:$p \
    -CAfile ca.pem -verify_return_error -verify_hostname localhost -brief </dev/null
done
```
```
=== port 13306 ===
CONNECTION ESTABLISHED
Protocol version: TLSv1.3
Verification: OK
Verified peername: localhost
```

and `13307`, `13308` and `13309` answered with those same four lines.

The two controls that prove the check is real. A wrong host name:

```
verify error:num=62:hostname mismatch
error:0A000086:SSL routines:tls_post_process_server_certificate:certificate verify failed
```

and the same command against the `--skip-ssl` server on `13310`:

```
Connecting to 127.0.0.1
MySQL server does not support SSL.
```

The same check with each series' own client, over TCP, reading the cipher back
from the live session:

```sh
docker exec tpl-mariadb-<series> mariadb --ssl-ca=/etc/mysql/tls/ca.pem \
  --ssl-verify-server-cert -h 127.0.0.1 -P 3306 -u tpl_reader -ptpl-reader-pw \
  -N -B -e "SELECT CONCAT(VERSION(), '  Ssl_cipher=', (SELECT VARIABLE_VALUE
       FROM information_schema.SESSION_STATUS WHERE VARIABLE_NAME='SSL_CIPHER'))"
```
```
10.11  10.11.19-MariaDB-ubu2204  Ssl_cipher=TLS_AES_256_GCM_SHA384
11.4   11.4.13-MariaDB-ubu2404   Ssl_cipher=TLS_AES_256_GCM_SHA384
11.8   11.8.9-MariaDB-ubu2404    Ssl_cipher=TLS_AES_256_GCM_SHA384
12.3   12.3.3-MariaDB-ubu2404    Ssl_cipher=TLS_AES_256_GCM_SHA384
```

And the server beside them that offers none, reachable over TCP in plaintext,
carrying the same schema:

```sh
docker exec tpl-mariadb-notls mariadb --skip-ssl -h 127.0.0.1 -P 3306 \
  -u tpl_reader -ptpl-reader-pw -N -B -e "..."
```
```
10.11.19-MariaDB-ubu2204  have_ssl=DISABLED  Ssl_cipher=[]  freight objects=23
```

## The nine observations made outside the process

`NFR-PERF-007` forbids verifying a requirement of form by reading the source,
and `BR-SRV-003` says why: a promise about what a process sends that can only be
checked by reading that process's own source is not a promise a caller can rely
on. Nine requirements are held to that standard, and `NFR-PERF-007` fixes
**four** instruments to reach them. Three are this fixture's and are described
below; the fourth is a **differential run**, which needs neither a server nor a
privilege and is therefore the test suite's, where it was built. Each instrument
is bound to the targets its row of that requirement names, and this table names
the instrument per row on the same terms.

| # | Requirement | The property | Instrument |
|---|---|---|---|
| 1 | `NFR-PERF-001` | No query per object on a full read | [statements](#the-statements-a-server-receives) |
| 2 | `NFR-PERF-002` | Reading one named object does not scale with the database | [statements](#the-statements-a-server-receives) |
| 3 | `NFR-PERF-003` | A cache hit opens no connection and issues no query | [connections](#the-connections-a-server-accepts) and [statements](#the-statements-a-server-receives) |
| 4 | `NFR-PERF-004` | At most one connection per invocation | [connections](#the-connections-a-server-accepts) |
| 5 | `NFR-PERF-005` | The commands of `FR-PROJ-025` touch nothing | Split by clause and by target: [connections](#the-connections-a-server-accepts) for the connection clause, on all four targets; a differential run for the discovery and configuration clauses, on all four; [files opened](#the-files-a-process-opens) for those same two clauses **as syscalls**, on the two Linux targets only |
| 6 | `NFR-PERF-006` | A command needing no catalogue opens no connection | [connections](#the-connections-a-server-accepts) |
| 7 | `FR-SRV-012` | The closed statement list of `FR-SRV-006` | [statements](#the-statements-a-server-receives) |
| 8 | `FR-SRV-013` | The read-only read-back, in its confirming outcome | [statements](#the-statements-a-server-receives), and the value the session reports |
| 9 | `FR-SRV-014` | The connection count, from the server side | [connections](#the-connections-a-server-accepts) |

Rows 4 and 9 name one property between them, so the nine requirements need
fewer than nine distinct observations. Rows 1 and 2 are the only two that need a
second and larger database to be conclusive, and they now have one: `WL-001` and
`WL-003` are loaded by [the benchmark workloads](#the-benchmark-workloads), where
the comparison `NFR-PERF-001` asks for is recorded.

Every observation below was made against a **substitute client** — the
`mariadb` client of the series being observed, or the one in the observer
image. It was made that way because `tpl` had no catalogue reader when these
instruments were established, so there was no binary to point them at, and what
was being established was the instrument rather than the behaviour of the
client: an instrument that shows what a substitute sent shows what any client
sent.

**That condition no longer holds, and the instruments are now reached by the
test suite.** `tpl` reads the catalogue, and the suite drives the distributed
binary against these servers through `tests/support/fixture.rs`, which wraps
the three instruments below. The observations recorded here are unchanged and
are what the substitute produced on the dates they carry; what changed is that
the client under test is now `tpl` itself, and the instrument stayed, exactly
as this paragraph said it would.

### The statements a server receives

The instrument is the server's own general log, directed to a table so that it
can be queried rather than parsed.

```sh
./observe.sh statements on   11.8     # log_output=TABLE, empty the log, general_log=ON
./observe.sh statements off  11.8
./observe.sh statements dump 11.8
```

`on` empties the log before enabling it, so a dump covers exactly the window
under test. `mysql.general_log` is a `CSV` table whose `event_time` is
`timestamp(6)` on all four series and on the `--skip-ssl` server, so the dump
orders by microsecond and the order it prints is the order the server received.

**It demonstrably shows what a client sent.** A known statement, sent through
the substitute client and then found:

```sh
./observe.sh statements on 11.8
docker exec -i tpl-mariadb-11.8 mariadb --ssl-ca=/etc/mysql/tls/ca.pem \
  --ssl-verify-server-cert -h 127.0.0.1 -P 3306 -u tpl_reader -ptpl-reader-pw \
  -N -B -e "SELECT 'KNOWN-STATEMENT-MARKER-9f3a'; SHOW DATABASES"
./observe.sh statements off 11.8
./observe.sh statements dump 11.8
```
```
thread_id	command_type	statement
18	Connect	tpl_reader@127.0.0.1 on  using SSL/TLS
18	Query	SELECT 'KNOWN-STATEMENT-MARKER-9f3a'
18	Query	SHOW DATABASES
18	Quit
```

The marker is there, and so is the `SHOW`, which `FR-SRV-007` forbids: a
statement outside the closed list shows up as a statement outside the closed
list. That is the second half of what the instrument has to do, and the half a
test for "four kinds and no fifth" depends on.

**`FR-SRV-012`, end to end.** `probe-session.sql` issues the four kinds of
statement `FR-SRV-006` admits — the version probe, the read-only session
statement, the read-back, then a catalogue read, in that order, which is the
script's own — and stood in for `tpl` while the reader was being built:

```sh
./observe.sh statements on 11.8
docker exec -i tpl-mariadb-11.8 mariadb --ssl-ca=/etc/mysql/tls/ca.pem \
  --ssl-verify-server-cert -h 127.0.0.1 -P 3306 \
  -u tpl_reader -ptpl-reader-pw < probe-session.sql
./observe.sh statements off 11.8
./observe.sh statements dump 11.8
```
```
thread_id	command_type	statement
11	Connect	tpl_reader@127.0.0.1 on  using SSL/TLS
11	Query	SELECT VERSION()
11	Query	SET SESSION TRANSACTION READ ONLY
11	Query	SELECT @@session.tx_read_only
11	Query	SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'freight'
11	Quit
```

Four kinds and no fifth; the three connection-start statements once each; and
one connection. **The order in the dump is the script's, and is not the order
that governs.** `FR-SRV-042` fixes that order — the read-only session
statement, its read-back immediately after, then the version probe — and the
table of `FR-SRV-006` states none at all, so neither is the list this dump can
be read against for sequence. `probe-session.sql` sends the version probe
first, which is the reverse of `FR-SRV-042`, and it is a substitute rather than
the client under test: what this run establishes is the instrument, the four
kinds, their count and the one connection. The order `tpl` itself sends is
asserted against `FR-SRV-042` by the test suite, in the same record.

**Identical on all four series and on the `--skip-ssl` server** apart from the
thread id — the same run against all five, reduced to the row count and the
statements:

```
  10.11  rows=6  SELECT VERSION() | SET SESSION TRANSACTION READ O... | SELECT @@session.tx_read_only | SELECT COUNT(*) FROM INFORMATI...
  11.4   rows=6  SELECT VERSION() | SET SESSION TRANSACTION READ O... | SELECT @@session.tx_read_only | SELECT COUNT(*) FROM INFORMATI...
  11.8   rows=6  SELECT VERSION() | SET SESSION TRANSACTION READ O... | SELECT @@session.tx_read_only | SELECT COUNT(*) FROM INFORMATI...
  12.3   rows=6  SELECT VERSION() | SET SESSION TRANSACTION READ O... | SELECT @@session.tx_read_only | SELECT COUNT(*) FROM INFORMATI...
  notls  rows=6  SELECT VERSION() | SET SESSION TRANSACTION READ O... | SELECT @@session.tx_read_only | SELECT COUNT(*) FROM INFORMATI...
```

The six rows are the `Connect`, the four `Query` rows and the `Quit`. The client adds nothing of its own, which is worth
knowing because it means a stray row in this dump would be the client under
test, not the harness.

The read-back answered `1` on all four, which is `FR-SRV-013`'s confirming
outcome observed twice over: in the statement the server received, and in the
value the session reported. That the setting bites was checked separately, on
`11.8`:

```sh
docker exec tpl-mariadb-11.8 mariadb -uroot -ptpl-root \
  -e "SET SESSION TRANSACTION READ ONLY; CREATE TABLE freight.x_probe (i INT);"
```
```
ERROR 1792 (25006) at line 1: Cannot execute statement in a READ ONLY transaction
```

**Counting catalogue queries**, which is rows 1 and 2:

```sh
./observe.sh statements dump 11.4 --catalogue --count
```

`--catalogue` keeps the rows naming `INFORMATION_SCHEMA` that mean *a statement
was issued* — a `Query` row for the text protocol and an `Execute` row for the
binary one — and drops the `Prepare` row, which registers a statement text
rather than an issue of it; `observe.sh` records why in the header above
`statements_dump`. `--count` prints the number alone. Three fixed catalogue
queries were sent against two schemas of different size, and the instrument
reported the count that matters rather than the size of the schema:

```
  schema=freight    objects=23   catalogue queries the server received=3
  schema=mysql      objects=31   catalogue queries the server received=3
```

The other filters are `--queries`, `--kind <command_type>`, `--user <user>` and
`--all`.

**The observer is a client too.** `observe.sh` reaches the server over the
container's Unix socket, and its own connection is logged like any other, the
dump query included. `dump` therefore drops connections made over the socket by
default, which is how this script and `up.sh` reach the server and is not how
anything under test reaches it; `--all` keeps them.

**The `Connect` row carries the transport.** It reads `using SSL/TLS` against
the four servers presenting the fixture certificate and `using TCP/IP` against
the `--skip-ssl` one:

```
13	Connect	tpl_reader@127.0.0.1 on  using TCP/IP
13	Query	SELECT 'MARKER-notls'
13	Quit
```

So the instrument also answers, per connection and from the server side, whether
the session was encrypted — which is what the mode table of `FR-CONF-038` asks
about.

### The connections a server accepts

```sh
./observe.sh connections 12.3
```
```
Variable_name	Value
Aborted_connects	3
Connections	16
Max_used_connections	1
Threads_connected	1
```

`Connections` is monotonic and counts every connection the server accepted since
it started, so a test brackets the invocation under test with two readings and
subtracts. `--value` prints `Connections` alone, for arithmetic.

**The observer's baseline is 1, measured rather than assumed.** Two consecutive
readings with nothing between them, on each of the five servers:

```
  10.11  two readings 10 -> 11, baseline delta=1
  11.4   two readings 5 -> 6,   baseline delta=1
  11.8   two readings 14 -> 15, baseline delta=1
  12.3   two readings 10 -> 11, baseline delta=1
  notls  two readings 10 -> 11, baseline delta=1
```

The 1 is the second reading's own connection. With three client connections
between the two readings:

```
a=19 b=23 delta=4 observed_baseline=1 => connections attributable to the client: 3
```

That subtraction is the whole of rows 4 and 9. Rows 3 and 6 are its zero case: a
command that must reach no server, bracketed the same way, with the statement
log checked alongside.

```sh
./observe.sh statements on 11.4
a=$(./observe.sh connections 11.4 --value)
docker run --rm tpl-mariadb-observer 'mariadb --version'
b=$(./observe.sh connections 11.4 --value)
./observe.sh statements off 11.4
./observe.sh statements dump 11.4 --count
```
```
Connections: 22 -> 23, delta=1, observer baseline=1 => attributable to the command: 0
statements the server received from anything but the observer: 0
```

The general log gives a second, independent count of the same thing: one
`Connect` row per connection the server accepted, carrying the user, the source
address and the transport. It differs from the counter in one respect that
matters — an unauthenticated connection, such as the gate's own probe, raises
`Connections` but produces no `Connect` row.

### The files a process opens

This is the third instrument, and the only one of the three the server cannot
provide. It serves the two clauses of row 5 that are stated as syscalls, on the
two Linux targets alone.

**On Linux, `strace` on the host**, which observes the real process:

```sh
./observe.sh opens -- <command...>
```

**On macOS there is no equivalent that runs here**, and the attempts are
recorded rather than hidden:

| Tried | Result |
|---|---|
| `strace` | Not a macOS tool; `command -v strace` finds nothing |
| `dtruss /bin/echo hello` | `dtrace: system integrity protection is on, some features will not be available` / `dtrace: failed to initialize dtrace: DTrace requires additional privileges` |
| `sudo -n fs_usage -w -f filesys` | `sudo: a password is required` |
| `csrutil status` | `System Integrity Protection status: enabled.` |

So on a macOS host the traced process runs inside a Linux container instead,
which observes a Linux build of it — a supported target of this project in its
own right, but **not** the Darwin one. `observe.sh opens` selects that backend
automatically when the host has no `strace`, and says which backend it used.

```sh
./observe.sh opens --server 11.8 -- mariadb --ssl-ca=/tls/ca.pem \
  --ssl-verify-server-cert -h 127.0.0.1 -P 3306 \
  -u tpl_reader -ptpl-reader-pw -N -B -e 'SELECT 1'
```
```
observe.sh: tracing inside tpl-mariadb-observer (network namespace of 11.8)
9  execve("/usr/bin/mariadb", ["mariadb", "--ssl-ca=/tls/ca.pem", ...]) = 0
9  newfstatat(AT_FDCWD, "/etc/my.cnf", {st_mode=S_IFREG|0644, st_size=333, ...}, 0) = 0
9  openat(AT_FDCWD, "/etc/my.cnf", O_RDONLY|O_LARGEFILE|O_CLOEXEC) = 3
9  openat(AT_FDCWD, "/etc/my.cnf.d/", O_RDONLY|O_LARGEFILE|O_CLOEXEC|O_DIRECTORY) = 4
9  newfstatat(AT_FDCWD, "/etc/mysql/my.cnf", 0xffffed9255d0, 0) = -1 ENOENT (No such file or directory)
9  newfstatat(AT_FDCWD, "/root/.my.cnf", 0xffffed9255d0, 0) = -1 ENOENT (No such file or directory)
9  socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) = 3
9  connect(3, {sa_family=AF_INET, sin_port=htons(3306), sin_addr=inet_addr("127.0.0.1")}, 16) = -1 EINPROGRESS
9  recvfrom(3, "Z\0\0\0\n11.8.9-MariaDB-ubu2404\0\37\0\0\0"..., 16384, MSG_DONTWAIT, NULL, NULL) = 94
9  openat(AT_FDCWD, "/tls/ca.pem", O_RDONLY|O_LARGEFILE) = 4
```

Everything `NFR-PERF-005` names is in there: each file opened, each path
`stat`ed — including the ones that were **not** found, which is how an upward
walk through ancestor directories would show itself — and the socket, with the
address and port it was connected to.

`--server <name>` joins the traced process to that server's network namespace,
so `127.0.0.1:3306` inside the container is the server and the fixture
certificate, which names `127.0.0.1`, matches. `tls/` is mounted at `/tls`.
Without `--server` the process is traced with no server in reach, which is the
shape of the `NFR-PERF-005` test itself:

```sh
./observe.sh opens -- mariadb --version > trace
grep -cE '\b(socket|connect)\(' trace
grep -E 'openat\(.*\) = [0-9]+$' trace | sed -E 's/.*openat\(AT_FDCWD, "([^"]+)".*/  \1/'
```
```
0
  /usr/lib/libssl.so.3
  /usr/lib/libcrypto.so.3
  /usr/lib/libz.so.1
  /usr/lib/libncursesw.so.6
  /usr/lib/libstdc++.so.6
  /usr/lib/libgcc_s.so.1
  /etc/my.cnf
  /etc/my.cnf.d/
  /etc/my.cnf.d/mariadb-server.cnf
```

Zero sockets, and a list of every file the process opened. `observer.Dockerfile`
builds the image `observe.sh` uses; it is `alpine:3.24` with `strace` and
`mariadb-client`, and the container is run with `--cap-add=SYS_PTRACE` and
`--security-opt seccomp=unconfined`, without which `ptrace` is refused.

### What could not be instrumented

Two things, recorded here rather than left to be discovered later.

**The failing outcome of `FR-SRV-013`.** That requirement assigns each outcome
of the read-back to the test form that can reach it, and this fixture's is the
confirming one, observed above. The failing one — a read-back that does not
confirm the setting — could not be produced by any server here, and the
requirement records the same three attempts itself, with the bound on what they
establish. These are them:

| Tried | What happened |
|---|---|
| `START TRANSACTION; SET SESSION TRANSACTION READ ONLY;` | Accepted on all four series, and the read-back still answers `1`. MariaDB does not reject it the way MySQL does |
| The same as `tpl_reader`, the reduced-privilege user | Accepted; read-back `1` |
| A server already `read_only` | `@@global.read_only` is `0` on the fixture, and setting it changes the global state, not whether a **session** setting takes effect |

A server that accepts the statement and does not apply it is exactly the case
`FR-SRV-009` exists to catch, and no real MariaDB behaves that way. Producing it
needs a seam in `tpl`, not a container, and `FR-SRV-013` authorises one on the
terms of `FR-ERR-031` — reachable only from within the system's own test
configuration — so that outcome is verified in process and nothing is owed to
this fixture for it. What the fixture establishes is the instrument: the
statement is visible in the log and the value is readable from the session, so
whichever outcome occurs is observable.

**Row 5 on Darwin, as a syscall.** The trace above is of a Linux process. macOS
offers no tracer that runs without root or without System Integrity Protection
disabled, as the table above records, so on a macOS host no clause of
`NFR-PERF-005` is observed as a syscall. The clauses themselves are not left
unobserved: the connection clause is the server's own record, on all four
targets, and the discovery and configuration clauses fall to the fourth
instrument, the differential run, which `NFR-PERF-005` makes the whole of the
evidence where no tracer exists. That instrument is the test suite's, not this
fixture's.

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
differences were found. The harness later found two more, from outside the
process rather than in the catalogue, and they are numbered 8 and 9.

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

3. **TLS.** Left to themselves the four do not agree: `10.11` offers none and
   reports `have_ssl=DISABLED`, while `11.4` and later generate a nameless
   self-signed certificate at every start and report `have_ssl=YES`. The
   fixture now configures its own certificate on all four, so all four report
   `have_ssl=YES` and present the same one — see [TLS](#tls) — and what is left
   of the divergence is three narrower observations:

   - A **MariaDB** client of 11.4 or later, given no flags, reaches `11.4`,
     `11.8` and `12.3` but fails against `10.11` with `Certificate
     verification failure: The certificate is NOT trusted.` The trust it uses
     on the three is established after authentication — a wrong password gives
     `ERROR 1045` on every series, and only the right one gets as far as `NOT
     trusted` on `10.11` — and `10.11` does not implement it. A `10.11`
     MariaDB client verifies nothing by default and reaches all four
     unmodified.
   - `--skip-ssl` given to the **server** disables TLS on every series tried,
     overriding a configured certificate. Clearing the paths instead —
     `--ssl-cert= --ssl-key= --ssl-ca=` — makes `10.11` abort at startup with
     `SSL error: Unable to get certificate from ''`, while `11.4` starts and
     silently falls back to its own generated certificate.
   - Connections made through `docker exec` over the Unix socket are still
     unaffected by all of this, with one exception worth knowing: an `11.4`
     client talking to an `11.4` server that was started `--skip-ssl` is
     refused over the socket too, with `SSL is required, but the server does
     not support it`, and needs `--skip-ssl` itself.

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

### Two more, found by the harness

Neither is a difference in the catalogue, which is why the comparison above
did not reach them. Both were observed from outside the process, and both
bear on a statement `tpl` must issue.

8. **`@@session.transaction_read_only` does not exist on `10.11`.** Reading it
   there fails with `ERROR 1193 (HY000): Unknown system variable
   'transaction_read_only'`, while `11.4`, `11.8` and `12.3` accept it.
   `@@session.tx_read_only` exists on all four and answers identically, so it is
   the only spelling a read-back can use across the supported set. This matters
   to the fourth entry of the closed list of `FR-SRV-006` — the read-back of
   `FR-SRV-009` — which is a statement `tpl` must issue on every connection:
   written with the later spelling it would fail on a quarter of the servers
   `tpl` claims to support, and fail with an error rather than with a wrong
   answer. `probe-session.sql` uses `tx_read_only` for that reason.

9. **The `5.5.5-` prefix in the handshake greeting, on `10.11` only.** The
   version a server sends in its initial handshake packet is
   `5.5.5-10.11.19-MariaDB-ubu2204` on `10.11` and `11.4.13-MariaDB-ubu2404`,
   `11.8.9-MariaDB-ubu2404`, `12.3.3-MariaDB-ubu2404` on the other three, with
   no prefix. `SELECT VERSION()` answers without the prefix on all four,
   `10.11` included, so the two readings of the version disagree on exactly one
   series. Observed with the gate's own probe, which reads the greeting and
   nothing else; see [The gate](#the-gate).

### Readings that differ without being differences between the series

Some readings differ across the four servers and are still not series
differences, because the **build** already explains them. This fixture names a
series and never pins a patch release, so two runs of it can observe two builds;
telling the two apart needs the build's own identifiers, and those are what this
reading takes.

```sh
./observe.sh build            # every server
./observe.sh build 12.3       # one of them
```

It enumerates the `version%` prefix instead of asking for names, and that choice
is the point of the subcommand. A name asked for and absent comes back as **no
row** — `SHOW GLOBAL VARIABLES LIKE 'malloc%'` prints nothing at all, header
included, and exits `0` — so a name misremembered by one character is
indistinguishable from a server that lacks the variable. Asking for the same
wrong name directly is the louder alternative:

```
ERROR 1193 (HY000): Unknown system variable 'malloc_library'
```

Both were observed on `11.8`. An enumeration returns what the server has, which
is also how these names were recovered when none was written down.

#### Recorded run

`./observe.sh build`, verbatim, with the fixture up:

```
SERVER  VARIABLE                 VALUE
10.11   version                  10.11.19-MariaDB-ubu2204
10.11   version_comment          mariadb.org binary distribution
10.11   version_compile_machine  aarch64
10.11   version_compile_os       debian-linux-gnu
10.11   version_malloc_library   system
10.11   version_source_revision  93e051860a9c7e87ee8cee6ed38b640d491f7170
10.11   version_ssl_library      OpenSSL 3.0.2 15 Mar 2022
11.4    version                  11.4.13-MariaDB-ubu2404
11.4    version_comment          mariadb.org binary distribution
11.4    version_compile_machine  aarch64
11.4    version_compile_os       debian-linux-gnu
11.4    version_malloc_library   system
11.4    version_source_revision  170b1d70737be6f134448f51713cdc1ae215b420
11.4    version_ssl_library      OpenSSL 3.0.13 30 Jan 2024
11.8    version                  11.8.9-MariaDB-ubu2404
11.8    version_comment          mariadb.org binary distribution
11.8    version_compile_machine  aarch64
11.8    version_compile_os       debian-linux-gnu
11.8    version_malloc_library   system
11.8    version_source_revision  bf9193a939f515e95dd8def1a5468088c91cede6
11.8    version_ssl_library      OpenSSL 3.0.13 30 Jan 2024
12.3    version                  12.3.3-MariaDB-ubu2404
12.3    version_comment          mariadb.org binary distribution
12.3    version_compile_machine  aarch64
12.3    version_compile_os       debian-linux-gnu
12.3    version_malloc_library   system
12.3    version_source_revision  83e909fc2a0dbc394b4b683fb3fa2d7dcf26cc5e
12.3    version_ssl_library      OpenSSL 3.0.13 30 Jan 2024
notls   version                  10.11.19-MariaDB-ubu2204
notls   version_comment          mariadb.org binary distribution
notls   version_compile_machine  aarch64
notls   version_compile_os       debian-linux-gnu
notls   version_malloc_library   system
notls   version_source_revision  93e051860a9c7e87ee8cee6ed38b640d491f7170
notls   version_ssl_library      OpenSSL 3.0.2 15 Mar 2022
```

Read on an Apple Silicon host, from the `linux/arm64` images, which is what
`version_compile_machine` records; one server of each series, at the four patch
releases above.

**The two variables are `version_source_revision` and `version_ssl_library`.**
The source revision is a distinct 40-character hash on each of the four. The SSL
library string splits them one against three, along the same line as the
distribution the image was built on:

| Reading | `10.11` | `11.4` | `11.8` | `12.3` |
|---|---|---|---|---|
| `version_source_revision` | `93e051860a9c7e87ee8cee6ed38b640d491f7170` | `170b1d70737be6f134448f51713cdc1ae215b420` | `bf9193a939f515e95dd8def1a5468088c91cede6` | `83e909fc2a0dbc394b4b683fb3fa2d7dcf26cc5e` |
| `version_ssl_library` | `OpenSSL 3.0.2 15 Mar 2022` | `OpenSSL 3.0.13 30 Jan 2024` | `OpenSSL 3.0.13 30 Jan 2024` | `OpenSSL 3.0.13 30 Jan 2024` |
| `version_malloc_library` | `system` | `system` | `system` | `system` |
| `version_comment` | `mariadb.org binary distribution` | `mariadb.org binary distribution` | `mariadb.org binary distribution` | `mariadb.org binary distribution` |
| `version_compile_os` | `debian-linux-gnu` | `debian-linux-gnu` | `debian-linux-gnu` | `debian-linux-gnu` |

**`version_malloc_library` returns a row on all four, and its value is
`system`.** It was read three ways on each of the five servers — `SHOW VARIABLES
LIKE`, `information_schema.GLOBAL_VARIABLES` and `SELECT
@@global.version_malloc_library` — and all three agreed everywhere. Any record
that no row comes back for a malloc-library variable on these servers is
contradicted by this run; what produces no row is the wrong name, as above.

**The fifth server is the control, not a fifth reading.** `notls` runs the same
`10.11` image, and it returns the same source revision and the same SSL library
string, byte for byte. Same build, same readings — which is what makes these
properties of the build, and it is also why the fifth listener settles nothing
about any series. No two builds of one series were compared here, so this run
does not establish whether the series or the distribution fixes the SSL library
string; it records what the four servers returned.

## Stopping and removing

`./down.sh` does all of this and then proves nothing is left; see [Taking it
down](#taking-it-down). By hand:

```sh
docker stop tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3 tpl-mariadb-notls
docker rm   tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3 tpl-mariadb-notls
```

Or in one step, discarding the anonymous volume the entrypoint created with it:

```sh
docker rm -f -v tpl-mariadb-10.11 tpl-mariadb-11.4 tpl-mariadb-11.8 tpl-mariadb-12.3 tpl-mariadb-notls
```

Leave no container running after a validation run. To drop the images too:

```sh
docker rmi tpl-mariadb:10.11 tpl-mariadb:11.4 tpl-mariadb:11.8 tpl-mariadb:12.3
```
