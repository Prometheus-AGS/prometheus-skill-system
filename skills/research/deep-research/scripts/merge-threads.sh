#!/usr/bin/env bash
# merge-threads.sh — fold thread artifacts back into stage 02/03/04, deterministically.
#
# This is the change that makes threading a refactor rather than a rewrite. It
# is a pure function of the thread artifacts: no model call, no network, no
# clock, no randomness. Two runs over the same threads emit byte-identical
# output, which is asserted by tests/merge-threads.sh.
#
# It is also the ENFORCEMENT point for the director's no-search rule. A `tools:`
# allowlist is intent; a harness may ignore frontmatter. A dossier citing a
# source absent from its own thread's sources.json means something fetched
# outside a worker — a searching director, or a worker that sub-dispatched — and
# that is refused as CRITICAL rather than merged.
#
# Usage: merge-threads.sh --package <dir>
# Exit:  0 merged, 1 usage/IO error, 2 CRITICAL contract violation.
#
# bash 3.2 compatible (C-05): no mapfile, no declare -A.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO="$(cd "$HERE/../../../.." && pwd -P)"

# One canonical URL rule for stage 02 and the merge; see the library's header.
LIB="$REPO/shared/scripts/lib/canonical-url.sh"
[ -f "$LIB" ] || { echo "merge-threads: missing $LIB" >&2; exit 1; }
# shellcheck source=/dev/null
. "$LIB"

PKG=""
while [ $# -gt 0 ]; do
    case "$1" in
        --package) PKG="${2:-}"; shift 2 ;;
        -h|--help) sed -n '2,16p' "$0"; exit 0 ;;
        *) echo "merge-threads: unknown argument: $1" >&2; exit 1 ;;
    esac
done

[ -n "$PKG" ] || { echo "merge-threads: --package <dir> is required" >&2; exit 1; }
[ -d "$PKG/threads" ] || { echo "merge-threads: no threads/ under $PKG" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "merge-threads: python3 is required" >&2; exit 1; }

mkdir -p "$PKG/sources"

# The whole merge is one python3 process: it is a pure data transform, and
# splitting it across shell pipelines would make determinism harder to argue
# than to achieve. The canonical URL rule reaches it via the shared library
# path, so there is exactly one implementation of it.

# The program lives beside this script rather than inside a heredoc. The
# heredoc form hangs indefinitely on some hosts (observed 2026-09-09: this
# exact program blocked past 90s inline and ran in under a second from a
# file). `exec` replaces the shell so the exit code — 2 for a CRITICAL
# contract violation — passes through unchanged.
HERE="$(cd "$(dirname "$0")" && pwd)"
PROG="$HERE/merge-threads.py"
[ -f "$PROG" ] || { echo "merge-threads: merge-threads.py missing beside this script" >&2; exit 1; }

exec python3 "$PROG" "$PKG" "$LIB"
