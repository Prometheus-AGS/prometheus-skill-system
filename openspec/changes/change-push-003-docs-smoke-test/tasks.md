# Tasks: change-push-003-docs-smoke-test

- [ ] Add ## Updating the Skill Pack section to skills/process/cowork-management/references/COMMANDS.md
- [ ] Document three update flows: skills-only (cowork pack update), full update (git pull --recurse-submodules + install-binaries.sh + install-skills-flat.sh), binary-only (bash scripts/install-binaries.sh)
- [ ] Build cowork v0.2.0 from source (tools/cowork-skills/cli) and install to ~/.local/bin/cowork
- [ ] Run smoke tests: cowork --version (expect 0.2.0), cowork pack status, cowork toolchain status
- [ ] Record smoke test results in COMMANDS.md
- [ ] Commit documentation changes
- [ ] Update KBD orchestrator to 3/3 and mark phase complete
