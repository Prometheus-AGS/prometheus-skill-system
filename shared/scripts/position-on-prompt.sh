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

BLOCK="$(waypoint_render 2>/dev/null || true)"

if [ -n "$BLOCK" ]; then
  printf '\n%s\n' "$BLOCK"
  printf 'MANDATORY: begin your response with the Position line above (updated to reflect any state you change this turn) and end with a Next: line stating the next step and remaining work.\n'
fi

hook_log_end 0
exit 0
