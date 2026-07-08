# Plan — phase-deep-research-skill

_Generated: 2026-07-08 · Change backend: OpenSpec (openspec/ directory detected)_

## Summary

9 changes building the complete `skills/research/deep-research/` skill from scratch.
All infrastructure is adopted (8 adopt, 1 defer). The deliverable is 38 files:
1 parent `SKILL.md`, 10 stage sub-skill `SKILL.md` files, `skill.toml`, 5 scripts,
5 templates, 9 references, 4 hooks, 4 agent definitions, plus 3 documentation
index updates and a commit/push.

No Rust code is written. No binary is built. The native `prometheus-research`
binary is deferred to `phase-prometheus-research-binary`.

## Change Order

| Order | Change ID | Description | Goals | Risk | Agent |
|-------|-----------|-------------|-------|------|-------|
| 1 | `change-drs-001` | Create directory structure + `skill.toml` | G-01 | LOW | general-purpose |
| 2 | `change-drs-002` | Write parent `deep-research/SKILL.md` | G-02 | LOW | general-purpose |
| 3 | `change-drs-003` | Write all 10 stage sub-skill `SKILL.md` files | G-03 | LOW | general-purpose |
| 4 | `change-drs-004` | Write 5 scripts + 5 templates | G-04 | LOW | general-purpose |
| 5 | `change-drs-005` | Write 9 references + 4 hooks + 4 agent definitions | G-04 | LOW | general-purpose |
| 6 | `change-drs-006` | Run validation + fix any errors | G-05 | LOW | general-purpose |
| 7 | `change-drs-007` | Update `SKILLS.md`, `README.md`, `docs/deep-research/index.md` | G-06 | LOW | general-purpose |
| 8 | `change-drs-008` | Install to `~/.claude/skills/` + smoke test trigger | G-05 | LOW | general-purpose |
| 9 | `change-drs-009` | Commit + push | G-07 | LOW | general-purpose |

**Ordering rationale:**
- Changes 1–5 are purely additive file creation — sequentially building the skill.
- Change 1 must precede all others (creates the directory tree).
- Change 2 (parent SKILL.md) must precede Change 3 (sub-skills reference the parent).
- Changes 4 and 5 are independent of each other and could parallelize, but are kept
  sequential to simplify tracking. Combined effort: ~2 hours.
- Change 6 (validation) gates Change 8 (install).
- Change 7 (docs updates) and Change 8 (install) are independent; 7 first for commit hygiene.
- Change 9 (commit/push) always last.

## Goals Mapping

| Goal | Changes | Criterion |
|------|---------|-----------|
| G-01: Directory structure | 1 | `find skills/research/deep-research -type d` shows all 12 dirs |
| G-02: Parent SKILL.md | 2 | Frontmatter valid, triggers defined, pipeline documented |
| G-03: 10 sub-skill SKILL.md files | 3 | Each has input/output contracts and integration refs |
| G-04: Scripts/templates/refs/hooks/agents | 4, 5 | All 27 supporting files present, scripts executable |
| G-05: Validation passes | 6, 8 | `npm run validate:strict skills/research/deep-research` exits 0 |
| G-06: Docs updated | 7 | Research category appears in SKILLS.md and README |
| G-07: Committed and pushed | 9 | Commit on main, `git log --oneline -1` shows feat: |

## No evolver bridge

This phase is not driven by an iterative-evolver cycle.
