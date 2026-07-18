#!/usr/bin/env bash
# Live Tier-2 UI round trip through render.sh and surface-bridge.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RENDER="$REPO_ROOT/skills/learn/ui-surface/scripts/render.sh"
SURFACE_URL="${SURFACE_BRIDGE_URL:-http://127.0.0.1:7890}"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

curl -fsS --max-time 3 "$SURFACE_URL/health" | jq -e '.status == "ok"' >/dev/null

PROGRESS_OUTPUT="$(bash "$RENDER" --tier tier2_mcp_app --intent-json \
  '{"intent_type":"progress","title":"Learning proof","body":"Running","options":null,"multiselect":false,"request_id":"surface-progress-proof"}')"
echo "$PROGRESS_OUTPUT" | jq -e \
  '.request_id == "surface-progress-proof" and .status == "rendered"' >/dev/null
echo "[PASS] Tier-2 progress intent is accepted by surface-bridge"

bash "$RENDER" --tier tier2_mcp_app --intent-json \
  '{"intent_type":"question","title":"Continue?","body":"Choose","options":["Yes","No"],"multiselect":false,"request_id":"surface-question-proof"}' \
  > "$TMP_ROOT/question-response.json" &
RENDER_PID=$!
sleep 1

curl -fsS --max-time 3 -X POST "$SURFACE_URL/mcp/submit-response" \
  -H 'Content-Type: application/json' \
  -d '{"request_id":"surface-question-proof","response":{"selected":["Yes"]}}' \
  | jq -e '.status == "ready"' >/dev/null
wait "$RENDER_PID"
jq -e \
  '.request_id == "surface-question-proof" and .status == "ready" and .response.selected == ["Yes"]' \
  "$TMP_ROOT/question-response.json" >/dev/null
echo "[PASS] Tier-2 question response completes render/submit/collect round trip"
