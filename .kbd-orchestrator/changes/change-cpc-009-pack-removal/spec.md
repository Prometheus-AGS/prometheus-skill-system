# change-cpc-009-pack-removal

**Title:** Remove the relocated crates, the sharing install path, and the daemon wiring from the pack  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-004-relocate-sovereign-sync`, `change-cpc-012-pack-sync-skills-removal`  
**Backend:** native-kbd

## Why

After change-cpc-004 and change-cpc-012 land, the pack still contains sovereign-sync, sovereign-client, kbd-mobile, the iroh-docs adapter, the sharing LaunchAgent and systemd unit, the `--sharing` install path, and 33 non-doc plus 42 doc files that name the daemon. The pack must be complete without any of them and emit no warning when the Companion is absent (D-02). The plist and unit edits are generator inputs for `shared/services.manifest.json` (change-cpc-001), so the manifest is regenerated here (C-01); the installers and probes are part of the Codex install flow, so `docs/codex-plugin.md` is updated here (C-03).

## What Changes

- Re-derive the footprint with the assessment command and record it; delete the four relocated sources and the iroh-docs feature; remove `--sharing` from `install-binaries.sh`, `install-skills-flat.sh`, `install-mcp-services.sh`, `install-system.js`; delete `install-sovereign-sync-sharing.sh`, the plist, and the systemd unit; regenerate `shared/services.manifest.json` and run `--check`.
- Repoint probes: `detect-toolchain.sh`, `service-probe.sh`, `check-mcp-health.sh`, `prometheus doctor`, and `setup.rs` treat the Companion node as an optional discovered endpoint (contract v1) and stay silent when absent; add the doctor test case.
- Retire the four sovereign-sync OpenSpec main specs from the pack (`sovereign-sync-daemon-health`, `sovereign-sync-ci`, `iroh-docs-adapter`, `mcp-client-pool`) with a decision record pointing at the Companion; delete `docs/SOVEREIGN_SYNC_TESTING.md`; cut the `estate` feature's sovereign-sync dependency in `crates/prometheus-exec` and `substrate/exec-remote` the same way skill-ffi's was cut (no pack crate references a relocated crate).
- The footprint criterion tolerates only three identifiers that contract v1 keeps by design (`sovereign-sync.sock`, `SOVEREIGN_SYNC_SOCKET`, `sovereign-sync/device-key.json`) and the historical records under `docs/decisions/`, `docs/audits/`, `docs/releases/`, `docs/reports/`, `CHANGELOG.md`, `openspec/changes/archive/`, and `memory/`.
- Update the four other plists that reference sovereign-sync in ordering or env; update `shared/scripts/tests/*`; update `site/docs`, `docs/codex-plugin.md`, and CLAUDE.md sections; regenerate plugin surfaces and run `npm run validate:codex` (C-01).

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `.claude-plugin/`
- `CLAUDE.md`
- `crates/prometheus-exec/`
- `docs/SOVEREIGN_SYNC_TESTING.md`
- `docs/codex-plugin.md`
- `docs/decisions/sovereign-sync-relocated-to-companion.md`
- `docs/guide/`
- `openspec/specs/iroh-docs-adapter/`
- `openspec/specs/mcp-client-pool/`
- `openspec/specs/sovereign-sync-ci/`
- `openspec/specs/sovereign-sync-daemon-health/`
- `scripts/check-mcp-health.sh`
- `scripts/install-binaries.sh`
- `scripts/install-mcp-services.sh`
- `scripts/install-skills-flat.sh`
- `scripts/install-sovereign-sync-sharing.sh`
- `scripts/install-system.js`
- `shared/launchagents/ai.prometheus.forge-mcp.plist`
- `shared/launchagents/ai.prometheus.liter-llm-api.plist`
- `shared/launchagents/ai.prometheus.pk-cherry.plist`
- `shared/launchagents/ai.prometheus.sovereign-sync.plist`
- `shared/launchagents/ai.prometheus.surface-bridge.plist`
- `shared/scripts/detect-toolchain.sh`
- `shared/scripts/service-probe.sh`
- `shared/scripts/tests/`
- `shared/services.manifest.json`
- `shared/systemd/ai.prometheus.sovereign-sync.service`
- `site/docs/`
- `site/scripts/`
- `substrate/exec-remote/`
- `substrate/kbd-mobile/`
- `substrate/sovereign-client/`
- `substrate/sovereign-sync/`
- `substrate/storage-provider/Cargo.toml`
- `substrate/storage-provider/src/iroh_docs.rs`
- `tools/cowork-skills/`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/control_transport.rs`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs`
- `tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs`

## Capabilities

- `pack-open-core` (new)

## ADDED Requirements

### Requirement: Pack contains no relocated code or daemon wiring
WHEN the daemon-name grep runs over the whole repository after the change, THEN nothing remains outside the contract-v1 identifiers and the historical records named in What Changes.

#### Scenario: Footprint
- **WHEN** the footprint command in `verification.md` runs (whole repo; excludes .git, node_modules, target, .kbd-orchestrator, .prometheus, .refiner, dist, memory, openspec/changes/archive; ignores the three contract-v1 identifiers; allows docs/integration-contract.md, docs/decisions, docs/audits, docs/releases, docs/reports, CHANGELOG.md)
- **THEN** it prints nothing

### Requirement: Service manifest stays in sync with its inputs
WHEN the plist and unit inputs change in this change, THEN `shared/services.manifest.json` is regenerated and `--check` passes.

#### Scenario: Manifest check
- **WHEN** `node scripts/generate-service-manifest.mjs --check` runs
- **THEN** it exits 0

### Requirement: Pack is silent and complete without the Companion
WHEN the pack is installed with no Companion, THEN `prometheus doctor`, `detect-toolchain.sh`, and a typed KBD mutation succeed with no warning about sharing or a control plane.

#### Scenario: Doctor
- **WHEN** `prometheus doctor --json` runs without the Companion
- **THEN** no item mentions sovereign-sync and the exit code is 0

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Unresolved review findings

Adversarial review (artifact mode, cross-model judge k3 via rest-gateway, `cross_model_check: verified-distinct`) ran two rounds on the change set. Round 1 (4 CRITICAL) was corrected in the regenerated set. Round 2 returned the CRITICAL below; its fix is applied in this change but, under the two-round cap, the finding is carried verbatim so plan and execute inherit it explicitly rather than trusting this document's own fix.

- CRITICAL (round 2, change-cpc-009-pack-removal/spec.md): "The whole-repo footprint acceptance criterion will fail on tracked files that are neither in the allowed match set nor in the change's declared scope." Evidence: `docs/SOVEREIGN_SYNC_TESTING.md`, `memory/project_sovereign_sync_plan.md`, archived OpenSpec proposals under `openspec/changes/archive/`, and `CHANGELOG.md` / `.mcp.json` were neither excluded by the grep nor in Scope. Disposition: the criterion now excludes archives, memory, and the knowledge and refiner stores, allows the historical docs and CHANGELOG, ignores the three contract-v1 identifiers, and the testing doc, the four OpenSpec main specs, and the prometheus-exec estate reference were added to Scope and task 1 (`.mcp.json` and `CHANGELOG.md` contain zero matches, verified 2026-09-02). Not re-vetted.
