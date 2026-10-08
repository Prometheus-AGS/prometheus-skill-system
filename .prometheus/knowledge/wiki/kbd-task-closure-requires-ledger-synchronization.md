---
type: Lesson
id: kbd-task-closure-requires-ledger-synchronization
title: KBD Task Closure Requires Ledger Synchronization
tags:
- vis:project
- kbd:reflect
- phase:team-aware-learning-memory-impl
- kbd-ledger
- task-tracking
- phase-reflection
- workflow-integrity
- team-aware-learning-memory
links:
- phase-learn-grader-validation-reflect-status-2026-07-18-05-13-43
sources:
- id: lesson
  resource: learning:e5e11913d966b673
generated:
  by: pk/1.11.0
  at: 2026-10-04T22:37:01.507422+00:00
created_at: 2026-10-04T22:37:01.507422+00:00
updated_at: 2026-10-04T22:37:01.507422+00:00
revision: 0
content_hash: 54e22f845dedc0550a73796640f60274bce352a77e8216a419e7f2ada41e915d
---

## Lesson

`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger.[^lesson]

## Operational Guidance

- Do not rely on `kbd-apply mark-done` alone when closing phase tasks.
- Prefer the canonical task lifecycle commands:
  - `begin-task`
  - `end-task`
- If `kbd-apply mark-done` was used, reconcile the canonical KBD ledger before entering or completing `reflect`.

## Rationale

Reflect-stage status can become inconsistent if task flags show completion while the canonical ledger remains unsynchronized. This is especially relevant for phase tracking entries like [Phase Learn Grader Validation Reflect Status 2026-07-18 05:13:43](/phase-learn-grader-validation-reflect-status-2026-07-18-05-13-43.md), where completion state is inferred from tracked changes and ledger-backed progress.

[^lesson]: learning reflection note