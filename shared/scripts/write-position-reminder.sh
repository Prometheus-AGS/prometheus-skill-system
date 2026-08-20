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

# Observability: emit a start/end record to ~/.prometheus/hooks.log so this
# hook is visible to `doctor` and to latency analysis. The library no-ops when
# the log directory is not writable, and never changes this script's exit code.
HOOK_LOG_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && . "$HOOK_LOG_LIB"
command -v hook_log_start >/dev/null 2>&1 && hook_log_start "PostToolUse" "write-position-reminder.sh"
command -v hook_log_end >/dev/null 2>&1 && trap 'hook_log_end $?' EXIT


# Resolve project root — walk up from CWD looking for .kbd-orchestrator/
_find_root() {
  local dir="$PWD"
  while [[ "$dir" != "/" ]]; do
    [[ -d "$dir/.kbd-orchestrator" ]] && echo "$dir" && return 0
    dir="$(dirname "$dir")"
  done
  # A tool root is accepted only when it carries an explicit project UUID.
  local candidate
  for candidate in "${REPO_ROOT:-}"; do
    [[ -n "$candidate" ]] || continue
    [[ -d "${candidate}/.kbd-orchestrator" ]] || continue
    [[ -f "${candidate}/.prometheus/project.json" ]] || continue
    echo "$candidate"
    return 0
  done
  return 1
}

ROOT=$(_find_root 2>/dev/null) || { echo "[write-position-reminder] could not find .kbd-orchestrator" >&2; exit 0; }

WAYPOINT="$ROOT/.kbd-orchestrator/current-waypoint.json"
REMINDER="$ROOT/.kbd-orchestrator/position-reminder.txt"

[[ -f "$WAYPOINT" ]] || { echo "[write-position-reminder] no waypoint found" >&2; exit 0; }

normalize_next_command() {
  local next="${1:-}" change="${2:-}" remainder=""
  case "$next" in
    "/opsx:apply")
      [[ -n "$change" ]] && { printf '/kbd-apply %s' "$change"; return 0; }
      ;;
    "/opsx:apply "*)
      remainder="${next#"/opsx:apply "}"
      [[ -n "$remainder" ]] && { printf '/kbd-apply %s' "$remainder"; return 0; }
      ;;
    "/speckit.implement")
      [[ -n "$change" ]] && { printf '/kbd-apply %s' "$change"; return 0; }
      ;;
    "/speckit.implement "*)
      remainder="${next#"/speckit.implement "}"
      [[ -n "$remainder" ]] && { printf '/kbd-apply %s' "$remainder"; return 0; }
      ;;
  esac
  printf '%s' "$next"
}

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
CHANGE_ID=$(jq -r '.change // .active_change // empty' "$WAYPOINT" 2>/dev/null) || CHANGE_ID=""
NEXT_CMD="$(normalize_next_command "$NEXT_CMD" "$CHANGE_ID")"
CHANGES_COMPLETED=$(jq -r '.implementationCompleted // .implementation_completed // .changesCompleted // .changes_completed // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_COMPLETED=0
CHANGES_TOTAL=$(jq -r '.implementationTotal // .implementation_total // .changesTotal // .changes_total // 0' "$WAYPOINT" 2>/dev/null) || CHANGES_TOTAL=0

# Try progress.json for more accurate counts
PROGRESS="$ROOT/.kbd-orchestrator/phases/$PHASE/progress.json"
if [[ -f "$PROGRESS" ]]; then
  PC=$(jq -r '.changes_completed // empty' "$PROGRESS" 2>/dev/null)
  PT=$(jq -r '.changes_total // empty' "$PROGRESS" 2>/dev/null)
  [[ -n "$PC" ]] && CHANGES_COMPLETED="$PC"
  [[ -n "$PT" ]] && CHANGES_TOTAL="$PT"
fi

SUSPENDED=false
case "$(printf '%s' "$STAGE" | tr '[:upper:] -' '[:lower:]__')" in
  pause_requested|paused|blocked|suspended) SUSPENDED=true ;;
esac
[[ -e "$ROOT/.kbd-orchestrator/PAUSE" ]] && SUSPENDED=true

if [[ "$SUSPENDED" == "true" ]]; then
cat > "$REMINDER" <<EOF
POSITION REMINDER — operator pause advisory recorded
Phase: $PHASE
Step: $CHANGES_COMPLETED of $CHANGES_TOTAL
Stage: $STAGE
Recorded next command: $NEXT_CMD

Confirm operator intent before advancing the recorded next command. Tools remain
available; the exclusive journal transaction lock governs concurrent writes.
EOF
echo "[write-position-reminder] wrote suspended reminder $REMINDER" >&2
exit 0
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
