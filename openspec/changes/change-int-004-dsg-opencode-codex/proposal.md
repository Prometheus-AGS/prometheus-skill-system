---
id: change-int-004-dsg-opencode-codex
title: dsg OpenCode + Codex plugin artifacts
phase: cowork-integration
priority: P1
effort: S
wave: 5
agent: general-purpose
status: done
gap_id: G-04-dsg
verdict: BUILD
scope:
  - tools/disk-space-guardian (submodule working tree)
  - tools/disk-space-guardian/.opencode/package.json (new)
  - tools/disk-space-guardian/.codex/config.toml (new)
---

# change-int-004 — dsg OpenCode + Codex plugin artifacts

## Context

The dsg submodule's `.opencode/` and `.codex/` directories already contain
OpenSpec skill files but are missing the plugin registration artifacts that
make dsg discoverable as an OpenCode npm plugin and a Codex TOML config.

## Strategy

1. Create `.opencode/package.json` with the @opencode-ai/plugin dependency
   declarations so OpenCode recognizes dsg as a registered plugin.
2. Create `.codex/config.toml` with a CODEX_PLUGIN=true marker and MCP stub
   section (to be populated when dsg Phase 3 MCP server lands).

Both files are committed to the dsg submodule working tree. The skill-pack
parent repo then updates its submodule pointer in the same commit.

## Scope

1. Create tools/disk-space-guardian/.opencode/package.json
2. Create tools/disk-space-guardian/.codex/config.toml
3. Commit in dsg submodule; update submodule pointer in skill-pack
4. Update KBD orchestrator
