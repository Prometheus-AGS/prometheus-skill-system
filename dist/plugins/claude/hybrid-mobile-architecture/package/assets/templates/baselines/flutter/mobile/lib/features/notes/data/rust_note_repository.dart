// TJ-ARCH-MOB-001 compliant
import '../../../bridge/generated/api/notes.dart' as rust;
import '../domain/note.dart';

class RustNoteRepository implements NoteRepository {
  RustNoteRepository(this.databasePath);
  final String databasePath;
  @override
  Future<List<Note>> list() async =>
      (await rust.listNotes(databasePath: databasePath))
          .map((note) => Note(id: note.id, title: note.title))
          .toList();
  @override
  Future<void> create(String title) async {
    await rust.createNote(databasePath: databasePath, title: title);
  }
}
