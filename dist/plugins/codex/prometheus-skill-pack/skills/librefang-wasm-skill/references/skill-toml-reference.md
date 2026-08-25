# skill.toml Reference (LibreFang)

Source: `librefang/crates/librefang-skills/src/lib.rs` — `SkillManifest`.

A LibreFang skill manifest has one required section (`[skill]`) and several
optional sections. The kernel deserializes via serde with `#[serde(default)]`
on most fields, so omitted sections fall back cleanly.

## Full Schema

```toml
# REQUIRED — skill identity. All fields except `name` have sensible defaults.
[skill]
name = "my-skill"           # required; matches /^[a-z0-9-]+$/, ≤64 chars
version = "0.1.0"           # semver; defaults to "0.0.0"
description = "..."         # human description shown in CLI/UI
author = "your-name"
tags = ["category", "..."]  # free-form

# OPTIONAL — runtime declaration. Defaults to type = "promptonly".
[runtime]
type = "wasm"               # one of: python | wasm | node | shell | builtin | promptonly
entry = "my_skill.wasm"     # Cargo normalizes package-name hyphens to underscores

# OPTIONAL — tool surface exposed to the LLM. Each [[tools]] entry produces
# one callable tool with the name and JSON Schema input contract below.
[[tools]]
name = "do-thing"           # unique within the skill
description = "..."         # shown to the LLM
input_schema = { type = "object", properties = { ... }, required = [...] }

# OPTIONAL — host requirements. The kernel grants capabilities listed here
# before instantiating the sandbox.
[requirements]
tools = []                  # built-in tools the skill needs
capabilities = [            # capability strings (see capability-model.md)
  'FileRead("/data/**")',
  'NetConnect("api.example.com:443")',
]

# OPTIONAL — config keys the skill expects in ~/.librefang/config.toml.
# The kernel resolves these against operator config and aborts install
# if a required key is missing.
[[config_vars]]
key = "my_skill.api_url"
description = "URL of the upstream API."
default = "https://api.example.com"

# OPTIONAL — markdown body for prompt-only skills. Rendered into the agent's
# system prompt verbatim. Ignored when [runtime].type ≠ "promptonly".
prompt_context = """..."""

# OPTIONAL — provenance. Set by the kernel on install; users rarely write this.
# [source]
# type = "Skillhub"
# slug = "my-org/my-skill"
# version = "0.1.0"
```

## Runtime Types

| `[runtime].type` | Entry file | Notes |
|---|---|---|
| `wasm` | `*.wasm` | Loaded into WasmSkillSandbox; this skill targets this runtime |
| `python` | `*.py` | Subprocess; isolated env vars; optional venv |
| `node` | `*.js` | OpenClaw compatibility |
| `shell` | `*.sh` | Subprocess; same isolation as python |
| `builtin` | (none) | Compiled into the binary; reserved for first-party |
| `promptonly` | (none) | No code; `prompt_context` injected into system prompt |

## Tool Schema Validation

The kernel validates each tool invocation's JSON against `input_schema`
**before** calling the guest's `execute`. Invalid input never reaches your
WASM module — the LLM gets a structured error and retries. Use this:

- Mark required fields explicitly via `required: [...]`.
- Use `type: "string"` for short literals; `type: "object"` for nested.
- Add `enum` for fixed choices (e.g., `enum: ["pdf","html","md"]`).
- Add `pattern` (regex) for IDs or filenames.

```toml
[[tools]]
name = "summarize"
description = "Summarize a document at a URL."
input_schema = { type = "object", properties = {
  url = { type = "string", format = "uri" },
  max_words = { type = "integer", minimum = 50, maximum = 1000 }
}, required = ["url"] }
```

## Manifest Validation Failures

| Error | Cause |
|---|---|
| `Skill not found: <name>` | Manifest exists but referenced by ID before install |
| `Invalid skill manifest: ...` | TOML parse / schema violation |
| `Already installed: <name>` | Re-install without `--force` |
| `Runtime not available: <type>` | E.g., `python` runtime with no Python on PATH |
| `Security blocked: ...` | The Verifier flagged a known-malicious pattern in the body or scripts |

## Example Manifests

### Minimal WASM skill

```toml
[skill]
name = "echo"
version = "0.1.0"
description = "Echo input."

[runtime]
type = "wasm"
entry = "echo.wasm"

[[tools]]
name = "echo"
description = "Echo a JSON payload."
input_schema = { type = "object", properties = { msg = { type = "string" } }, required = ["msg"] }
```

### Networked WASM skill with config

```toml
[skill]
name = "weather-check"
version = "1.0.0"
description = "Look up current weather for a city."
author = "ops-team"
tags = ["weather", "api"]

[runtime]
type = "wasm"
entry = "weather-check.wasm"

[[tools]]
name = "current"
description = "Get current weather for a city."
input_schema = { type = "object", properties = {
  city = { type = "string" },
  units = { type = "string", enum = ["metric", "imperial"], default = "metric" }
}, required = ["city"] }

[requirements]
capabilities = [
  'NetConnect("api.openweathermap.org:443")',
  'EnvRead("OPENWEATHER_API_KEY")',
]

[[config_vars]]
key = "weather.cache_ttl_secs"
description = "How long to cache lookups (seconds)."
default = "300"
```
