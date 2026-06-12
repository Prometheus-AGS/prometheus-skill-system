#!/usr/bin/env bash
# memory-outbox-flush.sh — SessionStart hook: drain the surreal-memory write
# outbox when the endpoint is reachable. Non-destructive on failure (a line that
# fails to send stays in the outbox for the next session). Always exits 0.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SessionStart" "memory-outbox-flush.sh"
# shellcheck source=/dev/null
source "$HERE/lib/memory-bridge.sh"
finish() { hook_log_end 0; exit 0; }
command -v jq >/dev/null 2>&1 || finish
command -v python3 >/dev/null 2>&1 || finish

_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_root)" || finish
OUTBOX="$ROOT/.kbd-orchestrator/memory-outbox.jsonl"
[ -f "$OUTBOX" ] && [ -s "$OUTBOX" ] || finish

# Only attempt a drain if the endpoint is reachable; otherwise leave it intact.
mem_available || {
  echo "[memory-outbox-flush] endpoint unreachable — leaving $(wc -l < "$OUTBOX" | tr -d ' ') queued line(s)" >&2
  finish
}

# Methods with a REST route can be drained by this script; the rest are
# MCP-tool-only (no REST route) — a bash flush can never send them, so retaining
# them would grow the outbox forever. Those are transient telemetry
# (task-streams) → DROPPED with a notice. The durable learning is add_memory,
# which IS REST-drainable. (Keeps `_mem_rest_route` as the single source of truth.)
REMAIN="$(mktemp)"
SENT=0; KEPT=0; DROPPED=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  method="$(printf '%s' "$line" | jq -r '.method // empty' 2>/dev/null)"
  args="$(printf '%s' "$line" | jq -c '.arguments // {}' 2>/dev/null)"
  [ -n "$method" ] || continue
  if [ -z "$(_mem_rest_route "$method")" ]; then
    # No REST route — can't drain via bash; drop (telemetry, not durable).
    DROPPED=$((DROPPED+1)); continue
  fi
  if _mem_call "$method" "$args"; then
    SENT=$((SENT+1))
  else
    # REST route exists but the call failed (transient) — keep for next session.
    printf '%s\n' "$line" >> "$REMAIN"; KEPT=$((KEPT+1))
  fi
done < "$OUTBOX"

if [ "$KEPT" -eq 0 ]; then
  rm -f "$OUTBOX"
else
  mv "$REMAIN" "$OUTBOX"
fi
rm -f "$REMAIN" 2>/dev/null || true
echo "[memory-outbox-flush] sent ${SENT}, kept ${KEPT}, dropped ${DROPPED} (MCP-tool-only)" >&2
finish
