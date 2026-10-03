#!/usr/bin/env bash
# log-reflection.sh — Logs reflection output after reflector subagent completes
# Exit 0 = OK, Exit 2 = feedback to agent

set -euo pipefail

# Observability: emit a start/end record to ~/.prometheus/hooks.log so this
# hook is visible to `doctor` and to latency analysis. The library no-ops when
# the log directory is not writable, and never changes this script's exit code.
HOOK_LOG_LIB="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/shared/scripts/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && . "$HOOK_LOG_LIB"
command -v hook_log_start >/dev/null 2>&1 && hook_log_start "SubagentStop" "log-reflection.sh"
command -v hook_log_end >/dev/null 2>&1 && trap 'hook_log_end $?' EXIT


STATE_FILE="evolution_state.json"
LOG_FILE="evolution_log.md"
DECISIONS_FILE="decisions.md"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Check if state file exists
if [ ! -f "$STATE_FILE" ]; then
  exit 0
fi

# Extract reflection data and log it
python3 -c "
import json

state = json.load(open('$STATE_FILE'))
reflection = state.get('latest_reflection', {})
convergence = reflection.get('convergence', {})
iteration = state.get('current_iteration', '?')
domain = state.get('domain', 'unknown')

# Summary line for log
decision = convergence.get('decision', 'unknown')
rationale = convergence.get('rationale', 'No rationale provided')
alignment = convergence.get('target_alignment', '?')

summary = f'Iteration {iteration} ({domain}): {decision} — alignment {alignment}% — {rationale}'
print(f'📊 {summary}')
" 2>/dev/null || true

exit 0
