# Verification — change-tli-c4-cross-repo-team-requests

Repository: `prometheus-skill-system`
Depends on: `change-tli-b7-cross-agent-awareness-beta`, `change-tli-b3b-agent-team-memory-envelope`, `change-tli-b6-file-tier-reduction`

## Acceptance criteria

- Three teams across two repos: discovery ranks the right team first; a same-repo request creates an intake task; a cross-repo request creates one labelled issue that team-intake imports exactly once; the issue is closed at the end.
- The intake role's next SubagentStart context includes the request digest line.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
test -n "${PROMETHEUS_TEAM_TEST_REPO:-}" || { echo "BLOCKED: PROMETHEUS_TEAM_TEST_REPO unset (sandbox repo for issue creation)" >&2; exit 2; }
command -v gh >/dev/null || { echo 'BLOCKED: gh required' >&2; exit 2; }
cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/teams.integration.mjs
```
