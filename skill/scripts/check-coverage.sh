#!/bin/sh
# Check that the skill's command map names every command path, and every
# alias, that the installed tpl binary publishes in `tpl help --format json`.
#
# Usage: sh check-coverage.sh [MAP_FILE]
#   TPL       the tpl binary to query (default: tpl on PATH)
#   MAP_FILE  the command map (default: ../references/commands.md next to this script)
#
# Exit 0 when everything is covered, 1 when something is missing (listed on
# stderr), 2 when tpl or the map cannot be used.

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
map=${1:-"$script_dir/../references/commands.md"}
tpl_bin=${TPL:-tpl}

if ! command -v "$tpl_bin" >/dev/null 2>&1; then
    echo "check-coverage: tpl not found; put it on PATH or set TPL=/path/to/tpl" >&2
    exit 2
fi
if [ ! -r "$map" ]; then
    echo "check-coverage: cannot read the command map: $map" >&2
    exit 2
fi

tmp=$(mktemp "${TMPDIR:-/tmp}/tpl-coverage.XXXXXX")
trap 'rm -f "$tmp" "$tmp.paths" "$tmp.aliases"' EXIT HUP INT TERM

# Write the document to a file: a reader that stops early would make tpl exit 74.
if ! "$tpl_bin" help --format json >"$tmp"; then
    echo "check-coverage: '$tpl_bin help --format json' failed" >&2
    exit 2
fi

if command -v jq >/dev/null 2>&1; then
    jq -r '.data.commands[].path | join(" ")' "$tmp" >"$tmp.paths"
    jq -r '.data.commands[].aliases[]' "$tmp" >"$tmp.aliases"
else
    # Pure-sh fallback: the document is one line of compact JSON in which each
    # command carries "path":[...] immediately followed by "aliases":[...].
    awk -v aliases_file="$tmp.aliases" '{
        n = split($0, parts, "\"path\":\\[")
        for (i = 2; i <= n; i++) {
            p = parts[i]; sub(/\].*/, "", p); gsub(/"/, "", p); gsub(/,/, " ", p)
            print p
            a = parts[i]; sub(/^[^]]*\],"aliases":\[/, "", a); sub(/\].*/, "", a)
            gsub(/"/, "", a); m = split(a, al, ",")
            for (j = 1; j <= m; j++) if (al[j] != "") print al[j] > aliases_file
        }
    }' "$tmp" >"$tmp.paths"
    : >>"$tmp.aliases"
fi

if [ ! -s "$tmp.paths" ]; then
    echo "check-coverage: no command path found in the help document" >&2
    exit 2
fi

missing=0
total=0
while IFS= read -r path; do
    [ -n "$path" ] || continue
    total=$((total + 1))
    # A path counts as covered when the map writes it as `tpl <path>` followed by
    # a closing backtick or a space (so "schema table" is not satisfied by "schema tables").
    if ! grep -F -e "\`tpl $path\`" -e "\`tpl $path " "$map" >/dev/null 2>&1; then
        echo "missing command: tpl $path" >&2
        missing=$((missing + 1))
    fi
done <"$tmp.paths"

aliases=0
while IFS= read -r alias; do
    [ -n "$alias" ] || continue
    aliases=$((aliases + 1))
    if ! grep -F -e "\`$alias\`" "$map" >/dev/null 2>&1; then
        echo "missing alias: $alias" >&2
        missing=$((missing + 1))
    fi
done <"$tmp.aliases"

version=$("$tpl_bin" version 2>/dev/null || echo "tpl (unknown version)")
if [ "$missing" -gt 0 ]; then
    echo "check-coverage: FAIL, $missing item(s) missing from $map ($version)" >&2
    exit 1
fi
echo "check-coverage: OK, $total command paths and $aliases aliases covered ($version)"
