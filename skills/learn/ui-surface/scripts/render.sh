#!/usr/bin/env bash
# render.sh — Tier-aware UI intent renderer for learn-* skills.
#
# Usage:
#   render.sh --tier <tier> --intent-json '<json>'
#
# Tiers: tier0_text | tier1_structured | tier2_mcp_app
# Intent JSON schema: see skills/learn/ui-surface/SKILL.md

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────
TIER=""
INTENT_JSON=""

while [ $# -gt 0 ]; do
  case "$1" in
    --tier)
      TIER="$2"
      shift 2
      ;;
    --intent-json)
      INTENT_JSON="$2"
      shift 2
      ;;
    *)
      echo "[ui-surface] Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ -z "$TIER" ] || [ -z "$INTENT_JSON" ]; then
  echo "[ui-surface] Error: --tier and --intent-json are required" >&2
  exit 1
fi

# ── Extract intent fields ─────────────────────────────────────────────────────
INTENT_TYPE=$(echo "$INTENT_JSON" | jq -r '.intent_type // "prompt"')
TITLE=$(echo "$INTENT_JSON" | jq -r '.title // ""')
BODY=$(echo "$INTENT_JSON" | jq -r '.body // ""')
# Build options as a shell array
OPTIONS_JSON=$(echo "$INTENT_JSON" | jq -r '(.options // []) | @json')

# ── Tier 0 renderer ───────────────────────────────────────────────────────────
_render_tier0() {
  case "$INTENT_TYPE" in
    question)
      echo ""
      echo "**Q: ${TITLE}**"
      if [ "$OPTIONS_JSON" != "[]" ] && [ "$OPTIONS_JSON" != "null" ]; then
        echo ""
        local i=1
        while IFS= read -r opt; do
          echo "${i}. ${opt}"
          i=$((i + 1))
        done < <(echo "$OPTIONS_JSON" | jq -r '.[]')
        echo ""
        echo "Reply with the number of your choice."
      else
        [ -n "$BODY" ] && echo "" && echo "$BODY"
      fi
      ;;
    prompt)
      echo ""
      echo "---"
      [ -n "$BODY" ] && echo "$BODY"
      echo "---"
      echo ""
      ;;
    feedback)
      echo ""
      echo "> [Feedback] ${BODY}"
      echo ""
      ;;
    progress)
      echo ""
      echo "## Progress"
      echo ""
      [ -n "$BODY" ] && echo "$BODY"
      echo ""
      ;;
    *)
      echo ""
      echo "**${TITLE}**"
      [ -n "$BODY" ] && echo "" && echo "$BODY"
      echo ""
      ;;
  esac
}

# ── Tier 1 renderer — Claude Code ─────────────────────────────────────────────
_render_tier1_claude_code() {
  if [ "$INTENT_TYPE" = "question" ]; then
    echo ""
    echo "[QUESTION — ${TITLE}]"
    [ -n "$BODY" ] && echo "$BODY"
    if [ "$OPTIONS_JSON" != "[]" ] && [ "$OPTIONS_JSON" != "null" ]; then
      local opts=""
      local i=1
      while IFS= read -r opt; do
        opts="${opts}(${i}) ${opt}  "
        i=$((i + 1))
      done < <(echo "$OPTIONS_JSON" | jq -r '.[]')
      echo "Options: ${opts}"
    fi
    echo "Your response:"
    echo ""
  else
    # Non-question intents fall back to Tier 0 on Claude Code
    _render_tier0
  fi
}

# ── Tier 1 renderer — file-pair harnesses ─────────────────────────────────────
_render_tier1_file_pair() {
  local ui_dir="${HOME}/.prometheus/learn/ui"
  mkdir -p "$ui_dir"

  local intent_file="${ui_dir}/__ui_intent__.json"
  local response_file="${ui_dir}/__ui_response__.json"

  # Write intent
  echo "$INTENT_JSON" > "$intent_file"

  # Poll for response (2 s intervals, 30 s timeout)
  local elapsed=0
  while [ ! -f "$response_file" ] && [ "$elapsed" -lt 30 ]; do
    sleep 2
    elapsed=$((elapsed + 2))
  done

  if [ -f "$response_file" ]; then
    cat "$response_file"
    rm -f "$intent_file" "$response_file"
  else
    rm -f "$intent_file"
    echo '{"error":"timeout","message":"No response received within 30 seconds"}'
  fi
}

# ── Detect harness for Tier 1 routing ─────────────────────────────────────────
_detect_harness() {
  if [ -n "${CLAUDE_CODE:-}" ] || [ -n "${CLAUDE_CODE_VERSION:-}" ] || \
     [ -n "${ANTHROPIC_CLAUDE_CODE:-}" ]; then
    echo "claude-code"
  elif [ -n "${OPENCODE:-}" ] || [ -n "${OPENCODE_VERSION:-}" ]; then
    echo "opencode"
  elif [ -n "${CODEX:-}" ] || [ -n "${OPENAI_CODEX:-}" ]; then
    echo "codex"
  elif [ -n "${KIMI_CODE:-}" ] || [ -n "${KIMI_CODE_VERSION:-}" ]; then
    echo "kimi"
  elif [ -n "${ZED_AI_CONTEXT:-}" ]; then
    echo "zed"
  elif [ -n "${CURSOR_AI:-}" ]; then
    echo "cursor"
  else
    echo "unknown"
  fi
}

# ── Dispatch ──────────────────────────────────────────────────────────────────
case "$TIER" in
  tier0_text)
    _render_tier0
    ;;

  tier1_structured)
    HARNESS=$(_detect_harness)
    case "$HARNESS" in
      claude-code)
        _render_tier1_claude_code
        ;;
      opencode|codex|kimi|zed|cursor)
        # zed and cursor were both detected by _detect_harness but omitted here,
        # so each silently fell through to Tier 0. The file-pair handshake is
        # harness-agnostic — it is two files on disk — so there was no mechanism
        # reason to exclude either, only an oversight in this list.
        _render_tier1_file_pair
        ;;
      *)
        # Unknown or text-only harness — fall back to Tier 0
        _render_tier0
        ;;
    esac
    ;;

  tier2_mcp_app)
    SURFACE_URL="${SURFACE_BRIDGE_URL:-http://127.0.0.1:7890}"
    if ! curl -fsS --connect-timeout 1 --max-time 2 "$SURFACE_URL/health" >/dev/null 2>&1; then
      echo "[ui-surface] surface-bridge unreachable — falling back to Tier 1" >&2
      HARNESS=$(_detect_harness)
      case "$HARNESS" in
        claude-code) _render_tier1_claude_code ;;
        opencode|codex|kimi|zed|cursor) _render_tier1_file_pair ;;
        *) _render_tier0 ;;
      esac
      exit 0
    fi

    REQUEST_ID=$(echo "$INTENT_JSON" | jq -r '.request_id // empty')
    if [ -z "$REQUEST_ID" ]; then
      REQUEST_ID="ui-$(date -u +%s)-$$"
    fi
    RENDER_INTENT=$(echo "$INTENT_JSON" | jq -c \
      --arg request_id "$REQUEST_ID" \
      '.request_id=$request_id | .options=(.options // null) | .multiselect=(.multiselect // false)')
    RENDER_RESPONSE=$(curl -fsS --connect-timeout 1 --max-time 5 \
      -X POST "$SURFACE_URL/mcp/render-ui-intent" \
      -H 'Content-Type: application/json' \
      -d "$RENDER_INTENT") || {
        echo "[ui-surface] surface-bridge render failed — falling back to Tier 0" >&2
        _render_tier0
        exit 0
      }

    if [ "$INTENT_TYPE" != "question" ]; then
      echo "$RENDER_RESPONSE"
      exit 0
    fi

    WAIT_SECONDS="${UI_SURFACE_TIMEOUT:-30}"
    elapsed=0
    while [ "$elapsed" -lt "$WAIT_SECONDS" ]; do
      COLLECT_RESPONSE=$(curl -fsS --connect-timeout 1 --max-time 3 \
        -X POST "$SURFACE_URL/mcp/collect-response" \
        -H 'Content-Type: application/json' \
        -d "{\"request_id\":\"$REQUEST_ID\",\"timeout_secs\":1}") || COLLECT_RESPONSE=''
      if [ -n "$COLLECT_RESPONSE" ] && [ "$(echo "$COLLECT_RESPONSE" | jq -r '.status // empty')" = "ready" ]; then
        echo "$COLLECT_RESPONSE"
        exit 0
      fi
      sleep 1
      elapsed=$((elapsed + 1))
    done
    echo "{\"request_id\":\"$REQUEST_ID\",\"status\":\"timeout\",\"response\":null}"
    ;;

  *)
    echo "[ui-surface] Unknown tier '${TIER}' — falling back to Tier 0" >&2
    _render_tier0
    ;;
esac

exit 0
