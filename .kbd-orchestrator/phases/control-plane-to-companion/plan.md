# PLAN: control-plane-to-companion

Project: prometheus-skill-pack (source) and prometheus-companion (destination)
Date: 2026-09-02
OpenSpec available: YES (repo-local `openspec/`), but this phase's changes are **native-kbd** (`.kbd-orchestrator/changes/change-cpc-*`), authored at spec because the adversarial packet builder and `kbd-apply` read that store; Companion-side changes are mirrored into the Companion's own `openspec/changes/` per the cross-repo rule (change-cpc-003).
Changes to implement: 13
Governing decision: D-02 option D (operator, 2026-09-02). The pack is open source and complete on its own; the Companion is the paid control and extension framework; the pack never depends on the Companion; capability is discovered at runtime, never assumed.

## CHANGE LIST (ordered)

1. `change-cpc-001-integration-contract`: Publish the open integration contract (G7): control-endpoint discovery, hook bundle extension points, generated service manifest, connected-skill-package declaration, `prometheus contract show|validate`.
   - Scope: docs | generator script | CLI subcommand | JSON schema | site docs (pack)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Library: none (pattern: existing CLI transport chain, existing LaunchAgent/systemd templates as manifest source)
   - Details: Write contract v1.0, add `scripts/generate-service-manifest.mjs` with idempotent output and `--check`, add `shared/schemas/skill-package.schema.json`, add the CLI subcommand with a process-level integration test that proves silence when no endpoint exists. From this change on, plists and units are C-01 generator inputs. The hook-bundle seam is **documented only**: this change does not modify `hooks/hooks.json` or any hook script (extensions register bundles through the existing `run-hook --bundle` mechanism), so C-03 is not triggered; `npm run validate:codex` still runs in its verify block to prove no plugin drift.

2. `change-cpc-002-skill-ffi-kbd-mobile-split`: Remove skill-ffi's dependency on kbd-mobile entirely and move the wire-compat test into kbd-mobile.
   - Scope: two pack crates' manifests, FFI API, generated bridge, one test move (pack)
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM
   - Library: none
   - Details: Delete the path dependency and the KBD-sync API surface, reserve an empty `kbd-sync` feature, regenerate `frb_generated.rs` at the exact 2.12.0 pin, and prove with `cargo metadata --all-features` that no relocated crate remains in skill-ffi's graph. Must land before change 3's pack commit is pinned.

3. `change-cpc-003-companion-workspace`: Create the Companion root workspace, the `prometheus-substrate` crate with desktop and headless presets, git-rev consumption of pack crates, staged pins, spec Appendix A rewrite, Companion openspec init, and Companion runtime reconciliation.
   - Scope: Companion Cargo workspace | substrate crate scaffold | docs | openspec | KBD runtime reconciliation (Companion)
   - Depends on: `change-cpc-002-skill-ffi-kbd-mobile-split`
   - Recommended agent: Claude Code, then **Manual** for the `versions.toml` pins edit
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Library: cand-016 (git deps pinned by rev, Companion-to-pack only), cand-009 (graph-explorer single-workspace layout), cand-013 (keyring =3.6.3 inherited pin)
   - Details: Workspace `members = ["crates/prometheus-substrate", "src-tauri"]`; pack crates by `git` + `rev` at the commit containing change 2; `docs/decisions/versions-pins-proposal.md` stages every pin (iroh 1.0.3, iroh-gossip 0.101, both iroh address-lookup crates 0.5.0, loro 1.13, keyring =3.6.3, dirs_next, axum 0.8, rmcp 1.8, redb 2, tauri-plugin-stronghold 2.3.2). **BLOCKED until the operator edits `versions.toml`** (agents are denied that file). Records the scaffold's audit-gate listing at 773aa5e as evidence and reconciles the Companion runtime's `docs-review-for-build-assessment` assess stage before any mirrored change lands.

4. `change-cpc-004-relocate-sovereign-sync`: Move sovereign-sync, sovereign-client, kbd-mobile, and the iroh-docs adapter into the Companion workspace under Companion governance.
   - Scope: four Companion crates | manifests | library-code unwrap removal | ported integration tests (Companion)
   - Depends on: `change-cpc-003-companion-workspace`
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Library: cand-001 (sovereign-sync 1.8.0, adapt), cand-016
   - Details: Copy with source-commit READMEs, replace path deps with git-rev pack deps, exact pins, `clippy::unwrap_used` on `--lib`, keep the socket REST/MCP/SSE surface and `--token-file` TCP path unchanged, port 22 + 4 + 1 tests, and prove a signed command from the installed pack CLI over the socket to the Companion-built daemon. The pack tree is untouched here; deletion is change 9.

5. `change-cpc-005-node-hosting`: Host the node in-process from the Tauri setup hook, add the headless entry point, and prove state-location and credential-store continuity between the daemon binary and the Companion binary.
   - Scope: substrate node/paths/identity modules | Tauri setup | headless binary | spec §15/§4.1 | two-process continuity test (Companion)
   - Depends on: `change-cpc-004-relocate-sovereign-sync`
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Library: cand-007 (clawdesk embedded-gateway pattern, reference), cand-018 (dirs_next + PROMETHEUS_DATA_DIR), cand-013 (keyring incumbent), cand-019 (headless preset)
   - Details: Startup router first, `tauri::async_runtime::spawn`, `app.manage` state; headless binary with no window or vault; `PROMETHEUS_DEVICE_KEY_FILE` fail-closed path with the documented message; a continuity test that launches the pack daemon and the headless Companion as separate processes on one data root and asserts revision, frontier, the signed audit export (`signed_audit_jsonl`), the receipt set, and the run history before and after an append, so the goal's "project.loro, receipts, run history migrate intact" has an explicit assertion.

6. `change-cpc-006-discovery-and-pairing`: Add mDNS and Mainline DHT address lookup (DHT off by default), keep pairing as admission, correct the spec's Noise attribution.
   - Scope: sovereign-sync p2p/config | Companion pairing QR | spec §14 and §25.2 | two-endpoint discovery test (Companion)
   - Depends on: `change-cpc-004-relocate-sovereign-sync`
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Library: cand-003 (iroh-mainline-address-lookup 0.5.0, adopt), cand-004 (iroh-mdns-address-lookup 0.5.0, adopt), cand-006 (iroh-gossip, adopt), cand-005 rejected
   - Details: Task 1 asserts the operator's `versions.toml` already carries both lookup crates before any Cargo edit. Endpoint builder: N0 preset + mDNS + Mainline (opt-in) + bootstrap list. Discovery test starts both endpoints and asserts `NeighborUp` with an empty bootstrap list and unauthorized-peer rejection. **BLOCKED until the pins land.**

7. `change-cpc-007-supervisor-and-health`: Adopt and supervise the pack's services from the Companion through the service manifest, with one five-state health aggregator, the tray, and a rendered launchd plist for the Companion itself.
   - Scope: substrate health/supervisor/services modules | REST route | tray | Companion plist + installer | spec §16.4/§25.2 | supervision test (Companion)
   - Depends on: `change-cpc-001-integration-contract`, `change-cpc-005-node-hosting`
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Library: cand-010 (HMA health-aggregator template + launchagent-supervisor skill, adopt), cand-011 rejected
   - Details: Services stay where they run; the supervisor adopts them as `ExternalSidecar` via `launchctl kickstart` / `systemctl --user`, backoff 1s→60s with a named failure per retry branch (Companion AGENTS.md §0.2), 10 failures → disabled. Test kills an adopted service and observes Down → Healthy through `GET /api/v1/services`.

8. `change-cpc-008-sync-skills-plugin`: Ship sync-status, sync-peers, sync-push as the Companion's skill plugin with a `skill-package.json`, and let the Companion register its MCP stdio binary.
   - Scope: Companion skills dir | skill-package.json | installer step (Companion)
   - Depends on: `change-cpc-001-integration-contract`, `change-cpc-004-relocate-sovereign-sync`
   - Recommended agent: Codex
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM
   - Library: none (cand-020 stdio shim rejected; MCP moves with sovereign-sync)
   - Details: First worked example of third-party extension. Validated with the pack's `prometheus contract validate`; install proves `/sync-status` reaches the Companion socket and that the Companion installer's own registration output (`scripts/install-skill-package.sh --dry-run` and the harness config it writes, `~/.claude/mcp-servers.json` on Claude Code) names the Companion MCP binary. The installer owns that file's Companion entry; the pack's installers stop writing sovereign-sync into it in change 10.

9. `change-cpc-012-pack-sync-skills-removal`: Remove the sync-* skills and every plugin-surface reference from the pack (C-01, C-03).
   - Scope: skills | skills index | Claude and Codex plugin surfaces | Codex catalog | docs/codex-plugin.md | CLAUDE.md (pack)
   - Depends on: `change-cpc-008-sync-skills-plugin`
   - Recommended agent: Codex
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM
   - Library: none
   - Details: `npm run generate:skills-index`, `npm run build:codex && npm run validate:codex` in the same change; greps prove no reference remains.

10. `change-cpc-009-pack-removal`: Remove the relocated crates, the sharing install path, the daemon wiring, the four sovereign-sync OpenSpec main specs, and the prometheus-exec estate reference; repoint probes and the CLI to optional contract discovery; regenerate the service manifest.
    - Scope: substrate deletions | four installers | plists and units | probes | CLI doctor/setup/transport | OpenSpec main specs | site docs, codex-plugin.md, CLAUDE.md | services manifest (pack)
    - Depends on: `change-cpc-004-relocate-sovereign-sync`, `change-cpc-012-pack-sync-skills-removal`
    - Recommended agent: Claude Code
    - Est. complexity: L
    - Complexity score: High
    - Model class: frontier
    - Customer value: HIGH
    - Library: none
    - Details: Re-derive the footprint at write time. The whole-repo footprint command must print nothing after excluding archives, memory, knowledge and refiner stores, tolerating the three contract-v1 identifiers, and allowing the historical docs. `prometheus doctor` gets a test case asserting silence with the Companion absent. Named deliverable: `docs/decisions/sovereign-sync-relocated-to-companion.md`, the retirement record for the four OpenSpec main specs (sovereign-sync-daemon-health, sovereign-sync-ci, iroh-docs-adapter, mcp-client-pool), with a verify step that the record exists and the four spec directories do not. Carries a round-2 review CRITICAL verbatim (footprint criterion).

11. `change-cpc-010-kbd-state-migration`: Adopt-or-archive the seven unmarked phase projections with receipts and prove the frontier and revision are unchanged.
    - Scope: kbd-runtime migration | CLI `migrate --projections` | seven progress.json files | archives (pack)
    - Depends on: NONE
    - Recommended agent: Codex
    - Est. complexity: M
    - Complexity score: Medium
    - Model class: medium
    - Customer value: MEDIUM
    - Library: cand-002 (kbd-runtime, adopt; stays in the pack)
    - Details: Dry-run first; the real run records receipts; a typed stage transition afterwards must emit no `refusing to overwrite` line. Independent of the Companion; can run any time in Round 1.

12. `change-cpc-011-integration-evidence`: Two-process G6 harness on the Companion: headless binary plus a second instance, mDNS lookup, gossip join, a signed KBD command via the pack CLI observed on the other node, and the services surface.
    - Scope: Companion two-process test | Companion docs (Companion)
    - Depends on: `change-cpc-005-node-hosting`, `change-cpc-006-discovery-and-pairing`, `change-cpc-007-supervisor-and-health`, `change-cpc-009-pack-removal`
    - Recommended agent: Claude Code
    - Est. complexity: L
    - Complexity score: High
    - Model class: frontier
    - Customer value: HIGH
    - Library: cand-019 (headless preset)
    - Details: Real processes, real network boundary, run locally; `domain_sync.rs` stays beneath it as the library regression. Depends on change 10 (`change-cpc-009`) because the harness drives the pack CLI through its repointed production transport, not an explicit endpoint override. No hosted CI is cited.

13. `change-cpc-013-pack-certification-without-companion`: Certify the pack fully with the Companion absent and update pack installation, updating, and release docs.
    - Scope: certification script | pack docs | CHANGELOG (pack)
    - Depends on: `change-cpc-009-pack-removal`, `change-cpc-010-kbd-state-migration`
    - Recommended agent: Codex
    - Est. complexity: S
    - Complexity score: Low
    - Model class: small
    - Customer value: HIGH
    - Library: none
    - Details: Install profile without the Companion, every kbd-* path the local runtime covers, `prometheus doctor --json`, OpenSpec, harness, and receipt-identity certification; `certification.log` contains no Companion or sovereign-sync mention.

## EXECUTION ROUND ORDER

Round 1 (parallel, pack): `change-cpc-001-integration-contract`, `change-cpc-002-skill-ffi-kbd-mobile-split`, `change-cpc-010-kbd-state-migration`
Round 2 (Companion, operator-gated): `change-cpc-003-companion-workspace` — waits for the operator's `versions.toml` pins edit
Round 3 (Companion): `change-cpc-004-relocate-sovereign-sync`
Round 4 (parallel, Companion): `change-cpc-005-node-hosting`, `change-cpc-006-discovery-and-pairing`
Round 5 (parallel, Companion): `change-cpc-007-supervisor-and-health`, `change-cpc-008-sync-skills-plugin`
Round 6 (pack): `change-cpc-012-pack-sync-skills-removal`
Round 7 (pack): `change-cpc-009-pack-removal`
Round 8 (parallel): `change-cpc-011-integration-evidence` (Companion; also depends on `change-cpc-009`), `change-cpc-013-pack-certification-without-companion` (pack)

Ordering rationale: the contract (1) precedes everything that consumes it (7, 8) and the pack-side dependency cut (2) precedes the Companion's pinned pack commit (3). The Companion node must exist and host (4, 5) before discovery (6), supervision (7), and the skill plugin (8) can be proven. Pack removal (12, 9) comes only after the Companion ships the replacement, so at no point is the sharing feature absent from both repositories. Evidence (11, 13) is last because it certifies the combined result. "Parallel" means independent changes; on this machine only one Cargo build may run at a time, so parallel Rust changes are executed sequentially in one worktree and never dispatched to competing agents.

## CROSS-REPO EXECUTION

The skill-pack run `sovereign-sync-service-reliability-20260829` owns this phase. Companion changes (3, 4, 5, 6, 7, 8, 11) execute in `/Users/gqadonis/Projects/prometheus/prometheus-companion`, are mirrored into that repo's `openspec/changes/<change-id>/` (`proposal.md` + `tasks.md`), and record their evidence in the pack change's `verification.md`. Change 3 reconciles the Companion runtime first. Pack changes (1, 2, 9, 10, 12, 13) execute here. No change edits both repositories.

## COMMANDS TO RUN

Change directories already exist from `/kbd-spec` (native-kbd). Register each change with the runtime in plan order, then apply the first:

```text
prometheus kbd --path . change register --command-id change-register:cpc-001 --phase control-plane-to-companion --id change-cpc-001-integration-contract --title "Publish the open integration contract" --sequence 1
prometheus kbd --path . change register --command-id change-register:cpc-002 --phase control-plane-to-companion --id change-cpc-002-skill-ffi-kbd-mobile-split --title "Remove skill-ffi dependency on kbd-mobile" --sequence 2
prometheus kbd --path . change register --command-id change-register:cpc-003 --phase control-plane-to-companion --id change-cpc-003-companion-workspace --title "Companion workspace and substrate crate" --sequence 3
prometheus kbd --path . change register --command-id change-register:cpc-004 --phase control-plane-to-companion --id change-cpc-004-relocate-sovereign-sync --title "Relocate sovereign-sync crates" --sequence 4
prometheus kbd --path . change register --command-id change-register:cpc-005 --phase control-plane-to-companion --id change-cpc-005-node-hosting --title "Host the node in-process with headless preset" --sequence 5
prometheus kbd --path . change register --command-id change-register:cpc-006 --phase control-plane-to-companion --id change-cpc-006-discovery-and-pairing --title "mDNS and Mainline lookup with pairing" --sequence 6
prometheus kbd --path . change register --command-id change-register:cpc-007 --phase control-plane-to-companion --id change-cpc-007-supervisor-and-health --title "Supervisor and health aggregator" --sequence 7
prometheus kbd --path . change register --command-id change-register:cpc-008 --phase control-plane-to-companion --id change-cpc-008-sync-skills-plugin --title "Companion sync skills plugin" --sequence 8
prometheus kbd --path . change register --command-id change-register:cpc-012 --phase control-plane-to-companion --id change-cpc-012-pack-sync-skills-removal --title "Remove sync skills from the pack" --sequence 9
prometheus kbd --path . change register --command-id change-register:cpc-009 --phase control-plane-to-companion --id change-cpc-009-pack-removal --title "Remove relocated crates and daemon wiring" --sequence 10
prometheus kbd --path . change register --command-id change-register:cpc-010 --phase control-plane-to-companion --id change-cpc-010-kbd-state-migration --title "Migrate unmarked projections" --sequence 11
prometheus kbd --path . change register --command-id change-register:cpc-011 --phase control-plane-to-companion --id change-cpc-011-integration-evidence --title "Two-process integration evidence" --sequence 12
prometheus kbd --path . change register --command-id change-register:cpc-013 --phase control-plane-to-companion --id change-cpc-013-pack-certification-without-companion --title "Certify the pack without the Companion" --sequence 13
/kbd-apply change-cpc-001-integration-contract
```

## SCOPE CUTS AND TRADE-OFFS

- The `prometheus` CLI, kbd-runtime, and every functional service stay in the pack (D-02). Nothing in this plan relocates function.
- kbd-mobile moves and skill-ffi loses its KBD-sync surface in the pack; mobile KBD sync becomes a Companion capability. This is a deliberate reduction of the open-source pack's mobile surface.
- The three sync-* skills leave the pack. Users without the Companion lose sharing; that is the product boundary, not a regression.
- The MCP stdio server moves with sovereign-sync; the pack ships no shim (cand-020 rejected). Harness registrations on user machines are re-pointed by the Companion installer, which is outside this phase's control.
- Mainline DHT publication is off by default; Goal 3's "without hand-maintained peer lists" is met by mDNS on a LAN and by DHT only where the operator opts in.
- The Companion spec is amended in five places (§4.1, §14, §15, §16.4/§25.2, Appendix A) rather than rewritten; a Companion UI phase owns dashboard and tray polish beyond the health state.
- Timer-service disposition (learning-worker, codex-skills-sync, hooks-logrotate, prometheus-nudge, pk-lint, mem0-compress), short-code pairing, and licensing of relocated code are operator decisions deferred to execution of changes 7, 6, and 4 respectively; each change names its question.
- Effort: eight rounds with five frontier-class changes (4, 5, 7, 9, 11); the evidence changes are 11 and 13. The Companion starts from 14 lines of Rust; the first shippable Companion build is at the end of Round 5, not Round 2.

## ADVERSARIAL REVIEW

Cross-model judge k3 via rest-gateway (`cross_model_check: verified-distinct`), one round: verdict PASS with 5 WARNINGs and 2 SUGGESTIONs, all folded in above (mcp-servers.json ownership in change 8, the retirement decision record in change 10, receipt and run-history continuity in change 5, change 12's dependency on change 10, the hooks-surface statement in change 1, and the two consistency fixes). Findings: `review/plan/findings.json`. Sycophancy self-check score 0.0 (`sycophancy/plan-*.json`).

## OPERATOR GATES

1. `versions.toml` pins in the Companion (blocks change 3, and any change that adds a crate).
2. Licensing of the relocated crates in the Companion (recorded, not decided, in change 4).
3. Timer-service disposition (change 7) and short-code pairing v1 (change 6).

## PLAN COMPLETE
