#!/usr/bin/env bash
# shared/lib/tests/test-kbd-grandchild.sh — arbitrary-depth child nesting.
set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
NEW_PHASE="$SKILL_ROOT/skills/kbd-new-phase/kbd-new-phase.sh"
NEW_CHILD="$SKILL_ROOT/skills/kbd-new-child/kbd-new-child.sh"
NEXT_CHILD="$SKILL_ROOT/skills/kbd-next-child/kbd-next-child.sh"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || fail "jq required"

WP=".kbd-orchestrator/current-waypoint.json"

# --- Test 1: create a child, descend into it, create a grandchild ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" parent-x >/dev/null 2>&1 || fail "setup phase"
  "$NEW_CHILD" alpha >/dev/null 2>&1 || fail "create child alpha"
  # DESCEND into alpha: set path to [parent-x, alpha] and CLEAR childPointer.
  # (This is the entered/descended state an outer agent establishes; a future
  #  /kbd-enter-child verb would do this — for now it is an explicit waypoint op.)
  jq '.path = ["parent-x","alpha"] | .childPointer = null' "$WP" > "$WP.tmp" && mv "$WP.tmp" "$WP"
  # Spawning here NESTS under alpha.
  "$NEW_CHILD" beta >/dev/null 2>&1 || fail "create grandchild beta"
  [[ -d .kbd-orchestrator/phases/parent-x/children/alpha/children/beta ]] \
    || fail "grandchild dir not nested under alpha"
  jq -e '.path == ["parent-x","alpha","beta"]' "$WP" >/dev/null \
    || fail "path[] should be depth 3: $(jq -c .path "$WP")"
  pass "grandchild nests under alpha; path[] length 3"

  # scope.json + handoff-in.md written for the grandchild
  gdir=.kbd-orchestrator/phases/parent-x/children/alpha/children/beta
  jq -e '(.allowedWritePaths | length >= 1) and (.inheritsConstraints == true)' "$gdir/scope.json" >/dev/null \
    || fail "grandchild scope.json malformed"
  [[ -f "$gdir/handoff-in.md" ]] || fail "grandchild handoff-in.md missing"
  pass "grandchild gets scope.json + handoff-in.md"

  # The grandchild is registered on alpha's progress.json (not the waypoint top level)
  jq -e '.childPhases == ["beta"]' \
    .kbd-orchestrator/phases/parent-x/children/alpha/progress.json >/dev/null \
    || fail "alpha progress.json should list beta as a child"
  pass "grandchild registered on parent node progress.json"
)

# --- Test 2: maxChildDepth rail blocks over-nesting ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" p >/dev/null 2>&1
  echo '{ "maxChildDepth": 2 }' > .kbd-orchestrator/project.json
  "$NEW_CHILD" c1 >/dev/null 2>&1 || fail "depth-2 child should be allowed"
  # Descend into c1 (path depth 2, pointer cleared).
  jq '.path = ["p","c1"] | .childPointer = null' "$WP" > "$WP.tmp" && mv "$WP.tmp" "$WP"
  out="$("$NEW_CHILD" c2 2>&1)" && fail "depth-3 should be blocked by maxChildDepth=2"
  echo "$out" | grep -qi "maxChildDepth" || fail "expected maxChildDepth error: $out"
  pass "maxChildDepth rail blocks over-nesting"
)

# --- Test 3: top-level siblings still work (no descent) ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$NEW_PHASE" p >/dev/null 2>&1
  "$NEW_CHILD" a >/dev/null 2>&1
  "$NEW_CHILD" b >/dev/null 2>&1 || fail "second top-level sibling"
  [[ -d .kbd-orchestrator/phases/p/children/a && -d .kbd-orchestrator/phases/p/children/b ]] \
    || fail "both siblings should be top-level children"
  [[ ! -d .kbd-orchestrator/phases/p/children/a/children/b ]] \
    || fail "b must NOT be nested under a"
  jq -e '.childPhases == ["a","b"]' "$WP" >/dev/null || fail "waypoint childPhases should list both"
  pass "successive new-child calls create top-level siblings, not nesting"
)

printf 'all kbd-grandchild tests passed\n'
