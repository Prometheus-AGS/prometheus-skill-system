// TJ-ARCH-MOB-001 compliant
import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteRepository } from '../types';
export const notesRepository: NoteRepository = {
  list: async signal => { signal?.throwIfAborted(); const notes = await invoke<Note[]>('list_notes'); signal?.throwIfAborted(); return notes; },
  create: title => invoke<Note>('create_note', { title }),
};

export async function listCapabilities(): Promise<import('../types').Capability[]> {
  const response = await fetch('/capabilities/index.json');
  if (!response.ok) throw new Error('Could not load capability registry');
  const local = await response.json() as { schemaVersion: number; capabilities: import('../types').Capability[] };
  if (local.schemaVersion !== 1) throw new Error('Unsupported capability registry schema');
  const backend = await invoke<import('../types').Capability[]>('list_capabilities');
  return [...new Map([...local.capabilities, ...backend].map(item => [item.id, item])).values()];
}
