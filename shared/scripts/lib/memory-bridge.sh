#!/usr/bin/env bash
# memory-bridge.sh — write-side bridge to surreal-memory with a mandatory
# durable outbox fallback. Source this file; no import side effects.
#
#   source "$(dirname "$0")/lib/memory-bridge.sh"
#   mem_add_memory "learned X" "$(mem_scope_for "$content")"
#
# CONTRACT: every function returns 0. Writes are atomically queued under the
# central Prometheus learning queue and delivered by the supervised worker.
# Hook latency therefore never depends on the memory service.
#
# Hooks never probe or call the memory service. They perform one bounded local
# write; the supervised worker reconciles the caller-supplied operation id with
# Surreal Memory's durable v2 receipt API outside hook latency.

MEM_PROJECT="${PROMETHEUS_PROJECT_ID:-prometheus-skill-pack}"

_mem_operation_id() { # <method> <canonical-arguments-json>
  printf '%s\0%s' "$1" "$2" | shasum -a 256 | awk '{print $1}'
}

# Persist a deferred operation to the central outbox (never fails the caller).
_mem_outbox_write() { # <method> <arguments-json> [dependencies-json]
  local method="$1" args="$2" dependencies="${3:-[]}" root pending now operation_id payload_hash target temporary
  root="${PROMETHEUS_LEARNING_QUEUE:-${HOME}/.prometheus/learning-queue}"
  pending="$root/memory/pending"
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo unknown)"
  mkdir -p "$pending" 2>/dev/null || return 0
  command -v jq >/dev/null 2>&1 || return 0
  command -v shasum >/dev/null 2>&1 || return 0
  args="$(printf '%s' "$args" | jq -cS . 2>/dev/null)" || return 0
  operation_id="$(_mem_operation_id "$method" "$args")"
  payload_hash="$(printf '%s' "$args" | shasum -a 256 | awk '{print $1}')"
  target="$pending/$operation_id.json"
  [ -e "$target" ] && return 0
  temporary="$pending/.$operation_id.$$.tmp"
  jq -cn --arg id "$operation_id" --arg method "$method" --argjson arguments "$args" \
    --argjson dependencies "$dependencies" --arg payloadHash "$payload_hash" --arg queued "$now" \
    '{schemaVersion:2,operationId:$id,method:$method,arguments:$arguments,dependencies:$dependencies,payloadHash:$payloadHash,state:"pending",queuedAt:$queued,lastError:null,receipt:null}' \
    > "$temporary" 2>/dev/null || return 0
  mv "$temporary" "$target" 2>/dev/null || true
  return 0
}

# Choose the memory scope for a piece of content. Content that carries a
# [GLOBAL] marker anywhere (a cross-project learning) is scoped global; the rest
# is project-scoped. A caller that wants strict line-level routing should split
# the content and call this per line.
mem_scope_for() { # <content>
  case "$1" in
    *"[GLOBAL]"*) printf 'global' ;;
    *) printf '%s' "$MEM_PROJECT" ;;
  esac
}

# mem_add_memory <content> [user_id]
mem_add_memory() {
  local content="$1" user_id="${2:-$(mem_scope_for "$1")}"
  local args
  args="$(python3 -c '
import sys, json
print(json.dumps({"content": sys.argv[1], "user_id": sys.argv[2]}))
' "$content" "$user_id" 2>/dev/null)" || args=""
  if [ -z "$args" ]; then _mem_outbox_write add_memory '{}'; return 0; fi
  _mem_outbox_write add_memory "$args"
  return 0
}

# mem_create_task_stream <name>
mem_create_task_stream() {
  local args; args="$(python3 -c 'import sys,json; print(json.dumps({"name": sys.argv[1]},separators=(",",":"),sort_keys=True))' "$1" 2>/dev/null)" || args='{}'
  _mem_outbox_write create_task_stream "$args"
  return 0
}

# mem_add_task_step <stream> <description>
mem_add_task_step() {
  local args stream_args dependency
  args="$(python3 -c 'import sys,json; print(json.dumps({"stream_name":sys.argv[1],"ordinal":1,"name":sys.argv[2],"description":sys.argv[2],"idempotency_key":sys.argv[2],"agent_id":None,"user_id":None},separators=(",",":"),sort_keys=True))' "$1" "$2" 2>/dev/null)" || args='{}'
  stream_args="$(python3 -c 'import sys,json; print(json.dumps({"name":sys.argv[1]},separators=(",",":"),sort_keys=True))' "$1" 2>/dev/null)" || stream_args='{}'
  dependency="$(_mem_operation_id create_task_stream "$stream_args")"
  _mem_outbox_write add_task_step "$args" "[\"$dependency\"]"
  return 0
}

# mem_complete_step <stream> <step>
mem_complete_step() {
  local args step_args dependency
  args="$(python3 -c 'import sys,json; print(json.dumps({"idempotency_key":sys.argv[1],"result":"completed via memory bridge"},separators=(",",":"),sort_keys=True))' "$2" 2>/dev/null)" || args='{}'
  step_args="$(python3 -c 'import sys,json; print(json.dumps({"stream_name":sys.argv[1],"ordinal":1,"name":sys.argv[2],"description":sys.argv[2],"idempotency_key":sys.argv[2],"agent_id":None,"user_id":None},separators=(",",":"),sort_keys=True))' "$1" "$2" 2>/dev/null)" || step_args='{}'
  dependency="$(_mem_operation_id add_task_step "$step_args")"
  _mem_outbox_write complete_step "$args" "[\"$dependency\"]"
  return 0
}
