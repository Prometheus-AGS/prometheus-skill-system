#!/usr/bin/env bash
# subagent-checkpoint-fallback.sh — SubagentStop fallback: generic checkpoint for unknown agents.
# Must always exit 0.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SubagentStop" "subagent-checkpoint-fallback.sh"

AGENT_NAME="${SUBAGENT_NAME:-unknown}"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo 'unknown')"
echo "SubagentStop checkpoint: agent=${AGENT_NAME} at=${TIMESTAMP}" >&2

hook_log_end 0
exit 0
