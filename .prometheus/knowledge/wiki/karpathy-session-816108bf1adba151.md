---
type: SessionRecord
id: karpathy-session-816108bf1adba151
title: Karpathy session 816108bf1adb
tags:
- karpathy
- session-learning
sources:
- session:69948acc-ad6d-4a8c-a02a-ba8a453e0345
timestamp: 2026-08-26T17:32:07.457415+00:00
created_at: 2026-08-26T17:32:07.457415+00:00
updated_at: 2026-08-26T17:32:07.457415+00:00
revision: 0
---

## Delta

All four parts of the request are done:

**1. Submodule links updated** — `tools/liter-llm` and `tools/prometheus-knowledge` were re-synced to their already-pinned commits (working trees had drifted behind); `tools/surreal-memory-server` fast-forwarded and its pin bumped (commit `069343ab`).

**2. All services rebuilt** — every Rust CLI/daemon (prometheus-exec, prometheus-cli, forge, pk/pk-cherry/learning-worker, learner-model, surface-bridge, sovereign-sync, liter-llm, openai-proxy, surreal-memory-server, sycophancy-correction, template-forge(-mcp), cowork/co, dsg, prometheus-research) built and installed cleanly. Worked around the known arm64-pinned hash on this x86_64 box via `PROMETHEUS_EXEC_EXPECTED_SHA256` (no manifest edit).

**3. All services redeployed** — found and fixed a real bug along the way: the new MLX-native-embeddings feature that came in with the `surreal-memory-server` submodule bump defaulted to `backend=mlx` unconditionally, which crash-loops on Intel Macs (MLX is Apple Silicon-only). Fixed the default to be architecture-aware (commit `2725a7eb`). All 8 launchd services now confirmed `running`/`OK`, including `ai.prometheus.exec` which needed one retry past a transient launchd bootstrap I/O error.

**4. Skills reinstalled for every AI tool** — hit a real collision (4 platforms had `artifact-refiner` symlinked to an unrelated standalone repo, blocking this pack's own submodule); per your choice, moved those foreign symlinks aside to `.foreign-20260826` backups and re-ran clean. Final counts: Claude Code 428, Kimi 596, MiniMax 203, OpenCode 15, Codex 29, Cursor 310 skills, MCP configs verified across all tools, plugin generation verified.

Two commits were made (submodule pin bump, embedding-backend arch fix) — nothing else was pushed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 69948acc-ad6d-4a8c-a02a-ba8a453e0345
- Captured: 2026-08-26T17:31:31.520442Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-12a16adb4103181d.md
- .prometheus/knowledge/wiki/karpathy-session-27e6e64b8b961dd1.md
- .prometheus/knowledge/wiki/karpathy-session-33f87f2d8886c38d.md
- .prometheus/knowledge/wiki/karpathy-session-3a86580c526e8a47.md
- .prometheus/knowledge/wiki/karpathy-session-3c8c8e130dbfbe95.md
- .prometheus/knowledge/wiki/karpathy-session-5c70858833a29a39.md
- .prometheus/knowledge/wiki/karpathy-session-5ccb1839beec80fc.md
- .prometheus/knowledge/wiki/karpathy-session-86e485b2eac6f5fe.md
- .prometheus/knowledge/wiki/karpathy-session-a5e7a522889fc26e.md
- .prometheus/knowledge/wiki/karpathy-session-ae8d059b586708f9.md
- .prometheus/knowledge/wiki/karpathy-session-c4707e1c05fa510c.md
- .prometheus/knowledge/wiki/karpathy-session-c65c456821829d4f.md
- .prometheus/knowledge/wiki/karpathy-session-cf25f0fbc8c9817b.md
- .prometheus/knowledge/wiki/karpathy-session-e7cfffb07f7a2ae4.md
- .prometheus/knowledge/wiki/karpathy-session-f8f0e6845e8bd9f4.md
- crates/prometheus-exec/.prometheus/
