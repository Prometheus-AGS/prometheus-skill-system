# change-cpc-004-relocate-sovereign-sync

**Title:** Relocate sovereign-sync, sovereign-client, kbd-mobile, and the iroh-docs adapter into the Companion workspace  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-003-companion-workspace`  
**Backend:** native-kbd

## Why

The node exists as `substrate/sovereign-sync` 1.8.0 in the pack. Under D-02 it, sovereign-client, kbd-mobile, and storage-provider's `iroh_docs.rs` are product, not operational core, and move to the Companion as library crates under Companion governance (no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer).

## What Changes

- Copy the four sources into `crates/sovereign-sync`, `crates/sovereign-client`, `crates/kbd-mobile`, and a new `crates/iroh-docs-adapter` (implements the pack's `StorageProvider` trait), recording the pack source commit in each crate's README.
- Replace path dependencies with the git-rev pack dependencies from change-cpc-003; keep the `sovereign-sync` binary (modes `daemon`, `mcp`, `status`, `pair-*`) as a Companion-built binary.
- Remove `unwrap()` from library code in favour of typed errors (test code may keep it); make every pin exact; keep the Unix-socket REST/MCP/SSE surface and the `--token-file` TCP path unchanged in behaviour.
- Port the 22 axum-boundary tests, the 4 domain_sync tests, and kbd-mobile's wire_compat test and keep them passing as library regressions.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `Cargo.toml`
- `crates/iroh-docs-adapter/`
- `crates/iroh-docs-adapter/Cargo.toml`
- `crates/iroh-docs-adapter/src/`
- `crates/kbd-mobile/`
- `crates/kbd-mobile/Cargo.toml`
- `crates/kbd-mobile/src/`
- `crates/kbd-mobile/tests/wire_compat.rs`
- `crates/sovereign-client/`
- `crates/sovereign-client/Cargo.toml`
- `crates/sovereign-client/src/`
- `crates/sovereign-sync/`
- `crates/sovereign-sync/Cargo.toml`
- `crates/sovereign-sync/src/`
- `crates/sovereign-sync/tests/cli_socket_contract.rs`
- `crates/sovereign-sync/tests/domain_sync.rs`
- `crates/sovereign-sync/tests/integration_tests.rs`

## Capabilities

- `companion-node-crates` (new)

## ADDED Requirements

### Requirement: Relocated crates preserve their integration contracts
The relocated sovereign-sync SHALL pass its ported axum-boundary and two-AppState tests unchanged, and its REST routes, socket path, and signed-v2 receipt semantics SHALL be identical to the pack's 1.8.0 behaviour.

#### Scenario: Ported test targets
- **WHEN** `--test integration_tests` and `--test domain_sync` run in the Companion workspace
- **THEN** 22 and 4 tests pass respectively with no assertion changes

#### Scenario: Signed command over the socket
- **WHEN** the pack's installed `prometheus kbd` submits a signed command to the Companion-built daemon over the socket
- **THEN** the receipt is committed and readable back

### Requirement: Companion governance applies to library code
The relocated library code SHALL contain no `unwrap()` and no wildcard or caret version, and the workspace SHALL pass `audit-all.sh`.

#### Scenario: Lint gate
- **WHEN** `cargo clippy --lib -- -D clippy::unwrap_used` runs for each relocated crate
- **THEN** it passes

#### Scenario: Audit gate
- **WHEN** `bash scripts/audit-all.sh` runs
- **THEN** it exits 0

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: any crate this change introduces is staged in `docs/decisions/versions-pins-proposal.md` (change-cpc-003) and the change stays BLOCKED until the operator's `versions.toml` edit includes it.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- MIT history of sovereign-sync remains public; licensing of the relocated code is the operator's call and is not decided here.
