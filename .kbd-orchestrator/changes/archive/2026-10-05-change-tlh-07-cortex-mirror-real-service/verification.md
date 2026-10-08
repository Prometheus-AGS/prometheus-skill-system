# Verification — change-tlh-07-cortex-mirror-real-service

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- The real case passes against Cortex 2.0.3: a lesson written through `learning_write.py` is returned by real `cortex_recall` for its projectId with the role text present.
- `~/.cortex` is unchanged by the test (mtime and size of `memory.db` before == after) and nothing is written under `~/.claude/plugins/cache`.
- With the model cache absent (simulated by pointing discovery at a copy without it), the case exits 2, not 0.
- The stub cases still pass.
- The `memory.db` and plugin-cache checks and the absent-model case are assertions inside `test-cortex-mirror.sh`, which the gate runs, so the gate proves them.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
ls -d ~/.claude/plugins/cache/cortex/cortex/*/dist/mcp-server.js >/dev/null 2>&1 || { echo 'BLOCKED: Cortex not installed' >&2; exit 2; }
/bin/bash shared/scripts/tests/test-cortex-mirror.sh
npm run check:distribution
```
