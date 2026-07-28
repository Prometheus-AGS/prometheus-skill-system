#!/usr/bin/env bash
# Regression coverage for the self-contained kbd-next-phase skill payload.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SKILL_REL="skills/process/kbd-process-orchestrator/skills/kbd-next-phase"
SKILL_DIR="${REPO_ROOT}/${SKILL_REL}"
CANONICAL_SCRIPT="${SKILL_DIR}/scripts/kbd-next-phase.sh"
CODEX_SCRIPT="${REPO_ROOT}/.codex/skills/kbd-next-phase/scripts/kbd-next-phase.sh"
COMMAND_PATH="${SKILL_REL}/scripts/kbd-next-phase.sh"

fail() {
  echo "[FAIL] $1" >&2
  exit 1
}

pass() {
  echo "[PASS] $1"
}

[[ -x "$CANONICAL_SCRIPT" ]] || fail "bundled helper is missing or not executable"
[[ -x "$CODEX_SCRIPT" ]] || fail "standalone Codex helper is missing or not executable"
cmp -s "$CANONICAL_SCRIPT" "$CODEX_SCRIPT" || fail "standalone Codex helper differs from canonical helper"
pass "installed skill payloads contain the executable helper"

COMMAND_FILES=(
  "${REPO_ROOT}/skills/process/kbd-process-orchestrator/workflows/templates/kbd-next-phase.md"
  "${REPO_ROOT}/.agent/workflows/kbd-next-phase.md"
  "${REPO_ROOT}/.clinerules/workflows/kbd-next-phase.md"
  "${REPO_ROOT}/.cursor/commands/kbd-next-phase.md"
  "${REPO_ROOT}/.opencode/commands/kbd-next-phase.md"
  "${REPO_ROOT}/.windsurf/workflows/kbd-next-phase.md"
)

for command_file in "${COMMAND_FILES[@]}"; do
  grep -Fq "$COMMAND_PATH" "$command_file" || fail "bundled helper path missing from ${command_file#"$REPO_ROOT"/}"
  if grep -Fq '/shared/scripts/kbd-next-phase.sh' "$command_file"; then
    fail "legacy unbundled helper path remains in ${command_file#"$REPO_ROOT"/}"
  fi
done
pass "generated command surfaces reference the bundled helper"

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
PLUGIN_ROOT="${TMP_ROOT}/plugin"
PROJECT_ROOT="${TMP_ROOT}/project"
INSTALLED_SKILL="${PLUGIN_ROOT}/${SKILL_REL}"

mkdir -p "$(dirname "$INSTALLED_SKILL")" "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-one"
cp -R "$SKILL_DIR" "$INSTALLED_SKILL"
[[ -x "${INSTALLED_SKILL}/scripts/kbd-next-phase.sh" ]] || fail "copied plugin payload lost executable helper"

cat > "${PROJECT_ROOT}/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{
  "phase": "phase-one",
  "stage": "reflect_complete",
  "changes_total": 2,
  "changes_completed": 2
}
JSON

cat > "${PROJECT_ROOT}/.kbd-orchestrator/project.json" <<'JSON'
{
  "name": "packaging-test",
  "active_phase": "phase-one",
  "status": "reflected"
}
JSON

cat > "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-one/reflection.md" <<'MARKDOWN'
# Reflection

## Recommended Next Phase

Build the bundled helper regression and preserve the reflection seed.
MARKDOWN

(
  cd "$PROJECT_ROOT"
  bash "${INSTALLED_SKILL}/scripts/kbd-next-phase.sh" phase-two >/dev/null
)

[[ -f "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-two/goals.md" ]] || fail "bundled helper did not create goals.md"
[[ -f "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-two/progress.json" ]] || fail "bundled helper did not create progress.json"
grep -Fq 'preserve the reflection seed' "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-two/goals.md" || fail "reflection seed was not copied into goals.md"
python3 - "${PROJECT_ROOT}/.kbd-orchestrator/current-waypoint.json" <<'PY'
import json
import sys

waypoint = json.load(open(sys.argv[1]))
assert waypoint["phase"] == "phase-two"
assert waypoint["schemaVersion"] == "5"
assert waypoint["status"] == "assessment_ready"
assert waypoint["exactNextCommand"] == "/kbd-assess phase-two"
assert "stage" not in waypoint
assert "exact_next_command" not in waypoint
PY
pass "bundled helper runs successfully from an isolated installed-plugin layout"

(
  cd "$PROJECT_ROOT"
  bash "${REPO_ROOT}/shared/scripts/kbd-next-phase.sh" phase-three >/dev/null
)
[[ -f "${PROJECT_ROOT}/.kbd-orchestrator/phases/phase-three/goals.md" ]] || fail "compatibility wrapper did not forward to the bundled helper"
pass "legacy shared-script path forwards to the bundled helper"
