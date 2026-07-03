# Tasks: change-credibility-010-bdd-tests

- [ ] Add `@cucumber/cucumber@^11`, `@types/node`, `ts-node` devDependencies to `package.json`
- [ ] Create `tests/features/forge-validate.feature` with clean-file and violation scenarios
- [ ] Create `tests/features/forge-enrich.feature` with JSON output and path traversal scenarios
- [ ] Create `tests/fixtures/` directory with fixture Rust source files and sample constitution
- [ ] Create `tests/steps/forge-steps.ts` with Given/When/Then step definitions using spawnSync
- [ ] Add `"cucumber": "cucumber-js tests/features/**/*.feature --require-module ts-node/register --require tests/steps/**/*.ts"` script to `package.json`
- [ ] Build forge binary: `cargo build --manifest-path tools/forge-rs/Cargo.toml`
- [ ] Run `npm run cucumber` — all scenarios pass
- [ ] Verify path traversal scenario exits non-zero
