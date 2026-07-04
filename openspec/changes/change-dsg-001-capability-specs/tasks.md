# Tasks: change-dsg-001-capability-specs

- [x] Read dsg `docs/README.md` to understand Phase 1 CLI surface and design
- [x] Read `CLAUDE.md` / `AGENTS.md` in dsg repo for project context
- [x] Check dsg `openspec/` structure (changes and specs directories)
- [x] Create `openspec/specs/` directory in dsg repo
- [x] Write `openspec/specs/cli.md` — all Phase 1 commands with flags, output formats, exit codes
- [x] Write `openspec/specs/config.md` — TOML schema with field reference and loading behavior
- [x] Write `openspec/specs/safety.md` — 7 rules, ordered pipeline, trash failure semantics, audit log
- [x] Write `openspec/specs/scanner.md` — ScanResult, EcosystemDetector trait, output formats, performance target
- [x] Write `docs/decisions.md` — bind D-01 (lsof TOCTOU), D-02 (symlink), D-03 (trash failure), D-04 (mtime)
- [x] Populate dsg `openspec/changes/change-001-establish-capability-specs/` with `proposal.md` and `tasks.md`
- [x] Commit all new files in dsg repo
