#!/usr/bin/env bash
# memory-bridge.sh — write-side bridge to surreal-memory with a mandatory
# durable outbox fallback. Source this file; no import side effects.
#
#   source "$(dirname "$0")/lib/memory-bridge.sh"
#   mem_add_memory "learned X" "$(mem_scope_for "$content")"
#
# CONTRACT: every function returns 0. When the endpoint is unreachable, curl is
# missing, or the call fails, the intended operation is appended as one JSON
# line to .kbd-orchestrator/memory-outbox.jsonl and the function still returns 0.
# Memory writes must never block the caller — the endpoint was observed timing
# out during this project's own assessment.
#
# Endpoint: $SURREAL_MEMORY_URL (default http://localhost:23001/mcp/sse).
# Project scope: $KBD_PROJECT_NAME (default prometheus-skill-pack).

MEM_URL="${SURREAL_MEMORY_URL:-http://localhost:23001/mcp/sse}"
MEM_PROJECT="${KBD_PROJECT_NAME:-prometheus-skill-pack}"

# Locate the orchestrator root for the outbox; fall back to $PWD.
_mem_outbox() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s/.kbd-orchestrator/memory-outbox.jsonl' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  printf '%s/.kbd-orchestrator/memory-outbox.jsonl' "$PWD"
}

# Append a deferred operation to the outbox (never fails the caller).
_mem_outbox_write() { # <method> <arguments-json>
  local method="$1" args="$2" outbox now
  outbox="$(_mem_outbox)"
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo unknown)"
  mkdir -p "$(dirname "$outbox")" 2>/dev/null || return 0
  if command -v jq >/dev/null 2>&1; then
    jq -cn --arg m "$method" --argjson a "$args" --arg t "$now" \
      '{queuedAt:$t, method:$m, arguments:$a}' >> "$outbox" 2>/dev/null || true
  else
    printf '{"queuedAt":"%s","method":"%s"}\n' "$now" "$method" >> "$outbox" 2>/dev/null || true
  fi
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

# Probe the endpoint. Returns 0 when reachable.
mem_available() {
  command -v curl >/dev/null 2>&1 || return 1
  curl -fsS --max-time 2 "${MEM_URL%/}/healthz" >/dev/null 2>&1 && return 0
  return 1
}

# Low-level JSON-RPC tools/call. Echoes nothing; returns 0 on HTTP 200, else 1.
_mem_call() { # <tool-name> <arguments-json>
  command -v curl >/dev/null 2>&1 || return 1
  command -v python3 >/dev/null 2>&1 || return 1
  local name="$1" args="$2" payload status
  payload="$(python3 -c '
import sys, json
name, args = sys.argv[1], json.loads(sys.argv[2])
print(json.dumps({"jsonrpc":"2.0","id":1,"method":"tools/call",
                  "params":{"name":name,"arguments":args}}))
' "$name" "$args" 2>/dev/null)" || return 1
  [ -n "$payload" ] || return 1
  status="$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 \
    -X POST "$MEM_URL" -H 'Content-Type: application/json' -d "$payload" 2>/dev/null || echo 000)"
  [ "$status" = "200" ]
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
  _mem_call add_memory "$args" || _mem_outbox_write add_memory "$args"
  return 0
}

# mem_create_task_stream <name>
mem_create_task_stream() {
  local args; args="$(python3 -c 'import sys,json; print(json.dumps({"name": sys.argv[1]}))' "$1" 2>/dev/null)" || args='{}'
  _mem_call create_task_stream "$args" || _mem_outbox_write create_task_stream "$args"
  return 0
}

# mem_add_task_step <stream> <description>
mem_add_task_step() {
  local args; args="$(python3 -c 'import sys,json; print(json.dumps({"stream": sys.argv[1], "description": sys.argv[2]}))' "$1" "$2" 2>/dev/null)" || args='{}'
  _mem_call add_task_step "$args" || _mem_outbox_write add_task_step "$args"
  return 0
}

# mem_complete_step <stream> <step>
mem_complete_step() {
  local args; args="$(python3 -c 'import sys,json; print(json.dumps({"stream": sys.argv[1], "step": sys.argv[2]}))' "$1" "$2" 2>/dev/null)" || args='{}'
  _mem_call complete_step "$args" || _mem_outbox_write complete_step "$args"
  return 0
}
