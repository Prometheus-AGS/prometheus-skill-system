---
type: SessionRecord
id: karpathy-session-e44b10814f408e33
title: Karpathy session e44b10814f40
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T14:46:42.671621+00:00
created_at: 2026-10-09T14:46:42.671621+00:00
updated_at: 2026-10-09T14:46:42.671621+00:00
revision: 0
---

## Delta

The catalog PR is open and the dirty-tree fix is committed locally on `fix/dirty-tree-runtime-state`, but I haven't pushed that one yet.

- **Catalog PR #173:** https://github.com/Prometheus-AGS/prometheus-skill-system/pull/173. The body covers the 106 entries with healthy descriptions, the hidden third-party skills and how to undo it. It lists the checks I couldn't run, plus the 4 pre-existing memory-config test failures.

- **Dirty-tree fix (`40677c54`):** one rule in the three places that decide "is this checkout clean". The rule is `git status --porcelain --ignore-submodules=dirty` with `.prometheus/knowledge` excluded.
  - **Where:** the topology check, `update-skill-pack.sh`, and the `sourceTreeState` provenance in `install-plugin-generation.js`.
  - **Still counted:** a moved submodule pointer and any other tracked or untracked file.
  - **`.gitignore`:** adds `crates/*/.prometheus/` and `/history.txt`. I did not delete `history.txt`.
  - **Result on your checkout:** 58 dirty lines drop to 0, and the topology check no longer reports `SOURCE_DIRTY`. After this lands you shouldn't need to stash the wiki files before running the update script.
  - **Tests:** the new cases (wiki churn doesn't count, a dirty submodule doesn't count, a moved pointer does) pass and fail without the fix. The topology test file was already failing on `main`, because its fixture was missing `lib/store-paths.js`; I added that file, which is what let the new cases run.
  - **Not run:** I haven't run `update-skill-pack.sh` on this tree.

**Merge order:** this branch and #173 both regenerate the same hook-bundle files, so whichever merges second will conflict there. Regenerating them fixes that, and I can do it when you merge.

Should I push this branch and open its PR? I've left it local, since you only authorized the catalog push. Then I'm ready for the new issue.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T14:46:41.097006Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- tools/surreal-memory-server
