# Tasks

- [ ] Report both defects with exact lines: kbd-reflect never writes .phase; kbd-next-phase.sh:270 writes a self-referential next
- [ ] State the one-line fix each needs, ready to apply where those skills are authored
- [ ] Add a check script IN THIS REPO that exits non-zero when .phase disagrees with the active phase dir
- [ ] Same check exits non-zero when next is self-referential
- [ ] Do NOT patch the installed skills from here — the next install destroys such edits
