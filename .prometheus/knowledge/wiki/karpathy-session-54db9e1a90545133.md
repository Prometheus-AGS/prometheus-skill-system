---
type: SessionRecord
id: karpathy-session-54db9e1a90545133
title: Karpathy session 54db9e1a9054
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T11:54:55.977985+00:00
created_at: 2026-11-17T11:54:55.977985+00:00
updated_at: 2026-11-17T11:54:55.977985+00:00
revision: 0
---

## Delta

The call graph confirms your concern: most skills are called by other skills, so making them user-invoked only would have broken the pack. I've withdrawn that recommendation in [PR #230](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/230) and changed no skills.

## Call graph

A new script, `scripts/certification/skill-call-graph.mjs`, records which skills name which other skills in their text.

- 139 of the 217 skills are named by at least one other skill.
- 25 of the 28 `kbd-*` skills are named by another skill. `kbd-reflect` has the most callers, at 15.
- Only `kbd-doctor`, `kbd-loop` and `kbd-bottleneck-detector` are named by no other skill.

The graph is static: it shows a skill is named, not that the hand-off happens at run time.

## Nested skills cost nothing

Claude Code registers only a plugin's top-level skills. A nested-only `kbd-status` could not be invoked by name (0 of 3); with a top-level copy it fired 3 of 3. So removing the 97 nested duplicates would save nothing on Claude Code, and I dropped that recommendation too.

## What each option saves

These are measured against the top-level listing of 72,736 characters and an assumed budget of 40,000.

| Option | Listing after | Share of budget |
|---|---|---|
| Today | 72,736 | 1.82x |
| Shorten descriptions of the 135 skills other skills call | 50,023 | 1.25x |
| Cap every description at 200 characters | 49,643 | 1.24x |
| Cap every description at 120 characters | 34,746 | 0.87x |

Only the 120-character cap fits, and at that length there is no room for the "use when" and "do not use" clauses the pilot showed matter. Shortening descriptions is worth doing but won't fix the full pack on its own.

## Domain plugins fit

Every domain plugin the marketplace already defines is well under the budget. The largest is `prometheus-ui-ux-skills` at 12,172 characters, and all of them together come to about 33,500.

The catch is coverage: they hold only 89 of the 213 skills. The Rust, Flutter and research-stage skills and most process skills, including `kbd-status`, are in no domain plugin, so today the full pack is the only way to get them.

So the one measured option that fits the budget with full descriptions is to complete the domain plugins and make them the normal install. That means assigning the 124 uncovered skills to a domain and adding the KBD lifecycle skills to the process plugin.

## Still unverified

The 40,000-character budget is derived from the documented 1% rule and an assumed four characters per token. I have not measured it directly.

## Decision for you

Should I complete the domain plugins now so they can become the normal install, or build the certification skill first and come back to this?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T11:54:42.847747Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
