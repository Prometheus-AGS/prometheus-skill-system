---
type: SessionRecord
id: karpathy-session-2b059734bb7a30c2
title: Karpathy session 2b059734bb7a
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T19:39:08.387980+00:00
created_at: 2026-08-05T19:39:08.387980+00:00
updated_at: 2026-08-05T19:39:08.387980+00:00
revision: 0
---

## Delta

Completed kbd-apply — kde-005-catalog-budget (5/5, execute complete)

## Verdict: Kimi is not Codex

The premise carried since assess was **false for this runtime**:

| | Codex | Kimi Desktop |
|---|---|---|
| Limit | One **shared** budget | **Per-skill** cap |
| Adding a skill | Shortens every *other* description | Costs others nothing |
| Cap | Elastic, degrades with count | Fixed **250** (`LISTING_DESC_MAX`) |
| Count cap | Effectively yes | **None** |

`getModelSkillListing()` loops every skill with no slice, no limit, no running total. **So no curation is needed** — the `codex-catalog.txt` remedy solves a problem that doesn't exist here.

## The measurement found something the question would have hidden

**89 of 145 descriptions (61%) exceed 250 chars** — median **278**, max **662** (`gitops-transform`). The median skill is truncated, not the outlier.

What gets cut is the tail, which is usually the *"use when X, Y, Z"* guidance the model selects on. So the budget answer is reassuring, but a real quality loss sits underneath it.

Two useful details: `truncate` is grapheme-aware, and **`whenToUse` is emitted untruncated** — the escape hatch.

## Deliberately not done

Rewriting 89 descriptions is user-visible content editing and deserves its own change and review. Recommendation recorded: front-load the trigger into the first 250 chars (portable across harnesses) rather than reaching for the Kimi-specific `whenToUse`.

## Phase execute complete — 5/5

| Change | Outcome |
|---|---|
| `kde-000` | ✅ `skillInstructions` adopted |
| `kde-003` | ✅ `hooks` supported, `systemPrompt` silently ignored; found `commands` |
| `kde-001` | ✅ 3 MCP servers; forge auth removed at source |
| `kde-002` | ⊘ **Dropped** — no suitable `sessionStart` payload |
| `kde-005` | ✅ Per-skill cap measured, OQ-3 closed |

Four of five answered by reading the shipped loader rather than probing — stronger evidence, and no writes into app-managed state.

**Still unproven:** that Kimi Desktop actually connects to those MCP servers, and that a parsed hook spawns. Schema-valid ≠ working.

Pushed as `f8da3f5`. Next: `/kbd-reflect kimi-desktop-extensibility`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T19:34:33.614049Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
