# Reconcile KBD task state

## Why
Reconciliation can report clean when task files are missing and can repair the active phase instead of an explicit target. Reflect also needs inspection of archived completed changes.

## What Changes
- Read-only reconciliation of OpenSpec, native KBD, and Spec Kit tasks, canonical identities, and phase counters.
- Distinguish clean, drift, and incomplete scans; restrict repair to unambiguous active-phase work.
- Update apply/reflect instructions and generated payloads; patch full pack to 1.12.2 and process plugin to 1.7.3.

## Capabilities
### New Capabilities
- `kbd-task-reconciliation`: task/ledger comparison and bounded repair.
### Modified Capabilities
None.

## Impact
KBD apply/reflect drivers and distributable metadata. No BAUAR state mutation or new runtime dependencies.
