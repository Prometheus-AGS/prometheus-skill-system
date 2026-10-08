# change-ldd-11-entrypoint-spec-contract

Title: Require production entry points in KBD acceptance templates
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G14
Depends on: none
Reuse: cand-010
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-spec/SKILL.md
  - skills/process/kbd-process-orchestrator/references/native-backend.md
  - skills/process/kbd-process-orchestrator/references/spec-backend-interface.md
  - skills/process/kbd-process-orchestrator/references/templates/acceptance.template.md (new)

## Required behavior

1. Add a reusable acceptance template naming the production entry point, real collaborating components, process/filesystem/network/database boundary, observable result, isolation roots, negative control and exact final local gate. Require kbd-spec to apply it to both native and OpenSpec authored artifacts.

2. State complete-production-before-test-authoring/execution and one consolidated final boundary. Unit tests, structural assertions and mocked collaborators are supplemental only and cannot satisfy integration acceptance. Missing service/tool/approval returns BLOCKED; exit 2 never means pass.

3. Keep normal SpecBackend semantics and task IDs unchanged. Do not add mutation guards or modify generated managed skill copies. Change 12 owns regeneration and the full local artifact/installed-guide exercise.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior phase shipped a critical installer defect through helper-only checks. Put the required execution boundary in authoring guidance rather than relying on reviewers to infer it.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-11-entrypoint-spec-contract` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
