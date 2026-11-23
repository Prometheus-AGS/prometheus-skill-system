---
type: SessionRecord
id: karpathy-session-9d0152b1af623fd6
title: Karpathy session 9d0152b1af62
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T10:07:28.828861+00:00
created_at: 2026-11-17T10:07:28.828861+00:00
updated_at: 2026-11-17T10:07:28.828861+00:00
revision: 0
---

## Delta

Phase 2 is done, and its main finding is about the pack as a whole: skills that trigger every time when loaded alone stop triggering once the full pack is loaded. `kbd-status` and `prometheus-rust-workspace` fired on 6 of 6 runs in isolation and 0 of 6 with all 218 skills present.

<pr-created>https://github.com/Prometheus-AGS/prometheus-skill-system/pull/227</pr-created>

<pr-created>https://github.com/Prometheus-AGS/prometheus-entity-management/pull/80</pr-created>

## What the pilot measured

I wrote 15 eval cases for five skills and ran each with and without the skill, three runs per arm, on Claude Code with `claude-sonnet-5-5`. Each case ran twice: with only that skill loaded, and with the whole generated pack loaded.

| Skill | Fired alone | Fired in full pack | Outcome gain when it fired |
|---|---|---|---|
| `learn-grade` | 6/6 | 6/6 | +0.58 to +0.67 |
| `feynman-loop` | 5/6 | 4/6 | +0.22 to +0.67 |
| `kbd-status` | 6/6 | 0/6 | +0.60 to +0.67 |
| `prometheus-rust-workspace` | 6/6 | 0/6 | +0.33 to +0.67 |
| `vlbv` | 0/6 | 0/6 | not measurable |

- **Where a skill fires, it works.** Every firing skill beat the no-skill baseline.
- **In the failing pack runs, no skill fired at all.** A diagnostic run confirmed the model invoked nothing, so these skills are not losing to a neighbour.
- **Likely cause, not verified:** the skill catalogue is larger than Claude Code presents in full. The repo already records the pack's description inventory at 44,882 characters with the Claude Code budget marked unmeasured.
- **`vlbv` never triggers**, even alone. Its description has no "use when" clause, which is what Phase 1's static check flagged for about 150 skills.
- **No false triggers:** all 30 near-miss runs stayed quiet.

The pilot cost $10.74 in total. One skill at this depth costs about $0.70 alone and about $1.00 in the pack. The report proposes pass thresholds from this data.

## Corrections and mistakes

- **I was wrong in the plan and Phase 1 report** to say the repo had no behavioural evals. `evals/vlbv-protocol/` is a cross-harness runner with reports for Claude Code and Kimi, and `evals/skill-activation/` holds 30 trigger cases. Phase 4 should extend that runner instead of adding Harbor beside it.
- **One of my own cases was defective.** My first `learn-grade` checks accepted only JSON while the skill reports a table, so a correct skill scored 0.42. After the fix it scored 0.75 and 1.00.
- **The downstream skills gate ran `cargo test`,** which I did not anticipate. It passed, but it was a Cargo build outside your one-at-a-time rule.

## What is in the PRs

- **Skill pack (#227):**
  - the 15 cases
  - `scripts/certification/run-skill-evals.sh` and `summarize-evals.mjs`
  - a generator change so `evals/` folders are not shipped in the built package
  - the [Phase 2 report](docs/reports/2026-11-17-certification-phase2-pilot.md), which ends with process notes written to become the certification skill
- **Entity management (#80):** the same frontmatter fix for its 28 source skills, which now pass the official validator 28 of 28. This was the only submodule that needed a change.

## Not verified

- **Checks were not re-run after the move onto `main`.** #226 had merged and `main` had 98 files of other changes, so I cherry-picked the commit onto it in a worktree. The build and validators passed before the move; the worktree has no submodules or `node_modules`, so I could not repeat them. The PR asks the reviewer to.
- **The runner wrapper script was syntax-checked only.** The pilot used the same commands directly.
- **The entity-management skills gate reported FAIL** in a checkout without installed dependencies. The one failure I traced fails identically on its unmodified `main`; I did not trace the other two.
- **Scope:** one harness, one model, three runs. No certificate was issued.

## Housekeeping

- A stale `.git/index.lock` appeared again with no git process running; I removed it.
- Your main checkout is on the local branch 

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T10:07:16.154799Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
