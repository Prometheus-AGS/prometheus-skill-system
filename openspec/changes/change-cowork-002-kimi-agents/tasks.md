# Tasks: change-cowork-002-kimi-agents

- [x] Read `cli/src/agents.rs` to confirm current state (includes Zed from change-cowork-001)
- [x] Add `kimi-code` entry to `get_all_agents()` with `~/.kimi-code/skills/` path
- [x] Add `kimi-desktop` entry to `get_all_agents()` with `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/` path (using Path::join for each component)
- [x] Add `kimi-code` detection check to `detect_installed_agents()` checks array
- [x] Add `kimi-desktop` detection check to `detect_installed_agents()` checks array
- [x] Add `kimi-code` to `get_agent_names()` list
- [x] Add `kimi-desktop` to `get_agent_names()` list
- [x] Run `cargo build --release` to confirm compilation succeeds
- [x] Run `cargo test` to verify all tests pass
- [x] Commit changes to cowork-skills fork
