# Verification — change-tli-c3b-skill-candidate-surfacing

Repository: `prometheus-skill-system`
Depends on: `change-tli-e1-pin-v1-11-0`, `change-tli-c1b-promotion-routing`

## Acceptance criteria

- With one new-skill and one update candidate seeded, kbd-open lists both; with none it prints nothing.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[1-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.11.0 required (E1 not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-skill-candidates.sh
npm run check:distribution
```
