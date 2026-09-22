#!/usr/bin/env bash
#
# The benchmark harness: the nine measurement points of `NFR-PERF-014` and the
# `WL-002` scalar, measured under the protocol of `NFR-PERF-009` through
# `NFR-PERF-012`.
#
# Usage:
#     ./run.sh                          # the full campaign: nine points and the scalar
#     ./run.sh 1 2 3 7                  # only the points named
#     ./run.sh wl-002                   # only the scalar
#     ./run.sh --proving --runs 20 --warmups 3
#                                       # a reduced run that proves the harness works
#     ./run.sh --help
#
# Options:
#     --runs N            timing runs per point (default 200, the NFR-PERF-009 floor)
#     --warmups N         warmup runs per point (default 20, the same floor)
#     --proving           permit a run below either floor, and say so in every
#                         record it produces
#     --series NAME       the series of FR-SRV-015 to measure against (default 12.3)
#     --tls MODE          the transport the benchmark entries ask for (default disabled)
#     --bin PATH          the binary under measurement (default target/release/tpl)
#     --target TRIPLE     state the target of NFR-PERF-018 instead of detecting it
#     --work-dir PATH     where the projects and the raw samples go
#                         (default target/bench-work)
#     --out FILE          write the records here as well as to stdout
#
# Output:
#     stdout   one JSON record per line, one line per measurement point
#     stderr   progress, and everything the fixture harness prints
#
# Exit code:
#     0   the campaign ran to the end
#     2   the invocation is wrong, or the environment cannot support it —
#         a missing tool, no binary, a target that is none of the four, a
#         fixture that would not stand up
#
# NO FIGURE CHANGES THE EXIT CODE. A campaign that completes exits `0` whatever
# it measured. This is an instrument: it reads no earlier figure, compares
# nothing with anything, computes no delta, and passes no judgement on a number.
# High, low, slow or dispersed, the reading is printed in full and the run
# succeeds. `BR-PERF-008` is the rule underneath that: no figure of this corpus
# fails, blocks, rejects or gates anything.
#
# WHAT THIS DOES NOT DO. It does not write `BENCHMARKS.md`, and it does not
# decide what any figure means. `NFR-PERF-011`'s five per cent is reported as
# the plain field `rsd_over_5pct` beside the dispersion it describes, for a
# reader to apply; the harness itself withholds nothing on account of it.

set -euo pipefail

# Every number this harness reads and writes uses a full stop for its decimal
# point, whatever the operator's locale does. `/usr/bin/time` on a Portuguese
# macOS prints `0,00 real`, and an `awk` under that locale would parse a median
# as zero.
export LC_ALL=C

BENCHES="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$BENCHES/.." && pwd)"

# shellcheck source=benches/protocol.sh
. "$BENCHES/protocol.sh"
# shellcheck source=benches/fixture.sh
. "$BENCHES/fixture.sh"

FIXTURE_DIR="$ROOT/scripts/mariadb"

# ----------------------------------------------------------------- defaults ---

RUNS="$PROTOCOL_MIN_RUNS"
WARMUPS="$PROTOCOL_MIN_WARMUPS"
PROVING=no
SERIES='12.3'
TLS='disabled'
BINARY="$ROOT/target/release/tpl"
WORK="$ROOT/target/bench-work"
TARGET=''
TARGET_SOURCE='detected'
OUT=''
SELECTED=()

# Filled in by `prepare`.
REC_HOST=''
REC_TAKEN_AT=''
REC_DIGEST='null'
REC_METHOD=''
METHOD_DEFAULT=''
STANDING='full'

# The template the canonical loop renders. `tpl init` writes it, so every
# project this harness builds carries it and nothing has to be authored here.
TEMPLATE='example'

FIXTURE_STARTED=no
MEASURED=0

# -------------------------------------------------------------------- usage ---

usage() {
    awk 'NR >= 2 { if ($0 !~ /^#/) exit; sub(/^# ?/, ""); print }' "$BENCHES/run.sh"
}

die() {
    printf 'benches/run.sh: %s\n' "$*" >&2
    exit 2
}

say() {
    printf '%s\n' "$*" >&2
}

# --------------------------------------------------------- argument parsing ---

parse_arguments() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --runs)      [ "$#" -ge 2 ] || die "--runs takes a value"; RUNS="$2"; shift 2 ;;
            --warmups)   [ "$#" -ge 2 ] || die "--warmups takes a value"; WARMUPS="$2"; shift 2 ;;
            --series)    [ "$#" -ge 2 ] || die "--series takes a value"; SERIES="$2"; shift 2 ;;
            --tls)       [ "$#" -ge 2 ] || die "--tls takes a value"; TLS="$2"; shift 2 ;;
            --bin)       [ "$#" -ge 2 ] || die "--bin takes a value"; BINARY="$2"; shift 2 ;;
            --target)    [ "$#" -ge 2 ] || die "--target takes a value"
                         TARGET="$2"; TARGET_SOURCE='given'; shift 2 ;;
            --work-dir)  [ "$#" -ge 2 ] || die "--work-dir takes a value"; WORK="$2"; shift 2 ;;
            --out)       [ "$#" -ge 2 ] || die "--out takes a value"; OUT="$2"; shift 2 ;;
            --proving)   PROVING=yes; shift ;;
            -h|--help)   usage; exit 0 ;;
            -*)          die "unknown option: $1" ;;
            1|2|3|4|5|6|7|8|9|wl-002)
                         SELECTED=(${SELECTED[@]+"${SELECTED[@]}"} "$1"); shift ;;
            all)         shift ;;
            *)           die "unknown measurement point: $1 (the points are 1..9 and wl-002)" ;;
        esac
    done

    case "$RUNS" in ''|*[!0-9]*) die "--runs takes a whole number" ;; esac
    case "$WARMUPS" in ''|*[!0-9]*) die "--warmups takes a whole number" ;; esac

    if [ "${#SELECTED[@]}" -eq 0 ]; then
        SELECTED=(1 2 3 4 5 6 7 8 9 wl-002)
    fi

    # `NFR-PERF-009` is a floor on how a reading is taken, and it is kept. A
    # run below it is allowed, because proving that the harness works end to end
    # must not cost an hour, and every record such a run produces says
    # `standing: reduced` so that nobody reads it as a measurement taken under
    # the full protocol.
    if [ "$RUNS" -lt "$PROTOCOL_MIN_RUNS" ] || [ "$WARMUPS" -lt "$PROTOCOL_MIN_WARMUPS" ]; then
        [ "$PROVING" = yes ] || die \
            "NFR-PERF-009 takes a measurement over at least $PROTOCOL_MIN_RUNS runs after at least $PROTOCOL_MIN_WARMUPS warmups; pass --proving to take a reduced run, which every record will say it was"
        STANDING='reduced'
    fi
}

# Was `$1` selected?
selected() {
    local wanted="$1" one
    for one in "${SELECTED[@]}"; do
        if [ "$one" = "$wanted" ]; then
            return 0
        fi
    done
    return 1
}

# Is any of `$@` selected?
any_selected() {
    local one
    for one in "$@"; do
        if selected "$one"; then
            return 0
        fi
    done
    return 1
}

# ------------------------------------------------------------------ the rig ---

prepare() {
    protocol_require hyperfine jq awk sed sort file || exit 2

    [ -x "$BINARY" ] || die "no binary at $BINARY; run cargo build --release, or pass --bin"

    BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"

    if [ -z "$TARGET" ]; then
        TARGET="$(protocol_target "$BINARY")" || die \
            "this host is none of the four targets of NFR-PERF-018, or the binary is not statically linked; state it with --target"
    fi

    case "$TARGET" in
        x86_64-unknown-linux-musl|aarch64-unknown-linux-musl|x86_64-apple-darwin|aarch64-apple-darwin) ;;
        *) die "$TARGET is not one of the four targets of NFR-PERF-018" ;;
    esac

    mkdir -p "$WORK" "$WORK/hyperfine" "$WORK/samples"

    REC_HOST="$(protocol_host)"
    REC_TAKEN_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    local digest
    digest="$(protocol_digest "$BINARY")"
    if [ -n "$digest" ]; then
        REC_DIGEST="\"$digest\""
    fi

    METHOD_DEFAULT="$(protocol_instrument), --shell=none"

    if [ -n "$OUT" ]; then
        : > "$OUT"
    fi

    # `NFR-PERF-010`: the first execution of a freshly built binary is
    # discarded. It happens here, once, before any sample of any point.
    protocol_discard_first_execution "$BINARY"

    say "target   $TARGET ($TARGET_SOURCE)"
    say "binary   $BINARY"
    say "protocol $RUNS runs after $WARMUPS warmups, standing: $STANDING"
    say "work     $WORK"
}

# ------------------------------------------------------- record bookkeeping ---

# Resets the per-record metadata to what most points carry, so that each point
# states only what is its own.
#
# `REC_CACHE` carries the words of the `Cache` column `NFR-PERF-014` gives the
# point, which `NFR-PERF-020` obliges the record to state. `not reached` is the
# column's entry for the four points whose workload is `none`, so it is the
# default here and the other six say their own.
record_defaults() {
    REC_ID=''
    REC_POINT='null'
    REC_NAME=''
    REC_WORKLOAD='none'
    REC_QUANTITY='wall_time'
    REC_UNIT='ms'
    REC_TARGET="$TARGET"
    REC_TARGET_SOURCE="$TARGET_SOURCE"
    REC_SERIES='null'
    REC_TLS='null'
    REC_SERVER='not required'
    REC_CACHE='not reached'
    REC_COMMAND=''
    REC_WARMUPS="$WARMUPS"
    REC_STANDING="$STANDING"
    REC_METHOD="$METHOD_DEFAULT"
    REC_AGGREGATION='null'
    REC_PARTS='null'
    REC_NOTE='null'
    REC_BINARY="$BINARY"
    SLUG=''

    # `REC_DIGEST`, `REC_HOST` and `REC_TAKEN_AT` are not reset: they describe
    # the campaign rather than the reading, `prepare` sets them once, and they
    # are the same in every record a run produces.
}

# The command as a reader would retype it: the binary by its base name, every
# argument as written, and a quoted argument only where one is needed.
display_argv() {
    local piece out=''
    local first=yes

    for piece in "$@"; do
        if [ "$first" = yes ]; then
            piece="$(basename "$piece")"
            first=no
        fi
        case "$piece" in
            *[[:space:]]*) piece="'$piece'" ;;
        esac
        out="$out $piece"
    done

    printf '%s\n' "${out# }"
}

# Emits the record `STAT_*` and `REC_*` describe, and counts it.
#
# Nothing here inspects the figure. The record goes out whole, on stdout and —
# when one was asked for — into the file, and one line of it is echoed to
# stderr so that an operator watching a long campaign can see it arrive.
emit() {
    local line

    MEASURED=$((MEASURED + 1))
    line="$(protocol_emit)"

    printf '%s\n' "$line"
    if [ -n "$OUT" ]; then
        printf '%s\n' "$line" >> "$OUT"
    fi

    say "  $(printf '%s' "$line" | jq -r '"\(.id)  median=\(.median) \(.unit)  n=\(.n)  rsd=\(.rsd_pct * 100 | round / 100)%"')"
}

# Samples a wall-time subject and leaves `STAT_*` set.
#
#     wall <ignore-failure> <prepare-command|""> <working-directory> <argv...>
wall() {
    local ignore="$1" prepare="$2" cwd="$3"
    shift 3

    local command samples
    command="$(protocol_quote_argv "$@")"
    REC_COMMAND="$(display_argv "$@")"
    samples="$WORK/samples/$SLUG.txt"

    (
        cd "$cwd"
        protocol_sample_wall "$RUNS" "$WARMUPS" "$ignore" "$prepare" \
            "$WORK/hyperfine/$SLUG.json" "$command"
    ) > "$samples"

    protocol_stats < "$samples"
}

# ------------------------------------------------- the four free points ---

# `NFR-PERF-014` rows 1, 2, 3 and 7: workload `none`, no fixture, no server,
# and a cache the invocation never reaches.
point_1() {
    record_defaults
    REC_ID='point-1'; SLUG='point-1'; REC_POINT=1
    REC_NAME='tpl --version'

    wall no '' "$WORK" "$BINARY" --version
    emit
}

point_2() {
    record_defaults
    REC_ID='point-2'; SLUG='point-2'; REC_POINT=2
    REC_NAME='tpl --help'

    wall no '' "$WORK" "$BINARY" --help
    emit
}

# `NFR-PERF-014` row 3: startup to the first byte of useful work.
#
# The invocation is not this harness's to choose. The thirty-sixth edition
# settled it in the row itself — `tpl template list` in a project holding no
# database entry — on the ground that it is the cheapest invocation which is
# **useful work** rather than static text: it discovers a project, reads a
# configuration and presents a result of its own, where `NFR-PERF-005` excuses
# every form of help and of version from discovery and from reading a
# configuration. That is also why this row's adopted figure is twice theirs.
#
# What the instrument actually times is the whole process, because hyperfine
# times a process and not a byte of its output; for a command whose work is
# listing two files, that is startup plus a rounding error. The record says so
# in its `note`.
point_3() {
    record_defaults
    REC_ID='point-3'; SLUG='point-3'; REC_POINT=3
    REC_NAME='Startup to the first byte of useful work, measured by tpl template list in a project holding no database entry'
    REC_NOTE='"the invocation is the one NFR-PERF-014 names for this row: tpl template list in a project holding no database entry. hyperfine times the whole process rather than the first byte of output; for a command whose work is listing two files that is startup plus a rounding error."'

    fixture_startup_project "$WORK" "$BINARY"

    wall no '' "$WORK/startup" "$BINARY" template list
    emit
}

point_7() {
    record_defaults
    REC_ID='point-7'; SLUG='point-7'; REC_POINT=7
    REC_NAME='tpl help --format json'

    wall no '' "$WORK" "$BINARY" help --format json
    emit
}

# ----------------------------------------------- the points with a server ---

# `NFR-PERF-014` row 4: `tpl schema dump` over `WL-001`, server time included.
#
# The `Cache` column of that row says `bypassed, --direct --no-cache`, and the
# thirty-sixth edition states why: the read is read-through by default, per
# `FR-CACHE-006` and `FR-CACHE-007`, so without those two flags the first run
# would reach the server and the other 199 would be served from the cache, and
# the median — which is what the protocol records — would be a cache figure
# under a row that says a server answered. `FR-CACHE-016` fixes the two flags as
# the pure read.
point_4() {
    record_defaults
    REC_ID='point-4'; SLUG='point-4'; REC_POINT=4
    REC_NAME='tpl schema dump'
    REC_WORKLOAD='WL-001'
    REC_SERIES="\"$SERIES\""
    REC_TLS="\"$TLS\""
    REC_SERVER='up'
    REC_CACHE='bypassed, --direct --no-cache'
    REC_NOTE='"--direct --no-cache is the Cache column of this row of NFR-PERF-014, and FR-CACHE-016 fixes it as the pure read: every run reaches the server."'

    wall no '' "$WORK/server" \
        "$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema dump --direct --no-cache
    emit
}

# `NFR-PERF-014` row 8: the canonical loop of 200 invocations over `WL-001`.
#
# The `Cache` column of that row says `empty when each run begins`, and the
# thirty-sixth edition states that emptying it is not part of what is measured.
# `--prepare` is what satisfies both halves: hyperfine runs it before each
# timing run and excludes it from the figure. Each run is then one server read
# and 199 cache hits, which is the same work every time and is what a caller's
# loop over 200 objects actually does.
point_8() {
    record_defaults
    REC_ID='point-8'; SLUG='point-8'; REC_POINT=8
    REC_NAME='The canonical loop of 200 invocations'
    REC_WORKLOAD='WL-001'
    REC_SERIES="\"$SERIES\""
    REC_TLS="\"$TLS\""
    REC_SERVER='up'
    REC_CACHE='empty when each run begins'
    REC_NOTE='"benches/loop200.sh drives the loop: one tpl render per table, 200 process startups, as BR-PERF-005 requires. The cache is emptied by hyperfine --prepare before each run and is excluded from what it times, which is the Cache column of this row of NFR-PERF-014; each run is one server read and 199 cache hits."'

    local clean
    clean="$(protocol_quote_argv "$BINARY" -d "$FIXTURE_ENTRY_LARGE" cache clean)"

    wall no "$clean" "$WORK/server" \
        "$BENCHES/loop200.sh" "$BINARY" "$FIXTURE_ENTRY_LARGE" "$TEMPLATE" \
        "$FIXTURE_WL001_NAMES"
    emit
}

# `NFR-PERF-014` row 9: peak resident memory over `WL-001`.
#
# `ru_maxrss`, taken outside the process, over the same invocation row 4
# measures and under the same `Cache` column entry — `bypassed,
# --direct --no-cache`. The thirty-sixth edition states the ground: the
# whole-catalogue read is the memory-heaviest thing `tpl` does over this
# workload, and the adopted figure that row carries was stated for a database of
# 200 tables.
point_9() {
    record_defaults
    REC_ID='point-9'; SLUG='point-9'; REC_POINT=9
    REC_NAME='Peak resident memory'
    REC_WORKLOAD='WL-001'
    REC_QUANTITY='peak_rss'
    REC_UNIT='bytes'
    REC_SERIES="\"$SERIES\""
    REC_TLS="\"$TLS\""
    REC_SERVER='up'
    REC_CACHE='bypassed, --direct --no-cache'

    local system
    system="$(uname -s)"
    if [ "$system" = Darwin ]; then
        REC_METHOD="/usr/bin/time -l, 'maximum resident set size', bytes"
    else
        REC_METHOD="/usr/bin/time -v, 'Maximum resident set size (kbytes)', scaled to bytes"
    fi
    REC_NOTE='"ru_maxrss is read from outside the process and never from inside it. hyperfine does not take part in this point: it measures time."'

    local samples argv
    samples="$WORK/samples/$SLUG.txt"
    argv=("$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema dump --direct --no-cache)
    REC_COMMAND="$(display_argv "${argv[@]}")"

    (
        cd "$WORK/server"
        protocol_sample_rss "$RUNS" "$WARMUPS" "$WORK/samples/$SLUG.time" "${argv[@]}"
    ) > "$samples"

    protocol_stats < "$samples"
    emit
}

# `WL-002`, the verification scalar: the byte size of the compact
# `tpl schema dump` of `WL-001`.
#
# It is not a row of `NFR-PERF-014` and carries no `Cache` column, so the
# posture below is this harness's and is stated as such: the dump is taken
# `--direct --no-cache` so that what is measured is the server-read document,
# whose envelope carries `source: server`.
#
# Compact is the default of `FR-OUT-007` — `--pretty` is what departs from it —
# so the invocation is the plain dump and the scalar is the length of what it
# wrote. It is taken more than once and described by the same statistics as
# everything else, because a document whose size is not stable is the very thing
# `WL-002` exists to detect. `WL-002` is a deterministic size and not a timing:
# a departure beyond its ±2% is a statement about the fixture or the document
# shape, which `BR-PERF-008` leaves standing as a functional matter.
scalar_wl002() {
    record_defaults
    REC_ID='wl-002'; SLUG='wl-002'
    REC_NAME='WL-002, the verification scalar'
    REC_WORKLOAD='WL-001'
    REC_QUANTITY='document_size'
    REC_UNIT='bytes'
    REC_WARMUPS=0
    REC_SERIES="\"$SERIES\""
    REC_TLS="\"$TLS\""
    REC_SERVER='up'
    REC_CACHE='bypassed, --direct --no-cache; WL-002 is not a row of NFR-PERF-014 and fixes no posture'
    REC_METHOD='the byte length of what the compact dump wrote to stdout'
    REC_NOTE='"compact is the default of FR-OUT-007; --pretty is not passed. The dump is server-read, so its envelope carries source=server: a cache-served dump of the same catalogue differs from this figure by the two bytes of that word."'

    local samples argv
    samples="$WORK/samples/$SLUG.txt"
    argv=("$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema dump --direct --no-cache)
    REC_COMMAND="$(display_argv "${argv[@]}")"

    local takes=5
    if [ "$STANDING" = reduced ]; then
        takes=2
    fi

    (cd "$WORK/server" && protocol_sample_bytes "$takes" "${argv[@]}") > "$samples"

    protocol_stats < "$samples"
    emit
}

# ---------------------------------------------- the points with no server ---

# `NFR-PERF-014` row 5: a cache-served read of one object over `WL-003`.
#
# The row says `Server: no` and `Cache: served from`, so this is measured with
# the server **down**: the cache was filled while it was up, and the gate is
# asked again here to prove that nothing could have answered. Taking the server
# away is what makes `served from` an observation rather than an intention.
point_5() {
    record_defaults
    REC_ID='point-5'; SLUG='point-5'; REC_POINT=5
    REC_NAME='A cache-served read of one object'
    REC_WORKLOAD='WL-003'
    REC_SERVER='down'
    REC_CACHE='served from'
    REC_NOTE='"the server was taken down before this point was measured, and the gate of scripts/mariadb/status.sh was asked again to prove it, so the Server and Cache columns NFR-PERF-014 gives this row are observed and not assumed."'

    wall no '' "$WORK/server" \
        "$BINARY" -d "$FIXTURE_ENTRY_SMALL" schema table "$FIXTURE_WL003_TABLE"
    emit
}

# `NFR-PERF-014` row 6: the failure path — a `64`, and a `66` with nearest match
# over every existing name of `WL-001`.
#
# **One point measured by two invocations, and its figure is the slower of the
# two.** That is the row's own rule, settled in the thirty-sixth edition, and
# both invocations are recorded, each in full, beside the figure that stands for
# the point. Each half is sampled on its own and is printed whole under `parts`;
# the point's own figures are the slower half's, in full, so that the `median`,
# the `mean` and the dispersion this record carries all describe one reading
# rather than two averaged together. `aggregation` says which rule chose them.
#
# The row says `Cache: served from`, and the `64` is refused before any read, so
# it reaches neither cache nor server. That is the column describing the point,
# which the thirty-sixth edition states in as many words.
point_6() {
    record_defaults
    REC_ID='point-6'; SLUG='point-6'; REC_POINT=6
    REC_NAME='The failure path: a 64, and a 66 with nearest match'
    REC_WORKLOAD='WL-001'
    REC_SERVER='down'
    REC_CACHE='served from'
    REC_AGGREGATION='"the slower of the two invocations, per NFR-PERF-014"'
    REC_NOTE='"the 66 names a table one edit away from a real one, so FR-ERR-020 offers a suggestion rather than withholding it and the edit distance of FR-ERR-019 is computed against all 200 names of WL-001, which is what BR-PERF-004 says this point exists to measure. The 64 is refused before any read and reaches neither cache nor server."'

    local first_real part_usage part_missing
    local usage_n usage_median usage_mean usage_stddev usage_min usage_max usage_rsd
    local missing_n missing_median missing_mean missing_stddev missing_min missing_max
    local missing_rsd
    local usage_command missing_command

    first_real="$(head -1 "$FIXTURE_WL001_NAMES")"

    SLUG='point-6-64'
    wall yes '' "$WORK/server" \
        "$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema table "$first_real" --no-such-flag
    usage_command="$REC_COMMAND"
    part_usage="$(protocol_part '64, an undeclared flag' "$usage_command" "$WARMUPS")"
    usage_n="$STAT_N"; usage_median="$STAT_MEDIAN"; usage_mean="$STAT_MEAN"
    usage_stddev="$STAT_STDDEV"; usage_min="$STAT_MIN"; usage_max="$STAT_MAX"
    usage_rsd="$STAT_RSD"

    SLUG='point-6-66'
    wall yes '' "$WORK/server" \
        "$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema table "$FIXTURE_ABSENT_NAME"
    missing_command="$REC_COMMAND"
    part_missing="$(protocol_part '66, with nearest match' "$missing_command" "$WARMUPS")"
    missing_n="$STAT_N"; missing_median="$STAT_MEDIAN"; missing_mean="$STAT_MEAN"
    missing_stddev="$STAT_STDDEV"; missing_min="$STAT_MIN"; missing_max="$STAT_MAX"
    missing_rsd="$STAT_RSD"

    SLUG='point-6'
    REC_PARTS="[$part_usage,$part_missing]"

    # The point's own figures are the slower half's, in full, which is what
    # `NFR-PERF-014` fixes for this row, so that the `median`, the `mean` and
    # the dispersion this record carries all describe one measurement rather
    # than two averaged together.
    if awk -v a="$usage_median" -v b="$missing_median" 'BEGIN { exit !(a > b) }'; then
        STAT_N="$usage_n"; STAT_MEDIAN="$usage_median"; STAT_MEAN="$usage_mean"
        STAT_STDDEV="$usage_stddev"; STAT_MIN="$usage_min"; STAT_MAX="$usage_max"
        STAT_RSD="$usage_rsd"
        REC_COMMAND="$usage_command"
    else
        STAT_N="$missing_n"; STAT_MEDIAN="$missing_median"; STAT_MEAN="$missing_mean"
        STAT_STDDEV="$missing_stddev"; STAT_MIN="$missing_min"; STAT_MAX="$missing_max"
        STAT_RSD="$missing_rsd"
        REC_COMMAND="$missing_command"
    fi

    emit
}

# ---------------------------------------------------------------- the fixture ---

teardown() {
    if [ "$FIXTURE_STARTED" = yes ]; then
        FIXTURE_STARTED=no
        fixture_down "$SERIES" || true
    fi
}

stand_the_fixture_up() {
    fixture_inventory

    fixture_is_series "$SERIES" || die \
        "$SERIES is not one of the four series of FR-SRV-015; scripts/mariadb/series.env names them"

    say "fixture  bringing $SERIES up"
    FIXTURE_STARTED=yes
    fixture_up "$SERIES"
    fixture_seed "$SERIES"
    fixture_address "$SERIES" || die "the gate did not report an address for $SERIES"

    say "fixture  $SERIES answered on $FIXTURE_HOST:$FIXTURE_PORT"

    fixture_server_project "$WORK" "$BINARY" "$TLS"
    fixture_prime "$WORK" "$BINARY"
    fixture_subjects "$WORK" "$BINARY" || die "the workloads are not what series.env states"

    say "fixture  WL-001 presents $(wc -l < "$FIXTURE_WL001_NAMES" | tr -d ' ') tables; WL-003 presents $FIXTURE_WL003_TABLE"
    say "fixture  the failure path will name $FIXTURE_ABSENT_NAME, which is one edit from a real table"

    verify_the_failure_path
}

# The failure path is this point only if it fails the way the row says. Both
# halves are run once, here, and their exit codes are checked before either is
# measured 200 times.
verify_the_failure_path() {
    local first code

    first="$(head -1 "$FIXTURE_WL001_NAMES")"

    set +e
    (cd "$WORK/server" && "$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema table "$first" \
        --no-such-flag >/dev/null 2>&1)
    code=$?
    set -e
    [ "$code" -eq 64 ] || die "the 64 half of point 6 exited $code, not 64"

    set +e
    (cd "$WORK/server" && "$BINARY" -d "$FIXTURE_ENTRY_LARGE" schema table \
        "$FIXTURE_ABSENT_NAME" >/dev/null 2>"$WORK/samples/nearest.txt")
    code=$?
    set -e
    [ "$code" -eq 66 ] || die "the 66 half of point 6 exited $code, not 66"

    grep -q 'hint' "$WORK/samples/nearest.txt" || die \
        "the 66 half of point 6 offered no suggestion, so it does not measure what BR-PERF-004 says it measures"
}

take_the_fixture_down() {
    say "fixture  taking $SERIES down, for the two points NFR-PERF-014 gives Server: no"
    fixture_prime "$WORK" "$BINARY"
    teardown

    if fixture_is_up "$SERIES" >/dev/null 2>&1; then
        die "$SERIES is still answering after down.sh, and points 5 and 6 need it silent"
    fi
}

# ---------------------------------------------------------------------- main ---

main() {
    parse_arguments "$@"
    prepare

    trap teardown EXIT INT TERM

    if selected 1; then point_1; fi
    if selected 2; then point_2; fi
    if selected 3; then point_3; fi
    if selected 7; then point_7; fi

    if any_selected 4 5 6 8 9 wl-002; then
        stand_the_fixture_up

        if selected 4; then point_4; fi
        if selected wl-002; then scalar_wl002; fi
        if selected 8; then point_8; fi
        if selected 9; then point_9; fi

        take_the_fixture_down

        if selected 5; then point_5; fi
        if selected 6; then point_6; fi
    fi

    say ""
    say "$MEASURED reading(s) taken"
    if [ -n "$OUT" ]; then
        say "records written to $OUT"
    fi
}

main "$@"
