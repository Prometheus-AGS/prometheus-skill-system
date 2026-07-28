#!/usr/bin/env bash
# Install prometheus-skill-pack skills as flat symlinks into platform skill directories.
# This makes each skill appear as a slash command (e.g., /evolve, /kbd-plan, /gitops-bootstrap).
#
# Usage:
#   ./scripts/install-skills-flat.sh              # all platforms
#   ./scripts/install-skills-flat.sh --uninstall   # remove all

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNINSTALL=false

if [[ "${1:-}" == "--uninstall" ]]; then
    UNINSTALL=true
fi

echo "🔥 Prometheus Skill Pack — Flat Skill Installation"
echo "=================================================="

if $UNINSTALL; then
    echo "  Mode: UNINSTALL"
else
    echo "  Mode: INSTALL"
fi
echo ""

# Collect all skill directories
SKILLS=()
while IFS= read -r -d '' skill_md; do
    skill_dir=$(dirname "$skill_md")
    skill_name=$(node -e "const fs=require('fs'); const s=fs.readFileSync(process.argv[1],'utf8'); const m=s.match(/^---\\n([\\s\\S]*?)\\n---/); const n=m&&m[1].match(/^name:\\s*['\\\"]?([^'\\\"\\n]+)['\\\"]?/m); console.log((n&&n[1].trim())||require('path').basename(require('path').dirname(process.argv[1])));" "$skill_md")
    SKILLS+=("$skill_name|$skill_dir")
done < <(find "$REPO_ROOT/skills" -name "SKILL.md" -not -path "*/imported/*" -print0)

echo "  Found ${#SKILLS[@]} skills"
echo ""

resolve_link_target() {
    local link="$1"
    python3 - "$link" <<'PY'
import os
import sys

link = sys.argv[1]
target = os.readlink(link)
if not os.path.isabs(target):
    target = os.path.join(os.path.dirname(link), target)
print(os.path.realpath(target))
PY
}

install_to_dir() {
    local platform_name="$1"
    local target_dir="$2"

    if [[ ! -d "$(dirname "$target_dir")" ]]; then
        echo "  — $platform_name: parent dir missing, skipping"
        return
    fi

    mkdir -p "$target_dir"
    local count=0

    for entry in "${SKILLS[@]}"; do
        local skill_name="${entry%%|*}"
        local skill_dir="${entry#*|}"
        local link="$target_dir/$skill_name"

        if $UNINSTALL; then
            local fallback="$target_dir/prometheus-$skill_name"
            for managed_link in "$link" "$fallback"; do
                if [[ ! -L "$managed_link" ]]; then
                    continue
                fi
                local target
                target=$(resolve_link_target "$managed_link")
                if [[ "$target" == "$REPO_ROOT"* ]]; then
                    rm "$managed_link"
                    count=$((count + 1))
                fi
            done
        else
            # Remove existing symlink if it points to our repo
            if [[ -L "$link" ]]; then
                local target
                target=$(resolve_link_target "$link")
                if [[ "$target" == "$REPO_ROOT"* ]]; then
                    rm "$link"
                fi
            fi
            # Only create if nothing else is there
            if [[ ! -e "$link" ]]; then
                ln -s "$skill_dir" "$link"
                count=$((count + 1))
            elif [[ ! -L "$link" ]] || [[ "$(resolve_link_target "$link" 2>/dev/null || true)" != "$skill_dir" ]]; then
                # Preserve user-owned collisions while still installing the pack
                # payload under a deterministic namespace.
                local fallback="$target_dir/prometheus-$skill_name"
                if [[ -L "$fallback" ]]; then
                    local fallback_target
                    fallback_target=$(resolve_link_target "$fallback")
                    [[ "$fallback_target" == "$REPO_ROOT"* ]] && rm "$fallback"
                fi
                if [[ ! -e "$fallback" ]]; then
                    ln -s "$skill_dir" "$fallback"
                    count=$((count + 1))
                    echo "  ⚠️  $platform_name: preserved collision at $link; installed pack payload as prometheus-$skill_name"
                fi
            fi
        fi
    done

    local action="installed"
    $UNINSTALL && action="removed"
    echo "  ✅ $platform_name: $count skills $action"
}

# Codex CLI (0.144.x) does not traverse symlinked skill directories — a symlink in
# ~/.codex/skills contributes zero skills to the catalog, silently. Codex also budgets
# the whole catalog, so the set it gets is curated in config/codex-catalog.txt.
# Delegated to codex-sync-skills.sh, which is also run on codex session_start.
install_to_codex() {
    if $UNINSTALL; then
        bash "$REPO_ROOT/scripts/codex-sync-skills.sh" --uninstall
    else
        bash "$REPO_ROOT/scripts/codex-sync-skills.sh"
    fi

    # Because Codex gets copies rather than symlinks, they go stale when a skill is
    # edited. A WatchPaths LaunchAgent re-syncs on any change under skills/.
    [[ "$(uname -s)" == "Darwin" ]] || return 0
    local label="ai.prometheus.codex-skills-sync"
    local dst="$HOME/Library/LaunchAgents/$label.plist"

    if $UNINSTALL; then
        launchctl unload "$dst" 2>/dev/null || true
        rm -f "$dst"
        echo "  ✅ codex: skills-sync agent removed"
        return 0
    fi

    mkdir -p "$HOME/Library/LaunchAgents" "$HOME/.prometheus/logs"
    sed -e "s|__REPO_ROOT__|$REPO_ROOT|g" -e "s|__HOME__|$HOME|g" \
        "$REPO_ROOT/shared/launchagents/$label.plist" > "$dst"
    launchctl unload "$dst" 2>/dev/null || true
    if launchctl load -w "$dst" 2>/dev/null; then
        echo "  ✅ codex: skills-sync agent loaded (re-syncs on skill edits)"
    else
        echo "  ⚠️  codex: skills-sync agent failed to load — run 'bash scripts/codex-sync-skills.sh' manually after editing skills"
    fi
}

# MiniMax requires real directories (not symlinks) plus a _meta.json alongside SKILL.md.
# Delegate to the canonical copier so scripts, references, templates, assets, and
# executable modes are preserved exactly instead of copying only SKILL.md.
# NOTE: minimax installs do not auto-update when skills change — re-run this script.
install_to_minimax() {
    local platform_name="minimax"
    local target_dir="$HOME/.minimax/skills"

    if [[ ! -d "$(dirname "$target_dir")" ]]; then
        echo "  — $platform_name: parent dir missing (~/.minimax not found), skipping"
        return
    fi

    local args=(--repo-root "$REPO_ROOT" --target-dir "$target_dir")
    $UNINSTALL && args+=(--uninstall)
    node "$REPO_ROOT/scripts/install-minimax-skills.js" "${args[@]}"
}

install_to_dir "claude-code"     "$HOME/.claude/skills"
install_to_dir "opencode"        "$HOME/.opencode/skills"
install_to_dir "kimi-code"       "$HOME/.kimi-code/skills"
install_to_minimax
install_to_dir "cursor"          "$HOME/.cursor/skills"
install_to_codex
install_to_dir "gemini"          "$HOME/.gemini/skills"
install_to_dir "roo"             "$HOME/.roo/skills"
install_to_dir "windsurf"        "$HOME/.windsurf/skills"
install_to_dir "windsurf-legacy" "$HOME/.codeium/windsurf/skills"
install_to_dir "amp"             "$HOME/.agents/skills"
install_to_dir "zed"             "$HOME/.config/zed/skills"
install_to_dir "antigravity"     "$HOME/.zed/skills"
install_to_dir "cline"           "$HOME/.cline/skills"

# Post-install: configure Kimi Code MCP servers if config.toml is absent
configure_kimi_mcp() {
    local kimi_dir="$HOME/.kimi-code"
    local config_file="$kimi_dir/config.toml"

    if [[ ! -d "$kimi_dir" ]]; then
        return
    fi

    local needs_write=false
    local existing=""
    [[ -f "$config_file" ]] && existing=$(cat "$config_file")

    local new_entries=""

    if ! grep -q "\[mcp_servers.surreal-memory\]" "$config_file" 2>/dev/null; then
        new_entries+=$'\n'"[mcp_servers.surreal-memory]"$'\n'"type = \"sse\""$'\n'"url = \"http://localhost:23001/mcp/sse\""
        needs_write=true
    fi
    if ! grep -q "\[mcp_servers.sycophancy-correction\]" "$config_file" 2>/dev/null; then
        new_entries+=$'\n\n'"[mcp_servers.sycophancy-correction]"$'\n'"command = \"sycophancy-correction\""
        needs_write=true
    fi
    if ! grep -q "\[mcp_servers.liter-llm\]" "$config_file" 2>/dev/null; then
        new_entries+=$'\n\n'"[mcp_servers.liter-llm]"$'\n'"command = \"liter-llm\""$'\n'"args = [\"mcp\", \"--transport\", \"stdio\"]"
        needs_write=true
    fi

    if $needs_write && ! $UNINSTALL; then
        [[ -f "$config_file" ]] && cp "$config_file" "${config_file}.bak.$(date +%Y%m%d%H%M%S)"
        printf '%s%s\n' "$existing" "$new_entries" > "$config_file"
        echo "  ✅ kimi-code: config.toml updated with MCP server entries"
    fi
}

configure_kimi_mcp

if [[ "${PROMETHEUS_INSTALL_SKILLS_ONLY:-0}" == "1" ]]; then
    echo ""
    echo "✨ Done — skills-only installation completed"
    exit 0
fi

# Post-install: install OpenCode goal plugin and write KBD-tuned config
configure_opencode_goal_plugin() {
    if $UNINSTALL; then
        return
    fi

    if ! command -v opencode &>/dev/null; then
        return
    fi

    # OpenCode's plugin subcommand takes a module name directly. The package
    # does not expose an npx executable, and `opencode plugins list` is not a
    # valid command in current OpenCode releases.
    local plugin_installed=false
    if node - "$HOME/.config/opencode/opencode.json" "$HOME/.config/opencode/tui.json" <<'JS'
const fs = require('fs');
const wanted = '@prevalentware/opencode-goal-plugin';
const found = process.argv.slice(2).some((path) => {
  if (!fs.existsSync(path)) return false;
  try {
    const plugins = JSON.parse(fs.readFileSync(path, 'utf8')).plugin || [];
    return plugins.some((entry) => entry === wanted || (Array.isArray(entry) && entry[0] === wanted));
  } catch {
    return false;
  }
});
process.exit(found ? 0 : 1);
JS
    then
        plugin_installed=true
    fi

    if ! $plugin_installed; then
        echo "  ⚙️  opencode: installing @prevalentware/opencode-goal-plugin..."
        opencode plugin -g @prevalentware/opencode-goal-plugin || {
            echo "  ⚠️  opencode: goal plugin install failed; run manually: opencode plugin -g @prevalentware/opencode-goal-plugin"
            return
        }
        echo "  ✅ opencode: goal plugin installed in global server and TUI configs"
    else
        echo "  ✅ opencode: goal plugin already configured"
    fi
}

configure_opencode_goal_plugin

# Post-install: register surreal-memory in MiniMax MCP config
configure_minimax_mcp() {
    local mcp_file="$HOME/.minimax/mcp/mcp.json"

    if [[ ! -f "$mcp_file" ]]; then
        return
    fi

    if $UNINSTALL; then
        return
    fi

    # Merge our MCP entries using node (already required for skill name extraction)
    # When using 'node - <file>', argv[0]=node, argv[1]='-', argv[2]=the file
    node - "$mcp_file" <<'JS'
const fs = require('fs');
const path = process.argv[2];

let config = {};
try { config = JSON.parse(fs.readFileSync(path, 'utf8')); } catch(e) { config = {}; }
if (!config.mcpServers) config.mcpServers = {};

let changed = false;
if (!config.mcpServers['surreal-memory']) {
    config.mcpServers['surreal-memory'] = {
        type: 'sse',
        url: 'http://localhost:23001/mcp/sse',
        enabled: true,
        configured: true,
        description: 'Surreal-memory distributed knowledge graph and task tracking'
    };
    changed = true;
}
if (!config.mcpServers['sycophancy-correction']) {
    config.mcpServers['sycophancy-correction'] = {
        command: 'sycophancy-correction',
        enabled: true,
        configured: true
    };
    changed = true;
}

if (changed) {
    const backup = path + '.bak.' + new Date().toISOString().replace(/[-:T]/g,'').slice(0,14);
    fs.copyFileSync(path, backup);
    fs.writeFileSync(path, JSON.stringify(config, null, 2) + '\n', 'utf8');
    console.log('  ✅ minimax: mcp.json updated with surreal-memory + sycophancy-correction');
} else {
    console.log('  ✅ minimax: mcp.json already contains required entries');
}
JS
}

configure_minimax_mcp

# Build and install learn-domain substrate crates
install_learn_substrate() {
    if $UNINSTALL; then
        return
    fi

    if ! command -v cargo &>/dev/null; then
        echo "  — learn-substrate: cargo not found, skipping Rust substrate builds"
        return
    fi

    local substrate_dir="$REPO_ROOT/substrate"

    # Build storage-provider (library only, no binary)
    echo ""
    echo "  Building substrate/storage-provider..."
    if cargo build --release --manifest-path "$substrate_dir/storage-provider/Cargo.toml" 2>/dev/null; then
        echo "  ✅ learn-substrate: storage-provider built"
    else
        echo "  ⚠️  learn-substrate: storage-provider build failed (non-fatal)"
    fi

    # Build learner-model (binary + lib)
    echo "  Building substrate/learner-model..."
    if cargo build --release --manifest-path "$substrate_dir/learner-model/Cargo.toml" 2>/dev/null; then
        echo "  ✅ learn-substrate: learner-model built"
        # Install binary to ~/.local/bin
        local bin_dir="$HOME/.local/bin"
        mkdir -p "$bin_dir"
        cp -f "$substrate_dir/learner-model/target/release/learner-model" "$bin_dir/learner-model" 2>/dev/null && \
            echo "  ✅ learn-substrate: learner-model installed to $bin_dir/learner-model" || true
    else
        echo "  ⚠️  learn-substrate: learner-model build failed (non-fatal)"
    fi

    # Build surface-bridge (Axum MCP App server)
    echo "  Building substrate/surface-bridge..."
    if cargo build --release --manifest-path "$substrate_dir/surface-bridge/Cargo.toml" 2>/dev/null; then
        echo "  ✅ learn-substrate: surface-bridge built"
        local bin_dir="$HOME/.local/bin"
        mkdir -p "$bin_dir"
        # -f: if the destination is a currently-running binary (e.g. the
        # live surface-bridge daemon), a plain cp fails with "Text file
        # busy"; -f unlinks and recreates instead of writing in place.
        cp -f "$substrate_dir/surface-bridge/target/release/surface-bridge" "$bin_dir/surface-bridge" 2>/dev/null || true
        echo "  ℹ️  learn-substrate: run 'npm run install:daemons' to install/start the surface-bridge service (macOS + Linux)"
    else
        echo "  ⚠️  learn-substrate: surface-bridge build failed (non-fatal)"
    fi

    # Build sovereign-sync (P2P CRDT daemon + MCP server)
    echo "  Building substrate/sovereign-sync..."
    if cargo build --release --manifest-path "$substrate_dir/sovereign-sync/Cargo.toml" 2>/dev/null; then
        echo "  ✅ learn-substrate: sovereign-sync built"
        local bin_dir="$HOME/.local/bin"
        mkdir -p "$bin_dir"
        cp -f "$substrate_dir/sovereign-sync/target/release/sovereign-sync" "$bin_dir/sovereign-sync" 2>/dev/null && \
            echo "  ✅ learn-substrate: sovereign-sync installed to $bin_dir/sovereign-sync" || true
    else
        echo "  ⚠️  learn-substrate: sovereign-sync build failed (non-fatal)"
    fi

    echo "  ℹ️  learn-substrate: run 'npm run install:daemons' to install/start sovereign-sync (port 7892)"

    # Register sovereign-sync as MCP server in Claude Code config
    local claude_mcp="$HOME/.claude/mcp-servers.json"
    if [[ -f "$claude_mcp" ]]; then
        node - "$claude_mcp" <<'JS'
const fs = require('fs');
const path = process.argv[1];
const config = JSON.parse(fs.readFileSync(path, 'utf8'));
const entry = {
    command: process.env.HOME + '/.local/bin/sovereign-sync',
    args: ['--mode', 'mcp'],
    env: { RUST_LOG: 'sovereign_sync=warn' }
};
let changed = false;
if (!config['sovereign-sync']) { config['sovereign-sync'] = entry; changed = true; }
if (changed) {
    const bak = path + '.bak.' + Date.now();
    fs.copyFileSync(path, bak);
    fs.writeFileSync(path, JSON.stringify(config, null, 2) + '\n', 'utf8');
    console.log('  ✅ claude: mcp-servers.json updated with sovereign-sync');
} else {
    console.log('  ✅ claude: sovereign-sync already in mcp-servers.json');
}
JS
    fi
}

install_learn_substrate

# Post-install: configure all Prometheus MCP servers across all AI tools
if [[ -f "$REPO_ROOT/scripts/configure-mcp-all-tools.sh" ]] && ! $UNINSTALL; then
    echo ""
    echo "Configuring Prometheus MCP servers across all AI tools..."
    bash "$REPO_ROOT/scripts/configure-mcp-all-tools.sh" 2>/dev/null || true
fi

echo ""
echo "✨ Done — skills available as slash commands on all platforms"
