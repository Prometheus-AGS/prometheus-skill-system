#!/usr/bin/env bash
# test-memory-bridge.sh — tests for shared/scripts/lib/memory-bridge.sh
# Uses a PATH-shimmed fake `curl` to exercise success and failure paths with
# no real surreal-memory endpoint.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB="$SCRIPT_DIR/../lib/memory-bridge.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
BIN="$TMP/bin"; mkdir -p "$BIN"
ROOT="$TMP/proj"; mkdir -p "$ROOT/.kbd-orchestrator"
OUTBOX="$ROOT/.kbd-orchestrator/memory-outbox.jsonl"

# Fake curl: behavior controlled by FAKE_CURL_MODE.
#   ok   → prints 200 (for -w status calls), exits 0, captures POST body to $CURL_BODY_FILE
#   fail → prints 000, exits 0 (simulates unreachable endpoint)
cat > "$BIN/curl" <<'FAKE'
#!/usr/bin/env bash
mode="${FAKE_CURL_MODE:-fail}"
body=""
while [ $# -gt 0 ]; do
  case "$1" in
    -d) body="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[ -n "${CURL_BODY_FILE:-}" ] && [ -n "$body" ] && printf '%s' "$body" >> "$CURL_BODY_FILE"
if [ "$mode" = "ok" ]; then
  # healthz probe and -w status both expect "200" on stdout
  printf '200'
  exit 0
else
  printf '000'
  exit 0
fi
FAKE
chmod +x "$BIN/curl"
export PATH="$BIN:$PATH"

cd "$ROOT"
export KBD_PROJECT_NAME="test-project"
# shellcheck source=/dev/null
source "$LIB"

# 1. mem_scope_for routes [GLOBAL] vs project
[ "$(mem_scope_for "[GLOBAL] some rule")" = "global" ] && ok || bad "scope: [GLOBAL] → global"
[ "$(mem_scope_for "ordinary note")" = "test-project" ] && ok || bad "scope: plain → project"

# 2. Failure path: every function returns 0 and writes the outbox
export FAKE_CURL_MODE=fail
mem_add_memory "a project learning" >/dev/null; rc=$?
[ "$rc" = "0" ] && ok || bad "mem_add_memory returns 0 on failure" "rc=$rc"
[ -f "$OUTBOX" ] && ok || bad "outbox file created on failure"
[ "$(wc -l < "$OUTBOX" | tr -d ' ')" = "1" ] && ok || bad "one outbox line after one failed add"
jq -e '.method == "add_memory" and (.arguments.user_id == "test-project")' "$OUTBOX" >/dev/null \
  && ok || bad "outbox line records method + project scope" "$(cat "$OUTBOX")"

# 3. [GLOBAL] content in outbox records global scope
mem_add_memory "[GLOBAL] a universal rule" >/dev/null
tail -1 "$OUTBOX" | jq -e '.arguments.user_id == "global"' >/dev/null \
  && ok || bad "[GLOBAL] content → global scope in outbox" "$(tail -1 "$OUTBOX")"

# 4. Other functions also append on failure
n_before="$(wc -l < "$OUTBOX" | tr -d ' ')"
mem_create_task_stream "kbd:test:phase" >/dev/null
mem_add_task_step "kbd:test:phase" "change-001" >/dev/null
mem_complete_step "kbd:test:phase" "change-001" >/dev/null
n_after="$(wc -l < "$OUTBOX" | tr -d ' ')"
[ "$((n_after - n_before))" = "3" ] && ok || bad "stream/step funcs append on failure" "$n_before→$n_after"

# 5. Success path: add_memory POST body carries correct user_id, no new outbox line
: > "$OUTBOX"
export FAKE_CURL_MODE=ok
export CURL_BODY_FILE="$TMP/body.txt"; : > "$CURL_BODY_FILE"
mem_add_memory "another note" >/dev/null
[ "$(wc -l < "$OUTBOX" | tr -d ' ')" = "0" ] && ok || bad "success path writes no outbox line" "$(cat "$OUTBOX")"
grep -q '"user_id": "test-project"' "$CURL_BODY_FILE" && ok || bad "POST body has project user_id" "$(cat "$CURL_BODY_FILE")"
grep -q '"name": "add_memory"' "$CURL_BODY_FILE" && ok || bad "POST body calls add_memory tool" "$(cat "$CURL_BODY_FILE")"

# 6. mem_available reflects the fake curl mode
export FAKE_CURL_MODE=ok
mem_available && ok || bad "mem_available true when curl returns 200"
# (fail mode: curl exits 0 with 000 body, but -fsS would fail on a real non-200;
#  our fake always exits 0, so we only assert the ok-mode positive here.)

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
