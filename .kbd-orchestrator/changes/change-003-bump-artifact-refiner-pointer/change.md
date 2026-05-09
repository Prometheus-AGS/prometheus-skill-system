# change-003 — bump-artifact-refiner-pointer

**Phase**: phase-a2ui-agui-artifact-refiner
**Backend**: native-kbd
**Repo**: prometheus-skill-pack
**Depends on**: change-001-finish-a2ui-domain (change-002 optional)

## Why

Once upstream `GQAdonis/artifact-refiner-skill` ships the A2UI completion (and optionally AG-UI), this repo needs to point at the new tag so users of the skill pack get the new domain support.

## What changes

- `skills/imported/artifact-refiner` submodule pointer → upstream tag (e.g. `v1.2.0` for A2UI-only, `v1.3.0` if AG-UI also ships).
- `marketplace/marketplace.json` if per-skill tags or domain lists need refreshing.

## Tasks

- [ ] `cd skills/imported/artifact-refiner && git fetch origin && git checkout <tag>`
- [ ] `cd ../../.. && git add skills/imported/artifact-refiner`
- [ ] Refresh `marketplace/marketplace.json` if needed
- [ ] `npm run validate` (lenient mode — covers imported skills)
- [ ] `npm run build` — verify `.claude-plugin/` symlinks rebuild cleanly
- [ ] Smoke test `/refine-a2ui` in Claude Code; AG-UI path if applicable
- [ ] Commit: `chore(submodules): bump artifact-refiner to <tag> (A2UI + AG-UI domains)`

## Verification

- `git submodule status skills/imported/artifact-refiner` shows the expected SHA
- `npm run validate` exits 0
- `npm run build` produces no symlink errors
- A2UI (and AG-UI if applicable) detection trigger works end-to-end

## Out of scope

- Any source edits inside the submodule (those happen in change-001/002 upstream)
- Adding new sibling skills here
- TheBoss / cherry-studio integration
