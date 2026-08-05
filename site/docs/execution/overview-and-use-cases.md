---
title: Execution overview and use cases
description: Where Prometheus Exec fits, what evidence it produces, and when to use each execution tier.
---

# Execution overview and use cases

`prometheus-exec` turns a bounded code run into a signed, portable evidence bundle. The durable result identifies the exact request, code, inputs, limits, sandbox profile, outputs, runtime measurements, executing device, and terminal state. Output streams and declared artifacts live in a SHA-256 content-addressed store and are independently checkable against the receipt.

The service is optional. It does not intercept or restrict an agent's ordinary Bash, Python, Edit, or Write tools. Use it when a run needs reproducibility, durable replay, constrained capabilities, or evidence that can be verified away from the machine that executed it.

## Pick the execution form by the proof you need

| Need | Form | Result |
| --- | --- | --- |
| Run generated Python, Node, or Bash with OS isolation | Tier P sidecar | Attested receipt from the host's process sandbox |
| Run a portable component with deterministic host capabilities | Tier W sidecar or embedded API | Verified receipt with component authorization and backend-independent projection |
| Embed execution in a desktop or mobile host | Standalone or bundled-mobile Tier W | Private local ledger, CAS, events, receipts, and offline verification |
| Deliver one signed request to enrolled remote targets | Tier R dispatch kernel | Per-target state and independently verified peer receipts |
| Check evidence without executing or contacting a daemon | `verify` or `verify-bundle` | Cryptographic, request, artifact, and environment checks |

## Concrete use cases

### Generated data transformation

An agent generates a Python transform for a supplied JSON dataset. Tier P admits only declared inputs and output paths, clears the ambient environment, denies networking, applies the wall/stream/artifact limits, and records the exact interpreter identity. A retry with the same request ID and canonical hash returns the original durable run instead of starting a second process.

### Deterministic graph optimization

The released `entity-graph-optimize` WebAssembly component runs under Tier W with typed `input`, `output`, `log`, `kv-store`, `clock`, and `random` capabilities. The signed plugin generation pins component SHA-256 `ba438895404a23985d5226735b8f362cf3e8044894a1140852ba0992f2fdbe78`, world `prometheus:component@0.1.0`, and the fixed WASI 0.2.9 adapter imports. Cranelift and Pulley can produce the same deterministic receipt projection even though their backend profiles differ.

### Response-loss reconciliation

If a caller loses the HTTP, MCP, embedded, or remote response after durable acceptance, it reads status and events using the original ID. Accepted requests, spawn boundaries, ordered events, receipt-log entries, and terminal state are persisted in an order that prevents a successful run from disappearing or being executed twice.

### Portable certification evidence

An evidence index packages the signed request and receipt, public verification identity, environment record, and every referenced artifact by relative path and hash. `prometheus-exec verify-bundle` checks that package without a daemon or network. Certification evaluates the declared evidence properties; it does not require the evidence producer to be `prometheus-exec`.

## What is certified today

On the release Mac, source/artifact and disposable local runtime checks have evidence; installer and installed-host contracts have disposable fixtures, while the final machine install is recorded only at phase close. The remote protocol kernel has disposable-peer fixtures, but a production remote transport deployment remains `pending_evidence`. Mobile cross-builds exist, but the release-size requirement is blocked and physical-device runtime remains `pending_evidence`. See [Platform and evidence status](./platform-and-evidence-status.md) for the exact boundaries.

Next: [Architecture and execution tiers](./architecture-and-tiers.md).
