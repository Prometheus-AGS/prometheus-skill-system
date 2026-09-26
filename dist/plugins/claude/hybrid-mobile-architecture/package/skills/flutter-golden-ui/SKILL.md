---
name: flutter-golden-ui
description: ALWAYS invoke after building or changing any Flutter widget/screen, before calling it done — scaffold a golden test for it and verify with the Dart & Flutter MCP hot-reload/widget-inspector loop. Triggers on Flutter widget, Flutter screen, golden test, golden file, widget test, visual regression, shadcn_flutter, ConsumerWidget, Flutter UI, mobile UI, pump widget, matchesGoldenFile.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Prefer simple, surgical, strongly typed changes; preserve strict
> layering and verify dependency versions. When installed in a project, also
> obey that project's `AGENT_BASE_RULES.md`; this skill remains self-contained.

# Flutter Golden UI

A Flutter widget is not done until it has a golden test and has been verified live via the
Dart & Flutter MCP. Goldens are the high-signal test form for visual widgets — far more
than brittle markup assertions.

## Verify loop (Dart & Flutter MCP)

1. **Run the app** on a simulator/device.
2. Use the **Dart & Flutter MCP** to **hot-reload** the changed widget and inspect it with
   the **widget inspector** — confirm layout, overflow, and both `ColorScheme`s before
   writing a golden. Fix visually first; snapshot second.

## Golden scaffolding (VGV workflow)

Follow the Very Good Ventures golden-test workflow.

1. **One golden per meaningful state**, both themes:
   ```dart
   testWidgets('ContentBlockCode renders (light)', (tester) async {
     await tester.pumpWidget(_wrap(const ContentBlockCode(...), Brightness.light));
     await expectLater(
       find.byType(ContentBlockCode),
       matchesGoldenFile('goldens/content_block_code_light.png'),
     );
   });
   ```
2. **Wrap with the real theme** (shadcn_flutter ThemeData from [[hybrid-design-tokens]]),
   not a bare `MaterialApp` default — the golden must reflect production tokens.
3. **Pump both `Brightness.light` and `Brightness.dark`.**
4. **Deterministic goldens:** load fonts, disable animations, pump to settle
   (`await tester.pumpAndSettle()`), fix a device size. No timers, no network.
5. **Update deliberately:** `flutter test --update-goldens` only after a visual change you
   intend — review the diff, don't blind-accept.

## Riverpod / layer-contract reminders (don't violate while testing)

- Providers use `@riverpod` codegen — never manual `Provider(...)`.
- Widgets call providers, never APIs/FFI directly. Streaming providers are `autoDispose`.
- ContentBlock state mutates only via `ChatNotifier.streamBlock()` — pump that, don't
  assign state directly in a test.

## Testing philosophy (this repo)

Golden/snapshot tests are the preferred form. Do NOT chase coverage percentage or write
unit tests of internal helpers. Write **3–5 behavior/golden tests per completed feature**,
at completion, at the widget's public surface. If a golden fails to fix twice, STOP and
report — do not `--update-goldens` to escape a red state.

## Related skills

- `flutter/skills` (official), VGV golden-test workflow, `shadcn-ui-flutter` (external)
- Dart & Flutter MCP — the hot-reload/inspector verify loop
- [[hybrid-design-tokens]] — the theme goldens must render with
- [[a11y-gate]] — Flutter Semantics/contrast checks
- [[content-block-ui]] — one golden per ContentBlock variant
