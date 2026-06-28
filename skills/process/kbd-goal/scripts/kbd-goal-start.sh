#!/usr/bin/env bash
# kbd-goal-start.sh — Create goal state and spawn child phases
# Usage: kbd-goal-start.sh <description> [--phases p1,p2,...] [--stop <condition>] [--tokens N] [--max-turns N] [--slug <slug>] [--auto-gates]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
KBD_ROOT="$REPO_ROOT/.kbd-orchestrator"

# ── Parse arguments ──────────────────────────────────────────────────────────
DESCRIPTION=""
PHASES="ideation,spec,creation"
STOP_CONDITION=""
TOKEN_BUDGET="200000"
MAX_TURNS="50"
SLUG=""
AUTO_GATES=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --phases)   PHASES="$2"; shift 2 ;;
    --stop)     STOP_CONDITION="$2"; shift 2 ;;
    --tokens)   TOKEN_BUDGET="$2"; shift 2 ;;
    --max-turns) MAX_TURNS="$2"; shift 2 ;;
    --slug)     SLUG="$2"; shift 2 ;;
    --auto-gates) AUTO_GATES=true; shift ;;
    --resume)
      SLUG="$2"
      echo "Resuming goal: $SLUG"
      cat "$KBD_ROOT/goals/$SLUG/goal.json" 2>/dev/null || { echo "Goal not found: $SLUG"; exit 1; }
      exit 0
      ;;
    -*) echo "Unknown flag: $1" >&2; exit 1 ;;
    *)  DESCRIPTION="$1"; shift ;;
  esac
done

if [[ -z "$DESCRIPTION" ]]; then
  echo "Usage: kbd-goal-start.sh <description> [--phases p1,p2,...] [--stop <condition>]" >&2
  exit 1
fi

# ── Detect tool ───────────────────────────────────────────────────────────────
TOOL="${TOOL:-}"
if [[ -z "$TOOL" ]]; then
  WAYPOINT="$KBD_ROOT/current-waypoint.json"
  if [[ -f "$WAYPOINT" ]]; then
    TOOL=$(jq -r '.tool // "claude-code"' "$WAYPOINT" 2>/dev/null || echo "claude-code")
  else
    TOOL="claude-code"
  fi
fi

# ── Generate slug ─────────────────────────────────────────────────────────────
if [[ -z "$SLUG" ]]; then
  SLUG=$(echo "$DESCRIPTION" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-\|-$//g' | cut -c1-48)
  SLUG="${SLUG}-$(date +%Y%m%d)"
fi

GOAL_DIR="$KBD_ROOT/goals/$SLUG"

if [[ -d "$GOAL_DIR" ]]; then
  echo "Goal already exists: $SLUG (use --resume $SLUG to continue)" >&2
  exit 1
fi

mkdir -p "$GOAL_DIR"

# ── Run skill/MCP discovery ───────────────────────────────────────────────────
DISCOVER_SCRIPT="$REPO_ROOT/scripts/kbd-goal-discover.sh"
if [[ -x "$DISCOVER_SCRIPT" ]]; then
  echo ""
  echo "── Skill & MCP Discovery ─────────────────────────────────────────"
  "$DISCOVER_SCRIPT" "$DESCRIPTION" || true
  echo "─────────────────────────────────────────────────────────────────"
  echo ""
fi

# ── Build phases array ────────────────────────────────────────────────────────
IFS=',' read -ra PHASE_LIST <<< "$PHASES"

PHASES_JSON="[]"
for p in "${PHASE_LIST[@]}"; do
  p=$(echo "$p" | xargs)  # trim whitespace
  case "$p" in
    ideation)
      STOP="3 candidates score ≥7.0 in IDEAS.md"
      TYPE="ideation"
      ;;
    spec)
      STOP="kbd-spec-reviewer returns PASS on SPEC.md"
      TYPE="spec"
      ;;
    creation)
      STOP="${STOP_CONDITION:-all tasks in TASKS.md marked complete with tests passing}"
      TYPE="creation"
      ;;
    deployment)
      STOP="${STOP_CONDITION:-deployment smoke test passes}"
      TYPE="deployment"
      ;;
    *)
      STOP="${STOP_CONDITION:-phase complete}"
      TYPE="custom"
      ;;
  esac

  HUMAN_GATE="true"
  if [[ "$AUTO_GATES" == "true" ]]; then
    HUMAN_GATE="false"
  fi

  PHASES_JSON=$(echo "$PHASES_JSON" | jq \
    --arg name "$p" \
    --arg type "$TYPE" \
    --arg stop "$STOP" \
    --arg gate "$HUMAN_GATE" \
    '. + [{"name": $name, "type": $type, "stopping_condition": $stop, "human_gate": ($gate == "true")}]')
done

FIRST_PHASE=$(echo "$PHASES_JSON" | jq -r '.[0].name')

# ── Write goal.json ───────────────────────────────────────────────────────────
NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GOAL_JSON_TMP=$(mktemp)
cat > "$GOAL_JSON_TMP" << GOALJSON
{
  "name": "$DESCRIPTION",
  "slug": "$SLUG",
  "description": "$DESCRIPTION",
  "phases": $(echo "$PHASES_JSON"),
  "active_phase": "$FIRST_PHASE",
  "status": "running",
  "tool": "$TOOL",
  "token_budget": $TOKEN_BUDGET,
  "max_turns_per_phase": $MAX_TURNS,
  "max_no_progress_turns": 3,
  "auto_gates": $AUTO_GATES,
  "created": "$NOW",
  "updated": "$NOW"
}
GOALJSON
mv "$GOAL_JSON_TMP" "$GOAL_DIR/goal.json"

# ── Initialize empty artifact files ──────────────────────────────────────────
for f in IDEAS.md SPEC.md TASKS.md; do
  touch "$GOAL_DIR/$f"
done

cat > "$GOAL_DIR/STATE.md" << STATEMD
# STATE — $DESCRIPTION

**Goal slug:** $SLUG
**Status:** running
**Active phase:** $FIRST_PHASE
**Tool:** $TOOL

## Progress

- completed: 0
- total: 0
- active_task: null

## Tasks

| ID | Status | Fail Count |
|----|--------|-----------|

## Escalations

_None_

## Promotions

_None_
STATEMD

# ── Write corresponding loop.json ─────────────────────────────────────────────
LOOPS_DIR="$KBD_ROOT/loops/$SLUG"
mkdir -p "$LOOPS_DIR"
cat > "$LOOPS_DIR/loop.json" << LOOPJSON
{
  "name": "$SLUG",
  "goal": {
    "description": "$DESCRIPTION",
    "measurable_criteria": [$(echo "$PHASES_JSON" | jq -r '[.[].stopping_condition | @json] | join(",")')]
  },
  "phases": $(echo "$PHASES_JSON"),
  "goal_slug": "$SLUG",
  "feedback_sources": [
    {"type": "file", "path": "$GOAL_DIR/STATE.md", "interpret": "read current phase progress"}
  ],
  "termination": {
    "goal_satisfied": true,
    "max_ticks": $MAX_TURNS,
    "max_no_progress_ticks": 3,
    "budget": {"max_tokens": $TOKEN_BUDGET}
  },
  "escalation_points": [
    "human_gate after each phase",
    "task fail_count >= 3",
    "ambiguity or security concern detected"
  ],
  "cadence": {"mode": "manual"},
  "evolution_name": "$SLUG"
}
LOOPJSON

echo "Goal created: $SLUG"
echo "Goal dir: $GOAL_DIR"
echo "Active phase: $FIRST_PHASE"
echo ""
echo "Next: /kbd-goal-evaluate or start the $FIRST_PHASE phase loop"
