# Verification — change-tli-d3-claude-md-memory-chain-cortex

Repository: `prometheus-skill-system`
Depends on: `change-tli-b6-file-tier-reduction`, `change-tli-b7-cross-agent-awareness-beta`

## Acceptance criteria

- With Cortex available, one attributed write is recalled by project and role tag; with Cortex absent the write exits 0 with no warning.
- CLAUDE.md no longer instructs subagents to read the full memory index.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
/bin/bash shared/scripts/tests/test-cortex-mirror.sh
if grep -n 'Read ~/.claude/projects/.*/memory/MEMORY.md' CLAUDE.md AGENTS.md; then echo 'full-index instruction remains' >&2; exit 1; fi
npm run check:distribution
```
