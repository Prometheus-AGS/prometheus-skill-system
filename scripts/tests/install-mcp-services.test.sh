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

env = plist["EnvironmentVariables"]
actual = env.get("SURREAL_STARTUP_OPERATION_TIMEOUT")
expected = "30m"
if actual != expected:
    raise SystemExit(
        f"expected rendered SURREAL_STARTUP_OPERATION_TIMEOUT={expected}, got {actual!r}"
    )
if "SURREAL_ROCKSDB_BLOCK_CACHE_SIZE" in env:
    raise SystemExit("SurrealDB 4.0 has no RocksDB engine; drop SURREAL_ROCKSDB_BLOCK_CACHE_SIZE")
datastore = plist["ProgramArguments"][-1]
if not datastore.startswith("surrealkv:"):
    raise SystemExit(f"expected a surrealkv: datastore, got {datastore!r}")
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
