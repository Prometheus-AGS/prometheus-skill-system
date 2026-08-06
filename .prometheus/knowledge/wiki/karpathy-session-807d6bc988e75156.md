---
type: SessionRecord
id: karpathy-session-807d6bc988e75156
title: Karpathy session 807d6bc988e7
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T10:49:31.613068+00:00
created_at: 2026-08-06T10:49:31.613068+00:00
updated_at: 2026-08-06T10:49:31.613068+00:00
revision: 0
---

## Delta

The landscape research is in and independently verified. Here's what it changed.

## The finding that reshapes the series

**OpenAI shipped `/goal` in Codex CLI 0.128.0 on 2026-04-30 — two months *before* your June article published.** Same plan→act→test→review→iterate cycle, same goal-conditioned termination, state persisted across sessions.

Your June comparison table implied that capability was Claude Code's differentiator. That row wasn't aged by events — **it was wrong on the day it published**, and one changelog check would have caught it.

I've moved it into **Part 1**, pairing it with the broken judge. Two confessions, same failure in both directions:

- The gate that never ran → *I didn't verify my own system.*
- The `/goal` row → *I didn't verify my claim about someone else's.*

That sets up the series thesis in the opening piece rather than deferring it to Part 5.

## Verdict on the thesis: commoditized, not validated

- Loop primitives converged across vendors **before the discourse named them**
- **Agent Skills won by adoption with no standards body** — OpenAI, Google, Cursor, and GitHub adopted rather than competed; Cursor ships `/migrate-to-skills` to convert its *own* proprietary rules
- Anthropic spent 2026-07-21→24 putting guardrails on its own fleets — capping subagents at 20, disabling nested spawns, then reinstating at depth 3, *"so one message can't fan out unbounded background agents"*

## One correction I made to the research

The agent reported METR "retracted" its slowdown study. **It didn't** — I checked METR directly. They *superseded* it, and their own caveat is the quotable part:

> "because of the selection effects in our experiment, our data is only very weak evidence for the size of this increase"

Calling that a retraction would be precisely the overclaim this series indicts. It's in the outline framed correctly.

## Eight external claims marked DO-NOT-USE

The research surfaced plausible-looking claims that don't survive checking — including two that are outright false ("Anthropic donated Agent Skills to the AAIF"; an MCP changelog entry "clarifying" its relationship to Skills), one unit slip (250M/week implies 2.5× the official figure — use **400–500M/month**), and several stale figures still circulating. All tabled with status.

## Outline status

`docs/articles/OUTLINE-loops-that-lie.md` — five parts, ~7,600 words total, each 1,400–1,600 words per Medium's 7-minute sweet spot. Landscape section complete.

The one thing still blocking a draft is your call on **AI disclosure**, since Medium's 2026 algorithm demotes detected AI-generated content and your provenance standard requires disclosing it. My recommendation is in the outline: keep the manifest, make each cold open unmistakably first-person.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T10:49:28.391837Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .kbd-orchestrator/phases/mobile-skill-portability/mobile-classification.json
- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-01bf5edf5a4599ef.md
- .prometheus/knowledge/wiki/karpathy-session-06fb138df2cedd2f.md
- .prometheus/knowledge/wiki/karpathy-session-273f3645230ec04d.md
- .prometheus/knowledge/wiki/karpathy-session-95cb4c69e3a13c3a.md
- .prometheus/knowledge/wiki/karpathy-session-b981029e4691b82f.md
- .prometheus/knowledge/wiki/karpathy-session-d36396632cc9ba19.md
- .prometheus/knowledge/wiki/karpathy-session-ed04ff580e446494.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-session-completion.md
- docs/articles/OUTLINE-loops-that-lie.md
