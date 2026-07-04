---
id: change-cowork-007-cowork-pack-subcommand
title: cowork pack subcommand — prometheus skill-pack management
phase: cowork-integration
priority: P0
effort: M
wave: 3
agent: general-purpose
status: done
gap_id: G-03
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/pack.rs (NEW)
  - cli/src/commands/mod.rs (add pack module)
  - cli/src/main.rs (add Pack variant + dispatch)
---

# change-cowork-007 — cowork pack subcommand

## Context

The `cowork pack` subcommand gives agents and users a CLI surface to
inspect the state of the prometheus-skill-pack, update it, and repair
broken installations. It satisfies G-03 (prometheus-pack awareness).

## Scope

1. Create `cli/src/commands/pack.rs` with:
   - `PackSubcommand` enum: `Status`, `Update`, `Repair`
   - `execute_status()` — reads `package.json` from pack root; counts installed
     skills per platform (claude-code, kimi, minimax, opencode, codex, cursor);
     prints summary table
   - `execute_update()` — shells to `bash <pack_root>/scripts/install-skills-flat.sh`
   - `execute_repair()` — detects broken symlinks; runs install for affected platforms
   - Pack location: `PROMETHEUS_SKILL_PACK` env var → `~/.cowork/prometheus-skill-pack/` → `~/Projects/prometheus/prometheus-skill-pack` (convenience fallback for local dev)
2. Wire `Pack` variant into `Commands` enum in `main.rs`
3. Register `pub mod pack` in `commands/mod.rs`

## Sub-command surface

```
cowork pack status            # Show prometheus-skill-pack version + skill counts
cowork pack update            # Run install-skills-flat.sh to update all platforms
cowork pack repair            # Detect broken symlinks; reinstall affected platforms
```

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass (unit tests for pack_root resolution + skill counting)
