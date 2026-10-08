#!/usr/bin/env bash
# Generated from verification.md for change-tlh-06-ledger-reconciliation. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-06}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( test -f skills/process/delivery-cadence/scripts/refresh-skill-pack.sh || { echo 'BLOCKED: dependency change-tlh-03 not in this base (rebase onto main after it merges)' >&2; exit 2; } ) || exit $?
( command -v prometheus >/dev/null || { echo 'BLOCKED: prometheus CLI not on PATH' >&2; exit 2; } ) || exit $?
( /bin/bash shared/scripts/tests/test-kbd-apply-reconcile.sh ) || exit $?
( /bin/bash shared/scripts/tests/test-cadence-refresh-procedure.sh ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tlh-06-ledger-reconciliation"
