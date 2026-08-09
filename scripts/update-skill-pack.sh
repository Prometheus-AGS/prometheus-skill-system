#!/usr/bin/env bash
# Update a clean source checkout, construct one verified immutable generation,
# refresh installed native plugin surfaces, and only then advance the receipt.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_REF_FILE="${HOME}/.prometheus/skill-pack-install-ref"
PLUGIN_ROOT="${HOME}/.prometheus/plugins/prometheus-skill-pack"
RUN_DOCTOR_REFRESH=false
FORCE_REFRESH=false

for arg in "$@"; do
    case "$arg" in
        --force) FORCE_REFRESH=true ;;
        --doctor-refresh) RUN_DOCTOR_REFRESH=true ;;
        *) echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

require_clean_source() {
    local stage="$1"
    local status
    status="$(git -C "$REPO_ROOT" status --porcelain)"
    if [[ -n "$status" ]]; then
        echo "ERROR: refusing update from a dirty source tree ($stage): $REPO_ROOT" >&2
        echo "$status" >&2
        return 1
    fi
}

echo "Prometheus Skill Pack — Verified Update"
echo "========================================"
echo ""
echo "Step 1: Verifying and updating the source checkout..."
require_clean_source "before pull"
if ! git -C "$REPO_ROOT" pull --ff-only; then
    echo "ERROR: git pull --ff-only failed; resolve the divergence and rerun." >&2
    exit 1
fi
git -C "$REPO_ROOT" submodule update --init --recursive
require_clean_source "after pull and submodule update"
CURRENT_SHA="$(git -C "$REPO_ROOT" rev-parse HEAD)"
echo "  ✅ clean source commit: $CURRENT_SHA"

echo ""
echo "Step 2: Verifying generated distribution artifacts..."
node "$REPO_ROOT/scripts/generate-harness-adapters.js" --check
node "$REPO_ROOT/scripts/build-codex-plugin.js" --check
echo "  ✅ generated harness and Codex artifacts are current"

echo ""
echo "Step 3: Building and verifying the immutable generation..."
GENERATION="$(node "$REPO_ROOT/scripts/install-plugin-generation.js" \
    --source-root "$REPO_ROOT" \
    --home "$HOME" \
    --plugin-root "$PLUGIN_ROOT" \
    --require-clean-source \
    --expected-source-commit "$CURRENT_SHA")"
VERIFIED_GENERATION="$(node "$REPO_ROOT/scripts/install-plugin-generation.js" \
    --home "$HOME" --plugin-root "$PLUGIN_ROOT" --verify)"
if [[ "$VERIFIED_GENERATION" != "$GENERATION" ]]; then
    echo "ERROR: installed generation changed during verification: expected $GENERATION, found $VERIFIED_GENERATION" >&2
    exit 1
fi
node - "$PLUGIN_ROOT/current/manifest.json" "$CURRENT_SHA" "$GENERATION" <<'NODE'
const fs = require('fs');
const [manifestPath, expectedCommit, expectedGeneration] = process.argv.slice(2);
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
if (manifest.generation !== expectedGeneration) {
  throw new Error(`manifest generation mismatch: ${manifest.generation}`);
}
if (manifest.sourceProvenance?.sourceCommit !== expectedCommit) {
  throw new Error(`manifest source commit mismatch: ${manifest.sourceProvenance?.sourceCommit}`);
}
if (manifest.sourceProvenance?.sourceTreeState !== 'clean') {
  throw new Error(`manifest source tree is not clean: ${manifest.sourceProvenance?.sourceTreeState}`);
}
if (!Array.isArray(manifest.targetPayloads) || manifest.targetPayloads.length !== 14) {
  throw new Error(`manifest target matrix is incomplete: ${manifest.targetPayloads?.length}`);
}
NODE
echo "  ✅ active generation: $GENERATION"

echo ""
echo "Step 4: Refreshing installed native plugin surfaces..."
refresh_args=(--source-root "$REPO_ROOT" --generation "$GENERATION")
if $FORCE_REFRESH; then
    refresh_args+=(--force)
fi
bash "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh" "${refresh_args[@]}"
require_clean_source "after native refresh"

echo ""
echo "Step 5: Writing the local installation receipt..."
mkdir -p "$(dirname "$INSTALL_REF_FILE")"
TEMP_REF="${INSTALL_REF_FILE}.$$.tmp"
printf '%s\n' "$CURRENT_SHA" > "$TEMP_REF"
mv "$TEMP_REF" "$INSTALL_REF_FILE"
echo "  Source commit: $CURRENT_SHA"
echo "  Generation:    $GENERATION"
echo "  Targets:       14/14"
echo "  ✅ receipt advanced after all detected surfaces verified"

if ! $RUN_DOCTOR_REFRESH; then
    echo "  Doctor refresh not requested."
    exit 0
fi

echo ""
echo "Step 6: Refreshing local doctor evidence..."
if ! command -v prometheus >/dev/null 2>&1; then
    echo "ERROR: prometheus CLI not found; install binaries before doctor refresh." >&2
    exit 1
fi
require_clean_source "before doctor refresh"
prometheus doctor --refresh --yes
prometheus doctor --json >/dev/null
echo "  ✅ doctor refresh completed"
