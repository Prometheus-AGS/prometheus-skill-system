# change-ldd-10-mini-executable-modes

Title: Preserve executable intent in mini distribution packages
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: mini
Gaps: G12, G13
Depends on: none
Reuse: cand-006
scope:
  - lib/distribution/package-builder.mjs
  - scripts/generate-skill-system-distribution.mjs
  - docs/distribution.md (existing equivalent or new focused document)

## Required behavior

1. Port the full generator mode-preserving pattern to mini copyTree and any byte-write path that materializes owned executable scripts. Preserve source executable intent and directory traversal permissions; do not introduce world-writable output or follow links outside declared package ownership.

2. Include executable intent in --check drift comparisons on platforms supporting it. Preserve Windows filesystem capability semantics; do not claim POSIX mode verification on a platform that cannot express it.

3. Regenerate mini outputs only through the named final reconciliation owner (change 12). At that boundary produce a new version-bound integration baseline rather than rewriting historical mini-baseline-failures.txt as if it were current. Historical 19 names versus reported 11 failures remains an unverified legacy discrepancy until mapped to evidence.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Analysis observed 755 source and 644 packaged helper. Prior phase depended on checkout layout; derive acceptance from the actual generated package and record source identity.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-10-mini-executable-modes` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
