# change-ldd-08-bounded-cortex-mirror

Title: Bound optional Cortex feeders without weakening durable lesson writes
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full
Gaps: G9, G10
Depends on: change-ldd-01-hook-bytecode
Reuse: cand-005
scope:
  - shared/scripts/lib/learning_write.py
  - shared/scripts/lib/cortex_feed_slots.py (new if needed)
  - docs/guide/memory-tiers.md

## Required behavior

1. Use a standard-library cross-process admission pool outside plugin generations, shared by writers for the same resolved user state root. Default maximum is 4 live feeder/server pairs; PROMETHEUS_CORTEX_MAX_FEEDERS accepts nonnegative integers, 0 disables the optional mirror, invalid values fall back to 4 with a nonsecret diagnostic in writer status.

2. Acquire capacity before spawning a detached feeder, transfer ownership safely to the child, retain it for the real server lifetime and release on successful completion, error, timeout and failed spawn. Crash recovery must not reclaim a slot still held by a live process, including PID reuse; use OS-backed ownership where possible. Do not treat age alone as evidence that a live owner is dead.

3. At saturation skip the optional Cortex mirror immediately; do not wait, retry in an unbounded background queue or drop the durable surreal-memory outbox/learning-log write. Preserve existing result fields and add a reason distinguishing accepted, disabled, absent, saturated and failed mirror attempts; acceptance is not a delivery receipt. Normal hooks remain silent.

4. Keep Cortex stdin open until the matching MCP reply or timeout. Teardown/reaping must not leave a server running after its capacity lease is released. Preserve bytecode suppression from change 01. No new dependency, envelope schema change or mandatory Cortex availability is intended.

5. Change 12 owns test repair: eliminate assertions or reads of real ~/.cortex/memory.db and point the actual Cortex server at scratch CORTEX_DATA_DIR. This work item specifies that behavior now; no test is authored or edited before all production changes finish.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Prior real-Cortex run showed process.exit on closed stdin lost memories despite stub success. Bound the entire real server lifetime; preserve durable surreal-memory independently of this optional mirror.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-08-bounded-cortex-mirror` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
