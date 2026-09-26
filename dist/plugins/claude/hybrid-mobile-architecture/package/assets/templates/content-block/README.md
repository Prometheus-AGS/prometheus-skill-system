# ContentBlock Variant Template

Quick reference for adding a new block type. Full guide: `references/rust/new-block-type.md`.

## Files to edit (7 steps)

| # | File | What to add |
|---|------|------------|
| 1 | `rust/gen_ui_core/src/streaming.rs` | `StreamEvent` enum variant |
| 2 | `rust/gen_ui_core/src/protocol/a2ui.rs` | `A2uiEvent` variant + `A2uiAdapter::ingest()` arm |
| 3 | `rust/gen_ui_core/src/protocol/agui.rs` | `AguiAdapter::translate()` arm |
| 4 | Run codegen | `flutter_rust_bridge_codegen generate` (Flutter) or update TS types |
| 5 | `mobile/lib/bridge/a2ui/a2ui_event.dart` (Flutter) OR `desktop/src/bridge/a2ui/types.ts` (Tauri) | Sealed class or discriminated union variant |
| 6 | `mobile/lib/bridge/a2ui/a2ui_content_driver.dart` / `desktop/src/bridge/a2ui/driver.ts` | Driver mapping ← **compiler error until added** |
| 7 | `mobile/lib/features/chat/models/message.dart` + widget / `desktop/src/bridge/a2ui/types.ts` + component | `ContentBlock` variant + Widget/Component ← **compiler error until added** |

## Naming convention

- Rust: `PascalCase` enum variant
- Dart: `camelCase` factory constructor → `PascalCaseBlock` class
- TypeScript: `camelCase` type discriminant
- Flutter widget: `PascalCaseBlockWidget`
- React component: `PascalCaseBlock`
