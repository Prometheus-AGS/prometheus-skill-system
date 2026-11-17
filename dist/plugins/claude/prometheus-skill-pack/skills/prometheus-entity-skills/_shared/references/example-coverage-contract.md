# Shared example coverage contract

Use this reference when an agent creates, modifies, reviews, or certifies a 3.0 showcase. The machine-readable sources remain authoritative:

- [`examples/shared/scenario-contract.json`](../../../examples/shared/scenario-contract.json) defines the domain, fixtures, transports, security assumptions, scenario IDs, and expected outcomes.
- [`examples/shared/scenario-contract.schema.json`](../../../examples/shared/scenario-contract.schema.json) defines the closed contract shape.
- [`examples/coverage.json`](../../../examples/coverage.json) maps stable capabilities and artifacts to scenarios and records downstream runtime and visual evidence.
- [`examples/coverage.schema.json`](../../../examples/coverage.schema.json) defines allowed evidence and showcase states.

Do not duplicate those inventories in generated code or skill text. Read them at task time so additions and removals fail visibly.

## Required workflow

1. Select scenario IDs from the canonical contract for the showcase's declared capabilities.
2. Preserve the Project/User/Task/Comment/Activity meanings, tenant boundary, fixed-clock behavior, ID-only lists, and deterministic keyless transport fixtures.
3. Implement the behavior through the target framework's proper binding. Components still read through hooks or bindings; stores/adapters own external I/O.
4. Add framework or platform tests that assert the shared expected outcomes rather than rewriting them locally.
5. Record reproducible commands and nonempty repository evidence paths in the showcase's `runtimeEvidence` and `visualEvidence`.
6. Run `pnpm run verify:example-coverage` and the owning showcase gates.

## Evidence interpretation

A passing semantic scenario proves that the shared fixture and expected outcome are coherent. It does not prove that a showcase implements the behavior. Keep the showcase `planned` until its own runtime evidence is implemented, and never fabricate browser/device, accessibility, screenshot, trace, video, golden, or hash receipts.

Overall example coverage may become `complete` only after every stable capability's release evidence and all five showcases are implemented. Publication still requires the separate release-certification and manual promotion gates.

## Public API effect

The shared contract is repository verification data and tooling. It does not add or change a package runtime export, so it does not justify regenerating `library-exports.json`. Run `pnpm run verify:skills` to prove the existing built React facade still matches that ledger. Refresh the ledger only when a publishable entry point actually changes.
