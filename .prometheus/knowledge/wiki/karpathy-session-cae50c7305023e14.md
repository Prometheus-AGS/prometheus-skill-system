---
type: SessionRecord
id: karpathy-session-cae50c7305023e14
title: Karpathy session cae50c730502
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-13T17:10:26.509345+00:00
created_at: 2026-08-13T17:10:26.509345+00:00
updated_at: 2026-08-13T17:10:26.509345+00:00
revision: 0
---

## Delta

The gate works and it found **19**, not 3. My audit undercounted by 6×. Stopping to report before I clean anything up, because the number has now moved twice and you need the real one.

## The gate is proven, and it caught my audit lying

**19 skills are unreachable at their canonical name** across 14 targets. I reported 3. I was wrong by 6×, for a reason worth stating precisely:

My audit asked *"is this a symlink that resolves?"* Sixteen of the nineteen **are** symlinks and **do** resolve — into `~/Projects/prometheus/prometheus-skill-pack/skills/imported/` (a live source checkout, April 15) or `~/Projects/travisjames/skills/` (July 29). Not into the installed generation. My check could not tell those apart. The installer's own `isManagedSkillLink` can.

That is the fourth time this session I've built a check whose scope was narrower than the claim I attached to it.

| Cause | Count | Example |
|---|---|---|
| Symlink → live source checkout | 16 | `sycophancy-correction` in 10 targets, `artifact-refiner` in 6 |
| Unowned directory | 3 | `deep-research` ×2, `kreuzberg` ×1 |

**And one of them is serving stale content right now:** `artifact-refiner` from the checkout hashes `615d98ed`; the generation has `3c776104`. Different. Anything loading it gets the April version. `sycophancy-correction` happens to match — today.

## What's done

- **Generation seal repaired.** My `stampProvenance()` had written two files inside the signed, content-addressed generation, breaking `--verify`. Moved beside it; `--verify` now exits 0. The installer's self-hash check caught my edit and forced a manifest regen — that guard works.
- **Collision gate built and observed failing**, listing every collision with occupant, mtime, and a `mv` command per case.

## What I have not done

No cleanup. Nineteen occupants need moving aside, and three of them (`deep-research`, `kreuzberg`, plus the `travisjames` links) may be things you deliberately placed. Before I touch them:

The 16 source-checkout symlinks look like an **older install convention** — link to the repo instead of the generation. If that was deliberate, moving them aside changes how those skills resolve. I'd archive rather than delete, as with the July tree, but this is 19 paths across 14 tools and I'm not doing it unasked after being wrong about the count twice.

Still owed: the completeness gate (step 2), replacing the fake freshness check (step 3), the §D-6-gated rule change, and the postmortem record.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-13T17:05:16.646865Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- hooks/codex-hooks.json
- hooks/hooks.json
- scripts/install-plugin-generation.js
- scripts/install.js
- shared/harnesses/generated/claude-hooks.json
- shared/harnesses/generated/release-manifest.json
- substrate/kbd-runtime/src/lib.rs
- .kbd-orchestrator/assessments/
- .prometheus/knowledge/wiki/karpathy-session-1043c07644fc0d61.md
- .prometheus/knowledge/wiki/karpathy-session-11edae69a3be4e37.md
- .prometheus/knowledge/wiki/karpathy-session-3b4d4bf41e85a6e4.md
- .prometheus/knowledge/wiki/karpathy-session-580ea375f49c30d3.md
- .prometheus/knowledge/wiki/karpathy-session-690e8f1510b4138a.md
- .prometheus/knowledge/wiki/karpathy-session-6fdf6d6855e9fa08.md
- .prometheus/knowledge/wiki/karpathy-session-730d78a8b336f747.md
- .prometheus/knowledge/wiki/karpathy-session-987780abbd4fc4e1.md
