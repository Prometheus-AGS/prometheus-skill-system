# Platform: Kimi Code

**Detection:** `$TOOL == "kimi"` OR `kimi-code` binary in PATH and `$TOOL` not set to another tool.

Kimi Code has a built-in `/goal next <objective>` command that maintains a
sequential goal queue: when the current goal completes, Kimi automatically
picks up the next queued goal. This is a **queue model**, not a
condition-based loop — Kimi does not evaluate a stopping condition; the user
or an external evaluator must signal completion.

KBD provides the condition evaluation layer that Kimi's native goal system
lacks, enabling parity with Claude Code's `/goal` evaluator pattern.

## Skill Discovery

Kimi Code auto-discovers SKILL.md skills from `~/.kimi-code/skills/` and
injects them into the system prompt. KBD ships both `kbd-goal` and
`kbd-goal-check` here. Skills are invoked via `/skill:<name>` or auto-triggered
by the agent when the keyword triggers match.

## Routing Decision Table

| Phase | Strategy |
|---|---|
| Ideation | KBD (kbd-idea-critic subagent loop). Queue via `/goal next "3 survivors in IDEAS.md"` — but KBD evaluates, not Kimi. |
| Specification | KBD (kbd-spec-reviewer loop). Queue via `/goal next "SPEC.md reviewer returns PASS"`. |
| Creation | Queue each task batch via `/goal next`. `/kbd-goal-check` evaluates after each turn. |
| Deployment | KBD orchestrates; `/goal next` used for per-environment deploy tasks. |

## Per-Phase Integration

### Starting a multi-phase goal on Kimi

```
/kbd-goal "build weekly standup generator" --phases ideation,spec,creation
```

This creates `.kbd-orchestrator/goals/<slug>/goal.json` and queues all phases
via Kimi's `/goal next`:

```
# KBD queues all phases at start
/goal next "3 candidates in IDEAS.md score ≥7.0 aggregate"
/goal next "SPEC.md reviewer returns PASS"
/goal next "all tasks in TASKS.md are [x] and all tests pass"
```

Kimi processes goals in order. The evaluator (`/kbd-goal-check`) runs after
each implementation turn to determine if the current goal/phase is complete.

### After each execution turn

The agent automatically invokes `kbd-goal-check` to evaluate stopping condition:

```
/kbd-goal-check
```

- `PASS` → current Kimi goal is satisfied; Kimi advances to the next queued goal
- `CONTINUE` → next action identified; Kimi continues with the next turn

## Evaluator Pattern

Since Kimi's `/goal next` is queue-based, KBD provides the condition evaluator
as a skill. This preserves the maker≠evaluator separation:

```
Turn 1: Implementer agent does work (adds features, runs tests)
Turn 2: /kbd-goal-check evaluates stopping condition (PASS or CONTINUE)
Turn 3 (if CONTINUE): Implementer continues with next action from CONTINUE output
```

The `/kbd-goal-check` skill reads `goal.json → phases[active].stopping_condition`
and tests it against STATE.md and evidence files — it never reads from the
agent's own turn output without going through file artifacts.

## YOLO Mode

For unattended multi-phase goal execution, use Kimi's `/yolo` flag to skip
per-turn confirmation. KBD recommends enabling it only after the Specification
phase is approved:

```
# After SPEC.md human gate approval, enable auto-continue for Creation
/yolo
/kbd-goal --resume <slug> --phases creation
```

## Skill Configuration (`~/.kimi-code/config.toml`)

KBD's `install-skills-flat.sh` installs both `kbd-goal` and `kbd-goal-check`
to `~/.kimi-code/skills/`. Additional Kimi config:

```toml
[skill_settings]
extra_skill_dirs = ["~/.kimi-code/skills"]

[mcp_servers.surreal-memory]
type = "sse"
url  = "http://localhost:23001/mcp/sse"
```

## Requirements

- Kimi Code with `/goal next` support (any current release)
- KBD skills installed: `bash scripts/install-skills-flat.sh` (installs `kbd-goal` and `kbd-goal-check` to `~/.kimi-code/skills/`)
- No separate binary required — evaluator logic is in the `kbd-goal-check` SKILL.md

## Human Gates

On Kimi Code, human gates are implemented via `pmpo-elicit`'s file-based async contract
combined with `kbd-goal-check`'s `pending_elicitation` detection.

### Ideation → Spec gate

After `kbd-goal-check` returns PASS for the ideation phase:

1. KBD writes a checkpoint to `goals/<slug>/elicitations/<id>/` via `pmpo-elicit-checkpoint.sh`.
2. KBD sets `goal.json → phases[ideation].status = "pending_elicitation"`.
3. `kbd-goal-check` detects `pending_elicitation` status on the next evaluation turn
   and returns `CONTINUE` with next action: "Review `goals/<slug>/elicitations/<id>/request-prompt.txt`
   and write `result.json` to unblock the gate."
4. The user writes `result.json` with their choice.
5. KBD calls `pmpo-elicit-resume.sh`, reads the decision, records it in `goal.json`,
   and queues the Spec phase goal via `/goal next`.

### Spec → Creation gate

Same pattern as above, with the three Creation options ("Approve", "Request revision",
"Stop here") written to `request-prompt.txt`.

### With --auto-gates

Both gates are skipped. KBD writes `{"decision": "auto-approved", "provenance": "implicit"}`
to `goal.json` and immediately queues the next phase goal via `/goal next`.
Equivalent to enabling Kimi YOLO mode for phase transitions only.
