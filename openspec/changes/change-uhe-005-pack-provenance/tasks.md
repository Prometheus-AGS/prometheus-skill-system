# Tasks

- [ ] Pack emits a version manifest (EXTEND skills-index; do not invent a parallel file)
- [ ] UAR reads it at load WITHOUT shelling out to git (impossible on mobile)
- [ ] Expose version + commit + skill count via a GET endpoint
- [ ] Test asserts the reported version changes when the manifest changes
- [ ] Confirm the 359-commit drift would have been visible through this endpoint
