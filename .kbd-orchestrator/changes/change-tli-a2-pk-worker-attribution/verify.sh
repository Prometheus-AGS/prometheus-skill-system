#!/usr/bin/env bash
# Generated from verification.md for change-tli-a2-pk-worker-attribution. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-a2}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 1bbaecc HEAD || { echo "base does not contain 1bbaecc" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( cargo test -p pk-learning-worker --test attribution ) || exit $?
( cargo test -p pk-learning-worker --test worker ) || exit $?
echo "verify OK: change-tli-a2-pk-worker-attribution"
