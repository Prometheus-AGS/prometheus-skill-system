---
type: SessionRecord
id: karpathy-session-dd2c1adcbb8217db
title: Karpathy session dd2c1adcbb82
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T20:03:22.259093+00:00
created_at: 2026-11-17T20:03:22.259093+00:00
updated_at: 2026-11-17T20:03:22.259093+00:00
revision: 0
---

## Delta

The machine is updated: both checkouts are at their merged `main`, and the full pack is reinstalled from commit `ff74e1ef`.

- **Full pack:** the update script exited 0 and activated generation `de92c016`. The installed skills now carry the OpenSpec 2.1.0 pin and `skill-certification` 1.0.1 with the parser fix.
- **Claude Code plugins:** `prometheus-runtime` is still the one enabled; `prometheus-skill-pack` and `prometheus-process-skills` remain disabled.
- **Mini pack:** the checkout is at `2b085a88`. I left its locally modified `tools/prometheus-knowledge` submodule alone, and the mini pack is not installed into any harness here, so there was nothing to refresh.
- **Checkout dependencies:** I ran `npm ci` in the full-pack checkout. Its `node_modules` still had OpenSpec 1.14.0; it now has 2.1.0.
- **Cleanup:** I removed this session's seven scratch worktrees and deleted their local branches, after confirming each was clean and its pull request merged. One worktree from a different session (`main-wt`) is still registered in the full-pack repo; I did not touch it.

Restart Claude Code and start a new Codex session to load the refreshed pack. I have not checked a live session in either.

- **Stale git lock:** the full-pack checkout had a zero-byte `.git/index.lock` again, with no git process running. I removed it so the pull could proceed. This has now happened several times today.
- **Old Codex copies:** the update log lists many entries under `~/.codex/skills` as "Preserving unverified copy". Those are older pack copies the installer could not prove it owns, so it left them in place. They may duplicate entries in Codex's skill listing.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T18:49:27.367388Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
