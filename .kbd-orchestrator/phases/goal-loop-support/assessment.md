# Assessment — goal-loop-support

**Phase:** goal-loop-support
**Assessed:** 2026-06-27
**Assessor:** Claude Sonnet 4.6

---

## What Was Asked

Add platform-agnostic goal-driven iterative loop support to the KBD orchestrator — implementing or augmenting Claude Code's `/goal` functionality using the existing KBD assess → analyze → plan → execute → reflect lifecycle. Must support multi-phase goal decomposition (Ideation → Specification → Creation → Deployment), inner loops for complex sub-tasks, platform portability (Claude Code, OpenCode, Codex, Kimi, Cursor, Windsurf), separated evaluator pattern, and skill/MCP auto-discovery.

---

## Claude Code `/goal` — What It Is (as of 2026-06)

Claude Code's `/goal` command (added v2.1.139, May 2026) is a native loop primitive with these characteristics:

| Property | Detail |
|----------|--------|
| **Invocation** | `/goal [--tokens N] [--worktree] <stopping condition>` |
| **Loop driver** | After each turn, a separate fast model (Haiku) evaluates the stopping condition and returns yes/no + reason |
| **Stopping** | Goal condition satisfied · token ceiling · turn limit written into condition · Ctrl+C |
| **Isolation** | `--worktree` flag spawns a clean git checkout per run |
| **Bias prevention** | Evaluator is a different model instance from the builder — critical invariant |
| **Scope** | Single-session, single-goal; not multi-phase natively |
| **Platform** | Claude Code CLI only; no equivalent in OpenCode, Codex, Kimi, Cursor, Windsurf |

**Key gap**: `/goal` is execution-only. It does not support Ideation, Specification, or multi-phase workflows. It is also Claude Code-specific.

---

## Current KBD State — What Already Exists

### Strengths (what we have)

| Capability | Where | Completeness |
|-----------|-------|-------------|
| Iterative loop engine (assess→analyze→plan→execute→reflect) | `iterative-evolver` | **Strong** — full cycle, state persistence, named evolutions, surreal-memory integration |
| Outer loop / standing goals | `pmpo-outer-loop` | **Good** — `/loop-define`, `/loop-tick`, `/loop-report`; `loop.json` schema with measurable_criteria, termination, escalation_points |
| Multi-phase orchestration | `kbd-process-orchestrator` | **Strong** — phases, child phases (nested), cross-tool handoff |
| Child phase / inner loop support | `kbd-new-child`, `kbd-next-child`, `kbd-child-exit` | **Good** — arbitrary depth, `path[]` chain, rollup |
| Human escalation gates | `pmpo-elicit` + escalation_points in loop.json | **Good** — pauses and prompts human at declared decision points |
| Platform portability | Cross-tool handoff, `.kbd-orchestrator/` as shared state | **Good** — supports 7 tools already |
| Skill loading | Skills are discovered from `~/.claude/skills/` and triggered | **Partial** — auto-trigger exists but no goal-time discovery |
| MCP server management | `install-mcp-services.sh`, `configure-mcp-all-tools.sh` | **Good** — 7 servers configured across 7 tools |
| Sycophancy gate (evaluator separation) | `sycophancy-correction` SubagentStop hook | **Partial** — catches reflection sycophancy but not goal completion grading |
| Ideation support | `ideation-mindmap` | **Partial** — mindmap generation but no convergence loop |

### Gaps (what is missing)

| Gap | ID | Severity |
|-----|----|---------|
| No unified `/kbd-goal` entry point | G-01 | HIGH — without this, users must manually chain loop-define → loop-tick → child phases |
| No separated evaluator for stopping conditions | G-02 | HIGH — current loops rely on the same model to determine if goal is met (self-grading bias) |
| No multi-phase goal decomposition workflow | G-03 | HIGH — Ideation → Specification → Creation phases are not wired together |
| No goal-time skill/MCP discovery | G-04 | MEDIUM — skills must be pre-loaded; goal start doesn't auto-identify what's needed |
| No inner-loop auto-promotion | G-05 | MEDIUM — complex tasks during execute aren't automatically promoted to child phases |
| Claude Code `/goal` bridge | G-06 | MEDIUM — when running on Claude Code, should delegate to native `/goal` where appropriate, not duplicate it |
| No Ideation convergence loop | G-07 | MEDIUM — `ideation-mindmap` generates ideas but no scoring/critic loop that iterates until N candidates pass threshold |
| No Specification validation loop | G-08 | MEDIUM — no spec-writer + spec-reviewer loop that iterates to PASS |
| Loop state not in `.kbd-orchestrator/loops/` yet | G-09 | LOW — `/loop-define` was shipped but no loops directory exists; integration not wired end-to-end |
| No deployment phase template | G-10 | LOW — Creation phase is covered by existing execute pattern; Deployment needs a template |

---

## Gap Analysis Detail

### G-01: `/kbd-goal` entry point
The closest existing thing is `/loop-define` + `/loop-tick`. But `/loop-define` requires pre-knowing the loop schema. A `/kbd-goal "description" [stopping condition]` command should: infer the goal type (Ideation / Specification / Creation / full pipeline), auto-create the `loop.json`, create child phases for each stage, and start ticking.

### G-02: Separated evaluator
The sycophancy gate on reflections is a partial analog — it uses the `sycophancy-correction` binary to grade the reflector's output. We need the same pattern for goal-condition evaluation: a dedicated subagent (or the `sycophancy-correction` MCP) prompted to evaluate "is condition X satisfied given transcript Y?" returning yes/no + reason, not the builder model doing it.

### G-03: Multi-phase goal decomposition
The KBD nested-phases system (`path[]`) supports arbitrary depth, so the structure exists. What's missing is the **orchestration script** that: (1) reads a goal, (2) creates child phases for Ideation / Spec / Creation, (3) runs each as a loop, (4) gates between them (human approval or auto-pass), and (5) chains outputs (IDEAS.md → SPEC.md → TASKS.md → STATE.md).

### G-04: Goal-time skill/MCP discovery
When a goal is stated (e.g., "build a weekly standup generator in Go"), the system should inspect the goal, identify relevant skills (`golang-patterns`, `golang-testing`, etc.) and MCP servers (Context7 for Go docs, etc.) and pre-load or recommend them. Currently this is manual.

### G-05: Inner-loop auto-promotion
During `/kbd-execute`, if a change is flagged as too complex (exceeds complexity threshold, or fails repeatedly), the system should auto-create a child phase for it rather than retrying in the same context. The nested-phases infrastructure exists but the trigger logic doesn't.

### G-06: Claude Code `/goal` bridge
On Claude Code, native `/goal` is fast and uses Haiku as evaluator. For simple single-phase goals, we should delegate to it. For multi-phase goals, we orchestrate above it, using `/goal` per phase. For other tools, we implement the evaluator pattern ourselves using the `sycophancy-correction` MCP or a dedicated subagent.

### G-07 + G-08: Ideation and Specification loops
These are the most novel additions. Each needs:
- **Ideation**: discovery agent → critic agent (stronger model) → score against rubric → write survivors to `IDEAS.md` → loop until N pass threshold → human gate
- **Specification**: spec-writer → spec-reviewer (ambiguity check) → revision loop → PASS → human gate

---

## Recommended Phase Structure

This phase should be broken into these changes (to be planned in `/kbd-analyze`):

| Priority | Change | Description |
|----------|--------|-------------|
| 1 | `goal-001` | Separated evaluator subagent — `agents/goal-evaluator.md` + integration with `sycophancy-correction` MCP |
| 2 | `goal-002` | `/kbd-goal` skill — entry point that parses goal type, creates loop.json, starts first child phase |
| 3 | `goal-003` | Ideation loop child-phase template — discovery + critic agents, IDEAS.md convergence, human gate |
| 4 | `goal-004` | Specification loop child-phase template — spec-writer + spec-reviewer loop, SPEC.md, human gate |
| 5 | `goal-005` | Creation loop enhancement — TASKS.md decomposition, worktree per task, verifier agent, STATE.md |
| 6 | `goal-006` | Claude Code `/goal` bridge — detect tool, delegate to native `/goal` for single-phase or per-phase |
| 7 | `goal-007` | Goal-time skill/MCP discovery — parse goal description, emit recommended skills/servers list |
| 8 | `goal-008` | Inner-loop auto-promotion — complexity threshold in execute triggers `kbd-new-child` automatically |
| 9 | `goal-009` | Wire `.kbd-orchestrator/loops/` — ensure loop-define creates the dir, loop-tick reads it end-to-end |
| 10 | `goal-010` | Deployment phase template + cross-platform test matrix for all 7 tools |

**Estimated complexity:** HIGH. Changes 1–5 are core and tightly coupled. Changes 6–10 are integration and polish. Suggest delivering in two sub-phases (core 1–5, then integration 6–10) using the child-phase mechanism.

---

## What to Do Next

Run `/kbd-analyze` to research implementation details for each gap, then `/kbd-plan` to produce the full change list with scoped file paths and acceptance criteria before `/kbd-execute`.

**Readiness gate:** Analysis should answer: (a) exactly how the evaluator subagent pattern works at the Claude Code API level, (b) whether `sycophancy-correction` MCP's `detect_sycophancy` can double as a goal-condition grader, (c) what the `loop.json` schema needs to add for multi-phase goals.

---

## Analysis Handoff (added 2026-06-27)

Analysis complete. See `analysis.md` for full research. Key confirmed decisions:
- 14 changes confirmed, grouped into sub-phase A (core, 1–6) and sub-phase B (integration, 7–14)
- `sycophancy-correction` MCP NOT used as evaluator — wrong abstraction; build `kbd-goal-evaluator` agent
- OpenCode: adopt `@prevalentware/opencode-goal-plugin`
- Codex: adopt native `codex /goal` + KBD-provided `continuation.md`/`budget_limit.md` templates
- Kimi: build `kbd-goal-check` evaluator skill (queue model needs condition checker)
- Zed: dual-track (ACP delegation vs. standalone loop emulation)
- All platforms: build `kbd-goal` SKILL.md as unified entry point with platform detection
