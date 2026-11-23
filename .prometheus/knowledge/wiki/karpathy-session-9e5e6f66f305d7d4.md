---
type: SessionRecord
id: karpathy-session-9e5e6f66f305d7d4
title: Karpathy session 9e5e6f66f305
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T16:08:09.807853+00:00
created_at: 2026-11-17T16:08:09.807853+00:00
updated_at: 2026-11-17T16:08:09.807853+00:00
revision: 0
---

## Delta

The update script is fixed and its test passes, but the change is uncommitted and has only been run against a fake Claude CLI. It is on branch `fix/refresh-stale-runtime-plugin` in a scratch worktree.

## What changed

In `scripts/refresh-native-plugin-installs.sh`:

- **Stale detection:** before refreshing, it compares each enabled hook-carrying plugin's cached `hooks/hooks.json` with the one this checkout would install. If they differ, it reinstalls that plugin without needing `--force`.
- **Verification:** after refreshing, it compares again. If a carrier still differs, the run fails with a message saying to rerun with `--force`, instead of leaving hooks that stop at the next prune.
- **Scope:** this applies to both `prometheus-runtime` and `prometheus-skill-pack`. A disabled plugin is still left alone.

## Verification

The script's test gained three cases, all passing:

- a stale runtime cache is reinstalled on a plain update;
- a disabled full plugin in the same run is not touched;
- a carrier that is still stale after the refresh fails the run.

The script also parses under macOS's bash 3.2, and seven existing gates pass, including the updater and installer entrypoint tests.

Getting the test green needed two fixes to the test itself, not the script: the fake Kimi installer nested its copy on a second run, and the fake listing's lag simulation returned old data to my new cases.

## Not verified

- The real Claude CLI has not been through the stale path. This machine's cache is currently in sync, so the first real exercise will be the next update after a hook change.
- The post-check assumes `claude plugin list` reports the install path where `hooks/hooks.json` lives. That matched what I saw on this machine today.

Until this merges, keep using `--force` for updates here.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T15:58:52.792691Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
