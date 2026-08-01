# Tasks

- [x] Add a pub skill facade to lib.rs: list / get / install / toggle / query
- [~] **NOT DONE AS WRITTEN** — `uar::runtime::skills` internals were already fully public (`pub` at every level). 16 files in `src/` and 6 integration tests import them; narrowing would break the build and is a deprecation with its own migration. What shipped instead is the *seam* (`SkillsApi`) that makes a future narrowing possible. Documented in `skills/mod.rs`.
- [x] Integration test in tests/ consumes ONLY the public API, as an embedder would
