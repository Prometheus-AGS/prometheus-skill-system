# Evidence — change-tlm-001-subagent-identity-probe

Run 2026-10-04 by `shared/scripts/tests/probe-subagent-identity.sh` (worktree `tlm-design`). Claude Code 2.1.289, codex-cli 0.158.0. Raw outputs: `probe-raw/` (Claude), `probe-raw-codex/` (Codex, underscore agent name), `probe-raw-codex-hyphen-rejected/` (Codex, kebab-case agent name rejected).

## Verdicts

| # | Harness | Behaviour | Verdict | Evidence / fallback |
|---|---|---|---|---|
| 1 | claude | SubagentStart `additionalContext` reaches the subagent | CONFIRMED | Nonce `TLM-NONCE-5d644f624eba` injected by the SubagentStart hook was repeated verbatim by `tlm-probe-role` (project-agent and plugin-agent runs). regin's 2026-06 contrary claim is refuted for 2.1.289. |
| 2 | claude | `agent_type` = agent name; `<plugin>:<name>` for plugin agents | CONFIRMED | SubagentStart/SubagentStop stdin: `agent_type: tlm-probe-role` (session `--agents` agent), `agent_type: tlmprobe:tlm-plug-role` (`--plugin-dir` plugin agent), both with `agent_id`. The main-thread PreToolUse(Agent) had no `agent_id`, so main thread and subagent are distinguishable. |
| 3 | codex | SubagentStart `agent_type` = custom agent name; output reaches the subagent | UNVERIFIABLE | Project `.codex/` agents and hooks did not load: `--ignore-user-config` also drops the user-level `projects.<path>.trust_level` table, and Codex loads a project layer only when trusted. A `-c projects."<path>".trust_level` override did not restore it. Fallback: resolve role identity from the `spawn_agent` call captured at PreToolUse (agent name argument) and correlate later events by `agent_id`. Confirm with a trusted-project run once the operator allows a project trust entry. |
| 4 | codex | PreToolUse inside a subagent carries `agent_id`/`agent_type` | UNVERIFIABLE | Same cause as 3: no probe hook loaded. Fallback: attribute subagent tool events through SubagentStart/SubagentStop `agent_id` correlation. The binary schema declares the fields optional. |
| 5 | claude | Project auto-memory `MEMORY.md` reaches a subagent | CONFIRMED | Run with cwd = main checkout: given an anchor line from the project `MEMORY.md`, the subagent returned the exact following line ("- [Prometheus Toolchain References](reference_prometheus_toolchain.md) — Key repos (surrea…"). About 14 KB of untargeted memory therefore reaches every Claude subagent, so reduction work is in scope (design D-2a). |
| 5 | codex | Native `memory_summary.md` reaches a subagent | UNVERIFIABLE | The subagent answered `NONE` for both nonce and memory line, but because the project agent did not load (behaviour 3), the spawned agent is not proven to be a native subagent with memories in context. Fallback: treat Codex memories as reaching every thread (conservative) and control them with `memories.use_memories` / `memories.generate_memories` (behaviour 6). |
| 6 | codex | Per-thread / per-run memory controls exist | CONFIRMED | 0.158 binary config keys `memories.use_memories`, `memories.generate_memories`, `memories.max_rollouts_per_startup`, `memories.max_raw_memories_for_consolidation`; the threads DB has `memory_mode TEXT NOT NULL DEFAULT 'enabled'` with value `polluted` excluding a thread from consolidation; `codex features list`: `memories` stable/true, `multi_agent` stable/true, `multi_agent_v2` stable/false. |

## Additional runtime findings (design-relevant)
- **Codex agent names must match `[a-z0-9_]`.** A kebab-case agent `tlm-probe-role` was rejected at spawn: "agent_name must use only lowercase letters, digits, and underscores". Agent-team role ids are kebab-case, so Codex exports and SubagentStart matchers need a normalisation (`-` → `_`) with a reverse map to the role id.
- **`codex exec --ephemeral` breaks subagents** ("no rollout found for thread"). Isolation must use `memories.generate_memories=false` instead.
- **`codex exec` blocks on an open stdin** ("Reading additional input from stdin..."). Automation must redirect from `/dev/null`.
- **Version-manager shims break hooks when `HOME` changes.** The pyenv `python3` shim failed under scratch `HOME`, so hooks must resolve an absolute interpreter.

## External writer observed
`~/.codex/config.toml` changed at 2026-10-03T19:46:29 local, **between** probe runs: before == after holds inside every run. The diff against `config.toml.bak-pre-1.11.1` shows the ChatGPT desktop app adding `NODE_REPL_*` MCP environment entries pointing into `/Applications/ChatGPT.app`. It was not written by the probe. Design consequence: the pack must never assume it is the only writer of `~/.codex/config.toml`.

## Persistent-state hashes
Each block brackets one representative harness run (Claude: project-agent run; Codex: multi-agent-v2 run). The learning-queue listing also changed during the Codex v1 run. That run used scratch `HOME`, so the churn came from the background learning worker draining the queue, not from the probe.

```json claude-before
{
 "~/.claude.json": "031b29f24818cac044737019b9783df19f2cc4af41805f8d34c0efcead06a50b",
 "~/.claude/settings.json": "cc66bc330cda9c287d79995754bf21a56aa2d240a97c50294fef1e4424d5b472",
 "~/.codex/config.toml": "b979e277cdbaa22dbb32d63714853debf0bc64a9bddf9835488573857e83d5c8",
 "~/.codex/hooks.json": "9e184236aaec1fc2d7c403bc762c6814b10b9c670b83e5e4a1bf20fc1f9378ab",
 "~/.codex/memories/MEMORY.md": "85dae970e6fdcbb284042e2745f0183859bc439c0e197e8cb3acc3cb1deb5722",
 "~/.codex/memories/memory_summary.md": "dca2502d2aa14f61031aa94be33bc7462116f13ae48eee833138b914687eda06",
 "~/.prometheus/learning-queue/memory/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/learning-queue/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/plugins/prometheus-skill-pack/pointers/current": "4ff30f702e1773c93fd0ac01d13756a0dc987865e48648bcb2d2a02fa5a66c3b"
}
```

```json claude-after
{
 "~/.claude.json": "714629d806c1ed9792232ad06b2c8c920cd9459294b9ebb846e8902d31332e57",
 "~/.claude/settings.json": "cc66bc330cda9c287d79995754bf21a56aa2d240a97c50294fef1e4424d5b472",
 "~/.codex/config.toml": "b979e277cdbaa22dbb32d63714853debf0bc64a9bddf9835488573857e83d5c8",
 "~/.codex/hooks.json": "9e184236aaec1fc2d7c403bc762c6814b10b9c670b83e5e4a1bf20fc1f9378ab",
 "~/.codex/memories/MEMORY.md": "85dae970e6fdcbb284042e2745f0183859bc439c0e197e8cb3acc3cb1deb5722",
 "~/.codex/memories/memory_summary.md": "dca2502d2aa14f61031aa94be33bc7462116f13ae48eee833138b914687eda06",
 "~/.prometheus/learning-queue/memory/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/learning-queue/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/plugins/prometheus-skill-pack/pointers/current": "4ff30f702e1773c93fd0ac01d13756a0dc987865e48648bcb2d2a02fa5a66c3b"
}
```

```json codex-before
{
 "~/.claude.json": "68ed59db9b0ecf8751deea460bd949d9b7f0ee47112f5bb9114459d02850eca9",
 "~/.claude/settings.json": "cc66bc330cda9c287d79995754bf21a56aa2d240a97c50294fef1e4424d5b472",
 "~/.codex/config.toml": "b1f3aed06b53b6b9945a47c46b1045b81f7958ba21930f9735447b6c15f84de7",
 "~/.codex/hooks.json": "9e184236aaec1fc2d7c403bc762c6814b10b9c670b83e5e4a1bf20fc1f9378ab",
 "~/.codex/memories/MEMORY.md": "85dae970e6fdcbb284042e2745f0183859bc439c0e197e8cb3acc3cb1deb5722",
 "~/.codex/memories/memory_summary.md": "dca2502d2aa14f61031aa94be33bc7462116f13ae48eee833138b914687eda06",
 "~/.prometheus/learning-queue/memory/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/learning-queue/pending": "e00e8f5872b90a6eed17c9249527f692dc0fbd3a5da137b6daf1e5ef890b2bfb",
 "~/.prometheus/plugins/prometheus-skill-pack/pointers/current": "4ff30f702e1773c93fd0ac01d13756a0dc987865e48648bcb2d2a02fa5a66c3b"
}
```

```json codex-after
{
 "~/.claude.json": "39bc298a7594045c899710337ce79a306ac5481a8c3b02b59a6f90b6e5532a06",
 "~/.claude/settings.json": "cc66bc330cda9c287d79995754bf21a56aa2d240a97c50294fef1e4424d5b472",
 "~/.codex/config.toml": "b1f3aed06b53b6b9945a47c46b1045b81f7958ba21930f9735447b6c15f84de7",
 "~/.codex/hooks.json": "9e184236aaec1fc2d7c403bc762c6814b10b9c670b83e5e4a1bf20fc1f9378ab",
 "~/.codex/memories/MEMORY.md": "85dae970e6fdcbb284042e2745f0183859bc439c0e197e8cb3acc3cb1deb5722",
 "~/.codex/memories/memory_summary.md": "dca2502d2aa14f61031aa94be33bc7462116f13ae48eee833138b914687eda06",
 "~/.prometheus/learning-queue/memory/pending": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
 "~/.prometheus/learning-queue/pending": "e00e8f5872b90a6eed17c9249527f692dc0fbd3a5da137b6daf1e5ef890b2bfb",
 "~/.prometheus/plugins/prometheus-skill-pack/pointers/current": "4ff30f702e1773c93fd0ac01d13756a0dc987865e48648bcb2d2a02fa5a66c3b"
}
```

```json residual
[{"key": "~/.claude.json", "reason": "Claude CLI per-project stats, updated by the probe's own claude -p runs and by the concurrently running Claude session"}]
```

user config unchanged: yes (strict keys identical before and after every run)
