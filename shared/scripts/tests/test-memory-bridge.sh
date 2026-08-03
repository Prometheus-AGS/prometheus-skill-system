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
export PROMETHEUS_PROJECT_ID="test-project"
PENDING="$PROMETHEUS_LEARNING_QUEUE/memory/pending"
# shellcheck source=/dev/null
source "$LIB"

! declare -F mem_scope_for >/dev/null && ok || bad "bridge does not infer scope from content"

mem_add_memory "a project learning" >/dev/null; rc=$?
[ "$rc" = "0" ] && ok || bad "mem_add_memory returns 0" "rc=$rc"
[ "$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')" = "1" ] \
  && ok || bad "one central operation queued"
first="$(find "$PENDING" -type f -name '*.json' -print -quit)"
jq -e '.schemaVersion == 2 and .method == "add_memory" and .arguments.user_id == "test-project" and .state == "pending" and (.payloadHash | length) == 64' "$first" >/dev/null \
  && ok || bad "queued operation records v2 schema, hash, state, and project scope"

mem_add_memory "[GLOBAL] a universal rule" "global" >/dev/null
find "$PENDING" -type f -name '*.json' -exec jq -e 'select(.arguments.user_id == "global")' {} + >/dev/null \
  && ok || bad "[GLOBAL] content routes to global scope"

mode="$(stat -f '%Lp' "$first" 2>/dev/null || stat -c '%a' "$first")"
[ "$mode" = 600 ] && grep -q 'os.fsync' "$SCRIPT_DIR/../enqueue-memory-operation.py" \
  && ok || bad "memory enqueue is mode-0600 and fsynced"

before="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
mem_create_task_stream "test:phase" >/dev/null
mem_add_task_step "test:phase" "change-001" >/dev/null
mem_complete_step "test:phase" "change-001" >/dev/null
after="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$((after - before))" = "3" ] && ok || bad "all operations use the central outbox" "$before→$after"
jq -s -e '
  (map(select(.method == "create_task_stream"))[0].operationId) as $create |
  (map(select(.method == "add_task_step"))[0]) as $add |
  (map(select(.method == "complete_step"))[0]) as $complete |
  ($add.dependencies == [$create]) and ($complete.dependencies == [$add.operationId])
' "$PENDING"/*.json >/dev/null \
  && ok || bad "compound operations encode explicit dependency ids"

# Repeated identical writes are idempotent and never call the service inline.
before="$after"
mem_add_memory "a project learning" >/dev/null
after="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$before" = "$after" ] && ok || bad "duplicate operation is idempotent" "$before→$after"

mkdir -p "$PROMETHEUS_LEARNING_QUEUE/memory/accepted"
mv "$first" "$PROMETHEUS_LEARNING_QUEUE/memory/accepted/"
mem_add_memory "a project learning" >/dev/null
[ ! -e "$first" ] && ok || bad "accepted operation is not duplicated into pending"

! declare -F mem_available >/dev/null \
  && ok || bad "hook bridge exposes no synchronous service probe"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
