---
title: Logs, recovery, and common failures
description: Owner-only logs, rotation, rollback, and evidence-preserving recovery.
---

# Logs, recovery, and common failures

Hook and worker logs contain operational metadata and must remain owner-only. The managed log-rotation definition and rendered configuration are part of doctor evidence. Certification proves the service definition is installed, loaded, rotates the configured file, and preserves mode `0600`.

## Recovery order

1. Stop new submissions at the caller boundary.
2. Preserve queue records, operation receipts/events, snapshots, plugin pointers, and logs.
3. Check `/health` and `/ready` separately.
4. Reconcile `submitting` and `accepted` memory records by operation ID.
5. Restore worker service state and let persisted plans resume.
6. Verify current snapshot pointers and active plugin generation.
7. Run read-only doctors, then the mutating receipt certification if authorized.

## Common failures

| Failure | Meaning | Recovery |
| --- | --- | --- |
| `/health` 200, `/ready` 503 | Process live; ledger unavailable | Repair storage/coordinator before writes. |
| Receipt remains non-terminal | Work is blocked, interrupted, or waiting | Inspect `blocked_by`, executor fields, and ordered events. |
| Worker service loaded but queue grows | Worker cannot claim or reconcile | Inspect queue ownership, logs, binary signature, and Memory readiness. |
| Plugin verify finds stale path | Hook/target bypasses `current` | Restore stable dispatcher or verified target receipt. |
| Snapshot pointer invalid | Atomic publication did not complete or path was altered | Repoint to a complete generation or republish. |
| Doctor warning | Optional/degraded surface | Record explicit release disposition; never relabel as green. |

Do not erase recovery records to make a doctor green. Repair the active pointer or service state while preserving published generations, terminal receipts, and command evidence.

