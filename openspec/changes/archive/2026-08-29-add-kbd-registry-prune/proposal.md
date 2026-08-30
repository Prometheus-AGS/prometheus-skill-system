## Why

Removed worktrees remain in the machine KBD registry forever, causing sovereign-sync to replay unavailable authorities and emit the same warnings on every supervised restart. Cleanup currently requires unsafe manual JSON editing and has no rollback evidence.

## What Changes

- Add a registry API that inventories registrations whose paths no longer exist.
- Add dry-run-by-default `prometheus kbd projects --prune-missing` behavior with an explicit `--apply` gate.
- Under the registry lock, write a timestamped backup, checksum, and receipt before removing only missing path registrations.
- Preserve project runtime directories and all registrations whose paths still exist.
- Return exact removed entries and prove dry-run, apply, idempotence, and recovery evidence with focused tests.

## Capabilities

### New Capabilities

- `kbd-registry-maintenance`: Explicit, evidence-preserving maintenance for missing KBD replica registrations.

### Modified Capabilities

None.

## Impact

Affects `substrate/kbd-runtime` registry ownership, the `prometheus kbd projects` CLI, focused Rust tests, and operator documentation. It does not delete retained runtime data or auto-prune during daemon startup.
