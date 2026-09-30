## Context
See proposal.md. Existing team runtime commits are the starting point; dirty primary checkouts are preserved. Shared helpers ship TypeScript 7 sources and compiled Node ESM, with no runtime dependencies beyond Node and bundled data.

## Goals / Non-Goals
Deliver automatic, inspectable routing and team adoption offline. Defer native Impeccable detector/browser parity, live harness certification and platform execution unavailable on this host. Do not change design tokens, role ownership, native permissions or operator versions.

## Decisions
- A canonical prometheus-ui-ux skill owns protocol, catalog and a self-contained runtime. Flat mini distribution mirrors portable bytes from the full UI collection. Bootstrap/injector call this runtime rather than maintaining independent routing logic.
- Routing is deterministic from operation, domain, model, affected paths and manifests; context and role decisions remain inspectable JSON. Project protocol overrides a bundled default. Refinement/review exclude taste.
- Managed pointers defer loading until UI work. Installation preflights every destination, preserves unrelated prose and linked in-project entrypoints, rejects corrupt markers and path escapes, and retains recovery copies for changed files.
- Team export remains proposal-only; install-project handles explicit project writes. Existing roles[].skills carries bindings; manifests remain the authority. Zed instruction precedence is accounted for without inventing delegation.
- Upstream guidance is vendored at immutable revisions with licenses and adaptations. Native SDK requirements remain explicit. Impeccable core is a distinct adaptation.
- The Boss consumes committed mini payloads including data and helper modules; packaged helper discovery uses its available Node executable. No home installation when full-pack presence is detected.

## Risks / Trade-offs
- Large upstream runtime → port by responsibility; record parity outputs and omissions honestly.
- Instruction-file writes are a trust boundary → root containment, linked-target resolution, marker validation, preflight and recoverable writes.
- Cross-platform claims → Windows/Linux execution remains an explicit gap if unavailable.
- Existing staged Boss work → isolate bounded edits and never commit unrelated staged changes.

## Migration Plan
1. Persist research and contracts.
2. Implement catalog, Pro Max port, routing/bootstrap and team installer concurrently by ownership.
3. Generate distributions and wire Boss mini pin/packaged assets.
4. At the complete phase boundary run one consolidated local integration gate. Batch corrections and rerun only failed gates.
5. Commit scoped results locally. Recovery uses recorded original instruction bytes and repository commits; do not publish or release externally.

