---
name: entity-realtime-channel
description: >
  Add or adjust RealtimeManager channel subscriptions: ChannelConfig per EntityType, optional normalize on
  register(), SubscriptionConfig label and replayOnConnect, and affectedListKeys in ChangeSet for list refresh.
license: MIT
metadata:
  version: '1.0.0'
  tags: "react, typescript, entity-management"
---

# `/entity-realtime-channel` — Channels and normalization

## When to use

- New table/topic/room must stream updates into existing manager setup
- Payload shape differs per table → dedicated **`normalize`** mapper
- Server can hint list invalidation → populate **`affectedListKeys`**

## `ChannelConfig`

| Field | Role |
|-------|------|
| **`type`** | `EntityType` string for graph writes |
| **`filter`** | Opaque record for adapter-specific subscription narrowing |
| **`id`** | Optional entity scope |
| **`operations`** | Optional subset of `insert` \| `update` \| `delete` \| `upsert` |

## Registration

```ts
import type {
  ChannelConfig,
  EntityChange,
  RealtimeAdapter,
  RealtimeManager,
} from "@prometheus-ags/prometheus-entity-management";

declare const manager: RealtimeManager;
declare const adapter: RealtimeAdapter;

const normalizeRawEvent = (raw: unknown): EntityChange | null =>
  raw && typeof raw === "object" ? (raw as EntityChange) : null;

const channels: ChannelConfig[] = [
  { type: "Task", filter: { status: "open" } },
  { type: "Comment" },
];
manager.register(adapter, channels, normalizeRawEvent);
```

- **`normalize`**: `(raw: unknown) => EntityChange | null` — drop heartbeats, unknown tables
- Adapters call the handler with **`ChangeSet`**; manager coalesces **`changes`** arrays

## `ChangeSet` extras

- **`affectedListKeys`**: string keys matching graph list invalidation predicate — use when totals/order change without per-row payloads
- **`timestamp`**: optional ISO string for logging

## `SubscriptionConfig`

Passed internally as `{ label: `${adapter.name}/${channel.type}`, replayOnConnect: true }`. Custom adapters should honor **replay** semantics if the vendor supports catch-up.

## Cleanup

Adding channels often means **new** `register` call — prefer **one** `register` per adapter with full channel array, or **`unregister(name)`** then re-register to avoid duplicate handlers.

## Playbook

**`agents/channel-configurator.md`**

## Parent skill

**`prometheus-entity-skills/entity-graph-realtime/SKILL.md`**
