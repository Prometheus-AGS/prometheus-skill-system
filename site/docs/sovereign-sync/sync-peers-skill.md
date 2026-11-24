---
id: sync-peers-skill
title: /sync-peers Skill
sidebar_label: /sync-peers
---

# Sync peers skill

Companion's `/sync-peers` instruction reads MCP `sync-peers` from the selected
host. It lists peers and transport state; it does not enroll, admit, restart or
authorize a peer. A listed peer is not evidence it applied a signed project push.

Use the separately authorized [pairing procedure](/docs/sovereign-sync/pair-two-machines)
for membership changes. Keep the same host socket and its configured identity,
and retain uncertainty rather than switching endpoints after a failed observation.

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
