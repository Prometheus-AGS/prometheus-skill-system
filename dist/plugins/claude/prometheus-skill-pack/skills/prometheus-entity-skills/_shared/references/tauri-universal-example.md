# Universal Tauri application agent contract

Load this reference when generating, changing, reviewing, or making evidence
claims about `examples/tauri-universal`.

## Preserve one application and one graph

- Keep one React 19/Vite 8 frontend for desktop, Android, and iOS.
- Components use hooks. Hooks select reactive entity projections from the
  canonical graph store and submit intents through the platform store; only
  the platform store calls the platform service. Components import no store,
  and components/hooks import no Tauri API or platform service.
- Use the canonical graph singleton required by the current Tauri facade. Do
  not create a second UI or native graph.
- Lists contain entity IDs only. Project and User details are joined from the
  canonical graph when Task views are read.
- Keep platform detection, persistence, commands, lifecycle, and deep links at
  the platform-service boundary rather than forking application logic.

## Preserve persistence ownership

- Tauri durability uses `createTauriSqlPersistenceAdapter` and
  `startLocalFirstGraph`; browser preview uses the same contract over
  localStorage.
- Keep queued offline mutations separately durable. Graph patches provide
  optimistic cross-view visibility; confirmation updates canonical data and
  clears the patch.
- The Rust plugin mirror remains in-memory. Its snapshot commands do not prove
  restart durability.

## Preserve actual trust boundaries

- Grant individual mutation permissions only to the webview that needs them.
  Keep clear, remove, and in-memory snapshot permissions out of the universal
  main capability unless a separately authorized flow is introduced.
- Treat deep links and persisted queue JSON as untrusted input. Validate the
  registered scheme, route, tenant, known graph ID, and queue structure before
  any mutation.
- Browser preview cannot certify native IPC authorization. A native claim needs
  a Tauri host receipt.

## Require evidence at the claimed level

Use these commands for focused source/runtime claims:

```bash
pnpm run typecheck:tauri-universal
pnpm run test:tauri-universal:unit
pnpm run test:tauri-universal:rust
pnpm run test:tauri-universal:contract
pnpm run verify:tauri-universal
```

The source verifier deliberately returns
`countsAsPlatformBuildEvidence: false`. It now requires and hash-verifies the
separate task-5 platform receipt, which proves:

- five Chromium graph, relationship, realtime, offline-restart, responsive,
  and accessibility flows with screenshots and traces;
- a packaged macOS command, capability-denial, native SQLite restart, and
  reconnect flow;
- Android API 36 emulator build/runtime and capability denial;
- iOS 26.5 simulator archive/runtime.

Require `pnpm run verify:tauri-universal` before repeating those claims. Keep
Windows/Linux, physical-device, signing, app-store, registry, and stable-release
claims separate.

Read `release/tauri-universal-example.md` and `examples/coverage.json` for the
current disposition. Do not infer npm publication, `latest`, app-store
authority, or stable 3.0.0 from an example gate.
