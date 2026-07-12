# MCP Env Round-Trip — change-cpv-005

_2026-07-12. Tested in a throwaway `CODEX_HOME=/tmp/cxenv` to avoid mutating the real `~/.codex`._

## What was confirmed

- `scripts/codex-provision-mcp-env.sh` works: sources key **names** from the
  environment (TAVILY_API_KEY found from `~/.bash_profile`), writes
  `[shell_environment_policy] inherit = "all"` idempotently, persists **no secret
  values**, bash 3.2 compatible.
- With the plugin installed, `codex mcp get tavily` shows the server registered
  **with a `TAVILY_API_KEY` env entry** (masked non-empty).
- Inline **literal** plugin-`.mcp.json` env values are **proven honored** (see the
  original spike, change-cpi-001: `CPI_SPIKE_ENV=env-honored-42` reached the server).

## What is NOT cleanly confirmed (honest gap)

The end-to-end claim — *`inherit="all"` forwards the **live** `TAVILY_API_KEY`
value to the spawned tavily MCP server so `codex doctor` stops warning* — was
**not** definitively verified:

- The throwaway home has **no Codex auth** (`✗ auth`), so `codex doctor`'s MCP
  section reported `0 servers` and could not evaluate the env warning.
- Whether Codex expands the plugin server's `${TAVILY_API_KEY}` reference (vs
  passing the literal string) was not isolated.
- Confirming it properly needs either the real authed `~/.codex` (which this test
  deliberately did not mutate) or a focused "run tavily-mcp and check keyless-mode"
  probe.

## Guidance (reliable path)

For **guaranteed** provisioning of a keyed plugin server, use an inline per-server
env block with the real value in `~/.codex/config.toml` (0600, user-local, not
committed) — the pattern proven for `tavily_web` earlier this session:

```toml
[mcp_servers.tavily]
env = { TAVILY_API_KEY = "tvly-…" }
```

`inherit="all"` (the helper's output) is the lighter, no-secrets-on-disk option;
prefer it, and fall back to inline env if a specific server still can't see its key.

## Verdict

Helper delivered + tested; env-registration observed. The live-value forwarding via
`inherit` is **documented but unconfirmed** — carried as a caveat to the reflection,
with the inline-env fallback as the guaranteed alternative. Cleaned up the throwaway home.
