# Goals

- G-01 Wire 'npm run build:codex' into install-platforms.ts codex target so install regenerates the plugin artifacts (reflection Delta 2)
- G-02 Add 'npm run validate:codex' to CI (.github/workflows) as a drift/validity gate next to validate (Delta 2)
- G-03 Manual hook-trust verification: interactively trust the plugin in a codex session and prove a plugin hook fires (writes to PLUGIN_DATA); record evidence (Delta 1)
- G-04 Env-provisioning helper: seed ~/.codex/config.toml env for the 7 MCP servers' keys from the environment, no committed secrets (Delta 3)
- G-05 External-distribution marketplace sources: support git-subdir/git source types for publishing beyond in-repo dogfood (tech debt)
- G-06 QA gate: author a lightweight .kbd-orchestrator/constraints.md so future phases run the artifact-refiner gate (Delta 4)
