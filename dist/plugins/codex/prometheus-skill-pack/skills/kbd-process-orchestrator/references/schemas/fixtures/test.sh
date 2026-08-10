#!/usr/bin/env bash
# references/schemas/fixtures/test.sh — waypoint loader & chain renderer smoke tests.
#
# Pure bash + jq. No bats dependency. Exits non-zero on the first failure.

set -euo pipefail

cd "$(dirname "$0")"
ROOT="$(cd ../../.. && pwd -P)"   # skill root: fixtures → schemas → references → skill-root
# shellcheck source=/dev/null
. "$ROOT/shared/lib/waypoint.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

assert_eq() {
  local got="$1" want="$2" label="$3"
  if [[ "$got" == "$want" ]]; then pass "$label"
  else fail "$label — got=[$got] want=[$want]"; fi
}

# ---- 1. pre-schema fixture: missing fields take documented defaults ----
out="$(waypoint_load waypoint/pre-schema.json)"
echo "$out" | grep -q '^phase=phase-1-foundation$'    || fail "pre-schema: phase"
echo "$out" | grep -q '^parentPhase=$'                || fail "pre-schema: parentPhase defaults empty"
echo "$out" | grep -q '^childPhases=$'                || fail "pre-schema: childPhases defaults empty"
echo "$out" | grep -q '^childPointer=$'               || fail "pre-schema: childPointer defaults empty"
echo "$out" | grep -q '^completionMetric=implementation$' || fail "pre-schema: completionMetric default"
echo "$out" | grep -q '^implementationCompleted=0$'   || fail "pre-schema: implementationCompleted default"
echo "$out" | grep -q '^implementationTotal=0$'       || fail "pre-schema: implementationTotal default"
echo "$out" | grep -q '^certificationStatus=NOT_TRACKED$' || fail "pre-schema: certificationStatus default"
echo "$out" | grep -q '^publicationStatus=NOT_TRACKED$' || fail "pre-schema: publicationStatus default"
pass "pre-schema fixture loads with documented defaults"

# ---- 2. parent-with-children fixture ----
out="$(waypoint_load waypoint/parent-with-children.json)"
echo "$out" | grep -q '^phase=parent-phase$'          || fail "p-w-c: phase"
echo "$out" | grep -q '^childPhases=w0,w1,w2$'        || fail "p-w-c: childPhases"
echo "$out" | grep -q '^childPointer=w1$'             || fail "p-w-c: childPointer"
pass "parent-with-children fixture loads"

chain="$(waypoint_chain "" "parent-phase" "w1")"
case "${LC_ALL:-${LANG:-}}" in
  POSIX|C|C.*) want="parent-phase > w1" ;;
  *)           want=$'parent-phase \xe2\x80\xba w1' ;;
esac
assert_eq "$chain" "$want" "parent-with-children chain rendering"

# ---- 3. child-row fixture ----
out="$(waypoint_load waypoint/child-row.json)"
echo "$out" | grep -q '^phase=inner$'                 || fail "child-row: phase"
echo "$out" | grep -q '^parentPhase=outer$'           || fail "child-row: parentPhase"
pass "child-row fixture loads"

chain="$(waypoint_chain "outer" "inner" "")"
case "${LC_ALL:-${LANG:-}}" in
  POSIX|C|C.*) want="outer > inner" ;;
  *)           want=$'outer \xe2\x80\xba inner' ;;
esac
assert_eq "$chain" "$want" "child-row chain rendering"

# ---- 4. POSIX locale fallback for chain_separator ----
got_posix="$(LC_ALL=POSIX bash -c '. "'"$ROOT"'/shared/lib/waypoint.sh"; chain_separator')"
assert_eq "$got_posix" " > " "chain_separator falls back to \" > \" under LC_ALL=POSIX"

# ---- 5. expand_kbd_path ----
got="$(expand_kbd_path '${HOME}/foo')"
assert_eq "$got" "$HOME/foo" "expand_kbd_path expands \${HOME}"

# ---- 6. is_descendant ----
tmp_parent="$(mktemp -d)"
mkdir -p "$tmp_parent/child"
if is_descendant "$tmp_parent/child" "$tmp_parent"; then pass "is_descendant: child of parent"
else fail "is_descendant should match child of parent"; fi
if is_descendant "$tmp_parent" "$tmp_parent"; then fail "is_descendant should reject same path"
else pass "is_descendant: same path is NOT a descendant"; fi
rm -rf "$tmp_parent"

# ---- 7. progress completion semantics regression ----
"$ROOT/shared/lib/tests/test-progress-semantics.sh"
pass "progress completion semantics fixtures"

printf '\nall waypoint fixture tests passed\n'
