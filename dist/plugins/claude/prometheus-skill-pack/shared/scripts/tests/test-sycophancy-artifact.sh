#!/usr/bin/env bash
# test-sycophancy-artifact.sh — tests for sycophancy-check-artifact.sh,
# the extracted lib, and the pipeline-enforce reflect_gate rule.
# Uses a PATH-shimmed fake `sycophancy-correction` so no real binary is needed.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT="$SCRIPT_DIR/../sycophancy-check-artifact.sh"
ENFORCE="$SCRIPT_DIR/../pipeline-enforce.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"; mkdir -p "$HOME"

# --- Fake MCP binary: emits a JSON-RPC response whose embedded score is read
#     from the FAKE_SCORE env var. Speaks just enough of the protocol. ---
BIN="$TMP/bin"; mkdir -p "$BIN"
cat > "$BIN/sycophancy-correction" <<'FAKE'
#!/usr/bin/env bash
cat >/dev/null 2>&1 || true   # drain stdin
# Emit a valid JSON-RPC tools/call response; build the nested JSON with python
# so escaping is correct regardless of the score value.
python3 - "${FAKE_SCORE:-0.0}" <<'PY'
import sys, json
score = float(sys.argv[1])
inner = json.dumps({"sycophancy_score": score, "classifications": []})
print(json.dumps({"jsonrpc":"2.0","id":1,"result":{}}))
print(json.dumps({"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":inner}]}}))
PY
FAKE
chmod +x "$BIN/sycophancy-correction"
export PATH="$BIN:$PATH"

# --- Fixture phase + artifact ---
ROOT="$TMP/proj"; PHASE="$ROOT/.kbd-orchestrator/phases/p1"
mkdir -p "$PHASE"
echo '{ "phase": "p1", "changes_total": 1, "changes_completed": 1 }' > "$PHASE/progress.json"
echo "# Reflection" > "$PHASE/reflection.md"
echo '{ "phase": "p1" }' > "$ROOT/.kbd-orchestrator/current-waypoint.json"

call_artifact() { # <score> → rc ; mutates progress.json
  printf '{"tool_input":{"file_path":"%s"}}' "$PHASE/reflection.md" \
    | FAKE_SCORE="$1" bash "$ARTIFACT" >/dev/null 2>&1
  echo $?
}

gate() { jq -r '.reflect_gate // "none"' "$PHASE/progress.json"; }

# 1. Clean artifact (score 0.0) → exit 0, gate cleared
rc="$(call_artifact 0.0)"
[ "$rc" = "0" ] && [ "$(gate)" = "none" ] && ok || bad "clean artifact: pass + no gate" "rc=$rc gate=$(gate)"

# 2. Sycophantic artifact (score 0.8) → exit 2, gate rejected
rc="$(call_artifact 0.8)"
[ "$rc" = "2" ] && [ "$(gate)" = "rejected" ] && ok || bad "sycophantic: block + gate set" "rc=$rc gate=$(gate)"

# 3. pipeline-enforce blocks kbd-new-phase while gate rejected
( cd "$ROOT" && echo '{"command":"/kbd-new-phase foo"}' | bash "$ENFORCE" Bash >/dev/null 2>&1 )
rc=$?
[ "$rc" = "2" ] && ok || bad "pipeline-enforce should block kbd-new-phase while rejected" "rc=$rc"

# 4. A passing rewrite clears the gate → pipeline-enforce allows again
rc="$(call_artifact 0.1)"   # counter is at 1; a pass clears gate + counter
[ "$(gate)" = "none" ] || bad "passing rewrite should clear gate" "gate=$(gate)"
( cd "$ROOT" && echo '{"command":"/kbd-new-phase foo"}' | bash "$ENFORCE" Bash >/dev/null 2>&1 )
rc=$?
[ "$rc" = "0" ] && ok || bad "pipeline-enforce should allow kbd-new-phase after clear" "rc=$rc"

# 5. Soft cap: two rejections then accept-with-warning (gate cleared)
call_artifact 0.8 >/dev/null   # rejection 1 (counter→1)
call_artifact 0.8 >/dev/null   # rejection 2 (counter→2)
rc="$(call_artifact 0.8)"      # 3rd: soft cap → accept
[ "$rc" = "0" ] && [ "$(gate)" = "none" ] && ok || bad "soft cap accepts 3rd + clears gate" "rc=$rc gate=$(gate)"

# 6. Non-artifact path → no-op pass, gate untouched
echo '{ "phase": "p1", "reflect_gate": "rejected" }' > "$PHASE/progress.json"
rc="$(printf '{"tool_input":{"file_path":"%s/src/app.ts"}}' "$ROOT" | bash "$ARTIFACT" >/dev/null 2>&1; echo $?)"
[ "$rc" = "0" ] && [ "$(gate)" = "rejected" ] && ok || bad "non-artifact path is a no-op" "rc=$rc gate=$(gate)"

# 7. Binary absent → graceful exit 0 (strip fake from PATH)
echo '{ "phase": "p1", "changes_total":1, "changes_completed":1 }' > "$PHASE/progress.json"
rc="$(PATH="/usr/bin:/bin" CLAUDE_PLUGIN_ROOT="" bash -c 'printf "{\"tool_input\":{\"file_path\":\"'"$PHASE"'/reflection.md\"}}" | bash "'"$ARTIFACT"'" >/dev/null 2>&1; echo $?')"
[ "$rc" = "0" ] && ok || bad "binary absent should exit 0" "rc=$rc"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
