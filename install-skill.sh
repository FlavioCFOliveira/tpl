#!/bin/sh
# Installs or updates the tpl skill for Claude Code from the latest GitHub Release.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install-skill.sh | sh
#
# Environment:
#   TPL_SKILL_DIR      Directory to install the skill into.
#                      Default: $CLAUDE_CONFIG_DIR/skills/tpl.
#   CLAUDE_CONFIG_DIR  Claude Code's configuration directory. Default: $HOME/.claude.
#
# The skill archive is verified against the release's SHA256SUMS before the
# destination is touched. An existing destination directory is replaced; a
# symbolic link there is removed, never its target. sudo is never used.
set -eu

repo=FlavioCFOliveira/tpl
dest=${TPL_SKILL_DIR:-${CLAUDE_CONFIG_DIR:-$HOME/.claude}/skills/tpl}
dest=${dest%/}

say() { printf 'install-skill.sh: %s\n' "$*" >&2; }
die() {
    say "error: $*"
    exit 1
}
has() { command -v "$1" >/dev/null 2>&1; }

[ -n "$dest" ] || die "refusing to install into /"

for tool in curl tar mkdir rm mv; do
    has "$tool" || die "'$tool' is required and was not found"
done
has sha256sum || has shasum || die "'sha256sum' or 'shasum' is required"

sha256_check() {
    if has sha256sum; then sha256sum -c - >/dev/null; else shasum -a 256 -c - >/dev/null; fi
}

# The /releases/latest page redirects to /releases/tag/<tag>; following it
# avoids the REST API and its unauthenticated rate limit.
say "resolving the latest release of $repo"
url=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest") ||
    die "could not reach https://github.com/$repo/releases/latest"
case "$url" in
    */releases/tag/?*) tag=${url##*/releases/tag/} ;;
    *) die "$repo has no published release" ;;
esac

tmp="${TMPDIR:-/tmp}/tpl-skill-install.$$"
mkdir -m 700 "$tmp" || die "could not create $tmp"
trap 'rm -rf "$tmp"' EXIT
trap 'exit 1' HUP INT TERM

archive="tpl-skill-$tag.tar.gz"
base="https://github.com/$repo/releases/download/$tag"
curl -fsSL -o "$tmp/SHA256SUMS" "$base/SHA256SUMS" || die "could not download $base/SHA256SUMS"

# Every release carries SHA256SUMS; one that ships the skill lists its archive.
expected=
while read -r sum file; do
    if [ "$file" = "$archive" ]; then expected=$sum; fi
done <"$tmp/SHA256SUMS"
[ -n "$expected" ] ||
    die "release $tag has no skill archive; the skill ships from the first release that carries tpl-skill-<tag>.tar.gz"

say "downloading $archive"
curl -fsSL -o "$tmp/$archive" "$base/$archive" || die "could not download $base/$archive"
(cd "$tmp" && printf '%s  %s\n' "$expected" "$archive" | sha256_check) ||
    die "checksum mismatch for $archive"
say "verified $archive"

mkdir "$tmp/stage"
tar -xzf "$tmp/$archive" -C "$tmp/stage" || die "could not extract $archive"
[ -f "$tmp/stage/SKILL.md" ] || die "$archive has no SKILL.md at its root"

case "$dest" in
    */*) parent=${dest%/*} ;;
    *) parent=. ;;
esac
parent=${parent:-/}
mkdir -p "$parent" || die "could not create $parent"

# A link is removed as a link (its target is never followed); a directory is
# replaced; anything else is left alone.
if [ -L "$dest" ]; then
    rm "$dest" || die "could not remove the symbolic link $dest"
elif [ -d "$dest" ]; then
    rm -rf "$dest" || die "could not remove $dest"
elif [ -e "$dest" ]; then
    die "$dest exists and is neither a directory nor a symbolic link; remove it or set TPL_SKILL_DIR"
fi
mv "$tmp/stage" "$dest" || die "could not move the skill into $dest"

say "installed the tpl skill $tag at $dest"
say "Claude Code reloads skills live; if its skills directory did not exist when the current session started, restart Claude Code"
