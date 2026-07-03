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

# install_bin <src> <dst> — copy a freshly built binary into place, then
# re-sign it ad-hoc on macOS. `cp` breaks the code signature of signed arm64
# binaries and the OS SIGKILLs them on first exec; `codesign --force --sign -`
# restores a valid ad-hoc signature. No-op on non-Darwin.
install_bin() {
    local src="$1" dst="$2"
    cp "${src}" "${dst}"
    if [ "$(uname -s)" = "Darwin" ]; then
        codesign --force --sign - "${dst}" >/dev/null 2>&1 || true
    fi
}

# ── 1. prometheus-cli ───────────────────────────────────────────────────────
info "Building prometheus-cli..."
(cd "${REPO_ROOT}/tools/prometheus-cli" && cargo build --release -p prometheus-cli 2>&1 | tail -3)
install_bin "${REPO_ROOT}/tools/prometheus-cli/target/release/prometheus" "${BIN_DIR}/prometheus"
ok "prometheus → ${BIN_DIR}/prometheus"

# ── 2. forge (forge-rs CLI) ──────────────────────────────────────────────────
# Upstream renamed the CLI package `forge` → `forge-cli` (still produces the `forge` binary).
if [ -d "${REPO_ROOT}/tools/forge-rs" ]; then
    info "Building forge..."
    (cd "${REPO_ROOT}/tools/forge-rs" && cargo build --release -p forge-cli 2>&1 | tail -3)
    install_bin "${REPO_ROOT}/tools/forge-rs/target/release/forge" "${BIN_DIR}/forge"
    ok "forge → ${BIN_DIR}/forge"
fi

# ── 3. pk + pk-cherry (prometheus-knowledge CLI + MCP server) ────────────────
# pk-cherry serves the knowledge MCP on :8942; pk is the CLI that the hooks
# (pk-focus-on-prompt.sh, pk-health.sh, Stop ingest) invoke. pk-mcp is a
# library, not a bin — do not try to build it as a binary target.
if [ -d "${REPO_ROOT}/tools/prometheus-knowledge" ]; then
    info "Building pk + pk-cherry..."
    (cd "${REPO_ROOT}/tools/prometheus-knowledge" && cargo build --release -p pk-cli -p pk-cherry 2>&1 | tail -3)
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk"        "${BIN_DIR}/pk"
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk-cherry" "${BIN_DIR}/pk-cherry"
    ok "pk        → ${BIN_DIR}/pk"
    ok "pk-cherry → ${BIN_DIR}/pk-cherry"
fi

# ── 4. liter-llm ─────────────────────────────────────────────────────────────
# Upstream renamed the CLI package to `liter-llm-cli` (still produces the `liter-llm` binary).
if [ -d "${REPO_ROOT}/tools/liter-llm" ]; then
    info "Building liter-llm..."
    (cd "${REPO_ROOT}/tools/liter-llm" && cargo build --release -p liter-llm-cli 2>&1 | tail -3)
    # liter-llm binary may be in workspace root target or crate target
    LLM_BIN=$(find "${REPO_ROOT}/tools/liter-llm/target/release" -maxdepth 1 -name "liter-llm" -type f 2>/dev/null | head -1)
    if [ -n "${LLM_BIN}" ]; then
        install_bin "${LLM_BIN}" "${BIN_DIR}/liter-llm"
        ok "liter-llm → ${BIN_DIR}/liter-llm"
    else
        fail "liter-llm binary not found after build"
    fi
fi

# ── 5. surreal-memory-server (memory MCP daemon on :23001) ───────────────────
# Installed to BOTH ~/.local/bin and /usr/local/bin: install-mcp-services.sh
# resolves the launchd binary via a PATH that lists /usr/local/bin first, so a
# stale copy there would shadow the fresh ~/.local/bin build.
# build.sh runs cargo clean + a server-dependent quality gate before building;
# call cargo directly with the same feature flags to avoid that gate.
if [ -d "${REPO_ROOT}/tools/surreal-memory-server" ]; then
    info "Building surreal-memory-server..."
    (cd "${REPO_ROOT}/tools/surreal-memory-server" && \
        RUSTFLAGS="-Dwarnings" cargo build --release --no-default-features \
            --features "embedded,metal,local-embeddings" 2>&1 | tail -3)
    SM_BIN="${REPO_ROOT}/tools/surreal-memory-server/target/release/surreal-memory-server"
    if [ -f "${SM_BIN}" ]; then
        install_bin "${SM_BIN}" "${BIN_DIR}/surreal-memory-server"
        ok "surreal-memory-server → ${BIN_DIR}/surreal-memory-server"
        if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
            install_bin "${SM_BIN}" "/usr/local/bin/surreal-memory-server"
            ok "surreal-memory-server → /usr/local/bin/surreal-memory-server"
        else
            info "skip /usr/local/bin/surreal-memory-server (not writable) — run: sudo cp ${SM_BIN} /usr/local/bin/ && sudo codesign --force --sign - /usr/local/bin/surreal-memory-server"
        fi
    else
        fail "surreal-memory-server binary not found after build"
    fi
fi

# ── 6. sycophancy-correction (S-01..S-08 MCP server + reflector gate) ─────────
# Must land on PATH — MCP configs invoke the bare `sycophancy-correction`
# command; /usr/local/bin is the canonical location the reflector gate and
# every tool's MCP config resolve.
SYCO_DIR="${REPO_ROOT}/skills/imported/sycophancy-correction"
if [ -d "${SYCO_DIR}" ]; then
    info "Building sycophancy-correction..."
    (cd "${SYCO_DIR}" && cargo build --release 2>&1 | tail -3)
    SYCO_BIN="${SYCO_DIR}/target/release/sycophancy-correction"
    if [ -f "${SYCO_BIN}" ]; then
        install_bin "${SYCO_BIN}" "${BIN_DIR}/sycophancy-correction"
        ok "sycophancy-correction → ${BIN_DIR}/sycophancy-correction"
        if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
            install_bin "${SYCO_BIN}" "/usr/local/bin/sycophancy-correction"
            ok "sycophancy-correction → /usr/local/bin/sycophancy-correction"
        else
            info "skip /usr/local/bin/sycophancy-correction (not writable) — run: sudo cp ${SYCO_BIN} /usr/local/bin/ && sudo codesign --force --sign - /usr/local/bin/sycophancy-correction"
        fi
    else
        fail "sycophancy-correction binary not found after build"
    fi
fi

# ── 7. template-forge + template-forge-mcp (artifact-refiner submodule) ──────
TEMPLATE_FORGE_DIR="${REPO_ROOT}/skills/imported/artifact-refiner/tools/template-forge-rs"
if [ -d "${TEMPLATE_FORGE_DIR}" ]; then
    info "Building template-forge and template-forge-mcp..."
    # rust-toolchain.toml in this submodule is comment-only — override via env
    (cd "${TEMPLATE_FORGE_DIR}" && RUSTUP_TOOLCHAIN=stable cargo build --release 2>&1 | tail -3)
    install_bin "${TEMPLATE_FORGE_DIR}/target/release/template-forge"     "${BIN_DIR}/template-forge"
    install_bin "${TEMPLATE_FORGE_DIR}/target/release/template-forge-mcp" "${BIN_DIR}/template-forge-mcp"
    ok "template-forge     → ${BIN_DIR}/template-forge"
    ok "template-forge-mcp → ${BIN_DIR}/template-forge-mcp"
else
    fail "template-forge-rs not found — run: git submodule update --init --recursive"
fi

echo ""
echo "✨ All binaries installed to ${BIN_DIR}"
echo "   Next: bash scripts/install-mcp-services.sh   # (re)install + start launchd/systemd daemons"
echo "   Then: prometheus setup --check               # verify full system health"
