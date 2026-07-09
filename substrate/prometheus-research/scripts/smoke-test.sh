#!/usr/bin/env bash
# smoke-test.sh — full HTTP API lifecycle test for prometheus-research
# Usage: bash substrate/prometheus-research/scripts/smoke-test.sh
# Exit 0 = all checks passed; non-zero = first failure

set -euo pipefail

BASE="http://127.0.0.1:7891"
BINARY="prometheus-research"
SERVER_PID=""
PASS=0
FAIL=0

# ── helpers ──────────────────────────────────────────────────────────────────

ok()   { echo "  ✓ $*"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $*" >&2; FAIL=$((FAIL + 1)); }

check() {
  local desc="$1"; shift
  if "$@" &>/dev/null; then ok "$desc"; else fail "$desc"; fi
}

require_bin() {
  command -v "$1" &>/dev/null || { echo "SKIP: $1 not found — install it and retry"; exit 0; }
}

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
    echo ""
    echo "  [cleanup] server PID $SERVER_PID stopped"
  fi
}
trap cleanup EXIT

# ── pre-flight ────────────────────────────────────────────────────────────────

require_bin curl
require_bin jq

echo ""
echo "══════════════════════════════════════════"
echo "  prometheus-research smoke test"
echo "══════════════════════════════════════════"

# Locate the binary: workspace target root first, then PATH.
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
BINARY_PATH=""
for candidate in \
    "$WORKSPACE_ROOT/target/release/$BINARY" \
    "$HOME/.local/bin/$BINARY" \
    "$(command -v "$BINARY" 2>/dev/null || true)"; do
  if [[ -x "$candidate" ]]; then
    BINARY_PATH="$candidate"
    break
  fi
done

if [[ -z "$BINARY_PATH" ]]; then
  echo ""
  echo "  SKIP: '$BINARY' binary not found."
  echo "  Build it first:  cargo build --release --manifest-path substrate/prometheus-research/Cargo.toml"
  exit 0
fi
echo "  binary: $BINARY_PATH"
echo ""

# ── phase 1: start server ─────────────────────────────────────────────────────

echo "── Phase 1: start server"

# Make sure port 7891 is free.
if curl -sf --max-time 1 "$BASE/health" &>/dev/null; then
  echo "  WARN: port 7891 already in use — testing against running instance"
  SKIP_KILL=1
else
  SKIP_KILL=0
  "$BINARY_PATH" --mode server &>/tmp/prometheus-research-smoke.log &
  SERVER_PID=$!
  echo "  started PID $SERVER_PID"
fi

# Poll /health for up to 10 s.
READY=0
for i in $(seq 1 20); do
  if curl -sf --max-time 1 "$BASE/health" &>/dev/null; then
    READY=1; break
  fi
  sleep 0.5
done

if [[ $READY -eq 1 ]]; then
  ok "/health responded within 10 s"
else
  fail "/health did not respond — server failed to start"
  if [[ -f /tmp/prometheus-research-smoke.log ]]; then
    echo "  server log:"
    tail -20 /tmp/prometheus-research-smoke.log | sed 's/^/    /' >&2
  fi
  exit 1
fi

# Validate /health body.
HEALTH=$(curl -sf --max-time 2 "$BASE/health")
if echo "$HEALTH" | jq -e '.status == "ok"' &>/dev/null; then
  ok "/health body has status:ok"
else
  fail "/health body missing status:ok (got: $HEALTH)"
fi

echo ""

# ── phase 2: POST /api/v1/jobs ────────────────────────────────────────────────

echo "── Phase 2: POST /api/v1/jobs"

JOB_RESP=$(curl -sf --max-time 5 -X POST "$BASE/api/v1/jobs" \
  -H "Content-Type: application/json" \
  -d '{"query":"smoke test — Rust async runtimes","depth":"shallow"}') || {
    fail "POST /api/v1/jobs returned error"
    exit 1
  }

ok "POST /api/v1/jobs returned 200"

JOB_ID=$(echo "$JOB_RESP" | jq -r '.job_id // empty')
if [[ -n "$JOB_ID" && "$JOB_ID" != "null" ]]; then
  ok "response contains job_id: $JOB_ID"
else
  fail "response missing job_id (got: $JOB_RESP)"
  exit 1
fi

echo ""

# ── phase 3: GET /api/v1/jobs/{id} ───────────────────────────────────────────

echo "── Phase 3: GET /api/v1/jobs/$JOB_ID"

STATUS_RESP=$(curl -sf --max-time 5 "$BASE/api/v1/jobs/$JOB_ID") || {
  fail "GET /api/v1/jobs/$JOB_ID returned error"
  exit 1
}

ok "GET /api/v1/jobs/$JOB_ID returned 200"

JOB_STATUS=$(echo "$STATUS_RESP" | jq -r '.status // empty')
if [[ -n "$JOB_STATUS" && "$JOB_STATUS" != "null" ]]; then
  ok "status field present: $JOB_STATUS"
else
  fail "status field missing in response"
fi

echo ""

# ── phase 4: SSE stream — read first event ────────────────────────────────────

echo "── Phase 4: SSE /api/v1/jobs/$JOB_ID/events"

SSE_OUT=$(curl -sf --max-time 3 -N \
  -H "Accept: text/event-stream" \
  "$BASE/api/v1/jobs/$JOB_ID/events" 2>/dev/null | head -5) || true

if [[ -n "$SSE_OUT" ]]; then
  ok "SSE stream returned data"
else
  # Non-fatal: job may complete before the curl opens.
  echo "  INFO: SSE stream was empty or job already completed — non-fatal"
fi

echo ""

# ── phase 5: DELETE /api/v1/jobs/{id} ────────────────────────────────────────

echo "── Phase 5: DELETE /api/v1/jobs/$JOB_ID"

DEL_STATUS=$(curl -so /dev/null -w "%{http_code}" --max-time 5 \
  -X DELETE "$BASE/api/v1/jobs/$JOB_ID")

if [[ "$DEL_STATUS" == "200" || "$DEL_STATUS" == "204" || "$DEL_STATUS" == "202" ]]; then
  ok "DELETE returned $DEL_STATUS"
elif [[ "$DEL_STATUS" == "404" ]]; then
  ok "DELETE returned 404 (job already completed — acceptable)"
else
  fail "DELETE returned unexpected $DEL_STATUS"
fi

echo ""

# ── phase 6: A2UI component endpoints ────────────────────────────────────────

echo "── Phase 6: A2UI component endpoints"

COMPONENTS=(progress-bar source-card citation-list graph-view contradiction-panel stage-timeline confidence-meter export-card)
for comp in "${COMPONENTS[@]}"; do
  HTTP_CODE=$(curl -so /dev/null -w "%{http_code}" --max-time 3 \
    "$BASE/components/$comp?job_id=smoke-test")
  if [[ "$HTTP_CODE" == "200" ]]; then
    ok "/components/$comp → 200"
  else
    fail "/components/$comp → $HTTP_CODE (expected 200)"
  fi
done

echo ""

# ── summary ───────────────────────────────────────────────────────────────────

echo "══════════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "══════════════════════════════════════════"
echo ""

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
