#!/usr/bin/env bash
# shared/lib/tests/test-kbd-child-phase.sh — kbd-new-child + kbd-next-child smoke tests.

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
NEW_PHASE="$SKILL_ROOT/skills/kbd-new-phase/kbd-new-phase.sh"
NEW_CHILD="$SKILL_ROOT/skills/kbd-new-child/kbd-new-child.sh"
NEXT_CHILD="$SKILL_ROOT/skills/kbd-next-child/kbd-next-child.sh"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

setup() {
  SANDBOX="$(mktemp -d)"
  cd "$SANDBOX"
  trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" parent-x >/dev/null 2>&1 || fail "setup: kbd-new-phase failed"
}

# --- Test 1: happy path create child ---
( setup
  "$NEW_CHILD" alpha "g1" >/dev/null 2>&1 || fail "1: kbd-new-child failed"
  [[ -d .kbd-orchestrator/phases/parent-x/children/alpha ]] || fail "1: child dir missing"
  [[ -f .kbd-orchestrator/phases/parent-x/children/alpha/goals.md ]] || fail "1: goals.md missing"
  jq -e '
    .phase == "alpha" and .parentPhase == "parent-x" and
    .completion.primaryCounter == "implementation" and
    .completion.implementation == {completed:0,total:0,status:"PENDING"} and
    .completion.evidence.status == "NOT_TRACKED"
  ' \
    .kbd-orchestrator/phases/parent-x/children/alpha/progress.json >/dev/null \
    || fail "1: progress.json fields wrong"
  jq -e '.childPhases == ["alpha"] and .childPointer == "alpha"' .kbd-orchestrator/current-waypoint.json \
    >/dev/null || fail "1: waypoint not updated"
) && pass "kbd-new-child happy path"

# --- Test 2: second child appended ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  "$NEW_CHILD" b >/dev/null 2>&1 || fail "2: second new-child failed"
  jq -e '.childPhases == ["a","b"] and .childPointer == "b"' .kbd-orchestrator/current-waypoint.json \
    >/dev/null || fail "2: second child not appended"
) && pass "second child appended, pointer moves to it"

# --- Test 3: duplicate refusal ---
( setup
  "$NEW_CHILD" dup >/dev/null 2>&1
  out="$("$NEW_CHILD" dup 2>&1)" && fail "3a: should fail"
  echo "$out" | grep -qi "already exists" || fail "3b: expected duplicate error"
) && pass "duplicate child name refused"

# --- Test 4: explicit jump ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  "$NEW_CHILD" b >/dev/null 2>&1
  "$NEXT_CHILD" a >/dev/null 2>&1 || fail "4a: explicit jump failed"
  jq -e '.childPointer == "a"' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "4b: pointer not at a"
) && pass "kbd-next-child explicit jump"

# --- Test 5: implicit advance ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  "$NEW_CHILD" b >/dev/null 2>&1
  "$NEXT_CHILD" a >/dev/null 2>&1  # set pointer to a
  "$NEXT_CHILD" >/dev/null 2>&1 || fail "5a: implicit advance failed"
  jq -e '.childPointer == "b"' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "5b: implicit advance did not move to b"
) && pass "kbd-next-child implicit advance"

# --- Test 6: refuse advance past last ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  # pointer is at "a" (only child)
  out="$("$NEXT_CHILD" 2>&1)" && fail "6a: should fail at last"
  echo "$out" | grep -qi "last child" || fail "6b: expected last-child error: $out"
) && pass "refuse implicit advance past last"

# --- Test 7: explicit unknown ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  out="$("$NEXT_CHILD" nonexistent 2>&1)" && fail "7a: should fail"
  echo "$out" | grep -qi "no such child" || fail "7b: expected unknown error"
) && pass "explicit jump to unknown name refused"

# --- Test 8: kbd-new-child without parent ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  out="$("$NEW_CHILD" orphan 2>&1)" && fail "8a: should fail"
  echo "$out" | grep -qi "no current-waypoint" || fail "8b: expected no-phase error: $out"
) && pass "kbd-new-child refuses with no active phase"

# --- Test 9: invariant — pointer in children ---
( setup
  "$NEW_CHILD" a >/dev/null 2>&1
  "$NEW_CHILD" b >/dev/null 2>&1
  ptr="$(jq -r '.childPointer' .kbd-orchestrator/current-waypoint.json)"
  jq -e --arg p "$ptr" '.childPhases | any(. == $p)' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "9: invariant violated"
) && pass "invariant: childPointer always in childPhases"

# --- Test 10: hook log entries ---
( setup
  "$NEW_CHILD" hooked >/dev/null 2>&1
  log=".kbd-orchestrator/phases/parent-x/hooks.log.jsonl"
  [[ -f "$log" ]] || fail "10a: hook log missing"
  jq -e 'select(.kind == "child" and .edge == "before" and .name == "hooked")' "$log" >/dev/null \
    || fail "10b: no child:before entry"
) && pass "kbd-new-child fires child:before in log"

printf '\nall kbd-child-phase smoke tests passed (10/10)\n'
