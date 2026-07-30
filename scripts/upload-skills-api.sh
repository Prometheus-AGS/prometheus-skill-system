#!/usr/bin/env bash
# Upload prometheus-skill-pack skills to claude.ai / Claude Desktop via the
# Skills API (POST /v1/skills). This is a genuinely different mechanism from
# install-skills-flat.sh: that script symlinks into ~/.claude/skills for
# Claude Code's filesystem-based discovery. This script zips each skill and
# POSTs it to Anthropic's workspace-scoped Skills API.
#
# WHAT THIS DOES NOT RESOLVE: whether workspace-uploaded skills surface
# inside the claude.ai / Desktop chat UI itself, versus only being available
# to your own Messages API calls via the `container.skills` parameter. Test
# with ONE skill first (see --only below) and confirm it appears where you
# expect before running the full batch.
#
# WHAT THIS DELIBERATELY SKIPS BY DEFAULT: skills whose SKILL.md references
# local-only infrastructure (surreal-memory MCP server, liter-llm's stdio
# MCP transport, localhost health checks, harness registration). Those run
# fine under Claude Code (execution happens on your Mac) but Desktop's
# Custom Skills execute in Anthropic's own sandboxed container, which
# cannot reach localhost:23001 or any other service on your machine. Use
# --include-local-dependent to upload them anyway (they'll just run
# degraded, per the pack's own "degrades gracefully" design).
#
# Usage:
#   export ANTHROPIC_API_KEY=sk-ant-...        # must already be set in your shell
#   bash scripts/upload-skills-api.sh --dry-run                # list only, no upload
#   bash scripts/upload-skills-api.sh --only clean-architecture # test one skill
#   bash scripts/upload-skills-api.sh                           # upload the safe set
#   bash scripts/upload-skills-api.sh --include-local-dependent # upload everything

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API_VERSION="2023-06-01"
API_BETA="skills-2025-10-02"
DRY_RUN=false
INCLUDE_LOCAL=false
ONLY=""

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --include-local-dependent) INCLUDE_LOCAL=true ;;
        --only=*) ONLY="${arg#--only=}" ;;
        --only) shift ;; # handled below if passed as separate token
    esac
done
# support `--only NAME` as two tokens too
prev=""
for arg in "$@"; do
    if [[ "$prev" == "--only" ]]; then ONLY="$arg"; fi
    prev="$arg"
done

if [[ -z "${ANTHROPIC_API_KEY:-}" && "$DRY_RUN" == false ]]; then
    echo "❌ ANTHROPIC_API_KEY is not set. Export it in this shell first — this script never asks for it directly." >&2
    exit 1
fi

echo "🔥 Prometheus Skill Pack — Skills API upload"
echo "============================================="
$DRY_RUN && echo "  Mode: DRY RUN (no uploads)"
$INCLUDE_LOCAL && echo "  Including local-dependent skills"
[[ -n "$ONLY" ]] && echo "  Filter: only '$ONLY'"
echo ""

# Patterns that indicate a skill assumes local-machine execution (a harness
# running on your Mac with bash access), which Desktop's sandboxed Custom
# Skills container does not have.
LOCAL_DEP_PATTERN='localhost|127\.0\.0\.1|surreal-memory|surreal_memory|stdio transport|mcp server|harness registration|~/.config/liter-llm|launchctl|launchd'

is_local_dependent() {
    local skill_dir="$1"
    grep -qEi "$LOCAL_DEP_PATTERN" "$skill_dir/SKILL.md" 2>/dev/null
}

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

uploaded=0
skipped_local=0
failed=0

while IFS= read -r -d '' skill_md; do
    skill_dir=$(dirname "$skill_md")
    skill_name=$(basename "$skill_dir")

    [[ -n "$ONLY" && "$skill_name" != "$ONLY" ]] && continue

    if is_local_dependent "$skill_dir" && [[ "$INCLUDE_LOCAL" == false ]]; then
        echo "  ⏭  $skill_name — skipped (local-machine dependency detected; use --include-local-dependent to force)"
        skipped_local=$((skipped_local + 1))
        continue
    fi

    zip_path="$WORKDIR/$skill_name.zip"

    if $DRY_RUN; then
        echo "  ○ would upload: $skill_name"
        continue
    fi

    # Zip with the skill directory itself as the single top-level entry,
    # per the Skills API requirement.
    (cd "$skill_dir/.." && zip -qr "$zip_path" "$skill_name" -x '*.git*')

    echo -n "  → uploading $skill_name... "

    # NOTE: the exact multipart field name for a zip-archive upload via raw
    # curl isn't 100% pinned down from public docs at the time this script
    # was written — the documented examples show `--file` via the `ant`
    # CLI helper and per-file `files[]=@path;filename=path` via curl, but
    # not a confirmed field name for a single zip via curl. Prefer the
    # `ant` CLI if you have it installed; the curl fallback below uses
    # `file=` as the best-supported guess — verify against the response on
    # your first real upload (--only <one-skill>) before trusting the loop.
    if command -v ant &>/dev/null; then
        response=$(ant beta:skills create --file "$zip_path" --beta "$API_BETA" 2>&1) && ok=true || ok=false
    else
        response=$(curl -sS -X POST "https://api.anthropic.com/v1/skills" \
            -H "x-api-key: $ANTHROPIC_API_KEY" \
            -H "anthropic-version: $API_VERSION" \
            -H "anthropic-beta: $API_BETA" \
            -F "display_title=$skill_name" \
            -F "file=@${zip_path};type=application/zip" \
            2>&1) && ok=true || ok=false
    fi

    if $ok && ! echo "$response" | grep -qi '"type":"error"'; then
        echo "✅"
        uploaded=$((uploaded + 1))
    else
        echo "❌"
        echo "      $response" | head -3
        failed=$((failed + 1))
    fi

done < <(find "$REPO_ROOT/skills" -name "SKILL.md" -not -path "*/imported/*" -not -path "*/tests/*" -not -path "*/fixtures/*" -print0)

echo ""
echo "Done. uploaded=$uploaded  skipped_local=$skipped_local  failed=$failed"
$DRY_RUN && echo "(dry run — nothing was actually uploaded)"
if [[ "$failed" -gt 0 ]]; then
    echo "⚠️  Check failures above before assuming the batch succeeded — confirm the multipart field name against the Skills API reference if every upload failed the same way."
    exit 1
fi
exit 0
