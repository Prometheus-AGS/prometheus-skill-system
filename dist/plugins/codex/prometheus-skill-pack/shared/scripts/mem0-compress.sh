#!/usr/bin/env bash
# mem0-compress: scheduled compression of prometheus-skill-pack mem0 memories
# Invoked by launchd (macOS) or cron (Linux) weekly.
# Reads: pk-cli memory bridge, or a best-effort surreal-memory MCP HTTP call.
# Writes: ~/.prometheus/mem0-compress.log

set -euo pipefail

LOG_FILE="${HOME}/.prometheus/mem0-compress.log"
SCOPE="prometheus-skill-pack"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

mkdir -p "$(dirname "$LOG_FILE")"

log() {
  echo "[${TIMESTAMP}] $*" | tee -a "$LOG_FILE"
}

log "Starting mem0 compress for scope: ${SCOPE}"

# Prefer pk-cli bridge if available
if command -v pk &>/dev/null; then
  if pk memory compress --scope "${SCOPE}" 2>>"$LOG_FILE"; then
    log "Completed mem0 compress via pk-cli"
    exit 0
  fi
  log "pk memory compress failed — falling back to surreal-memory MCP HTTP"
fi

# No `pk` (or it failed). `compress_memories` is an MCP TOOL with NO REST route
# (the server exposes /api/v1/memory|entities|relations|mindmaps as REST, but
# NOT compress — see tools/surreal-memory-server/src/contracts.rs). A
# fire-and-forget HTTP call cannot invoke it: POSTing JSON-RPC to the GET-only
# /mcp/sse stream 405s, and the streamable /mcp/messages route needs a session
# handshake. So there is no shell path here — compression must run via the
# agent's mcp__surreal-memory__compress_memories tool (or `pk memory compress`).
log "NOTE: compress_memories is MCP-tool-only (no REST route) — no shell fallback."
log "      Run compression via the agent (mcp__surreal-memory__compress_memories) or 'pk memory compress'."
exit 0  # Non-fatal: memory compression is best-effort and agent-driven.
