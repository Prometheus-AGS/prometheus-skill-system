---
name: evolve-execute
description: >
  Run only the Execute phase — carry out a previously created improvement plan.
  For software domain evolutions, delegates to kbd-process-orchestrator as the
  inner loop for fine-grained task execution with OpenSpec detection and
  artifact-refiner QA. Requires an existing plan.json.
---

# Evolve Execute

Run only the PMPO Execute phase. Carries out the actions in an existing improvement plan.

## Setup

1. Load `plan.json` (required)
2. Load `assessment.json` and `analysis.json` for context
3. Load domain adapter from `references/domain/<domain>.md`
4. Run only `prompts/execute.md`

## KBD Delegation (Software Domain)

When `evolution_domain` is `software` and a `.kbd-orchestrator/` directory exists
(or can be created), the Execute phase delegates to the KBD inner loop:

```
evolve-execute (outer loop)
  │
  ├─ Map plan.json items → KBD phase goals
  │   ├─ Create .kbd-orchestrator/phases/<phase>/ if needed
  │   └─ Write evolver-bridge.json linking items ↔ changes
  │
  ├─ Invoke KBD inner loop:
  │   ├─ /kbd-plan    → decomposes evolver items into fine-grained changes
  │   │                  auto-detects OpenSpec for structured change management
  │   ├─ /kbd-execute → dispatches to best tool per change
  │   │   └─ per-change: artifact-refiner QA gate
  │   └─ /kbd-reflect → reports back via evolver-bridge.json
  │
  └─ Read KBD results back into evolution_state.json
      ├─ execution_results.changes_completed
      ├─ execution_results.artifact_quality
      └─ execution_results.blockers
```

### When KBD is NOT available

For non-software domains, or when the user opts out with `--no-kbd`:

- Execute plan items directly using domain-appropriate tools
- Log actions to `evolution_log.md`
- Update `evolution_state.json` with results

### OpenSpec Detection (via KBD)

The KBD inner loop handles OpenSpec detection automatically. When it finds
an `openspec/` directory, changes are tracked as OpenSpec proposals with
full traceability. When not found, KBD uses its native change management.
The evolver does not need to detect OpenSpec directly — KBD abstracts this.

## Prerequisites

A plan must exist. If not, suggest running `/evolve-plan` first.

## User Input

The user will provide: $ARGUMENTS

Parse for:

- Specific phases or actions to execute (optional — defaults to all)
- Whether to skip approval gates
- `--no-kbd` to bypass KBD delegation
- `--skip-qa` to bypass artifact-refiner QA

## Output

- Execution results updated in `evolution_state.json`
- Execution log appended to `evolution_log.md`
- Created/modified files as specified in the plan
- If KBD delegated: `.kbd-orchestrator/phases/<phase>/` artifacts
- If KBD delegated: `.refiner/artifacts/` QA logs per change
