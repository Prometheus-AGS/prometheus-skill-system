---
type: SessionRecord
id: karpathy-session-1de3e51975cdca1e
title: Karpathy session 1de3e51975cd
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T09:56:39.130130+00:00
created_at: 2026-11-16T09:56:39.130130+00:00
updated_at: 2026-11-16T09:56:39.130130+00:00
revision: 0
---

## Delta

The 503 comes from a leftover `sovereign-sync` daemon that turns off every KBD authority route if any registered project fails to open, and 122 of its 133 registered projects failed. I changed nothing while investigating.

**What answers the socket**

- The control endpoint is `~/Library/Application Support/prometheus/run/sovereign-sync.sock`, served by the LaunchAgent `ai.prometheus.sovereign-sync`.
- It runs `~/.local/bin/sovereign-sync` version 1.7.0, built Aug 26. The pack retired this service on Sep 19 (`edbdb47a`, "retire pack-owned sync service") and handed it to `prometheus-companion`, which I could not find installed on this machine.
- The process started Nov 8 01:36, failed authority initialization about two seconds later, and has stayed in `stage: failed` for eight days without retrying. `/health` still returns 200, which is why the service looks alive.

**Why 122 projects failed**

- **118 are dead test fixtures:** temp directories such as `stage-gate-*` and `kbd-boundary-*` under `/var/folders/.../T/` that no longer exist but are still listed in `~/Library/Application Support/prometheus/kbd/registry.json`. They fail in 0–1 ms.
- **4 are real projects:** `prometheus-skill-system`, `universal-agent-runtime`, `prometheus-entity-management` and `prometheus-parking-lot-rs`. Each fails after about 300 ms of real work.

I could not get the reason for those four: this build logs only `success=false`, and the per-project diagnostics route is itself behind the 503. My best guess is version skew — all four have large `project.loro` stores written by a newer runtime than the daemon embeds — but that is unproven.

**Why one failure takes everything down**

The 1.7.0 build treats any failed open as fatal for all routes. The last pack-owned source (1.8.0) had already changed this to keep healthy projects available and fail only when all of them fail.

Pruning the dead registry entries alone would not clear the 503, because the four real projects would still fail on this build. It needs either a current daemon from `prometheus-companion` or the stale LaunchAgent unloaded; with no endpoint present, the pack treats absence as normal.

The doctor refresh is still running in the background. Its log already shows `prometheus-exec install failed — continuing with the remaining binaries`; I'll report the full result when it finishes.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T09:56:37.223442Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- No changed paths detected.
