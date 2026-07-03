# Tasks: change-credibility-008-unit-tests

- [ ] Replace `→` with `->` in `forge-enricher/src/lib.rs:16` doc comment
- [ ] Run `cargo test --workspace` to confirm doctest failure is fixed
- [ ] Add ≥15 `#[cfg(test)]` tests across forge-enricher, forge-core, forge-reflect, forge-skills
- [ ] Cover: language detection, constitution checking, skill registry operations, task model, drift path
- [ ] All tests pass: `cargo test --workspace` green
- [ ] Confirm test count: `cargo test --workspace 2>&1 | grep "test result"`
