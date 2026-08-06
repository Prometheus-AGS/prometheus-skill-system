---
type: SessionRecord
id: karpathy-session-273f3645230ec04d
title: Karpathy session 273f3645230e
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T09:18:42.229642+00:00
created_at: 2026-08-06T09:18:42.229642+00:00
updated_at: 2026-08-06T09:18:42.229642+00:00
revision: 0
---

## Delta

Let me give you the state of things now, since the codebase research is complete and only the landscape section remains open.

The outline is drafted at `docs/articles/OUTLINE-loops-that-lie.md` (302 lines). Here's where it stands.

## Recommended thesis

**"Your Loop Is Lying to You."** The first article argued the unit of work is the loop. The follow-up argues the loop was the *easy part* — once loops run unattended, the binding constraint becomes **verification**.

The repo's own reflection file names the defect better than I could, and it should be the article's spine:

> **"Asserting on a result without checking that the code path producing it executed."**

Every war story is that sentence in a different costume.

## The cold open

The first article promoted an anti-sycophancy gate — a second model reviewing the first — with a ✅ in the comparison table. **It had never once run.** All 8 stored `findings.json` recorded `judge_model: "harness-subagent (claude, parent-session family)"`. Claude reviewing Claude. Every one `PASS`.

Five compounding causes, and **every one produced a green check**. Not one produced an error.

## Verified deltas since 2026-06-24

| Metric | Then | Now |
|---|---|---|
| Version | 1.2.0 | 1.7.0 (+443 commits) |
| Native skills | 102 | **147** |
| Rust crates | **0** | 17 (55,732 LOC, 385 tests) |
| Install targets | 7 tools | 14 |
| MCP servers | 7 | **7 — unchanged** |
| Completed KBD phases | — | 37 |

`substrate/` first appeared **2026-06-28 — four days after the article published.** The system described in the first piece was markdown and shell; it now has a compiled core.

## A caveat that belongs in the article

Three of my own numbers were wrong on first pass — skill count (311→**147**, I'd counted submodules), Rust LOC (32k→**55.7k**), tests (234→**385**). All three were caught by re-checking. That error rate is itself the thesis in miniature, and I've written it into the outline rather than quietly fixing it.

## Also included

- **The OpenRaft story** — a consensus layer initialized with one voter that *could never gain a second*; 236 MB backing a 49 KB journal; a static-constant `/health` taking 12 seconds. Removed entirely. The story is the deletion.
- **A 12-row DO-NOT-CLAIM table**, independently verified where load-bearing (OpenRaft genuinely gone; exec evidence file confirms `mobile-size: blocked`).
- **The failure scores as the selling point** — two phases at `0/5`, one refusing to round `3.5/4` up, one recorded as "MET (the answer is no)."
- A fact-check protocol requiring every number to be re-run at draft time.

## Still open

**Part V (competitive landscape) is a placeholder.** The web research agent is still running — it's done substantial work but hasn't reported. I won't fill that section with anything until it does, and I'd rather tell you it's pending than guess at what shipped in Claude Code or Codex since June.

Five questions for you are at the end of the outline; the two that most change the draft:

1. **How hard to lean on the failure narrative?** I recommend hard — it's differentiated and true — but it's your reputation.
2. **Length** — match the original's ~7,600 words, or run leaner at ~4,000 given a tighter thesis?

I'll fold in the landscape findings as soon as they land.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T09:18:38.810288Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .kbd-orchestrator/phases/mobile-skill-portability/mobile-classification.json
- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-06fb138df2cedd2f.md
- .prometheus/knowledge/wiki/karpathy-session-95cb4c69e3a13c3a.md
- .prometheus/knowledge/wiki/karpathy-session-b981029e4691b82f.md
- .prometheus/knowledge/wiki/karpathy-session-d36396632cc9ba19.md
- docs/articles/OUTLINE-loops-that-lie.md
