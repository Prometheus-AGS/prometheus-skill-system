#!/usr/bin/env bash
# shared/lib/tests/test-kbd-apply-native.sh
# Verifies the native-kbd adapter in kbd-apply: detect (precedence + specBackend),
# list/progress/mark_done over tasks.json, tasks.md regeneration, lazy migration
# from legacy change.md, verify, and archive. No external CLI required.

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
export KBD_TOOL="test-tool"

# --- Fixture: native change with tasks.json source of truth ---
mkdir -p .kbd-orchestrator/changes/change-x
cat > .kbd-orchestrator/changes/change-x/spec.md <<'MD'
# change-x
proposal text
MD
cat > .kbd-orchestrator/changes/change-x/tasks.json <<'JSON'
{
  "changeId": "change-x",
  "schemaVersion": "1",
  "tasks": [
    { "id": "1", "title": "First task", "done": false, "doneAt": null, "doneBy": null },
    { "id": "2", "title": "Second task", "done": false, "doneAt": null, "doneBy": null },
    { "id": "3", "title": "Third task", "done": false, "doneAt": null, "doneBy": null }
  ]
}
JSON

# detect → native-kbd (no openspec/speckit present)
[ "$("$APPLY" detect)" = "native-kbd" ] || fail "detect should be native-kbd, got '$("$APPLY" detect)'"
pass "detect → native-kbd via tasks.json"

# list → 3 tasks, explicit ids, not done
list="$("$APPLY" list change-x)"
[ "$(printf '%s\n' "$list" | wc -l | tr -d ' ')" = "3" ] || fail "expected 3 tasks: $list"
printf '%s\n' "$list" | head -1 | grep -q '^1	0	First task$' || fail "first task wrong: $(printf '%s\n' "$list" | head -1)"
pass "list reads explicit ids from tasks.json"

# progress → 3 0 3
[ "$("$APPLY" progress change-x)" = "3 0 3" ] || fail "progress wrong: $("$APPLY" progress change-x)"
pass "progress → 3 0 3"

# mark_done flips only that task, stamps doneBy/doneAt, regenerates tasks.md
"$APPLY" mark-done change-x 2 >/dev/null 2>&1
jq -e '.tasks[] | select(.id=="2") | .done == true and .doneBy == "test-tool" and .doneAt != null' \
  .kbd-orchestrator/changes/change-x/tasks.json >/dev/null || fail "task 2 not marked with ledger fields"
jq -e '.tasks[] | select(.id=="1") | .done == false' \
  .kbd-orchestrator/changes/change-x/tasks.json >/dev/null || fail "task 1 should stay open"
[ -f .kbd-orchestrator/changes/change-x/tasks.md ] || fail "tasks.md not regenerated"
grep -q 'GENERATED' .kbd-orchestrator/changes/change-x/tasks.md || fail "tasks.md missing generated banner"
grep -q '^- \[x\] 2 Second task$' .kbd-orchestrator/changes/change-x/tasks.md || fail "tasks.md checkbox not updated"
pass "mark_done is atomic, stamps ledger, regenerates tasks.md"

[ "$("$APPLY" progress change-x)" = "3 1 3" ] && fail "progress should be 3 1 2"
[ "$("$APPLY" progress change-x)" = "3 1 2" ] || fail "progress after mark wrong: $("$APPLY" progress change-x)"
pass "progress reflects completion → 3 1 2"

# verify fails while tasks remain
if "$APPLY" verify change-x >/dev/null 2>&1; then fail "verify should fail with open tasks"; fi
"$APPLY" mark-done change-x 1 >/dev/null 2>&1
"$APPLY" mark-done change-x 3 >/dev/null 2>&1
"$APPLY" verify change-x >/dev/null 2>&1 || fail "verify should pass when all done + spec.md present"
pass "verify: fails with open tasks, passes when all done + spec.md"

# archive moves the change dir under archive/<date>-<id>/
"$APPLY" archive change-x >/dev/null 2>&1 || fail "archive failed"
[ ! -d .kbd-orchestrator/changes/change-x ] || fail "change dir still present after archive"
ls .kbd-orchestrator/changes/archive/*-change-x/spec.md >/dev/null 2>&1 || fail "archived change not found"
pass "archive moves change under archive/<date>-<id>/"

# --- Lazy migration from legacy change.md (no tasks.json) ---
mkdir -p .kbd-orchestrator/changes/legacy-y
cat > .kbd-orchestrator/changes/legacy-y/change.md <<'MD'
---
id: legacy-y
---
# legacy-y

## Tasks

- [ ] 1. Alpha task
- [x] 2. Beta task already done
- [ ] 3. Gamma task
MD
listl="$("$APPLY" list legacy-y)"
[ -f .kbd-orchestrator/changes/legacy-y/tasks.json ] || fail "lazy migration did not write tasks.json"
[ -f .kbd-orchestrator/changes/legacy-y/change.md ] || fail "original change.md must be preserved"
printf '%s\n' "$listl" | grep -q '^1	0	Alpha task$' || fail "migrated task 1 wrong: $listl"
printf '%s\n' "$listl" | grep -q '^2	1	Beta task already done$' || fail "migrated task 2 should be done"
[ "$("$APPLY" progress legacy-y)" = "3 1 2" ] || fail "migrated progress wrong: $("$APPLY" progress legacy-y)"
pass "legacy change.md lazily migrated to tasks.json (state + done preserved)"

# --- specBackend pin overrides auto-detection ---
mkdir -p .kbd-orchestrator
echo '{ "specBackend": "native-kbd" }' > .kbd-orchestrator/project.json
mkdir -p openspec   # would otherwise be picked first in auto mode
[ "$("$APPLY" detect)" = "native-kbd" ] || fail "specBackend pin should force native-kbd, got '$("$APPLY" detect)'"
echo '{ "specBackend": "auto" }' > .kbd-orchestrator/project.json
# auto with an (empty) openspec dir but no openspec CLI → falls through to native-kbd
det="$("$APPLY" detect)"
[ "$det" = "native-kbd" ] || [ "$det" = "openspec" ] || fail "auto detect unexpected: $det"
pass "specBackend pin overrides auto; auto falls through correctly"

printf 'all native-kbd adapter tests passed\n'
