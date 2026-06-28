# Tasks — change-evolver-006

- [ ] 1. Create `skills/process/pmpo-evolver/skills/validate-idea/` directory
- [ ] 2. Write `skills/process/pmpo-evolver/skills/validate-idea/SKILL.md` per proposal (all three gates, archive protocol)
- [ ] 3. Run `npm run validate:strict skills/process/pmpo-evolver/skills/validate-idea` — must pass
- [ ] 4. Write `skills/process/pmpo-evolver/scripts/idea-gate-1.sh` per proposal
- [ ] 5. Set executable: `chmod +x skills/process/pmpo-evolver/scripts/idea-gate-1.sh`
- [ ] 6. Test rejection: `bash skills/process/pmpo-evolver/scripts/idea-gate-1.sh "add rust skills" default` exits 1
- [ ] 7. Test pass: `bash skills/process/pmpo-evolver/scripts/idea-gate-1.sh "add quantum computing bridge" default` exits 0
- [ ] 8. Verify all three gates have `[MODEL_ROUTING]` directives in validate-idea/SKILL.md
- [ ] 9. Verify archive manifest format is documented
