#!/usr/bin/env bash
# Generated from verification.md for change-tli-d1b-mini-pk-tags. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-d1b}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 56cbf12 HEAD || { echo "base does not contain 56cbf12" >&2; exit 1; }
( git fetch -q origin ) || exit $?
( git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge fetch -q origin --tags; pk_c="$(git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge rev-list -n1 v1.10.0)"; a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; test -n "$pk_c" && test -n "$a" && git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge merge-base --is-ancestor "$pk_c" "$a" || { echo "BLOCKED-ON-OPERATOR: mini pk pin is not at or after v1.10.0" >&2; exit 2; } ) || exit $?
( node --test lib/karpathy/transport.test.mjs ) || exit $?
( node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi ) || exit $?
echo "verify OK: change-tli-d1b-mini-pk-tags"
