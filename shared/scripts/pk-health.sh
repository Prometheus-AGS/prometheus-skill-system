#!/usr/bin/env bash
# pk-health.sh — SessionStart hook: surface prometheus-knowledge (pk) health
# once per day so a stale or unsynced knowledge base is visible. Throttled to
# one run per 24h. No-op when pk is absent. Always exits 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SessionStart" "pk-health.sh"
finish() { hook_log_end 0; exit 0; }

command -v pk >/dev/null 2>&1 || finish

LAST_RUN="${HOME}/.prometheus/pk-health-last-run"
mkdir -p "$(dirname "$LAST_RUN")" 2>/dev/null || true

# 24h throttle (86400s). Skip when the marker is newer than that.
if [ -f "$LAST_RUN" ]; then
  now="$(date -u +%s 2>/dev/null || echo 0)"
  then_ts="$(cat "$LAST_RUN" 2>/dev/null || echo 0)"
  if [ "$now" != "0" ] && [ "$then_ts" != "0" ] && [ "$((now - then_ts))" -lt 86400 ]; then
    finish
  fi
fi
date -u +%s > "$LAST_RUN" 2>/dev/null || true

# Run the read-only lint command. Its exit status is authoritative; an empty or
# failed invocation must never be translated into a successful health report.
LINT_OUTPUT="$(pk lint 2>&1)"
LINT_STATUS=$?
SUMMARY="$(printf '%s\n' "$LINT_OUTPUT" | tail -1)"

if [ "$LINT_STATUS" -ne 0 ]; then
  printf 'pk health: FAIL (pk lint exited %s)%s\n' "$LINT_STATUS" \
    "$([ -n "$SUMMARY" ] && printf ': %s' "$SUMMARY")"
elif [ -n "$SUMMARY" ]; then
  printf 'pk health: %s\n' "$SUMMARY"
else
  printf 'pk health: WARN (pk lint produced no diagnostic output)\n'
fi
finish
