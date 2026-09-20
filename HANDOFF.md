# HANDOFF — gofast/liter-llm-mcp-secrets

Branch from `main` (00995fa). One commit. Additive in effect: only the `liter-llm` MCP server entry changes.
Not pushed, not merged, not rebuilt, not reinstalled — those are the owner's.

## Defect

The `liter-llm` MCP server never starts in any harness session on a machine configured by this pack.
`~/.config/liter-llm/liter-llm-proxy.toml` sets `master_key = "${LITER_LLM_MASTER_KEY}"`. The gateway launch agent
(`shared/scripts/liter-llm-api-launch.sh`) sources `~/.prometheus/kbd/secrets.env` before exec, so the gateway works.
The MCP entry ran `liter-llm mcp …` directly; a harness does not have that variable, liter-llm's fail-closed guard
exits 1 —

    Error: [general] master_key is set but empty — an unset ${VAR} interpolates to "" …

— and the harness reports only "Connection closed". Reproduced 2026-09-20 with both installed binaries (1.18.2).

## Change

`.mcp.json` and both generated copies (`dist/plugins/{claude,codex}/prometheus-skill-pack/.mcp.json`): the entry now
runs `bash -c 'set -a; [ -f secrets.env ] && . secrets.env; set +a; exec liter-llm mcp --transport stdio --config …'`
— the same thing the launchd wrapper does, inline so no install path has to be resolved. No secret value is written
anywhere. The entry contains no `${…}` placeholder (Claude Code expands those and fails on an unset variable).

## Verified

- Spawned from the patched entry with a clean environment (HOME, PATH, USER only; no master key exported):
  `initialize` answered in 2 s, serverInfo `liter-llm 1.18.2`; `tools/list` returns 22 tools.
- The generator's `sanitizedMcp()` accepts it (no machine-specific path; `$HOME` is literal).

## NOT verified

- `scripts/tests/skill-system-distribution.test.mjs` cannot run in a worktree (submodules are not initialised:
  "skill inventory root is unavailable: skills/imported/artifact-refiner/skills"). Run it on `main` after merging;
  regenerate `dist/` with `scripts/generate-skill-system-distribution.js` rather than trusting my hand-synced copies.
- Windows: the entry now needs `bash`.

## Same defect, not changed here (kept out to stay surgical)

- `skills/process/liter-llm-bridge/scripts/configure-mcp.sh` writes the old direct form into Claude, OpenCode, Cursor
  and Codex configs (four `register_*` functions).
- `.codex/config.toml` `[mcp_servers.liter-llm]` has the direct form and no `--config` at all.
- Cleaner long-term: `install-liter-llm.sh` installs a `liter-llm-mcp` launcher on PATH (secrets sourced, config
  path fixed) and every registration calls that one name.
- `liter-llm-api.stderr.log` still shows an August parse error (`unknown field target`) with no timestamps, which
  reads like a current failure. It is stale; the config parses. Rotate the log or timestamp stderr.

## After merging

Rebuild and reinstall the plugin, restart the harness, confirm `plugin:prometheus-skill-pack:liter-llm` connects.
