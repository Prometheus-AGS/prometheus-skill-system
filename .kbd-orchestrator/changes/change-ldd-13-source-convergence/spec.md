# change-ldd-13-source-convergence

Title: Reconcile outstanding source across full mini and dependency worktrees
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini+dependencies
Gaps: G15
scope:
  - phase:evidence/repository-inventory.json
  - phase:source-disposition.md
  - full:source changes identified in the disposition
  - mini:source changes identified in the disposition
  - dependencies:explicitly selected relevant source changes

## Requirements

1. Inventory every registered full/mini worktree, dirty/untracked source, local-only commit, open PR, archived branch/ref and gitlink; record source hashes and owner/activity. Inspect relevant dependency repositories and UAR team-consumer repairs, not unrelated UAR application feature work. Each item has include/already-merged/superseded/out-of-scope disposition with reasons; no silent omission.

2. Full #157 and #158 are already merged, verified at c80baa03887e2e110a9d825de5092d3e65568372 and ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61. Preserve that baseline. Mini #41 remains open as of inventory; integrate its source locally and retain user merge ownership. Older full worktrees have no unique commits and contain learning bookkeeping; preserve their data rather than claiming unique product work.

3. Reconcile surreal-memory local skill commit 704528b and uncommitted src/hook.rs, src/lib.rs, src/main.rs with current origin/main before cache changes. Preserve local author edits; capture a reviewable patch and stage explicit paths. Inspect UAR canonical task-acceptance/model catalog repairs only for pack contracts. Do not absorb an entire unrelated convergence branch.

4. Refresh every full/mini gitlink origin before the release freeze, including imported skill packs. Current deltas include openai-proxy ad32f5d and entity-management 071b9e5b. Latest means the final compatible, reviewed, remotely reachable source containing all in-scope work, not the largest tag string or a dirty working directory. Recheck at freeze; record newly arriving work for disposition.

5. Use existing clean worktrees or new worktrees from origin/main. Never reset original dirty checkouts or take over another live agent's worktree. A moving source requires coordination/provenance capture; do not cherry-pick a half-written patch.

## Binding constraints

Source work uses isolated worktrees under ../worktrees with declared base/provenance; protect deploy-main/deploy/main and other agents' changes. All tests, builds and reviewers wait for completed production and run only locally under change 12. Use scratch HOME/CODEX_HOME/CORTEX_DATA_DIR/plugin/learning roots; never test against live :23001. One Cargo/rustc process machine-wide; pgrep first. User merges every PR and explicitly approves versions.toml edits, tags and real Codex/Claude changes. No approval is inferred from planning. See the phase plan for ordering, exact task model assignments and final gates.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-13-source-convergence` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
