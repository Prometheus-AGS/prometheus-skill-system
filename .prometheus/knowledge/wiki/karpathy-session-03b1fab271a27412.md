---
type: SessionRecord
id: karpathy-session-03b1fab271a27412
title: Karpathy session 03b1fab271a2
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T11:45:15.722988+00:00
created_at: 2026-11-17T11:45:15.722988+00:00
updated_at: 2026-11-17T11:45:15.722988+00:00
revision: 0
---

## Delta

The catalogue measurement is done: the pack's skill listing is about 1.8 times Claude Code's budget, and triggering falls steadily as the catalogue grows.

<pr-created>https://github.com/Prometheus-AGS/prometheus-skill-system/pull/230</pr-created>

## What I measured

Claude Code documents a listing budget of 1% of the model's context window and says it drops descriptions when the listing overflows. That works out to roughly 40,000 characters on a 1M-token model, assuming four characters per token.

The generated pack lists 217 top-level skills in about 72,700 characters. Another 97 nested skills would add 29,100 if they are listed.

I then built trimmed copies of the pack at several sizes and ran the failing cases against each, three runs per case:

| Catalogue size (listing characters) | `prometheus-rust-workspace` fired | `kbd-status` fired |
|---|---|---|
| Skill alone (about 1,000) | 3/3 | 3/3 |
| About 10,000 | not run | 3/8 |
| 12,500 | 3/3 | not run |
| About 21,000–22,000 | 3/3 and 2/3 | 0/3 |
| 42,400 | 0/3 | not run |
| Full pack, 72,700 | 0/3 | 0/3 |

- **A closely matched request holds until the budget region, then stops completely.** The Rust skill's prompt names Rust, Cargo and tokio. It flips somewhere between 22,000 and 42,400 characters, which brackets the 40,000 figure.
- **A loosely matched request fails much earlier.** The `kbd-status` prompt is "where are we on this project?". It degrades in a 29-skill catalogue of only 9,600 characters, well under any budget.
- **Nothing takes the skill's place.** When `kbd-status` didn't fire, no skill fired; the model read the files and answered directly.

So budget overflow is real for this pack, but it is not the only cause.

## What is not established

- **Budget versus dilution.** The decisive test is raising the budget and re-running the full pack. The eval tool doesn't pass that setting through, so I couldn't run it.
- **Whether nested skills count** towards the listing.
- **Real-machine behaviour.** The eval runs in a fresh home with no usage history. Claude Code keeps descriptions for your most-used skills, so your own results may differ.
- **`vlbv` is now merged** (PR #229), so the hook carries that default regardless of this.

One run of mine was invalid and is excluded: my first trimmed copies deleted the skill under test. The report says so.

## Recommendations in the report

1. Make command-style skills user-invoked only. The 28 `kbd-*` lifecycle skills are called by name, and marking them that way removes them from the listing.
2. Make the domain plugins the normal install and the full pack the exception.
3. Remove the 97 nested duplicates from the listing.
4. Carry must-have defaults in hooks, as VLBV now does.
5. Add a listing-size check to the release gate. The new `scripts/certification/skill-listing-size.mjs` provides the numbers.

None of these are implemented. The experiment cost $3.92.

## Decision for you

Before I build the certification skill: should I mark the command-style `kbd-*` skills as user-invoked only? It frees the most listing space, but it also stops those skills preloading into subagents and stops Claude from starting them on its own. The alternative is to skip that for now and go straight to building the certifier.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T11:44:57.400294Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
