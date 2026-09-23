#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT INT TERM

bash "$REPO_ROOT/scripts/install-mcp-services.sh" \
  --render-only "$TMP_ROOT/rendered" >/dev/null

python3 - "$TMP_ROOT/rendered/ai.prometheus.surreal-memory-native.plist" <<'PY'
import plistlib
import sys
from pathlib import Path

path = Path(sys.argv[1])
with path.open("rb") as handle:
    plist = plistlib.load(handle)

actual = plist["EnvironmentVariables"].get("SURREAL_EXECUTOR_STARTUP_MS")
expected = "300000"
if actual != expected:
    raise SystemExit(
        f"expected rendered SURREAL_EXECUTOR_STARTUP_MS={expected}, got {actual!r}"
    )
PY

python3 - "$TMP_ROOT/rendered/ai.prometheus.surrealdb-native.plist" <<'PY'
import plistlib
import sys
from pathlib import Path

path = Path(sys.argv[1])
with path.open("rb") as handle:
    plist = plistlib.load(handle)

actual = plist["EnvironmentVariables"].get("SURREAL_ROCKSDB_BLOCK_CACHE_SIZE")
expected = "1073741824"
if actual != expected:
    raise SystemExit(
        f"expected rendered SURREAL_ROCKSDB_BLOCK_CACHE_SIZE={expected}, got {actual!r}"
    )
PY

reload_body="$(sed -n '/^reload_launch_agent()/,/^}/p' "$REPO_ROOT/scripts/install-mcp-services.sh")"
if grep -q 'kickstart -k' <<<"$reload_body"; then
    echo 'reload_launch_agent restarts a newly bootstrapped RunAtLoad job' >&2
    exit 1
fi
if ! grep -q 'kill -0.*previous_pid' <<<"$reload_body"; then
    echo 'reload_launch_agent does not wait for the previous process to exit' >&2
    exit 1
fi

echo 'PASS: managed memory services render bounded startup contracts'
