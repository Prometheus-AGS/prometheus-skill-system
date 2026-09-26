// TJ-ARCH-MOB-001 compliant
import { registerEntityTransport, type ChangeEvent } from '@prometheus-ags/prometheus-entity-management';
import { create } from 'zustand';
import { notesRepository, listCapabilities } from '../api/notes';
import type { Capability, Note } from '../types';
const listeners = new Set<(event: ChangeEvent<Note>) => void>();
export function registerNotes() {
  registerEntityTransport<Note>('Note', { identify: note => note.id, authoritative: false,
    list: async query => { const rows = await notesRepository.list(query.signal); return { rows, total: rows.length }; },
    subscribe: listener => { listeners.add(listener); return () => { listeners.delete(listener); }; },
  });
  registerEntityTransport<Capability>('Capability', { identify: capability => capability.id, authoritative: true,
    list: async query => { query.signal?.throwIfAborted(); const rows = await listCapabilities(); query.signal?.throwIfAborted(); return { rows, total: rows.length }; },
  });
}
export const useNoteDraft = create<{ draft: string; setDraft: (draft: string) => void }>(set => ({ draft: '', setDraft: draft => set({ draft }) }));
export async function createNote(title: string): Promise<Note> {
  const note = await notesRepository.create(title);
  for (const listener of listeners) listener({ op: 'insert', id: note.id, row: note });
  return note;
}
