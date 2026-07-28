#!/usr/bin/env bash
# kbd-next-phase.sh — Seed and initialize the next KBD phase from reflection
#
# Usage: kbd-next-phase.sh [new-phase-name]
#   If no argument: extracts suggested name from reflection.md "Recommended Next Phase" section.
#   If argument provided: uses it as the new phase name (normalized to kebab-case).
#
# Reads:
#   .kbd-orchestrator/current-waypoint.json         -> current (completed) phase + stage
#   .kbd-orchestrator/phases/<phase>/reflection.md  -> next phase seed content
#   .kbd-orchestrator/project.json                  -> project name
#
# Writes:
#   .kbd-orchestrator/phases/<new-phase>/goals.md          (seeded goals)
#   .kbd-orchestrator/phases/<new-phase>/progress.json     (skeleton)
#   .kbd-orchestrator/current-waypoint.json                (updated)
#   .kbd-orchestrator/current-waypoint.md                  (updated)
#   .kbd-orchestrator/project.json                         (active_phase updated)

set -euo pipefail

PROPOSED_NAME="${1:-}"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
DATE_SLUG="$(date +%Y-%m-%d)"

# ── Locate project root (walk up until .kbd-orchestrator/ found) ───────────
find_project_root() {
  local dir="${PWD}"
  while [[ "$dir" != "/" ]]; do
    [[ -d "${dir}/.kbd-orchestrator" ]] && { echo "$dir"; return; }
    dir="$(dirname "$dir")"
  done
  echo "${PWD}"
}

PROJECT_ROOT="$(find_project_root)"
KBD_DIR="${PROJECT_ROOT}/.kbd-orchestrator"
WAYPOINT_JSON="${KBD_DIR}/current-waypoint.json"
WAYPOINT_MD="${KBD_DIR}/current-waypoint.md"
PROJECT_JSON="${KBD_DIR}/project.json"

# ── Validate waypoint exists ──────────────────────────────────────────────
if [[ ! -f "$WAYPOINT_JSON" ]]; then
  echo ""
  echo "[kbd-next-phase] ERROR: .kbd-orchestrator/current-waypoint.json not found."
  echo "  Run /kbd-init first to initialize KBD for this project."
  exit 1
fi

# ── Read current waypoint ─────────────────────────────────────────────────
CURRENT_PHASE="$(python3 -c "import json; d=json.load(open('$WAYPOINT_JSON')); print(d.get('phase','unknown'))" 2>/dev/null || echo "unknown")"
# Read the terminal marker from `stage` OR `status` OR `project.json` — the
# waypoint carries the same concept under different keys across generations,
# and reflect writes the terminal state to project.json (status: reflected)
# while the waypoint may lag. Checking all three avoids a spurious "not
# reflect_complete" warning on a genuinely reflected phase.
CURRENT_STAGE="$(python3 -c "import json; d=json.load(open('$WAYPOINT_JSON')); print(d.get('stage') or d.get('status') or 'unknown')" 2>/dev/null || echo "unknown")"
CHANGES_TOTAL="$(jq -r '.implementationTotal // .changesTotal // .changes_total // 0' "$WAYPOINT_JSON" 2>/dev/null || echo 0)"
CHANGES_COMPLETED="$(jq -r '.implementationCompleted // .changesCompleted // .changes_completed // 0' "$WAYPOINT_JSON" 2>/dev/null || echo 0)"

PROJECT_NAME="unknown"
if [[ -f "$PROJECT_JSON" ]]; then
  PROJECT_NAME="$(python3 -c "import json; d=json.load(open('$PROJECT_JSON')); print(d.get('name','unknown'))" 2>/dev/null || echo "unknown")"
fi

# ── Warn if reflection not complete ───────────────────────────────────────
# Accept any recognized "reflection done" vocabulary; project.json is the
# authoritative fallback when the waypoint status lags behind reflect.
PROJECT_STATUS=""
if [[ -f "$PROJECT_JSON" ]]; then
  PROJECT_STATUS="$(python3 -c "import json; d=json.load(open('$PROJECT_JSON')); print(d.get('status') or '')" 2>/dev/null || echo "")"
fi
case "${CURRENT_STAGE}:${PROJECT_STATUS}" in
  reflect_complete:*|reflected:*|phase_complete:*|*:reflect_complete|*:reflected|*:phase_complete) : ;;
  *)
    echo ""
    echo "[kbd-next-phase] WARNING: Current stage is '${CURRENT_STAGE}' (project status '${PROJECT_STATUS:-none}'), not a recognized reflection-complete state."
    echo "  It is recommended to run /kbd-reflect before continuing to the next phase."
    echo "  Proceeding anyway -- next phase will be seeded from whatever reflection content exists."
    echo ""
    ;;
esac

# ── Read reflection for seed content ─────────────────────────────────────
REFLECTION="${KBD_DIR}/phases/${CURRENT_PHASE}/reflection.md"
SEED_CONTENT=""
SEED_PHASE_NAME=""

if [[ -f "$REFLECTION" ]]; then
  # Extract content under "Recommended Next Phase", "Next Phase Seed", or "Next Phase" H2
  SEED_CONTENT="$(python3 - "$REFLECTION" << 'PYEOF'
import re, sys

reflection_path = sys.argv[1]

try:
    with open(reflection_path) as f:
        content = f.read()
except Exception:
    print("")
    sys.exit(0)

# Try progressively looser patterns
patterns = [
    r'^##\s+Recommended Next Phase(?:\s+Seed)?\s*$',
    r'^##\s+Next Phase Seed\s*$',
    r'^##\s+Next Phase\s*$',
    r'^##.*[Nn]ext.*[Pp]hase',
    r'^##.*[Rr]ecommended',
]

match = None
for pat in patterns:
    matches = list(re.finditer(pat, content, re.MULTILINE))
    if matches:
        match = matches[-1]
        break

if not match:
    print("")
    sys.exit(0)

start = match.end()
# Find next H2 heading or end of file
next_h2 = re.search(r'^##\s', content[start:], re.MULTILINE)
end = start + next_h2.start() if next_h2 else len(content)
print(content[start:end].strip())
PYEOF
)"

  # Try to extract a phase name from the seed content
  SEED_PHASE_NAME="$(echo "$SEED_CONTENT" | python3 -c "
import sys, re
text = sys.stdin.read()
# Look for backtick or bold phase-name
m = re.search(r'[\`\*]{1,2}(phase-[a-z0-9][a-z0-9-]*)[\`\*]{0,2}', text, re.IGNORECASE)
if m:
    print(m.group(1))
    sys.exit(0)
# Look for standalone slug
m = re.search(r'\b(phase-[a-z0-9][a-z0-9-]*)\b', text)
if m:
    print(m.group(1))
    sys.exit(0)
print('')
" 2>/dev/null || echo "")"
else
  echo ""
  echo "[kbd-next-phase] WARNING: No reflection.md found at:"
  echo "  ${REFLECTION}"
  echo "  Run /kbd-reflect first to generate a reflection, then re-run /kbd-next-phase."
  echo "  Continuing with empty seed content."
  echo ""
fi

# ── Determine new phase name ───────────────────────────────────────────────
if [[ -n "$PROPOSED_NAME" ]]; then
  NEW_PHASE="$PROPOSED_NAME"
elif [[ -n "$SEED_PHASE_NAME" ]]; then
  NEW_PHASE="${SEED_PHASE_NAME}-${DATE_SLUG}"
else
  NEW_PHASE="phase-next-${DATE_SLUG}"
fi

# Normalize: lowercase, spaces -> hyphens, strip non-[a-z0-9-] chars
NEW_PHASE="$(echo "$NEW_PHASE" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')"

# ── Guard: phase must not already exist ───────────────────────────────────
NEW_PHASE_DIR="${KBD_DIR}/phases/${NEW_PHASE}"
if [[ -d "$NEW_PHASE_DIR" ]]; then
  echo ""
  echo "[kbd-next-phase] ERROR: Phase '${NEW_PHASE}' already exists:"
  echo "  ${NEW_PHASE_DIR}"
  echo "  Choose a different name or resume it with: /kbd-assess ${NEW_PHASE}"
  exit 1
fi

# ── Create phase directory and seed files ─────────────────────────────────
mkdir -p "$NEW_PHASE_DIR"

# goals.md — seeded from reflection
cat > "${NEW_PHASE_DIR}/goals.md" << GOALS_EOF
# Goals: ${NEW_PHASE}

Seeded from: \`${CURRENT_PHASE}/reflection.md\`
Created: ${TIMESTAMP}

## Seeded Goals

${SEED_CONTENT:-"(No seed content found in reflection -- add goals manually before running /kbd-assess)"}

---

## Instructions

Review and refine the goals above before running \`/kbd-assess\`.
Add, remove, or clarify as needed. When ready:

\`\`\`
/kbd-assess ${NEW_PHASE}
\`\`\`
GOALS_EOF

# In runtime-authority mode the phase and active path are typed journal
# mutations. The compatibility JSON and waypoint markdown are projections.
SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
runtime_lib="$SKILL_ROOT/shared/lib/runtime-authority.sh"
if [[ -f "$runtime_lib" ]]; then
  # shellcheck source=/dev/null
  . "$runtime_lib"
fi
if command -v kbd_runtime_authoritative >/dev/null 2>&1 &&
   kbd_runtime_authoritative "$PROJECT_ROOT"; then
  mutation="$(kbd_runtime_mutation_args "$PROJECT_ROOT" "phase-create:${NEW_PHASE}")"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  lease_id="$(printf '%s\n' "$mutation" | sed -n '3p')"
  fencing_token="$(printf '%s\n' "$mutation" | sed -n '4p')"
  prometheus kbd --path "$PROJECT_ROOT" phase create \
    --expected-revision "$revision" --command-id "phase-create:${NEW_PHASE}" \
    --lease-id "$lease_id" --fencing-token "$fencing_token" \
    --id "$NEW_PHASE" --title "$NEW_PHASE" >/dev/null
  mutation="$(kbd_runtime_mutation_args "$PROJECT_ROOT" "phase-activate:${NEW_PHASE}")"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  lease_id="$(printf '%s\n' "$mutation" | sed -n '3p')"
  fencing_token="$(printf '%s\n' "$mutation" | sed -n '4p')"
  prometheus kbd --path "$PROJECT_ROOT" phase activate \
    --expected-revision "$revision" --command-id "phase-activate:${NEW_PHASE}" \
    --lease-id "$lease_id" --fencing-token "$fencing_token" \
    --id "$NEW_PHASE" --exact-next-work "/kbd-assess ${NEW_PHASE}" >/dev/null
  printf '\nCompleted kbd-next-phase — %s ready for /kbd-assess\n' "$NEW_PHASE"
  printf '  phase: %s\n' "$NEW_PHASE"
  printf '  goals: %s\n' "$NEW_PHASE_DIR/goals.md"
  printf '  Next:  /kbd-assess %s\n' "$NEW_PHASE"
  exit 0
fi

# Skeleton progress.json
cat > "${NEW_PHASE_DIR}/progress.json" << PROGRESS_EOF
{
  "schemaVersion": "2",
  "phase": "${NEW_PHASE}",
  "started": "${TIMESTAMP}",
  "last_updated": "${TIMESTAMP}",
  "last_updated_by": "kbd-next-phase",
  "assessment_complete": false,
  "plan_complete": false,
  "execution_dispatched": false,
  "reflection_complete": false,
  "changes_total": 0,
  "changes_completed": 0,
  "implementation_total": 0,
  "implementation_completed": 0,
  "completion": {
    "primaryCounter": "implementation",
    "implementation": {"completed": 0, "total": 0, "status": "PENDING"},
    "evidence": {"status": "NOT_TRACKED", "summary": null, "blockers": []},
    "certification": {"status": "NOT_TRACKED", "summary": null, "blockers": []},
    "publication": {"status": "NOT_TRACKED", "summary": null, "blockers": []}
  },
  "changes": []
}
PROGRESS_EOF

# ── Update current-waypoint.json ─────────────────────────────────────────
jq \
  --arg phase "$NEW_PHASE" \
  --arg previous "$CURRENT_PHASE" \
  --arg now "$TIMESTAMP" '
  del(
    .stage, .previous_phase, .last_updated, .last_updated_by,
    .exact_next_command, .fallback_command, .active_change,
    .next_pending_change, .changes_total, .changes_completed,
    .changesCompleted, .changesTotal
  ) |
  .schemaVersion = "5" |
  .phase = $phase |
  .previousPhase = $previous |
  .status = "assessment_ready" |
  .currentTask = ("run kbd-assess for " + $phase) |
  .sourceTool = "kbd-next-phase" |
  .exactNextCommand = ("/kbd-assess " + $phase) |
  .change = null |
  .nextPendingChange = null |
  .completionMetric = "implementation" |
  .implementationCompleted = 0 |
  .implementationTotal = 0 |
  .certificationStatus = "NOT_TRACKED" |
  .publicationStatus = "NOT_TRACKED" |
  .planRevision = 1 |
  .revision = ((.revision // 0) + 1) |
  .updatedAt = $now
' "$WAYPOINT_JSON" > "$WAYPOINT_JSON.tmp"
mv -f "$WAYPOINT_JSON.tmp" "$WAYPOINT_JSON"

# ── Update project.json active_phase ─────────────────────────────────────
if [[ -f "$PROJECT_JSON" ]]; then
  jq --arg phase "$NEW_PHASE" --arg now "$TIMESTAMP" '
    del(.active_phase) |
    .activePhase = $phase |
    .updatedAt = $now
  ' "$PROJECT_JSON" > "$PROJECT_JSON.tmp"
  mv -f "$PROJECT_JSON.tmp" "$PROJECT_JSON"
fi

# ── Update current-waypoint.md ────────────────────────────────────────────
cat > "$WAYPOINT_MD" << WAYPT_EOF
# Current Waypoint

**Phase**: \`${NEW_PHASE}\`
**Stage**: \`assess_pending\`
**Created**: ${TIMESTAMP}
**Previous phase**: \`${CURRENT_PHASE}\`

## Summary

New phase seeded from \`${CURRENT_PHASE}/reflection.md\`.
Goals are pre-loaded in \`.kbd-orchestrator/phases/${NEW_PHASE}/goals.md\`.
Review the goals, then run \`/kbd-assess\` to begin.

## Next action

\`\`\`
/kbd-assess ${NEW_PHASE}
\`\`\`

## References

- [goals.md](phases/${NEW_PHASE}/goals.md)
- [Previous reflection](phases/${CURRENT_PHASE}/reflection.md)
WAYPT_EOF

# ── Confirmation banner ───────────────────────────────────────────────────
PREV_GOALS_MET="${CHANGES_COMPLETED}/${CHANGES_TOTAL}"
[[ "$CHANGES_TOTAL" -eq 0 ]] && PREV_GOALS_MET="(see reflection)"

echo ""
echo "=================================================================="
printf "KBD NEXT PHASE -- %s\n" "$PROJECT_NAME"
echo ""
echo "Previous phase:  ${CURRENT_PHASE}"
echo "  Goals met:     ${PREV_GOALS_MET}"
echo ""
echo "New phase:       ${NEW_PHASE}"
echo "  Goals file:    .kbd-orchestrator/phases/${NEW_PHASE}/goals.md"
echo "  Seeded from:   ${CURRENT_PHASE}/reflection.md"
echo ""
if [[ -n "$SEED_CONTENT" ]]; then
  echo "Seeded content preview:"
  echo "$SEED_CONTENT" | head -8 | sed 's/^/  /'
  echo ""
fi
echo "Waypoint:        stage=assess_pending  next=/kbd-assess"
echo ""
echo "Run /kbd-assess to start the new phase."
echo "=================================================================="
echo ""

exit 0
