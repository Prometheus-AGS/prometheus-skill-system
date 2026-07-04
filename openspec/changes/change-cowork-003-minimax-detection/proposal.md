---
id: change-cowork-003-minimax-detection
title: Add MiniMax agent detection + document MMX CLI scope
phase: cowork-integration
priority: P0
effort: S
wave: 1
agent: general-purpose
status: done
gap_id: G-02
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/agents.rs (add minimax agent + dual-path detection + mmx doc comment)
  - README.md (clarify MiniMax Desktop coverage, remove MMX CLI promise)
---

# change-cowork-003 — MiniMax agent detection + MMX CLI scope documentation

## Context

MiniMax has two relevant software products:
1. **MiniMax Code IDE / CLI** — installs to `~/.minimax/`; supports a `~/.minimax/skills/` directory
   that prometheus-skill-pack already uses for skill distribution.
2. **MiniMax Agent Desktop** — a native macOS/Windows app that installs to
   `~/Library/Application Support/MiniMax Agent/` on macOS.

Both share the same `~/.minimax/skills/` directory for skill storage, so a single agent entry covers
both products. Detection should confirm MiniMax is installed by checking EITHER path.

The `mmx` binary is a **media-generation CLI** (text/image/video/audio generation via MiniMax API).
It has NO plugin system, no skills directory, and no concept of agent skills. It is explicitly OUT
of scope for cowork.

## Scope

1. Add `minimax` agent entry to `get_all_agents()` in `cli/src/agents.rs`:
   - Key: `"minimax"`, display: `"MiniMax Code"`, skills_dir: `".minimax/skills"`,
     global_skills_dir: `home.join(".minimax/skills")`
2. Add dual-path detection in `detect_installed_agents()`:
   - Primary: `~/.minimax/` exists
   - Fallback: `~/Library/Application Support/MiniMax Agent/` exists
   - (same pattern as Zed's dual-path fallback after the for-loop)
3. Add `"minimax"` to `get_agent_names()`
4. Add doc comment above the checks array explaining mmx media CLI exclusion
5. Update `README.md` to document MiniMax Desktop coverage and clarify MMX CLI is out of scope

## Implementation Notes

### Detection pattern (mirrors Zed's dual-path approach)

The `checks` array handles the primary path (`~/.minimax/`).
After the loop, add a fallback block:

```rust
// MiniMax Desktop installs to ~/Library/Application Support/MiniMax Agent/
// Both MiniMax Code CLI and MiniMax Desktop share ~/.minimax/skills/ for skill storage.
// The mmx media-generation binary has no skill system and is NOT detected here.
if !installed.contains(&"minimax") {
    let minimax_desktop = home
        .join("Library")
        .join("Application Support")
        .join("MiniMax Agent");
    if minimax_desktop.exists() {
        installed.push("minimax");
    }
}
```

## Verification

- `cargo build --release` exits 0 in `cli/`
- `cargo test` passes (10 tests)
- `cowork status` shows minimax when `~/.minimax/` exists
