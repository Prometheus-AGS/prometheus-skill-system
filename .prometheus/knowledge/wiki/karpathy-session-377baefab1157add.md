---
type: SessionRecord
id: karpathy-session-377baefab1157add
title: Karpathy session 377baefab115
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T05:06:53.880749+00:00
created_at: 2026-08-05T05:06:53.880749+00:00
updated_at: 2026-08-05T05:06:53.880749+00:00
revision: 0
---

## Delta

Completed kbd-assess — session-defect-closure (step 0 of 1)

## Assessment complete — 7 findings, all verified

Written to `.kbd-orchestrator/phases/session-defect-closure/` with `goals.md` and `assessment.md`.

I ran 11 gates and reproduced every finding with a command. I did **not** use deep-research — you scoped this to verified repo findings, and adding external research would have widened it past what you chose.

### Blocking a "finished" declaration (3)

| | Finding | Fix |
|---|---|---|
| **F1** | `SKILLS.md` index out of date — fails a documented publishing-checklist gate | `npm run generate:skills-index` |
| **F2** | Hook siblings carry the `67d29a2` bundle update, uncommitted | Commit |
| **F3** | Two in-flight fixes built and verified, never committed | fmt/clippy, commit |

### Needs one decision each (4)

- **F4** — Submodule local fixes exist only in your working tree; a `submodule update` loses them
- **F5** — `liter-llm-api` plist: 7 unsubstituted placeholders, referenced by zero installers. Wire it or delete it
- **F6** — `pk`: 429 issues, and I verified `--fix` **cannot** touch them (`auto_fixable` is hardcoded false on the mechanical path). Upstream ticket or accept
- **F7** — The `.git/index.lock` recurrence, cause still unidentified

### Two candidates I rejected

I checked and dismissed both rather than padding the list: `install-mcp-services.sh`'s `declare -A` (no plist invokes it) and the remaining `:7892` references (the deliberate `--tcp` docs). Recorded so nobody re-raises them.

**F8** is informational — the exec engine is 1/4 complete scaffolding, so there's no code-executor service to run, and that's correct.

### Also verified healthy

145 skills validate clean, harness and Codex parity green, all 7 ports live, generation down to 91M, and neither installer defect class survives anywhere in `scripts/`.

Changes 1–2 are mechanical and get you to green gates. Want me to execute those two now, or take the decisions on 3–6 first?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T05:06:51.632676Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- hooks/codex-hooks.json
- scripts/generate-commands.js
- shared/harnesses/generated/claude-hooks.json
- substrate/sovereign-sync/src/main.rs
- tools/prometheus-knowledge
- tools/surreal-memory-server
- .kbd-orchestrator/phases/session-defect-closure/
- .prometheus/knowledge/wiki/karpathy-session-0197e8e8ea2e0af0.md
- .prometheus/knowledge/wiki/karpathy-session-01aec7fc2ad20b6d.md
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
- .prometheus/knowledge/wiki/karpathy-session-51e369fef53bc3a6.md
- .prometheus/knowledge/wiki/karpathy-session-52e9afea06c445c2.md
- .prometheus/knowledge/wiki/karpathy-session-69155c8f9e3cb18f.md
- .prometheus/knowledge/wiki/karpathy-session-6ff9a4514321e9e8.md
- .prometheus/knowledge/wiki/karpathy-session-776f1cc3e285ecae.md
- .prometheus/knowledge/wiki/karpathy-session-7a6fdfec66ed334a.md
- .prometheus/knowledge/wiki/karpathy-session-82de973ea6de7500.md
- .prometheus/knowledge/wiki/karpathy-session-907cda1a89139ecb.md
- .prometheus/knowledge/wiki/karpathy-session-9bec42541eb29323.md
- .prometheus/knowledge/wiki/karpathy-session-9dcc47a9275511db.md
- .prometheus/knowledge/wiki/karpathy-session-a02e77eb120e7f7c.md
- .prometheus/knowledge/wiki/karpathy-session-a309f7488926c040.md
- .prometheus/knowledge/wiki/karpathy-session-a655ae785ae82490.md
- .prometheus/knowledge/wiki/karpathy-session-ae2bd20fb3a2760f.md
- .prometheus/knowledge/wiki/karpathy-session-bda3e0b29a3b2fe9.md
- .prometheus/knowledge/wiki/karpathy-session-c071fcc30c3a34fc.md
