#!/usr/bin/env bash
#
# Start the whole fixture: one server of each series of FR-MYSQL-003 presenting
# the certificate of tls/, a server offering no TLS at each of those series,
# and the server of a series outside FR-MYSQL-003 that FR-MYSQL-009's refusal is
# verified against.
#
# The script is idempotent. It builds an image only when it is missing, starts a
# container only when it is not already running, and it does not return until
# every server has answered on its published port and been confirmed to have
# initialised cleanly — a failure inside /docker-entrypoint-initdb.d leaves the
# schema incomplete, so container status is not the check.
#
# Usage:
#     ./up.sh                  # all five
#     ./up.sh 8.4 notls-8.4    # only the named servers
#
# It ends by asking `status.sh` about the same servers it was asked to start,
# and exits with that answer: 0 when every requested server answered.
#
# down.sh is the other half. Leave no container running after a validation run.

set -euo pipefail

cd "$(dirname "$0")"
. ./series.env

TIMEOUT_SECONDS="${TPL_MYSQL_TIMEOUT:-180}"

log()  { printf '%s\n' "$*" >&2; }
fail() { printf 'up.sh: %s\n' "$*" >&2; exit 1; }

# A container that cannot be made healthy is removed, not left behind: the
# fixture's rule is that a run leaves nothing running, and a failed run is still
# a run. Its last log lines are printed first, because removing the container
# removes the log with it.
fail_container() {
    local container="$1"; shift
    printf 'up.sh: %s\n' "$*" >&2
    printf -- '--- last 30 log lines of %s ---\n' "$container" >&2
    docker logs --tail 30 "$container" >&2 2>&1 || true
    printf -- '--- removing %s ---\n' "$container" >&2
    docker rm -f -v "$container" >/dev/null 2>&1 || true
    exit 1
}

build_if_missing() {
    local series="$1" image="$2" base
    if docker image inspect "$image" >/dev/null 2>&1; then
        log "  image   $image already built"
        return
    fi
    base="$(tpl_mysql_base_image "$series")" || fail "series.env pins no image for series $series"
    log "  image   $image building from $base"
    # The build's progress is kept out of the way and printed only when the
    # build fails, which is the one time it is worth reading.
    local output
    if ! output="$(docker build \
            --build-arg "MYSQL_IMAGE=$base" \
            --build-arg "MYSQL_SERIES=$series" \
            -t "$image" . 2>&1)"; then
        printf '%s\n' "$output" >&2
        fail "building $image from $base failed"
    fi
}

start_if_absent() {
    local image="$3" container="$4" port="$5" tls="$6"
    local state
    state="$(docker inspect -f '{{.State.Status}}' "$container" 2>/dev/null || true)"
    case "$state" in
        running) log "  server  $container already running on :$port"; return ;;
        exited|created) fail "$container exists but is $state; run ./down.sh first" ;;
    esac

    # `--tls-version=` sets tls_version to the empty string, which is how the
    # MySQL manual says to start a server "with encrypted connections disabled
    # on the main interface". It wins over the certificate 90-tls.cnf
    # configures, so a no-TLS server is the same image with one argument more.
    local extra=""
    [ "$tls" = no ] && extra="--tls-version="

    # The data directory is a tmpfs, not the anonymous volume the image
    # declares. Nothing in it has to outlive the container: the entrypoint
    # builds it from setup.sql and seed.sql on every start. In memory it leaves
    # no volume behind to be mistaken for a fresh schema, and it costs the
    # Docker host's disk nothing, which matters at five servers of about 215 MB
    # each. 1 GB bounds it at four times what a loaded server holds.
    docker run -d \
        --name "$container" \
        -e "MYSQL_ROOT_PASSWORD=$TPL_MYSQL_ROOT_PASSWORD" \
        --tmpfs "/var/lib/mysql:rw,size=1g" \
        -p "$port:3306" \
        "$image" $extra >/dev/null
    log "  server  $container started on :$port${extra:+ ($extra)}"
}

# Readiness is the published port answering, never a `docker exec` client.
# During initialisation the official entrypoint runs a temporary server with
# --skip-networking: it answers over the Unix socket while the init scripts are
# still running, and it then goes away to be replaced by the real one. A check
# made from inside the container therefore returns true in the middle of the
# init and hands the caller a server that is about to restart. The published
# port is up only when the server under test is.
wait_ready() {
    local container="$4" port="$5" deadline version
    deadline=$(( $(date +%s) + TIMEOUT_SECONDS ))
    while :; do
        version="$(tpl_mysql_handshake "$port" 2>/dev/null || true)"
        if [ -n "$version" ]; then
            log "  listen  $container answers on :$port as $version"
            return 0
        fi
        [ "$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null)" = true ] \
            || fail_container "$container" "$container stopped while starting"
        [ "$(date +%s)" -lt "$deadline" ] || \
            fail_container "$container" "$container did not answer on :$port after ${TIMEOUT_SECONDS}s"
        sleep 1
    done
}

# What a clean initialisation leaves behind, counted from the catalogue. Every
# figure is compared with the one series.env states, and the three schemas are
# counted as present first: an absent schema holds no object either, so a count
# of zero in freight_empty proves nothing until the schema is known to exist.
CATALOGUE_COUNTS="
SELECT
  (SELECT COUNT(*) FROM information_schema.SCHEMATA
     WHERE SCHEMA_NAME IN ('$TPL_MYSQL_SCHEMA', '$TPL_MYSQL_SECOND_SCHEMA', '$TPL_MYSQL_EMPTY_SCHEMA')),
  (SELECT COUNT(*) FROM information_schema.TABLES
     WHERE TABLE_SCHEMA = '$TPL_MYSQL_SCHEMA' AND TABLE_TYPE = 'BASE TABLE'),
  (SELECT COUNT(*) FROM information_schema.TABLES
     WHERE TABLE_SCHEMA = '$TPL_MYSQL_SCHEMA' AND TABLE_TYPE = 'VIEW'),
  (SELECT COUNT(*) FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = '$TPL_MYSQL_SCHEMA'),
  (SELECT COUNT(*) FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = '$TPL_MYSQL_SCHEMA'),
  (SELECT COUNT(*) FROM information_schema.EVENTS   WHERE EVENT_SCHEMA   = '$TPL_MYSQL_SCHEMA'),
  (SELECT COUNT(*) FROM information_schema.TABLES   WHERE TABLE_SCHEMA   = '$TPL_MYSQL_SECOND_SCHEMA'),
  (SELECT COUNT(*) FROM information_schema.TABLES   WHERE TABLE_SCHEMA   = '$TPL_MYSQL_EMPTY_SCHEMA')
  + (SELECT COUNT(*) FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = '$TPL_MYSQL_EMPTY_SCHEMA')
  + (SELECT COUNT(*) FROM information_schema.EVENTS   WHERE EVENT_SCHEMA   = '$TPL_MYSQL_EMPTY_SCHEMA')"

# The TLS state the server is actually in, read from the server rather than from
# the configuration that asked for it: the main interface's channel status, and
# the certificate file it was told to serve.
TLS_STATE="
SELECT VALUE FROM performance_schema.tls_channel_status
  WHERE CHANNEL = 'mysql_main' AND PROPERTY = 'Enabled'"

# A failure inside /docker-entrypoint-initdb.d stops the container, which
# wait_ready catches, and is otherwise reported in the log; a schema can still
# come up short of what setup.sql declares if a statement is lost. Neither
# `docker ps` nor the port answering is the check: the log, the counts and the
# TLS state together are.
verify_clean() {
    local name="$1" container="$4" tls="$6"
    local errors counts expected tls_enabled cert
    errors="$(docker logs "$container" 2>&1 | awk '/\[ERROR\]/ { n++ } END { print n + 0 }')"
    [ "$errors" -eq 0 ] \
        || fail_container "$container" "$container logged $errors [ERROR] lines"

    counts="$(tpl_mysql_sql "$name" -N -B -e "$CATALOGUE_COUNTS" | tr '\t' ' ')"
    expected="3 $TPL_MYSQL_EXPECTED_TABLES $TPL_MYSQL_EXPECTED_VIEWS $TPL_MYSQL_EXPECTED_ROUTINES"
    expected="$expected $TPL_MYSQL_EXPECTED_TRIGGERS $TPL_MYSQL_EXPECTED_EVENTS"
    expected="$expected $TPL_MYSQL_EXPECTED_SECOND_TABLES $TPL_MYSQL_EXPECTED_EMPTY_OBJECTS"
    [ "$counts" = "$expected" ] \
        || fail_container "$container" \
             "$container catalogue counts are [$counts], expected [$expected] (schemas tables views routines triggers events second empty)"

    tls_enabled="$(tpl_mysql_sql "$name" -N -B -e "$TLS_STATE")"
    cert="$(tpl_mysql_sql "$name" -N -B -e "SELECT @@global.ssl_cert")"
    case "$tls" in
        yes) [ "$tls_enabled" = Yes ] && [ "$cert" = /etc/mysql/tls/server-cert.pem ] \
                 || fail_container "$container" \
                      "$container should serve the fixture certificate; TLS enabled=$tls_enabled, ssl_cert=$cert" ;;
        no)  [ "$tls_enabled" = No ] \
                 || fail_container "$container" "$container should offer no TLS; TLS enabled=$tls_enabled" ;;
    esac

    local IFS=' '
    set -- $counts
    log "  ready   $container: $TPL_MYSQL_SCHEMA tables=$2 views=$3 routines=$4 triggers=$5 events=$6;" \
        "$TPL_MYSQL_SECOND_SCHEMA tables=$7; $TPL_MYSQL_EMPTY_SCHEMA objects=$8;" \
        "TLS enabled=$tls_enabled; no [ERROR] in log"
}

wanted() {
    # Called as `wanted <name> "$@"`: with no selection on the command line
    # there is exactly one argument, and everything is wanted.
    [ $# -le 1 ] && return 0
    local name="$1"; shift
    for w in "$@"; do [ "$w" = "$name" ] && return 0; done
    return 1
}

main() {
    local name series image container port tls standing w
    # An unknown name is an error, not a filter that matches nothing: a typo
    # would otherwise start nothing and exit with the gate's answer for nothing.
    for w in "$@"; do
        tpl_mysql_record "$w" >/dev/null || fail "unknown server: $w (the names are in series.env)"
    done

    # The inventory is read on file descriptor 9, not on stdin: `docker exec -i`
    # reads stdin, and a loop that feeds itself from stdin hands the rest of its
    # own input to the first such command, which then stops the loop after one
    # server, silently. Descriptor 9 and not 3: `tpl_mysql_handshake` opens the
    # TCP port on 3.
    while read -r name series image container port tls standing <&9; do
        [ -n "$name" ] || continue
        wanted "$name" "$@" || continue
        log "$name"
        build_if_missing  "$series" "$image"
        start_if_absent   "$name" "$series" "$image" "$container" "$port" "$tls" "$standing"
        wait_ready        "$name" "$series" "$image" "$container" "$port" "$tls" "$standing"
        verify_clean      "$name" "$series" "$image" "$container" "$port" "$tls" "$standing"
    done 9<<< "$TPL_MYSQL_SERVERS"
    log ""
    # The closing verification covers what was asked for and nothing else: the
    # same selection is handed to the gate, so `./up.sh 8.4` reports 8.4 and
    # exits with 8.4's answer.
    ./status.sh "$@"
}

main "$@"
