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
# TRANSPORT NOTE (verified 2026-06-12): the default surreal-memory server speaks
# the two-connection SSE MCP transport — GET /mcp/sse opens a stream that emits a
# sessionId, and tool calls go to POST /mcp/messages?sessionId=<id>. A
# fire-and-forget bash POST to /mcp/sse therefore returns 405 and falls to the
# outbox. This is BY DESIGN for hooks: the bash bridge buffers, and the AGENT
# (Claude Code et al.) — which holds the real mcp__surreal-memory__* tools and a
# live session — is the path that actually writes to the server (and can drain
# the outbox). `mem_available` correctly reports server health via /health; a
# `true` there means the agent's MCP tools can reach it, not that this bash
# POST can. A future enhancement could implement the SSE session handshake here
# (see sycophancy.sh's FIFO initialize→notify→call pattern for a stdio analog).
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
#
# The server serves its health endpoint at the HOST ROOT (/health), not under
# the MCP path. MEM_URL is the MCP SSE URL (e.g. http://localhost:23001/mcp/sse),
# so derive scheme://host:port and probe /health there. A HEAD probe accepting
# any non-000 response is the same detect-first semantics the repo's
# check_running_service helper uses — a listening server proves availability.
mem_available() {
  command -v curl >/dev/null 2>&1 || return 1
  # Strip any path component to get scheme://host:port.
  local base="$MEM_URL"
  base="$(printf '%s' "$base" | sed -E 's#^(https?://[^/]+).*#\1#')"
  curl -fsS --max-time 2 "${base}/health" >/dev/null 2>&1 && return 0
  return 1
}

# Derive scheme://host:port from MEM_URL (which is the MCP SSE URL). The REST
# API lives at <base>/api/v1/... on the same host:port.
_mem_base() {
  printf '%s' "$MEM_URL" | sed -E 's#^(https?://[^/]+).*#\1#'
}

# Map an MCP tool name to its REST route, or empty when the tool has no REST
# route (task-streams, compress, etc. are MCP-tool-only — callers must outbox
# those). The server exposes only a subset of its 10 MCP tools as REST routes
# (see tools/surreal-memory-server/src/contracts.rs).
_mem_rest_route() { # <tool-name> → "<METHOD> <path>" or empty
  case "$1" in
    add_memory)      printf 'POST /api/v1/memory' ;;
    create_entity)   printf 'POST /api/v1/entities' ;;
    create_relation) printf 'POST /api/v1/entities/relations' ;;
    *)               printf '' ;;   # no REST route → caller outboxes
  esac
}

# Low-level write. Dispatches by tool name to the REST API. The request body is
# the tool's arguments JSON as-is (the REST structs match the MCP tool args for
# the routed tools). Echoes nothing; returns 0 on HTTP 200/201, else 1. Returns
# 1 for any tool with no REST route so the caller outboxes it.
_mem_call() { # <tool-name> <arguments-json>
  command -v curl >/dev/null 2>&1 || return 1
  local name="$1" args="$2"
  local route; route="$(_mem_rest_route "$name")"
  [ -n "$route" ] || return 1            # MCP-tool-only → outbox
  local method="${route%% *}" path="${route#* }"
  local url; url="$(_mem_base)$path"
  local status
  status="$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 \
    -X "$method" "$url" -H 'Content-Type: application/json' -d "$args" 2>/dev/null || echo 000)"
  case "$status" in 200|201) return 0 ;; *) return 1 ;; esac
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
