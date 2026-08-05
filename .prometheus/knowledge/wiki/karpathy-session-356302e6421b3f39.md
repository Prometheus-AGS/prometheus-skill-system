---
type: SessionRecord
id: karpathy-session-356302e6421b3f39
title: Karpathy session 356302e6421b
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:37:54.650203+00:00
created_at: 2026-08-05T04:37:54.650203+00:00
updated_at: 2026-08-05T04:37:54.650203+00:00
revision: 0
---

## Delta

Full status retrieved. Here's the result in the skill's format.

```
sovereign-sync status
─────────────────────
State     : Ready
Peers     : 0
Domains   : kbd-control, learner-model, skill-index, surreal-memory
Version   : 1.7.0
Transport : unix socket (attempt 1, lastError: null)
Latency   : p50/p95/p99 = 1.085 ms, failures=0, timeouts=0
```

## sovereign-sync is healthy and verified

`sovereign-sync --mode status` — the binary's own supported check — returns **healthy**, exit 0, with `{"service":"sovereign-sync","status":"ok","version":"1.7.0"}` over the socket. This closes the verification gap I left open earlier.

Domain adapters, with their privacy classes:

| Domain | Adapter | Privacy |
|---|---|---|
| `kbd-control` | wired | trusted |
| `learner-model` | wired | trusted |
| `skill-index` | wired | public |
| `surreal-memory` | never-synced | **local_only** |

`surreal-memory` showing `never-synced` / `local_only` is the privacy guarantee working — that domain's payloads are structurally prevented from leaving the device.

**0 peers** is expected on a single node with none enrolled. `node_state: Ready` rather than `Connected` reflects having nothing to connect to, not a fault.

## The skill's documentation is wrong

`/sync-status` as written **cannot work on this version**. It instructs querying `http://127.0.0.1:7892/health` and `/api/v1/sync/status`; both return HTTP 000 because 1.7.0 defaults to a **unix socket**, not loopback TCP. `--help` confirms: `--tcp  Expose loopback TCP instead of the default Unix socket`.

So the skill's Prerequisites ("must be running on `127.0.0.1:7892`"), both Instructions steps, and the "check that port 7892 is not blocked by a firewall" troubleshooting are all stale. Its `node_state` table also lacks `Ready`, the state actually returned.

Worth noting the daemon's own `--mode daemon` help text still says "HTTP on :7892", so the drift exists in the binary's help too, not only the skill.

Two smaller things: the skill invocation returned a literal `{{file:...}}` placeholder rather than the instructions, so I read `SKILL.md` directly — that templating may be broken for this skill. And my earlier socket probe failed because `lsof` showed a peer-connection socket rather than the listener at `…/run/sovereign-sync.sock`; my check was wrong, the service was fine throughout.

Want me to fix the `sync-status` skill to use the socket (with `--tcp` as the documented alternative) and add `Ready` to the state table? `sync-peers` and `sync-push` likely carry the same stale endpoint assumption and are worth checking together.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:37:48.589100Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- hooks/codex-hooks.json
- shared/harnesses/generated/claude-hooks.json
- tools/prometheus-knowledge
- tools/surreal-memory-server
- .prometheus/knowledge/wiki/karpathy-session-0197e8e8ea2e0af0.md
- .prometheus/knowledge/wiki/karpathy-session-06022874f6380713.md
- .prometheus/knowledge/wiki/karpathy-session-0f32748dc5508f4c.md
- .prometheus/knowledge/wiki/karpathy-session-1d4b68de958f65de.md
- .prometheus/knowledge/wiki/karpathy-session-1f107845d0001160.md
- .prometheus/knowledge/wiki/karpathy-session-219d1a1afa91a629.md
- .prometheus/knowledge/wiki/karpathy-session-27a481f8cf0172f7.md
- .prometheus/knowledge/wiki/karpathy-session-2d47dc18780cb546.md
- .prometheus/knowledge/wiki/karpathy-session-32ee1be19537e6d9.md
- .prometheus/knowledge/wiki/karpathy-session-3974a3094c9d9a73.md
- .prometheus/knowledge/wiki/karpathy-session-40521496d375f876.md
- .prometheus/knowledge/wiki/karpathy-session-4261972b77d5dabe.md
- .prometheus/knowledge/wiki/karpathy-session-47d6de518d674636.md
- .prometheus/knowledge/wiki/karpathy-session-69155c8f9e3cb18f.md
- .prometheus/knowledge/wiki/karpathy-session-6ff9a4514321e9e8.md
- .prometheus/knowledge/wiki/karpathy-session-7a6fdfec66ed334a.md
- .prometheus/knowledge/wiki/karpathy-session-82de973ea6de7500.md
- .prometheus/knowledge/wiki/karpathy-session-907cda1a89139ecb.md
- .prometheus/knowledge/wiki/karpathy-session-9bec42541eb29323.md
- .prometheus/knowledge/wiki/karpathy-session-9dcc47a9275511db.md
- .prometheus/knowledge/wiki/karpathy-session-a02e77eb120e7f7c.md
- .prometheus/knowledge/wiki/karpathy-session-a309f7488926c040.md
- .prometheus/knowledge/wiki/karpathy-session-ae2bd20fb3a2760f.md
- .prometheus/knowledge/wiki/karpathy-session-bda3e0b29a3b2fe9.md
- .prometheus/knowledge/wiki/karpathy-session-c071fcc30c3a34fc.md
- .prometheus/knowledge/wiki/karpathy-session-c5ad08b0efd384b1.md
- .prometheus/knowledge/wiki/karpathy-session-d7c8face5c7a0e8f.md
- .prometheus/knowledge/wiki/karpathy-session-e4e3b6d3c2bfe524.md
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
- .prometheus/knowledge/wiki/karpathy-session-f4eee3dc3ecd9100.md
- .prometheus/knowledge/wiki/karpathy-session-f82a6d56360e80f3.md
- .prometheus/knowledge/wiki/karpathy-session-fc341e3307d51188.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-record.md
