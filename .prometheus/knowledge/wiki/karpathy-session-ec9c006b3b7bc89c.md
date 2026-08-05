---
type: SessionRecord
id: karpathy-session-ec9c006b3b7bc89c
title: Karpathy session ec9c006b3b7b
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:47:49.133086+00:00
created_at: 2026-08-05T04:47:49.133086+00:00
updated_at: 2026-08-05T04:47:49.133086+00:00
revision: 0
---

## Delta

Committed as `e0c40ec`. Done — and the scope was wider than the one skill you asked about.

## Three distinct inaccuracies, all verified against the running daemon

**1. Transport.** All three skills documented `http://127.0.0.1:7892`, which the daemon doesn't bind — 1.7.0 defaults to a Unix socket. Every documented `curl` returned exit 7 against a healthy node, and the troubleshooting section told you to check a firewall on a port nothing listens to. Now uses `--unix-socket "$SOCK"` with per-platform paths taken from `default_socket_path()` in `main.rs`, plus `--tcp` documented as the opt-in alternative including its `--token-file` requirement.

**2. State vocabulary.** My first draft was wrong too. I initially documented `Connected`/`Syncing`/`Idle` as `node_state` values, copying the old table. Testing showed the API returns `"Ready"` — `rest_api.rs:114-120` collapses all three into `P2PTransportState::Ready`, and the internal names never surface. Reporting them would have been fabrication, so the table now documents the rollup.

**3. Node ID.** `sync-peers` said to read it via `/health | jq .node_id`. That returns `null` — `/health` carries only `{service, status, version}`. It's `transport.nodeId` on the status and peers endpoints. I only caught this because I ran the documented command instead of assuming it worked.

## Verification

All five documented commands executed verbatim against the live daemon:

| Command | Result |
|---|---|
| `sovereign-sync --mode status` | exit 0 |
| status endpoint | `node_state` present |
| peers endpoint | `peers` present |
| `transport.nodeId` | `b12cd66c0e34c6ea…` |
| sync-push prerequisite `jq` | works |

`npm run validate`: 145 skills, 0 errors. All three pass `validate:strict`.

I also documented the two traps that cost me time earlier: `Ready` with zero peers is healthy on a single node, and an `lsof` entry may be a peer connection rather than the listener — which is exactly what made this service look broken during my first probe.

Two things I did **not** fix: the daemon's own `--mode daemon` help text still says "HTTP on :7892", which is the same drift at the source; and the `/sync-status` skill invocation returned a literal `{{file:...}}` placeholder instead of instructions, so the skill templating may be broken independently of content.

Commit is local. Push?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:47:45.746920Z
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
- .prometheus/knowledge/wiki/karpathy-session-356302e6421b3f39.md
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
- .prometheus/knowledge/wiki/karpathy-session-dd5ce3ce69b6a275.md
- .prometheus/knowledge/wiki/karpathy-session-e4e3b6d3c2bfe524.md
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
- .prometheus/knowledge/wiki/karpathy-session-f4eee3dc3ecd9100.md
- .prometheus/knowledge/wiki/karpathy-session-f82a6d56360e80f3.md
