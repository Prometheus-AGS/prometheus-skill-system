# Tasks — change-learn-024

- [ ] Write `tests/learn/integration-meta.sh`: invoke `learn-about-system --area kbd` in dry-run/stub mode (or with a minimal mock for `learn-goal`), capture stdout, and assert that the output references `kbd-lifecycle-corpus.json` as the active corpus
- [ ] Assert `learn-harness --harness claude-code` produces capability map output: invoke with `--map-only`, capture stdout, assert that the output contains sections for "Skills", "MCP", "Hooks", and "AskUserQuestion"
- [ ] Validate `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` schema: run `npm run validate:strict` or a `jsonschema` check against `grounding-corpus.schema.json` and assert exit 0; also assert that the concept IDs `kbd.assess`, `kbd.analyze`, `kbd.plan`, `kbd.execute`, `kbd.reflect`, `kbd.evolve` are all present
- [ ] Validate `docs/learn/meta-corpus/skill-pack-corpus.json` schema: same schema check, and assert that concept entries cover at least the skill category names (`react`, `rust`, `ui-ux`, `devops`, `testing`, `documentation`, `learn`)
- [ ] Add routing smoke test for `--area skills`: invoke `learn-about-system --area skills` and assert that the output lists at least three skill domain names without erroring
