---
id: privacy-model
title: Privacy Model
sidebar_label: Privacy Model
---

# Replication privacy

Transport membership, project authorization and content classification are
separate decisions. Local-only surreal-memory has a never-synced adapter;
trusted/public domain labels are not permission to transmit arbitrary data.
Keep raw memory, credentials, session material and private keys local.

Default connected control uses a same-user Unix socket. Explicit loopback TCP
requires a private bearer-token file and does not enable the deprecated unsigned
mutation route. The MCP bridge has no TCP fallback. Pairing secrets require
private transfer, while KBD replica signing authority remains independently checked.

Optional DHT publication and iroh relay traffic are distinct configuration
choices. Disabling DHT alone is not a no-network/no-relay guarantee. See
[peer settings](/docs/sovereign-sync/p2p-network) and
[data scopes](/docs/sovereign-sync/data-scope).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
