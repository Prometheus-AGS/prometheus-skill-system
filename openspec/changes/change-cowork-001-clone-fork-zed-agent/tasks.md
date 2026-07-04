# Tasks: change-cowork-001-clone-fork-zed-agent

- [x] Clone `git@github.com:GQAdonis/cowork-skills.git` to `/Users/gqadonis/Projects/prometheus/cowork-skills`
- [x] Run `cargo build --release` in the cloned repo to verify baseline build succeeds
- [x] Inspect `cli/src/agents.rs` to understand the existing agent struct pattern and AgentType enum
- [x] Add `Zed` variant to `AgentType` enum
- [x] Add Zed agent entry to the agent list with dual-path detection (`~/.config/zed/` primary, `~/.zed/` fallback)
- [x] Run `cargo build --release` again to confirm the new entry compiles cleanly
- [x] Run `cargo test` to verify all tests pass
