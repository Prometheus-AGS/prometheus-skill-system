#!/usr/bin/env bash
# Canonical cross-harness Karpathy hook dispatcher.
#
# Prompt events synchronously return bounded local context. Stop and executor
# events atomically enqueue a metadata-only learning job for the supervised
# worker. Every path is fail-open to the calling harness.
set -uo pipefail
umask 077

EVENT="${1:-unknown}"
HARNESS="${2:-${PROMETHEUS_HARNESS:-unknown}}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENQUEUE="$SCRIPT_DIR/enqueue-learning-job.py"

# Stop and executor completion do exactly one local, durable enqueue. Keep
# logging, inference, memory, and service-manager work out of this path.
case "$EVENT" in
  stop|Stop|session.idle|executor_complete|SubagentStop:executor)
    [ -x "$ENQUEUE" ] && "$ENQUEUE" "$EVENT" "$HARNESS" 2>/dev/null || true
    exit 0
    ;;
esac

command -v jq >/dev/null 2>&1 || exit 0
INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || INPUT='{}'

if [ -z "$EVENT" ] || [ "$EVENT" = "auto" ]; then
  EVENT="$(printf '%s' "$INPUT" | jq -r \
    '.hook_event_name // .event_name // .event // .type // "unknown"' 2>/dev/null | head -n 1)"
fi

case "$EVENT" in
  stop|Stop|session.idle|executor_complete|SubagentStop:executor)
    printf '%s' "$INPUT" | "$ENQUEUE" "$EVENT" "$HARNESS" 2>/dev/null || true
    exit 0
    ;;
esac

HOOK_LOG_LIB="$SCRIPT_DIR/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "$EVENT" "karpathy-hook-dispatch.sh"

finish() {
  hook_log_end 0
  exit 0
}

json_value() {
  local expression="$1"
  printf '%s' "$INPUT" | jq -r "$expression // empty" 2>/dev/null | head -n 1
}

case "$EVENT" in
  prompt|user_prompt_submit|UserPromptSubmit|chat.message)
    prompt="$(json_value '.prompt // .message // .content // .text')"
    [ -n "$prompt" ] || finish
    if command -v pk >/dev/null 2>&1; then
      if ! RUST_LOG=error pk context "$prompt" \
        --scope project --scope shared --scope global \
        --limit "${PROMETHEUS_CONTEXT_LIMIT:-8}" \
        --max-candidates "${PROMETHEUS_CONTEXT_MAX_CANDIDATES:-128}" \
        --max-bytes "${PROMETHEUS_CONTEXT_MAX_BYTES:-6000}" \
        --format hook 2>/dev/null; then
        printf '[prometheus-context] status=partial reason=pk-query-failed\n' >&2
      fi
    else
      printf '[prometheus-context] status=unavailable reason=pk-not-found\n' >&2
    fi
    finish
    ;;

  *) finish ;;
esac
