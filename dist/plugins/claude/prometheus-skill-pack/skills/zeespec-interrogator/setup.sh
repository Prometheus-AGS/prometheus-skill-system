#!/usr/bin/env bash
# setup.sh — One-time setup for zeespec-interrogator
# Run this once after cloning or installing the skill pack.
# Usage: bash skills/process/zeespec-interrogator/setup.sh

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔧 Setting up zeespec-interrogator..."

# Make all scripts executable
chmod +x "${SKILL_DIR}/scripts/"*.sh
echo "  ✅ Scripts marked executable"

# Validate JSON files
echo "  🔍 Validating JSON files..."
for f in \
  "${SKILL_DIR}/hooks/hooks.json" \
  "${SKILL_DIR}/.mcp.json" \
  "${SKILL_DIR}/references/schemas/constraint-manifest.schema.json" \
  "${SKILL_DIR}/references/schemas/interrogation-state.schema.json" \
  "${SKILL_DIR}/references/schemas/coverage-score.schema.json"; do
  python3 -c "import json; json.load(open('$f')); print(f'  ✅ $(basename $f)')" 2>/dev/null \
    || echo "  ❌ $(basename $f) — invalid JSON"
done

# Validate dimension files have 10 questions each
echo "  🔍 Validating dimension files..."
for dim in what where who when why how; do
  f="${SKILL_DIR}/references/dimensions/${dim}.md"
  count=$(grep -c "^## Q" "$f" 2>/dev/null || echo 0)
  if [ "$count" -eq 10 ]; then
    echo "  ✅ ${dim}: 10 questions"
  else
    echo "  ❌ ${dim}: ${count} questions (expected 10)"
  fi
done

# Validate SKILL.md frontmatter in all sub-skills
echo "  🔍 Validating sub-skill frontmatter..."
for f in "${SKILL_DIR}/skills/"*/SKILL.md; do
  head -3 "$f" | grep -q "^---" \
    && echo "  ✅ $(basename $(dirname $f))/SKILL.md" \
    || echo "  ❌ $(basename $(dirname $f))/SKILL.md — missing frontmatter"
done

# Test provider resolution
echo "  🔍 Testing state provider resolution..."
PROVIDER=$(bash "${SKILL_DIR}/scripts/state-resolve-provider.sh" 2>/dev/null)
if echo "$PROVIDER" | python3 -c "import json,sys; json.load(sys.stdin); print('  ✅ Provider resolved:', json.loads(open('/dev/stdin').read() if False else '$PROVIDER').get('provider_type','?'))" 2>/dev/null; then
  :
else
  PROVIDER_TYPE=$(echo "$PROVIDER" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('provider_type','unknown'))" 2>/dev/null || echo "filesystem")
  echo "  ✅ Provider resolved: ${PROVIDER_TYPE}"
fi

echo ""
echo "✅ zeespec-interrogator setup complete."
echo ""
echo "Quick start:"
echo "  /zeespec-interrogate \"my-system-name\""
echo "  /zeespec-status \"my-system-name\""
