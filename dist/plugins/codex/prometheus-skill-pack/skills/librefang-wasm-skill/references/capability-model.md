# LibreFang Capability Model

Source: `librefang/crates/librefang-types/src/capability.rs`.

Capabilities are deny-by-default. Every host call (except `time_now`) goes
through `check_capability` against the list declared in your skill.toml's
`[requirements].capabilities`. If no granted capability matches, the host
returns `{"error": "Capability denied: ..."}`.

## Variants

| Capability | String form (in skill.toml) | Argument | Matches |
|---|---|---|---|
| `FileRead` | `FileRead("<glob>")` | glob pattern | Files whose canonical path matches the glob |
| `FileWrite` | `FileWrite("<glob>")` | glob pattern | Files whose canonical path matches the glob |
| `NetConnect` | `NetConnect("<host:port>")` | `host:port` or `*:port` | Outbound TCP to matching peer |
| `NetListen` | `NetListen(<port>)` | u16 port | Bind a server on the given port |
| `ToolInvoke` | `ToolInvoke("<tool-id>")` | tool id | Invoke a specific built-in tool |
| `ToolAll` | `ToolAll` | (none) | Invoke any tool — DANGEROUS, use only when absolutely required |
| `ShellExec` | `ShellExec("<cmd>")` | cmd path or `*` | Spawn the named subprocess |
| `EnvRead` | `EnvRead("<var>")` | env var name | Read the var (`*` for any) |
| `MemoryRead` | `MemoryRead("<key>")` | key glob | Read kernel KV |
| `MemoryWrite` | `MemoryWrite("<key>")` | key glob | Write kernel KV |
| `AgentMessage` | `AgentMessage("<id>")` | agent id glob | Send to matching agents |
| `AgentSpawn` | `AgentSpawn` | (none) | Spawn child agents |
| `LlmQuery` | `LlmQuery("<model>")` | model glob | Query matching models |

## Glob Semantics

`librefang_types::capability::glob_matches` handles `*` (single-segment)
and `**` (recursive). Examples:

| Glob | Matches | Doesn't match |
|---|---|---|
| `/data/*.json` | `/data/users.json`, `/data/config.json` | `/data/sub/x.json`, `/etc/x.json` |
| `/data/**` | Anything under `/data/` | `/etc/x.json` |
| `api.*.com:443` | `api.example.com:443`, `api.test.com:443` | `api.com:443`, `api.example.org:443` |
| `*:443` | Any host on port 443 | Any host on port 80 |

## Composition Patterns

```toml
# Read-only skill against a fixed path tree
capabilities = ['FileRead("/data/inputs/**")']

# Bidirectional skill with scoped writes
capabilities = [
  'FileRead("/data/inputs/**")',
  'FileWrite("/data/outputs/**")',
]

# Web-aware skill with one allowed origin
capabilities = [
  'NetConnect("api.openai.com:443")',
  'EnvRead("OPENAI_API_KEY")',
]

# Inter-agent worker
capabilities = [
  'AgentMessage("research-*")',
  'MemoryRead("research.cache.*")',
  'MemoryWrite("research.results.*")',
]
```

## What NOT to grant

- **`ToolAll`** — equivalent to `sudo`. Almost no skill genuinely needs
  this; auditors should treat its presence as a security review trigger.
- **`FileWrite("/**")` or `FileWrite("**")`** — gives the skill the host's
  full write surface. Scope to a directory the skill actually owns.
- **`NetConnect("*:*")`** — allows arbitrary outbound. Combine with the
  host's SSRF protection only if you've reviewed the skill code for
  user-controlled URL construction.
- **`ShellExec("*")`** — arbitrary subprocess execution. The Capability
  model is a defense-in-depth layer; don't rely on it alone for shell-out
  skills, also review the argument-construction sites.

## Source Verification

```rust
// From librefang-types/src/capability.rs
pub fn capability_matches(granted: &Capability, required: &Capability) -> bool {
    match (granted, required) {
        (Capability::FileRead(g), Capability::FileRead(r)) => glob_matches(g, r),
        (Capability::NetConnect(g), Capability::NetConnect(r)) => glob_matches(g, r),
        // ... per variant ...
        (Capability::ToolAll, _) => false, // ToolAll only matches ToolInvoke
        (Capability::ToolAll, Capability::ToolInvoke(_)) => true,
        _ => false,
    }
}
```

`ToolAll` only matches `ToolInvoke`, never (e.g.) `FileRead` — the model
prevents accidental broad grants from leaking across resource types.
