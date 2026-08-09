#!/usr/bin/env bash
# codex-sync-skills.sh — sync prometheus-skill-pack skills into ~/.codex/skills.
#
# WHY THIS IS NOT A SYMLINK INSTALL
# --------------------------------
# Codex CLI 0.144.x does not traverse symlinked skill directories: a symlink in
# ~/.codex/skills contributes ZERO skills to the catalog, silently. The pack's
# 138 symlinks loaded 0 skills. Real directories load correctly, so this script
# copies and keeps the copies fresh.
#
# Usage:
#   bash scripts/codex-sync-skills.sh              # sync catalog skills
#   bash scripts/codex-sync-skills.sh --dry-run    # show sync/prune plan only
#   bash scripts/codex-sync-skills.sh --report     # sync, then print catalog cost
#   bash scripts/codex-sync-skills.sh --uninstall  # remove pack skills from codex
#   bash scripts/codex-sync-skills.sh --quiet      # no output unless something changed

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CODEX_SKILLS="${CODEX_HOME:-$HOME/.codex}/skills"
MANIFEST="$REPO_ROOT/config/codex-catalog.txt"
MARKER=".prometheus-pack"

UNINSTALL=false
REPORT=false
QUIET=false
DRY_RUN=false
for arg in "$@"; do
    case "$arg" in
        --uninstall) UNINSTALL=true ;;
        --report)    REPORT=true ;;
        --quiet)     QUIET=true ;;
        --dry-run)   DRY_RUN=true ;;
    esac
done

say() { $QUIET || echo "$@"; }

# Codex not installed -> nothing to do (never fail a caller's hook chain).
if [[ ! -d "$(dirname "$CODEX_SKILLS")" ]]; then
    say "  — codex: ~/.codex not found, skipping"
    exit 0
fi

mkdir -p "$CODEX_SKILLS"

# ---------------------------------------------------------------------------
# Resolve the catalog: top-level skill dirs (a dir with SKILL.md whose ancestor
# is not itself a skill dir), filtered through config/codex-catalog.txt.
# ---------------------------------------------------------------------------
resolve_catalog() {
    python3 - "$REPO_ROOT" "$MANIFEST" <<'PY'
import fnmatch, os, sys

repo, manifest = sys.argv[1], sys.argv[2]

skill_dirs = set()
for root, dirs, files in os.walk(os.path.join(repo, "skills")):
    dirs[:] = [d for d in dirs if d not in (".git", "node_modules")]
    if "SKILL.md" in files:
        skill_dirs.add(os.path.relpath(root, repo))

# imported/ skills are submodules with their own lifecycle: never copied.
skill_dirs = {d for d in skill_dirs if not d.startswith("skills/imported")}

# Top-level = no ancestor is also a skill dir.
def is_top(d):
    parts = d.split("/")
    return not any("/".join(parts[:i]) in skill_dirs for i in range(1, len(parts)))

tops = sorted(d for d in skill_dirs if is_top(d))

rules = []
if os.path.exists(manifest):
    for line in open(manifest):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        verb, _, pat = line.partition(" ")
        if verb in ("include", "exclude"):
            rules.append((verb, pat.strip()))
        else:
            rules.append(("include", line))

def selected(d):
    keep = not rules  # no manifest -> everything
    for verb, pat in rules:
        # 'skills/**' should match 'skills/rust/actor-model'
        if fnmatch.fnmatch(d, pat) or fnmatch.fnmatch(d, pat.rstrip("/*") + "/*") or d == pat.rstrip("/*"):
            keep = (verb == "include")
    return keep

for d in tops:
    if selected(d):
        print(d)
PY
}

# ---------------------------------------------------------------------------
# Uninstall: remove only dirs we own (marker file present).
# ---------------------------------------------------------------------------
if $UNINSTALL; then
    removed=0
    for dest in "$CODEX_SKILLS"/*; do
        [[ -d "$dest" && -f "$dest/$MARKER" ]] || continue
        if $DRY_RUN; then
            say "  [dry-run] would remove $dest"
        else
            rm -rf "$dest"
        fi
        removed=$((removed + 1))
    done
    # Legacy: the old installer left symlinks pointing into this repo.
    for link in "$CODEX_SKILLS"/*; do
        if [[ -L "$link" ]] && [[ "$(readlink "$link")" == "$REPO_ROOT"* ]]; then
            if $DRY_RUN; then
                say "  [dry-run] would remove legacy symlink $link"
            else
                rm -f "$link"
            fi
            removed=$((removed + 1))
        fi
    done
    if $DRY_RUN; then
        say "  ✅ codex: dry-run planned removal of $removed skills"
    else
        say "  ✅ codex: $removed skills removed"
    fi
    exit 0
fi

# ---------------------------------------------------------------------------
# Sync: copy each catalog skill as a real directory.
# ---------------------------------------------------------------------------
# NOTE: this script runs under launchd with /bin/bash, which on macOS is bash 3.2 —
# no `mapfile`, no `declare -A`. Keep everything here bash 3.2 compatible.
CATALOG=()
while IFS= read -r line; do
    [[ -n "$line" ]] && CATALOG+=("$line")
done < <(resolve_catalog)

if [[ ${#CATALOG[@]} -eq 0 ]]; then
    echo "  ⚠️  codex: catalog resolved to 0 skills — check $MANIFEST" >&2
    exit 1
fi

# Drop legacy symlinks from the old ln -s installer (these loaded nothing).
legacy=0
for link in "$CODEX_SKILLS"/*; do
    if [[ -L "$link" ]] && [[ "$(readlink "$link")" == "$REPO_ROOT"* ]]; then
        if $DRY_RUN; then
            say "  [dry-run] would remove dead symlink $link"
        else
            rm -f "$link"
        fi
        legacy=$((legacy + 1))
    fi
done
[[ $legacy -gt 0 ]] && say "  🧹 codex: removed $legacy dead symlinks from the old installer"

WANTED=""   # newline-delimited set (bash 3.2 has no associative arrays)
synced=0
for rel in "${CATALOG[@]}"; do
    name="$(basename "$rel")"
    src="$REPO_ROOT/$rel"
    dest="$CODEX_SKILLS/$name"
    WANTED="$WANTED$name
"

    # Immutable-generation copies are canonical and carry their own receipt.
    # Do not overwrite them with the legacy .prometheus-pack format.
    if [[ -d "$dest" && -f "$dest/.prometheus-generation" ]]; then
        say "  — codex: $name is managed by the active immutable generation"
        continue
    fi

    # Refuse to clobber a directory we do not own (user's own skill, codex builtin).
    if [[ -d "$dest" && ! -f "$dest/$MARKER" ]]; then
        say "  ⚠️  codex: $name exists and is not pack-owned — skipping"
        continue
    fi

    if $DRY_RUN; then
        say "  [dry-run] would sync $src -> $dest"
    elif command -v rsync >/dev/null 2>&1; then
        rsync -a --delete \
            --exclude '.git' --exclude 'node_modules' --exclude "$MARKER" \
            "$src/" "$dest/"
    else
        rm -rf "$dest"
        mkdir -p "$dest"
        cp -R "$src/." "$dest/"
    fi
    if ! $DRY_RUN; then
        printf 'source=%s\n' "$rel" > "$dest/$MARKER"
    fi
    synced=$((synced + 1))
done

# Prune pack-owned skills that left the catalog (e.g. newly excluded, renamed).
pruned=0
for dest in "$CODEX_SKILLS"/*; do
    [[ -d "$dest" && -f "$dest/$MARKER" ]] || continue
    name="$(basename "$dest")"
    if ! printf '%s' "$WANTED" | grep -qx "$name"; then
        if $DRY_RUN; then
            say "  [dry-run] would prune $dest"
        else
            rm -rf "$dest"
        fi
        pruned=$((pruned + 1))
    fi
done

if $DRY_RUN; then
    say "  ✅ codex: dry-run planned sync of $synced skills$([[ $pruned -gt 0 ]] && echo ", $pruned pruned")"
else
    say "  ✅ codex: $synced skills synced as real directories$([[ $pruned -gt 0 ]] && echo ", $pruned pruned")"
fi

# ---------------------------------------------------------------------------
# Report: what the catalog actually costs the model.
# ---------------------------------------------------------------------------
if $REPORT; then
    entries=$(find "$CODEX_SKILLS" -name SKILL.md -not -path "*/.system/*" 2>/dev/null | wc -l | tr -d ' ')
    echo ""
    echo "  Catalog cost (~/.codex/skills):"
    echo "    top-level skills copied : $synced"
    echo "    catalog entries (incl. nested sub-skills): $entries"
    echo ""
    echo "  Codex budgets the whole catalog, so description length falls as entries rise:"
    echo "    ~130 entries -> ~166 char descriptions (auto-trigger reliable)"
    echo "    ~200 entries ->  ~66 char descriptions (usable)"
    echo "    ~360 entries ->  ~10 char descriptions (auto-trigger broken)"
    echo ""
    echo "  Measure the live figure with:"
    echo "    codex debug prompt-input | python3 scripts/codex-catalog-stat.py"
fi
