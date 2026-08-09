#!/usr/bin/env bash
# evolver-seed-phase.sh — Seed a new KBD phase from an approved evolver idea
# Usage: evolver-seed-phase.sh <evolution-name> <idea-id> [--phase-name <name>]
# Creates: .kbd-orchestrator/phases/<phase-name>/{goals.md, progress.json, evolver-bridge.json}
# Updates: .kbd-orchestrator/current-waypoint.json
set -euo pipefail

EVOLUTION_NAME="${1:?Usage: evolver-seed-phase.sh <evolution-name> <idea-id> [--phase-name <name>]}"
IDEA_ID="${2:?Missing idea-id}"
PHASE_NAME_OVERRIDE=""

shift 2
while [[ $# -gt 0 ]]; do
  case "$1" in
    --phase-name) PHASE_NAME_OVERRIDE="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

EVOLVER_DIR=".evolver/${EVOLUTION_NAME}"
ARCHIVE_DIR="${EVOLVER_DIR}/archive/${IDEA_ID}"

if [ ! -d "${ARCHIVE_DIR}" ]; then
  echo "ERROR: Archive not found at ${ARCHIVE_DIR}" >&2
  exit 1
fi

MANIFEST="${ARCHIVE_DIR}/manifest.json"
if [ ! -f "${MANIFEST}" ]; then
  echo "ERROR: manifest.json not found at ${MANIFEST}" >&2
  exit 1
fi

# Read idea metadata from manifest
IDEA_TEXT=$(python3 -c "import json; d=json.load(open('${MANIFEST}')); print(d.get('text',''))")
SPEC_PATH=$(python3 -c "import json; d=json.load(open('${MANIFEST}')); print(d.get('gate3_spec_path',''))")

# Determine phase name
if [ -n "${PHASE_NAME_OVERRIDE}" ]; then
  PHASE_NAME="${PHASE_NAME_OVERRIDE}"
else
  # Generate from idea-id: remove 'idea-' prefix, kebab-case remaining
  PHASE_NAME=$(echo "${IDEA_ID}" | sed 's/^idea-//' | tr '_' '-' | tr '[:upper:]' '[:lower:]')
  PHASE_NAME="evolver-${PHASE_NAME}"
fi

# Validate phase name
if ! echo "${PHASE_NAME}" | grep -qE '^[a-z0-9][a-z0-9._-]*$'; then
  echo "ERROR: Invalid phase name '${PHASE_NAME}' — must match ^[a-z0-9][a-z0-9._-]*$" >&2
  exit 1
fi

PHASE_DIR=".kbd-orchestrator/phases/${PHASE_NAME}"

if [ -d "${PHASE_DIR}" ]; then
  echo "ERROR: Phase directory already exists at ${PHASE_DIR}" >&2
  exit 1
fi

echo "[evolver-seed-phase] Creating phase: ${PHASE_NAME}" >&2
mkdir -p "${PHASE_DIR}"

# Write goals.md
GOALS_FILE="${PHASE_DIR}/goals.md"
{
  echo "# Goals: ${PHASE_NAME}"
  echo ""
  echo "Seeded by pmpo-evolver from idea: ${IDEA_ID}"
  echo "Evolution: ${EVOLUTION_NAME}"
  echo ""
  echo "## Primary goal"
  echo ""
  echo "${IDEA_TEXT}"
  echo ""
  if [ -n "${SPEC_PATH}" ] && [ -f "${SPEC_PATH}" ]; then
    echo "## Spec"
    echo ""
    echo "See: ${SPEC_PATH}"
    echo ""
    cat "${SPEC_PATH}"
  fi
} > "${GOALS_FILE}"

RUNTIME_AUTHORITY=0
REPO_ROOT="$(pwd -P)"
RUNTIME_LIB="$REPO_ROOT/shared/scripts/lib/runtime-authority.sh"
if [ -f "$RUNTIME_LIB" ]; then
  # shellcheck source=/dev/null
  . "$RUNTIME_LIB"
fi
if command -v kbd_runtime_authoritative >/dev/null 2>&1 &&
   kbd_runtime_authoritative "$REPO_ROOT"; then
  RUNTIME_AUTHORITY=1
  prometheus kbd --path "$REPO_ROOT" phase create \
    --command-id "phase-create:${PHASE_NAME}" \
    --id "$PHASE_NAME" --title "$PHASE_NAME" >/dev/null
  prometheus kbd --path "$REPO_ROOT" phase activate \
    --command-id "phase-activate:${PHASE_NAME}" \
    --id "$PHASE_NAME" --exact-next-work "/kbd-assess ${PHASE_NAME}" >/dev/null
fi

# Write progress.json
PROGRESS_FILE="${PHASE_DIR}/progress.json"
if [ "$RUNTIME_AUTHORITY" = "0" ]; then
python3 -c "
import json
data = {
    'phase': '${PHASE_NAME}',
    'stage': 'assessment_ready',
    'changes_total': 0,
    'changes_completed': 0,
    'active_change': None,
    'next_pending_change': None,
    'assessment_complete': False,
    'plan_complete': False,
    'changes': [],
    'seeded_by': {
        'evolution_name': '${EVOLUTION_NAME}',
        'idea_id': '${IDEA_ID}'
    },
    'started': '$(date -u +%Y-%m-%d 2>/dev/null || python3 -c \"from datetime import date; print(date.today())\")'
}
print(json.dumps(data, indent=2))
" > "${PROGRESS_FILE}"
fi

# Write evolver-bridge.json
BRIDGE_FILE="${PHASE_DIR}/evolver-bridge.json"
python3 -c "
import json
data = {
    'evolution_name': '${EVOLUTION_NAME}',
    'idea_id': '${IDEA_ID}',
    'evolver_plan_path': '${EVOLVER_DIR}/plan.json',
    'seeded_from_archive': '${ARCHIVE_DIR}',
    'item_to_change_map': {}
}
print(json.dumps(data, indent=2))
" > "${BRIDGE_FILE}"

# Update current-waypoint.json
WAYPOINT_FILE=".kbd-orchestrator/current-waypoint.json"
if [ "$RUNTIME_AUTHORITY" = "0" ] && [ -f "${WAYPOINT_FILE}" ]; then
  if python3 -c "import json; json.load(open('${WAYPOINT_FILE}'))" 2>/dev/null; then
    PREV_PHASE=$(python3 -c "import json; d=json.load(open('${WAYPOINT_FILE}')); print(d.get('phase',''))")
    python3 -c "
import json
with open('${WAYPOINT_FILE}') as f:
    w = json.load(f)
w['previousPhase'] = w.get('phase', '')
w['phase'] = '${PHASE_NAME}'
w['stage'] = 'assessment_ready'
w['status'] = 'assessment_ready'
w['exact_next_command'] = '/kbd-assess ${PHASE_NAME}'
with open('${WAYPOINT_FILE}', 'w') as f:
    json.dump(w, f, indent=2)
"
    echo "[evolver-seed-phase] Updated waypoint: ${PREV_PHASE} → ${PHASE_NAME}" >&2
  fi
fi

# Update manifest with seeded phase info
python3 -c "
import json
with open('${MANIFEST}') as f:
    m = json.load(f)
m['seeded_phase'] = '${PHASE_NAME}'
m['revisit_weight'] = 0.5
with open('${MANIFEST}', 'w') as f:
    json.dump(m, f, indent=2)
"

echo "[evolver-seed-phase] Phase seeded successfully." >&2
echo ""
echo "Phase:       ${PHASE_NAME}"
echo "Goals:       ${GOALS_FILE}"
echo "Progress:    ${PROGRESS_FILE}"
echo "Bridge:      ${BRIDGE_FILE}"
echo "Next:        /kbd-assess ${PHASE_NAME}"
echo ""

printf '{"phase_name": "%s", "goals_file": "%s", "next_command": "/kbd-assess %s"}\n' \
  "${PHASE_NAME}" "${GOALS_FILE}" "${PHASE_NAME}"
