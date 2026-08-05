---
type: SessionRecord
id: karpathy-session-776f1cc3e285ecae
title: Karpathy session 776f1cc3e285
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:48:29.586025+00:00
created_at: 2026-08-05T04:48:29.586025+00:00
updated_at: 2026-08-05T04:48:29.586025+00:00
revision: 0
---

## Delta

Pushed: `67d29a2..e0c40ec`. Local and `origin/main` are level.

## Four commits shipped this session

| Commit | Change |
|---|---|
| `83a9cc5` | Advanced two submodule pins |
| `858e6e2` | Installer: re-sign after copy, verify by hash |
| `67d29a2` | Generation: exclude submodule test evidence (188M → 98M) |
| `e0c40ec` | Sync skills: correct transport, state vocabulary, node ID |

Beyond the commits: Rust toolchain repaired, 18 binaries rebuilt, all services verified healthy, generation live on 14/14 targets, and both Karpathy loops and sovereign-sync verified by exercising them rather than reading config.

## Open items

**1. The stale `.git/index.lock`** — appeared three times, always during installer runs, each time blocking a commit. Something dies mid-git-operation; a hook is the likely cause. Highest priority, since it silently blocks work.

**2. `sovereign-sync --mode daemon` help text** still says "HTTP on :7892" — the same drift I just fixed in the skills, but at the source. A user reading `--help` gets the wrong endpoint.

**3. Skill templating** — invoking `/sync-status` returned a literal `{{file:...}}` placeholder instead of instructions. I worked around it by reading `SKILL.md` directly, but if that affects other skills, slash-command invocation is degraded pack-wide.

**4. `pk` health: 463 issues** — `pk lint --fix --mechanical-only` is structurally a no-op here. Real fix is upstream in `prometheus-knowledge`: populate `description` at write time, or the count regrows every session.

**5. Two submodules carry uncommitted local fixes** (`surreal-memory-server`, `prometheus-knowledge`) that exist only in your working tree.

**6. `ai.prometheus.liter-llm-api` plist** — no installer hook, unsubstituted placeholders. Inert, but dead weight.

Items 2 and 3 are the ones I'd look at next — both affect what users see when they reach for these tools.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:47:48.503012Z
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
- .prometheus/knowledge/wiki/karpathy-session-52e9afea06c445c2.md
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
- .prometheus/knowledge/wiki/karpathy-session-ec9c006b3b7bc89c.md
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
