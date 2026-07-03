EXECUTION: phase-sovereign-sync-hardening
Project: prometheus-skill-pack
Date: 2026-06-29T12:31:56Z
Selected backend: openspec
Dispatched to: /kbd-apply
Backend rationale: OpenSpec change skeletons already exist, the phase needs spec-backed traceability, and KBD must preserve progress/hook ownership. Bare /opsx:apply is explicitly disallowed by the execute protocol because it bypasses KBD hooks, progress.json, and waypoint refresh.
Backend entrypoint: /kbd-apply <change-id>
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/plan.md

EXECUTION SCOPE

- change-hardening-001-iroh-docs-share-import: Add iroh-docs share/import support and a two-node sync regression.
- change-hardening-002-sovereign-sync-ci: Add CI for sovereign-sync substrate crates.
- change-hardening-003-mcp-client-pool-e2e: Add end-to-end MCP client pool forwarding coverage.
- change-hardening-004-docusaurus-brand-and-lock: Apply KnowMe Docusaurus branding and package reproducibility.
- change-hardening-005-daemon-health-detect-toolchain: Add daemon health detection for localhost port 7892.

DISPATCH CONTRACTS

- change-hardening-001-iroh-docs-share-import → /kbd-apply
  Entry: /kbd-apply change-hardening-001-iroh-docs-share-import
  Model class: frontier
  Concrete model: gpt-5 (fallback; .kbd-orchestrator/project.json model_policy absent)
  Model rationale: public sync API and multi-node regression require frontier reasoning across iroh-docs, blobs, endpoint lifecycle, and async test behavior.
  Progress file: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
  Handoff: Read openspec/changes/change-hardening-001-iroh-docs-share-import/tasks.md, walk tasks through kbd-apply, update progress.json and waypoint on each task boundary.

  HANDOFF NOTE for /kbd-apply:
  1. Read .kbd-orchestrator/current-waypoint.json
  2. Read the change spec: openspec/changes/change-hardening-001-iroh-docs-share-import/
  3. On start: update progress.json status → IN_PROGRESS, started_by → kbd-apply
  4. On each task done: use kbd-apply end-task so hooks fire and waypoint/progress sync
  5. On completion: status → DONE, completed_by → kbd-apply; run artifact-refiner QA when required, then kbd-apply verify and archive
  6. On blocker: status → BLOCKED, add to blocked_changes, and refresh waypoint

- change-hardening-002-sovereign-sync-ci → /kbd-apply
  Entry: /kbd-apply change-hardening-002-sovereign-sync-ci
  Model class: small
  Concrete model: gpt-5 (fallback; .kbd-orchestrator/project.json model_policy absent)
  Model rationale: bounded workflow/config work; plan class normalized from "standard" to "small" for execute schema compatibility.
  Progress file: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
  Handoff: Read openspec/changes/change-hardening-002-sovereign-sync-ci/tasks.md, walk tasks through kbd-apply, update progress.json and waypoint on each task boundary.

  HANDOFF NOTE for /kbd-apply:
  1. Read .kbd-orchestrator/current-waypoint.json
  2. Read the change spec: openspec/changes/change-hardening-002-sovereign-sync-ci/
  3. On start: update progress.json status → IN_PROGRESS, started_by → kbd-apply
  4. On each task done: use kbd-apply end-task so hooks fire and waypoint/progress sync
  5. On completion: status → DONE, completed_by → kbd-apply; run artifact-refiner QA when required, then kbd-apply verify and archive
  6. On blocker: status → BLOCKED, add to blocked_changes, and refresh waypoint

- change-hardening-003-mcp-client-pool-e2e → /kbd-apply
  Entry: /kbd-apply change-hardening-003-mcp-client-pool-e2e
  Model class: frontier
  Concrete model: gpt-5 (fallback; .kbd-orchestrator/project.json model_policy absent)
  Model rationale: end-to-end transport/process lifecycle testing crosses implementation, fixture design, and CI determinism.
  Progress file: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
  Handoff: Read openspec/changes/change-hardening-003-mcp-client-pool-e2e/tasks.md, walk tasks through kbd-apply, update progress.json and waypoint on each task boundary.

  HANDOFF NOTE for /kbd-apply:
  1. Read .kbd-orchestrator/current-waypoint.json
  2. Read the change spec: openspec/changes/change-hardening-003-mcp-client-pool-e2e/
  3. On start: update progress.json status → IN_PROGRESS, started_by → kbd-apply
  4. On each task done: use kbd-apply end-task so hooks fire and waypoint/progress sync
  5. On completion: status → DONE, completed_by → kbd-apply; run artifact-refiner QA when required, then kbd-apply verify and archive
  6. On blocker: status → BLOCKED, add to blocked_changes, and refresh waypoint

- change-hardening-004-docusaurus-brand-and-lock → /kbd-apply
  Entry: /kbd-apply change-hardening-004-docusaurus-brand-and-lock
  Model class: small
  Concrete model: gpt-5 (fallback; .kbd-orchestrator/project.json model_policy absent)
  Model rationale: bounded docs-site theming and package-lock work; plan class normalized from "standard" to "small" for execute schema compatibility.
  Progress file: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
  Handoff: Read openspec/changes/change-hardening-004-docusaurus-brand-and-lock/tasks.md, walk tasks through kbd-apply, update progress.json and waypoint on each task boundary.

  HANDOFF NOTE for /kbd-apply:
  1. Read .kbd-orchestrator/current-waypoint.json
  2. Read the change spec: openspec/changes/change-hardening-004-docusaurus-brand-and-lock/
  3. On start: update progress.json status → IN_PROGRESS, started_by → kbd-apply
  4. On each task done: use kbd-apply end-task so hooks fire and waypoint/progress sync
  5. On completion: status → DONE, completed_by → kbd-apply; run artifact-refiner QA when required, then kbd-apply verify and archive
  6. On blocker: status → BLOCKED, add to blocked_changes, and refresh waypoint

- change-hardening-005-daemon-health-detect-toolchain → /kbd-apply
  Entry: /kbd-apply change-hardening-005-daemon-health-detect-toolchain
  Model class: small
  Concrete model: gpt-5 (fallback; .kbd-orchestrator/project.json model_policy absent)
  Model rationale: bounded health endpoint/diagnostic work; plan class normalized from "standard" to "small" for execute schema compatibility.
  Progress file: .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
  Handoff: Read openspec/changes/change-hardening-005-daemon-health-detect-toolchain/tasks.md, walk tasks through kbd-apply, update progress.json and waypoint on each task boundary.

  HANDOFF NOTE for /kbd-apply:
  1. Read .kbd-orchestrator/current-waypoint.json
  2. Read the change spec: openspec/changes/change-hardening-005-daemon-health-detect-toolchain/
  3. On start: update progress.json status → IN_PROGRESS, started_by → kbd-apply
  4. On each task done: use kbd-apply end-task so hooks fire and waypoint/progress sync
  5. On completion: status → DONE, completed_by → kbd-apply; run artifact-refiner QA when required, then kbd-apply verify and archive
  6. On blocker: status → BLOCKED, add to blocked_changes, and refresh waypoint

APPROVAL GATES

- Artifact-refiner QA gate after each completed change unless skipped by rule: fewer than 3 files modified, documentation-only, or explicit --skip-qa.
- OpenSpec verification and archive through `kbd-apply verify` and `kbd-apply archive`.

FALLBACK CONDITIONS

- If `/kbd-apply detect` cannot resolve OpenSpec, use the tasks files manually but still update KBD progress and hooks at task boundaries.
- If OpenSpec validation fails, keep the change unarchived and mark it BLOCKED with the failing validation output.
- If a task requires external credentials or account access, mark the change BLOCKED and record the missing input in progress.json.

VERIFICATION REQUIREMENTS

- change-hardening-001: `cargo test` in `substrate/storage-provider`; existing `substrate/sovereign-sync` tests remain green.
- change-hardening-002: local equivalents for workflow commands: `cargo fmt --check`, `cargo clippy`, and `cargo test` for relevant substrate crates.
- change-hardening-003: `cargo test` in `substrate/sovereign-sync`.
- change-hardening-004: docs package install/build or validation command for the Docusaurus package.
- change-hardening-005: relevant Rust and shell validation for health detection, including occupied-port behavior where practical.

PROGRESS LEDGER

- [DONE] change-hardening-001-iroh-docs-share-import — /kbd-apply
- [DONE] change-hardening-002-sovereign-sync-ci — /kbd-apply
- [DONE] change-hardening-003-mcp-client-pool-e2e — /kbd-apply
- [DONE] change-hardening-004-docusaurus-brand-and-lock — /kbd-apply
- [DONE] change-hardening-005-daemon-health-detect-toolchain — /kbd-apply

OUTPUTS

- .kbd-orchestrator/phases/phase-sovereign-sync-hardening/execution.md
- .kbd-orchestrator/phases/phase-sovereign-sync-hardening/progress.json
- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/current-waypoint.md
- .kbd-orchestrator/position-reminder.txt
- .kbd-orchestrator/phases/phase-sovereign-sync-hardening/model-routing.log

BLOCKERS

- No `.kbd-orchestrator/project.json` model_policy exists; concrete model routing falls back to gpt-5/frontier execution.
- No formal assessment handoff exists for this phase; execution relies on the completed plan handoff.

REFLECTION HANDOFF

- `kbd-reflect` should consume execution.md, progress.json, each OpenSpec change archive/validation result, artifact-refiner logs when applicable, and the verification command outputs recorded by each change.

EXECUTION READY
