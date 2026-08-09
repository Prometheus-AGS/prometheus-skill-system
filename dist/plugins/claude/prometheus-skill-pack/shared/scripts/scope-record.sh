#!/usr/bin/env bash
# scope-record.sh — PostToolUse(Write|Edit|MultiEdit) companion to scope-guard.
#
# When an out-of-scope write actually happened (user approved it in ask mode, or
# warn mode let it through), record a scope_overrides entry on the waypoint so
# the same path is not re-flagged on subsequent edits. Idempotent.
#
# Always exits 0 (PostToolUse must never disrupt the chain).
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PostToolUse" "scope-record.sh"
finish() { hook_log_end 0; exit 0; }

command -v jq >/dev/null 2>&1 || finish
command -v python3 >/dev/null 2>&1 || finish
INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || finish
FILE_PATH="$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try: d = json.load(sys.stdin)
except Exception: print(''); raise SystemExit
ti = d.get('tool_input', {}) or {}
print(ti.get('file_path') or ti.get('path') or '')
" 2>/dev/null || true)"
[ -n "$FILE_PATH" ] || finish

_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_root)" || finish
WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
[ -f "$WP" ] && jq empty "$WP" 2>/dev/null || finish

CHANGE="$(jq -r '.change // .active_change // empty' "$WP")"
[ -n "$CHANGE" ] || finish
SCOPED="$(jq -c '.scoped_paths // []' "$WP")"
[ "$SCOPED" = "[]" ] && finish

# Relativize against the discovered root, canonicalizing both sides so macOS
# /var vs /private/var symlinks don't defeat the prefix match.
_canon() { ( cd "$1" 2>/dev/null && pwd -P ) || printf '%s' "$1"; }
ROOT_REAL="$(_canon "$ROOT")"
FP_DIR_REAL="$(_canon "$(dirname "$FILE_PATH")")"
FP_REAL="$FP_DIR_REAL/$(basename "$FILE_PATH")"
REL="$FP_REAL"
case "$FP_REAL" in "$ROOT_REAL"/*) REL="${FP_REAL#"$ROOT_REAL"/}" ;; esac
case "$REL" in .kbd-orchestrator/*|SCRATCHPAD.md) finish ;; esac

# Was it out of scope and not already recorded?
NEEDS="$(REL="$REL" SCOPED="$SCOPED" WP="$WP" python3 -c '
import json, os, fnmatch
rel = os.environ.get("REL", "")
scoped = json.loads(os.environ.get("SCOPED", "[]"))
try:
    wp = json.load(open(os.environ["WP"]))
except Exception:
    print("no"); raise SystemExit
if rel in [o.get("path") for o in (wp.get("scope_overrides") or [])]:
    print("no"); raise SystemExit
for g in scoped:
    if fnmatch.fnmatch(rel, g):
        print("no"); raise SystemExit
print("yes")
' 2>/dev/null || echo "no")"
[ "$NEEDS" = "yes" ] || finish

NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo unknown)"
TMP="$(mktemp)"
jq --arg p "$REL" --arg now "$NOW" \
  '.scope_overrides = ((.scope_overrides // []) + [{path:$p, reason:"post-write recorded", approvedAt:$now}])' \
  "$WP" > "$TMP" 2>/dev/null && mv "$TMP" "$WP" || rm -f "$TMP"
echo "[scope-record] recorded scope expansion: $REL ($CHANGE)" >&2
finish
