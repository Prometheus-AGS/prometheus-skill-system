# Verification — change-tli-b3b-agent-team-memory-envelope

Repository: `prometheus-skill-system`
Depends on: `change-tli-b3-envelope-write-library`

## Acceptance criteria

- A `memory-publish` run stores a record whose content validates against `learning-envelope.schema.json` with `agent_id` `<team>/<role>` and non-null `user_id`.
- Calling queueMemory without a project id throws.
- Existing `models-memory.integration.mts` still passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/memory-envelope.integration.mjs ../tests/models-memory.integration.mjs
```
