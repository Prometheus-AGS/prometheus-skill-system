#!/usr/bin/env bash
# shared/lib/tests/test-position-sync.sh — smoke tests for position.sh.
# Pure bash + jq. Exits non-zero on first failure.

set -uo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

# --- Fixture: top-level phase with active change ---
mkdir -p "$SANDBOX/repo/.kbd-orchestrator/phases/p1"
cat > "$SANDBOX/repo/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{
  "revision": 0,
  "phase": "p1",
  "status": "execute_ready",
  "change": "change-002-demo",
  "exactNextCommand": "/kbd-apply change-002-demo"
}
EOF
cat > "$SANDBOX/repo/.kbd-orchestrator/phases/p1/progress.json" <<'EOF'
{
  "changes_total": 6,
  "changes_completed": 2,
  "changes": [
    {"id": "change-002-demo", "status": "IN_PROGRESS", "tasks_total": 4, "tasks_done": 1}
  ]
}
EOF

# Test 1: sync produces position.json with cursor and tree
( cd "$SANDBOX/repo" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync ) \
  || fail "test 1 — sync returned non-zero"
POS="$SANDBOX/repo/.kbd-orchestrator/position.json"
[ -f "$POS" ] || fail "test 1 — position.json not written"
jq -e '.schemaVersion == "1"' "$POS" >/dev/null || fail "test 1 — schemaVersion"
jq -e '.cursor == ["p1", "change-002-demo", "task:1/4"]' "$POS" >/dev/null \
  || fail "test 1 — cursor wrong: $(jq -c '.cursor' "$POS")"
pass "sync writes cursor [phase, change, task:i/n]"

# Test 2: root node shape + change child + progress
jq -e '.root.type == "phase" and .root.id == "p1" and .root.status == "execute_ready"
       and .root.progress == {done: 2, total: 6}
       and .root.children[0] == {type:"change", id:"change-002-demo",
                                 status:"IN_PROGRESS", progress:{done:1,total:4}}' \
  "$POS" >/dev/null || fail "test 2 — tree shape wrong: $(jq -c '.root' "$POS")"
pass "root phase node carries progress and change child"

# Test 3: idempotent — derive twice, identical except updatedAt
FIRST="$(jq -c 'del(.updatedAt)' "$POS")"
( cd "$SANDBOX/repo" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync )
SECOND="$(jq -c 'del(.updatedAt)' "$POS")"
[ "$FIRST" = "$SECOND" ] || fail "test 3 — derive not idempotent"
pass "derive-twice idempotent"

# Test 4: child pointer — active node is the child, cursor has both levels
mkdir -p "$SANDBOX/repo/.kbd-orchestrator/phases/p1/children/kid"
cat > "$SANDBOX/repo/.kbd-orchestrator/phases/p1/children/kid/progress.json" <<'EOF'
{ "changes_total": 2, "changes_completed": 0,
  "changes": [{"id": "change-002-demo", "status": "PENDING", "tasks_total": 3, "tasks_done": 0}] }
EOF
jq '.childPointer = "kid"' "$SANDBOX/repo/.kbd-orchestrator/current-waypoint.json" \
  > "$SANDBOX/wp.tmp" && mv "$SANDBOX/wp.tmp" "$SANDBOX/repo/.kbd-orchestrator/current-waypoint.json"
( cd "$SANDBOX/repo" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync )
jq -e '.cursor == ["p1", "kid", "change-002-demo", "task:0/3"]' "$POS" >/dev/null \
  || fail "test 4 — child cursor wrong: $(jq -c '.cursor' "$POS")"
jq -e '.root.status == "active-parent" and .root.children[0].type == "phase"
       and .root.children[0].id == "kid"' "$POS" >/dev/null \
  || fail "test 4 — child tree wrong: $(jq -c '.root' "$POS")"
pass "child pointer nests phase node and extends cursor"

# Test 5: annotations ingest .evolver/ and .zeespec/ read-only
mkdir -p "$SANDBOX/repo/.evolver/evolutions/my-evo" "$SANDBOX/repo/.zeespec/my-subject"
( cd "$SANDBOX/repo" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync )
jq -e '.root.annotations | map(.source) | sort == ["evolver", "zeespec"]' "$POS" >/dev/null \
  || fail "test 5 — annotations wrong: $(jq -c '.root.annotations' "$POS")"
jq -e '.root.annotations[] | select(.source=="evolver") | .summary | contains("my-evo")' \
  "$POS" >/dev/null || fail "test 5 — evolver summary missing evolution name"
pass "foreign state annotated, not migrated"

# Test 5b: stale child pointers are trimmed to the longest existing path
cat > "$SANDBOX/repo/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{
  "revision": 0,
  "phase": "p1",
  "path": ["p1", "ghost-child"],
  "status": "execute_ready",
  "change": "change-002-demo"
}
EOF
( cd "$SANDBOX/repo" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync )
jq -e '.cursor == ["p1", "change-002-demo", "task:1/4"]' "$POS" >/dev/null \
  || fail "test 5b — stale child pointer should trim to existing phase, got: $(jq -c '.cursor' "$POS")"
pass "stale child pointers are trimmed from the derived cursor"

# Test 6: no orchestrator → silent no-op
( cd "$SANDBOX" && . "$SKILL_ROOT/shared/lib/position.sh" && kbd_position_sync ) \
  || fail "test 6 — must return 0 without orchestrator"
[ ! -f "$SANDBOX/.kbd-orchestrator/position.json" ] || fail "test 6 — wrote where it shouldn't"
pass "no orchestrator → no-op"

# Test 7: waypoint-render prefers a revision-matched position cursor
RENDER_LIB="$SKILL_ROOT/../../../shared/scripts/lib/waypoint-render.sh"
[ -f "$RENDER_LIB" ] || fail "test 7 — renderer lib not found at $RENDER_LIB"
out="$(cd "$SANDBOX/repo" && source "$RENDER_LIB" && waypoint_render)"
echo "$out" | grep -qF 'Position: p1 › change-002-demo › task:1/4' \
  || fail "test 7 — renderer did not use position cursor, got: $out"
pass "waypoint-render prefers revision-matched position.json cursor"

# Test 8: a newer mtime cannot make a mismatched projection authoritative
jq '.sourceRevision = 99' "$POS" > "$SANDBOX/pos.tmp" &&
  mv "$SANDBOX/pos.tmp" "$POS"
touch "$POS"
out="$(cd "$SANDBOX/repo" && source "$RENDER_LIB" && waypoint_render)"
echo "$out" | grep -qF 'Position: p1 › change-002-demo › tasks 1/4' \
  || fail "test 8 — mismatched revision should fall back to waypoint, got: $out"
pass "revision mismatch outranks filesystem mtime"

printf 'all position-sync tests passed\n'
