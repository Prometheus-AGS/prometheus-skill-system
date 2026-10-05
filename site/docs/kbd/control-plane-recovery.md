---
id: control-plane-recovery
title: Local Recovery and Refresh
sidebar_label: Local Recovery & Refresh
---

# KBD local recovery and refresh

The signed local runtime owns ordinary KBD lifecycle authority. An optional
replication service is separate; its availability cannot replace project identity,
causal revision, claims or task receipts.

## Preserve authority and evidence

1. Honor an operator pause and stop scheduling new work.
2. Preserve `.prometheus/project.json`, journals, checkpoints and task records.
3. Read canonical status and audit history before reconciling compatibility files.
4. Rebuild stale projections from the verified journal instead of editing their
   revision or inferring authority from file timestamps.
5. Resume through the canonical operator contract with the expected plan revision.

```bash
prometheus kbd --path "/path/to/project" status --json
prometheus kbd --path "/path/to/project" audit --json
```

See [Identity and authentication](./tokens-and-authentication) and
[Checkpoints and recovery](./checkpoints-compaction-recovery) for the precise
ownership and replay boundaries. Read-only inspection does not grant mutation rights.

## Refresh the installed pack

Finish all phase production work, then run the applicable local integration gates.
Serialize Cargo/rustc across the machine. Build and sign only the authorized native
artifacts, and generate the distribution once at the final boundary. Install the
verified generation through the selected profile and targets; preserve custom
harness configuration and the previous recoverable generation.

Follow [Installation and upgrades](/docs/operations/installation-and-upgrades) for
the actual installer interface and [Service operations](/docs/guide/service-operations)
for service owners, platforms, ports and data. An installed binary, loaded service,
reachable endpoint and successful task are separate evidence.

The skill pack does not build, repair or start Companion's replication daemon.
There is no pack `--sharing` install contract. Use the retained
[Companion boundary](/docs/sovereign-sync/overview) when replication is explicitly
required; ordinary local KBD work remains independent.

## Memory and hook recovery

Inspect learning queue state and reconcile uncertain submissions by their existing
operation identity and hash. Never discard records or invent replacement identities
to make diagnosis green. Restore a complete immutable snapshot from its verified
manifest or publish a new one from authoritative knowledge records.

Hooks are bounded adapters. They can emit context, queue observations and check
canonical task completion, but do not intercept Bash, Python, Edit or Write.
[Hooks and waypoints](./hooks-and-waypoints) describes the task-receipt boundary.
