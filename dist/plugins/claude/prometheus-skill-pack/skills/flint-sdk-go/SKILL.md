---
name: flint-sdk-go
description: >
  Install and use the Flint Realtime Fabric Go SDK (github.com/prometheusags/frf/sdks/go).
  Covers SpineClient setup, channel subscriptions, event publishing, ack handling, and
  Connect-RPC transport configuration for Go services.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, go, golang, sdk, grpc, connect-rpc]
---

# flint-sdk-go

Use the **Flint Realtime Fabric** Go SDK to subscribe to channels, publish events, and handle acknowledgements in Go services.

## When to use

- Adding realtime event streaming to a Go microservice or daemon.
- Consuming FRF channels via gRPC/Connect-RPC in Go.
- Building a Flint adapter or bridge in Go.

## Installation

```bash
go get github.com/prometheusags/frf/sdks/go@latest
```

Requires Go 1.25+.

## Module path

```
github.com/prometheusags/frf/sdks/go
```

## Core packages

| Package | Purpose |
|---------|---------|
| `github.com/prometheusags/frf/sdks/go/client` | `SpineClient` — subscribe / publish / ack |
| `github.com/prometheusags/frf/sdks/go/gen/flint/v1` | Generated protobuf types |

## Minimal example

```go
package main

import (
    "context"
    "log"
    "net/http"

    "connectrpc.com/connect"
    frf "github.com/prometheusags/frf/sdks/go/client"
)

func main() {
    c := frf.NewSpineClient("https://your-frf-gateway", http.DefaultClient)

    stream, err := c.Subscribe(context.Background(), &connect.Request[frf.SubscribeRequest]{
        Msg: &frf.SubscribeRequest{Channel: &frf.Channel{Name: "my-channel"}},
    })
    if err != nil {
        log.Fatal(err)
    }
    defer stream.Close()

    for stream.Receive() {
        event := stream.Msg()
        log.Printf("event: %v", event)
        // ack here
    }
}
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `FRF_GATEWAY_URL` | Base URL of the FRF gateway |
| `FRF_AUTH_TOKEN` | Bearer token (set as Authorization header) |

## SDK source

Source code lives at: `<flint-realtime-fabric>/sdks/go/`. Resolve the repository root from the current workspace or `FLINT_REPO_ROOT`; never assume a machine-specific path.
