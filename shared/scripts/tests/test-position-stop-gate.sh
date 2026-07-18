#!/usr/bin/env bash
# test-position-stop-gate.sh — fixture tests for shared/scripts/position-stop-gate.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GATE="$SCRIPT_DIR/../position-stop-gate.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS + 1)); }
bad()  { FAIL=$((FAIL + 1)); printf 'FAIL: %s\n%s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Isolate the soft-cap file from the real ~/.prometheus
export HOME="$TMP/home"
mkdir -p "$HOME"

# --- Fixture repo with active (non-terminal) waypoint ---
mkdir -p "$TMP/repo/.kbd-orchestrator/phases/phase-x"
cat > "$TMP/repo/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{
  "phase": "phase-x",
  "status": "execute_ready",
  "change": "change-001-demo",
  "currentTask": "do the work",
  "exactNextCommand": "/kbd-apply change-001-demo",
  "changesTotal": 4,
  "changesCompleted": 1
}
EOF
cat > "$TMP/repo/.kbd-orchestrator/phases/phase-x/progress.json" <<'EOF'
{ "changes_total": 4, "changes_completed": 1, "changes": [] }
EOF

mk_transcript() { # <path> <assistant-text>
  python3 - "$1" "$2" <<'PY'
import json, sys
path, text = sys.argv[1], sys.argv[2]
with open(path, "w") as f:
    f.write(json.dumps({"type": "user", "message": {"content": "hello"}}) + "\n")
    f.write(json.dumps({"type": "assistant",
                        "message": {"content": [{"type": "text", "text": text}]}}) + "\n")
PY
}

# 1. Missing footer → block JSON with rendered footer in reason
T1="$TMP/t1.jsonl"; mk_transcript "$T1" "All done. I finished the task."
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s1"}' "$T1" | bash "$GATE")"
RC=$?
[ "$RC" -eq 0 ] && ok || bad "block path rc" "rc=$RC"
printf '%s' "$OUT" | jq -e '.decision == "block"' >/dev/null 2>&1 && ok || bad "block decision" "$OUT"
printf '%s' "$OUT" | jq -r '.reason' 2>/dev/null | grep -q 'Position: phase-x' && ok || bad "reason carries footer" "$OUT"

# 2. Soft cap: identical stop (same session + transcript) → silent second time
OUT2="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s1"}' "$T1" | bash "$GATE")"
[ -z "$OUT2" ] && ok || bad "soft cap second call silent" "$OUT2"

# 3. Footer present without false completion language → silent
T3="$TMP/t3.jsonl"; mk_transcript "$T3" $'Progress note.\n<!-- prometheus-position -->\nPosition: phase-x | status: execute_ready\n<!-- /prometheus-position -->'
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s3"}' "$T3" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "footer present silent" "$OUT"

# 3b. Footer present but falsely claims completion while waypoint is active → block once
T3B="$TMP/t3b.jsonl"; mk_transcript "$T3B" $'All done here.\n<!-- prometheus-position -->\nPosition: phase-x | status: execute_ready\nNext: /kbd-apply change-001-demo\n<!-- /prometheus-position -->'
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s3b"}' "$T3B" | bash "$GATE")"
printf '%s' "$OUT" | jq -e '.decision == "block"' >/dev/null 2>&1 && ok || bad "false completion blocks" "$OUT"
printf '%s' "$OUT" | jq -r '.reason' 2>/dev/null | grep -q '/kbd-apply change-001-demo' && ok || bad "false completion reason carries next command" "$OUT"

# 4. stop_hook_active true → silent (loop protection)
T4="$TMP/t4.jsonl"; mk_transcript "$T4" "no footer here"
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":true,"transcript_path":"%s","session_id":"s4"}' "$T4" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "stop_hook_active silent" "$OUT"

# 5. Terminal waypoint status → silent
mkdir -p "$TMP/done/.kbd-orchestrator"
cat > "$TMP/done/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "phase-x", "status": "phase_complete" }
EOF
T5="$TMP/t5.jsonl"; mk_transcript "$T5" "no footer"
OUT="$(cd "$TMP/done" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s5"}' "$T5" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "terminal status silent" "$OUT"

# 5b. Terminal status "reflected" (reflect stage vocabulary) → silent.
# Regression: this string was NOT in the old phase_complete|reflect_complete
# set, so a reflected phase re-nagged /kbd-execute forever.
mkdir -p "$TMP/reflected/.kbd-orchestrator"
cat > "$TMP/reflected/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "phase-x", "status": "reflected", "exactNextCommand": "/kbd-new-phase" }
EOF
T5B="$TMP/t5b.jsonl"; mk_transcript "$T5B" "no footer, phase is reflected"
OUT="$(cd "$TMP/reflected" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s5b"}' "$T5B" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "reflected status silent" "$OUT"

# 5c. Terminal status "reflect_complete" (canonical) → silent
mkdir -p "$TMP/rc/.kbd-orchestrator"
cat > "$TMP/rc/.kbd-orchestrator/current-waypoint.json" <<'EOF'
{ "phase": "phase-x", "status": "reflect_complete", "exactNextCommand": "/kbd-new-phase" }
EOF
T5C="$TMP/t5c.jsonl"; mk_transcript "$T5C" "no footer"
OUT="$(cd "$TMP/rc" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s5c"}' "$T5C" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "reflect_complete status silent" "$OUT"

# 5d. Non-terminal status ("done"-adjacent but active) still gates.
# Guards against the terminal set going so broad it swallows active work.
T5D="$TMP/t5d.jsonl"; mk_transcript "$T5D" "no footer, still executing"
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s5d"}' "$T5D" | bash "$GATE")"
printf '%s' "$OUT" | jq -e '.decision == "block"' >/dev/null 2>&1 && ok || bad "active status still gates" "$OUT"

# 6. No orchestrator → silent
mkdir -p "$TMP/bare"
OUT="$(cd "$TMP/bare" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s6"}' "$T5" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "no orchestrator silent" "$OUT"

# 7. Missing/empty stdin → silent exit 0
OUT="$(cd "$TMP/repo" && printf '' | bash "$GATE")"
RC=$?
[ -z "$OUT" ] && [ "$RC" -eq 0 ] && ok || bad "empty stdin silent" "rc=$RC out=$OUT"

echo "---"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
