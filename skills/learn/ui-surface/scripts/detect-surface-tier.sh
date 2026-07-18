#!/usr/bin/env bash
# detect-surface-tier.sh — Probe for the highest reliably-available UI surface tier.
#
# Usage:
#   eval "$(bash detect-surface-tier.sh)"        # sets $SURFACE_TIER in calling shell
#   bash detect-surface-tier.sh --print          # prints tier to stdout
#   bash detect-surface-tier.sh --json           # prints JSON: {"tier":"tier1_structured","harness":"claude-code"}
#
# Output tiers: tier0_text | tier1_structured | tier2_mcp_app | tier3_full
# See docs/learn/surface-tier-detection.md for full specification.

set -euo pipefail

PRINT_MODE="${1:-}"
TIER="tier0_text"
HARNESS="unknown"

# ── Tier 2: surface-bridge MCP App server ─────────────────────────────────────
_check_tier2() {
  local pid_file="${HOME}/.prometheus/run/surface-bridge.pid"
  if [ -f "$pid_file" ]; then
    local pid
    pid=$(cat "$pid_file" 2>/dev/null || echo "")
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      if [ "${CLAUDE_MCP_APP_CAPABLE:-0}" = "1" ]; then
        echo "tier2_mcp_app"
        return 0
      fi
    fi
  fi
  return 1
}

# ── Claude Code ───────────────────────────────────────────────────────────────
if [ -n "${CLAUDE_CODE:-}" ] || [ -n "${CLAUDE_CODE_VERSION:-}" ] || \
   [ -n "${ANTHROPIC_CLAUDE_CODE:-}" ]; then
  HARNESS="claude-code"
  TIER="tier1_structured"
  if _check_tier2 2>/dev/null; then
    TIER="tier2_mcp_app"
  fi
fi

# ── OpenCode ──────────────────────────────────────────────────────────────────
if [ "$HARNESS" = "unknown" ] && \
   { [ -n "${OPENCODE:-}" ] || [ -n "${OPENCODE_VERSION:-}" ]; }; then
  HARNESS="opencode"
  TIER="tier1_structured"
fi

# ── Codex (OpenAI) ────────────────────────────────────────────────────────────
if [ "$HARNESS" = "unknown" ] && \
   { [ -n "${CODEX:-}" ] || [ -n "${OPENAI_CODEX:-}" ]; }; then
  HARNESS="codex"
  TIER="tier1_structured"
fi

# ── Kimi Code ─────────────────────────────────────────────────────────────────
if [ "$HARNESS" = "unknown" ] && \
   { [ -n "${KIMI_CODE:-}" ] || [ -n "${KIMI_CODE_VERSION:-}" ]; }; then
  HARNESS="kimi"
  TIER="tier1_structured"
fi

# ── Zed ───────────────────────────────────────────────────────────────────────
if [ "$HARNESS" = "unknown" ] && [ -n "${ZED_AI_CONTEXT:-}" ]; then
  HARNESS="zed"
  TIER="tier0_text"
fi

# ── Cursor ────────────────────────────────────────────────────────────────────
if [ "$HARNESS" = "unknown" ] && [ -n "${CURSOR_AI:-}" ]; then
  HARNESS="cursor"
  TIER="tier0_text"
fi

# ── Fallback ──────────────────────────────────────────────────────────────────
# tier0_text, harness=unknown — safe floor

# ── Output ────────────────────────────────────────────────────────────────────
case "${PRINT_MODE}" in
  --json)
    printf '{"tier":"%s","harness":"%s"}\n' "$TIER" "$HARNESS"
    ;;
  --print)
    echo "$TIER"
    ;;
  *)
    # Default: emit shell assignment for eval
    printf 'export SURFACE_TIER="%s"\nexport SURFACE_HARNESS="%s"\n' "$TIER" "$HARNESS"
    ;;
esac
