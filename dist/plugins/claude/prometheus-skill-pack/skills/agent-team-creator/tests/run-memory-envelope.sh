#!/bin/bash
# run-memory-envelope.sh — run memory-envelope.integration.mjs against a SCRATCH
# surreal-memory-server (never the live :23001). bash 3.2.
# Exit 0 pass, 1 fail, 2 BLOCKED (binary/executor missing or too old).
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../.." && pwd)"
. "$REPO_ROOT/shared/scripts/tests/lib/scratch-surreal.sh"

# Refuse to run against the live service, whatever the environment says.
case "${SURREAL_MEMORY_URL:-}" in
  *:23001|*:23001/*) echo "FAIL: SURREAL_MEMORY_URL points at the live :23001 service; this runner uses a scratch server only" >&2; exit 1 ;;
esac
unset SURREAL_MEMORY_URL
command -v node >/dev/null 2>&1 || blocked "node missing"
[ -f "$HERE/memory-envelope.integration.mjs" ] || blocked "memory-envelope.integration.mjs missing (npm run build:tests)"

S="$(mktemp -d "${TMPDIR:-/tmp}/tlh-envelope.XXXXXX")"
cleanup() { scratch_surreal_stop >/dev/null 2>&1; rm -rf "$S"; }
trap cleanup EXIT

PORT="$(scratch_surreal_pick_port)"
[ "$PORT" != 23001 ] || { echo "FAIL: scratch port resolved to the live :23001" >&2; exit 1; }
SCRATCH_SURREAL_PORT="$PORT"
scratch_surreal_start "$S/sm" envelope
case "$SURREAL_MEMORY_URL" in *:23001|*:23001/*) echo "FAIL: scratch URL is :23001" >&2; exit 1 ;; esac

( cd "$HERE/../runtime" && node ../tests/memory-envelope.integration.mjs )
rc=$?

dir="$SCRATCH_SURREAL_DIR"
scratch_surreal_stop || { echo "FAIL: scratch surreal-memory left a process or listener" >&2; exit 1; }
if pgrep -f "$dir" >/dev/null 2>&1; then echo "FAIL: a process still holds $dir" >&2; exit 1; fi
[ "$rc" -eq 0 ] || { echo "FAIL: memory-envelope.integration exited $rc" >&2; [ "$rc" -eq 2 ] && exit 2; exit 1; }
echo "run-memory-envelope: passed against scratch $SURREAL_MEMORY_URL"
