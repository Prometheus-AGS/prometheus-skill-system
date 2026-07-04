---
id: change-cowork-002-kimi-agents
title: Add Kimi Code CLI and Kimi Desktop agent entries
phase: cowork-integration
priority: P1
effort: S
wave: 1
agent: general-purpose
status: done
gap_id: G-03
verdict: ADOPT
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills/cli/src/agents.rs
---

# change-cowork-002 — Add Kimi Code CLI + Kimi Desktop agents

## Context

Two Kimi platform agents are missing from cowork's agent registry:

- **Kimi Code CLI** — installs skills to `~/.kimi-code/skills/`; detected by presence of `~/.kimi-code/`
- **Kimi Desktop** — installs skills to `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/` (macOS); detected by presence of `~/Library/Application Support/kimi-desktop/`

prometheus-skill-pack already installs to both locations via `scripts/install-skills-flat.sh`. Adding these agents to cowork ensures `cowork install` and `cowork status` handle them correctly.

## Scope

1. Add `kimi-code` entry to `get_all_agents()` in `cli/src/agents.rs`
2. Add `kimi-desktop` entry to `get_all_agents()` in `cli/src/agents.rs`
3. Add both to the checks array in `detect_installed_agents()`
4. Add both to `get_agent_names()` list

## Implementation Notes

### kimi-code

```rust
agents.insert(
    "kimi-code",
    AgentConfig {
        name: "kimi-code",
        display_name: "Kimi Code",
        skills_dir: ".kimi-code/skills",
        global_skills_dir: home.join(".kimi-code/skills"),
    },
);
```

Detection check: `("kimi-code", home.join(".kimi-code"))`

### kimi-desktop

```rust
agents.insert(
    "kimi-desktop",
    AgentConfig {
        name: "kimi-desktop",
        display_name: "Kimi Desktop",
        skills_dir: "Library/Application Support/kimi-desktop/daimon-share/daimon/skills",
        global_skills_dir: home
            .join("Library")
            .join("Application Support")
            .join("kimi-desktop")
            .join("daimon-share")
            .join("daimon")
            .join("skills"),
    },
);
```

Detection check: `("kimi-desktop", home.join("Library").join("Application Support").join("kimi-desktop"))`

Note: `Path::join` is used for each component of the Kimi Desktop path to avoid issues with spaces in path names when used via string interpolation.

## Verification

- `cargo build --release` exits 0
- `cargo test` passes
- `cowork status` shows `kimi-code` when `~/.kimi-code/` exists
- `cowork status` shows `kimi-desktop` when `~/Library/Application Support/kimi-desktop/` exists (macOS)
