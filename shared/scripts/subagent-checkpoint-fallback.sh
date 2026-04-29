#!/usr/bin/env bash
# subagent-checkpoint-fallback.sh — SubagentStop fallback: generic checkpoint for unknown agents.
# Must always exit 0.
set -uo pipefail
AGENT_NAME="${SUBAGENT_NAME:-unknown}"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo 'unknown')"
echo "SubagentStop checkpoint: agent=${AGENT_NAME} at=${TIMESTAMP}" >&2
exit 0
