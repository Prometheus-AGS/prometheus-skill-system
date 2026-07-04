#!/usr/bin/env bash
# write-position-reminder.sh
# PostToolUse(Write|Edit|MultiEdit) companion — regenerates
# .kbd-orchestrator/position-reminder.txt so the model reads it as its FIRST
# tool call instead of parsing JSON or HTML comments.
#
# Gating: this only regenerates when the tool call that just ran actually
# wrote the waypoint or the active phase's progress.json — i.e. a real state
# change, not an incidental edit somewhere else in the repo. Without this
# gate, the PostToolUse matcher fires on every Write/Edit/MultiEdit call in
# the whole session and unconditionally clobbers whatever richer content
# (goal-tracking status, next-path options, lessons-to-act-on) kbd-reflect or
# an agent had placed in this file, since the regenerated template only ever
# contains the fixed Phase/Step/Stage/Next-command fields. When invoked with
# no stdin payload (e.g. a manual/direct call), the gate is skipped and the
# file is always regenerated — that call site is asserting the update itself.

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

# Gate on the PostToolUse payload (if any): only regenerate when the file the
# tool just wrote is the waypoint or the active phase's progress.json.
INPUT="$(cat 2>/dev/null || true)"
if [[ -n "$INPUT" ]] && command -v jq >/dev/null 2>&1; then
  TOOL_FILE="$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // .tool_input.path // empty' 2>/dev/null || true)"
  if [[ -n "$TOOL_FILE" ]]; then
    PHASE_FOR_GATE="$(jq -r '.phase // empty' "$WAYPOINT" 2>/dev/null || true)"
    PROGRESS_FOR_GATE="$ROOT/.kbd-orchestrator/phases/$PHASE_FOR_GATE/progress.json"
    case "$TOOL_FILE" in
      "$WAYPOINT"|*/current-waypoint.json) : ;;
      "$PROGRESS_FOR_GATE") : ;;
      *)
        echo "[write-position-reminder] skip: $TOOL_FILE is not the waypoint or active progress.json" >&2
        exit 0
        ;;
    esac
  fi
fi

PHASE=$(jq -r '.phase // "unknown"' "$WAYPOINT" 2>/dev/null) || PHASE="unknown"
STAGE=$(jq -r '.stage // .status // "unknown"' "$WAYPOINT" 2>/dev/null) || STAGE="unknown"
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
