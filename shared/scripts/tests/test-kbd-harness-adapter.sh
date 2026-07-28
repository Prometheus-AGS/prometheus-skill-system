#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ADAPTER="$REPO_ROOT/shared/scripts/kbd-harness-adapter.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT
mkdir -p "$TEST_ROOT/.prometheus" "$TEST_ROOT/.kbd-orchestrator"
printf '{"schemaVersion":"1","projectId":"00000000-0000-4000-8000-000000000001","repositoryFingerprint":"sha256:test"}\n' \
  >"$TEST_ROOT/.prometheus/project.json"

printf 'paused\n' >"$TEST_ROOT/.kbd-orchestrator/PAUSE"
set +e
OUTPUT="$(cd "$TEST_ROOT" && bash "$ADAPTER" pre_mutation claude-code 2>&1)"
STATUS=$?
set -e
[[ "$STATUS" -eq 2 ]]
[[ "$OUTPUT" == *"emergency PAUSE"* ]]

(cd "$TEST_ROOT" && bash "$ADAPTER" stop claude-code)

rm "$TEST_ROOT/.kbd-orchestrator/PAUSE"
PATH="/usr/bin:/bin" \
  bash -c "cd '$TEST_ROOT' && bash '$ADAPTER' interrupt kimi"
[[ -f "$TEST_ROOT/.kbd-orchestrator/PAUSE" ]]
grep -q '^lifecycle=pause_requested$' "$TEST_ROOT/.kbd-orchestrator/PAUSE"

rm "$TEST_ROOT/.kbd-orchestrator/PAUSE"
mkdir -p "$TEST_ROOT/bin"
cat >"$TEST_ROOT/bin/curl" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' '{"revision":42,"planRevision":7,"lifecycle":"paused","activePath":{"phaseId":"phase-a","stageId":"audit","changeId":"change-a","taskId":"task-a"},"exactNextWork":"Review the committed architecture decision","lease":{"owner":{"harness":"codex","device":"device-a"},"fencingToken":9}}'
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
