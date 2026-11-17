# Examples gallery — certified 3.0 consumer evidence

Each example is a certified consumer of the public packages. An example proves
only its declared evidence boundary; none authorizes publication by itself.

| Example | Demonstrates | Gate |
| ------- | ------------ | ---- |
| `examples/vite-app` | React 19 / Vite 8: graph lifecycle, realtime coalescing, Suspense, axe-clean browser lanes | `pnpm run verify:vite-react19` |
| `examples/nextjs-app` | Next.js App Router: SSR/hydration handoff, route transitions, mutations + realtime | `pnpm run verify:nextjs-app-router` |
| `examples/agentic-a2ui` | A2UI surfaces + A2A policy boundary: cross-view updates, tenant mismatch denial, approval-gated destructive actions | `pnpm run verify:agentic-a2ui` |
| `examples/flutter-riverpod` | Flutter/Riverpod mobile with A2UI rendering over the Dart graph | `pnpm run verify:flutter-a2ui-v1 -- --report <file>` |
| `examples/tauri-app` | Universal Tauri 2: one React/Vite frontend on desktop + Android + iOS, capabilities, offline restart, deep links | `pnpm run verify:tauri-universal` |
| `examples/supabase` | Shared Supabase demo project (config only) used by the example apps | configuration notes; no certified gate |

Shared scenario vocabulary lives in `examples/shared/`; the semantic scenario
gate is `pnpm run verify:example-coverage`, and the machine-readable coverage
ledger is `examples/coverage.json`.

## Using an example as a starting point

1. Copy the example, keep its `BridgeDeniedError`/capability and evidence
   patterns intact.
2. Replace the domain (`src/domain/`), never the graph data flow: components
   consume hooks; stores/adapters own I/O.
3. Re-run the example's verifier after changes; a green fork is only as strong
   as its own evidence.
