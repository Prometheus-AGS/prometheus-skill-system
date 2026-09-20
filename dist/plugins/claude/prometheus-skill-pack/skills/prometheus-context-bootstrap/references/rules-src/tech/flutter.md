---
paths: ['**/*.dart', '**/pubspec.yaml', '**/analysis_options.yaml']
---

# Flutter and Dart

Loaded when a Dart file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `dart analyze` |
| T1 unit complete | `flutter test test/<file>` |
| T2 phase complete | `flutter test`; `scripts/check-file-lines.sh`; `scripts/check-architecture.sh` |
| T3 milestone only | platform builds (`flutter build` for ios / apk / appbundle); device certification |

Platform builds are the expensive tier. Never platform-build mid-phase; when only project config changed,
use the `--config-only` form.

## Layering — `Widget → Provider (the store) → Repository → Data source`

- `hooks_riverpod` 3.x is the store layer. Widgets extend `HookConsumerWidget`; providers are `@riverpod`
  codegen, never manual `Provider(...)`; `autoDispose` on streaming providers; `flutter_hooks` only for
  local widget state. This **overrides** any skill that says `flutter_riverpod` + `ConsumerWidget`.
- A widget renders and submits intent. State lives in providers. Only repositories and data sources touch
  `entity_graph_flutter`, FFI or the network.

## Feature folders

```
lib/
  app/        router, root providers
  core/       theme, errors, network
  features/<feature>/
    data/          repositories (implement domain interfaces), datasources, models
    domain/        entities, repository interfaces, usecases
    presentation/  providers, screens, widgets
  shared/     widgets, providers
  bridge/     generated FFI bindings
```

`presentation → domain ← data`. `domain` imports no Flutter, no Riverpod, no data package. `presentation`
never imports `data`. Features never import each other.

## File names and size

Dart files are `lower_snake_case` (the `file_names` lint) — the kebab-case rule is React-only. No `.dart`
file over 500 lines: turn `session_screen.dart` into `session_screen/` with a barrel file, split by widget
responsibility. Generated files (`*.g.dart`, `*.freezed.dart`, bridge output) are exempt and listed in
`rules/line-limit-allowlist.txt`.
