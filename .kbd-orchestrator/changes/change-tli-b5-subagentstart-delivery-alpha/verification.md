# Verification — change-tli-b5-subagentstart-delivery-alpha

Repository: `prometheus-skill-system`
Depends on: `change-tli-b4-per-agent-recall-and-kbd-loop`, `change-tli-b2-namespaced-subagent-matchers`

## Acceptance criteria

- Alpha gate, fixture team `tlm-fixture` (roles `api-dev` owns `src/api/**`, `ui-dev` owns `src/ui/**`), scratch git repo, scratch HOME/CODEX_HOME/PROMETHEUS_PLUGIN_ROOT, real plugin generation untouched:
  - seed private `ALPHA-API` (api-dev), private `ALPHA-UI` (ui-dev) and 30 KB of unrelated project lessons;
  - Claude Code (`claude -p --plugin-dir <generated claude package>`): api-dev reports ALPHA-API and not ALPHA-UI, ui-dev the reverse;
  - Codex (`codex exec --dangerously-bypass-hook-trust < /dev/null`, generated Codex plugin installed into the scratch CODEX_HOME, trusted project, `[features].hooks=true`): same two assertions, and the child rollout shows the token as a developer message;
  - delivery.jsonl: SubagentStart ≤ 8,000 chars (Claude); Codex ≤ 7,000 chars and ≤ 2,000 tokens estimated at 3.5 chars/token; total per-subagent ≤ 12 KB; 0 leaks;
  - the gate writes `evidence/b5-codex-trust-path.md` recording the path exercised: trusted project + `[features].hooks` + plugin hook trust (bypass flag in the scratch gate; one-time interactive trust prompt documented for normal use), or the PreToolUse fallback contingency if the native path failed.
- With surreal-memory stopped the hook exits 0 with empty stdout.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
command -v claude >/dev/null && command -v codex >/dev/null || { echo 'BLOCKED: claude and codex CLIs required' >&2; exit 2; }
node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist
npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex
/bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness both
test -s "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b5-codex-trust-path.md"
```
