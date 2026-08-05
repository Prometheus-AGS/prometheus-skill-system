---
title: Security and trust boundaries
description: Threat model, capability attenuation, identities, component authorization, evidence integrity, and explicit non-claims.
---

# Security and trust boundaries

Prometheus Exec assumes generated code may be wrong, hostile, or simply over-authorized. It also assumes a caller can lose responses, a process can crash between durable steps, stored evidence can be tampered with, and a valid signature can still refer to the wrong request or artifact. The design addresses those cases with separate boundaries rather than one broad “trusted” flag.

## Threats in scope

- generated code reading or writing outside declared paths;
- ambient environment or credential discovery;
- undeclared network access;
- runaway time, output, memory, or artifact growth;
- request-ID reuse with different content;
- duplicate execution after response loss;
- forged, truncated, reordered, or cross-run events and receipts;
- substituted code, inputs, components, environment records, or artifacts;
- stale plugin authorization after rollback;
- unknown or mismatched remote endpoints and signing keys; and
- path traversal or symlink escape in portable evidence bundles.

## Authority is attenuated in layers

1. The signed request declares code, inputs, limits, capabilities, targets, and provenance.
2. Contract validation rejects malformed or internally inconsistent requests.
3. Baseline policy denies undeclared authority.
4. Cedar may further restrict the request but cannot broaden baseline authority.
5. The selected runtime exposes only its supported platform or typed capabilities.
6. Artifact collection accepts only bounded safe paths under the output namespace.

An unavailable sandbox or authorization source fails readiness or the operation. It never enables a direct fallback that emits the stronger receipt class.

## Separate identities answer separate questions

| Identity or trust source | Question it answers |
| --- | --- |
| Local Ed25519 device identity | Which configured host signed this request or receipt? |
| SSH-signed grant manifest | Did an approved operator grant this exact request/capability/purpose within the validity window? |
| Signed plugin trust store and generation | Are these exact component bytes and metadata authorized for estate Tier W? |
| Standalone or bundled exact pins | Did the embedding product compile or configure these exact component bytes? |
| Remote enrollment snapshot | Is this endpoint ID bound to this public signing key for this dispatch? |

Trust does not flow sideways. A device key cannot authorize a component, plugin trust cannot enroll a peer, and peer enrollment cannot broaden local execution authority.

## Evidence integrity

Code, inputs, streams, artifacts, and environments are addressed by SHA-256. Request and receipt signatures use canonical JSON. Events and receipts form immutable hash-linked segments. The service verifies the chain before appending or reading evidence.

A portable bundle uses relative indexed paths and exact size/hash declarations. Verification rejects traversal, absolute paths, symlinks, duplicates, missing content, identity mismatch, invalid signatures, and request/receipt disagreement.

## Private material stays local

- CLI and service identities are created atomically with mode `0600`.
- MCP tools never accept private key bytes.
- Embedded FFI and thin Tauri adapters never accept private key bytes.
- Portable bundles contain public verification identities only.
- Pairing tickets, group secrets, and unrelated service credentials do not belong in execution queues or receipts.

## What a receipt proves—and what it does not

A valid receipt proves the contract fields that were signed and verified: request identity, selected code and inputs, declared authority and limits, runtime evidence, outputs, terminal state, and signer.

It does not prove:

- that the generated algorithm is correct for an unstated business requirement;
- that a reviewer approved the result;
- that a service is installed or externally operated;
- that an unavailable platform has runtime evidence;
- that the entire conversation or native agent was deterministic; or
- that a successful one-shot operation is safe to promote into a persistent service.

Those claims require their own review, installation, deployment, or platform evidence.

## Operational security defaults

- Use the private Unix socket rather than loopback TCP.
- Keep state, identity, and service files user-private.
- Prefer the smallest named inputs and output ceiling.
- Deny network unless a future reviewed contract explicitly represents it.
- Verify the active signed plugin generation before Tier W execution.
- Preserve original receipts and immutable generation history during rollback.
- Run doctor read-only before repair; exclusions apply before check construction.
- Treat logs as diagnostics, not as a substitute for signed evidence.

Next: [Installation, doctor, and recovery](./installation-doctor-and-recovery.md).
