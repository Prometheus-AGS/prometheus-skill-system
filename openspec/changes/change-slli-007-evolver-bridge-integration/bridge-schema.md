# evolver-bridge.json Schema

Canonical documentation for the `evolver-bridge.json` file that links the
KBD inner loop to the `iterative-evolver` outer loop. This file was
previously only described informally in code comments.

**File location**: `.kbd-orchestrator/phases/<phase-name>/evolver-bridge.json`

**Written by**: `/kbd-plan` when creating a plan driven by an evolver cycle.

**Read by**:
- `/kbd-execute` (to look up `evolver_item_id` when writing back per-change results)
- `/kbd-reflect` (to compute per-item completion and update evolver state)

---

## Top-level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `evolution_name` | string | yes | The evolver's named evolution key. Matches the directory name under `.evolver/evolutions/`. |
| `evolver_plan_path` | string | yes | Relative path to the evolver's plan file: `.evolver/evolutions/<name>/plan.json` |
| `item_to_change_map` | object | yes | Map from evolver item ID → array of KBD change IDs |
| `execution_results` | array | no | Appended by `/kbd-execute` as changes complete. Empty on creation. |

---

## `item_to_change_map`

```json
{
  "evolver-item-1": ["change-001", "change-002"],
  "evolver-item-2": ["change-003"]
}
```

Keys are evolver item IDs from `plan.json`. Values are arrays of KBD change
IDs (from `progress.json`) that together fulfil that evolver item.

A single evolver item may map to multiple KBD changes (decomposed). A single
KBD change maps to exactly one evolver item.

---

## `execution_results` entries

Each entry is appended by the KBD executor after a change reaches `DONE`:

```json
{
  "change_id": "change-slli-003",
  "evolver_item_id": "evolver-item-2",
  "status": "completed",
  "completed_at": "2026-06-23T03:00:00Z"
}
```

| Field | Type | Values |
|-------|------|--------|
| `change_id` | string | KBD change ID (e.g. `change-slli-003`) |
| `evolver_item_id` | string | Matching key from `item_to_change_map` |
| `status` | enum | `completed` `skipped` `failed` |
| `completed_at` | ISO8601 | UTC timestamp of completion |

---

## Reflect Write-back Target

`/kbd-reflect` writes computed results into:

```
.evolver/evolutions/<evolution_name>/state.json
  └─ current_iteration
       └─ kbd_results
            ├─ phase: "<phase-name>"
            ├─ reflected_at: "<ISO8601>"
            └─ items:
                 ├─ evolver-item-1: "completed"
                 └─ evolver-item-2: "in_progress"
```

When all items are `completed`, `/kbd-reflect` also sets
`current_iteration.status = "ready_for_reflect"` to allow the outer
`/evolve-reflect` to proceed.

---

## Minimal Example

```json
{
  "evolution_name": "slli-integration",
  "evolver_plan_path": ".evolver/evolutions/slli-integration/plan.json",
  "item_to_change_map": {
    "evolver-item-1": ["change-slli-002", "change-slli-003"],
    "evolver-item-2": ["change-slli-001"],
    "evolver-item-3": ["change-slli-004", "change-slli-005"]
  },
  "execution_results": [
    {
      "change_id": "change-slli-002",
      "evolver_item_id": "evolver-item-1",
      "status": "completed",
      "completed_at": "2026-06-23T01:00:00Z"
    },
    {
      "change_id": "change-slli-003",
      "evolver_item_id": "evolver-item-1",
      "status": "completed",
      "completed_at": "2026-06-23T03:00:00Z"
    }
  ]
}
```
