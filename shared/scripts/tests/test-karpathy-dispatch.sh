#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DISPATCH="$ROOT/shared/scripts/karpathy-hook-dispatch.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { PASS=$((PASS + 1)); printf '[PASS] %s\n' "$1"; }
fail() { FAIL=$((FAIL + 1)); printf '[FAIL] %s\n' "$1" >&2; }

mkdir -p "$TMP/bin" "$TMP/home/.prometheus/knowledge/shared/wiki" \
  "$TMP/home/.prometheus/knowledge/wiki" "$TMP/project/.git" \
  "$TMP/project/.prometheus/knowledge/wiki"

cat > "$TMP/bin/pk" <<'SH'
#!/usr/bin/env bash
printf '[project] Project result\nproject-token\n[shared] Shared result\nshared-token\n[global] Global result\nglobal-token\n'
SH
chmod +x "$TMP/bin/pk"

prompt_output="$(
  cd "$TMP/project"
  printf '{"prompt":"project-token shared-token global-token","cwd":"%s"}' "$TMP/project" |
    HOME="$TMP/home" PATH="$TMP/bin:$PATH" PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" \
      "$DISPATCH" prompt codex
)"
if [[ "$prompt_output" == *'[project]'* && "$prompt_output" == *'[shared]'* && "$prompt_output" == *'[global]'* ]]; then
  pass "prompt dispatch emits labeled local context"
else
  fail "prompt dispatch emits labeled local context"
fi

payload="$(printf '{"session_id":"fixture-session","cwd":"%s","transcript_path":"%s"}' \
  "$TMP/project" "$TMP/transcript.jsonl")"
started="$(python3 -c 'import time; print(time.monotonic_ns())')"
printf '%s' "$payload" | HOME="$TMP/home" PROMETHEUS_LEARNING_QUEUE="$TMP/queue" \
  PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" "$DISPATCH" stop codex
finished="$(python3 -c 'import time; print(time.monotonic_ns())')"
elapsed_ms=$(((finished - started) / 1000000))
if [ "$elapsed_ms" -lt 250 ]; then
  pass "stop dispatch returns within 250 ms"
else
  fail "stop dispatch returns within 250 ms (observed ${elapsed_ms} ms)"
fi

job_count="$(find "$TMP/queue/pending" -type f -name '*.json' | wc -l | tr -d ' ')"
if [ "$job_count" = 1 ] && jq -e \
  '.schemaVersion == 2 and .harness == "codex" and .sessionId == "fixture-session"' \
  "$TMP/queue/pending"/*.json >/dev/null; then
  pass "stop dispatch atomically enqueues one normalized v2 job"
else
  fail "stop dispatch atomically enqueues one normalized v2 job"
fi

printf '%s' "$payload" | HOME="$TMP/home" PROMETHEUS_LEARNING_QUEUE="$TMP/queue" \
  PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" "$DISPATCH" stop codex
job_count="$(find "$TMP/queue/pending" -type f -name '*.json' | wc -l | tr -d ' ')"
if [ "$job_count" = 1 ]; then
  pass "duplicate delivery is idempotent"
else
  fail "duplicate delivery is idempotent"
fi

printf '\nPASS=%d FAIL=%d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
