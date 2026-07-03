# Tasks: change-credibility-016-sycophancy-claim-audit

- [ ] Draft bounded production readiness claim (WHAT IS ready vs WHAT IS NOT ready vs BOUNDARY)
- [ ] Write claim to `.kbd-orchestrator/phases/phase-credibility-closure/production-readiness-claim.md`
- [ ] Call `detect_sycophancy` MCP tool with claim text, strictness="strict"
- [ ] If score ≥ 0.15: identify sycophantic patterns from tool response, revise claim, re-run
- [ ] If score < 0.15: record score, tool version, and timestamp in the claim document
- [ ] Phase is COMPLETE when score < 0.15 is achieved and recorded
