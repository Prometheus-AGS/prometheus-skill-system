---
type: SessionRecord
id: karpathy-session-b8ba30b536b746c1
title: Karpathy session b8ba30b536b7
tags:
- karpathy
- session-learning
sources:
- resource: session:0ffe50ed-a878-4de2-b03a-ec3e58a38524
generated:
  by: pk/1.11.0
  at: 2026-11-16T06:32:05.959459+00:00
created_at: 2026-11-16T06:32:05.959459+00:00
updated_at: 2026-11-16T06:32:05.959459+00:00
revision: 0
---

## Delta

I haven't started R4. This is the description you asked for first.

## What R4 is

R4 is the last stage in the VLBV change register, called "Evidence of portability." It covers two change requests:

**CR-31: protocol evals (large).**
- **What it is:** an eval suite under `evals/`. It turns the VLBV acceptance tests AT 01–22, plus the §18 fresh-session check, into cases. The cases run on each harness against its capability profile.
- **The questions it asks each harness:**
  - Does the agent read its position before acting?
  - Does it refuse to run code before the B4 boundary?
  - Does it honour the policy block?
  - Does it keep the scenario verdicts honest?
- **Output:** one report per harness with pass / fail / not-run counts. Capabilities that weren't exercised are listed as unknown, not assumed.

**CR-33: passive scan for boundary violations (small).**
- **What it is:** an addition to `kbd-phase-close-eval.sh`. At phase closure, it lists runner outputs (test reports, build artifacts) dated *before* the boundary was authorized.
- **What it doesn't do:** it only reports and blocks nothing. It replaces an earlier idea, a hook that would intercept tool calls, which your AGENTS.md policy forbids (conflict CF-36).

## Why you'd need it

1.15.0 claims that one method runs the same way on every harness. Almost none of that claim has evidence behind it yet.

- **Nearly every capability is unknown.** `shared/harnesses/capabilities.json` profiles 8 harnesses with 7 capability fields each. Exactly one value is `supported`: the Codex skill root I tested. The other 55 are `unknown`, including all seven for Claude Code, your main harness. That's honest labelling, but it means nothing shows that Codex, OpenCode, Kimi or Qwen follow the first-action protocol or hold back execution until B4. Five of these harnesses are installed here: codex, opencode, kimi, qwen and zed.
- **Two VLBV requirements still have gaps.**
  - VLBV 19 asks that a second environment keeps the same claim semantics. CR-31 is the conformance run for that.
  - VLBV 07 asks for the oracle sensitivity challenge. The register notes it went unexercised in issues 195 and 196.
- **Without CR-33, early execution goes unnoticed.** The policy rightly forbids interception, so an agent that runs tests early anyway leaves no trace. CR-33 makes that visible at closure without adding a guard.

## When you don't need it

- **If you mostly work in Claude Code**, the method works today and R4 mainly turns "we believe" into "we measured." That's useful but not urgent.
- **CR-31 is costly.** It's sized L. It needs real model sessions per harness and per case, a grader, and someone to keep the cases current as the harnesses change.
- **CR-33 is small and cheap**, and it's useful on any harness, Claude Code included.

## Before starting

- **The KBD runtime still has `issue-195-closeout` as the active phase**, with its next step `/kbd-assess`. Starting R4 means parking or closing that phase first.
- **Under VLBV, R4 needs an adopted v2 contract before any generation.** It would be the first real v2 adoption, so the contract lint and closure packet from 1.15.0 get their first live run.

My recommendation is to do CR-33 first, since it's cheap, works on every harness and needs no evals. Then decide on CR-31 once you know which harnesses you actually plan to use. If that's only Claude Code and Codex, CR-31 shrinks to two profiles and becomes much more affordable.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 0ffe50ed-a878-4de2-b03a-ec3e58a38524
- Captured: 2026-11-16T06:29:21.800699Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
