---
type: SessionRecord
id: karpathy-session-a91825b4d1a87748
title: Karpathy session a91825b4d1a8
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T12:42:20.650395+00:00
created_at: 2026-10-09T12:42:20.650395+00:00
updated_at: 2026-10-09T12:42:20.650395+00:00
revision: 0
---

## Delta

Everything is on `main` and `prometheus doctor` passes with 0 required failures. It now shows 4 warnings, down from 6. The learning-queue backlog and Codex memory warnings are gone.

**State now**
- **Source:** `main` includes PR #170 (CPU embeddings on Intel, the v1.10.1 pin, the doctor fixes). The local and remote branches are deleted.
- **Skills:** redistributed from `main`, generation `612fbe96…`, receipt written. Claude Code and Codex plugins are refreshed, and this time no skill copies were preserved as unverified. Restart Claude Code and start a new Codex session to pick up the refresh.
- **Memory daemon:** `surreal-memory-server` is v1.10.1, running on CPU with no Metal. `/ready` shows every capability true, and the server has no paused or non-terminal operations. All 12 launch agents are running after a final restart.
- **Learning queue:** the doctor no longer flags the backlog. The worker's `accepted` count was still 100 when I last looked, and I didn't re-check it after the final worker pass.

**Remaining warnings (all expected or outside this repo)**
- **Marketplace source:** your wiki files are modified again. I merged both temporary stashes into the working tree, so nothing is lost. It clears when you commit or discard them.
- **KBD runtime authority:** the installed `sovereign-sync` daemon (Aug 26) is owned by `prometheus-companion`. The identity fix is already in `main`, so the daemon needs rebuilding by its own installer. I didn't touch it.
- **KBD rollout gate:** shadow mode needs 7 days of evidence. This is expected, and the docs now say so.
- **Discovery budgets:** no harness has recorded a real trace yet, and the doctor won't invent a ceiling.

**Follow-ups I did not do**
- **Upstream self-heal:** make `surreal-memory-server` restart a poisoned executor and fall back to CPU (would be v1.10.2). The CPU default is what actually fixes this machine.
- **Memory policy on update:** `update-skill-pack.sh` never applies the Codex memory policy, so a fresh update can bring that warning back. I applied the policy by hand this time.
- **Other pin drift:** `openai-proxy` and `prometheus-entity-management` are out of sync with `skill-system.json`, and the new check script flags them.
- **Codex catalog size:** `~/.codex/skills` holds about 661 entries against the roughly 360 where the docs say Codex stops auto-triggering skills properly. This probably deserves its own change.
- **Backups to clean up once you're satisfied:** `~/.prometheus/backups/foreign-skills-20261009-064751/` and `~/.prometheus/backups/learning-queue-20261009-072214/`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T12:42:15.244202Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- crates/prometheus-exec/.prometheus/
- history.txt
