#!/usr/bin/env bash
#
# The measurement protocol of `NFR-PERF-009` through `NFR-PERF-012`, as code.
#
# This file is sourced by `run.sh` and is never run on its own. It holds the
# whole of the protocol and nothing about any particular budget: what a sample
# is, how a set of samples becomes a statistic, and what a record carries. A
# budget names an invocation; this file says how it is measured and what is
# printed about it.
#
# THIS IS AN INSTRUMENT AND NOT A GATE. It measures the machine as it finds it
# and prints what it saw. It reads no baseline, compares nothing with anything,
# computes no delta, passes no judgement on a figure, and never fails a run
# because a number came out high. A record is a reading; what anybody does with
# it is decided elsewhere and by a person.
#
# The three obligations it implements, each cited where it is enforced:
#
#     NFR-PERF-009  the median of at least 200 runs, after at least 20 warmups
#     NFR-PERF-010  no intervening shell, and the first execution of a freshly
#                   built binary is discarded
#     NFR-PERF-012  a recorded measurement names its target, and names the
#                   server series where a server answered
#
# `NFR-PERF-011` is the fourth, and it is treated as a **reported property of
# the reading** rather than as a gate: every record carries its relative
# standard deviation, and a boolean saying whether that dispersion is above
# five per cent. Nothing here withholds a figure, marks a run failed, or
# changes an exit code because of it. A reader who wants the rule that
# requirement states applies it to the field.

# ---------------------------------------------------------------- constants ---

# The dispersion `NFR-PERF-011` speaks of, in per cent. It is a **reporting
# threshold** here: a record says whether it is above this, and nothing else
# follows from the answer inside this harness.
PROTOCOL_RSD_THRESHOLD=5

# `NFR-PERF-009`, the two floors a measurement under the full protocol is taken
# at. They govern how a reading is taken, which is why they are kept.
PROTOCOL_MIN_RUNS=200
PROTOCOL_MIN_WARMUPS=20

# The schema every record carries in its first field, so that a consumer can
# tell one generation of this harness from another.
PROTOCOL_RECORD_SCHEMA='tpl-bench/1'

# ------------------------------------------------------------------- tools ---

# Refuses to start without a tool the protocol depends on.
protocol_require() {
    local tool missing=''
    for tool in "$@"; do
        command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
    done

    if [ -n "$missing" ]; then
        printf 'benches/run.sh: this harness needs%s on PATH\n' "$missing" >&2
        return 2
    fi
}

# The version string of the instrument, for the record's `method` field.
protocol_instrument() {
    hyperfine --version 2>/dev/null | head -1
}

# ------------------------------------------------------------------ target ---

# The target of `NFR-PERF-018` this host is, or nothing when it is none of the
# four.
#
# Nothing here is hardcoded to the host the harness was written on: the triple
# is derived from `uname` at run time, and a Linux host is only one of the four
# when the binary under measurement is statically linked, which is what that
# requirement's two Linux rows state. Anything else is left unnamed rather than
# guessed at, and `--target` is how a caller states what detection cannot.
protocol_target() {
    local binary="$1" system machine

    system="$(uname -s)"
    machine="$(uname -m)"

    case "$system/$machine" in
        Darwin/arm64)   printf 'aarch64-apple-darwin\n' ;;
        Darwin/x86_64)  printf 'x86_64-apple-darwin\n' ;;
        Linux/aarch64)  protocol_linux_target aarch64 "$binary" ;;
        Linux/arm64)    protocol_linux_target aarch64 "$binary" ;;
        Linux/x86_64)   protocol_linux_target x86_64 "$binary" ;;
        *)              return 1 ;;
    esac
}

# The Linux half of the detection. `NFR-PERF-018` names `musl`, statically
# linked, for both Linux rows, so a dynamically linked binary is not one of the
# four targets and is not reported as one.
protocol_linux_target() {
    local architecture="$1" binary="$2"

    if file "$binary" 2>/dev/null | grep -q 'statically linked'; then
        printf '%s-unknown-linux-musl\n' "$architecture"
        return 0
    fi

    return 1
}

# A one-line description of the host, for the record.
protocol_host() {
    printf '%s %s %s\n' "$(uname -s)" "$(uname -r)" "$(uname -m)"
}

# The digest of the binary under measurement, so that two records can be told
# apart when they were taken against two builds.
protocol_digest() {
    local binary="$1"

    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$binary" | awk '{print $1}'
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$binary" | awk '{print $1}'
    fi
}

# `NFR-PERF-010`: the first execution of a freshly built binary is discarded.
# It is executed here, once, before any sample of any budget is taken, and its
# result is thrown away.
protocol_discard_first_execution() {
    "$1" --version >/dev/null 2>&1 || true
}

# ------------------------------------------------------------- the quoting ---

# One argv, as the single command string `hyperfine --shell=none` parses back
# into that same argv.
#
# hyperfine splits a command string on shell-like word boundaries even with the
# shell disabled, so an argument carrying a space would otherwise become two.
# Every element is single-quoted here and every embedded quote is escaped,
# which makes the split exact for any argument a budget can name.
protocol_quote_argv() {
    local piece out=''

    for piece in "$@"; do
        # `'` closes the quote, adds an escaped quote, and opens it again.
        piece="$(printf '%s' "$piece" | sed "s/'/'\\\\''/g")"
        out="$out '$piece'"
    done

    printf '%s\n' "${out# }"
}

# ---------------------------------------------------------------- the stats ---

# Reads one number per line and sets `STAT_N`, `STAT_MEDIAN`, `STAT_MEAN`,
# `STAT_STDDEV`, `STAT_MIN`, `STAT_MAX` and `STAT_RSD`.
#
# One implementation serves every quantity the harness measures — wall time,
# peak resident memory and document size — so that `median` and `relative
# standard deviation` mean exactly one thing across the whole record set.
#
# The median of an even-sized sample is the mean of the two middle values. The
# standard deviation is the sample standard deviation, with `n - 1` in the
# denominator. The relative standard deviation is the standard deviation over
# the mean, in per cent.
protocol_stats() {
    local computed

    computed="$(sort -g | awk '
        { value[NR] = $1 + 0; total += $1 }
        END {
            n = NR
            if (n == 0) { print "0 0 0 0 0 0 0"; exit }

            mean = total / n
            if (n % 2) { median = value[(n + 1) / 2] }
            else       { median = (value[n / 2] + value[n / 2 + 1]) / 2 }

            if (n > 1) {
                for (i = 1; i <= n; i++) {
                    deviation = value[i] - mean
                    sum_of_squares += deviation * deviation
                }
                stddev = sqrt(sum_of_squares / (n - 1))
            }

            rsd = (mean != 0) ? stddev / mean * 100 : 0

            printf "%d %.10g %.10g %.10g %.10g %.10g %.10g\n", \
                n, median, mean, stddev, value[1], value[n], rsd
        }')"

    STAT_N="$(printf '%s' "$computed" | awk '{print $1}')"
    STAT_MEDIAN="$(printf '%s' "$computed" | awk '{print $2}')"
    STAT_MEAN="$(printf '%s' "$computed" | awk '{print $3}')"
    STAT_STDDEV="$(printf '%s' "$computed" | awk '{print $4}')"
    STAT_MIN="$(printf '%s' "$computed" | awk '{print $5}')"
    STAT_MAX="$(printf '%s' "$computed" | awk '{print $6}')"
    STAT_RSD="$(printf '%s' "$computed" | awk '{print $7}')"
}

# `true` when the dispersion of the reading just taken is above the threshold
# of `NFR-PERF-011`, `false` when it is not.
#
# This is a **description of the reading**, printed in the record beside the
# dispersion it describes. Nothing in this harness branches on it: a dispersed
# reading is recorded in full, exactly like any other, and the caller's exit
# code is the same either way.
protocol_rsd_over_threshold() {
    if awk -v rsd="$STAT_RSD" -v threshold="$PROTOCOL_RSD_THRESHOLD" \
        'BEGIN { exit !(rsd > threshold) }'; then
        printf 'true\n'
    else
        printf 'false\n'
    fi
}

# ---------------------------------------------------------------- the samples ---

# Wall-time samples, in milliseconds, one per line.
#
#     protocol_sample_wall <runs> <warmups> <ignore-failure> <prepare> <export> <command>
#
# `ignore-failure` is `yes` for a budget whose subject exits non-zero by
# design, which is the failure path of `NFR-PERF-014` and nothing else.
# `prepare` is a command string run before each timing run and excluded from
# it, or empty. `command` is one already-quoted command string.
#
# `--shell=none` is `NFR-PERF-010`'s no-intervening-shell clause, and it is
# passed on every invocation without exception.
protocol_sample_wall() {
    local runs="$1" warmups="$2" ignore="$3" prepare="$4" export_to="$5" command="$6"
    local flags

    flags=(--shell=none --style none --warmup "$warmups" --runs "$runs")
    flags=("${flags[@]}" --export-json "$export_to")

    if [ "$ignore" = yes ]; then
        flags=("${flags[@]}" --ignore-failure)
    fi

    if [ -n "$prepare" ]; then
        flags=("${flags[@]}" --prepare "$prepare")
    fi

    hyperfine "${flags[@]}" "$command" >/dev/null || return 1

    # hyperfine reports seconds; every figure this harness states is in
    # milliseconds, and the conversion happens once, here.
    jq -r '.results[0].times[] | . * 1000' "$export_to"
}

# Peak resident memory samples, in bytes, one per line.
#
#     protocol_sample_rss <runs> <warmups> <scratch> <argv...>
#
# `ru_maxrss`, taken from outside the process, which is what `NFR-PERF-014`'s
# ninth row requires: the number is the kernel's accounting of the child and is
# never read from inside it.
#
#     macOS  /usr/bin/time -l, `maximum resident set size`, in bytes
#     Linux  /usr/bin/time -v, `Maximum resident set size (kbytes)`, scaled
#
# There is no hyperfine here: it reports time and not memory, and a second
# instrument for a second quantity is honest where a derived figure would not
# be.
protocol_sample_rss() {
    local runs="$1" warmups="$2" scratch="$3"
    shift 3

    local run system flag total
    system="$(uname -s)"

    case "$system" in
        Darwin) flag='-l' ;;
        Linux)  flag='-v' ;;
        *)      printf 'benches: no ru_maxrss instrument on %s\n' "$system" >&2; return 1 ;;
    esac

    if [ ! -x /usr/bin/time ]; then
        printf 'benches: /usr/bin/time is needed for the peak-memory budget\n' >&2
        return 1
    fi

    run=0
    while [ "$run" -lt "$warmups" ]; do
        /usr/bin/time "$flag" "$@" >/dev/null 2>/dev/null || true
        run=$((run + 1))
    done

    run=0
    while [ "$run" -lt "$runs" ]; do
        /usr/bin/time "$flag" "$@" >/dev/null 2>"$scratch" || true

        if [ "$system" = Darwin ]; then
            total="$(awk '/maximum resident set size/ { print $1; exit }' "$scratch")"
        else
            total="$(awk -F: '/Maximum resident set size/ { gsub(/ /, "", $2); print $2 * 1024; exit }' \
                "$scratch")"
        fi

        if [ -z "$total" ]; then
            printf 'benches: /usr/bin/time reported no maximum resident set size\n' >&2
            return 1
        fi
        printf '%s\n' "$total"

        run=$((run + 1))
    done
}

# Document-size samples, in bytes, one per line.
#
#     protocol_sample_bytes <runs> <argv...>
#
# The `WL-002` scalar, which is a size and not a duration: it needs no warmup,
# because nothing about it is timed, and it is taken more than once so that the
# record can report whether the size held still across the takes.
protocol_sample_bytes() {
    local runs="$1"
    shift

    local run=0
    while [ "$run" -lt "$runs" ]; do
        "$@" | wc -c | tr -d ' '
        run=$((run + 1))
    done
}

# ---------------------------------------------------------------- the record ---

# One record, on stdout, as one line of JSON.
#
# The caller sets the `REC_*` variables and the sampling functions set the
# `STAT_*` ones; this reads both and writes the line. Every figure it was given
# is printed. Nothing is withheld, nothing is compared with anything, and the
# function's return status is success whenever it managed to write a line.
#
# The caller sets these. Eight of them are **JSON literals** and not plain
# text, because each may be absent: `REC_BUDGET`, `REC_NORMATIVE`,
# `REC_SERIES`, `REC_TLS`, `REC_AGGREGATION`, `REC_PARTS`, `REC_NOTE` and
# `REC_DIGEST` are written either as `null` or as a quoted JSON value. The rest
# are plain text.
#
#     REC_ID REC_BUDGET REC_NAME REC_NORMATIVE REC_WORKLOAD REC_QUANTITY
#     REC_UNIT REC_TARGET REC_TARGET_SOURCE REC_SERIES REC_TLS REC_SERVER
#     REC_COMMAND REC_WARMUPS REC_STANDING REC_METHOD REC_AGGREGATION
#     REC_PARTS REC_NOTE REC_BINARY REC_DIGEST REC_HOST REC_TAKEN_AT
protocol_emit() {
    local dispersed

    dispersed="$(protocol_rsd_over_threshold)"

    jq -cn \
        --arg record "$PROTOCOL_RECORD_SCHEMA" \
        --arg id "$REC_ID" \
        --argjson budget "$REC_BUDGET" \
        --arg budget_name "$REC_NAME" \
        --argjson normative "$REC_NORMATIVE" \
        --arg workload "$REC_WORKLOAD" \
        --arg quantity "$REC_QUANTITY" \
        --arg unit "$REC_UNIT" \
        --arg target "$REC_TARGET" \
        --arg target_source "$REC_TARGET_SOURCE" \
        --argjson series "$REC_SERIES" \
        --argjson tls "$REC_TLS" \
        --arg server "$REC_SERVER" \
        --arg command "$REC_COMMAND" \
        --argjson n "$STAT_N" \
        --argjson warmups "$REC_WARMUPS" \
        --argjson median "$STAT_MEDIAN" \
        --argjson mean "$STAT_MEAN" \
        --argjson stddev "$STAT_STDDEV" \
        --argjson minimum "$STAT_MIN" \
        --argjson maximum "$STAT_MAX" \
        --argjson rsd_pct "$STAT_RSD" \
        --argjson rsd_over_5pct "$dispersed" \
        --arg standing "$REC_STANDING" \
        --arg protocol 'NFR-PERF-009..NFR-PERF-012' \
        --arg method "$REC_METHOD" \
        --argjson aggregation "$REC_AGGREGATION" \
        --argjson parts "$REC_PARTS" \
        --argjson note "$REC_NOTE" \
        --arg binary "$REC_BINARY" \
        --argjson binary_sha256 "$REC_DIGEST" \
        --arg host "$REC_HOST" \
        --arg taken_at "$REC_TAKEN_AT" \
        '{
            record: $record, id: $id, budget: $budget, budget_name: $budget_name,
            normative: $normative, workload: $workload,
            quantity: $quantity, unit: $unit,
            target: $target, target_source: $target_source,
            series: $series, tls: $tls, server: $server,
            command: $command,
            n: $n, warmups: $warmups,
            median: $median, mean: $mean, stddev: $stddev,
            min: $minimum, max: $maximum,
            rsd_pct: $rsd_pct, rsd_over_5pct: $rsd_over_5pct,
            standing: $standing, protocol: $protocol, method: $method,
            aggregation: $aggregation, parts: $parts, note: $note,
            binary: $binary, binary_sha256: $binary_sha256,
            host: $host, taken_at: $taken_at
        }'
}

# One part of a multi-part budget, as a JSON object, for the `parts` array.
#
# It carries its own `n`, `median` and dispersion, because a budget made of two
# invocations is two readings and each is worth printing whole.
protocol_part() {
    local label="$1" command="$2" warmups="$3" dispersed

    dispersed="$(protocol_rsd_over_threshold)"

    jq -cn \
        --arg part "$label" \
        --arg command "$command" \
        --argjson n "$STAT_N" \
        --argjson warmups "$warmups" \
        --argjson median "$STAT_MEDIAN" \
        --argjson mean "$STAT_MEAN" \
        --argjson stddev "$STAT_STDDEV" \
        --argjson minimum "$STAT_MIN" \
        --argjson maximum "$STAT_MAX" \
        --argjson rsd_pct "$STAT_RSD" \
        --argjson rsd_over_5pct "$dispersed" \
        '{
            part: $part, command: $command, n: $n, warmups: $warmups,
            median: $median, mean: $mean, stddev: $stddev,
            min: $minimum, max: $maximum,
            rsd_pct: $rsd_pct, rsd_over_5pct: $rsd_over_5pct
        }'
}
