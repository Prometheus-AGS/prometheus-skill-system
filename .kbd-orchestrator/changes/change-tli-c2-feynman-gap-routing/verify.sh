#!/usr/bin/env bash
# Generated from verification.md for change-tli-c2-feynman-gap-routing. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-c2}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; } ) || exit $?
( pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-prompt-gap.sh ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tli-c2-feynman-gap-routing"
