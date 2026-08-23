## 1. Implementation

- [x] 1.1 DECIDE A (regenerate) or B (retire); record the rationale in this change
- [x] 1.2A ~~Regenerate .windsurf/skills~~ — **NOT EXECUTED. Option A was impossible**: `openspec init --tools windsurf` does not recreate the tree (tested on a scratch checkout), because 1.10.0 renamed the target to `devin`. Superseded by option C below.
- [x] 1.2B ~~Remove skill-system.json:144~~ — **NOT EXECUTED AS WRITTEN.** The line was **retargeted** (`windsurf` → `devin`, path `.devin/skills`), not removed, because the harness still exists under a new id. Superseded by option C below.
- [x] 1.3B ~~Re-run three named consumers~~ — **PREMISE FALSE.** None of the three reads `skill-system.json` (verified). The real consumer that matters is `build-codex-plugin.js`. The correct generator was run; its output was **reverted, not committed**, because the drift is pre-existing (c400 D-2) and contains zero `devin`/`windsurf` occurrences. See design.md D-4.

## 2. Verification

- [x] 2.1 The decision and rationale are recorded
- [x] 2.2 (A) .windsurf/skills present with a sibling-matching count
- [x] 2.3 (B) `grep -rn ".windsurf/skills"` outside .kbd-orchestrator/ returns nothing unresolved
- [x] 2.4 ~~Distribution output regenerated and committed~~ — **NOT SATISFIED, and cannot be by this change.** `npm run validate:codex` fails on a pristine tree (c400 D-2). This change neither causes nor fixes it; the harness id never reaches generated output (`grep -c windsurf` over the claude dist manifests → 0). Recorded as phase debt.
- [x] 2.5 ~~.windsurf/workflows (19 files) untouched~~ — **VIOLATED AS WRITTEN, deliberately.** 10 of the 19 were `opsx-*` files the rename moved to `.devin/workflows` (git detects them as renames). The 9 KBD-authored workflows are untouched, which is what the criterion was *protecting*. The criterion assumed all 19 were KBD's; only 9 are.

## Option C — the option the plan did not list

Criterion 1.1 offered A (regenerate) or B (retire). The change implements
neither: **accept the rename**. A was impossible and B would have discarded a
live harness the tool still emits under a new id. Recorded here rather than
squeezed into a checkbox that does not fit it, because the adversarial review
was right that marking 1.2A and 1.2B complete misrepresented what happened.
