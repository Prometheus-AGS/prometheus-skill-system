---
type: SessionRecord
id: karpathy-session-e8cee4541fb1a905
title: Karpathy session e8cee4541fb1
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T12:31:29.915842+00:00
created_at: 2026-10-09T12:31:29.915842+00:00
updated_at: 2026-10-09T12:31:29.915842+00:00
revision: 0
---

## Delta

The three commits are pushed and the PR is open: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/170

- **Services:** CPU embeddings by default off Apple Silicon, and the `metal` feature only on arm64.
- **Pins:** `surreal-memory-server` pinned to v1.10.1, plus a new gitlink-vs-manifest check script.
- **Doctor:** clearer diagnostics for the learning-queue and Codex-memory checks, plus a docs note.

**State on this machine**
- The daemon runs v1.10.1 on CPU: `/ready` shows all capabilities true and no new poisoned-lock errors since the restart.
- The server shows 135 operations committed and none stuck.
- The Codex memory policy is applied in `~/.codex/config.toml`. The helper made its own backup, and there is a copy in the session scratch directory.
- The worker's local `accepted` count (100) hasn't dropped yet; it falls as each record's backed-off poll comes around.

**Two caveats**
- **The server rebuild is still running.** It is compiling without the `metal` feature and hasn't finished. When it does, the new binary needs the same re-sign (the macOS signature-cache problem) and an agent restart. I didn't run the final `prometheus doctor`, and the PR says so.
- **Gaps left out of the PR:**
  - The upstream self-heal fix (v1.10.2) and the `update-skill-pack.sh` memory-policy gap.
  - The `sovereign-sync` 404, which belongs to `prometheus-companion`.
  - Two other drifted submodule pins, which the new check script reports.
  - The Codex skill directory holding about 661 entries, against the roughly 360 where the docs say Codex auto-trigger breaks.

I'm still on the PR branch, so the doctor's topic-branch warnings will return until you merge. After you merge, tell me and I'll switch to `main`, pull, delete the branch, and finish the rebuild, re-sign, restart and doctor steps.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T12:31:28.210212Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- crates/prometheus-exec/.prometheus/
- history.txt
