# Evolver Integration: kbd-process-orchestrator

The iterative-evolver delegates its **Execute** phase to `kbd-process-orchestrator`
when the evolution domain is `software` and KBD infrastructure is available.
This creates a **nested PMPO loop** — the evolver runs the strategic outer cycle
while KBD handles fine-grained task decomposition and multi-tool execution.

**Skill location**: `skills/process/kbd-process-orchestrator/SKILL.md`
**Entry command**: `/kbd-plan`, `/kbd-execute`, `/kbd-reflect`
**State backend**: `.kbd-orchestrator/phases/<phase>/`

---

## Architecture: Nested PMPO Loops

```
OUTER LOOP (iterative-evolver) — strategic evolution
  Assess → Analyze → Plan → Execute → Reflect
                              │
                    ┌─────────┴──────────┐
                    │  INNER LOOP (KBD)  │
                    │  Plan → Execute →  │
                    │  Reflect           │
                    │    │               │
                    │    ├─ OpenSpec     │
                    │    │  detection    │
                    │    ├─ Multi-tool   │
                    │    │  dispatch     │
                    │    └─ Artifact     │
                    │       refiner QA   │
                    └────────────────────┘
```

The evolver owns **what** to improve (goals, priorities, constraints).
KBD owns **how** to implement changes (tool selection, task decomposition, progress tracking).

---

## When to Delegate to KBD

| Condition | Delegate? |
|-----------|-----------|
| `evolution_domain: software` AND `.kbd-orchestrator/` exists | YES |
| `evolution_domain: software` AND project has `AGENTS.md` or `CLAUDE.md` | YES (KBD auto-initializes) |
| `evolution_domain: software` AND user passes `--no-kbd` | NO |
| Non-software domain | NO |

---

## Delegation Protocol (Evolver → KBD)

### 1. Map Plan Items to KBD Phase

The evolver's `plan.json` items become KBD phase goals:

```json
// plan.json (evolver)
{
  "items": [
    { "id": "item-1", "action": "Add input validation to API endpoints", "priority": "high" },
    { "id": "item-2", "action": "Increase test coverage to 80%", "priority": "medium" }
  ]
}
```

Becomes:

```
/kbd-new-phase "<evolution-name>-execute"
  goals: ["Add input validation to API endpoints", "Increase test coverage to 80%"]
```

### 2. Write Evolver Bridge

Create `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`:

```json
{
  "evolution_name": "<evolution-name>",
  "evolver_plan_path": ".evolver/evolutions/<name>/plan.json",
  "item_to_change_map": {}
}
```

The `item_to_change_map` is populated by `/kbd-plan` during change decomposition.

### 3. Run KBD Inner Loop

```
/kbd-plan    → decomposes evolver items into fine-grained changes
/kbd-execute → dispatches changes to best-fit tools, runs artifact-refiner QA
/kbd-reflect → aggregates results, writes back to evolver via bridge
```

### 4. Read Results Back

After `/kbd-reflect`, the evolver reads:
- `.kbd-orchestrator/phases/<phase>/reflection.md` — full phase retrospective
- `.kbd-orchestrator/phases/<phase>/evolver-bridge.json` — item completion map
- `.refiner/artifacts/*/refinement_log.md` — QA metrics

Update `evolution_state.json`:
```json
{
  "execution_results": {
    "delegate": "kbd-process-orchestrator",
    "phase": "<phase-name>",
    "changes_planned": 8,
    "changes_completed": 7,
    "changes_blocked": 1,
    "artifact_quality": {
      "first_pass_rate": 0.75,
      "recurring_violations": ["no-any-types", "test-coverage-80"]
    },
    "item_completion": {
      "item-1": "complete",
      "item-2": "partial"
    }
  }
}
```

---

## OpenSpec Awareness

The evolver does NOT need to detect or manage OpenSpec directly. KBD
handles this transparently:

- If `openspec/` exists → KBD emits OpenSpec changes with traceability
- If not → KBD uses native change management
- The evolver sees the same results either way via the reflection and bridge

---

## Artifact-Refiner QA (via KBD)

KBD invokes artifact-refiner per completed change. The evolver benefits from
this automatically — QA metrics appear in the KBD reflection and are
propagated back through the evolver bridge.

The evolver's `/evolve-report` aggregates these metrics into the evolution
report under "Artifact Quality Metrics".

---

## State File Locations

| File | Owner | Purpose |
|------|-------|---------|
| `.evolver/evolutions/<name>/plan.json` | Evolver | Strategic plan items |
| `.evolver/evolutions/<name>/state.json` | Evolver | Evolution state with execution_results |
| `.kbd-orchestrator/phases/<phase>/evolver-bridge.json` | KBD (via evolver) | Maps evolver items ↔ KBD changes |
| `.kbd-orchestrator/phases/<phase>/reflection.md` | KBD | Phase retrospective |
| `.kbd-orchestrator/phases/<phase>/progress.json` | KBD + tools | Live task progress |
| `.refiner/artifacts/<change-id>/refinement_log.md` | Artifact-refiner | Per-change QA results |

---

## When NOT to Use KBD

- For non-software evolutions (business, product, content, etc.)
- When the evolution has only 1-2 plan items — the overhead of KBD phase
  management isn't justified
- When the user explicitly opts out with `--no-kbd`
- When the executing tool has its own sophisticated planning (use direct
  tool dispatch from `prompts/execute.md` instead)
