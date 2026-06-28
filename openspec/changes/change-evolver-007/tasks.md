# Tasks — change-evolver-007

- [ ] 1. Write `skills/process/pmpo-evolver/references/domain-taxonomy.md` with AI tooling, Rust, LLM infrastructure, developer tooling clusters
- [ ] 2. Verify domain taxonomy includes general detection queries for unknown domains
- [ ] 3. Write `skills/process/pmpo-evolver/scripts/carry-forward-aggregate.sh` per proposal
- [ ] 4. Set executable: `chmod +x skills/process/pmpo-evolver/scripts/carry-forward-aggregate.sh`
- [ ] 5. Smoke test: `bash skills/process/pmpo-evolver/scripts/carry-forward-aggregate.sh .kbd-orchestrator/phases default` exits 0
- [ ] 6. Verify the script finds carry-forwards from existing phase reflections (e.g., pmpo-elicit)
- [ ] 7. Verify output JSON is valid: pipe to `python3 -m json.tool`
- [ ] 8. Verify `[MODEL_ROUTING]` comment is present in the script
