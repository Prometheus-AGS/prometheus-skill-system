---
type: SessionRecord
id: karpathy-session-8f8ef5f0cbeeb3bd
title: Karpathy session 8f8ef5f0cbee
tags:
- karpathy
- session-learning
sources:
- resource: session:fab27b8e-2324-4f2e-ade9-dcde5ddd3403
generated:
  by: pk/1.9.0
  at: 2026-10-04T13:18:24.077971+00:00
created_at: 2026-10-04T13:18:24.077971+00:00
updated_at: 2026-10-04T13:18:24.077971+00:00
revision: 0
---

## Delta

The 19 failing tests on mini's main are fixed, and the full suite now passes locally: 1047 pass, 0 fail, and the same 2 skips as main. The PR is open for you to merge: [Prometheus-AGS/prometheus-skills-mini#31](https://github.com/Prometheus-AGS/prometheus-skills-mini/pull/31).

I worked in a fresh worktree off `origin/main` (bf2c1ee) and ran `npm ci` first. `npm run check` and `npm run check:distribution` also pass, and `dist/` is regenerated.

**What was wrong:**
- **Packaging fixtures (tests 19–28, 934–935):** commit c70b1c5 made every package include the KBD scripts (`kbd-apply.mjs` and the others) plus the OpenSpec README. The test fixtures only supplied `hook-entry.mjs`. The script list is now one exported constant, and the fixtures supply those files. I also added two assertions: both packages contain every script, and a missing script fails the build.
- **OpenSpec version pin (584, 585, 890, 891):** the pin and the minimum version were both raised to 1.14.0 on purpose; only the tests still expected 1.10.0. They now expect 1.14.0, and one also checks that the pin equals the minimum.
- **Shared lock (484, 895):** these weren't version problems. The tests used the real `~/.prometheus/openspec` folder, which has one lock for the whole machine. Test files running in parallel, and session hooks, collided on it and exited with code 75. A new helper, `lib/platform/openspec-test-home.mjs`, gives each test its own private copy of the pinned OpenSpec.
- **Lock collision in the kbd-apply integration test:** with the fixes above in place, the same lock collision appeared there once, so its fixture uses the helper too.

**Decision for you — test 464:** this one was a real behavior change, not a fixture problem. c70b1c5 removed the check that OpenSpec can actually run before `detectBackend` chooses it. That contradicts the original `kbd-apply.sh` (`command -v openspec`) and the function's own "mirrors exactly" comment. I put the check back. The cost: in a repo with an OpenSpec change, each `detectBackend` call now launches an extra `openspec --version` run, and a locked or disabled OpenSpec counts as "not available". If you'd rather keep c70b1c5's "always available" behavior, the alternative is dropping half of test 464, which would weaken it. The PR flags this for your review.

`versions.toml` was not touched. GitHub shows 14 checks pending on the PR; I didn't watch or cite them, since repo policy is local-only validation.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: fab27b8e-2324-4f2e-ade9-dcde5ddd3403
- Captured: 2026-10-04T13:16:07.104272Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/optimistic-morse-06f4c1

## Changed Paths

- No changed paths detected.
