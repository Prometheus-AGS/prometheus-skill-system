# change-tlm-003-revised-implementation-plan

**Title:** Revise the PR 4–14 implementation plan onto the team-aware design, with per-agent integration gates in both harnesses
**Repository:** `prometheus-skill-pack`
**Phase:** team-aware-learning-memory
**Depends on:** change-tlm-002-team-aware-learning-design, change-tlm-004-hook-timeout-seconds (its seconds-based timeout rule and the work it defers)
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` at or after `20d97f2` (contains PRs #111, #121–#123), at `/Users/gqadonis/Projects/prometheus/worktrees/tlm-design`. Never this checkout's `codex/delivery-cadence-recovery` branch, which lacks #121. Delivered through a pull request.

## Why

The approved plan (`~/.claude/plans/yes-fix-it-in-logical-mccarthy.md`, PRs 4–14) assumes project, user and global scopes only. It has no agent or role dimension, no SubagentStart delivery and no treatment of the file-memory tier, and it does not account for the pk truncation defect or the surreal-memory category filter. Phase goal 5 requires a revised plan built on the design.

## What Changes

- Write `docs/plans/team-aware-learning-memory-implementation.md`. It is an ordered PR list across four repos (skill-pack, prometheus-knowledge, surreal-memory-server, prometheus-skills-mini), replacing PRs 4–14. For each PR: goal, repo, files, design section implemented, dependencies, and an integration gate that exercises a production entry point (repo policy: integration tests only).
- The parent plan numbered these PRs 4–14 (PR 14 was the final pin bump); the phase goals' "4–13" names the same span without the pin bump. The revised plan replaces PRs 4–14.
- The upstream pk and surreal-memory prerequisites come first, then a pin bump (v1.10.0). They are followed by identity capture, envelope writes, per-agent recall and SubagentStart delivery, file-tier reduction, cross-agent awareness, promotion, Feynman gaps, skill discovery and cross-team requests, each made role-aware. Also included, each as a gated PR:
  - SubagentStop matcher namespacing (deferred from change-tlm-004);
  - the prometheus-skills-mini hook-timeout fix (deferred from change-tlm-004);
  - the CLAUDE.md memory-chain update, so Cortex and file-memory writes carry envelope tags (analyze W4, handed over by change-tlm-002).
- **Per-agent end-to-end gates.** In a scratch project with a two-role team (`alpha`, `beta`, disjoint `owns`), lessons written by `alpha` reach a spawned `alpha` subagent and not a `beta` subagent. The gate measures delivered bytes against the budget and runs in both Claude Code and Codex. A `role:beta`-addressed lesson from `alpha` reaches `beta`.
- Update the session plan file (`$HOME/.claude/plans/yes-fix-it-in-logical-mccarthy.md`) to point at the revised plan, so the approved plan does not drift from it. First copy it to `.kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md` and record its sha256. Only a pointer section is appended; nothing above it changes.

## Scope

- `docs/plans/team-aware-learning-memory-implementation.md`
- `.kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md`
- `/Users/gqadonis/.claude/plans/yes-fix-it-in-logical-mccarthy.md` (outside the repo; this session's own plan file; pointer section only)

## Capabilities

- none (plan)
