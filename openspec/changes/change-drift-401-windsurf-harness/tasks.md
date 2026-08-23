## 1. Implementation

- [ ] 1.1 DECIDE A (regenerate) or B (retire); record the rationale in this change
- [ ] 1.2A Regenerate .windsurf/skills; confirm the count matches sibling harnesses
- [ ] 1.2B Remove skill-system.json:144 and resolve every other .windsurf/skills reference
- [ ] 1.3B Re-run and commit the output of generate-skill-system-distribution.js, install-system.js, skill-matrix.js

## 2. Verification

- [ ] 2.1 The decision and rationale are recorded
- [ ] 2.2 (A) .windsurf/skills present with a sibling-matching count
- [ ] 2.3 (B) `grep -rn ".windsurf/skills"` outside .kbd-orchestrator/ returns nothing unresolved
- [ ] 2.4 (B) Distribution output regenerated and committed — no drift
- [ ] 2.5 .windsurf/workflows (19 files) untouched
