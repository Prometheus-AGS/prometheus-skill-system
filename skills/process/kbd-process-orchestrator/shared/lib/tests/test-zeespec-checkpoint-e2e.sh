#!/usr/bin/env bash
# Full integration proof for ZeeSpec's three completion checkpoints against the
# signed KBD boundary detector. Uses the real CLI and a disposable state store.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd -P)"
CHECKPOINT="$ROOT/../zeespec-interrogator/scripts/state-checkpoint.sh"
STATE_INIT="$ROOT/../zeespec-interrogator/scripts/state-init.sh"
PROJECT_ROOT="$(cd "$ROOT/../../.." && pwd -P)"

command -v jq >/dev/null 2>&1 || { printf 'FAIL: jq is required\n' >&2; exit 1; }
command -v prometheus >/dev/null 2>&1 || { printf 'FAIL: prometheus is required\n' >&2; exit 1; }

STATE_DIR="$(mktemp -d)"
trap 'rm -rf "$STATE_DIR"' EXIT
export ZEESPEC_STATE_DIR="$STATE_DIR"
export KBD_ORCHESTRATOR_ROOT="$ROOT"

cd "$PROJECT_ROOT"
before="$(prometheus kbd --path . status --json)"
before_revision="$(printf '%s' "$before" | jq -r '.revision')"

"$STATE_INIT" bottleneck-e2e kbd >/dev/null
for phase in interrogate score manifest; do
  output="$("$CHECKPOINT" bottleneck-e2e "$phase" phase_complete 2>&1)"
  printf '%s' "$output" | grep -q "Position:" \
    || { printf 'FAIL: %s checkpoint omitted canonical position\n' "$phase" >&2; exit 1; }
done

state="$STATE_DIR/subjects/bottleneck-e2e/state.json"
jq -e '
  (.checkpoints | length) == 3 and
  ([.checkpoints[].phase] == ["interrogate", "score", "manifest"])
' "$state" >/dev/null

for phase in interrogate score manifest; do
  checkpoint="$(find "$STATE_DIR/subjects/bottleneck-e2e/checkpoints" -name "$phase-*.json" -type f)"
  [ -n "$checkpoint" ] || { printf 'FAIL: missing %s checkpoint\n' "$phase" >&2; exit 1; }
  jq -e --arg phase "$phase" '
    .phase == $phase and
    .event_type == "phase_complete" and
    (.source_state_sha256 | length) == 64 and
    (.state_snapshot | type) == "object"
  ' "$checkpoint" >/dev/null
done

after="$(prometheus kbd --path . status --json)"
after_revision="$(printf '%s' "$after" | jq -r '.revision')"
[ "$after_revision" -eq $((before_revision + 6)) ] \
  || { printf 'FAIL: expected six signed ZeeSpec boundaries, got revisions %s -> %s\n' "$before_revision" "$after_revision" >&2; exit 1; }
printf '%s' "$after" | jq -e '
  [.outstandingBoundaryObligations[]? | select(.boundary == "zeespec")] | length == 0
' >/dev/null

if find "$STATE_DIR" -type f -name '.*.tmp*' | grep -q .; then
  printf 'FAIL: atomic checkpoint left temporary files\n' >&2
  exit 1
fi

printf 'pass: ZeeSpec interrogate, score, and manifest checkpoints committed atomically with signed detector receipts\n'
