#!/usr/bin/env bash
# write-position-reminder.sh
# Called by state-checkpoint.sh after every waypoint update.
# Writes .kbd-orchestrator/position-reminder.txt so the model reads it
# as its FIRST tool call instead of parsing JSON or HTML comments.

set -euo pipefail

# Resolve project root — walk up from CWD looking for .kbd-orchestrator/
_find_root() {
  local dir="$PWD"
  while [[ "$dir" != "/" ]]; do
    [[ -d "$dir/.kbd-orchestrator" ]] && echo "$dir" && return 0
    dir="$(dirname "$dir")"
  done
  # Fallbacks
  for candidate in "${REPO_ROOT:-}" "${CLAUDE_PLUGIN_ROOT:-}" "${HOME}/Projects/prometheus/prometheus-skill-pack"; do
    [[ -d "${candidate}/.kbd-orchestrator" ]] && echo "$candidate" && return 0
  done
  return 1
}

ROOT=$(_find_root 2>/dev/null) || { echo "[write-position-reminder] could not find .kbd-orchestrator" >&2; exit 0; }

WAYPOINT="$ROOT/.kbd-orchestrator/current-waypoint.json"
REMINDER="$ROOT/.kbd-orchestrator/position-reminder.txt"

[[ -f "$WAYPOINT" ]] || { echo "[write-position-reminder] no waypoint found" >&2; exit 0; }

PHASE=$(jq -r '.phase // "unknown"' "$WAYPOINT" 2>/dev/null) || PHASE="unknown"
STAGE=$(jq -r '.stage // "unknown"' "$WAYPOINT" 2>/dev/null) || STAGE="unknown"
NEXT_CMD=$(jq -r '.exact_next_command // .exactNextCommand // "unknown"' "$WAYPOINT" 2>/dev/null) || NEXT_CMD="unknown"
CHANGES_COMPLETED=$(jq -r '.changes_completed // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_COMPLETED=0
CHANGES_TOTAL=$(jq -r '.changes_total // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_TOTAL=0

# Try progress.json for more accurate counts
PROGRESS="$ROOT/.kbd-orchestrator/phases/$PHASE/progress.json"
if [[ -f "$PROGRESS" ]]; then
  PC=$(jq -r '.changes_completed // empty' "$PROGRESS" 2>/dev/null)
  PT=$(jq -r '.changes_total // empty' "$PROGRESS" 2>/dev/null)
  [[ -n "$PC" ]] && CHANGES_COMPLETED="$PC"
  [[ -n "$PT" ]] && CHANGES_TOTAL="$PT"
fi

cat > "$REMINDER" <<EOF
POSITION REMINDER — read this as your FIRST tool call every turn
Phase: $PHASE
Step: $CHANGES_COMPLETED of $CHANGES_TOTAL
Stage: $STAGE
Next command: $NEXT_CMD

Required signal format (emit BEFORE any tool call):
  Starting <kbd-skill> — $PHASE (step $CHANGES_COMPLETED of $CHANGES_TOTAL)
  [do work]
  Completed <kbd-skill> — $PHASE (step $CHANGES_COMPLETED of $CHANGES_TOTAL)
EOF

echo "[write-position-reminder] wrote $REMINDER (step $CHANGES_COMPLETED of $CHANGES_TOTAL)" >&2
