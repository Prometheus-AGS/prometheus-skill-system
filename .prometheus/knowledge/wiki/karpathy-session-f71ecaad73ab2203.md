---
type: SessionRecord
id: karpathy-session-f71ecaad73ab2203
title: Karpathy session f71ecaad73ab
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T10:50:44.509863+00:00
created_at: 2026-11-17T10:50:44.509863+00:00
updated_at: 2026-11-17T10:50:44.509863+00:00
revision: 0
---

## Delta

The pack was not on 5.0.0, and I've opened a pull request that moves it there. The downstream PR #80 was already based on the 5.0.0 release, so that part was fine.

<pr-created>https://github.com/Prometheus-AGS/prometheus-skill-system/pull/228</pr-created>

## What I found

- **The submodule was pinned at v4.2.0**, three releases behind (4.3.0, 4.4.0, 5.0.0).
- **The entity skills the pack ships were much older than the pin.** They are a hand-kept copy at `skills/react/prometheus-entity-skills`, and that copy matched no upstream release. It differed from v2.1.0 in 64 files and from v5.0.0 in 73, with 44 upstream files missing. It had none of the A2UI 1.0, AG-UI 1.0, A2A, Flutter or Tauri guidance.
- **There is no sync script or drift check** for that copy.

## What the PR does

- Pins the submodule to the `v5.0.0` tag and records the commit in `skill-system.json`.
- Replaces the shipped copy with the 5.0.0 skills: 44 new files, 73 updated, nothing removed. The three pack-only entries are kept (the plugin manifest and the two Wasm artifacts under `entity-graph-optimize`).
- Applies the spec-conformant frontmatter, so the copy matches the #80 branch exactly.
- Regenerates `dist/`.

I also pushed one more commit to #80: the root entity skill had no version, so it now carries `metadata.version: '1.0.0'` like the 27 sub-skills. Without it the pack's strict validator rejects an unmodified copy.

## What you should know before merging

- **33 of the copy's 93 relative links are now broken**, up from 2. The 5.0.0 skills link into their own repository (`../../packages`, `../release`), and those paths don't exist in the pack. I left them, because fixing them in the copy would make it diverge from upstream again.
- **The Wasm artifacts were not rebuilt.** `skill.wasm` and `component` were built against the old content; I did not check they still correspond.
- **The `prometheus-react-skills` plugin manifest is unchanged** (version 1.5.2, React-only description). It likely needs a bump, but that touches the release-version matrix.
- **Re-pin after #80 merges and is released**, so the pin itself carries the frontmatter fix.

## Validation

Passed in a worktree from current `main`: the distribution build and its drift check, strict skill validation, plugin manifest validation, and the official validator on all 28 entity skills. Not run: `npm test`, any install, and any behavioural eval of the new content.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T10:50:35.314188Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
