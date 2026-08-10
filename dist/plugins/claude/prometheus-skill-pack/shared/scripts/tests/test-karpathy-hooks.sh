#!/usr/bin/env bash
# Behavioral tests for the canonical bounded-context and atomic-enqueue hook.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISPATCHER="$SCRIPT_DIR/../karpathy-hook-dispatch.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

mkdir -p "$TMP_ROOT/bin" "$TMP_ROOT/work" "$TMP_ROOT/queue"
git -C "$TMP_ROOT/work" init -q

# Missing pk is a visible degraded condition on stderr but remains fail-open and
# emits no fabricated context on stdout.
NO_PK_OUTPUT="$(printf '%s\n' '{"prompt":"tower middleware"}' \
  | PATH="/usr/bin:/bin" bash "$DISPATCHER" UserPromptSubmit test-harness \
    2>"$TMP_ROOT/no-pk.err")"
[[ -z "$NO_PK_OUTPUT" ]] || fail "dispatcher emitted context without pk"
grep -q 'status=unavailable reason=pk-not-found' "$TMP_ROOT/no-pk.err" \
  || fail "missing pk was not observable"
echo "[PASS] prompt context degrades observably without false output"

cat > "$TMP_ROOT/bin/pk" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${PK_CALLS:?}"
if [[ "${1:-}" == "context" ]]; then
  printf '%s\n' '[prometheus-context] status=ready candidates=1 bytes=34'
  printf '%s\n' 'bounded Tower middleware knowledge'
fi
SH
chmod +x "$TMP_ROOT/bin/pk"

CONTEXT_OUTPUT="$(printf '%s\n' '{"prompt":"Explain Tower middleware authentication ordering"}' \
  | PATH="$TMP_ROOT/bin:/usr/bin:/bin" PK_CALLS="$TMP_ROOT/pk-calls" \
    bash "$DISPATCHER" UserPromptSubmit test-harness)"
grep -q 'bounded Tower middleware knowledge' <<< "$CONTEXT_OUTPUT" \
  || fail "dispatcher did not return bounded context"
grep -q '^context .* --scope project --scope shared --scope global .* --format hook$' \
  "$TMP_ROOT/pk-calls" || fail "dispatcher did not invoke bounded pk context"
echo "[PASS] prompt hook uses committed bounded project/shared/global context"

# Stop is one metadata-only, atomic local enqueue. It never runs inference,
# Forge, Memory, curl, or synchronous knowledge writeback.
(
  cd "$TMP_ROOT/work"
  PROMETHEUS_LEARNING_QUEUE="$TMP_ROOT/queue" \
    CODEX_THREAD_ID="karpathy-hook-fixture" \
    bash "$DISPATCHER" Stop codex
)
JOB_COUNT="$(find "$TMP_ROOT/queue/pending" -maxdepth 1 -type f -name '*.json' | wc -l | tr -d ' ')"
[[ "$JOB_COUNT" == "1" ]] || fail "Stop did not produce exactly one pending job"
JOB="$(find "$TMP_ROOT/queue/pending" -maxdepth 1 -type f -name '*.json' | head -1)"
[[ "$(stat -f '%Lp' "$JOB" 2>/dev/null || stat -c '%a' "$JOB")" == "600" ]] \
  || fail "pending job is not mode 0600"
node -e "const x=require(process.argv[1]); if(x.schemaVersion!==2||x.eventType!=='Stop'||x.harness!=='codex') process.exit(1)" "$JOB" \
  || fail "pending job metadata is invalid"
echo "[PASS] Stop atomically enqueues one owner-only metadata job"

# The content-addressed event ID makes identical deliveries idempotent.
(
  cd "$TMP_ROOT/work"
  PROMETHEUS_LEARNING_QUEUE="$TMP_ROOT/queue" \
    CODEX_THREAD_ID="karpathy-hook-fixture" \
    bash "$DISPATCHER" Stop codex
)
JOB_COUNT="$(find "$TMP_ROOT/queue/pending" -maxdepth 1 -type f -name '*.json' | wc -l | tr -d ' ')"
[[ "$JOB_COUNT" == "1" ]] || fail "duplicate Stop created another pending job"
echo "[PASS] duplicate hook delivery is idempotent"
