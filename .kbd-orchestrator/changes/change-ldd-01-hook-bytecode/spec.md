# change-ldd-01-hook-bytecode

Title: Prevent Python bytecode in immutable hook generations
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G1
Depends on: none
Reuse: cand-001
scope:
  - scripts/generate-harness-adapters.js
  - shared/scripts/sessionstart-learning.sh
  - shared/scripts/subagentstart-learning.sh
  - shared/scripts/subagentstop-learning.sh
  - shared/scripts/lib/learning_write.py
  - docs/codex-plugin.md

## Required behavior

1. Emit PYTHONDONTWRITEBYTECODE=1 in the generated dispatcher before any hook command. Both compiled and shell runtime paths already converge there; do not duplicate dispatch logic or change the Rust runtime unnecessarily.

2. Suppress bytecode for directly invoked learning wrappers and learning_write.py before local imports. Python descendants, including the detached Cortex feeder and enqueue helper, inherit suppression even when the caller sets PYTHONDONTWRITEBYTECODE=0. Preserve command results and hook payloads.

3. Keep payload verification strict. Do not add __pycache__ exclusions, patch installed caches, delete existing live bytecode or write generated files by hand. Record C-01 generated-output ownership in change 12.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior-context: helper-only checks missed the actual installer branch; test packaged production dispatch. Analysis pinned Python inheritance semantics; -B on one parent is insufficient for new interpreters.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-01-hook-bytecode` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
