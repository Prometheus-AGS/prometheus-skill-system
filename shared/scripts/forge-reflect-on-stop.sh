#!/usr/bin/env bash
# forge-reflect-on-stop.sh — Stop hook: runs forge reflect if forge and an iterations dir exist.
# Must always exit 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "Stop" "forge-reflect-on-stop.sh"

if ! command -v forge &>/dev/null; then
  hook_log_end 0
  exit 0
fi

if [ ! -d ".forge/iterations" ]; then
  hook_log_end 0
  exit 0
fi

# Run forge reflect; errors are non-fatal
forge reflect 2>&1 || hook_log_error "$LINENO"

# If pk is available, ingest the reflection
if command -v pk &>/dev/null; then
  pk ingest 2>&1 || hook_log_error "$LINENO"
fi

hook_log_end 0
exit 0
