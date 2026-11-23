---
type: SessionRecord
id: karpathy-session-050dfb19eaed5cdc
title: Karpathy session 050dfb19eaed
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T15:07:25.259341+00:00
created_at: 2026-11-17T15:07:25.259341+00:00
updated_at: 2026-11-17T15:07:25.259341+00:00
revision: 0
---

## Delta

The runtime-only plugin is built and tested, but not committed and not yet tried in a live Claude Code session. It is on branch `feat/claude-runtime-plugin` in a scratch worktree, and I have not touched your Claude Code settings.

## What it is

`prometheus-runtime` is a second generated Claude plugin at `dist/plugins/claude/prometheus-runtime`, offered in the marketplace. It carries the pack's hook file (identical to the full plugin's) and the MCP server config, and no skills. It is 184K.

With it enabled instead of `prometheus-skill-pack`, the installer's domain selection controls what Claude Code lists. In a scratch home with only the runtime plugin enabled and `--domains rust`, the Claude listing came to 14 skills and about 5,000 characters, against about 68,900 today.

## The rule it introduces

Enable exactly one of `prometheus-skill-pack` and `prometheus-runtime`. Both carry the same hooks, so enabling both runs every hook twice, and enabling neither runs none.

- **Installer advice corrected.** It no longer says "disable the plugin". It now says to enable `prometheus-runtime` and disable `prometheus-skill-pack`, names domain plugins that only duplicate skills, and warns when both hook carriers are on.
- **Update script.** It now refreshes `prometheus-runtime`, no longer fails when the full plugin is disabled, and leaves disabled plugins alone, because reinstalling one would switch it back on. It warns on zero or two hook carriers.

## Verification

- A new test confirms the runtime plugin ships no `SKILL.md` and that all of its hooks dispatch from a generation installed from the full payload.
- `claude plugin validate` passes on it.
- Eleven existing gates pass, including payload activation, hook dispatch and the distribution checks.
- The update script's own test was failing on a version number pinned in its fixture, before reaching any real check. I made the fixture read versions from the repo; it now passes and exercises my edit.

## Not verified

- Claude Code has not loaded the plugin in a real session, so I have not seen its hooks fire there or confirmed it lists no skills.
- The update script's new paths for a disabled full plugin and for the runtime plugin are not covered by its test.
- I did not rerun the domain-selection integration test after changing the installer's warning function, or full `npm test`.
- Codex is unchanged; its listing is managed by the separate catalogue policy.

Once this merges, the switch on this machine would be: update, enable `prometheus-runtime`, disable `prometheus-skill-pack` and `prometheus-process-skills`, pick domains, restart. I would do that only on your say-so.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T15:06:23.639103Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
