#!/usr/bin/env bash
# write-session-summary.sh — Stop hook (runs first in Stop chain)
# Writes a structured session summary to ~/.prometheus/last-session-summary.txt
# for operator inspection and asynchronous worker reconciliation.
# Must always exit 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "Stop" "write-session-summary.sh"

SUMMARY_DIR="${HOME}/.prometheus"
mkdir -p "$SUMMARY_DIR" 2>/dev/null || true

SUMMARY_FILE="${SUMMARY_DIR}/last-session-summary.txt"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || echo "unknown")

# Read waypoint fields (graceful if outside a git repo or waypoint absent)
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || REPO_ROOT=""
WAYPOINT=""
[[ -n "$REPO_ROOT" ]] && WAYPOINT="${REPO_ROOT}/.kbd-orchestrator/current-waypoint.json"

PHASE="unknown"
STAGE="unknown"
LAST_COMPLETED="none"
CHANGES_DONE="0"
CHANGES_TOTAL="0"
NEXT_PENDING="none"

if [[ -n "$WAYPOINT" && -f "$WAYPOINT" ]]; then
  PHASE=$(jq -r '.phase // "unknown"' "$WAYPOINT" 2>/dev/null) || PHASE="unknown"
  STAGE=$(jq -r '.stage // .status // "unknown"' "$WAYPOINT" 2>/dev/null) || STAGE="unknown"
  LAST_COMPLETED=$(jq -r '.last_completed // "none"' "$WAYPOINT" 2>/dev/null) || LAST_COMPLETED="none"
  CHANGES_DONE=$(jq -r '.changes_completed // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_DONE="0"
  CHANGES_TOTAL=$(jq -r '.changes_total // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_TOTAL="0"
  NEXT_PENDING=$(jq -r '.next_pending_change // "none"' "$WAYPOINT" 2>/dev/null) || NEXT_PENDING="none"
fi

# Write the summary
{
  echo "session_ended: ${TIMESTAMP}"
  echo "phase: ${PHASE}"
  echo "stage: ${STAGE}"
  echo "last_completed: ${LAST_COMPLETED}"
  echo "progress: ${CHANGES_DONE} of ${CHANGES_TOTAL} changes done"
  echo "next_pending: ${NEXT_PENDING}"
} > "$SUMMARY_FILE" 2>/dev/null || true

hook_log_end 0
exit 0
