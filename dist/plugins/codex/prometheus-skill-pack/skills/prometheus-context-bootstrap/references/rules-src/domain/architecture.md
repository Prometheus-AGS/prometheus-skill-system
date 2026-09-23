---
paths: ['**/features/**', '**/src/**', '**/lib/**', '**/crates/**']
---

# Architecture (v3 §B, in full)

Loaded when source files are read. Not resident. The per-stack file says how each rule looks in that stack.

- **B-1 · Open standards first.** Prefer MCP, OpenAI-compatible APIs, A2A, AG-UI, A2UI, ACP, HTMX, WASM Component Model, JSON Schema, OpenAPI, GraphQL where apt, PostgreSQL-compatible storage, IPFS-compatible distribution where apt. Avoid lock-in unless explicitly required.
- **B-2 · Feature-based clean architecture.** Organize by business capability/bounded context, not technical layer (`features/<domain>/{components,hooks,stores,services, types,schemas,pages,tests}` + `shared/ core/ infrastructure/`). No global dumping-ground folders. Cross-feature dependencies explicit.
- **B-3 · Strict layering.** `UI → Hooks/ViewModels → Stores → Services → External`. Reverse flow only via reactive state/events. Forbidden: UI→API/Service/DB, Hook→API/Service, Component→store-mutation logic.
- **B-4 · Layer responsibilities.** UI is pure (render, interact, layout, style, a11y — no fetching/business logic). Hooks/ViewModels coordinate UI state (no direct API/DB). Stores own application state and are its single source of truth (Zustand/Riverpod/etc.; no render logic). Services own all external communication (API, DB, MCP, agents, filesystem; reusable, testable, framework-independent). State changes propagate through the framework's native reactive mechanism — no manual refresh, no imperative UI sync.
- **B-5 · UI is a projection of state.** UI renders state and submits intent; domain logic validates; durable systems persist; events describe changes. No business rules that exist only in frontend components.
- **B-6 · Architecture is language-invariant.** React/Flutter/Rust-HTMX/Vue/Svelte all follow `View → ViewModel/Hook → Store → Service → Repository/API`. Technology changes; architecture does not.
- **B-7 · Strong typing; no framework magic.** Use strong types where the language supports them (no implicit/needless `any`, no stringly-typed domain models; prefer schema-generated types; keep contracts typed and versioned). Avoid opaque caches, hidden globals, framework-owned business logic, and uninspectable runtime behavior.
- **B-8 · Portability & local-first.** Consider web/mobile/desktop/local/cloud/offline for any feature. Prefer architectures that run locally and sync outward; cloud is allowed but do not become unnecessarily cloud-dependent. Prefer deterministic behavior; document intentional non-determinism.

## What "feature-based clean architecture" means in every layer

| Layer | Feature root | Inside a feature | Dependency direction |
|---|---|---|---|
| Web (React) | `<web>/src/features/<feature>/` | `components hooks stores services entities schemas types pages tests` | `Component → Hook → Store → Service → External` |
| Mobile (Flutter) | `<mobile>/lib/features/<feature>/` | `data domain presentation` | `presentation → domain ← data` |
| Rust service | `<crate>/src/features/<feature>/` | `domain application infrastructure interface` | `interface → application → domain ← infrastructure` |
| Rust library | the crate is the bounded context | modules by capability (`orders/`, `pricing/`) | the crate graph |

- Organise by business capability, never by technical kind. No `utils/`, `helpers/`, `common/`, `misc/`.
- Features never import each other. What two features share moves to `shared/` (or `core/` for
  cross-cutting infrastructure); the composition root (`app/`, `main.rs`) wires features together.
- The innermost layer knows nothing about frameworks, transport or storage. Dependencies point inward.
- The same capability keeps the same name in every layer: `orders` on the server, in the web app, in the mobile app.
- `scripts/check-architecture.sh` checks the forbidden edges; ESLint checks them in React packages; crate
  boundaries check them in Rust.

## The 500-line rule — partition by architecture, not by scissors

No code file over 500 physical lines. When a file approaches the limit, it is doing more than one thing:

1. Name the responsibilities in the file (state / actions / selectors; types / operation / errors; screen
   / sections / widgets).
2. Create a directory named after the file, one file per responsibility, and a thin entry point
   (`index.ts`, `mod.rs`, a Dart barrel) that re-exports the same public surface, so importers do not change.
3. If a responsibility belongs to another layer or feature, move it there instead.
4. Never split mid-responsibility, never number files (`part2`), never compress code to fit, never add to
   `rules/line-limit-allowlist.txt` for code we wrote. The allowlist is for generated and vendored files only.
