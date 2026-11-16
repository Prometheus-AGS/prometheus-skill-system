#!/usr/bin/env bash
# cargo_release_dir <workspace-dir> — print the directory cargo writes that
# workspace's release binaries to. Sourced by install-binaries.sh and
# install-skills-flat.sh.
#
# Never assume <workspace-dir>/target. `build.target-dir` (in any
# .cargo/config.toml up the tree or in $CARGO_HOME) and CARGO_TARGET_DIR move
# build output elsewhere. With a redirected target dir, a hardcoded
# <workspace-dir>/target/release path is either missing, which aborted
# install-binaries.sh at the first such binary and left every later one
# uninstalled, or it still holds a binary from an older build, which was then
# installed as if it were the one just compiled. Both were observed on
# 2026-11-16: `prometheus` 1.14.0 was installed immediately after 1.15.0
# compiled.
#
# Asks cargo from inside the workspace so directory-scoped config applies.
# Falls back to <workspace-dir>/target when cargo cannot answer, and always
# returns 0 so a caller running under `set -e` is never aborted by the lookup.

cargo_release_dir() {
    local dir="$1" target_dir=""
    if command -v cargo >/dev/null 2>&1; then
        target_dir="$(cd "${dir}" 2>/dev/null \
            && cargo metadata --format-version 1 --no-deps 2>/dev/null \
            | tr ',' '\n' \
            | awk -F'"' '$2 == "target_directory" { print $4; exit }')" || true
    fi
    printf '%s/release\n' "${target_dir:-${dir}/target}"
}
