# change-sync-008: rmcp MCP server (stdio mode)

**Phase:** phase-learn-sovereign-sync
**Tier:** 2 (after Tier 1)
**Status:** pending
**Library:** cand-004 (rmcp 1.8.0)
**Gap:** G-03

## Summary

Implement the MCP server using rmcp 1.8.0. Exposes four sync tools plus
SkillIndex keyword search. UAR passthrough mode via env var. Tool prefix
`sovereign:` when `--prefix-tools` flag set.

## Files to change

- `substrate/sovereign-sync/src/mcp_server.rs` — new file
- `substrate/sovereign-sync/Cargo.toml` — add rmcp = "1.8"

## SkillIndex (keyword-only, Phase A)

```rust
pub struct SkillIndex {
    skills: Vec<SkillEntry>,
}
impl SkillIndex {
    pub fn load_from_dir(skills_dir: &Path) -> Result<Self> { ... }
    pub fn search(&self, query: &str) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| {
            s.name.to_lowercase().contains(&query.to_lowercase())
            || s.description.to_lowercase().contains(&query.to_lowercase())
            || s.keywords.iter().any(|k| query.to_lowercase().contains(k))
        }).collect()
    }
}
```

## UAR detection

On startup, check `UAR_SKILL_SERVICE_URL` env var. If set, only expose
`sync_push`, `sync_pull`, `sync_status`, `sync_peers` tools (passthrough mode).
If not set, also expose `skill_search` and `skill_list`.

## Tasks

- [ ] Implement 4 sync tools with rmcp #[tool] macro
- [ ] Implement SkillIndex loader (parse YAML frontmatter from SKILL.md)
- [ ] Implement UAR passthrough detection
- [ ] Implement --prefix-tools flag (prepend "sovereign:" to all tool names)
- [ ] Wire into main.rs --mode mcp branch
- [ ] Integration test: rmcp stdio client connects, calls sync_status
