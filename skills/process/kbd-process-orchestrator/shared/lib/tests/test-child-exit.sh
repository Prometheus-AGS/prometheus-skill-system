#!/usr/bin/env bash
# shared/lib/tests/test-child-exit.sh — kbd-child-exit (enter + exit) + rollup.
set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
NEW_PHASE="$SKILL_ROOT/skills/kbd-new-phase/kbd-new-phase.sh"
NEW_CHILD="$SKILL_ROOT/skills/kbd-new-child/kbd-new-child.sh"
NEXT_CHILD="$SKILL_ROOT/skills/kbd-next-child/kbd-next-child.sh"
EXIT="$SKILL_ROOT/skills/kbd-child-exit/kbd-child-exit.sh"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || fail "jq required"
WP=".kbd-orchestrator/current-waypoint.json"

# --- Test 1: enter descends, exit rolls up + pops ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" parent-x >/dev/null 2>&1 || fail "setup phase"
  "$NEW_CHILD" alpha >/dev/null 2>&1 || fail "create alpha"

  # --enter descends into alpha (pointer is alpha after spawn)
  "$EXIT" --enter >/dev/null 2>&1 || fail "enter alpha"
  jq -e '.path == ["parent-x","alpha"] and (.childPointer == null)' "$WP" >/dev/null \
    || fail "enter: path/pointer wrong: $(jq -c '{path,childPointer}' "$WP")"
  pass "--enter descends into alpha (path depth 2, pointer cleared)"

  # Give alpha some progress + a reflection so exit is allowed
  adir=.kbd-orchestrator/phases/parent-x/children/alpha
  jq '.changes_total=3 | .changes_completed=3 | .reflect_complete=true' "$adir/progress.json" > "$adir/p.tmp" && mv "$adir/p.tmp" "$adir/progress.json"
  echo "# Reflection — alpha" > "$adir/reflection.md"

  # exit: handoff-out written, rolled up into parent, path popped
  "$EXIT" >/dev/null 2>&1 || fail "exit alpha"
  [[ -f "$adir/handoff-out.md" ]] || fail "handoff-out.md not written"
  jq -e '.path == ["parent-x"]' "$WP" >/dev/null || fail "path not popped to [parent-x]: $(jq -c .path "$WP")"
  jq -e '.children.alpha.status == "DONE" and .children.alpha.changes_completed == 3' \
    .kbd-orchestrator/phases/parent-x/progress.json >/dev/null \
    || fail "rollup wrong: $(jq -c '.children' .kbd-orchestrator/phases/parent-x/progress.json)"
  pass "exit writes handoff-out, rolls up into parent children block, pops path[]"
)

# --- Test 2: exit refused without a child reflection ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" p >/dev/null 2>&1
  "$NEW_CHILD" c >/dev/null 2>&1
  "$EXIT" --enter >/dev/null 2>&1 || fail "enter c"
  out="$("$EXIT" 2>&1)" && fail "exit without reflection should fail"
  echo "$out" | grep -qi "reflection" || fail "expected reflection error: $out"
  pass "exit refused without child reflection.md"
)

# --- Test 3: exit refused at top level (path depth 1) ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" p >/dev/null 2>&1
  out="$("$EXIT" 2>&1)" && fail "exit at top level should fail"
  echo "$out" | grep -qi "not inside a child" || fail "expected top-level error: $out"
  pass "exit refused at top level"
)

# --- Test 4: grandchild rollup chains up two levels ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" p >/dev/null 2>&1
  "$NEW_CHILD" a >/dev/null 2>&1
  "$EXIT" --enter >/dev/null 2>&1 || fail "enter a"
  "$NEW_CHILD" b >/dev/null 2>&1 || fail "create grandchild b"
  "$EXIT" --enter >/dev/null 2>&1 || fail "enter b"
  bdir=.kbd-orchestrator/phases/p/children/a/children/b
  jq '.changes_total=2 | .changes_completed=2 | .reflect_complete=true' "$bdir/progress.json" > "$bdir/p.tmp" && mv "$bdir/p.tmp" "$bdir/progress.json"
  echo "# Reflection — b" > "$bdir/reflection.md"
  "$EXIT" >/dev/null 2>&1 || fail "exit b"
  # a now has a children.b block; path is [p, a]
  jq -e '.children.b.status == "DONE"' .kbd-orchestrator/phases/p/children/a/progress.json >/dev/null \
    || fail "grandchild not rolled into a"
  jq -e '.path == ["p","a"]' "$WP" >/dev/null || fail "path should be [p,a] after exiting b"
  pass "grandchild exit rolls up into its parent child node; path popped one level"
)

printf 'all child-exit tests passed\n'
