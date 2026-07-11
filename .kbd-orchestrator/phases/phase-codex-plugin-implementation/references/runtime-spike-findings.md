# Runtime Spike Findings — change-cpi-001

_Executed 2026-07-11 against **codex-cli 0.144.1** with a throwaway plugin (`cpi-spike`: 1 skill + 1 hook + 1 `.mcp.json` server with `env` + a local marketplace). Installed, inspected, removed._

## Verdicts (these gate 004 and 005)

| Question | Verdict | Evidence |
|---|---|---|
| Does `codex plugin` install a local plugin? | **YES** | `codex plugin marketplace add <dir>` → `codex plugin add <name>@<marketplace>`. Recorded in `~/.codex/config.toml` as `[marketplaces.<mp>]` + `[plugins."<name>@<mp>"] enabled = true`; full plugin copied to `~/.codex/plugins/cache/<mp>/<name>/<version>/` (= `PLUGIN_ROOT`). |
| Are all components bundled? | **YES** | Cache contains `.codex-plugin/plugin.json`, `skills/<n>/SKILL.md`, `hooks/hooks.json`, `.mcp.json`, `.agents/plugins/marketplace.json`. |
| **Does plugin `.mcp.json` honor `env`?** | **YES ✅** | `codex mcp get cpi-spike-mcp` shows `env: CPI_SPIKE_ENV=…` on the plugin's stdio server. → **change-004 may use inline `env`** (still keep secrets out of git — values from the environment). |
| Plugin `.mcp.json` format accepted? | **Direct map works** | `{ "<name>": { command, args, env } }` at plugin root installed cleanly and registered in `codex mcp list`. (`mcp_servers` wrapper also spec-valid.) |
| Marketplace `source` for a plugin at repo root | `source.source="local"`, `source.path="./"` | `codex plugin marketplace add <dir>` resolved the `.agents/plugins/marketplace.json` and listed the plugin. `source_type="local"` recorded. |
| **Do plugin (non-managed) hooks fire?** | **Bundled + schema-accepted; trust is INTERACTIVE (not headlessly verifiable)** | `hooks/hooks.json` with PascalCase `SessionStart` copied into `PLUGIN_ROOT`. No hook-trust field is written to `config.toml` on install — consistent with the spec's "non-managed hooks are skipped until the user reviews & trusts them." Actual firing requires a trusted interactive `codex` session; it cannot be confirmed in headless/CI. |

## Commands (verb reference for codex-cli 0.144.1)

- `codex plugin marketplace add <path|url|repo>` · `codex plugin marketplace list|remove`
- `codex plugin add <name>@<marketplace>` (install) · `codex plugin list` · `codex plugin remove <name>`
- **No** `install` / `details` verbs (they exist in *claude* plugin, not codex). Inspect MCP via `codex mcp list` / `codex mcp get <server>`; health via `codex doctor`.

## Implications for the plan

- **change-004 (plugin .mcp.json):** use the **direct-map** form with inline `env`; source secret values from the environment (never commit). Confirmed working.
- **change-005 (hooks):** wire `plugin.json.hooks → hooks/hooks.json` (PascalCase, same as Claude — no snake_case translation). Ship + document the **interactive non-managed trust** flow (`${PLUGIN_ROOT}`/`${PLUGIN_DATA}`); mark end-to-end firing as **requiring a manual trusted-session check** — it is NOT auto-verifiable headlessly. Scope stays "wire + document," matching the assessment's fallback.
- **change-003 (marketplace):** per-plugin `source.source ∈ {local, git-subdir, …}` + `source.path` (`./`-relative) confirmed; `[marketplaces.*]`/`[plugins.*]` live in `~/.codex/config.toml`.
- **PLUGIN_DATA** is created lazily (first session), not at install.
