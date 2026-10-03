#!/usr/bin/env bash
# skills/process/prometheus-context-bootstrap/scripts/bootstrap-v4.sh
# Prometheus Rules Architecture v4: seed the single-source rules tree and render it.
#
#   Layer 0  CLAUDE.md (<=120 lines; AGENTS.md and GEMINI.md link to it)
#   Layer 1  .claude/rules/*.md, every file path-scoped
#   Layer 3  docs/skill-routing.md          Layer 4  hooks, .githooks/commit-msg, two check scripts
#
# Everything generated comes from <project>/rules/src/ via <project>/rules/build.sh. This script only SEEDS:
# a file that already exists is reported SKIP and never overwritten without --force, because rules/src/ is the
# project's own source once it has been edited. Reached through `bootstrap.sh --layout v4`.
set -euo pipefail

SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$SKILL_ROOT/references/rules-src"
ASSETS="$SKILL_ROOT/assets"

die() { printf 'prometheus-context-bootstrap: %s\n' "$*" >&2; exit 1; }

project_path="."; dry_run=0; force=0; no_hooks=0; stacks=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)     project_path="${2:?--path requires a value}"; shift 2 ;;
    --stacks)   stacks="${2:?--stacks requires a value}"; shift 2 ;;
    --layout|--profile) shift 2 ;;          # --layout already decided; profiles do not exist in v4
    --dry-run)  dry_run=1; shift ;;
    --force)    force=1; shift ;;
    --no-hooks) no_hooks=1; shift ;;
    *) die "unknown flag for --layout v4: $1" ;;
  esac
done
[[ -d "$project_path" ]] || die "--path is not a directory: $project_path"
project_path="$(cd "$project_path" && pwd)"

# Shared UI contract preflight before legacy writes; the helper uses Node on every platform.
uiux_runtime="$SKILL_ROOT/../prometheus-ui-ux/scripts/cli.mjs"
[[ -f "$uiux_runtime" ]] || uiux_runtime="$SKILL_ROOT/../../ui-ux/prometheus-ui-ux/scripts/cli.mjs"
[[ -f "$uiux_runtime" ]] || die "bundled prometheus-ui-ux runtime missing"
node "$uiux_runtime" install --project "$project_path" --dry-run >/dev/null

[[ -d "$SRC" && -f "$ASSETS/v4/rules/build.py" ]] || die "skill payload missing: references/rules-src or assets/v4"
command -v python3 >/dev/null 2>&1 || die "python3 is required for the v4 layout"

# Stack names: accept the legacy spelling, render the v4 file names.
resolved=()
if [[ -z "$stacks" ]]; then
  [[ -n "$(find "$project_path" -maxdepth 3 -name Cargo.toml    -not -path '*/node_modules/*' -print -quit 2>/dev/null)" ]] && resolved+=("rust")
  [[ -n "$(find "$project_path" -maxdepth 3 -name package.json  -not -path '*/node_modules/*' -print -quit 2>/dev/null)" ]] && resolved+=("typescript-react")
  [[ -n "$(find "$project_path" -maxdepth 3 -name pubspec.yaml  -print -quit 2>/dev/null)" ]] && resolved+=("flutter")
  [[ -n "$(find "$project_path" -maxdepth 3 -name go.mod        -print -quit 2>/dev/null)" ]] && resolved+=("go")
  [[ -n "$(find "$project_path" -maxdepth 3 -name pyproject.toml -print -quit 2>/dev/null)" ]] && resolved+=("python")
else
  IFS=',' read -r -a asked <<< "$stacks"
  for s in "${asked[@]}"; do
    [[ "$s" == "typescript" ]] && s="typescript-react"
    [[ -f "$SRC/tech/$s.md" ]] || die "unknown stack: $s (have: $(cd "$SRC/tech" && ls | sed 's/\.md$//' | tr '\n' ' '))"
    resolved+=("$s")
  done
fi
stack_line="${resolved[*]:-}"
[[ -n "$stack_line" ]] || printf 'prometheus-context-bootstrap: warn: no stack detected — pass --stacks; only domain rules will render\n' >&2

printf 'Starting prometheus-context-bootstrap — %s (layout: v4; stacks: %s)\n' "$project_path" "${stack_line:-none}"

declare -a REPORT
SEEDED=" "      # space-separated relative paths written in THIS run; a SKIPped file is the project's own
record() { REPORT+=("$(printf '%-8s %-46s %s' "$1" "$2" "$3")"); }

seed() {   # src dest [mode]
  local src="$1" dest="$project_path/$2" rel="$2" mode="${3:-}"
  if [[ -e "$dest" && "$force" != "1" ]]; then record "SKIP" "$rel" "exists — yours to edit; --force to re-seed"; return 0; fi
  [[ -e "$dest" ]] && record "CREATE" "$rel" "re-seeded (--force)" || record "CREATE" "$rel" "seeded"
  [[ "$dry_run" == "1" ]] && return 0
  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  SEEDED="$SEEDED$rel "
  [[ -n "$mode" ]] && chmod "$mode" "$dest"
  return 0
}

# ---- rules/src — the single source
name="$(basename "$project_path")"
while IFS= read -r f; do
  rel="${f#"$SRC"/}"
  seed "$f" "rules/src/$rel"
done < <(find "$SRC" -type f -name '*.md' | sort)
# Only a constitution seeded by this run gets the project name; an existing one is never touched.
if [[ "$dry_run" != "1" && "$SEEDED" == *" rules/src/constitution.md "* ]]; then
  python3 - "$project_path/rules/src/constitution.md" "$name" <<'PY'
import sys
path, name = sys.argv[1], sys.argv[2]
text = open(path).read()
if "__PROJECT__" in text:
    open(path, "w").write(text.replace("__PROJECT__", name))
PY
fi

# ---- generator, config, allowlist
seed "$ASSETS/v4/rules/build.sh" "rules/build.sh" 755
seed "$ASSETS/v4/rules/build.py" "rules/build.py" 755
seed "$ASSETS/v4/rules/line-limit-allowlist.txt" "rules/line-limit-allowlist.txt"
if [[ ! -e "$project_path/rules/build.conf" || "$force" == "1" ]]; then
  record "CREATE" "rules/build.conf" "stacks: ${stack_line:-none}"
  if [[ "$dry_run" != "1" ]]; then
    mkdir -p "$project_path/rules"
    sed "s|__STACKS__|$stack_line|" "$ASSETS/v4/rules/build.conf" > "$project_path/rules/build.conf"
  fi
else
  record "SKIP" "rules/build.conf" "exists"
fi

# ---- Layer 4
seed "$ASSETS/scripts/check-file-lines.sh"   "scripts/check-file-lines.sh" 755
seed "$ASSETS/scripts/check-architecture.sh" "scripts/check-architecture.sh" 755
seed "$ASSETS/githooks/commit-msg"           ".githooks/commit-msg" 755
if [[ "$no_hooks" != "1" ]]; then
  for h in tier-guard single-writer sycophancy-gate reanchor file-lines-guard build-guard; do
    seed "$ASSETS/hooks/$h.sh" ".claude/hooks/$h.sh" 755
  done
  seed "$ASSETS/agents/artifact-critic.md" ".claude/agents/artifact-critic.md"
fi

# ---- settings.json: create from the template, or add only the hook entries that are absent
settings="$project_path/.claude/settings.json"
if [[ "$no_hooks" == "1" ]]; then
  record "SKIP" ".claude/settings.json" "--no-hooks"
elif ! command -v jq >/dev/null 2>&1; then
  record "SKIP" ".claude/settings.json" "jq absent — hooks installed but NOT wired"
elif [[ "$dry_run" == "1" ]]; then
  record "MERGE" ".claude/settings.json" "would wire every installed hook that is not wired yet"
else
  mkdir -p "$(dirname "$settings")"
  [[ -f "$settings" ]] || cp "$SKILL_ROOT/references/settings.template.json" "$settings"
  merged="$settings.tmp.$$"
  if jq '
      def wire($event; $matcher; $script):
        if ([.hooks[$event][]?.hooks[]?.command // ""] | any(test($script))) then .
        else .hooks[$event] = ((.hooks[$event] // []) + [{matcher: $matcher, hooks: [{type: "command", command: ("${CLAUDE_PROJECT_DIR}/.claude/hooks/" + $script)}]}]) end;
      .hooks //= {}
      | wire("PreToolUse"; "Bash"; "tier-guard.sh")
      | wire("PreToolUse"; "Bash"; "build-guard.sh")
      | wire("PostToolUse"; "Edit|Write"; "single-writer.sh")
      | wire("PostToolUse"; "Edit|Write"; "file-lines-guard.sh")
      | wire("Stop"; ""; "sycophancy-gate.sh")
      | wire("SessionStart"; ""; "reanchor.sh")
      | wire("PreCompact"; ""; "reanchor.sh")
      | .hooks |= with_entries(.value |= map(if .matcher == "" then del(.matcher) else . end))
    ' "$settings" > "$merged" 2>/dev/null && [[ -s "$merged" ]]; then
    mv -f "$merged" "$settings"; record "MERGE" ".claude/settings.json" "every installed hook wired (existing entries kept)"
  else
    rm -f "$merged"; record "SKIP" ".claude/settings.json" "merge failed — new hooks NOT wired"
  fi
fi

# ---- Layer 3 memory (same layout as the legacy bootstrap; never overwritten)
today="$(date -u +%Y-%m-%d)"
for f in session-log decisions gotchas; do
  dest="$project_path/.prometheus/$f.md"
  if [[ -f "$dest" ]]; then record "SKIP" ".prometheus/$f.md" "exists"; continue; fi
  record "CREATE" ".prometheus/$f.md" "append-only"
  [[ "$dry_run" == "1" ]] || { mkdir -p "$project_path/.prometheus"; printf '# %s\n\nAppend-only. Dated entries. Mark superseded entries; do not delete them.\n\n## %s\n- Initialized by prometheus-context-bootstrap (layout v4).\n' "$f" "$today" > "$dest"; }
done
[[ "$dry_run" == "1" ]] || mkdir -p "$project_path/.prometheus/postmortems" "$project_path/.prometheus/knowledge" "$project_path/tasks"

# ---- render
if [[ "$dry_run" == "1" ]]; then
  record "RENDER" "CLAUDE.md, AGENTS.md, GEMINI.md, .claude/rules/*" "would run rules/build.sh"
else
  for legacy in CLAUDE.md AGENTS.md; do
    f="$project_path/$legacy"
    if [[ -f "$f" && ! -L "$f" ]] && ! grep -q 'prometheus-rules: v4' "$f" 2>/dev/null; then
      mkdir -p "$project_path/.prometheus/knowledge"
      cp "$f" "$project_path/.prometheus/knowledge/${legacy%.md}.pre-v4-$today.md"
      record "ARCHIVE" "$legacy" "copied to .prometheus/knowledge/ before rendering — move its project rules into rules/src/"
    fi
  done
  bash "$project_path/rules/build.sh" >/dev/null || die "rules/build.sh failed — see its output by running it directly"
  record "RENDER" "CLAUDE.md, AGENTS.md, GEMINI.md, .claude/rules/*" "rendered from rules/src/"
fi

printf '\n%-8s %-46s %s\n' "MODE" "PATH" "NOTE"
printf '%s\n' "${REPORT[@]}"
cat <<EOF

Layout v4. Edit rules/src/, then:  bash rules/build.sh   (and --check in CI).
Next: replace the first §P line in rules/src/constitution.md; set Status in rules/src/routing.md;
      add nested AGENTS.md targets in rules/build.conf; git config core.hooksPath .githooks; run verify.sh.
$( [[ "$dry_run" == "1" ]] && echo "DRY RUN — nothing was written." )
Completed prometheus-context-bootstrap — $project_path
EOF

if [[ "$dry_run" != "1" ]]; then
  node "$uiux_runtime" install --project "$project_path"
fi
