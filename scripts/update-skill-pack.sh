#!/usr/bin/env bash
# Update prometheus-skill-pack and delta-install changed skills to all platforms.
#
# Usage:
#   ./scripts/update-skill-pack.sh          # pull + delta install
#   ./scripts/update-skill-pack.sh --force  # reinstall all (ignore delta)
#   ./scripts/update-skill-pack.sh --doctor-refresh  # after a clean update, run prometheus doctor --refresh --yes
#
# Stores the last-installed git SHA at ~/.prometheus/skill-pack-install-ref
# so only skills changed since that commit are re-installed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_REF_FILE="${HOME}/.prometheus/skill-pack-install-ref"
FORCE=false
RUN_DOCTOR_REFRESH=false

for arg in "$@"; do
    case "$arg" in
        --force) FORCE=true ;;
        --doctor-refresh) RUN_DOCTOR_REFRESH=true ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

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

collect_all_skills() {
    while IFS= read -r -d '' skill_md; do
        CHANGED_SKILLS+=("$(dirname "$skill_md")")
    done < <(find "$REPO_ROOT/skills" -name "SKILL.md" -not -path "*/imported/*" -print0)
}

if $FORCE; then
    echo ""
    echo "Step 2: Force mode — reinstalling all skills"
    CHANGED_SKILLS=()
    collect_all_skills
else
    if [[ -f "$INSTALL_REF_FILE" ]]; then
        PREV_SHA=$(cat "$INSTALL_REF_FILE")
        echo ""
        echo "Step 2: Computing delta since $PREV_SHA..."
        CHANGED_FILES=$(git -C "$REPO_ROOT" diff --name-only "$PREV_SHA" "$CURRENT_SHA" 2>/dev/null || echo "")
        CHANGED_SKILLS=()

        # Installer/catalog changes affect distribution semantics globally.
        # Reinstall every skill so copied platforms cannot retain stale payloads.
        if grep -qE '^(scripts/(install-skills-flat\.sh|install-minimax-skills\.js|install-platforms\.ts|codex-sync-skills\.sh)|config/codex-catalog\.txt)$' <<< "$CHANGED_FILES"; then
            echo "  Installation infrastructure changed — promoting to full skill refresh"
            FORCE=true
            collect_all_skills
        else
            while IFS= read -r changed_file; do
                [[ "$changed_file" == skills/* ]] || continue
                candidate="$REPO_ROOT/$(dirname "$changed_file")"

                # A nested skill is installed both as its own flat command and as
                # part of every ancestor bundle. Refresh all skill ancestors.
                while [[ "$candidate" == "$REPO_ROOT/skills/"* ]]; do
                    if [[ -f "$candidate/SKILL.md" ]] && [[ ! " ${CHANGED_SKILLS[*]:-} " =~ " $candidate " ]]; then
                        CHANGED_SKILLS+=("$candidate")
                    fi
                    candidate=$(dirname "$candidate")
                done
            done <<< "$CHANGED_FILES"
        fi
    else
        echo ""
        echo "Step 2: No previous install reference found — performing full install"
        FORCE=true
        CHANGED_SKILLS=()
        collect_all_skills
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
    ["gemini"]="$HOME/.gemini/skills"
    ["roo"]="$HOME/.roo/skills"
    ["windsurf"]="$HOME/.windsurf/skills"
    ["windsurf-legacy"]="$HOME/.codeium/windsurf/skills"
    ["amp"]="$HOME/.agents/skills"
    ["zed"]="$HOME/.config/zed/skills"
    ["antigravity"]="$HOME/.zed/skills"
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
        if [[ -L "$link" ]]; then
            target=$(readlink "$link")
            if [[ "$target" == "$REPO_ROOT"* ]]; then
                rm -f "$link"
            else
                fallback="$target_dir/prometheus-$skill_name"
                if [[ -L "$fallback" ]] && [[ "$(readlink "$fallback")" == "$REPO_ROOT"* ]]; then
                    rm -f "$fallback"
                fi
                if [[ ! -e "$fallback" ]]; then
                    ln -s "$skill_dir" "$fallback"
                fi
                echo "    ⚠️  $platform: preserved $link; refreshed namespaced pack fallback"
                if [[ ! " ${PLATFORMS_UPDATED[*]:-} " =~ " $platform " ]]; then
                    PLATFORMS_UPDATED+=("$platform")
                fi
                continue
            fi
        elif [[ -e "$link" ]]; then
            fallback="$target_dir/prometheus-$skill_name"
            if [[ -L "$fallback" ]] && [[ "$(readlink "$fallback")" == "$REPO_ROOT"* ]]; then
                rm -f "$fallback"
            fi
            if [[ ! -e "$fallback" ]]; then
                ln -s "$skill_dir" "$fallback"
            fi
            echo "    ⚠️  $platform: preserved $link; refreshed namespaced pack fallback"
            if [[ ! " ${PLATFORMS_UPDATED[*]:-} " =~ " $platform " ]]; then
                PLATFORMS_UPDATED+=("$platform")
            fi
            continue
        fi
        ln -s "$skill_dir" "$link"
        if [[ ! " ${PLATFORMS_UPDATED[*]:-} " =~ " $platform " ]]; then
            PLATFORMS_UPDATED+=("$platform")
        fi
    done

done

# Copy-based platforms are refreshed through their canonical installers so
# nested resources, executable bits, stale-file pruning, and catalog policy are
# applied consistently.
if [[ -d "$MINIMAX_DIR" ]]; then
    node "$REPO_ROOT/scripts/install-minimax-skills.js" \
        --repo-root "$REPO_ROOT" --target-dir "$MINIMAX_DIR"
    PLATFORMS_UPDATED+=("minimax")
fi

if [[ -d "$HOME/.codex" ]]; then
    bash "$REPO_ROOT/scripts/codex-sync-skills.sh" --quiet
    PLATFORMS_UPDATED+=("codex")
fi

# 3b. Refresh native plugin caches.
#
# Platform skill dirs (above) and plugin CACHES are different things. The caches
# under ~/.claude/plugins/cache/... are versioned by the plugin version, so a
# same-version edit is NOT picked up — the harness keeps serving the stale copy.
# That is the mechanism by which a previous session's "fix" (editing scripts
# directly in the cache) both worked temporarily and then silently evaporated.
# Refresh them here so the repo is always the single source of truth.
for _cache_base in "$HOME/.claude/plugins/cache/prometheus-skill-pack/prometheus-skill-pack" \
                   "$HOME/.codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack"; do
    [ -d "$_cache_base" ] || continue
    for _ver in "$_cache_base"/*/; do
        [ -d "$_ver" ] || continue
        if [ -d "${_ver}skills" ]; then
            rsync -a --delete "$REPO_ROOT/skills/" "${_ver}skills/" 2>/dev/null || true
        fi
        if [ -d "${_ver}shared" ]; then
            rsync -a "$REPO_ROOT/shared/" "${_ver}shared/" 2>/dev/null || true
        fi
        echo "  ✅ refreshed plugin cache: ${_ver}"
    done
done

# 4. Update install reference
mkdir -p "$(dirname "$INSTALL_REF_FILE")"
echo "$CURRENT_SHA" > "$INSTALL_REF_FILE"

echo ""
echo "Done."
echo "  Skills updated: $CHANGED_COUNT"
echo "  Platforms refreshed: ${#PLATFORMS_UPDATED[@]} (${PLATFORMS_UPDATED[*]:-none})"
echo "  Install ref: $CURRENT_SHA"

echo ""
echo "Step 4: Doctor refresh handoff..."
if ! command -v prometheus >/dev/null 2>&1; then
    echo "  ○ prometheus CLI not found on PATH — run 'bash scripts/install-binaries.sh' first."
    exit 0
fi

if [[ -n "$(git -C "$REPO_ROOT" status --short --untracked-files=no)" ]]; then
    echo "  ○ Skipping doctor refresh because the checkout is dirty."
    echo "    Offer: after reviewing local changes, run:"
    echo "      prometheus doctor --refresh --yes"
    exit 0
fi

if $RUN_DOCTOR_REFRESH; then
    echo "  → Running prometheus doctor --refresh --yes"
    if prometheus doctor --refresh --yes; then
        echo "  ✅ doctor refresh completed"
        echo "  → Rescanning with prometheus doctor --json"
        prometheus doctor --json >/dev/null
        echo "  ✅ doctor rescan completed"
    else
        echo "  ❌ doctor refresh reported unresolved issues"
        exit 1
    fi
else
    echo "  ○ Offer: run 'prometheus doctor --refresh --yes' to rebuild managed components and rescan health."
fi
