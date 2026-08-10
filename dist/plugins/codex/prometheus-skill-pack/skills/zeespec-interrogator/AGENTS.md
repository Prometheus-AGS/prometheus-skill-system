# Contributing to ZeeSpec Interrogator

Guidelines for contributors working on this skill.

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add healthcare domain enrichment to Who dimension
fix: correct coverage threshold logic for partial-dimension scoring
docs: clarify integration-contract for kbd caller protocol
refactor: extract common question-classification logic
chore: bump version in SKILL.md and plugin.json
```

## Branch Strategy

- `main` — Stable, release-ready
- `feat/*` — Feature branches
- `fix/*` — Bug fix branches

## Pull Request Checklist

- [ ] YAML frontmatter present on all `.md` skill files
- [ ] Dimension files still contain exactly 10 questions (Q1–Q10)
- [ ] `constraint-manifest.schema.json` version bumped if schema changed
- [ ] `integration-contract.md` updated if manifest output changed
- [ ] `coverage-scoring.md` and `score-coverage.sh` are in sync with SKILL.md thresholds
- [ ] All `references/` paths resolve to real files
- [ ] Hook scripts are executable (`chmod +x scripts/*.sh`)
- [ ] JSON schemas validate with draft-07
- [ ] No caller-specific logic added to dimension question files

## Architecture Overview

See `CLAUDE.md` for development guidelines.
See `README.md` for project overview.
See `SKILL.md` for the canonical skill definition.
See `references/integration-contract.md` for the caller protocol.
