# B5 Codex trust path — evidence

Written by `shared/scripts/tests/test-subagent-delivery.sh --harness codex|both` on 2026-10-04 14:53 UTC.

**Path exercised: native plugin hook.** The PreToolUse fallback contingency (design §5) was **not** needed and was not exercised.

- codex-cli 0.158.0; generated Codex package (bundle `f1f0a86ed52fec7a…`) installed into a scratch `CODEX_HOME` with
  `codex plugin marketplace add <scratch marketplace>` + `codex plugin add prometheus-skill-pack@b5-fixture`
  (package copied verbatim except `.mcp.json` emptied so no MCP server starts; `hooks/hooks.json` untouched).
- Scratch `config.toml`: `[features] hooks = true` (not the deprecated `codex_hooks`), `multi_agent = true`,
  `[memories] generate_memories = false`, `[projects."<pwd -P>"] trust_level = "trusted"`; `auth.json` symlinked.
- Hook trust: `codex exec --dangerously-bypass-hook-trust --skip-git-repo-check … < /dev/null` (scratch gate only).
  Normal interactive use shows a one-time hook-trust prompt for the plugin's non-managed hooks (docs/codex-plugin.md).
- The generated SubagentStart entry (matcher `*`) fired for both custom agents (`api_dev`, `ui_dev`); its
  `hookSpecificOutput.additionalContext` reached each CHILD thread as a `developer` message
  (`content_item_kinds: hooks.additional_context`) and never a parent thread.

| Rollout | Agent role | Hook developer messages | ALPHA-API | ALPHA-UI |
|---|---|---|---|---|
| `rollout-2026-10-04T09-52-54-01a10767-5263-7833-8918-20045a26b572.jsonl` | (parent) | 3 | no | no |
| `rollout-2026-10-04T09-53-10-01a10767-92e5-7e53-bf2e-dc2d2179a131.jsonl` | api_dev | 4 | yes | no |
| `rollout-2026-10-04T09-53-23-01a10767-c343-7711-b25a-f1ab531a281d.jsonl` | ui_dev | 4 | no | yes |

Delivery (delivery.jsonl):

```
  codex api-dev: 6696 chars, 1914 est. tokens, 6696 B SubagentStart + 0 B file tier, scopes ['@project', 'tlm-fixture/api-dev']
  codex ui-dev: 6684 chars, 1910 est. tokens, 6684 B SubagentStart + 0 B file tier, scopes ['@project', 'tlm-fixture/ui-dev']
```
