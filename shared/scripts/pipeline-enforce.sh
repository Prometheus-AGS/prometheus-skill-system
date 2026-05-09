#!/usr/bin/env bash
# SP-012: KBD pipeline enforcement hook
#
# Validates that the assess → plan → execute → reflect layer order is respected.
# Fires as a PreToolUse hook when bash commands reference kbd-execute or kbd-reflect
# without the required prerequisite artifacts being present.
#
# Exit codes:
#   0 — prerequisite checks pass; proceed
#   2 — prerequisite missing; emit blocking error with guidance
#
# Environment variables read:
#   CLAUDE_PLUGIN_ROOT  — path to skill-pack root (set by Claude Code)
#   KBD_PHASE           — the active KBD phase (optional override)

set -euo pipefail

HOOK_LOG="${HOME}/.prometheus/hooks.log"
mkdir -p "$(dirname "$HOOK_LOG")"

log_error() {
  local msg="$1"
  echo "[pipeline-enforce] ERROR: ${msg}" | tee -a "$HOOK_LOG" >&2
}

log_info() {
  local msg="$1"
  echo "[pipeline-enforce] ${msg}" >> "$HOOK_LOG"
}

# Read the tool input JSON from stdin (Claude Code passes it as JSON)
TOOL_INPUT="$(cat)"
TOOL_NAME="${1:-unknown}"

log_info "$(date -u +%Y-%m-%dT%H:%M:%SZ) tool=${TOOL_NAME}"

# Only enforce on bash tool calls that reference kbd-execute or kbd-reflect
if ! echo "$TOOL_INPUT" | grep -qE '"kbd-execute|kbd-reflect|/kbd-execute|/kbd-reflect'; then
  exit 0
fi

# Find the KBD orchestrator directory (walk up from cwd)
find_kbd_dir() {
  local dir="$PWD"
  while [[ "$dir" != "/" ]]; do
    if [[ -d "${dir}/.kbd-orchestrator" ]]; then
      echo "${dir}/.kbd-orchestrator"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  return 1
}

KBD_DIR=""
if ! KBD_DIR="$(find_kbd_dir)"; then
  log_info "no .kbd-orchestrator found; skipping enforcement"
  exit 0
fi

# Find the active phase from current-waypoint.json
WAYPOINT="${KBD_DIR}/current-waypoint.json"
if [[ ! -f "$WAYPOINT" ]]; then
  log_info "no current-waypoint.json; skipping enforcement"
  exit 0
fi

PHASE="$(python3 -c "import json,sys; d=json.load(open('${WAYPOINT}')); print(d.get('phase',''))" 2>/dev/null || true)"
if [[ -z "$PHASE" ]]; then
  log_info "could not read phase from waypoint; skipping"
  exit 0
fi

PHASE_DIR="${KBD_DIR}/phases/${PHASE}"
ASSESSMENT="${PHASE_DIR}/assessment.md"
PLAN="${PHASE_DIR}/plan.md"
PROGRESS="${PHASE_DIR}/progress.json"

# Rule: kbd-execute requires plan.md to exist
if echo "$TOOL_INPUT" | grep -qE '"kbd-execute|/kbd-execute'; then
  if [[ ! -f "$PLAN" ]]; then
    log_error "kbd-execute attempted without plan.md for phase '${PHASE}'"
    cat >&2 <<EOF

[pipeline-enforce] BLOCKED — out-of-order execution detected.

  Phase:  ${PHASE}
  Missing: ${PLAN}

  The KBD pipeline requires:
    1. /kbd-assess  → produces assessment.md
    2. /kbd-plan    → produces plan.md        ← REQUIRED before execute
    3. /kbd-execute → runs the plan
    4. /kbd-reflect → writes reflection.md

  Run /kbd-plan to produce plan.md before attempting execution.

EOF
    exit 2
  fi

  if [[ ! -f "$ASSESSMENT" ]]; then
    log_error "kbd-execute attempted without assessment.md for phase '${PHASE}'"
    cat >&2 <<EOF

[pipeline-enforce] BLOCKED — assessment missing.

  Phase:  ${PHASE}
  Missing: ${ASSESSMENT}

  Run /kbd-assess first to produce assessment.md.

EOF
    exit 2
  fi
fi

# Rule: kbd-reflect requires all changes DONE in progress.json
if echo "$TOOL_INPUT" | grep -qE '"kbd-reflect|/kbd-reflect'; then
  if [[ ! -f "$PROGRESS" ]]; then
    log_error "kbd-reflect attempted without progress.json for phase '${PHASE}'"
    cat >&2 <<EOF

[pipeline-enforce] BLOCKED — progress.json missing.

  Phase:  ${PHASE}
  Missing: ${PROGRESS}

  Run /kbd-execute to produce progress.json before reflecting.

EOF
    exit 2
  fi

  # Check if all changes are DONE
  NOT_DONE="$(python3 - "$PROGRESS" <<'PYEOF' 2>/dev/null || true
import json, sys
data = json.load(open(sys.argv[1]))
total = data.get("changes_total", 0)
completed = data.get("changes_completed", 0)
if completed < total:
    print(f"{completed}/{total}")
PYEOF
  )"

  if [[ -n "$NOT_DONE" ]]; then
    log_error "kbd-reflect attempted with incomplete changes (${NOT_DONE}) for phase '${PHASE}'"
    cat >&2 <<EOF

[pipeline-enforce] BLOCKED — not all changes are DONE.

  Phase:    ${PHASE}
  Progress: ${NOT_DONE} changes completed

  Complete all changes before running /kbd-reflect.
  Check progress: cat ${PROGRESS}

EOF
    exit 2
  fi
fi

log_info "pipeline-enforce PASS for phase '${PHASE}'"
exit 0
