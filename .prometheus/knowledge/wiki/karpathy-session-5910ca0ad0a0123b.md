---
type: SessionRecord
id: karpathy-session-5910ca0ad0a0123b
title: Karpathy session 5910ca0ad0a0
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T18:37:35.837856+00:00
created_at: 2026-08-05T18:37:35.837856+00:00
updated_at: 2026-08-05T18:37:35.837856+00:00
revision: 0
---

## Delta

Completed kbd-apply — kde-001-mcp-servers (step 3 of 5)

## Every blocking assumption was wrong — the loader said so

I resolved t1, t2, and t2b by reading `McpServerConfigSchema` instead of probing:

| Assumption | Reality |
|---|---|
| Loopback `http://` may be refused | `url: z.string().url()` — **no** scheme or host restriction |
| SSE has no vendor precedent | `transport: "sse"` is **first-class** |
| Auth isn't expressible | `headers` **and** `bearerTokenEnvVar` both exist |

All three vendor packages being remote HTTPS was a **biased sample**, not a rule. The three servers now emit cleanly.

## The forge-rs 401 was never a broken service

forge *required* a bearer token with no opt-out; my probe sent none. Per your call, I fixed it at the source rather than threading the credential into Kimi.

**`--no-auth` is opt-in and loopback-only — and it refuses to start, rather than warning:**

```
Error: refusing to start: --no-auth is only permitted on a loopback bind,
but --bind is 0.0.0.0. Without auth this would expose forge_enrich /
forge_reflect / forge_drift / forge_validate — which read and write project
files — to every host that can reach this address.
```

A warning would be the wrong instrument for publishing an unauthenticated file-writing API. `is_loopback_bind` parses the address rather than string-matching, so `127.0.0.0/8` and IPv6 `::1` work — and `localhost` is accepted as a literal but deliberately **not resolved**, since a hosts-file entry shouldn't decide whether auth is required.

The shared dev token is now gone from the plist, `.mcp.json`, and the port table.

## Verified

19 tests pass · unauthenticated `initialize` → **HTTP 200** · `--bind 0.0.0.0 --no-auth` → exit 1 · goal-4 gate: deleted, reinstalled, **145 skills + 3 servers** restored · clippy and fmt clean.

## Two things worth flagging

**Not yet proven:** that Kimi Desktop actually *connects* to these servers. The manifest satisfies the schema the loader enforces — necessary, not sufficient.

**Correction:** I assumed `tools/forge-rs` was a submodule and tried pushing upstream from it. It's a regular directory in this repo, so it committed here. The parent-repo `.git/index.lock` also reappeared (4th time this session).

Pushed as `4678ede`. Next: `/kbd-apply kde-002-session-start`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T18:37:31.320212Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
