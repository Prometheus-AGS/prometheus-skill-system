// TJ-ARCH-MOB-001 compliant
import { useNoteDraft } from '../stores/notes';
import { useNoteEntities } from '../entities/useNoteEntities';
export function useNotes() {
  const { notes, capabilities, mutation } = useNoteEntities();
  const { draft, setDraft } = useNoteDraft();
  const save = async () => { const result = await mutation.mutate(draft); if (result) setDraft(''); };
  return { capabilities: capabilities.items, capabilityError: capabilities.error?.message, items: notes.items, loading: notes.isLoading, error: mutation.state.error ?? notes.error?.message, saving: mutation.state.isPending, draft, setDraft, save };
}
