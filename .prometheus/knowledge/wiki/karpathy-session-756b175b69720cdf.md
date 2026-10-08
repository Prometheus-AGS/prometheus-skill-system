---
type: SessionRecord
id: karpathy-session-756b175b69720cdf
title: Karpathy session 756b175b6972
tags:
- karpathy
- session-learning
sources:
- resource: session:733a99fd-b357-4127-87d1-0c794fcde506
generated:
  by: pk/1.9.0
  at: 2026-10-04T04:20:17.595833+00:00
created_at: 2026-10-04T04:20:17.595833+00:00
updated_at: 2026-10-04T04:20:17.595833+00:00
revision: 0
---

## Delta

Seven of the 25 changes are built and passing their gates. They're in 7 PRs waiting for your merge. The next change (A5a, the v1.10.0 release) needs those merges plus one approval from you, so I've stopped here.

**Please merge, in this order:**

| Order | PR | What it does |
|---|---|---|
| 1 | [prometheus-knowledge-rs#31](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/31) | pk recall scores every entry. Before this, a 200-entry knowledge base was mostly invisible to recall. |
| 2 | [prometheus-knowledge-rs#33](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/33) | `ingest --type/--tag` and `context --tag`. Stacked on #31. |
| 3 | [prometheus-knowledge-rs#32](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/32) | The worker attributes lessons to project/team/role and makes session records recallable. |
| 4 | [surreal-memory-server#44](https://github.com/Prometheus-AGS/surreal-memory-server/pull/44) | Search responses drop embeddings, the category filter actually works, and an unattributed-record re-key that only runs from the same machine. |
| 5 | [prometheus-skill-system#126](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/126) | One project-id resolver and an agent identity resolver. |
| 6 | [prometheus-skill-system#127](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/127) | Team roles stop triggering iterative-evolver hooks. Stacked on #126. |
| 7 | [prometheus-skills-mini#29](https://github.com/Prometheus-AGS/prometheus-skills-mini/pull/29) | Mini hook timeouts in seconds. |

The two stacked PRs (#33 and #127) retarget to `main` on their own once their parent merges.

**Decision needed:** after the merges, may I bump prometheus-knowledge-rs and surreal-memory-server to 1.10.0 and push `v1.10.0` tags? Then I'll move the skill-pack pins. Mini's pins are yours to bump in `versions.toml`; I'll file the exact request.

**What's settled so far:**
- **The Codex side works.** SubagentStart and SubagentStop both fire through plugin hooks, and the injected lessons land in the child agent, not the parent. Last phase couldn't show this.
- **Bug found in surreal-memory:** a memory saved without an `agent_id` can silently overwrite a similar memory belonging to a different agent. The new writers always set `agent_id`, which avoids it. I noted it in #44 for a follow-up fix.
- **The spec review caught gates that could never fail.** In most verify scripts a build or test failure was hidden, so the script still printed "verify OK". I fixed this across all 25.
- **Mini has 19 test failures on `main` that exist without any of my changes.** The mini gates now check "no new failures" against that recorded list. A task to fix the 19 is offered as a chip.

**Deviations from the spec:**
- Mini timeouts got a 5-second startup floor instead of 1 second, so a Node hook isn't killed while starting.
- The A4 gate tests in-process against an embedded database instead of a separately started server.

**What's left:** 18 changes. The first per-agent check in both harnesses (the alpha check) comes after the release plus 3 more PRs (B3, B4, B5).

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 733a99fd-b357-4127-87d1-0c794fcde506
- Captured: 2026-10-04T04:19:14.973852Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
