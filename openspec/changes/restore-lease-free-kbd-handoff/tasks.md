## 1. Skill

- [x] 1.1 Write `skills/kbd-handoff/kbd-handoff.sh` (read-only state reads, preconditions, target checks, atomic packet and sidecar).
- [x] 1.2 Write `skills/kbd-handoff/SKILL.md` and `references/schemas/harness-handoff.schema.json`.

## 2. Registration and docs

- [x] 2.1 Register the skill in the orchestrator SKILL.md, `workflows.yaml` and the generated slash commands.
- [x] 2.2 Update `references/cross-tool-handoff.md` and add `site/docs/kbd/harness-handoffs.md` to the KBD sidebar.
- [x] 2.3 Regenerate the skill distribution.

## 3. Verification

- [x] 3.1 Add `shared/lib/tests/test-kbd-handoff.sh`: refusal on open obligations, dry run writes nothing, packet sections and schema, unknown harness, broken skill symlink.
- [x] 3.2 Run strict skill validation, progress-signal lint, the fixture, the distribution check and the KBD control-plane suite locally.
