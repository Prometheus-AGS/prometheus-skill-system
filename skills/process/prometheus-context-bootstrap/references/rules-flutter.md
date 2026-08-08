---
paths: ['**/*.dart', '**/pubspec.yaml']
---

# Flutter / Dart

Loaded when a Dart file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `dart analyze` |
| T1 unit complete | `flutter test test/<file>` |
| T2 phase complete | `flutter test` |
| T3 milestone only | `flutter build ios`; `flutter build apk`; device certification |

## Hard rules

- Platform builds are the expensive tier. A single heavy plugin can add minutes
  to a cold Xcode build. Never platform-build mid-phase.
- Use `flutter build ios --config-only` when only project config changed.
- State lives in providers, not in widgets. A widget renders and submits intent.

<!-- Replace the commands above with this project's real ones if they differ. -->
