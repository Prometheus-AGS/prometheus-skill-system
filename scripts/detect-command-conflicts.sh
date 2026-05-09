#!/usr/bin/env bash
# SP-017: detect slash command name collisions across installed packs.
# Usage: bash scripts/detect-command-conflicts.sh [dir1] [dir2] ...
#        If no dirs given, auto-discovers from known pack locations.

set -euo pipefail

FOUND_CONFLICTS=0

# Collect .claude/commands directories to inspect
COMMANDS_DIRS=()
if [[ $# -gt 0 ]]; then
  for d in "$@"; do COMMANDS_DIRS+=("$d"); done
else
  # Auto-discover: cwd and sibling repos
  CWD="$(pwd)"
  for candidate in \
    "${CWD}/.claude/commands" \
    "${HOME}/.claude/commands" \
    "$(dirname "$CWD")/prometheus-knowledge/.claude/commands" \
    "$(dirname "$CWD")/prometheus-skill-pack/.claude/commands"; do
    [[ -d "$candidate" ]] && COMMANDS_DIRS+=("$candidate")
  done
fi

if [[ ${#COMMANDS_DIRS[@]} -eq 0 ]]; then
  echo "No .claude/commands directories found to inspect."
  exit 0
fi

echo "Scanning ${#COMMANDS_DIRS[@]} commands directory/directories..."
echo ""

# Collect all command basenames (strip .md, strip subdir prefix)
declare -A SEEN  # basename -> "dir1,dir2,..."

for dir in "${COMMANDS_DIRS[@]}"; do
  while IFS= read -r -d '' file; do
    base=$(basename "$file" .md)
    if [[ -v "SEEN[$base]" ]]; then
      SEEN["$base"]="${SEEN[$base]},${dir}"
    else
      SEEN["$base"]="$dir"
    fi
  done < <(find "$dir" -name "*.md" -print0 2>/dev/null)
done

# Report conflicts
for base in "${!SEEN[@]}"; do
  dirs="${SEEN[$base]}"
  if [[ "$dirs" == *","* ]]; then
    echo "CONFLICT: /$base"
    echo "  defined in:"
    IFS=',' read -ra locs <<< "$dirs"
    for loc in "${locs[@]}"; do echo "    $loc"; done
    echo ""
    FOUND_CONFLICTS=1
  fi
done

if [[ "$FOUND_CONFLICTS" -eq 0 ]]; then
  echo "No slash command conflicts found."
fi

exit "$FOUND_CONFLICTS"
