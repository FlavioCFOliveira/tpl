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

# Both loops below read the inventory on file descriptor 9, not on stdin, on
# the same terms `tpl_mariadb_each` and up.sh read it: a command in either body
# that reads stdin would drain the records still to be read, and the loop would
# end after the first server — silently, and with exit 0, leaving four
# containers running while reporting success. `docker rm` and `docker rmi` do
# not read stdin, so the defect is latent rather than present, and the
# descriptor is what keeps it that way. Descriptor 9 and not 3:
# `tpl_mariadb_handshake` opens the TCP port on 3.
removed=0
while read -r name series image container port tls <&9; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    if docker inspect "$container" >/dev/null 2>&1; then
        docker rm -f -v "$container" >/dev/null
        log "  removed   $container"
        removed=$((removed + 1))
    else
        log "  absent    $container"
    fi
done 9<<< "$TPL_MARIADB_SERVERS"

if [ "$DROP_IMAGES" = yes ]; then
    while read -r name series image container port tls <&9; do
        [ -n "$name" ] || continue
        docker image inspect "$image" >/dev/null 2>&1 && docker rmi "$image" >/dev/null 2>&1 || true
    done 9<<< "$TPL_MARIADB_SERVERS"
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
