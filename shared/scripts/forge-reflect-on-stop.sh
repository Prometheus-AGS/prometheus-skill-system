#!/usr/bin/env bash
# forge-reflect-on-stop.sh — Stop hook: runs forge reflect if forge and an iterations dir exist.
# Falls back to direct pk ingest from last-session-summary.txt when forge is absent.
# Must always exit 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "Stop" "forge-reflect-on-stop.sh"

SUMMARY_FILE="${HOME}/.prometheus/last-session-summary.txt"

if command -v forge &>/dev/null && [ -d ".forge/iterations" ]; then
  # --- Forge path: run reflect then ingest ---
  forge reflect 2>&1 || hook_log_error "$LINENO"

  if command -v pk &>/dev/null; then
    pk ingest 2>&1 || hook_log_error "$LINENO"
  fi
else
  # --- Fallback path: ingest session summary directly into pk ---
  if command -v pk &>/dev/null && [ -f "$SUMMARY_FILE" ]; then
    pk ingest < "$SUMMARY_FILE" 2>/dev/null || hook_log_error "$LINENO"
  fi

  # Also push to surreal-memory REST if reachable
  SM_URL="${SURREAL_MEMORY_URL:-http://localhost:23001}"
  if [ -f "$SUMMARY_FILE" ] && curl -sf "${SM_URL}/health" -o /dev/null 2>/dev/null; then
    CONTENT=$(python3 -c "import sys, json; print(json.dumps(open('${SUMMARY_FILE}').read()))" 2>/dev/null) || CONTENT=""
    if [ -n "$CONTENT" ]; then
      curl -s -X POST "${SM_URL}/api/v1/memory" \
        -H "Content-Type: application/json" \
        -d "{\"content\": ${CONTENT}, \"user_id\": \"prometheus-skill-pack\", \"metadata\": {\"source\": \"forge-reflect-on-stop\"}}" \
        -o /dev/null 2>/dev/null || hook_log_error "$LINENO"
    fi
  fi
fi

hook_log_end 0
exit 0
