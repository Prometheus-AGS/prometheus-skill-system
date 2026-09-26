# Flutter Feature Module Template

Use `scripts/new-feature.mjs <name> <adopted-project-root>` to scaffold this structure.

```
features/<feature-name>/
  data/
    repositories/<Feature>RepositoryImpl.dart   ← implements domain interface
    datasources/<Feature>RemoteDatasource.dart  ← Rust FFI / HTTP calls
    models/<Feature>Model.dart                  ← DTO with fromJson/toEntity
  domain/
    entities/<Feature>Entity.dart               ← freezed business object
    repositories/<Feature>Repository.dart       ← abstract interface
    usecases/<Feature>UseCase.dart              ← single-responsibility use cases
  presentation/
    providers/<feature>_provider.dart           ← @riverpod Notifier / AsyncNotifier
    screens/<Feature>Screen.dart                ← ConsumerWidget screen
    widgets/<Feature>Widget.dart                ← feature-specific components
```

## Dependency direction

```
presentation → domain ← data
```

Cross-feature navigation: via GoRouter in `app/router.dart`.
Shared state: via providers in `shared/providers/`.
