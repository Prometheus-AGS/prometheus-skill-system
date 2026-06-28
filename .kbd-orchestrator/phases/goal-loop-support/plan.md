# Plan — goal-loop-support

**Phase:** goal-loop-support
**Planned:** 2026-06-27
**Backend:** OpenSpec (`openspec/changes/`)
**Changes total:** 14 (sub-phase A: 6 core, sub-phase B: 8 integration)

---

## Ordering Rationale

Sub-phase A builds the platform-agnostic engine first — evaluator agent, unified entry point, schema, and three phase templates. Nothing in sub-phase B can be tested without sub-phase A in place. Within A, the evaluator agent (goal-001) is first because every other component references it. The schema (goal-003) comes after the entry point spec (goal-002) so we know what fields to define. Ideation → Spec → Creation templates follow in workflow order.

Sub-phase B delivers platform bridges. They are ordered by number of users affected descending: Claude Code first (most users), then Codex, OpenCode, Kimi, Zed. Changes 12–14 (inner-loop promotion, skill discovery, schema extension) are additive polish that depend on the phase templates being in place.

---

## Sub-phase A — Core Engine (changes 1–6)

### change-goal-001: Separated Evaluator Subagent
**File:** `agents/kbd-goal-evaluator.md`
**Verdict:** BUILD
**Depends on:** nothing
**Acceptance criteria:**
- [ ] `agents/kbd-goal-evaluator.md` exists with YAML frontmatter (`name`, `description`, `model: claude-haiku-4-5-20251001`)
- [ ] System prompt instructs agent to: read a stopping condition + `STATE.md` / test output excerpt → return exactly `PASS` or `FAIL` + one-sentence reason
- [ ] Agent has read-only tool access only (no write, no bash execution)
- [ ] Frontmatter declares `disable-model-invocation: false` (can be auto-invoked by orchestrator)
- [ ] Tested manually: given "all tests pass" + a sample STATE.md with failing tests → returns `FAIL` with reason

---

### change-goal-002: `/kbd-goal` Unified Entry Point Skill
**File:** `skills/process/kbd-goal/SKILL.md` + `scripts/kbd-goal-start.sh`
**Verdict:** BUILD
**Depends on:** goal-001 (references evaluator)
**Acceptance criteria:**
- [ ] `skills/process/kbd-goal/SKILL.md` exists, valid agentskills frontmatter with `name: kbd-goal`, `version: 1.0.0`, `license: MIT`, `metadata.tags`
- [ ] Skill documents platform detection: reads `$TOOL` env var OR `.kbd-orchestrator/current-waypoint.json → tool` field; falls back to `claude-code`
- [ ] Skill documents three invocation patterns:
  - `/kbd-goal "description"` — infers single Creation phase, delegates to platform native `/goal` where available
  - `/kbd-goal "description" --phases ideation,spec,creation` — full multi-phase pipeline
  - `/kbd-goal "description" --phases creation --stop "all tests pass, lint clean"` — explicit stopping condition
- [ ] `scripts/kbd-goal-start.sh` creates `.kbd-orchestrator/goals/<goal-slug>/goal.json` and calls `kbd-new-child` per phase
- [ ] Skill is installed to all 5 platform skill dirs by `install-skills-flat.sh`

---

### change-goal-003: `goal.json` Schema + Goals Directory
**Files:** `skills/process/kbd-goal/references/schemas/goal.schema.json`, `skills/process/kbd-goal/references/goal-directory-layout.md`
**Verdict:** BUILD
**Depends on:** goal-002
**Acceptance criteria:**
- [ ] `goal.schema.json` is valid JSON Schema with these fields:
  - `name`, `slug`, `description` (required)
  - `phases[]` — array of `{name, type: ideation|spec|creation|deployment, stopping_condition, human_gate: boolean}`
  - `active_phase`, `status: pending|running|paused|complete|escalated`
  - `created`, `updated` timestamps
  - `tool` — detected platform
  - `token_budget`, `max_turns_per_phase`, `max_no_progress_turns`
- [ ] Schema is backward-compatible additive extension of existing `loop.json` schema
- [ ] `goal-directory-layout.md` documents `.kbd-orchestrator/goals/<slug>/` structure with `goal.json`, `IDEAS.md`, `SPEC.md`, `TASKS.md`, `STATE.md`
- [ ] `kbd-goal-start.sh` validated against schema (jq -e passes)

---

### change-goal-004: Ideation Child-Phase Template
**Files:** `skills/process/kbd-goal/references/templates/ideation-phase.md`, `agents/kbd-idea-critic.md`
**Verdict:** BUILD
**Depends on:** goal-003 (needs goal directory structure)
**Acceptance criteria:**
- [ ] `ideation-phase.md` template documents the convergence loop:
  1. Discovery agent reads goal + user inputs → proposes candidate ideas → writes to `IDEAS.md` (draft section)
  2. `kbd-idea-critic` agent scores each candidate against rubric (feasibility, pain addressed, stack fit, weekend-buildable) → numeric score 0–10 per dimension
  3. Loop continues until ≥ 3 candidates score ≥ 7.0 aggregate, OR `max_turns` reached
  4. Human gate: `IDEAS.md` surfaced to user for selection; loop pauses
- [ ] `agents/kbd-idea-critic.md` subagent defined with: stronger model (Sonnet), scoring rubric in system prompt, JSON output schema `{candidates: [{title, scores: {feasibility, pain, stack_fit, buildability}, aggregate, verdict: PASS|FAIL}]}`
- [ ] `IDEAS.md` format documented (scored table + rationale per candidate)
- [ ] Template is referenced from `kbd-goal/SKILL.md` under `## Ideation Phase`

---

### change-goal-005: Specification Child-Phase Template
**Files:** `skills/process/kbd-goal/references/templates/spec-phase.md`, `agents/kbd-spec-reviewer.md`
**Verdict:** BUILD
**Depends on:** goal-004 (Ideation produces the input)
**Acceptance criteria:**
- [ ] `spec-phase.md` template documents the writer→reviewer loop:
  1. Spec-writer agent reads `IDEAS.md` (selected idea) → drafts `SPEC.md` with: user stories, exact CLI/API signatures, I/O formats, explicit non-goals, acceptance criteria per story
  2. `kbd-spec-reviewer` agent stress-tests for: ambiguity ("nice" → FAIL), untestable criteria ("clean" → FAIL), missing edge cases → writes gaps back
  3. Writer revises; loop until reviewer returns `PASS` OR `max_turns` reached
  4. Human gate: `SPEC.md` surfaced for approval
- [ ] `agents/kbd-spec-reviewer.md` subagent: system prompt is hardline — "You are adversarial. Reject any criterion that cannot be checked by a script or a specific human action. Return PASS only when all criteria are machine-verifiable or precisely human-evaluable."
- [ ] `SPEC.md` output format documented (user stories, CLI/API contract, acceptance criteria table, non-goals)
- [ ] Template referenced from `kbd-goal/SKILL.md` under `## Specification Phase`

---

### change-goal-006: Creation Loop Enhancement
**Files:** `skills/process/kbd-goal/references/templates/creation-phase.md`, `agents/kbd-task-verifier.md`
**Verdict:** BUILD
**Depends on:** goal-005 (SPEC.md is input)
**Acceptance criteria:**
- [ ] `creation-phase.md` template documents the build loop:
  1. On phase start: decompose `SPEC.md` into `TASKS.md` checklist (`[ ] task-NNN: <description> [acceptance]`)
  2. Per task: implementer takes next unchecked task → writes code + tests in isolated git worktree (where platform supports it) → runs tests + linter
  3. `kbd-task-verifier` agent checks result against `SPEC.md` acceptance criteria → `PASS` (commit, advance) or `FAIL` (errors fed back, retry up to `max_retries: 3`)
  4. On 3 consecutive fails: auto-promote task to child phase (calls `kbd-new-child <task-NNN>`) — see goal-012
  5. Anything requiring user input: writes to `STATE.md → escalations[]` and pauses loop
  6. Update `STATE.md` after every task (completed count, current task, fail counts)
- [ ] `agents/kbd-task-verifier.md` subagent: reads `SPEC.md` acceptance criteria + task description + test/lint output → `PASS` or `FAIL` + failure reason
- [ ] `TASKS.md` and `STATE.md` formats documented
- [ ] Template referenced from `kbd-goal/SKILL.md` under `## Creation Phase`

---

## Sub-phase B — Platform Bridges (changes 7–14)

### change-goal-007: Claude Code Bridge
**Files:** `skills/process/kbd-goal/references/platforms/claude-code.md`
**Verdict:** ADOPT (native `/goal`) + thin wrapper
**Library:** `claude-code-goal-native`
**Depends on:** goal-002, goal-006
**Acceptance criteria:**
- [ ] `claude-code.md` platform reference documents routing logic:
  - Single-phase Creation goal → delegate to `claude /goal --tokens <budget> "<stopping_condition>"`
  - Multi-phase goal → KBD orchestrates; per-phase Creation delegates to `claude /goal --worktree "<phase-stopping-condition>"`
  - Ideation and Spec phases never delegate to native `/goal` (KBD owns these)
- [ ] `kbd-goal/SKILL.md` platform detection section updated with Claude Code routing
- [ ] Sample invocations documented with expected output

---

### change-goal-008: Codex CLI Bridge
**Files:** `skills/process/kbd-goal/references/platforms/codex.md`, `skills/process/kbd-goal/templates/codex/continuation.md`, `skills/process/kbd-goal/templates/codex/budget_limit.md`, `scripts/kbd-goal-codex-setup.sh`
**Verdict:** ADOPT (native `codex /goal`) + template files
**Library:** `codex-goal-native`
**Depends on:** goal-002, goal-006
**Acceptance criteria:**
- [ ] `continuation.md` template: instructs Codex to re-read `STATE.md`, take the next unchecked task in `TASKS.md`, implement it, run tests
- [ ] `budget_limit.md` template: instructs Codex to write a progress wrap-up to `STATE.md → budget_summary` and stop
- [ ] `kbd-goal-codex-setup.sh` writes templates to `~/.codex/goals/` and ensures `goals.enabled = true` in `~/.codex/config.toml`
- [ ] `.codex-kbd-context.md` generator: script writes phase-specific context (current goal, active phase, TASKS.md path, stopping condition) for user to `@include` in `AGENTS.md`
- [ ] `codex.md` platform reference documents full routing + setup procedure

---

### change-goal-009: OpenCode Plugin Auto-Install
**Files:** `scripts/install-skills-flat.sh` (modify), `skills/process/kbd-goal/references/platforms/opencode.md`
**Verdict:** ADOPT (`@prevalentware/opencode-goal-plugin`)
**Library:** `opencode-goal-plugin`
**Depends on:** goal-002
**Acceptance criteria:**
- [ ] `install-skills-flat.sh` gains an OpenCode section that:
  - Detects OpenCode is installed (`command -v opencode`)
  - Checks if goal plugin installed (`opencode plugins list | grep -q goal-plugin`)
  - If missing: runs `npx @prevalentware/opencode-goal-plugin install`
  - Writes plugin config block to `.opencode/config.toml` (or `~/.opencode/config.toml`):
    ```toml
    [goal_plugin]
    auto_continue = true
    max_auto_turns = 20
    no_progress_token_threshold = 5000
    max_no_progress_turns = 3
    default_token_budget = 200000
    ```
- [ ] KBD's `kbd-goal` skill documented for OpenCode: uses `create_goal` agent tool to set goal state, then per-phase advancement via `update_goal` + phase transition
- [ ] `opencode.md` platform reference documents full routing + plugin install procedure

---

### change-goal-010: Kimi Code Evaluator Skill
**Files:** `skills/process/kbd-goal-check/SKILL.md`
**Verdict:** BUILD
**Depends on:** goal-001 (evaluator pattern), goal-003 (STATE.md/goal.json)
**Acceptance criteria:**
- [ ] `skills/process/kbd-goal-check/SKILL.md` exists, valid frontmatter (`name: kbd-goal-check`, `description: Evaluate goal stopping condition after each Kimi turn`)
- [ ] Skill body instructs agent to:
  1. Read `goal.json → phases[active_phase].stopping_condition`
  2. Read `STATE.md` and/or run the condition's check command
  3. Return: `PASS` (with evidence quote) → call Kimi's goal completion; `CONTINUE` (with next action hint from TASKS.md)
- [ ] `kbd-goal/SKILL.md` updated: Kimi section documents `/goal next <phase-condition>` for phase queueing + `kbd-goal-check` for condition evaluation
- [ ] `kimi.md` platform reference documents `/goal next` queue pattern + evaluator skill integration
- [ ] Skill installed to `~/.kimi-code/skills/` by `install-skills-flat.sh`

---

### change-goal-011: Zed Dual-Track Strategy
**Files:** `skills/process/kbd-goal/references/platforms/zed.md`, `scripts/kbd-goal-zed-detect.sh`
**Verdict:** BUILD (ACP detection + loop emulation fallback)
**Depends on:** goal-001 (evaluator), goal-002 (entry point)
**Acceptance criteria:**
- [ ] `kbd-goal-zed-detect.sh` detects ACP-connected agent:
  - Checks `$ZED_ACP_AGENT` env var
  - Falls back to checking `~/.zed/acp-agents.json` for active connection
  - Outputs: `claude-code`, `codex`, or `standalone`
- [ ] `kbd-goal/SKILL.md` Zed section:
  - ACP-connected to Claude Code → delegate to Claude Code bridge (goal-007)
  - ACP-connected to Codex → delegate to Codex bridge (goal-008)
  - Standalone → KBD implements loop: after each turn, invoke `kbd-goal-evaluator` subagent; if `FAIL`, inject continuation guidance as next Zed `session/prompt`
- [ ] `zed.md` platform reference documents both tracks with setup steps
- [ ] Skill installed to `~/.zed/skills/` by `install-skills-flat.sh`

---

### change-goal-012: Inner-Loop Auto-Promotion
**Files:** `scripts/kbd-goal-promote.sh`, `skills/process/kbd-goal/SKILL.md` (update Creation phase section)
**Verdict:** BUILD
**Depends on:** goal-006 (Creation loop), `kbd-new-child` (existing)
**Acceptance criteria:**
- [ ] `kbd-goal-promote.sh` reads `STATE.md` for a task:
  - If `fail_count >= 3`: calls `kbd-new-child <task-slug>` to create child phase
  - Writes `handoff-in.md` with full task context (description, last 3 failure reasons, SPEC.md acceptance criteria for this task)
  - Updates parent `TASKS.md`: marks task as `[~] promoted to child: <task-slug>`
  - Updates `STATE.md → promotions[]`
- [ ] `STATE.md` schema updated to include `fail_count` per task and `promotions[]` array
- [ ] Agent can also set `NEEDS_CHILD_PHASE: true` in STATE.md to trigger promotion on next loop tick
- [ ] Creation phase template (goal-006) updated to reference this promotion logic

---

### change-goal-013: Goal-Time Skill/MCP Discovery
**Files:** `scripts/kbd-goal-discover.sh`, `skills/process/kbd-goal/references/skill-discovery.md`
**Verdict:** BUILD
**Depends on:** goal-002 (entry point calls this at start)
**Acceptance criteria:**
- [ ] `kbd-goal-discover.sh <goal-description>` outputs a JSON block:
  ```json
  {
    "recommended_skills": ["golang-patterns", "golang-testing"],
    "recommended_mcps": ["context7", "surreal-memory"],
    "rationale": "Goal mentions Go; golang-patterns covers idioms; context7 for Go docs"
  }
  ```
- [ ] Discovery uses keyword matching against a mapping file (`skill-discovery.md` — language/domain → skills/MCPs table)
- [ ] Output is printed to user at goal start as an advisory (not blocking)
- [ ] `kbd-goal/SKILL.md` Start section documents: "At goal start, kbd-goal-discover.sh is called and outputs recommended skills/MCPs. Load them with `/skill-name` or configure MCPs before proceeding."

---

### change-goal-014: `loop.json` Schema Extension
**Files:** `skills/process/pmpo-outer-loop/references/loop-schema.md` (update), `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` (update)
**Verdict:** BUILD (additive extension)
**Depends on:** goal-003 (must align with goal.json)
**Acceptance criteria:**
- [ ] `loop-definition.schema.json` gains two new optional fields:
  - `phases[]` — array of `{name, type, stopping_condition, human_gate}` — when present, overrides single-goal model
  - `goal_slug` — link back to `.kbd-orchestrator/goals/<slug>/goal.json`
- [ ] Change is backward-compatible: existing `loop.json` files without `phases[]` continue to work
- [ ] `loop-schema.md` updated with new fields, examples, and cross-reference to `goal.schema.json`
- [ ] `kbd-goal-start.sh` (goal-002) writes both `goal.json` AND a corresponding `loop.json` in `.kbd-orchestrator/loops/<slug>/` so `/loop-tick` can drive goal advancement

---

## Waypoint After Plan

- **changes_total:** 14
- **next_pending_change:** `change-goal-001`
- **stage:** `execute_ready`
- **exact_next_command:** `/kbd-execute`

## Scoped Paths

```
agents/kbd-goal-evaluator.md
agents/kbd-idea-critic.md
agents/kbd-spec-reviewer.md
agents/kbd-task-verifier.md
skills/process/kbd-goal/SKILL.md
skills/process/kbd-goal/scripts/kbd-goal-start.sh
skills/process/kbd-goal/references/schemas/goal.schema.json
skills/process/kbd-goal/references/goal-directory-layout.md
skills/process/kbd-goal/references/templates/ideation-phase.md
skills/process/kbd-goal/references/templates/spec-phase.md
skills/process/kbd-goal/references/templates/creation-phase.md
skills/process/kbd-goal/references/platforms/claude-code.md
skills/process/kbd-goal/references/platforms/codex.md
skills/process/kbd-goal/references/platforms/opencode.md
skills/process/kbd-goal/references/platforms/kimi.md
skills/process/kbd-goal/references/platforms/zed.md
skills/process/kbd-goal/references/skill-discovery.md
skills/process/kbd-goal/templates/codex/continuation.md
skills/process/kbd-goal/templates/codex/budget_limit.md
skills/process/kbd-goal-check/SKILL.md
skills/process/pmpo-outer-loop/references/loop-schema.md
skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json
scripts/install-skills-flat.sh
scripts/kbd-goal-start.sh
scripts/kbd-goal-codex-setup.sh
scripts/kbd-goal-zed-detect.sh
scripts/kbd-goal-promote.sh
scripts/kbd-goal-discover.sh
```
