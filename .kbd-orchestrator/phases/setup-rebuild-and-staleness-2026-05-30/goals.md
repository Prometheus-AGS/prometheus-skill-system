# Goals

- Add prometheus setup --rebuild flag that forces cargo build+install of all 4 binaries regardless of detected presence
- Add per-component staleness detection comparing binary mtime to submodule HEAD commit time
- Stale components surface as ComponentStatus::Stale with appropriate UX in setup output
- prometheus setup --check reports stale separately from missing
- Unit tests cover the staleness comparator
