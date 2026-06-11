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

REMAIN="$(mktemp)"
SENT=0; KEPT=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  method="$(printf '%s' "$line" | jq -r '.method // empty' 2>/dev/null)"
  args="$(printf '%s' "$line" | jq -c '.arguments // {}' 2>/dev/null)"
  if [ -n "$method" ] && _mem_call "$method" "$args"; then
    SENT=$((SENT+1))
  else
    printf '%s\n' "$line" >> "$REMAIN"; KEPT=$((KEPT+1))
  fi
done < "$OUTBOX"

if [ "$KEPT" -eq 0 ]; then
  rm -f "$OUTBOX"
else
  mv "$REMAIN" "$OUTBOX"
fi
rm -f "$REMAIN" 2>/dev/null || true
echo "[memory-outbox-flush] sent ${SENT}, kept ${KEPT}" >&2
finish
