# liter-llm MCP Tools

`liter-llm mcp --transport stdio` exposes 22 tools (per the upstream README). The bridge skill uses a small subset; the rest are documented for completeness.

## Tools the Bridge Uses

### `complete`

Primary routing tool. Sends a prompt to a model alias and returns the completion. This is the tool harness hooks call to mediate `[MODEL_ROUTING]` directives.

```json
{
  "name": "complete",
  "arguments": {
    "model": "medium",
    "messages": [{"role": "user", "content": "..."}],
    "max_tokens": 4096
  }
}
```

`model` accepts either an alias from `~/.config/liter-llm/config.toml` (`small`, `medium`, `frontier`) or a fully qualified `provider/model` identifier (`anthropic/claude-sonnet-4-6`).

### `list_models`

Returns the available aliases plus the resolved provider/model behind each. Used by `/liter-llm-bridge status`.

### `health`

Verifies each configured provider is reachable. Used by the configure phase smoke test.

## Tools the Bridge References But Does Not Use

| Tool                | Purpose                                                |
|---------------------|--------------------------------------------------------|
| `create_api_key`    | Mint virtual API keys for downstream callers           |
| `set_rate_limit`    | RPM / TPM caps per virtual key                         |
| `get_cost`          | Cost tracking per request / per key                    |
| `set_budget`        | Hard budget enforcement                                |
| `cache_get` / `cache_set` | Response caching (content-hash keyed)            |
| `stream`            | SSE streaming variant of `complete`                    |
| Plus 13 other tools for key management, observability, and provider config |

For the bridge's purpose (per-phase routing), `complete` is sufficient. Cost tracking via `get_cost` is useful for verifying the cost reduction is actually happening — consider running a weekly sweep.

## Transport

`liter-llm mcp` supports `stdio` (default for Claude Code, opencode, codex) and `http`. The bridge always registers stdio — http requires a long-running server which adds operational complexity that defeats the "one binary on PATH" model.

## Versioning

The bridge's smoke test verifies the `complete` tool is present. If liter-llm renames or restructures tools in a future release, the bridge install script will need to bump the pinned commit / tag in `scripts/install-liter-llm.sh`. Track upstream changes via `liter-llm --version` and the fork's CHANGELOG.
