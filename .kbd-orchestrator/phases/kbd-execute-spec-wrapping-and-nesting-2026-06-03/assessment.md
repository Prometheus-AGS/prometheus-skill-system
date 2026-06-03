# Assessment — kbd-execute-spec-wrapping-and-nesting

- **Project:** prometheus-skill-system
- **Phase:** kbd-execute-spec-wrapping-and-nesting-2026-06-03
- **Date:** 2026-06-03
- **Backend present:** OpenSpec (`openspec_present: true`, root `openspec/`)
- **Sycophancy gate:** viability verdict scored **0.0 / clean** at `strict` strictness (no S-01..S-08 patterns) before being recorded here.

---

## 0. TL;DR (sycophancy-corrected)

The request leads with "build these commands," but **~70% of the requested
machinery already ships on `main`** — the four nesting commands, the hook
engine, the default progress reporter, and the assess→plan→execute→reflect loop
are all present. Building more commands is **not** the fix.

The real defect is narrow and specific: **the Execute phase does not actually
wrap the spec backend.** `kbd-execute` writes a dispatch contract and then the
flow hands the turn to **unmodified upstream `/opsx:apply`**, which has zero KBD
awareness. That single seam produces every symptom the user reported ("funneled
into openspec," "lose connection to execute," "don't know where I am").

A secondary, honesty-critical finding: the "report every single turn no matter
what" guarantee **cannot** be met by the current `report-progress` shell hook —
it writes to stderr (not surfaced to the user) and only fires if the model
voluntarily sources `hooks.sh`. It needs a different mechanism.

---

## 1. What already exists (verified in tree)

| Capability | Status | Evidence |
|---|---|---|
| `/kbd-new-phase` (sibling) | **Ships** | `skills/.../kbd-new-phase/{SKILL.md,kbd-new-phase.sh}` |
| `/kbd-next-phase` | **Ships** | `skills/.../kbd-next-phase/SKILL.md` |
| `/kbd-new-child` (nested loop) | **Ships** | `skills/.../kbd-new-child/{SKILL.md,kbd-new-child.sh}` |
| `/kbd-next-child` | **Ships** | `skills/.../kbd-next-child/{SKILL.md,kbd-next-child.sh}` |
| Nesting state model | **Ships** | waypoint `parentPhase`, `childPhases[]`, `childPointer`; `waypoint.sh::waypoint_chain` renders `parent › phase › child` |
| Hook engine | **Ships** | `shared/lib/hooks.sh` (~12 KB), `hooks/hooks.json` |
| Default progress reporter | **Ships (but see §3.3)** | `report-progress` hook, `event:"*:*"`, mode `augment`, emits `starting/ending <kind> <name> [i/n]` **to stderr** |
| Hook taxonomy | **Ships** | `assess/plan/execute/task/child` × `before/after`; legacy `on_*` aliases |
| Memory mirror + recall hooks | **Ships** | `kbd-memory-log`, `auto-memory-recall` |
| assess/plan/execute/reflect skills | **Ships** | all fire `<phase>:before/after` |

**Implication:** This phase is mostly *repair + wiring*, not greenfield. Treat
new-command requests skeptically — most already resolve to existing skills.

---

## 2. Goal-by-goal gap analysis

### Goal 1 — Execute must wrap the spec backend  ❌ **BROKEN (root cause)**

**Current behavior**
- `kbd-plan` already reaches into OpenSpec: on detection it emits
  `/opsx:new <change-id>` and creates `openspec/changes/<id>/{proposal,tasks}.md`.
  → The plan/execute boundary is **already blurred**: plan creates the change.
- `kbd-execute` (`prompts/execute.md`) selects a backend, writes `execution.md`,
  and for self-executing OpenSpec says only: *"Treat OpenSpec tasks as the
  working execution surface … Sync progress back into progress.json after each
  task."* This is a **soft instruction to the model**, not an enforced loop.
- `kbd-execute/SKILL.md` claims: *"task:before/task:after are fired per OpenSpec
  task by `/opsx:apply`."*

**Verified contradiction**
- `grep` across `.agents/skills/openspec-apply-change/` and
  `.agents/skills/source-command-opsx-apply/` for
  `kbd_hooks_fire | current-waypoint | KBD_ORCHESTRATOR_ROOT | task:before | progress.json`
  → **zero matches.** Those are vanilla upstream OpenSpec skills (`generatedBy`
  OpenSpec; "Requires openspec CLI"). They fire no KBD hooks, touch no waypoint,
  write no `progress.json`.

**Conclusion:** the SKILL.md claim is **false**. When the flow proceeds from
plan into `/opsx:apply`, the turn is handed to a KBD-blind skill. No task hooks,
no progress signals, no waypoint refresh, no QA gate, no archive-back, no reflect
trigger. → "I get funneled into openspec and lose all connection / don't know
where I am." **This single seam is the reported bug.**

**What good looks like:** a KBD-owned apply *driver* that does not delegate the
whole turn. It reads the backend's machine-readable task surface, drives **one
task at a time**, and fires the hook + syncs state on every task boundary.

Both candidate backends expose exactly such a surface (web-verified):
- **OpenSpec:** `openspec instructions apply --change <name> --json` returns
  `{ contextFiles, progress:{total,complete,remaining}, tasks:[…] }`; plus
  `openspec status --change <name> --json`. Workflow is
  `propose → apply → archive` (extended: `new/continue/ff/verify/bulk-archive`).
- **GitHub Spec Kit:** `/speckit.specify → .plan → .tasks → .implement`;
  `tasks.md` is the parseable task list; `.clarify/.analyze/.checklist` are
  quality gates.

### Goal 2 — Report position every turn  ⚠️ **PARTIAL / mechanism wrong**

- A default reporter exists (`report-progress`, `*:*`). **But** it `printf … >&2`
  (stderr). Claude Code does **not** surface hook stderr into the conversation,
  so the user does not see it. It also only fires when the model sources
  `hooks.sh` and calls `kbd_hooks_fire` — which never happens during a bare
  `/opsx:apply` turn.
- The **reliable** channel is the plain-text "Progress Signals (MANDATORY)" the
  model emits directly (the project CLAUDE.md rule). Those depend on model
  discipline, not enforcement.
- **Honest verdict:** "every single turn no matter what" is **not** achievable
  by the existing shell hook alone. It requires either (a) the new apply driver
  emitting plain-text signals on each task boundary, and/or (b) a Claude Code
  `Stop`/`PostToolUse` settings hook that injects the current position
  (`waypoint_chain` + `i/n`) into context each turn. The shell hook stays as the
  user-overridable extension point, not the user-facing guarantee.

### Goal 3 — Nesting commands  ✅ **EXIST**, ⚠️ **unverified against wrapped execute**

- All four ship with scripts and atomic waypoint updates. Gaps to verify in
  later phases: (a) `/kbd-next-child` "refuse past last child" path; (b) child
  loops get their own `execution.md` driven by the **same** wrapped apply driver
  (today a child would hit the same broken `/opsx:apply` seam); (c) position
  reporter renders the **full** `parent › phase › child` chain, not just the
  innermost name.

### Goal 4 — Backend-agnostic wrapping  ❌ **MISSING**

- No driver abstraction exists. `execute.md` lists `openspec` and `hybrid`
  backends but no `speckit`. Recommendation: one `SpecBackend` interface
  (`list_tasks → [{id,title,done}]`, `mark_done(id)`, `verify()`, `archive()`)
  with `openspec` and `speckit` adapters. Do **not** fork the apply loop per tool.

---

## 3. Findings ranked by severity

| # | Severity | Finding | Fix locus |
|---|---|---|---|
| F1 | **Critical** | Execute does not wrap the spec backend; `/opsx:apply` runs outside KBD. SKILL.md claim of per-task hooks is false. | new `kbd-apply` driver + rewrite `execute.md` dispatch + correct SKILL.md |
| F2 | **High** | Per-turn position guarantee unmet — reporter writes to stderr and is model-voluntary. | apply driver emits plain-text signal; optional Claude Code `Stop` hook |
| F3 | Medium | plan/execute boundary blurred — plan creates the OpenSpec change. | keep change-creation in plan, doc the contract; execute owns the loop only |
| F4 | Medium | No `speckit` adapter / no backend interface. | `SpecBackend` trait + two adapters |
| F5 | Low | `memory-log` hook emits a `jq: parse error` to stderr on fire (observed when firing `assess:before` this session). | harden `memory-log.sh` input parsing |
| F6 | Low | Child loops will inherit the same broken apply seam until F1 lands. | covered by F1 if driver is backend- and depth-agnostic |
| F7 | Low | `hooks.sh` calls `chain_separator`/`waypoint_chain` but does not source `waypoint.sh` — fires `command not found` if the caller sources hooks.sh alone (observed firing `assess:after` this session). | `hooks.sh` should source `waypoint.sh` defensively, or guard the calls |

---

## 4. Recommendations (what to do differently)

1. **Repair, don't rebuild.** The headline deliverable is **one** new
   KBD-owned apply driver (`kbd-apply`, or folded into `kbd-execute`) plus
   corrected docs — not a fleet of new commands. The nesting commands already
   exist; this phase wires them to a correct execute loop.
2. **KBD owns the loop; the spec tool is a subroutine.** Never hand the turn to
   bare `/opsx:apply`. Call `openspec instructions apply --json`, iterate tasks
   one at a time, mark each done via the CLI, sync `progress.json` + waypoint,
   fire `task:before/after`, emit the plain-text signal — then advance.
3. **Separate the user-facing guarantee from the extension hook.** Plain-text
   signals (driver-emitted) are the guarantee; the `*:*` shell hook is the
   user's override/extension point. Document this split; stop implying the
   stderr hook is what the user sees.
4. **Abstract the backend once.** `SpecBackend` interface with `openspec` first,
   `speckit` as a thin second adapter. Defer Spec Kit until OpenSpec wrapping
   works end-to-end.
5. **Fix the false claim immediately.** Correct `kbd-execute/SKILL.md` — it
   currently documents behavior (`/opsx:apply` fires task hooks) that does not
   exist, which is how the seam went unnoticed.

---

## 5. Out of scope for this assessment

- Implementing the driver (that is Execute).
- Spec Kit installation/onboarding.
- Refactoring the upstream OpenSpec skills themselves (we wrap, not fork them).

---

## 6. Cross-tool progress

`progress.json` for this phase will be initialized by `kbd-execute`. No other
tool has contributed to this phase yet.

## 7. Next action

Proceed to **`/kbd-plan kbd-execute-spec-wrapping-and-nesting-2026-06-03`** to
turn F1–F6 into an ordered change list (F1 + F2 first; F4/Spec Kit last).
