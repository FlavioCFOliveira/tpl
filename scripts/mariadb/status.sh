#!/usr/bin/env bash
#
# Is the fixture up? The gate a server-dependent test asks before it runs.
#
# It probes each published port the way a test actually reaches it: from the
# host, over TCP, and with no client installed. A MariaDB server sends its
# handshake packet the moment the socket opens, and that packet carries the
# server version, so what this reports is "a MariaDB of series X answered on
# this port" rather than "something is listening". No client is needed, which
# matters: a contributor may have none, and the gate must still be askable.
#
# Usage:
#     ./status.sh                 # human-readable table
#     ./status.sh --quiet         # exit code only
#     ./status.sh --export        # shell assignments for a test runner to eval
#     ./status.sh 11.8 notls      # only the named servers
#
# Exit code, which is the gate:
#     0   every requested server answered            -> run the server tests
#     1   none answered                              -> skip them, and say why
#     2   some answered and some did not             -> a broken fixture, not an
#                                                       absent one; do not skip
#
# The three are distinguished deliberately. A test runner that treats "no
# fixture" and "half a fixture" alike either skips silently over a real failure
# or fails the build of every contributor who has no Docker.

set -euo pipefail

cd "$(dirname "$0")"
. ./series.env

MODE=table
ARGS=()
for a in "$@"; do
    case "$a" in
        --quiet)  MODE=quiet ;;
        --export) MODE=export ;;
        *) ARGS+=("$a") ;;
    esac
done

wanted() {
    [ "${#ARGS[@]}" -eq 0 ] && return 0
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

up=0; down=0; rows=""
while read -r name series image container port tls; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    version="$(tpl_mariadb_handshake "$port" 2>/dev/null || true)"
    if [ -n "$version" ]; then
        up=$((up + 1))
        rows="$rows$name|$port|$tls|up|$version"$'\n'
    else
        down=$((down + 1))
        rows="$rows$name|$port|$tls|down|-"$'\n'
    fi
done <<< "$TPL_MARIADB_SERVERS"

case "$MODE" in
    table)
        printf '%-7s %-7s %-5s %-5s %s\n' SERVER PORT TLS STATE VERSION
        printf '%s' "$rows" | while IFS='|' read -r n p t s v; do
            [ -n "$n" ] && printf '%-7s %-7s %-5s %-5s %s\n' "$n" "$p" "$t" "$s" "$v"
        done
        ;;
    export)
        # For a test runner: eval "$(./status.sh --export)". TPL_MARIADB_READY
        # is the gate in one variable, for a runner that cannot read an exit
        # code where it needs the answer.
        printf '%s' "$rows" | while IFS='|' read -r n p t s v; do
            [ -n "$n" ] || continue
            var="TPL_MARIADB_$(printf '%s' "$n" | tr 'a-z.' 'A-Z_')"
            [ "$s" = up ] && printf '%s=127.0.0.1:%s; export %s\n' "$var" "$p" "$var"
        done
        if   [ "$down" -eq 0 ]; then printf 'TPL_MARIADB_READY=all; export TPL_MARIADB_READY\n'
        elif [ "$up"   -eq 0 ]; then printf 'TPL_MARIADB_READY=none; export TPL_MARIADB_READY\n'
        else                         printf 'TPL_MARIADB_READY=partial; export TPL_MARIADB_READY\n'
        fi
        ;;
esac

if   [ "$down" -eq 0 ]; then exit 0
elif [ "$up"   -eq 0 ]; then exit 1
else                         exit 2
fi
