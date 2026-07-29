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

PROJECT_ROOT="$(find_project_root 2>/dev/null || true)"
[[ -n "$PROJECT_ROOT" ]] || exit 0

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

# Every remaining event is observational. The adapter records it for the daemon
# and returns success; it never intercepts or blocks a tool call.
queue_deferred_event
exit 0
