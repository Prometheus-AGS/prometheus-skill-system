## Why

Issue #194 reports a phase blocked by duplicate change identities and misleading task-boundary failures. Current registration already refuses an existing map key; imported data, embedded identity and projection ambiguity must be diagnosed without assuming the reported root cause.

## What Changes

- Name offending phase/change/task keys and embedded IDs during reconcile and boundary diagnosis.
- Preserve existing registration rejection and prevent lossy duplicate legacy conversion.
- Add an explicit signed, revision-bound canonical identity repair with retained source records and receipt/task lineage.
- Document projection-only recovery separately from canonical repair; never rewrite the journal or infer completion.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kbd-control-plane-reconciliation`: loss-preserving identity diagnosis and authorized canonical repair.

## Impact

scope: `substrate/kbd-runtime/`, `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`, apply/reconcile source and operator docs. Reuses cand-001/cand-002 from the phase analysis; no new library. Parent phase: issues-194-195-196. Generated reconciliation and acceptance belong to the named final change `kbd-194-196-delivery-qualification`.
