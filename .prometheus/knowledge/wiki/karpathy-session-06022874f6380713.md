---
type: SessionRecord
id: karpathy-session-06022874f6380713
title: Karpathy session 06022874f638
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:33:42.314223+00:00
created_at: 2026-08-05T04:33:42.314223+00:00
updated_at: 2026-08-05T04:33:42.314223+00:00
revision: 0
---

## Delta

Pushed: `858e6e2..67d29a2`. Local and `origin/main` are level.

## Session complete — three commits on `main`

| Commit | Change |
|---|---|
| `83a9cc5` | Advanced two submodule pins |
| `858e6e2` | Installer: re-sign after copy, verify by hash |
| `67d29a2` | Generation: exclude submodule test evidence (188M → 98M) |

Alongside the commits: Rust toolchain repaired, 18 binaries rebuilt, services restarted and healthy, generation `44c5e0e5…` live on 14/14 targets, and Karpathy loops verified working by firing all three hooks and watching queue state change.

## Open items

**1. The stale `.git/index.lock` — worth root-causing.** It appeared three times, always during installer runs, each time blocking a commit until manually cleared. Something is dying mid-git-operation; a hook is the likely culprit. This is the item I'd prioritize, since it silently blocks work.

**2. `pk` health: 463 issues.** I established `pk lint --fix --mechanical-only` is structurally a no-op here — the mechanical rules never set `auto_fixable`, and the sole non-LLM repair targets a missing `type` field none of these issues have. The durable fix is upstream in `prometheus-knowledge`: populate `description` at write time, or the count regrows every session.

**3. `ai.prometheus.liter-llm-api` plist** — still no installer hook, still has unsubstituted `__PROMETHEUS_ROOT__` placeholders. Inert today because `openai-proxy` serves :8181, but it's dead weight until wired or removed.

**4. Two submodules carry uncommitted local fixes** (`surreal-memory-server`, `prometheus-knowledge`). They survive in your working tree but exist nowhere else — a `git submodule update` or fresh clone loses them.

One correction worth restating: early on I reported binaries as current and needing no rebuild. That was accurate at the time, but repairing the toolchain invalidated the premise, and the rebuild that followed was substantial rather than the two-crate job the dry run suggested.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:33:40.145606Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- hooks/codex-hooks.json
- shared/harnesses/generated/claude-hooks.json
- tools/prometheus-knowledge
- tools/surreal-memory-server
- .prometheus/knowledge/wiki/karpathy-session-0197e8e8ea2e0af0.md
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
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-session-complete.md
