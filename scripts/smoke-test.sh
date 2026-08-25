#!/usr/bin/env bash
# Smoke test: confirm every Prometheus skill-pack tool binary is reachable
# and responds to --version. Used by `npm run doctor` and CI.
#
# Exits non-zero if any required binary is missing or fails. Optional
# binaries (currently surreal-memory-server when running in Docker) are
# allowed to be absent.

set -uo pipefail

PASS=0
FAIL=0
SKIP=0

# Binaries that implement --version (we run it as a smoke check).
required=(forge pk liter-llm prometheus)

# Binaries we want present but that don't have a --version flag — for these
# we only check that they exist on PATH.
#   surreal-memory-server: SurrealDB-derived; uses subcommands, not --version.
#   sycophancy-correction: stdio MCP server; expects an initialize request.
optional_presence_only=(surreal-memory-server sycophancy-correction)
optional=()

echo "🧪 Prometheus Skill System — Smoke Test"
echo "========================================"

check_bin() {
    local bin="$1"
    local required="$2"
    if command -v "$bin" >/dev/null 2>&1; then
        if "$bin" --version >/dev/null 2>&1; then
            local ver
            ver=$("$bin" --version 2>&1 | head -1)
            echo "  ✅ $bin — $ver"
            PASS=$((PASS + 1))
        else
            echo "  ❌ $bin found but --version failed"
            FAIL=$((FAIL + 1))
        fi
    else
        if [ "$required" = "required" ]; then
            echo "  ❌ $bin not found (required)"
            FAIL=$((FAIL + 1))
        else
            # surreal-memory-server in Docker is normal.
            if [ "$bin" = "surreal-memory-server" ] && \
               docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
                echo "  ✅ $bin running in Docker (binary not required on PATH)"
                PASS=$((PASS + 1))
            else
                echo "  ⚠️  $bin not found (optional)"
                SKIP=$((SKIP + 1))
            fi
        fi
    fi
}

check_presence() {
    local bin="$1"
    if command -v "$bin" >/dev/null 2>&1; then
        echo "  ✅ $bin (on PATH at $(command -v "$bin"))"
        PASS=$((PASS + 1))
    else
        # surreal-memory-server in Docker counts as present.
        if [ "$bin" = "surreal-memory-server" ] && \
           docker ps --format '{{.Names}}' 2>/dev/null | grep -q "surreal-memory"; then
            echo "  ✅ $bin running in Docker"
            PASS=$((PASS + 1))
        else
            echo "  ⚠️  $bin not found (optional)"
            SKIP=$((SKIP + 1))
        fi
    fi
}

echo ""
echo "  Required binaries (--version check):"
for b in "${required[@]}"; do check_bin "$b" required; done

if [ ${#optional[@]} -gt 0 ]; then
    echo ""
    echo "  Optional binaries (--version check):"
    for b in "${optional[@]}"; do check_bin "$b" optional; done
fi

echo ""
echo "  Optional binaries (presence only):"
for b in "${optional_presence_only[@]}"; do check_presence "$b"; done

test_forge_package_librefang() {
    local agent_dir="skills/rust/librefang-wasm-skill"
    local zip_out="/tmp/smoke-librefang-wasm-skill.lf-skill.zip"

    echo ""
    echo "  forge package-librefang pipeline:"

    if ! command -v forge >/dev/null 2>&1; then
        echo "    ⚠️  forge not on PATH — skipping package-librefang test"
        SKIP=$((SKIP + 1))
        return
    fi

    local wasm_path="${agent_dir}/target/wasm32-unknown-unknown/release/librefang_wasm_skill.wasm"
    if [[ ! -f "$wasm_path" ]]; then
        if [[ ! -f "${agent_dir}/Cargo.toml" ]]; then
            echo "    ⚠️  librefang WASM source is not included in this checkout — skipping"
            SKIP=$((SKIP + 1))
            return
        fi
        echo "    Building WASM (this may take a minute)..."
        if ! cargo build --manifest-path "${agent_dir}/Cargo.toml" \
                --release --target wasm32-unknown-unknown --quiet 2>/dev/null; then
            echo "    ⚠️  cargo build failed (wasm32 toolchain may not be installed) — skipping"
            SKIP=$((SKIP + 1))
            return
        fi
    fi

    rm -f "$zip_out"
    if ! forge package-librefang "$agent_dir" --no-build --output "$zip_out" >/dev/null 2>&1; then
        echo "    ❌ forge package-librefang exited non-zero"
        FAIL=$((FAIL + 1))
        return
    fi

    if ! unzip -l "$zip_out" 2>/dev/null | grep -q "skill.toml"; then
        echo "    ❌ skill.toml missing from produced zip"
        FAIL=$((FAIL + 1))
        rm -f "$zip_out"
        return
    fi
    if ! unzip -l "$zip_out" 2>/dev/null | grep -q "\.wasm"; then
        echo "    ❌ .wasm binary missing from produced zip"
        FAIL=$((FAIL + 1))
        rm -f "$zip_out"
        return
    fi
    echo "    ✅ forge package-librefang → zip with skill.toml + .wasm"
    PASS=$((PASS + 1))

    if command -v librefang >/dev/null 2>&1; then
        local tmp_skills_dir
        tmp_skills_dir=$(mktemp -d)
        if librefang skill install "$zip_out" --skills-dir "$tmp_skills_dir" --quiet 2>/dev/null; then
            if librefang skill info librefang-wasm-skill \
                    --skills-dir "$tmp_skills_dir" 2>/dev/null \
                    | grep -qi "runtime.*wasm\|type.*wasm"; then
                echo "    ✅ librefang install + runtime.type=wasm verified"
                PASS=$((PASS + 1))
            else
                echo "    ❌ runtime.type=wasm not found in librefang skill info"
                FAIL=$((FAIL + 1))
            fi
        else
            echo "    ⚠️  librefang skill install failed (may need config) — skipping"
            SKIP=$((SKIP + 1))
        fi
        rm -rf "$tmp_skills_dir"
    else
        echo "    ⚠️  librefang not on PATH — install verification skipped"
        SKIP=$((SKIP + 1))
    fi

    rm -f "$zip_out"
}

test_forge_package_librefang

# ── Platform installation checks ─────────────────────────────────────────────
test_platform_installs() {
    echo ""
    echo "  Platform skill installation:"

    # Claude Code
    if [[ -d "$HOME/.claude/skills" ]]; then
        PASS=$((PASS + 1))
        echo "    ✅ claude-code: ~/.claude/skills exists"
    else
        SKIP=$((SKIP + 1))
        echo "    ⚠️  claude-code: ~/.claude/skills not found — run: bash scripts/install-skills-flat.sh"
    fi

    # Kimi Code — skills dir
    if [[ -d "$HOME/.kimi-code/skills" ]]; then
        PASS=$((PASS + 1))
        echo "    ✅ kimi-code: ~/.kimi-code/skills exists"
    else
        SKIP=$((SKIP + 1))
        echo "    ⚠️  kimi-code: ~/.kimi-code/skills not found — run: bash scripts/install-skills-flat.sh"
    fi

    # Kimi Code — MCP config
    if grep -q "surreal-memory" "$HOME/.kimi-code/config.toml" 2>/dev/null; then
        PASS=$((PASS + 1))
        echo "    ✅ kimi-code: config.toml has surreal-memory MCP entry"
    else
        SKIP=$((SKIP + 1))
        echo "    ⚠️  kimi-code: config.toml missing surreal-memory — run: bash scripts/install-skills-flat.sh"
    fi

    # MiniMax — skills dir
    if [[ -d "$HOME/.minimax/skills" ]]; then
        PASS=$((PASS + 1))
        echo "    ✅ minimax: ~/.minimax/skills exists"
    else
        SKIP=$((SKIP + 1))
        echo "    ⚠️  minimax: ~/.minimax/skills not found — run: bash scripts/install-skills-flat.sh"
    fi

    # MiniMax — MCP config
    if [[ -f "$HOME/.minimax/mcp/mcp.json" ]]; then
        if node -e "const c=JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')); process.exit(c.mcpServers&&c.mcpServers['surreal-memory']?0:1);" "$HOME/.minimax/mcp/mcp.json" 2>/dev/null; then
            PASS=$((PASS + 1))
            echo "    ✅ minimax: mcp.json has surreal-memory entry"
        else
            SKIP=$((SKIP + 1))
            echo "    ⚠️  minimax: mcp.json missing surreal-memory — run: bash scripts/install-skills-flat.sh"
        fi
    else
        SKIP=$((SKIP + 1))
        echo "    ⚠️  minimax: mcp.json not found (MiniMax may not be installed)"
    fi
}

test_platform_installs

echo ""
echo "========================================"
echo "  Pass: $PASS  Fail: $FAIL  Skip: $SKIP"
if [ "$FAIL" -gt 0 ]; then
    echo "  ❌ Smoke test failed"
    echo "  Run: ./scripts/check-prerequisites.sh --install --build-tools"
    exit 1
fi
echo "✨ Smoke test passed"
