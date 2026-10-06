---
title: Activation, rollback, and uninstall
description: Safe plugin lifecycle and collision handling.
---

# Activation, rollback, and uninstall

Complete all planned phase production before generation, verification or lifecycle integration. Use isolated homes and roots for gate evidence; these commands mutate installation state unless `--verify` is selected. Source implementation does not claim a current installed fix.

A nonempty inherited `CODEX_HOME` wins over `--home` for Codex. Unset or empty values use the selected home plus `.codex`. Keep that same effective root through install, rollback, verification and uninstall; logical signed receipts remain in the plugin store and do not substitute for inspection of the actual selected copy.

## Activate

```bash
node scripts/install-plugin-generation.js
node scripts/install-plugin-generation.js --verify
```

Installation stages with private permissions, verifies the manifest signature
against the plugin trust store, validates all files, the shared skill index, and
14 signed projections, updates `previous`, and atomically switches `current`. A
failed verification leaves the active generation unchanged.

## Roll back

```bash
node scripts/install-plugin-generation.js --rollback
node scripts/install-plugin-generation.js --verify
```

Rollback verifies `previous`, restores verified copy targets and the index from
that generation, revalidates every signed receipt and stable dispatcher, and
then swaps `current` and `previous`.

## Uninstall

```bash
node scripts/install-plugin-generation.js --uninstall
```

Uninstall removes only managed entries with ownership tied to known verified generations or applicable existing receipts. Marker syntax alone is insufficient. Unknown, malformed and unrelated copies are preserved with a diagnosis.
User-created collisions and unrelated skill directories remain untouched.
Generation payloads, trust policy, signatures, receipts, and recovery records
should be archived according to release policy before deliberate removal.

## Retention and pruning

Use the installer lifecycle's verified ownership and reference accounting when pruning generation history. Active and previous pointers, signed receipts, selected projections and copy ownership are references to preserve. Do not remove arbitrary cache versions or edit harness-managed caches as a troubleshooting shortcut. Retain recovery records until the owner's release policy authorizes disposal.
