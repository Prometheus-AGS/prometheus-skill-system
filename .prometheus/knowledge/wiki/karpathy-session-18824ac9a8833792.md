---
type: SessionRecord
id: karpathy-session-18824ac9a8833792
title: Karpathy session 18824ac9a883
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T19:34:42.936866+00:00
created_at: 2026-08-05T19:34:42.936866+00:00
updated_at: 2026-08-05T19:34:42.936866+00:00
revision: 0
---

## Delta

Completed kbd-apply — kde-002-session-start (dropped, 4/5)

## Outcome: dropped, not shipped

The blocking t1 asked whether `kbd-status` is *suitable* — spec time only verified it was *present*. It isn't, and neither is any alternative.

| Candidate | Why it fails |
|---|---|
| `kbd-status` | Opens *"Show current KBD process state for the active project"* and reads waypoint/`progress.json`. Verified from a scratch dir: no `.kbd-orchestrator` resolves at all — nothing to render |
| `learn-harness` | Strongest on the axis that mattered (**zero** KBD-state refs), but auto-detects via `detect-surface-tier.sh`, which isn't inside the skill — it's in `shared/scripts/` and `ui-surface/scripts/` |
| `learn-about-system` | Manifest-only and its lone KBD mention is teaching material — but without `--area` it **asks the operator a question**, and `sessionStart` takes no arguments |

## One correction I made mid-investigation

I initially concluded `learn-harness` failed because scripts don't survive packaging. Wrong — **40 of the 145 installed skills do carry their `scripts/`**. `learn-harness` simply has none of its own. Different diagnosis, same verdict, but the wrong one would have implied a packaging bug that doesn't exist.

## Why dropping is the right result

AC3 permits exactly two outcomes when t1 fails: pick an already-suitable skill, or drop with the finding recorded. Fixing `kbd-status` is **out of scope** — a skill change belongs in its own change with its own review, not smuggled into a manifest change.

Shipping it anyway would have put a broken status report at the head of every Kimi Desktop session — the same inertness class this phase has hit four times, except user-visible.

Gates pass on the dropped branch: no `sessionStart` emitted, manifest parses, 145 skills intact, `kbd-status` unmodified.

Finding recorded at `phases/kimi-desktop-extensibility/session-start-finding.md`, including what *would* make this shippable: a small, argument-free, manifest-only orientation skill. None of the 145 currently qualifies.

Pushed as `6a08a43`. Next: `/kbd-apply kde-005-catalog-budget` — the last change.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T19:34:28.649389Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
