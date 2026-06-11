#!/usr/bin/env bash
# test-position-render.sh — fixture tests for shared/scripts/lib/waypoint-render.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB="$SCRIPT_DIR/../lib/waypoint-render.sh"

PASS=0
FAIL=0

assert_contains() { # <label> <haystack> <needle>
  if printf '%s' "$2" | grep -qF "$3"; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    printf 'FAIL: %s — missing %q\nOUTPUT:\n%s\n' "$1" "$3" "$2" >&2
  fi
}

assert_empty() { # <label> <value>
  if [ -z "$2" ]; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    printf 'FAIL: %s — expected empty, got:\n%s\n' "$1" "$2" >&2
  fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- Fixture: orchestrator with camelCase + stale snake_case keys ---
mkdir -p "$TMP/repo/.kbd-orchestrator/phases/phase-x"
cat > "$TMP/repo/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{
  "phase": "phase-x",
  "status": "execute_ready",
  "change": "change-002-demo",
  "currentTask": "implement the demo",
  "exactNextCommand": "/kbd-apply change-002-demo",
  "exact_next_command": "/stale-snake-command",
  "next_action": "because the plan ordered it second",
  "childPointer": null,
  "changesTotal": 6,
  "changesCompleted": 1
}
EOF
cat > "$TMP/repo/.kbd-orchestrator/phases/phase-x/progress.json" <<'EOF'
{
  "changes_total": 6,
  "changes_completed": 1,
  "changes": [
    {"id": "change-002-demo", "status": "IN_PROGRESS", "tasks_total": 3, "tasks_done": 1}
  ]
}
EOF

# 1. Render from repo root
OUT="$(cd "$TMP/repo" && source "$LIB" && waypoint_render)"
assert_contains "sentinel open" "$OUT" '<!-- prometheus-position -->'
assert_contains "sentinel close" "$OUT" '<!-- /prometheus-position -->'
assert_contains "breadcrumb phase+change" "$OUT" 'Position: phase-x › change-002-demo › tasks 1/3'
assert_contains "status" "$OUT" 'status: execute_ready'
assert_contains "progress fraction" "$OUT" 'Progress: changes 1/6'
assert_contains "last line" "$OUT" 'Last: implement the demo'
assert_contains "camelCase wins over snake" "$OUT" 'Next: /kbd-apply change-002-demo'

# 2. Render from a nested directory (walk-up)
mkdir -p "$TMP/repo/src/deep"
OUT="$(cd "$TMP/repo/src/deep" && source "$LIB" && waypoint_render)"
assert_contains "walk-up render" "$OUT" 'Position: phase-x'

# 3. Explain verbosity adds Why
OUT="$(cd "$TMP/repo" && source "$LIB" && PROMETHEUS_POSITION_VERBOSITY=explain waypoint_render)"
assert_contains "explain why line" "$OUT" 'Why: because the plan ordered it second'

# 4. Dense (default) omits Why
OUT="$(cd "$TMP/repo" && source "$LIB" && waypoint_render)"
if printf '%s' "$OUT" | grep -q '^Why:'; then
  FAIL=$((FAIL + 1)); echo "FAIL: dense mode must omit Why" >&2
else
  PASS=$((PASS + 1))
fi

# 5. snake_case fallback when camelCase absent
mkdir -p "$TMP/snake/.kbd-orchestrator/phases/p1"
cat > "$TMP/snake/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "p1", "stage": "plan_ready", "exact_next_command": "/kbd-plan p1", "current_task": "draft plan" }
EOF
OUT="$(cd "$TMP/snake" && source "$LIB" && waypoint_render)"
assert_contains "snake status fallback" "$OUT" 'status: plan_ready'
assert_contains "snake next fallback" "$OUT" 'Next: /kbd-plan p1'
assert_contains "snake last fallback" "$OUT" 'Last: draft plan'

# 6. No orchestrator → silent, exit 0
mkdir -p "$TMP/bare"
OUT="$(cd "$TMP/bare" && source "$LIB" && waypoint_render)"
RC=$?
assert_empty "no orchestrator silent" "$OUT"
[ "$RC" -eq 0 ] && PASS=$((PASS + 1)) || { FAIL=$((FAIL + 1)); echo "FAIL: no-orchestrator rc=$RC" >&2; }

# 7. Malformed waypoint → silent, exit 0
mkdir -p "$TMP/broken/.kbd-orchestrator"
echo '{ not json' > "$TMP/broken/.kbd-orchestrator/current-waypoint.json"
OUT="$(cd "$TMP/broken" && source "$LIB" && waypoint_render)"
RC=$?
assert_empty "malformed waypoint silent" "$OUT"
[ "$RC" -eq 0 ] && PASS=$((PASS + 1)) || { FAIL=$((FAIL + 1)); echo "FAIL: malformed rc=$RC" >&2; }

# 8. Child pointer renders in breadcrumb and uses child progress dir
mkdir -p "$TMP/repo2/.kbd-orchestrator/phases/parent/children/kid"
cat > "$TMP/repo2/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "parent", "childPointer": "kid", "status": "execute_ready", "exactNextCommand": "/kbd-apply c1" }
EOF
cat > "$TMP/repo2/.kbd-orchestrator/phases/parent/children/kid/progress.json" <<'EOF'
{ "changes_total": 2, "changes_completed": 1, "changes": [] }
EOF
OUT="$(cd "$TMP/repo2" && source "$LIB" && waypoint_render)"
assert_contains "child breadcrumb" "$OUT" 'Position: parent › kid'
assert_contains "child progress" "$OUT" 'Progress: changes 1/2'

# 9. position-on-prompt.sh hook: emits block + instruction, exit 0, inside fixture repo
HOOK="$SCRIPT_DIR/../position-on-prompt.sh"
OUT="$(cd "$TMP/repo" && echo '{"prompt":"hi"}' | bash "$HOOK")"
RC=$?
[ "$RC" -eq 0 ] && PASS=$((PASS + 1)) || { FAIL=$((FAIL + 1)); echo "FAIL: hook rc=$RC" >&2; }
assert_contains "hook emits block" "$OUT" '<!-- prometheus-position -->'
assert_contains "hook emits instruction" "$OUT" 'MANDATORY: begin your response with the Position line'

# 10. position-on-prompt.sh hook: silent + exit 0 outside any orchestrator
OUT="$(cd "$TMP/bare" && echo '{}' | bash "$HOOK")"
RC=$?
assert_empty "hook silent without state" "$OUT"
[ "$RC" -eq 0 ] && PASS=$((PASS + 1)) || { FAIL=$((FAIL + 1)); echo "FAIL: bare hook rc=$RC" >&2; }

echo "---"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
