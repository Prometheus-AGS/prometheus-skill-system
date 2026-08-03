#!/usr/bin/env bash
# SessionStart compatibility hook: migrate the former project JSONL outbox to
# the central supervised learning queue. Normal delivery is performed by the
# learning worker and never blocks session startup.
set -uo pipefail
umask 077

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SessionStart" "memory-outbox-flush.sh"
# shellcheck source=/dev/null
source "$HERE/lib/memory-bridge.sh"
finish() { hook_log_end 0; exit 0; }
command -v jq >/dev/null 2>&1 || finish

find_project_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}

ROOT="$(find_project_root 2>/dev/null || true)"
[ -n "$ROOT" ] || finish
LEGACY="$ROOT/.kbd-orchestrator/memory-outbox.jsonl"
[ -s "$LEGACY" ] || finish

valid=0
invalid=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  method="$(printf '%s' "$line" | jq -r '.method // empty' 2>/dev/null || true)"
  arguments="$(printf '%s' "$line" | jq -c '.arguments // empty' 2>/dev/null || true)"
  if [ -z "$method" ] || [ -z "$arguments" ]; then
    invalid=$((invalid + 1))
    continue
  fi
  _mem_outbox_write "$method" "$arguments"
  valid=$((valid + 1))
done < "$LEGACY"

if [ "$invalid" -eq 0 ] && [ "$valid" -gt 0 ]; then
  migrated="$LEGACY.migrated.$(date -u +%Y%m%dT%H%M%SZ)"
  mv "$LEGACY" "$migrated" 2>/dev/null || true
fi
finish
