---
type: SessionRecord
id: karpathy-session-01bf5edf5a4599ef
title: Karpathy session 01bf5edf5a45
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T09:24:28.434706+00:00
created_at: 2026-08-06T09:24:28.434706+00:00
updated_at: 2026-08-06T09:24:28.434706+00:00
revision: 0
---

## Delta

Restructured as a 5-part series at `docs/articles/OUTLINE-loops-that-lie.md`.

## What the format research changed

| Constraint | Finding | Effect |
|---|---|---|
| Length | **1,200–2,000 words**, 7-min sweet spot | Original was **7,616** — ~4× the ceiling |
| Series | **3–5 parts** is the recommended shape | 5 parts |
| Paragraphs | **2–4 sentences max** | Hard rule; original violated it often |
| Algorithm | Rewards reading time + **completion rate** | Five finishable pieces beat one abandoned wall |
| Curation | Curated reach is **10–100×** | Each part optimized for curation independently |

Same ~7,600-word budget, but five entry points, five curation chances, and four cliffhangers.

## The series

1. **"Eight Adversarial Reviews. Zero Adversaries."** (1,600w) — the confession. The anti-sycophancy gate the first article promoted with a ✅ had never once run. Five causes, every one producing a green check.
2. **"The Consensus Layer That Could Never Gain a Second Voter"** (1,500w) — OpenRaft with one voter, structurally unable to gain another. 236 MB backing a 49 KB journal; a static-constant `/health` taking 12 seconds. Deleted entirely.
3. **"Green Tests That Assert Nothing"** (1,600w) — 26 tests green in `0.00s`; the 18-of-18 residual-class failure; the drift check that could never catch its own bug. Likely the best standalone performer.
4. **"From Markdown to a Compiled Core"** (1,500w) — `substrate/` didn't exist when the first article published. Now 55,732 LOC of Rust.
5. **"37 Phases, Two Zeros, and One 3.5"** (1,400w) — the closer, opening with my own miscount.

Leaning hard, as you asked: Part 1 is a confession that indicts your prior article, and Part 5 opens by admitting **three of my own numbers were wrong** on first pass (skills 311→**147**, Rust LOC 32k→**55.7k**, tests 234→**385**). All three caught by re-running the command. That's the thesis in miniature — the impressive number was always the unverified one.

## One thing you need to decide before drafting

**Medium actively detects and deprioritizes purely AI-generated content in 2026.** Your provenance standard requires disclosing AI drafting. That's a real distribution risk, and I've written three options into the outline — my recommendation is keeping the manifest but making the opening 200 words of each part unmistakably first-person, since detection keys on generic cadence rather than on disclosure.

## Still outstanding

The **landscape research is still running** — it's been going ~25 minutes and has produced a large transcript, but hasn't reported. It feeds one section of Part 5 only, so the series architecture isn't blocked. If it comes back thin, Part 5 works without it and I'd cut that section rather than pad it.

Five questions are at the end of the outline; the AI-disclosure one is the only one that blocks drafting.

**Sources:** [Teract 2026 Medium guide](https://www.teract.ai/resources/grow-medium-audience-2026) · [Grow on Medium in 2026](https://medium.com/@Saifullah-Ghanghro/grow-on-medium-in-2026-proven-new-strategies-that-actually-work-6196dd3a3a6d) · [Optimal structure and length](https://medium.com/@florian-schroeder/the-optimal-structure-and-length-for-medium-articles-in-2025-0bd49fdddd7c) · [Medium's Series feature](https://mediumcourse.com/how-to-use-mediums-series-feature-to-publish-articles-on-a-single-topic/) · [Titles, subtitles, kickers](https://medium.com/blogging-guide/medium-titles-subtitles-and-kickers-ce28a5700487)

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T09:24:14.755319Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
