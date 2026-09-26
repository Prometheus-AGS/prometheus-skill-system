// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/notes.dart';
import '../providers/capabilities.dart';

class NotesScreen extends ConsumerStatefulWidget {
  const NotesScreen({super.key});
  @override
  ConsumerState<NotesScreen> createState() => _NotesScreenState();
}

class _NotesScreenState extends ConsumerState<NotesScreen> {
  final controller = TextEditingController();
  bool saving = false;
  String? error;
  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  Future<void> save() async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await ref.read(notesProvider.notifier).create(controller.text);
      if (mounted) controller.clear();
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) setState(() => saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final notes = ref.watch(notesProvider);
    final capabilities = ref.watch(capabilitiesProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Your notes')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text('A small place for things worth keeping.'),
              const SizedBox(height: 24),
              TextField(
                key: const Key('note-title'),
                controller: controller,
                decoration: const InputDecoration(
                  labelText: 'New note',
                  filled: true,
                ),
                onSubmitted: (_) {
                  if (!saving) {
                    save();
                  }
                },
              ),
              const SizedBox(height: 12),
              FilledButton(
                key: const Key('save-note'),
                onPressed: saving ? null : save,
                child: Text(saving ? 'Saving…' : 'Save note'),
              ),
              if (error != null)
                Semantics(liveRegion: true, child: Text(error!)),
              capabilities.when(
                data: (items) => items.isEmpty
                    ? const SizedBox.shrink()
                    : Text(
                        'Available capabilities: '
                        '${items.map((item) => item.name).join(', ')}',
                      ),
                error: (error, stack) =>
                    Text('Could not load capabilities: $error'),
                loading: () => const SizedBox.shrink(),
              ),
              const SizedBox(height: 24),
              Expanded(
                child: notes.when(
                  data: (items) => items.isEmpty
                      ? const Text('Your saved notes will appear here.')
                      : ListView.builder(
                          itemCount: items.length,
                          itemBuilder: (context, index) =>
                              ListTile(title: Text(items[index].title)),
                        ),
                  error: (failure, stack) =>
                      Text('Could not load notes: $failure'),
                  loading: () =>
                      const Center(child: CircularProgressIndicator()),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
