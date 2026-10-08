# Verification — change-tli-c1b-promotion-routing

Repository: `prometheus-skill-system`
Depends on: `change-tli-e1-pin-v1-11-0`, `change-tli-b7-cross-agent-awareness-beta`

## Acceptance criteria

- A `[GLOBAL]` reflect line is stored under `@global` and pk shared; a subagent in a third scratch project receives it within its global quota.
- kbd-open shows a pending candidate and prints nothing when there are none.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[1-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.11.0 required (E1 not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-promotion-routing.sh
npm run check:distribution
```
