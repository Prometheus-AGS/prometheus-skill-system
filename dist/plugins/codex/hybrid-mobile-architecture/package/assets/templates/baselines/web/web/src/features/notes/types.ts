// TJ-ARCH-MOB-001 compliant
export interface Note { id: string; title: string }
export interface NoteRepository { list(signal?: AbortSignal): Promise<Note[]>; create(title: string): Promise<Note> }

export interface Capability { id: string; name: string; kind: string }
