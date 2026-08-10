#!/usr/bin/env bash
# Compatibility wrapper. The canonical helper is bundled with the skill so
# installed skill packages remain self-contained.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="${REPO_ROOT}/skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh"

if [[ ! -x "$SCRIPT" ]]; then
  echo "[kbd-next-phase] ERROR: Bundled helper is missing or not executable: $SCRIPT" >&2
  exit 1
fi

exec "$SCRIPT" "$@"
