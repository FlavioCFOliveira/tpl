#!/usr/bin/env bash
#
# Stop and remove the fixture, and prove that nothing of it is left running.
#
# It removes exactly the containers series.env names, because the host may be
# running containers that belong to other projects, and it removes the
# anonymous volume each one created with it: the entrypoint populates the data
# directory only when it is empty, so a volume that outlives its container is a
# stale schema waiting to be mistaken for a fresh one.
#
# Usage:
#     ./down.sh              # all five
#     ./down.sh 11.8         # only the named servers
#     ./down.sh --images     # ...and drop the four images too
#
# Safe to run when nothing is up, and safe to run twice. It is the second half
# of up.sh and the answer to "leave no container running after a validation
# run".

set -euo pipefail

cd "$(dirname "$0")"
. ./series.env

log() { printf '%s\n' "$*" >&2; }

DROP_IMAGES=no
ARGS=()
for a in "$@"; do
    case "$a" in
        --images) DROP_IMAGES=yes ;;
        *) ARGS+=("$a") ;;
    esac
done

wanted() {
    [ "${#ARGS[@]}" -eq 0 ] && return 0
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

removed=0
while read -r name series image container port tls; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    if docker inspect "$container" >/dev/null 2>&1; then
        docker rm -f -v "$container" >/dev/null
        log "  removed   $container"
        removed=$((removed + 1))
    else
        log "  absent    $container"
    fi
done <<< "$TPL_MARIADB_SERVERS"

if [ "$DROP_IMAGES" = yes ]; then
    while read -r name series image container port tls; do
        [ -n "$name" ] || continue
        docker image inspect "$image" >/dev/null 2>&1 && docker rmi "$image" >/dev/null 2>&1 || true
    done <<< "$TPL_MARIADB_SERVERS"
    log "  images dropped"
fi

# The proof, not the intention: ask the daemon what is left of this fixture.
left="$(docker ps --filter 'name=tpl-mariadb-' --format '{{.Names}}')"
if [ -n "$left" ]; then
    log ""
    log "down.sh: still running after teardown:"
    printf '  %s\n' $left >&2
    exit 1
fi

log ""
log "removed $removed container(s); docker ps lists no tpl-mariadb container"
