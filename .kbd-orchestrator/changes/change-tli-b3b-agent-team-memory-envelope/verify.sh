#!/usr/bin/env bash
# Generated from verification.md for change-tli-b3b-agent-team-memory-envelope. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b3b}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; } ) || exit $?
( cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/memory-envelope.integration.mjs ../tests/models-memory.integration.mjs ) || exit $?
echo "verify OK: change-tli-b3b-agent-team-memory-envelope"
