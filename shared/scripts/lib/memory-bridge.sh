#!/usr/bin/env bash
# memory-bridge.sh — write-side bridge to surreal-memory with a mandatory
# durable outbox fallback. Source this file; no import side effects.
#
#   source "$(dirname "$0")/lib/memory-bridge.sh"
#   mem_add_memory "learned X" "$PROMETHEUS_PROJECT_ID"
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

_MEM_BRIDGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_MEM_ENQUEUE="$_MEM_BRIDGE_DIR/../enqueue-memory-operation.py"

_mem_outbox_write_verified() { # <method> <arguments-json> [dependencies-json]
  [ -x "$_MEM_ENQUEUE" ] || return 1
  "$_MEM_ENQUEUE" "$1" "$2" "${3:-[]}" 2>/dev/null
}

# Persist a deferred operation to the central outbox (never fails the caller).
_mem_outbox_write() { # <method> <arguments-json> [dependencies-json]
  _mem_outbox_write_verified "$1" "$2" "${3:-[]}" >/dev/null 2>&1 || true
  return 0
}

# mem_add_memory <content> [user_id]
mem_add_memory() {
  local content="$1" user_id="${2:-$MEM_PROJECT}"
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
