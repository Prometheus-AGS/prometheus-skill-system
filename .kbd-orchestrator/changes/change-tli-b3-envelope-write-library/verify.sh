#!/usr/bin/env bash
# Generated from verification.md for change-tli-b3-envelope-write-library. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b3}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; } ) || exit $?
( command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; } ) || exit $?
( pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-learning-write.sh ) || exit $?
( node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist ) || exit $?
( npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex ) || exit $?
echo "verify OK: change-tli-b3-envelope-write-library"
