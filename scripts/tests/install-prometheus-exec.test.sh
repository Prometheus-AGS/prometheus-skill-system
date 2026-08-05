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
