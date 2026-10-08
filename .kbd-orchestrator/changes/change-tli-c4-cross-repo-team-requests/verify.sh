#!/usr/bin/env bash
# Generated from verification.md for change-tli-c4-cross-repo-team-requests. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-c4}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( test -n "${PROMETHEUS_TEAM_TEST_REPO:-}" || { echo "BLOCKED: PROMETHEUS_TEAM_TEST_REPO unset (sandbox repo for issue creation)" >&2; exit 2; } ) || exit $?
( command -v gh >/dev/null || { echo 'BLOCKED: gh required' >&2; exit 2; } ) || exit $?
( cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/teams.integration.mjs ) || exit $?
echo "verify OK: change-tli-c4-cross-repo-team-requests"
