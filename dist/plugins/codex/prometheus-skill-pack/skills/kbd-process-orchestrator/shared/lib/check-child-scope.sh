#!/usr/bin/env bash
# check-child-scope.sh — PreToolUse(Write|Edit|MultiEdit) advisory enforcement
# of a child loop's scope.json. When the waypoint path[] is inside a child
# (depth > 1), writes outside that child's allowedWritePaths are flagged.
#
# This is HOOK-LEVEL (advisory) isolation, not an OS sandbox — the threat model
# is agent drift, not an adversary. Paths are canonicalized (cd && pwd -P) so
# macOS /var vs /private/var symlinks do not defeat matching.
#
# Modes (PROMETHEUS_CHILD_SCOPE_ENFORCE):
#   off  → never acts          warn → non-blocking notice (default)
#   ask  → permissionDecision:"ask" JSON
#
# Always-allowed: .kbd-orchestrator/**, SCRATCHPAD.md.
# Degrades to exit 0 when: not inside a child, no scope.json, off mode.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/../../../../../shared/scripts/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PreToolUse" "check-child-scope.sh" 2>/dev/null || true

# Shared path-scope helper. From
# skills/process/kbd-process-orchestrator/shared/lib the repo root is five
# levels up (lib→shared→kbd-process-orchestrator→process→skills→root).
PSCOPE_LIB="$(cd "$(dirname "$0")" && pwd)/../../../../../shared/scripts/lib/path-scope.sh"
[ -f "$PSCOPE_LIB" ] && source "$PSCOPE_LIB"

MODE="${PROMETHEUS_CHILD_SCOPE_ENFORCE:-warn}"
finish() { command -v hook_log_end >/dev/null 2>&1 && hook_log_end "${1:-0}"; exit "${1:-0}"; }
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

# Locate orchestrator root + waypoint.
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

# Resolve path[] depth; only act when inside a child (depth > 1).
CHAIN="$(jq -r '
  if (.path | type) == "array" and (.path | length) > 0 then .path | join(" ")
  else ([ (.phase // empty), (.childPointer // empty) ]
        | map(select(. != "" and . != null)) | join(" ")) end
' "$WP" 2>/dev/null)"
[ -n "$CHAIN" ] || finish 0
# shellcheck disable=SC2206
TOKENS=($CHAIN); DEPTH="${#TOKENS[@]}"
[ "$DEPTH" -gt 1 ] || finish 0

# Build the child node dir and read its scope.json.
NODE=".kbd-orchestrator/phases/${TOKENS[0]}"
for ((i=1; i<DEPTH; i++)); do NODE="$NODE/children/${TOKENS[$i]}"; done
SCOPE_FILE="$ROOT/$NODE/scope.json"
[ -f "$SCOPE_FILE" ] && jq empty "$SCOPE_FILE" 2>/dev/null || finish 0
ALLOWED="$(jq -c '.allowedWritePaths // []' "$SCOPE_FILE")"
[ "$ALLOWED" = "[]" ] && finish 0

# Relativize via the shared helper (macOS symlink-safe).
REL="$(pscope_relativize "$ROOT" "$FILE_PATH")"

# Always-allowed paths.
pscope_always_allowed "$REL" && finish 0

# The child node dir and anything it owns are always in scope.
case "$REL" in
  "$NODE"|"$NODE"/*) finish 0 ;;
esac

# In-scope check via the shared matcher over allowedWritePaths.
[ "$(pscope_match "$REL" "$ALLOWED")" = "in" ] && finish 0

CHILD_LABEL="$(IFS=' '; echo "${TOKENS[*]}" | sed 's/ / › /g')"
if [ "$MODE" = "ask" ]; then
  REASON="File ${REL} is outside the scope of child loop ${CHILD_LABEL} (allowedWritePaths in ${NODE}/scope.json). Approve to widen the child's scope, or deny to keep the inner loop contained."
  if OUT="$(jq -cn --arg r "$REASON" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"ask",permissionDecisionReason:$r}}' 2>/dev/null)"; then
    printf '%s\n' "$OUT"; finish 0
  fi
  echo "[child-scope] $REASON" >&2; finish 2
fi

# warn (default).
echo "[child-scope] NOTICE: ${REL} is outside the declared scope of child loop ${CHILD_LABEL}. (warn mode — not blocked; set PROMETHEUS_CHILD_SCOPE_ENFORCE=ask to require approval)" >&2
finish 0
