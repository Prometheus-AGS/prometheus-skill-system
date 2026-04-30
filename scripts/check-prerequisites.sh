#!/usr/bin/env bash
# Check and optionally install prerequisites for the Prometheus Skill System.
#
# Usage:
#   ./scripts/check-prerequisites.sh                       # check only
#   ./scripts/check-prerequisites.sh --install             # install missing
#   ./scripts/check-prerequisites.sh --install --build-tools # +build all submodule binaries
#   ./scripts/check-prerequisites.sh --build-tools         # build tools only (skip install)
#
# --install:     Installs Node, Rust, npm deps, and (best-effort) the legacy
#                three Rust binaries the script always knew about.
# --build-tools: Builds the four submodule tools (forge, pk, liter-llm,
#                surreal-memory-server) and the prometheus-cli, copying the
#                binaries to ~/.local/bin (or /usr/local/bin if writable).
#                Idempotent — skips builds when the binary is already on PATH.

set -euo pipefail

INSTALL=false
BUILD_TOOLS=false
for arg in "$@"; do
    case "$arg" in
        --install) INSTALL=true ;;
        --build-tools) BUILD_TOOLS=true ;;
        --help|-h)
            sed -n '2,15p' "$0" | sed 's/^# //;s/^#//'
            exit 0
            ;;
        *) echo "Unknown flag: $arg (use --help)"; exit 1 ;;
    esac
done

MISSING=0
TOOL_FAILURES=()

# Resolve repo root regardless of cwd.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Where binaries land. Prefer system-wide if writable, else fall back to user.
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "🔍 Prometheus Skill System — Prerequisite Check"
echo "================================================"
echo "  Install dir: $INSTALL_DIR"
echo "  Mode: install=$INSTALL, build-tools=$BUILD_TOOLS"
echo ""

# ── Node.js ──────────────────────────────────────────────────────────────────
check_node() {
    if command -v node >/dev/null 2>&1; then
        local ver
        ver=$(node --version)
        local major
        major=$(echo "$ver" | sed 's/v//' | cut -d. -f1)
        if [ "$major" -ge 18 ]; then
            echo "  ✅ Node.js $ver (>= 18 required)"
        else
            echo "  ❌ Node.js $ver is too old (>= 18 required)"
            MISSING=$((MISSING + 1))
            if $INSTALL; then
                echo "     Installing Node.js via nvm..."
                if command -v nvm >/dev/null 2>&1; then
                    nvm install 22
                elif command -v brew >/dev/null 2>&1; then
                    brew install node@22
                else
                    echo "     ⚠️  Install manually: https://nodejs.org/"
                fi
            fi
        fi
    else
        echo "  ❌ Node.js not found (>= 18 required)"
        MISSING=$((MISSING + 1))
        if $INSTALL; then
            echo "     Installing Node.js..."
            if command -v brew >/dev/null 2>&1; then
                brew install node@22
            elif [ "$(uname)" = "Linux" ]; then
                curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
                sudo apt-get install -y nodejs
            else
                echo "     ⚠️  Install manually: https://nodejs.org/"
            fi
        fi
    fi
}

# ── Rust toolchain ───────────────────────────────────────────────────────────
check_rust() {
    if command -v rustc >/dev/null 2>&1; then
        local ver
        ver=$(rustc --version | awk '{print $2}')
        echo "  ✅ Rust $ver"
        if command -v cargo >/dev/null 2>&1; then
            echo "  ✅ Cargo $(cargo --version | awk '{print $2}')"
        else
            echo "  ❌ Cargo not found (should come with Rust)"
            MISSING=$((MISSING + 1))
        fi
        # WASM target: required for librefang-wasm-skill packaging path.
        check_wasm_target
    else
        echo "  ❌ Rust toolchain not found (required for CLI + MCP servers)"
        MISSING=$((MISSING + 1))
        if $INSTALL; then
            echo "     Installing Rust via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            # shellcheck disable=SC1091
            source "$HOME/.cargo/env" 2>/dev/null || true
            if command -v rustc >/dev/null 2>&1; then
                echo "     ✅ Rust $(rustc --version | awk '{print $2}') installed"
                check_wasm_target
            else
                echo "     ⚠️  Restart your shell to use Rust, or run: source ~/.cargo/env"
            fi
        else
            echo "     Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        fi
    fi
}

check_wasm_target() {
    if ! command -v rustup >/dev/null 2>&1; then
        echo "  ℹ️  rustup not found — cannot verify wasm32-unknown-unknown target"
        return
    fi
    if rustup target list --installed 2>/dev/null | grep -q "^wasm32-unknown-unknown$"; then
        echo "  ✅ wasm32-unknown-unknown target installed"
    else
        echo "  ⚠️  wasm32-unknown-unknown target NOT installed (needed for librefang-wasm packaging)"
        if $INSTALL; then
            echo "     Adding wasm32-unknown-unknown target..."
            if rustup target add wasm32-unknown-unknown 2>/dev/null; then
                echo "     ✅ wasm32-unknown-unknown target added"
            else
                echo "     ⚠️  Failed to add target — try manually: rustup target add wasm32-unknown-unknown"
            fi
        else
            echo "     Add with: rustup target add wasm32-unknown-unknown"
        fi
    fi
}

# ── Git ──────────────────────────────────────────────────────────────────────
check_git() {
    if command -v git >/dev/null 2>&1; then
        echo "  ✅ Git $(git --version | awk '{print $3}')"
    else
        echo "  ❌ Git not found"
        MISSING=$((MISSING + 1))
    fi
}

# ── Docker (optional) ────────────────────────────────────────────────────────
check_docker() {
    if command -v docker >/dev/null 2>&1; then
        echo "  ✅ Docker $(docker --version 2>/dev/null | awk '{print $3}' | tr -d ',')"
        # Compose v2 plugin
        if docker compose version >/dev/null 2>&1; then
            echo "  ✅ Docker Compose v2 ($(docker compose version --short 2>/dev/null))"
        else
            echo "  ⚠️  Docker Compose v2 plugin not found (needed for native-agent stacks)"
        fi
        # Docker Desktop on macOS
        if [ "$(uname)" = "Darwin" ]; then
            if [ -d "/Applications/Docker.app" ]; then
                if pgrep -q -f "Docker Desktop"; then
                    echo "  ✅ Docker Desktop installed and running"
                else
                    echo "  ℹ️  Docker Desktop installed but not running"
                fi
            else
                echo "  ℹ️  Docker Desktop not installed (CLI Docker is fine for non-GUI use)"
            fi
        fi
        # Surface running services we care about.
        if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
            echo "  ✅ surreal-memory running in Docker"
        fi
    else
        echo "  ℹ️  Docker not found (optional — needed for native-agent docker-compose flows)"
    fi
}

# ── npm packages ─────────────────────────────────────────────────────────────
check_npm() {
    if [ -f "$REPO_ROOT/package.json" ]; then
        if [ -d "$REPO_ROOT/node_modules" ]; then
            echo "  ✅ npm dependencies installed"
        else
            echo "  ⚠️  npm dependencies not installed"
            if $INSTALL; then
                echo "     Running npm install..."
                (cd "$REPO_ROOT" && npm install)
                echo "     ✅ Dependencies installed"
            else
                echo "     Run: npm install"
            fi
        fi
    fi
}

# ── Tool builder helper ──────────────────────────────────────────────────────
# Builds a Rust workspace and copies a single binary out.
#
# Usage: build_and_install <bin-name> <workspace-dir> [<cargo-package>] [<expected-version>]
#
# - Skips the build if `<bin-name>` is already on PATH (idempotency).
# - Records failures in TOOL_FAILURES so the caller can surface them at the end
#   without aborting other builds.
build_and_install() {
    local bin="$1"
    local workspace="$2"
    local pkg="${3:-}"   # optional -p flag
    local label="$bin"

    if command -v "$bin" >/dev/null 2>&1; then
        echo "    ✅ $label already on PATH ($(command -v "$bin"))"
        return 0
    fi

    if [ ! -d "$REPO_ROOT/$workspace" ]; then
        echo "    ⚠️  $label workspace missing at $workspace — run: git submodule update --init"
        TOOL_FAILURES+=("$label: workspace missing ($workspace)")
        return 1
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        echo "    ⚠️  cargo not on PATH — cannot build $label"
        TOOL_FAILURES+=("$label: cargo missing")
        return 1
    fi

    echo "    🔨 Building $label from $workspace..."
    local cargo_args=("build" "--release")
    [ -n "$pkg" ] && cargo_args+=("-p" "$pkg")

    if (cd "$REPO_ROOT/$workspace" && cargo "${cargo_args[@]}"); then
        local built="$REPO_ROOT/$workspace/target/release/$bin"
        if [ -f "$built" ]; then
            cp "$built" "$INSTALL_DIR/" && chmod +x "$INSTALL_DIR/$bin"
            echo "    ✅ $label installed → $INSTALL_DIR/$bin"
        else
            echo "    ⚠️  Build succeeded but binary not found at $built"
            TOOL_FAILURES+=("$label: binary not produced")
            return 1
        fi
    else
        echo "    ❌ Build failed for $label"
        TOOL_FAILURES+=("$label: cargo build failed")
        return 1
    fi
}

# ── Submodule tool builds (--build-tools) ────────────────────────────────────
build_submodule_tools() {
    echo ""
    echo "  Submodule Tool Builds:"

    # Make sure submodules are checked out before any cargo build.
    if [ -f "$REPO_ROOT/.gitmodules" ]; then
        if (cd "$REPO_ROOT" && git submodule update --init --recursive >/dev/null 2>&1); then
            echo "    ✅ Submodules initialized"
        else
            echo "    ⚠️  git submodule update reported issues (continuing anyway)"
        fi
    fi

    # Order matters only insofar as long-running builds (surreal-memory) come last
    # so users see the quick wins first.
    build_and_install "forge"                 "tools/forge-rs"               "forge-cli"             || true
    build_and_install "pk"                    "tools/prometheus-knowledge"   "pk-cli"                || true
    build_and_install "liter-llm"             "tools/liter-llm"              "liter-llm-cli"         || true
    build_and_install "prometheus"            "tools/prometheus-cli"         ""                      || true

    # surreal-memory-server can run in Docker; only build the binary if Docker is absent.
    if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
        echo "    ℹ️  surreal-memory running in Docker — skipping native build"
    else
        build_and_install "surreal-memory-server" "tools/surreal-memory-server" "" || true
    fi

    # sycophancy-correction lives under skills/imported/ rather than tools/.
    build_and_install "sycophancy-correction" "skills/imported/sycophancy-correction" "" || true
}

# ── Legacy global binaries (--install only) ──────────────────────────────────
# Kept for backwards compatibility — superseded by --build-tools, but harmless
# when run on top of an existing install since build_and_install is idempotent.
check_binaries() {
    echo ""
    echo "  Global Binaries (best-effort under --install):"
    for bin in prometheus sycophancy-correction surreal-memory-server; do
        if command -v "$bin" >/dev/null 2>&1; then
            echo "    ✅ $bin"
        else
            echo "    ⚠️  $bin not found"
            MISSING=$((MISSING + 1))
            if $INSTALL && ! $BUILD_TOOLS; then
                echo "       Build with: $0 --install --build-tools"
            fi
        fi
    done
}

# ── Run all checks ───────────────────────────────────────────────────────────
echo "  Core Requirements:"
check_node
check_rust
check_git
echo ""
echo "  Optional:"
check_docker
check_npm

if $BUILD_TOOLS; then
    build_submodule_tools
else
    check_binaries
fi

echo ""
echo "================================================"
if [ "$MISSING" -eq 0 ] && [ ${#TOOL_FAILURES[@]} -eq 0 ]; then
    echo "✨ All prerequisites met"
else
    if [ "$MISSING" -gt 0 ]; then
        echo "⚠️  $MISSING core prerequisite(s) missing"
    fi
    if [ ${#TOOL_FAILURES[@]} -gt 0 ]; then
        echo "⚠️  ${#TOOL_FAILURES[@]} tool build(s) failed:"
        for f in "${TOOL_FAILURES[@]}"; do
            echo "     • $f"
        done
    fi
    if ! $INSTALL && ! $BUILD_TOOLS; then
        echo "   Run with --install for core prereqs, --build-tools for tool binaries:"
        echo "     ./scripts/check-prerequisites.sh --install --build-tools"
    fi
    [ "$MISSING" -gt 0 ] || [ ${#TOOL_FAILURES[@]} -gt 0 ] && exit 1
fi
