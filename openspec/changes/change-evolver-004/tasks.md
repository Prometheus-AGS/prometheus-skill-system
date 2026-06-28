# Tasks — change-evolver-004

- [ ] 1. Write `skills/process/pmpo-evolver/references/competitive-analysis.md` with registry + parity matrix formats and ingestion protocol
- [ ] 2. Write `skills/process/pmpo-evolver/scripts/competitor-registry-init.sh` per proposal
- [ ] 3. Set executable: `chmod +x skills/process/pmpo-evolver/scripts/competitor-registry-init.sh`
- [ ] 4. Write `skills/process/pmpo-evolver/scripts/changelog-fetch.sh` per proposal
- [ ] 5. Set executable: `chmod +x skills/process/pmpo-evolver/scripts/changelog-fetch.sh`
- [ ] 6. Smoke test registry init: `bash skills/process/pmpo-evolver/scripts/competitor-registry-init.sh test-evol` exits 0 and creates `.evolver/test-evol/competitor-registry.json`
- [ ] 7. Smoke test changelog fetch: script exits 0 (or fails gracefully) when gh is available
- [ ] 8. Clean up test artifacts: `rm -rf .evolver/test-evol`
