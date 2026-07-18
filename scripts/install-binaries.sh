#!/usr/bin/env bash
# install-binaries.sh — build and install all project binaries to ~/.local/bin/
# Run this after cloning or pulling to keep binaries in sync.
# Safe to re-run: only reinstalls if build succeeds.
#
# Usage:
#   bash scripts/install-binaries.sh
#   bash scripts/install-binaries.sh --dry-run

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "${BIN_DIR}"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --help|-h)
            sed -n '1,12p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

info()  { echo "  → $*"; }
ok()    { echo "  ✅ $*"; }
fail()  { echo "  ❌ $*" >&2; }

# install_bin <src> <dst> — copy a freshly built binary into place, then
# re-sign it ad-hoc on macOS. `cp` breaks the code signature of signed arm64
# binaries and the OS SIGKILLs them on first exec; `codesign --force --sign -`
# restores a valid ad-hoc signature. No-op on non-Darwin.
install_bin() {
    local src="$1" dst="$2"
    if $DRY_RUN; then
        info "[dry-run] would install ${src} -> ${dst}"
        return 0
    fi
    # -f: if dst exists and can't be opened for write (e.g. owned by another
    # user, as happens when a prior install ran under sudo), unlink and retry
    # instead of failing outright.
    cp -f "${src}" "${dst}"
    if [ "$(uname -s)" = "Darwin" ]; then
        codesign --force --sign - "${dst}" >/dev/null 2>&1 || true
    fi
}

# ── 1. prometheus-cli ───────────────────────────────────────────────────────
if [ -f "${REPO_ROOT}/tools/prometheus-cli/Cargo.toml" ]; then
    info "Building prometheus-cli..."
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/prometheus-cli" && cargo build --release -p prometheus-cli 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release -p prometheus-cli"
    fi
    install_bin "${REPO_ROOT}/tools/prometheus-cli/target/release/prometheus" "${BIN_DIR}/prometheus"
    ok "prometheus → ${BIN_DIR}/prometheus"
else
    info "skip prometheus-cli (submodule not initialized)"
fi

# ── 2. forge (forge-rs CLI) ──────────────────────────────────────────────────
# Upstream renamed the CLI package `forge` → `forge-cli` (still produces the `forge` binary).
if [ -f "${REPO_ROOT}/tools/forge-rs/Cargo.toml" ]; then
    info "Building forge..."
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/forge-rs" && cargo build --release -p forge-cli 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release -p forge-cli"
    fi
    install_bin "${REPO_ROOT}/tools/forge-rs/target/release/forge" "${BIN_DIR}/forge"
    ok "forge → ${BIN_DIR}/forge"
else
    info "skip forge-rs (submodule not initialized)"
fi

# ── 3. pk + pk-cherry (prometheus-knowledge CLI + MCP server) ────────────────
# pk-cherry serves the knowledge MCP on :8942; pk is the CLI that the hooks
# (pk-focus-on-prompt.sh, pk-health.sh, Stop ingest) invoke. pk-mcp is a
# library, not a bin — do not try to build it as a binary target.
if [ -f "${REPO_ROOT}/tools/prometheus-knowledge/Cargo.toml" ]; then
    info "Building pk + pk-cherry..."
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/prometheus-knowledge" && cargo build --release -p pk-cli -p pk-cherry 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release -p pk-cli -p pk-cherry"
    fi
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk"        "${BIN_DIR}/pk"
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk-cherry" "${BIN_DIR}/pk-cherry"
    ok "pk        → ${BIN_DIR}/pk"
    ok "pk-cherry → ${BIN_DIR}/pk-cherry"
else
    info "skip prometheus-knowledge (submodule not initialized)"
fi

# ── 4. liter-llm ─────────────────────────────────────────────────────────────
# Upstream renamed the CLI package to `liter-llm-cli` (still produces the `liter-llm` binary).
if [ -f "${REPO_ROOT}/tools/liter-llm/Cargo.toml" ]; then
    info "Building liter-llm..."
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/liter-llm" && cargo build --release -p liter-llm-cli 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release -p liter-llm-cli"
    fi
    # liter-llm binary may be in workspace root target or crate target
    LLM_BIN=$(find "${REPO_ROOT}/tools/liter-llm/target/release" -maxdepth 1 -name "liter-llm" -type f 2>/dev/null | head -1)
    if [ -n "${LLM_BIN}" ]; then
        install_bin "${LLM_BIN}" "${BIN_DIR}/liter-llm"
        ok "liter-llm → ${BIN_DIR}/liter-llm"
    else
        fail "liter-llm binary not found after build"
    fi
    # `liter-llm mcp` REQUIRES a config file — without it the stdio server exits
    # and Claude Code fails to load it. Install the default proxy/MCP config
    # (routes via the local :8181 openai-proxy) unless the user already has one.
    LLM_CFG_DIR="${HOME}/.config/liter-llm"
    LLM_CFG="${LLM_CFG_DIR}/liter-llm-proxy.toml"
    if [ ! -f "${LLM_CFG}" ] && [ -f "${REPO_ROOT}/shared/config/liter-llm-proxy.toml" ]; then
        mkdir -p "${LLM_CFG_DIR}"
        if $DRY_RUN; then
            info "[dry-run] would install liter-llm config at ${LLM_CFG}"
        else
            cp "${REPO_ROOT}/shared/config/liter-llm-proxy.toml" "${LLM_CFG}"
        fi
        ok "liter-llm config → ${LLM_CFG}"
    fi
else
    info "skip liter-llm (submodule not initialized)"
fi

# ── 5. surreal-memory-server (memory MCP daemon on :23001) ───────────────────
# Installed to BOTH ~/.local/bin and /usr/local/bin: install-mcp-services.sh
# resolves the launchd binary via a PATH that lists /usr/local/bin first, so a
# stale copy there would shadow the fresh ~/.local/bin build.
# build.sh runs cargo clean + a server-dependent quality gate before building;
# call cargo directly with the same feature flags to avoid that gate.
# GPU feature is platform-dependent: `metal` is Apple-only (pulls in objc2,
# which hard-fails compile_error! on any non-Apple target); `cuda` requires an
# NVIDIA GPU. Select the right accelerator instead of hardcoding `metal`.
if [ -f "${REPO_ROOT}/tools/surreal-memory-server/Cargo.toml" ]; then
    info "Building surreal-memory-server..."
    SM_FEATURES="embedded,local-embeddings"
    if [ "$(uname -s)" = "Darwin" ]; then
        SM_FEATURES="embedded,metal,local-embeddings"
    elif command -v nvidia-smi >/dev/null 2>&1; then
        SM_FEATURES="embedded,cuda,local-embeddings"
    fi
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/surreal-memory-server" && \
            RUSTFLAGS="-Dwarnings" cargo build --release --no-default-features \
                --features "${SM_FEATURES}" 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build for surreal-memory-server with features ${SM_FEATURES}"
    fi
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
else
    info "skip surreal-memory-server (submodule not initialized)"
fi

# ── 6. sycophancy-correction (S-01..S-08 MCP server + reflector gate) ─────────
# Must land on PATH — MCP configs invoke the bare `sycophancy-correction`
# command; /usr/local/bin is the canonical location the reflector gate and
# every tool's MCP config resolve.
SYCO_DIR="${REPO_ROOT}/skills/imported/sycophancy-correction"
if [ -f "${SYCO_DIR}/Cargo.toml" ]; then
    info "Building sycophancy-correction..."
    if ! $DRY_RUN; then
        (cd "${SYCO_DIR}" && cargo build --release 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release for sycophancy-correction"
    fi
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
else
    info "skip sycophancy-correction (submodule not initialized)"
fi

# ── 7. template-forge + template-forge-mcp (artifact-refiner submodule) ──────
TEMPLATE_FORGE_DIR="${REPO_ROOT}/skills/imported/artifact-refiner/tools/template-forge-rs"
if [ -f "${TEMPLATE_FORGE_DIR}/Cargo.toml" ]; then
    info "Building template-forge and template-forge-mcp..."
    # rust-toolchain.toml in this submodule is comment-only — override via env
    if ! $DRY_RUN; then
        (cd "${TEMPLATE_FORGE_DIR}" && RUSTUP_TOOLCHAIN=stable cargo build --release 2>&1 | tail -3)
    else
        info "[dry-run] would run RUSTUP_TOOLCHAIN=stable cargo build --release for template-forge-rs"
    fi
    install_bin "${TEMPLATE_FORGE_DIR}/target/release/template-forge"     "${BIN_DIR}/template-forge"
    install_bin "${TEMPLATE_FORGE_DIR}/target/release/template-forge-mcp" "${BIN_DIR}/template-forge-mcp"
    ok "template-forge     → ${BIN_DIR}/template-forge"
    ok "template-forge-mcp → ${BIN_DIR}/template-forge-mcp"
else
    fail "template-forge-rs not found — run: git submodule update --init --recursive"
fi

# ── 8. cowork + co (cowork-skills CLI — skill management utility) ────────────
# Two-path install:
#   Path A (preferred): source build from tools/cowork-skills submodule
#   Path B (fallback):  download pre-built binary from GitHub Releases
install_cowork() {
    local cowork_dir="${REPO_ROOT}/tools/cowork-skills"
    local cli_dir="${cowork_dir}/cli"

    if [ -d "${cli_dir}" ]; then
        # Path A — source build
        info "Building cowork from source (tools/cowork-skills)..."
        if ! $DRY_RUN; then
            (cd "${cli_dir}" && cargo build --release 2>&1 | tail -3)
        else
            info "[dry-run] would run cargo build --release in ${cli_dir}"
        fi
        install_bin "${cli_dir}/target/release/cowork" "${BIN_DIR}/cowork"
        install_bin "${cli_dir}/target/release/co"     "${BIN_DIR}/co"
        ok "cowork → ${BIN_DIR}/cowork"
        ok "co     → ${BIN_DIR}/co"
        return
    fi

    # Path B — download from GitHub Releases
    info "cowork-skills submodule not present; downloading from GitHub Releases..."
    local os arch target archive_ext
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}-${arch}" in
        Darwin-arm64)   target="aarch64-apple-darwin";        archive_ext="tar.gz" ;;
        Darwin-x86_64)  target="x86_64-apple-darwin";         archive_ext="tar.gz" ;;
        Linux-x86_64)   target="x86_64-unknown-linux-musl";   archive_ext="tar.gz" ;;
        MINGW*|MSYS*|Windows*) target="x86_64-pc-windows-msvc"; archive_ext="zip" ;;
        *)
            fail "Unsupported platform ${os}-${arch} for cowork download; run: cd tools/cowork-skills/cli && cargo build --release"
            return
            ;;
    esac

    # Resolve latest release tag via GitHub API redirect
    local latest_url="https://github.com/GQAdonis/cowork-skills/releases/latest/download"
    local version
    version="$(curl -fsL -o /dev/null -w '%{url_effective}' \
        "https://github.com/GQAdonis/cowork-skills/releases/latest" \
        | sed 's|.*/tag/v||')" || version="latest"

    local archive="cowork-${version}-${target}.${archive_ext}"
    local download_url="${latest_url}/${archive}"
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    info "Downloading ${archive} from GitHub Releases..."
    if ! curl -fsL "${download_url}" -o "${tmp_dir}/${archive}"; then
        fail "Failed to download ${download_url}; run: git submodule update --init tools/cowork-skills"
        rm -rf "${tmp_dir}"
        return
    fi

    # Extract
    if [ "${archive_ext}" = "tar.gz" ]; then
        tar -C "${tmp_dir}" -xzf "${tmp_dir}/${archive}"
        local bin_dir="${tmp_dir}/cowork-${version}-${target}"
        install_bin "${bin_dir}/cowork" "${BIN_DIR}/cowork"
        # Create co symlink/copy
        cp "${BIN_DIR}/cowork" "${BIN_DIR}/co"
        if [ "$(uname -s)" = "Darwin" ]; then
            codesign --force --sign - "${BIN_DIR}/co" >/dev/null 2>&1 || true
        fi
    else
        fail "zip extraction on Windows: run scripts/install-binaries.ps1 instead"
        rm -rf "${tmp_dir}"
        return
    fi

    rm -rf "${tmp_dir}"
    ok "cowork → ${BIN_DIR}/cowork (downloaded)"
    ok "co     → ${BIN_DIR}/co (downloaded)"
}

install_cowork

# ── 9. dsg (disk-space-guardian — safe build cache cleanup) ──────────────────
# Two-path install:
#   Path A (preferred): source build from tools/disk-space-guardian submodule
#   Path B (fallback):  download pre-built binary from GitHub Releases
install_dsg() {
    local dsg_dir="${REPO_ROOT}/tools/disk-space-guardian"

    if [ -f "${dsg_dir}/Cargo.toml" ]; then
        # Path A — source build
        info "Building dsg from source (tools/disk-space-guardian)..."
        (cd "${dsg_dir}" && cargo build --release 2>&1 | tail -3)
        local dsg_bin
        dsg_bin="$(find "${dsg_dir}/target/release" -maxdepth 1 -name "dsg" -type f 2>/dev/null | head -1)"
        if [ -n "${dsg_bin}" ]; then
            install_bin "${dsg_bin}" "${BIN_DIR}/dsg"
            ok "dsg → ${BIN_DIR}/dsg"
            return
        else
            fail "dsg binary not found after source build; falling through to download"
        fi
    fi

    # Path B — download from GitHub Releases
    info "dsg source not present or build failed; downloading from GitHub Releases..."
    local os arch target archive_ext
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}-${arch}" in
        Darwin-arm64)   target="aarch64-apple-darwin";        archive_ext="tar.gz" ;;
        Darwin-x86_64)  target="x86_64-apple-darwin";         archive_ext="tar.gz" ;;
        Linux-x86_64)   target="x86_64-unknown-linux-musl";   archive_ext="tar.gz" ;;
        MINGW*|MSYS*|Windows*) target="x86_64-pc-windows-msvc"; archive_ext="zip" ;;
        *)
            fail "Unsupported platform ${os}-${arch} for dsg download; run: cd tools/disk-space-guardian && cargo build --release"
            return
            ;;
    esac

    local version
    version="$(curl -fsL -o /dev/null -w '%{url_effective}' \
        "https://github.com/GQAdonis/disk-space-guardian/releases/latest" \
        | sed 's|.*/tag/v||')" || version="latest"

    local archive="dsg-${version}-${target}.${archive_ext}"
    local download_url="https://github.com/GQAdonis/disk-space-guardian/releases/latest/download/${archive}"
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    info "Downloading ${archive} from GitHub Releases..."
    if ! curl -fsL "${download_url}" -o "${tmp_dir}/${archive}"; then
        fail "Failed to download ${download_url}; run: git submodule update --init tools/disk-space-guardian && cd tools/disk-space-guardian && cargo build --release"
        rm -rf "${tmp_dir}"
        return
    fi

    if [ "${archive_ext}" = "tar.gz" ]; then
        tar -C "${tmp_dir}" -xzf "${tmp_dir}/${archive}"
        local extracted_bin
        extracted_bin="$(find "${tmp_dir}" -name "dsg" -type f | head -1)"
        if [ -n "${extracted_bin}" ]; then
            install_bin "${extracted_bin}" "${BIN_DIR}/dsg"
            ok "dsg → ${BIN_DIR}/dsg (downloaded)"
        else
            fail "dsg binary not found in archive"
        fi
    else
        fail "zip extraction on Windows: run scripts/install-binaries.ps1 instead"
    fi

    rm -rf "${tmp_dir}"
}

install_dsg

# ── 10. prometheus-research (HTTP + MCP research server on :7891) ────────────
if [ -f "${REPO_ROOT}/substrate/prometheus-research/Cargo.toml" ]; then
    info "Building prometheus-research..."
    (cd "${REPO_ROOT}/substrate/prometheus-research" && cargo build --release 2>&1 | tail -3)
    PR_BIN="${REPO_ROOT}/substrate/prometheus-research/target/release/prometheus-research"
    if [ -f "${PR_BIN}" ]; then
        install_bin "${PR_BIN}" "${BIN_DIR}/prometheus-research"
        ok "prometheus-research → ${BIN_DIR}/prometheus-research"

        STATIC_SRC="${REPO_ROOT}/substrate/prometheus-research/src/static"
        STATIC_DST="${REPO_ROOT}/docs/deep-research/static"
        if [ -d "${STATIC_SRC}" ]; then
            mkdir -p "${STATIC_DST}"
            cp -r "${STATIC_SRC}/." "${STATIC_DST}/"
            ok "prometheus-research static assets → docs/deep-research/static/"
        fi

        if [ "$(uname -s)" = "Darwin" ]; then
            PLIST_SRC="${REPO_ROOT}/substrate/prometheus-research/com.prometheus.research.plist"
            PLIST_DST="${HOME}/Library/LaunchAgents/com.prometheus.research.plist"
            mkdir -p "${HOME}/Library/LaunchAgents"
            sed "s|__HOME__|${HOME}|g" "${PLIST_SRC}" > "${PLIST_DST}"
            launchctl bootout "gui/$(id -u)" "${PLIST_DST}" 2>/dev/null || true
            launchctl bootstrap "gui/$(id -u)" "${PLIST_DST}"
            ok "prometheus-research launchd service registered"
        else
            info "skip launchd (not macOS) — start manually: prometheus-research --mode mcp"
        fi
    else
        fail "prometheus-research binary not found after build"
    fi
else
    info "skip prometheus-research (substrate/prometheus-research not found)"
fi

echo ""
echo "✨ All binaries installed to ${BIN_DIR}"
echo "   Next: bash scripts/install-mcp-services.sh   # (re)install + start launchd/systemd daemons"
echo "   Then: prometheus setup --check               # verify full system health"
