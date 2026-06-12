#!/usr/bin/env bash
# shared/lib/tests/test-waypoint-path.sh — tests for the v3 path[] resolver.
set -uo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"
# shellcheck source=/dev/null
. "$SKILL_ROOT/shared/lib/waypoint.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || fail "jq required"

SANDBOX="$(mktemp -d)"; trap 'rm -rf "$SANDBOX"' EXIT
cd "$SANDBOX"

# 1. kbd_node_dir at depths 1/2/3
[ "$(kbd_node_dir p0)" = ".kbd-orchestrator/phases/p0" ] || fail "node_dir depth 1"
[ "$(kbd_node_dir p0 c1)" = ".kbd-orchestrator/phases/p0/children/c1" ] || fail "node_dir depth 2"
[ "$(kbd_node_dir p0 c1 g2)" = ".kbd-orchestrator/phases/p0/children/c1/children/g2" ] || fail "node_dir depth 3"
pass "kbd_node_dir builds nested paths to depth 3"

# 2. Empty segments are skipped
[ "$(kbd_node_dir p0 "" c1)" = ".kbd-orchestrator/phases/p0/children/c1" ] || fail "node_dir skips empty segs"
pass "kbd_node_dir skips empty segments"

# 3. kbd_node_chain renders a breadcrumb with the active separator
chain="$(LC_ALL=C kbd_node_chain p0 c1 g2)"
[ "$chain" = "p0 > c1 > g2" ] || fail "node_chain (C locale) wrong: $chain"
pass "kbd_node_chain renders N-level breadcrumb"

# 4. Synthesis from a v2 waypoint (no .path): top-level only
mkdir -p .kbd-orchestrator
echo '{ "phase": "p0" }' > .kbd-orchestrator/current-waypoint.json
[ "$(_kbd_path_from_waypoint .kbd-orchestrator/current-waypoint.json)" = "p0" ] || fail "v2 synth depth 1"
[ "$(kbd_current_node_dir)" = ".kbd-orchestrator/phases/p0" ] || fail "current_node_dir v2 depth 1"
pass "v2 waypoint synthesizes [phase]"

# 5. Synthesis from v2 with childPointer
echo '{ "phase": "p0", "childPointer": "c1" }' > .kbd-orchestrator/current-waypoint.json
[ "$(_kbd_path_from_waypoint .kbd-orchestrator/current-waypoint.json)" = "p0 c1" ] || fail "v2 synth depth 2"
[ "$(kbd_current_node_dir)" = ".kbd-orchestrator/phases/p0/children/c1" ] || fail "current_node_dir v2 depth 2"
pass "v2 waypoint synthesizes [phase, childPointer]"

# 6. Explicit path[] wins (depth 3)
echo '{ "phase": "p0", "childPointer": "c1", "path": ["p0","c1","g2"] }' > .kbd-orchestrator/current-waypoint.json
[ "$(_kbd_path_from_waypoint .kbd-orchestrator/current-waypoint.json)" = "p0 c1 g2" ] || fail "explicit path[] depth 3"
[ "$(kbd_current_node_dir)" = ".kbd-orchestrator/phases/p0/children/c1/children/g2" ] || fail "current_node_dir path[] depth 3"
pass "explicit path[] resolves to depth 3"

# 7. waypoint_load emits the synthesized path field
out="$(echo '{ "phase":"p0","childPointer":"c1" }' > wp.json && waypoint_load wp.json)"
echo "$out" | grep -q '^path=p0,c1$' || fail "waypoint_load path emission: $out"
pass "waypoint_load emits path= (synthesized)"

# 8. waypoint_load path emission prefers explicit path[]
out="$(echo '{ "phase":"p0","path":["p0","c1","g2"] }' > wp2.json && waypoint_load wp2.json)"
echo "$out" | grep -q '^path=p0,c1,g2$' || fail "waypoint_load explicit path: $out"
pass "waypoint_load prefers explicit path[]"

printf 'all waypoint-path tests passed\n'
