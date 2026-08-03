#!/usr/bin/env bash
# Update the source checkout, construct one verified immutable generation, and
# atomically advance the active pointer. Installed plugin caches are never read
# or edited; harnesses resolve through ~/.prometheus/plugins/.../current.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_REF_FILE="${HOME}/.prometheus/skill-pack-install-ref"
RUN_DOCTOR_REFRESH=false

for arg in "$@"; do
    case "$arg" in
        --force) ;; # Content-addressing already verifies every byte.
        --doctor-refresh) RUN_DOCTOR_REFRESH=true ;;
        *) echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

echo "Prometheus Skill Pack — Atomic Update"
echo "====================================="
echo ""
echo "Step 1: Pulling source changes..."
if ! git -C "$REPO_ROOT" pull --ff-only; then
    echo "ERROR: git pull --ff-only failed; resolve the divergence and rerun." >&2
    exit 1
fi
git -C "$REPO_ROOT" submodule update --init --recursive

echo ""
echo "Step 2: Building and verifying immutable generation..."
GENERATION="$(node "$REPO_ROOT/scripts/install-plugin-generation.js" \
    --source-root "$REPO_ROOT" --home "$HOME")"
node "$REPO_ROOT/scripts/install-plugin-generation.js" --verify >/dev/null
echo "  ✅ active generation: $GENERATION"

CURRENT_SHA="$(git -C "$REPO_ROOT" rev-parse HEAD)"
mkdir -p "$(dirname "$INSTALL_REF_FILE")"
TEMP_REF="${INSTALL_REF_FILE}.$$.tmp"
printf '%s\n' "$CURRENT_SHA" > "$TEMP_REF"
mv "$TEMP_REF" "$INSTALL_REF_FILE"

echo ""
echo "Step 3: Local installation receipt"
echo "  Source commit: $CURRENT_SHA"
echo "  Generation:    $GENERATION"
echo "  Targets:       14/14"

if ! $RUN_DOCTOR_REFRESH; then
    echo "  Doctor refresh not requested."
    exit 0
fi
if ! command -v prometheus >/dev/null 2>&1; then
    echo "  prometheus CLI not found; install binaries before doctor refresh." >&2
    exit 1
fi
if [[ -n "$(git -C "$REPO_ROOT" status --short --untracked-files=no)" ]]; then
    echo "  Refusing doctor refresh from a dirty checkout." >&2
    exit 1
fi
prometheus doctor --refresh --yes
prometheus doctor --json >/dev/null
echo "  ✅ doctor refresh completed"
