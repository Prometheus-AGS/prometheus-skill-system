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

`model` accepts either a `[[models]]` **name** from `~/.config/liter-llm/liter-llm-proxy.toml` (e.g. `kbd-judge`, `gpt-5.4`) or a fully qualified `provider/model` identifier (`moonshot/kimi-k2.5`, `zai/glm-4.7`).

> The older `~/.config/liter-llm/config.toml` with a flat `[aliases]` table of `small`/`medium`/`frontier` is retired. liter-llm's real schema is an `[[aliases]]` **array of tables** keyed by `pattern`, so a flat table parsed to nothing — callers silently fell back to sending the class name as a model id. Manage this file with `/liter-llm-bridge configure`.

### `list_models`

Returns the models the proxy can route to — the `[[models]]` names plus the registry
models reachable through `[[aliases]]` patterns. **Not** aliases in the retired
`small`/`medium`/`frontier` sense; those no longer exist.

## The complete tool list (verified)

`liter-llm mcp` exposes exactly these 22 tools, enumerated live from
`tools/list` on 2026-07-30:

```
cancel_batch  cancel_response  chat            create_batch   create_file
create_response  delete_file   embed           file_content   generate_image
list_batches  list_files       list_models     moderate       ocr
rerank        retrieve_batch   retrieve_file   retrieve_response
search        speech           transcribe
```

Notable absences — earlier revisions of this file documented every one of these as if it existed:

| Documented before | Reality |
|---|---|
| `complete` | **Does not exist.** The chat tool is `chat`. |
| `health` | **Does not exist.** Probe `GET /v1/models` instead. |
| `stream` | **Does not exist.** Streaming is a `chat` parameter. |
| `get_cost`, `create_api_key`, `set_rate_limit`, `set_budget`, `cache_*` | **Do not exist as MCP tools.** Rate limits, budgets, and caching are `liter-llm-proxy.toml` config sections (`[rate_limit]`, `[budget]`, `[cache]`), not callable tools. |

There is likewise no `liter-llm complete`, `liter-llm mcp-call`, or `liter-llm
list_models` **CLI** subcommand — the binary ships only `api` and `mcp`. `list_models`
is an MCP tool, reachable over the MCP transport or as `GET /v1/models` on the HTTP
server; it is not something you can run from a shell.

For the bridge's purpose (per-phase routing), `chat` is sufficient — or, from shell,
`kbd_complete` in `shared/scripts/lib/kbd-model-resolve.sh`, which POSTs
`/v1/chat/completions` and reports failures instead of swallowing them.

## Transport

`liter-llm mcp` supports `stdio` (default for Claude Code, opencode, codex) and `http`. Registration MUST pass `--config <abs path>`: `ProxyConfig::discover()` walks the CWD upward and never searches `$HOME`, and without it the stdio server does not merely load zero models — it **fails to start**, because `[mcp] stdio_trust_local` lives in the config it was never given. The bridge always registers stdio — http requires a long-running server which adds operational complexity that defeats the "one binary on PATH" model.

## Versioning

The bridge's smoke test is `configure-models.sh verify`, which checks that `GET /v1/models` returns 200 (not 401) and that one real completion succeeds per role. If liter-llm renames or restructures tools in a future release, the bridge install script will need to bump the pinned commit / tag in `scripts/install-liter-llm.sh`. Track upstream changes via `liter-llm --version` and the fork's CHANGELOG.
