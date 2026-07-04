---
id: change-cowork-006-opencode-plugin-registration
title: OpenCode JSON plugin registration
phase: cowork-integration
priority: P0
effort: S
wave: 2
agent: general-purpose
status: done
gap_id: G-04
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/opencode_config.rs (NEW — plugin[] JSON merge + package.json ensure)
  - cli/src/commands/mod.rs (add opencode_config module)
  - cli/src/commands/install.rs (call configure_opencode after opencode agent install)
---

# change-cowork-006 — OpenCode JSON plugin registration

## Context

When `cowork install --agent opencode` completes, cowork should register the
installed skill-pack's `.opencode/` directory in `~/.opencode/opencode.json`'s
`plugin[]` array, and ensure the pack's `.opencode/package.json` declares the
required OpenCode dependencies.

## Scope

1. Create `cli/src/commands/opencode_config.rs` with:
   - `register_opencode_plugin(plugin_path: &Path) -> Result<bool>` — idempotent JSON array append
   - `ensure_opencode_package_json(pack_opencode_dir: &Path) -> Result<()>` — write/merge package.json with required deps
   - `configure_opencode(pack_root: Option<&Path>) -> Result<()>` — top-level orchestrator
2. Register module in `cli/src/commands/mod.rs`
3. Call `configure_opencode` in `install.rs` after completing opencode agent install

## Required package.json dependencies

```json
{
  "@opencode-ai/plugin": "^1.15.0",
  "@opencode-ai/sdk": "^1.15.0",
  "zod": "^3.23.0"
}
```

## opencode.json plugin[] format

`~/.opencode/opencode.json` has a top-level `"plugin": [...]` array of path strings.
Paths can be absolute or relative. We register the absolute path of the pack's
`.opencode/` directory.

## Verification

- `cargo build --release` exits 0
- `cargo test` — all existing + new tests pass
