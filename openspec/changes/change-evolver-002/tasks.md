# Tasks — change-evolver-002

- [ ] 1. Read `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json`
- [ ] 2. Add `staleness_ttl_minutes` to the base feedback_source object (or each new entry)
- [ ] 3. Add `gh-issues` type entry to feedback_sources oneOf
- [ ] 4. Add `commit-history` type entry
- [ ] 5. Add `sentiment-feed` type entry
- [ ] 6. Add `telemetry-url` type entry
- [ ] 7. Add `competitor-scan` type entry
- [ ] 8. Add `changelog` type entry
- [ ] 9. Validate schema: `python3 -m json.tool skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` exits 0
- [ ] 10. Create `skills/process/pmpo-evolver/references/` directory if needed
- [ ] 11. Write `skills/process/pmpo-evolver/references/feedback-sources.md` with examples for each new type
