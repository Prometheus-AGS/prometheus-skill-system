---
id: ag-ui-sse
title: AG-UI / A2UI SSE
sidebar_label: AG-UI SSE
---

# Connected event streams

`/api/v1/stream` is the AG-UI event surface. It is separate from the durable
signed-push event sequence at
`GET /api/v2/sync/pushes/{request_id}/events`. A progress event can report local
work without proving replication or a peer's application.

Resume push events with `after` or `Last-Event-ID` and the last actual sequence.
After response loss, look up the original receipt on the selected host before
new mutation. Do not substitute scaffold events, a fresh stream connection or
an empty event list for durable acceptance evidence. See
[request and receipt recovery](/docs/sovereign-sync/signed-pushes-and-receipts).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
