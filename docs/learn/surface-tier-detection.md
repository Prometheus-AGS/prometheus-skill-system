# Surface Tier Detection — `detect-surface-tier.sh`

**Version:** 1.0.0
**Used by:** `ui-surface` skill, `surface-bridge` substrate
**Output:** `SURFACE_TIER` environment variable, one of: `tier0_text | tier1_structured | tier2_mcp_app | tier3_full`

---

## Tier Definitions

| Tier | Label | Description | Universal |
|---|---|---|---|
| 0 | `tier0_text` | Plain markdown + file-based prompts | Yes — all harnesses |
| 1 | `tier1_structured` | `AskUserQuestion` (Claude Code) or `__ui_intent__.json` file pair (others) | Most harnesses |
| 2 | `tier2_mcp_app` | MCP App iframe or AG-UI → A2UI spec serving | Claude Code + surface-bridge service |
| 3 | `tier3_full` | Full external browser/desktop panel | Explicit opt-in only |

**Rule:** every skill declares `min_tier: 0` and `preferred_tier: 1`. The probe returns the highest tier reliably available in the current session. Skills silently fall to the highest available tier.

---

## Detection Signals Per Harness

### Claude Code
- `$CLAUDE_CODE` or `$CLAUDE_CODE_VERSION` set in environment → at minimum `tier1_structured`
- `AskUserQuestion` tool available in session → confirmed `tier1_structured`
- `surface-bridge` MCP server running (check `~/.prometheus/run/surface-bridge.pid`) → `tier2_mcp_app`

### OpenCode
- `$OPENCODE` or `$OPENCODE_VERSION` set → `tier1_structured` (file pair convention)
- No native `AskUserQuestion` → uses `__ui_intent__.json` + `__ui_response__.json` file pair

### Codex (OpenAI)
- `$CODEX` or `$OPENAI_CODEX` set → `tier1_structured` (file pair convention)
- Same file pair convention as OpenCode

### Kimi Code
- `$KIMI_CODE` or `$KIMI_CODE_VERSION` set → `tier1_structured` (file pair convention)

### Zed
- `$ZED_AI_CONTEXT` set → `tier0_text` (Zed has no file-pair or AskUserQuestion convention)
- Note: Zed AI context is text-only; structured prompt delivery not yet standardized

### Cursor
- `$CURSOR_AI` set → `tier0_text` (similar to Zed; cursor rules are static markdown)

### Unknown / fallback
- None of the above signals present → `tier0_text`

---

## File Pair Convention (Tier 1, non-Claude Code harnesses)

When `tier1_structured` is detected on a non-Claude Code harness, `ui-surface` uses the file pair protocol:

```
Write:  __ui_intent__.json   (intent + content + options)
Wait:   __ui_response__.json (operator response, written by human or harness)
```

`__ui_intent__.json` schema:
```json
{
  "intent_type": "survey|explanation|grading|review|report|kb_query",
  "prompt": "...",
  "options": ["option A", "option B"],
  "free_text": true,
  "session_id": "uuid"
}
```

`__ui_response__.json` schema:
```json
{
  "session_id": "uuid",
  "selected_option": "option A",
  "free_text_response": "...",
  "responded_at": "ISO8601"
}
```

The skill polls for `__ui_response__.json` existence with a configurable timeout (default 5 minutes). On timeout, falls to `tier0_text` and asks the operator to continue via chat.

---

## SURFACE_TIER Environment Variable

The probe sets `SURFACE_TIER` in the calling shell:

```bash
export SURFACE_TIER="tier1_structured"
```

Skills source or call the probe:
```bash
eval "$(bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/detect-surface-tier.sh)"
# Now $SURFACE_TIER is available
```

---

## Tier 2 Detection

Tier 2 requires the `surface-bridge` MCP App server to be running. Detection:

```bash
if [ -f "$HOME/.prometheus/run/surface-bridge.pid" ]; then
  PID=$(cat "$HOME/.prometheus/run/surface-bridge.pid")
  if kill -0 "$PID" 2>/dev/null; then
    # surface-bridge is running; check if MCP App capability is present in session
    # MCP App capability is signaled by $CLAUDE_MCP_APP_CAPABLE=1
    if [ "${CLAUDE_MCP_APP_CAPABLE:-0}" = "1" ]; then
      echo "tier2_mcp_app"
      exit 0
    fi
  fi
fi
```

`$CLAUDE_MCP_APP_CAPABLE` is expected to be set by Claude Code when the session includes a connected MCP App server. This environment variable name is provisional pending Claude Code API confirmation — skills must treat Tier 2 detection as best-effort.

---

## Tier 3

Tier 3 (full external surface) requires explicit operator opt-in via `--tier 3` flag on the invoking skill. It is never auto-detected. Not implemented in this phase.
