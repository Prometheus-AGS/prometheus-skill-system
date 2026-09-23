---
paths: ['**/*.dart', '**/pubspec.yaml']
---

# Flutter / Dart

Loaded when a Dart file is read. Not resident.

Batch implementation until the app path is complete. Use narrow analyzer feedback
earlier only to unblock work. At a completed change boundary, run the smallest app,
platform, or device integration that exercises the real providers, repositories, and
data sources. Unit, widget-only, mock-only, and per-edit tests are not completion
evidence. Reserve full platform builds and device certification for the final phase
or release boundary.

## Hard rules

- Platform builds are expensive. A single heavy plugin can add minutes
  to a cold Xcode build. Never platform-build mid-phase.
- Use `flutter build ios --config-only` when only project config changed.
- State lives in providers, not in widgets. A widget renders and submits intent.

<!-- Replace example boundaries with this project's real production-path gates. -->
