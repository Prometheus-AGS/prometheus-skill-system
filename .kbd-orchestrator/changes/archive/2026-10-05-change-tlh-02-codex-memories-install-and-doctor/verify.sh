#!/usr/bin/env bash
# Generated from verification.md for change-tlh-02-codex-memories-install-and-doctor. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-02}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor e421715 HEAD || { echo "base does not contain e421715" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( /bin/bash shared/scripts/tests/test-codex-memories-config.sh ) || exit $?
( cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor ) || exit $?
( npm run check:distribution ) || exit $?
echo "verify OK: change-tlh-02-codex-memories-install-and-doctor"
