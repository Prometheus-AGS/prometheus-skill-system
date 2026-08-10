#!/usr/bin/env bash
# smoke tests for kbd-goal-start.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$SCRIPT_DIR/kbd-goal-start.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS + 1)); }
bad()  { FAIL=$((FAIL + 1)); printf 'FAIL: %s\n%s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

REPO="$TMP/repo"
mkdir -p "$REPO/.kbd-orchestrator"

# Stub discovery script so the test stays local and deterministic.
mkdir -p "$REPO/scripts"
cat > "$REPO/scripts/kbd-goal-discover.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$REPO/scripts/kbd-goal-discover.sh"

run_case() {
  local tool="$1" slug="$2"
  TOOL="$tool" KBD_GOAL_REPO_ROOT="$REPO" "$SCRIPT" "Ship resilient loop" --slug "$slug" >/dev/null 2>&1
}

# 1. Claude Code persists a supervised native-goal contract.
run_case "claude-code" "claude-loop" || bad "claude-code goal create failed"
GOAL="$REPO/.kbd-orchestrator/goals/claude-loop/goal.json"
CTRL="$REPO/.kbd-orchestrator/goals/claude-loop/CONTROL.md"
[ -f "$GOAL" ] && ok || bad "claude goal.json missing"
[ -f "$CTRL" ] && ok || bad "claude CONTROL.md missing"
jq -e '
  .loop_controller.owner == "kbd" and
  .loop_controller.tool == "claude-code" and
  .loop_controller.mode == "native-goal-supervised" and
  .loop_controller.native_goal_supported == true and
  .loop_controller.evaluator_mode == "native-goal+position-stop-gate" and
  .loop_controller.stop_guard == "position-stop-gate"
' "$GOAL" >/dev/null 2>&1 && ok || bad "claude loop_controller fields wrong" "$(cat "$GOAL")"
grep -q 'mode: native-goal-supervised' "$CTRL" && ok || bad "claude CONTROL.md missing mode" "$(cat "$CTRL")"

# 2. Kimi persists queue-supervised evaluator state.
run_case "kimi" "kimi-loop" || bad "kimi goal create failed"
GOAL="$REPO/.kbd-orchestrator/goals/kimi-loop/goal.json"
jq -e '
  .loop_controller.tool == "kimi" and
  .loop_controller.mode == "queue-goal-supervised" and
  .loop_controller.native_goal_supported == false and
  .loop_controller.evaluator_mode == "kbd-goal-check"
' "$GOAL" >/dev/null 2>&1 && ok || bad "kimi loop_controller fields wrong" "$(cat "$GOAL")"

echo "---"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
