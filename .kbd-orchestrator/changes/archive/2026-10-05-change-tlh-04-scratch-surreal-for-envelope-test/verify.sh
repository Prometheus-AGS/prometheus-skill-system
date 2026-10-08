#!/usr/bin/env bash
# Generated from verification.md for change-tlh-04-scratch-surreal-for-envelope-test. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-04}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; } ) || exit $?
( /bin/bash skills/process/agent-team-creator/tests/run-memory-envelope.sh ) || exit $?
( if grep -q 23001 skills/process/agent-team-creator/tests/memory-envelope.integration.mjs; then echo 'compiled .mjs still references :23001 (rebuild with npm run build:tests)' >&2; exit 1; fi ) || exit $?
( ( cd skills/process/agent-team-creator/runtime && unset SURREAL_MEMORY_URL; node ../tests/memory-envelope.integration.mjs; test $? -eq 2 ) ) || exit $?
( /bin/bash shared/scripts/tests/test-kbd-memory-loop.sh ) || exit $?
( /bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tlh-04-scratch-surreal-for-envelope-test"
