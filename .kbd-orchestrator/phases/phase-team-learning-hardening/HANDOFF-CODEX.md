# Handoff to Codex (2026-10-05): after phase-team-learning-hardening

Claude Code handed this work to Codex on 2026-10-05 because Claude's weekly limit is near. This file is self-contained. Read it first, then `AGENTS.md` and `CLAUDE.md` in the skill pack. The skill pack's `CLAUDE.md` is the canonical rules source for the whole Prometheus stack, and its rules apply to Codex.

## 1. Where things stand

**Phase `phase-team-learning-hardening` is complete:** 9 of 9 changes and 24 of 24 tasks are merged, verified and archived. Reflection is done (`reflection.md` in this directory).

| Repository | Path | Branch state | Notes |
|---|---|---|---|
| prometheus-skill-system (full pack) | `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack` | `main` only (+ local `deploy/main`, see below) | Main checkout on `main`. KBD state lives in its `.kbd-orchestrator/`. |
| prometheus-skills-mini | `/Users/gqadonis/Projects/prometheus/prometheus-skills-mini` | `main` only | In parity for this phase (mini #39 and #40). |
| surreal-memory-server | `/Users/gqadonis/Projects/prometheus/surreal-memory-server` | `main` + tag `v1.10.1` | Query-embedding cache released. |
| prometheus-knowledge-rs (pk) | `tools/prometheus-knowledge` submodule | pinned v1.11.0 | No change this phase. |

- **The one deliberate exception to "main only":** `/Users/gqadonis/Projects/prometheus/worktrees/deploy-main`, on a **local-only** branch `deploy/main` that tracks `origin/main` and is never pushed.
  - It is the install source. `~/.claude/settings.json`, `~/.claude/plugins/known_marketplaces.json` and `~/.codex/config.toml` (`[marketplaces.prometheus-skill-pack]`) all point at it.
  - `scripts/update-skill-pack.sh` runs `git pull --ff-only` there, so it needs a tracking branch.
  - The main checkout cannot be the source, because the KBD runtime writes tracked files in it.
  - **Do not delete this worktree or branch.**
- **Left in place because they belonged to live sessions at handoff:**
  - full-pack worktrees `worktrees/hook-activation-closure` (`fix/hook-activation-closure`) and `.claude/worktrees/{elastic-moore-46d8e7,goofy-ritchie-6e4a6c,optimistic-morse-06f4c1}`, with their branches;
  - remote branch `origin/fix/prometheus-exec-path-independent-build`.

  Remove each one once its session ends. Archive unmerged work first with `git update-ref refs/tags/archive/<date>/<branch> <sha>`: this repo has `tag.gpgsign=true`, so plain `git tag` fails without a signature. Then grep the harness configs for the path before removing it.
- **Two archived branches contain a Slack API token** in commit `1e18a9b`: `backup/skill-pack-1-10-pre-push-rewrite` and `feat/cpc-001-002-integration-contract`. GitHub push protection blocked them, so they exist only as local tags and in `branch-archive-20261005/prometheus-skill-pack/local-only-secret-blocked.bundle`. **Never push them.** If that token is real, rotate it.
- **Unmerged work from older branches** is preserved as `archive/20261005/*` tags on each repository's origin. Uncommitted worktree state is in `/Users/gqadonis/Projects/prometheus/branch-archive-20261005/` (patches, untracked tarballs, a `.kbd-orchestrator` snapshot, and the previous local cadence procedure).
- **Installed on this machine** (refresh receipt `evidence/refresh-full-20261005.json`):
  - skill-pack `f35fe2a`, plugin generation `580c6476…`;
  - pk and the learning worker at 1.11.0;
  - surreal-memory-server **1.10.0** (the pins still point at 1.10.0; see 3.1).
- **Doctor:** `prometheus doctor --json` shows `codex.memories` = pass.
- **Cadence shim:** installed at `prometheus-skill-pack/.prometheus/cadence/procedures/refresh-skill-pack.sh`.

## 2. Operator rules (binding)

- **The user merges PRs and approves outward-facing actions** explicitly in chat: merges, tags, `versions.toml` edits, and changes to machine config such as `~/.codex` or `~/.claude`. Pushing feature branches and opening PRs is the normal workflow and is allowed.
- **All validation is local.** Never use GitHub Actions as evidence.
- **Implementation first, integration tests only.** Implement a whole change, then run its gate once. Exit 2 means BLOCKED, never a pass.
- **One cargo or rustc build on the machine at a time.** Other agent sessions build too. Check `pgrep -x cargo || pgrep -x rustc` first, and treat a BLOCKED gate as "rerun later".
- **Tests use scratch homes.** Use scratch `HOME`, `CODEX_HOME`, `CORTEX_DATA_DIR` and `PROMETHEUS_PLUGIN_ROOT`. Never write the real `~/.claude`, `~/.codex`, `~/.cortex` or `~/.prometheus`, or the live surreal-memory on `:23001`, from a test.
- **Never edit a plugin cache.**
- **KBD tasks.** Use `kbd-apply begin-task`/`end-task`. `mark-done` now also syncs the ledger (#153). `kbd-apply reconcile <phase>` reports drift.
- **Work in worktrees.** Use worktrees off `origin/main` under `/Users/gqadonis/Projects/prometheus/worktrees/`, and remove them after their PR merges. Before removing any worktree, grep `~/.claude/settings.json`, `~/.claude/plugins/known_marketplaces.json` and `~/.codex/config.toml` for its path.
- **Generated outputs:** after editing `shared/` or `skills/`, regenerate with `node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js`. For generated-only rebase conflicts, use `scripts/rebase-regenerate.sh`.
- **Merges:** after each merge, check that `origin/main` contains the PR's final head.
- **End every turn with the full KBD status.**

## 3. Next phase: `phase-learning-deploy-and-debt` (seeded in `reflection.md`)

Start it with `/kbd-next-phase phase-learning-deploy-and-debt`, then `/kbd-assess`. Its scope:

1. **Deploy what shipped:**
   - with operator approval, bump `tools/surreal-memory-server` to v1.10.1 in the full pack and in mini (gitlink plus the `versions.toml` line, which the operator authors);
   - refresh the machine (`.prometheus/cadence/procedures/refresh-skill-pack.sh --mode full`);
   - measure SubagentStart recall under load with the cache live.
2. **Close the review debt** (`reflection.md` → Technical Debt, 12 items). Highest value first:
   - **Hook Python writes `__pycache__` into the immutable installed generation.** It broke this phase's refresh twice. Run hook Python with `-B` or set `PYTHONDONTWRITEBYTECODE=1`, or make the generation verifier ignore `__pycache__`.
   - **Codex regrew `memory_summary.md` despite `generate_memories = false`.** Investigate the Codex memories keys (consolidation, `use_memories`).
   - **surreal-memory cache:** key it on the model id rather than the dimensions; fix the hit/miss counter semantics.
   - `assertSharedGeneratedPaths` in `generate-skill-system-distribution.js` is vacuous.
   - The `codex.memories` doctor repair hint uses a repo-relative path.
   - `install-system.js` overrides a custom `CODEX_HOME`.
   - The Cortex feeder has no concurrency cap.
   - The real-Cortex test asserts on the live `~/.cortex/memory.db`, which is flaky.
3. **Process:**
   - add "test through the production entry point" to the spec template;
   - refresh mini's test-failure baseline, since mini main has 11 known failures;
   - fix mini's dist generator dropping the executable bit on `refresh-skill-pack.sh`.

## 4. Useful files

- **Phase directory:** `prometheus-skill-pack/.kbd-orchestrator/phases/phase-team-learning-hardening/`
  - `assessment.md`, `analysis.md`, `plan.md` (Task model assignments), `execution.md` (full evidence log), `reflection.md`;
  - `review/` (all adversarial review findings), `evidence/`.
- **Archived changes:** `prometheus-skill-pack/.kbd-orchestrator/changes/archive/2026-10-05-change-tlh-*`
- **Branch and worktree archive:** `/Users/gqadonis/Projects/prometheus/branch-archive-20261005/`
