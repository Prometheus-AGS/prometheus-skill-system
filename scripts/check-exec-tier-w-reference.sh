#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
component_dir="$repo_root/skills/react/prometheus-entity-skills/entity-graph-optimize/component"
artifact="$component_dir/../skill.wasm"
versions="$repo_root/substrate/exec-tier-w/versions.toml"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-exec-tier-w-reference.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

python3 - "$repo_root" "$versions" <<'PY'
from pathlib import Path
import hashlib
import json
import sys
import tomllib

root = Path(sys.argv[1])
with open(sys.argv[2], "rb") as stream:
    versions = tomllib.load(stream)
reference = versions["reference_component"]
errors = []

if reference["world"] != "prometheus:component@0.1.0":
    errors.append("reference component world is not pinned to 0.1.0")

for name in ("types.wit", "capabilities.wit", "skill.wit"):
    canonical = (root / "wit/prometheus-component" / name).read_bytes()
    guest = (
        root
        / "skills/react/prometheus-entity-skills/entity-graph-optimize/component/wit"
        / name
    ).read_bytes()
    if guest != canonical:
        errors.append(f"reference guest {name} differs from canonical WIT")

artifact = root / "skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
if actual != reference["artifact_sha256"]:
    errors.append(f"checked artifact hash differs: expected {reference['artifact_sha256']}, got {actual}")

hash_fixture = (
    root / "substrate/exec-tier-w/fixtures/reference-component.sha256"
).read_text().strip()
if hash_fixture != actual:
    errors.append("reference-component.sha256 differs from checked artifact")

run_fixture = json.loads(
    (root / "substrate/exec-tier-w/fixtures/reference-run-absent.json").read_text()
)
if run_fixture["component_sha256"] != actual:
    errors.append("reference run fixture does not bind the checked artifact")
if run_fixture["world"] != reference["world"]:
    errors.append("reference run fixture world differs from versions.toml")

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(1)
PY

expected_version="$(python3 - "$versions" <<'PY'
import sys, tomllib
with open(sys.argv[1], "rb") as stream:
    print(tomllib.load(stream)["reference_component"]["cargo_component"])
PY
)"
actual_version="$(cargo component --version | awk '{print $NF}')"
if [[ "$actual_version" != "$expected_version" ]]; then
  echo "cargo-component version mismatch: expected $expected_version, got $actual_version" >&2
  exit 1
fi

export SOURCE_DATE_EPOCH=0
for build in a b; do
  CARGO_TARGET_DIR="$scratch/$build" \
    cargo component build \
      --manifest-path "$component_dir/Cargo.toml" \
      --release \
      --target wasm32-wasip2 \
      --locked >/dev/null
done

built_a="$scratch/a/wasm32-wasip2/release/entity_graph_optimize_skill.wasm"
built_b="$scratch/b/wasm32-wasip2/release/entity_graph_optimize_skill.wasm"
cmp "$built_a" "$built_b"
cmp "$built_a" "$artifact"

echo "exec_tier_w_reference=PASS"
