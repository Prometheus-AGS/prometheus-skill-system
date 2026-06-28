# Reflection — pmpo-elicit

**Phase:** pmpo-elicit
**Reflected:** 2026-06-28
**Changes:** 6 of 6 (100% complete)
**Previous phase:** goal-loop-support

---

## Goal Achievement

| Goal | Description | Status |
|------|-------------|--------|
| G1 | Ship `skills/process/pmpo-elicit/SKILL.md` with updated platform-mode section and correct SKILLS.md entry | **MET** |
| G2 | Define the elicitation schema (`elicitation.schema.json`) with request/result union | **MET** (pre-existing, verified) |
| G3 | Wire `/pmpo-elicit` into KBD lifecycle at all documented escalation points | **MET** |
| G4 | Support async elicitation — checkpoint/resume scripts + caller-side contract | **MET** |
| G5 | Platform-agnostic: same `elicit.json` checkpoint file and install across all platforms | **MET** |

**Overall: 5/5 goals MET (100%)**

---

## Delivered Changes

| Change | Title | Status | Gaps Closed |
|--------|-------|--------|-------------|
| change-elicit-001 | Async checkpoint/resume infrastructure | DONE | G-01, G-02, G-04 |
| change-elicit-002 | Escalation-points guide + platform routing table | DONE | G-03, G-05 (doc half) |
| change-elicit-003 | Install pmpo-elicit to all platforms + SKILL.md platform section | DONE | G-05 (install half) |
| change-elicit-004 | kbd-analyze operative contested-stack escalation protocol | DONE | G-06 |
| change-elicit-005 | kbd-goal human-gate wiring via pmpo-elicit | DONE | G-07 |
| change-elicit-006 | pmpo-outer-loop stall escalation protocol + SKILLS.md description update | DONE | G-08, G-09 |

All 9 gaps from the assessment (G-01 through G-09) are closed.

---

## Delta Analysis

**What was planned vs. what was delivered:**

All 6 changes landed exactly as specified in `plan.md`. No scope creep, no unplanned additions. One deviation worth noting:

- **G-05 install gap was partially a documentation artefact.** The assessment identified `install-skills-flat.sh` as not installing pmpo-elicit to non-Claude-Code platforms. Investigation during change-elicit-003 revealed the script uses dynamic `find "$REPO_ROOT/skills" -name "SKILL.md"` — pmpo-elicit was already auto-discovered and installed across all 104 skills on all platforms. The real gap was the SKILL.md platform-mode section (which was missing) and the escalation-points.md reference doc (which didn't exist). Both were delivered. The tasks.md for change-elicit-003 documents this finding to avoid re-discovering it.

**Planned effort vs. actual:** M+M+S+S+M+S = estimated ~10 tool sessions. Actual: completed across two Claude Code sessions (one pre-compaction, one post-compaction resume). No rework.

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes total | 6 |
| Changes with validate:strict run | 4 (001, 002, 004, 006) |
| validate:strict clean passes | 4/4 |
| Smoke tests run | 2 (pmpo-elicit-checkpoint.sh, pmpo-elicit-resume.sh) |
| Smoke test passes | 2/2 |
| Changes skipped QA (doc-only or <3 files) | 2 (003 install fix, 005 doc-only wiring) |

**Recurring constraint flags:** None. No backslash warnings, no name mismatches, no frontmatter errors.

**Notable quality outcome:** kbd-analyze SKILL.md hit a pre-existing 548-line length warning (validate:strict). This was documented in change-elicit-004 tasks.md as pre-existing, not introduced by this phase.

---

## Lessons Captured

1. **`install-skills-flat.sh` uses dynamic `find` — no explicit skill list to maintain.** When the assessment flags a skill as "not in install list," always verify the actual script mechanism before planning a change. The dynamic discovery means the only install gap is ever about script structure changes, not list additions. Carry this forward to every future phase that mentions the install script.

2. **`python3` is the right JSON tool inside bash scripts on all target platforms.** Used it in `pmpo-elicit-resume.sh` for `result.json` validation. More portable and reliable than `jq` (not always present) or bash string manipulation (brittle). Pattern: `python3 -c "import json,sys; d=json.load(sys.stdin); ..."` as a heredoc.

3. **Context compaction during kbd-execute does not lose state** — waypoint.json + position-reminder.txt + progress.json together are sufficient to resume any mid-execute session cleanly. The `reflect` step completed cleanly after a full compaction event during execute.

4. **Operative protocol subsections need all four elements:** condition trigger, the exact bash invocation (with real argument values, not placeholders), the outcome branch logic for every option, and the decision-log recording template. Missing any one of these means the operator has to improvise. All four wiring changes (004–006) include all four elements.

5. **Platform routing should be a single reference, not scattered.** `escalation-points.md` as the single source of truth for the platform routing table proved clean — all three wiring changes (004, 005, 006) cross-reference it rather than duplicating the table. Future phases that add new escalation callers should extend this file, not create new routing tables.

---

## Technical Debt Introduced

| Item | Severity | Notes |
|------|----------|-------|
| kbd-analyze SKILL.md at 548 lines | LOW | Pre-existing. Exceeds 500-line soft cap. Not introduced by this phase. Future refactor candidate: move `### Contested stack escalation — operative protocol` to a `references/` file. |
| `result.json` has no write helper script | LOW | Operators on non-Claude-Code platforms must write `result.json` by hand following the schema. A `pmpo-elicit-write-result.sh` helper would reduce errors but was not in scope for this phase. |
| `STATE.md escalations[]` write protocol is prose-only | LOW | Documented in kbd-goal SKILL.md but not implemented as a script. Operators must follow manually. Low risk given the gate's criticality guard. |

---

## Carry-Forwards

The following items are out of scope for this phase but should be addressed in a follow-on:

1. **`pmpo-elicit-write-result.sh` helper script** — A thin script that writes a conformant `result.json` from CLI args `<elicit-dir> <answer> <provenance>`. Reduces operator error on non-Claude-Code platforms. Estimated: 1 change, S effort.

2. **kbd-analyze SKILL.md refactor** — Extract the operative protocol subsection to `references/analysis-protocols.md` to bring the file under 500 lines. Estimated: 1 change, S effort.

3. **Codex-specific pmpo-elicit UX documentation** — The platform routing table covers Codex at the schema level (file-based checkpoint). A `references/platforms/codex.md` file (parallel to kimi.md, claude-code.md) would document the exact operator workflow for Codex's stop-and-wait model. Estimated: 1 change, XS effort.

4. **`pmpo-elicit` skill — integration test** — A smoke-test script (`scripts/test-roundtrip.sh`) that exercises the full checkpoint → write-result → resume cycle in a temp directory. Would give confidence on new platforms without needing a live KBD session.

---

## Recommended Next Phase

**`pmpo-evolver`** — The outer loop (`pmpo-outer-loop`) and goal loop (`kbd-goal`) now have human escalation wired. The natural next capability gap is the evolution driver itself: the PMPO evolver that decides *what* to run next in the outer loop, synthesizes reflection results, and manages the strategy layer above individual KBD phases. This is the missing link between the per-phase KBD cycle and the long-horizon learning loop.

Alternatively, **`kbd-analyze-deepen`** — the analysis phase currently does a tiered research pipeline but has no structured output format that feeds cleanly into the plan phase's change-ordering logic. This would close the analyze→plan handoff gap.

Priority recommendation: **pmpo-evolver** first — it compounds the value of everything shipped so far (goal loop, elicit, outer loop) into a coherent autonomous improvement engine.

---

## Phase Summary

The `pmpo-elicit` phase completed the human-escalation primitive from its prior-phase skeleton into a production-ready, platform-agnostic capability. The checkpoint/resume contract is defined and tested. The three KBD lifecycle stages that documented pmpo-elicit in prose now have operative call protocols with exact bash invocations, outcome branches, and decision-log recording. The escalation-points platform routing table consolidates what previously lived in five separate places into one reference. All 9 identified gaps are closed. No goals were deferred.
