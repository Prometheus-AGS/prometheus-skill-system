#!/usr/bin/env bash
# skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh
# Scaffold the lean-context agent structure into a new or existing project.
#
# Write modes: CREATE (absent), SPLICE (marked region only), SKIP (present,
# untouched). --force promotes SKIP to CREATE for hooks/rules/settings only —
# never for agent prose or .prometheus history.

set -euo pipefail

SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REF="$SKILL_ROOT/references"
ASSETS="$SKILL_ROOT/assets"

die()  { printf 'prometheus-context-bootstrap: %s\n' "$*" >&2; exit 1; }
warn() { printf 'prometheus-context-bootstrap: warn: %s\n' "$*" >&2; }

START_MARK='<!-- prometheus-base:start v1 -->'
END_MARK='<!-- prometheus-base:end -->'
STACK_MARK='<!-- prometheus-base:stacks -->'

project_path="."
dry_run=0
force=0
no_hooks=0
stacks_override=""
profile="mixed"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)    project_path="${2:?--path requires a value}"; shift 2 ;;
    --stacks)  stacks_override="${2:?--stacks requires a value}"; shift 2 ;;
    --profile) profile="${2:?--profile requires a value}"; shift 2 ;;
    --dry-run) dry_run=1; shift ;;
    --force)   force=1; shift ;;
    --no-hooks) no_hooks=1; shift ;;
    -h|--help)
      cat <<'USAGE'
Usage: bootstrap.sh [--path <root>] [--stacks rust,typescript,...]
                    [--profile mixed|strict|lean]
                    [--dry-run] [--force] [--no-hooks]

  --path      project root (default: .)
  --stacks    override detection: rust,typescript,flutter,go,python
  --profile   mixed (default) and strict include the execution scaffold; lean
              omits it. AGENTS.md is per repo, not per model, so the weakest
              model in the fleet governs its content. Choose lean only after
              measuring that no model regressed. See references/MODEL-PROFILES.md
  --dry-run   print the plan and agent-file diffs; write nothing
  --force     re-copy hooks, rules, settings that already exist
  --no-hooks  skip .claude/hooks and the settings hook block

Exit: 0 applied or previewed, 1 usage error, 2 refused (corrupt markers)
USAGE
      exit 0 ;;
    *) die "unknown flag: $1 (try --help)" ;;
  esac
done

case "$profile" in
  mixed|strict|lean) : ;;
  *) die "--profile must be mixed, strict, or lean (got: $profile)" ;;
esac

[[ -d "$project_path" ]] || die "--path is not a directory: $project_path"
project_path="$(cd "$project_path" && pwd)"

for f in "$REF/AGENTS.base.md" "$REF/settings.template.json"; do
  [[ -f "$f" ]] || die "skill payload missing: $f"
done
if [[ "$profile" != "lean" ]]; then
  [[ -f "$REF/AGENTS.scaffold.md" ]] || die "skill payload missing: $REF/AGENTS.scaffold.md"
fi

printf 'Starting prometheus-context-bootstrap — %s (profile: %s)\n' "$project_path" "$profile"

# ---------------------------------------------------------------- detection --
detect_stacks() {
  local found=()
  [[ -f "$project_path/Cargo.toml"     ]] && found+=("rust")
  [[ -f "$project_path/package.json"   ]] && found+=("typescript")
  [[ -f "$project_path/pubspec.yaml"   ]] && found+=("flutter")
  [[ -f "$project_path/go.mod"         ]] && found+=("go")
  [[ -f "$project_path/pyproject.toml" ]] && found+=("python")
  printf '%s\n' "${found[@]:-}"
}

if [[ -n "$stacks_override" ]]; then
  IFS=',' read -r -a STACKS <<< "$stacks_override"
else
  mapfile -t STACKS < <(detect_stacks)
fi
# Drop empties produced by a no-match detection.
tmp_stacks=(); for s in "${STACKS[@]:-}"; do [[ -n "$s" ]] && tmp_stacks+=("$s"); done
STACKS=("${tmp_stacks[@]:-}")

for s in "${STACKS[@]:-}"; do
  [[ -z "$s" ]] && continue
  case "$s" in
    rust|typescript|flutter|go|python) : ;;
    *) die "unknown stack: $s" ;;
  esac
done

if [[ ${#STACKS[@]} -eq 0 || -z "${STACKS[0]:-}" ]]; then
  warn "no stack detected — writing rules-agnostic structure; pass --stacks to force"
fi

# ------------------------------------------------------------------ reporting --
declare -a REPORT
record() { REPORT+=("$1|$2|$3"); }   # mode | path | note

print_report() {
  printf '\n%-8s %-46s %s\n' "MODE" "PATH" "NOTE"
  printf '%-8s %-46s %s\n' "--------" "----------------------------------------------" "----"
  local line mode p note
  for line in "${REPORT[@]:-}"; do
    [[ -z "$line" ]] && continue
    mode="${line%%|*}"; line="${line#*|}"
    p="${line%%|*}"; note="${line#*|}"
    printf '%-8s %-46s %s\n' "$mode" "$p" "$note"
  done
}

# Write $2 to $1 unless dry-run. Returns 0 if written.
put() {
  local dest="$1" content="$2"
  if [[ "$dry_run" == "1" ]]; then return 0; fi
  mkdir -p "$(dirname "$dest")"
  printf '%s' "$content" > "$dest"
}

copy_managed() {   # src dest label
  local src="$1" dest="$2" label="$3" rel="${2#$project_path/}"
  if [[ -f "$dest" && "$force" != "1" ]]; then
    record "SKIP" "$rel" "exists; --force to replace"
    return
  fi
  local mode="CREATE"; [[ -f "$dest" ]] && mode="REPLACE"
  if [[ "$dry_run" != "1" ]]; then
    mkdir -p "$(dirname "$dest")"
    cp "$src" "$dest"
    [[ "$dest" == *.sh ]] && chmod +x "$dest"
  fi
  record "$mode" "$rel" "$label"
}

# ------------------------------------------------------------ region render --
# Build the AGENTS.md managed region, substituting the stack line.
render_region() {
  # awk -v cannot carry newlines, so multi-line substitutions go through files
  # and are read with getline. Same constraint kbd-inject-agent-rules hit.
  local stackfile="$region_scratch.stacks"
  : > "$stackfile"
  local s wrote=0
  for s in "${STACKS[@]:-}"; do
    [[ -z "$s" ]] && continue
    printf -- '- `.claude/rules/%s.md` — %s tiers and hard rules\n' "$s" "$s" >> "$stackfile"
    wrote=1
  done
  [[ "$wrote" == "0" ]] && \
    printf -- '- No stack rules installed. Add `.claude/rules/<stack>.md` when one applies.\n' >> "$stackfile"

  # Splice the execution scaffold ahead of the end marker when the profile
  # calls for it. One marker pair, not two, so switching profile is a
  # re-splice rather than a second managed region.
  local scaffold=""
  [[ "$profile" != "lean" ]] && scaffold="$REF/AGENTS.scaffold.md"

  awk -v stf="$stackfile" -v mark="$STACK_MARK" -v endm="$END_MARK" \
      -v sf="$scaffold" -v prof="$profile" '
    function emit(f,   line) {
      if (f == "") return
      while ((getline line < f) > 0) print line
      close(f)
    }
    $0 == mark { emit(stf); next }
    $0 == endm {
      if (sf != "") { print ""; emit(sf) }
      print ""
      print "<!-- profile: " prof " — see references/MODEL-PROFILES.md before changing -->"
      print $0
      next
    }
    { print }
  ' "$REF/AGENTS.base.md"
}

# ------------------------------------------------------------ marker splice --
# Splice the region into an existing file. Refuses on a corrupt marker pair.
splice_markers() {   # file regionfile -> writes to stdout
  local target="$1" regionfile="$2"
  awk -v start="$START_MARK" -v end="$END_MARK" -v rf="$regionfile" '
    function emit(   line) { while ((getline line < rf) > 0) print line; close(rf) }
    BEGIN { inb = 0; done = 0 }
    {
      if ($0 == start) { inb = 1; if (!done) { emit(); done = 1 } ; next }
      if (inb && $0 == end) { inb = 0; next }
      if (!inb) print
    }
  ' "$target"
}

check_markers() {   # file -> exit 2 on corruption
  local f="$1" starts ends
  [[ -f "$f" ]] || return 0
  starts="$(grep -cF "$START_MARK" "$f" 2>/dev/null; :)"
  ends="$(grep -cF "$END_MARK" "$f" 2>/dev/null; :)"
  if [[ "$starts" -gt 1 ]]; then
    printf 'REFUSED: %s has %s start markers. Deduplicate by hand.\n' "$f" "$starts" >&2
    exit 2
  fi
  if [[ "$starts" == "1" && "$ends" == "0" ]]; then
    printf 'REFUSED: %s has a start marker with no end marker. Repair by hand.\n' "$f" >&2
    exit 2
  fi
  if [[ "$starts" == "0" && "$ends" -gt 0 ]]; then
    printf 'REFUSED: %s has an end marker with no start. Repair by hand.\n' "$f" >&2
    exit 2
  fi
  printf '%s' "$starts"
}

# ------------------------------------------------------------------ AGENTS.md --
agents_file="$project_path/AGENTS.md"
region_tmp="$(mktemp)"
region_scratch="$region_tmp"
trap 'rm -f "$region_tmp" "$region_tmp".*' EXIT
render_region > "$region_tmp"

starts="$(check_markers "$agents_file")"

if [[ ! -f "$agents_file" ]]; then
  put "$agents_file" "$(cat "$region_tmp")"$'\n'
  record "CREATE" "AGENTS.md" "resident invariants"
elif [[ "$starts" == "1" ]]; then
  new_tmp="$region_tmp.new"
  splice_markers "$agents_file" "$region_tmp" > "$new_tmp"
  if cmp -s "$agents_file" "$new_tmp"; then
    record "SKIP" "AGENTS.md" "managed region already current"
  else
    [[ "$dry_run" == "1" ]] && { printf '\n--- AGENTS.md (current)\n+++ AGENTS.md (proposed)\n'; diff -u "$agents_file" "$new_tmp" || true; }
    [[ "$dry_run" != "1" ]] && mv -f "$new_tmp" "$agents_file"
    record "SPLICE" "AGENTS.md" "managed region updated"
  fi
else
  # Exists, unmanaged: append the region, preserve every existing byte.
  new_tmp="$region_tmp.app"
  { cat "$agents_file"; printf '\n'; cat "$region_tmp"; } > "$new_tmp"
  [[ "$dry_run" == "1" ]] && { printf '\n--- AGENTS.md (current)\n+++ AGENTS.md (proposed)\n'; diff -u "$agents_file" "$new_tmp" || true; }
  [[ "$dry_run" != "1" ]] && mv -f "$new_tmp" "$agents_file"
  record "SPLICE" "AGENTS.md" "region appended; existing prose kept"
fi

# ------------------------------------------------------------------ CLAUDE.md --
claude_file="$project_path/CLAUDE.md"
if [[ -L "$claude_file" ]]; then
  record "SKIP" "CLAUDE.md" "already a symlink"
elif [[ ! -e "$claude_file" ]]; then
  [[ "$dry_run" != "1" ]] && ln -s AGENTS.md "$claude_file"
  record "CREATE" "CLAUDE.md" "symlink -> AGENTS.md"
elif grep -q '^@AGENTS\.md[[:space:]]*$' "$claude_file" 2>/dev/null; then
  record "SKIP" "CLAUDE.md" "import line present"
else
  # Real file with content. Never replace it; prepend the import.
  new_tmp="$region_tmp.cl"
  { printf '@AGENTS.md\n\n'; cat "$claude_file"; } > "$new_tmp"
  [[ "$dry_run" == "1" ]] && { printf '\n--- CLAUDE.md (current)\n+++ CLAUDE.md (proposed)\n'; diff -u "$claude_file" "$new_tmp" || true; }
  [[ "$dry_run" != "1" ]] && mv -f "$new_tmp" "$claude_file"
  record "SPLICE" "CLAUDE.md" "import prepended; prose kept, NOT shrunk"
fi

# ---------------------------------------------------------------- path rules --
for s in "${STACKS[@]:-}"; do
  [[ -z "$s" ]] && continue
  src="$REF/rules-${s}.md"
  if [[ ! -f "$src" ]]; then
    warn "no rules template for stack: $s"
    continue
  fi
  copy_managed "$src" "$project_path/.claude/rules/${s}.md" "path-scoped, loads on file read"
done

# --------------------------------------------------------------------- hooks --
if [[ "$no_hooks" == "1" ]]; then
  record "SKIP" ".claude/hooks/" "--no-hooks"
else
  for h in tier-guard single-writer sycophancy-gate reanchor; do
    src="$ASSETS/hooks/${h}.sh"
    [[ -f "$src" ]] || { warn "hook payload missing: $src"; continue; }
    copy_managed "$src" "$project_path/.claude/hooks/${h}.sh" "deterministic enforcement"
  done
fi

# ------------------------------------------------------------------ subagent --
copy_managed "$ASSETS/agents/artifact-critic.md" \
             "$project_path/.claude/agents/artifact-critic.md" \
             "artifact-only critic"

# ------------------------------------------------------------------ settings --
settings="$project_path/.claude/settings.json"
if [[ -f "$settings" && "$force" != "1" ]]; then
  record "SKIP" ".claude/settings.json" "exists; merge hooks by hand or --force"
  if command -v jq >/dev/null 2>&1; then
    if ! jq -e '.hooks.PreToolUse' "$settings" >/dev/null 2>&1; then
      warn "settings.json has no PreToolUse hooks — tier-guard is installed but NOT wired"
    fi
    if ! jq -e '.skillListingBudgetFraction' "$settings" >/dev/null 2>&1; then
      warn "settings.json has no skillListingBudgetFraction — default 1% may drop skill descriptions"
    fi
  fi
else
  if [[ "$no_hooks" == "1" ]] && command -v jq >/dev/null 2>&1; then
    [[ "$dry_run" != "1" ]] && { mkdir -p "$(dirname "$settings")"; jq 'del(.hooks)' "$REF/settings.template.json" > "$settings"; }
    record "CREATE" ".claude/settings.json" "permissions + skill budget (hooks omitted)"
  else
    copy_managed "$REF/settings.template.json" "$settings" "permissions, skill budget, hook wiring"
  fi
fi

# --------------------------------------------------------------- .prometheus --
today="$(date -u +%Y-%m-%d)"
prom="$project_path/.prometheus"
for f in session-log decisions gotchas; do
  dest="$prom/${f}.md"
  if [[ -f "$dest" ]]; then
    record "SKIP" ".prometheus/${f}.md" "append-only history; never replaced"
  else
    title="$(printf '%s' "$f" | tr '-' ' ')"
    put "$dest" "# ${title}

Append-only. Dated entries. Mark superseded entries; do not delete them.

## ${today}
- Initialized by prometheus-context-bootstrap.
"
    record "CREATE" ".prometheus/${f}.md" "append-only"
  fi
done
for d in postmortems knowledge; do
  if [[ -d "$prom/$d" ]]; then
    record "SKIP" ".prometheus/${d}/" "exists"
  else
    [[ "$dry_run" != "1" ]] && mkdir -p "$prom/$d"
    record "CREATE" ".prometheus/${d}/" "directory"
  fi
done

# ------------------------------------------------------------------ waypoint --
wp="$project_path/.kbd-orchestrator/current-waypoint.json"
if [[ -f "$wp" ]]; then
  record "SKIP" ".kbd-orchestrator/current-waypoint.json" "exists"
else
  put "$wp" "{
  \"phase\": \"spec\",
  \"task\": null,
  \"waypoint\": \"bootstrap\",
  \"updated\": \"${today}\",
  \"note\": \"Authoritative for position. tier-guard.sh reads .phase — set it to milestone before Tier 3.\"
}
"
  record "CREATE" ".kbd-orchestrator/current-waypoint.json" "position authority"
fi

# ----------------------------------------------------------------- fleet doc --
fleet="$project_path/.prometheus/model-fleet.md"
if [[ -f "$fleet" ]]; then
  record "SKIP" ".prometheus/model-fleet.md" "exists; profile now ${profile}"
elif [[ -f "$REF/model-fleet.template.md" ]]; then
  put "$fleet" "$(sed -e "s/__PROFILE__/${profile}/" -e "s/__DATE__/${today}/" "$REF/model-fleet.template.md")"$'\n'
  record "CREATE" ".prometheus/model-fleet.md" "profile: ${profile}"
else
  warn "model-fleet template missing; fleet not recorded"
fi

# -------------------------------------------------------------- versions.toml --
vt="$project_path/versions.toml"
if [[ -f "$vt" ]]; then
  record "SKIP" "versions.toml" "exists"
else
  put "$vt" "# Authoritative architecture decisions and dependency pins.
# Agents must not contradict this file, and must not edit it — .claude/settings.json
# denies Edit(versions.toml). Change it deliberately, by hand.

[meta]
created = \"${today}\"

[pins]
# name = \"x.y.z\"

[decisions]
# id = \"rationale\"
"
  record "CREATE" "versions.toml" "decision + pin authority"
fi

# -------------------------------------------------------------------- gitignore --
gi="$project_path/.gitignore"
if [[ -f "$gi" ]] && ! grep -q '^\.prometheus/\.writer\.lock$' "$gi" 2>/dev/null; then
  [[ "$dry_run" != "1" ]] && printf '\n# prometheus-context-bootstrap\n.prometheus/.writer.lock\n.prometheus/.review-pending\n' >> "$gi"
  record "SPLICE" ".gitignore" "lock + marker files ignored"
fi

print_report

printf '\nProfile: %s — ' "$profile"
if [[ "$profile" == "lean" ]]; then
  printf 'execution scaffold OMITTED.\n'
  printf 'Safe only if every model reading this repo is a current frontier model.\n'
  printf 'A smaller model here fabricates identifiers and elides code silently.\n'
else
  printf 'execution scaffold INCLUDED.\n'
  printf 'Costs a frontier model some tokens. Prevents defect escape on smaller ones.\n'
fi
printf 'Record the fleet in .prometheus/model-fleet.md and measure before changing.\n'

if [[ "$dry_run" == "1" ]]; then
  printf '\nDRY RUN — nothing was written.\n'
else
  printf '\nNext: run scripts/verify.sh --path %s, then /kbd-init, then /doctor.\n' "$project_path"
fi
printf 'Completed prometheus-context-bootstrap — %s\n' "$project_path"
