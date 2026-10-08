# change-tlm-001-subagent-identity-probe

**Title:** Confirm per-subagent identity and context injection at runtime in Claude Code and Codex
**Repository:** `prometheus-skill-pack`
**Phase:** team-aware-learning-memory
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` at or after `20d97f2` (contains PRs #111, #121–#123), at `/Users/gqadonis/Projects/prometheus/worktrees/tlm-design`. Never this checkout's `codex/delivery-cadence-recovery` branch, which lacks #121. Delivered through a pull request.

## Why

The design rests on four harness behaviours that are documented but unconfirmed at runtime (assessment sections A and B, analysis Q-1):

1. Claude Code SubagentStart `hookSpecificOutput.additionalContext` reaches the subagent's context. A 2026-06 third-party design doc (regin) says it does not; the current official docs say it does, capped at 10,000 chars.
2. Claude `agent_type` is the frontmatter `name` for project agents, and `<plugin>:<name>` for plugin agents.
3. Codex 0.158 SubagentStart `agent_type` equals the custom agent's `name` from `.codex/agents/<name>.toml`, and its stdout is added as developer context for that subagent.
4. Codex PreToolUse inside a subagent carries `agent_id` and `agent_type` (optional fields in the binary schema).
5. Whether a Codex subagent receives the native `memory_summary.md` (about 10.8 KB, user-level) and a Claude subagent receives the project auto-memory `MEMORY.md` (about 14 KB). Probed **without writing to any memory store**: the subagent is asked whether its context contains specific, distinctive lines that already exist in those files (chosen by the script at run time), and the answer is checked against the files. An evasive or ungrounded answer is recorded UNVERIFIABLE.
6. Whether Codex per-thread memory controls (`memory_mode`, `use_memories`/`generate_memories`, as found in the 0.158 binary) exist as configurable keys, and their effect on subagent threads. Recorded from `codex --help`, the config schema and the binary, without changing user config.

Building the delivery path on an unconfirmed behaviour repeats the PR #121 failure mode, so these are probed first.

## What Changes

- Add `shared/scripts/tests/probe-subagent-identity.sh`. It builds a scratch project containing:
  - a project-local agent named `tlm-probe-role` (`.claude/agents/tlm-probe-role.md`, `.codex/agents/tlm-probe-role.toml`);
  - project-local SubagentStart, PreToolUse and SubagentStop hooks (`.claude/settings.json`, `.codex/hooks.json`). The hooks log their stdin JSON to a marker file and inject a random nonce as additional context.

  It then runs the harness non-interactively (`claude -p …` / `codex exec --dangerously-bypass-hook-trust …`) with a prompt that spawns `tlm-probe-role` and asks it to repeat any nonce it was given.

  **Isolation (revised after spec review):**
  - Claude: `claude -p --setting-sources project,local --strict-mcp-config --no-session-persistence --settings <scratch>/hooks.json --agents '<probe agent JSON>'`. User settings, user hooks and enabled plugins are not loaded, and no transcript is persisted. The probe agent and its hooks exist for the session only, so nothing is written into any project. Authentication still comes from the user's login.
    - Behaviours 1 and 2 run with the scratch directory as cwd.
    - Behaviour 5 runs a second time with cwd = this repository's main checkout, where the project auto-memory `MEMORY.md` exists, so the observation is real. The repository's own project settings load, as in any normal session there, and are recorded.
  - Codex: `codex exec --ephemeral --ignore-user-config -c features.multi_agent=true -c features.memories=true -c 'projects."<scratch>".trust_level="trusted"' --dangerously-bypass-hook-trust`, with `HOME` set to a scratch directory and `CODEX_HOME` left pointing at `~/.codex` (needed for authentication).
    - `--ignore-user-config` keeps user plugins, user hooks and user features out of the run. Only the scratch project's `.codex/` agents and hooks load.
    - `--ignore-user-config` drops the user's `[features] multi_agent = true` and `memories = true` and project trust, so the probe re-enables exactly those through `-c`. Behaviours 3–5 therefore observe the user's real feature set, without the user's plugins or hooks.
    - `--ephemeral` keeps the thread from being persisted, and therefore from being consolidated into `~/.codex/memories`.
    - If `--ignore-user-config` also drops authentication or project hooks, the run is recorded `UNVERIFIABLE` with the reason, and no user config is changed.
    - The plugin-agent form for behaviour 2 is observed with `claude --plugin-dir <scratch plugin>` (session-only).
  - **Residual side effects:** with `--ignore-user-config` none are expected from user hooks or plugins. Any observed side effect is recorded as residual with its source, not hidden.
  - **Proof of no persistent change:** the script hashes, before and after each run:
    - `~/.claude/settings.json`, `~/.codex/config.toml`, `~/.codex/hooks.json`
    - `~/.prometheus/plugins/prometheus-skill-pack/pointers/current`
    - `~/.claude.json`
    - listings of `~/.prometheus/learning-queue/{pending,memory/pending}` and `~/.prometheus/memory-outbox*`
    - `~/.codex/memories/{memory_summary.md,MEMORY.md}`
    - the file count of `~/.claude/projects/<scratch-slug>`

    Every hash block must be non-empty and must contain all four strict keys (`~/.claude/settings.json`, `~/.codex/config.toml`, `~/.codex/hooks.json`, the plugin `pointers/current`). Residuals are a list of `{key, reason}`, never a strict key. `~/.claude.json` (per-project stats the Claude CLI updates) may be declared residual with that reason.
    The script records these immediately before and after **each** harness run, as `json claude-before` / `json claude-after` and `json codex-before` / `json codex-after` blocks in the evidence file. Any difference within a run outside the declared residuals fails the probe. Changes the executing session makes between runs are not attributed to the probe.
  - If a harness cannot load project-level agents or hooks without changing user config, the behaviour is recorded `UNVERIFIABLE`; user config is never edited.
- Record the results in `.kbd-orchestrator/phases/team-aware-learning-memory/evidence/subagent-identity-probe.md`. For each harness and behaviour 1–6: CONFIRMED / REFUTED / UNVERIFIABLE, with the captured stdin JSON and the subagent's reply.
- Name the fallback for each behaviour that is REFUTED or UNVERIFIABLE, which the design (change-tlm-002) must adopt:
  1. Claude SubagentStart injection: inject through PreToolUse(Agent) `updatedInput`, appending to `prompt`.
  2. Claude `agent_type` form: matcher `^(.+:)?<role>$`, plus normalisation in the hook.
  3. Codex `agent_type` is not the role name: resolve the role through the agent-team registry by `agent_id` or the spawn nickname, captured at PreToolUse(spawn_agent).
  4. Codex PreToolUse carries no agent fields: attribute subagent tool events through SubagentStart/SubagentStop `agent_id` correlation.
  5. File-memory tier reaches subagents: the reduction is designed as in-scope work. Not reaching them means there is no reduction work.
  6. Codex per-thread memory controls absent: subagent threads are kept from polluting user memories by documenting `features.memories` guidance and by explicit envelope writes from the pack instead.

## Scope

- `shared/scripts/tests/probe-subagent-identity.sh`
- `.kbd-orchestrator/phases/team-aware-learning-memory/evidence/subagent-identity-probe.md`

## Capabilities

- none (evidence only)
