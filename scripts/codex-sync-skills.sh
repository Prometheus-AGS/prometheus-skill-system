#!/usr/bin/env bash
# codex-sync-skills.sh — retire the legacy pack copies in ~/.codex/skills.
#
# WHY THIS NO LONGER COPIES ANYTHING
# ----------------------------------
# Codex reads the canonical skill root ~/.agents/skills, where the signed
# generation installer places every pack skill ONCE as a real directory (Codex
# does not traverse symlinked skill directories). A second copy under
# ~/.codex/skills only duplicated every catalog entry and halved the description
# budget, so this script now only removes what it previously created there:
#
#   - directories carrying this script's `.prometheus-pack` marker whose
#     `source=skills/...` names a skill of the same basename, and
#   - legacy symlinks from the old `ln -s` installer that point into this repo.
#
# Generation-owned copies (`.prometheus-generation`) are retired by
# install-plugin-generation.js, which verifies their owner generation first.
# Nothing else in ~/.codex/skills is touched.
#
# Usage:
#   bash scripts/codex-sync-skills.sh              # retire legacy pack copies
#   bash scripts/codex-sync-skills.sh --dry-run    # show the retirement plan only
#   bash scripts/codex-sync-skills.sh --report     # retire, then print catalog cost
#   bash scripts/codex-sync-skills.sh --uninstall  # same as the default
#   bash scripts/codex-sync-skills.sh --quiet      # no output unless something changed
#
# NOTE: keep this bash 3.2 compatible (macOS /bin/bash): no mapfile, no declare -A.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CODEX_SKILLS="${CODEX_HOME:-$HOME/.codex}/skills"
AGENTS_SKILLS="$HOME/.agents/skills"
MARKER=".prometheus-pack"

REPORT=false
QUIET=false
DRY_RUN=false
for arg in "$@"; do
    case "$arg" in
        --uninstall) ;;
        --report)    REPORT=true ;;
        --quiet)     QUIET=true ;;
        --dry-run)   DRY_RUN=true ;;
    esac
done

say() { $QUIET || echo "$@"; }

# True when $1 is a copy this script created: marker names skills/<...>/<basename>.
is_legacy_copy() {
    local dest="$1" source rel
    [[ -d "$dest" && ! -L "$dest" && -f "$dest/$MARKER" ]] || return 1
    [[ ! -e "$dest/.prometheus-generation" ]] || return 1
    source="$(head -n 1 "$dest/$MARKER" 2>/dev/null || true)"
    rel="${source#source=}"
    [[ "$source" == source=skills/* && "${rel##*/}" == "$(basename "$dest")" ]] || return 1
    case "$rel" in *..*) return 1 ;; esac
    return 0
}

is_legacy_link() {
    [[ -L "$1" ]] && [[ "$(readlink "$1")" == "$REPO_ROOT"/* ]]
}

retire_entry() {
    local entry="$1" kind="$2"
    if $DRY_RUN; then
        say "  [dry-run] would remove legacy $kind $entry"
    else
        rm -rf "$entry"
    fi
}

retire_legacy() {
    local removed=0 entry
    for entry in "$CODEX_SKILLS"/*; do
        if is_legacy_link "$entry"; then
            retire_entry "$entry" "symlink"
        elif is_legacy_copy "$entry"; then
            retire_entry "$entry" "copy"
        elif [[ -d "$entry" && -f "$entry/.prometheus-generation" ]]; then
            say "  — codex: $(basename "$entry") is managed by the active immutable generation (retired by install-plugin-generation.js)"
            continue
        else
            continue
        fi
        removed=$((removed + 1))
    done
    if [[ $removed -gt 0 ]]; then
        if $DRY_RUN; then
            echo "  ✅ codex: dry-run planned removal of $removed legacy pack entries from $CODEX_SKILLS"
        else
            echo "  🧹 codex: removed $removed legacy pack entries from $CODEX_SKILLS (served by ~/.agents/skills)"
        fi
    else
        say "  ✅ codex: no legacy pack entries in $CODEX_SKILLS"
    fi
}

report_cost() {
    local root="$AGENTS_SKILLS" entries
    if [[ -L "$root" ]]; then
        root="$(cd "$root" 2>/dev/null && pwd -P || echo "$root")"
    fi
    entries=$(find "$root" -name SKILL.md -not -path "*/.system/*" 2>/dev/null | wc -l | tr -d ' ')
    echo ""
    echo "  Catalog cost (canonical root $root):"
    echo "    catalog entries (incl. nested sub-skills): $entries"
    echo ""
    echo "  Codex budgets the whole catalog, so description length falls as entries rise:"
    echo "    ~130 entries -> ~166 char descriptions (auto-trigger reliable)"
    echo "    ~200 entries ->  ~66 char descriptions (usable)"
    echo "    ~360 entries ->  ~10 char descriptions (auto-trigger broken)"
    echo ""
    echo "  Selection is applied by shared/scripts/codex-catalog-config.sh; measure with:"
    echo "    codex debug prompt-input | python3 scripts/codex-catalog-stat.py"
}

# Codex not installed, or nothing there -> nothing to do (never fail a caller).
if [[ ! -d "$CODEX_SKILLS" ]]; then
    say "  — codex: $CODEX_SKILLS not found, nothing to retire"
else
    retire_legacy
fi
if $REPORT; then
    report_cost
fi
exit 0
