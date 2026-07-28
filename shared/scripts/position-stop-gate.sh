#!/usr/bin/env bash
# position-stop-gate.sh — Stop hook: record a missing position footer as an
# advisory without ever overriding an operator's request to stop.
# Contract: reads Stop JSON from stdin and always permits the stop. Missing
# footers are written to a user-local advisory log at most once per
# session/state revision. Every failure path exits 0 and prints nothing.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "Stop" "position-stop-gate.sh"

finish() { hook_log_end 0; exit 0; }

command -v python3 >/dev/null 2>&1 || finish
command -v jq >/dev/null 2>&1 || finish

INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || finish

STOP_ACTIVE="$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false' 2>/dev/null || echo false)"
[ "$STOP_ACTIVE" = "true" ] && finish

TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null || true)"
SESSION="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null || echo unknown)"
[ -n "$TRANSCRIPT" ] && [ -f "$TRANSCRIPT" ] || finish

# --- Only gate inside an orchestrator project with a non-terminal waypoint ---
RENDER_LIB="$(cd "$(dirname "$0")" && pwd)/lib/waypoint-render.sh"
[ -f "$RENDER_LIB" ] || finish
# shellcheck source=/dev/null
source "$RENDER_LIB"

ROOT="$(_wr_find_root)" || finish
# Emergency operator authority is unconditional. This check intentionally
# happens before parsing any waypoint so it still works with malformed state.
[ -e "$ROOT/.kbd-orchestrator/PAUSE" ] && finish

WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
[ -f "$WP" ] || finish
STATUS="$(jq -r '.status // .stage // empty' "$WP" 2>/dev/null || true)"
# Stop gating once the phase is terminal (any "done" vocabulary — see
# _wr_is_terminal_status in waypoint-render.sh). Gating a completed phase is
# what turned a stale waypoint into a perpetual /kbd-execute nag.
_wr_is_terminal_status "$STATUS" && finish
_wr_is_suspended_status "$STATUS" && finish

# --- Stable advisory cap. Transcript size is deliberately excluded because it
# grows after each response and therefore cannot bound retries. Prefer an
# explicit runtime revision; legacy waypoints fall back to a digest of the
# fields that define the active cursor. ---
CAP_FILE="${HOME}/.prometheus/position-stop-advisories.txt"
ADVISORY_LOG="${HOME}/.prometheus/position-stop-advisories.log"
STATE_REVISION="$(jq -r '
  .revision // .stateRevision // .state_revision //
  ([.phase // "", .status // .stage // "", .change // .active_change // "",
    .exactNextCommand // .exact_next_command // ""] | @tsv)
' "$WP" 2>/dev/null || true)"
[ -n "$STATE_REVISION" ] || finish
STATE_FPRINT="$(printf '%s' "$STATE_REVISION" | cksum 2>/dev/null | awk '{print $1}')"
[ -n "$STATE_FPRINT" ] || finish
CAP_KEY="${SESSION}:${STATE_FPRINT}:position-footer"
if [ -f "$CAP_FILE" ] && grep -qF "$CAP_KEY" "$CAP_FILE" 2>/dev/null; then
  finish
fi

# --- Extract the last assistant message text from the JSONL transcript ---
LAST_TEXT="$(python3 - "$TRANSCRIPT" <<'PY' 2>/dev/null || true
import json, sys
last = ""
with open(sys.argv[1], encoding="utf-8", errors="replace") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if entry.get("type") != "assistant":
            continue
        msg = entry.get("message") or {}
        content = msg.get("content")
        parts = []
        if isinstance(content, str):
            parts.append(content)
        elif isinstance(content, list):
            for block in content:
                if isinstance(block, dict) and block.get("type") == "text":
                    parts.append(block.get("text", ""))
        if parts:
            last = "\n".join(parts)
print(last)
PY
)"
[ -n "$LAST_TEXT" ] || finish

HAS_FOOTER=false
if printf '%s' "$LAST_TEXT" | grep -qE 'prometheus-position|^Position:' ; then
  HAS_FOOTER=true
fi

if [ "$HAS_FOOTER" = "true" ]; then
  finish
fi

FOOTER="$(waypoint_render 2>/dev/null || true)"
[ -n "$FOOTER" ] || finish

mkdir -p "$(dirname "$CAP_FILE")" 2>/dev/null || finish
printf '%s\n' "$CAP_KEY" >> "$CAP_FILE" 2>/dev/null || true
{
  printf 'session=%s state=%s status=%s\n' "$SESSION" "$STATE_FPRINT" "$STATUS"
  printf '%s\n' "$FOOTER"
} >> "$ADVISORY_LOG" 2>/dev/null || true
finish
