---
id: sync-status-skill
title: /sync-status Skill
sidebar_label: /sync-status
---

# Sync status skill

The `/sync-status` instruction belongs to Companion's separate connected-skill
package. It uses MCP `sync-status` against the selected host and preserves its
raw transport/domain state. Optional `domain` annotates the request; it does not
prove convergence. Unreachable host is unavailable, not an empty ready network.

Read transport attempt, last error, next retry and peer IDs. Listener health,
local authority readiness, network readiness and destination application are
separate observations. Installing either skill pack or registering MCP does not
start or certify the Companion host. See [registration](/docs/sovereign-sync/installation).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
