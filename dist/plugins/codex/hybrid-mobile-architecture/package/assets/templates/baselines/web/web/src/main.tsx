// TJ-ARCH-MOB-001 compliant
import { createRoot } from 'react-dom/client';
import { NotesScreen } from './features/notes/components/NotesScreen';
import { registerNotes } from './features/notes/stores/notes';
import './style.css';
registerNotes();
const root = document.getElementById('root');
if (!root) throw new Error('Missing application root');
createRoot(root).render(<NotesScreen />);
