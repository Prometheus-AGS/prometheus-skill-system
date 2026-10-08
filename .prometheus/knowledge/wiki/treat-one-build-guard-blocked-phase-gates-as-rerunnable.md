---
type: Lesson
id: treat-one-build-guard-blocked-phase-gates-as-rerunnable
title: Treat One-Build Guard BLOCKED Phase Gates as Rerunnable
tags:
- vis:project
- kbd:reflect
- phase:phase-team-learning-hardening
- phase-team-learning-hardening
- phase-gates
- build-guard
- blocked-status
- rerun-policy
- repo-workflow
links:
- karpathy-session-85bb63c70e5e4579
sources:
- id: lesson
  resource: learning:e0085ae85a309bb4
generated:
  by: pk/1.11.0
  at: 2026-10-05T07:08:00.727076+00:00
created_at: 2026-10-05T07:08:00.727076+00:00
updated_at: 2026-10-05T07:08:00.727076+00:00
revision: 0
content_hash: fa72f83297a8f5c07fc83fd02dd12b334c76e5b9da7dc645600d93377690def2
---

## Lesson

A one-build-at-a-time guard can block a phase gate because of builds outside the current session. A `BLOCKED` result from this guard is not evidence that the gate passed; treat it as a transient condition and rerun the gate after the external build clears.[^lesson]

## Operational Guidance

- Interpret `BLOCKED` as **rerun required**, never as pass/success.
- Before rerunning, check whether another build is currently holding the one-build-at-a-time guard.
- Preserve the distinction between:
  - **pass**: gate completed successfully;
  - **fail**: gate completed and found a problem;
  - **blocked**: gate could not execute because an external precondition was unavailable.
- This is consistent with gate semantics where blocked prerequisites must not be treated as successful completion, as captured in [Karpathy session 85bb63c70e5e](/karpathy-session-85bb63c70e5e4579.md).

[^lesson]: phase team-learning-hardening reflection lesson