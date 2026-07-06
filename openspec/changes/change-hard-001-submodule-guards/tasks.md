# Tasks — change-hard-001-submodule-guards

- [x] Fix line 47: replace `[ -d "${REPO_ROOT}/tools/prometheus-knowledge" ]` with `[ -f "${REPO_ROOT}/tools/prometheus-knowledge/Cargo.toml" ]`
- [x] Fix line 58: replace `[ -d "${REPO_ROOT}/tools/liter-llm" ]` with `[ -f "${REPO_ROOT}/tools/liter-llm/Cargo.toml" ]`
- [x] Fix line 77: replace `[ -d "${REPO_ROOT}/tools/surreal-memory-server" ]` with `[ -f "${REPO_ROOT}/tools/surreal-memory-server/Cargo.toml" ]`
- [x] Run `bash scripts/install-binaries.sh` and verify it completes without aborting
- [x] Verify `dsg --version` returns expected version
- [x] Commit the change

## Notes

Also fixed two additional uninitialized submodule guards discovered during verification:
- `skills/imported/sycophancy-correction` (line 102)
- `skills/imported/artifact-refiner/tools/template-forge-rs` (line 122)

5 guards fixed total. Commit: b3fa3dd
