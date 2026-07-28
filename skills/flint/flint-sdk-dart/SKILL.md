---
name: flint-sdk-dart
description: >
  Install and use the Flint Realtime Fabric Dart/Flutter SDK (frf_dart). Generated from Rust FFI
  via flutter_rust_bridge 2.11.1. Covers pub.dev installation, SpineClient setup, channel
  subscriptions, and event publishing for Flutter and Dart applications.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, dart, flutter, sdk, ffi, rust-bridge]
---

# flint-sdk-dart

Use the **Flint Realtime Fabric** Dart/Flutter SDK (`frf_dart`) — generated from the Rust core via `flutter_rust_bridge` 2.11.1.

## When to use

- Adding realtime event streaming to a Flutter app (iOS, Android, macOS, Web, Windows, Linux).
- Consuming FRF channels from Dart via FFI bridge to the Rust core.

## Requirements

- Dart SDK ≥ 3.3.0, < 4.0.0
- Flutter 3.19+
- `flutter_rust_bridge: ^2.11.1`

## Installation

Build the native libraries first:
```bash
# In flint-realtime-fabric repo
bash sdks/dart/build_dart.sh
```

Add to `pubspec.yaml`:
```yaml
dependencies:
  frf_dart:
    path: /path/to/flint-realtime-fabric/sdks/dart
  flutter_rust_bridge: ^2.11.1
```

Then:
```bash
flutter pub get
```

## Minimal example

```dart
import 'package:frf_dart/frf_dart.dart';

final client = SpineClient(gatewayUrl: 'https://your-frf-gateway');

// Subscribe
client.subscribe(channel: 'my-channel').listen((event) {
  print(event);
  client.ack(cursor: event.cursor);
});

// Publish
await client.publish(
  channel: 'my-channel',
  payload: Uint8List.fromList('hello'.codeUnits),
);
```

## SDK source

Source code: `<flint-realtime-fabric>/sdks/dart/`. Resolve the repository root from the current workspace or `FLINT_REPO_ROOT`.
Package name: `frf_dart`  
Generated types: `lib/` (from `build_dart.sh` — regenerate if proto schema changes)
