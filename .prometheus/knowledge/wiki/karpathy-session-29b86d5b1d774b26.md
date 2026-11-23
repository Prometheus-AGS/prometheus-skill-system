---
type: SessionRecord
id: karpathy-session-29b86d5b1d774b26
title: Karpathy session 29b86d5b1d77
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T15:42:41.325437+00:00
created_at: 2026-11-17T15:42:41.325437+00:00
updated_at: 2026-11-17T15:42:41.325437+00:00
revision: 0
---

## Delta

The fix is committed and open as [Prometheus-AGS/prometheus-skill-system#237](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/237), based on current `main` (one commit, `6b1f2bbd`).

**What it does:** a `.claude-plugin` directory that sits beside a `SKILL.md` is no longer copied into the packaged payloads or into an installed generation. Twelve skill directories carry one. The manifests stay in the repository, where the marketplace reads them.

**Verification:**
- Packaged payloads have none under `skills/` after regeneration.
- An install from the repo root into a scratch home has none in the generation or the projected Claude skills.
- The domain-selection integration test passes with a new assertion for this.
- Payload activation, the runtime plugin test and the usual distribution gates pass.

**Not confirmed:** that Claude Code actually drops the four `@skills-dir` plugins once the manifests are gone. That needs an update and a restart on this machine.

This machine is unchanged by this PR so far. After it merges, one update from `main` and a Claude Code restart should remove the four duplicates; `claude plugin list` will show whether it worked. Tell me when it's merged and I'll run the update and check.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T15:14:28.158523Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
