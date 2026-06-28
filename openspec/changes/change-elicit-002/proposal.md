---
id: change-elicit-002
title: Escalation-points guide and platform routing table
phase: pmpo-elicit
gaps: [G-03, G-05]
goals: [G3, G5]
priority: HIGH
effort: M
agent: claude-code
status: done
scope:
  - skills/process/pmpo-elicit/references/escalation-points.md
---

# change-elicit-002 — Escalation-points guide

## Context

Multiple KBD stages reference pmpo-elicit in prose but lack a shared reference
for WHEN to call it, HOW to call it on each platform, and WHERE to write the
elicitation artifacts. This single doc becomes the source of truth that all
wiring changes (003–006) reference.

## Scope

**New file:** `skills/process/pmpo-elicit/references/escalation-points.md`

### Section 1 — Stage trigger map

| Stage | Trigger condition | Criticality | State dir |
|-------|-----------------|-------------|-----------|
| `kbd-analyze` | Stack contest: score gap < 15% between top two stack candidates | high | `.kbd-orchestrator/phases/<phase>/` |
| `kbd-goal` — Ideation→Spec gate | Human gate at phase boundary (skipped when `--auto-gates`) | high | `goals/<slug>/` |
| `kbd-goal` — Spec→Creation gate | Human gate at phase boundary (skipped when `--auto-gates`) | high | `goals/<slug>/` |
| `kbd-goal` — Creation inner-loop | Any escalation[] entry during task execution | medium | `goals/<slug>/` |
| `pmpo-outer-loop` — `loop-tick` | Regression or `max_no_progress_ticks` reached | blocking | `.kbd-orchestrator/loops/<name>/` |
| `pmpo-outer-loop` — `loop-define` | Ambiguous field during interactive loop definition | medium | `.kbd-orchestrator/loops/<name>/` |

### Section 2 — Platform routing table

| Platform | Question UI | Sync/Async | Resume signal |
|----------|------------|-----------|---------------|
| Claude Code | `AskUserQuestion` (built-in tool) | Synchronous — no checkpoint needed | Immediate; write result.json then call resume.sh |
| Codex CLI | None — present via `request-prompt.txt` | Async file-based | User writes `result.json`; re-invoke `codex` |
| OpenCode | Goal state via `update_goal` tool surfaces message | Async file-based | Next agent tick polls `result.json` |
| Kimi Code | `kbd-goal-check` detects `pending_elicitation` in `goal.json` | Async file-based | `kbd-goal-check` reads result, unblocks `/goal next` queue |
| Zed (standalone) | `request-prompt.txt` in workspace; pause background task | Async file-based | User writes `result.json`; re-arm task |
| Zed (ACP-connected) | Delegates to connected agent's native UI | Depends on connected agent | Depends on connected agent |

### Section 3 — Shared file contract

**Elicitation directory:** `<state-dir>/elicitations/<caller>-<timestamp>/`

**Files (all in elicitation dir):**

| File | Written by | Required | Contents |
|------|-----------|----------|---------|
| `request.json` | `pmpo-elicit-checkpoint.sh` | YES | `elicitation.schema.json` (kind=request) |
| `checkpoint.json` | `pmpo-elicit-checkpoint.sh` | YES | Caller state: id, caller, timestamp, status |
| `request-prompt.txt` | `pmpo-elicit-checkpoint.sh` | YES (for non-Claude-Code) | Human-readable question + response instructions |
| `result.json` | Operator or Claude Code AskUserQuestion handler | YES (to resume) | `elicitation.schema.json` (kind=result) |

**Exit codes for checkpoint.sh:**
- `0` — internal use (should not occur in caller context)
- `1` — error (bad arguments, write failure)
- `2` — BLOCKED — loop should pause and await result.json

**Exit codes for resume.sh:**
- `0` — success; answer + provenance on stdout as JSON
- `1` — not ready (result.json absent or malformed)

### Section 4 — Recording resolved answers

After resume.sh returns, the calling stage MUST:
1. Record `elicitation_id` alongside the resolved value (e.g., in `decision-log.md`, `goal.json`, `loop.json`)
2. Record `provenance` (user / source / research / implicit)
3. Clear the `checkpoint.json → status` from "pending" to "resolved" (resume.sh does this automatically)

## Tasks

- [ ] 1. Write `references/escalation-points.md` with all four sections
- [ ] 2. `npm run validate:strict skills/process/pmpo-elicit` passes clean
