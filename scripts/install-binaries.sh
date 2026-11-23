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
BIN_DIR="${PROMETHEUS_COMMAND_DIRECTORY:-${HOME}/.local/bin}"
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

# Components whose install failed but did not stop the run. Reported at the end
# and turned into a non-zero exit, so a partial install is never mistaken for a
# clean one. A plain string, not an array: `${#arr[@]}` on an empty array is an
# unbound-variable error under `set -u` in macOS's bash 3.2.
FAILED_COMPONENTS=""

# Prefer the exact package release; source builds cover unpublished targets.
. "$REPO_ROOT/scripts/binary-release.sh"
if $DRY_RUN; then
    info "[dry-run] would verify the release bundle for this pack version and host before any source build"
else
    if [ -n "${PROMETHEUS_BINARY_RELEASE_STAGE:-}" ] || [ "${PROMETHEUS_BINARY_RELEASE_PROBED:-}" != 1 ]; then
        prepare_binary_release
    fi
    mkdir -p "$BIN_DIR"
fi

# prometheus-exec has a stricter evidence-producing installation contract than
# the legacy installers below: version/hash/signature readback is mandatory and
# replacement is atomic with rollback evidence.
#
# A failure here must not abort the whole installer. It runs first, so under
# `set -e` a hash mismatch (e.g. any git-worktree build before the path-
# independent build existed) used to leave every other binary uninstalled.
# The installer's own rollback keeps the previously installed prometheus-exec.
PROMETHEUS_EXEC_ARGS=()
if $DRY_RUN; then
    PROMETHEUS_EXEC_ARGS=(--dry-run)
fi
if ! (
    if release_has prometheus-exec && [ -z "${PROMETHEUS_EXEC_SOURCE_BIN:-}" ]; then
        export PROMETHEUS_EXEC_RELEASE_MANIFEST="$release_stage/manifest.json"
        export PROMETHEUS_EXEC_SOURCE_BIN="$release_stage/bin/prometheus-exec$release_suffix"
    fi
    PROMETHEUS_EXEC_BIN_DIR="${PROMETHEUS_EXEC_BIN_DIR:-$BIN_DIR}" PROMETHEUS_BINARY_RELEASE_PROBED=1 bash "${REPO_ROOT}/scripts/install-prometheus-exec.sh" ${PROMETHEUS_EXEC_ARGS[@]+"${PROMETHEUS_EXEC_ARGS[@]}"}
); then
    fail "prometheus-exec install failed — continuing with the remaining binaries"
    FAILED_COMPONENTS="${FAILED_COMPONENTS} prometheus-exec"
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
    if [ -n "$release_stage" ] && [ "$src" = "$release_stage/bin/$(basename "$dst")" ]; then
        node "$REPO_ROOT/scripts/binary-release.mjs" copy "$dst" "$src"
        return
    fi
    # -f: if dst exists and can't be opened for write (e.g. owned by another
    # user, as happens when a prior install ran under sudo), unlink and retry
    # instead of failing outright.
    cp -f "${src}" "${dst}" || return 1
    if [ "$(uname -s)" = "Darwin" ]; then
        codesign --force --sign - "${dst}" >/dev/null 2>&1 || true
    fi
    node "$REPO_ROOT/scripts/binary-release.mjs" mark "$dst"
}

# cargo_release_dir <workspace-dir> prints where cargo really writes that
# workspace's release binaries. Every install below goes through it: a
# hardcoded <workspace-dir>/target/release path is wrong whenever the target
# directory is redirected (see the helper for what that broke).
# shellcheck source=cargo-release-dir.sh
. "${REPO_ROOT}/scripts/cargo-release-dir.sh"

# Install a workspace's release products as one group. Cargo is needed only if
# this host/version lacks the requested product in the published bundle.
install_products() (
    local workspace="$1" products="$2" release_dir name
    shift 2
    if release_has $products; then
        release_dir="$release_stage/bin"
    elif [ ! -f "$workspace/Cargo.toml" ]; then
        fail "No published binary or initialized source for $products ($workspace)"
        return 1
    else
        if [ "$release_suffix" = ".exe" ] && [ "$products" = "cowork co" ]; then
            if $DRY_RUN; then
                info "[dry-run] would stage the Windows cowork source and run cargo build --release --locked in its cli directory"
                return 0
            fi
            local build_stage
            build_stage="$(mktemp -d)" || return 1
            trap 'rm -rf "$build_stage"' EXIT
            node "$REPO_ROOT/scripts/release/stage-windows-cowork.mjs" "$build_stage/cowork" || return 1
            workspace="$build_stage/cowork/cli"
            set -- --locked "$@"
        elif $DRY_RUN; then
            info "[dry-run] source fallback: cargo build --release $* in $workspace"
            return 0
        fi
        (cd "$workspace" && cargo build --release "$@") || return 1
        release_dir="$(cargo_release_dir "$workspace")"
    fi
    for name in $products; do
        install_bin "$release_dir/$name$release_suffix" "$BIN_DIR/$name$release_suffix" || return 1
        ok "$name → $BIN_DIR/$name$release_suffix"
    done
)
required_products() {
    install_products "$@" || FAILED_COMPONENTS="${FAILED_COMPONENTS} $2"
}

required_products "$REPO_ROOT/tools/prometheus-cli" prometheus -p prometheus-cli
required_products "$REPO_ROOT/crates/prometheus-hook" prometheus-hook
required_products "$REPO_ROOT/tools/prometheus-rust-auditor" prometheus-rust-auditor
required_products "$REPO_ROOT/tools/forge-rs" forge -p forge-cli
required_products "$REPO_ROOT/tools/prometheus-knowledge" 'pk pk-cherry prometheus-learning-worker' -p pk-cli -p pk-cherry -p pk-learning-worker
for substrate_bin in learner-model surface-bridge; do
    required_products "$REPO_ROOT/substrate/$substrate_bin" "$substrate_bin"
done

if install_products "$REPO_ROOT/tools/liter-llm" liter-llm -p liter-llm-cli; then
    # `liter-llm mcp` REQUIRES a config file — without it the stdio server exits
    # and Claude Code fails to load it. Install the default proxy/MCP config
    # (routes via the local :8181 openai-proxy) unless the user already has one.
    LLM_CFG_DIR="${HOME}/.config/liter-llm"
    LLM_CFG="${LLM_CFG_DIR}/liter-llm-proxy.toml"
    if [ ! -f "${LLM_CFG}" ] && [ -f "${REPO_ROOT}/shared/config/liter-llm-proxy.toml" ]; then
        if $DRY_RUN; then
            info "[dry-run] would install liter-llm config at ${LLM_CFG}"
        else
            mkdir -p "${LLM_CFG_DIR}"
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
    FAILED_COMPONENTS="${FAILED_COMPONENTS} liter-llm"
fi

# ── 5a. liter-llm MCP launcher ───────────────────────────────────────────────
# Harness MCP entries run ${BIN_DIR}/liter-llm-mcp, not the binary: the launcher
# sources ~/.prometheus/kbd/secrets.env first, so the server starts in a harness
# launched without the keys in its environment (Dock, Finder, a stale shell).
# The same step moves entries still in the old pack-written form onto it. It is
# independent of the build above (an older installed binary works with it too)
# and a failure is recorded, never fatal.
if command -v node >/dev/null 2>&1; then
    _launcher_args=""
    $DRY_RUN && _launcher_args="--dry-run"
    # shellcheck disable=SC2086
    if node "${REPO_ROOT}/scripts/install-liter-llm-mcp-launcher.mjs" ${_launcher_args}; then
        ok "liter-llm MCP launcher → ${BIN_DIR}/liter-llm-mcp"
    else
        fail "liter-llm MCP launcher install failed — continuing with the remaining binaries"
        FAILED_COMPONENTS="${FAILED_COMPONENTS} liter-llm-mcp-launcher"
    fi
    unset _launcher_args
else
    info "skip liter-llm MCP launcher (node unavailable)"
fi

# The judge gateway is optional. Its own source build failure never obstructs
# the remaining installation; corrupt release archives are rejected above.
if ! install_products "$REPO_ROOT/tools/openai-proxy" openai-proxy; then
    info "Optional openai-proxy unavailable; see docs/decisions/openai-proxy-vendoring.md"
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
install_mlx_executor() (
    if release_has surreal-memory-mlx-executor; then
        SM_MLX_BIN="$release_stage/bin/surreal-memory-mlx-executor"
        SM_MLX_BUNDLE="$release_stage/bin/mlx-swift_Cmlx.bundle"
    else
        if $DRY_RUN; then
            mlx_build="<temporary MLX build directory>"
            info "[dry-run] would run python3 scripts/release/binaries.py mlx-build in an empty stage"
        else
            mlx_build="$(mktemp -d)"
            trap 'rm -rf "$mlx_build"' EXIT
            python3 "$REPO_ROOT/scripts/release/binaries.py" mlx-build "$mlx_build"
        fi
        SM_MLX_BIN="$mlx_build/surreal-memory-mlx-executor"
        SM_MLX_BUNDLE="$mlx_build/mlx-swift_Cmlx.bundle"
    fi
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
    if ! $DRY_RUN && ! release_has surreal-memory-mlx-executor; then
        ditto "${SM_MLX_BUNDLE}" "${BIN_DIR}/mlx-swift_Cmlx.bundle"
    fi
    ok "surreal-memory-mlx-executor → ${BIN_DIR}/surreal-memory-mlx-executor"
    if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
        install_bin "${SM_MLX_BIN}" "/usr/local/bin/surreal-memory-mlx-executor"
        if ! $DRY_RUN && ! release_has surreal-memory-mlx-executor; then
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
)

if release_has surreal-memory-server || [ -f "${REPO_ROOT}/tools/surreal-memory-server/Cargo.toml" ]; then
    SM_MLX_DIR="${REPO_ROOT}/tools/surreal-memory-server/executors/mlx"
    if [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ] && \
       { release_has surreal-memory-mlx-executor || [ -f "${SM_MLX_DIR}/Package.swift" ]; }; then
        install_mlx_executor
    fi

    info "Building surreal-memory-server..."
    SM_FEATURES="embedded,local-embeddings"
    # `metal` only on Apple Silicon: on Intel Macs the daemon runs the CPU device
    # (see install-mcp-services.sh), and Candle's Metal compile path is unreliable.
    if [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ]; then
        SM_FEATURES="embedded,metal,local-embeddings"
    elif command -v nvidia-smi >/dev/null 2>&1; then
        SM_FEATURES="embedded,cuda,local-embeddings"
    fi
    if release_has surreal-memory-server; then
        SM_BIN="$release_stage/bin/surreal-memory-server$release_suffix"
    else
        if ! $DRY_RUN; then
            (cd "${REPO_ROOT}/tools/surreal-memory-server" &&
                RUSTFLAGS="-Dwarnings" cargo build --release --no-default-features --features "${SM_FEATURES}")
        else
            info "[dry-run] would build surreal-memory-server with features ${SM_FEATURES}"
        fi
        SM_BIN="$(cargo_release_dir "${REPO_ROOT}/tools/surreal-memory-server")/surreal-memory-server$release_suffix"
    fi
    if [ -f "${SM_BIN}" ]; then
        install_bin "${SM_BIN}" "${BIN_DIR}/surreal-memory-server$release_suffix"
        ok "surreal-memory-server → ${BIN_DIR}/surreal-memory-server"
        if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
            install_bin "${SM_BIN}" "/usr/local/bin/surreal-memory-server$release_suffix"
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
if release_has sycophancy-correction || [ -f "${SYCO_DIR}/Cargo.toml" ]; then
    info "Building sycophancy-correction..."
    if release_has sycophancy-correction; then
        SYCO_BIN="$release_stage/bin/sycophancy-correction$release_suffix"
    else
        if ! $DRY_RUN; then
            (cd "${SYCO_DIR}" && cargo build --release)
        else
            info "[dry-run] would build sycophancy-correction"
        fi
        SYCO_BIN="$(cargo_release_dir "${SYCO_DIR}")/sycophancy-correction$release_suffix"
    fi
    if [ -f "${SYCO_BIN}" ]; then
        install_bin "${SYCO_BIN}" "${BIN_DIR}/sycophancy-correction$release_suffix"
        ok "sycophancy-correction → ${BIN_DIR}/sycophancy-correction"
        if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
            install_bin "${SYCO_BIN}" "/usr/local/bin/sycophancy-correction$release_suffix"
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

TEMPLATE_FORGE_DIR="${REPO_ROOT}/skills/imported/artifact-refiner/tools/template-forge-rs"
RUSTUP_TOOLCHAIN=stable required_products "$TEMPLATE_FORGE_DIR" 'template-forge template-forge-mcp'
required_products "$REPO_ROOT/tools/cowork-skills/cli" 'cowork co'
required_products "$REPO_ROOT/tools/disk-space-guardian" dsg

# ── 10. prometheus-research (HTTP + MCP research server on :7891) ────────────
if release_has prometheus-research || [ -f "${REPO_ROOT}/substrate/prometheus-research/Cargo.toml" ]; then
    info "Building prometheus-research..."
    if release_has prometheus-research; then
        PR_BIN="$release_stage/bin/prometheus-research$release_suffix"
    else
        PR_BIN="$(cargo_release_dir "${REPO_ROOT}/substrate/prometheus-research")/prometheus-research$release_suffix"
    fi
    # A dry run must change nothing. Beyond the build and the binary, this
    # section copies static assets into the repository and re-registers a
    # launchd service, which restarts the running daemon.
    if $DRY_RUN; then
        info "[dry-run] would run cargo build --release for prometheus-research"
        install_bin "${PR_BIN}" "${BIN_DIR}/prometheus-research$release_suffix"
        info "[dry-run] would copy static assets to docs/deep-research/static/"
        if [ "$(uname -s)" = "Darwin" ]; then
            info "[dry-run] would render and re-register the com.prometheus.research launchd service"
        fi
    elif ! release_has prometheus-research && ! (cd "${REPO_ROOT}/substrate/prometheus-research" && cargo build --release 2>&1 | tail -3); then
        # Record it so the run still ends non-zero, as it did when this build
        # ran unguarded and aborted the script.
        fail "prometheus-research build failed"
        FAILED_COMPONENTS="${FAILED_COMPONENTS} prometheus-research"
    elif [ -f "${PR_BIN}" ]; then
        install_bin "${PR_BIN}" "${BIN_DIR}/prometheus-research$release_suffix"
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
            # The daemon spawns a headless harness child, so it needs a PATH that
            # can resolve `claude` or `codex`. launchd grants a service only what
            # its plist declares: substituting __HOME__ alone left PATH unset and
            # every research_start blocked (defect D-A). Same PATH composition the
            # sibling services use in scripts/install-mcp-services.sh.
            RESEARCH_PATH="/usr/local/bin:/usr/local/sbin:/opt/homebrew/bin:/opt/homebrew/sbin:${HOME}/.cargo/bin:${HOME}/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
            sed -e "s|__HOME__|${HOME}|g" \
                -e "s|__PROMETHEUS_HOME__|${HOME}|g" \
                -e "s|__PROMETHEUS_USER__|$(id -un)|g" \
                -e "s|__PROMETHEUS_PATH__|${RESEARCH_PATH}|g" \
                "${PLIST_SRC}" > "${PLIST_DST}"
            # Fail loudly rather than bootstrapping a plist that still carries a
            # placeholder — an unsubstituted PATH is exactly defect D-A again.
            if grep -q '__PROMETHEUS_\|__HOME__' "${PLIST_DST}"; then
                fail "prometheus-research plist still has unsubstituted placeholders: $(grep -o '__[A-Z_]*__' "${PLIST_DST}" | sort -u | tr '\n' ' ')"
            fi
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
if [ -n "${FAILED_COMPONENTS}" ]; then
    fail "install incomplete; failed components:${FAILED_COMPONENTS}"
    echo "   Every other binary above was installed to ${BIN_DIR}." >&2
    exit 1
fi
echo "✨ All binaries installed to ${BIN_DIR}"
echo "   Next: bash scripts/install-mcp-services.sh   # install managed local services"
echo "   Control-plane extensions are installed by their owning products."
echo "   Then: prometheus setup --check               # verify full system health"
