// TJ-ARCH-MOB-001 compliant
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:path_provider/path_provider.dart';

import '../../data/rust_note_repository.dart';
import '../../domain/note.dart';
part 'notes.g.dart';

Duration? noRetry(int count, Object error) => null;
@Riverpod(keepAlive: true, retry: noRetry)
Future<NoteRepository> noteRepository(Ref ref) async {
  final directory = await getApplicationSupportDirectory();
  return RustNoteRepository('${directory.path}/notes.sqlite3');
}

@Riverpod(retry: noRetry)
class Notes extends _$Notes {
  @override
  Future<List<Note>> build() async =>
      (await ref.watch(noteRepositoryProvider.future)).list();
  Future<void> create(String title) async {
    final repository = await ref.read(noteRepositoryProvider.future);
    await repository.create(title);
    final notes = await repository.list();
    if (ref.mounted) state = AsyncData(notes);
  }
}
