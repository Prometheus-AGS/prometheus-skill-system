# Platform: OpenCode

**Detection:** `$TOOL == "opencode"` OR `opencode` binary in PATH and `$TOOL` not set to another tool.

OpenCode has no built-in `/goal` command. KBD uses the
`@prevalentware/opencode-goal-plugin` (also known as `watzon/opencode-goal`),
which provides `/goal`, `/goal history`, `/goal edit`, `/goal pause`,
`/goal resume`, and persistent goal state. KBD auto-installs this plugin during
`install-skills-flat.sh` if it is missing.

## One-Time Setup (per machine)

`install-skills-flat.sh` handles this automatically for new installs. To run
manually:

```bash
# Check if plugin is installed
opencode plugins list | grep -q goal-plugin || \
  npx @prevalentware/opencode-goal-plugin install

# Or via opencode plugin manager
opencode plugins add @prevalentware/opencode-goal-plugin
```

KBD writes plugin configuration during install to `~/.opencode/config.toml`:

```toml
[goal_plugin]
auto_continue         = true
max_auto_turns        = 20
no_progress_token_threshold = 5000
max_no_progress_turns = 3
default_token_budget  = 200000
```

These values are KBD-tuned defaults. Adjust in your `~/.opencode/config.toml`
after install if needed.

## Routing Decision Table

| Phase | Strategy |
|---|---|
| Ideation | Always KBD (kbd-idea-critic subagent loop) |
| Specification | Always KBD (kbd-spec-reviewer subagent loop) |
| Creation | Delegate goal state to plugin; KBD manages phase transitions |
| Deployment | KBD orchestrates; delegates specific deploy tasks to plugin |

## Per-Goal Integration

When KBD starts a goal on OpenCode, it uses the plugin's agent tools to wire
up goal state:

```
# At goal start
create_goal(name=<slug>, description=<goal description>, context=<SPEC.md path>)

# When a new phase begins
update_goal(id=<slug>, context=<new-phase-context.md>)

# When a phase completes (human gate passed)
update_goal(id=<slug>, status="phase_complete", next_phase=<next-phase-name>)
```

The plugin drives the within-phase execution loop. KBD drives cross-phase
transitions.

## Goal State

OpenCode goal state is held by the plugin in its own persistence layer (SQLite
or JSON, depending on plugin version). KBD also writes its own `goal.json` and
`STATE.md` under `.kbd-orchestrator/goals/<slug>/` — the KBD files are the
source of truth for cross-platform resumability.

## Evaluator

OpenCode with the goal plugin has its own continuation/stop logic. KBD's
`kbd-goal-evaluator` is invoked at phase boundaries to confirm that the
phase-level stopping condition is met before KBD advances to the next phase
(even if the plugin's within-phase loop has stopped).

## Requirements

- OpenCode ≥ latest stable
- `@prevalentware/opencode-goal-plugin` installed (auto-installed by KBD)
- Node.js ≥ 18 (for plugin install via npx)
- Run `install-skills-flat.sh` once per machine to configure
