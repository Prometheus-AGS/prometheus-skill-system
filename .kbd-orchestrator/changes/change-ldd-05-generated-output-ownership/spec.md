# change-ldd-05-generated-output-ownership

Title: Replace the self-confirming generated-path assertion
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G6
Depends on: none
Reuse: cand-009
scope:
  - scripts/generate-skill-system-distribution.js
  - scripts/generated-paths.mjs
  - scripts/rebase-regenerate.sh
  - docs/generated-output-ownership.md

## Required behavior

1. Remove the claim that asking the same generator for its root list independently verifies that list. Maintain one declaration source, but compare declared generated ownership against actual materialized output/operations and downstream rebase classification.

2. The check must detect an output written outside declared ownership and an owned required output missing from materialization. It must distinguish legitimate uninitialized gitlinks from generated stray files without mutating unrelated submodules.

3. Preserve --list-outputs and --check contracts for consumers, deterministic path ordering and source/generated separation. Avoid a second manually synchronized root list.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Recalled rebase gate misclassified uninitialized gitlinks. Do not equate one developer checkout layout with the contract.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-05-generated-output-ownership` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
