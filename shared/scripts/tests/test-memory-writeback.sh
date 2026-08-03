#!/usr/bin/env bash
# Tests reflection writeback into the central supervised memory outbox.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WRITEBACK="$SCRIPT_DIR/../memory-writeback.sh"
FLUSH="$SCRIPT_DIR/../memory-outbox-flush.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
ROOT="$TMP/proj"; PHASE="$ROOT/.kbd-orchestrator/phases/p1"
export HOME="$TMP/home"
export PROMETHEUS_LEARNING_QUEUE="$TMP/queue"
PENDING="$PROMETHEUS_LEARNING_QUEUE/memory/pending"
mkdir -p "$PHASE" "$HOME"
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

writeback_post() {
  ( cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$1" | bash "$WRITEBACK" >/dev/null 2>&1 )
}

echo '{ "phase":"p1" }' > "$PHASE/progress.json"
writeback_post "$PHASE/reflection.md"
[ "$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')" = "1" ] \
  && ok || bad "accepted reflection queues one operation"
operation="$(find "$PENDING" -type f -name '*.json' -print -quit)"
jq -e '.arguments.user_id == "global"' "$operation" >/dev/null \
  && ok || bad "[GLOBAL] corrective action routes globally"
jq -e '.arguments.content | contains("Deltas:")' "$operation" >/dev/null \
  && ok || bad "payload includes Delta section"
jq -e '.arguments.content | contains("Root causes:")' "$operation" >/dev/null \
  && ok || bad "payload includes Root Cause section"

echo '{ "phase":"p1", "reflect_gate":"rejected" }' > "$PHASE/progress.json"
before="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
writeback_post "$PHASE/reflection.md"
after="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$before" = "$after" ] && ok || bad "rejected reflection is not queued"

echo '{ "phase":"p1" }' > "$PHASE/progress.json"
writeback_post "$ROOT/src/app.ts"
after_nonreflection="$(find "$PENDING" -type f -name '*.json' | wc -l | tr -d ' ')"
[ "$after" = "$after_nonreflection" ] && ok || bad "non-reflection path is a no-op"

# Legacy JSONL is migrated without contacting the endpoint and retained under
# a timestamped migrated name.
LEGACY="$ROOT/.kbd-orchestrator/memory-outbox.jsonl"
printf '%s\n' '{"queuedAt":"t","method":"add_memory","arguments":{"content":"legacy","user_id":"p"}}' > "$LEGACY"
( cd "$ROOT" && bash "$FLUSH" >/dev/null 2>&1 )
[ ! -e "$LEGACY" ] && ok || bad "legacy active outbox is retired after migration"
[ "$(find "$ROOT/.kbd-orchestrator" -name 'memory-outbox.jsonl.migrated.*' | wc -l | tr -d ' ')" = "1" ] \
  && ok || bad "legacy outbox is preserved as migrated archive"
find "$PENDING" -type f -name '*.json' -exec jq -e 'select(.arguments.content == "legacy")' {} + >/dev/null \
  && ok || bad "legacy memory operation reaches central queue"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
