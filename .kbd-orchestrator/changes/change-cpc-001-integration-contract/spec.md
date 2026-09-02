# change-cpc-001-integration-contract

**Title:** Publish the open integration contract the Companion and third parties consume (G7)  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** none  
**Backend:** native-kbd

## Why

The pack's extension seams exist but are implicit: the CLI transport chain discovers a control socket, hooks invoke run-hook bundles, and the LaunchAgent and systemd templates describe services, but nothing versions or documents them. A paid Companion or a third-party repository integrating today would need private knowledge of the pack. Operator decision D-02 makes the contract the first design item.

## What Changes

- Write `docs/integration-contract.md` v1.0 covering four seams: control-endpoint discovery (`PROMETHEUS_CONTROL_ENDPOINT`, `SOVEREIGN_SYNC_SOCKET`, then `data_local_dir()/prometheus/run/sovereign-sync.sock`; the socket name is kept in v1 for CLI compatibility), hook bundle extension points (`runtime/v1/run-hook --bundle <name>` and how an extension declares a bundle), the service manifest, and the connected-skill-package declaration (`skill-package.json`: name, version, skills dir, hooks, MCP servers, minimum contract version).
- Generate `shared/services.manifest.json` from `shared/launchagents/*.plist` and `shared/systemd/*` with `scripts/generate-service-manifest.mjs` (label, program, port or socket, health probe, dependency order, restart semantics); `--check` exits non-zero on drift (constraint C-04). This generator's inputs are C-01 generator inputs from this change onward: any later change that edits a plist or unit must regenerate and run `--check` in the same change.
- Add `prometheus contract show --json` printing the contract version, the discovered control endpoint (or `absent`, never a warning), and the manifest path, and `prometheus contract validate <skill-package.json>`; process-level CLI integration test.
- Publish `site/docs/kbd/integration-contract.md` and a CLAUDE.md pointer; the pack emits nothing when the Companion is absent.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `.kbd-orchestrator/constraints.md`
- `CLAUDE.md`
- `docs/integration-contract.md`
- `package.json`
- `scripts/generate-service-manifest.mjs`
- `shared/schemas/skill-package.schema.json`
- `shared/services.manifest.json`
- `site/docs/kbd/integration-contract.md`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/contract.rs`
- `tools/prometheus-cli/crates/prometheus-cli/src/main.rs`
- `tools/prometheus-cli/crates/prometheus-cli/tests/contract.rs`

## Capabilities

- `pack-integration-contract` (new)

## ADDED Requirements

### Requirement: Control endpoint discovery is deterministic and silent when absent
The CLI SHALL resolve the control endpoint in the documented order and report `absent` without emitting any warning or degraded-mode message when nothing is discovered.

#### Scenario: No endpoint configured or listening
- **WHEN** `prometheus contract show --json` runs with no env override and no socket present
- **THEN** the output reports `endpoint: absent`, exit code 0, and stderr is empty

#### Scenario: Endpoint discovered
- **WHEN** a listener exists at the documented socket path
- **THEN** the output names that path as the endpoint

### Requirement: Service manifest is generated and drift-checked
`shared/services.manifest.json` SHALL be generated from the LaunchAgent and systemd templates and SHALL be byte-identical across two generator runs; `--check` SHALL fail on drift.

#### Scenario: Idempotent generation
- **WHEN** the generator runs twice on an unchanged tree
- **THEN** both outputs are byte-identical

#### Scenario: Drift detected
- **WHEN** a template changes without regenerating
- **THEN** `--check` exits non-zero and names the stale entry

### Requirement: Third-party package declaration is validated
A `skill-package.json` SHALL declare name, version, skills directory, optional hooks and MCP servers, and a minimum contract version; the pack SHALL validate it with a JSON schema.

#### Scenario: Valid declaration
- **WHEN** a repo ships a conforming `skill-package.json`
- **THEN** `prometheus contract validate <path>` exits 0

#### Scenario: Contract version too new
- **WHEN** the declaration requires a contract version above the pack's
- **THEN** validation fails with the required and available versions

## Hooks surface

The hook-bundle seam is documented only. This change does not modify `hooks/hooks.json`, `hooks/codex-hooks.json`, or any hook script; extensions register bundles through the existing `run-hook --bundle` mechanism. C-03 is therefore not triggered; `npm run validate:codex` runs in the verify block to prove no plugin drift.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- Whether the control socket is renamed from `sovereign-sync.sock` in contract v2 once the Companion is the only listener.
