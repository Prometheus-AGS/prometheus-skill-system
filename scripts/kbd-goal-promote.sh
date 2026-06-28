#!/usr/bin/env bash
# kbd-goal-promote.sh — Auto-promote a failed task to a child KBD phase.
#
# Called when a task's fail_count reaches the threshold (default: 3).
# Creates a child phase with full context, marks the parent task as promoted.
#
# Usage:
#   kbd-goal-promote.sh <goal-slug> <task-id>
#   kbd-goal-promote.sh standup-gen task-003
#
# Exit codes:
#   0  → promotion created
#   1  → usage error
#   2  → required files missing
#   3  → task already promoted

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Args ──────────────────────────────────────────────────────────────────────
if [[ $# -lt 2 ]]; then
  echo "Usage: kbd-goal-promote.sh <goal-slug> <task-id>" >&2
  exit 1
fi

GOAL_SLUG="$1"
TASK_ID="$2"
FAIL_THRESHOLD="${KBD_GOAL_PROMOTE_THRESHOLD:-3}"

GOAL_DIR="$REPO_ROOT/.kbd-orchestrator/goals/$GOAL_SLUG"
STATE_FILE="$GOAL_DIR/STATE.md"
TASKS_FILE="$GOAL_DIR/TASKS.md"
GOAL_JSON="$GOAL_DIR/goal.json"

# ── Validate ──────────────────────────────────────────────────────────────────
for f in "$STATE_FILE" "$TASKS_FILE" "$GOAL_JSON"; do
  if [[ ! -f "$f" ]]; then
    echo "Error: required file missing: $f" >&2
    exit 2
  fi
done

# Check not already promoted
if grep -q "\[~\].*$TASK_ID" "$TASKS_FILE" 2>/dev/null; then
  echo "Task $TASK_ID is already promoted — skipping." >&2
  exit 3
fi

# ── Extract task context ──────────────────────────────────────────────────────
# Get the task description from TASKS.md
TASK_LINE=$(grep "$TASK_ID" "$TASKS_FILE" | head -1 || echo "")
TASK_DESC=$(echo "$TASK_LINE" | sed 's/^\[.\] //; s/^[[:space:]]*//')

# Get last 3 failure reasons from STATE.md
FAILURE_REASONS=$(grep -A2 "FAIL.*$TASK_ID\|$TASK_ID.*FAIL" "$STATE_FILE" 2>/dev/null | head -9 || echo "(no failure reasons recorded)")

# Get acceptance criteria from SPEC.md if it exists
SPEC_CRITERIA=""
SPEC_FILE="$GOAL_DIR/SPEC.md"
if [[ -f "$SPEC_FILE" ]]; then
  # Extract acceptance criteria section(s) related to this task
  SPEC_CRITERIA=$(grep -A10 "AC-\|Acceptance Criteria" "$SPEC_FILE" 2>/dev/null | head -30 || echo "")
fi

# ── Create child phase directory ──────────────────────────────────────────────
CHILD_PHASE_NAME="goal-${GOAL_SLUG}-${TASK_ID}"
CHILD_PHASE_DIR="$REPO_ROOT/.kbd-orchestrator/phases/$CHILD_PHASE_NAME"

mkdir -p "$CHILD_PHASE_DIR"

# Write goals.md for the child phase
cat > "$CHILD_PHASE_DIR/goals.md" << GOALS_EOF
# Goals — $CHILD_PHASE_NAME

Auto-promoted from parent goal: $GOAL_SLUG, task: $TASK_ID

**Promoted because:** Task failed $FAIL_THRESHOLD consecutive times in the parent goal loop.

## Primary Goal

Complete task **$TASK_ID**: $TASK_DESC

## Acceptance Criteria

$([ -n "$SPEC_CRITERIA" ] && echo "$SPEC_CRITERIA" || echo "(Read SPEC.md in the parent goal directory for full acceptance criteria.)")
GOALS_EOF

# Write handoff-in.md with full context
cat > "$CHILD_PHASE_DIR/handoff-in.md" << HANDOFF_EOF
# Handoff In — $CHILD_PHASE_NAME

**Source:** Parent goal loop — $GOAL_SLUG, task $TASK_ID
**Promoted:** $(date -u +%Y-%m-%dT%H:%M:%SZ)
**Reason:** Task failed ${FAIL_THRESHOLD}+ consecutive times.

## Task

$TASK_ID: $TASK_DESC

## Last 3 Failure Reasons

$FAILURE_REASONS

## Acceptance Criteria

$([ -n "$SPEC_CRITERIA" ] && echo "$SPEC_CRITERIA" || echo "(Read parent SPEC.md: $SPEC_FILE)")

## Parent Context Files

- Goal definition: $GOAL_JSON
- Full specification: $SPEC_FILE
- Task list: $TASKS_FILE
- Execution state: $STATE_FILE

## Instructions

1. Run `/kbd-assess` on this child phase to understand the specific failure mode.
2. Do NOT try to shortcut or simplify — the parent loop already tried 3 times.
3. Focus on correctness for this single task's acceptance criteria.
4. When complete, the parent loop will resume with the remaining tasks.
HANDOFF_EOF

# Write progress.json for the child phase
cat > "$CHILD_PHASE_DIR/progress.json" << PROGRESS_EOF
{
  "phase": "$CHILD_PHASE_NAME",
  "stage": "assessment_ready",
  "changes_total": 0,
  "changes_completed": 0,
  "active_change": null,
  "next_pending_change": null,
  "started": "$(date -u +%Y-%m-%d)",
  "updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "parent_goal": "$GOAL_SLUG",
  "parent_task": "$TASK_ID"
}
PROGRESS_EOF

# ── Update parent TASKS.md — mark task as promoted ────────────────────────────
# Replace [ ] or [/] with [~] for this task, append child phase reference
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "s/\[[ /]\] .*$TASK_ID.*$/[~] $TASK_DESC (promoted to: $CHILD_PHASE_NAME)/" "$TASKS_FILE"
else
  sed -i "s/\[[ /]\] .*$TASK_ID.*$/[~] $TASK_DESC (promoted to: $CHILD_PHASE_NAME)/" "$TASKS_FILE"
fi

# ── Update parent STATE.md ────────────────────────────────────────────────────
cat >> "$STATE_FILE" << STATE_EOF

## Promotion — $(date -u +%Y-%m-%dT%H:%M:%SZ)

- **Task promoted:** $TASK_ID
- **Child phase:** $CHILD_PHASE_NAME
- **Reason:** ${FAIL_THRESHOLD} consecutive failures
- **Parent loop status:** Continuing with remaining tasks

EOF

# Update promotions array in goal.json
if command -v jq &>/dev/null; then
  UPDATED_GOAL=$(jq --arg task "$TASK_ID" --arg child "$CHILD_PHASE_NAME" \
    '.promotions = (.promotions // []) + [{"task": $task, "child_phase": $child, "promoted": now | todate}]' \
    "$GOAL_JSON")
  echo "$UPDATED_GOAL" > "$GOAL_JSON"
fi

echo ""
echo "✅ Task $TASK_ID promoted to child phase: $CHILD_PHASE_NAME"
echo ""
echo "Child phase directory: $CHILD_PHASE_DIR"
echo "Next step for child phase: /kbd-assess $CHILD_PHASE_NAME"
echo ""
echo "Parent goal loop can continue with remaining unchecked tasks."
