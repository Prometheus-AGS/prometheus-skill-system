---
id: data-scope
title: Exactly What Syncs
sidebar_label: Exactly What Syncs
---

# Replication domains and ownership

The current host metadata labels `skill-index` public, `learner-model` and
`kbd-control` trusted, and `surreal-memory` local-only with a never-synced adapter.
These describe policy and wiring, not completed replication. Never push local-only
memory payloads or treat a project ID as permission to share its data.

Full native memory uses `memory/mcp`; mini Compose uses `memory/main_local_384`.
Companion service adoption or peer configuration does not merge or migrate them.
Git carries reviewed source/specification documents. Raw memory, enrolled private
keys, session credentials and transcripts are not portable authority.

Choose an implemented syncable domain, authorized project/replica and target,
then preserve its exact request, frontier and durable receipt. Destination
application needs its own actual evidence.

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
