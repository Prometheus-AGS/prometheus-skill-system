# Goals

- Land upstream prerequisites A1-A4 (pk scores every scope entry, worker project/team/role + snapshot commit, pk tag filter + settable type, surreal-memory lean search + filterable categories + agent_id re-key) and the A5 v1.10.0 pin bump in both skills repos
- Identity, envelope writes and per-agent recall (B1-B4): one project-id resolver, namespaced SubagentStop matchers, learning_write/learning_recall libraries, KBD memory loop reading lessons back into the next cycle
- SubagentStart delivery passes the alpha gate in Claude Code and Codex: each fixture role receives only its own lessons, 0 leaks, within 8,000 chars / 2,000 tokens and 12 KB total, with the Codex trust path solved or the fallback path recorded
- File-tier reduction and cross-agent awareness pass the beta gate: MEMORY.md <= 4 KB, path-routed lessons and team digest delivered across cycles, bytes per agent measured against the 14 KB / 10.8 KB baseline
- Promotion, Feynman gaps, skill discovery, cross-repo team requests, mini port, mini timeouts and CLAUDE.md memory chain (C1-C4, D1-D3) each land with their integration gate
