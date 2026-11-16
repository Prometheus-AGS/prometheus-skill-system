---
type: SessionRecord
id: karpathy-session-89c0d41facf19e50
title: Karpathy session 89c0d41facf1
tags:
- karpathy
- session-learning
sources:
- resource: session:0ffe50ed-a878-4de2-b03a-ec3e58a38524
generated:
  by: pk/1.11.0
  at: 2026-11-16T09:23:53.509683+00:00
created_at: 2026-11-16T09:23:53.509683+00:00
updated_at: 2026-11-16T09:23:53.509683+00:00
revision: 0
---

## Delta

**Yes, the OpenSpec 2.0 update is code complete.** It shipped as release 1.15.0 (PR #224, `d59a5a81`), and nothing in R4 blocks using it.

**R4 is paused, and nothing is lost.** At 03:15:49 another Claude Code session working in this same checkout stashed all the R4 work ("wip: vlbv-r4 before 1.15.0 install 2026-11-16") and switched the checkout to `main`. Everything is in `stash@{0}`, including the untracked files.

**Speeding up the rerun.** The five harnesses now run at once, and each runs 4 cases at a time. That should bring it from 3–4 hours to about 20 minutes. Running them at once departs from the contract's in-order packet; I'll record it as a deviation.

**Defects the faster rerun found, all fixed:**
- **Kimi:** prompt mode rejects both `--yolo` and `--auto`, so every Kimi session exited before starting. The adapter now uses plain `-p`.
- **False isolation alarm:** the check counted `~/.claude.json`, which the Claude Code session running *this* conversation rewrites constantly. That file and Claude's keychain items are now left out; the eval sessions never touch them.
- **Already working:** Claude Code's token auth. Before the stop, Claude Code was passing 3 of 3 on most cases, including AT04 (no early execution). Codex had two genuine FAILs (AT02, AT04) from the grader's judgement.

**Done so far:** both partial attempts' logs are saved. My restarted run then found its contract snapshot missing (it had been stashed) and exited before any model session, so nothing was spent on it.

**Next.** Once the other session is finished with this checkout, I'll pop `stash@{0}` onto `feat/vlbv-r4-portability` and restart the parallel rerun, about 20 minutes. To avoid a repeat, that session (or any future one) should work in its own worktree rather than this checkout.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 0ffe50ed-a878-4de2-b03a-ec3e58a38524
- Captured: 2026-11-16T09:17:25.584530Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
