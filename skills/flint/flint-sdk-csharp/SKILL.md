---
name: flint-sdk-csharp
description: >
  Install and use the Flint Realtime Fabric C#/.NET SDK (FlintSdk). Covers NuGet / project
  reference setup, SpineClient configuration, channel subscriptions, and event publishing
  for .NET 8+ applications.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, csharp, dotnet, sdk, grpc, connect-rpc]
---

# flint-sdk-csharp

Use the **Flint Realtime Fabric** C#/.NET SDK (`FlintSdk`) to subscribe to channels and publish events in .NET 8+ applications.

## When to use

- Adding realtime event streaming to an ASP.NET Core service, Blazor app, or .NET worker.
- Consuming FRF channels via gRPC/Connect-RPC from C#.

## Requirements

- .NET 8.0+
- Grpc.Net.Client or ConnectRpc transport

## Installation

Reference the project directly (until published to NuGet):
```xml
<!-- In your .csproj -->
<ItemGroup>
  <ProjectReference Include="/path/to/flint-realtime-fabric/sdks/csharp/FlintSdk/FlintSdk.csproj" />
</ItemGroup>
```

Or build and reference the DLL:
```bash
# In flint-realtime-fabric/sdks/csharp
dotnet build -c Release
```

## Minimal example

```csharp
using FlintSdk;
using Grpc.Net.Client;

var channel = GrpcChannel.ForAddress("https://your-frf-gateway");
var client = new SpineClient(channel);

// Subscribe
await foreach (var ev in client.SubscribeAsync("my-channel", cancellationToken: ct))
{
    Console.WriteLine(ev);
    await client.AckAsync(ev.Cursor, ct);
}

// Publish
await client.PublishAsync("my-channel", payload: "hello"u8.ToArray(), ct);
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `FRF_GATEWAY_URL` | Base URL of the FRF gateway |
| `FRF_AUTH_TOKEN` | Bearer token for authenticated channels (optional) |

## SDK source

Source code: `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/sdks/csharp/`  
Solution: `FlintSdk.slnx`
