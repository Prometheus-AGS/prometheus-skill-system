#!/usr/bin/env bash
# install-binaries.sh — build and install all project binaries to ~/.local/bin/
# Run this after cloning or pulling to keep binaries in sync.
# Safe to re-run: only reinstalls if build succeeds.
#
# Usage:
#   bash scripts/install-binaries.sh
#   bash scripts/install-binaries.sh --dry-run
#   bash scripts/install-binaries.sh --sharing   # also build the optional sync daemon

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "${BIN_DIR}"
DRY_RUN=false
SHARING=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --sharing) SHARING=true; shift ;;
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

# prometheus-exec has a stricter evidence-producing installation contract than
# the legacy installers below: version/hash/signature readback is mandatory and
# replacement is atomic with rollback evidence.
if $DRY_RUN; then
    bash "${REPO_ROOT}/scripts/install-prometheus-exec.sh" --dry-run
else
    bash "${REPO_ROOT}/scripts/install-prometheus-exec.sh"
fi

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

# ── 3. pk + pk-cherry + learning worker ─────────────────────────────────────
# pk-cherry serves the knowledge MCP on :8942; pk is the CLI that the hooks
# (bounded context dispatch, pk-health.sh, and the learning worker) invoke. pk-mcp is a
# library, not a bin — do not try to build it as a binary target.
if [ -f "${REPO_ROOT}/tools/prometheus-knowledge/Cargo.toml" ]; then
    info "Building pk + pk-cherry + prometheus-learning-worker..."
    if ! $DRY_RUN; then
        (cd "${REPO_ROOT}/tools/prometheus-knowledge" && cargo build --release -p pk-cli -p pk-cherry -p pk-learning-worker 2>&1 | tail -3)
    else
        info "[dry-run] would run cargo build --release -p pk-cli -p pk-cherry -p pk-learning-worker"
    fi
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk"        "${BIN_DIR}/pk"
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/pk-cherry" "${BIN_DIR}/pk-cherry"
    install_bin "${REPO_ROOT}/tools/prometheus-knowledge/target/release/prometheus-learning-worker" "${BIN_DIR}/prometheus-learning-worker"
    ok "pk        → ${BIN_DIR}/pk"
    ok "pk-cherry → ${BIN_DIR}/pk-cherry"
    ok "prometheus-learning-worker → ${BIN_DIR}/prometheus-learning-worker"
else
    info "skip prometheus-knowledge (submodule not initialized)"
fi

# ── 4. Learning substrate binaries ───────────────────────────────────────────
substrate_bins=(learner-model surface-bridge)
$SHARING && substrate_bins+=(sovereign-sync)
for substrate_bin in "${substrate_bins[@]}"; do
    substrate_manifest="${REPO_ROOT}/substrate/${substrate_bin}/Cargo.toml"
    if [ ! -f "$substrate_manifest" ]; then
        info "skip ${substrate_bin} (manifest not found)"
        continue
    fi
    info "Building ${substrate_bin}..."
    if ! $DRY_RUN; then
        cargo build --release --manifest-path "$substrate_manifest" 2>&1 | tail -3
    else
        info "[dry-run] would build ${substrate_bin}"
    fi
    install_bin \
        "${REPO_ROOT}/substrate/${substrate_bin}/target/release/${substrate_bin}" \
        "${BIN_DIR}/${substrate_bin}"
    ok "${substrate_bin} → ${BIN_DIR}/${substrate_bin}"
done

# ── 5. liter-llm ─────────────────────────────────────────────────────────────
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
    elif [ -f "${LLM_CFG}" ]; then
        # An existing config is NOT necessarily a working one. Two omissions make
        # the proxy unable to serve a single request, and both were present in the
        # config this installer shipped before 2026-07-30:
        #   - no [general] master_key  -> every /v1/* route answers 401
        #   - outbound_policy default  -> `deny_private` refuses localhost base_urls
        # Warn rather than overwrite: the file may carry real user models/keys.
        _llm_missing=""
        grep -qE '^[[:space:]]*master_key[[:space:]]*=' "${LLM_CFG}" 2>/dev/null \
            || grep -qE '^\[\[keys\]\]' "${LLM_CFG}" 2>/dev/null \
            || _llm_missing="${_llm_missing} master_key"
        if grep -qE '^[[:space:]]*base_url[[:space:]]*=.*(localhost|127\.0\.0\.1)' "${LLM_CFG}" 2>/dev/null; then
            grep -qE '^[[:space:]]*outbound_policy[[:space:]]*=' "${LLM_CFG}" 2>/dev/null \
                || _llm_missing="${_llm_missing} outbound_policy"
        fi
        if [ -n "${_llm_missing}" ]; then
            echo "  ⚠️  liter-llm config at ${LLM_CFG} is missing:${_llm_missing}" >&2
            echo "      without these the proxy returns 401 on every request / blocks loopback" >&2
            echo "      repair with: /liter-llm-bridge configure   (merges, never clobbers)" >&2
        fi
        unset _llm_missing
    fi
else
    info "skip liter-llm (submodule not initialized)"
fi

# ── 5b. openai-proxy (judge gateway on :8181 — OPTIONAL) ─────────────────────
# This is what the `kbd-judge` role resolves to, and its absence silently
# degrades every adversarial review to a same-model self-grade. It is therefore
# vendored (change-arc-009) so the source is pinned and obtainable.
#
# STRICTLY NON-FATAL, and that is the whole point of the block below.
# This script runs under `set -euo pipefail`. Earlier this session, `tools/liter-llm`
# was pinned to a commit whose Cargo.toml hardcoded version = "1.9.3" against a
# workspace that had moved to 1.11.0; `cargo metadata` exited 101, THIS SCRIPT
# ABORTED MID-RUN, and 7 of 14 binaries were left stale. Nothing about that was
# specific to liter-llm — any required submodule build can do it.
#
# So every failure path here warns and continues:
#   - submodule not initialized  -> skip
#   - cargo missing              -> skip
#   - build fails                -> warn, keep going
#   - binary not produced        -> warn, keep going
# A user who never runs an adversarial review must never lose an install to this.
if [ -f "${REPO_ROOT}/tools/openai-proxy/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    info "Building openai-proxy (optional judge gateway)..."
    _oap_built=1
    if ! $DRY_RUN; then
        # `|| _oap_built=0` keeps a non-zero cargo exit from tripping `set -e`.
        (cd "${REPO_ROOT}/tools/openai-proxy" && cargo build --release 2>&1 | tail -3) \
            || _oap_built=0
    else
        info "[dry-run] would run cargo build --release for openai-proxy"
    fi
    if [ "${_oap_built}" -eq 1 ]; then
        # `|| true` is required, not decorative: under `set -e`, an assignment
        # from a command substitution whose command FAILS aborts the script, and
        # `find` on a missing target/release (never built, or a dry-run) exits
        # non-zero. Without it, `--dry-run` died right here.
        OAP_BIN=$(find "${REPO_ROOT}/tools/openai-proxy/target/release" -maxdepth 1 \
                       -name "openai-proxy" -type f 2>/dev/null | head -1) || true
        if [ -n "${OAP_BIN:-}" ]; then
            install_bin "${OAP_BIN}" "${BIN_DIR}/openai-proxy"
            ok "openai-proxy → ${BIN_DIR}/openai-proxy"
        else
            echo "  ⚠️  openai-proxy binary not found after build — adversarial review" >&2
            echo "      will degrade to a same-model self-grade. Install continues." >&2
        fi
    else
        echo "  ⚠️  openai-proxy build failed — adversarial review will degrade to a" >&2
        echo "      same-model self-grade. Install continues; see" >&2
        echo "      docs/decisions/openai-proxy-vendoring.md" >&2
    fi
    unset _oap_built
else
    info "skip openai-proxy (submodule not initialized or cargo unavailable)"
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
    SM_MLX_DIR="${REPO_ROOT}/tools/surreal-memory-server/executors/mlx"
    if [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ] && \
       [ -f "${SM_MLX_DIR}/Package.swift" ]; then
        info "Building surreal-memory MLX executor..."
        if ! $DRY_RUN; then
            swift build --package-path "${SM_MLX_DIR}" -c release \
                --product surreal-memory-mlx-executor
        else
            info "[dry-run] would build surreal-memory-mlx-executor with SwiftPM"
        fi
        SM_MLX_BIN="${SM_MLX_DIR}/.build/release/surreal-memory-mlx-executor"
        SM_MLX_BUNDLE="${SM_MLX_DIR}/.build/release/mlx-swift_Cmlx.bundle"
        if ! $DRY_RUN && [ ! -x "${SM_MLX_BIN}" ]; then
            fail "surreal-memory-mlx-executor binary not found after build"
            exit 1
        fi
        if ! $DRY_RUN && [ ! -f "${SM_MLX_BUNDLE}/Contents/Resources/default.metallib" ]; then
            fail "MLX default.metallib resource bundle not found after build"
            exit 1
        fi
        if ! $DRY_RUN; then
            MODEL_CACHE_DIR="${MODEL_CACHE_DIR:-${HOME}/.cache/huggingface}" \
            LOCAL_EMBEDDING_MODEL="BAAI/bge-small-en-v1.5" \
            LOCAL_EMBEDDING_MODEL_REVISION="5c38ec7c405ec4b44b94cc5a9bb96e735b38267a" \
                "${SM_MLX_BIN}" --prefetch
        else
            info "[dry-run] would prefetch and warm the pinned BGE snapshot"
        fi
        install_bin "${SM_MLX_BIN}" "${BIN_DIR}/surreal-memory-mlx-executor"
        if ! $DRY_RUN; then
            ditto "${SM_MLX_BUNDLE}" "${BIN_DIR}/mlx-swift_Cmlx.bundle"
        fi
        ok "surreal-memory-mlx-executor → ${BIN_DIR}/surreal-memory-mlx-executor"
        if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
            install_bin "${SM_MLX_BIN}" "/usr/local/bin/surreal-memory-mlx-executor"
            if ! $DRY_RUN; then
                ditto "${SM_MLX_BUNDLE}" "/usr/local/bin/mlx-swift_Cmlx.bundle"
            fi
            ok "surreal-memory-mlx-executor → /usr/local/bin/surreal-memory-mlx-executor"
        else
            fail "/usr/local/bin is not writable; MLX deployment requires both installed copies"
            exit 1
        fi
        if ! $DRY_RUN; then
            cmp -s "${BIN_DIR}/surreal-memory-mlx-executor" \
                "/usr/local/bin/surreal-memory-mlx-executor" || {
                fail "installed MLX executor copies differ"
                exit 1
            }
            codesign --verify "${BIN_DIR}/surreal-memory-mlx-executor"
            codesign --verify "/usr/local/bin/surreal-memory-mlx-executor"
            cmp -s "${SM_MLX_BUNDLE}/Contents/Resources/default.metallib" \
                "${BIN_DIR}/mlx-swift_Cmlx.bundle/Contents/Resources/default.metallib" || {
                fail "user-local MLX default.metallib differs from the staged resource"
                exit 1
            }
            cmp -s "${SM_MLX_BUNDLE}/Contents/Resources/default.metallib" \
                "/usr/local/bin/mlx-swift_Cmlx.bundle/Contents/Resources/default.metallib" || {
                fail "system MLX default.metallib differs from the staged resource"
                exit 1
            }
            MODEL_CACHE_DIR="${MODEL_CACHE_DIR:-${HOME}/.cache/huggingface}" \
            LOCAL_EMBEDDING_MODEL="BAAI/bge-small-en-v1.5" \
            LOCAL_EMBEDDING_MODEL_REVISION="5c38ec7c405ec4b44b94cc5a9bb96e735b38267a" \
                "/usr/local/bin/surreal-memory-mlx-executor" --smoke
        fi
    fi

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
        if $DRY_RUN; then
            info "[dry-run] would run cargo build --release in ${cli_dir}"
            return
        elif (cd "${cli_dir}" && cargo build --release 2>&1 | tail -3); then
            if [ -x "${cli_dir}/target/release/cowork" ] && [ -x "${cli_dir}/target/release/co" ]; then
                install_bin "${cli_dir}/target/release/cowork" "${BIN_DIR}/cowork"
                install_bin "${cli_dir}/target/release/co"     "${BIN_DIR}/co"
                ok "cowork → ${BIN_DIR}/cowork"
                ok "co     → ${BIN_DIR}/co"
                return
            fi
            fail "cowork source build completed without both binaries; falling through to release download"
        else
            fail "cowork source build failed; falling through to release download"
        fi
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
        if (cd "${dsg_dir}" && cargo build --release 2>&1 | tail -3); then
            local dsg_bin
            dsg_bin="$(find "${dsg_dir}/target/release" -maxdepth 1 -name "dsg" -type f 2>/dev/null | head -1)"
            if [ -n "${dsg_bin}" ]; then
                install_bin "${dsg_bin}" "${BIN_DIR}/dsg"
                ok "dsg → ${BIN_DIR}/dsg"
                return
            fi
            fail "dsg binary not found after source build; falling through to download"
        else
            fail "dsg source build failed; falling through to download"
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
echo "   Next: bash scripts/install-mcp-services.sh   # install local daemons; control plane stays disabled"
echo "   Sharing only: bash scripts/install-binaries.sh --sharing && bash scripts/install-mcp-services.sh --sharing"
echo "   Then: prometheus setup --check               # verify full system health"
