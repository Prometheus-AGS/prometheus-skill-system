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
printf '%s\n' "$@" > "${PROMETHEUS_PK_ARGS:?}"
printf '[project] Project result\nproject-token\n[shared] Shared result\nshared-token\n[global] Global result\nglobal-token\n'
SH
chmod +x "$TMP/bin/pk"
for forbidden in curl surreal-memory learner-model; do
  cat > "$TMP/bin/$forbidden" <<'SH'
#!/usr/bin/env bash
touch "${PROMETHEUS_FORBIDDEN_SENTINEL:?}"
exit 99
SH
  chmod +x "$TMP/bin/$forbidden"
done

prompt_output="$(
  cd "$TMP/project"
  printf '{"prompt":"project-token shared-token global-token","cwd":"%s"}' "$TMP/project" |
    HOME="$TMP/home" PATH="$TMP/bin:$PATH" PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" \
      PROMETHEUS_PK_ARGS="$TMP/pk-args" PROMETHEUS_FORBIDDEN_SENTINEL="$TMP/forbidden" \
      "$DISPATCH" prompt codex
)"
if [[ "$prompt_output" == *'[project]'* && "$prompt_output" == *'[shared]'* && "$prompt_output" == *'[global]'* ]]; then
  pass "prompt dispatch emits labeled local context"
else
  fail "prompt dispatch emits labeled local context"
fi
if grep -Fx -- '--max-candidates' "$TMP/pk-args" >/dev/null && \
  grep -Fx -- '--max-bytes' "$TMP/pk-args" >/dev/null && \
  ! grep -Fx -- '--timeout-ms' "$TMP/pk-args" >/dev/null; then
  pass "prompt dispatch uses candidate and byte bounds without a latency deadline"
else
  fail "prompt dispatch uses candidate and byte bounds without a latency deadline"
fi

payload="$(printf '{"session_id":"fixture-session","cwd":"%s","transcript_path":"%s"}' \
  "$TMP/project" "$TMP/transcript.jsonl")"
hook_lines_before="$(wc -l < "$TMP/home/.prometheus/hooks.log" | tr -d ' ')"
printf '%s' "$payload" | HOME="$TMP/home" PATH="$TMP/bin:$PATH" \
  PROMETHEUS_LEARNING_QUEUE="$TMP/queue" PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" \
  PROMETHEUS_FORBIDDEN_SENTINEL="$TMP/forbidden" "$DISPATCH" stop codex
hook_lines_after="$(wc -l < "$TMP/home/.prometheus/hooks.log" | tr -d ' ')"
if [ ! -e "$TMP/forbidden" ] && [ "$hook_lines_before" = "$hook_lines_after" ]; then
  pass "stop dispatch performs no inference, network, memory, or hook-log work"
else
  fail "stop dispatch performs no inference, network, memory, or hook-log work"
fi

auto_payload="$(printf '{"event":"stop","session_id":"auto-session","cwd":"%s"}' "$TMP/project")"
printf '%s' "$auto_payload" | HOME="$TMP/home" PATH="$TMP/bin:$PATH" \
  PROMETHEUS_LEARNING_QUEUE="$TMP/auto-queue" \
  PROMETHEUS_FORBIDDEN_SENTINEL="$TMP/forbidden" "$DISPATCH" auto codex
hook_lines_auto="$(wc -l < "$TMP/home/.prometheus/hooks.log" | tr -d ' ')"
if [ "$hook_lines_after" = "$hook_lines_auto" ] && \
  [ "$(find "$TMP/auto-queue/pending" -name '*.json' -type f | wc -l | tr -d ' ')" = 1 ]; then
  pass "auto-classified stop enqueues before any observational side effect"
else
  fail "auto-classified stop enqueues before any observational side effect"
fi

job_count="$(find "$TMP/queue/pending" -type f -name '*.json' | wc -l | tr -d ' ')"
if [ "$job_count" = 1 ] && jq -e \
  '.schemaVersion == 2 and .harness == "codex" and .sessionId == "fixture-session" and
   .scope == "project" and (has("attempt") | not)' \
  "$TMP/queue/pending"/*.json >/dev/null; then
  job_mode="$(stat -f '%Lp' "$TMP/queue/pending"/*.json 2>/dev/null || stat -c '%a' "$TMP/queue/pending"/*.json)"
  if [ "$job_mode" = 600 ] && grep -q 'os.fsync' "$ROOT/shared/scripts/enqueue-learning-job.py"; then
    pass "stop dispatch atomically enqueues and fsyncs one mode-0600 v2 job"
  else
    fail "stop dispatch atomically enqueues and fsyncs one mode-0600 v2 job"
  fi
else
  fail "stop dispatch atomically enqueues and fsyncs one mode-0600 v2 job"
fi

mkdir -p "$TMP/queue/processing"
mv "$TMP/queue/pending"/*.json "$TMP/queue/processing/"
printf '%s' "$payload" | HOME="$TMP/home" PROMETHEUS_LEARNING_QUEUE="$TMP/queue" \
  PROMETHEUS_HOOK_LOG="$TMP/hooks.jsonl" "$DISPATCH" stop codex
job_count="$(find "$TMP/queue/pending" -type f -name '*.json' | wc -l | tr -d ' ')"
processing_count="$(find "$TMP/queue/processing" -type f -name '*.json' | wc -l | tr -d ' ')"
if [ "$job_count" = 0 ] && [ "$processing_count" = 1 ]; then
  pass "duplicate delivery is idempotent across durable states"
else
  fail "duplicate delivery is idempotent across durable states"
fi

printf '\nPASS=%d FAIL=%d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
