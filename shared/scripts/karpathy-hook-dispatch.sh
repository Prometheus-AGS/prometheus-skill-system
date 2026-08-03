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
HOOK_LOG_LIB="$SCRIPT_DIR/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "$EVENT" "karpathy-hook-dispatch.sh"

finish() {
  hook_log_end 0
  exit 0
}

command -v jq >/dev/null 2>&1 || finish
INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || INPUT='{}'

if [ -z "$EVENT" ] || [ "$EVENT" = "auto" ]; then
  EVENT="$(printf '%s' "$INPUT" | jq -r \
    '.hook_event_name // .event_name // .event // .type // "unknown"' 2>/dev/null | head -n 1)"
fi

json_value() {
  local expression="$1"
  printf '%s' "$INPUT" | jq -r "$expression // empty" 2>/dev/null | head -n 1
}

find_project_root() {
  local requested="$1" cursor
  if [ -n "$requested" ] && [ -d "$requested" ]; then
    cursor="$(cd "$requested" 2>/dev/null && pwd -P)" || cursor=""
  else
    cursor="$PWD"
  fi
  while [ -n "$cursor" ] && [ "$cursor" != "/" ]; do
    if [ -e "$cursor/.git" ] || [ -f "$cursor/.prometheus/project.json" ] \
      || [ -f "$cursor/Cargo.toml" ] || [ -f "$cursor/package.json" ]; then
      printf '%s' "$cursor"
      return 0
    fi
    cursor="$(dirname "$cursor")"
  done
  return 1
}

case "$EVENT" in
  prompt|user_prompt_submit|UserPromptSubmit|chat.message)
    prompt="$(json_value '.prompt // .message // .content // .text')"
    [ -n "$prompt" ] || finish
    if command -v pk >/dev/null 2>&1; then
      if ! RUST_LOG=error pk context "$prompt" \
        --scope project --scope shared --scope global \
        --limit "${PROMETHEUS_CONTEXT_LIMIT:-8}" \
        --timeout-ms "${PROMETHEUS_CONTEXT_TIMEOUT_MS:-2000}" \
        --format hook 2>/dev/null; then
        printf '[prometheus-context] status=partial reason=pk-query-failed\n' >&2
      fi
    else
      printf '[prometheus-context] status=unavailable reason=pk-not-found\n' >&2
    fi
    finish
    ;;

  stop|Stop|session.idle|executor_complete|SubagentStop:executor)
    command -v shasum >/dev/null 2>&1 || finish
    cwd="$(json_value '.cwd // .working_directory // .workingDirectory')"
    project_root="$(find_project_root "$cwd" 2>/dev/null || true)"
    [ -n "$project_root" ] || finish

    session_id="$(json_value '.session_id // .sessionId // .conversation_id // .conversationId')"
    [ -n "$session_id" ] || session_id="${CLAUDE_SESSION_ID:-${CODEX_THREAD_ID:-unknown}}"
    transcript_path="$(json_value '.transcript_path // .transcriptPath')"
    created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || printf unknown)"
    payload_digest="$(printf '%s' "$INPUT" | shasum -a 256 | awk '{print $1}')"
    event_id="$(printf '%s\0%s\0%s\0%s\0%s\0%s' \
      "$HARNESS" "$EVENT" "$session_id" "$project_root" "$transcript_path" "$payload_digest" \
      | shasum -a 256 | awk '{print $1}')"

    queue_root="${PROMETHEUS_LEARNING_QUEUE:-$HOME/.prometheus/learning-queue}"
    pending="$queue_root/pending"
    mkdir -p "$pending" "$queue_root/processing" "$queue_root/completed" \
      "$queue_root/retry" "$queue_root/dead-letter" 2>/dev/null || finish
    target="$pending/$event_id.json"
    [ -e "$target" ] && finish
    temporary="$pending/.$event_id.$$.tmp"
    jq -cn \
      --arg event_id "$event_id" \
      --arg event_type "$EVENT" \
      --arg harness "$HARNESS" \
      --arg session_id "$session_id" \
      --arg project_root "$project_root" \
      --arg transcript_path "$transcript_path" \
      --arg captured_at "$created_at" \
      --arg payload_digest "$payload_digest" \
      '{schemaVersion:2,eventId:$event_id,eventType:$event_type,harness:$harness,
        sessionId:$session_id,projectRoot:$project_root,
        transcriptPath:(if $transcript_path == "" then null else $transcript_path end),
        capturedAt:$captured_at,payloadDigest:$payload_digest,attempt:0}' \
      > "$temporary" 2>/dev/null || finish
    mv "$temporary" "$target" 2>/dev/null || finish

    # launchd's WatchPaths and systemd's path unit wake the worker. Avoid a
    # synchronous service-manager call here so Stop always remains bounded.
    finish
    ;;

  *) finish ;;
esac
