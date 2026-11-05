#!/usr/bin/env bash
#
# The instruments. NFR-PERF-007 forbids verifying a requirement of form by
# reading the source, and BR-SRV-003 says why; FR-MYSQL-042 and FR-MYSQL-043
# hold a `mysql` entry to the same observations, and FR-MYSQL-044 obliges this
# fixture to instrument the two of them that a server provides:
#
#   statements   what the server received      the server's general log
#   connections  what the server accepted      the server's status counters
#
# A third subcommand, `conditions`, verifies no requirement. It reads the
# conditions a reading depends on — the TLS material the server serves, its
# lower_case_table_names, and its show_gipk_in_create_table_and_information_schema
# — which FR-MYSQL-044 requires to be recorded with every reading taken against
# the server.
#
# Usage:
#     ./observe.sh statements on|off|dump <server> [filters]
#     ./observe.sh connections <server> [--value]
#     ./observe.sh conditions [<server>]
#
# <server> is a name from series.env: 8.4, 9.7, notls-8.4, notls-9.7 or 8.0.
#
# Every subcommand is a command you can also type by hand; README.md records
# each of them with the output it actually produced.

set -euo pipefail

# The header above is this script's own usage text, and `usage` reads it back
# out of the file. The name is taken before the `cd`, because `$0` is the path
# the caller wrote and stops resolving the moment the working directory changes.
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
# under test. The log goes to the table mysql.general_log, so that it can be
# queried rather than parsed; its event_time is timestamp(6) on every series of
# the fixture, so the dump orders by microsecond and prints the order the
# server received.
#
# The observer's own connection is a connection like any other and is logged
# too; `dump` therefore drops connections made over the Unix socket by default,
# which is how this script and up.sh reach the server and is not how anything
# under test reaches it. The log spells such a connection "root[root] @
# localhost []" on a Query row and "[root] @ localhost []" on its Connect row,
# and one pattern matches both. --all keeps them.
#
# --catalogue admits two command types, because a client may send one
# statement in either of two ways, and the general log spells them differently:
#
#   Query                     the text protocol: one row per statement issued
#   Prepare, then Execute     the binary protocol: one Prepare per distinct
#                             statement text, and one Execute per issue of it
#
# The filter admits Query and Execute, the two rows that mean *a statement was
# issued*, and excludes Prepare, which means *a statement text was registered*:
# counting Prepare counts texts rather than issues, and admitting both Prepare
# and Execute would count every prepared statement twice.
# --------------------------------------------------------------------------

statements_on() {
    tpl_mysql_sql "$1" -e "
        SET GLOBAL general_log = OFF;
        SET GLOBAL log_output  = 'TABLE';
        TRUNCATE TABLE mysql.general_log;
        SET GLOBAL general_log = ON;"
}

statements_off() { tpl_mysql_sql "$1" -e "SET GLOBAL general_log = OFF;"; }

statements_dump() {
    local server="$1"; shift
    local where="user_host NOT LIKE '% @ localhost []'"
    local count=no

    while [ $# -gt 0 ]; do
        case "$1" in
            --all)       where="1 = 1" ;;
            # The general log spells the user two ways: a Query row carries
            # "u[u] @  [host]" and the Connect row that opened the same
            # connection carries "[u] @  [host]". Matching on the bracketed
            # form is the one filter that keeps both.
            --user)      shift; [ $# -gt 0 ] || die "--user needs a user"
                         where="$where AND user_host LIKE '%[$1]%'" ;;
            --kind)      shift; [ $# -gt 0 ] || die "--kind needs a command type"
                         where="$where AND command_type = '$1'" ;;
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
        tpl_mysql_sql "$server" -N -B -e \
            "SELECT COUNT(*) FROM mysql.general_log WHERE $where"
    else
        tpl_mysql_sql "$server" --default-character-set=utf8mb4 -B -e "
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
#
# The value is read from performance_schema.global_status: MySQL has no
# information_schema.GLOBAL_STATUS, which is where the MariaDB fixture reads it.
# --------------------------------------------------------------------------

connections() {
    local server="$1"; shift
    if [ "${1:-}" = --value ]; then
        tpl_mysql_sql "$server" -N -B -e \
            "SELECT VARIABLE_VALUE FROM performance_schema.global_status
             WHERE VARIABLE_NAME = 'Connections'"
    else
        tpl_mysql_sql "$server" -B -e "
            SHOW GLOBAL STATUS WHERE Variable_name IN
              ('Connections', 'Aborted_connects', 'Threads_connected', 'Max_used_connections')"
    fi
}

# --------------------------------------------------------------------------
# conditions — what a reading taken against a server depends on, read from the
# server rather than from the configuration that asked for it.
#
#   version                        the build, as the server reports it
#   tls_enabled                    whether the main interface offers TLS
#   tls_cert                       the certificate file it serves
#   tls_cert_not_before/_after     that certificate's validity, which names the
#                                  generation of tls/ the image carries
#   lower_case_table_names         how the server stores and compares names
#   show_gipk_in_create_table_and_information_schema
#   sql_generate_invisible_primary_key
#                                  the two settings that decide whether a
#                                  generated invisible primary key is created
#                                  and whether the catalogue shows it
#
# Every name is asked for and counted: a variable asked for by a name the
# server does not have comes back as no row rather than as an error, so a name
# misremembered by one character would read as a server that lacks it. A
# missing row is a failure here, not a blank.
# --------------------------------------------------------------------------

CONDITIONS_SQL="
SELECT 'version', VERSION()
UNION ALL
SELECT CONCAT('tls_', CASE PROPERTY
                        WHEN 'Enabled'               THEN 'enabled'
                        WHEN 'Current_tls_cert'      THEN 'cert'
                        WHEN 'Ssl_server_not_before' THEN 'cert_not_before'
                        WHEN 'Ssl_server_not_after'  THEN 'cert_not_after'
                      END),
       VALUE
  FROM performance_schema.tls_channel_status
 WHERE CHANNEL = 'mysql_main'
   AND PROPERTY IN ('Enabled', 'Current_tls_cert', 'Ssl_server_not_before', 'Ssl_server_not_after')
UNION ALL
SELECT VARIABLE_NAME, VARIABLE_VALUE
  FROM performance_schema.global_variables
 WHERE VARIABLE_NAME IN ('lower_case_table_names',
                         'show_gipk_in_create_table_and_information_schema',
                         'sql_generate_invisible_primary_key')"

CONDITIONS_EXPECTED=8

conditions_one() {
    local rows count
    rows="$(tpl_mysql_sql "$1" -N -B -e "$CONDITIONS_SQL")"
    count="$(printf '%s\n' "$rows" | awk 'NF { n++ } END { print n + 0 }')"
    [ "$count" -eq "$CONDITIONS_EXPECTED" ] \
        || die "$1 answered $count of the $CONDITIONS_EXPECTED conditions; a name the server does not have reads as no row"
    printf '%s\n' "$rows" | awk -F'\t' -v s="$1" '{ printf "%-10s %-49s %s\n", s, $1, $2 }'
}

conditions() {
    [ $# -le 1 ] || die "conditions takes at most one server"
    [ $# -eq 0 ] || tpl_mysql_record "$1" >/dev/null || die "unknown server: $1"
    printf '%-10s %-49s %s\n' SERVER CONDITION VALUE
    if [ $# -eq 1 ]; then
        conditions_one "$1"
    else
        tpl_mysql_each conditions_one
    fi
}

# --------------------------------------------------------------------------

[ $# -ge 1 ] || usage
subcommand="$1"; shift

case "$subcommand" in
    statements)
        [ $# -ge 2 ] || die "statements needs an action and a server"
        action="$1"; server="$2"; shift 2
        tpl_mysql_record "$server" >/dev/null || die "unknown server: $server"
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
        tpl_mysql_record "$server" >/dev/null || die "unknown server: $server"
        connections "$server" "$@"
        ;;
    conditions) conditions "$@" ;;
    *) usage ;;
esac
