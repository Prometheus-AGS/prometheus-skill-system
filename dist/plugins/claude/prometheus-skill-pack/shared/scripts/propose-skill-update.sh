#!/usr/bin/env bash
# propose-skill-update.sh
# Called by evaluate-session.sh when learning patterns match an existing skill name.
# Does NOT apply changes — writes a candidate entry for human review via
# /pmpo-skill-creator --update <skill-name>.
#
# Usage: propose-skill-update.sh <skill-name>
# Exit codes: 0 always (non-blocking; called with || true)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="${SCRIPT_DIR}/lib"
if [[ -f "${LIB_DIR}/hook-log.sh" ]]; then
  # shellcheck source=/dev/null
  . "${LIB_DIR}/hook-log.sh"
  HOOK_LOG_NAME="propose-skill-update"
  hook_log_start
else
  hook_log_start()  { :; }
  hook_log_end()    { :; }
  hook_log_error()  { echo "[propose-skill-update] ERROR: $*" >&2; }
fi

SKILL_NAME="${1:-}"
if [[ -z "$SKILL_NAME" ]]; then
  hook_log_error "No skill name provided — usage: propose-skill-update.sh <skill-name>"
  hook_log_end 0
  exit 0
fi

SKILL_PATH="${HOME}/.claude/skills/${SKILL_NAME}/SKILL.md"
if [[ ! -f "$SKILL_PATH" ]]; then
  # Skill not installed; nothing to propose
  hook_log_end 0
  exit 0
fi

UPDATES_DIR="${HOME}/.prometheus/skill-updates"
mkdir -p "$UPDATES_DIR"

PENDING_LOG="${UPDATES_DIR}/pending.log"
LEARNING_LOG_DIR="${HOME}/.prometheus/learning-log"

# Count learning log files that mention this skill
LEARNING_HITS=0
if [[ -d "$LEARNING_LOG_DIR" ]]; then
  # Use find to avoid glob expansion errors when no .jsonl files exist yet
  LEARNING_HITS=$(find "$LEARNING_LOG_DIR" -maxdepth 1 -name '*.jsonl' -exec grep -l "$SKILL_NAME" {} + 2>/dev/null | wc -l | tr -d ' ')
fi

if [[ "$LEARNING_HITS" -eq 0 ]]; then
  # No learning entries for this skill — nothing to propose
  hook_log_end 0
  exit 0
fi

# Write candidate entry to pending log (idempotent — avoid duplicates for same day)
TODAY="$(date +%Y-%m-%d)"
MARKER="${SKILL_NAME}::${TODAY}"
if ! grep -qF "$MARKER" "$PENDING_LOG" 2>/dev/null; then
  echo "${MARKER} hits=${LEARNING_HITS}" >> "$PENDING_LOG"
  echo "Run: /pmpo-skill-creator --update ${SKILL_NAME}  # to review and optionally apply" \
    >> "$PENDING_LOG"
fi

# Write a placeholder diff file so the user knows a candidate exists
DIFF_FILE="${UPDATES_DIR}/${SKILL_NAME}-${TODAY}.diff"
if [[ ! -f "$DIFF_FILE" ]]; then
  cat > "$DIFF_FILE" <<EOF
# Skill update candidate — ${SKILL_NAME} (${TODAY})
# Learning entries matched: ${LEARNING_HITS}
#
# This file was created by propose-skill-update.sh.
# It does NOT contain the actual diff yet.
#
# To generate and review the proposed changes, run:
#   /pmpo-skill-creator --update ${SKILL_NAME}
#
# The --update command will:
#   1. Read ~/.claude/skills/${SKILL_NAME}/SKILL.md
#   2. Pull matched learning patterns from ~/.prometheus/learning-log/
#   3. Generate a real unified diff
#   4. Present the diff for your approval before applying anything
EOF
fi

hook_log_end 0
exit 0
