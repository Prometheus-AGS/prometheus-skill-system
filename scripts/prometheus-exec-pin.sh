#!/usr/bin/env bash
# Look up the certified prometheus-exec build hash for the machine doing the
# build. Sourced by install-prometheus-exec.sh and its integration test.
#
# config/prometheus-exec-binary.json pins one hash PER RUST HOST TARGET in
# expectedBuildSha256ByTarget. A release binary for x86_64-apple-darwin and one
# for aarch64-apple-darwin are different files, so a single hash can only ever
# match the architecture that certified it: until 2026-11-16 each
# recertification on one of the two development machines made the installer
# reject every build on the other.
#
# The manifest is read with awk, not jq, to match the installer's existing
# dependencies. That requires one map entry per line:
#     "x86_64-apple-darwin": "<64 hex chars>",

# prometheus_exec_host_target [crate-dir] — print the Rust host target triple
# the build will produce a binary for.
#
# Asks rustc from inside the crate directory, because rust-toolchain.toml is
# resolved from the current directory and an x86_64 toolchain on Apple Silicon
# builds x86_64 binaries. Falls back to the kernel's view when rustc is absent
# (a dry run, or an install from a prebuilt PROMETHEUS_EXEC_SOURCE_BIN).
prometheus_exec_host_target() {
    local crate_dir="${1:-.}" target=""
    if command -v rustc >/dev/null 2>&1; then
        target="$(cd "$crate_dir" 2>/dev/null \
            && rustc -vV 2>/dev/null | awk '/^host:/ { print $2; exit }')" || true
    fi
    if [ -z "$target" ]; then
        case "$(uname -s)-$(uname -m)" in
            Darwin-arm64)  target="aarch64-apple-darwin" ;;
            Darwin-x86_64) target="x86_64-apple-darwin" ;;
            Linux-x86_64)  target="x86_64-unknown-linux-gnu" ;;
            Linux-aarch64) target="aarch64-unknown-linux-gnu" ;;
        esac
    fi
    printf '%s\n' "$target"
}

# prometheus_exec_pin <manifest> <target> — print the certified build hash for
# that target, or nothing when the manifest has none.
#
# A manifest without the per-target map (a release from before it existed)
# still carries the single legacy expectedBuildSha256, which is used as is.
prometheus_exec_pin() {
    local manifest="$1" target="$2" pin=""
    [ -f "$manifest" ] || return 0
    pin="$(awk -F'"' -v target="$target" '
        /"expectedBuildSha256ByTarget"/ { in_map = 1; next }
        in_map && /}/ { in_map = 0 }
        in_map && $2 == target { print $4; exit }
    ' "$manifest")" || true
    if [ -z "$pin" ]; then
        pin="$(awk -F'"' '/"expectedBuildSha256"/ { print $4; exit }' "$manifest")" || true
    fi
    printf '%s\n' "$pin"
}
