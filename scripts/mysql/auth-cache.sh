#!/usr/bin/env bash
#
# Put a server's caching_sha2_password cache in a known state.
#
# FR-MYSQL-045 makes the outcome of a login over a session TLS does not protect
# depend on that cache, and on nothing in the entry: while the server holds a
# cached login for the account, the challenge-response exchange needs no
# secure connection and the login succeeds; once the cache is emptied, the full
# exchange needs the server's RSA public key, which `tpl` never requests, and
# the read ends with 78. A test of either outcome therefore has to set the
# cache first, or its result depends on which tests ran before it.
#
# Usage:
#     ./auth-cache.sh flush <server>             # empty the cache: every account
#     ./auth-cache.sh prime <server> [<user>]    # fill it for one account, by one login
#
# <server> is a name from series.env. <user> is tpl_reader (the default) or
# root, the two accounts reachable over TCP; each is created for the host `%`.
#
# flush runs FLUSH PRIVILEGES, which the MySQL manual names as one of the
# events that empty the cache, the others being a server restart and a change
# to the account. It is issued as root over the container's Unix socket.
#
# prime makes one successful login of the account over TCP from inside the
# container, to 127.0.0.1, so that it authenticates as user@'%' — the account a
# test reaches from the host — and not as an account for `localhost`. On a
# server offering TLS the login is encrypted, a secure connection on which the
# full exchange needs no RSA key. On a server offering none, the login asks the
# server for its public key (--get-server-public-key). That is the harness
# filling a cache, and it is exactly what `tpl` must not do; it is done here,
# outside `tpl`, so that the cached outcome can be observed at all on a server
# that offers no TLS.
#
# State after ./up.sh: the cache holds no entry for root@'%' or tpl_reader@'%'.
# The entrypoint and every harness script connect as root@'localhost' over the
# socket, and no harness script logs in over TCP but this one.

set -euo pipefail

cd "$(dirname "$0")"
. ./series.env

die() { printf 'auth-cache.sh: %s\n' "$*" >&2; exit 2; }

[ $# -ge 2 ] || die "usage: auth-cache.sh flush <server> | prime <server> [<user>]"
action="$1"; server="$2"; shift 2
tpl_mysql_record "$server" >/dev/null || die "unknown server: $server (the names are in series.env)"

case "$action" in
    flush)
        [ $# -eq 0 ] || die "flush takes no user: it empties the cache of every account"
        # FLUSH PRIVILEGES draws warning 1681, "deprecated and will be removed
        # in a future release", on 9.7 and none on 8.4; on both it empties the
        # cache, which is what README.md's matrix was produced with.
        tpl_mysql_sql "$server" -e "FLUSH PRIVILEGES"
        printf 'flushed   %s: the caching_sha2_password cache holds no account\n' "$server" >&2
        ;;
    prime)
        user="${1:-$TPL_MYSQL_READER_USER}"
        case "$user" in
            "$TPL_MYSQL_READER_USER") password="$TPL_MYSQL_READER_PASSWORD" ;;
            root)                     password="$TPL_MYSQL_ROOT_PASSWORD" ;;
            *) die "unknown account: $user ($TPL_MYSQL_READER_USER or root)" ;;
        esac
        container="$(tpl_mysql_field "$server" 4)"
        if [ "$(tpl_mysql_field "$server" 6)" = yes ]; then
            transport=(--ssl-mode=REQUIRED)
        else
            transport=(--ssl-mode=DISABLED --get-server-public-key)
        fi
        docker exec -e "MYSQL_PWD=$password" "$container" \
            mysql "${transport[@]}" -h 127.0.0.1 -P 3306 -u "$user" -N -B -e "SELECT 1" >/dev/null
        printf 'primed    %s: %s@%% holds a cached login (%s)\n' "$server" "$user" "${transport[*]}" >&2
        ;;
    *) die "unknown action: $action (flush, prime)" ;;
esac
