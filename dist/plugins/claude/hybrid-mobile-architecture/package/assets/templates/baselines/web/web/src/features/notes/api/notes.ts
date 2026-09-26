// TJ-ARCH-MOB-001 compliant
import type { Note, NoteRepository } from '../types';
async function request<T>(init?: RequestInit): Promise<T> {
  const response = await fetch('/api/notes', init);
  if (!response.ok) throw new Error(await response.text());
  return response.json() as Promise<T>;
}
export const notesRepository: NoteRepository = { list: signal => request<Note[]>({ signal }), create: title => request<Note>({ method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ title }) }) };

export async function listCapabilities(): Promise<import('../types').Capability[]> {
  const response = await fetch('/capabilities/index.json');
  if (!response.ok) throw new Error('Could not load capability registry');
  const local = await response.json() as { schemaVersion: number; capabilities: import('../types').Capability[] };
  if (local.schemaVersion !== 1) throw new Error('Unsupported capability registry schema');
  const backendResponse = await fetch('/api/capabilities'); if (!backendResponse.ok) throw new Error('Could not load backend capabilities'); const backend = await backendResponse.json() as import('../types').Capability[];
  return [...new Map([...local.capabilities, ...backend].map(item => [item.id, item])).values()];
}
