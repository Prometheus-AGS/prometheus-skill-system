#!/usr/bin/env bash
# Behavioral tests for the Karpathy-loop prompt focus and stop-ingest hooks.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FOCUS_HOOK="$SCRIPT_DIR/../pk-focus-on-prompt.sh"
STOP_HOOK="$SCRIPT_DIR/../forge-reflect-on-stop.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

# Missing pk must degrade silently and successfully.
NO_PK_OUTPUT="$(printf '%s\n' '{"prompt":"tower middleware"}' \
  | HOME="$TMP_ROOT/no-pk-home" PATH="/usr/bin:/bin" bash "$FOCUS_HOOK")"
[[ -z "$NO_PK_OUTPUT" ]] || fail "focus hook emitted output without pk"
echo "[PASS] prompt focus degrades silently when pk is unavailable"

mkdir -p "$TMP_ROOT/bin" "$TMP_ROOT/home/.prometheus" "$TMP_ROOT/work"
cat > "$TMP_ROOT/bin/pk" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${PK_CALLS:?}"
if [[ "${1:-}" == "focus" ]]; then
  printf '%s\n' 'focused Tower middleware knowledge'
elif [[ "${1:-}" == "ingest" ]]; then
  cat > "${PK_STDIN:?}"
fi
SH
chmod +x "$TMP_ROOT/bin/pk"

FOCUS_OUTPUT="$(printf '%s\n' '{"prompt":"Explain Tower middleware authentication ordering"}' \
  | HOME="$TMP_ROOT/home" \
    PATH="$TMP_ROOT/bin:/usr/bin:/bin" \
    PK_CALLS="$TMP_ROOT/pk-calls" \
    PK_STDIN="$TMP_ROOT/pk-stdin" \
    PROMETHEUS_FOCUS_SEMANTIC=0 \
    bash "$FOCUS_HOOK")"
grep -q -- '--- prometheus-knowledge context ---' <<< "$FOCUS_OUTPUT" \
  || fail "focus hook did not emit a context boundary"
grep -q 'focused Tower middleware knowledge' <<< "$FOCUS_OUTPUT" \
  || fail "focus hook did not surface pk output"
grep -q '^focus .* --k 3$' "$TMP_ROOT/pk-calls" \
  || fail "focus hook did not invoke pk focus with k=3"
echo "[PASS] prompt focus injects bounded pk context"

cat > "$TMP_ROOT/home/.prometheus/last-session-summary.txt" <<'EOF'
phase: phase-learning-runtime
last_completed: learner-model review RPC
next_pending: cross-tool verification
progress: 2 of 3 changes
EOF
(
  cd "$TMP_ROOT/work"
  HOME="$TMP_ROOT/home" \
    PATH="$TMP_ROOT/bin:/usr/bin:/bin" \
    PK_CALLS="$TMP_ROOT/pk-calls" \
    PK_STDIN="$TMP_ROOT/pk-stdin" \
    SURREAL_MEMORY_URL="http://127.0.0.1:1" \
    bash "$STOP_HOOK"
)
grep -q '^ingest$' "$TMP_ROOT/pk-calls" || fail "stop hook did not invoke pk ingest"
cmp -s "$TMP_ROOT/home/.prometheus/last-session-summary.txt" "$TMP_ROOT/pk-stdin" \
  || fail "stop hook did not ingest the session summary"
echo "[PASS] stop hook ingests a meaningful session summary through pk"

: > "$TMP_ROOT/pk-calls"
cat > "$TMP_ROOT/home/.prometheus/last-session-summary.txt" <<'EOF'
phase: unknown
last_completed: none
next_pending: none
progress: 0 of 0 changes
EOF
(
  cd "$TMP_ROOT/work"
  HOME="$TMP_ROOT/home" \
    PATH="$TMP_ROOT/bin:/usr/bin:/bin" \
    PK_CALLS="$TMP_ROOT/pk-calls" \
    PK_STDIN="$TMP_ROOT/pk-stdin" \
    bash "$STOP_HOOK"
)
[[ ! -s "$TMP_ROOT/pk-calls" ]] || fail "empty session should not be ingested"
echo "[PASS] stop hook rejects empty-session knowledge noise"
