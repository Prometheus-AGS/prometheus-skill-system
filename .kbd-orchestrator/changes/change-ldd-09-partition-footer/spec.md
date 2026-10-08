# change-ldd-09-partition-footer

Title: Make partition footer budgeting exact and idempotent
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G11
Depends on: none
Reuse: cand-010
scope:
  - scripts/memory-index-partition.py
  - docs/guide/memory-tiers.md

## Required behavior

1. Calculate kept/moved entries against the actual emitted footer byte length and cumulative moved count, including digit-width transitions. Do not reserve unrelated structure/old-footer entries as if all will be moved.

2. Preserve ranking, visibility routing, stable tie order, cumulative counts and UTF-8 byte limits. Reapplying a successful partition with the same limit/phase must preserve the output bytes and produce no duplicate queued lesson. Report structure-only overflow explicitly rather than silently exceeding the limit.

3. Keep dry-run read-only. Apply mode operates only on explicit input/output and existing durable routing conventions; do not broaden file traversal or touch live memory indexes.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior reflection reported footer decimal-boundary debt without fresh reproduction; Spec requires production CLI evidence and durable-write idempotence.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-09-partition-footer` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
