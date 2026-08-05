---
type: SessionRecord
id: karpathy-session-2c9ae2ec0b01465f
title: Karpathy session 2c9ae2ec0b01
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T13:58:37.265067+00:00
created_at: 2026-08-05T13:58:37.265067+00:00
updated_at: 2026-08-05T13:58:37.265067+00:00
revision: 0
---

## Delta

Completed kbd-apply — kde-003-hooks-probe (step 2 of 5)

## Answered from source, not a probe

The spec called for a throwaway probe package. I found the shipped loader instead:

```
daimon-bundle/…/@moonshot-ai/agent-core/dist/index.mjs → parseManifest()
```

Its returned object **is** the supported field set. Stronger than a black-box probe — a probe shows one path failing, the loader shows the entire contract and can't produce a false negative from a wrong event name, which the spec itself named as the probe's central weakness. And nothing was written into app-managed state.

## Verdicts

**`hooks` — SUPPORTED.** But with two shape differences that would have broken a naive port of the pack's 30-hook bundle:

- Kimi wants a **flat array** of `{event, matcher?, command, timeout?}`; Claude Code nests hooks inside matcher groups keyed by event
- `timeout` is integer **seconds capped at 600**; the pack uses milliseconds (`30000`) — rejected

Same 7 events otherwise.

**`systemPrompt` — NOT SUPPORTED**, and worse than absent: it's not in `UNSUPPORTED_RUNTIME_FIELDS` either, so declaring it emits **no diagnostic** and is silently ignored. Exactly the inertness failure this change existed to catch. E5 moves CONSIDER → REJECT; nothing lost, since `skillInstructions` already covers routing and is genuinely consumed.

## Unplanned finding: `commands`

`parseManifest` returns `commands` (string or string[]). It appears in **none** of the 12 vendor packages and no documentation reviewed during assess or analyze. The pack ships 147 slash commands to Claude Code and Codex — this may be the Kimi equivalent. Recorded as **E8, unowned**.

## Limit I'm stating plainly

**Parsing is not execution.** The loader accepting `hooks` proves the field reaches the data model, not that a command spawns. That's `kde-004`'s job — which this verdict now unblocks.

All four gates pass; no probe package was created, shipping package intact at 145 skills.

Pushed as `87fb5d4`. Next: `/kbd-apply kde-001-mcp-servers` — whose blocking t1 asks whether the daimon accepts a loopback `http://` URL, and which I can now likely answer the same way, by reading `readMcpServers`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T13:58:26.429580Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
