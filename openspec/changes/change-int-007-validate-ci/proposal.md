---
id: change-int-007-validate-ci
title: Final validate + CI update + npm build
phase: cowork-integration
priority: P1
effort: S
wave: 5
agent: general-purpose
status: done
gap_id: G-05-integration
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - .github/workflows/validate.yml (submodule checkout + cargo check steps)
  - openspec/changes/change-int-007-validate-ci (this change)
---

# change-int-007 — Final validate + CI update + npm build

## Context

All 23 prior changes are complete. This final change validates the full state of
the skill-pack, updates CI to properly initialize the two new tool submodules
(tools/disk-space-guardian and tools/cowork-skills), rebuilds the marketplace
symlinks with `npm run build`, and marks the cowork-integration phase execute
stage complete at 24/24.

## Strategy

1. Create OpenSpec files (this file + tasks.md)
2. Run `npm run validate:strict` to confirm all native skills pass
3. Update `.github/workflows/validate.yml`:
   - Ensure submodule checkout initializes tools/disk-space-guardian and tools/cowork-skills
   - Add cargo-check step for tool submodules that have Cargo.toml
4. Run `npm run build` to rebuild marketplace symlinks
5. Commit all changes
6. Update KBD orchestrator to 24/24 (phase execute complete)

## Scope

1. Create OpenSpec proposal.md + tasks.md
2. Run npm run validate:strict (must pass)
3. Update .github/workflows/validate.yml for submodule CI
4. Run npm run build + final commit
