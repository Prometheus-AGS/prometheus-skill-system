---
name: kbd-goal
version: '1.0.0'
license: MIT
description: >
  Platform-agnostic goal-driven iterative loop. State a goal; KBD orchestrates
  Ideation → Specification → Creation phases autonomously until the goal is met,
  a budget is exhausted, or human approval is needed. Works identically on Claude
  Code, Codex CLI, OpenCode, Kimi Code, and Zed — routing to each platform's
  native /goal primitive where available, implementing the evaluator pattern
  where not.
metadata:
  author: Prometheus AGS
  category: process
  tags: [process, orchestration, goal, autonomous, loop, multi-platform]
triggers:
  keywords:
    - /kbd-goal
    - goal loop
    - autonomous goal
    - ideation phase
    - specification phase
    - creation loop
    - goal-driven
  semantic: >
    User wants to drive a project or feature from idea to implementation
    autonomously, with KBD managing the iterative loop across phases.
---

# /kbd-goal

Drive a complete goal from statement to implementation using KBD's iterative
loop engine — platform-agnostic, multi-phase, with bias-resistant evaluation.

## Quick Start

```
/kbd-goal "build a weekly standup generator CLI in Go"
/kbd-goal "add dark mode to the dashboard" --phases spec,creation
/kbd-goal "refactor auth module" --phases creation --stop "all tests pass, lint clean"
```

## Invocation Modes

### 1. Full pipeline (default)
```
/kbd-goal "<description>"
```
Runs all three phases: Ideation → Specification → Creation.
Human gate between each phase.

### 2. Multi-phase, explicit
```
/kbd-goal "<description>" --phases ideation,spec,creation
/kbd-goal "<description>" --phases ideation,spec,creation,deployment
```
Specify which phases to run. `deployment` requires `--deploy-target <env>`.

### 3. Creation only (fastest)
```
/kbd-goal "<description>" --phases creation --stop "<stopping condition>"
```
Skip Ideation and Specification — you supply the stopping condition directly.
On Claude Code and Codex, this delegates to native `/goal` for the execution loop.

### 4. With budget
```
/kbd-goal "<description>" --tokens 200000 --max-turns 30
```
Sets a token ceiling and turn limit across all phases.

## What Happens at Start

1. **Skill/MCP discovery** — `kbd-goal-discover.sh` prints recommended skills
   and MCP servers for your goal's domain (advisory, non-blocking).
2. **Goal state created** — `.kbd-orchestrator/goals/<slug>/goal.json` written.
3. **Child phases spawned** — one KBD child phase per requested phase.
4. **Platform routing** — active tool detected; platform-specific strategy selected.
5. **Loop starts** — first phase begins; evaluator runs after each turn.

## Platform Detection

The skill reads `$TOOL` env var → `.kbd-orchestrator/current-waypoint.json → tool`
→ defaults to `claude-code`.

| Detected Tool | Strategy |
|---|---|
| `claude-code` | Single-phase Creation → native `/goal`. Multi-phase → KBD orchestrates, delegates Creation per-phase. Ideation/Spec always KBD-owned. |
| `codex` | KBD writes `goals/continuation.md` + `goals/budget_limit.md`; invokes `codex /goal "<phase-condition>"` per phase. |
| `opencode` | Requires `@prevalentware/opencode-goal-plugin` (auto-installed). KBD sets goal via `create_goal` tool; manages phase transitions via `update_goal`. |
| `kimi` | Uses `/goal next` to queue phases. `kbd-goal-check` skill evaluates stopping condition after each turn. |
| `zed` | ACP-connected to Claude Code/Codex → delegates to their bridge. Standalone → KBD drives loop via `session/prompt` + `kbd-goal-evaluator` subagent. |

## Evaluator Pattern (Bias Resistance)

KBD never uses the builder model to grade its own output. After each execution
turn, `kbd-goal-evaluator` (a separate Haiku-class subagent, read-only) checks
the stopping condition against STATE.md and returns `PASS` or `FAIL + reason`.

This mirrors Claude Code's native `/goal` evaluator design and applies it to all
five platforms uniformly.

## Goal State Files

All goal state lives in `.kbd-orchestrator/goals/<slug>/`:

| File | Contents |
|---|---|
| `goal.json` | Goal definition, phases, status, tool, budgets |
| `IDEAS.md` | Ideation phase output — scored candidates, survivor table |
| `SPEC.md` | Specification phase output — user stories, CLI/API contract, acceptance criteria |
| `TASKS.md` | Creation phase task checklist (`[ ]`, `[/]`, `[x]`, `[~]`) |
| `STATE.md` | Execution state — completed/total, active task, fail counts, escalations, promotions |

## Ideation Phase

> Converges on 3 validated candidates before proceeding.

See [references/templates/ideation-phase.md](references/templates/ideation-phase.md)

**Loop:** discovery agent proposes candidates → `kbd-idea-critic` (Sonnet) scores
each on 4 rubric dimensions → loop continues until ≥3 candidates score ≥7.0
aggregate → human selects from `IDEAS.md`.

## Specification Phase

> Ends only when adversarial reviewer returns PASS.

See [references/templates/spec-phase.md](references/templates/spec-phase.md)

**Loop:** spec-writer drafts `SPEC.md` → `kbd-spec-reviewer` stress-tests for
ambiguity and untestable criteria → writer revises → repeat until PASS →
human approves `SPEC.md`.

## Creation Phase

> Per-task verify loop; auto-promotes complex tasks to child phases.

See [references/templates/creation-phase.md](references/templates/creation-phase.md)

**Loop:** decompose `SPEC.md` → `TASKS.md` → per task: implement + test →
`kbd-task-verifier` checks against SPEC acceptance criteria → PASS (commit,
advance) or FAIL (retry up to 3) → fail≥3 promotes to child phase.

## Inner-Loop Auto-Promotion

When a task fails 3+ times, `kbd-goal-promote.sh` automatically spawns a child
KBD phase (`kbd-new-child <task-slug>`) with full context in `handoff-in.md`.
The parent loop marks the task `[~] promoted` and continues with other tasks.

## Stopping Conditions

The loop stops when ANY of:
- All phases complete (`goal.json → status = complete`)
- Token budget exhausted (`--tokens` ceiling)
- Turn limit reached (`--max-turns`)
- `max_no_progress_turns` consecutive turns with no STATE.md change
- Human escalation triggered (ambiguity, security concern, or explicit pause)

## Skill/MCP Discovery

At goal start, `kbd-goal-discover.sh` analyzes the goal description and prints:
```
Recommended skills: golang-patterns, golang-testing
Recommended MCPs:   context7, surreal-memory
Rationale: Goal mentions Go; golang-patterns covers idioms; context7 for Go docs
```
Load recommended skills with `/skill-name` before the loop begins.

## Resuming

Goals are resumable across sessions:
```
/kbd-goal --resume <slug>
```
Reads `goal.json` + `STATE.md`, picks up from the last incomplete task.

## Human Gates

Each phase ends with a human gate (unless `--auto-gates` is set). Gates use
`/pmpo-elicit` to collect the decision with provenance and record it in `goal.json`.

### Ideation → Spec gate (after evaluator PASS)

When `--auto-gates` is NOT set:

1. Invoke `/pmpo-elicit` with the candidate list from `IDEAS.md`:
   - `question`: "Ideation complete. `IDEAS.md` has `<N>` candidates. Which direction to pursue?"
   - `options`: 2–4 candidate names extracted from IDEAS.md
   - `criticality`: high
   - `caller`: kbd-goal/ideation

   On Claude Code: use `AskUserQuestion` with candidate options.
   On other platforms: call `pmpo-elicit-checkpoint.sh`, write checkpoint to
   `goals/<slug>/elicitations/<id>/`, pause the goal loop.

2. On result:
   - Record in `goal.json → phases[ideation].human_gate_result`:
     `{"decision": "<candidate>", "provenance": "<provenance>", "elicitation_id": "<id>"}`
   - "revision-needed": re-enter ideation loop with revision notes
   - Named candidate: proceed to Spec phase, seed SPEC.md from the selected idea

When `--auto-gates` IS set:
   - Record: `{"decision": "auto-approved", "provenance": "implicit", "elicitation_id": null}`

### Spec → Creation gate (after evaluator PASS)

When `--auto-gates` is NOT set:

1. Invoke `/pmpo-elicit`:
   - `question`: "Specification complete. SPEC.md is ready. How do you want to proceed?"
   - `options`: ["Approve — begin Creation", "Request revision", "Stop here"]
   - `criticality`: high
   - `caller`: kbd-goal/spec

2. On result:
   - "Approve — begin Creation": record approved, proceed to Creation phase
   - "Request revision": re-enter Spec loop with revision notes as evaluator feedback
   - "Stop here": write goal state summary, set `goal.json → status = "stopped-at-spec"`
   - Record in `goal.json → phases[spec].human_gate_result` (same structure as above)

### STATE.md escalations[] — Creation phase

When any elicitation is triggered during the Creation phase (ambiguity, security concern,
or blocked task):

- On checkpoint written: append to `STATE.md → escalations[]`:
  `{"id": "<elicitation-id>", "question": "<question>", "status": "pending", "task_id": "<active-task>"}`
- On result received: update the entry:
  `{"status": "resolved", "provenance": "<provenance>", "answer": "<answer>"}`

See `skills/process/pmpo-elicit/references/escalation-points.md` for the full
platform routing table and async checkpoint contract.

## References

- [Platform: Claude Code](references/platforms/claude-code.md)
- [Platform: Codex CLI](references/platforms/codex.md)
- [Platform: OpenCode](references/platforms/opencode.md)
- [Platform: Kimi Code](references/platforms/kimi.md)
- [Platform: Zed](references/platforms/zed.md)
- [Ideation Phase Template](references/templates/ideation-phase.md)
- [Specification Phase Template](references/templates/spec-phase.md)
- [Creation Phase Template](references/templates/creation-phase.md)
- [Goal Directory Layout](references/goal-directory-layout.md)
- [Skill Discovery](references/skill-discovery.md)
