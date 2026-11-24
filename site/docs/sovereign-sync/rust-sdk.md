---
id: rust-sdk
title: Rust SDK
sidebar_label: Rust SDK
---

# Companion Rust client

The `sovereign-client` crate belongs to Companion. A pack dependency on the old
`substrate/sovereign-client` path is invalid. Use the separately selected approved
Companion source identity and its client contract.

On Unix, `SovereignClient::unix_socket(path)` selects same-user control with
proxies/redirects disabled and a ten-second request timeout. The retained
`new(base_url)` constructor is a different HTTP client; do not assume it selects
the socket or supplies explicit TCP authentication.

`sync_push_signed` submits an already signed v2 request once and returns HTTP
status/body. Persist the exact request and endpoint before invoking it;
`sync_push_receipt` resolves its original ID. SDK callers do not automatically
receive the MCP bridge's durable intent outbox. Keep transport/decoding failure
uncertain until reconciled on the original host. The legacy unsigned helper is
same-user Unix-only and does not replace the v2 receipt contract.

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
