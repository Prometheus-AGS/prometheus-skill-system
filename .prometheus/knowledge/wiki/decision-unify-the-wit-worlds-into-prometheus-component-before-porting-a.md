---
type: Decision
id: decision-unify-the-wit-worlds-into-prometheus-component-before-porting-a
title: "Decision: unify the WIT worlds into `prometheus:component/*` before porting any skill"
tags:
- decision
- outcome-pending
outcome_status: pending
decided_at: 2026-07-31T08:52:33Z
links: []
sources: []
---

# Decision: unify the WIT worlds into `prometheus:component/*` before porting any skill

## Decision

Define one WIT package family, **`prometheus:component/*`**, and settle it
**before a single skill is ported to WASM**. UAR's and KnowMe's existing worlds
become views onto it rather than independent contracts.

## Assumptions

(none stated)

## Falsifier

(none stated)

## Outcome

**Status: pending.** Nothing has been recorded yet.

A decision without a recorded outcome cannot be checked against what actually
happened — and idea rankings are known to flip after execution, so the judgement
made here is exactly the thing that needs checking later.

Record it with:

```
decision-log.sh outcome --id decision-unify-the-wit-worlds-into-prometheus-component-before-porting-a --result -
```
