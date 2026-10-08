#!/usr/bin/env bash
# Generated from verification.md for change-tli-d2-mini-hook-timeouts-seconds. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-d2}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 56cbf12 HEAD || { echo "base does not contain 56cbf12" >&2; exit 1; }
( node -e 'const h=require("./hooks/hooks.json").hooks;for(const g of Object.values(h).flat())for(const x of g.hooks)if(x.timeout!==undefined&&(x.timeout<1||x.timeout>600))process.exit(1)' ) || exit $?
( node --test hooks/hooks.test.mjs lib/karpathy/hooks-budget.test.mjs ) || exit $?
( node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tli-d2-mini-hook-timeouts-seconds"
