#!/usr/bin/env bash
# position-stop-gate.sh — Stop hook: if the final assistant message lacks the
# position footer, block the stop ONCE and hand the model the rendered footer
# to append. Honest ceiling: one enforced retry, never a loop.
# Contract: reads Stop JSON from stdin. Emits {"decision":"block",...} to
# stdout to block; otherwise prints nothing. Every failure path exits 0 so the
# Stop chain is never broken by this gate.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
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
source "$RENDER_LIB"

ROOT="$(_wr_find_root)" || finish
WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
[ -f "$WP" ] || finish
STATUS="$(jq -r '.status // .stage // empty' "$WP" 2>/dev/null || true)"
case "$STATUS" in
  phase_complete|reflect_complete|"") finish ;;
esac

# --- Soft cap: never block the same stop twice (belt-and-suspenders beyond
# stop_hook_active, keyed on session + transcript fingerprint) ---
CAP_FILE="${HOME}/.prometheus/position-stop-block.txt"
FPRINT="$(wc -c < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
CAP_KEY="${SESSION}:${FPRINT}"
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

if printf '%s' "$LAST_TEXT" | grep -qE 'prometheus-position|^Position:' ; then
  finish
fi

FOOTER="$(waypoint_render 2>/dev/null || true)"
[ -n "$FOOTER" ] || finish

mkdir -p "$(dirname "$CAP_FILE")" 2>/dev/null || finish
printf '%s\n' "$CAP_KEY" >> "$CAP_FILE" 2>/dev/null || true

REASON="Final response is missing the mandatory position footer. Append the current position block exactly, updated for any state changed this turn:

$FOOTER"

jq -cn --arg reason "$REASON" '{"decision":"block","reason":$reason}'
hook_log_end 0
exit 0
