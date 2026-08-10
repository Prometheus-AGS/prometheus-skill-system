---
license: MIT
name: clean-architecture
version: '1.0.0'
description: >
  CLEAN architecture patterns common to all languages in the Prometheus AGS stack.
  Defines the four-layer model (Domain → Application → Infrastructure → Interface),
  dependency inversion rules, trait/interface boundaries, and how the pattern maps
  to Rust crates, Flutter features, React feature slices, and Go packages. Apply
  across all languages to ensure consistent layering and testability.
language: rust
metadata:
  tags: [architecture, clean-architecture, patterns]
---

# CLEAN Architecture — Cross-Language Patterns

## The Four-Layer Model

All Prometheus AGS projects enforce the same dependency direction regardless of language:

```
┌─────────────────────────────────────────────────────────┐
│  Interface Layer (I)                                    │
│  HTTP handlers, CLI commands, Tauri commands, Dart UI  │
│  → depends on: Application                             │
├─────────────────────────────────────────────────────────┤
│  Infrastructure Layer (I)                               │
│  DB adapters, HTTP clients, filesystem, LLM APIs        │
│  → depends on: Domain (via traits)                      │
├─────────────────────────────────────────────────────────┤
│  Application Layer (A)                                  │
│  Use cases, orchestration, business rules               │
│  → depends on: Domain only                             │
├─────────────────────────────────────────────────────────┤
│  Domain Layer (D)                                       │
│  Entities, value objects, domain traits                 │
│  → depends on: nothing (no I/O, no framework)          │
└─────────────────────────────────────────────────────────┘
```

**The Dependency Rule**: Source code dependencies only point inward. The Domain
layer never imports from Application or Infrastructure. Infrastructure depends
on Domain traits, not concrete implementations.

## Rust Mapping

| Layer | Crate | Example |
|---|---|---|
| Domain | `*-core` | `WikiEntry`, `RawDoc`, `StorePort` trait |
| Application | `*-librarian` | `Librarian::compile()` — orchestrates store + LLM |
| Infrastructure | `*-store` | `MarkdownStore: StorePort` — concrete filesystem impl |
| Interface | `*-mcp`, `*-cli` | Axum handlers, clap commands |

```rust
// Domain — pure types and traits (no I/O)
// forge-core/src/lib.rs
pub trait SkillRepository: Send + Sync {
    async fn find_by_language(&self, lang: &Language) -> Vec<SkillManifest>;
    async fn save(&self, manifest: &SkillManifest) -> anyhow::Result<()>;
}

// Infrastructure — concrete implementation
// forge-skills/src/lib.rs
pub struct FilesystemSkillRepository { root: PathBuf }
impl SkillRepository for FilesystemSkillRepository { /* ... */ }

// Application — use case, depends only on the trait
// forge-enricher/src/lib.rs
pub struct Enricher<R: SkillRepository> {
    skills: R,
}
impl<R: SkillRepository> Enricher<R> {
    pub async fn enrich(&self, task: &Task) -> EnrichmentContext { /* ... */ }
}
```

## Flutter Mapping

```
lib/src/
  features/
    inference/
      domain/              ← entities, abstract repositories (interfaces)
        inference_entity.dart
        inference_repository.dart
      application/         ← use cases, Riverpod notifiers
        run_inference_usecase.dart
        inference_notifier.dart
      infrastructure/      ← Rust FFI calls, HTTP clients
        rust_ffi_inference_repository.dart
      presentation/        ← Flutter widgets
        inference_screen.dart
        inference_widget.dart
```

```dart
// Domain — abstract
abstract class InferenceRepository {
  Future<String> complete(String prompt, String model);
  Stream<String> stream(String prompt, String model);
}

// Infrastructure — Rust FFI implementation
class RustFfiInferenceRepository implements InferenceRepository {
  @override
  Future<String> complete(String prompt, String model) =>
      runInference(prompt: prompt, model: model); // generated bridge

  @override
  Stream<String> stream(String prompt, String model) =>
      streamInferenceTokens(prompt: prompt, model: model);
}

// Application — use case injected with repository
class RunInferenceUseCase {
  final InferenceRepository _repo;
  RunInferenceUseCase(this._repo);

  Future<String> execute(String prompt, String model) =>
      _repo.complete(prompt, model);
}
```

## React Mapping

```
src/features/posts/
  domain/               ← TypeScript interfaces (no framework imports)
    Post.ts             ← type Post = { id: string; title: string }
    PostRepository.ts   ← interface PostRepository { list(): Promise<Post[]> }
  application/          ← hooks that orchestrate (TanStack Query or entity hooks)
    usePostList.ts
    usePostMutation.ts
  infrastructure/       ← API clients implementing the interface
    RestPostRepository.ts
  presentation/         ← React components
    PostsPage.tsx
    PostsTable.tsx
```

## Go Mapping

```
internal/
  domain/           ← entities, interfaces
    post.go
    repository.go
  application/      ← use cases
    post_service.go
  infrastructure/   ← HTTP client, DB
    postgres_repo.go
  interface/        ← HTTP handlers
    post_handler.go
```

## Dependency Injection Rules

Construct dependencies at the composition root (main function, Tauri setup, test
fixture), not deep in business logic.

```rust
// In main.rs — composition root
let store = Arc::new(FilesystemSkillRepository::new(&skills_root));
let enricher = Enricher::new(store);
let state = McpState { enricher };
axum::serve(listener, router(state)).await?;
```

Never `new` infrastructure objects inside application or domain code.

## Testability Principle

The Domain and Application layers must be testable without I/O:
- Domain: pure unit tests, no async, no mocks needed
- Application: inject mock repositories, test business logic in isolation
- Infrastructure: integration tests against real I/O
- Interface: end-to-end tests only for critical paths

```rust
// Application test — no real filesystem needed
#[tokio::test]
async fn test_enricher_applies_correct_skills() {
    let mock_skills = MockSkillRepository::with_skills(vec![rust_performance_skill()]);
    let enricher = Enricher::new(mock_skills);
    let ctx = enricher.enrich(&rust_task()).await;
    assert!(ctx.applied_skills.contains(&"rust/performance".to_string()));
}
```

## Forbidden Patterns (All Languages)

- Domain entities importing framework types (Axum, Flutter, React)
- Application use cases calling `fetch()` / `reqwest` directly — use repository trait
- Infrastructure repositories containing business logic
- Presentation components talking to infrastructure directly (bypass application)
- `new ConcreteRepository()` inside application or domain code
