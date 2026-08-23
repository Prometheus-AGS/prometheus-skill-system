## 1. Implementation

- [x] 1.1 DECIDE A (regenerate) or B (retire); record the rationale in this change
- [x] 1.2A Regenerate .windsurf/skills; confirm the count matches sibling harnesses
- [x] 1.2B Remove skill-system.json:144 and resolve every other .windsurf/skills reference
- [x] 1.3B Re-run and commit the output of generate-skill-system-distribution.js, install-system.js, skill-matrix.js

## 2. Verification

- [x] 2.1 The decision and rationale are recorded
- [x] 2.2 (A) .windsurf/skills present with a sibling-matching count
- [x] 2.3 (B) `grep -rn ".windsurf/skills"` outside .kbd-orchestrator/ returns nothing unresolved
- [x] 2.4 (B) Distribution output regenerated and committed — no drift
- [x] 2.5 .windsurf/workflows (19 files) untouched
