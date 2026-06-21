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
            if [[ -L "$link" ]]; then
                local target
                target=$(resolve_link_target "$link")
                if [[ "$target" == "$REPO_ROOT"* ]]; then
                    rm "$link"
                    count=$((count + 1))
                fi
            fi
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
            fi
        fi
    done

    local action="installed"
    $UNINSTALL && action="removed"
    echo "  ✅ $platform_name: $count skills $action"
}

# MiniMax requires real directories (not symlinks) plus a _meta.json alongside SKILL.md.
# This function copies SKILL.md and writes _meta.json rather than symlinking.
# NOTE: minimax installs do not auto-update when skills change — re-run this script.
install_to_minimax() {
    local platform_name="minimax"
    local target_dir="$HOME/.minimax/skills"

    if [[ ! -d "$(dirname "$target_dir")" ]]; then
        echo "  — $platform_name: parent dir missing (~/.minimax not found), skipping"
        return
    fi

    mkdir -p "$target_dir"
    local count=0

    for entry in "${SKILLS[@]}"; do
        local skill_name="${entry%%|*}"
        local skill_dir="${entry#*|}"
        local dest="$target_dir/$skill_name"

        if $UNINSTALL; then
            # Only remove if it has our marker in _meta.json
            if [[ -d "$dest" ]] && [[ -f "$dest/_meta.json" ]]; then
                if grep -q '"platform": "minimax"' "$dest/_meta.json" 2>/dev/null; then
                    rm -rf "$dest"
                    count=$((count + 1))
                fi
            fi
        else
            mkdir -p "$dest"
            cp "$skill_dir/SKILL.md" "$dest/SKILL.md"
            # Derive a deterministic numeric id from a CRC32-like hash of skill name
            local skill_id
            skill_id=$(printf '%d' "0x$(echo -n "$skill_name" | md5sum | cut -c1-8)" 2>/dev/null || echo "0")
            local version
            version=$(node -e "const fs=require('fs'); const s=fs.readFileSync(process.argv[1],'utf8'); const m=s.match(/^---\\n([\\s\\S]*?)\\n---/); const v=m&&m[1].match(/^version:\\s*['\\\"]?([^'\\\"\\n]+)['\\\"]?/m); console.log((v&&v[1].trim())||'1.0.0');" "$skill_dir/SKILL.md" 2>/dev/null || echo "1.0.0")
            local ts
            ts=$(date +%s)000
            cat > "$dest/_meta.json" <<META
{
  "id": $skill_id,
  "version": "$version",
  "name": "$skill_name",
  "updated_at": $ts,
  "platform": "minimax"
}
META
            count=$((count + 1))
        fi
    done

    local action="installed"
    $UNINSTALL && action="removed"
    echo "  ✅ $platform_name: $count skills $action (copies + _meta.json)"
}

install_to_dir "claude-code"     "$HOME/.claude/skills"
install_to_dir "opencode"        "$HOME/.opencode/skills"
install_to_dir "kimi-code"       "$HOME/.kimi-code/skills"
install_to_minimax
install_to_dir "cursor"          "$HOME/.cursor/skills"
install_to_dir "codex"           "$HOME/.codex/skills"
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

echo ""
echo "✨ Done — skills available as slash commands on all platforms"
