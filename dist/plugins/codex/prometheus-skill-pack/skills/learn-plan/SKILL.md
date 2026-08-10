---
name: learn-plan
description: Adaptive curriculum planner for the Feynman learning loop. Reads the survey result and learner model to build a concept DAG in surreal-memory, sequences prerequisites, estimates time per concept, and produces a curriculum.json that feynman-loop and learn-practice consume. Supports --replan when mastery diverges.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, plan, curriculum, dag, prerequisites, adaptive, surreal-memory]
---

# learn-plan

Adaptive curriculum planner for the Feynman learning loop. Reads the survey
result and learner model, builds a concept prerequisite DAG in surreal-memory,
topologically sorts the concepts, estimates time per concept, and writes a
`curriculum.json` that `feynman-loop` and `learn-practice` consume.

## When to invoke

Run after `/learn-survey` completes and `survey-result.json` exists. Invoke as:

```
/learn-plan <goal-id> [--replan]
```

The `goal-id` references artifacts at
`~/.prometheus/learn/goals/<goal-id>/`.

## Flow

### 1. Load inputs

Read both artifacts from `~/.prometheus/learn/goals/<goal-id>/`:

- `goal.json` — contains `corpus_path` and goal metadata
- `survey-result.json` — contains owned concepts, misconceptions, recursion floor

If either file is missing, halt with a clear error message and the path that
was not found.

### 2. Build concept DAG

From the corpus at `goal.corpus_path`, extract all concept entities. For each:

- Call `create_entity(name, entityType="concept", observations=[...])` in
  surreal-memory to register the node.
- Call `create_relation(from, to, relationType="requires_prerequisite")` for
  every prerequisite edge.

Concepts already present in surreal-memory (detected via `search_entities`) are
not re-created — only missing ones are added.

### 3. Topological sort

Order concepts from most-prerequisite (leaves) to most-advanced (roots):

- Concepts whose `concept_id` appears in the survey result's
  `recursion_floor_concepts` list are placed at position 0 and marked
  `status: "owned"`.
- Remaining concepts are ordered by shortest prerequisite chain to the goal.
- Concepts with equal depth are ordered alphabetically for stability.

### 4. Estimate time per concept

Use corpus confidence and concept complexity to produce a range:

| Complexity class     | Estimated range |
|----------------------|-----------------|
| Simple definition    | 15–30 min       |
| Applied concept      | 45–90 min       |
| Complex system       | 2–4 hours       |

Output both `estimated_minutes_min` and `estimated_minutes_max`. Do not
collapse to a single number — the range communicates uncertainty honestly.

Derive complexity class from corpus evidence: number of sub-concepts, cross-
references, and whether the concept has known failure modes or exceptions.

### 5. Write curriculum.json

Assemble the sorted phases and persist:

```bash
bash scripts/write-curriculum.sh \
  --goal-id "<goal-id>" \
  --curriculum-json '<json>'
```

See **Curriculum schema** below for the full shape.

### 6. Render plan via ui-surface

The plan output format depends on the learner's ui-surface tier:

- **Tier 0 (markdown):** emit a numbered list with concept label and estimated
  time range. Owned concepts are prefixed with `[owned]`.
- **Tier 2 (mindmap):** if `generate_ideation_mindmap` is available in
  surreal-memory, call it with the concept DAG and embed the output. Fall back
  to Tier 0 if the tool is absent or returns an error.

### 7. Replan mode (`--replan`)

Replan is triggered when either:

- The `--replan` flag is passed explicitly, or
- Any concept's actual mastery (from learner-model) is > 0.2 below the
  expected mastery at the current phase index in the existing curriculum.

When replanning:

1. Keep all concepts with `status: "owned"` in place.
2. For remaining concepts, compute gap = `expected_mastery - actual_mastery`.
3. Re-sort by descending gap size so the widest gaps are addressed first.
4. Recompute time estimates (mastery < 0.5 → add 20% to estimated range).
5. Overwrite `curriculum.json` with the new ordering.
6. Log a `replan_reason` field in the new curriculum.

## Curriculum schema

```json
{
  "curriculum_id": "string",
  "goal_id": "string",
  "created_at": "ISO datetime",
  "total_estimated_hours_min": "number",
  "total_estimated_hours_max": "number",
  "replan_reason": "string|null",
  "phases": [
    {
      "phase_index": 0,
      "concept_id": "string",
      "label": "string",
      "status": "owned|pending|in_progress|complete",
      "prerequisites": ["concept-id"],
      "estimated_minutes_min": 30,
      "estimated_minutes_max": 60,
      "mastery_at_start": 0.0,
      "audience_levels_required": ["novice", "peer"]
    }
  ]
}
```

Field notes:

- `curriculum_id`: `<goal-id>-<ISO-date>` (e.g., `ml-fundamentals-2026-06-28`)
- `replan_reason`: `null` on initial plan; descriptive string on replan
- `mastery_at_start`: pulled from learner-model at the time of planning
- `audience_levels_required`: Feynman depth levels the learner must reach to
  mark the concept complete. Minimum is always `["novice"]`.

## Prerequisite gating rule

A concept phase does NOT begin until all entries in its `prerequisites` list
have `status: "complete"` or `status: "owned"`. The plan enforces this
ordering. `feynman-loop` must not skip ahead — it must check this constraint
before starting any phase.

## Handoff

After writing `curriculum.json`, emit the following summary:

```
Curriculum: <N> concepts, <X>–<Y> hours total
<N-owned> already owned (recursion floor), <N-new> to learn
Next: /feynman-loop --concept-id <first-concept-id> --goal-id <goal-id> --depth 0
```

Where `<first-concept-id>` is the first concept with `status: "pending"` (the
first non-owned concept in phase order).

## Error conditions

| Condition | Action |
|-----------|--------|
| `goal.json` missing | Halt: print path, suggest running `/learn-goal` |
| `survey-result.json` missing | Halt: print path, suggest running `/learn-survey` |
| Corpus path not found | Halt: print `goal.corpus_path`, suggest `/learn-goal --fix` |
| surreal-memory unavailable | Warn; continue without DAG entities; skip mindmap render |
| `jq` not found | Halt with install instructions |
| Zero concepts extracted | Halt: corpus may be empty or unreadable |
