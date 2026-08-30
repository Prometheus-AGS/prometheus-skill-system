#!/usr/bin/env bash
# detect-toolchain.sh — Platform-agnostic toolchain and service detection.
# Works on Claude Code, Kimi Code, MiniMax, OpenCode, Codex, and any CLI platform.
# Prints a status summary and exits 0 even when items are missing.
#
# Usage:
#   bash shared/scripts/detect-toolchain.sh
#   bash shared/scripts/detect-toolchain.sh --json      # machine-readable output
#   bash shared/scripts/detect-toolchain.sh --required  # exit 1 if any REQUIRED item missing

set -euo pipefail

JSON_MODE=false
REQUIRED_MODE=false
MISSING=0

for arg in "$@"; do
    case "$arg" in
        --json)     JSON_MODE=true ;;
        --required) REQUIRED_MODE=true ;;
    esac
done

declare -A STATUS
declare -A VERSION

check() {
    local key="$1"
    local cmd="$2"
    local version_cmd="${3:-}"
    local required="${4:-false}"

    if command -v "$cmd" >/dev/null 2>&1; then
        STATUS[$key]="ok"
        if [[ -n "$version_cmd" ]]; then
            VERSION[$key]=$(eval "$version_cmd" 2>/dev/null | head -1 | tr -d '\n' || echo "?")
        else
            VERSION[$key]="present"
        fi
    else
        STATUS[$key]="missing"
        VERSION[$key]=""
        if [[ "$required" == "true" ]]; then
            MISSING=$((MISSING + 1))
        fi
    fi
}

check_http() {
    local key="$1"
    local url="$2"

    if curl -sf --max-time 2 "$url" >/dev/null 2>&1; then
        STATUS[$key]="ok"
        VERSION[$key]="reachable"
    else
        STATUS[$key]="missing"
        VERSION[$key]="unreachable"
    fi
}

port_open() {
    local host="$1"
    local port="$2"

    (exec 3<>"/dev/tcp/$host/$port") >/dev/null 2>&1
}

check_sovereign_sync_daemon() {
    local key="sovereign-sync-daemon"
    local output=""
    local code=0

    if command -v sovereign-sync >/dev/null 2>&1; then
        set +e
        output=$(sovereign-sync --mode status --format json 2>/dev/null)
        code=$?
        set -e

        case "$code" in
            0)
                STATUS[$key]="ok"
                # `--mode status` is transport-agnostic; it does not tell us
                # whether the daemon is on a Unix socket (the 1.7.0 default) or
                # on :7892 (--tcp), so do not claim a specific endpoint here.
                VERSION[$key]="healthy"
                ;;
            1)
                if [[ "${PROMETHEUS_SHARING:-0}" =~ ^(1|true|yes)$ ]]; then
                    STATUS[$key]="missing"
                    VERSION[$key]="sharing requested but not running"
                else
                    STATUS[$key]="disabled"
                    VERSION[$key]="optional; enable only for sharing"
                fi
                ;;
            2)
                STATUS[$key]="occupied"
                VERSION[$key]="port :7892 is occupied by a different service"
                ;;
            *)
                STATUS[$key]="missing"
                VERSION[$key]="status command failed"
                ;;
        esac
        return
    fi

    # 1.7.0 serves HTTP on a same-user Unix socket by default and binds no TCP
    # port unless started with --tcp. Probe the socket FIRST; only fall back to
    # :7892 for an explicitly --tcp-configured instance.
    local sovereign_sock="${SOVEREIGN_SYNC_SOCKET:-$HOME/Library/Application Support/prometheus/run/sovereign-sync.sock}"
    if [ -S "$sovereign_sock" ] && curl -sf --max-time 2 --unix-socket "$sovereign_sock" \
        "http://localhost/health" 2>/dev/null \
        | grep -q '"service"[[:space:]]*:[[:space:]]*"sovereign-sync"'; then
        STATUS[$key]="ok"
        VERSION[$key]="healthy on unix socket"
    elif curl -sf --max-time 2 "http://127.0.0.1:7892/health" 2>/dev/null \
        | grep -q '"service"[[:space:]]*:[[:space:]]*"sovereign-sync"'; then
        STATUS[$key]="ok"
        VERSION[$key]="healthy on :7892 (--tcp)"
    elif port_open "127.0.0.1" "7892"; then
        STATUS[$key]="occupied"
        VERSION[$key]="port :7892 is occupied by a different service"
    else
        STATUS[$key]="disabled"
        VERSION[$key]="optional; enable only for sharing"
    fi
}

# ── Core tools ────────────────────────────────────────────────────────────────
check "node"   "node"   "node --version" "true"
check "npm"    "npm"    "npm --version"
check "git"    "git"    "git --version | awk '{print \$3}'" "true"

# ── Rust toolchain ────────────────────────────────────────────────────────────
check "rustc"  "rustc"  "rustc --version | awk '{print \$2}'"
check "cargo"  "cargo"  "cargo --version | awk '{print \$2}'"
check "rustup" "rustup" "rustup --version 2>/dev/null | awk '{print \$2}'"

# ── Go toolchain ──────────────────────────────────────────────────────────────
check "go" "go" "go version | awk '{print \$3}'"

# ── Container runtime ─────────────────────────────────────────────────────────
check "docker" "docker" "docker --version | awk '{print \$3}' | tr -d ','"

# ── AI platform CLIs ─────────────────────────────────────────────────────────
check "kimi"   "kimi"   "kimi --version 2>/dev/null || echo ''"
check "mmx"    "mmx"    ""
check "claude" "claude" "claude --version 2>/dev/null | head -1 || echo ''"

# ── Prometheus binaries ───────────────────────────────────────────────────────
check "forge"                    "forge"                    "forge --version 2>/dev/null | head -1"
check "pk"                       "pk"                       "pk --version 2>/dev/null | head -1"
check "liter-llm"                "liter-llm"                "liter-llm --version 2>/dev/null | head -1"
check "prometheus"               "prometheus"               "prometheus --version 2>/dev/null | head -1"
check "prometheus-rust-auditor"  "prometheus-rust-auditor"  "prometheus-rust-auditor --version 2>/dev/null | head -1"
check "sycophancy-correction"    "sycophancy-correction"    "sycophancy-correction --version 2>/dev/null | head -1"
check "learner-model"            "learner-model"            "learner-model --version 2>/dev/null | head -1"
check "cowork"                   "cowork"                   "cowork --version 2>/dev/null | head -1"
check "dsg"                      "dsg"                      "dsg --version 2>/dev/null | head -1"

# ── MCP services (HTTP reachability) ─────────────────────────────────────────
check_http "surreal-memory"          "http://localhost:23001/health"
check_http "forge-rs"                "http://localhost:8943/mcp"
check_http "prometheus-knowledge"    "http://localhost:8942/mcp"
check_http "surface-bridge"          "http://127.0.0.1:7890/health"
check_sovereign_sync_daemon

# ── WASM target ───────────────────────────────────────────────────────────────
if command -v rustup >/dev/null 2>&1; then
    if rustup target list --installed 2>/dev/null | grep -q "wasm32-unknown-unknown"; then
        STATUS["wasm32"]="ok"
        VERSION["wasm32"]="installed"
    else
        STATUS["wasm32"]="missing"
        VERSION["wasm32"]=""
    fi
else
    STATUS["wasm32"]="missing"
    VERSION["wasm32"]=""
fi

# ── Output ────────────────────────────────────────────────────────────────────

if $JSON_MODE; then
    echo "{"
    first=true
    for key in node npm git rustc cargo rustup go docker kimi mmx claude forge pk liter-llm prometheus prometheus-rust-auditor sycophancy-correction learner-model cowork dsg surreal-memory forge-rs prometheus-knowledge surface-bridge sovereign-sync-daemon wasm32; do
        [[ -n "${STATUS[$key]:-}" ]] || continue
        $first || echo ","
        first=false
        printf '  "%s": {"status": "%s", "version": "%s"}' \
            "$key" "${STATUS[$key]}" "${VERSION[$key]}"
    done
    echo ""
    echo "}"
else
    echo "═══ Prometheus Toolchain Status ═══════════════════════════"

    section() { echo ""; echo "  $1"; }
    item() {
        local key="$1"
        local label="${2:-$1}"
        local hint="${3:-}"
        case "${STATUS[$key]:-missing}" in
            ok)
                printf "  ✅ %-28s %s\n" "$label" "${VERSION[$key]:-}"
                ;;
            occupied)
                printf "  ⚠️  %-28s %s\n" "$label" "${VERSION[$key]:-occupied}"
                ;;
            disabled)
                printf "  ⏸️  %-28s %s\n" "$label" "${VERSION[$key]:-optional}"
                ;;
            *)
                printf "  ℹ️  %-28s missing%s\n" "$label" "${hint:+ — $hint}"
                ;;
        esac
    }

    section "Core"
    item "node"  "Node.js"
    item "npm"   "npm"
    item "git"   "Git"

    section "Rust Toolchain"
    item "rustc"   "rustc"
    item "cargo"   "cargo"
    item "rustup"  "rustup"
    item "wasm32"  "wasm32-unknown-unknown target" "run: rustup target add wasm32-unknown-unknown"

    section "Go Toolchain"
    item "go" "Go"

    section "Container Runtime"
    item "docker" "Docker"

    section "AI Platform CLIs"
    item "kimi"   "Kimi Code CLI"   "https://moonshotai.github.io/kimi-code/"
    item "mmx"    "MiniMax CLI"     "npm install -g @minimax-ai/cli"
    item "claude" "Claude Code CLI" "https://claude.ai/code"

    section "Prometheus Binaries"
    item "forge"                   "forge"
    item "pk"                      "pk (prometheus-knowledge)"
    item "liter-llm"               "liter-llm"
    item "prometheus"              "prometheus CLI"
    item "prometheus-rust-auditor" "prometheus-rust-auditor"
    item "sycophancy-correction"   "sycophancy-correction"
    item "learner-model"  "learner-model (learn substrate)"  "cd substrate/learner-model && cargo build --release"
    item "cowork"         "cowork (skill manager)"           "bash scripts/install-binaries.sh"
    item "dsg"            "dsg (disk-space-guardian)"        "bash scripts/install-binaries.sh"

    section "MCP Services (HTTP)"
    item "surreal-memory"       "surreal-memory (:23001)" "start: cd tools/surreal-memory-server && docker compose up -d"
    item "forge-rs"             "forge-rs (:8943)"
    item "prometheus-knowledge" "prometheus-knowledge (:8942)"
    item "surface-bridge"  "surface-bridge (:7890)" "install: bash scripts/install-skills-flat.sh"
    item "sovereign-sync-daemon" "sovereign-sync sharing daemon" "enable: prometheus setup --full --sharing"

    echo ""
    echo "═══════════════════════════════════════════════════════════"

    if [[ $MISSING -gt 0 ]]; then
        echo "⚠️  $MISSING required tool(s) missing. Run: bash scripts/check-prerequisites.sh --install"
        $REQUIRED_MODE && exit 1
    else
        echo "✨ All required tools present"
    fi
fi
