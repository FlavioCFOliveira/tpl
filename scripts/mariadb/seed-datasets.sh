#!/usr/bin/env bash
#
# Load the two published datasets of FR-EX-006 into a named running server, and
# count what arrived.
#
# `sakila` and `world` are not in the image. The Dockerfile copies `setup.sql`
# and `seed.sql` into /docker-entrypoint-initdb.d, so every container carries
# `freight` from the moment it starts; these two stay out of that path on the
# grounds that keep the benchmark workload out of it, and that `seed-bench.sh`
# states: only the worked examples read them, and a correctness run must not pay
# for 20 schemas and 113 catalogue objects it never touches. This script is how
# they get in, and it is the only way they should: a hand-written `docker exec`
# skips the counts below.
#
# The SQL is vendored verbatim under `datasets/`. Each dataset directory holds
# its NOTICE.md, recording the source URL, the date the file was taken and the
# checksum it was taken at, and its LICENSE where upstream ships one.
#
# Usage:
#     ./seed-datasets.sh 12.3            # load into 12.3, then verify
#     ./seed-datasets.sh 12.3 11.8       # load into both
#     ./seed-datasets.sh --verify 12.3   # count what is there; load nothing
#     ./seed-datasets.sh --drop 12.3     # drop both schemas
#
# The server is required, and that is the one place this script departs from
# `seed-bench.sh`, which loads into all five when told nothing. FR-EX-006 reads
# one server of one series, so loading five is work no requirement asks for;
# naming the server also makes a typo an error instead of a silent no-op.
#
# It requires that server to be up: `./up.sh` first, `./down.sh` after. Loading
# is idempotent — each dataset drops its schema before creating it — so running
# it twice leaves the same two schemas.
#
# Exit code:
#     0   every named server holds both datasets, at every stated count
#     1   a server did not, and the line naming the count says which
#     2   the invocation is wrong, or the fixture is not up

set -euo pipefail

# The header above is this script's own usage text, and `--help` reads it back
# out of the file. The name is taken before the `cd`, because `$0` is the path
# the caller wrote and stops resolving the moment the working directory changes.
SELF="$(basename "$0")"

cd "$(dirname "$0")"
. ./series.env

DATASETS_DIR=datasets

# One record per dataset: <schema> <directory> <sql file>...
#
# The order of the files is the order they load in, and for sakila it is not
# interchangeable: the schema file creates the tables the data file fills, and
# the triggers that fill `film_text` while it does.
SAKILA_FILES="$DATASETS_DIR/sakila/sakila-schema.sql $DATASETS_DIR/sakila/sakila-data.sql"
WORLD_FILES="$DATASETS_DIR/world/world.sql"

log() { printf '%s\n' "$*" >&2; }
die() { printf 'seed-datasets.sh: %s\n' "$*" >&2; exit 2; }

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

[ "${#ARGS[@]}" -gt 0 ] || die "name the server to work on, for instance: $SELF 12.3"

for a in "${ARGS[@]}"; do
    tpl_mariadb_record "$a" >/dev/null \
        || die "unknown server: $a (the names are in series.env)"
done

for f in $SAKILA_FILES $WORLD_FILES; do
    [ -f "$f" ] || die "$f is missing"
done

# The gate, asked of the harness rather than assumed: a load against a server
# that is not up fails halfway and leaves a partial schema behind.
if ! ./status.sh --quiet "${ARGS[@]}"; then
    die "the fixture is not up for the named servers; run ./up.sh first"
fi

# The nine catalogue counts, in one statement per schema. `verify_reader`
# below adds the tenth, which no root session can answer.
#
# The counting rule for each is `seed-bench.sh`'s, unchanged and stated there
# beside the statement that counts it, so that both loaders answer the same
# question the same way and README.md documents one rule rather than two.
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

# Compare one schema's counts with the figures series.env states. Every expected
# figure is checked, and a schema that is right in eight places and wrong in one
# fails on the ninth.
verify_schema() {
    local server="$1" schema="$2"; shift 2
    local -a expected=("$@")
    local measured name value want index=0 wrong=0 line

    measured="$(counts_of "$server" "$schema")"
    while IFS=$'\t' read -r name value; do
        [ -n "$name" ] || continue
        want="${expected[$index]}"
        index=$((index + 1))
        [ "$name" = "${want%%=*}" ] || die "count order changed: got $name, expected ${want%%=*}"
        if [ "$value" != "${want#*=}" ]; then
            log "  MISMATCH $schema: $name = $value, expected ${want#*=}"
            wrong=$((wrong + 1))
        fi
    done <<< "$measured"

    [ "$index" -eq "${#expected[@]}" ] \
        || die "counted $index quantities of $schema, expected ${#expected[@]}"

    line="$(printf '%s\n' "$measured" | awk -F'\t' 'NF { printf "%s%s=%s", separator, $1, $2; separator = " " }')"
    if [ "$wrong" -eq 0 ]; then
        log "  ok      $schema: $line"
    else
        failures=$((failures + 1))
    fi
}

# One dataset into one server. Each file is fed to its own client: the sakila
# schema file ends with a `DELIMITER ;` that only matters within one client
# session, and a single stream would also lose the per-file exit code below.
load_dataset() {
    local server="$1" schema="$2"; shift 2
    local file
    for file in "$@"; do
        log "  load    $file into $schema on $(tpl_mariadb_field "$server" 4)"
        tpl_mariadb_sql "$server" < "$file"
    done
}

# Both dataset files create their schema from nothing, and a dropped schema
# takes its grants with it, so the reader has to be re-granted on every load.
#
# This is not decoration. `setup.sql` grants the reader SELECT and EXECUTE on
# `freight`, and FR-EX-007 obliges all four worked examples to read the three
# schemas of FR-EX-006 through entries that "differ in nothing a read can
# observe". Without the same grant on these two, INFORMATION_SCHEMA shows the
# reader nothing of them and an example would render an empty data layer, exit
# 0, and say nothing — which is the FR-PRIV-001 hazard UC-013 names.
grant_reader() {
    local server="$1" schema="$2"
    tpl_mariadb_sql "$server" -e \
        "GRANT SELECT, EXECUTE ON \`$schema\`.* TO '$TPL_MARIADB_READER_USER'@'%';"
    log "  grant   SELECT, EXECUTE on $schema to $TPL_MARIADB_READER_USER"
}

load_one() {
    local server="$1"
    load_dataset "$server" "$TPL_MARIADB_SAKILA_SCHEMA" $SAKILA_FILES
    grant_reader "$server" "$TPL_MARIADB_SAKILA_SCHEMA"
    load_dataset "$server" "$TPL_MARIADB_WORLD_SCHEMA" $WORLD_FILES
    grant_reader "$server" "$TPL_MARIADB_WORLD_SCHEMA"
}

drop_one() {
    local server="$1"
    tpl_mariadb_sql "$server" -e "
        DROP DATABASE IF EXISTS $TPL_MARIADB_SAKILA_SCHEMA;
        DROP DATABASE IF EXISTS $TPL_MARIADB_WORLD_SCHEMA;"
    log "  dropped $TPL_MARIADB_SAKILA_SCHEMA and $TPL_MARIADB_WORLD_SCHEMA on $server"
}

# What the unprivileged reader sees, asked as that reader.
#
# INFORMATION_SCHEMA shows a user only the objects it holds some privilege on,
# so this is the one count that cannot be taken from the root session above: a
# schema fully loaded and never granted passes all nine checks and is still
# unreadable by the examples. The figure compared is the schema's own
# `tables + views`, counted here as root and there as the reader.
verify_reader() {
    local server="$1" schema="$2" expected="$3"
    local seen
    seen="$(tpl_mariadb_sql_as "$server" \
        "$TPL_MARIADB_READER_USER" "$TPL_MARIADB_READER_PASSWORD" -N -B -e \
        "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA = '$schema'")"
    if [ "$seen" = "$expected" ]; then
        log "  ok      $schema: $TPL_MARIADB_READER_USER sees $seen of $expected objects"
    else
        log "  MISMATCH $schema: $TPL_MARIADB_READER_USER sees $seen objects, expected $expected"
        failures=$((failures + 1))
    fi
}

verify_one() {
    local server="$1"
    verify_schema "$server" "$TPL_MARIADB_SAKILA_SCHEMA" \
        "tables=$TPL_MARIADB_SAKILA_TABLES" \
        "columns=$TPL_MARIADB_SAKILA_COLUMNS" \
        "indexes=$TPL_MARIADB_SAKILA_INDEXES" \
        "foreign_keys=$TPL_MARIADB_SAKILA_FOREIGN_KEYS" \
        "generated=$TPL_MARIADB_SAKILA_GENERATED" \
        "triggers=$TPL_MARIADB_SAKILA_TRIGGERS" \
        "views=$TPL_MARIADB_SAKILA_VIEWS" \
        "routines=$TPL_MARIADB_SAKILA_ROUTINES" \
        "commented_tables=$TPL_MARIADB_SAKILA_COMMENTED"
    verify_schema "$server" "$TPL_MARIADB_WORLD_SCHEMA" \
        "tables=$TPL_MARIADB_WORLD_TABLES" \
        "columns=$TPL_MARIADB_WORLD_COLUMNS" \
        "indexes=$TPL_MARIADB_WORLD_INDEXES" \
        "foreign_keys=$TPL_MARIADB_WORLD_FOREIGN_KEYS" \
        "generated=$TPL_MARIADB_WORLD_GENERATED" \
        "triggers=$TPL_MARIADB_WORLD_TRIGGERS" \
        "views=$TPL_MARIADB_WORLD_VIEWS" \
        "routines=$TPL_MARIADB_WORLD_ROUTINES" \
        "commented_tables=$TPL_MARIADB_WORLD_COMMENTED"
    verify_reader "$server" "$TPL_MARIADB_SAKILA_SCHEMA" \
        "$((TPL_MARIADB_SAKILA_TABLES + TPL_MARIADB_SAKILA_VIEWS))"
    verify_reader "$server" "$TPL_MARIADB_WORLD_SCHEMA" \
        "$((TPL_MARIADB_WORLD_TABLES + TPL_MARIADB_WORLD_VIEWS))"
}

wanted() {
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

# The inventory is read on file descriptor 9, not on stdin, on the same terms
# up.sh, down.sh, status.sh and seed-bench.sh read it: `docker exec -i` reads
# stdin, and `load_dataset` above feeds a whole SQL file to one. A loop fed from
# stdin would hand the records still to be read to the first server's client,
# and would end after that server — silently, and with exit 0. Descriptor 9 and
# not 3: `tpl_mariadb_handshake` opens the TCP port on 3.
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
    log "both datasets dropped"
    exit 0
fi

log ""
if [ "$failures" -ne 0 ]; then
    log "seed-datasets.sh: $failures schema(s) did not match the dataset they carry"
    exit 1
fi
log "every named server holds sakila and world at the counts series.env states"
