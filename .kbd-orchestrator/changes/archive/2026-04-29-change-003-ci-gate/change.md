# change-003-ci-gate

**Phase:** phase-corpus-strict-compliance
**Status:** PENDING
**Gaps:** G3 (CI gate documentation)
**Effort:** XS

## Goal

Update `CLAUDE.md` to require `validate:strict` for new skills and reflect the three required
strict fields (`version`, `license`, `metadata.tags`) in contributing docs and the publishing checklist.

## Files

- `CLAUDE.md` — update Validation, Publishing Checklist, Skill Development Workflow, AgentSkills.io Compliance sections

## Tasks

- [ ] Add `npm run validate:strict` to Essential Commands validation section
- [ ] Update Publishing Checklist: replace `validate` with `validate:strict`
- [ ] Update Skill Development Workflow "Validate" step to mention `validate:strict`
- [ ] Add `version` and `metadata.tags` to AgentSkills.io Compliance required elements list
- [ ] Run `npm run validate` → still 0 errors

## Acceptance Criteria

1. `CLAUDE.md` contributing workflow explicitly requires `validate:strict` for new skills
2. Publishing checklist uses `validate:strict`
3. `version` and `metadata.tags` listed as required fields in AgentSkills.io Compliance section
