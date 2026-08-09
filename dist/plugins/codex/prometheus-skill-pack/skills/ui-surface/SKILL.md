---
name: ui-surface
description: Cross-harness UI surface resolver for learn-* skills. Detects the active surface tier via detect-surface-tier.sh and renders UI intent (prompts, questions, feedback) at the best available tier. Degrades gracefully to Tier 0 text when richer surfaces are unavailable.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [ui, surface, cross-harness, learn, tier-detection]
---

# ui-surface

## Overview

ui-surface is the shared rendering layer for all learn-* skills. No learn skill
renders UI directly — they all invoke ui-surface with a UI intent object.
ui-surface resolves the active surface tier via `detect-surface-tier.sh` and
renders the intent at the best tier available for the current harness.

This design keeps tier logic in one place. Adding a new harness or a new tier
requires only changes to ui-surface and `detect-surface-tier.sh`, not to every
learn skill that shows UI.

## UI Intent Schema

Pass a JSON object with the following shape to `scripts/render.sh`:

```json
{
  "intent_type": "question|prompt|feedback|progress",
  "title": "string",
  "body": "string (markdown)",
  "options": ["string"] | null,
  "multiselect": false,
  "metadata": {}
}
```

Fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `intent_type` | string | yes | `question`, `prompt`, `feedback`, or `progress` |
| `title` | string | yes | Short heading or question text |
| `body` | string | yes | Markdown body shown below the title |
| `options` | array or null | no | List of choices for `question` intents |
| `multiselect` | boolean | no | Whether multiple options can be selected |
| `metadata` | object | no | Arbitrary key-value data for the caller |

## Rendering by Tier

### Tier 0 — text/markdown (universal floor)

Applies when `tier = tier0_text` or when no richer tier is available.

| intent_type | Rendered form |
|---|---|
| `question` | `**Q: <title>**` followed by a numbered option list; user replies with the number |
| `prompt` | `---` separator, markdown body, `---` separator |
| `feedback` | `> [Feedback] <body>` blockquote |
| `progress` | `## Progress` section heading with body |

Example — Tier 0 question:

```
**Q: Which learning style fits you best?**

1. Reading (text and examples)
2. Practice (exercises first)
3. Discussion (Socratic Q&A)

Reply with the number of your choice.
```

### Tier 1 — structured prompt

Applies when `tier = tier1_structured`.

#### Claude Code (`harness: claude-code`)

Use AskUserQuestion-style presentation:

```
[QUESTION — <title>]
<body>
Options: (1) <opt1>  (2) <opt2>  ...
Your response:
```

Non-question intents fall back to Tier 0 formatting on Claude Code because only
`question` has a native structured prompt form.

#### File-pair harnesses (opencode, codex, kimi, zed)

Write the intent JSON to `~/.prometheus/learn/ui/__ui_intent__.json`, then poll
for `~/.prometheus/learn/ui/__ui_response__.json` (2-second intervals, 30-second
timeout). When the response file appears, read it and echo its contents to
stdout, then delete both files.

```bash
UI_DIR="${HOME}/.prometheus/learn/ui"
mkdir -p "$UI_DIR"
echo "$INTENT_JSON" > "${UI_DIR}/__ui_intent__.json"

# Poll for response
ELAPSED=0
while [ ! -f "${UI_DIR}/__ui_response__.json" ] && [ "$ELAPSED" -lt 30 ]; do
  sleep 2
  ELAPSED=$((ELAPSED + 2))
done

if [ -f "${UI_DIR}/__ui_response__.json" ]; then
  cat "${UI_DIR}/__ui_response__.json"
  rm -f "${UI_DIR}/__ui_intent__.json" "${UI_DIR}/__ui_response__.json"
else
  echo '{"error":"timeout","message":"No response received within 30 seconds"}'
fi
```

### Tier 2 — MCP App iframe (surface-bridge)

Tier 2 uses the `surface-bridge` Axum server (`http://127.0.0.1:7890`) to render
a UiIntent as an MCP App iframe embedded in the Claude Code artifact panel.

**Prerequisites:** surface-bridge must be running. The canonical launchd service
`ai.prometheus.surface-bridge` is installed by `scripts/install-mcp-services.sh`.
Check with `curl -s http://127.0.0.1:7890/health`.

**4-step render flow:**

1. `render.sh` POSTs the UiIntent JSON to `/mcp/render-ui-intent`:

   ```bash
   curl -s -X POST http://127.0.0.1:7890/mcp/render-ui-intent \
     -H "Content-Type: application/json" \
     -d '{"intent_type":"question","title":"...","body":"...","options":null,"multiselect":false,"request_id":"<uuid>"}'
   ```

   surface-bridge returns `{"request_id":"<uuid>","status":"rendered","message":null}`.

2. surface-bridge serves an iframe fragment from its embedded HTTP server. Claude
   Code's MCP App panel renders this as an inline artifact.

3. The user interacts with the rendered component. Its adapter POSTs the response
   to `/mcp/submit-response` with the same `request_id`.

4. `render.sh` polls `/mcp/collect-response` with the `request_id` (up to 30 s)
   and returns the JSON response to the calling learn-* skill.

**UiIntent → surface-bridge field mapping (from `prometheus-research` events):**

| AguiEvent type   | `intent_type` | `title`                       | `body`              |
|------------------|---------------|-------------------------------|---------------------|
| `agent.status`   | `progress`    | `"Stage N: <stage_name>"`     | `"<status> (N%)"`   |
| `agent.message`  | `feedback`    | `<level>`                     | `<message>`         |
| `agent.error`    | `feedback`    | `"Error at stage N"`          | `<message>`         |
| `a2ui.component` | `prompt`      | `<component>`                 | `<props JSON>`      |

`request_id` is always the job ID. `options` is `null`. `multiselect` is `false`.

When `tier = tier2_mcp_app` and surface-bridge is unreachable, `render.sh` logs
a warning and falls back to Tier 1 automatically:

```
[ui-surface] surface-bridge unreachable — falling back to Tier 1
```

## How learn-* skills use ui-surface

Every learn-* skill that needs to surface UI to the user follows this pattern:

```bash
# 1. Detect tier
UI_SURFACE_DIR="<directory containing this SKILL.md>"
TIER_JSON=$(bash "${UI_SURFACE_DIR}/scripts/detect-surface-tier.sh" --json)
TIER=$(echo "$TIER_JSON" | jq -r '.tier')

# 2. Invoke ui-surface render
RESPONSE=$(bash "${UI_SURFACE_DIR}/scripts/render.sh" \
  --tier "$TIER" \
  --intent-json '{"intent_type":"question","title":"Which topic next?","body":"Choose the area to explore.","options":["Ownership","Traits","Async"],"multiselect":false,"metadata":{}}')

# 3. Use the response
echo "$RESPONSE"
```

The skill never calls `detect-surface-tier.sh` for rendering decisions — it
passes the tier to `render.sh` which handles all tier logic internally.

## Degradation Rule

Never block on an unavailable tier.

```
Tier 2 unavailable → fall back to Tier 1
Tier 1 unavailable (non-interactive harness) → fall back to Tier 0
Tier 0 is always available — it never fails
```

Render scripts must not exit non-zero due to a missing tier. A degraded render
is always preferable to a failed render.

## Cross-harness Parity

| Harness | Tier 1 mechanism | Notes |
|---|---|---|
| Claude Code | AskUserQuestion structured prompt | `[QUESTION — title]` format |
| OpenCode | file-pair (`__ui_intent__.json` + `__ui_response__.json`) | Polls 30 s |
| Codex | file-pair | Same as OpenCode |
| Kimi | file-pair | Same as OpenCode |
| Zed | Tier 0 only | No structured prompt support |
| Cursor | Tier 0 only | No structured prompt support |

Zed and Cursor report `tier0_text` from `detect-surface-tier.sh`, so
`render.sh` never attempts Tier 1 on those harnesses.

## Directory Layout

```
skills/learn/ui-surface/
├── SKILL.md          — this file
└── scripts/
    └── render.sh     — tier-aware renderer
```
