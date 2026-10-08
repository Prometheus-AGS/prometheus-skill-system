#!/usr/bin/env bash
# Generated from verification.md for change-tlh-09-rebase-regenerate. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-09}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( node scripts/generated-paths.mjs | grep -q '^dist/' || { echo 'generated-paths lists no dist paths' >&2; exit 1; } ) || exit $?
( grep -q generated-paths scripts/generate-skill-system-distribution.js || { echo 'distribution check does not use generated-paths' >&2; exit 1; } ) || exit $?
( /bin/bash scripts/tests/test-rebase-regenerate.sh ) || exit $?
( npm run check:distribution ) || exit $?
( npm run validate:harness-adapters ) || exit $?
( npm run validate:codex ) || exit $?
echo "verify OK: change-tlh-09-rebase-regenerate"
