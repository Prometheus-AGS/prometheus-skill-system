# Verification — change-tlh-04-scratch-surreal-for-envelope-test

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- `run-memory-envelope.sh` passes. It refuses (exit 1) if `SURREAL_MEMORY_URL` points at port 23001. It reads every record the test wrote back from the scratch server. After `scratch_surreal_stop` it checks that no process still holds the scratch data dir (`pgrep -f <dir>`), exiting 1 otherwise. All of this is checked inside the script the gate runs. The live service is never contacted.
- `node tests/memory-envelope.integration.mjs` with `SURREAL_MEMORY_URL` unset exits 2.
- `test-kbd-memory-loop.sh` and `test-subagent-delivery.sh --harness none` still pass using the shared library. `scratch_surreal_stop` itself verifies that no process survives on its data dir and fails otherwise, so every caller gets the check.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; }
/bin/bash skills/process/agent-team-creator/tests/run-memory-envelope.sh
if grep -q 23001 skills/process/agent-team-creator/tests/memory-envelope.integration.mjs; then echo 'compiled .mjs still references :23001 (rebuild with npm run build:tests)' >&2; exit 1; fi
( cd skills/process/agent-team-creator/runtime && unset SURREAL_MEMORY_URL; node ../tests/memory-envelope.integration.mjs; test $? -eq 2 )
/bin/bash shared/scripts/tests/test-kbd-memory-loop.sh
/bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none
npm run check:distribution
```
