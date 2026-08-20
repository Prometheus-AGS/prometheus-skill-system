---
type: SessionRecord
id: karpathy-session-24a335662b189b18
title: Karpathy session 24a335662b18
tags:
- karpathy
- session-learning
sources:
- session:9db42325-24eb-4238-98b2-20ac2824b479
timestamp: 2026-08-20T11:09:09.390912+00:00
created_at: 2026-08-20T11:09:09.390912+00:00
updated_at: 2026-08-20T11:09:09.390912+00:00
revision: 0
---

## Delta

All instrumentation confirmed working. The work is complete.

## Summary

I reviewed the audit, verified its claims against the actual repo, and executed the actionable recommendations. **All 14 AI tools are updated** (`Targets: 14/14`, exit 0), tree clean, pushed to `main`.

### The audit needed correcting first

Verification found it unsafe to follow literally:

- **Its roadmap points at infrastructure that doesn't exist.** ~15 of 33 weaknesses route to six HMA skills and their verifier scripts — HMA has 30 skills, **none of them these**, and none of the scripts. `prometheus-companion` is a 2-commit scaffold behind a 4405-line spec. Pillars 5–6 aren't skill-pack work.
- **Five claims were false**: "40+ skills" (actually **312**); "descriptions exceed 1024 chars" (**zero** do, max 663); "no plist has ThrottleInterval" (4 already did); `service-probe.sh` "missing" (exists); "TCP probes" (they're HTTP).

I appended §18 to the audit and re-marked 28 `§15` rows as *"Not implemented — design only."*

### What shipped

| Fix | Evidence |
|---|---|
| **60s hook stall** → PID-aware lock | Stale-lock recovery **60s → 0.036s**; live holder still respected |
| **Plist throttling** | 5 plists fixed; all 12 lint clean; doctor check proven to catch regressions |
| **Hardcoded username** | `prometheus-services.sh` refused to run for anyone but `gqadonis`; now `config/defaults.env` |
| **Hook observability** | 7 of 17 contract-wired hooks were invisible; exit-code propagation verified with a script exiting 42 |
| **Skill descriptions** | Guide + validator check; 16% had triggers, **0%** had exclusions |
| **sccache** | Was installed with **0 compile requests**; clean build **8.62s → 3.33s** |
| **Plugin schema** | 27/27 manifests validate |

### Where I overrode the audit on measurement

- **Withdrew R6.3** (cache the SHA): measured at **~14ms** against `evaluate-session.sh` at 16.4s — wrong target, and it trades away per-invocation integrity.
- **Withdrew R6.5** (anchor matchers): the semantics are undocumented, and live telemetry proves bare matchers work today. Anchoring risked silently disabling 15 hooks.
- **Withdrew W6.3's fallback claim**: it's a documented, deliberate catch-all — the log shows it firing 7× alongside 5 role runs.
- **Rejected lld** after adding it: measured as *not faster* and it added linker warnings to signed binaries.

### Two things you should know

1. **`artifact-refiner` PR #5** is open, not merged, per your instruction. I proved the bug: the quoted heredoc passed the literal string `'$STATE_FILE'` to Python. The submodule is back on its pinned sha, so your install stays reproducible until you merge.

2. **`prometheus doctor` exits 1** on a pre-existing issue I did not cause: `tier-w-trust` fails because `~/.prometheus` holds **25 generations, 2.1 GB**, 12 sharing a component digest. The supported `--prune-obsolete` halts on an unsigned legacy generation. This didn't block the refresh, but it's worth clearing separately — I'd rather flag it than force past a signature check.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 9db42325-24eb-4238-98b2-20ac2824b479
- Captured: 2026-08-20T11:08:29.615998Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
