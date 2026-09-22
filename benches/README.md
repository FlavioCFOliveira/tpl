# The benchmark harness

The instrument for the nine budgets of `NFR-PERF-014` and the `WL-002` scalar,
taken under the protocol of `NFR-PERF-009` through `NFR-PERF-012`. It stands
the MariaDB fixture up for the readings that need a server, takes it down for
the readings that must not have one, and writes one machine-readable record per
budget.

## It measures, and it does nothing else

This harness is an instrument, not a gate. It

- reads no baseline, and never opens `BENCHMARKS.md`;
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
# the full campaign: nine budgets and the scalar, under the full protocol
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

Around half an hour on an Apple M4, almost all of it the eighth budget: 220 runs
of a 200-invocation loop is 44 000 process startups. The other nine readings
together take under a minute, and the fixture adds about a minute of standing
up, loading and tearing down. A proving run at `--runs 5 --warmups 2` does the
whole thing in about a minute and a half.

## The record

One line of JSON per reading. Every field is a statement about the reading and
none is a judgement on it.

| Field | What it is |
|---|---|
| `record` | the record schema, `tpl-bench/1` |
| `id` | `budget-1` … `budget-9`, or `wl-002` |
| `budget` | the row of `NFR-PERF-014`, or `null` for the scalar |
| `budget_name` | that row's own words |
| `normative` | `true` for the one normative budget of `NFR-PERF-015`, and for nothing else |
| `workload` | `WL-001`, `WL-003` or `none` |
| `quantity`, `unit` | `wall_time` in `ms`, `peak_rss` in `bytes`, or `document_size` in `bytes` |
| `target`, `target_source` | the target of `NFR-PERF-018`, and whether it was detected or given |
| `series` | the series of `FR-SRV-015` a server answered as, or `null` where none did |
| `tls` | the transport that entry asked for, or `null` |
| `server` | `up`, `down` or `not required` while the reading was taken |
| `command` | the invocation, as a reader would retype it |
| `n`, `warmups` | the samples behind the figures, and the runs discarded before them |
| `median`, `mean`, `stddev`, `min`, `max` | the sample, described |
| `rsd_pct`, `rsd_over_5pct` | the dispersion, and whether it is above the five per cent `NFR-PERF-011` speaks of |
| `standing` | `full` under both floors of `NFR-PERF-009`, `reduced` below either |
| `protocol`, `method` | which protocol, and which instrument took the reading |
| `aggregation`, `parts` | for a budget made of more than one invocation; `null` otherwise |
| `note` | what a reader of this figure has to know about how it was taken |
| `binary`, `binary_sha256` | what was measured |
| `host`, `taken_at` | where, and when |

`median` is the median of `n` samples; for an even `n` it is the mean of the two
middle values. `stddev` is the sample standard deviation, with `n - 1` in the
denominator. `rsd_pct` is `stddev / mean`, in per cent. One implementation
computes all of them, for all three quantities, so the words mean one thing
across every record.

## The nine budgets, as wired

| # | Budget | Invocation | Fixture | Server |
|---|---|---|---|---|
| 1 | `tpl --version` | `tpl --version` | no | no |
| 2 | `tpl --help` | `tpl --help` | no | no |
| 3 | Startup to the first byte of useful work | `tpl template list` | no | no |
| 4 | `tpl schema dump` | `tpl -d bench_wl001 schema dump --direct --no-cache` | yes | **up** |
| 5 | A cache-served read of one object | `tpl -d bench_wl003 schema table <table>` | yes | **down** |
| 6 | The failure path | a `64` and a `66`, both over `WL-001` | yes | **down** |
| 7 | `tpl help --format json` | `tpl help --format json` | no | no |
| 8 | The canonical loop of 200 invocations | `benches/loop200.sh` | yes | **up** |
| 9 | Peak resident memory | `tpl -d bench_wl001 schema dump --direct --no-cache` | yes | **up** |
| — | `WL-002` | the byte length of the compact dump of `WL-001` | yes | **up** |

Five of them need a decision the specification does not make, and each is made
here, in the open.

**Budget 3 names a quantity and no invocation.** `/specification` says
`Startup to the first byte of useful work`, with a workload of `none` and no
server, and names no command. What is measured is `tpl template list` in a
project carrying no database entry: the cheapest invocation that is *useful
work* rather than static text — it discovers a project, reads a configuration,
and presents a result of its own. `NFR-PERF-005` excuses help and version from
discovery and from reading a configuration, so neither of them can carry this
quantity, which is also why this budget's provisional figure is twice theirs.
What the instrument times is the whole process, because hyperfine times a
process and not a byte of output; for a command whose work is listing two files
that is startup plus a rounding error. Every record of this budget carries the
choice in its `note`.

**Budget 4 passes `--direct --no-cache`.** Its row says `Server: yes` and its
provisional figure is stated with server time included. Without those two flags
the read is read-through: the first run would reach the server and the other 199
would be served from the cache, and the median — which is what the protocol
records — would be a cache figure under a row that says otherwise.

**Budget 5 runs with the server taken down.** `NFR-PERF-013` requires every
normative budget to run over `--context` or over the cache and therefore to need
no server. The cache is filled while the server is up, the server is removed
through `down.sh`, and `status.sh` is asked again before the reading is taken,
so that the record's `server: down` is an observation and not an intention.

**Budget 6 is two invocations.** The `64` is an undeclared flag on a real table;
the `66` names a table one character away from a real one, so `FR-ERR-020`
offers a suggestion instead of withholding it and the edit distance of
`FR-ERR-019` is computed against all 200 names of `WL-001` — which is what
`BR-PERF-004` says this budget exists to measure. The near-miss name is derived
from a real table at run time and checked against the whole list, and both
halves are run once and their exit codes checked before either is sampled. Each
half is printed whole under `parts`; the budget's own figures are the slower
half's, in full, so that the `median`, the `mean` and the dispersion of the
record all describe one reading. `aggregation` says which rule chose them.

**Budget 8 is driven by `benches/loop200.sh`.** That is the loop the help of
`tpl render` prints — one invocation per object, because `FR-RND-002` renders
once per invocation and `BR-RND-002` makes iterating the caller's job — and what
`BR-PERF-005` says it measures is 200 **process startups**. The driver is a
`read` and an exec per line and nothing else. `NFR-PERF-010` keeps a shell out
of the *instrument*, which `--shell=none` satisfies here as everywhere; the loop
is the subject, not a wrapper around it. The cache is emptied by hyperfine's
`--prepare`, which is excluded from what it times, so every run is one server
read followed by 199 cache hits — the same work each time, with the server in
all of it.

**Budget 9 does not use hyperfine**, which measures time. `ru_maxrss` is read
from outside the process, with `/usr/bin/time -l` on macOS (bytes) and
`/usr/bin/time -v` on Linux (kbytes, scaled). The record says which. The
invocation is budget 4's, because the whole-catalogue read is the
memory-heaviest thing `tpl` does over this workload and the provisional figure
of that row was stated for a database of 200 tables.

**The `WL-002` scalar is the length of the compact dump.** Compact is the
default of `FR-OUT-007`; `--pretty` is what departs from it, and is not passed.
The dump is server-read, so its envelope carries `source: server`; a
cache-served dump of the same catalogue differs by the two bytes of that word.
The scalar is taken more than once so that the record can report whether the
size held still.

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

**The campaign leaves the fixture down.** Budgets 5 and 6 are measured after the
server has been removed, so that is where a run ends. Only the series the run
was told to use is touched.

**The benchmark entries authenticate as `root`.** `tpl_reader` holds
`SELECT, EXECUTE ON freight.*` and `seed-bench.sql` grants it nothing on
`freight_wl001` or `freight_wl003`, so a reader-backed entry would present an
empty catalogue — which `FR-PRIV-001` makes a silent success rather than an
error, and a reading taken over an empty catalogue is not a reading over
`WL-001`.

**`up.sh` exits `2` on success when it is asked for one server.** It ends by
printing an unfiltered `./status.sh`, which reports the other four as down and
returns that command's `2`. Its exit code is therefore not read here; the
filtered gate is asked immediately afterwards and that is what decides whether
the server came up.

## What the work directory holds

Everything the harness writes lives under `--work-dir`, `target/bench-work` by
default, which `cargo clean` removes and git ignores:

```
startup/            a project with no database entry, for budget 3
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
budgets that need a server standing up, a workload loaded and a cache primed
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
