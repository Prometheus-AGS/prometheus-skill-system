// TJ-ARCH-MOB-001 compliant
import 'package:flutter_test/flutter_test.dart';
import 'package:__APP_CRATE___mobile/features/notes/domain/note.dart';

void main() {
  test('keeps the native note identity and Unicode title intact', () {
    const note = Note(id: '42', title: 'Remember café ☕');
    expect(note.id, '42');
    expect(note.title, 'Remember café ☕');
  });
}
