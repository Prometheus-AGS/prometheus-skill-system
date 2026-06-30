#!/usr/bin/env bash
# Check and optionally install prerequisites for the Prometheus Skill System.
#
# Cross-platform: Linux, macOS, and Windows (run under Git Bash / MSYS2 / WSL).
#
# Usage:
#   ./scripts/check-prerequisites.sh                          # check only
#   ./scripts/check-prerequisites.sh --install                # install missing
#   ./scripts/check-prerequisites.sh --build-tools            # build tool binaries
#   ./scripts/check-prerequisites.sh --install --build-tools  # both
#   ./scripts/check-prerequisites.sh --build-tools --docker   # run daemons via Docker
#   ./scripts/check-prerequisites.sh --build-tools --native   # run daemons natively
#
# --install:     Installs Node, Rust, the C/C++ build toolchain, npm deps, and
#                (best-effort) the legacy Rust binaries the script always knew about.
#                Installation commands are chosen per-OS (apt/dnf/pacman/zypper/apk
#                on Linux, Homebrew/xcode-select on macOS, winget/MSYS2 on Windows).
# --build-tools: Builds submodule tools (forge, pk, pk-cherry, liter-llm,
#                surreal-memory-server), prometheus-cli, and the rust auditor, copying the
#                binaries to ~/.local/bin (or /usr/local/bin if writable).
#                Idempotent — skips builds when the binary is already on PATH.
# --runtime=X:   How to run daemon services (surreal-memory, etc.):
#                  docker — via docker compose;  native — build + run binaries;
#                  auto   — prompt interactively, or pick a default when headless (default).
#                Shorthands: --docker (= --runtime=docker), --native (= --runtime=native).

set -euo pipefail

INSTALL=false
BUILD_TOOLS=false
RUNTIME="auto"   # auto | docker | native — how to run daemon services
for arg in "$@"; do
    case "$arg" in
        --install) INSTALL=true ;;
        --build-tools) BUILD_TOOLS=true ;;
        --runtime=*) RUNTIME="${arg#*=}" ;;
        --docker) RUNTIME="docker" ;;
        --native) RUNTIME="native" ;;
        --help|-h)
            # Print the leading comment block (up to `set -euo`) as help text.
            sed -n '2,/^set -euo/p' "$0" | sed '/^set -euo/d' | sed 's/^# \{0,1\}//;s/^#//'
            exit 0
            ;;
        *) echo "Unknown flag: $arg (use --help)"; exit 1 ;;
    esac
done

case "$RUNTIME" in
    auto|docker|native) ;;
    *) echo "Invalid --runtime='$RUNTIME' (expected: auto, docker, or native)"; exit 1 ;;
esac

MISSING=0
TOOL_FAILURES=()

# Resolve repo root regardless of cwd.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# ── Platform detection ───────────────────────────────────────────────────────
# OS:      linux | macos | windows | unknown
# PKG_MGR: the Linux package manager (apt|dnf|yum|pacman|zypper|apk) or "unknown".
# EXE:     executable suffix (".exe" on Windows, empty elsewhere).
detect_platform() {
    case "$(uname -s 2>/dev/null)" in
        Linux*)               OS="linux" ;;
        Darwin*)              OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT) OS="windows" ;;
        *)                    OS="unknown" ;;
    esac

    EXE=""
    [ "$OS" = "windows" ] && EXE=".exe"

    PKG_MGR="unknown"
    if [ "$OS" = "linux" ]; then
        for m in apt-get dnf yum pacman zypper apk; do
            if command -v "$m" >/dev/null 2>&1; then
                case "$m" in
                    apt-get) PKG_MGR="apt" ;;
                    *)       PKG_MGR="$m" ;;
                esac
                break
            fi
        done
    fi
}
detect_platform

# Where binaries land. Prefer system-wide if writable, else fall back to user.
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "🔍 Prometheus Skill System — Prerequisite Check"
echo "================================================"
PLATFORM_LABEL="$OS"
[ "$OS" = "linux" ] && [ "$PKG_MGR" != "unknown" ] && PLATFORM_LABEL="$OS ($PKG_MGR)"
echo "  Platform:    $PLATFORM_LABEL"
echo "  Install dir: $INSTALL_DIR"
echo "  Mode: install=$INSTALL, build-tools=$BUILD_TOOLS, runtime=$RUNTIME"
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
                install_node
            fi
        fi
    else
        echo "  ❌ Node.js not found (>= 18 required)"
        MISSING=$((MISSING + 1))
        if $INSTALL; then
            install_node
        fi
    fi
}

# Cross-platform Node.js install (best-effort).
install_node() {
    echo "     Installing Node.js..."
    if command -v nvm >/dev/null 2>&1; then
        nvm install 22
    elif command -v brew >/dev/null 2>&1; then
        brew install node@22
    elif [ "$OS" = "linux" ]; then
        case "$PKG_MGR" in
            apt)    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs ;;
            dnf)    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo -E bash - && sudo dnf install -y nodejs ;;
            yum)    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo -E bash - && sudo yum install -y nodejs ;;
            pacman) sudo pacman -S --needed --noconfirm nodejs npm ;;
            zypper) sudo zypper install -y nodejs22 || sudo zypper install -y nodejs ;;
            apk)    sudo apk add nodejs npm ;;
            *)      echo "     ⚠️  Unknown package manager — install Node 22 manually: https://nodejs.org/" ;;
        esac
    elif [ "$OS" = "windows" ]; then
        if command -v winget >/dev/null 2>&1; then
            winget install -e --id OpenJS.NodeJS.LTS
        else
            echo "     ⚠️  Install via: winget install OpenJS.NodeJS.LTS  (or https://nodejs.org/)"
        fi
    else
        echo "     ⚠️  Install manually: https://nodejs.org/"
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
            if [ "$OS" = "windows" ]; then
                echo "     Installing Rust on Windows..."
                if command -v winget >/dev/null 2>&1; then
                    winget install -e --id Rustlang.Rustup
                    echo "     ℹ️  Choose the MSVC toolchain, then install the MSVC build tools (see C/C++ toolchain check)."
                else
                    echo "     ⚠️  Install via: winget install Rustlang.Rustup  (or https://rustup.rs/)"
                fi
            else
                echo "     Installing Rust via rustup..."
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                # shellcheck disable=SC1091
                source "$HOME/.cargo/env" 2>/dev/null || true
            fi
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

# ── C/C++ build toolchain (linker) ───────────────────────────────────────────
# Rust links through a system C compiler/linker (`cc`). Without it EVERY cargo
# build fails with `error: linker `cc` not found`. This is the #1 cause of the
# submodule build failures, so check it explicitly and offer per-OS installation.
check_build_tools() {
    local have_cc=false which_cc=""
    for c in cc gcc clang; do
        if command -v "$c" >/dev/null 2>&1; then have_cc=true; which_cc=$(command -v "$c"); break; fi
    done
    # On Windows the MSVC toolchain links via cl.exe/link.exe rather than cc.
    if ! $have_cc && [ "$OS" = "windows" ]; then
        for c in cl link; do
            if command -v "$c" >/dev/null 2>&1; then have_cc=true; which_cc=$(command -v "$c"); break; fi
        done
    fi

    if $have_cc; then
        echo "  ✅ C/C++ build toolchain ($which_cc)"
        return
    fi

    echo "  ❌ C/C++ build toolchain (linker 'cc') not found — required to compile Rust tools"
    MISSING=$((MISSING + 1))

    case "$OS" in
        linux)
            local hint
            case "$PKG_MGR" in
                apt)    hint="sudo apt-get update && sudo apt-get install -y build-essential pkg-config libssl-dev" ;;
                dnf)    hint="sudo dnf groupinstall -y 'Development Tools' && sudo dnf install -y pkgconf-pkg-config openssl-devel" ;;
                yum)    hint="sudo yum groupinstall -y 'Development Tools' && sudo yum install -y pkgconfig openssl-devel" ;;
                pacman) hint="sudo pacman -S --needed --noconfirm base-devel pkgconf openssl" ;;
                zypper) hint="sudo zypper install -y -t pattern devel_basis && sudo zypper install -y pkg-config libopenssl-devel" ;;
                apk)    hint="sudo apk add build-base pkgconf openssl-dev" ;;
                *)      hint="install your distro's build tools (a C compiler, make, pkg-config, OpenSSL dev headers)" ;;
            esac
            if $INSTALL && [ "$PKG_MGR" != "unknown" ]; then
                echo "     Installing build tools via $PKG_MGR..."
                if eval "$hint"; then
                    echo "     ✅ Build tools installed"
                    MISSING=$((MISSING - 1))
                else
                    echo "     ⚠️  Automatic install failed — run manually: $hint"
                fi
            else
                echo "     Install with: $hint"
            fi
            ;;
        macos)
            if $INSTALL; then
                echo "     Installing Xcode Command Line Tools..."
                if xcode-select --install 2>/dev/null; then
                    echo "     ℹ️  Accept the GUI prompt to finish, then re-run this script."
                else
                    echo "     ⚠️  Already triggered or unavailable — run manually: xcode-select --install"
                fi
            else
                echo "     Install with: xcode-select --install"
            fi
            ;;
        windows)
            echo "     Install the MSVC C++ build tools (used by the Rust MSVC toolchain):"
            echo "       winget install Microsoft.VisualStudio.2022.BuildTools \\"
            echo "         --override \"--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended\""
            echo "     Or install the GNU toolchain via MSYS2:  pacman -S mingw-w64-x86_64-gcc"
            ;;
        *)
            echo "     Install a C compiler/linker (gcc or clang) for your platform."
            ;;
    esac
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

# ── Reachability probes ──────────────────────────────────────────────────────
# probe_port / check_running_service live in the shared lib so this script and
# scripts/install-mcp-services.sh detect "already running" the same way.
# shellcheck source=../shared/scripts/service-probe.sh
. "$REPO_ROOT/shared/scripts/service-probe.sh"

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
        local target_dir built
        target_dir=$(cd "$REPO_ROOT/$workspace" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
            | node -e "let s='';process.stdin.on('data',d=>s+=d);process.stdin.on('end',()=>{try{console.log(JSON.parse(s).target_directory)}catch{process.exit(1)}})" 2>/dev/null \
            || printf '%s/target' "$REPO_ROOT/$workspace")
        built="$target_dir/release/$bin$EXE"
        if [ -f "$built" ]; then
            cp "$built" "$INSTALL_DIR/" && chmod +x "$INSTALL_DIR/$bin$EXE"
            echo "    ✅ $label installed → $INSTALL_DIR/$bin$EXE"
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

# ── SurrealDB backend detection ──────────────────────────────────────────────
# surreal-memory-server needs a SurrealDB engine to talk to. If SurrealDB is
# ALREADY installed (the `surreal` binary on PATH) or ALREADY running (default
# :8000, or the docker-mapped :28000), reuse it instead of installing/starting a
# new one. Sets:
#   SURREALDB_PRESENT  — true if a usable SurrealDB (binary or server) was found
#   SURREALDB_ENDPOINT — ws:// endpoint of a running server to reuse (if any)
SURREALDB_PRESENT=false
SURREALDB_ENDPOINT=""
SURREALDB_CHECKED=false
check_surrealdb() {
    $SURREALDB_CHECKED && return
    SURREALDB_CHECKED=true

    local found_bin="" running_port=""

    command -v surreal >/dev/null 2>&1 && found_bin=$(command -v surreal)

    # Probe SurrealDB's HTTP ports: 8000 (native default), 28000 (docker-mapped).
    local p
    for p in 8000 28000; do
        if probe_port "$p" "/health" || probe_port "$p" "/version"; then
            running_port="$p"
            break
        fi
    done

    if [ -n "$found_bin" ]; then
        local ver
        ver=$(surreal version 2>/dev/null | head -1 || true)
        echo "    ✅ SurrealDB installed — reusing it ($found_bin${ver:+, $ver})"
        SURREALDB_PRESENT=true
    fi
    if [ -n "$running_port" ]; then
        SURREALDB_ENDPOINT="ws://localhost:$running_port"
        echo "    ✅ SurrealDB already running on :$running_port — reusing it ($SURREALDB_ENDPOINT)"
        SURREALDB_PRESENT=true
    fi

    if ! $SURREALDB_PRESENT; then
        echo "    ℹ️  No existing SurrealDB found (no 'surreal' binary; nothing on :8000/:28000)"
        if $INSTALL && [ "$RUNTIME" = "native" ]; then
            install_surrealdb
        fi
    fi
}

# Install the SurrealDB engine (only called when none exists and runtime=native).
install_surrealdb() {
    echo "       Installing SurrealDB engine..."
    if command -v brew >/dev/null 2>&1; then
        brew install surrealdb/tap/surreal
    elif [ "$OS" = "windows" ]; then
        if command -v winget >/dev/null 2>&1; then
            winget install -e --id SurrealDB.SurrealDB \
                || echo "       ⚠️  Install manually: https://surrealdb.com/install"
        else
            echo "       ⚠️  Install via PowerShell: iwr https://windows.surrealdb.com -useb | iex"
            echo "          (or see https://surrealdb.com/install)"
        fi
    else
        # Official installer (Linux + macOS).
        curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh
    fi
    if command -v surreal >/dev/null 2>&1; then
        echo "       ✅ SurrealDB installed ($(command -v surreal))"
        SURREALDB_PRESENT=true
    else
        echo "       ⚠️  SurrealDB not on PATH yet — restart your shell (see https://surrealdb.com/install)"
    fi
}

# ── Runtime choice for daemon services (docker vs native) ────────────────────
# Daemon services (surreal-memory, etc.) can either run as Docker containers or
# as natively-built binaries/daemons. The user chooses via --docker/--native;
# when left on "auto" we prompt interactively, or fall back to a sensible default
# in non-interactive (CI/headless) runs. Resolution happens once and is cached.
RUNTIME_RESOLVED=false
docker_available() {
    command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1
}
resolve_runtime() {
    $RUNTIME_RESOLVED && return
    RUNTIME_RESOLVED=true

    [ "$RUNTIME" != "auto" ] && { echo "  ℹ️  Daemon runtime: $RUNTIME (from flag)"; return; }

    # Default leans to Docker only when Docker + Compose are actually usable.
    local default_choice="native"
    docker_available && default_choice="docker"

    # Non-interactive (no TTY): can't prompt — use the default.
    if [ ! -t 0 ]; then
        RUNTIME="$default_choice"
        echo "  ℹ️  Daemon runtime: $RUNTIME (auto-selected; no TTY). Override with --docker or --native."
        return
    fi

    # Interactive prompt.
    echo ""
    echo "  How should daemon services (surreal-memory, etc.) run?"
    echo "    [d] Docker  — via docker compose (isolated; requires Docker + Compose)"
    echo "    [n] Native  — build + run binaries directly on $OS"
    docker_available || echo "    (Docker/Compose not detected — native recommended)"
    local reply
    read -r -p "  Choose [d/n] (default: ${default_choice:0:1}): " reply || reply=""
    reply="${reply:-${default_choice:0:1}}"
    case "$reply" in
        d|D|docker|Docker) RUNTIME="docker" ;;
        n|N|native|Native) RUNTIME="native" ;;
        *) RUNTIME="$default_choice"; echo "  ⚠️  Unrecognized choice — using $RUNTIME." ;;
    esac
    echo "  → Daemon runtime: $RUNTIME"
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
    build_and_install "pk-cherry"             "tools/prometheus-knowledge"   "pk-cherry"             || true
    build_and_install "liter-llm"             "tools/liter-llm"              "liter-llm-cli"         || true
    build_and_install "prometheus"            "tools/prometheus-cli"         ""                      || true
    build_and_install "prometheus-rust-auditor" "tools/prometheus-rust-auditor" ""                    || true

    # SurrealDB backend: reuse an existing install/instance if present rather than
    # installing or starting a new one.
    echo "    Datastore (SurrealDB):"
    check_surrealdb

    # surreal-memory-server: honor the user's runtime choice (docker vs native).
    # An already-running instance (any provenance, detected via HTTP/port probe)
    # always wins so we never stomp on a service started elsewhere.
    if check_running_service "surreal-memory-server" 23001 "/mcp/sse"; then
        :  # already up — leave it alone
    elif [ "$RUNTIME" = "docker" ]; then
        if docker_available; then
            echo "    ℹ️  Runtime=docker — surreal-memory-server runs via docker compose:"
            echo "       (cd $REPO_ROOT/tools/surreal-memory-server && docker compose up -d)"
            echo "       (requires .env with OPENAI_API_KEY for embeddings)"
            if $SURREALDB_PRESENT && [ -n "$SURREALDB_ENDPOINT" ]; then
                echo "    ℹ️  An external SurrealDB is already running ($SURREALDB_ENDPOINT)."
                echo "       To reuse it instead of the bundled DB: set SURREAL_ENDPOINT and start"
                echo "       only the server — docker compose up -d surreal-memory-server"
            fi
            if $INSTALL; then
                if (cd "$REPO_ROOT/tools/surreal-memory-server" && docker compose up -d); then
                    echo "    ✅ surreal-memory-server started via Docker"
                else
                    echo "    ❌ docker compose up failed for surreal-memory-server"
                    TOOL_FAILURES+=("surreal-memory-server: docker compose failed")
                fi
            fi
        else
            echo "    ⚠️  Runtime=docker requested but Docker/Compose unavailable — falling back to native build."
            build_and_install "surreal-memory-server" "tools/surreal-memory-server" "" || true
        fi
    else
        # Native runtime: build the binary directly.
        if $SURREALDB_PRESENT && [ -n "$SURREALDB_ENDPOINT" ]; then
            echo "    ℹ️  Point the server at the existing DB: export SURREAL_ENDPOINT=$SURREALDB_ENDPOINT"
        fi
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
    for bin in prometheus prometheus-rust-auditor pk-cherry sycophancy-correction; do
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

    # SurrealDB backend: reuse an existing install/instance if present.
    echo "    Datastore (SurrealDB):"
    check_surrealdb

    # surreal-memory-server can be satisfied by a running service (any provenance)
    # OR a native binary. Don't double-count.
    if check_running_service "surreal-memory-server" 23001 "/mcp/sse"; then
        :
    elif command -v surreal-memory-server >/dev/null 2>&1; then
        echo "    ✅ surreal-memory-server (native binary)"
    else
        echo "    ⚠️  surreal-memory-server not found (no running service, no binary)"
        MISSING=$((MISSING + 1))
        if $INSTALL && ! $BUILD_TOOLS; then
            if [ "$RUNTIME" = "docker" ] && docker_available; then
                echo "       Start via Docker: (cd $REPO_ROOT/tools/surreal-memory-server && docker compose up -d)"
            else
                echo "       Build native:  $0 --build-tools --native"
                docker_available && echo "       Or via Docker: $0 --build-tools --docker"
            fi
        fi
    fi
}

# ── MCP service reachability report (informational) ─────────────────────────
# Probes the URL-based MCP servers configured in .mcp.json. These services may
# be started by Docker, native binaries, or external repos — the probe is
# provenance-agnostic.
check_mcp_services() {
    echo ""
    echo "  Running MCP services (informational):"
    check_running_service "surreal-memory   (:23001)" 23001 "/mcp/sse" \
        || echo "    ✗ surreal-memory   not reachable on :23001"
    check_running_service "forge-rs         (:8943)"  8943  "/mcp" \
        || echo "    ✗ forge-rs         not reachable on :8943"
    check_running_service "prometheus-knowledge (:8942)" 8942 "/mcp" \
        || echo "    ✗ prometheus-knowledge not reachable on :8942"
}

# ── Run all checks ───────────────────────────────────────────────────────────
echo "  Core Requirements:"
check_node
check_rust
check_build_tools
check_git
echo ""
echo "  Optional:"
check_docker
check_npm

  # AI CLI platforms
  echo ""
  echo "  AI CLI Platforms (optional — needed for cross-platform skill installation):"
  if command -v kimi >/dev/null 2>&1; then
      echo "  ✅ kimi $(kimi --version 2>/dev/null || echo '(version unknown)')"
  else
      echo "  ℹ️  kimi not found — install from https://moonshotai.github.io/kimi-code/"
  fi
  if command -v mmx >/dev/null 2>&1; then
      echo "  ✅ mmx (MiniMax CLI) found"
  else
      echo "  ℹ️  mmx not found — install via: npm install -g @minimax-ai/cli"
  fi

# Decide how daemon services should run before we attempt to build/start them.
resolve_runtime

if $BUILD_TOOLS; then
    build_submodule_tools
else
    check_binaries
fi

check_mcp_services

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
