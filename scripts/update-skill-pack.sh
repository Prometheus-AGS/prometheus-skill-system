#!/usr/bin/env bash
# Update prometheus-skill-pack and delta-install changed skills to all platforms.
#
# Usage:
#   ./scripts/update-skill-pack.sh          # pull + delta install
#   ./scripts/update-skill-pack.sh --force  # reinstall all (ignore delta)
#
# Stores the last-installed git SHA at ~/.prometheus/skill-pack-install-ref
# so only skills changed since that commit are re-installed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_REF_FILE="${HOME}/.prometheus/skill-pack-install-ref"
FORCE=false

if [[ "${1:-}" == "--force" ]]; then
    FORCE=true
fi

echo "Prometheus Skill Pack — Update"
echo "==============================="

# 1. Pull latest (non-destructive)
echo ""
echo "Step 1: Pulling latest changes..."
if ! git -C "$REPO_ROOT" pull --ff-only 2>&1; then
    echo ""
    echo "ERROR: git pull --ff-only failed. This means your local branch has diverged."
    echo "Resolve the conflict manually, then re-run this script."
    exit 1
fi

# Update submodules
git -C "$REPO_ROOT" submodule update --remote --merge 2>&1 | grep -v "^$" || true

# 2. Determine delta
CURRENT_SHA=$(git -C "$REPO_ROOT" rev-parse HEAD)

if $FORCE; then
    echo ""
    echo "Step 2: Force mode — reinstalling all skills"
    CHANGED_SKILLS=()
    while IFS= read -r -d '' skill_md; do
        CHANGED_SKILLS+=("$(dirname "$skill_md")")
    done < <(find "$REPO_ROOT/skills" -name "SKILL.md" -not -path "*/imported/*" -print0)
else
    if [[ -f "$INSTALL_REF_FILE" ]]; then
        PREV_SHA=$(cat "$INSTALL_REF_FILE")
        echo ""
        echo "Step 2: Computing delta since $PREV_SHA..."
        CHANGED_FILES=$(git -C "$REPO_ROOT" diff --name-only "$PREV_SHA" "$CURRENT_SHA" 2>/dev/null || echo "")
        CHANGED_SKILLS=()
        while IFS= read -r changed_file; do
            # If any file under a skill dir changed, re-install that skill
            skill_dir=$(echo "$changed_file" | grep -oE "^skills/[^/]+/[^/]+" || true)
            if [[ -n "$skill_dir" && -f "$REPO_ROOT/$skill_dir/SKILL.md" ]]; then
                # avoid duplicates
                if [[ ! " ${CHANGED_SKILLS[*]:-} " =~ " $REPO_ROOT/$skill_dir " ]]; then
                    CHANGED_SKILLS+=("$REPO_ROOT/$skill_dir")
                fi
            fi
        done <<< "$CHANGED_FILES"
    else
        echo ""
        echo "Step 2: No previous install reference found — performing full install"
        FORCE=true
        CHANGED_SKILLS=()
        while IFS= read -r -d '' skill_md; do
            CHANGED_SKILLS+=("$(dirname "$skill_md")")
        done < <(find "$REPO_ROOT/skills" -name "SKILL.md" -not -path "*/imported/*" -print0)
    fi
fi

CHANGED_COUNT=${#CHANGED_SKILLS[@]}

if [[ $CHANGED_COUNT -eq 0 ]]; then
    echo ""
    echo "0 skills changed since last install — nothing to do."
    echo "Install ref: $CURRENT_SHA"
    # Update ref even if nothing changed (tracks the pull)
    mkdir -p "$(dirname "$INSTALL_REF_FILE")"
    echo "$CURRENT_SHA" > "$INSTALL_REF_FILE"
    exit 0
fi

echo "  $CHANGED_COUNT skill(s) to update"

# 3. Install changed skills to each platform
HOME="${HOME}"
declare -A PLATFORM_DIRS=(
    ["claude-code"]="$HOME/.claude/skills"
    ["opencode"]="$HOME/.opencode/skills"
    ["kimi-code"]="$HOME/.kimi-code/skills"
    ["cursor"]="$HOME/.cursor/skills"
    ["codex"]="$HOME/.codex/skills"
    ["gemini"]="$HOME/.gemini/skills"
    ["roo"]="$HOME/.roo/skills"
    ["windsurf"]="$HOME/.windsurf/skills"
    ["windsurf-legacy"]="$HOME/.codeium/windsurf/skills"
    ["amp"]="$HOME/.agents/skills"
    ["zed"]="$HOME/.config/zed/skills"
    ["cline"]="$HOME/.cline/skills"
)

MINIMAX_DIR="$HOME/.minimax/skills"
PLATFORMS_UPDATED=()

echo ""
echo "Step 3: Delta-installing ${CHANGED_COUNT} skill(s)..."
echo ""

for skill_dir in "${CHANGED_SKILLS[@]}"; do
    skill_md="$skill_dir/SKILL.md"
    if [[ ! -f "$skill_md" ]]; then
        continue
    fi

    # Extract skill name
    skill_name=$(node -e "
        const fs=require('fs');
        const s=fs.readFileSync(process.argv[1],'utf8');
        const m=s.match(/^---\\n([\\s\\S]*?)\\n---/);
        const n=m&&m[1].match(/^name:\\s*['\"]?([^'\"\\n]+)['\"]?/m);
        console.log((n&&n[1].trim())||require('path').basename(require('path').dirname(process.argv[1])));
    " "$skill_md")

    echo "  Updating: $skill_name"

    # Install to symlink-based platforms
    for platform in "${!PLATFORM_DIRS[@]}"; do
        target_dir="${PLATFORM_DIRS[$platform]}"
        if [[ ! -d "$target_dir" ]]; then
            continue
        fi
        link="$target_dir/$skill_name"
        rm -f "$link"
        ln -s "$skill_dir" "$link"
        if [[ ! " ${PLATFORMS_UPDATED[*]:-} " =~ " $platform " ]]; then
            PLATFORMS_UPDATED+=("$platform")
        fi
    done

    # MiniMax: copy (not symlink)
    if [[ -d "$MINIMAX_DIR" ]]; then
        minimax_skill_dir="$MINIMAX_DIR/$skill_name"
        rm -rf "$minimax_skill_dir"
        cp -r "$skill_dir" "$minimax_skill_dir"
        # Regenerate _meta.json
        updated_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        skill_id=$(echo "$skill_name" | md5 | cut -c1-8 2>/dev/null || echo "$skill_name" | md5sum | cut -c1-8)
        version=$(node -e "
            const fs=require('fs');
            const s=fs.readFileSync(process.argv[1],'utf8');
            const m=s.match(/^---\\n([\\s\\S]*?)\\n---/);
            const n=m&&m[1].match(/^version:\\s*['\"]?([^'\"\\n]+)['\"]?/m);
            console.log((n&&n[1].trim())||'1.0.0');
        " "$skill_md")
        cat > "$minimax_skill_dir/_meta.json" <<EOF
{
  "id": "$skill_id",
  "version": "$version",
  "name": "$skill_name",
  "updated_at": "$updated_at",
  "platform": "minimax"
}
EOF
        if [[ ! " ${PLATFORMS_UPDATED[*]:-} " =~ " minimax " ]]; then
            PLATFORMS_UPDATED+=("minimax")
        fi
    fi
done

# 4. Update install reference
mkdir -p "$(dirname "$INSTALL_REF_FILE")"
echo "$CURRENT_SHA" > "$INSTALL_REF_FILE"

echo ""
echo "Done."
echo "  Skills updated: $CHANGED_COUNT"
echo "  Platforms refreshed: ${#PLATFORMS_UPDATED[@]} (${PLATFORMS_UPDATED[*]:-none})"
echo "  Install ref: $CURRENT_SHA"
