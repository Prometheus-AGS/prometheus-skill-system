## Context
Harnesses differ in configuration, agent discovery, team capabilities and permissions.

## Goals / Non-Goals
Provide guided and expert setup, offline task coordination, source-linked native exports and one identical runtime. Do not replace native execution loops or claim skill-level Cedar enforcement.

## Decisions
Use a versioned JSON manifest with agents, model policy and opaque native options/artifacts. A self-contained Node >=22 runtime uses TypeScript 7 .mts compiled to .mjs, with no runtime dependencies. Adapters return staged files, command descriptions, limitations and source URLs. Guidance recommends the smallest useful team and asks about budget and independent review. Atomic local state with exclusive locks and revision checks owns task transfers; portable packets carry Git identity, evidence and memory references. Model capabilities, strength and price must be declared rather than guessed from names. Optional MCP memory publication uses tool discovery and configured argument mappings; failures remain in a local outbox. Existing native settings are never silently overwritten. Generated artifacts require native-version review before installation. No auto-install to home directories.

## Risks / Trade-offs
DeepSeek teams remain experimental and opt-in. MiniMax canonical agents are user-scoped and mcode has no agent-selection flag. UAR transport is operator-configured against its deployed contract; exporting intent does not invent a team API. Cross-harness handoff starts a fresh session rather than translating internal session state.

## Verification
Complete production first; then exercise the compiled CLI through real child processes and temporary files. Verify all eight exports, ownership conflicts, handoffs, model constraints, offline memory, identical package payloads, skills/plugins, OpenSpec and both site builds. Perform adversarial review before publication.
