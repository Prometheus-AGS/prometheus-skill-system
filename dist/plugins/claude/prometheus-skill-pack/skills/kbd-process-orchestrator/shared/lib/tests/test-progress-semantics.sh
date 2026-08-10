#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
skill_root="$(cd ../../.. && pwd -P)"
# shellcheck source=/dev/null
. "$skill_root/shared/lib/progress.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

fixture="$skill_root/references/schemas/fixtures/progress/implementation-complete-evidence-pending.json"
contradictory="$skill_root/references/schemas/fixtures/progress/contradictory-legacy-counter.json"

kbd_progress_validate "$fixture" || fail "valid separated completion fixture rejected"
pass "24/24 implementation remains complete while evidence and publication are pending"

[[ "$(kbd_progress_implementation_completed "$fixture")" == "4" ]] \
  || fail "implementation completed counter is not 4"
[[ "$(kbd_progress_implementation_total "$fixture")" == "4" ]] \
  || fail "implementation total counter is not 4"
[[ "$(kbd_progress_dimension_status "$fixture" certification)" == "PENDING" ]] \
  || fail "certification must remain independently pending"
pass "helpers keep implementation and certification independent"

if kbd_progress_validate "$contradictory" >/dev/null 2>&1; then
  fail "contradictory legacy and canonical counters were accepted"
fi
pass "validator rejects a legacy counter that reopens completed implementation"

legacy="$(mktemp)"
trap 'rm -f "$legacy"' EXIT
printf '%s\n' '{"changes_total":2,"changes_completed":1,"changes":[]}' > "$legacy"
kbd_progress_validate "$legacy" || fail "legacy pre-v4 ledger rejected"
[[ "$(kbd_progress_implementation_completed "$legacy")" == "1" ]] \
  || fail "legacy completion fallback failed"
pass "pre-v4 ledgers remain backward compatible"

mutable="$(mktemp)"
trap 'rm -f "$legacy" "$mutable"' EXIT
cat > "$mutable" <<'JSON'
{
  "changes_total": 2,
  "changes_completed": 0,
  "changes": [
    {"id":"code-a","status":"IN_PROGRESS","implementation_status":"IN_PROGRESS","evidence_status":"PENDING"},
    {"id":"code-b","status":"BLOCKED","implementation_status":"COMPLETE","evidence_status":"BLOCKED"}
  ]
}
JSON
kbd_progress_mark_implementation_complete "$mutable" code-a \
  || fail "mark implementation complete helper failed"
jq -e '
  .completion.implementation == {completed:2,total:2,status:"COMPLETE"} and
  .changes_completed == 2 and .implementation_completed == 2 and
  .changes[0].evidence_status == "PENDING" and
  .changes[1].evidence_status == "BLOCKED"
' "$mutable" >/dev/null || fail "implementation mutator changed evidence or derived the wrong counter"
pass "implementation mutator is atomic and leaves evidence state untouched"

canonical="$(mktemp)"
trap 'rm -f "$legacy" "$mutable" "$canonical"' EXIT
cat > "$canonical" <<'JSON'
{
  "schemaVersion": "2",
  "phase": "phase-x",
  "last_updated": "2026-07-28T12:00:00Z",
  "last_updated_by": "codex",
  "changes_total": 2,
  "changes_completed": 1,
  "changes": [
    {"id":"code-a","status":"DONE","implementation_status":"COMPLETE"},
    {"id":"code-b","status":"PENDING","implementation_status":"PENDING"}
  ]
}
JSON
kbd_progress_validate "$canonical" || fail "canonical schema v2 ledger rejected"
jq '.changes[1].id = "code-a"' "$canonical" > "$canonical.duplicate"
if kbd_progress_validate "$canonical.duplicate" >/dev/null 2>&1; then
  fail "schema v2 accepted duplicate change IDs"
fi
rm -f "$canonical.duplicate"
pass "schema v2 requires ordered object rows with unique IDs"

printf 'all completion-semantics tests passed\n'
