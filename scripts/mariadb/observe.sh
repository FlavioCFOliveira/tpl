#!/usr/bin/env bash
#
# The instruments. NFR-PERF-007 forbids verifying a requirement of form by
# reading the source, and BR-SRV-003 says why, so nine requirements are held to
# an observation made from outside the process. Three instruments cover all
# nine:
#
#   statements   what the server received      the server's general log
#   connections  what the server accepted      the server's status counters
#   opens        what the process opened       a system-call tracer
#
# A fourth subcommand, `build`, is not one of them: it verifies no requirement
# and observes the server rather than the process. It reads the conditions the
# other three are taken under, and it is described above its own function.
#
# Usage:
#     ./observe.sh statements on|off|dump <server> [filters]
#     ./observe.sh connections <server> [--value]
#     ./observe.sh opens [--server <name>] [--backend auto|strace|container] -- <command...>
#     ./observe.sh build [<server>]
#
# <server> is a name from series.env: 10.11, 11.4, 11.8, 12.3 or notls.
#
# Every subcommand is a command you can also type by hand; README.md records
# each of them with the output it actually produced.

set -euo pipefail

# The header above is this script's own usage text, and `usage` reads it back
# out of the file. The name is taken before the `cd`, because `$0` is the path
# the caller wrote and stops resolving the moment the working directory changes:
# invoked as `scripts/mariadb/observe.sh`, the `cd` below lands in that folder
# and `awk` is then handed a path relative to it that does not exist.
SELF="$(basename "$0")"

cd "$(dirname "$0")"
. ./series.env

die() { printf 'observe.sh: %s\n' "$*" >&2; exit 2; }

usage() {
    awk 'NR >= 3 { if ($0 !~ /^#/) exit; sub(/^# ?/, ""); print }' "$SELF"
    exit 2
}

# --------------------------------------------------------------------------
# statements — the general log, which records every statement the server
# receives, and the Connect and Quit of every connection it accepts.
#
# `on` empties the log before enabling it, so a dump covers exactly the window
# under test. The observer's own connection is a connection like any other and
# is logged too; `dump` therefore drops connections made over the Unix socket by
# default, which is how this script and up.sh reach the server and is not how
# anything under test reaches it. --all keeps them.
#
# --catalogue admits two command types, and the count is what decides which.
# A client may send one statement in either of two ways, and the general log
# spells the two differently:
#
#   Query                     the text protocol: one row per statement issued
#   Prepare, then Execute     the binary protocol: one Prepare per distinct
#                             statement text, and one Execute per issue of it
#
# `tpl` uses the binary protocol for every catalogue statement and the text
# protocol for the three connection-start statements, so both spellings occur
# in one window and a filter naming one kind alone sees part of the read. The
# filter therefore admits Query and Execute, which are the two rows that mean
# *a statement was issued*, and excludes Prepare, which means *a statement text
# was registered*: a statement issued twice on one connection is prepared once
# and executed twice, so counting Prepare counts texts rather than issues, and
# admitting both Prepare and Execute would count every prepared statement
# twice. Observed on 2026-09-20, on all four series: `tpl schema dump` leaves
# 11 Prepare rows and 11 Execute rows, and this filter returns 11.
# --------------------------------------------------------------------------

statements_on() {
    tpl_mariadb_sql "$1" -e "
        SET GLOBAL general_log = OFF;
        SET GLOBAL log_output  = 'TABLE';
        TRUNCATE TABLE mysql.general_log;
        SET GLOBAL general_log = ON;"
}

statements_off() { tpl_mariadb_sql "$1" -e "SET GLOBAL general_log = OFF;"; }

statements_dump() {
    local server="$1"; shift
    local where="user_host NOT LIKE '% @ localhost []'"
    local count=no

    while [ $# -gt 0 ]; do
        case "$1" in
            --all)       where="1 = 1" ;;
            # The general log spells the user two ways: a Query row carries
            # "u[u] @ host []" and the Connect row that opened the same
            # connection carries "[u] @  [host]". Matching on the bracketed
            # form is the one filter that keeps both.
            --user)      shift; where="$where AND user_host LIKE '%[$1]%'" ;;
            --kind)      shift; where="$where AND command_type = '$1'" ;;
            --queries)   where="$where AND command_type = 'Query'" ;;
            # Query and Execute, never Prepare: the header records why.
            --catalogue) where="$where AND command_type IN ('Query', 'Execute')
                                AND argument LIKE '%INFORMATION_SCHEMA%'" ;;
            --count)     count=yes ;;
            *) die "unknown filter for statements dump: $1" ;;
        esac
        shift
    done

    if [ "$count" = yes ]; then
        tpl_mariadb_sql "$server" -N -B -e \
            "SELECT COUNT(*) FROM mysql.general_log WHERE $where"
    else
        tpl_mariadb_sql "$server" -B -e "
            SELECT thread_id,
                   command_type,
                   REPLACE(REPLACE(CONVERT(argument USING utf8mb4), '\r', ' '), '\n', ' ') AS statement
            FROM mysql.general_log
            WHERE $where
            ORDER BY event_time, thread_id"
    fi
}

# --------------------------------------------------------------------------
# connections — the server's own count of what it accepted.
#
# Connections is monotonic and counts every connection attempt since the server
# started, so a test brackets the invocation under test with two readings and
# subtracts. The observer's own connection is counted too: two consecutive
# readings differ by 1 with nothing in between, and that 1 is the baseline to
# subtract. README.md records the measurement.
# --------------------------------------------------------------------------

connections() {
    local server="$1"; shift
    if [ "${1:-}" = --value ]; then
        tpl_mariadb_sql "$server" -N -B -e \
            "SELECT VARIABLE_VALUE FROM information_schema.GLOBAL_STATUS
             WHERE VARIABLE_NAME = 'CONNECTIONS'"
    else
        tpl_mariadb_sql "$server" -B -e "
            SHOW GLOBAL STATUS WHERE Variable_name IN
              ('Connections', 'Aborted_connects', 'Threads_connected', 'Max_used_connections')"
    fi
}

# --------------------------------------------------------------------------
# build — what identifies the build a server is, as against the series it
# belongs to.
#
# Not one of the three instruments above. It verifies no requirement and
# observes the server rather than the process under test. It is here because
# the values it reads are the conditions under which the other three are taken:
# this fixture selects a series and never pins a patch release, so two runs can
# observe two builds, and a reading that differs across the four servers is a
# difference between the series only once the build fails to explain it. That
# question cannot be asked without these values.
#
# It enumerates the `version%` prefix rather than asking for names. Asking for
# a name that does not exist returns no row, and no row is indistinguishable
# from a variable that exists and is empty — so a name misremembered by one
# character reads as a server that lacks the variable. An enumeration returns
# what the server has, which is also how a name is recovered when nobody wrote
# it down.
# --------------------------------------------------------------------------

build_one() {
    tpl_mariadb_sql "$1" -N -B -e "SHOW GLOBAL VARIABLES LIKE 'version%'" \
        | awk -F'\t' -v s="$1" '{ printf "%-7s %-24s %s\n", s, $1, $2 }'
}

build() {
    [ $# -le 1 ] || die "build takes at most one server"
    [ $# -eq 0 ] || tpl_mariadb_record "$1" >/dev/null || die "unknown server: $1"
    printf '%-7s %-24s %s\n' SERVER VARIABLE VALUE
    if [ $# -eq 1 ]; then
        build_one "$1"
    else
        tpl_mariadb_each build_one
    fi
}

# --------------------------------------------------------------------------
# opens — the files a process opens, and the sockets it connects.
#
# strace on the host when there is one, which observes the real process. On a
# host without it the command runs inside the observer image instead, which
# observes a Linux build of it: macOS has no tracer that runs without root
# (fs_usage) or without System Integrity Protection disabled (dtruss), and
# README.md records that attempt and its failure rather than hiding it.
#
# --server joins the observed process to that server's network namespace, so
# 127.0.0.1:3306 inside the container is the server and the fixture certificate,
# which names 127.0.0.1, matches. tls/ is mounted at /tls.
# --------------------------------------------------------------------------

OBSERVER_IMAGE=tpl-mariadb-observer
TRACE_SYSCALLS='%file,%network'

# Quote a command for the /bin/sh inside the observer. Joining the arguments
# with spaces loses the quoting the caller wrote, and `-e 'SELECT 1'` then
# arrives as two arguments.
shq() {
    local a out=""
    for a in "$@"; do out="$out '$(printf '%s' "$a" | sed "s/'/'\\\\''/g")'"; done
    printf '%s' "${out# }"
}

opens() {
    local server="" backend=auto
    while [ $# -gt 0 ]; do
        case "$1" in
            --server)  shift; server="$1" ;;
            --backend) shift; backend="$1" ;;
            --) shift; break ;;
            *) die "unknown option for opens: $1 (did you forget --?)" ;;
        esac
        shift
    done
    [ $# -gt 0 ] || die "opens needs a command after --"

    if [ "$backend" = auto ]; then
        if command -v strace >/dev/null 2>&1; then backend=strace; else backend=container; fi
    fi

    case "$backend" in
        strace)
            printf 'observe.sh: tracing on the host with strace\n' >&2
            # The trace goes to a file and the traced command's stdout is
            # discarded, as the container backend does: with `-o /dev/stdout`
            # the command's own output is interleaved with the trace, and a
            # path the command merely prints reads as a path it opened.
            local trace status=0
            trace="$(mktemp)"
            strace -f -e "trace=$TRACE_SYSCALLS" -o "$trace" -- "$@" >/dev/null || status=$?
            cat "$trace"
            rm -f "$trace"
            return "$status"
            ;;
        container)
            docker image inspect "$OBSERVER_IMAGE" >/dev/null 2>&1 \
                || docker build -q -f observer.Dockerfile -t "$OBSERVER_IMAGE" . >/dev/null
            local netns=()
            if [ -n "$server" ]; then
                netns=(--network "container:$(tpl_mariadb_field "$server" 4)")
            fi
            printf 'observe.sh: tracing inside %s%s\n' "$OBSERVER_IMAGE" \
                "${server:+ (network namespace of $server)}" >&2
            docker run --rm \
                --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
                "${netns[@]+"${netns[@]}"}" \
                -v "$PWD/tls:/tls:ro" \
                "$OBSERVER_IMAGE" \
                "strace -f -e trace=$TRACE_SYSCALLS -o /trace -- $(shq "$@") >/dev/null 2>&1; cat /trace"
            ;;
        *) die "unknown backend: $backend (strace, container)" ;;
    esac
}

# --------------------------------------------------------------------------

[ $# -ge 1 ] || usage
subcommand="$1"; shift

case "$subcommand" in
    statements)
        [ $# -ge 2 ] || die "statements needs an action and a server"
        action="$1"; server="$2"; shift 2
        tpl_mariadb_record "$server" >/dev/null || die "unknown server: $server"
        case "$action" in
            on)   statements_on   "$server" ;;
            off)  statements_off  "$server" ;;
            dump) statements_dump "$server" "$@" ;;
            *) die "unknown action: $action (on, off, dump)" ;;
        esac
        ;;
    connections)
        [ $# -ge 1 ] || die "connections needs a server"
        server="$1"; shift
        tpl_mariadb_record "$server" >/dev/null || die "unknown server: $server"
        connections "$server" "$@"
        ;;
    opens) opens "$@" ;;
    build) build "$@" ;;
    *) usage ;;
esac
