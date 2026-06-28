# Tasks — change-evolver-001

- [ ] 1. Create directory `skills/process/pmpo-evolver/references/schemas/`
- [ ] 2. Write `skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json` per proposal spec
- [ ] 3. Read `skills/process/iterative-evolver/references/schemas/evolution-state.schema.json`
- [ ] 4. Add `learning_signals` and `perspective` fields to `evolution-state.schema.json` (additive only)
- [ ] 5. Validate both files: `python3 -m json.tool skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json` exits 0
- [ ] 6. Validate: `python3 -m json.tool skills/process/iterative-evolver/references/schemas/evolution-state.schema.json` exits 0
