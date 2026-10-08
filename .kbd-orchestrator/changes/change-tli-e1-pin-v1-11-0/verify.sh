#!/usr/bin/env bash
# Generated from verification.md for change-tli-e1-pin-v1-11-0. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-e1}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git -C tools/prometheus-knowledge rev-list -n1 v1.11.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b" ) || exit $?
( npm run check:distribution ) || exit $?
( (cd tools/prometheus-cli && cargo check) && (cd tools/forge-rs && cargo check) ) || exit $?
( bash scripts/install-binaries.sh ) || exit $?
( pk --version | grep -Eq '(^|[^0-9.])1\.11\.0([^0-9]|$)' ) || exit $?
( pk candidates list --kind promotion >/dev/null ) || exit $?
echo "verify OK: change-tli-e1-pin-v1-11-0"
