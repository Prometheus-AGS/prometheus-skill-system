# KBD Process Orchestrator — Reflect Phase

You are executing the **Reflect** phase of the KBD lifecycle for the **current project**.

> **IMPORTANT**: Do NOT hard-code project names, tech stacks, or crate structure. Derive all project-specific details from context files.

## Goal

Generate a complete phase reflection report that:

1. Measures goal achievement honestly, including cross-tool contributions
2. Surfaces technical debt introduced across all executing tools
3. Captures lessons for the knowledge base, attributed and scoped for recall
4. States which recalled lessons recurred
5. Seeds the next phase

## Model Selection

**Required model class: `frontier`**

Read `project.json → model_policy.phases.kbd-reflect`. Reflection is the quality judgment gate — it feeds `prior_assessments` for the next cycle. A degraded reflection propagates compounding errors. If the hosting model is not frontier-class, stop and emit `MODEL MISMATCH`:

```
MODEL MISMATCH: kbd-reflect requires a frontier model.
Expected: <model from policy.registry.frontier.<active_environment>>
Re-invoke via prom-lanes/UAR with the correct model.
```

See `references/model-routing.md` for the full routing contract.

## Inputs

- **Project identity**: `.kbd-orchestrator/project.json` or inferred
- **Phase goals**: from `assessment.md` and `plan.md`
- **Assessment**: `.kbd-orchestrator/phases/<phase>/assessment.md`
- **Plan**: `.kbd-orchestrator/phases/<phase>/plan.md`
- **Cross-tool progress**: `.kbd-orchestrator/phases/<phase>/progress.json`
- **Recalled lessons**: `.kbd-orchestrator/phases/<phase>/prior-context.md`
- **Archived changes**:
  - If OpenSpec: `openspec/changes/archive/<date>-<id>/` directories
  - If native KBD: `.kbd-orchestrator/changes/archive/<date>-<id>/` directories
- **Refinement logs** (if artifact-refiner was used): `.refiner/artifacts/`
- [**AGENTS.md**](http://AGENTS.md) — architectural rules to check integrity against

## Prerequisites

Before running reflect, verify all changes in this phase are complete:

- `progress.json` shows all changes as `DONE`
- If OpenSpec: all changes show `kbd-apply verify` and `kbd-apply archive` complete
- If native KBD: all change directories have been moved to `archive/`

If any changes are `BLOCKED`, note them explicitly and proceed with reflection on what was completed.

## Recalled lessons

Read `.kbd-orchestrator/phases/<phase>/prior-context.md` first (the
`reflect:before` hook refreshes it). For every recalled lesson, decide whether
it **recurred** in this phase (the same mistake or pattern showed up again) or
was **applied** (it prevented a repeat). Recurrence is a root-cause signal:
name it in Root Cause.

## Reflection Dimensions

The report leads with what diverged, not with what succeeded (S-08).

### 1. Delta

What diverged from the plan: unmet or partial goals, dropped or reworked
changes, regressions, verification gaps. One numbered item per divergence,
with evidence (progress.json, verification output, file paths).

### 2. Root Cause

For each delta, the cause — not the symptom. Include recalled lessons that
recurred (cite them from `prior-context.md`).

### 3. Corrective Actions

Concrete actions for the next phase, one per root cause. Prefix an action
`[GLOBAL]` when it applies to any project, `[USER]` when it is this operator's
preference; such actions are also stored as lessons in that scope.

### 4. Goal Achievement

For each stated phase goal: **MET | PARTIAL | NOT MET**, with an honest reason. Credit completed work regardless of which tool executed it. Calculate overall completion percentage.

### 5. What Was Delivered

List all changes that were implemented and archived, noting which tool executed each. Format: `- <change-id>` — (by: )

### 6. Technical Debt Introduced

List any shortcuts, stubs, TODOs, or known violations deferred from this phase. Be specific — mention file paths where known. Note which tool introduced the debt.

### 7. Architecture Integrity

Check against `AGENTS.md` "Never Do" section and `.kbd-orchestrator/constraints.md`:

- Were any "Never Do" rules violated?
- Are known constraint violations present?
- What technical patterns were broken?

### 8. Cross-Tool Coordination Review

- Were typed task transitions and their progress projections recorded reliably?
- Were there any gaps where state was lost between tools?
- What handoff notes worked well? What was unclear?

### 9. Lessons Learned

Concrete, reusable learnings, one bullet each, written so that an agent
reading it out of context can act on it. These bullets are written back to
memory one lesson per bullet (`reflect:after` → `memory-writeback.sh`):

- no prefix → project scope (this project's agents recall it)
- `[GLOBAL] …` → global scope (every project)
- `[USER] …` → this operator's user scope

### 10. Next Phase Seed

The recommended next phase name (as `phase-<slug>`) and its top 3 priority
areas. `/kbd-next-phase` seeds the next phase's goals from this section.

### 11. Codify as Skill?

Patterns that recurred often enough to become a skill or a skill change, or
`NONE`. This section is for the operator and is **never** written back to
memory.

## Output Format

Write to `.kbd-orchestrator/phases/<phase-name>/reflection.md`:

```markdown
# Phase Reflection: <phase-name>

**Project:** <project-name>
**Date:** <ISO date>
**Phase completion:** <N>%
**Changes completed:** <N> / <total>

## Delta

1. <what diverged from the plan, with evidence>

## Root Cause

1. <cause of delta 1; cite any recalled lesson that recurred>

## Corrective Actions

1. <action for root cause 1>
2. [GLOBAL] <action that applies to any project>

## Recalled Lessons

- Recurred: <lesson cited from prior-context.md> — <where it showed up again>
- Applied: <lesson> — <what it prevented>
- (NONE if prior-context.md held no applicable lessons)

## Goals

| Goal   | Status              | Notes           |
| ------ | ------------------- | --------------- |
| <goal> | MET/PARTIAL/NOT MET | <honest reason> |

## Delivered Changes

- `<change-id>` — <description> (by: <tool>)

## Technical Debt

- <specific debt item with file path or location>
- (NONE if clean)

## Architecture Integrity

- AGENTS.md violations: NONE | <violations found>
- Constraint violations: NONE | N/A | <specific violations>

## Cross-Tool Coordination Notes

- Progress tracking: RELIABLE | GAPS FOUND — <detail>
- Handoff quality: CLEAR | UNCLEAR — <detail>

## Lessons Learned

- <project lesson>
- [GLOBAL] <lesson for every project>
- [USER] <operator preference>

## Next Phase Seed

`phase-<slug>` — <top 3 priority areas>

## Codify as Skill?

- <recurring pattern worth a skill> | NONE

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation.
```

Older reflections used `## Next Phase Focus`; the write-back still reads it
as a fallback, but new reflections use `## Next Phase Seed`.

## Sycophancy Self-Check (MANDATORY)

Before finalizing this reflection, verify it is not sycophantic:

1. **S-08 (Reflect Phase Inversion)**: Does this reflection open with "successfully completed" or "all requirements met" before surfacing deltas and failures? If yes, restructure: lead with what diverged from the plan, then root causes, then corrective actions.
2. **S-03 (Caveat Collapse)**: Does the reflection surface at least one area of concern, trade-off, or technical debt item? If the phase was truly clean, state that explicitly with evidence — don't default to success language without verification.
3. **S-02 (Agreement Without Grounding)**: Does the "Goals" table independently verify goal status from `progress.json` data, or does it echo the plan's expected outcomes without checking execution reality?

### Invoking the sycophancy-correction skill

If the `sycophancy-correction` MCP skill is available (check for the `analyze_reflect_phase` tool in the MCP tool list), invoke it on the generated reflection **before writing the file**. Reflect-phase analysis is the skill's specialist domain (AC-08 applies — Delta → Root Cause → Corrective Actions structure is mandatory at the Reflect gate).

Invocation:

- Tool: `analyze_reflect_phase`
- `content`: the full text of the reflection draft
- `context.evaluation_domain`: `"pmpo_reflect_phase"`
- `strictness`: `"strict"`
- `correction_mode`: `"detect_only"` on the first pass

Action based on returned `sycophancy_score`:

ScoreAction&lt; 0.3Proceed — write reflection as-is0.3 – 0.5Annotate — append pattern notes to the reflection, warn user≥ 0.5Re-invoke with `correction_mode: "rewrite"`, use corrected output≥ 0.7 with S-08Block — do not write; regenerate reflection from scratch

Save the full tool response to `.kbd-orchestrator/phases/<phase>/sycophancy/reflect-<ISO-timestamp>.json`for audit trail.

**Caveat:** The sycophancy-correction skill's `AnthropicClient` is currently stubbed — rewrite outputs are placeholder text until that integration lands. Until then, treat `correction_mode: rewrite` output as advisory, not final. Pattern detection and scoring are fully functional.

After writing, record the reflect stage through typed `prometheus kbd stage`
commands and write its handoff. When advancing is authorized, use
`/kbd-next-phase` to activate the next phase and generate its projections.
Never hand-edit the waypoint or advance it merely because reflection was
written. Review and commit only the intended artifacts under project policy.
