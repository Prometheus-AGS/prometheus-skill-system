# change-ldd-02-codex-memory-control

Title: Stop Codex startup consolidation through the feature gate
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G2
Depends on: change-ldd-01-hook-bytecode
Reuse: cand-002
scope:
  - shared/scripts/codex-memories-config.sh
  - scripts/install-system.js
  - scripts/install-skills-flat.sh
  - tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs
  - docs/codex-plugin.md
  - docs/guide/memory-tiers.md
  - CLAUDE.md

## Required behavior

1. Update the existing helper to set features.memories=false plus memories.generate_memories=false and memories.use_memories=false. Preserve unrelated TOML/comments and make repeated application idempotent. Validate parsing before replacement; invalid or unsupported syntax must leave the original intact and return an explicit failure. Retain collision-safe backup/restore behavior; no new TOML dependency is required.

2. --check is read-only and reports each setting and known v1/v2 summary presence separately. Preserve existing JSON fields and add explicit feature/use status. Treat malformed values as unhealthy. Apply mode archives only known memory_summary.md artifacts into version-distinguishable, collision-safe archives; preserve MEMORY.md, raw_memories.md and database state.

3. Wire the policy through actual full installer install dispatch, including absent/new config. Keep unrelated uninstall behavior separate. Doctor must not PASS solely on generation=false or summary absence: its reported predicate must establish all three settings and relevant summary disposition. A static config check must not claim running sessions have stopped.

4. Document the exact 0.158.0 source mechanism and effective-profile/CLI-override limitations. New startup is blocked; in-flight work may retain old config. A machine application/restart is a later owner-approved operation, never part of repository implementation or a test. Agent/profile examples that override these settings must be audited within the declared docs/source scope and added to scope before editing.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior-context hardening lesson: sourced-helper checks let the uninstall-only wiring defect ship. Analysis source and scratch-copied aggregate metadata corroborate consolidation; they are not controlled acceptance evidence.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-02-codex-memory-control` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
