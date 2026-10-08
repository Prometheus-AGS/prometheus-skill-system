# Verification — change-tli-b2-namespaced-subagent-matchers

Repository: `prometheus-skill-system`
Depends on: `change-tli-b1-identity-resolver`

## Acceptance criteria

- In generated `hooks/hooks.json`, no SubagentStop matcher matches any bare role id from `.agent-team/*/team.json` or `planner`/`executor`.
- Per-phase args of all five groups are byte-identical before and after.
- Codex: payload `agent_type: planner` with an active team containing role `planner` runs no iterative-evolver target (guard exits 0, no side effect); with no team it runs `state-checkpoint.sh … plan`.
- Claude: `iterative-evolver:planner` still runs the planner hooks; a team role `planner` does not.
- The iterative-evolver executor still runs `karpathy-hook-dispatch.sh executor_complete`.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
test -s .kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/b2-claude-iterative-evolver-payload.json || test -s "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b2-claude-iterative-evolver-payload.json"
node --test scripts/tests/subagent-matchers.test.mjs
node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist
npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex
```
