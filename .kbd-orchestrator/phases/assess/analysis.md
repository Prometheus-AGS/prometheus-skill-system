# Analysis — Self-Learning Loop Integration

**Phase:** assess → analyze
**Date:** 2026-06-23
**Scope:** All parameters for looping, handoffs, child KBD skills, self-learning across sessions,
and — critically — why per-turn progress signaling has never reliably worked and what closes that gap.

---

## 1. The Complete Loop Parameter Space

Every loop in this system has exactly six parameters. No loop should be constructed without
explicitly specifying all six.

### 1.1 Six canonical loop parameters

```
GOAL          — What "done" looks like. Must be machine-checkable.
               Examples: "cargo test green", "open questions = 0", "spec valid + tests pass"

FEEDBACK      — The data sources consulted at the START of each tick to evaluate progress.
               Types: command (shell), gh-query (PR/issue), file (content check), url (HTTP status)

TERMINATION   — Three guards (all three required):
               max_ticks:             hard ceiling on iterations (never omit)
               max_no_progress_ticks: stall detection — escalate after N ticks with no delta
               budget:                per-tick wall-time cap (e.g., "30m", "2 USD")

ESCALATION    — Which decision points surface to the human (via /pmpo-elicit).
               Never: "always run to completion"
               Always: "pause at every tick" (defeats the loop)
               Declared: specific conditions — regression, contested decision, stall

CADENCE       — How the tick is re-armed:
               manual:     user runs /loop-tick
               background: claude -p /loop-tick (background task)
               cron:       scheduled agent (launchd / cloud routine / codex exec in CI)

EVOLUTION     — The named evolution this loop is running against (links L3 → L2 → L1).
               evolution_name binds to the iterative-evolver's named state.
```

### 1.2 Loop layer taxonomy (L0–L3) — complete parameter table

| Layer | Name | Entry point | State file | Termination | Cadence primitive |
|-------|------|-------------|------------|-------------|-------------------|
| L0 | Model tool loop | model decision | context window | plain text response (no tool calls) | none — harness-owned |
| L1 | KBD tactical | `/kbd-plan` → `/kbd-execute` → `/kbd-reflect` | `.kbd-orchestrator/phases/<name>/progress.json` | changes_completed == changes_total | not applicable (one phase = one unit of work) |
| L2 | Iterative evolver | `/evolve "<name>"` | `.evolver/evolutions/<name>/state.json` | iteration_count >= max_iterations (default 5) OR goal satisfied | usually: user-driven; can be one L3 tick |
| L3 | Outer standing loop | `/loop-define` + `/loop-tick` | `.kbd-orchestrator/loops/<name>/loop.json` | max_ticks / max_no_progress_ticks / budget | manual / background / cron |

**Key rule:** Each layer's termination predicate must be evaluated by the LAYER ABOVE it, not
by the layer doing the work. This is the Claude Code `/goal` model: a separate verifier checks
completion, not the same model instance that did the work.

---

## 2. Handoffs — Complete Protocol

A handoff is the boundary artifact between two phases or two layers. It has three jobs:
(1) proves the prior stage is done, (2) passes context that the next stage must read,
(3) serves as the stage gate — the next stage may not begin until its incoming handoff exists.

### 2.1 Intra-phase handoffs (KBD stage gates)

These live at `.kbd-orchestrator/phases/<phase>/handoffs/`:

```
assess-to-analyze.md    — written by /kbd-assess, read by /kbd-analyze
analyze-to-plan.md      — written by /kbd-analyze (or skip handoff), read by /kbd-plan  
plan-to-execute.md      — written by /kbd-plan, read by /kbd-execute
execute-to-reflect.md   — written by /kbd-execute (final change complete), read by /kbd-reflect
reflect-to-next.md      — written by /kbd-reflect, read by the NEXT PHASE's /kbd-assess
```

**Schema fields (required in every handoff):**
```json
{
  "from": "<stage>",
  "to": "<stage>",
  "created": "<ISO-8601>",
  "summary": "<1-3 sentences: key findings, decisions, open questions>",
  "artifact": "<primary artifact filename>",
  "skip_reason": null
}
```

A skip (e.g., `/kbd-analyze --skip "no external deps"`) writes a handoff with `"skip_reason"` set
so the next stage gate passes deliberately rather than by drift. Never skip silently.

### 2.2 Cross-phase handoffs (phase-to-phase)

The reflect-to-next handoff (`reflection.md`) is the seed for the next phase's `/kbd-assess`.
The new-phase invocation reads:
1. `reflection.md` — goals not met, carry-forwards, recommended focus
2. `progress.json` — completed vs total changes
3. `evolver-bridge.json` — if inside an evolution cycle, the evolver item map

### 2.3 Cross-layer handoffs (evolver ↔ KBD)

`evolver-bridge.json` is the bidirectional contract:

**Evolver → KBD (written by evolve-execute):**
```json
{
  "evolution_name": "<name>",
  "evolver_plan_path": ".evolver/evolutions/<name>/plan.json",
  "item_to_change_map": {}
}
```

**KBD → Evolver (populated by kbd-plan and kbd-reflect):**
```json
{
  "item_to_change_map": {
    "evolver-item-1": ["change-001", "change-002"],
    "evolver-item-2": ["change-003"]
  },
  "execution_results": {
    "evolver-item-1": { "status": "DONE", "artifact_quality": 0.85 },
    "evolver-item-2": { "status": "DONE", "artifact_quality": 0.92 }
  }
}
```

**Current gap:** `evolver-bridge.json` is specified but not written by any existing script.
It must be implemented in `evolve-execute/SKILL.md` and `kbd-reflect/SKILL.md`.

### 2.4 Cross-harness handoffs (tool-to-tool)

When work moves between Claude Code and OpenCode or Codex, the handoff is the disk state itself:
`.kbd-orchestrator/current-waypoint.json` is the universal re-entry point. Any harness reads it,
reconstructs position, and continues. No conversational context is carried — disk state IS the
handoff. This is the fundamental architectural guarantee.

---

## 3. Child KBD Skills — Complete Mechanics

### 3.1 The child skill verbs

| Verb | When to use | Effect on path[] |
|------|-------------|-----------------|
| `/kbd-new-child <name>` | Decompose a phase into sub-phases | Appends to path[], creates `phases/<parent>/children/<name>/` |
| `/kbd-next-child <name>` | Select which child to enter next | Sets childPointer in waypoint |
| `/kbd-child-exit --enter` | Descend into selected child | Clears childPointer, path[] tail = child |
| `/kbd-child-exit` | Complete child, return to parent | Writes `handoff-out.md`, rolls up progress, pops path[] |

**Selected vs. entered invariant (critical):**
- `path[]` trailing token == `childPointer` → child is SELECTED (parent is still active node)
- `childPointer` cleared → child is ENTERED (child IS the active node)
- `/kbd-new-child` when ENTERED → nests under current node
- `/kbd-new-child` when SELECTED → adds a SIBLING (strips the pointer token first)

### 3.2 Nesting depth

`maxChildDepth` in `project.json` (default 4). Rendered in position breadcrumb as:
```
parent-phase › child-loop › grandchild-task
```

### 3.3 Progress rollup

When `/kbd-child-exit` fires, `shared/lib/rollup.sh` recomputes `children{}` aggregate in
each ancestor's `progress.json`. The parent's `changes_completed` reflects its children's
work. This is what makes the outer progress display accurate across nested work.

### 3.4 When to use child phases vs. changes

- **Changes** (OpenSpec): distinct shippable artifacts, independent review, sequential or parallel
- **Child phases**: when the work within a phase is itself a multi-phase process (e.g., a phase
  that requires its own assess→plan→execute→reflect sub-cycle)
- Rule of thumb: if you'd write a sub-plan.md, use a child phase. If you'd write a change ticket,
  use an OpenSpec change.

---

## 4. Self-Learning Across Sessions — Complete Mechanism

### 4.1 The four channels that make cross-session continuity work

```
CHANNEL 1: Disk state (.kbd-orchestrator/)
  → current-waypoint.json + progress.json are never in-context only
  → any session, any harness, can re-enter from them
  → no context needed — disk IS memory for loop position

CHANNEL 2: Knowledge base (pk / prometheus-knowledge)
  → UserPromptSubmit hook: pk-focus-on-prompt.sh injects relevant KB context
  → Stop hook: forge-reflect-on-stop.sh → pk ingest pushes lessons into KB
  → Cross-session: the KB grows; future sessions get richer focus context

CHANNEL 3: surreal-memory (semantic, cross-tool)
  → add_memory / hybrid_search_memories / semantic_search
  → available to ALL harnesses via MCP at :23001
  → memory-outbox.jsonl → memory-outbox-flush.sh: reliable eventual write
  → entities + relations: architectural knowledge (not just text)

CHANNEL 4: Skill files (the procedural memory layer)
  → ~/.claude/skills/learned/ (continuous-learning-v2 instincts)
  → prometheus-skill-pack/skills/ (the canonical skill pack)
  → skill auto-update: when a better approach is found, the skill is updated
  → across ALL future sessions that load that skill, the improvement is available
```

### 4.2 The session lifecycle (current state — what actually fires)

```
SessionStart
  [hooks.json fires]
  → detect-project-context.sh   (identity discovery, ~15s timeout)
  → memory-outbox-flush.sh      (drain pending surreal-memory writes)
  → pk-health.sh                (health check, ~8s timeout)

Each UserPromptSubmit
  [hooks.json fires]
  → pk-focus-on-prompt.sh       (keyword→pk focus→inject KB context, 3s timeout)
  → position-on-prompt.sh       (inject position block if waypoint found, 5s timeout)

Each Write/Edit/MultiEdit (PostToolUse)
  [hooks.json fires]
  → validate-state.sh
  → validate-gitops-write.sh
  → scope-record.sh
  → sycophancy-check-artifact.sh (35s timeout)
  → memory-writeback.sh

SubagentStop[reflector]
  → sycophancy-check-reflection.sh (gate — rejects hollow reflections)
  → log-reflection.sh
  → state-checkpoint.sh
  → workflow-dispatch.sh

Stop
  → position-stop-gate.sh       (blocks once if position footer missing)
  → state-finalize.sh
  → workflow-dispatch.sh (cycle_complete)
  → forge-reflect-on-stop.sh    (ONLY if forge installed — else silent no-op)
```

### 4.3 Self-learning: current gap map

| Learning action | Hermes does it | We do it | Gap |
|----------------|----------------|----------|-----|
| Inject prior context at session start | ✅ MEMORY.md always-on | ✅ pk-focus + surreal-memory | Focus is LEXICAL not semantic |
| Extract reusable skill from task completion | ✅ Automatic | ❌ Not wired | continuous-learning-v2 exists; not in hooks |
| Update skill in-place when better approach found | ✅ Automatic | ❌ Not implemented | pmpo-skill-creator is manual-only |
| Periodic consolidation nudge | ✅ Scheduled | ❌ Not implemented | No trigger for between-session consolidation |
| Session lessons → KB | ✅ Native | ⚠️ Forge-dependent | forge-reflect-on-stop.sh no-ops without forge |
| Cross-session strategic state | ✅ session archive | ✅ surreal-memory + .evolver/ | Functionally equivalent |

---

## 5. Why Progress Signaling Has Never Reliably Worked — Root Cause Analysis

**This is the most important section.** The user has asked for "always show step N of M at start
and end of every turn" multiple times. It has never stuck. Here is exactly why.

### 5.1 The three-layer architecture (and the one that doesn't reach the user)

```
Layer 1 — plain-text model output    ← THE ONLY LAYER THE USER SEES
  "Starting task 3 of 10: wire MCP servers"
  "Completed task 3 of 10: wire MCP servers"
  This is guaranteed only when the MODEL emits it.

Layer 2 — hooks stderr              ← INVISIBLE TO THE USER
  The report-progress hook in hooks-config.json fires and writes to stderr.
  Claude Code does NOT surface hook stderr into the conversation.
  This is telemetry / logs, not user-visible.

Layer 2b — UserPromptSubmit stdout injection  ← VISIBLE BUT UNRELIABLE
  position-on-prompt.sh prints a <!-- prometheus-position --> block to stdout.
  The hook output IS injected into the model's context (it is in the prompt).
  BUT: the MODEL must then reproduce it in its response for the user to see it.
  The model does not always do this.
```

### 5.2 The specific failure modes

**Failure Mode A: The model does not read the injected position block.**
`position-on-prompt.sh` injects the position block at the start of the prompt as a
`<!-- prometheus-position -->` HTML comment block. Models deprioritize HTML comments.
The instruction `MANDATORY: begin your response with the Position line above` is appended,
but it competes with the system prompt, CLAUDE.md, the skill content, and the user's
actual request. When the model has a rich task to execute, it often skips the position
header and goes straight to the task.

**Failure Mode B: The position block is injected but the model reformats it.**
Even when the model reads the block, it paraphrases rather than reproducing the
`Position: phase › child | status: X` format, making it hard to parse consistently.

**Failure Mode C: The stop gate blocks once, then the next turn lacks it again.**
`position-stop-gate.sh` blocks the Stop event ONCE per session/transcript fingerprint
when the footer is missing. After one block, it records a cap key and never blocks again.
This means the gate catches the first omission but not subsequent ones.

**Failure Mode D: The waypoint-render.sh library is not found.**
`position-on-prompt.sh` sources `$(dirname "$0")/lib/waypoint-render.sh`. The script lives at
`shared/scripts/position-on-prompt.sh` and the library at `shared/scripts/lib/waypoint-render.sh`.
The `$CLAUDE_PLUGIN_ROOT` environment variable must be set for the hook to resolve the
path correctly. If the plugin root is wrong or unset, the render lib is not found and
the hook prints nothing — silently.

**Failure Mode E: No `.kbd-orchestrator` in scope.**
`waypoint_render()` walks up from `$PWD` looking for `.kbd-orchestrator/`. If Claude Code's
working directory is not the project root, or if the phase is not in an orchestrated project,
`_wr_find_root()` returns 1 and nothing is printed. The user sees no position signal, and
because nothing was injected, the model doesn't know it should have emitted one.

**Failure Mode F: The skill instructions say "emit progress signals" but don't specify the format.**
Skills say `emit plain-text progress signals` in a general section, but the instruction is
not given in every individual skill invocation's system context. The model must remember to
apply a general rule to every specific task. This is not how model instruction-following works
reliably — rules embedded at invocation time outperform general rules read at skill load time.

### 5.3 The correct architecture — what actually works

The per-turn-position-hook.md reference already documents this correctly, but it needs to be
the source of truth that all other references point to:

**The guarantee (Layer 1):** Every skill must emit progress signals in its OWN RESPONSE TEXT.
Not via hooks. Not via injected context. The model writing the response must emit:
```
Starting <thing> <N> of <M>: <canonical-name>
```
at the top of its response and:
```
Completed <thing> <N> of <M>: <canonical-name>
```
at the bottom, BEFORE making any tool calls for that unit of work.

The totals (`M`) must be read from `progress.json` or `plan.md` — never estimated.

**The hook layer (Layer 2b):** The `position-on-prompt.sh` injection is a BACKUP — it gives
the model the current position in case it doesn't know it, but it cannot FORCE the model to
emit it. It is belt-and-suspenders, not the guarantee.

**The stop gate (position-stop-gate.sh):** Provides one enforcement opportunity per session
when the footer is absent. The soft cap is correct (preventing infinite block loops), but the
gate only triggers at stop, not at the start of each turn.

### 5.4 The three changes needed to make it actually work

**Fix 1 — Explicit format enforcement in every skill's Progress Signals section.**
Every `/kbd-*` skill must include this verbatim in its "Progress Signals" section:

```
## Progress Signals — EMIT IN YOUR RESPONSE, NOT VIA HOOKS

At the VERY START of your response (before any tool call), emit:
  Starting <kind> <N> of <M>: <canonical-name>

Where N and M come from:
  - For phases: read .kbd-orchestrator/phases/<phase>/progress.json → changes_completed / changes_total
  - For tasks within a change: read the plan.md task index
  - NEVER estimate or guess totals — read them from the file

At the END of your response (after all tool calls complete), emit:
  Completed <kind> <N> of <M>: <canonical-name>

If you are partway through (turn interrupted or multi-turn): emit both at turn boundaries.
```

**Fix 2 — Add a UserPromptSubmit hook that WRITES the position to a file the model reads.**
Instead of injecting into the prompt text (which the model deprioritizes), write the position
to `.kbd-orchestrator/position-reminder.txt` and have the model's first tool call read it:

```bash
# position-on-prompt.sh (enhanced)
waypoint_render > "$ROOT/.kbd-orchestrator/position-reminder.txt"
echo "KBD_POSITION_REMINDER_PATH=$ROOT/.kbd-orchestrator/position-reminder.txt"
```

Then every skill's first instruction is: `Read .kbd-orchestrator/position-reminder.txt and begin
your response with its content, updated for your current task.` This makes the position a
data source the model actively reads rather than an HTML comment it may ignore.

**Fix 3 — PreToolUse hook on Write/Edit that checks for starting signal in the CURRENT turn.**
If the model writes code but hasn't emitted a Starting signal in this turn, the PreToolUse hook
can inject a reminder into the tool input context. This catches the most common failure path
(the model dives into edits without signaling first).

---

## 6. Build-vs-Adopt Decisions for Each Gap

### 6.1 L3 outer loop instantiation

**Decision: BUILD** — `/loop-define`, `/loop-tick`, `/loop-report` as skill files (markdown slash
commands). The `pmpo-outer-loop` SKILL.md already has the complete spec; what is missing is the
backing skill files that harnesses resolve to when the user types the command.

**What to build:**
- `skills/process/pmpo-outer-loop/skills/loop-define/SKILL.md`
- `skills/process/pmpo-outer-loop/skills/loop-tick/SKILL.md`
- `skills/process/pmpo-outer-loop/skills/loop-report/SKILL.md`
- `references/schemas/loop-definition.schema.json`

### 6.2 Auto skill extraction

**Decision: ADOPT** — `continuous-learning-v2` has the complete instinct extraction engine.
The gap is wiring: add `continuous-learning-v2/evaluate-session.sh` to the
`SubagentStop[executor]` hook in `hooks.json`. No new code needed — wire existing code.

### 6.3 Periodic nudge

**Decision: BUILD (small)** — A `scripts/scheduled/periodic-nudge.sh` that:
1. Reads instinct count from `~/.claude/homunculus/`
2. If count > N (threshold), invokes `continuous-learning-v2` consolidation
3. Can be wired to `~/.claude/settings.json` Stop hook as a cross-session trigger

### 6.4 Skill auto-improvement

**Decision: BUILD (medium)** — `pmpo-skill-creator` exists for new skill creation.
Extend it with an `update` mode: when a reflection contains a lesson about a skill that was
used in the phase, `pmpo-skill-creator update <skill-name> "<lesson>"` merges the lesson
into the skill's SKILL.md. The SubagentStop[reflector] hook calls this after logging.

### 6.5 Semantic pk-focus

**Decision: BUILD (small)** — Add a secondary search path to `pk-focus-on-prompt.sh`:
```bash
# After keyword-based pk focus:
MEMORIES="$(curl -s -X POST http://localhost:23001/api/v1/memory/search \
  -H 'Content-Type: application/json' \
  -d "{\"query\":\"$PROMPT_TEXT\",\"user_id\":\"prometheus-skill-pack\",\"limit\":3}" \
  2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); \
  print('\n'.join(m.get('memory','') for m in d.get('memories',[])))" 2>/dev/null || true)"
[ -n "$MEMORIES" ] && printf '\nRelevant memory context:\n%s\n' "$MEMORIES"
```

### 6.6 forge-independence for pk ingest

**Decision: BUILD (small)** — Replace `forge-reflect-on-stop.sh` logic with direct `pk ingest`:
```bash
# In forge-reflect-on-stop.sh, after forge no-op path:
if command -v pk &>/dev/null; then
  # Write a session summary to a temp file for pk ingest
  SESSION_SUMMARY_FILE="$(mktemp /tmp/kbd-session-XXXXXX.md)"
  # ... generate summary from position.json + last assistant message ...
  pk ingest "$SESSION_SUMMARY_FILE" 2>&1 || true
  rm -f "$SESSION_SUMMARY_FILE"
fi
```

### 6.7 evolver-bridge.json

**Decision: BUILD (small)** — Add to `evolve-execute/SKILL.md` step list: write
`.kbd-orchestrator/phases/<phase>/evolver-bridge.json` before delegating to KBD.
Add to `kbd-reflect/SKILL.md`: update `execution_results` in the bridge file after
all changes are complete.

---

## 7. Open Questions for Spec

1. **Nudge threshold:** How many instincts before triggering consolidation? 10? 25? Or
   time-based (after 7 days without nudge)?

2. **Skill update approval gate:** Should skill auto-improvement be auto-applied or
   require user approval? Hermes applies automatically. The sycophancy gate provides some
   protection. Recommendation: auto-apply with a `diff` shown to the user; human can revert.

3. **Loop.json location:** Should outer loop definitions live in `.kbd-orchestrator/loops/`
   (project-scoped, committed) or `~/.prometheus/loops/` (user-scoped, cross-project)?
   Recommendation: project-scoped by default; support `~/.prometheus/loops/` for global loops.

4. **Position signal enforcement:** The Fix 2 approach (write to a file) is elegant but
   adds a file-read step to every skill invocation. Is the overhead acceptable? Alternative:
   accept that Layer 1 (model instruction) is the only reliable mechanism and focus on making
   the skill instructions iron-clad rather than adding a hook mechanism.

5. **Cross-harness self-learning:** For Kimi/MMX (no hook support), the skill must self-invoke
   focus and reflect at session boundaries using in-prompt instructions. How do we detect which
   harness is active inside a skill file?

---

## 8. Candidate Adoption Table

| Component | Adopt (existing) | Build | Notes |
|-----------|-----------------|-------|-------|
| L3 loop commands | — | New skill files (small) | pmpo-outer-loop SKILL.md is the spec |
| Auto skill extraction | continuous-learning-v2 | Wire into hooks only | Evaluate-session.sh already written |
| Periodic nudge | — | New shell script (small) | Wire to Stop hook |
| Skill auto-improvement | pmpo-skill-creator | Extend with update mode | New --update flag |
| Semantic focus | surreal-memory REST | Small enhancement to existing hook | ~15 lines bash |
| forge-independent reflect | pk (already installed) | Small enhancement to existing hook | ~10 lines bash |
| evolver-bridge.json | — | Add to existing skills (small) | Already specced, not coded |
| Progress signal enforcement | karpathy-guidelines | Standardize format in skill files | Template standardization |

**Total scope: ~5 small builds + 3 hook wires + 1 skill template standardization pass.**
This is a Phase, not a multi-phase program.

---

## 9. Decision Log

| Decision | Verdict | Rationale |
|----------|---------|-----------|
| Layer 1 (model output) vs hooks for progress signals | Layer 1 is the guarantee | Hooks stderr not surfaced to user; injected context deprioritized by model |
| Auto-approve skill updates vs gate | Auto-apply with diff | Hermes approach; sycophancy gate provides check; user can revert |
| Loop definitions location | `.kbd-orchestrator/loops/` (project-scoped) | Shared with all harnesses; committed to repo |
| forge dependency | Remove for pk ingest path | forge not in install script; silent no-op is unacceptable for core learning |
| evolver-bridge.json | Build in both evolve-execute and kbd-reflect | Bridge must be bidirectional; both sides must write it |

---

*Analysis complete. 5 candidates to adopt/extend, 3 to build from scratch, 1 template pass.
Next stage: Spec — define the concrete implementation contracts for each.*
