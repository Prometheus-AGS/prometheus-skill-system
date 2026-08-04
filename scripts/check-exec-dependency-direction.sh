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
    "exec-tier-w": {"prometheus-exec-contracts", "prometheus-exec-core"},
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

tier_w_path = root / "substrate" / "exec-tier-w" / "Cargo.toml"
with tier_w_path.open("rb") as stream:
    tier_w = tomllib.load(stream)
wasmtime = tier_w.get("dependencies", {}).get("wasmtime", {})
if wasmtime.get("version") != "=46.0.0":
    errors.append("exec-tier-w: Wasmtime must be pinned exactly to 46.0.0")
if set(tier_w.get("features", {}).get("mobile", [])) != {"pulley"}:
    errors.append("exec-tier-w: mobile feature must select Pulley")

versions_path = root / "substrate" / "exec-tier-w" / "versions.toml"
with versions_path.open("rb") as stream:
    versions = tomllib.load(stream)
if versions.get("component_world") != "prometheus:component@0.1.0":
    errors.append("exec-tier-w: component world version is not pinned")
if versions.get("wasmtime") != "46.0.0":
    errors.append("exec-tier-w: versions.toml disagrees with Cargo.toml")
if versions.get("profiles", {}).get("mobile", {}).get("backend") != "pulley":
    errors.append("exec-tier-w: mobile backend must be Pulley")

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(1)

print("exec_dependency_direction=PASS")
PY
