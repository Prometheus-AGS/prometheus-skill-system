# change-cpc-011-integration-evidence

**Title:** Two-process G6 evidence and Companion-absent certification of the pack  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-005-node-hosting`, `change-cpc-006-discovery-and-pairing`, `change-cpc-007-supervisor-and-health`, `change-cpc-009-pack-removal`  
**Backend:** native-kbd

## Why

Goal 6 requires two-node runs through production entry points proving discovery, CRDT convergence, a signed KBD command over P2P, and service management, plus proof that the pack certifies fully without the Companion. The existing `domain_sync.rs` never starts its endpoints and is not that evidence (D-14). The pack-side certification script is authored in the pack (see change-cpc-013).

## What Changes

- Companion: a local two-process harness launching the headless binary and a second instance on separate data roots and ports, asserting mDNS address lookup, gossip join, a signed KBD command submitted with the pack's `prometheus kbd` on node A and observed on node B, and the supervisor's services surface.
- Docs on the Companion: installation, the contract as consumed, and release notes.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `crates/prometheus-substrate/tests/two_node.rs`
- `docs/00-architecture-and-implementation-plan.md`
- `docs/installation.md`

## Capabilities

- `control-plane-evidence` (new)

## ADDED Requirements

### Requirement: Two real nodes converge and deliver a signed command
WHEN the harness runs locally, THEN node B observes the signed command committed on node A with matching receipt hash and both journals converge.

#### Scenario: Convergence
- **WHEN** the harness completes
- **THEN** both nodes report the same frontier and the receipt appears on B's `events/stream`

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: any crate this change introduces is staged in `docs/decisions/versions-pins-proposal.md` (change-cpc-003) and the change stays BLOCKED until the operator's `versions.toml` edit includes it.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
