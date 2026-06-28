# Assessment — pmpo-elicit

**Phase:** pmpo-elicit
**Assessed:** 2026-06-28
**Assessor:** kbd-assess

---

## Executive Summary

The `/pmpo-elicit` skill has a solid v1 SKILL.md from the `canonical-lifecycle` phase
(change-005, status: done). The core protocol — four option classes, budget guards,
progress signals, schema — is in place. However, the skill is **incomplete against
the goals of this phase**: it lacks the async checkpoint/resume mechanism, has no
active wiring into any calling stage, is not installed on non-Claude-Code platforms,
and the `references/escalation-points.md` platform guide doesn't exist.

Goals G1 and G2 are **partially met**. Goals G3, G4, and G5 are **not met**.

---

## Artifact Inventory

### What exists (`skills/process/pmpo-elicit/`)

| Artifact | Status | Notes |
|----------|--------|-------|
| `SKILL.md` | EXISTS — solid | 4 option classes, budget guards, inline-fallback mode, progress signals. Passes `validate:strict` clean. |
| `references/integration-contract.md` | EXISTS | Documents caller protocol (kbd-analyze, zeespec-interrogate, kbd-capability). Caller → `request.json`; pmpo-elicit → `result.json`. |
| `references/schemas/elicitation.schema.json` | EXISTS | Complete: request + result, provenance enum (user/source/research/implicit), confidence, evidence, cost fields. |
| `references/schemas/elicit.schema.json` | MISSING | `current-waypoint.json → scoped_paths` expected this name; `elicitation.schema.json` exists instead. Not a blocker — name in SKILL.md matches existing file. |
| `references/escalation-points.md` | MISSING | Platform-agnostic guide for when each KBD stage triggers elicitation. Not created yet. |
| `scripts/pmpo-elicit-checkpoint.sh` | MISSING | The async pause mechanism: writes `elicitations/<id>/request.json` + `checkpoint.json`, suspends the loop. |
| `scripts/pmpo-elicit-resume.sh` | MISSING | The async resume mechanism: reads `result.json` from a completed elicitation, unblocks the caller. |

### Plugin/install registration

| Check | Result |
|-------|--------|
| `.claude-plugin/plugin.json` includes `pmpo-elicit` | ✅ YES (line 37) |
| `SKILLS.md` index documents it | ✅ YES (line 196) |
| `scripts/install-skills-flat.sh` installs to all platforms | ❌ NO — `pmpo-elicit` not in the install list; only installed to Claude Code via the plugin symlink |

---

## Gap Analysis by Goal

### G1 — Ship `skills/process/pmpo-elicit/SKILL.md` — a `/pmpo-elicit` slash command
**Status: PARTIAL**

The SKILL.md exists and is structurally complete. Gaps:
- The "How to invoke" section says `pmpo-elicit` writes `elicitations/<id>/request.json` under the caller's state dir, but there is no script to actually do this — agents invoking it must do this inline.
- No `elicitations/` directory template or creation protocol documented.
- The SKILL.md description in `SKILLS.md` says "PMPO artifact elicitation: draw out requirements, constraints, and goals" — this is the old description from before the ask-or-research design. Should be updated to match the current four-option-class framing.

**Required:** SKILL.md update to tighten the invoke protocol + correct the SKILLS.md entry.

### G2 — Define the elicitation schema (`elicit.json`)
**Status: MET (minor naming gap)**

`elicitation.schema.json` fully covers the spec: request fields (question, context, hints, criticality, caller, write_back_path), result fields (answer, provenance, source_ref, confidence, evidence, cost, resolved_at). The oneOf union cleanly separates request from result.

The only gap: `current-waypoint.json → scoped_paths` listed `references/schemas/elicit.schema.json` but the actual file is `elicitation.schema.json`. This is a documentation artefact from phase setup, not a real gap — SKILL.md correctly references the existing name.

**Required:** None on the schema itself. Update the `scoped_paths` entry to correct the name mismatch.

### G3 — Wire `/pmpo-elicit` into KBD lifecycle at all documented escalation points
**Status: NOT MET**

Documented escalation points (from `loops-architecture-spec.md`, `kbd-analyze/SKILL.md`, `kbd-goal/SKILL.md`, `pmpo-outer-loop/SKILL.md`):

| Escalation Point | Current State | Required Action |
|-----------------|---------------|-----------------|
| `kbd-analyze` — contested stack choice (score gap < 15%) | References `/pmpo-elicit` in prose ("when available") but no active call protocol | Add explicit call protocol: read `library-candidates.json`, detect gap < 15%, construct request, invoke `/pmpo-elicit`, write result to `decision-log.md` |
| `kbd-goal` — Ideation → Spec human gate | Human gate documented as "review IDEAS.md" — no pmpo-elicit integration | Wire the gate: after evaluator returns PASS, invoke `/pmpo-elicit --criticality high` to collect gate decision (approve/revision-needed), record in `goal.json → phases[].human_gate_result` |
| `kbd-goal` — Spec → Creation human gate | Same gap | Same fix |
| `pmpo-outer-loop` — regression/stall escalation | Prose says "escalate via /pmpo-elicit (continue/re-plan/stop)" but no operative protocol | Document operative protocol: tick-on-regression → write elicitation request with 3 options (continue/replan/stop), block loop, resume on result |
| `pmpo-outer-loop` — `escalation_points[]` in `loop.json` | `loop.json` schema has `escalation_points` but no wiring to pmpo-elicit | Wire: at loop-define, populate escalation_points from `/pmpo-elicit` responses |
| `STATE.md → escalations[]` | Field exists in kbd-goal STATE.md but no write protocol | Document: on elicitation triggered during Creation phase, append `{id, question, status: "pending"}` to `escalations[]`; on result, update to `{status: "resolved", provenance}` |

**Required:** `references/escalation-points.md` + wiring updates to kbd-analyze and kbd-goal SKILL.md sections.

### G4 — Support async elicitation (pause/resume without losing state)
**Status: NOT MET**

The current SKILL.md documents inline-fallback mode only: the question is presented via `AskUserQuestion` and the loop blocks in-session waiting for a response. This works for synchronous sessions but breaks the async contract:

- No `pmpo-elicit-checkpoint.sh` to write `request.json` and suspend the loop to disk
- No `pmpo-elicit-resume.sh` to read `result.json` and inject the answer back into the calling stage
- No documented checkpoint contract (what state the calling stage must preserve across a checkpoint/resume cycle)

The inline mode is explicitly labelled "Inline-fallback (current)" in SKILL.md — the async mechanism was always scoped for this phase.

**Required:**
- `scripts/pmpo-elicit-checkpoint.sh` — accepts `<elicit-dir> <question> <criticality> <caller>`, writes `request.json` + `checkpoint.json` (preserving caller state), exits 2 to signal "blocked"
- `scripts/pmpo-elicit-resume.sh` — accepts `<elicit-dir>`, reads `result.json`, outputs answer + provenance to stdout for the caller to inject
- `references/checkpoint-contract.md` — documents the caller-side requirements for preserving state across a checkpoint (what goes in `checkpoint.json`)

### G5 — Platform-agnostic: same `elicit.json` checkpoint file across all five platforms
**Status: NOT MET**

`pmpo-elicit` is registered in `.claude-plugin/plugin.json` (Claude Code only via symlink). It is **not** installed to Kimi Code, Codex, OpenCode, Cursor, or Zed via `install-skills-flat.sh`.

Platform-specific behaviors not documented:
- **Codex** — no native question UI; elicitation must write `request.json`, stop the agent, and require the user to respond in a file before re-invoking
- **OpenCode** — no AskUserQuestion equivalent; can use the `update_goal` tool to surface the question in the goal state, then poll for a response
- **Kimi Code** — `/goal next` queue means the elicitation must be queued as a named step and the `kbd-goal-check` evaluator detects `pending_elicitation` state
- **Zed standalone** — same async file-based contract as Codex
- **Claude Code** — `AskUserQuestion` works natively (current inline mode)

**Required:**
- Add `pmpo-elicit` to `install-skills-flat.sh` skill copy list for all platforms
- Create `references/escalation-points.md` with platform routing table
- SKILL.md platform-mode section documenting the file-based async fallback for non-Claude-Code platforms

---

## Identified Gaps (G-01 through G-09)

| ID | Gap | Goal | Priority |
|----|-----|------|----------|
| G-01 | `scripts/pmpo-elicit-checkpoint.sh` missing | G4 | HIGH |
| G-02 | `scripts/pmpo-elicit-resume.sh` missing | G4 | HIGH |
| G-03 | `references/escalation-points.md` missing — no platform routing or stage-to-trigger map | G3, G5 | HIGH |
| G-04 | `references/checkpoint-contract.md` missing — no caller-side state preservation protocol | G4 | MEDIUM |
| G-05 | `install-skills-flat.sh` does not install pmpo-elicit to Kimi, Codex, OpenCode, Cursor, Zed | G5 | HIGH |
| G-06 | kbd-analyze SKILL.md: contested-stack escalation is prose-only, no operative call protocol | G3 | MEDIUM |
| G-07 | kbd-goal SKILL.md: human gates between phases not wired to pmpo-elicit | G3 | MEDIUM |
| G-08 | pmpo-outer-loop: tick stall/regression escalation has no operative protocol | G3 | MEDIUM |
| G-09 | SKILLS.md description still reflects old framing ("artifact elicitation: draw out requirements") | G1 | LOW |

---

## What Is NOT a Gap

- The four-option-class design in SKILL.md is correct and complete — no changes needed
- The schema (`elicitation.schema.json`) is correct — no schema changes needed
- The integration-contract.md accurately documents the caller protocol — no changes needed
- `plugin.json` registration is correct for Claude Code
- Progress signals are declared correctly in SKILL.md

---

## Recommended Change Plan (for `/kbd-plan`)

**5–6 changes**, ordered by dependency:

1. **change-elicit-001** — `scripts/pmpo-elicit-checkpoint.sh` + `scripts/pmpo-elicit-resume.sh` + `references/checkpoint-contract.md` (G4 foundation — all wiring depends on this)
2. **change-elicit-002** — `references/escalation-points.md` (platform routing + stage trigger map) (G3, G5 shared reference)
3. **change-elicit-003** — Add pmpo-elicit to `install-skills-flat.sh` + SKILL.md platform-mode section (G5)
4. **change-elicit-004** — kbd-analyze wiring: contested-stack escalation operative protocol (G3)
5. **change-elicit-005** — kbd-goal wiring: human gate integration at Ideation→Spec and Spec→Creation (G3)
6. **change-elicit-006** *(optional)* — pmpo-outer-loop + STATE.md escalation[] write protocol (G3, MEDIUM — can defer if scope is tight)

---

## Open Questions for Plan/Execute

1. **Async file format for non-Claude-Code platforms**: should `request.json` be written to the caller's state dir (e.g., `.kbd-orchestrator/phases/<phase>/elicitations/<id>/`) or to a shared `~/.pmpo/elicitations/<id>/` for cross-tool accessibility? Current SKILL.md says caller's state dir — this is correct for single-tool sessions but breaks when the operator switches tools mid-loop.

2. **Platform-specific question UI**: On Codex, the agent must stop and wait for a file to be written. Should the checkpoint script also write a human-readable prompt file (e.g., `request-prompt.txt`) alongside `request.json` so the operator sees the question without parsing JSON?

3. **kbd-goal human gates**: The current gate says "Review IDEAS.md, select your preferred candidate" as an inline prose instruction. Should wiring replace this (pmpo-elicit becomes the gate mechanism) or augment it (pmpo-elicit is called when `--auto-gates` is not set)?

---

## Assessment Conclusion

The `pmpo-elicit` skill has a solid core (SKILL.md + schema + integration contract) from a prior phase. This phase must complete it: ship the checkpoint/resume scripts, write the escalation-points platform guide, wire pmpo-elicit into the install script, and add operative call protocols to kbd-analyze and kbd-goal. 6 changes, starting from the async infrastructure upward.

**Recommended next step:** `/kbd-plan pmpo-elicit`
