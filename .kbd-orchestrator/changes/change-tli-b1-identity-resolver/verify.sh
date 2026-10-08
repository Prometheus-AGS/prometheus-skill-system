#!/usr/bin/env bash
# Generated from verification.md for change-tli-b1-identity-resolver. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-b1}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( /bin/bash shared/scripts/tests/test-identity.sh ) || exit $?
( if grep -rn 'PROMETHEUS_PROJECT_ID:-prometheus-skill-pack' shared; then echo 'hardcoded project id default remains' >&2; exit 1; fi ) || exit $?
( node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tli-b1-identity-resolver"
