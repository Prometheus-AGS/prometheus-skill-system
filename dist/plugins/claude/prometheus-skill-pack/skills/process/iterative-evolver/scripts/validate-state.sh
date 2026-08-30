#!/usr/bin/env bash
# validate-state.sh — Validates evolution_state.json after file writes
# Exit 0 = OK, Exit 2 = feedback to agent

set -euo pipefail

# Observability: emit a start/end record to ~/.prometheus/hooks.log so this
# hook is visible to `doctor` and to latency analysis. The library no-ops when
# the log directory is not writable, and never changes this script's exit code.
HOOK_LOG_LIB="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/shared/scripts/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && . "$HOOK_LOG_LIB"
command -v hook_log_start >/dev/null 2>&1 && hook_log_start "PostToolUse" "validate-state.sh"
command -v hook_log_end >/dev/null 2>&1 && trap 'hook_log_end $?' EXIT


STATE_FILE="evolution_state.json"

# Only validate if state file exists
if [ ! -f "$STATE_FILE" ]; then
  exit 0
fi

# Check valid JSON
if ! python3 -c "import json; json.load(open('$STATE_FILE'))" 2>/dev/null; then
  echo "⚠️  evolution_state.json is not valid JSON"
  exit 2
fi

# Check required fields
REQUIRED_FIELDS='["evolution_id", "domain", "current_iteration", "convergence_status"]'
MISSING=$(python3 -c "
import json, sys
state = json.load(open('$STATE_FILE'))
required = $REQUIRED_FIELDS
missing = [f for f in required if f not in state]
if missing:
    print(f'Missing fields: {missing}')
    sys.exit(1)
" 2>&1) || {
  echo "⚠️  $MISSING"
  exit 2
}

exit 0
