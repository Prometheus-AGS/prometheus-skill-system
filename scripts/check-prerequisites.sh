#!/usr/bin/env bash
# Check and optionally install prerequisites for the Prometheus Skill System.
#
# Usage:
#   ./scripts/check-prerequisites.sh          # check only
#   ./scripts/check-prerequisites.sh --install # check + install missing

set -euo pipefail

INSTALL=false
[ "${1:-}" = "--install" ] && INSTALL=true

MISSING=0

echo "🔍 Prometheus Skill System — Prerequisite Check"
echo "================================================"
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
        # Check cargo too
        if command -v cargo >/dev/null 2>&1; then
            echo "  ✅ Cargo $(cargo --version | awk '{print $2}')"
        else
            echo "  ❌ Cargo not found (should come with Rust)"
            MISSING=$((MISSING + 1))
        fi
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
            else
                echo "     ⚠️  Restart your shell to use Rust, or run: source ~/.cargo/env"
            fi
        else
            echo "     Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
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
        # Check if surreal-memory is running in Docker
        if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
            echo "  ✅ surreal-memory running in Docker"
        fi
    else
        echo "  ℹ️  Docker not found (optional — needed for docker-based surreal-memory)"
    fi
}

# ── npm packages ─────────────────────────────────────────────────────────────
check_npm() {
    if [ -f "package.json" ]; then
        if [ -d "node_modules" ]; then
            echo "  ✅ npm dependencies installed"
        else
            echo "  ⚠️  npm dependencies not installed"
            if $INSTALL; then
                echo "     Running npm install..."
                npm install
                echo "     ✅ Dependencies installed"
            else
                echo "     Run: npm install"
            fi
        fi
    fi
}

# ── Global binaries ──────────────────────────────────────────────────────────
check_binaries() {
    echo ""
    echo "  Global Binaries:"
    for bin in prometheus sycophancy-correction surreal-memory-server; do
        if command -v "$bin" >/dev/null 2>&1; then
            echo "    ✅ $bin"
        else
            echo "    ❌ $bin not found (build with: see README.md)"
            MISSING=$((MISSING + 1))
            if $INSTALL && command -v cargo >/dev/null 2>&1; then
                case "$bin" in
                    prometheus)
                        echo "       Building prometheus CLI..."
                        (cd tools/prometheus-cli && cargo build --release) 2>/dev/null
                        cp tools/prometheus-cli/target/release/prometheus /usr/local/bin/ 2>/dev/null || \
                            (mkdir -p ~/.local/bin && cp tools/prometheus-cli/target/release/prometheus ~/.local/bin/)
                        echo "       ✅ Built and installed"
                        ;;
                    sycophancy-correction)
                        echo "       Building sycophancy-correction MCP server..."
                        (cd skills/imported/sycophancy-correction && cargo build --release) 2>/dev/null
                        cp skills/imported/sycophancy-correction/target/release/sycophancy-correction /usr/local/bin/ 2>/dev/null || \
                            (mkdir -p ~/.local/bin && cp skills/imported/sycophancy-correction/target/release/sycophancy-correction ~/.local/bin/)
                        echo "       ✅ Built and installed"
                        # Verify the binary actually responds
                        if bash skills/imported/sycophancy-correction/scripts/smoke-test.sh >/dev/null 2>&1; then
                            echo "       ✅ Smoke test passed"
                        else
                            echo "       ⚠️  Binary installed but smoke test failed — run skills/imported/sycophancy-correction/scripts/smoke-test.sh for details"
                        fi
                        ;;
                    surreal-memory-server)
                        # Check Docker first
                        if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
                            echo "       ℹ️  Running in Docker — binary install not needed"
                        else
                            echo "       Building surreal-memory-server (this takes ~10 minutes)..."
                            (cd tools/surreal-memory-server && cargo build --release) 2>/dev/null
                            cp tools/surreal-memory-server/target/release/surreal-memory-server /usr/local/bin/ 2>/dev/null || \
                                (mkdir -p ~/.local/bin && cp tools/surreal-memory-server/target/release/surreal-memory-server ~/.local/bin/)
                            echo "       ✅ Built and installed"
                        fi
                        ;;
                esac
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
check_binaries

echo ""
echo "================================================"
if [ "$MISSING" -eq 0 ]; then
    echo "✨ All prerequisites met"
else
    echo "⚠️  $MISSING prerequisite(s) missing"
    if ! $INSTALL; then
        echo "   Run with --install to auto-install: ./scripts/check-prerequisites.sh --install"
    fi
fi
