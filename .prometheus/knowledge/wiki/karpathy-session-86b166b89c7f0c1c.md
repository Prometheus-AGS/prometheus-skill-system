---
type: SessionRecord
id: karpathy-session-86b166b89c7f0c1c
title: Karpathy session 86b166b89c7f
tags:
- karpathy
- session-learning
sources:
- resource: session:ddfed9c0-3939-414b-b8a9-3a0b30d2e01c
generated:
  by: pk/1.11.0
  at: 2026-10-27T17:01:12.578148+00:00
created_at: 2026-10-27T17:01:12.578148+00:00
updated_at: 2026-10-27T17:01:12.578148+00:00
revision: 0
---

## Delta

Everything is committed, pushed and in a PR. Nothing in these PRs is merged.

| Repo | PR | What it holds |
|---|---|---|
| Full pack | [#197](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/197) | The #194/#195/#196 implementation (merged earlier) |
| Full pack | [#198](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/198) | The generated hooks files that #197 missed (merged earlier) |
| Full pack | [#199](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/199) | Your uncommitted working tree, committed as is (open) |
| Mini | [#70](https://github.com/Prometheus-AGS/prometheus-skills-mini/pull/70) | #68, #69, the #194 guard, partial #63, bumped to 1.13.2 (open) |

**#199 contents, in four commits**
- **OpenSpec mirrors:** the agent command and skill mirrors under `.agents`, `.cursor`, `.devin` and `.opencode`.
- **KBD projections:** the phase projections, the position files, and the removal of 50 `tasks.md` files. Those deletions were already in your working tree; please review them before merging.
- **Handoff document:** `CLAUDE-DESKTOP-HANDOFF.md`.
- **Knowledge wiki:** 128 new wiki pages, the wiki index and log, and the session log.

**Left out of #199**
- The dirty `tools/surreal-memory-server` submodule.
- The wiki's `.index.lock` file.
- The 528 MB prompt-snapshot folder, which is gitignored. The repo is public, so I didn't want to publish conversation snapshots.

A secret scan over the committed files found nothing. It flagged one wiki page title, which is a false positive.

**Not covered by these PRs**
- **Binaries install:** it is still compiling in the background (`surreal-memory-server`). The plugin refresh finished (commit `8638656b`, 14 of 14 targets). The CLI is not installed yet and nothing is verified.
- **Checkout position:** the full repo is on the `chore/preserve-working-tree-2026-10-27` branch, not `main`. I'll switch back to `main` after the install finishes, before I verify it.
- **Stashes:** the older stashes `stash@{0}`–`{2}` are not mine and are untouched. I popped my own stash into the #199 branch.

Mini is also not installed on this machine, because its skill names collide with the full pack's.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: ddfed9c0-3939-414b-b8a9-3a0b30d2e01c
- Captured: 2026-10-27T17:01:09.509704Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- tools/surreal-memory-server
