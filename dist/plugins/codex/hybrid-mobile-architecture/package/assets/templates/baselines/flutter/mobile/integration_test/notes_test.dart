// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:__APP_CRATE___mobile/main.dart' as app;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('creates a persisted note through the real Rust bridge', (
    tester,
  ) async {
    await app.main();
    await tester.pumpAndSettle();
    const title = 'Native note restart proof';
    if (const bool.fromEnvironment('VERIFY_RESTART')) {
      expect(find.text(title), findsAtLeastNWidgets(1));
      return;
    }
    await tester.enterText(find.byKey(const Key('note-title')), title);
    await tester.tap(find.byKey(const Key('save-note')));
    await tester.pumpAndSettle();
    expect(find.text(title), findsOneWidget);
  });
}
