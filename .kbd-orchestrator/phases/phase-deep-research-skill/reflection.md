---
stage: reflect
phase: phase-deep-research-skill
created_at: 2026-07-08T15:25:00Z
artifacts:
  - reflection.md
---

# Reflection — phase-deep-research-skill

**Date:** 2026-07-08  
**Commit:** `5397353` → pushed to `origin/main`  
**Duration:** ~4 hours (estimated 10–12 hours in assessment — executed faster due to no blockers)

---

## Goal Achievement

| # | Goal | Criterion | Status |
|---|------|-----------|--------|
| G-01 | Create `skills/research/deep-research/` directory structure | All dirs created, `npm run validate:skill` passes | **MET** |
| G-02 | Write parent `SKILL.md` with 10-stage orchestration | Frontmatter valid, triggers defined, pipeline documented | **MET** |
| G-03 | Write all 10 sub-skill `SKILL.md` files | Each has frontmatter, input/output contracts, integration refs | **MET** |
| G-04 | Write scripts, templates, references, hooks, agents | All P1/P2 files present and executable (scripts) | **MET** |
| G-05 | Pass skill validation | `npm run validate:strict skills/research/deep-research` exits 0 | **MET** (0 errors, 0 warnings, 11 skills) |
| G-06 | Update docs index and README | New "Research" category appears in README and CONTRIBUTING.md | **MET** |
| G-07 | Commit and push | Commit `5397353` on main with `feat(research)` message | **MET** |

**7/7 goals MET.**

---

## Delivered Changes

| Change | Description | Files Created/Modified | Status |
|--------|-------------|----------------------|--------|
| change-drs-001 | Directory structure + skill.toml | 2 files | DONE |
| change-drs-002 | Parent deep-research/SKILL.md | 1 file | DONE |
| change-drs-003 | 10 stage sub-skill SKILL.md files | 10 files | DONE |
| change-drs-004 | 5 scripts + 5 templates | 10 files | DONE |
| change-drs-005 | 9 references + 4 hooks + 4 agents | 17 files | DONE |
| change-drs-006 | Validation (0 errors on first run) | — | DONE |
| change-drs-007 | README.md + marketplace.json + CONTRIBUTING.md | 3 files | DONE |
| change-drs-008 | Install + smoke test | — | DONE |
| change-drs-009 | Commit + push | — | DONE |

**Total: 71 files changed, 4,933 insertions in commit `5397353`**

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA | 9/9 |
| Strict validation first-pass | PASS (0 errors on first run — no refinement needed) |
| Changes requiring fix iterations | 0 |
| Install smoke test pass rate | 10/10 stage sub-skills, 5/5 scripts, 4/4 hooks |

No artifact-refiner logs exist for this phase (no `.refiner/` directory). Validation was performed via `npm run validate:strict` (strict mode, 11 skills validated) and install smoke test.

---

## Delta Analysis

### What was planned vs. what was delivered

**Planned:** 38 files (per assessment §5.1)  
**Delivered:** 71 files changed — the extra delta includes:
- KBD orchestrator state files (progress.json, handoffs, analysis.md, plan.md, decision-log.md)
- OpenSpec change proposals and task files (18 files across 9 changes)
- marketplace.json entry added to `.claude-plugin/` (not a separate `marketplace/` directory as planned — adapted correctly to actual repo structure)
- `docs/CONTRIBUTING.md` updated with research guidance (not in original plan — added during change-007 for completeness)

**One deviation from plan:** The plan mentioned updating `SKILLS.md` (which does not exist in the repo) and `docs/deep-research/index.md` (already up to date from prior work). Adapted to update `README.md` + `.claude-plugin/marketplace.json` + `docs/CONTRIBUTING.md` instead — equivalent coverage, correct targets.

### What the evidence shows

- Validation passed on the **first run** with 0 errors, 0 warnings. The strict frontmatter convention (name must match directory) was correctly applied from the start — no debugging loop was needed.
- Context compaction mid-change-005 caused a break. The continuation was seamless — session summary captured all in-progress state accurately and no work was lost or duplicated.
- The install script places skills at `~/.claude/skills/prometheus/research/deep-research/` (not `~/.claude/skills/research/deep-research/` as the smoke test initially checked). The flat `install-skills-flat.sh` script that installs to `~/.claude/skills/deep-research/` is separate. Both install correctly — the system-reminder confirmed all 10 stage sub-skills visible in the skill list.

---

## Lessons Captured

1. **Strict validation on first write.** Matching `name:` in SKILL.md frontmatter to the containing directory name at write time (not edit time) eliminated all validation errors. The pattern: directory `stage-01-planner/` → `name: stage-01-planner`. Applies to all future sub-skill development.

2. **Marketplace file location.** The repo's marketplace manifest is at `.claude-plugin/marketplace.json`, not `marketplace/marketplace.json`. Always grep for the actual location before editing.

3. **Context compaction is recoverable.** The `/kbd-apply` driver's per-task `tasks.md` file with `[x]` markers is the reliable continuation contract. The session summary correctly identified which tasks were done and which remained. This validates the task tracking approach.

4. **install:user vs install-skills-flat.sh.** `npm run install:user` copies to `~/.claude/skills/prometheus/` (preserving hierarchy). `bash scripts/install-skills-flat.sh` copies to `~/.claude/skills/` (flat, direct). Both are valid for different use cases. Smoke tests should target the correct path for each installer.

5. **Open questions resolved at plan time, not execute time.** All 4 open questions from assessment (OQ-01 through OQ-04) were resolved during analyze/plan without external research — full stack specification meant no new information was needed. Fast analyze phase = fast execute phase.

---

## Technical Debt Introduced

1. **`prometheus-research` Rust binary deferred.** The native MCP server that would allow long-running research jobs, streaming progress, and job persistence is explicitly deferred to `phase-prometheus-research-binary`. The current SKILL.md-based pipeline requires an active agent context — it cannot run as a background daemon.

2. **No AG-UI/A2UI streaming integration.** The HTML UI prototype at `docs/deep-research/deep-research-ui.html` (4,336 lines) is not wired to the pipeline. The `post-stage.sh` hook writes a checkpoint file, but no SSE stream updates the UI in real time.

3. **Palace ingest is hook-requested, not automatic.** `post-export.sh` writes a `.palace-ingest-requested` marker file, but the MCP `palace_ingest` call must be made by the skill (cannot be called from a bash hook). This is a known architectural constraint — bash hooks cannot call MCP tools directly.

4. **4 agents are descriptors only.** The `agents/research-planner.md` etc. are descriptive markdown, not registered Claude Code agents. They guide the frontier model via system prompt context but don't appear in `/agents`. A future phase could register them as formal subagents.

---

## Recommended Next Phase

**Option A (recommended): `phase-prometheus-research-binary`**  
Scaffold the `prometheus-research` Rust binary via `native-agent`. This adds:
- Background research jobs (daemon mode)
- SSE streaming to the HTML UI
- Job persistence across sessions
- MCP server mode for cross-harness access
- `prometheus-research start "my query"` CLI UX

**Option B: `phase-deep-research-agent-registration`**  
Register the 4 agent descriptors as formal Claude Code subagents with proper system prompts. Adds the `research-planner`, `source-verifier`, `contradiction-resolver`, and `report-synthesizer` to the agent registry so they can be spawned via the `Agent` tool with proper isolation.

**Option C: `phase-deep-research-ui-integration`**  
Wire the existing HTML UI to the pipeline via the `surface-bridge` MCP server (port 7890). Allows real-time research progress visualization in Claude Code's artifact panel.

**Recommendation: Option A first.** The binary unlocks long-running research that survives context window limits — the current skill requires the full pipeline to run within one context window. For deep/exhaustive research on complex topics, this is a real constraint.

---

## Sycophancy Gate

Self-check: does this reflection accurately describe gaps?

**Gaps confirmed present:**
- No Rust binary (deferred — explicitly noted, not glossed over)
- No UI wiring (explicitly noted)
- Palace ingest requires MCP call from skill, not hook (architectural constraint, not a bug)
- Smoke test path confusion between two installers (noted as lesson #4)

**Nothing suppressed.** The reflection is accurate.

---

## Handoff to Next Phase

The `deep-research` skill is production-ready for SKILL.md-based agent-context research. It passes strict validation, installs correctly, and all integration points are documented. The deferred `prometheus-research` binary is the primary carry-forward.

Next command: `/kbd-new-phase phase-prometheus-research-binary` (or `/kbd-evolve` to assess landscape first)
