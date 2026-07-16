#!/usr/bin/env bash
# forge-reflect-on-stop.sh — Stop hook: runs forge reflect if forge and an iterations dir exist.
# Falls back to direct pk ingest from last-session-summary.txt when forge is absent.
# Must always exit 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "Stop" "forge-reflect-on-stop.sh"

SUMMARY_FILE="${HOME}/.prometheus/last-session-summary.txt"

# An "empty session" carried no KBD progress: no phase, no completed/pending
# changes, zero of zero changes. Ingesting these produces noise wiki entries
# ("Empty Session Termination Metadata …") that dilute real knowledge, so the
# fallback ingest paths below skip them. Returns 0 (empty) / 1 (has content).
is_empty_session() {
  local f="$1"
  [ -f "$f" ] || return 0
  grep -qE '^phase:[[:space:]]*unknown$'        "$f" || return 1
  grep -qE '^last_completed:[[:space:]]*none$'  "$f" || return 1
  grep -qE '^next_pending:[[:space:]]*none$'    "$f" || return 1
  grep -qE '^progress:[[:space:]]*0 of 0 '      "$f" || return 1
  return 0
}

if command -v forge &>/dev/null && [ -d ".forge/iterations" ]; then
  # --- Forge path: run reflect then ingest ---
  forge reflect 2>&1 || hook_log_error "$LINENO"

  if command -v pk &>/dev/null; then
    pk ingest 2>&1 || hook_log_error "$LINENO"
  fi
elif is_empty_session "$SUMMARY_FILE"; then
  # --- Empty session: skip all ingest to avoid noise wiki entries ---
  :
else
  # --- Fallback path: ingest session summary directly into pk ---
  if command -v pk &>/dev/null && [ -f "$SUMMARY_FILE" ]; then
    pk ingest < "$SUMMARY_FILE" 2>/dev/null || hook_log_error "$LINENO"
  fi

  # Also push to surreal-memory REST if reachable.
  # Bound both curls: without --connect-timeout/--max-time, a port that is OPEN but
  # unresponsive (surreal-memory mid-startup / hung) makes curl wait the OS default
  # (~2min), which stalls the Stop hook and the whole turn. Refused connections already
  # fail fast; these caps also bound the degraded-service case.
  SM_URL="${SURREAL_MEMORY_URL:-http://localhost:23001}"
  if [ -f "$SUMMARY_FILE" ] && curl -sf --connect-timeout 2 --max-time 4 "${SM_URL}/health" -o /dev/null 2>/dev/null; then
    CONTENT=$(python3 -c "import sys, json; print(json.dumps(open('${SUMMARY_FILE}').read()))" 2>/dev/null) || CONTENT=""
    if [ -n "$CONTENT" ]; then
      curl -s --connect-timeout 2 --max-time 5 -X POST "${SM_URL}/api/v1/memory" \
        -H "Content-Type: application/json" \
        -d "{\"content\": ${CONTENT}, \"user_id\": \"prometheus-skill-pack\", \"metadata\": {\"source\": \"forge-reflect-on-stop\"}}" \
        -o /dev/null 2>/dev/null || hook_log_error "$LINENO"
    fi
  fi
fi

hook_log_end 0
exit 0
