---
title: Dynamic Operations use-case cookbook
description: Concrete patterns for generated scripts, portable components, remote jobs, recovery, and evidence-producing workflows.
---

# Dynamic Operations use-case cookbook

Choose Prometheus Exec because a bounded result needs proof, not because the system happens to contain code. Each pattern below names the program form, tier, evidence value, and the point where a different Prometheus feature is the better choice.

## Generated data transformation

**Problem:** An agent generates a transform for customer-supplied JSON or CSV and duplicate execution could create inconsistent reports.

**Use:** Tier P Python or Node with named read-only inputs, a short wall limit, no network, and declared output artifacts.

**Evidence:** request/code/input hashes, interpreter and sandbox profile, stdout/stderr, report digest, terminal state, and device signature.

**Do not use:** a generated native agent unless users or peers need a persistent API around the transformation.

## Repository or artifact analysis

**Problem:** A generated script must inspect an exported source snapshot or build artifact and produce a machine-readable finding set.

**Use:** Tier P with the snapshot as a named input. Write findings below `PROMETHEUS_OUTPUT_DIR`; keep repository mutation outside the operation.

**Evidence:** the analysis is bound to exact input bytes and can be reviewed later without trusting a pasted console summary.

**Do not use:** Tier P to edit the live repository. Ordinary agent tools remain the correct authoring interface; use the operation for bounded analysis or plan generation.

## Migration planning before mutation

**Problem:** A schema, configuration, or content migration needs a deterministic plan before an operator applies changes.

**Use:** Tier P to read exported state and emit a proposed migration plus risk report. Review the signed evidence, then apply through the system that owns the data.

**Evidence:** exact source snapshot, migration algorithm, proposed output, warnings, and limits.

**Do not use:** an execution receipt as authorization to mutate an external production system.

## Portable graph or document optimization

**Problem:** The same pure or capability-bounded algorithm should run on desktop, embedded hosts, and later portable verification.

**Use:** Tier W with explicit input/output, clock, random, log, and K/V capabilities. Authorize exact component bytes through signed plugin distribution or exact pins.

**Evidence:** component authorization, engine version, capability values, artifacts, and backend-independent deterministic projection.

**Reference:** the released `entity-graph-optimize` component.

## Expensive operation with response-loss risk

**Problem:** A caller may disconnect after acceptance and cannot safely retry work blindly.

**Use:** REST or MCP with a caller-controlled request ID and issued-at value. On reconnect, resubmit the same canonical payload or resume events after the last durable sequence.

**Evidence:** same-ID/same-hash replay returns the original run; a changed payload produces a conflict instead of a duplicate.

## Offline audit package

**Problem:** A reviewer cannot access the execution host or daemon.

**Use:** package the signed request and receipt, public identity, environment record, and every receipt-referenced artifact through a relative-path evidence index.

**Evidence:** `verify-bundle` checks hashes, paths, identity, signatures, request binding, and artifact completeness without network or daemon state.

## Persistent native agent with evidenced sub-jobs

**Problem:** A long-lived research or operations agent needs a UI, model policy, A2A endpoint, and scheduling, but some calculations must be independently verifiable.

**Use:** generate and deploy the native agent normally. Add an explicit local adapter that turns only the bounded sub-job into a Tier P or Tier W request. Store the returned run ID and receipt reference in the agent's domain state.

**Evidence:** the native agent owns conversation and orchestration; Prometheus Exec proves the bounded calculation. Neither receipt claims that the whole model conversation was deterministic.

## Remote estate fan-out

**Problem:** One signed operation must reach several enrolled machines that may be offline or slow.

**Use:** Tier R to persist one per-target dispatch, validate enrollment and expiry, deliver to each target's local facade, and store independently signed peer responses.

**Evidence:** mixed target states remain visible; a slow or unavailable target cannot turn another target's receipt into a synthetic aggregate success.

**Boundary:** disposable peer fixtures exist; a production transport adapter remains pending evidence.

## Reusable procedure without runtime evidence

**Problem:** Agents keep rediscovering the same steps, but the work does not require a constrained receipt-producing runtime.

**Use:** create or update a skill. Include scripts when helpful, and let ordinary agent tools execute them.

**Do not use:** Prometheus Exec simply to make the procedure feel more formal.

## Long-lived service or interactive application

**Problem:** The product must remain available, accept requests, route models, expose UI, or communicate over A2A.

**Use:** `/create-native-agent` or the normal application toolchain.

**Do not use:** Prometheus Exec as a process supervisor. Operations are terminal and bounded; services have lifecycle, availability, upgrades, and network responsibilities.

## Selection checklist

Before submitting an operation, answer:

1. What exact code and inputs are being bound?
2. Why is ordinary tool execution insufficient?
3. Is the program eligible for Tier P or Tier W?
4. What authority is declared, and what remains denied?
5. Which outputs must be durable artifacts?
6. What limit makes failure bounded and understandable?
7. Who will verify the receipt, and with which public material?
8. Does the evidence support only this operation, or is a broader deployment claim being inferred incorrectly?

Next: [Security and trust](./security-and-trust.md).
