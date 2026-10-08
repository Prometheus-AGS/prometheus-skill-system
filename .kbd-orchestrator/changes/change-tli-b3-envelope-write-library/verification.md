# Verification — change-tli-b3-envelope-write-library

Repository: `prometheus-skill-system`
Depends on: `change-tli-b1-identity-resolver`, `change-tli-b2-namespaced-subagent-matchers`, `change-tli-a5a-skillpack-pin-v1-10-0`

## Acceptance criteria

- One write per visibility level is stored under the design-table `agent_id` and every stored content envelope validates against the schema.
- A repeated write with the same contentHash stores once.
- With surreal-memory unreachable the op lands in the outbox and the script exits 0 with empty stdout.
- A SubagentStop payload for a resolved role with `LESSON: x` in `last_assistant_message` produces a memory under `<team>/<role>` and a queued learning job carrying team/role.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-learning-write.sh
node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist
npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex
```
