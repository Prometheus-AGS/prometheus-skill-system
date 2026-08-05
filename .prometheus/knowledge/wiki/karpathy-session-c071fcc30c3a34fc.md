---
type: SessionRecord
id: karpathy-session-c071fcc30c3a34fc
title: Karpathy session c071fcc30c3a
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:35:40.084416+00:00
created_at: 2026-08-05T04:35:40.084416+00:00
updated_at: 2026-08-05T04:35:40.084416+00:00
revision: 0
---

## Delta

The socket path from `lsof` doesn't exist on disk — it's an abstract/unlinked socket, so HTTP probing can't reach it. That's a limitation of my check, not proof of a fault. The process is alive and actively logging work.

## Answer: yes for the running stack — but there is **no code executor service**, by design

### Services up (8/8)

| Service | PID | Endpoint | Health |
|---|---|---|---|
| surrealdb-native | 64194 | :28000 | LISTENING |
| surreal-memory-native | 64262 | :23001 | HTTP 200 |
| pk-cherry | 64353 | :8942 | LISTENING |
| forge-mcp | 64437 | :8943 | LISTENING |
| surface-bridge | 64514 | :7890 | HTTP 200 |
| prometheus-research | 56030 | :7891 | HTTP 200 |
| openai-proxy | 84170 | :8181 | HTTP 200 |
| sovereign-sync | 37106 | unix socket | alive, 4h uptime |

Plus 4 scheduled jobs registered and idle (learning-worker, nudge, logrotate, codex-skills-sync) — `-` PID is correct for those; they fire on trigger.

Two status codes that look alarming but aren't: `openai-proxy` shows `-9` and several show `-15`. Those are *last exit* codes from restarts I performed earlier, not current state. Every one has a live PID and answers.

`sovereign-sync` has no TCP listener on :7892 because **1.7.0 moved it to a unix socket** — consistent with your memory note. It's running and actively writing to its log.

### The code executor: not running because it does not exist yet

There is **no `prometheus-exec` service to start**:

- No `prometheus-exec` binary in `~/.local/bin`
- No exec launchd plist
- `substrate/exec-service/src/lib.rs` is a 10-line stub — just a `VERSION` constant and a doc comment
- `install-binaries.sh` doesn't reference the exec crates at all

The phase progress explains why:

```
change-exec-001-contracts-verification  completed
change-exec-002-tier-p-sidecar          pending
change-exec-003-tier-w-mobile           pending
change-exec-004-remote-mcp-docs         pending
```

Only the contracts layer landed in commit `8464db7` — signed receipts, offline verification, hash-linked logs, plus 10.7k lines of schemas and tests. **Tier P (the sidecar that would actually execute code) is `change-exec-002`, unstarted.** So there's nothing to run, and its absence is correct rather than a failure.

This matches what I said in the first assessment: the exec crates are scaffolding, correctly not installed.

One caveat on rigor: I verified `sovereign-sync` is alive and logging, but couldn't complete an HTTP health check — the socket `lsof` reports isn't present on disk as a filesystem path, so my probe returned HTTP 000. That's a gap in my verification, not evidence of a problem. If you want it proven, `/sync-status` would exercise the real client path.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:35:36.721296Z
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
- .prometheus/knowledge/wiki/karpathy-session-c5ad08b0efd384b1.md
- .prometheus/knowledge/wiki/karpathy-session-d7c8face5c7a0e8f.md
- .prometheus/knowledge/wiki/karpathy-session-e4e3b6d3c2bfe524.md
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
- .prometheus/knowledge/wiki/karpathy-session-f4eee3dc3ecd9100.md
- .prometheus/knowledge/wiki/karpathy-session-f82a6d56360e80f3.md
- .prometheus/knowledge/wiki/karpathy-session-fc341e3307d51188.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-record.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-status.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-executor-session-complete.md
