// TJ-ARCH-MOB-001 compliant
class Note {
  const Note({required this.id, required this.title});
  final String id;
  final String title;
}

abstract interface class NoteRepository {
  Future<List<Note>> list();
  Future<void> create(String title);
}
