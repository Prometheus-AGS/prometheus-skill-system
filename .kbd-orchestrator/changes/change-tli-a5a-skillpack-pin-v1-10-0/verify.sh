#!/usr/bin/env bash
# Generated from verification.md for change-tli-a5a-skillpack-pin-v1-10-0. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLI_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tli-a5a}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 206ebbd HEAD || { echo "base does not contain 206ebbd" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git -C tools/prometheus-knowledge rev-list -n1 v1.10.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b" ) || exit $?
( a="$(git ls-tree HEAD tools/surreal-memory-server | awk "{print \$3}")"; b="$(git -C tools/surreal-memory-server rev-list -n1 v1.10.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b" ) || exit $?
( npm run check:distribution ) || exit $?
( (cd tools/prometheus-cli && cargo check) && (cd tools/forge-rs && cargo check) ) || exit $?
# Amended at execute (2026-10-04): install-binaries.sh cannot run from a worktree because
# prometheus-exec's certified hash is build-path dependent (follow-up task). Prove the pins
# instead: build pk and surreal-memory-server from the pinned submodules and read versions.
( cd tools/prometheus-knowledge && cargo build --release --locked -p pk-cli -p pk-learning-worker ) || exit $?
( cd tools/surreal-memory-server && cargo build --release --locked ) || exit $?
( for b in pk prometheus-learning-worker; do f="$(cd tools/prometheus-knowledge && cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')/release/$b"; "$f" --version | grep -Eq '(^|[^0-9.])1\.10\.0([^0-9]|$)' || { echo "$b is not 1.10.0" >&2; exit 1; }; done ) || exit $?
( f="$(cd tools/surreal-memory-server && cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')/release/surreal-memory-server"; "$f" --version | grep -Eq '(^|[^0-9.])1\.10\.0([^0-9]|$)' || { echo "surreal-memory-server is not 1.10.0" >&2; exit 1; } ) || exit $?
echo "verify OK: change-tli-a5a-skillpack-pin-v1-10-0"
