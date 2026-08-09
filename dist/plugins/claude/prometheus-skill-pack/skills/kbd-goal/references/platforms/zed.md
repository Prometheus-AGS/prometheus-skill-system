# Platform: Zed

**Detection:** `$TOOL == "zed"` OR `ZED_SESSION_ID` env var is set and `$TOOL` not set to another tool.

Zed has no built-in `/goal` command native to Zed itself. However, Zed supports
two modes relevant to KBD:

1. **ACP-connected mode** — Zed is running Claude Code, Codex, or OpenCode as
   an Agent Client Protocol (ACP) backend. The external agent's native `/goal`
   is available through Zed's agent panel.
2. **Standalone mode** — Zed is using its own built-in LLM (no ACP agent). KBD
   implements the loop directly via the `kbd-goal` skill, using repeated prompts
   and the `kbd-goal-evaluator` subagent.

## ACP Detection

KBD detects which mode Zed is in using `scripts/kbd-goal-zed-detect.sh`:

```bash
bash scripts/kbd-goal-zed-detect.sh
# Outputs: "claude-code", "codex", "opencode", or "standalone"
```

Detection logic:
1. Check `$ZED_ACP_AGENT` env var (set by Zed when launching an ACP-connected session)
2. Check `~/.zed/acp-agents.json` for an active connection entry
3. If neither found: output `standalone`

## Routing Decision Table

| Mode | Strategy |
|---|---|
| ACP: `claude-code` | Delegate to Claude Code bridge (see `claude-code.md`). `/goal` is available from the connected Claude Code instance. |
| ACP: `codex` | Delegate to Codex bridge (see `codex.md`). `codex /goal` is available. |
| ACP: `opencode` | Delegate to OpenCode bridge (see `opencode.md`). OpenCode goal plugin drives the loop. |
| `standalone` | KBD emulates the loop: per-turn evaluation via `kbd-goal-evaluator` + continuation injection. |

## Skill Installation

KBD installs `kbd-goal` to `~/.zed/skills/` via `install-skills-flat.sh`.
Zed discovers skills from `~/.zed/skills/` and `~/.config/zed/skills/`
(KBD installs to both paths for compatibility).

Invoke the skill in Zed:
```
/kbd-goal "build weekly standup generator" --phases ideation,spec,creation
```

## Standalone Loop Emulation

When `kbd-goal-zed-detect.sh` outputs `standalone`, the `kbd-goal` skill
drives the loop directly within Zed's context:

### Loop Structure

```
1. Set up goal state (goal.json, STATE.md, TASKS.md)
2. For each task:
   a. Execute: implement + test (implementer turn)
   b. Evaluate: /kbd-goal-evaluator checks stopping condition
   c. If PASS → advance; if FAIL → inject continuation as next prompt
3. Human gate at end of each phase
```

### Continuation Injection (Standalone)

When the evaluator returns FAIL, KBD injects a continuation guidance block
into the next Zed prompt:

```
KBD Goal Loop — Phase: creation — Turn N of max_turns
Current task: task-003 (implement retry logic)
Failure reason: TestRetryExponentialBackoff fails (tests/retry_test.go:47)
Next action: Fix the exponential backoff calculation in pkg/retry/retry.go

Continue with the above. Do not declare completion — /kbd-goal-check evaluates that.
```

This is injected as the `system` or `user` context in the next Zed prompt,
simulating what Claude Code's native evaluator does at the framework level.

### Turn Budget

In standalone mode, KBD enforces turn budget locally:

- `max_turns_per_phase` (from `goal.json`) — hard ceiling
- `max_no_progress_turns` — consecutive turns with no STATE.md change before pausing
- Token budget — tracked via cumulative STATE.md update timestamps (approximate)

## ACP-Connected Mode

When Zed is connected to an ACP agent (Claude Code, Codex, or OpenCode), the
agent's native `/goal` is accessible from Zed's agent panel or via the agent's
slash commands forwarded through ACP.

KBD's role in ACP-connected mode:
- Manage Ideation and Specification phases (always KBD-owned)
- Write `goal.json`, `IDEAS.md`, `SPEC.md` artifacts
- At Creation phase: issue `/goal <stopping-condition>` to the connected agent
  via ACP's `agent/send_message` call

## Setup Requirements

- Zed ≥ any current stable release (ACP was added 2025)
- For ACP mode: Claude Code ≥ v2.1.139 or Codex ≥ 0.128.0 as the ACP backend
- For standalone mode: no additional dependencies — `kbd-goal-evaluator` is a
  built-in KBD agent shipped in `agents/kbd-goal-evaluator.md`
- Run `install-skills-flat.sh` once to install skills to `~/.zed/skills/`

## Skills in Zed

Zed uses the same SKILL.md format as agentskills.io. Skills appear in Zed's
slash command panel. The `disable-model-invocation: true` frontmatter flag
(if set) prevents Zed from auto-invoking the skill; KBD skills do NOT set
this flag — they are designed to be auto-discoverable.
