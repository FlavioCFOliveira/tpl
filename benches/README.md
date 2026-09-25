# The benchmark harness

The instrument for the nine **measurement points** of `NFR-PERF-014` and the
`WL-002` scalar, taken under the protocol of `NFR-PERF-009` through
`NFR-PERF-012`. It stands the MariaDB fixture up for the readings that need a
server, takes it down for the readings that must not have one, and writes one
machine-readable record per point.

## It measures, and it does nothing else

This harness is an instrument, not a gate. `BR-PERF-008` is the rule underneath
that: no figure named in the corpus, and no figure recorded against it in
`BENCHMARKS.md`, fails, blocks, rejects or gates a change. So this harness

- reads no earlier figure, and never opens `BENCHMARKS.md`;
- compares nothing with anything, and computes no delta, no regression and no
  pass or fail;
- withholds no figure, whatever it came out at;
- exits `0` whenever the campaign ran to the end, however fast or slow anything
  was.

The only non-zero exit is `2`, and it means the run could not happen at all: a
missing tool, no binary to measure, a host that is none of the four targets of
`NFR-PERF-018`, or a fixture that would not stand up.

`NFR-PERF-011`'s five per cent is reported and not enforced. Every record
carries `rsd_pct`, the relative standard deviation of the samples behind it, and
`rsd_over_5pct`, a plain boolean saying whether that figure is above five. Both
are data. A reader who wants the rule that requirement states applies it to the
field; nothing here applies it to the run.

## What it needs

| Tool | For |
|---|---|
| `hyperfine` 1.20 or later | every wall-time reading, always with `--shell=none` |
| `jq` | reading hyperfine's export, and writing the records |
| `/usr/bin/time` | the peak-memory reading, which hyperfine does not take |
| `awk`, `sed`, `sort`, `file` | the statistics, the quoting and the target detection |
| Docker | only for the six readings that need the fixture |

The binary under measurement is `target/release/tpl` unless `--bin` names
another. Build it first: the release profile is the one a user runs, and a debug
binary measures nothing anybody has.

```sh
cargo build --release
```

## Running it

```sh
# the full campaign: nine points and the scalar, under the full protocol
./benches/run.sh

# a reduced run that proves the harness works, in about a minute
./benches/run.sh --proving --runs 5 --warmups 2

# only some of them
./benches/run.sh 1 2 3 7          # the four that need no fixture and no server
./benches/run.sh 5 6              # the two served from the cache, with the server down
./benches/run.sh wl-002           # the verification scalar alone

# against another series of FR-SRV-015, with the records kept in a file
./benches/run.sh --series 11.8 --out readings.jsonl
```

| Option | Default | What it does |
|---|---|---|
| `--runs N` | `200` | timing runs per reading, the floor of `NFR-PERF-009` |
| `--warmups N` | `20` | warmup runs per reading, the same requirement's other floor |
| `--proving` | off | permits a run below either floor; every record then says `standing: reduced` |
| `--series NAME` | `12.3` | the series of `FR-SRV-015` the server-backed readings are taken against |
| `--tls MODE` | `disabled` | the transport the benchmark entries ask for, named in every record it reaches |
| `--bin PATH` | `target/release/tpl` | the binary under measurement |
| `--target TRIPLE` | detected | states the target of `NFR-PERF-018` where detection cannot |
| `--work-dir PATH` | `target/bench-work` | where the projects, the raw samples and the hyperfine exports go |
| `--out FILE` | none | writes the records here as well as to stdout |

Records go to **stdout**, one line of JSON each. Progress, and everything the
fixture harness prints, go to **stderr**. So a campaign pipes cleanly:

```sh
./benches/run.sh 2>/dev/null | jq -r '[.id, .median, .unit, .target] | @tsv'
```

### What the full campaign costs

Around half an hour on an Apple M4, almost all of it the eighth point: 220 runs
of a 200-invocation loop is 44 000 process startups. The other nine readings
together take under a minute, and the fixture adds about a minute of standing
up, loading and tearing down. A proving run at `--runs 5 --warmups 2` does the
whole thing in about a minute and a half.

## The record

One line of JSON per reading. Every field is a statement about the reading and
none is a judgement on it.

| Field | What it is |
|---|---|
| `record` | the record schema, `tpl-bench/2` |
| `id` | `point-1` … `point-9`, or `wl-002` |
| `point` | the row of `NFR-PERF-014`, by its `#`, or `null` for the scalar |
| `point_name` | that row's own words |
| `workload` | `WL-001`, `WL-003` or `none` |
| `quantity`, `unit` | `wall_time` in `ms`, `peak_rss` in `bytes`, or `document_size` in `bytes` |
| `target`, `target_source` | the target of `NFR-PERF-018`, and whether it was detected or given |
| `series` | the series of `FR-SRV-015` a server answered as, or `null` where none did |
| `tls` | the transport that entry asked for, or `null` |
| `server` | `up`, `down` or `not required` while the reading was taken |
| `cache` | the words of the `Cache` column `NFR-PERF-014` gives the point |
| `command` | the invocation, as a reader would retype it |
| `n`, `warmups` | the samples behind the figures, and the runs discarded before them |
| `median`, `mean`, `stddev`, `min`, `max` | the sample, described |
| `rsd_pct`, `rsd_over_5pct` | the dispersion, and whether it is above the five per cent `NFR-PERF-011` speaks of |
| `standing` | `full` under both floors of `NFR-PERF-009`, `reduced` below either |
| `protocol`, `method` | which protocol, and which instrument took the reading |
| `aggregation`, `parts` | for a point made of more than one invocation; `null` otherwise |
| `note` | what a reader of this figure has to know about how it was taken |
| `binary`, `binary_sha256` | what was measured |
| `host`, `taken_at` | where, and when |

### What changed from `tpl-bench/1`

The thirty-sixth edition of `specification/performance-requirements.md` made
the nine a **measurement set** of **measurement points**, so `budget` became
`point`, `budget_name` became `point_name`, and the `id` values became
`point-1` … `point-9`. The `normative` field is **removed**: it marked the one
point that could fail a change on its own merits, and the requirement that
created that status is withdrawn and its identifier retired, so the field has
nothing left to name. `cache` is **added**, because `NFR-PERF-020` obliges a
record to state the cache posture its point was measured under, and
`NFR-PERF-014` now fixes that posture in a column of its own.

*Rejected: keeping `budget` and only dropping `normative`.* `budget` is the
withdrawn regime's word. The corpus keeps it only where it still means a limit
— the invocation timeout of `FR-GLOB-011` and the shared phase budget of
`FR-CONF-005` — so a field of this record called `budget` sends a reader looking
for a limit that does not exist. *Also rejected: `measurement_point`, and
`measurement-point-1` for the `id`.* Exact, and too long for a value that
prefixes every sample filename and is read line by line; `point` is the head of
the corpus's own noun. *Also rejected: `mp-1`.* An abbreviation this corpus
never uses. *Also rejected: keeping `normative: false` on every record as a
tombstone.* It would state a property of a regime that no longer exists, and
`false` on all ten records says nothing. *Also rejected: leaving the schema at
`tpl-bench/1`.* The field exists so that a consumer can tell one generation from
another, and this generation renames two fields, drops one and adds one.

`median` is the median of `n` samples; for an even `n` it is the mean of the two
middle values. `stddev` is the sample standard deviation, with `n - 1` in the
denominator. `rsd_pct` is `stddev / mean`, in per cent. One implementation
computes all of them, for all three quantities, so the words mean one thing
across every record.

## The nine points, as wired

The `Cache` column is `NFR-PERF-014`'s own, reproduced here so that what the
harness does can be checked against what that table fixes.

| # | Measurement point | Invocation | Fixture | Server | Cache |
|---|---|---|---|---|---|
| 1 | `tpl --version` | `tpl --version` | no | no | not reached |
| 2 | `tpl --help` | `tpl --help` | no | no | not reached |
| 3 | Startup to the first byte of useful work | `tpl template list` | no | no | not reached |
| 4 | `tpl schema dump` | `tpl -d bench_wl001 schema dump --direct --no-cache` | yes | **up** | bypassed |
| 5 | A cache-served read of one object | `tpl -d bench_wl003 schema table <table>` | yes | **down** | served from |
| 6 | The failure path | a `64` and a `66`, both over `WL-001` | yes | **down** | served from |
| 7 | `tpl help --format json` | `tpl help --format json` | no | no | not reached |
| 8 | The canonical loop of 200 invocations | `benches/loop200.sh` | yes | **up** | empty when each run begins |
| 9 | Peak resident memory | `tpl -d bench_wl001 schema dump --direct --no-cache` | yes | **up** | bypassed |
| — | `WL-002` | the byte length of the compact dump of `WL-001` | yes | **up** | bypassed, by this harness's choice |

Five of them are worth a paragraph. **Four of the five are decided by
`NFR-PERF-014` and not here** — the thirty-sixth edition settled point 3's
invocation, point 6's aggregation and the cache posture of points 4, 8 and 9 —
and each paragraph says which rule it follows. The fifth, the `WL-002` scalar,
is not a row of that table, and the posture it is measured under is this
harness's own and is stated as such.

**Point 3's invocation is fixed by `NFR-PERF-014`.** The row reads *Startup to
the first byte of useful work, measured by `tpl template list` in a project
holding no database entry*, with a workload of `none` and no server. Until the
thirty-sixth edition it named a quantity and no command, and this harness chose
one; that edition settled it, on the ground the harness had reasoned from, and
the choice is no longer the harness's to make. `tpl template list` is the
cheapest invocation that is *useful work* rather than static text — it discovers
a project, reads a configuration, and presents a result of its own — where
`NFR-PERF-005` excuses help and version from discovery and from reading a
configuration, which is also why this row's adopted figure is twice theirs. What
the instrument times is the whole process, because hyperfine times a process and
not a byte of output; for a command whose work is listing two files that is
startup plus a rounding error. Every record of this point says so in its
`note`.

**Point 4 passes `--direct --no-cache`, because its `Cache` column says so.**
The column reads *bypassed, `--direct --no-cache`*, and `FR-CACHE-016` fixes
those two flags as the pure read. Without them the read is read-through, per
`FR-CACHE-006` and `FR-CACHE-007`: the first run would reach the server and the
other 199 would be served from the cache, and the median — which is what the
protocol records — would be a cache figure under a row that says a server
answered. Point 9 carries the same column entry and is measured over this same
invocation.

**Point 5 runs with the server taken down.** Its row says `Server: no` and
`Cache: served from`. The cache is filled while the server is up, the server is
removed through `down.sh`, and `status.sh` is asked again before the reading is
taken, so that the record's `server: down` and `cache: served from` are
observations and not intentions.

**Point 6 is one point measured by two invocations, and its figure is the slower
of the two.** That aggregation is `NFR-PERF-014`'s, settled in the thirty-sixth
edition, which also obliges both invocations to be recorded, each in full,
beside the figure that stands for the point. The `64` is an undeclared flag on a
real table; the `66` names a table one character away from a real one, so
`FR-ERR-020` offers a suggestion instead of withholding it and the edit distance
of `FR-ERR-019` is computed against all 200 names of `WL-001` — which is what
`BR-PERF-004` says this point exists to measure, and why the `66` is the
expensive half. The near-miss name is derived from a real table at run time and
checked against the whole list, and both halves are run once and their exit
codes checked before either is sampled. Each half is printed whole under
`parts`; the point's own figures are the slower half's, in full, so that the
`median`, the `mean` and the dispersion of the record all describe one reading.
`aggregation` names the rule. The row's `Cache` column says *served from*; the
`64` is refused before any read and reaches neither cache nor server.

**Point 8 is driven by `benches/loop200.sh`.** That is the loop the help of
`tpl render` prints — one invocation per object, because `FR-RND-002` renders
once per invocation and `BR-RND-002` makes iterating the caller's job — and what
`BR-PERF-005` says it measures is 200 **process startups**. The driver is a
`read` and an exec per line and nothing else. `NFR-PERF-010` keeps a shell out
of the *instrument*, which `--shell=none` satisfies here as everywhere; the loop
is the subject, not a wrapper around it. Its `Cache` column says *empty when
each run begins*, and the thirty-sixth edition adds that emptying it is not part
of what is measured: hyperfine's `--prepare` does both, since it runs before
each timing run and is excluded from what it times. Every run is therefore one
server read followed by 199 cache hits — the same work each time, with the
server in all of it.

**Point 9 does not use hyperfine**, which measures time. `ru_maxrss` is read
from outside the process, with `/usr/bin/time -l` on macOS (bytes) and
`/usr/bin/time -v` on Linux (kbytes, scaled). The record says which. The
invocation is point 4's, under the same `Cache` column entry, because — as the
thirty-sixth edition states — the whole-catalogue read is the memory-heaviest
thing `tpl` does over this workload and the adopted figure of that row was
stated for a database of 200 tables.

**The `WL-002` scalar is the length of the compact dump.** It is not a row of
`NFR-PERF-014` and fixes no cache posture, so bypassing the cache is this
harness's choice and the record says so. Compact is the default of `FR-OUT-007`;
`--pretty` is what departs from it, and is not passed. The dump is server-read,
so its envelope carries `source: server`; a cache-served dump of the same
catalogue differs by the two bytes of that word. The scalar is taken more than
once so that the record can report whether the size held still. It is a
deterministic size and not a timing: a departure beyond its ±2% is a statement
about the fixture or the document shape, and is the one thing this directory
measures that `BR-PERF-008` leaves able to fail.

## The fixture

Operated through its own harness and through nothing else:

| Step | Script |
|---|---|
| start and verify the server | `scripts/mariadb/up.sh <series>` |
| ask whether it answered | `scripts/mariadb/status.sh --quiet <series>` |
| find out where | `scripts/mariadb/status.sh --export <series>` |
| load `WL-001` and `WL-003`, and verify every count | `scripts/mariadb/seed-bench.sh <series>` |
| remove it, and prove nothing is left | `scripts/mariadb/down.sh <series>` |

No `docker` command is written in this directory and none may be. The port, the
credentials and the two benchmark schema names are read from `series.env` and
from `status.sh --export` at run time, so a change to the fixture reaches this
harness without anything here being edited.

**The campaign leaves the fixture down.** Points 5 and 6 are measured after the
server has been removed, so that is where a run ends. Only the series the run
was told to use is touched.

**The benchmark entries authenticate as `root`, and keep it.** `seed-bench.sh`
grants `tpl_reader` `SELECT, EXECUTE` on `freight_wl001` and `freight_wl003`,
the grant `setup.sql` gives it on `freight`, so a reader-backed entry does
present both catalogues. It presents less of them: the reader sees no
foreign-key constraint, no trigger, no view definition and no routine body
(`scripts/mariadb/README.md`, *Users*). A reading over that document, with its
views and routines marked `restricted` and left out of the cache, is not a
reading over `WL-001` as `NFR-PERF-014` states it, so `root` is what reads the
workload.

## What the work directory holds

Everything the harness writes lives under `--work-dir`, `target/bench-work` by
default, which `cargo clean` removes and git ignores:

```
startup/            a project with no database entry, for point 3
server/             a project with one entry per benchmark workload
wl001-tables.txt    the 200 table names, for the canonical loop
samples/            the raw samples behind every figure, one file per reading
hyperfine/          hyperfine's own JSON export, one file per reading
```

The raw samples are kept deliberately. A median is a summary, and the numbers it
summarises are what somebody re-reading a campaign will want.

## Why this is shell, and not a `cargo bench` target

Every one of the ten readings measures **the distributed binary as a caller runs
it**: a whole process, from `exec` to exit, with its startup, its configuration
read, its connection and its output in the figure. A `criterion` benchmark
measures a function inside a test binary and cannot see any of that, and the
points that need a server standing up, a workload loaded and a cache primed
between two readings are orchestration rather than measurement.

So the harness is a runner script beside the fixture's own scripts, and
`benches/` carries no `.rs` file. Cargo discovers a bench target only from
`benches/*.rs` and `benches/*/main.rs`, so nothing here is compiled, nothing is
linted, and the mandatory validation pipeline is untouched. `Cargo.toml` needs
no `[[bench]]` table and no `[dev-dependencies]`, which is also the answer the
dependency budget of `CLAUDE.md` wants: the instrument is `hyperfine`, which is
already how this project measures wall time, and no crate was added to reach it.

Adding a `criterion` benchmark later is not blocked by any of this. It would
measure a different thing — a function, not an invocation — and would need a
`[[bench]]` table naming it, so that the runner and the fixture scripts stay off
the target list.

## The files

| File | What it is |
|---|---|
| `run.sh` | the runner: arguments, the ten readings, and the campaign's order |
| `protocol.sh` | the protocol: samples, statistics, and the record. Sourced, never run |
| `fixture.sh` | the fixture harness and the two projects. Sourced, never run |
| `loop200.sh` | the canonical loop, as one command, so that it can be measured as one |
| `README.md` | this file |
