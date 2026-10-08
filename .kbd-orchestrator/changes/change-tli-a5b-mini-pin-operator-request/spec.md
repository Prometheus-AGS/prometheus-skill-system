# change-tli-a5b-mini-pin-operator-request

**Title:** Operator pin request: mini gitlinks and versions.toml to v1.10.0 (operator-authored)
**Plan PR:** A5b
**Repository:** `prometheus-skills-mini`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a5a-skillpack-pin-v1-10-0`
**Backend:** native-kbd
**Base branch:** no code branch: an operator-authored commit on prometheus-skills-mini `main`.
**Design section:** `docs/design/team-aware-learning-memory.md` §9

## Why

Mini's `versions.toml` is operator-authored (line 1, `docs/versions-toml.md:5`) and `rules/test/versions-toml.test.mjs` requires gitlinks to match it, so the gitlink bump and the versions.toml edit must be one operator commit (assessment round 2, analysis D-3). G1's mini half stays BLOCKED-ON-OPERATOR until it lands.

## What Changes

- Agent prepares the exact request: the two v1.10.0 commits and the precise `versions.toml` diff, in `.kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/a5b-pin-request.md`.
- Agent files it as an issue in prometheus-skills-mini labelled `operator` (outward-facing: confirm with the user before creating it).
- The operator lands the commit; the change completes only when mini `origin/main` carries both gitlinks at v1.10.0.

## Scope

- `.kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/a5b-pin-request.md`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
