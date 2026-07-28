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

# 1. Missing footer → stop remains allowed and an advisory is recorded
T1="$TMP/t1.jsonl"; mk_transcript "$T1" "All done. I finished the task."
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s1"}' "$T1" | bash "$GATE")"
RC=$?
[ "$RC" -eq 0 ] && ok || bad "advisory path rc" "rc=$RC"
[ -z "$OUT" ] && ok || bad "operator stop is never blocked" "$OUT"
grep -q 'Position: phase-x' "$HOME/.prometheus/position-stop-advisories.log" 2>/dev/null && ok || bad "advisory carries footer"

# 2. Stable cap: a growing transcript at the same state does not create a
# second advisory.
printf '%s\n' '{"type":"user","message":{"content":"another turn"}}' >> "$T1"
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"still no footer"}]}}' >> "$T1"
OUT2="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s1"}' "$T1" | bash "$GATE")"
[ -z "$OUT2" ] && ok || bad "soft cap second call silent" "$OUT2"
[ "$(wc -l < "$HOME/.prometheus/position-stop-advisories.txt" | tr -d ' ')" = "1" ] && ok || bad "growing transcript keeps one advisory"

# 3. Footer present without false completion language → silent
T3="$TMP/t3.jsonl"; mk_transcript "$T3" $'Progress note.\n<!-- prometheus-position -->\nPosition: phase-x | status: execute_ready\n<!-- /prometheus-position -->'
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s3"}' "$T3" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "footer present silent" "$OUT"

# 3b. Ordinary completion prose is never interpreted as control state
T3B="$TMP/t3b.jsonl"; mk_transcript "$T3B" $'All done here.\n<!-- prometheus-position -->\nPosition: phase-x | status: execute_ready\nNext: /kbd-apply change-001-demo\n<!-- /prometheus-position -->'
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s3b"}' "$T3B" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "completion prose does not block" "$OUT"

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

# 5d. Non-terminal status records an advisory but still permits stop.
T5D="$TMP/t5d.jsonl"; mk_transcript "$T5D" "no footer, still executing"
OUT="$(cd "$TMP/repo" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s5d"}' "$T5D" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "active status does not block" "$OUT"

# 5e. Suspended lifecycle states never steer.
for state in pause_requested paused blocked suspended; do
  mkdir -p "$TMP/$state/.kbd-orchestrator"
  printf '{"phase":"phase-x","status":"%s"}\n' "$state" > "$TMP/$state/.kbd-orchestrator/current-waypoint.json"
  OUT="$(cd "$TMP/$state" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s-%s"}' "$T5D" "$state" | bash "$GATE")"
  [ -z "$OUT" ] && ok || bad "$state status silent" "$OUT"
done

# 5f. Emergency PAUSE wins even when waypoint JSON is malformed.
mkdir -p "$TMP/emergency/.kbd-orchestrator"
printf '{broken\n' > "$TMP/emergency/.kbd-orchestrator/current-waypoint.json"
: > "$TMP/emergency/.kbd-orchestrator/PAUSE"
OUT="$(cd "$TMP/emergency" && printf '{"stop_hook_active":false,"transcript_path":"%s","session_id":"s-emergency"}' "$T5D" | bash "$GATE")"
[ -z "$OUT" ] && ok || bad "emergency pause with malformed state silent" "$OUT"

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
