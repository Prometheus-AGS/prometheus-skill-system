#!/usr/bin/env bash
# Thin, bounded KBD adapter shared by all supported harnesses.
set -u
umask 077

EVENT="${1:-status}"
HARNESS="${2:-${PROMETHEUS_HARNESS:-unknown}}"

find_project_root() {
  local cursor="${PWD}"
  while [[ "$cursor" != "/" ]]; do
    if [[ -f "$cursor/.prometheus/project.json" ]]; then
      printf '%s\n' "$cursor"
      return 0
    fi
    cursor="$(dirname "$cursor")"
  done
  return 1
}

deny_mutation() {
  local reason="$1"
  if [[ "$HARNESS" == "claude-code" || "$HARNESS" == "kimi" ]]; then
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' \
      "$(printf '%s' "$reason" | jq -Rs .)"
  else
    printf '%s\n' "$reason" >&2
  fi
  exit 2
}

PROJECT_ROOT="$(find_project_root 2>/dev/null || true)"
[[ -n "$PROJECT_ROOT" ]] || {
  [[ "$EVENT" == "pre_mutation" ]] && deny_mutation "KBD project identity is unavailable."
  exit 0
}

PROJECT_ID="$(sed -n 's/.*"projectId"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
  "$PROJECT_ROOT/.prometheus/project.json" 2>/dev/null | head -n 1)"

runtime_data_root() {
  case "$(uname -s)" in
    Darwin) printf '%s\n' "${HOME}/Library/Application Support" ;;
    *) printf '%s\n' "${XDG_DATA_HOME:-${HOME}/.local/share}" ;;
  esac
}

queue_deferred_event() {
  [[ -n "$PROJECT_ID" ]] || return 0
  local outbox
  outbox="$(runtime_data_root)/prometheus/kbd/projects/$PROJECT_ID/deferred-hooks"
  mkdir -p "$outbox" 2>/dev/null || return 0
  local target
  target="$outbox/$(date -u '+%Y%m%dT%H%M%S').$$.${RANDOM:-0}.json"
  printf '{"schemaVersion":"1","event":%s,"harness":%s,"createdAt":%s}\n' \
    "$(printf '%s' "$EVENT" | jq -Rs . 2>/dev/null || printf '"unknown"')" \
    "$(printf '%s' "$HARNESS" | jq -Rs . 2>/dev/null || printf '"unknown"')" \
    "$(date -u '+\"%Y-%m-%dT%H:%M:%SZ\"')" >"$target" 2>/dev/null || true
}

PAUSE_FILE="$PROJECT_ROOT/.kbd-orchestrator/PAUSE"
if [[ -e "$PAUSE_FILE" ]]; then
  [[ "$EVENT" == "pre_mutation" ]] && deny_mutation "KBD emergency PAUSE is active."
  if [[ "$EVENT" == "session_start" || "$EVENT" == "post_compact" ]]; then
    printf 'KBD REANCHOR: emergency PAUSE is active. Audit before resuming.\n'
  fi
  # Stop/interrupt acknowledgement is unconditional while paused.
  exit 0
fi

if [[ "$EVENT" == "interrupt" ]]; then
  mkdir -p "$PROJECT_ROOT/.kbd-orchestrator"
  {
    printf 'requestedAt=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    printf 'reason=Native %s interrupt\n' "$HARNESS"
    printf 'lifecycle=pause_requested\n'
  } >"$PAUSE_FILE"
  queue_deferred_event
  # Durable acknowledgement is deferred; the local valve already owns safety.
  (prometheus kbd --path "$PROJECT_ROOT" pause --reason "Native $HARNESS interrupt" >/dev/null 2>&1 || true) &
  exit 0
fi

render_reanchor() {
  [[ -n "$PROJECT_ID" ]] || return 0
  command -v curl >/dev/null 2>&1 || return 0
  command -v jq >/dev/null 2>&1 || return 0
  local data_root token_file token endpoint status reanchor
  data_root="$(runtime_data_root)"
  token_file="${PROMETHEUS_CONTROL_TOKEN_FILE:-$data_root/prometheus/kbd/projects/$PROJECT_ID/control-token}"
  [[ -f "$token_file" ]] || return 0
  token="$(tr -d '\r\n' <"$token_file")"
  endpoint="${PROMETHEUS_CONTROL_ENDPOINT:-http://127.0.0.1:7892}"
  status="$(curl --silent --show-error --fail --max-time 0.18 \
    -H "Authorization: Bearer $token" \
    "$endpoint/api/v1/kbd/projects/$PROJECT_ID/status" 2>/dev/null || true)"
  [[ -n "$status" ]] || return 0
  reanchor="$(printf '%s' "$status" | jq -r '
    "KBD REANCHOR (committed revision \(.revision), plan \(.planRevision))\n" +
    "Lifecycle: \(.lifecycle)\n" +
    "Active path: " +
      ([.activePath.phaseId, .activePath.stageId, .activePath.changeId, .activePath.taskId]
       | map(select(. != null and . != "")) | join(" → ")) + "\n" +
    "Exact next work: \(.exactNextWork // "not recorded")\n" +
    (if .lease then
       "Writer: \(.lease.owner.harness) on \(.lease.owner.device), fence \(.lease.fencingToken)"
     else "Writer: unclaimed" end)
  ' 2>/dev/null || true)"
  # The renderer is deliberately far below the 1,200-token contract. The
  # 4,800-character hard ceiling remains as a deterministic final guard.
  printf '%s\n' "${reanchor:0:4800}"
}

if [[ "$EVENT" == "session_start" || "$EVENT" == "post_compact" ]]; then
  render_reanchor
  queue_deferred_event
  exit 0
fi

if [[ "$EVENT" != "pre_mutation" ]]; then
  # Noncritical summaries, learning, and memory work are consumed by a daemon.
  queue_deferred_event
  exit 0
fi

command -v jq >/dev/null 2>&1 || deny_mutation "jq is required for the KBD mutation guard."
command -v curl >/dev/null 2>&1 || deny_mutation "curl is required for the KBD mutation guard."

[[ -n "$PROJECT_ID" ]] || deny_mutation "KBD projectId is unavailable."

DATA_ROOT="$(runtime_data_root)"
TOKEN_FILE="${PROMETHEUS_CONTROL_TOKEN_FILE:-$DATA_ROOT/prometheus/kbd/projects/$PROJECT_ID/control-token}"
[[ -f "$TOKEN_FILE" ]] || deny_mutation "KBD control token is unavailable."
TOKEN="$(tr -d '\r\n' <"$TOKEN_FILE")"
ENDPOINT="${PROMETHEUS_CONTROL_ENDPOINT:-http://127.0.0.1:7892}"
STATUS="$(curl --silent --show-error --fail --max-time 0.18 \
  -H "Authorization: Bearer $TOKEN" \
  "$ENDPOINT/api/v1/kbd/projects/$PROJECT_ID/status" 2>/dev/null || true)"
[[ -n "$STATUS" ]] || deny_mutation "KBD fence check could not reach the local control plane."

LIFECYCLE="$(printf '%s' "$STATUS" | jq -r '.lifecycle // empty' 2>/dev/null)"
case "$LIFECYCLE" in
  pause_requested|paused|blocked|completed|cancelled|failed)
    deny_mutation "KBD lifecycle is $LIFECYCLE; mutating tools are disabled."
    ;;
esac

OWNER="$(printf '%s' "$STATUS" | jq -r '.lease.owner.harness // empty' 2>/dev/null)"
FENCE="$(printf '%s' "$STATUS" | jq -r '.lease.fencingToken // empty' 2>/dev/null)"
[[ "$OWNER" == "$HARNESS" && -n "$FENCE" ]] ||
  deny_mutation "This harness does not hold the current KBD mutation lease."
exit 0
