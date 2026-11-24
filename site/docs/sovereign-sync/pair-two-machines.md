---
id: pair-two-machines
title: Pair Two Machines
sidebar_label: Pair Two Machines
---

# Pairing two selected hosts

Choose the configuration and identity on each device; retain separate private
signing keys. Network membership and KBD project/replica enrollment are separate.
From the selected Companion binary, A exports with
`sovereign-sync --config /absolute/config.toml --mode pair-export`.
Transfer its secret ticket privately to B; B imports with `--mode pair-import`
and `--ticket <private-ticket>`, then exports its updated ticket back to A for
import. This round trip keeps the group secret shared.

Import replaces the selected identity's group secret and updates its allowlist;
back up existing membership first. Ticket arguments may appear in shell history
or process listings. Never put tickets in shared logs, issues or transcripts.

Add each authorized endpoint ID to the other configuration's `[peers] bootstrap`
list, then restart the one selected host to read identity/config changes.
mDNS supplies addresses for known IDs. The source provides a pairing URI payload,
not a shipped QR screen, short-code flow or group UI. Neither pairing nor
connectivity proves an authorized signed push was applied at the destination.

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
