# Handoff in — kbd-control-plane-recovery› bottleneck-evaluator-and-detector

**Spawned by:** kbd-control-plane-recovery

## Why this child was spawned

Long agent runs repeatedly lost build/test policy, progress signaling, and the
canonical waypoint. The parent itself demonstrated the defect: its authored
four-change plan was projected as zero changes and pointed back to assessment.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/kbd-control-plane-recovery/assessment.md
- .kbd-orchestrator/phases/kbd-control-plane-recovery/plan.md

## Success criteria

- Signed boundary and gate receipts replay through the canonical KBD runtime.
- A deterministic guard repairs projections only and never blocks operator Stop.
- KBD/OpenSpec/ZeeSpec lifecycle boundaries surface revision-bound progress.
- Rust gates enforce implementation-first timing and machine-wide build exclusion.
- Terminal adversarial review is screened for sycophancy and locally certified.

## Expected deliverables

- `prometheus kbd guard evaluate` and `prometheus kbd gate run`.
- The discoverable `kbd-bottleneck-detector` skill and portable hook wiring.
- Local full-integration evidence plus a child-to-parent handoff.
