#!/usr/bin/env bash
# position-on-prompt.sh — UserPromptSubmit hook: injects the KBD position block
# into context every turn so the model (and therefore the user) always sees the
# current place in the order of operations.
# Contract: reads JSON from stdin (unused), prints the block to stdout, or
# nothing on the degraded path. Must always exit 0 — context injection must
# never block a prompt.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "UserPromptSubmit" "position-on-prompt.sh"

# Drain stdin so the harness never blocks on an unread pipe.
cat > /dev/null 2>&1 || true

RENDER_LIB="$(cd "$(dirname "$0")" && pwd)/lib/waypoint-render.sh"
if [ ! -f "$RENDER_LIB" ]; then
  hook_log_end 0
  exit 0
fi
source "$RENDER_LIB"

ROOT="$(_wr_find_root 2>/dev/null || true)"
STATUS=""
SUSPENDED=false
if [ -n "$ROOT" ]; then
  if [ -e "$ROOT/.kbd-orchestrator/PAUSE" ]; then
    SUSPENDED=true
  elif [ -f "$ROOT/.kbd-orchestrator/current-waypoint.json" ] && command -v jq >/dev/null 2>&1; then
    STATUS="$(jq -r '.status // .stage // empty' "$ROOT/.kbd-orchestrator/current-waypoint.json" 2>/dev/null || true)"
    _wr_is_suspended_status "$STATUS" && SUSPENDED=true
  fi
fi

BLOCK="$(waypoint_render 2>/dev/null || true)"

if [ -n "$BLOCK" ]; then
  printf '\n%s\n' "$BLOCK"
  if [ "$SUSPENDED" = "true" ]; then
    printf 'PAUSE ADVISORY: KBD records a pause request. Confirm intent before advancing planned work; tools remain available and journal transactions govern writes.\n'
  else
    printf 'POSITION ADVISORY: report meaningful state changes and the remaining next step, but never override an operator request to stop, pause, audit, or change course.\n'
  fi
fi

hook_log_end 0
exit 0
