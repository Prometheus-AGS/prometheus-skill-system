#!/usr/bin/env bash
# Compatibility wrapper. The canonical helper is bundled with the skill so
# installed skill packages remain self-contained. It lives at
# skills/process/kbd-process-orchestrator/... in the source tree and at
# skills/kbd-process-orchestrator/... in generated payloads.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT=""
for candidate in \
  "${REPO_ROOT}/skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh" \
  "${REPO_ROOT}/skills/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh"; do
  if [[ -x "$candidate" ]]; then SCRIPT="$candidate"; break; fi
done

if [[ -z "$SCRIPT" ]]; then
  echo "[kbd-next-phase] ERROR: Bundled helper is missing or not executable under ${REPO_ROOT}/skills" >&2
  exit 1
fi

exec "$SCRIPT" "$@"
