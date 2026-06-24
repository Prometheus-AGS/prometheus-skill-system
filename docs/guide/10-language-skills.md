# 10 · Language & Domain Skills

The process skills decide *how the loop runs*. The language and domain skills are what the loop *knows* — the production patterns, idioms, and code templates the enrichment engine injects before an agent writes a line. This page documents all of them, by category. Every native skill here is `v1.0.0` and MIT-licensed unless noted; skills that ship Tera (`.tera`) code-generation templates are marked, because those templates are what forge-rs renders at enrichment time.

## Rust (10 skills)

The Rust skills encode the patterns behind the entire Prometheus stack — the tools in this repository are themselves built on them.

| Skill | Templates | What it encodes |
|---|---|---|
| **actor-model** | — | Tokio-native actor pattern: actors as tasks over `mpsc` channels with a typed message enum, no shared locks. Used in UAR session management and event broadcast. |
| **async-patterns** | — | Canonical async Rust: task spawning, `Arc<RwLock<T>>` vs. `parking_lot::Mutex` selection, blocking-guard prevention, broadcast channels, graceful shutdown, structured concurrency. |
| **axum-patterns** | `router.rs`, `handler.rs`, `middleware.rs`, `app_state.rs`, `app_error.rs` | Axum 0.8 routing, typed state via `Extension`/`State`, Tower middleware, structured errors, native SSE for MCP. |
| **error-handling** | — | The thiserror/anyhow boundary (thiserror for libraries, anyhow for applications), `#[cold]` error paths, no `unwrap`/`expect` outside tests, `?` propagation. |
| **karpathy-tokenizer** | `train_tokenizer.py`, `load_tokenizer.rs` | Train GPT-style BPE tokenizers with `rustbpe` (Karpathy's `minbpe` approach), export to tiktoken for fast Rust inference, enforce prompt budgets. |
| **librefang-wasm-skill** | `skill.toml`, `Cargo.toml`, `src/lib.rs`, `src/host.rs` | Generate a LibreFang WASM-ABI-compliant Rust skill: a `cdylib` exporting the Guest ABI over capability-checked host functions, targeting `wasm32-unknown-unknown`/wasmtime. Ships `validate-wasm-abi.sh`. |
| **mcp-server** | — | The canonical Axum MCP server: JSON-RPC 2.0 over `POST /mcp` plus optional SSE at `GET /events`, tool registration and dispatch, broadcast fan-out, stdio transport for Claude Desktop. |
| **performance** | — | Production perf primitives: jemalloc global allocator, `#[cold]`/`#[inline(never)]`, `MaybeUninit`, `std::mem::take`, Arc placement, parking_lot over `std::sync`, SIMD-aware buffers. |
| **prometheus-rust-auditor** | — | Staged autonomous Rust code-quality remediation: Clippy enforcement, formatting, dependency policy, workspace inventory, partition-based invariant audits, CI generation. Also shipped as a standalone tool — see [Tools Reference](13-tools-reference.md). |
| **workspace-structure** | — | Multi-crate workspace layout: `resolver=2`, workspace-level dependency versions, domain-driven crate separation (`*-core`/`*-store`/`*-librarian`/`*-mcp`/`*-cli`), feature-flag discipline, release profiles. |

## React (2 skills)

| Skill | Templates | What it encodes |
|---|---|---|
| **react-vite-stack** | `page_component.tsx`, `feature_hook.ts`, `store.ts`, `api_client.ts` | The canonical React 19 + Vite 8 stack: TanStack Router (file-based), TanStack Query, TanStack React-Table, Zustand 5, shadcn/ui + Tailwind 4. Always `.tsx`. |
| **prometheus-entity-skills** | — | A full-stack entity-management suite built on `@prometheus-ags/prometheus-entity-management`. Eight sub-skills (below). |

The **prometheus-entity-skills** bundle is a progressive-disclosure catalog of installable plugin groups:

1. **entity-graph-setup** — adopt the library in an existing app; detect legacy data layers, infer entity types, emit `registerSchema`, phased migration. *(sub-skills: entity-graph-init, -detect, -migrate)*
2. **entity-graph-crud** — generate CRUD screens via `useEntityCRUD`; TanStack Table column defs, FieldDescriptor forms, detail/form sheets, cascade invalidation; strict Components→Hooks→Stores→APIs flow. *(entity-crud-page, -form, -table, -relations)*
3. **entity-graph-graphql** — a GraphQL layer: `GQLClient`, EntityDescriptor trees, Zustand normalization, typed hooks (`useGQLEntity`, `useGQLList`, `useGQLMutation`, `useGQLSubscription`), graphql-ws realtime. *(entity-gql-setup, -hooks, -subscription)*
4. **entity-graph-realtime** — realtime sync across WebSocket, Supabase Realtime, Convex, GraphQL subscriptions, or ElectricSQL+PGlite; 16 ms coalescing. *(entity-realtime-setup, -channel, -local-first)*
5. **entity-graph-prisma** — Prisma integration: analyze `schema.prisma`, generate `registerSchema` relation graphs, `toPrismaWhere`/`toPrismaOrderBy`, Next.js App Router CRUD routes. *(entity-prisma-setup, -generator, -api, -migrate)*
6. **entity-graph-optimize** — audit integrations for architecture violations, selector/subscription churn, missing cascade registration, memory growth. *(entity-audit, -perf, -gc)*
7. **entity-realtime-surreal-live** *(standalone)* — wire SurrealDB LIVE SELECT into the graph via `createSurrealLiveAdapter`, with select-then-live seeding and exponential-backoff reconnect.

The underlying library (`@prometheus-ags/prometheus-entity-management`, the imported `prometheus-entity-management` submodule) is a normalized, globally-reactive entity graph store for React on Zustand + immer — one application-wide graph that replaces TanStack Query's per-view cache. Its core hook is `useEntity({type, id, fetch, normalize})`.

## Flutter, Tauri, HTMX

| Skill | Templates | What it encodes |
|---|---|---|
| **flutter/flutter-rust-ffi** | `riverpod_notifier.dart`, `feature_repository.dart`, `go_router_config.dart` | Flutter + Rust FFI via `flutter_rust_bridge` v2 + Riverpod; a shared `gen_ui_core` Rust crate, bridge codegen, FFI thread safety, bidirectional React↔Flutter embedding. |
| **tauri/tauri-react-vite** | — | Tauri 2 + React 19 + Vite 8 desktop: the command API, IPC `invoke()`, Rust sidecar/plugin, secure window config, `gen_ui_core` sharing between the Tauri backend and flutter_rust_bridge. |
| **htmx/htmx-alpine-lit** | `page.html`, `lit_component.ts`, `react_island.tsx`, `axum_fragment_handler.rs` | HTMX 2.0.8 + Alpine.js + Lit server-driven UI: HTMX request/response, Alpine `x-data` controllers, Lit web components, server-side fragments, HTMX-in-React islands. |

The architecture principle is consistent across these: **the server drives, the client declares, and a shared Rust core does the heavy lifting**. In React: Components compose Hooks, Hooks orchestrate Stores, Stores own API calls — components never import stores or call `fetch()` directly. In Flutter: Widgets watch providers, Notifiers call repositories, only the Rust FFI repository calls bridge functions. In HTMX: the server returns HTML fragments, Alpine handles local state, Lit encapsulates complex elements, and React hosts HTMX islands.

## TypeScript, Go, Python, Architecture

| Skill | What it encodes |
|---|---|
| **typescript/base-patterns** | TypeScript 6: no `any`, no `@ts-ignore`, discriminated unions, Result types, branded ID types, zod runtime validation. |
| **go/base-patterns** | Go 1.22: `%w` error wrapping, context propagation, interface-based dependency injection, slog structured logging, cmd/internal/pkg layout, table-driven tests. |
| **python/pyo3-bridge** | PyO3 0.22 Rust→Python bridging: `#[pyfunction]`/`#[pyclass]`, maturin builds, GIL management, async via pyo3-asyncio. Primary use is the skill-executor calling Rust crates (forge-rs, pk-librarian, surreal-memory) from Python skill servers. |
| **architecture/clean-architecture** | The CLEAN four-layer model (Domain → Application → Infrastructure → Interface), dependency inversion, and trait/interface boundaries mapped onto Rust crates, Flutter features, React feature slices, and Go packages. |

## Testing (2 skills)

| Skill | What it encodes |
|---|---|
| **bdd-testing** | BDD integration tests with Cucumber.js + Gherkin + Playwright, across API/UI/agent layers, with automatic video capture. Ships `run-bdd.sh` and `generate-report.sh`. |
| **bdd-video-proof** | Record MP4 video evidence per passing Cucumber scenario and pin it to IPFS for an immutable audit trail. |

These two connect to a system-wide rule: the **BDD Immutable-Tests Rule** (`BDD-006`), enforced by the `protect-tests.sh` hook. Code-generation agents may add new `.feature` files under `tests/features/drafts/` but may not edit existing tests to make failing tests pass. The full treatment is on the [Hooks & Lifecycle](15-hooks-and-lifecycle.md) page.

## DevOps (4 skills)

All four conform to the internal standard `TJ-CICD-001 v1.1` and declare `allowed-tools`.

| Skill | What it does |
|---|---|
| **gitops-bootstrap** | Scaffold a full multi-cloud GitOps CI/CD system from scratch — detect GKE/AKS/EKS, build base/cloud/env Kustomize overlays, GitHub Actions with keyless OIDC per cloud, ArgoCD Application CRs/ApplicationSets, remote cluster registration. Ships `detect-cloud.sh`, `register-clusters.sh`. |
| **gitops-transform** | Detect and transform existing GitHub Actions / Kubernetes deploy configs to `TJ-CICD-001`, with a diff-based plan before changes and static-credential → OIDC migration. Ships `detect-stack.sh`. |
| **argocd-multicloud** | Install and manage ArgoCD as a multi-cloud control plane on GKE with AKS/EKS as remote destinations — App-of-Apps root, ApplicationSet fan-out, project isolation, RBAC, Dex OIDC SSO. |
| **kustomize-overlay** | Generate three-dimensional Kustomize overlays (base/cloud/env) with cloud-specific identity patches (GKE Workload Identity, Azure Workload Identity, EKS IRSA), and validate/repair broken overlay chains. |

A system-wide guard backs these up: `guard-direct-deploy.sh` blocks `kubectl apply` and `helm upgrade` as deploy mechanisms, because in a GitOps world the cluster state is owned by Git, not by an agent running `kubectl`.

## Document extraction

| Skill | License | What it does |
|---|---|---|
| **kreuzberg** | **Elastic-2.0** | Extract text, tables, metadata, and images from 91+ formats (PDF, Office, images, HTML, email, archives, academic papers) via Kreuzberg, with Python/Node/Rust/CLI bindings, OCR, chunking, and batch processing. |

Note the license: `kreuzberg` is Elastic-2.0, not MIT — the one license exception in the native skill set, called out here so it is not a surprise in a compliance review.

## Flint Realtime Fabric SDK skills (6 skills)

Each is an install-and-usage guide for the same Flint Realtime Fabric event system — `SpineClient`, channel subscriptions, event publishing, ack handling — in a different language.

| Skill | Package / target |
|---|---|
| **flint-sdk-ts** | `@prometheusags/frf-sdk` — browser + Node, WebSocket + Connect-RPC |
| **flint-sdk-go** | `github.com/prometheusags/frf/sdks/go` — Connect-RPC transport |
| **flint-sdk-dart** | `frf_dart` — generated from Rust FFI via `flutter_rust_bridge` 2.11.1 |
| **flint-sdk-swift** | `FrfClient` — SPM, iOS 16+/macOS 13+ |
| **flint-sdk-kotlin** | `frf-kotlin` — Gradle, Android/JVM via JNI |
| **flint-sdk-csharp** | `FlintSdk` — NuGet, .NET 8+, gRPC/Connect-RPC |

## How language skills feed the loop

These skills are not just documentation an agent reads. The ones that ship `.tera` templates feed directly into the forge-rs enrichment engine: when a task is detected as Rust, the `axum-patterns` and `error-handling` skills resolve, their templates render with the task context and the active constitution, and the result lands in the enriched context file the agent reads before writing code. The non-template skills contribute their `SKILL.md` guidance and `references/`. Either way, the language knowledge is injected *before* implementation, not offered as an afterthought. That sequencing is the [four-layer pipeline](04-four-layer-pipeline.md) at work.

---

*Previous: [← 09 · Process & Orchestration Skills](09-process-skills.md) · Next: [11 · The Artifact Refiner →](11-artifact-refiner.md)*
