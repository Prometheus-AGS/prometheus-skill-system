#!/usr/bin/env bash
# mem0-compress: scheduled compression of prometheus-skill-pack mem0 memories
# Invoked by launchd (macOS) or cron (Linux) weekly.
# Reads: MCP surreal-memory compress_memories tool (via pk-cli bridge or curl)
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
  log "pk memory compress failed — falling back to MCP HTTP"
fi

# Fallback: direct HTTP call to surreal-memory MCP if running locally
SURREAL_MCP_URL="${SURREAL_MEMORY_MCP_URL:-http://127.0.0.1:3001}"
PAYLOAD=$(printf '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"compress_memories","arguments":{"user_id":"%s"}}}' "${SCOPE}")

HTTP_STATUS=$(curl -s -o /tmp/mem0-compress-response.json -w "%{http_code}" \
  -X POST "${SURREAL_MCP_URL}" \
  -H "Content-Type: application/json" \
  -d "${PAYLOAD}" 2>>"$LOG_FILE" || echo "000")

if [[ "${HTTP_STATUS}" == "200" ]]; then
  RESULT=$(python3 -c "import json,sys; d=json.load(open('/tmp/mem0-compress-response.json')); print(d.get('result',''))" 2>/dev/null || echo "")
  log "Completed mem0 compress via MCP HTTP. Result: ${RESULT}"
else
  log "WARN: mem0 compress MCP call returned HTTP ${HTTP_STATUS}. Surreal-memory may not be running."
  exit 0  # Non-fatal: memory compression is best-effort
fi
