#!/usr/bin/env bash
# Full integration proof that Claude's TaskCompleted hook blocks a tracked task
# until canonical completion and its signed KBD after receipt both exist.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd -P)"
PROJECT_ROOT="$(cd "$ROOT/../../.." && pwd -P)"
GATE="$PROJECT_ROOT/shared/scripts/kbd-task-completed-gate.sh"

command -v jq >/dev/null 2>&1 || { printf 'FAIL: jq is required\n' >&2; exit 1; }
command -v prometheus >/dev/null 2>&1 || { printf 'FAIL: prometheus is required\n' >&2; exit 1; }

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
export PROMETHEUS_DATA_DIR="$SANDBOX/data"
export SOVEREIGN_SYNC_SOCKET="$SANDBOX/no-control-plane.sock"
mkdir -p "$SANDBOX/.prometheus" "$SANDBOX/.kbd-orchestrator"
jq -n '{
  schemaVersion: "1",
  projectId: "38b6229d-53ce-49fe-b3fa-6ebfcd87fc32",
  repositoryFingerprint: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
}' > "$SANDBOX/.prometheus/project.json"

kbd() { prometheus kbd --path "$SANDBOX" "$@" >/dev/null; }
kbd phase create --command-id gate-phase-create --id gate-phase --title 'Gate phase'
kbd phase activate --command-id gate-phase-activate --id gate-phase --exact-next-work '/kbd-apply gate-change'
kbd phase transition --command-id gate-phase-start --id gate-phase --status in-progress
kbd change register --command-id gate-change-register --phase gate-phase --id gate-change --title 'Gate change' --sequence 1
kbd change transition --command-id gate-change-start --phase gate-phase --id gate-change --status in-progress
kbd task register --command-id gate-task-register --phase gate-phase --change gate-change --id gate-task --title 'Gate task' --sequence 1
kbd task transition --command-id gate-task-start --phase gate-phase --change gate-change --id gate-task --status in-progress

payload="$(jq -n --arg cwd "$SANDBOX" '{cwd:$cwd, task_id:"gate-task", task_subject:"Gate task"}')"
if printf '%s' "$payload" | "$GATE" >/dev/null 2>&1; then
  printf 'FAIL: in-progress canonical task was not blocked\n' >&2
  exit 1
fi

kbd guard evaluate --boundary task --edge before --subject gate-task --json --precommit --repair-projections
kbd guard evaluate --boundary task --edge before --subject gate-task --json --repair-projections
kbd task transition --command-id gate-task-finish --phase gate-phase --change gate-change --id gate-task --status complete
if printf '%s' "$payload" | "$GATE" >/dev/null 2>&1; then
  printf 'FAIL: task without signed after receipt was not blocked\n' >&2
  exit 1
fi

kbd guard evaluate --boundary task --edge after --subject gate-task --json --precommit --repair-projections
kbd guard evaluate --boundary task --edge after --subject gate-task --json --repair-projections
printf '%s' "$payload" | "$GATE" >/dev/null

status="$(prometheus kbd --path "$SANDBOX" status --json)"
printf '%s' "$status" | jq -e '
  .phases["gate-phase"].changes["gate-change"].tasks["gate-task"].status == "complete" and
  .latestBoundaryReceipts["task:gate-task"].edge == "after" and
  (.outstandingBoundaryObligations | length) == 0
' >/dev/null

printf 'pass: Claude TaskCompleted blocks tracked tasks until canonical completion and a signed after receipt\n'
