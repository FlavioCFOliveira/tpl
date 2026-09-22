#!/usr/bin/env bash
#
# Load the benchmark workloads of NFR-PERF-001 into a running fixture, and count
# what was loaded.
#
# `seed-bench.sql` is not in the image. The Dockerfile copies `setup.sql` and
# `seed.sql` into /docker-entrypoint-initdb.d, so every container carries
# `freight` from the moment it starts; BR-PERF-002 keeps the benchmark workload
# out of that path, because a correctness run must not pay for 200 tables. This
# script is how the workload gets in, and it is the only way it should: a
# hand-written `docker exec` skips the counts below.
#
# Usage:
#     ./seed-bench.sh                # load into all five servers, then verify
#     ./seed-bench.sh 11.8 notls     # only the named servers
#     ./seed-bench.sh --verify       # count what is there; load nothing
#     ./seed-bench.sh --drop         # drop both schemas
#
# It requires the fixture to be up: `./up.sh` first, `./down.sh` after. Loading
# is idempotent — `seed-bench.sql` drops each schema before creating it — so
# running it twice leaves the same two schemas.
#
# Exit code:
#     0   every requested server holds both workloads, at every stated count
#     1   a server did not, and the line naming the count says which
#     2   the invocation is wrong, or the fixture is not up

set -euo pipefail

# The header above is this script's own usage text, and `--help` reads it back
# out of the file. The name is taken before the `cd`, because `$0` is the path
# the caller wrote and stops resolving the moment the working directory changes.
SELF="$(basename "$0")"

cd "$(dirname "$0")"
. ./series.env

SQL_FILE=seed-bench.sql

log()  { printf '%s\n' "$*" >&2; }
die()  { printf 'seed-bench.sh: %s\n' "$*" >&2; exit 2; }

MODE=load
ARGS=()
for a in "$@"; do
    case "$a" in
        --verify) MODE=verify ;;
        --drop)   MODE=drop ;;
        -h|--help)
            awk 'NR >= 3 { if ($0 !~ /^#/) exit; sub(/^# ?/, ""); print }' "$SELF"
            exit 2
            ;;
        -*) die "unknown option: $a" ;;
        *)  ARGS+=("$a") ;;
    esac
done

[ -f "$SQL_FILE" ] || die "$SQL_FILE is missing"

wanted() {
    [ "${#ARGS[@]}" -eq 0 ] && return 0
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

# The gate, asked of the harness rather than assumed: a load against a server
# that is not up fails halfway and leaves a partial schema behind.
if ! ./status.sh --quiet ${ARGS[@]+"${ARGS[@]}"}; then
    die "the fixture is not up for the requested servers; run ./up.sh first"
fi

# The twelve counts, in one statement per schema.
#
# Each is counted on a rule this script states once and README.md documents:
#
#   tables            TABLE_TYPE = 'BASE TABLE'; a view is not a table
#   views             TABLE_TYPE = 'VIEW'
#   columns           columns of the base tables, generated ones included
#   generated         EXTRA names the storage, 'STORED' or 'VIRTUAL GENERATED'
#   indexes           distinct (TABLE_NAME, INDEX_NAME); PRIMARY is an index
#   foreign_keys      one row per REFERENTIAL_CONSTRAINTS constraint
#   triggers          one row per trigger
#   routines          procedures and functions together
#   commented_tables  base tables whose TABLE_COMMENT is not empty
counts_of() {
    local server="$1" schema="$2"
    tpl_mariadb_sql "$server" -N -B -e "
        SELECT 'tables', COUNT(*) FROM information_schema.TABLES
          WHERE TABLE_SCHEMA = '$schema' AND TABLE_TYPE = 'BASE TABLE'
        UNION ALL
        SELECT 'columns', COUNT(*) FROM information_schema.COLUMNS c
          JOIN information_schema.TABLES t
            ON t.TABLE_SCHEMA = c.TABLE_SCHEMA AND t.TABLE_NAME = c.TABLE_NAME
          WHERE c.TABLE_SCHEMA = '$schema' AND t.TABLE_TYPE = 'BASE TABLE'
        UNION ALL
        SELECT 'indexes', COUNT(*) FROM (
          SELECT DISTINCT TABLE_NAME, INDEX_NAME FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = '$schema') AS distinct_indexes
        UNION ALL
        SELECT 'foreign_keys', COUNT(*) FROM information_schema.REFERENTIAL_CONSTRAINTS
          WHERE CONSTRAINT_SCHEMA = '$schema'
        UNION ALL
        SELECT 'generated', COUNT(*) FROM information_schema.COLUMNS c
          JOIN information_schema.TABLES t
            ON t.TABLE_SCHEMA = c.TABLE_SCHEMA AND t.TABLE_NAME = c.TABLE_NAME
          WHERE c.TABLE_SCHEMA = '$schema' AND t.TABLE_TYPE = 'BASE TABLE'
            AND c.EXTRA LIKE '%GENERATED%'
        UNION ALL
        SELECT 'triggers', COUNT(*) FROM information_schema.TRIGGERS
          WHERE TRIGGER_SCHEMA = '$schema'
        UNION ALL
        SELECT 'views', COUNT(*) FROM information_schema.TABLES
          WHERE TABLE_SCHEMA = '$schema' AND TABLE_TYPE = 'VIEW'
        UNION ALL
        SELECT 'routines', COUNT(*) FROM information_schema.ROUTINES
          WHERE ROUTINE_SCHEMA = '$schema'
        UNION ALL
        SELECT 'commented_tables', COUNT(*) FROM information_schema.TABLES
          WHERE TABLE_SCHEMA = '$schema' AND TABLE_TYPE = 'BASE TABLE'
            AND TABLE_COMMENT <> ''"
}

failures=0

# Compare one schema's counts with the figures the workload states. Every
# expected figure is checked, and a schema that is right in eight places and
# wrong in one fails on the ninth.
verify_schema() {
    local server="$1" schema="$2" workload="$3"; shift 3
    local -a expected=("$@")
    local measured name value want index=0 wrong=0 line

    measured="$(counts_of "$server" "$schema")"
    while IFS=$'\t' read -r name value; do
        [ -n "$name" ] || continue
        want="${expected[$index]}"
        index=$((index + 1))
        [ "$name" = "${want%%=*}" ] || die "count order changed: got $name, expected ${want%%=*}"
        if [ "$value" != "${want#*=}" ]; then
            log "  MISMATCH $schema ($workload): $name = $value, expected ${want#*=}"
            wrong=$((wrong + 1))
        fi
    done <<< "$measured"

    [ "$index" -eq "${#expected[@]}" ] \
        || die "counted $index quantities of $schema, expected ${#expected[@]}"

    line="$(printf '%s\n' "$measured" | awk -F'\t' 'NF { printf "%s%s=%s", separator, $1, $2; separator = " " }')"
    if [ "$wrong" -eq 0 ]; then
        log "  ok      $schema ($workload): $line"
    else
        failures=$((failures + 1))
    fi
}

load_one() {
    local server="$1"
    log "  load    $SQL_FILE into $(tpl_mariadb_field "$server" 4)"
    tpl_mariadb_sql "$server" < "$SQL_FILE"
}

drop_one() {
    local server="$1"
    tpl_mariadb_sql "$server" -e "
        DROP DATABASE IF EXISTS $TPL_MARIADB_BENCH_LARGE_SCHEMA;
        DROP DATABASE IF EXISTS $TPL_MARIADB_BENCH_SMALL_SCHEMA;"
    log "  dropped $TPL_MARIADB_BENCH_LARGE_SCHEMA and $TPL_MARIADB_BENCH_SMALL_SCHEMA on $server"
}

verify_one() {
    local server="$1"
    verify_schema "$server" "$TPL_MARIADB_BENCH_LARGE_SCHEMA" WL-001 \
        "tables=$TPL_MARIADB_WL001_TABLES" \
        "columns=$TPL_MARIADB_WL001_COLUMNS" \
        "indexes=$TPL_MARIADB_WL001_INDEXES" \
        "foreign_keys=$TPL_MARIADB_WL001_FOREIGN_KEYS" \
        "generated=$TPL_MARIADB_WL001_GENERATED" \
        "triggers=$TPL_MARIADB_WL001_TRIGGERS" \
        "views=$TPL_MARIADB_WL001_VIEWS" \
        "routines=$TPL_MARIADB_WL001_ROUTINES" \
        "commented_tables=$TPL_MARIADB_WL001_COMMENTED"
    verify_schema "$server" "$TPL_MARIADB_BENCH_SMALL_SCHEMA" WL-003 \
        "tables=$TPL_MARIADB_WL003_TABLES" \
        "columns=$TPL_MARIADB_WL003_COLUMNS" \
        "indexes=$TPL_MARIADB_WL003_INDEXES" \
        "foreign_keys=0" \
        "generated=0" \
        "triggers=0" \
        "views=0" \
        "routines=0" \
        "commented_tables=1"
}

# The inventory is read on file descriptor 9, not on stdin, on the same terms
# up.sh, down.sh and status.sh read it: `docker exec -i` reads stdin, and
# `load_one` below feeds a whole SQL file to one. A loop fed from stdin would
# hand the records still to be read to the first server's client, and would end
# after that server — silently, and with exit 0. Descriptor 9 and not 3:
# `tpl_mariadb_handshake` opens the TCP port on 3.
while read -r name series image container port tls <&9; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    log "$name"
    case "$MODE" in
        load)   load_one "$name"; verify_one "$name" ;;
        verify) verify_one "$name" ;;
        drop)   drop_one "$name" ;;
    esac
done 9<<< "$TPL_MARIADB_SERVERS"

if [ "$MODE" = drop ]; then
    log ""
    log "both benchmark schemas dropped"
    exit 0
fi

log ""
if [ "$failures" -ne 0 ]; then
    log "seed-bench.sh: $failures schema(s) did not match the workload they realise"
    exit 1
fi
log "every requested server holds WL-001 and WL-003 at the counts the specification states"
