#!/usr/bin/env bash
# shared/lib/tests/test-kbd-apply-speckit.sh
# Verifies the Spec Kit adapter in kbd-apply: detect, list, progress, mark_done
# against a specs/<feature>/tasks.md checklist. No CLI required.

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

mkdir -p specs/my-feature
cat > specs/my-feature/tasks.md <<'MD'
# Tasks

- [ ] T001 Set up the module
- [ ] T002 Wire the handler
- [ ] T003 Add the test
MD

# detect → speckit (no openspec dir present)
[ "$("$APPLY" detect)" = "speckit" ] || fail "detect should be speckit, got '$("$APPLY" detect)'"
pass "detect → speckit via specs/*/tasks.md"

# list → 3 tasks with Txxx ids, all not done
list="$("$APPLY" list my-feature)"
[ "$(printf '%s\n' "$list" | wc -l | tr -d ' ')" = "3" ] || fail "expected 3 tasks: $list"
printf '%s\n' "$list" | head -1 | grep -q '^T001	0	Set up the module$' || fail "first task wrong: $(printf '%s\n' "$list" | head -1)"
pass "list parses Txxx ids and titles"

# progress → 3 0 3
[ "$("$APPLY" progress my-feature)" = "3 0 3" ] || fail "progress wrong: $("$APPLY" progress my-feature)"
pass "progress → 3 0 3"

# mark_done by Txxx id flips only that task
"$APPLY" mark-done my-feature T002 >/dev/null 2>&1
grep -q '^\- \[ \] T001' specs/my-feature/tasks.md || fail "T001 should stay open"
grep -q '^\- \[x\] T002' specs/my-feature/tasks.md || fail "T002 should be done"
grep -q '^\- \[ \] T003' specs/my-feature/tasks.md || fail "T003 should stay open"
pass "mark-done flips the Txxx-addressed task only"

[ "$("$APPLY" progress my-feature)" = "3 1 2" ] || fail "progress after mark wrong"
pass "progress reflects 1 complete"

printf '\nkbd-apply Spec Kit adapter tests passed\n'
