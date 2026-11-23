---
type: SessionRecord
id: karpathy-session-9a713f539f073a00
title: Karpathy session 9a713f539f07
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T12:09:11.094231+00:00
created_at: 2026-11-17T12:09:11.094231+00:00
updated_at: 2026-11-17T12:09:11.094231+00:00
revision: 0
---

## Delta

Every native skill is now in a domain plugin that fits the listing budget, with a check that keeps it that way. The PR is open and not merged.

<pr-created>https://github.com/Prometheus-AGS/prometheus-skill-system/pull/231</pr-created>

## What changed

- **Existing plugins filled.** The process plugin went from 19 to 51 skills, including the 23 KBD lifecycle skills. React went from 6 to 28, research from 1 to 11, devops from 4 to 5.
- **New plugins.** Rust (13 skills), Flint (6), and ten one-skill plugins for the single-skill categories. `iterative-evolver` already had a manifest and is now in the marketplace.
- **Coverage check.** `npm run check:domain-plugins` fails when a skill is in no domain plugin, in two, or when a plugin's listing exceeds the ceiling. It runs in `npm test`.
- **Docs.** `CLAUDE.md` now recommends domain plugins over the full pack and states the two rules.

Coverage went from 95 of 196 native skills to all 196.

## Does it work

| Case | Full pack | Domain plugin |
|---|---|---|
| Rust skill, two cases | fired 0 of 6 | fired 6 of 6 |
| `kbd-status` asked by name | not measured | all outcome checks passed 3 of 3 |
| `kbd-status` from "where are we on this project?" | fired 0 of 3 | fired 0 of 3 |

The Rust plugin restores triggering. `kbd-status` loads and works when named, but still doesn't start from a plain request; that is a description problem this PR doesn't fix.

## A regression I caused, now fixed

The distribution test has been failing on `main` since PR #227. That PR stopped packaging `evals/` folders, but the test still compared against the full source tree, and I never ran it. The fix is in this PR.

## What you should know

- **Domain plugins have no shared runtime or hooks.** They are source directories, not generated packages. Skills that call the shared scripts won't find them (15 skill files do), and the VLBV and learning hooks don't run. This was already true of the existing domain plugins, and it's the main gap between them and the full pack.
- **KBD couldn't be its own plugin.** Its directory holds KBD's own `hooks/hooks.json`, which Claude Code tries to load as plugin hooks and rejects. KBD stays in the process plugin.
- **Ten one-skill plugins.** A plugin's skills must sit under its root, so each single-skill category became its own plugin. Consolidating them means moving skill directories.
- **Installing every domain plugin equals the full pack in size.** The benefit comes from installing only the domains you use.
- **Entity plugin validation fails with 58 errors, before and after this change.** The cause is malformed nested manifests that came from the upstream repo.
- **A stray test run of mine** wrote a `certevals/` folder into your main checkout. I removed it and it isn't in the PR.

Not run: the full `npm test`, any install, and Codex.

## Decision for you

The shared-runtime gap is what stops domain plugins from being a full replacement for the pack. Should I build self-contained domain packages (generated, with the shared runtime and hooks) next, or go on to the certification skill?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T12:09:01.113777Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
