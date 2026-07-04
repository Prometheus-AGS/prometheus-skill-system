---
id: change-cowork-001-clone-fork-zed-agent
title: Clone cowork fork to dedicated worktree + add Zed agent entry
phase: cowork-integration
priority: P0
effort: S
wave: 1
agent: general-purpose
status: done
gap_id: G-01 G-02
verdict: ADOPT
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (new worktree)
  - cli/src/agents.rs (add Zed agent)
---

# change-cowork-001 — Clone cowork fork + Zed agent entry

## Context

The cowork-skills fork (`git@github.com:GQAdonis/cowork-skills.git`) needs to be cloned to a
dedicated working directory at `/Users/gqadonis/Projects/prometheus/cowork-skills` so it can be
developed independently from the prometheus-skill-pack worktree.

Zed editor is missing from cowork's 16-agent list. prometheus-skill-pack installs to
`~/.config/zed/skills/` (primary) and `~/.zed/skills/` (fallback). Zed requires only a directory
drop of `SKILL.md` — no manifest, no plugin API.

## Scope

1. Clone `git@github.com:GQAdonis/cowork-skills.git` → `/Users/gqadonis/Projects/prometheus/cowork-skills`
2. Verify `cargo build --release` succeeds (establishes the baseline)
3. Add `Zed` agent entry to `cli/src/agents.rs` with dual-path detection

## Implementation Notes

### Agent entry structure (matching existing pattern in agents.rs)

```rust
AgentInfo {
    name: "Zed".to_string(),
    agent_type: AgentType::Zed,
    skills_dir: dirs::home_dir()
        .map(|h| h.join(".config/zed/skills"))
        .unwrap_or_default(),
    fallback_dir: Some(
        dirs::home_dir()
            .map(|h| h.join(".zed/skills"))
            .unwrap_or_default()
    ),
    status: AgentStatus::Community,
    description: "Zed code editor AI skills".to_string(),
    detection: vec![
        dirs::home_dir().map(|h| h.join(".config/zed")).unwrap_or_default(),
        dirs::home_dir().map(|h| h.join(".zed")).unwrap_or_default(),
    ],
    install_method: InstallMethod::Symlink,
    mcp_config: None,
}
```

### Detection logic

Check whether `~/.config/zed/` OR `~/.zed/` exists (either parent dir present = Zed installed).
Install skills to the first path that exists (prefer `~/.config/zed/skills/`).

## Verification

- `cargo build --release` exits 0 in cowork-skills worktree
- `cowork status` shows Zed in the agent list when `~/.config/zed/` exists
- `cargo test` passes
