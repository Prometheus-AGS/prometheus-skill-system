---
type: SessionRecord
id: karpathy-session-cf42b51b61e6a824
title: Karpathy session cf42b51b61e6
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T08:47:37.787092+00:00
created_at: 2026-08-06T08:47:37.787092+00:00
updated_at: 2026-08-06T08:47:37.787092+00:00
revision: 0
---

## Delta

Understood — nothing touched. Here's the verdict from static evidence.

## Yes, with one caveat

All 15 Prometheus agents are configured to come back on reboot. Every one of them passes the four checks that determine cold-start survival:

| Check | Result |
|---|---|
| Present in `~/Library/LaunchAgents/` | 15/15 |
| `RunAtLoad=true` (or correct calendar/interval trigger) | 15/15 |
| `plutil -lint` valid | 15/15 |
| Zero unsubstituted `__PLACEHOLDER__` tokens | 15/15 |
| Referenced binary exists and is executable | 15/15 |

Services covered: `exec`, `surrealdb-native`, `surreal-memory-native`, `pk-cherry`, `forge-mcp`, `surface-bridge`, `sovereign-sync`, `liter-llm-api`, `research`, `openai-proxy`, plus the `codex-skills-sync`, `learning-worker`, `hooks-logrotate`, and `prometheus-nudge` timers.

Three correctly have `RunAtLoad=false` — `hooks-logrotate` (daily 03:15), `prometheus-nudge` (every 4h), and the calendar-driven jobs. That's intended: they're timers, not daemons, and launchd will fire them on schedule after boot.

`prometheus-exec` specifically is the strongest-evidenced of the set, since this session also demonstrated `KeepAlive` recovering it from a SIGKILL — which is the same restart mechanism a reboot exercises.

## The caveat

This is strong configuration evidence, not an executed cold start. What it can't rule out is a **startup-ordering** problem: `surreal-memory-native` depends on `surrealdb-native`, and launchd doesn't guarantee ordering between agents. On a real boot, a dependent service could start first, fail, and rely on `KeepAlive` to retry. That would self-heal within seconds but could look like a brief failure in logs.

Two unrelated things I noticed that predate this session and are worth knowing:

- `sovereign-sync` shows **UNREACHABLE** on its `:7892` health probe despite running (pid 83323). Per your CLAUDE.md, 1.7.0 moved it to a unix socket rather than TCP — so the probe may simply be checking the wrong thing, but it means that health row is not currently informative.
- `forge-rs` returns **MCP ERROR (422)**.

Neither is caused by this work. Say the word if you want either looked at, or if you'd like me to run the cold-start test later when downtime is convenient.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T08:47:32.139174Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-20cd2b940d3ce6f8.md
- .prometheus/knowledge/wiki/karpathy-session-7f65f7375cbaf4fb.md
- .prometheus/knowledge/wiki/karpathy-session-8559ecda957810cd.md
