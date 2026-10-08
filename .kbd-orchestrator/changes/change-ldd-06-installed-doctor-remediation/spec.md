# change-ldd-06-installed-doctor-remediation

Title: Resolve Codex repair instructions from a verified installation
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G7
Depends on: change-ldd-02-codex-memory-control
Reuse: cand-007
scope:
  - tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs
  - docs/codex-plugin.md

## Required behavior

1. Doctor must resolve the configured plugin root and a verified installed generation using existing generation/receipt verification. Only suggest an existing helper inside that generation; validate containment and manifest ownership. Do not trust arbitrary current symlink targets or repo cwd.

2. Emit a safely quoted absolute bash helper command and explicit effective Codex-home context. If no verified helper exists, report installation remediation without an executable-looking broken relative command. Preserve the distinction between diagnostic check and owner-approved apply.

3. Compose with change 02 predicate and change 07 home resolution; no duplicate helper/parser implementation. Shared doctor/docs ownership is serialized in that order.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior-context requires installed-artifact evidence under original failure conditions; source-relative helpers cannot prove installed CLI repair.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-06-installed-doctor-remediation` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
