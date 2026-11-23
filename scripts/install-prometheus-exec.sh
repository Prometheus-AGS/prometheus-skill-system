#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=prometheus-exec-pin.sh
. "${REPO_ROOT}/scripts/prometheus-exec-pin.sh"
EXPECTED_VERSION="${PROMETHEUS_EXEC_EXPECTED_VERSION:-prometheus-exec 1.7.0}"
HASH_MANIFEST="${PROMETHEUS_EXEC_HASH_MANIFEST:-${REPO_ROOT}/config/prometheus-exec-binary.json}"
EXPECTED_BUILD_HASH="${PROMETHEUS_EXEC_EXPECTED_SHA256:-}"
BIN_DIR="${PROMETHEUS_EXEC_BIN_DIR:-${HOME}/.local/bin}"
DESTINATION="${BIN_DIR}/prometheus-exec"
MANIFEST_DIR="${PROMETHEUS_EXEC_MANIFEST_DIR:-${HOME}/.prometheus/install/manifests}"
MANIFEST_PATH="${MANIFEST_DIR}/prometheus-exec.json"
BACKUP_DIR="${PROMETHEUS_EXEC_BACKUP_DIR:-${HOME}/.prometheus/install/backups}"
SOURCE_BIN="${PROMETHEUS_EXEC_SOURCE_BIN:-${REPO_ROOT}/crates/prometheus-exec/target/release/prometheus-exec}"
DRY_RUN=false

if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN=true
elif [ "$#" -gt 0 ]; then
    echo "unknown argument: $1" >&2
    exit 2
fi

sha256_file() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        sha256sum "$1" | awk '{print $1}'
    fi
}

verify_version() {
    local binary="$1" output
    output="$("$binary" --version 2>/dev/null)" || return 1
    [ "$output" = "$EXPECTED_VERSION" ]
}

sign_and_verify() {
    local binary="$1"
    if [ "${PROMETHEUS_EXEC_PLATFORM:-$(uname -s)}" != "Darwin" ]; then
        return 0
    fi
    local signer="${PROMETHEUS_EXEC_CODESIGN:-codesign}"
    "$signer" --force --sign - "$binary" >/dev/null
    "$signer" --verify --strict "$binary" >/dev/null
}

verify_signature() {
    local binary="$1"
    if [ "${PROMETHEUS_EXEC_PLATFORM:-$(uname -s)}" != "Darwin" ]; then
        return 0
    fi
    local signer="${PROMETHEUS_EXEC_CODESIGN:-codesign}"
    "$signer" --verify --strict "$binary" >/dev/null
}

# Release verification authenticates the archive, platform manifest, and any
# independent target pin before this installer probes or signs the executable.
if $DRY_RUN; then
    echo "[dry-run] would select the verified release or build crates/prometheus-exec --release, then pin, sign and verify $DESTINATION"
    exit 0
fi
release_stage=""
owned_release_stage=""
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*|Windows*) DESTINATION="${DESTINATION}.exe";; esac
if [ -n "${PROMETHEUS_EXEC_RELEASE_MANIFEST:-}" ]; then
    release_stage="$(dirname "$PROMETHEUS_EXEC_RELEASE_MANIFEST")"
    node "$REPO_ROOT/scripts/binary-release.mjs" verify "$release_stage"
elif [ -z "${PROMETHEUS_EXEC_SOURCE_BIN:-}" ] && [ "${PROMETHEUS_BINARY_RELEASE_PROBED:-}" != 1 ]; then
    . "$REPO_ROOT/scripts/binary-release.sh"
    prepare_binary_release
    if [ -z "${PROMETHEUS_BINARY_RELEASE_STAGE:-}" ]; then owned_release_stage="$release_stage"; fi
fi
RELEASE_TAG=""
RELEASE_PLATFORM=""
RELEASE_MANIFEST_HASH=""
if [ -n "$release_stage" ] && [ -f "$release_stage/manifest.json" ]; then
    read -r RELEASE_TAG RELEASE_PLATFORM RELEASE_HASH BUILD_TARGET < <(node -e '
      const manifest=require(process.argv[1]), catalog=require(process.argv[2]);
      const exec=manifest.files.find(entry => entry.name==="prometheus-exec");
      console.log(manifest.tag,manifest.platform,exec.sha256,catalog.platforms[manifest.platform].target);
    ' "$release_stage/manifest.json" "$REPO_ROOT/config/release-binaries.json")
    SOURCE_BIN="$release_stage/bin/$(basename "$DESTINATION")"
    RELEASE_MANIFEST_HASH="$(sha256_file "$release_stage/manifest.json")"
    EXPECTED_BUILD_HASH="$(prometheus_exec_pin "$HASH_MANIFEST" "$BUILD_TARGET")"
    EXPECTED_BUILD_HASH="${EXPECTED_BUILD_HASH:-$RELEASE_HASH}"
else
    BUILD_TARGET="$(prometheus_exec_host_target "${REPO_ROOT}/crates/prometheus-exec")"
    EXPECTED_BUILD_HASH="${EXPECTED_BUILD_HASH:-$(prometheus_exec_pin "$HASH_MANIFEST" "$BUILD_TARGET")}"
fi
if [[ ! "$EXPECTED_BUILD_HASH" =~ ^[0-9a-f]{64}$ ]]; then
    echo "No published binary or certified source build hash for ${BUILD_TARGET:-unknown}; certify the source build before installation" >&2
    exit 1
fi

if [ -z "$RELEASE_TAG" ] && [ -z "${PROMETHEUS_EXEC_SOURCE_BIN:-}" ]; then
    # build-prometheus-exec.sh makes the release binary independent of the
    # checkout path (staged workspace + --remap-path-prefix) and builds with
    # the crate's pinned toolchain. A plain `cargo build --release` embeds the
    # checkout path, so its hash matches only the checkout that certified it
    # and every git worktree build fails the certified-hash gate below.
    bash "${REPO_ROOT}/scripts/build-prometheus-exec.sh" >/dev/null
fi
[ -f "$SOURCE_BIN" ] || { echo "prometheus-exec build artifact is missing: $SOURCE_BIN" >&2; exit 1; }
[ -x "$SOURCE_BIN" ] || { echo "prometheus-exec build artifact is not executable: $SOURCE_BIN" >&2; exit 1; }
BUILD_HASH="$(sha256_file "$SOURCE_BIN")"
if [ "$BUILD_HASH" != "$EXPECTED_BUILD_HASH" ]; then
    echo "prometheus-exec source hash mismatch for target ${BUILD_TARGET:-unknown}; expected $EXPECTED_BUILD_HASH, got $BUILD_HASH" >&2
    echo "if crates/prometheus-exec, substrate/exec-*, wit/ or the toolchain changed on purpose, recertify EVERY target in expectedBuildSha256ByTarget, each on a machine of that target, with: bash scripts/build-prometheus-exec.sh --print-hash" >&2
    exit 1
fi

verify_version "$SOURCE_BIN" || {
    echo "prometheus-exec source version mismatch; expected: $EXPECTED_VERSION" >&2
    exit 1
}

mkdir -p "$BIN_DIR" "$MANIFEST_DIR" "$BACKUP_DIR"
chmod 700 "$MANIFEST_DIR" "$BACKUP_DIR"
STAGED="$(mktemp "${BIN_DIR}/.prometheus-exec.stage.XXXXXX")"
MANIFEST_STAGE="$(mktemp "${MANIFEST_DIR}/.prometheus-exec.manifest.XXXXXX")"
BACKUP_PATH=""
cleanup() {
    rm -f "$STAGED" "$MANIFEST_STAGE"
    if [ -n "$owned_release_stage" ]; then rm -rf "$owned_release_stage"; fi
}
trap cleanup EXIT INT TERM

install -m 0755 "$SOURCE_BIN" "$STAGED"
sign_and_verify "$STAGED" || {
    echo "prometheus-exec staged signature verification failed" >&2
    exit 1
}
verify_version "$STAGED" || {
    echo "prometheus-exec staged version mismatch after signing" >&2
    exit 1
}
STAGED_HASH="$(sha256_file "$STAGED")"

if [ -e "$DESTINATION" ]; then
    EXISTING_HASH="$(sha256_file "$DESTINATION")"
    BACKUP_PATH="${BACKUP_DIR}/prometheus-exec.${EXISTING_HASH}"
    if [ ! -e "$BACKUP_PATH" ]; then
        BACKUP_STAGE="$(mktemp "${BACKUP_DIR}/.prometheus-exec.backup.XXXXXX")"
        cp "$DESTINATION" "$BACKUP_STAGE"
        chmod 700 "$BACKUP_STAGE"
        mv -f "$BACKUP_STAGE" "$BACKUP_PATH"
    fi
fi

mv -f "$STAGED" "$DESTINATION"
STAGED=""
rollback() {
    if [ -n "$BACKUP_PATH" ] && [ -f "$BACKUP_PATH" ]; then
        cp "$BACKUP_PATH" "$DESTINATION"
        chmod 755 "$DESTINATION"
    else
        rm -f "$DESTINATION"
    fi
}

if ! verify_version "$DESTINATION"; then
    rollback
    echo "prometheus-exec installed version readback failed; prior binary restored" >&2
    exit 1
fi
if ! verify_signature "$DESTINATION"; then
    rollback
    echo "prometheus-exec installed signature readback failed; prior binary restored" >&2
    exit 1
fi
INSTALLED_HASH="$(sha256_file "$DESTINATION")"
if [ "$INSTALLED_HASH" != "$STAGED_HASH" ]; then
    rollback
    echo "prometheus-exec installed hash readback failed; prior binary restored" >&2
    exit 1
fi

signature=not-applicable
if [ "${PROMETHEUS_EXEC_PLATFORM:-$(uname -s)}" = Darwin ]; then signature=verified; fi
node - "$MANIFEST_STAGE" "$signature" "$EXPECTED_VERSION" "$BUILD_TARGET" "$EXPECTED_BUILD_HASH" "$SOURCE_BIN" "$DESTINATION" "$BUILD_HASH" "$INSTALLED_HASH" "$RELEASE_TAG" "$RELEASE_PLATFORM" "$RELEASE_MANIFEST_HASH" <<'JS'
const [file,signature,version,target,expectedBuildSha256,source,destination,buildSha256,installedSha256,releaseTag,releasePlatform,releaseManifestSha256] = process.argv.slice(2);
const info = {schemaVersion:1,name:'prometheus-exec',version,target,expectedBuildSha256,source,destination,buildSha256,installedSha256,signature};
if (releaseTag) Object.assign(info, {releaseTag,releasePlatform,releaseManifestSha256});
require('node:fs').writeFileSync(file, JSON.stringify(info, null, 2) + '\n');
JS
chmod 600 "$MANIFEST_STAGE"
mv -f "$MANIFEST_STAGE" "$MANIFEST_PATH"
MANIFEST_STAGE=""
if [ -f "$REPO_ROOT/scripts/binary-release.mjs" ]; then
    if [ -n "$RELEASE_TAG" ]; then
        node "$REPO_ROOT/scripts/binary-release.mjs" mark "$DESTINATION" "$release_stage" "$BUILD_HASH"
    else
        node "$REPO_ROOT/scripts/binary-release.mjs" mark "$DESTINATION"
    fi
fi

echo "prometheus-exec installed and verified: $DESTINATION ($EXPECTED_VERSION, sha256:$INSTALLED_HASH)"
