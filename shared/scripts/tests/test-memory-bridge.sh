#!/usr/bin/env bash
# Tests for the non-blocking central memory outbox bridge.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB="$SCRIPT_DIR/../lib/memory-bridge.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
BIN="$TMP/bin"; mkdir -p "$BIN"
cat > "$BIN/curl" <<'FAKE'
#!/usr/bin/env bash
[ "${FAKE_CURL_MODE:-fail}" = "ok" ] && exit 0
exit 1
FAKE
chmod +x "$BIN/curl"
export PATH="$BIN:$PATH"
export PROMETHEUS_LEARNING_QUEUE="$TMP/queue"
export KBD_PROJECT_NAME="test-project"
PENDING="$PROMETHEUS_LEARNING_QUEUE/memory/pending"
# shellcheck source=/dev/null
source "$LIB"

[ "$(mem_scope_for "[GLOBAL] some rule")" = "global" ] && ok || bad "scope: [GLOBAL] → global"
[ "$(mem_scope_for "ordinary note")" = "test-project" ] && ok || bad "scope: plain → project"

mem_add_memory "a project learning" >/dev/null; rc=$?
[ "$rc" = "0" ] && ok || bad "mem_add_memory returns 0" "rc=$rc"
[ "$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')" = "1" ] \
  && ok || bad "one central operation queued"
first="$(find "$PENDING" -type f -name '*.json' -print -quit)"
jq -e '.schemaVersion == 1 and .method == "add_memory" and .arguments.user_id == "test-project"' "$first" >/dev/null \
  && ok || bad "queued operation records schema, method, and project scope"

mem_add_memory "[GLOBAL] a universal rule" >/dev/null
find "$PENDING" -type f -name '*.json' -exec jq -e 'select(.arguments.user_id == "global")' {} + >/dev/null \
  && ok || bad "[GLOBAL] content routes to global scope"

before="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
mem_create_task_stream "kbd:test:phase" >/dev/null
mem_add_task_step "kbd:test:phase" "change-001" >/dev/null
mem_complete_step "kbd:test:phase" "change-001" >/dev/null
after="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$((after - before))" = "3" ] && ok || bad "all operations use the central outbox" "$before→$after"

# Repeated identical writes are idempotent and never call the service inline.
before="$after"
mem_add_memory "a project learning" >/dev/null
after="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$before" = "$after" ] && ok || bad "duplicate operation is idempotent" "$before→$after"

export FAKE_CURL_MODE=ok
mem_available && ok || bad "mem_available still reports service health"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
