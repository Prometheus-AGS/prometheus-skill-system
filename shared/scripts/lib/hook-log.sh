#!/usr/bin/env bash
# hook-log.sh — JSONL observability library for prometheus hook scripts.
# Source this file at the top of every hook script:
#   HOOK_LOG_LIB="$(dirname "$0")/lib/hook-log.sh"
#   [ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
#
# Functions exposed:
#   hook_log_start  <hook-type> <script-name>
#   hook_log_end    <exit-code> [duration-ms]
#   hook_log_error  <lineno>
#
# Output goes to ~/.prometheus/hooks.log in JSONL format.
# Concurrent writes are serialized via flock(1).
# All functions are no-ops when the log directory is not writable.

_HOOK_LOG_DIR="${HOME}/.prometheus"
_HOOK_LOG_FILE="${_HOOK_LOG_DIR}/hooks.log"
_HOOK_LOG_LOCK="${_HOOK_LOG_DIR}/.hooks.lock"

# Internal state populated by hook_log_start
_HOOK_LOG_TYPE=""
_HOOK_LOG_SCRIPT=""
_HOOK_LOG_PID="$$"
_HOOK_LOG_START_MS=""

# Derive session ID from environment (Claude Code sets CLAUDE_SESSION_ID or falls back to pid)
_hook_session_id() {
  if [ -n "${CLAUDE_SESSION_ID:-}" ]; then
    printf '%s' "$CLAUDE_SESSION_ID"
  elif [ -n "${DEEP_SESSION_ID:-}" ]; then
    printf '%s' "$DEEP_SESSION_ID"
  else
    printf 'pid-%s' "$$"
  fi
}

# Milliseconds since epoch — portable across macOS (date) and Linux
_hook_now_ms() {
  if command -v python3 &>/dev/null; then
    python3 -c "import time; print(int(time.time() * 1000))" 2>/dev/null || echo "0"
  else
    echo "0"
  fi
}

# ISO-8601 UTC timestamp
_hook_ts() {
  date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "unknown"
}

# Ensure log directory exists; return non-zero if unusable
_hook_log_ensure_dir() {
  mkdir -p "$_HOOK_LOG_DIR" 2>/dev/null || return 1
  touch "$_HOOK_LOG_LOCK" 2>/dev/null || return 1
  return 0
}

# Write a single JSONL line atomically via flock
_hook_log_write() {
  local line="$1"
  _hook_log_ensure_dir || return 0
  if command -v flock &>/dev/null; then
    (
      flock -x 200
      printf '%s\n' "$line" >> "$_HOOK_LOG_FILE"
    ) 200>"$_HOOK_LOG_LOCK" 2>/dev/null || true
  else
    printf '%s\n' "$line" >> "$_HOOK_LOG_FILE" 2>/dev/null || true
  fi
}

# hook_log_start <hook-type> <script-name>
# Must be called once at the top of a hook script, after sourcing this lib.
hook_log_start() {
  _HOOK_LOG_TYPE="${1:-unknown}"
  _HOOK_LOG_SCRIPT="${2:-unknown}"
  _HOOK_LOG_PID="$$"
  _HOOK_LOG_START_MS="$(_hook_now_ms)"
  local ts session
  ts="$(_hook_ts)"
  session="$(_hook_session_id)"
  _hook_log_write "{\"ts\":\"${ts}\",\"hook\":\"${_HOOK_LOG_TYPE}\",\"script\":\"${_HOOK_LOG_SCRIPT}\",\"pid\":${_HOOK_LOG_PID},\"session_id\":\"${session}\",\"event\":\"start\"}"
}

# hook_log_end <exit-code> [duration-ms]
# Call immediately before exiting the hook script.
hook_log_end() {
  local exit_code="${1:-0}"
  local now_ms duration_ms
  now_ms="$(_hook_now_ms)"
  if [ -n "$_HOOK_LOG_START_MS" ] && [ "$_HOOK_LOG_START_MS" != "0" ] && [ "$now_ms" != "0" ]; then
    duration_ms=$(( now_ms - _HOOK_LOG_START_MS ))
  else
    duration_ms="${2:-0}"
  fi
  local ts session
  ts="$(_hook_ts)"
  session="$(_hook_session_id)"
  _hook_log_write "{\"ts\":\"${ts}\",\"hook\":\"${_HOOK_LOG_TYPE}\",\"script\":\"${_HOOK_LOG_SCRIPT}\",\"pid\":${_HOOK_LOG_PID},\"session_id\":\"${session}\",\"event\":\"end\",\"exit_code\":${exit_code},\"duration_ms\":${duration_ms}}"
}

# hook_log_error <lineno>
# Call in place of || true when a command fails inside a hook script.
# Captures the last line of stderr if available.
hook_log_error() {
  local lineno="${1:-0}"
  local ts session
  ts="$(_hook_ts)"
  session="$(_hook_session_id)"
  _hook_log_write "{\"ts\":\"${ts}\",\"hook\":\"${_HOOK_LOG_TYPE}\",\"script\":\"${_HOOK_LOG_SCRIPT}\",\"pid\":${_HOOK_LOG_PID},\"session_id\":\"${session}\",\"event\":\"error\",\"lineno\":${lineno}}"
}
