---
name: learn-harness
description: Per-harness capability orientation for the Prometheus Skill Pack. Auto-detects the running AI harness via detect-surface-tier.sh, emits a capability map, and optionally routes into the Feynman loop for deeper harness understanding. Covers Claude Code, OpenCode, Codex, Kimi Code, and Zed.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, harness, claude-code, opencode, codex, kimi, zed, capability-map, orientation]
---

# learn-harness

Per-harness capability orientation for the Prometheus Skill Pack. Tells you
exactly what the skill pack can do on your current AI harness and optionally
routes you into the Feynman loop for deeper understanding.

## When to invoke

```
/learn-harness [--harness claude-code|opencode|codex|kimi|zed] [--map-only]
```

| Flag | Description |
|---|---|
| `--harness <name>` | Force a specific harness. Skips auto-detection. |
| `--map-only` | Print the capability map and exit. Do not offer Feynman routing. |

Without `--harness`: the skill auto-detects via `detect-surface-tier.sh`.
Without `--map-only`: after the map, the skill offers to route into the
Feynman loop for a 30–60 minute guided orientation.

## Auto-detection

```bash
TIER_JSON=$(bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/detect-surface-tier.sh --json)
HARNESS=$(echo "$TIER_JSON" | jq -r '.harness')
TIER=$(echo "$TIER_JSON" | jq -r '.tier')
```

`detect-surface-tier.sh` returns a JSON object with at minimum:

```json
{ "harness": "claude-code", "tier": 1 }
```

Supported harness values: `claude-code`, `opencode`, `codex`, `kimi`, `zed`.
When the harness is unknown the skill defaults to `tier: 0` (text-only) and
warns the user that orientation may be incomplete.

## Capability map

The capability map is the primary output. Print it immediately after detecting
the harness. Highlight the row for the detected (or forced) harness.

| Capability | Claude Code | OpenCode | Codex | Kimi Code | Zed |
|---|---|---|---|---|---|
| Skills (agentskills.io) | ✓ | ✓ | ✓ | ✓ | ✓ |
| /skill-name invocation | ✓ | ✓ | ✓ | ✓ | partial |
| MCP servers | ✓ | ✓ | ✗ | partial | ✗ |
| AskUserQuestion (Tier 1) | ✓ | ✗ | ✗ | ✗ | ✗ |
| File-pair UI (Tier 1 alt) | ✗ | ✓ | ✓ | ✓ | ✗ |
| Hooks (PostToolUse, Stop) | ✓ | partial | ✗ | ✗ | ✗ |
| Subagents | ✓ | ✓ | ✓ | partial | ✗ |
| surreal-memory MCP | ✓ | ✓ | ✗ | partial | ✗ |
| sycophancy-correction | ✓ | partial | ✗ | partial | ✗ |
| learn domain skills | ✓ | ✓ | ✓ | ✓ | Tier 0 |
| feynman-loop | ✓ | ✓ | ✓ | ✓ | ✓ (text only) |
| learn-certify | ✓ | ✓ | ✓ | ✓ | ✓ |
| learn-retain | ✓ | ✓ | ✓ | ✓ | ✓ (text only) |

Legend: ✓ full support · partial = works with caveats · ✗ not supported

After printing the map, print one sentence stating the detected tier:
```
Detected harness: <harness> (Tier <N>)
```

## Per-harness orientation

Print only the section for the detected (or forced) harness.

### Claude Code

Full capability harness. All skills load from `~/.claude/skills/`, all MCP
servers configure via `.mcp.json`, hooks fire on PostToolUse and Stop, and
AskUserQuestion enables Tier 1 interactive UI within the assistant panel.
Subagents run with full tool access. The surreal-memory and
sycophancy-correction MCP servers integrate automatically when configured.
This is the reference implementation harness — all skills are developed and
tested here first.

Install path: `~/.claude/skills/`

### OpenCode

Near-parity with Claude Code for skills and subagents. Uses the file-pair
convention for Tier 1 UI: the skill writes `__ui_intent__.json`, the harness
renders it, and the user's response lands in `__ui_response__.json`. MCP
servers configure via `~/.opencode/config.json`. Stop hooks are supported;
PostToolUse hooks are not. surreal-memory and sycophancy-correction work when
the MCP server is running and configured.

Install path: `~/.opencode/skills/`

### Codex (OpenAI)

Skills work via the agentskills.io format. No MCP server support — neither
surreal-memory nor sycophancy-correction are available. No hooks. All UI
degrades to Tier 0 (plain text in the chat thread). Subagents are supported.
The full Feynman loop operates at Tier 0. Install skills to `~/.codex/skills/`.

Install path: `~/.codex/skills/`

### Kimi Code

Skills and subagents supported. Partial MCP support — configurable via
`~/.kimi-code/config.toml`. File-pair Tier 1 UI works using the same
`__ui_intent__.json` / `__ui_response__.json` convention as OpenCode. Hooks
are not supported. surreal-memory and sycophancy-correction load when the MCP
server is running, but reliability varies across Kimi versions.

Install path: `~/.kimi-code/skills/`

### Zed

Skills load when Zed's AI assistant is configured to read from a skills
directory (configuration varies by Zed version). No MCP servers, no hooks, no
subagents. All learn-* skills degrade to Tier 0 text-only mode — interactive
UI, file-pair exchanges, and AskUserQuestion are all unavailable. The Feynman
loop works entirely through markdown text in the assistant panel. Skill
invocation via `/skill-name` is partial — support depends on Zed's slash
command integration.

Install path: `~/.zed/skills/` (when configured)

## Feynman loop routing

When `--map-only` is NOT passed, offer the following after displaying the map
and orientation paragraph:

```
Would you like to learn more about [harness] via the Feynman loop?
This will take 30–60 minutes and produce a mastery certificate.

Type yes to begin, or no to stop here.
```

If the user confirms: invoke `learn-about-system` with:
- `--area harness`
- `--system <detected-or-forced-harness>`

This pre-fills the harness so the Feynman loop begins immediately on the
correct subject without a separate selection step.

If the user declines: print the path to the detailed parity reference:

```
Full parity details: skills/learn/learn-harness/references/harness-parity.md
```

## Reference

- Detailed cross-harness parity: [harness-parity.md](references/harness-parity.md)
- Surface tier detection: `shared/scripts/detect-surface-tier.sh`
- UI rendering: `skills/learn/ui-surface/scripts/render.sh`
