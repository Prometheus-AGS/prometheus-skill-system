---
name: flint-sdk-swift
description: >
  Install and use the Flint Realtime Fabric Swift SDK (FrfClient). Covers Swift Package Manager
  integration, SpineClient setup, channel subscriptions, and event publishing for iOS (16+)
  and macOS (13+) applications.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, swift, ios, macos, sdk, grpc, connect-rpc]
---

# flint-sdk-swift

Use the **Flint Realtime Fabric** Swift SDK (`FrfClient`) to subscribe to channels and publish events in iOS or macOS apps.

## When to use

- Adding realtime streaming to an iOS 16+ or macOS 13+ app.
- Consuming FRF channels from Swift via gRPC/Connect-RPC.

## Requirements

- iOS 16+ or macOS 13+
- Xcode 15+

## Installation (Swift Package Manager)

Add to `Package.swift` or via Xcode → File → Add Package Dependencies:

```swift
.package(
    url: "https://github.com/prometheusags/flint-realtime-fabric",
    from: "0.1.0"
),
```

Then add `FrfClient` to your target's dependencies:
```swift
.target(name: "MyApp", dependencies: ["FrfClient"])
```

Or build and link the XCFramework directly:
```bash
# In flint-realtime-fabric repo
bash sdks/swift/build_xcframework.sh
```

## Minimal example

```swift
import FrfClient

let client = SpineClient(baseURL: URL(string: "https://your-frf-gateway")!)

// Subscribe
for try await event in try await client.subscribe(channel: "my-channel") {
    print(event)
}

// Publish
try await client.publish(channel: "my-channel", payload: Data("hello".utf8))
```

## SDK source

Source code: `<flint-realtime-fabric>/sdks/swift/`. Resolve the repository root from the current workspace or `FLINT_REPO_ROOT`.
Package name: `FrfClient`
