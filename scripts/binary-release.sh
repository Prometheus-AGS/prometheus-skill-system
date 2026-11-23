# Sourced by Full pack installers. Parent installers can share a private
# stage; authenticate its manifest and bytes before consuming it.
release_stage=""
release_suffix=""
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*|Windows*) release_suffix=".exe";; esac
prepare_binary_release() {
    if [ -n "${PROMETHEUS_BINARY_RELEASE_STAGE:-}" ]; then
        release_stage="$PROMETHEUS_BINARY_RELEASE_STAGE"
        node "$REPO_ROOT/scripts/binary-release.mjs" verify "$release_stage"
        return
    fi
    release_stage="$(mktemp -d)" || return 1
    trap 'rm -rf "$release_stage"' EXIT
    local rc=0
    node "$REPO_ROOT/scripts/binary-release.mjs" prepare "$release_stage" || rc=$?
    case "$rc" in
        0|3) return 0 ;; # absent asset falls through to the existing source path
        *) return "$rc" ;;
    esac
}
release_has() {
    [ -n "$release_stage" ] || return 1
    local name
    for name in "$@"; do
        [ -f "$release_stage/bin/$name$release_suffix" ] || return 1
    done
}
