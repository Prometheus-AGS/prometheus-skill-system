#!/usr/bin/env bash
# evaluate-session.sh — SubagentStop[executor] hook
# Extracts patterns from the completed executor run and ingests them into the
# prometheus-knowledge (pk) knowledge base.
# Must always exit 0 — failure here must never block the stop chain.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SubagentStop[executor]" "evaluate-session.sh"

LEARNING_LOG_DIR="${HOME}/.prometheus/learning-log"
mkdir -p "$LEARNING_LOG_DIR" 2>/dev/null || true

# Locate waypoint relative to repo root (graceful if outside a git repo)
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || REPO_ROOT=""
WAYPOINT=""
if [[ -n "$REPO_ROOT" ]]; then
  WAYPOINT="${REPO_ROOT}/.kbd-orchestrator/current-waypoint.json"
fi

if [[ -z "$WAYPOINT" || ! -f "$WAYPOINT" ]]; then
  hook_log_end 0
  exit 0
fi

# Read active change and phase from waypoint
PHASE=$(jq -r '.phase // empty' "$WAYPOINT" 2>/dev/null) || PHASE=""
CHANGE=$(jq -r '.active_change // .last_completed // empty' "$WAYPOINT" 2>/dev/null) || CHANGE=""

if [[ -z "$PHASE" && -z "$CHANGE" ]]; then
  hook_log_end 0
  exit 0
fi

# Collect scoped paths (up to 15) for the learning summary
SCOPE_CSV=$(jq -r '.scoped_paths[]? // empty' "$WAYPOINT" 2>/dev/null \
  | head -15 \
  | tr '\n' ',' \
  | sed 's/,$//') || SCOPE_CSV=""

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || echo "unknown")

SUMMARY="executor session complete | phase: ${PHASE:-unknown} | change: ${CHANGE:-unknown}"
[[ -n "$SCOPE_CSV" ]] && SUMMARY="${SUMMARY} | files: ${SCOPE_CSV}"

PATTERNS_FILE=$(mktemp 2>/dev/null) || {
  hook_log_end 0
  exit 0
}
printf '%s\n' "$SUMMARY" > "$PATTERNS_FILE"

# Ingest into prometheus-knowledge via pk CLI
if command -v pk &>/dev/null; then
  pk ingest < "$PATTERNS_FILE" 2>/dev/null || hook_log_error "$LINENO"
fi

# Ingest via surreal-memory REST API (available regardless of pk)
SM_URL="${SURREAL_MEMORY_URL:-http://localhost:23001}"
if curl -sf "${SM_URL}/health" -o /dev/null 2>/dev/null; then
  curl -s -X POST "${SM_URL}/api/v1/memory" \
    -H "Content-Type: application/json" \
    -d "{\"content\": $(jq -Rs '.' "$PATTERNS_FILE"), \"user_id\": \"prometheus-skill-pack\", \"metadata\": {\"source\": \"evaluate-session\", \"phase\": \"${PHASE:-unknown}\", \"change\": \"${CHANGE:-unknown}\"}}" \
    -o /dev/null 2>/dev/null || hook_log_error "$LINENO"
fi

# Append structured JSONL log entry
LOG_ENTRY="{\"ts\":\"${TIMESTAMP}\",\"phase\":\"${PHASE:-}\",\"change\":\"${CHANGE:-}\",\"scoped_paths\":\"${SCOPE_CSV:-}\",\"pk_available\":$(command -v pk &>/dev/null && echo true || echo false)}"
printf '%s\n' "$LOG_ENTRY" >> "${LEARNING_LOG_DIR}/$(date +%Y-%m-%d 2>/dev/null || echo 'unknown').jsonl" 2>/dev/null || true

rm -f "$PATTERNS_FILE" 2>/dev/null || true

hook_log_end 0
exit 0
