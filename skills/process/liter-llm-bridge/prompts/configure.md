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

3. **Write `~/.config/liter-llm/config.toml`** — map class aliases to concrete provider models. Example:

   ```toml
   [aliases]
   small    = "groq/llama-3.3-70b-versatile"
   medium   = "anthropic/claude-haiku-4-5"
   frontier = "anthropic/claude-sonnet-4-6"

   [providers.anthropic]
   api_key_env = "ANTHROPIC_API_KEY"

   [providers.groq]
   api_key_env = "GROQ_API_KEY"

   [providers.ollama]
   base_url = "${OLLAMA_HOST:-http://localhost:11434}"
   ```

   Pick the cheapest available provider per class from the coverage report. Prefer:
   - **small**: Ollama (local, free) → Groq (fast, cheap) → Together
   - **medium**: Groq (Mixtral-class) → Anthropic Haiku → vLLM
   - **frontier**: Anthropic Sonnet → OpenAI GPT-4-class → (no fallback — frontier-required phases must fail loud rather than degrade)

   Run `bash scripts/configure-mcp.sh write-toml` with the chosen mapping.

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

5. **Smoke test** — invoke the `liter-llm` MCP `complete` tool with `model: "small"` and a trivial prompt (`"say hi"`). Verify it routes to the configured small-class provider and returns a response. If the smoke test fails, the registration is suspect — surface the error and do not declare success.

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
