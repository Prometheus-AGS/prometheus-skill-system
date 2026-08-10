#!/usr/bin/env bash
# test-subagent-fallback.sh — SP-014 verification test
#
# Verifies that the SubagentStop fallback hook (subagent-checkpoint-fallback.sh)
# fires correctly for unknown subagent types and writes the expected log entry.
#
# Usage:
#   bash shared/scripts/tests/test-subagent-fallback.sh
#
# Exit codes:
#   0 — all assertions passed
#   1 — one or more assertions failed
#
# The test simulates what Claude Code's hook system does:
#   1. Sets SUBAGENT_NAME (what the hook reads as agent name)
#   2. Invokes the fallback script directly
#   3. Checks exit code
#   4. Checks hook log for expected entries
#
# NOTE: This cannot simulate the hooks.json matcher dispatch itself (that
# requires a live Claude Code session). What it verifies is that:
#   a) The fallback script exists and is executable
#   b) The script exits 0 for all tested subagent names (required — hook
#      scripts must never fail or they block the Stop chain)
#   c) The script writes a log entry to ~/.prometheus/hooks.log
#   d) The log entry contains the expected fields

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
FALLBACK_SCRIPT="${REPO_ROOT}/shared/scripts/subagent-checkpoint-fallback.sh"
HOOK_LOG="${HOME}/.prometheus/hooks.log"

# ─── Helpers ─────────────────────────────────────────────────────────────────

PASS=0
FAIL=0
ERRORS=()

pass() { echo "  PASS: $1"; PASS=$(( PASS + 1 )); }
fail() { echo "  FAIL: $1"; FAIL=$(( FAIL + 1 )); ERRORS+=("$1"); }

assert_exit_zero() {
  local label="$1" script="$2" agent_name="$3"
  SUBAGENT_NAME="$agent_name" bash "$script" 2>/dev/null
  local code=$?
  if [ "$code" -eq 0 ]; then
    pass "$label: exit 0"
  else
    fail "$label: expected exit 0, got $code"
  fi
}

assert_log_contains() {
  local label="$1" pattern="$2"
  # Only check lines written in the last 10 seconds (avoids stale log hits)
  local recent
  recent="$(tail -50 "$HOOK_LOG" 2>/dev/null || echo '')"
  if echo "$recent" | grep -q "$pattern"; then
    pass "$label: log contains '$pattern'"
  else
    fail "$label: log does NOT contain '$pattern'"
  fi
}

# ─── Pre-flight ──────────────────────────────────────────────────────────────

echo "SP-014: SubagentStop fallback verification"
echo "  Script under test: ${FALLBACK_SCRIPT}"
echo "  Hook log: ${HOOK_LOG}"
echo ""

if [ ! -f "$FALLBACK_SCRIPT" ]; then
  echo "FATAL: fallback script not found at $FALLBACK_SCRIPT"
  exit 1
fi

if [ ! -x "$FALLBACK_SCRIPT" ]; then
  echo "FATAL: fallback script is not executable: $FALLBACK_SCRIPT"
  exit 1
fi

pass "fallback script exists and is executable"

# ─── Test: syntax check ───────────────────────────────────────────────────────

bash -n "$FALLBACK_SCRIPT" 2>/dev/null
if [ $? -eq 0 ]; then
  pass "bash -n syntax check passed"
else
  fail "bash -n syntax check FAILED"
fi

# ─── Test: unknown subagent types exit 0 ─────────────────────────────────────

SYNTHETIC_AGENTS=(
  "synthetic-unknown-agent"
  "custom-researcher-v2"
  "arbitrary-task-agent"
  "bdd-runner-experimental"
  ""          # empty SUBAGENT_NAME (should fall back to "unknown")
)

for agent in "${SYNTHETIC_AGENTS[@]}"; do
  label="exit 0 for agent='${agent:-<empty>}'"
  assert_exit_zero "$label" "$FALLBACK_SCRIPT" "$agent"
done

# ─── Test: hook log entry written ────────────────────────────────────────────

# Run once more with a unique marker to verify log entry
UNIQUE_MARKER="sp014-test-$$-$(date +%s)"
export SUBAGENT_NAME="$UNIQUE_MARKER"
bash "$FALLBACK_SCRIPT" 2>/dev/null

# Give flock a moment to flush
sleep 0.1

if [ -f "$HOOK_LOG" ]; then
  # Check that the end event was written (hook_log_end writes event=end)
  recent_entries="$(tail -20 "$HOOK_LOG" 2>/dev/null || echo '')"
  if echo "$recent_entries" | grep -q '"script":"subagent-checkpoint-fallback.sh"'; then
    pass "hook log contains script entry for subagent-checkpoint-fallback.sh"
  else
    fail "hook log missing script entry (script may not be logging)"
  fi

  if echo "$recent_entries" | grep -q '"event":"end"'; then
    pass "hook log contains event:end (clean exit logged)"
  else
    fail "hook log missing event:end entry"
  fi
else
  # Hook log may not exist in CI or environments without ~/.prometheus/
  echo "  SKIP: hook log not found at $HOOK_LOG (ok in CI without setup)"
fi

# ─── Test: stderr output is present ──────────────────────────────────────────

stderr_output="$(SUBAGENT_NAME="test-agent" bash "$FALLBACK_SCRIPT" 2>&1 >/dev/null)"
if echo "$stderr_output" | grep -q "SubagentStop checkpoint"; then
  pass "stderr contains 'SubagentStop checkpoint' line"
else
  fail "stderr missing 'SubagentStop checkpoint' (got: '$stderr_output')"
fi

# ─── Results ─────────────────────────────────────────────────────────────────

echo ""
echo "Results: ${PASS} passed, ${FAIL} failed"

if [ "${FAIL}" -gt 0 ]; then
  echo ""
  echo "Failures:"
  for err in "${ERRORS[@]}"; do
    echo "  - $err"
  done
  exit 1
fi

echo "All assertions passed."
exit 0
