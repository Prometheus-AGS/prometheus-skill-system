---
license: MIT
name: kbd-process-orchestrator
version: '2.0.0'
description: >
  The Universal Knowledge-Based Development (KBD) process orchestrator. Drives
  the full iterative PMPO lifecycle for ANY project — Assess → Analyze → Plan →
  Execute (backend dispatch) → Reflect — at every granularity level: global
  phases, OpenSpec changes, and artifact-level QA. Implements the process
  defined in TJ-KBD-UNIVERSAL-001. Coordinates execution across multiple AI
  tools (Claude Code, Codex, OpenCode, Kimi, and compatible adapters) using the
  append-only KBD runtime as the shared control plane.
authors:
  - 'Prometheus AGS'
allowed-tools: file_system web_search code_interpreter sequential_thinking memory
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    kbd-assess: frontier
    kbd-plan: frontier
    kbd-status: small
    kbd-pause: small
    kbd-resume: small
    kbd-cancel: small
    kbd-audit: small
    kbd-handoff: small
    kbd-reflect: frontier
    opsx-new: small
    opsx-apply: tiered
    opsx-verify: medium
    opsx-archive: small
  routing_reference: "references/model-routing.md"
triggers:
  keywords:
    - kbd
    - phase
    - openspec
    - planning phase
    - reflect phase
    - kbd-plan
    - kbd-assess
    - kbd-execute
    - kbd-reflect
    - kbd-status
    - iterative evolver
  semantic: >
    Orchestrate a project phase, run the KBD assessment loop, create or manage
    OpenSpec changes, invoke artifact-refiner QA, or generate a phase
    reflection report.
metadata:
  tags: [process, orchestration, automation]
---

# KBD Process Orchestrator

The universal process orchestrator for any software project. Implements the Knowledge-Based Development lifecycle defined in **TJ-KBD-UNIVERSAL-001** using PMPO orchestration at three nested levels: phase, change, and artifact.

This skill is **project-agnostic**. It derives project identity from context ([AGENTS.md](http://AGENTS.md), [CLAUDE.md](http://CLAUDE.md), [README.md](http://README.md), package.json, Cargo.toml, pyproject.toml, or explicit prompt arguments). Do not hard-code project names into this skill.

---

## Progress Signals (MANDATORY)

Every KBD skill emits signals at the start and end of its work. The orchestrator coordinates but does not double-emit — individual skills own their own signals.

**Cross-phase signal rule**: When orchestrating a full phase cycle (Assess → Plan → Execute → Reflect), emit phase-level signals at the orchestrator level:

```
Starting phase <N> of <total>: <phase-name>
```

When the phase cycle is complete:

```
Completed phase <N> of <total>: <phase-name>
```

Read `changes_total` and the phase list from `progress.json` or `current-waypoint.json` for accurate totals — never guess. Emit to plain response text — no tool call needed. Individual skills (`kbd-assess`, `kbd-plan`, etc.) emit their own skill-level signals independently.

---

## Project Context Discovery

On every invocation, before acting, KBD MUST:

1. **Identify the project name** — read, in priority order:
   - Explicit argument (`/kbd-plan my-project`)
   - `.kbd-orchestrator/project.json` if it exists
   - `AGENTS.md` header or `CLAUDE.md` header
   - `README.md` first heading
   - `package.json` → `name`, `Cargo.toml` → `[package] name`, `pyproject.toml` → `name`

2. **Identify project-specific constraints** — read `AGENTS.md` and the
   project's canonical spec files (OpenSpec `openspec/specs/*.md`, or
   equivalent spec directory defined in `.kbd-orchestrator/project.json`).

3. **Derive the technology stack** — identify from lock files / config files
   to tailor the Build Health and Test Coverage assessment dimensions.

---

## The Three Levels

### Level 1 — Global Phase (this skill)

Assess → Analyze → Plan → Execute (backend selection + dispatch) → Reflect
KBD owns canonical phase state and delegates execution to OpenSpec, a native
planner backend, or a designated AI tool.

### Level 2 — OpenSpec Change (inner loop)

`/opsx:new` (change created in **plan**) → **`/kbd-apply`** (drives the spec
backend task-by-task in **execute**) → `verify` → `archive`.
`/kbd-apply` wraps the OpenSpec CLI **one task at a time** so KBD hooks fire and
`progress.json`/the waypoint stay in sync. Never invoke bare `/opsx:apply` — it
runs outside KBD. Delegates QA to `artifact-refiner` when available.

### Level 3 — Artifact QA (innermost)

`artifact-refiner` Specify → Plan → Execute → Reflect → Persist

---

## Multi-Tool Coordination Architecture

KBD uses the append-only event journal in
`.kbd-orchestrator/runtime/events.jsonl` as its universal coordination
contract. Harnesses mutate it through the CLI, REST, or MCP contract while one
writer holds the fenced lease. Compatibility JSON is generated from replay and
is never an independent authority.

### Runtime and compatibility files

| File                                             | Written by         | Read by     | Purpose                           |
| ------------------------------------------------ | ------------------ | ----------- | --------------------------------- |
| `.kbd-orchestrator/runtime/events.jsonl`         | kbd-runtime        | All adapters | Canonical append-only journal |
| `.kbd-orchestrator/current-waypoint.json`        | projection writer  | All tools   | Derived resume view |
| `.kbd-orchestrator/current-waypoint.md`          | Any orchestrator   | All tools   | Human-readable waypoint summary   |
| `.kbd-orchestrator/phases/<phase>/assessment.md` | kbd-assess         | kbd-analyze/kbd-plan | Gap analysis output      |
| `.kbd-orchestrator/phases/<phase>/analysis.md`   | kbd-analyze        | kbd-spec/kbd-plan | Engineering-landscape research |
| `.kbd-orchestrator/phases/<phase>/library-candidates.json` | kbd-analyze | kbd-spec/kbd-plan | Build-vs-adopt candidate set |
| `.kbd-orchestrator/phases/<phase>/handoffs/*.handoff.json` | each stage | next stage gate | Stage precondition + summary |
| `.kbd-orchestrator/position.json`                | projection writer  | kbd-status/renderer | Revision-bound derived position tree |
| `.kbd-orchestrator/phases/<phase>/plan.md`       | kbd-plan           | kbd-execute | Ordered change list               |
| `.kbd-orchestrator/phases/<phase>/execution.md`  | kbd-execute        | All tools   | Backend dispatch contract         |
| `.kbd-orchestrator/phases/<phase>/progress.json` | projection writer  | kbd-status  | Derived implementation/evidence/certification/publication ledger |
| `.kbd-orchestrator/phases/<phase>/reflection.md` | kbd-reflect        | Next phase  | Phase retrospective               |
| `.kbd-orchestrator/project.json`                 | Initial setup      | All tools   | Project identity + config         |

### progress.json Protocol

All workflow mutations MUST use `prometheus kbd` typed commands (or equivalent
MCP/REST commands). `progress.json` is read-only when `generatedBy` is
`kbd-runtime`; `sourceRevision` identifies the exact canonical revision.
Legacy shadow-mode writers are migration inputs only and must stop at cutover.

```json
{
  "schemaVersion": "2",
  "phase": "<phase-name>",
  "last_updated": "<ISO 8601 timestamp>",
  "last_updated_by": "<tool-name: antigravity|roo|cursor|codex|cline|opencode|windsurf|human>",
  "implementation_total": 0,
  "implementation_completed": 0,
  "changes_total": 0,
  "changes_completed": 0,
  "completion": {
    "primaryCounter": "implementation",
    "implementation": { "completed": 0, "total": 0, "status": "PENDING" },
    "evidence": { "status": "NOT_TRACKED", "summary": null, "blockers": [] },
    "certification": { "status": "NOT_TRACKED", "summary": null, "blockers": [] },
    "publication": { "status": "NOT_TRACKED", "summary": null, "blockers": [] }
  },
  "changes": [
    {
      "id": "<change-id>",
      "status": "PENDING|IN_PROGRESS|DONE|BLOCKED|SKIPPED",
      "implementation_status": "PENDING|IN_PROGRESS|COMPLETE|BLOCKED|SKIPPED",
      "evidence_status": "NOT_TRACKED|NOT_REQUIRED|PENDING|IN_PROGRESS|COMPLETE|BLOCKED",
      "certification_status": "NOT_TRACKED|NOT_REQUIRED|PENDING|IN_PROGRESS|COMPLETE|BLOCKED",
      "publication_status": "NOT_TRACKED|NOT_REQUIRED|PENDING|IN_PROGRESS|COMPLETE|BLOCKED",
      "tasks_total": 0,
      "tasks_done": 0,
      "last_task_completed": "<task description or null>",
      "next_task_pending": "<task description or null>",
      "started_by": "<tool-name>",
      "completed_by": "<tool-name or null>",
      "blockers": []
    }
  ]
}
```

The ledger's canonical counter is `completion.implementation`. Legacy
`changes_completed` and `changes_total` remain compatibility aliases of that
counter only. They MUST NOT count evidence, certification, authorization,
elapsed-time, external-adopter, or publication gates.

**Completion invariant:** once a change's code and integration contract is
implemented, run `prometheus kbd change transition ... --status complete`;
the projection then shows `implementation_status: COMPLETE` even when evidence
or publication remains pending. Unchecked OpenSpec tasks marked
`EVIDENCE`, `TIME_BOUND`, `AUTHORIZATION`, or `EXTERNAL` never reopen
implementation. Never edit counters in the projection.

### Tool Registry

> Detailed registry, knowledge stack, integration layer, integration timing
> diagram, and the typical phase progression are in
> [`references/architecture.md`](references/architecture.md). Read on first
> use; not loaded on every invocation.

---

## Execution Model (PMPO Loop)

### Startup — always do this first

1. **Discover project identity** — follow Project Context Discovery above
2. **Load waypoint** — read `.kbd-orchestrator/current-waypoint.json` as the
   preferred resume contract before inferring the next action
3. **Load phase context** — identify active phase, load existing phase artifacts
4. **Load domain knowledge** — read `AGENTS.md`, spec files
5. **Check runtime status** — use `prometheus kbd status --json`; use
   `progress.json` only as a revision-matched human-readable projection

### Loop

1. **Assess** (`prompts/assess.md`) — inspect repo, reconcile with spec, surface gaps
2. **Analyze** — identify highest-leverage missing features, prioritize
3. **Plan** (`prompts/plan.md`) — produce ordered list of changes for this phase
4. **Execute** (`prompts/execute.md`) — select backend, write `execution.md`, dispatch
5. **Reflect** (`prompts/reflect.md`) — run evolver report, capture lessons, seed next phase
6. **Persist** — write phase state, refresh waypoint, commit

After each phase: checkpoint + dispatch workflow triggers.

---

## OpenSpec Availability

OpenSpec is **optional**. KBD adapts:

### When OpenSpec IS available (`openspec/` directory exists)

- Use `/opsx:new` to create structured changes with proposal → design → tasks
- Progress tracked in `openspec/changes/<id>/tasks.md`
- Archiving via `/opsx:archive` feeds the reflection phase

### When OpenSpec is NOT available

- Use KBD's built-in change management via `.kbd-orchestrator/changes/<id>/`
- Create `change.md` (same structure as OpenSpec proposal + tasks combined)
- Track task status with `[ ]` / `[/]` / `[x]` in `change.md`
- Archive by moving to `.kbd-orchestrator/changes/archive/<date>-<id>/`

KBD **never** requires OpenSpec. The `execution.md` format accommodates both.

---

## Wayfinding State

KBD maintains a resumable return point for the current phase.

- Canonical files:
  - `.kbd-orchestrator/current-waypoint.md`
  - `.kbd-orchestrator/current-waypoint.json`
- Minimum fields:
  - `active_phase` — current phase name
  - `backend` — selected execution backend
  - `last_completed_change` — last archived/completed change ID
  - `next_pending_change` — next change to start
  - `preferred_re_entry_skill` — which skill to invoke on next session
  - `exact_next_command` — the exact `/opsx:new`, `/kbd-execute`, etc.
  - `fallback_command` — what to do if primary command fails

When the waypoint exists, any AI tool should consult it before deriving
status from broader phase discovery.

### Nested phases

The waypoint supports a *parent → child* relationship between phases via three
optional fields. They are additive: every existing reader keeps working when
the fields are absent, and tools that don't recognise them must silently ignore
them.

- `parentPhase: string | null` — name of the enclosing phase when this row
  represents a child. Default `null` (top-level phase).
- `childPhases: string[]` — ordered list of child-phase names owned by this
  row's phase. Default `[]`. The order is the canonical iteration order used by
  `/kbd-next-child`.
- `childPointer: string | null` — name of the currently-active child within
  `childPhases`, or `null` when no child is active. Default `null`.

**Cross-field invariants** (writer-enforced):

- `childPointer`, when non-null, MUST be a member of `childPhases`.
- `childPhases` MUST NOT contain duplicates.

Writers (`/kbd-new-child`, `/kbd-next-child`) reject inconsistent waypoints;
readers (`/kbd-status`) render best-effort and warn on violations.

**Arbitrary-depth nesting (v3 `path[]`).** The waypoint also carries `path:
string[]` — the canonical position chain for *any* nesting depth, synthesized
additively from the v2 fields when absent. Lifecycle: `/kbd-new-child` →
`/kbd-next-child` (select) → `/kbd-child-exit --enter` (descend) →
`/kbd-child-exit` (close + roll up + pop). **Full depth model, node-dir
resolution, and the critical selected-vs-entered invariant (when
`/kbd-new-child` nests vs. siblings — read before touching `path[]` directly):
[`references/nested-phases.md`](references/nested-phases.md).**

**Template versioning.** The canonical template at
`references/schemas/current-waypoint.template.json` carries `__schemaVersion:
"3"` as **documentation only**. No skill reads `__schemaVersion` at runtime;
the only contract is the per-field default declared in the template and in
this section. Writers MAY set the field; readers MUST NOT depend on it.

> **Scope-guard note.** The scope guard and child-scope hook ship in `warn`
> mode; the flip to `ask` is held until a reload session confirms they fire
> correctly in live operation.

**Worktree integration.** `project.json`'s `worktreeRoot` (default
`${HOME}/.claude/worktrees`) is consumed by `/kbd-status` to render the active
checkout and warn when outside the configured root. See `skills/kbd-status/SKILL.md`.

---

## Hooks

KBD ships an extensible hook surface fired around every lifecycle boundary:
each skill fires `<kind>:<edge>` events (`kind` ∈ phase/child/plan/execute/
reflect/task/assess/spec/analyze, `edge` ∈ before/after), and any project can
plug in *augment* or *override* entries via `.kbd-orchestrator/hooks-config.json`.
The built-in `report-progress` reporter writes `starting/ending <kind> <name>`
to stderr — telemetry, NOT the user-facing guarantee (that is the plain-text
Progress Signals every skill emits).

**Full reference — event taxonomy, legacy aliases, discovery order, per-fire
`KBD_HOOK_*` context, the wiring stanza, hook log schema, and debugging — lives
in [`references/hooks.md`](references/hooks.md).**

---

## Cross-Tool Reporting Protocol

When an AI tool (Roo, Cursor, Cline, Codex, etc.) is dispatched to execute a KBD
change, it MUST follow the start/during/completion/blocker protocol — update
`progress.json` + the waypoint and commit `.kbd-orchestrator/` on each boundary.
**Full steps in [`references/cross-tool-protocol.md`](references/cross-tool-protocol.md).**

---

## Blocking Constraints (Project-Derived)

Unlike the previous version, KBD does not hard-code Rust/DocuMind constraints.
Project constraints are defined in:

1. `AGENTS.md` — "Never Do" and code style rules
2. `.kbd-orchestrator/constraints.md` — project-specific blocking/warning rules

The executing tool MUST read these files and apply constraints when verifying work.

---

## Required Tools

- `file_system` — Read/write spec files, phase reports, progress ledger
- `sequential_thinking` — Multi-step phase planning and gap analysis

## Optional Tools

- `web_search` / `tavily` — External research during Analyze phase
- `code_interpreter` — Run build/test commands during QA
- `surreal-memory` — Cross-session persistence, multi-tool coordination, Graph-RAG queries
- `memory` — Generic cross-session persistence for phase state

## Surreal-Memory Integration

**Default-on when reachable.** When the surreal-memory MCP endpoint is detected,
KBD mirrors every hook fire into the store as a `kbd_lifecycle_event` entity and
exposes `/kbd-memory-recall` for prior-work retrieval; it cleanly no-ops when the
endpoint is unreachable. Built-in hooks: `kbd-memory-log` (`*:*`) and
`auto-memory-recall` (`assess:before`). **Detection contract, what the
integration provides, and the entity schema are in
[`references/memory-integration.md`](references/memory-integration.md).**

---

## Quick Start Commands

### First use in a new project

```
/kbd-init               # Auto-discover project and generate .kbd-orchestrator/project.json
/kbd-new-phase <name>   # Start the first phase
/kbd-assess             # Run the first assessment
```

> **IMPORTANT — project.json is GENERATED, not shipped.**
> `.kbd-orchestrator/project.json` is always created by `/kbd-init` using auto-discovery.
> It lives in the project repository, not in this skill directory.
> The skill only ships the template: `references/schemas/project.template.json`.
> Never commit project-specific values into the skill files.

### Ongoing workflow

- `/kbd-init [--force] [--dry-run]` — Initialize or re-initialize project context
- `/kbd-assess [phase-name]` — Assess current codebase against active phase goals
- `/kbd-plan [phase-name]` — Create prioritized change list for current phase
- `/kbd-execute [phase-name]` — Select execution backend and dispatch phase
- `/kbd-apply <change>` — Drive the spec backend (OpenSpec/Spec Kit) one task at a time, firing per-task hooks + position signals (implemented in `skills/kbd-apply/`). Replaces bare `/opsx:apply`.
- `/kbd-reflect [phase-name]` — Generate phase reflection report + seed next phase
- `/kbd-status` — Show current phase, change inventory, and waypoint-guided next action
- `/kbd-pause` — Checkpoint and suspend the active run
- `/kbd-resume` — Resume a paused run at a validated plan revision
- `/kbd-cancel` — Gracefully terminate the active run
- `/kbd-audit` — Inspect causal history, ownership, and drift
- `/kbd-handoff` — Transfer the single-writer lease to another harness
- `/kbd-new-phase <name> [goals...]` — Start a new named phase with goals (implemented in `skills/kbd-new-phase/`)
- `/kbd-new-child <name> [goals...]` — Spawn a child phase inside the active top-level phase (implemented in `skills/kbd-new-child/`)
- `/kbd-next-child [<name>]` — Advance childPointer (implicit) or jump to a named child (implemented in `skills/kbd-next-child/`)
- `/kbd-child-exit [--enter]` — Exit the active child (handoff-out + roll up + pop path) or, with `--enter`, descend into the selected child so new children nest under it (implemented in `skills/kbd-child-exit/`)
- `/kbd-memory-recall [<phase>]` — Populate `prior-context.md` from surreal-memory before assess (implemented in `skills/kbd-memory-recall/`)
- `/kbd-inject-agent-rules [--target …] [--refresh] [--dry-run]` — Inject Karpathy + Boris Cherny rule sets into CLAUDE.md / AGENTS.md (implemented in `skills/kbd-inject-agent-rules/`)
- `/kbd-full-phase <name>` — Run full Assess → Plan → Execute → Reflect cycle

See `references/domain/kbd.md` (KBD philosophy), `references/cross-tool-handoff.md` (multi-tool coordination), and `prompts/` (detailed phase execution protocols).

---

## Evolver Bridge — Reflect Read-back

When `/kbd-reflect` runs and `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`
exists, the reflect phase must propagate KBD results back to the evolver's
outer state so `/evolve-status` shows accurate per-item completion.

### Protocol

1. **Read bridge** — load `item_to_change_map` and `execution_results` from
   `evolver-bridge.json`.

2. **Compute per-item status** — for each evolver item in `item_to_change_map`:
   - All mapped changes `completed` → item status = `completed`
   - Any mapped change `failed` → item status = `failed`
   - Any mapped change still `pending`/`in_progress` → item status = `in_progress`
   - No mapped changes in `execution_results` yet → item status = `pending`

3. **Write status to evolver state** — update
   `.evolver/evolutions/<evolution_name>/state.json` by merging
   `kbd_results` under `current_iteration`:

   ```json
   {
     "kbd_results": {
       "phase": "<phase-name>",
       "reflected_at": "ISO8601",
       "items": {
         "evolver-item-1": "completed",
         "evolver-item-2": "in_progress"
       }
     }
   }
   ```

4. **Signal outer loop readiness** — if ALL evolver items are `completed`:
   set `current_iteration.status` to `ready_for_reflect` in evolver state,
   which allows `/evolve-reflect` to proceed without waiting for another
   executor run.

5. **No bridge → no-op** — when `evolver-bridge.json` is absent, `/kbd-reflect`
   runs as normal with no evolver state writes.

### Bridge file schema

See `.kbd-orchestrator/changes/change-slli-007-evolver-bridge-integration/bridge-schema.md`
for the full schema narrative and field definitions.
