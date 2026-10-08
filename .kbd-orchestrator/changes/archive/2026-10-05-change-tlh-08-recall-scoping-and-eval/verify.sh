#!/usr/bin/env bash
# Generated from verification.md for change-tlh-08-recall-scoping-and-eval. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-08}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( test -f shared/scripts/tests/lib/scratch-surreal.sh || { echo 'BLOCKED: dependency change-tlh-04 not in this base (rebase onto main after it merges)' >&2; exit 2; } ) || exit $?
( command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; } ) || exit $?
( command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-recall-quality.sh ) || exit $?
( /bin/bash shared/scripts/tests/test-kbd-memory-loop.sh ) || exit $?
( /bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tlh-08-recall-scoping-and-eval"
