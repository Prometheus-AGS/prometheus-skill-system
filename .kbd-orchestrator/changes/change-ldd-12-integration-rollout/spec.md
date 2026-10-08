# change-ldd-12-integration-rollout

Title: Reconcile generated outputs certify locally and hand off approved rollout
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini+surreal-memory
Gaps: G1, G2, G3, G4, G5, G6, G7, G8, G9, G10, G11, G12, G13, G14
Depends on: change-ldd-01-hook-bytecode, change-ldd-02-codex-memory-control, change-ldd-03-published-memory-pins, change-ldd-04-cache-identity-accounting, change-ldd-05-generated-output-ownership, change-ldd-06-installed-doctor-remediation, change-ldd-07-custom-codex-home, change-ldd-08-bounded-cortex-mirror, change-ldd-09-partition-footer, change-ldd-10-mini-executable-modes, change-ldd-11-entrypoint-spec-contract
Reuse: cand-010
scope:
  - full:generator-owned output paths from scripts/generated-paths.mjs and harness-adapter generator
  - mini:generator-owned dist/ and commands outputs
  - full:scripts/tests/test-learning-deploy-and-debt.sh (new integration coordinator)
  - full:scripts/tests/test-hook-bytecode-integration.sh (new)
  - full:scripts/tests/test-codex-memory-integration.sh (new)
  - full:scripts/tests/test-codex-home-integration.sh (new)
  - full:shared/scripts/tests/test-cortex-mirror.sh
  - full:scripts/tests/test-memory-partition.sh
  - full:scripts/tests/test-rebase-regenerate.sh
  - mini:scripts/tests/learning-deploy-integration.test.mjs (new)
  - surreal-memory:tests/query_cache_production.rs (new)
  - phase:evidence/**
  - phase:execution.md
  - phase:review/**

## Required behavior

1. This is the sole C-01 generated-output reconciliation and final integration owner. All production changes 01-11 and 13-18 must have complete production implementations, including approved pin edits, before test authoring or execution. Regenerate full and mini owned outputs from their actual candidate sources before any tests. No preceding change may claim distribution certification.

2. After production completion, author/repair only necessary full integration scenarios. Classify protected files first; changes to protected scenarios require the SSH-signed canonical approval manifest. Never weaken a protected assertion to get green. Add a coordinator accepting explicit --full, --mini, --surreal and --evidence absolute paths; defaulting to live paths is forbidden.

3. Run one consolidated local batch. Check pgrep -x cargo and pgrep -x rustc immediately before each Cargo invocation; if busy, wait or report BLOCKED. Use per-worktree default targets and sccache, never competing builds or cargo clean. Select integration targets only; avoid broad package scripts that transitively run unit tests.

4. Prove generator idempotence by two full materializations and byte/mode-identical owned outputs, then production --check and applicable local distribution/harness/release checks. Run protected integrity from a committed candidate Git state: a private scratch commit of the completed candidate tree may establish this without pushing. Bind receipts to that tree and recheck identity after final commits.

5. Run the package installation/hook/Codex/Cortex/partition/rebase/mini/service scenarios specified by siblings. Produce current mini integration baseline with commit/tool/command/result metadata and explicit mapping to historical failures (unmapped remains unverified). No broad legacy suite is an acceptance substitute.

6. Run independent cumulative review across every source and spec in the phase with a verified judge different from the producer; reviewers remain dormant until this boundary. Record findings honestly. Batch any production fixes and rerun only the affected final integration gate; changed artifacts invalidate earlier evidence for those artifacts.

7. After local gates/review pass, create reviewable PRs with exact local commands/results. User merges every PR. Wait for user merge before deleting any worktree; audit marketplace references and live processes before cleanup and preserve deploy-main/deploy/main unconditionally.

8. Prepare an exact machine refresh/configuration proposal from the merged source identity. Obtain explicit chat approval before touching real ~/.codex, ~/.claude or activating a machine refresh; this includes memory configuration, known-summary archival and session restart/drain instructions. Never assume a prior-phase approval covers this rollout.

9. For approved deployment, follow versioned delivery-cadence and record read-only installed identity/health. Load and recall tests remain on independently started scratch services at ephemeral/non-23001 endpoints, even after deployment. Report p50/p95, attempts/cache reuse, timeout counts and relevance under idle and 8-writer load for the final frozen release against cache-disabled and recorded prior-release controls; no live load test. All included cache debt is identified by its final frozen source; historical v1.10.1 evidence is labeled as historical.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Recalled lessons require real production entry points, real Cortex lifetime, scratch-only tests and honest BLOCKED results. Previous marketplace cleanup broke refresh; deployment references and live processes must be checked before removal.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Release-closure scope amendment

The owner expanded this phase to finish full/mini as stable products. This final boundary also covers changes 13-18, both Docusaurus production builds and locally served page/route/example checks, all content inventory dispositions, actual team/handoff/model recipes, Companion two-process and absent-service behavior, current mini production baseline and fresh remote clone/installability of every final pin. Do not run aggregate scripts that implicitly add unit suites as acceptance.

Companion check-in/push is explicitly authorized by the user; after local source/security/integration gates, stage only manifest-listed source, create the proposed private remote if needed, push and verify the remote commit/fresh clone. User-owned merges, versions.toml changes and tags retain their explicit boundaries. Publish site content only from merged locally certified identities. Do not merely prepare a repository without pushing it and call publication complete.

Reconcile the four historical control-plane changes through their existing canonical IDs only after the final evidence; preserve and explain missing-receipt provenance. Finish all in-scope release blockers, record maintenance readiness and remove merged worktrees only after checking live processes and marketplace references. Never alter deploy-main/deploy/main.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-12-integration-rollout` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.

### Owner-approved release sequencing — 2026-10-05
Direct owner chat: "I approve the release proposal". release-approval.json records the exact approved proposal hashes, eleven Companion protected pins, stated conditional tags/plugin bumps and the named Immutable Implementation-First and Integration-Only Policy sequencing exception in that proposal. Preserve all other policy.03/task3 may apply the approved currently-resolvable metadata batch while03/task2 retains final dependency/remote identity freeze. After every functional and known packaging source edit is complete,12/task1 reconciles generation and authors scenarios, then12/task2 runs the initial consolidated real-candidate gate. Dependency certification/owner merge and exact mini SHA approval precede final parent metadata and the final fresh-clone gate.03 tasks are not falsely completed from an initial approval/batch. Tags, maintenance readiness and historical closure remain after required gates/owner merges. Approval resolves the preceding goal blocker and resets the blocked audit; no invented final SHA or validation evidence.
