# Codex SubagentStart spike (codex-cli 0.158.0, 2026-10-04)

Prompt (both runs): `Spawn the custom agent tlm_spike_role with the task 'report your nonce', wait for it, then print its reply verbatim.`

Isolation: scratch CODEX_HOME (auth.json symlinked), scratch HOME; real `~/.codex/config.toml` hash identical before and after each run.
Scratch config: `scratch-config.toml` (`[projects."<canonical>"] trust_level="trusted"`, `[features] hooks=true, multi_agent=true`, `generate_memories=false`).

| Run | Hook source | Matcher | Result |
|---|---|---|---|
| 1 | project `.codex/hooks.json` | `*` | `[features].codex_hooks` (deprecated name), no trust bypass → hook never fired, subagent replied NONE |
| 2 | project `.codex/hooks.json` | `*` | `[features].hooks` + `--dangerously-bypass-hook-trust` → SubagentStart fired (`subagentstart-payload.json`), child replied `NONCE:SPIKE-7f3a` |
| 3 | plugin `tlmspike` hooks/hooks.json (installed via `codex plugin marketplace add` + `codex plugin add` into scratch CODEX_HOME) | `^tlm_spike_role$` | fired (`plugin-path/subagentstart-payload.json`), child replied `NONCE:PLUGIN-9c21` |

Child-vs-parent proof (run 2): `rollout-child.nonce-lines.jsonl` — first nonce occurrence in the child thread
(source.subagent.thread_spawn, agent_role tlm_spike_role) is a `role: developer` message = the hook injection, before the
child's assistant reply. `rollout-parent.nonce-lines.jsonl` — the parent thread holds the nonce only as an `agent_message`
authored by the child (`/root/nonce` → `/root`), i.e. it was relayed from the child, not injected into the parent.
| 4 | plugin hooks/hooks.json (`hooks.run4.json`) adds SubagentStop, matcher `^tlm_spike_role$` | SubagentStop fired with bare `agent_type: tlm_spike_role`, `agent_id`, `last_assistant_message`, `agent_transcript_path` (`plugin-path/subagentstop-payload.json`); SubagentStart again injected `NONCE:PLUGIN-9c21` (`plugin-path/rollout-child.nonce-lines.jsonl`: developer message in the child thread) |
