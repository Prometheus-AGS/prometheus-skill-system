#!/usr/bin/env bash
# install-binaries.sh — build and install all project binaries to ~/.local/bin/
# Run this after cloning or pulling to keep binaries in sync.
# Safe to re-run: only reinstalls if build succeeds.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "${BIN_DIR}"

info()  { echo "  → $*"; }
ok()    { echo "  ✅ $*"; }
fail()  { echo "  ❌ $*" >&2; }

# ── 1. prometheus-cli ───────────────────────────────────────────────────────
info "Building prometheus-cli..."
(cd "${REPO_ROOT}/tools/prometheus-cli" && cargo build --release -p prometheus-cli 2>&1 | tail -3)
cp "${REPO_ROOT}/tools/prometheus-cli/target/release/prometheus" "${BIN_DIR}/"
ok "prometheus → ${BIN_DIR}/prometheus"

# ── 2. forge (forge-rs CLI) ──────────────────────────────────────────────────
if [ -d "${REPO_ROOT}/tools/forge-rs" ]; then
    info "Building forge..."
    (cd "${REPO_ROOT}/tools/forge-rs" && cargo build --release -p forge 2>&1 | tail -3)
    cp "${REPO_ROOT}/tools/forge-rs/target/release/forge" "${BIN_DIR}/"
    ok "forge → ${BIN_DIR}/forge"
fi

# ── 3. pk-cherry (prometheus-knowledge MCP server) ───────────────────────────
if [ -d "${REPO_ROOT}/tools/prometheus-knowledge" ]; then
    info "Building pk-cherry..."
    (cd "${REPO_ROOT}/tools/prometheus-knowledge" && cargo build --release -p pk-cherry 2>&1 | tail -3)
    cp "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk-cherry" "${BIN_DIR}/"
    ok "pk-cherry → ${BIN_DIR}/pk-cherry"
fi

# ── 4. liter-llm ─────────────────────────────────────────────────────────────
if [ -d "${REPO_ROOT}/tools/liter-llm" ]; then
    info "Building liter-llm..."
    (cd "${REPO_ROOT}/tools/liter-llm" && cargo build --release 2>&1 | tail -3)
    # liter-llm binary may be in workspace root target or crate target
    LLM_BIN=$(find "${REPO_ROOT}/tools/liter-llm/target/release" -maxdepth 1 -name "liter-llm" -type f 2>/dev/null | head -1)
    if [ -n "${LLM_BIN}" ]; then
        cp "${LLM_BIN}" "${BIN_DIR}/"
        ok "liter-llm → ${BIN_DIR}/liter-llm"
    else
        fail "liter-llm binary not found after build"
    fi
fi

# ── 5. template-forge + template-forge-mcp (artifact-refiner submodule) ──────
TEMPLATE_FORGE_DIR="${REPO_ROOT}/skills/imported/artifact-refiner/tools/template-forge-rs"
if [ -d "${TEMPLATE_FORGE_DIR}" ]; then
    info "Building template-forge and template-forge-mcp..."
    # rust-toolchain.toml in this submodule is comment-only — override via env
    (cd "${TEMPLATE_FORGE_DIR}" && RUSTUP_TOOLCHAIN=stable cargo build --release 2>&1 | tail -3)
    cp "${TEMPLATE_FORGE_DIR}/target/release/template-forge"     "${BIN_DIR}/"
    cp "${TEMPLATE_FORGE_DIR}/target/release/template-forge-mcp" "${BIN_DIR}/"
    ok "template-forge     → ${BIN_DIR}/template-forge"
    ok "template-forge-mcp → ${BIN_DIR}/template-forge-mcp"
else
    fail "template-forge-rs not found — run: git submodule update --init --recursive"
fi

echo ""
echo "✨ All binaries installed to ${BIN_DIR}"
echo "   Run 'prometheus setup --check' to verify full system health."
