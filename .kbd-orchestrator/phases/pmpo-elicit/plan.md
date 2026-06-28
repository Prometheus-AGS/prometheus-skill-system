# Plan — pmpo-elicit

**Phase:** pmpo-elicit
**Planned:** 2026-06-28
**Backend:** OpenSpec (`openspec/changes/change-elicit-NNN/`)
**Changes:** 6

---

## Summary

The `/pmpo-elicit` skill has a solid core from `canonical-lifecycle` (SKILL.md + schema + integration-contract). This phase completes it: async infrastructure first, then the cross-platform install + escalation-points guide, then operative wiring into the three KBD stages that currently reference pmpo-elicit only in prose.

**Ordering rationale:**
1. Checkpoint/resume scripts + contract first — everything else (wiring, async mode) depends on these artifacts existing.
2. Escalation-points guide + install wiring second — sets the cross-platform table so the wiring changes can reference it.
3. kbd-analyze operative protocol third — simplest wiring (one call site, clear condition).
4. kbd-goal human-gate wiring fourth — more complex (two gate points, five platforms).
5. pmpo-outer-loop wiring fifth — depends on escalation-points guide and the loop.json schema extension already shipped.
6. SKILLS.md/SKILL.md polish last — lowest risk, no dependencies.

---

## Change List

### change-elicit-001 — Async checkpoint/resume infrastructure

**Gaps addressed:** G-01, G-02, G-04
**Goals:** G4
**Priority:** HIGH — foundation; all wiring changes depend on this
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh` (NEW)
- `skills/process/pmpo-elicit/scripts/pmpo-elicit-resume.sh` (NEW)
- `skills/process/pmpo-elicit/references/checkpoint-contract.md` (NEW)

**What it does:**
- `pmpo-elicit-checkpoint.sh <elicit-dir> <question> <criticality> <caller> [hints...]`
  - Creates `<elicit-dir>/` directory
  - Writes `request.json` (valid against elicitation.schema.json, kind=request)
  - Writes `checkpoint.json` (caller state snapshot: `{caller, phase, stage, timestamp}`)
  - Writes `request-prompt.txt` (human-readable question + options, for non-Claude-Code platforms)
  - Exits 2 to signal "blocked — awaiting elicitation result"
- `pmpo-elicit-resume.sh <elicit-dir>`
  - Checks `<elicit-dir>/result.json` exists and `kind == "result"`
  - Outputs `answer` and `provenance` to stdout (JSON: `{"answer":"...","provenance":"..."}`)
  - Exits 0 on success, exits 1 if result.json absent or malformed
- `checkpoint-contract.md` documents:
  - What goes in `checkpoint.json` (caller state fields)
  - How a caller resumes: read stdout from `pmpo-elicit-resume.sh`, apply answer, clear checkpoint
  - The `<elicit-dir>` path convention: `<caller-state-dir>/elicitations/<id>/`
  - The `id` generation convention: `<caller>-<timestamp-ms>`

---

### change-elicit-002 — Escalation-points guide + platform routing table

**Gaps addressed:** G-03, G-05 (documentation half)
**Goals:** G3, G5
**Priority:** HIGH — wiring changes reference this doc
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-elicit/references/escalation-points.md` (NEW)

**What it does:**

A single reference file that documents:

**Section 1 — Stage trigger map**: when each KBD stage calls pmpo-elicit

| Stage | Trigger condition | Criticality | State dir |
|-------|------------------|-------------|-----------|
| `kbd-analyze` | Stack contest: score gap < 15% between top candidates | high | `.kbd-orchestrator/phases/<phase>/` |
| `kbd-goal` — Ideation→Spec gate | Human gate (unless `--auto-gates`) | high | `goals/<slug>/` |
| `kbd-goal` — Spec→Creation gate | Human gate (unless `--auto-gates`) | high | `goals/<slug>/` |
| `pmpo-outer-loop` — `loop-tick` | Regression or `max_no_progress_ticks` reached | blocking | `.kbd-orchestrator/loops/<name>/` |
| `pmpo-outer-loop` — `loop-define` | Any field ambiguity during loop definition | medium | `.kbd-orchestrator/loops/<name>/` |

**Section 2 — Platform routing table**: how elicitation works on each platform

| Platform | Question UI | Async method | Resume signal |
|----------|------------|--------------|---------------|
| Claude Code | `AskUserQuestion` tool (inline) | In-session (synchronous) | Immediate (no checkpoint needed) |
| Codex CLI | None — write `request-prompt.txt`, stop agent | File-based checkpoint: write `request.json`, exit | User writes `result.json`, re-invokes codex |
| OpenCode | `update_goal` surfaces question in goal state | File-based checkpoint | Agent polls `result.json` on next tick |
| Kimi Code | `/goal next` queues elicitation as a named step | File-based (kbd-goal-check detects `pending_elicitation`) | `kbd-goal-check` reads result, continues queue |
| Zed (standalone) | Write `request-prompt.txt`, pause background task | File-based checkpoint | User writes `result.json`, re-arms task |
| Zed (ACP-connected) | Delegates to connected agent's native UI | Depends on connected agent | Depends on connected agent |

**Section 3 — Shared state contract**: where the `elicitations/<id>/` directory lives relative to the caller, and which files are required vs. optional.

---

### change-elicit-003 — Install pmpo-elicit to all platforms

**Gaps addressed:** G-05 (install half)
**Goals:** G5
**Priority:** HIGH
**Agent:** claude-code
**Effort:** S

**Files:**
- `scripts/install-skills-flat.sh` (MODIFY — add pmpo-elicit to the skill copy list for Kimi, Codex, OpenCode, Cursor, Zed)
- `skills/process/pmpo-elicit/SKILL.md` (MODIFY — add platform-mode section)

**What it does:**

In `install-skills-flat.sh`: `pmpo-elicit` is a process skill, not language-specific. Add it to the `PROCESS_SKILLS` list (alongside `kbd-goal`, `kbd-goal-check`) that gets copied to every detected platform's skills directory.

In `SKILL.md`: Add a `## Platform Mode` section after "## Modes":

```markdown
## Platform Mode

On Claude Code, option 2 ("research it for me") and the AskUserQuestion-based
question UI run in-session synchronously. On all other platforms, elicitation
uses the file-based async contract:

1. The caller invokes `pmpo-elicit-checkpoint.sh` to write `request.json` + pause.
2. The operator reads `request-prompt.txt` (human-readable), writes `result.json`
   with `{kind:"result", id, answer, provenance}`.
3. The caller invokes `pmpo-elicit-resume.sh` to read the result and continue.

See `references/escalation-points.md` for the platform routing table and
`references/checkpoint-contract.md` for the caller-side state protocol.
```

---

### change-elicit-004 — kbd-analyze operative call protocol

**Gaps addressed:** G-06
**Goals:** G3
**Priority:** MEDIUM
**Agent:** claude-code
**Effort:** S

**Files:**
- `skills/process/kbd-process-orchestrator/skills/kbd-analyze/SKILL.md` (MODIFY)

**What it does:**

The current text says "contested choice (score gap < 15%) escalates via `/pmpo-elicit` when available; otherwise flag it for the user in `analysis.md`." This is prose-only. Add an operative call protocol in the "Stack discovery" section:

```
### Contested stack escalation (score gap < 15%)

When the top two stack candidates are within 15% of each other:

1. Construct the elicitation request:
   - question: "Two stacks are equally matched: {A} ({score}%) vs {B} ({score}%). Which should we use?"
   - hints: ["{A} pros", "{B} pros", "key tradeoffs"]
   - criticality: high
   - caller: kbd-analyze
   - write_back_path: analysis.md → stack_decision

2. Write request.json to `.kbd-orchestrator/phases/<phase>/elicitations/<id>/request.json`
   using `pmpo-elicit-checkpoint.sh` (or inline on Claude Code via AskUserQuestion).

3. On result: record choice in decision-log.md with provenance + elicitation_id.
   Continue analysis with the selected stack.

4. If pmpo-elicit is unavailable: flag the contest in analysis.md and
   decision-log.md, note both options, and ask the user inline.
```

---

### change-elicit-005 — kbd-goal human-gate wiring

**Gaps addressed:** G-07
**Goals:** G3
**Priority:** MEDIUM
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/kbd-goal/SKILL.md` (MODIFY — human gate section)
- `skills/process/kbd-goal/references/platforms/claude-code.md` (MODIFY — inline gate)
- `skills/process/kbd-goal/references/platforms/kimi.md` (MODIFY — gate via kbd-goal-check)

**What it does:**

Current human gate section says "Review IDEAS.md, select your preferred candidate" as inline prose. Replace with pmpo-elicit integration:

**Ideation → Spec gate (after evaluator PASS):**
```
1. Invoke /pmpo-elicit:
   - question: "Ideation complete. IDEAS.md has <N> candidates. Which direction to pursue?"
   - options (from IDEAS.md): ["<candidate-1>", "<candidate-2>", ...]
   - criticality: high
   - caller: kbd-goal/ideation
   - write_back_path: goal.json → phases[ideation].human_gate_result

2. Record result:
   - goal.json → phases[ideation].human_gate_result = {decision, provenance, elicitation_id}
   - If decision == "revision-needed": re-enter ideation loop
   - If decision == approved candidate: proceed to Spec phase with selected candidate

3. Skip condition: if --auto-gates is set, record "auto-approved" with provenance: implicit
```

**Spec → Creation gate (after evaluator PASS):**
```
1. Invoke /pmpo-elicit:
   - question: "Specification complete. SPEC.md ready. Approve to begin Creation?"
   - options: ["Approve — begin Creation", "Request revision", "Stop here"]
   - criticality: high
   - caller: kbd-goal/spec
   - write_back_path: goal.json → phases[spec].human_gate_result

2. Record result as above. Stop if "Stop here". Revision if "Request revision".
```

Also add STATE.md `escalations[]` write protocol: on any elicitation triggered during Creation phase, append `{id, question, status: "pending"}` to `escalations[]`, update to `{status: "resolved", provenance, answer}` on result.

Platform notes:
- Claude Code: `AskUserQuestion` in-session
- Kimi: `kbd-goal-check` detects `pending_elicitation` state in `goal.json`, queues next `/goal` step as the resume
- All others: file-based checkpoint contract

---

### change-elicit-006 — pmpo-outer-loop stall escalation + SKILLS.md polish

**Gaps addressed:** G-08, G-09
**Goals:** G3, G1
**Priority:** MEDIUM (G-08) / LOW (G-09)
**Agent:** claude-code
**Effort:** S

**Files:**
- `skills/process/pmpo-outer-loop/SKILL.md` (MODIFY — loop-tick regression/stall section)
- `SKILLS.md` (MODIFY — update pmpo-elicit description)

**What it does:**

In `pmpo-outer-loop/SKILL.md`, the `loop-tick` section currently says "escalate via `/pmpo-elicit` (continue / re-plan / stop) — a declared decision point." Expand with operative protocol:

```
### Stall/regression escalation

When `max_no_progress_ticks` is reached or a regression is detected:

1. Invoke /pmpo-elicit:
   - question: "Loop '<name>' stalled after <N> ticks with no progress. How to proceed?"
   - options: ["Continue — run another tick", "Re-plan — revise the evolution goal", "Stop — terminate loop"]
   - context: "Last tick result: <diff vs measurable_criteria>"
   - criticality: blocking
   - caller: pmpo-outer-loop/loop-tick
   - write_back_path: loops/<name>/journal.md → last_entry.escalation_result

2. On "Continue": reset `no_progress_ticks` counter, run next tick.
3. On "Re-plan": write updated `loop.json → goal` via a new /pmpo-elicit call (or inline edit), reset tick counters.
4. On "Stop": write final `/loop-report`, set loop status = "terminated-by-operator".

File-based async: write `checkpoint.json` to `.kbd-orchestrator/loops/<name>/elicitations/<id>/`,
pause the loop. On resume, read result and apply the above logic.
```

In `SKILLS.md`: Update line 196 description from "PMPO artifact elicitation: draw out requirements, constraints, and goals" to "Ask-or-research human escalation primitive: present a decision with 4 option classes, collect a structured answer with provenance, support async pause/resume across all platforms."

---

## Dependency Order

```
change-elicit-001   (checkpoint/resume scripts + contract)
    ↓
change-elicit-002   (escalation-points guide)
    ↓
change-elicit-003   (install wiring + SKILL.md platform section)
    ↓
change-elicit-004   (kbd-analyze wiring)          ← depends on 001+002
change-elicit-005   (kbd-goal wiring)             ← depends on 001+002+003
    ↓
change-elicit-006   (pmpo-outer-loop + polish)    ← depends on 001+002
```

Changes 004, 005, 006 can be parallelized after 003 completes.

---

## First Change to Apply

**change-elicit-001** — `scripts/pmpo-elicit-checkpoint.sh` + `scripts/pmpo-elicit-resume.sh` + `references/checkpoint-contract.md`

See `openspec/changes/change-elicit-001/proposal.md` for full specification.
