# kbd-doctor check catalog

Current doctor registry groups:

- `learning`
- `skills`
- `binaries`
- `services`
- `mcp`
- `hooks`
- `state`

Current concrete checks:

- `learning.surreal-memory`
- `learning.trace-store`
- `skills.directory`
- `skills.installed-agents`
- `binaries.manifest`
- `services.launch-agents`
- `mcp.config`
- `hooks.lifecycle`
- `state.kbd-orchestrator`
- `state.evolver`

The CLI is the source of truth for this catalog. Update this file when new
check IDs are added or when safe repair actions become available.
