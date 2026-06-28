#!/usr/bin/env bash
# kbd-goal-codex-setup.sh — One-time Codex bridge setup for kbd-goal
# Copies KBD goal prompt templates to ~/.codex/goals/ and enables goal mode.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATES_DIR="$REPO_ROOT/skills/process/kbd-goal/templates/codex"
CODEX_GOALS_DIR="$HOME/.codex/goals"
CODEX_CONFIG="$HOME/.codex/config.toml"

# ── Check codex is installed ──────────────────────────────────────────────────
if ! command -v codex &>/dev/null; then
  echo "Codex CLI not found. Install from: https://github.com/openai/codex" >&2
  exit 1
fi

CODEX_VERSION=$(codex --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "0.0.0")
REQUIRED="0.128.0"
if [[ "$(printf '%s\n' "$REQUIRED" "$CODEX_VERSION" | sort -V | head -1)" != "$REQUIRED" ]]; then
  echo "Codex CLI $CODEX_VERSION < $REQUIRED required for /goal support" >&2
  exit 1
fi
echo "✓ Codex CLI $CODEX_VERSION detected"

# ── Enable goal mode in config ────────────────────────────────────────────────
mkdir -p "$(dirname "$CODEX_CONFIG")"
if [[ -f "$CODEX_CONFIG" ]]; then
  if grep -q "goals.enabled" "$CODEX_CONFIG"; then
    sed -i.bak 's/goals.enabled.*=.*/goals.enabled = true/' "$CODEX_CONFIG"
  else
    echo "" >> "$CODEX_CONFIG"
    echo "goals.enabled = true" >> "$CODEX_CONFIG"
  fi
else
  cat > "$CODEX_CONFIG" << 'TOML'
goals.enabled = true
TOML
fi
echo "✓ goals.enabled = true in $CODEX_CONFIG"

# ── Install KBD goal prompt templates ─────────────────────────────────────────
mkdir -p "$CODEX_GOALS_DIR"

for template in continuation.md budget_limit.md; do
  src="$TEMPLATES_DIR/$template"
  dst="$CODEX_GOALS_DIR/$template"
  if [[ -f "$src" ]]; then
    cp "$src" "$dst"
    echo "✓ Installed $template → $dst"
  else
    echo "Warning: template not found: $src" >&2
  fi
done

echo ""
echo "Codex bridge setup complete."
echo ""
echo "To activate KBD goal context for a project, add to your AGENTS.md:"
echo "  @include .codex-kbd-context.md"
echo ""
echo "KBD will write .codex-kbd-context.md when you run /kbd-goal on a Codex session."
