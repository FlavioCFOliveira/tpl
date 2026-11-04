#!/usr/bin/env bash
#
# Is the fixture up? The gate a server-dependent test asks before it runs.
#
# It probes each published port the way a test actually reaches it: from the
# host, over TCP, and with no client installed. A MySQL server sends its
# handshake packet the moment the socket opens, and that packet carries the
# server version, so what this reports is "a MySQL server of version X answered
# on this port" rather than "something is listening". No client is needed,
# which matters: a contributor may have none, and the gate must still be
# askable.
#
# Usage:
#     ./status.sh                    # human-readable table
#     ./status.sh --quiet            # exit code only
#     ./status.sh --export           # shell assignments for a test runner to read
#     ./status.sh 9.7 notls-9.7      # only the named servers
#
# Exit code, which is the gate:
#     0   every requested server answered            -> run the server tests
#     1   none answered                              -> skip them, and say why
#     2   some answered and some did not, or an      -> a broken fixture, not an
#         unknown server was named                      absent one; do not skip
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
        *) tpl_mysql_record "$a" >/dev/null \
               || { printf 'status.sh: unknown server: %s (the names are in series.env)\n' "$a" >&2; exit 2; }
           ARGS+=("$a") ;;
    esac
done

wanted() {
    [ "${#ARGS[@]}" -eq 0 ] && return 0
    for w in "${ARGS[@]}"; do [ "$w" = "$1" ] && return 0; done
    return 1
}

up=0; down=0; rows=""
# The inventory is read on file descriptor 9, not on stdin, on the same terms
# `tpl_mysql_each` and up.sh read it. Descriptor 9 and not 3:
# `tpl_mysql_handshake` opens the TCP port on 3.
while read -r name series image container port tls standing <&9; do
    [ -n "$name" ] || continue
    wanted "$name" || continue
    version="$(tpl_mysql_handshake "$port" 2>/dev/null || true)"
    if [ -n "$version" ]; then
        up=$((up + 1))
        rows="$rows$name|$series|$port|$tls|$standing|up|$version"$'\n'
    else
        down=$((down + 1))
        rows="$rows$name|$series|$port|$tls|$standing|down|-"$'\n'
    fi
done 9<<< "$TPL_MYSQL_SERVERS"

case "$MODE" in
    table)
        printf '%-10s %-7s %-6s %-4s %-10s %-5s %s\n' SERVER SERIES PORT TLS STANDING STATE VERSION
        printf '%s' "$rows" | while IFS='|' read -r n s p t st state v; do
            if [ -n "$n" ]; then
                printf '%-10s %-7s %-6s %-4s %-10s %-5s %s\n' "$n" "$s" "$p" "$t" "$st" "$state" "$v"
            fi
        done
        ;;
    export)
        # For a test runner, which parses these lines rather than evaluating
        # them. TPL_MYSQL_READY is the gate in one variable, for a runner that
        # cannot read an exit code where it needs the answer. The name of a
        # server becomes a variable name by upper-casing it and turning every
        # dot and hyphen into an underscore: notls-8.4 is TPL_MYSQL_NOTLS_8_4.
        printf '%s' "$rows" | while IFS='|' read -r n s p t st state v; do
            [ -n "$n" ] || continue
            var="TPL_MYSQL_$(printf '%s' "$n" | tr 'a-z.-' 'A-Z__')"
            # An `if`, not `[ ] && printf`: a loop whose last command fails
            # fails the pipeline, and under `set -e -o pipefail` that ends the
            # script before the gate is printed whenever the last server is down.
            if [ "$state" = up ]; then
                printf '%s=127.0.0.1:%s; export %s\n' "$var" "$p" "$var"
            fi
        done
        if   [ "$down" -eq 0 ]; then printf 'TPL_MYSQL_READY=all; export TPL_MYSQL_READY\n'
        elif [ "$up"   -eq 0 ]; then printf 'TPL_MYSQL_READY=none; export TPL_MYSQL_READY\n'
        else                         printf 'TPL_MYSQL_READY=partial; export TPL_MYSQL_READY\n'
        fi
        ;;
esac

if   [ "$down" -eq 0 ]; then exit 0
elif [ "$up"   -eq 0 ]; then exit 1
else                         exit 2
fi
