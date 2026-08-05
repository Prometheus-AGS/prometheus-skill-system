---
type: SessionRecord
id: karpathy-session-e4e3b6d3c2bfe524
title: Karpathy session e4e3b6d3c2bf
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T04:21:55.064112+00:00
created_at: 2026-08-05T04:21:55.064112+00:00
updated_at: 2026-08-05T04:21:55.064112+00:00
revision: 0
---

## Delta

Committed as `67d29a2`.

## Result: 188M → 98M (48% smaller)

The scope was much larger than the two `.app` binaries I originally flagged. **97M of the 188M payload was test evidence** from one submodule — Playwright `trace.zip` archives, a 4.3M `.webm` screen recording, and two iOS `.app` bundles each containing a 28M Mach-O binary. All shipped to 14 targets on every install.

`copyEntry` excluded only `node_modules`, `target`, and `.git`, so `.kbd-orchestrator/` evidence passed straight through.

## The design decision that mattered

I made the exclusion **path-scoped, not name-scoped**. A bare `name === '.kbd-orchestrator'` check would have been much simpler — and would have silently dropped *this* repo's own orchestrator state: waypoints, `progress.json`, phase plans, all load-bearing. Only evidence under `skills/imported/**` is disposable.

That distinction is covered by a 7-case matcher test: submodule evidence excluded, this repo's evidence and orchestrator root both preserved.

One real bug I caught while wiring it: the `PAYLOAD_ROOTS` loop copies from `source/<root>`, so a defaulted `repoRoot` would have made paths relative to `<repo>/skills` — and my `skills/imported/**` pattern would never have matched. The call site now passes `source` explicitly.

## An integrity gate did its job

The first rebuild failed with `release payload verification failed for scripts/install-plugin-generation.js`. That's the bundle-pinning system working correctly — `runtimeFiles` pins a SHA-256 per runtime file, and I'd changed one. I regenerated the manifest rather than bypassing the check. New bundle `b0144d09` (30 hooks × 2 manifests, parity green).

## Verified nothing was lost

| | Before | After |
|---|---|---|
| Payload | 188M | **98M** |
| Index entries | 145 | 145 |
| `SKILL.md` files | 311 | 311 |
| Mach-O in payload | 2 | **0** |

Plus: `update-skill-pack.sh --force` exit 0 on 14/14 targets, `run-hook --resolve-only` resolves `b0144d09` → generation `44c5e0e5`, and `npm run validate` passes 145 skills with 0 errors.

## Note

The stale `.git/index.lock` appeared a third time; I cleared it again. Three occurrences in one session, always during installer activity — that's a real pattern worth root-causing.

Commit is local. Push?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T04:21:51.767309Z
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
- .prometheus/knowledge/wiki/karpathy-session-f073f6aaddb394a0.md
- .prometheus/knowledge/wiki/karpathy-session-f35607e7ef8ae7be.md
- .prometheus/knowledge/wiki/karpathy-session-f4eee3dc3ecd9100.md
- .prometheus/knowledge/wiki/karpathy-session-f82a6d56360e80f3.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-record.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-status.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-executor-session-complete.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-session-complete.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-session-completion.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-executor-session-complete.md
