---
id: change-learn-024
title: learn-about-system + learn-harness integration test
type: test
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-016
  - change-learn-017
  - change-learn-018
---

# change-learn-024 — meta-skill integration test

## Summary

Write an integration test covering `learn-about-system` and `learn-harness`.
Asserts that `--area kbd` routes to `learn-goal` with `kbd-lifecycle-corpus.json`
as the active KB, that `--harness claude-code` produces a capability map
output, and that both corpus files pass schema validation with expected concept
IDs and skill category entries.

## Motivation

`learn-about-system` and `learn-harness` are routing skills — their primary
contract is correct dispatch and corpus selection. Integration tests here catch
routing regressions without requiring a full Feynman loop run.

## Scope

- New test: `tests/learn/integration-meta.sh`
- Validates routing, capability map output, and corpus schema
- No Feynman loop execution required in this test

## Tasks

- [x] Write `tests/learn/integration-meta.sh`: invoke `learn-about-system --area kbd` in dry-run/stub mode (or with a minimal mock for `learn-goal`), capture stdout, and assert that the output references `kbd-lifecycle-corpus.json` as the active corpus
- [x] Assert `learn-harness --harness claude-code` produces capability map output: invoke with `--map-only`, capture stdout, assert that the output contains sections for "Skills", "MCP", "Hooks", and "AskUserQuestion"
- [x] Validate `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` schema: run `npm run validate:strict` or a `jsonschema` check against `grounding-corpus.schema.json` and assert exit 0; also assert that the concept IDs `kbd.assess`, `kbd.analyze`, `kbd.plan`, `kbd.execute`, `kbd.reflect`, `kbd.evolve` are all present
- [x] Validate `docs/learn/meta-corpus/skill-pack-corpus.json` schema: same schema check, and assert that concept entries cover at least the skill category names (`react`, `rust`, `ui-ux`, `devops`, `testing`, `documentation`, `learn`)
- [x] Add routing smoke test for `--area skills`: invoke `learn-about-system --area skills` and assert that the output lists at least three skill domain names without erroring
