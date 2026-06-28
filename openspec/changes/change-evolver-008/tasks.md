# Tasks — change-evolver-008

- [ ] 1. Write `skills/process/pmpo-evolver/references/strategic-dreaming.md` with full protocol, output format, and dreaming prompt
- [ ] 2. Verify strategic-dreaming.md clearly distinguishes from PMPO Reflect and KBD Reflect
- [ ] 3. Write `skills/process/pmpo-evolver/scripts/post-cycle-dream.sh` per proposal
- [ ] 4. Set executable: `chmod +x skills/process/pmpo-evolver/scripts/post-cycle-dream.sh`
- [ ] 5. Test graceful skip: `bash skills/process/pmpo-evolver/scripts/post-cycle-dream.sh nonexistent-evolution` exits 0 with "skipping" message
- [ ] 6. Verify `[MODEL_ROUTING] phase=evolver-strategic-dream class=frontier` comment present
- [ ] 7. Verify context management section in strategic-dreaming.md explains isolated subprocess invocation
