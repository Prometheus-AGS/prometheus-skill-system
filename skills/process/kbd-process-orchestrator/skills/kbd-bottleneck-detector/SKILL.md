---
license: MIT
name: kbd-bottleneck-detector
version: '1.0.0'
description: >
  Evaluate or repair canonical KBD task, phase, and ZeeSpec boundaries. Use
  when progress receipts, projections, or build gates may be stale, or when
  the user mentions "bottleneck detector". Do NOT use for creating or
  advancing phases (see kbd-new-child and kbd-next-phase).
metadata:
  tags: [process, orchestration, recovery]
---

# /kbd-bottleneck-detector

Use the bundled adapter for deterministic local checks:

```bash
DETECTOR="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}/skills/kbd-bottleneck-detector/scripts/kbd-bottleneck-detector.sh"
"$DETECTOR" status
"$DETECTOR" evaluate task before <task-id>
"$DETECTOR" repair phase after <phase-id>
```

`evaluate` does not repair projections. `repair` may rewrite only derived KBD
waypoint, progress, and position projections; it must not change the canonical
revision. Treat `blocked` as a real lifecycle blocker and use the exact progress
signal returned by the command.

Read [references/receipt-contract.md](references/receipt-contract.md) when
diagnosing duplicate, missing, or out-of-order receipts, certification failures,
or authority ambiguity.

Ordinary evaluation is deterministic, local, and network-free. At phase or
child completion, on ambiguous canonical authority, or after the same violation
twice at one revision, load and run the installed `adversarial-review` skill.
It produces `findings.json`; screen that file with its
`scripts/check-findings-sycophancy.sh --findings <path>` adapter, which invokes
`sycophancy-correction`. If screening exits `2`, its stdout is judge feedback:
pass it to one `dispatch-judge.sh --feedback <file>` regeneration only when the
reported score is at least `0.4` or a high/critical pattern exists. An unavailable
or unresolved review becomes `pending_review` and blocks certification, not
implementation; this terminal certification rule is intentionally stricter than
ordinary best-effort artifact screening.

Never attach this detector to operator-requested Stop. Stop remains advisory and
fail-open.
