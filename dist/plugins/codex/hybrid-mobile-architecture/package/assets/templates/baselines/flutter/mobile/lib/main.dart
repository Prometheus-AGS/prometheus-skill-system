// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'bridge/generated/frb_generated.dart';
import 'features/notes/presentation/screens/notes_screen.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const ProviderScope(child: NotesApp()));
}

class NotesApp extends StatelessWidget {
  const NotesApp({super.key});
  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Notes',
    theme: ThemeData(
      colorSchemeSeed: const Color(0xff235d75),
      useMaterial3: true,
    ),
    home: const NotesScreen(),
  );
}
