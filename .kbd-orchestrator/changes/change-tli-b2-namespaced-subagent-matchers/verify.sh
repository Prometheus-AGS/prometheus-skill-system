#!/usr/bin/env bash
# Generated from verification.md for change-tli-b2-namespaced-subagent-matchers. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b2}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( test -s .kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/b2-claude-iterative-evolver-payload.json || test -s "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b2-claude-iterative-evolver-payload.json" ) || exit $?
( node --test scripts/tests/subagent-matchers.test.mjs ) || exit $?
( node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist ) || exit $?
( npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex ) || exit $?
echo "verify OK: change-tli-b2-namespaced-subagent-matchers"
