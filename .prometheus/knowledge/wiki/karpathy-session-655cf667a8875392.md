---
type: SessionRecord
id: karpathy-session-655cf667a8875392
title: Karpathy session 655cf667a887
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T11:14:20.434904+00:00
created_at: 2026-11-17T11:14:20.434904+00:00
updated_at: 2026-11-17T11:14:20.434904+00:00
revision: 0
---

## Delta

Step 1 is done on both tracks. VLBV is now the stated default inside KBD-managed projects and opt-in everywhere else, and the entity skills have a sync script with a drift check. Neither PR is merged.

<pr-created>https://github.com/Prometheus-AGS/prometheus-skill-system/pull/229</pr-created>

## VLBV (PR #229)

A session-start hook prints the short boundary rule when the project has a `.kbd-orchestrator/` directory at or above it, and prints nothing otherwise. Outside KBD you turn it on or off with the first of these that is set:

| Setting | Scope |
|---|---|
| `PROMETHEUS_VLBV_DEFAULT` | the shell that starts the harness |
| `<project>/.prometheus/vlbv-default` (one word) | one project |
| `~/.prometheus/config/vlbv-default` (one word) | you, on this machine |

The values are `kbd` (the built-in default), `always` and `off`. The rule is an instruction, not a block: the operator can name another method, and a signed `KBD-METHOD` decision still wins.

The PR also rewrites the `vlbv` description, documents the setting in the skill, adds routes from `kbd-apply` and `prometheus-rust-workspace`, and adds two eval cases (operator override, and a plain project that must stay quiet).

**What the measurements show** (Claude Code, `claude-sonnet-5-5`, three runs each, full pack loaded):

- **The rule text works.** With it in the session, the KBD cases passed all five outcome checks on 3 of 3 runs. Without it they passed none.
- **Outside KBD it stays quiet.** The plain-project case gave a normal test command on 3 of 3 runs with no boundary rule.
- **The description rewrite alone did not help.** Without the rule text, `vlbv` still fired 0 of 3 times. The hook is what carries the default.

**What is not proven:**

- **The hook was not run inside a live session.** The eval tool cannot run any pack hook, because the hook entry point requires an activated installation and refuses in the eval's temporary home. I confirmed separately that the built dispatcher prints the rule in a KBD directory, and that the rule text changes behaviour when supplied to the session. The two were not tested together.
- **The Phase 2 pack results were measured with no pack hooks running**, for the same reason.
- **The override case has a gap.** The model followed the operator's method on 3 of 3 runs but never said VLBV was not being applied. That may be an artifact of the empty test workspace; I did not resolve it.

I also changed one existing test: the pinned hook count in `hook-dispatch.test.mjs` went from 67 to 69 for the new hook.

## Entity skills (PR #228, second commit)

`scripts/sync-entity-skills.mjs` now generates the shipped copy from the pinned submodule, and `--check` fails with a list of differences. It runs in `npm test`. I verified it passes on the branch, catches a one-line edit, and repairs it while leaving the three pack-only entries alone.

The sync gives a skill version `1.0.0` when upstream has none. Without that, the check would fail against the pinned v5.0.0 until #80 is merged and released.

## Still open

- PRs #228, #229 and entity-management #80 are waiting on you.
- I did not run the full `npm test` or an install on any of these branches.
- The broken upstream links in the entity skills are not fixed.

Shall I continue to the next step, measuring Claude Code's skill-catalogue budget, or pause here?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T11:14:09.942837Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
