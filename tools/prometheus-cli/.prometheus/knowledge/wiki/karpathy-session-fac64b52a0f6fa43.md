---
type: SessionRecord
id: karpathy-session-fac64b52a0f6fa43
title: Karpathy session fac64b52a0f6
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T11:49:19.223986+00:00
created_at: 2026-08-07T11:49:19.223986+00:00
updated_at: 2026-08-07T11:49:19.223986+00:00
revision: 0
---

## Delta

Yes, that makes sense — and I've started running that loop. Here's where things stand.

## Counter corrected ✅

```
C-00: PENDING → COMPLETE (4/4 tasks)
Phase: 0/21 → 1/21, active_change: C-01
```

## The root-cause defect — found, fixed, building

Correcting the counter would have been treating the symptom. The real defect is in the pack, and it's the one that cost us the work:

**15 call sites read state via a bare `client.status().await?` before every mutation.** That call is TCP-only. The managed daemon never binds TCP (`--mode daemon` only, Unix socket), and the CLI's `reqwest` can't speak `unix://`. So every write died at the *precondition read* and never reached `submit_fresh`'s correct `Unreachable → execute_locally` fallback.

**This was a known bug with an incomplete fix.** Commit `374f313` — *"commit locally when the control plane cannot adjudicate"* — fixed exactly **one** call site, with a comment naming *"the exact failure Codex hit."* The other 15 were left.

Fix: added `state_or_replay()` mirroring that proven pattern, applied to all 15. Two of them (`Pause`, `Cancel`) were worse than the rest — they write a local emergency-pause file *before* the read, so failure stranded the operator paused-on-disk with no durable record.

`cargo check` passes clean. Release build running now.

## Why this is the marquee finding

The data loss needed **two** defects to compose:

| | |
|---|---|
| 05:54 | Codex archives 4 changes; records completion in `progress.json` — the only place it *could*, since writes were broken |
| 06:09 | My `migrate --apply` rebuilds `progress.json` from canonical state that never received those completions |
| Result | C-00 silently reverts to PENDING while 4 changes sit archived on disk |

That's logged as **F-002** (migrate should refuse, or at least warn, when the projection is *ahead* of canonical). F-001's fix narrows the window sharply but doesn't close it.

I also logged **F-003**: `KBD-HANDOFF.md` and `KBD-RECOVERY-PROBLEM-REPORT.md` describe a control plane that no longer exists — they say sovereign-sync doesn't compile and health takes 12s; it's loaded, healthy, p50 **2.7ms**. An agent reading those as current would conclude KBD is unusable.

All in [skill-system-findings.md](.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/skill-system-findings.md), which also records what **worked** — migration on real legacy state, the cross-harness handoff, and the adversarial loop correctly *rejecting* one critic claim on re-measurement.

Next: install, then re-run the exact failing command (`prometheus kbd decision record` for D1/D2/D3) as proof. I'll hold your Codex prompt until that's verified — no point resuming Codex if writes are still broken.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T11:49:17.477896Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-cli

## Changed Paths

- ../../.prometheus/knowledge/wiki/index.md
- ../../.prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-complete-unknown-change.md
- ../../.prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- ../../.prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-unknown-change.md
- ../../.prometheus/knowledge/wiki/log.md
- .prometheus/events.jsonl
- .prometheus/knowledge/.prompt-snapshots/project/current
- .prometheus/knowledge/wiki/executor-session-completion-kimi-desktop-extensibility.md
- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- crates/prometheus-cli/src/commands/kbd.rs
- ../../.prometheus/knowledge/wiki/karpathy-session-01343af7d0fe4293.md
- ../../.prometheus/knowledge/wiki/karpathy-session-20dadb89eb822742.md
- ../../.prometheus/knowledge/wiki/karpathy-session-253e6d789e51e2ca.md
- ../../.prometheus/knowledge/wiki/karpathy-session-2580f8aab12a344f.md
- ../../.prometheus/knowledge/wiki/karpathy-session-5ba81ce56f70adfa.md
- ../../.prometheus/knowledge/wiki/karpathy-session-6e6925d2d6588b9d.md
- ../../.prometheus/knowledge/wiki/karpathy-session-8e446017ed66cb65.md
- ../../.prometheus/knowledge/wiki/karpathy-session-b59e456a02d42622.md
- ../../.prometheus/knowledge/wiki/karpathy-session-c78d9c4b94ed6241.md
- ../../.prometheus/knowledge/wiki/karpathy-session-da3c988e8062b513.md
- ../../.prometheus/knowledge/wiki/karpathy-session-e6f5d70de34880a9.md
- ../../.prometheus/knowledge/wiki/karpathy-session-f2c5b757e52fc16e.md
- ../../.prometheus/knowledge/wiki/kimi-desktop-extensibility-session-completed-change-unknown.md
- .prometheus/knowledge/.prompt-snapshots/project/generations/08f8dab316aa33a1cc148d8c6b37f588e9df1e23633df6019ccbd6c50bfe64ee.json
- .prometheus/knowledge/wiki/karpathy-session-d6126f64f63475e4.md
