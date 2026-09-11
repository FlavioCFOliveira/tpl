#!/usr/bin/env bash
#
# Start the whole fixture: the four series of FR-SRV-015, each presenting the
# certificate of tls/, plus the server beside them that offers none.
#
# The script is idempotent. It builds an image only when it is missing, starts a
# container only when it is not already running, and it does not return until
# every server has answered a query and been confirmed to have initialised
# cleanly — a failure inside /docker-entrypoint-initdb.d leaves the schema
# incomplete while `docker ps` still reports the container up, so container
# status is not the check.
#
# Usage:
#     ./up.sh                # all five
#     ./up.sh 10.11 notls    # only the named servers
#
# down.sh is the other half. Leave no container running after a validation run.

set -euo pipefail

cd "$(dirname "$0")"
. ./series.env

TIMEOUT_SECONDS="${TPL_MARIADB_TIMEOUT:-120}"

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
    local image="$2" series="$1"
    if docker image inspect "$image" >/dev/null 2>&1; then
        log "  image   $image already built"
    else
        log "  image   $image building from mariadb:$series"
        docker build --build-arg "MARIADB_SERIES=$series" -t "$image" . >/dev/null
    fi
}

start_if_absent() {
    local name="$1" image="$3" container="$4" port="$5" tls="$6"
    local state
    state="$(docker inspect -f '{{.State.Status}}' "$container" 2>/dev/null || true)"
    case "$state" in
        running) log "  server  $container already running on :$port"; return ;;
        exited|created) fail "$container exists but is $state; run ./down.sh first" ;;
    esac

    local extra=""
    [ "$tls" = no ] && extra="--skip-ssl"
    docker run -d \
        --name "$container" \
        -e "MARIADB_ROOT_PASSWORD=$TPL_MARIADB_ROOT_PASSWORD" \
        -p "$port:3306" \
        "$image" $extra >/dev/null
    log "  server  $container started on :$port ${extra:+($extra)}"
}

# Readiness is the published port answering, never a `docker exec` client.
# During initialisation the official entrypoint runs a temporary server with
# --skip-networking: it answers SELECT 1 over the Unix socket while the init
# scripts are still running, and it then goes away to be replaced by the real
# one. A check made from inside the container therefore returns true in the
# middle of the init and hands the caller a server that is about to restart,
# which is exactly the race this fixture hit. The published port is up only when
# the server under test is.
wait_ready() {
    local container="$4" port="$5" deadline version
    deadline=$(( $(date +%s) + TIMEOUT_SECONDS ))
    while :; do
        version="$(tpl_mariadb_handshake "$port" 2>/dev/null || true)"
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

# A failure inside /docker-entrypoint-initdb.d is reported in the log and leaves
# the schema incomplete while the container still reports itself up, so neither
# `docker ps` nor the port answering is the check. Both halves are.
verify_clean() {
    local name="$1" container="$4" errors count
    errors="$(docker logs "$container" 2>&1 | grep -c '\[ERROR\]' || true)"
    [ "$errors" -eq 0 ] \
        || fail_container "$container" "$container logged $errors [ERROR] lines"

    count="$(tpl_mariadb_sql "$name" -N -B \
        -e "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA='$TPL_MARIADB_SCHEMA'")"
    [ "$count" = "$TPL_MARIADB_EXPECTED_TABLES" ] \
        || fail_container "$container" \
             "$container has $count objects in $TPL_MARIADB_SCHEMA, expected $TPL_MARIADB_EXPECTED_TABLES"
    log "  ready   $container: $count catalogue objects in $TPL_MARIADB_SCHEMA, no [ERROR] in log"
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
    local name series image container port tls
    # The inventory is read on file descriptor 3, not on stdin. `docker exec -i`
    # reads stdin, and a loop that feeds itself from stdin hands the rest of its
    # own input to the first such command: the loop then stops after one server,
    # silently and looking like a filter that matched once.
    while read -r name series image container port tls <&3; do
        [ -n "$name" ] || continue
        wanted "$name" "$@" || continue
        log "$name"
        build_if_missing  "$series" "$image"
        start_if_absent   "$name" "$series" "$image" "$container" "$port" "$tls"
        wait_ready        "$name" "$series" "$image" "$container" "$port" "$tls"
        verify_clean      "$name" "$series" "$image" "$container" "$port" "$tls"
    done 3<<< "$TPL_MARIADB_SERVERS"
    log ""
    ./status.sh
}

main "$@"
