#!/usr/bin/env bash
#
# The canonical loop, as one command, so that it can be measured as one.
#
#     ./loop200.sh <binary> <entry> <template> <names-file>
#
# This is the eighth budget of `NFR-PERF-014` and it is the loop the help of
# `tpl render` prints: one invocation per object, because `FR-RND-002` renders
# once per invocation and `BR-RND-002` makes iterating the caller's job. What
# `BR-PERF-005` says it measures is **200 process startups**, not 200 iterations
# inside one process, so the startups are what this file performs and nothing
# here is amortised across them.
#
# WHY A SCRIPT AT ALL, WHEN `NFR-PERF-010` FORBIDS AN INTERVENING SHELL. That
# clause keeps a shell out of the *instrument*, so that a figure is the
# program's cost and not `sh -c`'s; `hyperfine --shell=none` satisfies it and is
# passed on every invocation this harness makes, this one included. Here the
# loop **is** the subject: a caller who renders 200 objects writes exactly this,
# and its cost is what the budget names. The driver is kept to a `read` and an
# exec per line so that what it adds to the subject is as close to nothing as a
# loop can be, and the record states that the loop was driven this way.
#
# It reads the names from a file rather than from `tpl schema tables`, which the
# canonical loop pipes into it, for the same reason: the enumeration is one
# invocation and the budget counts 200, so enumerating inside the measured
# command would put a 201st there and make each run's work depend on a pipe.

set -eu

if [ "$#" -ne 4 ]; then
    printf 'usage: loop200.sh <binary> <entry> <template> <names-file>\n' >&2
    exit 2
fi

BINARY="$1"
ENTRY="$2"
TEMPLATE="$3"
NAMES="$4"

while IFS= read -r name; do
    [ -n "$name" ] || continue
    "$BINARY" -d "$ENTRY" render "$TEMPLATE" --table "$name" > /dev/null
done < "$NAMES"
