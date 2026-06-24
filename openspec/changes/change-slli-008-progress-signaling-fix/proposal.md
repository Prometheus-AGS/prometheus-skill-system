# change-slli-008-progress-signaling-fix

**Phase**: self-learning-loop-integration
**Status**: DONE
**Priority**: 1 of 10 — FIRST CHANGE TO EXECUTE

## Summary

Fix the six concurrent failure modes that have prevented per-turn progress signaling from reliably working. The user has requested this across multiple sessions. This change closes all six root causes simultaneously.

## Root Causes Being Fixed

1. **HTML comment format** — `<!-- prometheus-position -->` is deprioritized by models
2. **Model paraphrase** — models rephrase exact format requirements
3. **Stop gate soft cap** — position gate blocks only once per session
4. **waypoint-render.sh path failure** — `$CLAUDE_PLUGIN_ROOT` unset on some installs
5. **CWD mismatch** — `_wr_find_root()` fails when CWD isn't project root
6. **Skill-load timing** — skill instructions at load time don't reliably override at invocation

## Files Modified

### ALL kbd-* skills — `## Progress Signals (MANDATORY)` section

Updated in: `kbd-assess`, `kbd-analyze`, `kbd-plan`, `kbd-execute`, `kbd-reflect`, `kbd-evolve`, `iterative-evolver`, `pmpo-outer-loop` (new)

**Exact required format (non-negotiable, must be reproduced verbatim in model output):**

```
Starting kbd-plan — self-learning-loop-integration (step 3 of 10)
```

```
Completed kbd-plan — self-learning-loop-integration (step 3 of 10)
```

**How to read N and T:**
- Read `.kbd-orchestrator/phases/<phase>/progress.json` → `changes_completed` = N, `changes_total` = T
- If progress.json is absent, read `current-waypoint.json` → `changes_completed` / `changes_total`
- NEVER estimate N or T — always read the file

**Enforcement in skills:** Every kbd-* skill's instruction block will have:
> "BEFORE any tool call, emit: `Starting <skill-name> — <phase-name> (step N of T)`. Read the actual N and T from progress.json. Emit to plain response text."

### `shared/scripts/lib/waypoint-render.sh`

Fix `_wr_find_root()`: also search `$REPO_ROOT`, `$CLAUDE_PLUGIN_ROOT`, and `$(pwd)` parent chain before failing. Log the resolved path to stderr so hook failures are diagnosable.

### `hooks/hooks.json`

Add PreToolUse variant on `Write|Edit` that checks whether the current turn has emitted a Starting signal. If not, the hook injects a position-reminder comment to stderr (visible in hook debug mode). This is advisory, not blocking — hooks stderr is not user-visible but aids debugging.

## Files Created

### `shared/scripts/write-position-reminder.sh`

Called from waypoint update logic (in `state-checkpoint.sh`) — writes:

```
.kbd-orchestrator/position-reminder.txt
```

Contents:
```
POSITION REMINDER — read this at the start of every turn
Phase: <phase-name>
Step: <N> of <T>
Stage: <stage>
Next command: <exact_next_command>
```

The model is instructed in each kbd-* skill to read this file as its FIRST tool call.

### `shared/scripts/check-progress-signal.sh`

Called by PreToolUse hook — checks `$KBD_TURN_HAS_STARTING_SIGNAL` env var (set by UserPromptSubmit hook). Advisory only, logs to `~/.prometheus/signal-gaps.log`.

## Acceptance Criteria

- Every kbd-* skill invocation emits `Starting … (step N of T)` as plain text BEFORE any tool call
- Every kbd-* skill invocation emits `Completed … (step N of T)` as plain text AFTER all work
- `.kbd-orchestrator/position-reminder.txt` always matches `current-waypoint.json` values
- N and T are always read from files, never estimated
- `waypoint-render.sh` resolves correctly even when CWD is a subdirectory
