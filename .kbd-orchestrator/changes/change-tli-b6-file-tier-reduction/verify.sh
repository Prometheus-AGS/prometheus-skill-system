#!/usr/bin/env bash
# Generated from verification.md for change-tli-b6-file-tier-reduction. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b6}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; } ) || exit $?
( /bin/bash scripts/tests/test-memory-partition.sh ) || exit $?
( awk '/^after:/{exit !($2<=4096)}' "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b6-live-index-size.txt" ) || exit $?
( cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/export.integration.mjs ) || exit $?
echo "verify OK: change-tli-b6-file-tier-reduction"
