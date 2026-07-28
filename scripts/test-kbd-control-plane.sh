#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
BASH_BIN="${BASH_BIN:-bash}"

run() {
  printf '\n=== %s ===\n' "$1"
  shift
  "$@"
}

run "KBD state schema conformance" node scripts/validate-kbd-state.js
run "KBD compatibility projection writer boundary" node scripts/check-kbd-direct-writers.js
run "Cross-harness adapter parity" node scripts/check-harness-adapters.js
run "Critical skill evaluation corpus" node scripts/check-skill-evals.js
run "Cross-harness emergency controls" "$BASH_BIN" shared/scripts/tests/test-kbd-harness-adapter.sh
run "Stop-hook operator authority" "$BASH_BIN" shared/scripts/tests/test-position-stop-gate.sh
run "Prompt steering suspension" "$BASH_BIN" shared/scripts/tests/test-position-on-prompt.sh
run "Position rendering" "$BASH_BIN" shared/scripts/tests/test-position-render.sh
run "Next-phase installed payload" "$BASH_BIN" shared/scripts/tests/test-kbd-next-phase-packaging.sh
run "Pipeline enforcement" "$BASH_BIN" shared/scripts/tests/test-pipeline-smoke.sh
run "Clean 14-target install parity" "$BASH_BIN" shared/scripts/tests/test-clean-install-parity.sh

while IFS= read -r fixture; do
  run "Fixture $(basename "$fixture")" "$BASH_BIN" "$fixture"
done < <(find skills/process/kbd-process-orchestrator/shared/lib/tests \
  -maxdepth 1 -type f -name 'test-*.sh' | sort)

printf '\nAll KBD control-plane fixtures passed.\n'
