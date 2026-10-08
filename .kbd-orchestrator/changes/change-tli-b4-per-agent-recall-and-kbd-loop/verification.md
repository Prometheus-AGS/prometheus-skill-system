# Verification — change-tli-b4-per-agent-recall-and-kbd-loop

Repository: `prometheus-skill-system`
Depends on: `change-tli-b3-envelope-write-library`

## Acceptance criteria

- A reflection containing a unique token, after the worker's `run-once` and `kbd-next-phase`, yields a `prior-context.md` for assess that cites the token from surreal-memory and from pk.
- The reflect:after hook resolves `memory-writeback.sh` in both the source tree and the installed flat layout (test runs both).
- `prior-context.md` stays under its byte budget and `delivery.jsonl` gains one line with per-channel bytes.
- Recall for role R never returns a memory keyed to another role's private `agent_id`.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-kbd-memory-loop.sh
npm run check:distribution
```
