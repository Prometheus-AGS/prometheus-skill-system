# Current Waypoint

**Phase**: `skill-pack-upgrade-2026-05-09`  
**Stage**: PLAN — ready to execute  
**Last updated**: 2026-05-09

## Summary

Phase 1 of the skill-pack upgrade has been assessed and planned. Six changes are queued in OpenSpec, derived from the architectural review in `docs/future-work/`. All six source tasks were confirmed as gaps by the assessment.

Change backend: **OpenSpec** (`openspec/changes/`)

## Change queue

| # | Change | Slot | Agent Role | Status |
|---|--------|------|------------|--------|
| 1 | change-001-bdd006-immutable-tests-rule | 1 (parallel) | docs-writer | ready |
| 2 | change-002-sp015-hooks-json-canonical | 1 (parallel) | skill-pack-maintainer | ready |
| 3 | change-003-bdd001-manifest-dual-key-cleanup | 2 (parallel) | bdd-engineer | ready |
| 4 | change-004-bdd002-flake-quarantine | 2 (parallel) | bdd-engineer | ready |
| 5 | change-005-sp006-stop-hook-observability | 3 | hooks-engineer | ready |
| 6 | change-006-sp013-sycophancy-reflector-hook | 4 (after 5) | hooks-engineer | ready |

## Next action

Run `/kbd-execute change-001-bdd006-immutable-tests-rule` (or dispatch in parallel with change-002) to begin Phase 1 execution.

See `.kbd-orchestrator/phases/skill-pack-upgrade-2026-05-09/plan.md` for full detail.

## References

- [plan.md](phases/skill-pack-upgrade-2026-05-09/plan.md)
- [assessment.md](phases/assess/skill-pack-upgrade-2026-05-09-assessment.md)
- [OpenSpec changes](../openspec/changes/)
