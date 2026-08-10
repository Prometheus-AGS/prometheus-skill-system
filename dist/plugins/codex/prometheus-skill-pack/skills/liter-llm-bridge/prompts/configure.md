# liter-llm-bridge — Configure

You are running the **configure** phase. The goal is to map the user's available providers to the three model classes (`small`, `medium`, `frontier`) and register liter-llm's MCP server with the active harness.

## Model Selection

**Required model class: `small`**

This is mechanical config work. Read `project.json → model_policy.phases.liter-bridge-configure`. If absent, proceed on the host model.

## Procedure

1. **Detect available providers** — run `bash scripts/detect-providers.sh`. The script scans environment variables and returns a JSON report:

   ```json
   {
     "providers": {
       "anthropic":  { "key_var": "ANTHROPIC_API_KEY",  "present": true,  "classes": ["frontier", "medium"] },
       "openai":     { "key_var": "OPENAI_API_KEY",     "present": false, "classes": ["frontier", "medium", "small"] },
       "groq":       { "key_var": "GROQ_API_KEY",       "present": true,  "classes": ["small", "medium"] },
       "together":   { "key_var": "TOGETHER_API_KEY",   "present": false, "classes": ["small", "medium"] },
       "ollama":     { "key_var": "OLLAMA_HOST",        "present": true,  "classes": ["small"] },
       "vllm":       { "key_var": "VLLM_BASE_URL",      "present": false, "classes": ["small", "medium"] }
     },
     "coverage": {
       "small": ["groq", "ollama"],
       "medium": ["anthropic", "groq"],
       "frontier": ["anthropic"]
     }
   }
   ```

   See `references/provider-env-vars.md` for the canonical list.

2. **Identify gaps** — for each class with empty `coverage`, the user has no viable provider. Use AskUserQuestion to either:
   - Collect a missing key for a high-priority provider
   - Accept the silent fallback to the host model for that class

   Do NOT silently downgrade a `frontier`-required phase — the routing contract is explicit that this case must `MODEL MISMATCH` rather than degrade.

3. **Write the model config** — run `bash scripts/configure-models.sh repair`, then
   `add-provider` for each provider you use. It merges and never clobbers.

   > **Do not hand-write `~/.config/liter-llm/config.toml` with a flat `[aliases]`
   > table.** That file and that shape are **retired**: liter-llm's real config is
   > `liter-llm-proxy.toml`, where `[[aliases]]` is an **array of tables** keyed by
   > `pattern` and models are `[[models]]` entries. A flat table parsed to nothing, so
   > callers silently sent the class name (`"frontier"`) as a model id — the root cause
   > of the adversarial judge never reaching a second model. `[providers.*]` is not a
   > liter-llm table at all. `configure-mcp.sh write-toml` has been removed.

   The generated config looks like this — note the three parts that are **mandatory**:

   ```toml
   [general]
   # Without this, EVERY /v1/* request answers 401 — including /v1/models.
   master_key = "${LITER_LLM_MASTER_KEY}"

   [security]
   # Default is deny_private, which REFUSES localhost base_urls.
   outbound_policy = "off"

   [[models]]
   name = "kbd-judge"                       # the name callers send as "model"
   provider_model = "openai/gpt-5.6-sol"
   api_key = "sk-proxy-local"               # any non-empty value for the local proxy
   base_url = "http://localhost:8181/v1"
   ```

   Roles map to these names in `~/.prometheus/kbd/models.toml`:

   ```toml
   [roles]
   generator = "kbd-frontier"   # the producer; compared against, never dispatched
   critic    = "kbd-critic"     # MUST differ from generator
   judge     = "kbd-judge"
   ```

   Secrets never go in the TOML — put keys in `~/.prometheus/kbd/secrets.env` (`0600`)
   and reference them as `${VAR}`. liter-llm supports `${VAR}` only (no
   `${VAR:-default}`) and expands an **unset** var to `""`, which surfaces later as an
   unexplained 401, so `configure-models.sh check` verifies each one is set.

   Provider choice: prefer the local `openai-proxy` (free, no inbound key) for
   everything, and add a second *different* provider so the judge can differ from the
   producer. A frontier-required phase must fail loud rather than degrade.

4. **Register the MCP server with the active harness** — run `bash scripts/configure-mcp.sh register`. The script auto-detects which harness invoked it (via env vars: `CLAUDE_CODE_*`, `OPENCODE_*`, `CURSOR_*`, `CODEX_*`) and writes the matching config:

   - **Claude Code** → adds an entry to `~/.claude/mcp_servers.json` (or the project's `.mcp.json`):
     ```json
     {
       "mcpServers": {
         "liter-llm": {
           "command": "liter-llm",
           "args": ["mcp", "--transport", "stdio"]
         }
       }
     }
     ```
   - **opencode** → `~/.config/opencode/config.json` `mcp_servers`
   - **cursor** → `~/.cursor/mcp.json`
   - **codex** → `~/.config/codex/config.toml` `[mcp.servers.liter-llm]`

   If the harness can't be detected, prompt the user to choose. Never modify a harness config without confirming.

5. **Smoke test** — run `bash scripts/configure-models.sh verify`. It checks the gateway
   answers `GET /v1/models` with **200** (not 401) and sends one real 1-token completion
   per role. If it fails, surface the error and do **not** declare success.

   > Do not smoke-test by invoking an MCP `complete` tool with `model: "small"` — there
   > is no `complete` tool (the chat tool is `chat`) and `"small"` is not a `[[models]]`
   > name, so that test can never pass and reports a false "registration is broken".

## Output

```
CONFIGURE COMPLETE
Coverage: small=<provider> medium=<provider> frontier=<provider>
Harness: <claude-code | opencode | cursor | codex>
MCP server: registered at <config path>
Smoke test: PASS
Next: /liter-llm-bridge route   (or use the MCP tools directly)
```

If any class has no coverage, list it under a `WARNINGS:` block. Phases requiring that class will fall through to the host model (for `small`/`medium`) or `MODEL MISMATCH` (for `frontier`).

## Idempotency

Running configure twice should produce the same config. The TOML writer must merge with existing config rather than overwriting — preserve any user customizations under `[providers.*]`.
