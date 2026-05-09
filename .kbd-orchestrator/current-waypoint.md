# Current Waypoint

**Phase**: `phase-a2ui-agui-artifact-refiner`
**Stage**: executing — 2 of 3 changes branch-pushed; 1 pending upstream merge + tag
**Backend**: split (OpenSpec upstream + native-kbd here)
**Previous phase**: `phase-corpus-strict-compliance` (complete)
**Last updated**: 2026-05-09

## Goal

Deliver A2UI domain completion (and AG-UI extension) inside the upstream
`GQAdonis/artifact-refiner-skill` repo, then bump the submodule pointer in this
skill pack. No coupling to TheBoss; no new submodule for cherry-studio.

## Where we are

- Assessment: [`.kbd-orchestrator/phases/phase-a2ui-agui-artifact-refiner/assessment.md`](phases/phase-a2ui-agui-artifact-refiner/assessment.md)
- Plan: [`.kbd-orchestrator/phases/phase-a2ui-agui-artifact-refiner/plan.md`](phases/phase-a2ui-agui-artifact-refiner/plan.md)
- Progress: [`.kbd-orchestrator/phases/phase-a2ui-agui-artifact-refiner/progress.json`](phases/phase-a2ui-agui-artifact-refiner/progress.json)
- change-003 dossier: [`.kbd-orchestrator/changes/change-003-bump-artifact-refiner-pointer/change.md`](changes/change-003-bump-artifact-refiner-pointer/change.md)

## Change status

| # | Change | Backend | Repo | Status |
|---|---|---|---|---|
| 1 | change-001-finish-a2ui-domain | OpenSpec | upstream artifact-refiner | **MERGED** — PR #1, SHA 522d3da |
| 2 | change-002-agui-spike-and-domain | OpenSpec | upstream artifact-refiner | **BRANCH-PUSHED** — blocked on user authorization to open PR |
| 3 | change-003-bump-artifact-refiner-pointer | native-kbd | this repo | PENDING — gated on change-002 merge + v1.2.0 tag |

## Next action

**Step 1**: Open PR for change-002:
```
https://github.com/GQAdonis/artifact-refiner-skill/pull/new/agui-spike-and-domain
```

**Step 2**: After PR merges, cut tag `v1.2.0` on upstream main (covers both A2UI + AG-UI):
```
cd skills/imported/artifact-refiner
git checkout main && git pull origin main
git tag v1.2.0 && git push origin v1.2.0
```

**Step 3**: Run change-003:
```
/kbd-execute change-003-bump-artifact-refiner-pointer
```

## Phase exit criteria

- A2UI refinement runs end-to-end on a demo spec ✅ (change-001 merged)
- AG-UI spike delivered a working example ✅ (change-002 branch-pushed)
- `skills/imported/artifact-refiner` points at v1.2.0 tag ⬜ (change-003 pending)
- `npm run validate` exits 0 ⬜ (after change-003)
- `/refine-a2ui` works in Claude Code ⬜ (after change-003)

## Out of scope (explicit)

- TheBoss / cherry-studio integration — separate ticket in `Know-Me-Tools/the-boss`
- New `ui-component-pipeline` orchestrator skill
- Making the refiner emit live AG-UI events
- Any cherry-studio submodule
