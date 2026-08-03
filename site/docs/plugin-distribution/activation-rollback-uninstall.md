---
title: Activation, rollback, and uninstall
description: Safe plugin lifecycle and collision handling.
---

# Activation, rollback, and uninstall

## Activate

```bash
node scripts/install-plugin-generation.js
node scripts/install-plugin-generation.js --verify
```

Installation stages with private permissions, validates all files and 14 projections, updates `previous`, and atomically switches `current`. A failed verification leaves the active generation unchanged.

## Roll back

```bash
node scripts/install-plugin-generation.js --rollback
node scripts/install-plugin-generation.js --verify
```

Rollback swaps `current` and `previous`, restores verified copy targets from the selected generation, and revalidates every receipt and stable dispatcher.

## Uninstall

```bash
node scripts/install-plugin-generation.js --uninstall
```

Uninstall removes only paths carrying Prometheus ownership/generation evidence. User-created collisions and unrelated skill directories remain untouched. Generation payloads and recovery records should be archived according to release policy before deliberate removal.

## Stale-cache removal

After activation, search configured hook paths, target symlinks, copy receipts, and service definitions for obsolete version directories. Remove only certified Prometheus-owned stale projections. Keep immutable release history and the active/previous generations.

