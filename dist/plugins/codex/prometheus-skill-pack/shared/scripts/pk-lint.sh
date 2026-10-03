#!/usr/bin/env bash
# pk-lint: scheduled weekly linting of all skills via pk lint --fix
# Invoked by launchd (macOS) or cron (Linux) weekly.
# Writes: ~/.prometheus/pk-lint.log

set -euo pipefail

LOG_FILE="${HOME}/.prometheus/pk-lint.log"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

mkdir -p "$(dirname "$LOG_FILE")"

log() {
  echo "[${TIMESTAMP}] $*" | tee -a "$LOG_FILE"
}

log "Starting pk lint --fix"

if ! command -v pk &>/dev/null; then
  log "WARN: pk binary not found — skipping lint. Install prometheus-knowledge to enable."
  exit 0
fi

# Run from the skill-pack root if we can locate it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACK_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if [[ -f "${PACK_ROOT}/package.json" ]]; then
  cd "${PACK_ROOT}"
  log "Running from: ${PACK_ROOT}"
fi

if pk lint --fix 2>>"$LOG_FILE"; then
  log "Completed pk lint --fix — no violations"
else
  EXIT_CODE=$?
  log "WARN: pk lint --fix exited with code ${EXIT_CODE} — check log for violations"
fi
