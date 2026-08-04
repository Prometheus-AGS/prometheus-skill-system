---
title: Immutable plugin generations
description: Content-addressed packaging, manifests, activation, and rollback pointers.
---

# Immutable plugin generations

The plugin installer never edits an active payload. It stages a complete
content-addressed generation, verifies its Ed25519-signed canonical manifest and
every signed target receipt against a separate trust store, then atomically
switches `current`.

```mermaid
flowchart LR
  Source["Certified repository"] --> Stage["Private staging directory"]
  Stage --> Hash["Canonical manifest + skill index"]
  Hash --> Sign["Ed25519 signature + trust verification"]
  Sign --> Generation["generations/hash"]
  Generation --> Receipts["Validate 14 target receipts"]
  Receipts --> Previous["previous pointer"]
  Previous --> Current["Atomic current switch"]
  Current --> Stable["Stable dispatchers"]
  Previous -. rollback .-> Current
```

The manifest records the generation, complete file inventory, modes, content
hashes, target payload modes, required stable entrypoints, canonical skill-index
receipt, source commit/tree state, and external gitlink commit pins. A generation
is active only when signature, trust, provenance, payload, index, and receipt
verification succeeds.

`current` points to the active payload and index as one transaction. `previous`
preserves the last certified generation for rollback. Hardcoded version
directories are forbidden because they bypass content verification and become
stale after upgrades.

The host, generated-agent, and mobile index projections are byte-identical and
use the shared deterministic selector. See [Signing, indexes, and receipts](./signing-index-and-receipts)
for the full trust and parity contract.

Verify without mutation:

```bash
node scripts/install-plugin-generation.js --verify
```
