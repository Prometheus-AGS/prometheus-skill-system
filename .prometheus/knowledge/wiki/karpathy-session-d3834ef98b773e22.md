---
type: SessionRecord
id: karpathy-session-d3834ef98b773e22
title: Karpathy session d3834ef98b77
tags:
- karpathy
- session-learning
sources:
- resource: session:0ffe50ed-a878-4de2-b03a-ec3e58a38524
generated:
  by: pk/1.11.0
  at: 2026-11-16T07:57:02.388600+00:00
created_at: 2026-11-16T07:57:02.388600+00:00
updated_at: 2026-11-16T07:57:02.388600+00:00
revision: 0
---

## Delta

I've fixed all seven findings from the glm-5.3 review (3 CRITICAL, 4 WARNING), and the contract passes the adoption lint again. The fixes:

- **Adoption first:** the plan now adopts the contract before any task starts.
- **Every machinery case has its own negative control:** each of the six gets a check where its trigger is removed and the real script must accept, so a script that always refuses can't pass.
- **Sample counts:** the 17 agent cases get 3 samples each, and the 6 machinery cases run once and carry the same verdict into every report. That puts spend at 255 harness sessions, not the 345 I quoted earlier.
- **Uninstalled harnesses** report the agent cases as not run and still carry the machinery verdicts.
- **Claude Code login:** the safety check now also compares a hash of the login-keychain item list before and after, because Claude Code's login lives in the macOS keychain and a file check alone can't see it.
- **Negative-control output** goes to `evals/reports/_negative/`, so it can't overwrite the real reports.
- **Capability evidence** must come from a case that exercises that same capability, with a negative control that cites the wrong case.

The second review round is running in the background. When it's back, I'll register the tasks and bring you the contract to adopt. Nothing has been implemented or committed yet.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 0ffe50ed-a878-4de2-b03a-ec3e58a38524
- Captured: 2026-11-16T07:56:52.509761Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
