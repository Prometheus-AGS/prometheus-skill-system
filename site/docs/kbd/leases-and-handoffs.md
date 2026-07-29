---
id: leases-and-handoffs
title: Leases & Handoffs
sidebar_label: Leases & Handoffs
---

# Leases and Handoffs

KBD permits one mutation owner at a time. The lease is committed through the
control plane and contains:

- scope;
- lease UUID;
- owner actor, device, harness, and session;
- monotonically increasing fencing token;
- acquisition, heartbeat, and display-expiry timestamps.

The default TTL is 90 seconds. Active writers heartbeat every 30 seconds.
Wall-clock expiry is audit metadata, not permission to self-promote: takeover
is authorized by the consensus leader’s monotonic grace rule or by an explicit
operator-forced claim.

## Inspect ownership

```bash
PROJECT_ROOT="/path/to/project"

prometheus kbd --path "$PROJECT_ROOT" status
prometheus kbd --path "$PROJECT_ROOT" status --json |
  jq '{revision, lifecycle, lease, lastFencingToken}'
```

## Claim for Claude Code

Claude Desktop coding sessions use the harness ID `claude-code`:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" claim
```

Verify:

```bash
prometheus kbd --path "$PROJECT_ROOT" status --json |
  jq '.lease.owner.harness, .lease.fencingToken'
```

Expected owner:

```json
"claude-code"
```

The claim does not make a paused, blocked, completed, cancelled, or failed
lifecycle writable. Ownership and lifecycle are independent gates.

## Claim for another harness

```bash
PROMETHEUS_HARNESS=codex \
  prometheus kbd --path "$PROJECT_ROOT" claim

PROMETHEUS_HARNESS=opencode \
  prometheus kbd --path "$PROJECT_ROOT" claim
```

An ordinary claim fails if a lease already exists. `--force` is an audited
operator takeover and should be used only after confirming the previous writer
is no longer active:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" claim --force
```

## Heartbeat and release

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" heartbeat

PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" release
```

Heartbeat and release require the current lease ID and fencing token. The CLI
reads them from committed state and submits a typed command.

## Atomic handoff

Handoff changes the owner and increments the fence in one committed event:

```bash
# Run as the current owner
PROMETHEUS_HARNESS=codex \
  prometheus kbd --path "$PROJECT_ROOT" handoff --to claude-code
```

The target is recorded with wildcard device/session values so the receiving
Claude Code session can continue on a trusted device after the event
replicates.

Recommended flow:

1. checkpoint or pause if work is mid-operation;
2. hand off to the exact harness ID;
3. verify lease ID changed and fencing token increased;
4. open/resume in the receiving harness;
5. begin 30-second heartbeats while it remains writer.

Ownership is advisory: since the pre-mutation fence was removed, a non-owner
harness is no longer prevented from editing files. The lease and fencing token
still record who holds write authority and still reject stale *control-plane
commands*, but they no longer gate tool calls. See [Tool guards](./bash-mutation-guard).

## Why fencing matters

Suppose Codex owns fence `8`, loses connectivity, and Claude Code takes over at
fence `9`. A delayed Codex command still carries fence `8`; the control plane
rejects it even if the old lease’s timestamp appears plausible. This prevents
split-brain writes after partitions, sleep, or stale sessions.

## CLI identity detection

The CLI resolves harness identity in this order:

1. `PROMETHEUS_HARNESS`;
2. `CODEX_THREAD_ID` → `codex`;
3. `CLAUDE_SESSION_ID` → `claude-code`;
4. fallback → `cli`.

For manual terminal operations, set `PROMETHEUS_HARNESS` explicitly so the
lease owner matches the adapter that will perform mutations.
