#!/usr/bin/env bash
# shared/lib/tests/test-kbd-apply-child.sh
# Verifies kbd-apply is child-aware: when childPointer is set, progress.json
# sync targets phases/<parent>/children/<child>/, and the position chain renders
# parent › child. Pure bash + jq. No spec backend needed (we test resolution).

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"
APPLY="$SKILL_ROOT/skills/kbd-apply/kbd-apply.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

command -v jq >/dev/null 2>&1 || fail "jq required"

SANDBOX="$(mktemp -d)"; trap 'rm -rf "$SANDBOX"' EXIT
cd "$SANDBOX"

# Build a waypoint with an active child loop.
mkdir -p .kbd-orchestrator/phases/parent-x/children/child-y
cat > .kbd-orchestrator/current-waypoint.json <<'JSON'
{ "phase": "parent-x", "parentPhase": null,
  "childPhases": ["child-y"], "childPointer": "child-y" }
JSON
cat > .kbd-orchestrator/phases/parent-x/children/child-y/progress.json <<'JSON'
{ "phase": "child-y", "changes": [ { "id": "c1", "tasks_done": 0, "tasks_total": 0 } ] }
JSON

# Source kbd-apply.sh in lib-only mode and call its private _phase_dir directly.
KBD_APPLY_LIB_ONLY=1
# shellcheck source=/dev/null
. "$APPLY"
resolved="$(_phase_dir)"
expected=".kbd-orchestrator/phases/parent-x/children/child-y"
[ "$resolved" = "$expected" ] \
  || fail "_phase_dir with active child should resolve to '$expected', got '$resolved'"
pass "_phase_dir resolves to the CHILD dir when childPointer is set"

# And to the parent dir when no child is active.
jq 'del(.childPointer)' .kbd-orchestrator/current-waypoint.json > wp.tmp && mv wp.tmp .kbd-orchestrator/current-waypoint.json
resolved2="$(_phase_dir)"
[ "$resolved2" = ".kbd-orchestrator/phases/parent-x" ] \
  || fail "_phase_dir without child should resolve to parent, got '$resolved2'"
pass "_phase_dir resolves to the PARENT dir when no child is active"

# Chain rendering: waypoint_chain(parent, phase, pointer) must include both.
# shellcheck source=/dev/null
. "$SKILL_ROOT/shared/lib/waypoint.sh"
chain="$(waypoint_chain "" "parent-x" "child-y")"
echo "$chain" | grep -q 'parent-x' || fail "chain missing parent: $chain"
echo "$chain" | grep -q 'child-y'  || fail "chain missing child: $chain"
pass "waypoint_chain renders full parent › child chain ($chain)"

printf '\nkbd-apply child-awareness tests passed\n'
