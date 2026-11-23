---
type: SessionRecord
id: karpathy-session-6bd8cfffec433642
title: Karpathy session 6bd8cfffec43
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T09:24:48.495628+00:00
created_at: 2026-11-17T09:24:48.495628+00:00
updated_at: 2026-11-17T09:24:48.495628+00:00
revision: 0
---

## Delta

Phase 1 is done, and the main result is that the pack passes its own validator but fails the official one. The agentskills.io reference validator passes only 16 of 198 native skills, while `npm run validate:strict` exits 0. The report is at [2026-11-17-certification-phase1-static-baseline.md](docs/reports/2026-11-17-certification-phase1-static-baseline.md). I fixed nothing and edited no skill.

## Why the official validator fails

The failures come from the pack's own frontmatter convention, not from individual mistakes:

- **`tags` as a list:** 147 native skills write `tags: [a, b, c]` under `metadata`. The spec says metadata values must be strings.
- **Top-level `version`:** 180 native skills declare it. The spec does not allow that field at the top level.

`docs/SKILL_TEMPLATE.md`, `scripts/validate-skills.js` and the compliance section of `CLAUDE.md` all steer authors towards these two constructs. Matching the spec means moving both into string-valued `metadata` keys, for example `metadata.version: '1.0.0'` and `metadata.tags: "a, b, c"`. That would also remove the DeepSeek Harness problem from the plan, where nested YAML is skipped. Keeping the convention means recording it as a declared deviation, and no skill reaches the first certification level as the plan defines it.

## Other findings

- **Lint (agnix):** 75 errors on native skills.
  - 33 are the metadata issue above.
  - 28 are `allowed-tools` entries that are not Claude Code tool names (`file_system`, `code_interpreter`, `web_search` and similar).
  - 11 are broken links, one of them in an intentional negative fixture.
  - 3 are `hooks/hooks.json` files that are a list where a map is expected.
- **Descriptions:** about 150 skills have no "use when" clause, and agnix independently flags 105 for the same thing.
- **Upload blockers:** four skill descriptions contain XML-like tags such as `<phase>`, which claude.ai and the API reject on upload.
- **Security (Cisco scanner):** 57 critical and 13 high findings on native skills. The ones I opened are pattern matches on legitimate code, mostly `child_process` use in the orchestration skills. I did not read the 257 medium and low findings. The scanner needs a reviewed policy file before it can gate anything.

## Counts that don't match

- `CLAUDE.md` says 310 skills; I found 198 native `SKILL.md` files (408 including imported mirrors).
- The repo validator reports checking 121 "top-level" skills. I did not verify whether the 77 nested ones are checked at all.

## Not run

- `skills-lint`, `skill-validator`, `avocetta` and `skillcheck` were not run.
- The scanner's description-overlap check was not run; it is the obvious next measurement.
- Raw tool output is in the session scratchpad, not committed. The report lists the exact commands to reproduce it.
- The `skills-ref` already installed on this machine is broken, so I used a fresh copy through `uvx`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T09:24:32.218165Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
