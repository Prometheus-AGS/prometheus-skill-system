#!/usr/bin/env bash
# Regression coverage for the self-contained kbd-init payload and command policy.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SKILL_REL="skills/process/kbd-process-orchestrator/skills/kbd-init"
SKILL_DIR="${REPO_ROOT}/${SKILL_REL}"
CANONICAL_ROOT="${REPO_ROOT}/skills/process/kbd-process-orchestrator/references"

fail() {
  echo "[FAIL] $1" >&2
  exit 1
}

pass() {
  echo "[PASS] $1"
}

REFERENCE_PAIRS=(
  "${CANONICAL_ROOT}/schemas/project.template.json|${SKILL_DIR}/references/schemas/project.template.json"
  "${CANONICAL_ROOT}/constraints.md|${SKILL_DIR}/references/constraints.md"
  "${CANONICAL_ROOT}/workspace-context.md|${SKILL_DIR}/references/workspace-context.md"
)

for pair in "${REFERENCE_PAIRS[@]}"; do
  source_file="${pair%%|*}"
  bundled_file="${pair#*|}"
  [[ -f "$bundled_file" ]] || fail "bundled reference missing: ${bundled_file#"$REPO_ROOT"/}"
  cmp -s "$source_file" "$bundled_file" || fail "bundled reference drift: ${bundled_file#"$REPO_ROOT"/}"
done
[[ -f "${SKILL_DIR}/scripts/kbd-init-validate.mjs" ]] || fail "bundled validator missing"
pass "kbd-init bundles byte-identical references and its validator"

COMMAND_FILES=(
  "${REPO_ROOT}/skills/process/kbd-process-orchestrator/workflows/templates/kbd-init.md"
  "${REPO_ROOT}/.agent/workflows/kbd-init.md"
  "${REPO_ROOT}/.clinerules/workflows/kbd-init.md"
  "${REPO_ROOT}/.cursor/commands/kbd-init.md"
  "${REPO_ROOT}/.opencode/commands/kbd-init.md"
  "${REPO_ROOT}/.windsurf/workflows/kbd-init.md"
)

for command_file in "${COMMAND_FILES[@]}"; do
  [[ -f "$command_file" ]] || fail "command surface missing: ${command_file#"$REPO_ROOT"/}"
  if grep -Eq '\.agent/skills/.*/kbd-init|/Users/[^/]+/.*/kbd-init|plugins/cache/.*/kbd-init' "$command_file"; then
    fail "non-portable kbd-init path remains in ${command_file#"$REPO_ROOT"/}"
  fi
  grep -Fq "installed skill" "$command_file" || fail "installed payload contract missing from ${command_file#"$REPO_ROOT"/}"
done
pass "all generated command surfaces use the installed self-contained payload"

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
INSTALLED_SKILL="${TMP_ROOT}/installed/kbd-init"
PROJECT_ROOT="${TMP_ROOT}/project"
MISSING_VOLUME="/Volumes/kbd-init-fixture-$$"
TARGET_DIR="${MISSING_VOLUME}/cargo-target/fixture"

mkdir -p "$(dirname "$INSTALLED_SKILL")" "${PROJECT_ROOT}/.kbd-orchestrator"
cp -R "$SKILL_DIR" "$INSTALLED_SKILL"
[[ -f "${INSTALLED_SKILL}/references/schemas/project.template.json" ]] || fail "isolated payload lost project template"
[[ -f "${INSTALLED_SKILL}/references/constraints.md" ]] || fail "isolated payload lost constraints template"
[[ -f "${INSTALLED_SKILL}/references/workspace-context.md" ]] || fail "isolated payload lost workspace reference"

cat > "${PROJECT_ROOT}/.kbd-orchestrator/project.json" <<JSON
{
  "name": "kbd-init-fixture",
  "build_health_command": "CARGO_TARGET_DIR=${TARGET_DIR} cargo check --workspace --locked",
  "test_command": "CARGO_TARGET_DIR=${TARGET_DIR} cargo test --workspace --locked",
  "lint_command": "CARGO_TARGET_DIR=${TARGET_DIR} cargo clippy --workspace --locked -- -D warnings",
  "dev_command": null
}
JSON

cat > "${PROJECT_ROOT}/.kbd-orchestrator/constraints.md" <<MARKDOWN
# Fixture constraints

\`\`\`yaml
constraints:
  - id: chained-cargo
    command: 'CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p one --locked && CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p two --locked'
  - id: format-only
    command: 'cargo fmt --all -- --check'
\`\`\`
MARKDOWN

OUTPUT="$(node "${INSTALLED_SKILL}/scripts/kbd-init-validate.mjs" "$PROJECT_ROOT" 2>&1)"
grep -Fq "initialization complete; execution blocked: required path unavailable: ${MISSING_VOLUME}" <<< "$OUTPUT" || \
  fail "missing external volume did not produce an execution-blocked warning: $OUTPUT"
grep -Fq "${TARGET_DIR}" "${PROJECT_ROOT}/.kbd-orchestrator/project.json" || fail "mandatory external target was not preserved"
if rg -n 'CARGO_TARGET_DIR=(\./)?target|CARGO_TARGET_DIR=[^ ]*/project/target' "${PROJECT_ROOT}/.kbd-orchestrator" >/dev/null; then
  fail "fixture generated a local target fallback"
fi
pass "missing external volume warns, preserves configuration, and does not fall back"

cat > "${PROJECT_ROOT}/.kbd-orchestrator/constraints.md" <<'MARKDOWN'
# Invalid fixture constraints

```yaml
constraints:
  - id: bare-cargo
    command: 'cargo test -p bare --locked'
```
MARKDOWN

if node "${INSTALLED_SKILL}/scripts/kbd-init-validate.mjs" "$PROJECT_ROOT" >/dev/null 2>&1; then
  fail "bare compiling Cargo command unexpectedly passed validation"
fi
pass "incomplete mandatory command propagation fails validation"
