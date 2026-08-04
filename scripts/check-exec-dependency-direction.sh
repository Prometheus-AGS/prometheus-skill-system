#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$repo_root" <<'PY'
from pathlib import Path
import sys
import tomllib

root = Path(sys.argv[1])
layers = {
    "exec-contracts": set(),
    "exec-core": {"prometheus-exec-contracts"},
    "exec-tier-p": {"prometheus-exec-contracts", "prometheus-exec-core"},
    "exec-service": {"prometheus-exec-contracts", "prometheus-exec-core"},
}
internal_prefix = "prometheus-exec-"
errors = []

for directory, allowed in layers.items():
    manifest_path = root / "substrate" / directory / "Cargo.toml"
    with manifest_path.open("rb") as stream:
        manifest = tomllib.load(stream)
    package = manifest["package"]
    if package["version"] != "1.7.0":
        errors.append(f"{directory}: expected version 1.7.0, got {package['version']}")
    dependencies = set(manifest.get("dependencies", {}))
    internal = {name for name in dependencies if name.startswith(internal_prefix)}
    forbidden = internal - allowed
    if forbidden:
        errors.append(f"{directory}: forbidden internal dependencies: {sorted(forbidden)}")

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(1)

print("exec_dependency_direction=PASS")
PY
