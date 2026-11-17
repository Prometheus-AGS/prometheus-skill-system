# Tauri plugin agent contract

Load this reference for `@prometheus-ags/entity-graph-tauri`, Tauri v2 IPC,
native plugin capabilities, generated bindings, or Android/iOS plugin work.

## Use the correct surface

- Register Rust with `entity_graph_tauri::init()`.
- Prefer `createTauriGraphPlugin` and its command facade for graph-aware
  TypeScript integration.
- Use `generatedCommands` and `generatedEvents` when the Rust-derived raw
  result contract is required.
- Run `check:bindings`; never hand-edit `generated-bindings.ts`.
- Keep `generated-public.ts` as the structural public facade over generated
  runtime bindings; public declarations must not expose an implementation-only
  module path.
- Run `verify:skills` after building. The Tauri ledger tracks both runtime and
  declaration exports.
- Require the packed TypeScript NodeNext consumer with `skipLibCheck: false`;
  enabling `skipLibCheck` would hide declaration-boundary regressions.

## Preserve least privilege

`entity-graph-tauri:default` grants only `allow-graph-get-entity`,
`allow-graph-get-list`, and `allow-graph-platform-ping`. Add explicit command
permissions such as `entity-graph-tauri:allow-graph-upsert-entity` only to the
window or webview that needs them. Do not describe `default` as full access.

## Preserve storage ownership

The Rust plugin mirror and its snapshot map are in-memory. A
`graph_persist_snapshot` call is not restart durability. Use the core
`createTauriSqlPersistenceAdapter` for durable SQLite state and keep remote
sync ownership in its adapter or service.

## Require the right evidence

`pnpm run verify:tauri-plugin` proves Rust-derived bindings, registered desktop
IPC, capability denial, packed Rust consumption, and recomputes the checked-in
Android physical-device and iOS simulator command/denial receipts. Require the
receipts in `release/tauri-mobile-device-lane.md` for mobile plugin claims, and
load `tauri-universal-example.md` before making complete application UX,
restart/offline, accessibility, or visual-parity claims. Plugin receipts do not
substitute for application receipts.

Read `release/tauri-mobile-plugin.md` for the complete evidence matrix and
explicit exclusions.
