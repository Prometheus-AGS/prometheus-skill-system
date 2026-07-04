# Tasks: change-cowork-003-minimax-detection

- [x] Add `minimax` entry to `get_all_agents()` HashMap in `cli/src/agents.rs`
- [x] Add `("minimax", home.join(".minimax"))` to the `checks` array in `detect_installed_agents()`
- [x] Add MiniMax Desktop fallback block after the Zed fallback in `detect_installed_agents()`
- [x] Add `"minimax"` to the `get_agent_names()` vec
- [x] Add doc comment above the `checks` array explaining mmx media CLI exclusion
- [x] Update `README.md`: document MiniMax Desktop coverage, clarify MMX CLI is out of scope
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit changes to cowork-skills fork
- [x] Mark proposal status: done
