---
type: SessionRecord
id: karpathy-session-724ebba86694cf23
title: Karpathy session 724ebba86694
tags:
- karpathy
- session-learning
sources:
- resource: session:0ffe50ed-a878-4de2-b03a-ec3e58a38524
generated:
  by: pk/1.11.0
  at: 2026-11-16T08:33:22.318351+00:00
created_at: 2026-11-16T08:33:22.318351+00:00
updated_at: 2026-11-16T08:33:22.318351+00:00
revision: 0
---

## Delta

All six R4 tasks are written. Following VLBV, none of it has been executed yet, not even a syntax check: everything runs once, at B4.

**What's new**
- **`evals/vlbv-protocol/cases/vlbv-protocol.json`:** 23 cases.
  - **17 agent cases** run in fresh scratch projects. Small marker scripts record whether tests, builds, generated code, publishing or sending actually ran, so the checks look at what happened on disk, not at what the agent says it did.
  - **6 machinery cases** test the real KBD scripts.
- **`run.mjs` plus `adapters/`:** drive Claude Code, Codex, OpenCode, Kimi and Qwen headlessly.
  - Each sample runs in a scratch home outside the repo, with only the minimum auth copied in and configs stripped of MCP servers, hooks and plugins.
  - Real harness files and the harness-related keychain items are hashed before and after; any change stops all further harnesses.
- **`lib/grade.mjs`:** grades through liter-llm with MiniMax-M3 (glm-5.3 when the harness is MiniMax). It refuses a reply from any other model, because the fallback gateway `openai-proxy` ignores the requested model.
- **`lib/report.mjs` and `validate-report.mjs`:** apply the declared 3-of-3 / 2-of-3 rule and re-derive every verdict from the retained transcripts and file effects, so a grader's PASS alone can't produce a pass.
- **`machinery.mjs`:** reuses the existing phase-closure conformance adapter, with the real `prometheus` runtime, for the six machinery cases and their six negative controls.
- **Capability promotion:**
  - `apply-capabilities.mjs` promotes a capability only from a PASS case that tests that field.
  - `check-harness-adapters.js` now requires per-field evidence and accepts `--capabilities <file>`.
  - Codex `skillRoots` got its evidence pointer.
- **CR-33:**
  - `kbd-boundary-authorize.mjs` records the B4/B5 authorization event before the first attempt.
  - The closure packet now lists runner output older than that event.
  - `runnerOutputGlobs` is added to the v2 schema; it was missing, so the adopted contract would have failed full schema validation.
  - `scan-fixture.mjs` drives it with mixed-date fixtures.
- **Docs:** `boundaries.md`, `value-scenario-verify` (1.2.0), the evals README and the CHANGELOG.

**Deviations, so you can judge them**
- **Leftover negative control:** Claude Code copies no auth file (its login is in the keychain). In `--keep-scratch` mode the runner plants a labelled non-secret probe file so the leftover check has something to find. It also runs one case instead of 17 and removes the kept scratch afterwards.
- **Report-validator negative (b):** this checks for "a forbidden command ran" through the marker file it leaves, not by searching transcript text. The contract says "transcript", but prose like "I won't run npm test" would make text matching unreliable.

**Not done or unverified**
- **Task ledger:** I couldn't mark T1–T6 complete in KBD because Bash permission checks kept failing. I'll retry that first next turn.
- **Generated outputs:** contract templates, `dist/` and formatting still need regenerating. That happens at B4 with the other generated outputs.
- **Unknowns until B4:**
  - Headless auth inside the scratch homes is unverified for every harness. Claude Code with a relocated config directory is the most uncertain.
  - The 600-minute budget leaves about 2.3 minutes per session. Harnesses that can't authenticate will report BLOCKED, never PASS.
- **Nothing committed.**

The next step is your B4 authorization; I'll record it with the new authorization script before the first attempt.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 0ffe50ed-a878-4de2-b03a-ec3e58a38524
- Captured: 2026-11-16T08:33:10.035225Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
