---
title: Receipts, verification, and certification
description: Signed receipt contents, offline checks, portable evidence bundles, and evidence-status semantics.
---

# Receipts, verification, and certification

A terminal receipt answers five concrete questions: which signed request ran, which code and inputs it bound, which authority and limits were enforced, which bytes came out, and which device signed the result.

## Receipt anatomy

`ExecutionReceipt` includes:

- run ID, canonical request hash, terminal state, tier, and evidence class;
- code, input-set, environment, toolchain, and sandbox-profile hashes;
- backend, exit status or trap, stable failure details, and resource usage;
- SHA-256 references for stdout, stderr, and declared artifacts;
- executing-device key ID, signature algorithm, and signature;
- grants used by Tier P; and
- component authorization, engine version, backend profile, and deterministic projection for Tier W.

Tier P receipts are `attested`; Tier W receipts are `verified`. A successful Tier W receipt must contain component provenance. A rejected or interrupted pre-execution Tier W receipt must name the failure rather than pretending a component ran.

```mermaid
flowchart TD
  Request["Signed request and canonical hash"] --> Ledger["Durable acceptance"]
  Ledger --> Evidence["Streams, artifacts, and environment records"]
  Evidence --> Receipt["Signed terminal receipt"]
  Receipt --> Index["Portable evidence index with relative paths and hashes"]
  Index --> Offline["Offline public-key verification"]
  Offline --> Certification{"Requirement evidence properties satisfied?"}
  Certification -->|"yes"| Complete["completed"]
  Certification -->|"environment unavailable"| Pending["pending_evidence"]
  Certification -->|"judge unavailable"| Review["pending_review"]
  Certification -->|"requirement violated"| Failed["failed or blocked"]
```

## Offline verification

Verify one receipt and its referenced artifacts without starting a service:

```bash
prometheus-exec verify \
  --receipt ./receipt.json \
  --request ./request.json \
  --public-key '<unpadded-base64url-public-key>' \
  --artifacts ./bundle \
  --format json
```

For Tier W, add `--component ./skill.wasm` and the exact named `--input NAME=PATH` values to re-execute under the portable Pulley profile. Verification first checks the signature/request binding, then compares state, output and artifact hashes, failure, authorization, engine version, and deterministic projection. Timestamps, measured usage, and backend-specific profile identity are not falsely required to match across backends.

Verify an indexed package with no network or daemon state:

```bash
prometheus-exec verify-bundle \
  --index ./execution-evidence.json \
  --root ./bundle \
  --format json
```

The checker rejects absolute or traversing paths, symlinks, duplicate entries, size/hash mismatches, identity mismatch, missing artifacts or environments, and invalid receipt/request binding.

## Certification semantics

Certification is method-independent. A requirement declares evidence properties; any producer may satisfy them with independently verifiable material. `prometheus-exec` is one evidence producer, not a mandatory gate on creative Bash or Python work.

Statuses stay separate:

- `completed`: the named evidence properties verify;
- `pending_evidence`: a required environment, peer, or physical device was unavailable;
- `pending_review`: the distinct review judge was unavailable;
- `blocked`: a known requirement is not satisfied, such as the mobile size budget; and
- `failed`: supplied evidence was present but invalid.

Artifact/source, disposable runtime, installed host, remote deployment, mobile size, physical device, and judge review are independent dimensions. A green source build cannot imply an installed or externally operated service.

Next: [Installation, doctor, and recovery](./installation-doctor-and-recovery.md).
