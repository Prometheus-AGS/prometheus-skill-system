---
id: kbd-runtime
title: kbd-runtime
sidebar_label: kbd-runtime
---

# kbd-runtime

`substrate/kbd-runtime` is the canonical workflow-authority library shared by
`prometheus kbd` and Sovereign Sync.

## Responsibilities

- immutable `.prometheus/project.json` identity;
- signed and hash-chained canonical events;
- deterministic `KbdStateV2` replay;
- lifecycle, checkpoint, plan, phase, stage, change, and task transitions;
- independent completion dimensions;
- decisions, blockers, and device trust;
- command idempotency and optimistic revisions;
- single-writer leases, heartbeats, handoffs, and fencing;
- atomic compatibility projections;
- legacy-ledger migration with checksummed backups;
- non-authoritative shadow/canary rollout evidence.

## Security primitives

Events use canonical JSON, SHA-256 hash chaining, and Ed25519 signatures.
Interactive runtimes use the supported platform credential store; headless
voters require an explicit mode-`0600` device-key file. The loopback REST
bearer token is a separate project-scoped secret.

## Canonical runtime

The repository stores only the immutable project manifest and compatibility
projections. The event journal, locks, token, deferred hooks, and consensus
storage live in the platform application-data directory under the project
UUID.

## Public interfaces

The crate is not normally called directly by an operator. Use:

```bash
prometheus kbd --path "/path/to/project" status --json
```

or the Sovereign Sync REST/MCP surfaces.

Detailed runbooks:

- [Canonical control plane](/docs/kbd/control-plane)
- [Tokens and authentication](/docs/kbd/tokens-and-authentication)
- [Leases and handoffs](/docs/kbd/leases-and-handoffs)
- [Migration and rollout](/docs/kbd/migration-and-rollout)

Canonical source:
[`substrate/kbd-runtime`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/kbd-runtime).
