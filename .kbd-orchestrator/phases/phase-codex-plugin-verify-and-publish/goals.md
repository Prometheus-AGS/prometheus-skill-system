# Goals

- G-01 Exercise validate:codex in a real GitHub Actions run — confirm the CI drift/validity gate actually runs and passes on a push/PR (reflection Delta 3)
- G-02 Confirm the MCP env round-trip: run codex-provision-mcp-env.sh with keys set, install the plugin, and verify codex doctor stops warning / a plugin MCP server sees its key (Delta 2)
- G-03 Verify the REAL plugin hooks run cleanly under Codex with the CLAUDE_PLUGIN_ROOT:-PLUGIN_ROOT fix (not just the probe) — SessionStart executes without empty-path errors
- G-04 Test git-subdir source resolution against a real remote: codex plugin marketplace add <git-url> resolves the published git-subdir sources; first external publish (Delta 3)
