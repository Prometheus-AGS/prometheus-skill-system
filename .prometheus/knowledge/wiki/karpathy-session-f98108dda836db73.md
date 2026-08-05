---
type: SessionRecord
id: karpathy-session-f98108dda836db73
title: Karpathy session f98108dda836
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T10:01:50.660062+00:00
created_at: 2026-08-05T10:01:50.660062+00:00
updated_at: 2026-08-05T10:01:50.660062+00:00
revision: 0
---

## Delta

Completed kbd-analyze — kimi-desktop-extensibility (step 0 of 1)

## The retry fix — pushed as `e40a3b3`

`dispatch-judge.sh` now escalates on timeout-shaped failures only:

```
[judge] retry 2/3: HTTP 000 looks like a timeout
[judge]        at 1s; escalating timeout to 2s and retrying.
```

- `ADV_JUDGE_TIMEOUT` (default 300s), `ADV_JUDGE_RETRIES` (default 3)
- Doubles each attempt, announces every escalation
- **Only** retries HTTP 000 or 502/503/504 with a Network/timeout body — a 401 isn't slow, it's wrong, and retrying it would just delay an actionable error

Proven by forcing `ADV_JUDGE_TIMEOUT=1`: escalated 1s → 2s → 4s, then failed with a message naming both env vars. With defaults, the previously-failing vet **completed on the first attempt**.

I also reverted the 600s hack I'd put in the gateway config — that was me hand-patching a symptom.

## Analyze result

The assessment's **blocking prerequisite is resolved**, and it turned out to be the wrong worry. `kimi-cu` ships a shim whose own header states the vendor rule verbatim: *"kimi-code only allows a stdio plugin's command to be a PATH command or a `./` relative path."* But 3 of 4 vendor packages use the **`url` form**, which sidesteps PATH entirely.

| Verdict | Candidate |
|---|---|
| **Adopt** | url-form `mcpServers` (surreal-memory, prometheus-knowledge) |
| **Adopt** | `sessionStart` → `kbd-status` |
| Defer | stdio shims |
| Reject | npx third-party servers; shipping binaries in-package |
| Blocked | `forge-rs` — returned 401, vendor schema shows no auth-header field |

I also corrected my own unevidenced claim from the assessment: the "7 MCP servers" figure conflated `.mcp.json` entries with pack binaries. They're different sets, and `surface-bridge` isn't an MCP server at all.

## Judge verdict: PASS, with 4 WARNINGs carried forward

The sharpest one, which I'd have missed: **HTTP 200/405 proves a listener exists, not that the endpoint speaks MCP.** Spec must verify an actual MCP handshake before emitting those servers. All four are in `handoffs/analyze.handoff.json`.

Next stage is `/kbd-spec`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T10:01:42.541960Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
