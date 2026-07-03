# Tasks: change-credibility-006-drift-readback

- [ ] Add `load_stale_skills(forge_dir: &Path, language: &Language) -> HashSet<String>` to `forge-enricher/src/lib.rs`
- [ ] Handle missing `.forge/memory/drift/` directory gracefully (return empty set)
- [ ] Parse drift JSON files: filter by language prefix, extract skills with acceptance_rate < 0.5
- [ ] Call `load_stale_skills` in `Enricher::enrich()` before `skill_registry.resolve()`
- [ ] Add `tracing::warn!` with stale skill names when set is non-empty
- [ ] Run `cargo build --workspace` — clean
- [ ] Test: drift dir absent → no warning; drift file with stale skill → warning logged
