#!/usr/bin/env bash
# skills/kbd-inject-agent-rules/kbd-inject-agent-rules.sh
# Inject a fenced "Agent rules" block into CLAUDE.md and/or AGENTS.md.

set -euo pipefail
die() { printf 'kbd-inject-agent-rules: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-inject-agent-rules: warn: %s\n' "$*" >&2; }

KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
HERE="$KBD_ORCHESTRATOR_ROOT/skills/kbd-inject-agent-rules"

# Defaults
target="both"
project_path="."
refresh=0
dry_run=0
pack="agent-rules"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)   target="${2:?--target requires a value}"; shift 2 ;;
    --path)     project_path="${2:?--path requires a value}"; shift 2 ;;
    --pack)     pack="${2:?--pack requires a value}"; shift 2 ;;
    --refresh)  refresh=1; shift ;;
    --dry-run)  dry_run=1; shift ;;
    -h|--help)
      cat <<USAGE
Usage: $0 [--target CLAUDE.md|AGENTS.md|both] [--path <root>] [--pack agent-rules|uiux-routing] [--refresh] [--dry-run]
USAGE
      exit 0 ;;
    *) die "unknown flag: $1" ;;
  esac
done

# Resolve template + cache + marker prefix from pack.
case "$pack" in
  agent-rules)
    marker_prefix="agent-rules"
    TEMPLATE="$HERE/references/template-agent-rules.md"
    CACHE="$HERE/references/cache-agent-rules.md"
    # Back-compat: fall back to the pre-pack file names.
    [[ -f "$TEMPLATE" ]] || TEMPLATE="$HERE/references/template.md"
    [[ -f "$CACHE"    ]] || CACHE="$HERE/references/rules-cache.md"
    ;;
  uiux-routing)
    marker_prefix="uiux-routing"
    TEMPLATE="$HERE/references/template-uiux-routing.md"
    CACHE="$HERE/references/cache-uiux-routing.md"
    # Prefer project-local roster when present (design D7).
    if [[ -f "$project_path/.kbd-orchestrator/references/uiux-skill-roster.md" ]]; then
      CACHE="$project_path/.kbd-orchestrator/references/uiux-skill-roster.md"
    fi
    ;;
  *)
    die "--pack must be agent-rules or uiux-routing (got: $pack)"
    ;;
esac

[[ -f "$TEMPLATE" ]] || die "template missing: $TEMPLATE"
[[ -f "$CACHE"    ]] || die "cache missing: $CACHE"

START_MARK="<!-- ${marker_prefix}:start v1 -->"
END_MARK="<!-- ${marker_prefix}:end -->"

case "$target" in
  CLAUDE.md|AGENTS.md|both) : ;;
  *) die "--target must be CLAUDE.md, AGENTS.md, or both (got: $target)" ;;
esac

[[ -d "$project_path" ]] || die "--path is not a directory: $project_path"

# Build target list
targets=()
case "$target" in
  CLAUDE.md) targets=("$project_path/CLAUDE.md") ;;
  AGENTS.md) targets=("$project_path/AGENTS.md") ;;
  both)      targets=("$project_path/CLAUDE.md" "$project_path/AGENTS.md") ;;
esac

# Refresh (best-effort)
if [[ "$refresh" == "1" ]]; then
  if ! command -v curl >/dev/null 2>&1; then
    warn "--refresh requested but curl is missing; skipping validation"
  else
    while IFS= read -r line; do
      # Parse "- <url> — anchor: \`<keyword>\`" lines.
      url="$(printf '%s' "$line" | sed -n 's/^- \(https[^ ]*\).*/\1/p')"
      anchor="$(printf '%s' "$line" | sed -n 's/.*anchor: `\(.*\)`.*/\1/p')"
      [[ -n "$url" && -n "$anchor" ]] || continue
      body="$(curl -fsS --max-time 10 "$url" 2>/dev/null || true)"
      if [[ -z "$body" ]]; then
        warn "refresh: $url unreachable; cache not updated for this source"
        continue
      fi
      if ! printf '%s' "$body" | grep -qF "$anchor"; then
        warn "refresh: $url no longer contains anchor '$anchor' — review rules-cache.md"
      fi
    done < <(grep '^- https' "$CACHE")
    # Stamp fetch dates (light touch — only update the "Last fetched:" lines).
    now="$(date -u +%Y-%m-%d)"
    sed -i.bak "s/^Last fetched: .*/Last fetched: $now/" "$CACHE" && rm -f "$CACHE.bak"
  fi
fi

# Read template content (the part between start/end markers, inclusive).
new_block="$(cat "$TEMPLATE")"

# Validate template self-consistency
grep -qF "$START_MARK" "$TEMPLATE" || die "template missing start marker"
grep -qF "$END_MARK"   "$TEMPLATE" || die "template missing end marker"

updated=0
unchanged=0

print_signal_start() {
  printf 'Starting kbd-inject-agent-rules — pack=%s target=%s\n' "$pack" "$target"
}
print_signal_complete() {
  printf 'Completed kbd-inject-agent-rules — %d file(s) updated, %d unchanged\n' "$updated" "$unchanged"
}

print_signal_start

for t in "${targets[@]}"; do
  if [[ ! -f "$t" ]]; then
    # Create empty file so we can append cleanly
    : > "$t"
  fi

  # grep -c always outputs an integer; under set -e, no-match (exit 1) would
  # abort, so we use the trailing : sentinel to make the subshell exit 0.
  starts="$(grep -cF "$START_MARK" "$t" 2>/dev/null; :)"
  ends="$(grep -cF "$END_MARK"     "$t" 2>/dev/null; :)"

  if [[ "$starts" -gt 1 ]]; then
    die "$t contains $starts start markers — refuse to write; dedupe by hand"
  fi
  if [[ "$starts" == "1" && "$ends" == "0" ]]; then
    die "$t contains a start marker without an end marker — refuse to write; repair by hand"
  fi
  if [[ "$starts" == "0" && "$ends" -gt 0 ]]; then
    die "$t contains end marker(s) without a matching start — refuse"
  fi

  tmp="$t.tmp.$$"
  if [[ "$starts" == "1" ]]; then
    # Replace between markers (inclusive). awk's -v cannot carry newlines, so
    # we splice the block via a file.
    block_file="$t.block.$$"
    printf '%s\n' "$new_block" > "$block_file"
    awk -v start="$START_MARK" -v end="$END_MARK" -v bf="$block_file" '
      function emit_block(   line) {
        while ((getline line < bf) > 0) print line
        close(bf)
      }
      BEGIN { in_block = 0; emitted = 0 }
      {
        if ($0 == start) { in_block = 1; if (!emitted) { emit_block(); emitted = 1 } ; next }
        if (in_block && $0 == end) { in_block = 0; next }
        if (!in_block) print
      }
    ' "$t" > "$tmp"
    rm -f "$block_file"
  else
    # First write: append with a separating blank line if file is non-empty.
    {
      cat "$t"
      if [[ -s "$t" ]] && [[ "$(tail -c1 "$t" | wc -l | tr -d ' ')" == "0" ]]; then printf '\n'; fi
      [[ -s "$t" ]] && printf '\n'
      printf '%s\n' "$new_block"
    } > "$tmp"
  fi

  if [[ "$dry_run" == "1" ]]; then
    if cmp -s "$t" "$tmp"; then
      printf '%s: no change\n' "$t"
    else
      printf -- '--- %s (current)\n+++ %s (proposed)\n' "$t" "$t"
      diff -u "$t" "$tmp" || true
    fi
    rm -f "$tmp"
    continue
  fi

  if cmp -s "$t" "$tmp"; then
    rm -f "$tmp"
    unchanged=$((unchanged + 1))
    printf '%s: unchanged\n' "$t"
  else
    mv -f "$tmp" "$t"
    updated=$((updated + 1))
    printf '%s: updated\n' "$t"
  fi
done

print_signal_complete
