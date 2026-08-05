#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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

if [ -z "$EXPECTED_BUILD_HASH" ] && [ -f "$HASH_MANIFEST" ]; then
    EXPECTED_BUILD_HASH="$(awk -F'"' '/"expectedBuildSha256"/ { print $4; exit }' "$HASH_MANIFEST")"
fi
if [[ ! "$EXPECTED_BUILD_HASH" =~ ^[0-9a-f]{64}$ ]]; then
    echo "prometheus-exec certified build hash is missing or invalid; set PROMETHEUS_EXEC_EXPECTED_SHA256 or update $HASH_MANIFEST" >&2
    exit 1
fi

if $DRY_RUN; then
    echo "[dry-run] would build crates/prometheus-exec --release"
    echo "[dry-run] would require source sha256:$EXPECTED_BUILD_HASH, then verify, stage, sign, atomically install, and read back $DESTINATION"
    exit 0
fi

if [ -z "${PROMETHEUS_EXEC_SOURCE_BIN:-}" ]; then
    cargo build --release --manifest-path "${REPO_ROOT}/crates/prometheus-exec/Cargo.toml"
fi
[ -f "$SOURCE_BIN" ] || { echo "prometheus-exec build artifact is missing: $SOURCE_BIN" >&2; exit 1; }
[ -x "$SOURCE_BIN" ] || { echo "prometheus-exec build artifact is not executable: $SOURCE_BIN" >&2; exit 1; }
verify_version "$SOURCE_BIN" || {
    echo "prometheus-exec source version mismatch; expected: $EXPECTED_VERSION" >&2
    exit 1
}
BUILD_HASH="$(sha256_file "$SOURCE_BIN")"
if [ "$BUILD_HASH" != "$EXPECTED_BUILD_HASH" ]; then
    echo "prometheus-exec source hash mismatch; expected $EXPECTED_BUILD_HASH, got $BUILD_HASH" >&2
    exit 1
fi

mkdir -p "$BIN_DIR" "$MANIFEST_DIR" "$BACKUP_DIR"
chmod 700 "$MANIFEST_DIR" "$BACKUP_DIR"
STAGED="$(mktemp "${BIN_DIR}/.prometheus-exec.stage.XXXXXX")"
MANIFEST_STAGE="$(mktemp "${MANIFEST_DIR}/.prometheus-exec.manifest.XXXXXX")"
BACKUP_PATH=""
cleanup() {
    rm -f "$STAGED" "$MANIFEST_STAGE"
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

cat >"$MANIFEST_STAGE" <<EOF
{
  "schemaVersion": 1,
  "name": "prometheus-exec",
  "version": "$EXPECTED_VERSION",
  "expectedBuildSha256": "$EXPECTED_BUILD_HASH",
  "source": "$SOURCE_BIN",
  "destination": "$DESTINATION",
  "buildSha256": "$BUILD_HASH",
  "installedSha256": "$INSTALLED_HASH",
  "signature": "verified"
}
EOF
chmod 600 "$MANIFEST_STAGE"
mv -f "$MANIFEST_STAGE" "$MANIFEST_PATH"
MANIFEST_STAGE=""

echo "prometheus-exec installed and verified: $DESTINATION ($EXPECTED_VERSION, sha256:$INSTALLED_HASH)"
