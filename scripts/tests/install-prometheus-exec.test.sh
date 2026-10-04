#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT INT TERM

make_binary() {
    local path="$1" version="$2"
    printf '#!/usr/bin/env bash\nprintf "%%s\\n" "%s"\n' "$version" >"$path"
    chmod +x "$path"
}

make_binary "$TMP_ROOT/good" "prometheus-exec 1.7.0"
make_binary "$TMP_ROOT/bad" "prometheus-exec 0.0.0"
make_binary "$TMP_ROOT/wrong-hash" "prometheus-exec 1.7.0"
printf '\n' >>"$TMP_ROOT/wrong-hash"
mkdir -p "$TMP_ROOT/bin" "$TMP_ROOT/manifests" "$TMP_ROOT/backups"
make_binary "$TMP_ROOT/bin/prometheus-exec" "prometheus-exec old"

cat >"$TMP_ROOT/codesign" <<'EOF'
#!/usr/bin/env bash
[ "${PROMETHEUS_EXEC_TEST_SIGN_FAIL:-0}" != "1" ]
EOF
chmod +x "$TMP_ROOT/codesign"

install_env=(
    PROMETHEUS_EXEC_BIN_DIR="$TMP_ROOT/bin"
    PROMETHEUS_EXEC_MANIFEST_DIR="$TMP_ROOT/manifests"
    PROMETHEUS_EXEC_BACKUP_DIR="$TMP_ROOT/backups"
    PROMETHEUS_EXEC_CODESIGN="$TMP_ROOT/codesign"
    PROMETHEUS_EXEC_PLATFORM=Darwin
)
good_hash="$(shasum -a 256 "$TMP_ROOT/good" | awk '{print $1}')"

env "${install_env[@]}" PROMETHEUS_EXEC_SOURCE_BIN="$TMP_ROOT/good" PROMETHEUS_EXEC_EXPECTED_SHA256="$good_hash" \
    bash "$REPO_ROOT/scripts/install-prometheus-exec.sh" >"$TMP_ROOT/success.out"
grep -Fq 'installed and verified' "$TMP_ROOT/success.out"
[ "$("$TMP_ROOT/bin/prometheus-exec" --version)" = "prometheus-exec 1.7.0" ]
grep -Fq '"signature": "verified"' "$TMP_ROOT/manifests/prometheus-exec.json"
installed_hash="$(shasum -a 256 "$TMP_ROOT/bin/prometheus-exec" | awk '{print $1}')"
grep -Fq "\"installedSha256\": \"$installed_hash\"" "$TMP_ROOT/manifests/prometheus-exec.json"

if env "${install_env[@]}" PROMETHEUS_EXEC_SOURCE_BIN="$TMP_ROOT/bad" PROMETHEUS_EXEC_EXPECTED_SHA256="$good_hash" \
    bash "$REPO_ROOT/scripts/install-prometheus-exec.sh" >"$TMP_ROOT/bad.out" 2>"$TMP_ROOT/bad.err"; then
    echo "FAIL: version mismatch returned success" >&2
    exit 1
fi
[ "$("$TMP_ROOT/bin/prometheus-exec" --version)" = "prometheus-exec 1.7.0" ]
if grep -Fq 'installed and verified' "$TMP_ROOT/bad.out"; then
    echo "FAIL: version mismatch printed false success" >&2
    exit 1
fi

if env "${install_env[@]}" PROMETHEUS_EXEC_SOURCE_BIN="$TMP_ROOT/wrong-hash" PROMETHEUS_EXEC_EXPECTED_SHA256="$good_hash" \
    bash "$REPO_ROOT/scripts/install-prometheus-exec.sh" >"$TMP_ROOT/hash.out" 2>"$TMP_ROOT/hash.err"; then
    echo "FAIL: certified hash mismatch returned success" >&2
    exit 1
fi
[ "$("$TMP_ROOT/bin/prometheus-exec" --version)" = "prometheus-exec 1.7.0" ]
grep -Fq 'source hash mismatch' "$TMP_ROOT/hash.err"
if grep -Fq 'installed and verified' "$TMP_ROOT/hash.out"; then
    echo "FAIL: certified hash mismatch printed false success" >&2
    exit 1
fi

if env "${install_env[@]}" PROMETHEUS_EXEC_SOURCE_BIN="$TMP_ROOT/good" PROMETHEUS_EXEC_EXPECTED_SHA256="$good_hash" \
    PROMETHEUS_EXEC_TEST_SIGN_FAIL=1 \
    bash "$REPO_ROOT/scripts/install-prometheus-exec.sh" >"$TMP_ROOT/sign.out" 2>"$TMP_ROOT/sign.err"; then
    echo "FAIL: signing failure returned success" >&2
    exit 1
fi
[ "$("$TMP_ROOT/bin/prometheus-exec" --version)" = "prometheus-exec 1.7.0" ]
if grep -Fq 'installed and verified' "$TMP_ROOT/sign.out"; then
    echo "FAIL: signing failure printed false success" >&2
    exit 1
fi

echo "PASS: prometheus-exec atomic install, version, signature, hash, rollback, and false-green contract"

# Real-build reproducibility: the certified hash must not depend on the
# checkout path. Before scripts/build-prometheus-exec.sh existed, the same
# commit hashed differently in every git worktree and the installer refused it.
# This builds for real (cargo, minutes on a cold cache), so it is opt-in:
#   PROMETHEUS_EXEC_TEST_REAL_BUILD=1 bash scripts/tests/install-prometheus-exec.test.sh
# Builds run strictly one after another (one cargo build at a time).
if [ "${PROMETHEUS_EXEC_TEST_REAL_BUILD:-0}" != "1" ]; then
    echo "SKIP: real-build reproducibility (set PROMETHEUS_EXEC_TEST_REAL_BUILD=1)"
    exit 0
fi

certified="$(awk -F'"' '/"expectedBuildSha256"/ { print $4; exit }' "$REPO_ROOT/config/prometheus-exec-binary.json")"

primary_hash="$(bash "$REPO_ROOT/scripts/build-prometheus-exec.sh" --print-hash 2>"$TMP_ROOT/primary-build.err")" || {
    tail -20 "$TMP_ROOT/primary-build.err" >&2
    echo "FAIL: primary checkout build failed" >&2
    exit 1
}

# A second checkout of the working tree at a different path and depth. mktemp
# lives under /var -> /private/var on macOS, so this also covers a symlinked
# checkout path. Copy the working tree (not HEAD) so uncommitted changes are
# what gets tested.
SECOND="$TMP_ROOT/elsewhere/a different depth/prometheus-skill-pack"
mkdir -p "$SECOND/crates" "$SECOND/substrate" "$SECOND/scripts" "$SECOND/config"
copy_tree() {
    (cd "$1" && tar -cf - --exclude ./target .) | (mkdir -p "$2" && cd "$2" && tar -xpf -)
}
copy_tree "$REPO_ROOT/crates/prometheus-exec" "$SECOND/crates/prometheus-exec"
for dep in "$REPO_ROOT"/substrate/exec-*/; do
    dep="${dep%/}"
    copy_tree "$dep" "$SECOND/substrate/$(basename "$dep")"
done
copy_tree "$REPO_ROOT/wit" "$SECOND/wit"
[ -d "$REPO_ROOT/.cargo" ] && copy_tree "$REPO_ROOT/.cargo" "$SECOND/.cargo"
cp "$REPO_ROOT/scripts/build-prometheus-exec.sh" "$REPO_ROOT/scripts/install-prometheus-exec.sh" "$SECOND/scripts/"
cp "$REPO_ROOT/config/prometheus-exec-binary.json" "$SECOND/config/"

# Drive the production entry point in the second checkout: it builds, then
# gates on the committed expectedBuildSha256.
mkdir -p "$TMP_ROOT/real/bin" "$TMP_ROOT/real/manifests" "$TMP_ROOT/real/backups"
if ! env PROMETHEUS_EXEC_BIN_DIR="$TMP_ROOT/real/bin" \
    PROMETHEUS_EXEC_MANIFEST_DIR="$TMP_ROOT/real/manifests" \
    PROMETHEUS_EXEC_BACKUP_DIR="$TMP_ROOT/real/backups" \
    PROMETHEUS_EXEC_CODESIGN="$TMP_ROOT/codesign" \
    PROMETHEUS_EXEC_PLATFORM=Darwin \
    bash "$SECOND/scripts/install-prometheus-exec.sh" >"$TMP_ROOT/second.out" 2>"$TMP_ROOT/second.err"; then
    tail -20 "$TMP_ROOT/second.err" >&2
    echo "FAIL: installer rejected a build from a second checkout path" >&2
    exit 1
fi
grep -Fq 'installed and verified' "$TMP_ROOT/second.out"
second_hash="$(awk -F'"' '/"buildSha256"/ { print $4; exit }' "$TMP_ROOT/real/manifests/prometheus-exec.json")"

if [ "$primary_hash" != "$second_hash" ]; then
    echo "FAIL: build hash depends on checkout path: $primary_hash (primary) vs $second_hash (second)" >&2
    exit 1
fi
if [ "$primary_hash" != "$certified" ]; then
    echo "FAIL: reproducible build $primary_hash does not match certified expectedBuildSha256 $certified" >&2
    exit 1
fi
if strings -a "$SECOND/crates/prometheus-exec/target/release/prometheus-exec" | grep -Fq "a different depth"; then
    echo "FAIL: second-checkout binary embeds its checkout path" >&2
    exit 1
fi

echo "PASS: prometheus-exec builds byte-identically from two checkout paths and matches the certified hash ($certified)"
