# Loops Architecture & Construction Spec

**Driving the kbd orchestrator + OpenSpec across Claude Code (`/loop` + Opus 4.8), OpenCode (GLM‑5.2), and Codex (GPT‑5.5)**

Scope: how to architect, construct, and operate long‑running agentic loops on top of the `prometheus-skill-pack` you already built, so that the *same* durable state substrate runs under three different harnesses. Written to live in `docs/` of the skill pack.

---

## 0. The one idea that makes this portable

> **The loop body is harness‑specific. The loop state is harness‑agnostic.**

Your pack already commits to this. The state lives on disk:

- `.kbd-orchestrator/` — the tactical state machine: `current-waypoint.json`, `position.json`, `phases/<name>/{plan.md,progress.json}`, `changes/<id>/`, `hooks-config.json`, `memory-outbox.jsonl`.
- `openspec/` — the artifact‑driven change lifecycle: `openspec/changes/<id>/` (proposal → tasks → spec deltas) and `openspec/specs/` (the living, approved spec).
- `.evolver/` (or surreal‑memory) — the strategic state: named evolutions, checkpoints, history.
- `.kbd-orchestrator/loops/<name>/loop.json` — the standing outer loop definition.

Every harness gets the *same* command surface installed by `scripts/install-skills-flat.sh` / `npm run install:platforms`:

| Harness | Command/skill dir | Driver mechanism |
|---|---|---|
| Claude Code | `.claude/commands/kbd-*.md`, `.claude/commands/opsx/` | slash commands + hooks + `/loop` |
| OpenCode | `.opencode/commands/*.md` + `.opencode/plugin.ts` (`kbd`/`evolve`/`gitops` tools) | plugin tools + headless `opencode run` |
| Codex | `.codex/skills/*` + `.codex/config.toml` | skills + MCP + headless `codex exec` |
| Cursor / Windsurf / Cline | `.cursor/commands/`, `.windsurf/workflows/`, `.clinerules/workflows/` | workflow files |

**Consequence:** to "use loops from now on" across tools, you never re‑implement the loop. You keep one state substrate and swap only the *driver* and the *cadence primitive*. Pick the harness per job (Opus for hard reasoning phases, GLM‑5.2 for cheap/local high‑volume execution, GPT‑5.5 where you want its harness), and they all read and advance the same files.

---

## 1. Theory: why loops, and the four loop layers

### 1.1 Loops vs. prompting

A single prompt is open‑loop: you specify the action, the model acts once, you inspect, you re‑prompt. A loop is closed‑loop: you specify the **goal** and the **feedback sources**, and the system iterates action → observation → correction until a **termination predicate** fires. The shift is from "write the next step" to "write the loop once." Your `pmpo-outer-loop` skill states the thesis directly: *write the loop; the framework discovers, researches, executes, and reports until the goal is met, pinging you only at declared decision points.*

The reason this works now and didn't 18 months ago is long‑horizon model competence plus durable external state. The model no longer has to hold the whole trajectory in context — it reconstructs position from `current-waypoint.json` and `progress.json` every tick. That is what makes a loop **resumable** and therefore safe to leave running.

### 1.2 The four layers

Construct loops as nested control loops, each with its own state file and its own termination predicate. Inner loops converge fast and cheap; outer loops converge slow and strategic.

```
L3  OUTER STANDING LOOP        goal + feedback_sources + termination + cadence
    pmpo-outer-loop            state: .kbd-orchestrator/loops/<name>/loop.json
    │   one tick = one evolver cycle
    ▼
L2  STRATEGIC EVOLVER LOOP      assess→analyze→plan→execute→reflect→persist
    iterative-evolver          state: .evolver/ or surreal-memory (named evolution)
    │   execute() in software domain delegates to ↓
    ▼
L1  TACTICAL KBD LOOP           assess→plan→execute→reflect over ONE phase/change
    kbd-process-orchestrator   state: .kbd-orchestrator/phases/* + openspec/changes/*
    │   execute backend = openspec | native-tool | hybrid | manual
    ▼
L0  HARNESS MICRO-LOOP          read→act→observe (the built-in agent tool loop)
    Claude Code / OpenCode / Codex agent runtime — you bound it, you don't build it
```

- **L0** you do not construct — it is the harness's agent runtime. You *bound* it: `--max-turns`, `/loop` interval, GLM effort level, `--max-budget-usd`.
- **L1 (KBD)** is one unit of shippable work — a phase or an OpenSpec change. This is where code actually changes. Durable, auditable, OpenSpec‑gated.
- **L2 (Evolver)** drives many L1 changes toward a goal, adding landscape research (`web_search`/`tavily`) and reflection across changes.
- **L3 (Outer)** is the "walk away" layer: a goal, feedback sources, termination guards, and a cadence (manual / background / cron). One L3 tick = one L2 cycle.

### 1.3 The Karpathy learning loop (the cross‑cutting feedback layer)

Orthogonal to L0–L3, your `change-006-karpathy-loop-hooks` wires a learning loop through Claude Code hooks so each iteration is *informed by* and *writes back to* memory:

- `UserPromptSubmit` → `pk-focus-on-prompt.sh` injects relevant `prometheus-knowledge` context into the prompt (top‑5 keywords → `pk focus`, 2.5s timeout, silent no‑op if `pk` absent).
- `Stop` → `forge-reflect-on-stop.sh` runs `forge reflect` then `pk ingest` when `.forge/iterations/` exists — the session's lessons re‑enter the knowledge base.
- `SubagentStop[reflector]` → `sycophancy-check-reflection.sh` gates the reflection artifact (Delta → Root Cause → Corrective Actions), rejecting sycophantic "everything went great" reflections, with a 2‑rejection soft cap to avoid infinite loops.

This is the loop *around* the loops: focus in, reflect out, ingest, repeat. It is currently Claude‑Code‑native (hooks); §6 covers replicating it under OpenCode and Codex.

---

## 2. The shared state substrate (read this before constructing anything)

### 2.1 kbd orchestrator state

```
.kbd-orchestrator/
  current-waypoint.json     # { active_phase, last_completed, next_action, changes_completed }
  current-waypoint.md       # human-readable mirror
  position.json             # fine-grained position within a phase
  hooks-config.json         # which lifecycle hooks are armed
  memory-outbox.jsonl       # pending memory writes (flushed by memory-outbox-flush.sh)
  phase-log.md              # append-only phase history
  phases/<phase-name>/
    plan.md                 # the decomposition (canonical task list + totals)
    progress.json           # machine-readable progress (totals MUST be accurate)
  changes/<change-id>/      # kbd-native change records (when backend != openspec)
  loops/<loop-name>/loop.json   # L3 outer-loop definitions
```

The orchestrator never trusts conversational context. Every tick re‑reads `current-waypoint.json` to know where it is. **Progress signaling is mandatory** (CLAUDE.md): emit `Starting phase N out of M: <canonical name>` / `Completed task N out of M: …` to stdout, with totals read from `progress.json` — never estimated.

### 2.2 OpenSpec lifecycle (the L1 execute backend you should default to)

OpenSpec is artifact‑driven and dependency‑gated. A change is a directory of artifacts that must be created in dependency order:

```
openspec/
  specs/                    # living approved capability specs
  changes/<change-id>/      # in-flight change: proposal → tasks → spec deltas
  changes/archive/          # applied + archived changes
```

The `opsx-*` command set wraps the `openspec` CLI:

| Command | CLI it wraps | Purpose |
|---|---|---|
| `/opsx-new <name>` | `openspec new change` → `openspec status` → `openspec instructions` | scaffold a change, show first artifact, **stop** |
| `/opsx-continue` | `openspec instructions <next-ready-artifact>` | draft the next dependency‑ready artifact |
| `/opsx-verify` | `openspec validate` | check artifacts are complete + consistent |
| `/opsx-apply` | apply spec deltas to `specs/` | promote the change into the living spec |
| `/opsx-archive` | move to `changes/archive/` | close out an applied change |
| `/opsx-explore`, `/opsx-ff`, `/opsx-sync`, `/opsx-onboard`, `/opsx-bulk-archive` | — | discovery, fast‑forward, spec sync, onboarding, batch close |

The gate that matters for loops: **an artifact only becomes "ready" when its dependencies are satisfied.** That is your built‑in per‑tick termination signal — the loop advances one artifact per tick and cannot skip ahead.

### 2.3 The model‑routing contract (per‑phase, harness‑independent)

`iterative-evolver` declares routing by phase; honor the same map in every harness:

| Phase | Routing class | Maps to |
|---|---|---|
| assess / analyze / plan / reflect | `frontier` | Opus 4.8 · GLM‑5.2 (Max) · GPT‑5.5 (high) |
| execute | `tiered` | drop to cheap/local per‑task: GLM‑5.2 (High) or self‑hosted, Sonnet/Haiku, GPT‑5.5‑mini |
| status | `small` | cheapest available |

This is your PMPO T1/T2/T3 tiering applied to loop phases: reasoning phases get the frontier model; execution fans out to the cheapest model that can pass the tests.

---

## 3. Native loop primitives per harness (accurate, mid‑2026)

You construct L3 cadence on top of whatever each harness gives you natively.

### 3.1 Claude Code

| Primitive | What it is | Use for |
|---|---|---|
| `/goal <end condition>` | declares what "done" looks like for the session | the termination predicate of an in‑session loop |
| `/loop <interval> /goal …` | in‑session repeating loop; **needs an open session**; tracked in `/tasks`; auto‑expires after 7 days; cancels when the terminal closes | watching/iterating while you stay attached |
| `claude -p "<prompt>"` | headless one‑shot: reads stdin, writes stdout, exits with a status code; flags `--max-turns`, `--max-budget-usd`, `--allowedTools`, `--permission-mode`, `--output-format stream-json` | the body of an *external* loop (cron, CI, bash `while`) |
| Background bash (`Ctrl+B` / `run_in_background`) | moves a long shell command off the critical path | dev servers, test suites — not agentic looping |
| Desktop tasks / Cloud Routines | scheduled, unattended (laptop‑on / Anthropic cloud) | recurring ticks when no session is open |
| Hooks (`UserPromptSubmit`/`Stop`/`SubagentStop`) | shell hooks at lifecycle points | your Karpathy focus/reflect/ingest layer |

**Rule of thumb:** `/loop` when you're watching; `claude -p` in cron/launchd when you're not.

### 3.2 OpenCode (GLM‑5.2)

| Primitive | What it is | Use for |
|---|---|---|
| `.opencode/plugin.ts` tools (`kbd`, `evolve`, `gitops`) | typed entry points; `kbd` returns an `invoke_skill` action pointing at `kbd-process-orchestrator` + `/kbd-<command>`, auto‑detecting `openspec/`, `constraints.md`, and the active waypoint | in‑session L1/L2 from the TUI |
| `.opencode/commands/*.md` (`kbd-*`, `opsx-*`) | markdown slash commands | same surface as Claude Code |
| `opencode run -p "<prompt>"` (headless, non‑interactive) | single‑shot run; all permissions auto‑approved for the run | the body of an external loop |
| `shell.env` hook | injects `PROMETHEUS_SKILL_PACK=1` into every shell env | provenance / guards |
| Plan vs Build agents | Plan reads‑only and proposes; Build edits + runs | map Plan→`assess/plan`, Build→`execute` |
| GLM‑5.2 effort levels (High / Max) + 1M context | per‑call compute lever; 1M window holds a full Cargo workspace | Max for `frontier` phases, High for `execute` |

OpenCode is the model‑agnostic driver: point `execute` at self‑hosted GLM‑5.2 and `assess/plan/reflect` at GLM‑5.2‑Max (or Opus via the Anthropic‑compatible endpoint) — same loop, your choice of provider per phase.

### 3.3 Codex (GPT‑5.5)

| Primitive | What it is | Use for |
|---|---|---|
| `.codex/skills/*` | `kbd-next-phase` + the full `openspec-*` skill set installed | L1/L2 inside Codex |
| `.codex/config.toml` MCP servers | `surreal-memory` (SSE :23001), `liter-llm` (stdio), `sequential-thinking`, `sycophancy-correction`, `tavily` | memory, routing, reflection gate, research — same services as the other harnesses |
| `codex exec "<prompt>"` (headless) | non‑interactive run, exits on completion | the body of an external loop |
| `AGENTS.md` | Codex's rules/memory file | the CLAUDE.md equivalent (your pack ships both) |
| `/goal` | long‑running objective with end condition | in‑session termination predicate |

Because `.codex/config.toml` wires `surreal-memory` and `sycophancy-correction` as MCP servers, Codex gets the same memory substrate and the same reflection gate as Claude Code — the difference is only that Codex reaches them as MCP tools rather than shell hooks.

---

## 4. Canonical loop topology (the thing you run)

This is the full nesting with the actual skills wired in.

```
/loop-define ship-auth                         # L3: write loop.json once
   └─ goal + measurable_criteria
      feedback_sources: [cargo test, gh pr checks, openspec validate]
      termination: { max_ticks: 20, max_no_progress_ticks: 2, budget: 30m }
      escalation_points, cadence: background, evolution_name: ship-auth

/loop-tick ship-auth                           # L3: one tick
   1. read loop.json + last journal.md
   2. collect feedback_sources, diff vs measurable_criteria
   3. satisfied → /loop-report + terminate
      regressed / stalled (max_no_progress) → /pmpo-elicit (continue|replan|stop)
      else → /evolve "ship-auth"               # L2: one evolver cycle
                ├─ assess   (frontier)         # state vs goals
                ├─ analyze  (frontier)         # tavily/web landscape
                ├─ plan     (frontier)         # prioritized improvements
                ├─ execute  (tiered) ──────────┐
                │    └─ /kbd-execute           │ L1: KBD inner loop
                │         ├─ /kbd-plan         │   decompose → openspec change(s)
                │         ├─ backend=openspec  │   /opsx-new → /opsx-continue* → /opsx-verify
                │         │     → /opsx-apply → /opsx-archive
                │         └─ artifact-refiner QA per completed change
                ├─ reflect  (frontier) ────────┘  Delta→RootCause→Corrective (sycophancy gate)
                └─ persist  → state provider (surreal-memory | .evolver/)
   4. append journal.md + decision-log.md
   5. cadence re-arms (background task / cron) or stops (manual)
```

Three guards keep it bounded: `max_ticks` (hard ceiling), `max_no_progress_ticks` (default 2 → escalate on stall), `budget` (per‑tick wall‑time). Every escalation routes through `/pmpo-elicit`, so you are consulted with a concrete decision, never left guessing.

---

## 5. The three long‑running task recipes

### 5.1 Spec‑first code generation — "write the spec, generate the code, prove it"

**Goal:** produce an OpenSpec change whose artifacts fully specify a capability, then generate code that satisfies it, with tests as the termination predicate.

**Topology:** L1 (KBD) with `backend=openspec`, optionally wrapped in L3 for unattended runs.

**Construction (Claude Code, in‑session):**

```bash
# 1. Scaffold the change — produces openspec/changes/<name>/ and shows the first artifact
/opsx-new add-deed-template-matrix

# 2. Draft artifacts one dependency-ready step at a time (proposal → tasks → spec deltas)
/opsx-continue        # repeat until `openspec status --change add-deed-template-matrix` is N/N
/opsx-verify          # openspec validate — gate before any code is written

# 3. Hand the verified spec to the KBD inner loop for implementation
/kbd-plan add-deed-template-matrix          # decompose tasks → fine-grained changes
/kbd-execute add-deed-template-matrix       # backend=openspec; generates code per task
                                            # artifact-refiner QA runs per completed change
/kbd-reflect add-deed-template-matrix       # Delta→RootCause→Corrective (sycophancy-gated)

# 4. Promote + close
/opsx-apply add-deed-template-matrix        # fold spec deltas into openspec/specs/
/opsx-archive add-deed-template-matrix
```

**Make it a loop (unattended):** the termination predicate is *"`cargo test` green AND `openspec validate` clean AND artifact‑refiner score ≥ threshold."* Wrap steps 3–4 in L3:

```bash
/loop-define gen-deed-matrix
# feedback_sources: [{command: "cargo test"}, {command: "openspec validate --change add-deed-template-matrix"}]
# measurable_criteria: "tests pass; spec valid; refiner ≥ 0.8"
# termination: { max_ticks: 12, max_no_progress_ticks: 2 }
/loop-tick gen-deed-matrix     # each tick: one evolver→KBD cycle that pushes the change toward green
```

**Immutable‑tests rule (enforced):** in BDD projects (e.g. `ssr-frontend`), the loop **may not** edit `tests/steps/*.steps.ts`, `tests/support/*.ts`, or `tests/features/*.feature` to make failing tests pass. It surfaces the failure instead. This is what stops a code‑gen loop from "winning" by deleting the test — guarded by `shared/scripts/protect-tests.sh` (PreToolUse).

### 5.2 Ideation specs for new projects — "interrogate the idea into a spec"

**Goal:** turn a vague project idea into a structured specification document, with the loop doing the divergent exploration and convergent structuring.

**Topology:** L2 evolver in `research`/`generic` domain (no code execute backend), composing `ideation-mindmap` + `zeespec-interrogator` + `pmpo-elicit`.

**Construction:**

```bash
# 1. Diverge: build the idea space as a mindmap (process/ideation-mindmap)
#    generate_ideation_mindmap / add_mindmap_node|edge via surreal-memory; export to markdown

# 2. Interrogate to remove under-constraint (process/zeespec-interrogator + pmpo-elicit)
#    the interrogator asks the questions that turn "an app for X" into measurable_criteria

# 3. Run the evolver in research domain — landscape analysis feeds the spec
/evolve "hotseater-v2-ideation"   # domain=research|generic
   # assess: what do we know / not know
   # analyze: tavily/web landscape, prior art, competitors
   # plan: structure the spec (goals, non-goals, constraints, success criteria)
   # execute: write the spec document sections
   # reflect: gaps remaining → next cycle or escalate via /pmpo-elicit

# 4. When convergent, hand off to OpenSpec to make it executable
/opsx-new hotseater-v2-core       # the ideation spec becomes the proposal artifact's seed
```

**Make it a loop:** feedback_source is *coverage of the idea space* (open questions remaining from the interrogator) + *landscape freshness*. Terminate when open questions = 0 and all `measurable_criteria` are stated. Cadence `manual` — ideation wants you in the escalation loop. The evolver chains end‑state → start‑state, so a second `/evolve "hotseater-v2-ideation"` resumes from the prior cycle's finalized spec rather than from scratch.

### 5.3 Evolving an existing codebase — "drive the repo toward a target state"

**Goal:** continuously move a real codebase (UAR, Ferrum Vault, San Saba `ssr-frontend`) toward a target architecture/health state across many changes.

**Topology:** the full nest — L3 → L2 (evolver, `software` domain) → L1 (KBD/OpenSpec) per change.

**Construction:**

```bash
/loop-define harden-uar-runtime
# goal: "UAR runtime: zero clippy warnings, p99 < target, 100% liter-llm provider coverage tests"
# feedback_sources:
#   - { command: "cargo clippy --all-targets -- -D warnings" }
#   - { command: "cargo test -p uar-runtime" }
#   - { gh-query: "is:pr is:open label:uar" }
# termination: { max_ticks: 30, max_no_progress_ticks: 3, budget: 45m }
# cadence: background
# evolution_name: harden-uar-runtime

/loop-tick harden-uar-runtime
#  → /evolve "harden-uar-runtime" (software domain)
#       assess  : cargo metrics + prometheus-rust-auditor findings
#       analyze : crates.io / upstream changes (tavily)
#       plan    : prioritize highest-impact fixes
#       execute : /kbd-execute → one OpenSpec change per fix
#                 (rust-auditor + rust-perf-primitives skills apply in-loop)
#       reflect : measure delta vs last tick; if regressed → /pmpo-elicit
```

The evolver's **nested loop** is the load‑bearing part: `execute()` delegates to KBD, KBD decomposes the plan into OpenSpec changes, runs artifact‑refiner QA per change, and aggregates results back to the evolver's reflect phase via `evolver-bridge.json`. You get strategic direction (evolver) and tactical, auditable, reversible change units (OpenSpec) in one loop.

---

## 6. Per‑harness construction guide (replicating the technique)

The L1/L2/L3 skills are identical across harnesses — only the *driver* and *cadence* differ. Below is exactly what changes.

### 6.1 Claude Code (Opus 4.8) — the reference implementation

**In‑session loop:**
```
/loop 30m /goal cargo test green AND openspec validate clean for change harden-uar-runtime; ping me only on regression
```

**Unattended loop (cron + headless):** the tick body is a headless invocation of your own `/loop-tick`:
```bash
# crontab — every 20 min, advance the standing loop one tick, bounded
*/20 * * * * cd ~/Projects/prometheus/uar && \
  claude -p "/loop-tick harden-uar-runtime" \
    --allowedTools "Bash,Edit,Read" \
    --max-turns 40 --max-budget-usd 2 \
    --permission-mode dontAsk \
    --output-format stream-json \
    >> ~/.prometheus/logs/harden-uar-$(date +\%F).log
```
**Karpathy hooks:** already wired via `hooks/hooks.json` (focus on `UserPromptSubmit`, `forge reflect` + `pk ingest` on `Stop`, sycophancy gate on `SubagentStop[reflector]`). Set `PROMETHEUS_REFLECT_STRICTNESS=strict` for the gate.

**Phase routing:** Opus 4.8 for `assess/analyze/plan/reflect`; drop to Sonnet/Haiku for `execute` sub‑tasks via subagents with bound models.

### 6.2 OpenCode (GLM‑5.2)

**Register the plugin** (installer does this; manual form):
```json
// opencode.json
{ "plugin": ["./.opencode"] }
```
**In‑session:**
```
/kbd status
/evolve "harden-uar-runtime" domain=software phase=full
/opsx-new add-deed-template-matrix
```
The `kbd` tool auto‑detects `openspec/`, `constraints.md`, and `current-waypoint.json` and routes to `/kbd-<command>` — so the loop logic is identical; OpenCode just dispatches it.

**Unattended loop (headless):**
```bash
*/20 * * * * cd ~/Projects/prometheus/uar && \
  opencode run -p "/kbd-execute harden-uar-runtime" \
    >> ~/.prometheus/logs/uar-opencode-$(date +\%F).log
```
**Provider/phase routing (the GLM lever):** point `execute` at self‑hosted GLM‑5.2 (High effort) and `assess/plan/reflect` at GLM‑5.2 **Max** (or Opus via the Anthropic‑compatible base‑URL). The 1M context means `assess` can hold the whole workspace; reserve Max effort for the reasoning phases and High for execution to halve output tokens.

**Replicating the Karpathy layer (no shell hooks):** OpenCode exposes `tool.execute.before/after` (reserved stubs in `plugin.ts`) and `shell.env`. Implement focus/reflect there: in `tool.execute.after`, when a session/loop tick ends, call `forge reflect` + `pk ingest` programmatically; in a `before` guard, inject `pk focus` context. Same behavior, plugin‑native instead of hook‑native.

### 6.3 Codex (GPT‑5.5)

**Config is already wired** (`.codex/config.toml`): `surreal-memory`, `liter-llm`, `sequential-thinking`, `sycophancy-correction`, `tavily` as MCP servers; `.codex/skills/` holds `kbd-next-phase` + the `openspec-*` set; `AGENTS.md` is the rules file.

**In‑session:** invoke the installed skills (`kbd-next-phase`, `openspec-new-change`, …) and use `/goal` for the termination predicate.

**Unattended loop (headless):**
```bash
*/20 * * * * cd ~/Projects/prometheus/uar && \
  codex exec "advance the kbd waypoint one phase for evolution harden-uar-runtime; \
              honor openspec backend; stop at one shippable change" \
    >> ~/.prometheus/logs/uar-codex-$(date +\%F).log
```
**Reflection gate without hooks:** because `sycophancy-correction` is an MCP server in `config.toml`, the reflect phase calls it as a tool (`detect_sycophancy` / `correct_sycophancy`) instead of via a `SubagentStop` shell hook — same Delta→RootCause→Corrective enforcement, same 2‑rejection discipline (implement the soft cap in the skill prompt).

**Phase routing:** GPT‑5.5 (high) for frontier phases; GPT‑5.5‑mini or a `liter-llm`‑routed cheaper model for `execute`.

### 6.4 Cross‑harness parity table

| Concern | Claude Code | OpenCode (GLM‑5.2) | Codex (GPT‑5.5) |
|---|---|---|---|
| L1/L2/L3 skills | `.claude/commands/*` | `.opencode/commands/*` + plugin tools | `.codex/skills/*` |
| Headless tick | `claude -p "/loop-tick <n>"` | `opencode run -p "…"` | `codex exec "…"` |
| In‑session loop | `/loop` + `/goal` | plugin tool + Plan/Build | skill + `/goal` |
| Memory | surreal‑memory MCP / file | surreal‑memory MCP / file | surreal‑memory MCP (`:23001`) |
| Focus‑in | `UserPromptSubmit` hook | `tool.execute.before` | prompt‑time MCP `pk focus` |
| Reflect‑out | `Stop` hook (`forge reflect`+`pk ingest`) | `tool.execute.after` | reflect‑phase MCP call |
| Sycophancy gate | `SubagentStop` shell hook | plugin `after` hook | `sycophancy-correction` MCP |
| Routing per phase | subagents w/ bound models | provider per phase (Max/High) | `liter-llm` route / model flag |

---

## 7. Termination, guards, and safety (do not skip)

A loop you "primarily use from now on" must be bounded and honest, or it will burn budget or fake success.

1. **Hard ceilings:** `max_ticks` (L3), `max_iterations` (L2, default 5), `--max-turns` / `--max-budget-usd` (L0). Always set at least one per layer.
2. **Stall detection:** `max_no_progress_ticks` (default 2). On a stall the loop escalates via `/pmpo-elicit` — continue / replan / stop — rather than spinning.
3. **Reflection honesty gate:** the sycophancy gate rejects "everything went great" reflections; a passing reflection must name the delta between planned and delivered, state root causes, and give corrective actions. 2‑rejection soft cap prevents the gate itself from looping forever.
4. **Immutable tests:** code‑gen loops may add `tests/features/drafts/*.feature` + new steps, but may not edit existing tests to pass. Tests are the termination predicate; a loop that can edit them has no predicate.
5. **Scope guard:** `shared/scripts/scope-guard.sh` / `scope-record.sh` keep a tick from wandering outside the declared change scope.
6. **Memory discipline:** check memory at session start (surreal‑memory → Cortex → file), write memories after every feature/bug fix (global vs project scope). The loop's cross‑session continuity depends on this, not on context.
7. **Escalation points are first‑class:** declare them in `loop.json`. The loop should ping you *only* at these — that is the whole value proposition.

---

## 8. "Primarily loops" operating model

A practical daily setup once loops are the default unit of work.

| Job | Layer | Harness | Cadence | Why |
|---|---|---|---|---|
| Hard architecture/spec reasoning | L1 spec‑first | Claude Code (Opus) | in‑session `/loop` | best long‑horizon reasoning; you watch the hard part |
| High‑volume code‑gen execution | L1 execute | OpenCode (self‑host GLM‑5.2) | headless cron | cheapest tokens, sovereign, 1M context |
| Repo hardening / health | L2 evolver `software` | OpenCode or Claude Code | background, 20‑min ticks | continuous, low‑attention |
| New‑project ideation | L2 evolver `research` | Claude Code (Opus) | manual ticks | wants you in the escalation loop |
| Overnight unattended push | L3 outer | `claude -p` / `codex exec` via launchd | cron | bounded, logged, resumable |

**Discipline that makes it sustainable:** one evolution name per goal; one OpenSpec change per shippable unit; accurate `progress.json` totals; escalations only at declared points; reflections that pass the gate. Run `/loop-report <name>` each morning — the dense tick table on top tells you position in one glance; the per‑tick narrative below tells you why.

---

## 9. Quickstart cheat sheet

```bash
# Define a standing loop once
/loop-define <name>            # writes .kbd-orchestrator/loops/<name>/loop.json

# Run / inspect
/loop-tick <name>              # one evolver→KBD cycle
/loop-report <name>            # tick table + narrative

# Spec-first codegen
/opsx-new <change> → /opsx-continue* → /opsx-verify → /kbd-execute <change> → /opsx-apply → /opsx-archive

# Evolve a codebase
/evolve "<name>" domain=software phase=full

# Ideation spec
/evolve "<name>" domain=research        # + ideation-mindmap + zeespec-interrogator + pmpo-elicit

# Headless tick (any harness)
claude  -p "/loop-tick <name>" --max-turns 40 --max-budget-usd 2 --permission-mode dontAsk
opencode run -p "/kbd-execute <name>"
codex   exec "advance kbd waypoint one phase for <name>"
```

**Golden rule:** keep `.kbd-orchestrator/` + `openspec/` as the single source of truth; swap the harness, never the state.
