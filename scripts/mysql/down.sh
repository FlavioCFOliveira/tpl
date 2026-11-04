#!/usr/bin/env bash
#
# Stop and remove the fixture, and prove that nothing of it is left running.
#
# It removes exactly the containers series.env names, because the host may be
# running containers that belong to other projects. up.sh gives every server a
# tmpfs data directory, so removing the container removes its schema with it;
# `-v` is still passed, so that a container started by hand with the image's
# anonymous volume leaves no volume behind either: the entrypoint populates the
# data directory only when it is empty, and a volume that outlives its
# container is a stale schema waiting to be mistaken for a fresh one.
#
# Usage:
#     ./down.sh              # all five
#     ./down.sh 9.7          # only the named servers
#     ./down.sh --images     # ...and drop the three images too
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
        *) tpl_mysql_record "$a" >/dev/null \
               || { printf 'down.sh: unknown server: %s (the names are in series.env)\n' "$a" >&2; exit 2; }
           ARGS+=("$a") ;;
    esac
done

wanted() {
    [ "${#ARGS[@]}" -eq 0 ] && return 0
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

# Both loops below read the inventory on file descriptor 9, not on stdin, on
# the same terms `tpl_mysql_each` and up.sh read it: a command in either body
# that reads stdin would drain the records still to be read, and the loop would
# end after the first server — silently, and with exit 0, leaving the others
# running while reporting success. Descriptor 9 and not 3:
# `tpl_mysql_handshake` opens the TCP port on 3.
removed=0
while read -r name series image container port tls standing <&9; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    if docker inspect "$container" >/dev/null 2>&1; then
        docker rm -f -v "$container" >/dev/null
        log "  removed   $container"
        removed=$((removed + 1))
    else
        log "  absent    $container"
    fi
done 9<<< "$TPL_MYSQL_SERVERS"

if [ "$DROP_IMAGES" = yes ]; then
    # An image is shared by every server of its series, so it is dropped only
    # once no container of the fixture is left to use it; `docker rmi` refuses
    # an image in use, and that refusal is not hidden.
    for image in $(printf '%s\n' "$TPL_MYSQL_SERVERS" | awk 'NF { print $3 }' | sort -u); do
        if docker image inspect "$image" >/dev/null 2>&1; then
            docker rmi "$image" >/dev/null
            log "  dropped   $image"
        fi
    done
fi

# The proof, not the intention: ask the daemon what is left of this fixture.
left="$(docker ps -a --filter 'name=tpl-mysql-' --format '{{.Names}}')"
if [ -n "$left" ]; then
    log ""
    if [ "${#ARGS[@]}" -gt 0 ]; then
        # A partial teardown leaves the servers that were not named, which is
        # what was asked for; what must be gone is every server that was.
        for w in "${ARGS[@]}"; do
            c="$(tpl_mysql_field "$w" 4)"
            if printf '%s\n' "$left" | awk -v c="$c" '$0 == c { found = 1 } END { exit !found }'; then
                log "down.sh: $c is still present after teardown"
                exit 1
            fi
        done
        log "removed $removed container(s); docker ps -a still lists the servers not named:"
        printf '  %s\n' $left >&2
        exit 0
    fi
    log "down.sh: still present after teardown:"
    printf '  %s\n' $left >&2
    exit 1
fi

log ""
log "removed $removed container(s); docker ps -a lists no tpl-mysql container"
