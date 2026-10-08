#!/usr/bin/env bash
# Generated from verification.md for change-tli-a5b-mini-pin-operator-request. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/prometheus-skills-mini}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
true
( git fetch -q origin ) || exit $?
( pk_c="$(git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge rev-list -n1 v1.10.0)"; sm_c="$(git -C /Users/gqadonis/Projects/prometheus/surreal-memory-server rev-list -n1 v1.10.0)"; a="$(git ls-tree origin/main tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git ls-tree origin/main tools/surreal-memory-server | awk "{print \$3}")"; test -n "$pk_c" && test -n "$sm_c" && test "$a" = "$pk_c" && test "$b" = "$sm_c" || { echo "BLOCKED-ON-OPERATOR: mini pins not yet at v1.10.0" >&2; exit 2; } ) || exit $?
echo "verify OK: change-tli-a5b-mini-pin-operator-request"
