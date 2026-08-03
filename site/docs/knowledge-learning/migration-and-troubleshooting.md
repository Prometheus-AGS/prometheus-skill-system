---
title: Migration and troubleshooting
description: Reconcile legacy queues, rebuild snapshots, and diagnose deterministic learning.
---

# Migration and troubleshooting

## Legacy migration

Before enabling the worker, inventory legacy queue directories and synchronous hook integrations. Preserve every record. Migrate each record to a stable operation identity, then reconcile it against the v2 ledger:

- no receipt: publish to `memory/pending`;
- matching non-terminal receipt: move to `memory/accepted` and continue reconciliation;
- matching terminal receipt: move to `completed` or `rejected`;
- different hash for the same ID: quarantine as rejected and retain both hashes.

Never delete `retry` or `dead-letter` evidence until every record has an explicit disposition.

## Snapshot continuity

If `current` is missing or invalid, do not edit it in place. Validate the latest complete immutable generation and atomically repoint `current`, or publish a new generation from authoritative knowledge records. Keep the old generation for audit.

## Common failures

| Symptom | Evidence to inspect | Correct action |
| --- | --- | --- |
| Hook latency | hook log and pending publication time | Confirm the hook only enqueues; move expensive work to the worker. |
| `processing` never clears | worker service state and record ownership | Reconcile stale claim using its operation receipt. |
| `submitting` after crash | v2 GET receipt | Reuse ID/hash; never generate a replacement ID. |
| Snapshot doctor failure | scope `current`, manifest, generation hash | Restore or republish a complete generation atomically. |
| `409` from Memory | stored and submitted payload hashes | Quarantine; determine which caller reused the ID incorrectly. |
| Empty `pk lint` output | command exit status and stderr | Treat as warning, never as OK. |

Logs are owner-only and rotated by the managed hook log-rotation service. See [Logs, recovery, and failures](../operations/logs-recovery-and-failures.md).

