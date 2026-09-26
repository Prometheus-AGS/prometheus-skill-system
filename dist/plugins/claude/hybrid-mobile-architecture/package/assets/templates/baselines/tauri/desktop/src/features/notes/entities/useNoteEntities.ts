// TJ-ARCH-MOB-001 compliant
import { useEntityQuery, useEntityMutation } from '@prometheus-ags/prometheus-entity-management';
import { createNote } from '../stores/notes';
import type { Capability, Note } from '../types';
export function useNoteEntities() {
  const notes = useEntityQuery<Note>('Note');
  const capabilities = useEntityQuery<Capability>('Capability');
  const mutation = useEntityMutation<string, Note, Note>({ type: 'Note', mutate: createNote, normalize: note => ({ id: note.id, data: note }) });
  return { notes, capabilities, mutation };
}
