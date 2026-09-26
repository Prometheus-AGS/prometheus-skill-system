// TJ-ARCH-MOB-001 compliant
// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'notes.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(noteRepository)
final noteRepositoryProvider = NoteRepositoryProvider._();

final class NoteRepositoryProvider
    extends
        $FunctionalProvider<
          AsyncValue<NoteRepository>,
          NoteRepository,
          FutureOr<NoteRepository>
        >
    with $FutureModifier<NoteRepository>, $FutureProvider<NoteRepository> {
  NoteRepositoryProvider._()
    : super(
        from: null,
        argument: null,
        retry: noRetry,
        name: r'noteRepositoryProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$noteRepositoryHash();

  @$internal
  @override
  $FutureProviderElement<NoteRepository> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<NoteRepository> create(Ref ref) {
    return noteRepository(ref);
  }
}

String _$noteRepositoryHash() => r'ea12630fc402c1333d3859d6885822700b2f6226';

@ProviderFor(Notes)
final notesProvider = NotesProvider._();

final class NotesProvider extends $AsyncNotifierProvider<Notes, List<Note>> {
  NotesProvider._()
    : super(
        from: null,
        argument: null,
        retry: noRetry,
        name: r'notesProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$notesHash();

  @$internal
  @override
  Notes create() => Notes();
}

String _$notesHash() => r'dd2566849b5de0cb9a347c48c195828a9987ce5a';

abstract class _$Notes extends $AsyncNotifier<List<Note>> {
  FutureOr<List<Note>> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AsyncValue<List<Note>>, List<Note>>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<List<Note>>, List<Note>>,
              AsyncValue<List<Note>>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
