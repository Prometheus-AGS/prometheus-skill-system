---
license: MIT
name: kbd-audit
version: '1.0.0'
argument-hint: '[--since <revision-or-event>]'
description: >
  Audit the exact KBD position, lifecycle history, plan revision, ownership,
  blockers, and uncommitted work without mutating the run.
metadata:
  tags: [process, orchestration, control, audit]
---

# /kbd-audit

Produce a read-only causal audit of the active KBD run.

## Progress Signals (MANDATORY)

Before reading state, emit:

```text
Starting kbd-audit — <phase-name>
```

After rendering the report, emit:

```text
Completed kbd-audit — <phase-name> at revision <n>
```

## Procedure

1. Prefer `prometheus kbd audit`, forwarding `--since` when supplied.
2. Report lifecycle status, plan revision, last completed work, exact next
   work, pause/block reason, decisions, actor/device provenance, and drift.
3. Include a read-only `git status --short` and diff summary; never include
   secrets or full transcripts.
4. If only legacy state exists, identify contradictory aliases, stale
   projections, malformed ledgers, and direct-writer risk explicitly.

This skill is read-only. Do not resume, migrate, repair, or execute work.
