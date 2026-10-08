#!/usr/bin/env bash
# Generated from verification.md for change-tli-b7-cross-agent-awareness-beta. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b7}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; } ) || exit $?
( command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; } ) || exit $?
( command -v claude >/dev/null && command -v codex >/dev/null || { echo 'BLOCKED: claude and codex CLIs required' >&2; exit 2; } ) || exit $?
( pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-team-awareness.sh --harness both ) || exit $?
( python3 scripts/report-learning-delivery.py --baseline-claude 14336 --baseline-codex 11059 --require-reduction ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tli-b7-cross-agent-awareness-beta"
