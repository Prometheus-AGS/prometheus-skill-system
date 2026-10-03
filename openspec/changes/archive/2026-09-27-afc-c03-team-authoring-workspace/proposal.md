## Why

The current agent-team creator can validate a supplied team or UAR package request, but it has no bounded authoring workspace that guides a user from team intent through a validated immutable package revision. C03 requires the full pack to preserve authored team semantics through migration and UAR export instead of reducing them to a flat manifest or accepting only top-level shape checks.

## What Changes

- Add a project-local, file-backed team authoring workspace with guided questions for agents, nested teams, communication edges, workflow dependencies, ownership, limits, and acceptance policy.
- Preserve the existing JSON `--input <request.json>` interface as a supported inline path; workspace-backed and inline authoring produce the same canonical validation and compilation result.
- Validate both top-level documents and nested team graphs against the pinned UAR collaboration profile `urn:prometheus:uar:collaboration:0.1.0-draft.2` schemas before a package can be built, versioned, or offered to UAR.
- Add immutable next-version maintenance: a changed definition or dependency creates a new semantic version and digest closure, while an existing version is never rewritten.
- Migrate existing schema-v1 team manifests and Draft 0.1.0-draft.1 authoring envelopes with JSON-pointer diagnostics that distinguish exact, translated, optional-unsupported, and required-unsupported fields. Required-unsupported fields block package creation.
- Update the canonical TypeScript source, compiled Node.js payload, skill guidance/assets/schema snapshots, and generated Claude/Codex distributions together.
- Add one final local live-UAR integration gate after the coherent implementation is complete. The gate exercises authoring, migration, package build, UAR preflight/install, cold status readback, and verifies that private grants and secrets never enter the portable package.

This is an additive change. Existing inline request files and schema-v1 project teams remain readable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-team-management`: Add bounded team-authoring workspaces, complete nested graph validation, immutable revision maintenance, lossless migration diagnostics, and UAR collaboration `0.1.0-draft.2` package generation while retaining inline input.

## Impact

- Initiative link: `afc-c03-lossless-definitions-and-collaboration-document-profile`, covering C03.1-C03.3 and REC-054/REC-055.
- Primary source: `skills/process/agent-team-creator/runtime/src/`, `assets/`, `schemas/`, `references/`, and `SKILL.md`.
- Generated artifacts: `skills/process/agent-team-creator/scripts/` and both `dist/plugins/{claude,codex}/prometheus-skill-pack/` trees through the existing distribution generator.
- Validation: the existing packaged runtime integration surface plus one final live UAR gate using an isolated project directory and UAR data root.
- External dependency: implementation waits for an immutable UAR commit publishing the official `0.1.0-draft.2` schema set. This repository consumes that exact schema set and records its commit/digests; it does not invent or reinterpret provider fields.
- No UAR, BossFang, mini-pack, host installation, service activation, or sibling-repository production files are changed by this repository-scoped change.
