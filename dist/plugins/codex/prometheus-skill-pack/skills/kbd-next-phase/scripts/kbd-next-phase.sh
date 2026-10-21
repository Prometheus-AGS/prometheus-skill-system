#!/usr/bin/env bash
# kbd-next-phase.sh — Seed and initialize the next KBD phase from reflection
#
# Usage: kbd-next-phase.sh [new-phase-name] [--authorize-successor <manifest> --signature <sig>]
#   If no argument: extracts suggested name from reflection.md "Recommended Next Phase" section.
#   If argument provided: uses it as the new phase name (normalized to kebab-case).
#
# Contract-bound phases (issue #192): closing and continuing are separate.
#   - the current phase must already be CLOSED in canonical state
#     (kbd-phase-close-eval.sh records that); otherwise exit 2, nothing written;
#   - a successor starts only with a signed start-successor manifest
#     (--authorize-successor/--signature; generate it with
#     shared/scripts/kbd-scope-authority-template.sh start-successor); without
#     it exit 4, nothing written; the successor id and title come from the
#     manifest, and the runtime defines + activates it in one event;
#   - a rerun after the successor started reports ALREADY-STARTED (exit 0);
#   - goals.md is written only after canonical success and is labelled as
#     reflection proposals, never adopted scope.
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
#   .kbd-orchestrator/project.json                         (activePhase updated)

set -euo pipefail

PROPOSED_NAME=""
AUTH_MANIFEST=""
AUTH_SIGNATURE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --authorize-successor) AUTH_MANIFEST="${2:?--authorize-successor needs a manifest path}"; shift 2 ;;
    --signature) AUTH_SIGNATURE="${2:?--signature needs a signature path}"; shift 2 ;;
    -*) echo "[kbd-next-phase] unknown flag: $1" >&2; exit 1 ;;
    *) [[ -z "$PROPOSED_NAME" ]] || { echo "[kbd-next-phase] unexpected argument: $1" >&2; exit 1; }
       PROPOSED_NAME="$1"; shift ;;
  esac
done
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
PHASE_SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
LOCAL_ORCHESTRATOR_ROOT="$(cd "$PHASE_SKILL_DIR/../.." && pwd -P)"
if [ ! -f "$LOCAL_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" ]; then
  LOCAL_ORCHESTRATOR_ROOT="$PHASE_SKILL_DIR/../kbd-process-orchestrator"
fi
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$LOCAL_ORCHESTRATOR_ROOT}"
export KBD_ORCHESTRATOR_ROOT
hooks_avail=0
if [[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" ]]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
  hooks_avail=1
fi

# ── Validate waypoint exists ──────────────────────────────────────────────
if [[ ! -f "$WAYPOINT_JSON" ]]; then
  echo ""
  echo "[kbd-next-phase] ERROR: .kbd-orchestrator/current-waypoint.json not found."
  echo "  Run /kbd-init first to initialize KBD for this project."
  exit 1
fi

# ── Read current waypoint ─────────────────────────────────────────────────
CURRENT_PHASE="$(python3 -c "import json; d=json.load(open('$WAYPOINT_JSON')); print(d.get('activePhaseId') or d.get('activePhase') or d.get('phase') or 'unknown')" 2>/dev/null || echo "unknown")"
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

# ── Canonical runtime libraries + contract-bound closure authority ─────────
SKILL_ROOT="$KBD_ORCHESTRATOR_ROOT"
for lib in runtime-authority.sh bottleneck-guard.sh phase-disposition.sh; do
  if [[ -f "$SKILL_ROOT/shared/lib/$lib" ]]; then
    # shellcheck source=/dev/null
    . "$SKILL_ROOT/shared/lib/$lib"
  fi
done
RUNTIME_AUTHORITATIVE=0
if command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "$PROJECT_ROOT"; then
  RUNTIME_AUTHORITATIVE=1
fi
CURRENT_DISPOSITION=""
REQUESTED_SUCCESSOR="$PROPOSED_NAME"
if [[ -z "$REQUESTED_SUCCESSOR" && -n "$AUTH_MANIFEST" && -f "$AUTH_MANIFEST" ]]; then
  REQUESTED_SUCCESSOR="$(jq -r '.successorPhaseId // empty' "$AUTH_MANIFEST" 2>/dev/null || true)"
fi
if [[ "$RUNTIME_AUTHORITATIVE" == 1 && -n "$REQUESTED_SUCCESSOR" && "$REQUESTED_SUCCESSOR" == "$CURRENT_PHASE" ]]; then
  # A rerun after the authorized successor started: the active phase already
  # is that successor. Report it instead of treating it as a new request.
  predecessor="$(kbd_runtime_status_json "$PROJECT_ROOT" 2>/dev/null | jq -r --arg s "$REQUESTED_SUCCESSOR" \
    '[(.phaseClosure.phases // {}) | to_entries[] | select(.value.successor.phaseId == $s) | .key] | first // empty')"
  if [[ -n "$predecessor" ]]; then
    echo "[kbd-next-phase] ALREADY-STARTED: successor '${REQUESTED_SUCCESSOR}' of '${predecessor}' was already started; no new events."
    printf '  Next:  /kbd-assess %s\n' "$REQUESTED_SUCCESSOR"
    exit 0
  fi
fi
if [[ "$RUNTIME_AUTHORITATIVE" == 1 ]]; then
  command -v kbd_phase_disposition_json >/dev/null 2>&1 \
    || { echo "[kbd-next-phase] phase-disposition library is missing (broken install)" >&2; exit 1; }
  CURRENT_DISPOSITION="$(kbd_phase_disposition_json "$PROJECT_ROOT" "$CURRENT_PHASE")" \
    || { echo "[kbd-next-phase] canonical phase disposition for '$CURRENT_PHASE' is unreadable; nothing was written" >&2; exit 1; }
fi
if [[ -n "$CURRENT_DISPOSITION" ]] && [[ "$(printf '%s' "$CURRENT_DISPOSITION" | jq -r '.contractBound')" == "true" ]]; then
  token="$(printf '%s' "$CURRENT_DISPOSITION" | jq -r '.disposition')"
  if [[ "$token" != "CLOSED" ]]; then
    echo "[kbd-next-phase] NOT-CLOSED: phase '${CURRENT_PHASE}' disposition is ${token}; nothing was written."
    echo "  A contract-bound phase closes only through its terminal evaluator:"
    echo "    bash shared/scripts/kbd-phase-close-eval.sh --phase ${CURRENT_PHASE}"
    exit 2
  fi
  started="$(printf '%s' "$CURRENT_DISPOSITION" | jq -r '.successor.phaseId // empty')"
  ALREADY_STARTED="$started"
  if [[ -z "$started" && ( -z "$AUTH_MANIFEST" || -z "$AUTH_SIGNATURE" ) ]]; then
    echo "[kbd-next-phase] SUCCESSOR-UNAUTHORIZED: '${CURRENT_PHASE}' is CLOSED; closure alone never starts a successor. Nothing was written."
    echo "  An operator signs a start-successor manifest naming the successor:"
    echo "    bash shared/scripts/kbd-scope-authority-template.sh start-successor --phase ${CURRENT_PHASE} --successor <id> --title '<title>' > successor.json"
    echo "    ssh-keygen -Y sign -f <operator-key> -n prometheus-kbd-scope-amendment successor.json"
    echo "  then: kbd-next-phase.sh --authorize-successor successor.json --signature successor.json.sig"
    exit 4
  fi
  [[ -n "$started" ]] || BOUND_SUCCESSOR=1
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

# goals.md is written only after the canonical writes succeed, so a failed
# start leaves no orphan successor directory and a rerun stays idempotent.
write_goals() {
  mkdir -p "$NEW_PHASE_DIR"
  cat > "${NEW_PHASE_DIR}/goals.md" << GOALS_EOF
# Goals: ${NEW_PHASE}

Seeded from: \`${CURRENT_PHASE}/reflection.md\`
Created: ${TIMESTAMP}

## Reflection proposals — not adopted scope

These are suggestions carried forward from the previous reflection. They are
not requirements: blocking scope exists only once this phase's contract is
adopted (and any later change is a signed amendment).

${SEED_CONTENT:-"(No seed content found in reflection -- add goals manually before running /kbd-assess)"}

---

## Instructions

Review and refine the goals above before running \`/kbd-assess\`.
Add, remove, or clarify as needed. When ready:

\`\`\`
/kbd-assess ${NEW_PHASE}
\`\`\`
GOALS_EOF
}

if [[ -n "${ALREADY_STARTED:-}" ]]; then
  NEW_PHASE="$ALREADY_STARTED"
  NEW_PHASE_DIR="${KBD_DIR}/phases/${NEW_PHASE}"
  [[ -f "$NEW_PHASE_DIR/goals.md" ]] || write_goals
  echo "[kbd-next-phase] ALREADY-STARTED: successor '${NEW_PHASE}' of '${CURRENT_PHASE}' was already started; no new events."
  printf '  Next:  /kbd-assess %s\n' "$NEW_PHASE"
  exit 0
fi

# ── Determine new phase name ───────────────────────────────────────────────
SUCCESSOR_TITLE=""
if [[ "${BOUND_SUCCESSOR:-0}" == 1 ]]; then
  manifest_raw="$(cat "$AUTH_MANIFEST")" || { echo "[kbd-next-phase] cannot read $AUTH_MANIFEST" >&2; exit 1; }
  NEW_PHASE="$(printf '%s' "$manifest_raw" | jq -r '.successorPhaseId // empty')"
  SUCCESSOR_TITLE="$(printf '%s' "$manifest_raw" | jq -r '.successorTitle // empty')"
  [[ -n "$NEW_PHASE" && -n "$SUCCESSOR_TITLE" ]] \
    || { echo "[kbd-next-phase] SUCCESSOR-UNAUTHORIZED: manifest names no successorPhaseId/successorTitle" >&2; exit 4; }
  if [[ -n "$PROPOSED_NAME" && "$PROPOSED_NAME" != "$NEW_PHASE" ]]; then
    echo "[kbd-next-phase] SUCCESSOR-UNAUTHORIZED: '${PROPOSED_NAME}' is not the authorized successor '${NEW_PHASE}'" >&2
    exit 4
  fi
elif [[ -n "$PROPOSED_NAME" ]]; then
  NEW_PHASE="$PROPOSED_NAME"
elif [[ -n "$SEED_PHASE_NAME" ]]; then
  NEW_PHASE="${SEED_PHASE_NAME}-${DATE_SLUG}"
else
  NEW_PHASE="phase-next-${DATE_SLUG}"
fi

# Normalize: lowercase, spaces -> hyphens, strip non-[a-z0-9-] chars
NEW_PHASE="$(echo "$NEW_PHASE" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')"
[[ -n "$SUCCESSOR_TITLE" ]] || SUCCESSOR_TITLE="$NEW_PHASE"

# ── Guard: phase must not already exist ───────────────────────────────────
NEW_PHASE_DIR="${KBD_DIR}/phases/${NEW_PHASE}"
if [[ -d "$NEW_PHASE_DIR" ]]; then
  echo ""
  echo "[kbd-next-phase] ERROR: Phase '${NEW_PHASE}' already exists:"
  echo "  ${NEW_PHASE_DIR}"
  echo "  Choose a different name or resume it with: /kbd-assess ${NEW_PHASE}"
  exit 1
fi

node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" refresh --project "$PROJECT_ROOT" --timeout-ms 120000 \
  || { echo "[kbd-next-phase] OpenSpec refresh failed; phase was not started" >&2; exit 1; }

if [[ "$RUNTIME_AUTHORITATIVE" == 1 ]]; then
  export KBD_BOTTLENECK_PATH="$PROJECT_ROOT"
  guard_enabled=false
  if command -v kbd_bottleneck_active >/dev/null 2>&1 && kbd_bottleneck_active; then
    guard_enabled=true
  fi
  if [[ "${BOUND_SUCCESSOR:-0}" == 1 ]]; then
    # One typed event closes nothing and starts exactly the authorized scope.
    if ! start_out="$(prometheus kbd --path "$PROJECT_ROOT" phase successor start \
        --predecessor "$CURRENT_PHASE" --id "$NEW_PHASE" --title "$SUCCESSOR_TITLE" \
        --exact-next-work "/kbd-assess ${NEW_PHASE}" \
        --authorization "$AUTH_MANIFEST" --signature "$AUTH_SIGNATURE" 2>&1)"; then
      echo "[kbd-next-phase] SUCCESSOR-UNAUTHORIZED: the runtime refused the successor; nothing was written." >&2
      printf '%s\n' "$start_out" >&2
      exit 4
    fi
    if [[ "$hooks_avail" == 1 ]]; then
      (cd "$PROJECT_ROOT" && kbd_hooks_fire phase after "$CURRENT_PHASE" 1 1) \
        || echo "[kbd-next-phase] phase completion hook failed" >&2
    fi
  else
    canonical_state="$(kbd_runtime_status_json "$PROJECT_ROOT")" \
      || { echo "[kbd-next-phase] canonical status read failed" >&2; exit 1; }
    current_phase_status="$(printf '%s' "$canonical_state" | jq -r --arg phase "$CURRENT_PHASE" '.phases[$phase].status // "missing"')"
    [[ "$current_phase_status" != "missing" ]] \
      || { echo "[kbd-next-phase] current phase '$CURRENT_PHASE' is absent from canonical state" >&2; exit 1; }
    completed_guard=""
    if [[ "$current_phase_status" != "complete" ]]; then
      if [[ "$guard_enabled" == true ]]; then
        kbd_bottleneck_evaluate phase after "$CURRENT_PHASE" 1 >/dev/null \
          || { echo "[kbd-next-phase] current phase completion precommit blocked" >&2; exit 1; }
      fi
      prometheus kbd --path "$PROJECT_ROOT" phase transition \
        --command-id "phase-complete:${CURRENT_PHASE}" \
        --id "$CURRENT_PHASE" --status complete >/dev/null
      if [[ "$guard_enabled" == true ]]; then
        completed_guard="$(kbd_bottleneck_evaluate phase after "$CURRENT_PHASE" 0)" \
          || { echo "[kbd-next-phase] current phase completion postcommit blocked" >&2; exit 1; }
      fi
      if [[ "$hooks_avail" == 1 ]]; then
        (cd "$PROJECT_ROOT" && kbd_hooks_fire phase after "$CURRENT_PHASE" 1 1) \
          || echo "[kbd-next-phase] phase completion hook failed" >&2
      fi
    fi
    prometheus kbd --path "$PROJECT_ROOT" phase create \
      --command-id "phase-create:${NEW_PHASE}" \
      --id "$NEW_PHASE" --title "$NEW_PHASE" >/dev/null
    if [[ "$guard_enabled" == true ]]; then
      kbd_bottleneck_evaluate phase before "$NEW_PHASE" 1 >/dev/null \
        || { echo "[kbd-next-phase] next phase start precommit blocked" >&2; exit 1; }
    fi
    prometheus kbd --path "$PROJECT_ROOT" phase activate \
      --command-id "phase-activate:${NEW_PHASE}" \
      --id "$NEW_PHASE" --exact-next-work "/kbd-assess ${NEW_PHASE}" >/dev/null
    prometheus kbd --path "$PROJECT_ROOT" phase transition \
      --command-id "phase-start:${NEW_PHASE}" \
      --id "$NEW_PHASE" --status in-progress >/dev/null
    [[ -z "$completed_guard" ]] || kbd_bottleneck_print_signal "$completed_guard"
  fi
  if [[ "$guard_enabled" == true ]]; then
    starting_guard="$(kbd_bottleneck_evaluate phase before "$NEW_PHASE" 0)" \
      || { echo "[kbd-next-phase] next phase start postcommit blocked; record it with: prometheus kbd --path . guard evaluate --boundary phase --edge before --subject ${NEW_PHASE} --json" >&2; exit 1; }
    kbd_bottleneck_print_signal "$starting_guard"
  fi
  write_goals
  printf '\nCompleted kbd-next-phase — %s ready for /kbd-assess\n' "$NEW_PHASE"
  printf '  phase: %s\n' "$NEW_PHASE"
  printf '  goals: %s\n' "$NEW_PHASE_DIR/goals.md"
  printf '  Next:  /kbd-assess %s\n' "$NEW_PHASE"
  exit 0
fi

write_goals

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
  .path = [$phase] |
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

# ── Update project.json activePhase ──────────────────────────────────────
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
