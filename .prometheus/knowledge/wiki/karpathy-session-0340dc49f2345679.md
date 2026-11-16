---
type: SessionRecord
id: karpathy-session-0340dc49f2345679
title: Karpathy session 0340dc49f234
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T11:11:51.925995+00:00
created_at: 2026-11-16T11:11:51.925995+00:00
updated_at: 2026-11-16T11:11:51.925995+00:00
revision: 0
---

## Delta

The 141 orphaned store directories are gone from `kbd/projects/`. I moved them to the Trash instead of deleting them outright, so they are recoverable until you empty it: `~/.Trash/prometheus-kbd-orphaned-stores-20261116/`.

- **What they were:** 138 belonged to projects that had only ever been registered at temp test paths, and 3 were not in the registry at all even before the prune.
- **What they held:** 140 contained nothing but two empty lock files. The one with data, about 11 KB, was `11111111-2222-3333-4444-555555555555`, a synthetic test project id.
- **Safety check:** I re-read the registry at move time and only moved a store if it was still unreferenced. All 141 qualified and none was skipped.
- **Result:** 17 store directories remain, each belonging to a registered project, and no registered project is missing its store. `prometheus kbd status` still resolves this project at revision 198.

The exec pin is still uncommitted. Its second-checkout test build is still running in the background, and I'll commit and push when it passes.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T11:11:50.048721Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/rust-toolchain.toml
- dist/plugins/claude/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- dist/plugins/codex/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/build-prometheus-exec.sh
- scripts/install-prometheus-exec.sh
- scripts/tests/install-prometheus-exec.test.sh
- skills/process/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/prometheus-exec-pin.sh
