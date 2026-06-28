#!/usr/bin/env bash
# kbd-goal-zed-detect.sh — Detect whether Zed is running with an ACP-connected
# agent or in standalone (own LLM) mode.
#
# Output (stdout, one word):
#   claude-code   → Zed is connected to Claude Code via ACP
#   codex         → Zed is connected to Codex via ACP
#   opencode      → Zed is connected to OpenCode via ACP
#   standalone    → Zed is using its own built-in model (no ACP agent)
#
# Exit codes:
#   0  → detection succeeded
#   1  → could not determine (treat as standalone)

set -euo pipefail

# ── 1. Check $ZED_ACP_AGENT env var (set by Zed when launching ACP session) ──
if [[ -n "${ZED_ACP_AGENT:-}" ]]; then
  agent_lower=$(echo "$ZED_ACP_AGENT" | tr '[:upper:]' '[:lower:]')
  case "$agent_lower" in
    claude*|claude-code*)  echo "claude-code"; exit 0 ;;
    codex*)                echo "codex";       exit 0 ;;
    opencode*)             echo "opencode";    exit 0 ;;
    *)                     echo "standalone";  exit 0 ;;
  esac
fi

# ── 2. Check ~/.zed/acp-agents.json for an active connection ─────────────────
ACP_AGENTS_FILE="$HOME/.zed/acp-agents.json"
if [[ -f "$ACP_AGENTS_FILE" ]]; then
  if command -v jq &>/dev/null; then
    active_agent=$(jq -r '
      .agents[]?
      | select(.status == "active")
      | .name // .id
      | ascii_downcase
    ' "$ACP_AGENTS_FILE" 2>/dev/null | head -1 || echo "")

    if [[ -n "$active_agent" ]]; then
      case "$active_agent" in
        claude*|claude-code*) echo "claude-code"; exit 0 ;;
        codex*)               echo "codex";       exit 0 ;;
        opencode*)            echo "opencode";    exit 0 ;;
      esac
    fi
  fi
fi

# ── 3. Check config/zed/settings.json for assistant.provider ────────────────
ZED_SETTINGS="$HOME/.config/zed/settings.json"
if [[ -f "$ZED_SETTINGS" ]] && command -v jq &>/dev/null; then
  provider=$(jq -r '.assistant.provider.type // .assistant.provider // ""' \
    "$ZED_SETTINGS" 2>/dev/null | tr '[:upper:]' '[:lower:]' || echo "")

  case "$provider" in
    anthropic*|claude*)  echo "claude-code"; exit 0 ;;
    openai*|codex*)      echo "codex";       exit 0 ;;
    opencode*)           echo "opencode";    exit 0 ;;
  esac
fi

# ── 4. Default: standalone ────────────────────────────────────────────────────
echo "standalone"
exit 0
