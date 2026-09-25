#!/usr/bin/env bash
#
# The fixture, and the projects the measurement points are measured in.
#
# This file is sourced by `run.sh` and is never run on its own.
#
# NO `docker` COMMAND IS WRITTEN HERE, AND NONE MAY BE. The fixture of
# `scripts/mariadb/` is operated through its own harness and through nothing
# else: `up.sh` starts and verifies the servers, `status.sh` is the gate and
# the inventory, `seed-bench.sh` loads `WL-001` and `WL-003` and verifies every
# count they state, and `down.sh` removes them and proves nothing is left.
# `series.env` is sourced, which is what it documents itself for. A hand-written
# `docker run` here would skip the verification each of those performs, which is
# the whole of what makes the workload the workload the specification names.
#
# Nothing about the fixture is restated here either. The port, the credentials
# and the two benchmark schema names are read from `series.env` and from
# `status.sh --export` at run time, so a change to the fixture reaches this
# harness without anything being edited in it.

# The harness this file drives. Set by `run.sh` from the repository root.
FIXTURE_DIR=''

# What the fixture answered, filled in by `fixture_address`.
FIXTURE_HOST=''
FIXTURE_PORT=''

# ------------------------------------------------------------ the inventory ---

# Reads `series.env`, which publishes the credentials and the two benchmark
# schema names the projects below are built from.
fixture_inventory() {
    # shellcheck source=/dev/null
    . "$FIXTURE_DIR/series.env"
}

# The four series of `FR-SRV-015`, one per line, as `series.env` states them.
fixture_series_names() {
    printf '%s\n' "$TPL_MARIADB_SERVERS" | awk '$6 == "yes" { print $1 }'
}

# Is `$1` one of them?
fixture_is_series() {
    fixture_series_names | grep -qx "$1"
}

# ------------------------------------------------------------- the lifecycle ---

# Starts the named server and does not return until `up.sh` has verified it.
#
# `up.sh` ends by asking the gate about the servers it was asked to start, so
# its exit code is that server's answer, and it is what this function returns.
fixture_up() {
    "$FIXTURE_DIR/up.sh" "$1" >&2
}

# Loads `WL-001` and `WL-003` into it, and verifies every count they state.
fixture_seed() {
    "$FIXTURE_DIR/seed-bench.sh" "$1" >&2
}

# Removes it, and proves nothing of it is left.
fixture_down() {
    "$FIXTURE_DIR/down.sh" "$1" >&2
}

# The gate. 0 when the named server answered, non-zero when it did not.
fixture_is_up() {
    "$FIXTURE_DIR/status.sh" --quiet "$1"
}

# Where the named server answered, as `FIXTURE_HOST` and `FIXTURE_PORT`.
#
# `status.sh --export` prints shell assignments. They are **parsed, never
# eval'ed**: the gate is a fixture the harness trusts to be correct, not one it
# trusts to be safe to execute, and the test suite reads it the same way.
fixture_address() {
    local exported address

    exported="$("$FIXTURE_DIR/status.sh" --export "$1")" || return 1

    address="$(printf '%s\n' "$exported" \
        | awk -F'[=;]' '/^TPL_MARIADB_/ && $1 != "TPL_MARIADB_READY" { print $2; exit }')"

    [ -n "$address" ] || return 1

    FIXTURE_HOST="${address%:*}"
    FIXTURE_PORT="${address##*:}"
}

# ---------------------------------------------------------------- the projects ---

# The entry names the benchmark projects define. They are this harness's own
# and appear nowhere else.
FIXTURE_ENTRY_LARGE='bench_wl001'
FIXTURE_ENTRY_SMALL='bench_wl003'

# The two projects the measurement points are measured in, under `$1`.
#
#     <work>/startup   a project with no database entry at all, for the points
#                      whose workload is `none` and which still have to discover
#                      a project and read a configuration
#     <work>/server    a project with one entry per benchmark workload
#
# They are separate because the points they serve are: `NFR-PERF-014` gives
# three of the nine a workload of `none`, and measuring those inside a project
# carrying two database entries would put the parse of those entries into a
# figure that is supposed to carry no workload at all.
#
# The account is `root` and not `tpl_reader`, although `seed-bench.sh` grants
# the reader `SELECT, EXECUTE` on both benchmark schemas. That grant is the one
# `setup.sql` gives on `freight`, and it shows the reader every table and view
# but no foreign-key constraint, no trigger, no view definition and no routine
# body (scripts/mariadb/README.md, "Users"). A reader-backed reading would be
# taken over a smaller document, with objects marked `restricted` that the
# cache does not store, which is not a reading over `WL-001` as the
# specification states it.

# The first of the two. It needs no fixture and no server, which is why it is
# built on its own: the points whose workload is `none` must be measurable
# with nothing stood up.
fixture_startup_project() {
    local work="$1" binary="$2"

    rm -rf "$work/startup"
    mkdir -p "$work/startup"

    (cd "$work/startup" && "$binary" init . >/dev/null)
}

# The second. It needs `fixture_address` to have answered first.
fixture_server_project() {
    local work="$1" binary="$2" tls="$3"

    rm -rf "$work/server"
    mkdir -p "$work/server"

    (cd "$work/server" && "$binary" init . >/dev/null)

    cat > "$work/server/.tpl/.cfg" <<CONFIGURATION
# Written by benches/run.sh. Every value is read from scripts/mariadb at run
# time and none of it is secret: the fixture's credentials are published in
# scripts/mariadb/README.md.

[core]
database = "$FIXTURE_ENTRY_LARGE"

[database.$FIXTURE_ENTRY_LARGE]
host = "$FIXTURE_HOST"
port = $FIXTURE_PORT
user = "root"
password = "$TPL_MARIADB_ROOT_PASSWORD"
database = "$TPL_MARIADB_BENCH_LARGE_SCHEMA"
tls = "$tls"

[database.$FIXTURE_ENTRY_SMALL]
host = "$FIXTURE_HOST"
port = $FIXTURE_PORT
user = "root"
password = "$TPL_MARIADB_ROOT_PASSWORD"
database = "$TPL_MARIADB_BENCH_SMALL_SCHEMA"
tls = "$tls"
CONFIGURATION

    chmod 600 "$work/server/.tpl/.cfg"
}

# Fills the cache of both entries from the server, so that the two points
# `NFR-PERF-014` gives `Server: no` and `Cache: served from` can be measured
# with it down.
fixture_prime() {
    local work="$1" binary="$2"

    (cd "$work/server" && "$binary" -d "$FIXTURE_ENTRY_LARGE" cache load >/dev/null)
    (cd "$work/server" && "$binary" -d "$FIXTURE_ENTRY_SMALL" cache load >/dev/null)
}

# ---------------------------------------------------------------- the subjects ---

# The names the points need, discovered from the catalogue rather than written
# down here. Sets `FIXTURE_WL001_NAMES`, `FIXTURE_WL003_TABLE` and
# `FIXTURE_ABSENT_NAME`.
#
# A second copy of a fixture fact is a second thing that can be wrong, and the
# fixture has already changed once. Everything below is asked of `tpl` itself,
# reading the cache these projects now hold.
fixture_subjects() {
    local work="$1" binary="$2"

    FIXTURE_WL001_NAMES="$work/wl001-tables.txt"

    (cd "$work/server" \
        && "$binary" -d "$FIXTURE_ENTRY_LARGE" schema tables --format json) \
        | jq -r '.data.tables[].name' > "$FIXTURE_WL001_NAMES"

    local count
    count="$(wc -l < "$FIXTURE_WL001_NAMES" | tr -d ' ')"

    if [ "$count" -ne "$TPL_MARIADB_WL001_TABLES" ]; then
        printf 'benches: WL-001 presented %s tables, and series.env states %s\n' \
            "$count" "$TPL_MARIADB_WL001_TABLES" >&2
        return 1
    fi

    FIXTURE_WL003_TABLE="$( (cd "$work/server" \
        && "$binary" -d "$FIXTURE_ENTRY_SMALL" schema tables --format json) \
        | jq -r '.data.tables[0].name')"

    [ -n "$FIXTURE_WL003_TABLE" ] || return 1

    FIXTURE_ABSENT_NAME="$(fixture_absent_name "$FIXTURE_WL001_NAMES")" || return 1
}

# A name that is one edit away from a real `WL-001` table and is not itself one.
#
# The failure path of `NFR-PERF-014` measures a `66` **with** a nearest match,
# and `BR-PERF-004` says why: the suggestion computes an edit distance against
# every existing name, which over `WL-001` is 200 of them. A name at distance 1
# from a real one is what makes `FR-ERR-020` offer a suggestion instead of
# withholding it, so the point measures the computation it exists to measure.
fixture_absent_name() {
    local names="$1" first suffix candidate

    first="$(head -1 "$names")"
    [ -n "$first" ] || return 1

    for suffix in x q z j v; do
        candidate="$first$suffix"
        if ! grep -qx "$candidate" "$names"; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    printf 'benches: every candidate near %s is itself a table\n' "$first" >&2
    return 1
}
