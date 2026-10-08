#!/usr/bin/env bash
# Generated from verification.md for change-tlh-07-cortex-mirror-real-service. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-07}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( ls -d ~/.claude/plugins/cache/cortex/cortex/*/dist/mcp-server.js >/dev/null 2>&1 || { echo 'BLOCKED: Cortex not installed' >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-cortex-mirror.sh ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tlh-07-cortex-mirror-real-service"
