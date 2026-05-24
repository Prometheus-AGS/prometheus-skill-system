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

install_to_dir "claude-code" "$HOME/.claude/skills"
install_to_dir "opencode"    "$HOME/.opencode/skills"
install_to_dir "cursor"      "$HOME/.cursor/skills"
install_to_dir "codex"       "$HOME/.codex/skills"
install_to_dir "gemini"      "$HOME/.gemini/skills"
install_to_dir "roo"         "$HOME/.roo/skills"
install_to_dir "windsurf"    "$HOME/.windsurf/skills"
install_to_dir "windsurf-legacy" "$HOME/.codeium/windsurf/skills"
install_to_dir "amp"         "$HOME/.agents/skills"
install_to_dir "zed"         "$HOME/.config/zed/skills"
install_to_dir "antigravity" "$HOME/.zed/skills"
install_to_dir "cline"       "$HOME/.cline/skills"

echo ""
echo "✨ Done — skills available as slash commands on all platforms"
