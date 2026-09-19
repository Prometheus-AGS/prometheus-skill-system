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

echo 'PASS: managed memory service allows five minutes for cold MLX startup'
