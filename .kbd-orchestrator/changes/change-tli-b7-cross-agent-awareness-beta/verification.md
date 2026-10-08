# Verification — change-tli-b7-cross-agent-awareness-beta

Repository: `prometheus-skill-system`
Depends on: `change-tli-b5-subagentstart-delivery-alpha`, `change-tli-b6-file-tier-reduction`

## Acceptance criteria

- Cycle 1: api-dev's SubagentStop writes `BETA-PRIV` (paths src/api/handler.ts) and `BETA-ROUTED` (src/ui/client.ts). Cycle 2, both harnesses: api-dev receives BETA-PRIV; ui-dev receives BETA-ROUTED and only the digest line for BETA-PRIV; the main thread receives digest lines and no private text.
- Every stored record validates against the envelope schema with the design-table agent_id.
- `report-learning-delivery.py --require-reduction` prints bytes per agent per channel and exits non-zero unless every agent's total is below the 14 KB (Claude) / 10.8 KB (Codex) baseline.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
command -v claude >/dev/null && command -v codex >/dev/null || { echo 'BLOCKED: claude and codex CLIs required' >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-team-awareness.sh --harness both
python3 scripts/report-learning-delivery.py --baseline-claude 14336 --baseline-codex 11059 --require-reduction
npm run check:distribution
```
