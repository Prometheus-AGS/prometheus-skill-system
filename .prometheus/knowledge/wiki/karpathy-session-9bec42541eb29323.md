---
type: SessionRecord
id: karpathy-session-9bec42541eb29323
title: Karpathy session 9bec42541eb2
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-04T19:41:00.522924+00:00
created_at: 2026-08-04T19:41:00.522924+00:00
updated_at: 2026-08-04T19:41:00.522924+00:00
revision: 0
---

## Delta

Yes — it's running and healthy.

```
{"service":"surreal-memory-server","status":"ok","version":"1.7.0"}   HTTP 200
```

| | |
|---|---|
| launchd job | `ai.prometheus.surreal-memory-native` |
| PID | 64262 (+ 64320 `embedding-executor` child) |
| Port | `:23001` LISTEN |
| Binary | `/usr/local/bin/surreal-memory-server` |
| Version | 1.7.0 |

Its SurrealDB backend is up too — `ai.prometheus.surrealdb-native`, PID 64194, `127.0.0.1:28000`, RocksDB at `~/.prometheus/data/surrealdb/database.db`.

Two notes:

**The `-15` in `launchctl list` is not an error.** It's the last exit status — SIGTERM from the `--restart` I ran earlier this session. The current PIDs are alive and serving.

**It's running the binary I rebuilt today.** Installed 12:08:54; process 64262 started after that during the service restart, so it includes your uncommitted local fixes — the heartbeat-flooding guard in `operations.rs` and the metal/cuda feature propagation.

One inconsistency worth flagging: `prometheus setup --check` earlier described this as "surreal-memory-server (Docker, port 23001) — running (Docker)". That's wrong — it's a native launchd process, not Docker. The health verdict is right, but the provenance label in that doctor output is misreporting.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-04T19:35:47.509337Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- tools/prometheus-knowledge
- tools/surreal-memory-server
- .prometheus/knowledge/wiki/karpathy-session-0f32748dc5508f4c.md
- .prometheus/knowledge/wiki/karpathy-session-1d4b68de958f65de.md
- .prometheus/knowledge/wiki/karpathy-session-27a481f8cf0172f7.md
- .prometheus/knowledge/wiki/karpathy-session-2d47dc18780cb546.md
- .prometheus/knowledge/wiki/karpathy-session-3974a3094c9d9a73.md
- .prometheus/knowledge/wiki/karpathy-session-4261972b77d5dabe.md
- .prometheus/knowledge/wiki/karpathy-session-47d6de518d674636.md
- .prometheus/knowledge/wiki/karpathy-session-69155c8f9e3cb18f.md
- .prometheus/knowledge/wiki/karpathy-session-6ff9a4514321e9e8.md
- .prometheus/knowledge/wiki/karpathy-session-82de973ea6de7500.md
- .prometheus/knowledge/wiki/karpathy-session-ae2bd20fb3a2760f.md
- .prometheus/knowledge/wiki/karpathy-session-bda3e0b29a3b2fe9.md
- .prometheus/knowledge/wiki/karpathy-session-c5ad08b0efd384b1.md
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
- .prometheus/knowledge/wiki/karpathy-session-f82a6d56360e80f3.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-record.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-status.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-executor-session-complete.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-session-complete.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-executor-session-complete.md
- .prometheus/knowledge/wiki/prometheus-exec-engine-executor-session-complete.md
