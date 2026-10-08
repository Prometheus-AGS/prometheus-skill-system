#!/usr/bin/env bash
# Generated from verification.md for change-tli-a4-surreal-lean-search-categories-rekey. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-a4}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 777cf72 HEAD || { echo "base does not contain 777cf72" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( cargo test --test team_scoping 2>&1 | tee "${TMPDIR:-/tmp}/tli-a4-test.$$.log"; grep -Eq 'test result: ok\. ([4-9]|[1-9][0-9]+) passed' "${TMPDIR:-/tmp}/tli-a4-test.$$.log" || { echo 'fewer than 4 team_scoping tests passed' >&2; exit 1; } ) || exit $?
( cargo test -p surreal-memory --test search_correctness ) || exit $?
echo "verify OK: change-tli-a4-surreal-lean-search-categories-rekey"
