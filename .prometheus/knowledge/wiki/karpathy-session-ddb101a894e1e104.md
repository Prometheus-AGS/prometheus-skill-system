---
type: SessionRecord
id: karpathy-session-ddb101a894e1e104
title: Karpathy session ddb101a894e1
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T15:00:29.752299+00:00
created_at: 2026-11-16T15:00:29.752299+00:00
updated_at: 2026-11-16T15:00:29.752299+00:00
revision: 0
---

## Delta

The guard is committed to `main` as `ec68ef8f` ("fix(installer): keep a liter-llm build failure from aborting the install"). It is not pushed, so `main` is one commit ahead of `origin/main`.

The commit holds the three files from this change: `scripts/install-binaries.sh`, `CLAUDE.md` and `CHANGELOG.md`.

I left out the 54 session wiki files under `.prometheus/knowledge/wiki`, which have been uncommitted all session and are not part of this fix. Say so if you want them committed as their own knowledge commit.

The stale hook bundle on `main` (from upstream commit `cd371c40`) is still there; `npm run validate:harness-adapters` keeps failing until it is regenerated.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T15:00:27.776125Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- No changed paths detected.
