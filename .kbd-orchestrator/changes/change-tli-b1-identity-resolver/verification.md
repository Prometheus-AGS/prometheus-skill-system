# Verification — change-tli-b1-identity-resolver

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- Each of the four precedence levels wins when set, in a real git repo; two worktrees of one repo resolve the same id.
- `agent_type: prometheus-skill-pack:backend-dev` → role `backend-dev`; Codex `backend_dev` → `backend-dev`; an unknown name with touched paths under a role's `owns` glob → that role; no team → `@solo`.
- `grep -rn 'PROMETHEUS_PROJECT_ID:-prometheus-skill-pack' shared` finds nothing.
- `/bin/bash shared/scripts/tests/test-identity.sh` passes (bash 3.2, C-05).

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
/bin/bash shared/scripts/tests/test-identity.sh
if grep -rn 'PROMETHEUS_PROJECT_ID:-prometheus-skill-pack' shared; then echo 'hardcoded project id default remains' >&2; exit 1; fi
node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist
npm run check:distribution
```
