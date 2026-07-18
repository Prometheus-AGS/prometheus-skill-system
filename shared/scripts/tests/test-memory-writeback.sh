#!/usr/bin/env bash
# test-memory-writeback.sh — tests for memory-writeback.sh + memory-outbox-flush.sh
# PATH-shimmed fake curl; no real surreal-memory endpoint.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WRITEBACK="$SCRIPT_DIR/../memory-writeback.sh"
FLUSH="$SCRIPT_DIR/../memory-outbox-flush.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
BIN="$TMP/bin"; mkdir -p "$BIN"
cat > "$BIN/curl" <<'FAKE'
#!/usr/bin/env bash
mode="${FAKE_CURL_MODE:-fail}"
body=""; method="GET"
while [ $# -gt 0 ]; do case "$1" in -d) body="$2"; shift 2 ;; -X) method="$2"; shift 2 ;; *) shift ;; esac; done
[ -n "${CURL_BODY_FILE:-}" ] && [ -n "$body" ] && printf '%s\n' "$body" >> "$CURL_BODY_FILE"
if [ "$mode" = "ok" ]; then case "$method" in POST|PUT) printf '201' ;; *) printf '200' ;; esac; else printf '000'; fi
exit 0
FAKE
chmod +x "$BIN/curl"
export PATH="$BIN:$PATH"

ROOT="$TMP/proj"; PHASE="$ROOT/.kbd-orchestrator/phases/p1"
mkdir -p "$PHASE"
echo '{ "phase": "p1" }' > "$ROOT/.kbd-orchestrator/current-waypoint.json"
cat > "$PHASE/reflection.md" <<'MD'
# Reflection — p1

## Delta
1. Something was missed.

## Root Cause
1. The active store path was assumed instead of inspected.

## Corrective Actions
1. [GLOBAL] A universal rule to apply everywhere.
MD

writeback_post() { # <file-path>  (PostToolUse style, stdin JSON)
  ( cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$1" | bash "$WRITEBACK" >/dev/null 2>&1 )
}

# 1. Accepted reflection (no gate), endpoint up → POST happens, [GLOBAL] → global scope
echo '{ "phase":"p1" }' > "$PHASE/progress.json"
export FAKE_CURL_MODE=ok; export CURL_BODY_FILE="$TMP/body.txt"; : > "$CURL_BODY_FILE"
writeback_post "$PHASE/reflection.md"
[ -s "$CURL_BODY_FILE" ] && ok || bad "accepted reflection should POST to memory"
grep -q '"user_id": "global"' "$CURL_BODY_FILE" && ok || bad "[GLOBAL] corrective action → global scope" "$(cat "$CURL_BODY_FILE")"
grep -q 'Deltas:' "$CURL_BODY_FILE" && ok || bad "payload includes Delta section"
grep -q 'Root causes:' "$CURL_BODY_FILE" && ok || bad "payload includes Root Cause section"

# 2. Rejected reflection → NOT persisted
echo '{ "phase":"p1", "reflect_gate":"rejected" }' > "$PHASE/progress.json"
: > "$CURL_BODY_FILE"
writeback_post "$PHASE/reflection.md"
[ ! -s "$CURL_BODY_FILE" ] && ok || bad "rejected reflection must not be persisted" "$(cat "$CURL_BODY_FILE")"

# 3. Non-reflection file → no-op
echo '{ "phase":"p1" }' > "$PHASE/progress.json"
: > "$CURL_BODY_FILE"
writeback_post "$ROOT/src/app.ts"
[ ! -s "$CURL_BODY_FILE" ] && ok || bad "non-reflection path must be a no-op"

# 4. Endpoint down → writeback buffers to outbox (via bridge), still exits 0
export FAKE_CURL_MODE=fail
rc="$( ( cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$PHASE/reflection.md" | bash "$WRITEBACK" >/dev/null 2>&1; echo $? ) )"
[ "$rc" = "0" ] && ok || bad "writeback exits 0 when endpoint down" "rc=$rc"
[ -s "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" ] && ok || bad "failed writeback buffers to outbox"

# 5. Outbox flush: endpoint down → outbox preserved
LINES_BEFORE="$(wc -l < "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" | tr -d ' ')"
export FAKE_CURL_MODE=fail
( cd "$ROOT" && bash "$FLUSH" >/dev/null 2>&1 )
LINES_AFTER="$(wc -l < "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" 2>/dev/null | tr -d ' ' || echo 0)"
[ "$LINES_BEFORE" = "$LINES_AFTER" ] && ok || bad "flush must preserve outbox when endpoint down" "$LINES_BEFORE→$LINES_AFTER"

# 6. Outbox flush: endpoint up → REST-drainable add_memory line drains
export FAKE_CURL_MODE=ok
( cd "$ROOT" && bash "$FLUSH" >/dev/null 2>&1 )
[ ! -f "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" ] || [ ! -s "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" ] \
  && ok || bad "flush should drain add_memory outbox when endpoint up" "$(cat "$ROOT/.kbd-orchestrator/memory-outbox.jsonl" 2>/dev/null)"

# 7. MCP-only lines (no REST route) are DROPPED on flush, not retained forever
OB="$ROOT/.kbd-orchestrator/memory-outbox.jsonl"
{
  printf '%s\n' '{"queuedAt":"t","method":"add_memory","arguments":{"content":"keep","user_id":"p"}}'
  printf '%s\n' '{"queuedAt":"t","method":"create_task_stream","arguments":{"name":"s"}}'
  printf '%s\n' '{"queuedAt":"t","method":"complete_step","arguments":{"stream":"s","step":"1"}}'
} > "$OB"
export FAKE_CURL_MODE=ok
( cd "$ROOT" && bash "$FLUSH" >/dev/null 2>&1 )
# add_memory drained (REST), the two MCP-only lines dropped → outbox gone/empty
[ ! -s "$OB" ] && ok || bad "MCP-only lines must be dropped, add_memory drained" "$(cat "$OB" 2>/dev/null)"

# 8. Endpoint down → ALL lines preserved (including REST-routable add_memory)
printf '%s\n' '{"queuedAt":"t","method":"add_memory","arguments":{"content":"x","user_id":"p"}}' > "$OB"
export FAKE_CURL_MODE=fail
( cd "$ROOT" && bash "$FLUSH" >/dev/null 2>&1 )
[ "$(wc -l < "$OB" 2>/dev/null | tr -d ' ')" = "1" ] && ok || bad "endpoint down must preserve add_memory line" "$(cat "$OB" 2>/dev/null)"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
