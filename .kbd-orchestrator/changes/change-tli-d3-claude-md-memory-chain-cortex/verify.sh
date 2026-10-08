#!/usr/bin/env bash
# Generated from verification.md for change-tli-d3-claude-md-memory-chain-cortex. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-d3}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( /bin/bash shared/scripts/tests/test-cortex-mirror.sh ) || exit $?
( if grep -n 'Read ~/.claude/projects/.*/memory/MEMORY.md' CLAUDE.md AGENTS.md; then echo 'full-index instruction remains' >&2; exit 1; fi ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tli-d3-claude-md-memory-chain-cortex"
