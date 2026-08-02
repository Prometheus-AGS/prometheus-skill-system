#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ADAPTER="$REPO_ROOT/shared/scripts/kbd-harness-adapter.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT
mkdir -p "$TEST_ROOT/.prometheus" "$TEST_ROOT/.kbd-orchestrator"
printf '{"schemaVersion":"1","projectId":"00000000-0000-4000-8000-000000000001","repositoryFingerprint":"sha256:test"}\n' \
  >"$TEST_ROOT/.prometheus/project.json"

# The adapter no longer intercepts tool calls: the pre-mutation fence was
# removed because it gated the operator's own shell on KBD lifecycle state. An
# emergency PAUSE must still be *observable* — session_start and
# post_compact announce it — but it must never deny a tool call, and no event
# may exit nonzero.
printf 'paused\n' >"$TEST_ROOT/.kbd-orchestrator/PAUSE"
OUTPUT="$(cd "$TEST_ROOT" && bash "$ADAPTER" session_start claude-code 2>&1)"
[[ "$OUTPUT" == *"emergency PAUSE"* ]]

for event in session_start post_compact prompt stop; do
  (cd "$TEST_ROOT" && bash "$ADAPTER" "$event" claude-code >/dev/null 2>&1) ||
    { printf 'FAIL: %s exited nonzero while paused\n' "$event" >&2; exit 1; }
done

rm "$TEST_ROOT/.kbd-orchestrator/PAUSE"
PATH="/usr/bin:/bin" \
  bash -c "cd '$TEST_ROOT' && bash '$ADAPTER' interrupt kimi"
[[ -f "$TEST_ROOT/.kbd-orchestrator/PAUSE" ]]
grep -q '^lifecycle=pause_requested$' "$TEST_ROOT/.kbd-orchestrator/PAUSE"

rm "$TEST_ROOT/.kbd-orchestrator/PAUSE"
mkdir -p "$TEST_ROOT/bin"
cat >"$TEST_ROOT/bin/curl" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' '{"revision":42,"planRevision":7,"lifecycle":"paused","activePath":{"phaseId":"phase-a","stageId":"audit","changeId":"change-a","taskId":"task-a"},"exactNextWork":"Review the committed architecture decision"}'
MOCK
chmod +x "$TEST_ROOT/bin/curl"
printf 'test-control-token-with-at-least-thirty-two-characters\n' >"$TEST_ROOT/control-token"
REANCHOR="$(
  cd "$TEST_ROOT"
  PATH="$TEST_ROOT/bin:$PATH" \
    PROMETHEUS_CONTROL_TOKEN_FILE="$TEST_ROOT/control-token" \
    bash "$ADAPTER" post_compact codex
)"
[[ "$REANCHOR" == *"committed revision 42"* ]]
[[ "$REANCHOR" == *"phase-a → audit → change-a → task-a"* ]]
[[ "${#REANCHOR}" -le 4800 ]]

printf 'KBD harness adapter emergency controls passed.\n'
