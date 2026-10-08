# change-ldd-07-custom-codex-home

Title: Honor one effective Codex home throughout installation lifecycle
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G8
Depends on: change-ldd-02-codex-memory-control, change-ldd-06-installed-doctor-remediation
Reuse: cand-008
scope:
  - scripts/install-system.js
  - scripts/lib/store-paths.js
  - scripts/lib/plugin-source-topology.js
  - scripts/codex-provision-mcp-env.sh
  - scripts/install-plugin-generation.js
  - scripts/install-skills-flat.sh
  - scripts/configure-mcp-all-tools.sh
  - shared/scripts/codex-memories-config.sh
  - tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs
  - docs/codex-plugin.md
  - CLAUDE.md

## Required behavior

1. Choose nonempty inherited CODEX_HOME first; otherwise derive .codex from the selected --home/HOME. Treat empty CODEX_HOME as unset, normalize the selected root consistently, and document that --home changes the fallback and other platforms but does not override an explicit CODEX_HOME. No new option is needed solely to duplicate this choice.

2. Propagate that effective root into Codex plugin inspection, memory helper, MCP configuration and actual skill projection. Use it for verification, uninstall, rollback and generation-reference/prune accounting as well. Do not merely remove the two install-system environment overrides while leaving targets in HOME/.codex.

3. Preserve copy-based Codex skill delivery and existing collision handling. Uninstall removes only managed entries at the selected Codex root, never unrelated files. Keep receipts/ownership at the correct root. No configuration or symlink is written under a real user home during development.

4. Update diagnostics/documentation in a serial batch after changes 02 and 06. Audit additional existing MCP launcher paths before editing; amend the explicit scope if an additional owner file is required.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior-context installer bug escaped helper tests. Tests that accidentally inherit real CODEX_HOME already changed live hooks in an earlier session; always set all scratch roots explicitly.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-07-custom-codex-home` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.

## Execute scope amendment — shared resolver and actual collaborators

Lead authorized 2026-10-05: scripts/lib/store-paths.js is the existing shared path module used by the generation installer and compiled-in doctor verifier; scripts/lib/plugin-source-topology.js inspects Codex configuration for the existing installed-source check; scripts/codex-provision-mcp-env.sh is the direct MCP environment launcher. Include all three in the serialized change07 owner scope to implement required behavior 4 without duplicate JS resolution or inconsistent subprocess roots. Unrelated kbd-goal-codex-setup.sh and register-slash-commands.sh are outside the selected installer lifecycle and remain untouched; no broader custom-home capability is inferred. Test/generation/publication/protected approval boundaries are unchanged.
