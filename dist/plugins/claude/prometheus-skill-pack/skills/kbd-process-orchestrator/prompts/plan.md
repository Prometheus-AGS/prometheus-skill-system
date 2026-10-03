# KBD Process Orchestrator — Plan Phase

You are executing the **Plan** phase of the KBD lifecycle for the **current project**.

> **IMPORTANT**: Do NOT hard-code project names, technology stacks, or domain areas. Derive these from project context files.

## Goal

Produce a prioritized, ordered list of changes to implement during this phase. Each change must be a discrete, independently implementable, verifiable feature slice.

Also determine whether OpenSpec is available and emit the appropriate commands.

## Model Selection

**Required model class: `frontier`**

Read `project.json → model_policy.phases.kbd-plan`. Plan quality directly determines change scope, which controls how often expensive models are needed in subsequent phases — running plan on a smaller model amplifies cost downstream.

If the hosting model is not frontier-class, stop and emit:

```
MODEL MISMATCH: kbd-plan requires a frontier model.
Expected: <model from policy.registry.frontier.<active_environment>>
Re-invoke via prom-lanes/UAR with the correct model.
```

The hosting phase policy above is separate from task assignment. Before choosing
execution models, draft each change's concrete tasks and backend identities. Read
[task model selection](../skills/kbd-plan/references/task-model-selection.md) and
author one **Task model assignments** table in `plan.md`. Analyze each task's
reasoning, uncertainty, scope, tools, context, modalities and independence needs;
discover the installed harness and configured provider capabilities. Prefer
demonstrated task suitability, then cost/latency for close ties, while honoring
explicit user, project and budget constraints. Complexity classes summarize the
change; they do not override explicit task assignments or select a cheapest model.

See `references/model-routing.md` for the full routing contract.

## Inputs

- **Project identity**: `.kbd-orchestrator/project.json` or inferred
- **Phase assessment**: `.kbd-orchestrator/phases/<phase-name>/assessment.md`
- **Project rules**: `AGENTS.md`, `CLAUDE.md`
- **Canonical specs**: `openspec/specs/*.md` (if OpenSpec), or project spec files
- **Cross-tool progress**: `.kbd-orchestrator/phases/<phase>/progress.json`

## OpenSpec Detection

Check if `openspec/` directory exists at the project root.

- **YES** → use OpenSpec changes; emit `/opsx:new <change-id>` commands
- **NO** → use native KBD changes; emit `mkdir .kbd-orchestrator/changes/<id>` instructions

## Planning Rules

1. **One change = one vertical slice** — each change should cover the feature end-to-end (data layer, business logic, API, UI if applicable). Never create purely horizontal changes (e.g., "add types everywhere").

2. **Order by dependency** — if change B depends on change A, list A first.

3. **Order by customer value** — prefer changes that unlock visible capability over internal refactors.

4. **Keep changes implementable in one agent session** — if an area is too large, split into multiple changes.

5. **Assign task models and execution routes** — select from observed harness
   capabilities, including Codex, Claude Code, OpenCode, DeepSeek Harness and Kimi
   Code where available. Record a concrete provider/model and supported reasoning
   setting, dated rationale/evidence, native or liter-llm route, documented worker
   mechanism, native alternative, verification status and unresolved prerequisites.
   The worker contract includes scope, working directory, tools, skills and result
   handoff. liter-llm is inference, not a workspace worker. Do not launch agents or
   change provider configuration while planning.

6. **Estimate complexity** — use S (&lt; 1 hour), M (1–4 hours), L (4–8 hours) as a rough guide for a skilled AI agent, not for a human.

## Priority Rules (project-derived)

Read the project's [AGENTS.md](http://AGENTS.md) or [CLAUDE.md](http://CLAUDE.md) for priority guidance. In the absence of explicit priorities, apply:

1. Foundation / blocking dependencies first
2. User-facing features over internal tooling
3. Security and data integrity over convenience features
4. Revenue-enabling features over operational improvements

## Output Format

```
PLAN: <phase-name>
Project: <project-name>
Date: <ISO date>
OpenSpec available: YES | NO
Changes to implement: <count>

CHANGE LIST (ordered)
1. <change-id>: <one-line description>
   - Scope: <layers affected, e.g., ui | api | db | all>
   - Depends on: NONE | <change-id>
   - Recommended agent: <tool from registry>
   - Est. complexity: S | M | L
   - Complexity score: Low | Medium | High   # change summary, not task selection
   - Model class: small | medium | frontier  # project policy summary only
   - Customer value: HIGH | MEDIUM | LOW
   - Details: <2-3 sentences describing what to build>

2. ...

TASK MODEL ASSIGNMENTS
<Insert the Task model assignments table from the task model selection reference;
every concrete task has a key: full phase path + change ID + backend task ID.>

EXECUTION ROUND ORDER
Round 1 (parallel): <change-ids with no dependencies>
Round 2 (parallel): <change-ids depending on Round 1>
...

COMMANDS TO RUN
<if OpenSpec>:
/opsx:new <change-id-1>
/opsx:new <change-id-2>

<if no OpenSpec>:
mkdir -p .kbd-orchestrator/changes/<change-id-1>
# Create .kbd-orchestrator/changes/<change-id-1>/change.md

PLAN COMPLETE
```

## Sycophancy Self-Check

Before finalizing the plan, verify it is not sycophantic:

- **S-02 (Agreement Without Grounding)**: Does the plan assume feasibility of the user's stated goal without grounding in evidence from the assessment? If the assessment showed the goal requires 80h and the user asked for 12h, the plan must surface that conflict — not paper over it.
- **S-07 (Scope Creep Flattery)**: Does the plan expand scope beyond what the phase goals strictly require? Cut back.
- **S-03 (Caveat Collapse)**: Does the plan surface at least one trade-off, deferred item, or explicit scope cut? Plans with zero friction are a structural sycophancy signal.

If the `sycophancy-correction` MCP skill is available, invoke `detect_sycophancy` with `context.evaluation_domain: "pmpo_plan_phase"` and `strictness: standard` on the plan draft. See `references/integrations/sycophancy-correction.md` §Plan Phase for thresholds.

Write output to `.kbd-orchestrator/phases/<phase-name>/plan.md`.

After emitting OpenSpec or native change artifacts, add non-task prose references
to their matching scoped assignment entries. Preserve checkbox syntax, task titles
and canonical identity resolution. Reconcile generated tasks against the table
before handoff; review material changes and record unresolved routes. Execution
must recheck the selected route and record explicit alternatives rather than
silently downgrading. Legacy plans receive explicit selections at execution time.

After writing, refresh the waypoint:

- Update `.kbd-orchestrator/current-waypoint.json` → `next_pending_change` = first change ID
- Set `exact_next_command` to the first `/opsx:new` or change creation command
- Update `.kbd-orchestrator/current-waypoint.md` with the same data
