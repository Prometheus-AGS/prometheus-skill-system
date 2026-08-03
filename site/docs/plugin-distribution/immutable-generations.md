---
title: Immutable plugin generations
description: Content-addressed packaging, manifests, activation, and rollback pointers.
---

# Immutable plugin generations

The plugin installer never edits an active payload. It stages a complete content-addressed generation, validates its manifest and every target receipt, then atomically switches `current`.

```mermaid
flowchart LR
  Source["Certified repository"] --> Stage["Private staging directory"]
  Stage --> Hash["Canonical manifest + SHA-256 generation"]
  Hash --> Generation["generations/hash"]
  Generation --> Receipts["Validate 14 target receipts"]
  Receipts --> Previous["previous pointer"]
  Previous --> Current["Atomic current switch"]
  Current --> Stable["Stable dispatchers"]
  Previous -. rollback .-> Current
```

The manifest records the generation, complete file inventory, modes, content hashes, target payload modes, and required stable entrypoints. A generation is active only when verification succeeds.

`current` points to the active generation. `previous` preserves the last certified generation for rollback. Hardcoded version directories are forbidden because they bypass content verification and become stale after upgrades.

Verify without mutation:

```bash
node scripts/install-plugin-generation.js --verify
```

