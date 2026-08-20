#!/usr/bin/env bash
# shared/lib/tests/test-stage-gate.sh — smoke tests for stage-gate.sh.
# Pure bash + jq. Exits non-zero on first failure.

set -uo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

# shellcheck source=/dev/null
. "$SKILL_ROOT/shared/lib/stage-gate.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

PHASE_DIR="$SANDBOX/.kbd-orchestrator/phases/p1"
mkdir -p "$PHASE_DIR"
export KBD_PHASE_DIR="$PHASE_DIR"

# Test 1: assess (first stage) always passes
kbd_stage_gate assess || fail "test 1 — assess gate must pass with no handoffs"
pass "assess gate passes as first stage"

# Test 2: no handoffs/ dir → the directory is CREATED and the normal rules apply.
#
# CONTRACT CHANGE, 2026-08-12. This test previously asserted "legacy mode —
# warn + pass". That exemption keyed on a condition every NEW phase also meets
# (a fresh phase has no handoffs/ until something writes one), so it disabled
# the gate for precisely the phases most at risk. It contributed to an incident
# where a full phase was authored against stale canonical state.
#
# New behaviour: create handoffs/ and fall through. Here `plan` then blocks on
# the genuinely missing `assess` handoff — a real rule, not a blanket bypass.
out="$(kbd_stage_gate plan 2>&1)"
rc=$?
[ "$rc" -eq 2 ] || fail "test 2 — expected rc=2 (missing assess handoff), got rc=$rc"
[ -d "$PHASE_DIR/handoffs" ] || fail "test 2 — handoffs/ should have been created"
echo "$out" | grep -q 'Remediation: run /kbd-assess first' \
  || fail "test 2 — expected assess remediation, got: $out"
pass "absent handoffs/ is created, not treated as a bypass"

# Test 3: handoffs/ exists but assess handoff missing → plan gate blocks rc=2
mkdir -p "$PHASE_DIR/handoffs"
out="$(kbd_stage_gate plan 2>&1)"
rc=$?
[ "$rc" -eq 2 ] || fail "test 3 — expected rc=2, got rc=$rc"
echo "$out" | grep -q 'Remediation: run /kbd-assess first' \
  || fail "test 3 — expected remediation command, got: $out"
pass "missing required handoff blocks with remediation"

# Test 4: write assess handoff → plan gate passes (walks back across analyze/spec)
kbd_stage_handoff_write assess "found 3 gaps" assessment.md \
  || fail "test 4 — handoff write failed"
[ -f "$PHASE_DIR/handoffs/assess.handoff.json" ] || fail "test 4 — handoff file missing"
kbd_stage_gate plan || fail "test 4 — plan gate must pass after assess handoff"
pass "plan gate walks back across optional analyze/spec to assess handoff"

# Test 5: handoff JSON shape
jq -e '.stage == "assess" and .skipped == false and .nextStage == "analyze"
       and (.outputs | index("assessment.md") != null)
       and .summaryForNext == "found 3 gaps"' \
  "$PHASE_DIR/handoffs/assess.handoff.json" >/dev/null \
  || fail "test 5 — handoff JSON shape wrong: $(cat "$PHASE_DIR/handoffs/assess.handoff.json")"
pass "handoff JSON matches schema shape"

# Test 6: explicit skip satisfies the gate and records the reason
kbd_stage_handoff_skip analyze "trivial phase, no research needed" \
  || fail "test 6 — skip write failed"
jq -e '.skipped == true and .skipReason == "trivial phase, no research needed"' \
  "$PHASE_DIR/handoffs/analyze.handoff.json" >/dev/null \
  || fail "test 6 — skip JSON wrong"
pass "explicit skip recorded with reason"

# Test 7: execute gate requires plan handoff even when analyze skipped
out="$(kbd_stage_gate execute 2>&1)"
rc=$?
[ "$rc" -eq 2 ] || fail "test 7 — expected rc=2 (plan handoff missing), got rc=$rc"
echo "$out" | grep -q '/kbd-plan' || fail "test 7 — expected /kbd-plan remediation, got: $out"
kbd_stage_handoff_write plan "6 changes ordered" plan.md
kbd_stage_gate execute || fail "test 7 — execute gate must pass after plan handoff"
pass "execute gate requires plan handoff; passes once written"

# Test 8: reflect chain — execute handoff then reflect gate, terminal nextStage null
kbd_stage_handoff_write execute "backend native-kbd" execution.md progress.json
kbd_stage_gate reflect || fail "test 8 — reflect gate must pass"
kbd_stage_handoff_write reflect "2 deltas, next phase suggested" reflection.md
jq -e '.nextStage == null' "$PHASE_DIR/handoffs/reflect.handoff.json" >/dev/null \
  || fail "test 8 — reflect nextStage must be null"
pass "full chain assess→…→reflect; reflect nextStage is null"

# Test 9: unknown stage rejected
out="$(kbd_stage_gate bogus 2>&1)"
rc=$?
[ "$rc" -eq 2 ] || fail "test 9 — unknown stage must rc=2"
pass "unknown stage rejected"

# Test 10: phase-dir resolution from waypoint (no KBD_PHASE_DIR)
unset KBD_PHASE_DIR
mkdir -p "$SANDBOX/wp/.kbd-orchestrator/phases/p2/handoffs"
cat > "$SANDBOX/wp/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "p2" }
EOF
( cd "$SANDBOX/wp" \
  && . "$SKILL_ROOT/shared/lib/stage-gate.sh" \
  && kbd_stage_handoff_write assess "via waypoint" assessment.md \
  && kbd_stage_gate plan ) \
  || fail "test 10 — waypoint-derived phase dir failed"
[ -f "$SANDBOX/wp/.kbd-orchestrator/phases/p2/handoffs/assess.handoff.json" ] \
  || fail "test 10 — handoff not written into waypoint phase dir"
pass "phase dir derived from waypoint"

printf 'all stage-gate tests passed\n'
