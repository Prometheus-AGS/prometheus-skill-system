---
type: SessionRecord
id: karpathy-session-89649f31e9c36de8
title: Karpathy session 89649f31e9c3
tags:
- karpathy
- session-learning
sources:
- resource: session:7824f7e5-c967-4eff-b317-36d83cd6d762
generated:
  by: pk/1.11.0
  at: 2026-10-21T11:32:50.988222+00:00
created_at: 2026-10-21T11:32:50.988222+00:00
updated_at: 2026-10-21T11:32:50.988222+00:00
revision: 0
---

## Delta

The mini trace shows the runtime choice covers both packs at once. Mini has no state store of its own and shells out to the same `prometheus kbd` CLI for every status read, decision and transition, so one typed close in the runtime becomes the shared authority. It also found a real cross-pack bug: the two packs compute different contract digests for the same file, because the full pack hashes jq's trailing newline and mini doesn't. A contract adopted by one pack therefore reads as stale to the other.

The mini checkout at `../prometheus-skills-mini` is 7 commits behind `origin/main`, and the cited files only exist upstream, so mini work starts from a fresh worktree off `origin/main`. Two mini helpers, `stage-gate.mjs` and `kbd-child-exit.mjs`, are also missing from its installed payload even though shipped skills import them. Waiting on the full-pack design agent now.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 7824f7e5-c967-4eff-b317-36d83cd6d762
- Captured: 2026-10-21T11:32:49.153194Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- .agents/skills/openspec-apply-change/SKILL.md
- .agents/skills/openspec-archive-change/SKILL.md
- .agents/skills/openspec-explore/SKILL.md
- .agents/skills/openspec-propose/SKILL.md
- .agents/skills/openspec-sync-specs/SKILL.md
- .agents/skills/openspec-update-change/SKILL.md
- .agents/workflows/opsx-explore.md
- .agents/workflows/opsx-propose.md
- .agents/workflows/opsx-update.md
- .cursor/commands/opsx-explore.md
- .cursor/commands/opsx-propose.md
- .cursor/commands/opsx-update.md
- .cursor/skills/openspec-apply-change/SKILL.md
- .cursor/skills/openspec-archive-change/SKILL.md
- .cursor/skills/openspec-explore/SKILL.md
- .cursor/skills/openspec-propose/SKILL.md
- .cursor/skills/openspec-sync-specs/SKILL.md
- .cursor/skills/openspec-update-change/SKILL.md
- .devin/skills/openspec-apply-change/SKILL.md
- .devin/skills/openspec-archive-change/SKILL.md
- .devin/skills/openspec-explore/SKILL.md
- .devin/skills/openspec-propose/SKILL.md
- .devin/skills/openspec-sync-specs/SKILL.md
- .devin/skills/openspec-update-change/SKILL.md
- .devin/workflows/opsx-explore.md
- .devin/workflows/opsx-propose.md
- .devin/workflows/opsx-update.md
- .opencode/commands/opsx-explore.md
- .opencode/commands/opsx-propose.md
- .opencode/commands/opsx-update.md
- .opencode/skills/openspec-apply-change/SKILL.md
- .opencode/skills/openspec-archive-change/SKILL.md
- .opencode/skills/openspec-explore/SKILL.md
- .opencode/skills/openspec-propose/SKILL.md
- .opencode/skills/openspec-sync-specs/SKILL.md
- .opencode/skills/openspec-update-change/SKILL.md
- tools/surreal-memory-server
