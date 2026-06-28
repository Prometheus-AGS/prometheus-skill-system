# Tasks — change-evolver-009

- [ ] 1. Read `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json`
- [ ] 2. Add `perspective` field (7-value enum, default "auto") to root properties
- [ ] 3. Validate: `python3 -m json.tool skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` exits 0
- [ ] 4. Read `skills/process/pmpo-outer-loop/scripts/loop-tick.sh`
- [ ] 5. Add perspective extraction logic and conditional `--perspective` flag pass-through
- [ ] 6. Verify backward-compatible: script with no `perspective` field in loop.json behaves identically to before
- [ ] 7. Read `skills/process/pmpo-outer-loop/SKILL.md`
- [ ] 8. Add perspective paragraph under `/loop-define` section with example loop.json
- [ ] 9. Verify `pmpo-outer-loop/SKILL.md` does not exceed original structure (no unrelated edits)
