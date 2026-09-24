#!/bin/sh
# Installs or updates tpl from its latest GitHub Release.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/FlavioCFOliveira/tpl/main/install.sh | sh
#
# Environment:
#   TPL_INSTALL_DIR  Directory to install tpl into. Default: /usr/local/bin.
#                    It is created when missing. sudo is used only when the
#                    directory, or its parent when creating it, is not writable.
#
# The archive for this OS and CPU is verified against the release's SHA256SUMS
# before anything is installed.
set -eu

repo=FlavioCFOliveira/tpl
dir=${TPL_INSTALL_DIR:-/usr/local/bin}
dir=${dir%/}
dir=${dir:-/}

say() { printf 'install.sh: %s\n' "$*" >&2; }
die() {
    say "error: $*"
    exit 1
}
has() { command -v "$1" >/dev/null 2>&1; }

for tool in curl tar uname mkdir rm; do
    has "$tool" || die "'$tool' is required and was not found"
done
has install || { has cp && has chmod; } || die "'install', or 'cp' and 'chmod', is required"
has sha256sum || has shasum || die "'sha256sum' or 'shasum' is required"

sha256_check() {
    if has sha256sum; then sha256sum -c - >/dev/null; else shasum -a 256 -c - >/dev/null; fi
}

os=$(uname -s)
cpu=$(uname -m)
case "$os/$cpu" in
    Linux/x86_64) target=x86_64-unknown-linux-musl ;;
    Linux/aarch64 | Linux/arm64) target=aarch64-unknown-linux-musl ;;
    Darwin/x86_64) target=x86_64-apple-darwin ;;
    Darwin/arm64) target=aarch64-apple-darwin ;;
    *) die "no tpl build for $os $cpu; supported: Linux and macOS on x86_64 and arm64" ;;
esac

# The /releases/latest page redirects to /releases/tag/<tag>; following it
# avoids the REST API and its unauthenticated rate limit.
say "target $target; resolving the latest release of $repo"
url=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest") ||
    die "could not reach https://github.com/$repo/releases/latest"
case "$url" in
    */releases/tag/?*) tag=${url##*/releases/tag/} ;;
    *) die "$repo has no published release" ;;
esac
version=${tag#v}

if [ -x "$dir/tpl" ] && [ "$("$dir/tpl" --version 2>/dev/null || true)" = "tpl $version" ]; then
    say "tpl $version is already installed at $dir/tpl"
    exit 0
fi

tmp="${TMPDIR:-/tmp}/tpl-install.$$"
mkdir -m 700 "$tmp" || die "could not create $tmp"
trap 'rm -rf "$tmp"' EXIT
trap 'exit 1' HUP INT TERM

archive="tpl-$tag-$target.tar.gz"
base="https://github.com/$repo/releases/download/$tag"
say "downloading $archive"
curl -fsSL -o "$tmp/$archive" "$base/$archive" || die "could not download $base/$archive"
curl -fsSL -o "$tmp/SHA256SUMS" "$base/SHA256SUMS" || die "could not download $base/SHA256SUMS"

expected=
while read -r sum file; do
    if [ "$file" = "$archive" ]; then expected=$sum; fi
done <"$tmp/SHA256SUMS"
[ -n "$expected" ] || die "SHA256SUMS lists no $archive"
(cd "$tmp" && printf '%s  %s\n' "$expected" "$archive" | sha256_check) ||
    die "checksum mismatch for $archive"
say "verified $archive"

tar -xzf "$tmp/$archive" -C "$tmp" tpl || die "could not extract tpl from $archive"

if [ -d "$dir" ]; then
    writable=$dir
else
    case "$dir" in
        */*) writable=${dir%/*} ;;
        *) writable=. ;;
    esac
    writable=${writable:-/}
fi
sudo=
if [ ! -w "$writable" ]; then
    has sudo || die "$writable is not writable and 'sudo' was not found"
    sudo=sudo
    say "$writable is not writable; using sudo"
fi
run() {
    if [ -n "$sudo" ]; then sudo "$@"; else "$@"; fi
}

[ -d "$dir" ] || run mkdir -p "$dir" || die "could not create $dir"
if has install; then
    run install -m 755 "$tmp/tpl" "$dir/tpl"
else
    run cp "$tmp/tpl" "$dir/tpl" && run chmod 755 "$dir/tpl"
fi || die "could not install tpl into $dir"

say "installed $("$dir/tpl" --version) at $dir/tpl"
