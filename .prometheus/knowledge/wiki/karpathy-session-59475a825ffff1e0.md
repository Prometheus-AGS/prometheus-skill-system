---
type: SessionRecord
id: karpathy-session-59475a825ffff1e0
title: Karpathy session 59475a825fff
tags:
- karpathy
- session-learning
sources:
- resource: session:733a99fd-b357-4127-87d1-0c794fcde506
generated:
  by: pk/1.9.0
  at: 2026-10-04T11:08:36.172009+00:00
created_at: 2026-10-04T11:08:36.172009+00:00
updated_at: 2026-10-04T11:08:36.172009+00:00
revision: 0
---

## Delta

# Team-aware learning: detailed status

**Overall:** 7 of 25 changes done, which is 25 of 82 tasks. Everything is merged except the surreal-memory release PR, [surreal-memory-server#45](https://github.com/Prometheus-AGS/surreal-memory-server/pull/45). The pk release PR, [prometheus-knowledge-rs#35](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/35), merged at `1126c30`.

**One bookkeeping note.** A bug in my change-file generator had dropped A5a and A5b from the phase index. Their files and runtime registration were never lost. The generator is fixed, and all 25 changes are listed again.

## Done (7 changes)

**A1: pk recall scores every entry** · [pk#31](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/31) merged
- ✅ Score every entry in each scope and apply the candidate cap after scoring, with a fixed order.
- ✅ Integration test: a 200-entry scope, a match that sorts last, a failed scope that holds no budget, and stable output.

**A2: the learning worker attributes lessons and makes them recallable** · [pk#32](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/32) merged
- ✅ Jobs carry project, team and role. The scope keys are never null.
- ✅ Commit the prompt snapshot after each write, so `pk context` can find session records.
- ✅ Integration test: memory keys, task-step keys, and the snapshot contents.

**A3: pk `ingest --type/--tag` and `context --tag`** · [pk#34](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/34) merged
- ✅ `--type` and repeatable `--tag` on ingest. A type the caller supplies is no longer overwritten.
- ✅ `--tag` filter on context, where an entry must carry every tag given.
- ✅ End-to-end test through a local model endpoint.

**A4: surreal-memory search and re-key** · [sm#44](https://github.com/Prometheus-AGS/surreal-memory-server/pull/44) merged
- ✅ Search responses no longer include embedding vectors unless asked.
- ✅ The category filter applies to both search methods. It used to be accepted and silently ignored.
- ✅ A re-key operation for memories with no owner, which only runs from the same machine and supports a dry run.
- ✅ In-process integration tests, 4/4.

**B1: one project-id resolver and an agent identity resolver** · [skill#126](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/126) merged
- ✅ `project_id.py`, plus a bash wrapper: one precedence order shared by every worktree.
- ✅ `agent_identity.py`: strips the plugin prefix, maps Codex `_` back to `-`, matches team roles, and answers "is this a team role?".
- ✅ The four memory callers now use the resolvers.
- ✅ Integration test, 14 checks.

**B2: team roles no longer trigger iterative-evolver hooks** · [skill#128](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/128) merged
- ✅ Captured a real Claude payload: plugin agents report `iterative-evolver:planner`.
- ✅ Claude matchers are anchored to the plugin prefix. Codex gets copies that run through a team-role guard.
- ✅ Hooks and distribution regenerated.
- ✅ Test, 7/7, including proof through the real dispatcher.

**D2: mini hook timeouts in seconds** · [mini#29](https://github.com/Prometheus-AGS/prometheus-skills-mini/pull/29) merged
- ✅ Converted to seconds, with a 5-second startup floor.
- ✅ Budget test rewritten so it still guards.

## In progress (1 change)

**A5a: release v1.10.0 and move the skill-pack pins**
- ◐ **Task 1. Version bumps and tags.**
  - pk: bumped to 1.10.0 and merged (#35). Not tagged yet.
  - surreal-memory: bumped in [sm#45](https://github.com/Prometheus-AGS/surreal-memory-server/pull/45), **waiting for your merge**.
  - Both PRs also record the OpenSpec changes each repo requires.
- ☐ **Task 2.** Move the two submodule pointers, the `skill-system.json` import commits, and the pk crate pins in prometheus-cli and forge-rs to v1.10.0.
- ☐ **Task 3.** Update the release version matrix and the installation docs to 1.10.0.

## Not started (17 changes)

**Release follow-up**
- **A5b: m

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 733a99fd-b357-4127-87d1-0c794fcde506
- Captured: 2026-10-04T11:08:26.062388Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
