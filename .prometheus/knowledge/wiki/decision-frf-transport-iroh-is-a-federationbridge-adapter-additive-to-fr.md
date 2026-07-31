---
type: Decision
id: decision-frf-transport-iroh-is-a-federationbridge-adapter-additive-to-fr
title: Decision: `frf-transport-iroh` is a `FederationBridge` adapter, additive to FRF
tags:
- decision
- outcome-pending
outcome_status: pending
decided_at: 2026-07-31T08:52:34Z
links: []
sources: []
---

# Decision: `frf-transport-iroh` is a `FederationBridge` adapter, additive to FRF

## Decision

Add iroh to `flint-realtime-fabric` as a **new crate implementing the existing
`FederationBridge` trait**, beside the bridges already there. No change to FRF's
architecture, no modification of existing crates.

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
decision-log.sh outcome --id decision-frf-transport-iroh-is-a-federationbridge-adapter-additive-to-fr --result -
```
