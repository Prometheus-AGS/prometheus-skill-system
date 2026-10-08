#!/usr/bin/env bash
# Generated from verification.md for change-tli-b5-subagentstart-delivery-alpha. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b5}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; } ) || exit $?
( command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; } ) || exit $?
( command -v claude >/dev/null && command -v codex >/dev/null || { echo 'BLOCKED: claude and codex CLIs required' >&2; exit 2; } ) || exit $?
( node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist ) || exit $?
( npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex ) || exit $?
( /bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness both ) || exit $?
( test -s "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b5-codex-trust-path.md" ) || exit $?
echo "verify OK: change-tli-b5-subagentstart-delivery-alpha"
