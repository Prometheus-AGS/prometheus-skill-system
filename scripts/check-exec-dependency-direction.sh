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
    forbidden_estate_dependencies = {
        name for name in dependencies if "kbd" in name or "sovereign" in name
    }
    if forbidden_estate_dependencies:
        errors.append(
            f"{directory}: execution layer imports estate orchestration: "
            f"{sorted(forbidden_estate_dependencies)}"
        )
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
if set(tier_w.get("features", {}).get("standalone", [])) != {"cranelift"}:
    errors.append("exec-tier-w: standalone feature must select Cranelift")
if set(tier_w.get("features", {}).get("bundled-mobile", [])) != {"mobile"}:
    errors.append("exec-tier-w: bundled-mobile must select only the mobile profile")
estate_dependencies = {"dep:base64", "dep:ed25519-dalek", "dep:serde_json"}
if set(tier_w.get("features", {}).get("estate", [])) != estate_dependencies:
    errors.append("exec-tier-w: estate trust dependencies must remain feature-gated")
for dependency in ("base64", "ed25519-dalek", "serde_json"):
    definition = tier_w.get("dependencies", {}).get(dependency, {})
    if not isinstance(definition, dict) or definition.get("optional") is not True:
        errors.append(f"exec-tier-w: {dependency} must remain optional and estate-only")

versions_path = root / "substrate" / "exec-tier-w" / "versions.toml"
with versions_path.open("rb") as stream:
    versions = tomllib.load(stream)
if versions.get("component_world") != "prometheus:component@0.1.0":
    errors.append("exec-tier-w: component world version is not pinned")
if versions.get("wasmtime") != "46.0.0":
    errors.append("exec-tier-w: versions.toml disagrees with Cargo.toml")
if versions.get("profiles", {}).get("mobile", {}).get("backend") != "pulley":
    errors.append("exec-tier-w: mobile backend must be Pulley")
if versions.get("profiles", {}).get("mobile", {}).get("trust") != "bundled-hash-pins":
    errors.append("exec-tier-w: mobile trust must use bundled hash pins")
if versions.get("profiles", {}).get("standalone", {}).get("trust") != "explicit-hash-pins":
    errors.append("exec-tier-w: standalone trust must use explicit hash pins")

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(1)

print("exec_dependency_direction=PASS")
PY
