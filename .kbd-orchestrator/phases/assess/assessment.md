# Assessment — Cross-Platform Self-Learning Loop Architecture

**Phase:** assess (ad-hoc / research)
**Date:** 2026-06-23
**Scope:** Gap analysis between current prometheus-skill-pack loop system and the ideal self-learning,
cross-harness, Karpathy-informed loop architecture with Hermes-parity features.
**Triggered by:** `/kbd-assess` with research arguments on Claude Code loop methodology,
Hermes harness, Karpathy learning loops, and cross-platform applicability.

---

## Executive Summary

The prometheus-skill-pack has **a sophisticated, well-specified loop architecture** that surpasses
most public implementations in structural rigor. The L0–L3 four-layer model, shared state substrate,
OpenSpec artifact gating, sycophancy gate, and per-harness driver tables are world-class.

However, **the specification is ahead of the implementation.** The loops-architecture-spec.md
document describes a system that is partially built. The `pk-*` Karpathy utilities are in partial
use but not fully wired into the self-learning feedback cycle across sessions. The critical Hermes-
equivalent features — autonomous skill creation, periodic nudge, instinct accumulation, and cross-
session skill improvement — exist as separate skills (`continuous-learning-v2`, `karpathy-guidelines`,
`autonomous-loops`) that are **not yet orchestratively connected** to the main loop lifecycle.

The gap to close is not conceptual — it is *integration*: connecting what exists into a
single, always-on self-improving loop harness that works identically across Claude Code,
Codex, OpenCode, Kimi, MiniMax, and Zed.

---

## Part 1: What Claude Code Loops Are (Research Findings)

### 1.1 Native primitives (mid-2026 state)

Claude Code's loop system as of June 2026:

- **`/loop <interval> <prompt>`** — schedules a recurring prompt in the session; minimum 1 min,
  maximum 3 days; auto-expires; maintains session context between runs. Added ~late 2025.
- **`/goal <condition>`** — sets a termination predicate; a *separate verifier model instance*
  checks the condition after each turn (not the same model that did the work — clean separation).
  Added version 2.1.139, week of May 11, 2026.
- **`claude -p "<prompt>"` (headless)** — single-shot non-interactive run; `--max-turns`,
  `--max-budget-usd`, `--allowedTools`, `--permission-mode dontAsk`, `--output-format stream-json`;
  the body of external cron/launchd loops.
- **Desktop Tasks / Cloud Routines** — scheduled unattended execution without an open session.
- **Hooks** — `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `Stop`;
  the Karpathy focus/reflect/ingest cycle lives here.
- **Git worktrees** — `isolation: worktree` on subagents; parallel agents on separate branches.

The **master agent loop** internally (codenamed "nO") is a while-loop that continues while the
model returns tool calls; plain text exits the loop. Single-threaded, single flat message history —
deliberately simple for debuggability.

**Key insight from research (Addy Osmani, June 7, 2026):** The six building blocks of loop
engineering are: Automation (trigger), Goal (termination predicate), Tools (action surface),
Memory (cross-iteration context), Verification (quality gate), and Parallelism (worktrees/agents).
The prometheus-skill-pack spec addresses all six.

### 1.2 The "Ralph Loop" pattern (community reference)

The Ralph Loop (Jeffrey Huntley) is the community's dominant pattern:
```
while not COMPLETE:
  claude -p "<step prompt>"
  check prd.json for passes=false tasks
  update progress.txt
  commit
```
This is L0/L1 only — no landscape research, no reflection gate, no cross-session learning.
Our four-layer architecture (L0→L3) is architecturally far superior. The gap is that Ralph
is *deployed and usable*; our L3 is *specified but not fully instantiated*.

---

## Part 2: Hermes Architecture Analysis

### 2.1 What Hermes is

Hermes (NousResearch/hermes-agent, MIT, active as of June 22, 2026) is an always-running
agent runtime with a first-class self-improving learning loop:

```
Do work
  ↓
Notice reusable procedure / user preference / lesson
  ↓
Save as memory (SQLite FTS + LLM summarization), skill file, or session history
  ↓
Load that context in future work via periodic nudge
  ↓
Improve next run
```

**Four-layer memory system:**
1. **MEMORY.md** — always-on context injected into every session (permanent, curated)
2. **USER.md** — user model: preferences, brand voice, project paths, workflow style
3. **Session archive** — searchable history; retrieved on-demand by topic
4. **Skills** — procedural memory; 40+ bundled; auto-created from successful task completions;
   auto-refined when better approaches are discovered; stored at `~/.hermes/skills/`

**Periodic nudge:** the agent periodically prompts itself (scheduled internal event) to decide what
from recent sessions is worth keeping in MEMORY.md vs session archive — the agent curates its own
memory rather than logging everything or nothing.

**Skill lifecycle in Hermes:**
- Agent completes a task → automatically extracts a reusable skill file
- On future similar tasks → loads the relevant skill → executes faster / better
- On discovering a better approach → updates the existing skill in-place
- Skills Hub for community sharing

**Meta-Harness (howdymary/hermes-agent-metaharness):** an outer-loop optimizer that searches over
harness code itself using execution traces, scores, and diagnostic context — improving *how Hermes
is run* rather than model weights.

### 2.2 Hermes vs. our system: feature comparison

| Capability | Hermes | Prometheus Skill Pack | Gap |
|-----------|--------|----------------------|-----|
| Self-creating skills | ✅ Auto after task completion | `continuous-learning-v2` exists; `pmpo-skill-creator` exists | Not auto-triggered from loop lifecycle |
| Periodic nudge (memory curation) | ✅ Built-in scheduled | `pk-focus-on-prompt.sh` (focus-in only) | No periodic outbound curation nudge |
| 4-layer memory | ✅ Native | `surreal-memory` + file memory + Cortex (3 providers) | Comparable but not unified under one API |
| Session archive + semantic search | ✅ SQLite FTS | `surreal-memory` semantic_search + hybrid_search | Functionally equivalent |
| Skills hub / sharing | ✅ | `agentskills.io` + marketplace + `npm run install:platforms` | Better than Hermes — cross-platform |
| Skill auto-improvement | ✅ In-place update | No equivalent — skills are edited manually or via `pmpo-skill-creator` | **Primary gap** |
| Cross-session loop continuity | ✅ Always-on runtime | `.kbd-orchestrator/` durable state + `surreal-memory` | Architecturally stronger |
| Cross-harness portability | ❌ Hermes-only | ✅ Claude Code, Codex, OpenCode, Kimi, MMX | We win here |
| Sycophancy gate | ❌ | ✅ `sycophancy-check-reflection.sh` + MCP server | We win here |
| Termination predicate (separate verifier) | ❌ | `/goal` delegation + measurable_criteria check | We win here |
| Meta-harness (outer-loop optimizer) | Research-only | `iterative-evolver` + `pmpo-outer-loop` (L2/L3) | Architecturally comparable |
| Loop-over-ideation→spec→dev→deploy | Partial (task types) | Full topology specified in spec | Not deployed |
| Karpathy feedback (focus-in, reflect-out) | ❌ | `pk-focus-on-prompt.sh`, `forge-reflect-on-stop.sh`, `pk ingest` | Partially wired — see gaps below |

---

## Part 3: Current State of the Karpathy (`pk-*`) Integration

### 3.1 What exists

The `prometheus-knowledge` (`pk`) CLI is installed at `/usr/local/bin/pk` with state at
`~/.prometheus/knowledge/`. The hooks that wire it into Claude Code:

| Hook | Script | What it does |
|------|--------|-------------|
| `UserPromptSubmit` | `pk-focus-on-prompt.sh` | Extracts top-5 keywords from prompt → calls `pk focus` → injects relevant KB context. Timeout 3s. |
| `Stop` | `forge-reflect-on-stop.sh` | Calls `forge reflect` (if `.forge/iterations/` exists) → `pk ingest` to push session lessons into KB |
| `SubagentStop[reflector]` | `sycophancy-check-reflection.sh` | Gates reflection quality (Delta→RootCause→Corrective) |
| `SessionStart` | `pk-health.sh` | Health check on pk service |

### 3.2 What is missing from the Karpathy loop

**The Karpathy philosophy from research:** "Software 2.0" — the agent should accumulate a
*parameterized knowledge substrate* (in our case, the KB + skills) that improves with use,
so that the system self-improves not by retraining weights but by enriching the context it
loads on the next run.

Current gaps in our implementation:

**Gap K-1: No automatic skill extraction from loop completions.**
When a kbd-execute phase completes successfully, no skill is automatically proposed from the
pattern of what was done. Hermes does this after every successful task. Our `continuous-learning-v2`
and `pmpo-skill-creator` exist but are not triggered by the evolver/KBD lifecycle hooks.
The `PostToolUse` hook calls `memory-writeback.sh` but not skill extraction.

**Gap K-2: No periodic nudge.**
The `forge-reflect-on-stop.sh` hook only fires on `Stop` (session end). There is no
scheduled mid-session or cross-session nudge that says "consolidate what you've learned
across the last N loop ticks into a skill or memory update." In Hermes this is autonomous.

**Gap K-3: pk-focus context is keyword-based, not semantic.**
`pk-focus-on-prompt.sh` extracts the top 5 longest words and calls `pk focus`. This is
lexical, not semantic. When the prompt is about "loop architecture for cross-platform
deployment," the keywords extracted may be "architecture", "cross-platform", "deployment"
but miss the relationship between them. The `surreal-memory` `semantic_search` tool is
available but not called by the focus hook.

**Gap K-4: No skill improvement / refinement loop.**
When the same skill is invoked across multiple loop ticks and lessons are learned about
how to do it better, there is no mechanism to update the skill file. Skills are static
artifacts edited only by humans or `pmpo-skill-creator` on explicit request.

**Gap K-5: forge is not universally installed.**
`forge-reflect-on-stop.sh` gracefully no-ops if `forge` is absent. Since forge is not
part of `scripts/install-skills-flat.sh`, this reflection path is silently skipped on
most installs. The `pk ingest` path after reflection only runs when forge is present.

**Gap K-6: Cross-harness Karpathy layer is specified but not implemented.**
The spec (§6.2, §6.3) says OpenCode should implement focus/reflect in `tool.execute.before/after`
stubs and Codex should call `pk focus`/`pk ingest` as MCP tools. Neither is implemented —
the `.opencode/plugin.ts` stubs do not yet call these; the Codex MCP config does not include
a pk-compatible tool.

---

## Part 4: Current Loop Architecture — What Works vs. What Doesn't

### 4.1 What works well today

**✅ L1 KBD loop (tactical, per-phase):** `kbd-process-orchestrator` is solid. The
assess→plan→execute→reflect cycle is implemented, state is durable in
`.kbd-orchestrator/phases/`, the sycophancy gate on reflect is working (confirmed in
`project_sycophancy_gate_tuning.md` memory), and OpenSpec artifact gating is the default
execute backend.

**✅ L2 iterative-evolver:** `iterative-evolver` is implemented with domain adapters,
provider abstraction (surreal-memory priority), SubagentStop lifecycle hooks, and
checkpoint/finalize scripts. The skill is installed and functional.

**✅ Hooks infrastructure:** The `hooks.json` is comprehensive — SessionStart, UserPromptSubmit,
PreToolUse, PostToolUse, SubagentStop (per-agent-name), Stop. The sycophancy checks for both
artifacts and reflections are armed. The scope guard and protect-tests hooks are working.

**✅ Shared state substrate:** `.kbd-orchestrator/` + `openspec/` are the single source of
truth. Cross-tool work is resumable because state is on-disk, not in-context.

**✅ Memory write transport:** The `memory-write-transport` fix (2026-06-12 memory) resolved
the broken bash→surreal-memory write path. REST POST to `/api/v1/memory` at :23001 now works.
The `memory-outbox.jsonl` → `memory-outbox-flush.sh` pattern provides reliable eventual
write consistency.

**✅ Cross-platform install:** `install-skills-flat.sh` and `npm run install:platforms` deploy
to Claude Code, Kimi, MiniMax, OpenCode, Codex, Cursor. The detect-toolchain.sh script exists.

**✅ kbd-evolve skill:** Newly shipped in the last phase (2026-06-21). This is the landscape-
research trigger for L2 evolution — it surveys the domain and produces ranked evolution briefs.

### 4.2 What is not yet deployed / missing

**❌ L3 outer loop (`/loop-define`, `/loop-tick`, `/loop-report`):** The `pmpo-outer-loop`
skill defines the interface but the *backing commands* — `loop-define` and `loop-tick` as
concrete scripts or slash commands — are not wired. The `.kbd-orchestrator/loops/` directory
does not exist (confirmed: `ls .kbd-orchestrator/loops/` returns "no loops dir"). This is the
highest-priority missing piece.

**❌ `loop.json` schema and scaffolding:** There is no `references/schemas/loop-definition.schema.json`
in the pack. The `/loop-define` command is specified but not implemented.

**❌ Automatic skill extraction from loop completions (Hermes parity gap K-1):** The Stop/
SubagentStop executor hooks call `state-checkpoint.sh` and `workflow-dispatch.sh` but do not
call `continuous-learning-v2/evaluate-session.sh` or `pmpo-skill-creator`. The learning loop
is not closed.

**❌ Periodic nudge (gap K-2):** No scheduled or automatic memory curation trigger exists.
The closes thing is `memory-outbox-flush.sh` on SessionStart, but that only flushes pending
writes — it doesn't curate or consolidate.

**❌ Karpathy semantic focus (gap K-3):** `pk-focus-on-prompt.sh` uses keyword extraction.
A semantic alternative via `surreal-memory` `hybrid_search_memories` is available but not used.

**❌ OpenCode Karpathy layer (gap K-6):** `.opencode/plugin.ts` `before/after` stubs not wired.

**❌ Codex pk-as-MCP (gap K-6):** pk not exposed as an MCP tool in `.codex/config.toml`.

**❌ ideation→spec→dev→deploy spanning loop:** The spec (§5.2) describes a full ideation
loop using `ideation-mindmap` + `zeespec-interrogator` + evolver in research domain. These
skills exist individually but no pre-built loop topology wires them together with the termination
predicate "open questions = 0 AND all measurable_criteria stated."

**❌ `evolver-bridge.json`:** The spec references this as the file that aggregates KBD execute
results back to the evolver's reflect phase. It is not implemented in any of the evolver scripts.

**❌ `/loop-report` rendering:** No implementation of the tick-table + narrative rendering
for `journal.md`.

---

## Part 5: Gap Prioritization — Closing to Hermes Parity + Beyond

Ranked by impact × effort (high impact, lower effort first):

### P0 — Must have for "primarily loops" operating model

| # | Gap | What to build | Effort |
|---|-----|--------------|--------|
| P0-1 | L3 loop not instantiated | Implement `/loop-define`, `/loop-tick`, `/loop-report` as concrete slash commands + `loop.json` schema | Medium |
| P0-2 | Auto skill extraction missing | Wire `continuous-learning-v2/evaluate-session.sh` into `SubagentStop[executor]` hook AND `Stop` hook | Small |
| P0-3 | Periodic nudge absent | Add a `ScheduleWakeup`-compatible nudge trigger: after N ticks, consolidate instincts → propose skill updates | Small |
| P0-4 | Skill auto-improvement | Implement a `skill-update` pass in `pmpo-skill-creator` triggered by the reflection phase when a skill is invoked | Medium |

### P1 — Closes Karpathy integration gaps

| # | Gap | What to build | Effort |
|---|-----|--------------|--------|
| P1-1 | pk-focus is lexical | Add `hybrid_search_memories` call to focus hook (surreal-memory REST) as semantic fallback | Small |
| P1-2 | forge not universal | Replace forge dependency with direct `pk ingest` from session summary; make forge optional | Small |
| P1-3 | Cross-harness Karpathy | Wire pk focus/reflect into `.opencode/plugin.ts` before/after stubs; add pk-compatible MCP tool stub to codex config | Medium |
| P1-4 | evolver-bridge.json | Implement the KBD→evolver result aggregation file in `evolve-execute/SKILL.md` | Small |

### P2 — Full spanning loop (ideation→spec→dev→deploy)

| # | Gap | What to build | Effort |
|---|-----|--------------|--------|
| P2-1 | Ideation loop topology | Wire `ideation-mindmap` + `zeespec-interrogator` + `pmpo-elicit` into a pre-built L2 evolver config with termination predicate | Medium |
| P2-2 | Deploy loop | Extend L3 topology with a post-execute deploy phase (gitops-bootstrap → argocd-multicloud) gated by test green | Medium |
| P2-3 | Cross-tool loop registry | Build `.kbd-orchestrator/loops/<name>/loop.json` as a shared registry that any harness reads | Small |

### P3 — Meta-harness level (self-improving harness, not just self-improving skills)

| # | Gap | What to build | Effort |
|---|-----|--------------|--------|
| P3-1 | Harness self-optimizer | Implement an outer-loop-optimizer (analogous to hermes-agent-metaharness) that scores loop runs and proposes hook/skill modifications | Large |
| P3-2 | Skill confidence scoring | Add confidence scores and invocation counts to skill files (like continuous-learning-v2 instincts) so the pack knows which skills are well-validated | Medium |
| P3-3 | Cross-session skill tournament | When two approaches exist for the same task, run them in parallel worktrees and promote the winner to the canonical skill | Large |

---

## Part 6: The Target Architecture — "Self-Learning Loop System"

The fully realized system has these properties:

```
SESSION START
  → surreal-memory semantic_search (not lexical) — load relevant KB context
  → memory-outbox-flush (drain pending writes)
  → pk-health check

USER PROMPT
  → pk-focus (semantic hybrid_search) → inject context

LOOP TICK (L3 outer loop, /loop-define + /loop-tick)
  → one evolver cycle (L2: assess→analyze→plan→execute→reflect)
      execute delegates to KBD (L1: kbd-plan→kbd-execute→kbd-reflect)
          artifact-refiner QA per change (L0: model tool loop)
  → on execute complete: SubagentStop[executor]
      → continuous-learning-v2/evaluate-session.sh (auto skill extraction)
      → pmpo-skill-creator (if new skill proposed)
      → evolver-bridge.json write (results → evolver reflect)
  → on reflect complete: SubagentStop[reflector]
      → sycophancy-check-reflection.sh (gate)
      → log-reflection.sh
      → pk ingest (lessons → KB)
  → loop-tick appends journal.md entry
  → cadence re-arms (cron / background / manual)

PERIODIC NUDGE (after N ticks or on schedule)
  → review instincts from continuous-learning-v2
  → propose skill improvements to modified skills
  → surreal-memory add_memory for cross-session lessons

SESSION STOP
  → position-stop-gate (must have emitted progress signals)
  → state-finalize (evolution state closed)
  → forge-reflect-on-stop / pk ingest (if forge present; else direct pk ingest)
  → workflow-dispatch cycle_complete

CROSS-HARNESS PARITY
  Claude Code: hooks.json (above)
  OpenCode:    plugin.ts before/after → same pk focus/ingest calls
  Codex:       pk-mcp tool in config.toml → same calls via MCP
  Kimi/MMX:    skills/process/continuous-learning-v2 skill loaded; no hook support
               → skill must self-invoke focus/reflect at session boundaries
```

This is Hermes-parity in capability, with our structural advantages:
- **Cross-platform** (6 harnesses vs Hermes single runtime)
- **Sycophancy gate** (Hermes has no reflection quality guard)
- **OpenSpec artifact gating** (Hermes has no dependency-ordered artifact lifecycle)
- **Four-layer loop (L0→L3)** vs Hermes single-layer agent loop
- **Model routing per phase** vs Hermes single model

---

## Part 7: Recommended Next Phase

Based on the gap analysis, the highest-value next phase is:

**"self-learning-loop-integration"** — closing P0 and P1 gaps:

1. Implement `/loop-define`, `/loop-tick`, `/loop-report` as concrete commands (P0-1)
2. Wire `continuous-learning-v2` into evolver/KBD executor SubagentStop hook (P0-2)
3. Implement periodic nudge via `scheduled/` directory + `ScheduleWakeup`-compatible trigger (P0-3)
4. Add `evolver-bridge.json` write to `evolve-execute` (P1-4)
5. Add semantic hybrid_search to `pk-focus-on-prompt.sh` fallback (P1-1)
6. Replace forge dependency with direct `pk ingest` from session summary (P1-2)

P2 and P3 form the phase after that.

---

## Appendix: Karpathy Principle Mapping

| Karpathy Principle | Current Implementation | Gap |
|-------------------|----------------------|-----|
| "Software 2.0" — weights are data not code | surreal-memory KB as the "weights"; skills as parameterized procedures | Skills not auto-updated; KB not auto-enriched per loop tick |
| Feedback loop over goals, not step-by-step prompts | `/goal` + measurable_criteria + L3 termination predicate | L3 not deployed |
| Minimal, verifiable success criteria | OpenSpec artifact gating + sycophancy gate | ✅ Working |
| Progress reporting (N of M, never guessed) | CLAUDE.md progress signals + karpathy-guidelines skill | ✅ Working |
| Knowledge accumulation across runs | surreal-memory + file memory + pk KB | Not auto-updated from loop completions |
| Self-correcting loops | sycophancy gate + corrective actions | ✅ Working at reflect level; not at skill level |
| "Write the loop; the framework runs it" | pmpo-outer-loop + iterative-evolver | Loop not instantiated (no loop.json, no /loop-tick command) |

---

*Assessment complete. Key finding: the architecture is sound and ahead of the market; the implementation
gap is in L3 loop instantiation, automatic skill extraction, and Karpathy semantic focus. Close P0
gaps first.*
