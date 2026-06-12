#!/usr/bin/env bash
# scope-guard.sh — PreToolUse(Write|Edit|MultiEdit) change-set boundary guard.
#
# When a KBD change is active, its declared scope: globs are copied into the
# waypoint's scoped_paths. This guard flags writes outside that scope. Ships in
# warn mode (PROMETHEUS_SCOPE_ENFORCE=warn, default) — observe before blocking;
# flip to ask later.
#
# Modes (PROMETHEUS_SCOPE_ENFORCE):
#   off  → never acts (exit 0)
#   warn → non-blocking stderr notice on out-of-scope edits (default)
#   ask  → emits permissionDecision:"ask" JSON to surface a user dialog
#
# Always-allowed: .kbd-orchestrator/**, SCRATCHPAD.md.
# Degrades to exit 0 when there is no orchestrator, no active change, or no
# scoped_paths.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PreToolUse" "scope-guard.sh"

# Shared path-scope helper (canonicalize + relativize + always-allowed).
PSCOPE_LIB="$(cd "$(dirname "$0")" && pwd)/lib/path-scope.sh"
[ -f "$PSCOPE_LIB" ] && source "$PSCOPE_LIB"

MODE="${PROMETHEUS_SCOPE_ENFORCE:-warn}"
finish() { hook_log_end "${1:-0}"; exit "${1:-0}"; }
[ "$MODE" = "off" ] && finish 0
command -v jq >/dev/null 2>&1 || finish 0
command -v python3 >/dev/null 2>&1 || finish 0

INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || finish 0
FILE_PATH="$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try: d = json.load(sys.stdin)
except Exception: print(''); raise SystemExit
ti = d.get('tool_input', {}) or {}
print(ti.get('file_path') or ti.get('path') or '')
" 2>/dev/null || true)"
[ -n "$FILE_PATH" ] || finish 0

# Locate orchestrator root.
_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_root)" || finish 0
WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
[ -f "$WP" ] && jq empty "$WP" 2>/dev/null || finish 0

CHANGE="$(jq -r '.change // .active_change // empty' "$WP")"
[ -n "$CHANGE" ] || finish 0
SCOPED="$(jq -c '.scoped_paths // []' "$WP")"
[ "$SCOPED" = "[]" ] && finish 0

# Relativize via the shared helper (macOS symlink-safe).
REL="$(pscope_relativize "$ROOT" "$FILE_PATH")"

# Always-allowed paths.
pscope_always_allowed "$REL" && finish 0

# Already-approved override → allow.
OVERRIDDEN="$(REL="$REL" WP="$WP" python3 -c '
import json, os
rel = os.environ.get("REL", "")
try: wp = json.load(open(os.environ["WP"]))
except Exception: print("no"); raise SystemExit
print("yes" if rel in [o.get("path") for o in (wp.get("scope_overrides") or [])] else "no")
' 2>/dev/null || echo "no")"
[ "$OVERRIDDEN" = "yes" ] && finish 0

# In-scope check via the shared matcher.
[ "$(pscope_match "$REL" "$SCOPED")" = "in" ] && finish 0

# Out of scope.
if [ "$MODE" = "ask" ]; then
  REASON="File ${REL} is outside the scope of ${CHANGE} (scope: $(printf '%s' "$SCOPED" | jq -r 'join(", ")')). Approve to expand scope, or deny to stay in scope."
  if OUT="$(jq -cn --arg r "$REASON" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"ask",permissionDecisionReason:$r}}' 2>/dev/null)"; then
    printf '%s\n' "$OUT"
    finish 0
  fi
  # JSON construction failed → hard fallback.
  echo "[scope-guard] $REASON" >&2
  finish 2
fi

# warn mode (default): non-blocking notice.
echo "[scope-guard] NOTICE: ${REL} is outside the declared scope of ${CHANGE}. (warn mode — not blocked; set PROMETHEUS_SCOPE_ENFORCE=ask to require approval)" >&2
finish 0
